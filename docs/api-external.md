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

Une clé d'API a toujours la forme **`kesh_pat_`** suivie de **27 caractères** alphanumériques (`0-9`, `A-Z`, `a-z`), soit **36 caractères au total**. Exemple (factice) :

```
kesh_pat_3kQ9mZ1aB7cD4eF6gH8iJ0kL2mN
```

> ⚠️ **Une clé `kesh_mcp_…` n'est pas une clé d'API** : présentée à `/api/v1`, elle est refusée en **`401`**.

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

> ⚠️ **La suppression d'un import bancaire est fermée aux clés, elle aussi** *(depuis la v0.13.2)*, et ce n'est pas une route d'administration : `DELETE /bank-imports/{id}` est ouverte au Comptable, mais **refuse toute clé**. Une clé `read-write` reçoit `403 API_KEY_OPERATION_FORBIDDEN` — rien n'est supprimé, rien n'est tracé ; une clé `read` est arrêtée avant, en `403 API_KEY_READ_ONLY`. La suppression se fait dans l'interface web, par un Comptable ou un Administrateur.

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

> ℹ️ **Depuis la 0.16.0, Kesh a son propre serveur MCP** (*Model Context Protocol*), sur `/mcp`, avec ses propres clés `kesh_mcp_…` : désactivé par défaut, portées fines, données montrées à un Administrateur avant de partir sous une clé à validation requise, chaque appel journalisé. C'est la voie à préférer pour brancher un agent IA ; elle est décrite au manuel d'administration, § *Le serveur MCP*. Une clé d'API (`kesh_pat_…`) y est refusée (`401`), comme une clé `kesh_mcp_…` l'est ici (§ 2).

L'API de Kesh reste consommable par **tout client HTTP**, donc aussi par un serveur MCP générique capable d'appeler des API REST avec un en-tête d'authentification. Cette voie reste possible, mais elle passe **sans** la validation avant envoi ni le journal des appels du serveur MCP : l'agent lit tout ce que la clé d'API ouvre, sans que personne ne voie ce qui part.

Exemple de configuration d'un serveur MCP HTTP générique pointant vers l'API de Kesh — **cette voie-là seulement**, non le serveur MCP de Kesh (le format exact dépend du serveur MCP utilisé) :

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

Les principales ressources accessibles via l'API (liste non exhaustive — toute route `/api/v1/*` de l'UI est consommable, **à trois exceptions près, toutes trois fermées aux clés API** : les routes d'administration (`403 API_KEY_ADMIN_FORBIDDEN`), **la consultation du journal d'audit** (`/api/v1/audit-log*`, `403 API_KEY_MANAGEMENT_FORBIDDEN`) et **la suppression d'un import bancaire** (`DELETE /bank-imports/{id}`, `403 API_KEY_OPERATION_FORBIDDEN`, depuis la v0.13.2) — cf. §4 pour la fermeture, §10 pour les codes d'erreur) :

| Ressource | Lecture (`read`) | Écriture (`read-write`) |
|-----------|------------------|--------------------------|
| Identité de la clé | `GET /auth/me` | — |
| Plan comptable | `GET /accounts` | `POST /accounts`, `PUT /accounts/{id}` ³, … |
| Contacts | `GET /contacts`, `GET /contacts/{id}` | `POST /contacts`, … |
| Produits | `GET /products`, `GET /products/{id}` | `POST /products`, … |
| Factures | `GET /invoices`, `GET /invoices/{id}` | `POST /invoices`, `PUT /invoices/{id}`, … ² |
| Écritures comptables | `GET /journal-entries`, `GET /journal-entries/{id}` | `POST /journal-entries`, `PUT /journal-entries/{id}` ⁴, `DELETE /journal-entries/{id}` ⁴, … |
| Lettrages | `GET /letterings/{key}` ⁵ | `POST /letterings` ⁵, `DELETE /letterings/{key}` ⁵ |
| Postes ouverts et rapprochements proposés | `GET /accounts/{id}/open-items` ⁶, `GET /accounts/{id}/lettering-proposals` ⁶ | — (accepter une proposition : `POST /letterings`) |
| Taux de TVA | `GET /vat-rates` | — ¹ |
| Comptes bancaires | `GET /bank-accounts` ⁹ | `POST /bank-accounts`, `PUT /bank-accounts/{id}` ⁷, `PATCH /bank-accounts/{id}` (lien au grand livre), `DELETE /bank-accounts/{id}` (archivage), `POST /bank-accounts/{id}/close`, `POST /bank-accounts/{id}/reopen` ⁹ |
| Imports bancaires | `GET /bank-imports`, `GET /bank-imports/{id}` | `POST /bank-imports/preview`, `POST /bank-imports` ⁸ |
| Exercices comptables | `GET /fiscal-years`, `GET /fiscal-years/{id}`, `GET /fiscal-years/{id}/closing-checks` ¹⁰ | `POST /fiscal-years`, `PUT /fiscal-years/{id}`, `POST /fiscal-years/{id}/close` (la réouverture est fermée aux clés) |

*(Préfixe `…/api/v1` omis dans le tableau. Les corps de requête d'écriture peuvent différer des champs renvoyés en lecture : référez-vous aux formulaires correspondants de l'interface web pour les champs attendus.)*

⁴ Depuis la v0.13.0 — voir « Modifier une écriture » et « Supprimer une écriture » ci-dessous.

⁵ Depuis la v0.13.0 — voir « Lettrer des lignes » ci-dessous.

⁶ Depuis la v0.13.0 — voir « Les postes ouverts d'un compte » ci-dessous.

⁷ *Depuis la v0.13.1* : les comptes bancaires portent un champ `description` (optionnel, 60 caractères au plus, `null` si absent) ; `PUT /bank-accounts/{id}` le remplace comme les autres champs — **l'omettre l'efface** ; `PATCH` (lien au grand livre) ne le touche pas.

⁸ *Depuis la v0.13.2* : la **suppression** d'un import dont aucune transaction n'est rapprochée existe à l'interface web et n'est **pas** ouverte aux clés (§4). Un import supprimé disparaît des lectures, avec ses transactions ; et une fois **tous** les imports d'un compte bancaire supprimés — s'il ne lui reste aucune transaction —, ce compte peut s'archiver par `DELETE /bank-accounts/{id}`, sous ses autres conditions (`412 BANK_ACCOUNT_HAS_TRANSACTIONS` tant qu'une transaction subsiste). *Depuis la v0.13.3*, un compte qui a servi se retire plutôt de l'usage par sa **clôture** (note ⁹), sans rien supprimer ; la voie de l'archivage reste.

⁹ *Depuis la v0.13.3* — **clôture d'un compte bancaire** (lecture seule, historique conservé ; [#642](https://github.com/guycorbaz/kesh/issues/642)) :

- **Lecture.** Chaque compte porte `closedOn` — la date de clôture `AAAA-MM-JJ`, `null` pour un compte ouvert. `GET /bank-accounts` **n'en rend plus les comptes clos** par défaut ; `includeClosed=true` les ajoute (`includeArchived=true`, inchangé, ajoute les archivés — les deux se combinent). `GET /companies/current` ne rend que les comptes ni archivés ni clos, sans paramètre. Les lectures d'un compte clos restent ouvertes : `GET /bank-imports`, `GET /bank-imports/{id}`, `GET /reconciliation/proposals?bankAccountId=…` (liste vide), l'export global (`bank_accounts.csv`, colonne `closed_on`) et la sauvegarde.
- **`POST /bank-accounts/{id}/close`** — écriture (`read-write`), rôle Comptable, ouverte aux clés. Corps `{ "version", "closedOn"? }` ; `closedOn` absent → date du jour **UTC** du serveur ; une année hors de `1000..=9999` est refusée en `400`. La date est informative : aucune autre borne. Réponse : le compte (`closedOn` posé, `version + 1`). Refus, dans l'ordre : `400` (forme), `404 BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` (inexistant, autre société, **archivé**), `409 OPTIMISTIC_LOCK_CONFLICT`, `409 BANK_ACCOUNT_CLOSED` (déjà clos), `412 BANK_ACCOUNT_CANNOT_CLOSE_PRIMARY`, `412 BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS` (`details.pendingCount` — une transaction **rejetée** compte), `412 BANK_ACCOUNT_HAS_OPEN_PAYMENT_BATCHES` (`details.batchCount`, lots générés). Trace `bank_account.closed`.
- **`POST /bank-accounts/{id}/reopen`** — mêmes droits. Corps `{ "version" }`. Le compte ne redevient pas principal. Refus : `400`, `404` (comme ci-dessus), `409 OPTIMISTIC_LOCK_CONFLICT`, `409 BANK_ACCOUNT_NOT_CLOSED`. Trace `bank_account.reopened`, dont `details.before.closed_on` garde la date levée.
- **Écritures refusées.** Toute écriture qui vise un compte clos est refusée en **`409 BANK_ACCOUNT_CLOSED`** (`details.bankAccountId`), sans rien écrire : aperçu et création d'un import, rapprochement manuel et ventilé, annulation d'un rapprochement, annulation d'un rejet, rattachement d'une transaction à un règlement et liste de ses candidats *(depuis la v0.13.4)*, rattachement d'une transaction à une écriture et liste de ses écritures candidates *(depuis la v0.17.0)*, règlement d'une facture client ou fournisseur par virement et son annulation, annulation d'une facture fournisseur payée par ce compte, création et confirmation d'un lot de paiement, `PUT`, `PATCH` et `DELETE /bank-accounts/{id}`. Pour les lots `POST /reconciliation/accept` et `/reject`, le refus est **par proposition**, dans `failed[]` en `200`, code `BANK_ACCOUNT_CLOSED`. **Exception** : l'**annulation** d'un lot de paiement généré (`POST /payment-batches/{id}/cancel`) reste permise.
- *Depuis la v0.13.4* — **une écriture déjà en cours au moment de la clôture** ([#651](https://github.com/guycorbaz/kesh/issues/651)) ne passe pas sur le compte clos : ou bien elle est refusée en `409 BANK_ACCOUNT_CLOSED`, sans rien écrire, ou bien elle se termine avant, et la clôture la voit (elle peut alors être refusée elle-même, par exemple en `412 BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS` si l'écriture a remis une transaction en attente). Pour une intégration, un import peut être refusé **après** l'envoi du fichier (analyse comprise) : rouvrir le compte, puis renvoyer le fichier. La clôture peut **attendre** quelques instants qu'une écriture en cours sur le compte se termine. Réciproquement, l'annulation d'un rapprochement (`POST /reconciliation/transactions/{id}/cancel`) peut attendre une clôture en cours en gardant le verrou de réconciliation du compte : pendant ce temps, les autres gestes de réconciliation sur le même compte (acceptation, rejet, rapprochement manuel ou ventilé, annulation d'un rejet, annulation d'un autre rapprochement, rattachement d'une transaction à un règlement *(depuis la v0.13.4)*) peuvent être refusés en `409 RECONCILIATION_ACCOUNT_LOCKED` au bout de 5 secondes — les renvoyer. *(Jusqu'à la v0.13.3, une telle écriture pouvait encore aboutir sur le compte clos.)*

¹⁰ *Depuis la v0.13.4* — **contrôles préalables à la clôture** (`GET /fiscal-years/{id}/closing-checks`, [#232](https://github.com/guycorbaz/kesh/issues/232)) : lecture seule, **tout rôle**, clés `read` comprises. `404` si l'exercice n'existe pas ou appartient à une autre société. Rien n'est écrit ni verrouillé : toutes les lectures voient la base au même instant, et la réponse ne bloque rien — la clôture (`POST /fiscal-years/{id}/close`) garde son seul refus, `409 EARLIER_FISCAL_YEAR_OPEN`. Réponse `200` :

```json
{ "fiscalYearId": 7, "fiscalYearName": "Exercice 2026", "startDate": "2026-01-01", "endDate": "2026-12-31",
  "checks": [ { "code": "PENDING_BANK_TRANSACTIONS", "severity": "warning", "status": "attention",
                "count": 4, "details": { "accounts": [ { "bankAccountId": 3, "name": "UBS", "count": 4, "rejectedCount": 1 } ] } },
              … ] }
```

- `checks` porte **toujours sept** contrôles, dans cet ordre : `EARLIER_FISCAL_YEAR_OPEN` (`blocking` ; `details.fiscalYear` = le plus ancien exercice antérieur ouvert, `{id, name}`, ou `null`), `PENDING_BANK_TRANSACTIONS` (`warning` ; transactions `pending` dont la date de comptabilisation bancaire est dans l'exercice, par compte, `rejectedCount` = celles écartées des propositions), `BANK_STATEMENT_AT_YEAR_END` (`warning` ; `details.accounts` = comptes en défaut, `reason` `MISSING` \| `ENDS_BEFORE_YEAR_END` \| `GAP`, avec `statementDate`, `statementClosingBalance`, `ledgerBalanceAtStatement`, `gap` ; `details.notCheckable` = comptes examinés mais non vérifiables, `reason` `NOT_COMPARABLE` \| `NO_BALANCE_STATEMENT`, **sans effet sur le statut**), `DRAFT_INVOICES` (`info` ; factures client en brouillon datées dans l'exercice — les avoirs ne sont pas examinés, Kesh les émettant sans brouillon — ; au plus 20 `{id, label, date}`, par date puis par `id`), `IMPORTED_SUPPLIER_INVOICES_TO_COMPLETE` (`info` ; les documents à traiter, avec ou sans QR-code depuis la v0.14.0 ; ⚠️ **non borné à l'exercice** : un document importé n'a que sa date d'import), `INCONSISTENT_ENTRIES` (`anomaly` ; `details.countByReason` par motif `UNBALANCED`, `TOO_FEW_LINES`, `DATE_OUTSIDE_ITS_YEAR`, `OTHER_YEAR_DATED_INSIDE` — une écriture peut compter sous deux motifs — et au plus 20 `{id, entryNumber, entryDate, fiscalYearName, reasons}`), `NEXT_FISCAL_YEAR` (`info` ; `attention` quand aucun exercice ne suit ; `details.fiscalYear` = `{id, name, startDate, status}` du suivant, c'est-à-dire du premier exercice qui commence après la fin de celui-ci, même après un trou d'une année).
- `status` vaut `ok` ou `attention` ; `count` compte les points relevés — **tous**, alors que les listes de `DRAFT_INVOICES` et `INCONSISTENT_ENTRIES` s'arrêtent à 20 ; pour `EARLIER_FISCAL_YEAR_OPEN` et `NEXT_FISCAL_YEAR`, `1` en `attention`, `0` sinon. Les codes sont stables ; un contrôle neuf serait un changement de contrat annoncé ici.
- Montants en chaînes décimales. L'écart `gap` est arrondi au centime (livres et relevé arrondis d'abord, demi loin de zéro) ; c'est le même que `statementGap` de `GET /bank-accounts`, `null` s'il est nul ou ne se calcule pas.

**Le plan comptable dit quels comptes se lettrent** *(depuis la v0.13.0)* : chaque compte de `GET /accounts` — archivés compris — et des réponses de `POST /accounts`, `PUT /accounts/{id}`, `PUT /accounts/{id}/archive` et `PUT /accounts/{id}/reactivate` porte `letterable` (booléen) : vrai pour un compte d'actif ou de passif qu'aucun compte bancaire — archivé compris — ne désigne. Le **Grand livre**, `GET /reports/general-ledger`, porte depuis la v0.13.0 `letteringCode` par ligne (le code du groupe de lettrage, `null` si la ligne est ouverte) ; ses exports CSV et PDF sont inchangés.

**Le Grand livre et les fins d'exercice** *(depuis la v0.16.0, [#777](https://github.com/guycorbaz/kesh/issues/777), [#778](https://github.com/guycorbaz/kesh/issues/778))* : sur un compte de produits ou de charges (`accountType` `Revenue` ou `Expense`), `GET /reports/general-ledger` pose une rupture (`fiscalYearBreaks`) à chaque **changement d'exercice** dans la période — entre deux lignes du compte, et aussi avant la première ligne et après la dernière (ci-dessous) —, et le solde progressif y repart de zéro. Il y en a une par changement : un exercice intermédiaire où le compte n'a aucune ligne n'en donne pas (son solde est nul). L'`opening` appartient à l'exercice qui contient `from` : si la première ligne tombe dans un exercice suivant, une rupture la précède, datée du jour de cette ligne, qui clôt l'ouverture. Une fin d'exercice franchie après la dernière ligne de la période donne une rupture rendue sur la seule dernière page (celle qui porte la dernière ligne, ou la première si la période n'en a aucune), datée du **premier jour** de l'exercice en cours à `to`. Quand la période franchit une fin d'exercice, `closing` est le **solde de l'exercice en cours à `to`** — le dernier solde progressif, ruptures comprises, `0` si cet exercice n'a aucune ligne dans la période — et non plus `opening` plus les mouvements ; le cumul de la période reste dans `totalDebit` et `totalCredit`, et, **sur une lecture entière** — toutes les pages réunies, `lineCount` disant combien de lignes ; sans `limit`, une page en porte au plus 500 —, `closing` égale `opening` + mouvements − la somme des `closingBalance` des ruptures. Une page ne porte que les ruptures franchies dans ses propres lignes (plus, sur la dernière, la rupture finale) : l'identité ne se vérifie pas page par page. Un compte d'actif ou de passif n'a jamais de rupture, et sa clôture reste `opening` plus les mouvements. L'export CSV écrit désormais chaque rupture comme une ligne (libellé « Clôture de l'exercice — le solde repart de zéro », sans débit ni crédit, le solde de l'exercice clos dans `SoldeProgressif`).

**Les lignes d'une écriture portent leur lettrage** *(depuis la v0.13.0)* : dans toute réponse qui expose les lignes (`GET /journal-entries`, `GET /journal-entries/{id}`, et les réponses de `POST`, `PUT` et de la contre-passation), chaque ligne porte trois champs, **toujours présents** : `letteringKey` (la clé du groupe de lettrage), `letteringCode` (son code affiché, `AA` pour la clé 27) et `letteringOrigin` (`manual`, posé par les routes du lettrage, ou `reversal`, posé par la contre-passation, ou `document`, posé par Kesh sur les lignes d'une facture client soldée — il suit ses règlements et son avoir, voir « Lettrer des lignes ») — tous trois `null` quand la ligne est **ouverte**.

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

### Contre-passer au début de l'exercice suivant — `POST /api/v1/journal-entries/{id}/reverse-at-next-fiscal-year-start`

*Depuis la v0.14.0* ([#232](https://github.com/guycorbaz/kesh/issues/232)) — pour les **transitoires** de fin d'exercice. Écriture (`read-write`), ouverte aux clés comme `/reverse`, **sans corps**. La contre-passation est celle de `/reverse` — mêmes lignes inversées, même liaison (`reversesEntryId`), même lettrage, mêmes refus —, à la date près : elle est datée du **premier jour de l'exercice qui suit** celui de l'écriture (le premier exercice de la société qui commence après la fin du sien, même après un trou), **dans cet exercice**. Réponse `201` et `JournalEntryResponse`, comme `/reverse`. Refus propres, `409` :

- `NEXT_FISCAL_YEAR_MISSING` — aucun exercice ne suit ; `details.fiscalYearName` nomme l'exercice **de l'écriture** ;
- `NEXT_FISCAL_YEAR_CLOSED` — l'exercice qui suit est clôturé ; `details.fiscalYearId`, `details.fiscalYearName`.

Les autres refus sont ceux de la contre-passation, dont `400 PERIOD_LOCKED` si ce premier jour est dans la période verrouillée et `400 LATER_FISCAL_YEAR_CLOSED` si un exercice postérieur à celui-ci est clôturé. Un interblocage est rejoué par le serveur. L'audit `journal_entry.reversed` porte en plus `"dating": "nextFiscalYearStart"`.

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

*(Depuis la v0.13.0 ; à l'écran : Mensuel → Postes ouverts.)* Un **lettrage** marque comme se soldant entre elles des lignes d'un **même compte** d'actif ou de passif — une écriture et sa contre-passation, un acompte et sa reprise. Un groupe réunit **de 2 à 200 lignes** dont la somme `débit − crédit` est **exactement nulle** ; pas de lettrage partiel. Sa **clé** est le plus petit identifiant de ses lignes, son **code** cette clé écrite en lettres (`1 → A`, `27 → AA`). L'origine d'un groupe est `manual` quand il est posé par ces routes, `reversal` quand la **contre-passation** l'a posé (une ligne et sa correction — « Modifier une écriture », ci-dessus). L'origine `document` est posée par Kesh sur les lignes d'une **facture client soldée** ou d'une **facture fournisseur payée** ; il suit la pièce et ses règlements. Facture client : dès qu'une facture est soldée — par ses règlements (`POST /invoices/{id}/settlements`, ou un rapprochement accepté), le solde de son reste ou son avoir —, sa ligne de créance et celles de ses règlements, de son solde ou de son avoir **sur le même compte** forment un groupe ; un règlement partiel ne lettre rien, et l'annulation d'un règlement ou du solde (ou le dé-rapprochement) délettre la facture. Le critère est la somme au grand livre : un avoir émis avant la v0.13.0 sur un autre compte que la créance, ou une créance non lettrable, ne forme aucun groupe. Facture fournisseur : le paiement (`POST /supplier-invoices/{id}/pay`, ou la confirmation d'un lot, `POST /payment-batches/{id}/confirm`) lettre sa ligne d'achat et celle du paiement **sur le compte créanciers** ; l'annulation du paiement, ou de la facture payée, la délettre. Le groupe se pose au geste qui solde la pièce ; **à la mise à jour vers la v0.13.0, l'existant est lettré** de la même façon — pièces soldées, contre-passations déjà passées —, sans entrée d'audit, **sauf** une pièce ou une contre-passation dont toutes les lignes sont en période close, qui reste ouverte. Ces routes ne posent ni ne défont un groupe `document`, et l'annulation qui défairait un groupe `document` figé par une période close est refusée (`409 LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, voir les annulations ci-dessous). ⚠️ Une écriture dont une ligne est lettrée ne se modifie ni ne se supprime (`409 ENTRY_LETTERED`) : délettrez d'abord.

**`POST /api/v1/letterings`** — écriture (`read-write`). Corps : `{ "lineIds": [ … ] }`. Réponse `201` : `{ key, code, origin: "manual", accountId, lines: [ { id, entryId, entryNumber, fiscalYearId, fiscalYearName, date, debit, credit } ] }`. ⚠️ Le numéro d'écriture repart à 1 à chaque exercice : il se lit avec `fiscalYearId` / `fiscalYearName`.

**`GET /api/v1/letterings/{key}`** — lecture (`read` suffit, tout rôle). `{key}` est la clé numérique **ou** le code (`27` ou `AA`, minuscules acceptées) ; toute autre valeur, comme un groupe inconnu ou d'une autre company, rend `404`. Réponse `200` — les champs du `POST`, avec l'`origin` réel, plus de quoi **afficher** le groupe et savoir d'avance si `DELETE` aboutira :

```json
{ "key": 27, "code": "AA", "origin": "reversal", "accountId": 7,
  "accountNumber": "2000", "accountName": "Créanciers",
  "manualDissolutionBlockedBy": "LETTERING_LINE_OWNED_BY_DOCUMENT",
  "lines": [ { "id": 27, "entryId": 3, "entryNumber": 12, "fiscalYearId": 1, "fiscalYearName": "2026",
               "date": "2026-03-01", "debit": "0.0000", "credit": "100.0000",
               "journal": "Achats", "description": "…",
               "document": { "type": "supplierInvoice", "id": 9, "number": "FF-12",
                             "invoiceId": null, "invoiceNumber": null },
               "ownedByDocument": true, "inOpenPeriod": true },
             { "id": 31, "entryId": 5, "entryNumber": 14, "fiscalYearId": 1, "fiscalYearName": "2026",
               "date": "2026-03-20", "debit": "100.0000", "credit": "0.0000",
               "journal": "Achats", "description": "Contre-passation écriture n° 12",
               "document": null, "ownedByDocument": false, "inOpenPeriod": true } ] }
```

- `accountNumber`, `accountName` : le compte du groupe.
- Par ligne, `journal` et `description` : ceux de l'écriture. `document` : la pièce qui possède l'écriture, **même objet et même valeur** que `document` des postes ouverts (ci-dessous) — `null` sans pièce. `ownedByDocument` : la ligne appartient à une pièce — facture, avoir, facture fournisseur, règlement ou solde ; une transaction bancaire seule ne la rend pas possédée (`document` la nomme, `ownedByDocument` reste `false`). `inOpenPeriod` : la ligne est en période ouverte, au sens de la règle des périodes ci-dessous — même valeur que dans les postes ouverts.
- `manualDissolutionBlockedBy` : le code du **premier** refus que rendrait `DELETE`, dans l'ordre du tableau des refus du `DELETE` ci-dessous — `LETTERING_IS_DOCUMENT`, `LETTERING_LINE_OWNED_BY_DOCUMENT` ou `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` —, `null` si le délettrage aboutirait. ⚠️ **Indicatif** : il est lu sans verrou, et `DELETE` fait autorité — un geste concurrent (une clôture, un verrou de période, une pièce annulée) peut le démentir. Comme le délettrage, il ne tient pas compte de la lettrabilité du compte : un groupe dont le compte a été retypé depuis se délettre.

Le `POST` garde sa réponse : ces champs n'y figurent pas.

**`DELETE /api/v1/letterings/{key}`** — écriture (`read-write`). Délettre le groupe : ses lignes redeviennent ouvertes. Réponse `204`.

Les deux gestes sont **tracés** au journal d'audit (`lettering.created`, `lettering.removed`), lignes et exercices compris — et **la clé** qui les a faits, quand c'est une clé. Ces mêmes événements sont émis quand Kesh lettre ou délettre une **facture client** — par un règlement, un solde, un avoir, un rapprochement ou une annulation —, avec en plus `documentType` (`"invoice"`), `documentId` et `documentNumber` ; la clé n'y figure que pour le **rapprochement** (`POST /reconciliation/accept`) : les autres gestes — dé-rapprochement compris — ne portent que l'utilisateur. *Depuis la v0.14.0*, il en va de même d'une **facture fournisseur** réglée par le rapprochement (`documentType` = `"supplierInvoice"`) : la clé figure au lettrage de son paiement.

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
| Groupe d'origine `document` — il suit sa pièce et ses règlements, et ne se délettre pas à la main | `LETTERING_IS_DOCUMENT` | `409` |
| Groupe `reversal` dont une ligne appartient à une pièce (par exemple l'écriture d'achat d'une facture fournisseur annulée) | `LETTERING_LINE_OWNED_BY_DOCUMENT` | `409` |
| Toutes les lignes en période close | `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` | `409` |

Le délettrage n'exige pas que le compte soit encore lettrable : un groupe dont le compte a été retypé ou rattaché depuis à un compte bancaire se délettre. ⚠️ `LETTERING_CONCURRENT_CHANGE` (`409`) signale qu'un groupe a changé entre la lecture et l'écriture : réessayez. ⚠️ Un interblocage est rejoué par le serveur ; s'il persiste, la réponse est un `500` — réessayez.

### Les postes ouverts d'un compte — `GET /api/v1/accounts/{id}/open-items`

*(Depuis la v0.13.0 ; à l'écran : Mensuel → Postes ouverts.)* Lecture (`read` suffit, tout rôle). La liste de ce qui reste **ouvert** sur un compte lettrable à une date — de quoi justifier son solde ligne à ligne, aujourd'hui ou à la date de clôture d'un exercice.

**Une ligne est ouverte à la date `X`** si son écriture est datée au plus tard de `X`, et qu'elle n'est pas lettrée, **ou** que son groupe de lettrage contient une ligne datée **après** `X` (le lettrage n'était pas acquis à `X`). « Ouvert » veut dire « non lettré » : **ce qui est listé** ne dépend ni du statut des factures ni de leur date de paiement — ces données ne servent qu'à décrire les lignes listées (`documentState`, ci-dessous). **Invariant** : la somme `débit − crédit` des lignes ouvertes à `X` égale le solde du compte à `X` — toutes ses lignes datées au plus tard de `X`, tous exercices confondus, c'est-à-dire, pour `X` en fin d'exercice, la clôture du compte dans la Balance de cet exercice (au signe près : la Balance donne le solde du côté naturel du compte, cette route `débit − crédit`).

⚠️ **« Au X » n'est pas un instantané** : la vue se calcule sur le lettrage d'**aujourd'hui**. Pour un `X` en période close — dans un exercice clôturé ou suivi d'un exercice clôturé, ou au plus tard à la date de la période verrouillée —, la liste ne change plus, **tant qu'aucun administrateur ne déverrouille la période, ne rouvre un exercice ni ne restaure une sauvegarde**. Au-dessus, elle peut changer : un délettrage, ou l'annulation d'un règlement, fait réapparaître des lignes « au X » ; l'invariant, lui, tient toujours.

Paramètres : `asOf` (`AAAA-MM-JJ`, défaut — y compris `asOf=` vide : la date du jour, en UTC ; aucune borne métier — une date future rend aussi les écritures datées dans le futur —, mais l'année doit être comprise entre 1000 et 9999), `limit` (défaut 50, ramené entre 1 et 500), `offset` (défaut 0, ramené à 0 s'il est négatif). Réponse `200` :

```json
{ "accountId": 7, "accountNumber": "1100", "asOf": "2026-12-31",
  "balance": "1234.5000", "openTotal": "1234.5000", "total": 42, "offset": 0, "limit": 50,
  "items": [ { "lineId": 1, "entryId": 3, "entryNumber": 12, "fiscalYearName": "2026",
               "date": "2026-03-01", "journal": "Ventes", "description": "…",
               "debit": "100.0000", "credit": "0.0000",
               "document": { "type": "invoice", "id": 9, "number": "F-2026-0009",
                             "invoiceId": null, "invoiceNumber": null },
               "letteringCode": null, "letteringOrigin": null, "letteredOn": null,
               "reason": "unlettered", "documentState": "unpaid", "amountDue": "100.00",
               "manuallyLetterable": false, "inOpenPeriod": true } ] }
```

- `balance` (le solde du compte à `X`), `openTotal` (la somme des lignes ouvertes, égale à `balance`) et `total` (le nombre de lignes ouvertes) portent sur **tout** l'ensemble, non sur la page, et sont lus avec elle dans une même lecture. Les lignes suivent l'ordre du Grand livre : date, exercice, numéro d'écriture, rang de la ligne. ⚠️ Le numéro d'écriture repart à 1 à chaque exercice : il se lit avec `fiscalYearName`.
- `document` : la pièce qui possède l'écriture — `type` parmi `invoice`, `creditNote`, `supplierInvoice`, `settlement` (règlement d'une facture client : `invoiceId` et `invoiceNumber` nomment la facture réglée), `bankTransaction` ; `number` nul pour un règlement, une transaction, et une facture fournisseur saisie sans numéro. Une écriture qui en porte plusieurs (un règlement rapproché) montre la première dans cet ordre. `null` sans pièce. Ces valeurs sont celles de `documentType` au journal d'audit du lettrage.
- `letteringCode`, `letteringOrigin` (`document`, `reversal`, `manual`) et `letteredOn` décrivent le lettrage d'**aujourd'hui** — nuls si la ligne est ouverte aujourd'hui ; `letteredOn` est la date de la ligne la plus tardive du groupe, celle où le lettrage est acquis.
- **Deux champs, deux dates** : `reason` dit pourquoi la ligne est ouverte **à `X`** — `unlettered`, ou `letteredAfterAsOf` (lettrée, mais par un groupe acquis après `X`) ; `documentState` dit où en est **aujourd'hui** la facture client d'une ligne (sa ligne de vente ou un de ses règlements ; `null` pour tout autre ligne, avoir compris) — `paidWithoutSettlementEntry` (marquée payée avant la v0.12.0 sans règlement enregistré : le grand livre porte encore la créance), `nothingDue` (rien à encaisser : réglée, créditée ou soldée), `partiallySettled`, `unpaid`, dans cet ordre de précédence. `amountDue` est son reste dû d'aujourd'hui, **au centime** — deux décimales, quand les montants des lignes en portent quatre — (`null` hors facture client ; négatif en cas de trop-perçu). ⚠️ Pour un `X` passé, `documentState` peut décrire un règlement postérieur à `X` : c'est voulu, l'état de la pièce sert à savoir quoi faire ; le motif **à `X`** est `reason`. Ces deux champs ne filtrent rien.
- `manuallyLetterable` : la ligne est ouverte aujourd'hui et n'est pas celle d'une pièce (facture, avoir, facture fournisseur, règlement) — elle se lettre par `POST /letterings`. Une ligne seulement rapprochée d'une transaction bancaire se lettre. `inOpenPeriod` : la ligne est en période ouverte (exercice ouvert, aucun exercice postérieur clôturé, date postérieure à la période verrouillée) — indicatif, lu sans verrou : un lettrage dont **toutes** les lignes sont hors période ouverte sera refusé.

Refus, **dans l'ordre où ils parlent** :

| Refus | Code | Statut |
|---|---|---|
| `limit` ou `offset` non numérique | (réponse du serveur HTTP, sans `code`) | `400` |
| `asOf` mal formé — rendu avant toute lecture, même pour le compte d'une autre company | `VALIDATION_ERROR` | `400` |
| Compte inconnu ou d'une autre company (indiscernables) | `NOT_FOUND` | `404` |
| Compte non lettrable (charge, produit, compte lié à un compte bancaire) — y compris **devenu** non lettrable : ses groupes restent consultables par `GET /letterings/{key}` | `LETTERING_ACCOUNT_NOT_LETTERABLE` | `409` |

### Les rapprochements proposés — `GET /api/v1/accounts/{id}/lettering-proposals`

*(Depuis la v0.13.0.)* Lecture (`read` suffit, tout rôle). Des **paires** de lignes du compte que Kesh propose de lettrer : ouvertes aujourd'hui, lettrables à la main (`manuallyLetterable`), l'une au débit et l'autre au crédit, **de montants exactement égaux** (aucune tolérance : un règlement amputé de frais bancaires n'est pas une paire), et dont une au moins est en période ouverte (une paire tout entière en période close serait refusée). ⛔ **Kesh n'écrit rien** : accepter une proposition, c'est `POST /letterings` avec ses deux lignes, une à une.

Classement : les paires d'une écriture et de sa **contre-passation** d'abord (`reversalPair`), puis par écart de dates croissant, puis par identifiant de ligne ; **chaque ligne n'apparaît que dans sa meilleure paire**. Pas de proposition à plus de deux lignes.

Paramètre : `limit` (défaut 100, ramené entre 1 et 500) ; **pas d'`offset`** — les paires suivantes se lisent après avoir accepté les premières. Réponse `200` :

```json
{ "accountId": 7, "candidateCount": 340, "total": 12, "limit": 100,
  "items": [ { "amount": "100.0000", "daysApart": 3, "reversalPair": false,
               "debit":  { "lineId": 1, "entryId": 3, "entryNumber": 12, "fiscalYearName": "2026",
                           "date": "2026-03-01", "journal": "Ventes", "description": "…",
                           "document": null, "inOpenPeriod": true },
               "credit": { "lineId": 8, "entryId": 5, "entryNumber": 15, "fiscalYearName": "2026",
                           "date": "2026-03-04", "journal": "Banque", "description": "…",
                           "document": null, "inOpenPeriod": true } } ] }
```

`candidateCount` : les lignes candidates (ouvertes et lettrables à la main) ; `total` : les paires retenues avant `limit`. Chaque ligne d'une paire porte les champs de même nom de la liste des postes ouverts. Un paramètre `offset` est ignoré (la première page est toujours rendue). Refus : ceux des postes ouverts (`400` pour un `limit` non numérique, `404`, `409`), puis, au-delà de **2 000** lignes candidates, `422 LETTERING_PROPOSALS_TOO_MANY_LINES` — jamais une liste tronquée en silence ; les lignes des pièces ne comptent pas.

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
| Règlement par virement sur un compte bancaire **clos** — rouvrir d'abord le compte ; vient avant les motifs qui suivent *(depuis la v0.13.3)* | `BANK_ACCOUNT_CLOSED` | `409`, `details.bankAccountId` |
| Facture créditée par un avoir — le règlement reste ouvert au compte débiteurs | `INVOICE_CREDITED` | `409` |
| Un **solde** existe sur la facture — annuler d'abord le solde (le solde lui-même reste annulable) | `INVOICE_WRITTEN_OFF` | `409` |
| Règlement d'un exercice **clos** — un administrateur doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent | `FISCAL_YEAR_CLOSED` | `409` |
| Règlement (ou solde) lettré avec sa facture dans une période close — un administrateur fait reculer le verrou avant la date la plus récente du lettrage et/ou rouvre les exercices clôturés jusqu'à celui de cette date, selon la cause *(depuis la v0.13.0)* | `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` | `409` |
| Règlement rapproché d'une transaction bancaire | `MATCHED_BANK_TRANSACTION` | `409`, `details.documentId` = la transaction |
| Compte du règlement archivé | `ACCOUNT_ARCHIVED` | `400`, `details.rejected[]` nomme les comptes |
| Aucun exercice ouvert ne couvre la date du jour | `FISCAL_YEAR_INVALID` | `400` |
| L'exercice du jour est suivi d'un exercice clôturé — la contre-passation y changerait son bilan *(depuis la v0.13.0)* | `LATER_FISCAL_YEAR_CLOSED` | `400`, `details.fiscalYearId` / `fiscalYearName` |
| Date du jour dans une période verrouillée | `PERIOD_LOCKED` | `400` |
| Facture ou règlement introuvable (ou d'une autre société) | `NOT_FOUND` | `404` |

**Refus de `POST /api/v1/invoices/{id}/settlements` (enregistrer un règlement)** — distinct de l'annulation ci-dessus : un règlement dont un compte est **le compte débiteurs de la facture** (celui que son écriture de vente a débité) est refusé en `400 SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`, sans rien écrire — l'écriture serait `D 1100 / C 1100`, et le reste dû baisserait sans que rien ne bouge au grand livre. `details` : `{ accountId, accountNumber, claim: "receivable", role }`, où `role` vaut `counterparty` (le compte interne choisi, ou le compte comptable lié au compte bancaire du virement — refusé après les refus propres de ce compte, avant le trop-perçu) ou `rounding` (le compte de différences d'arrondi des réglages, quand le paiement solde au centime — à remplacer dans *Administration → Facturation*). *(Depuis la v0.13.0.)*

*Depuis la v0.13.3* : un règlement **par virement** (`settlementType = bank_transfer`) vers un compte bancaire **archivé** est refusé comme vers un compte inexistant — `404 NOT_FOUND`, sans rien écrire ; jusqu'ici, il était accepté. L'annulation d'un règlement ancien de ce compte reste possible. Un virement vers un compte bancaire **clos** est refusé en `409 BANK_ACCOUNT_CLOSED` (`details.bankAccountId`) — lu sous le verrou du compte, sans fenêtre.

### Solder le reste d'une facture

**`POST /api/v1/invoices/{id}/write-off`** — écriture (`read-write`), ouverte aux clés comme `POST /invoices/{id}/settlements`. Corps : `{ nature, settledOn, version }`, `nature` parmi `discount` (escompte accordé), `bank_fees` (frais bancaires retenus par la banque du client), `bad_debt` (perte sur débiteur), `rounding` (reste d'arrondi, moins de 5 centimes). **Sans montant** : le serveur solde **tout** le reste dû, au reste exact, si bien que la facture est payée. La `version` de la facture est exigée — un écran périmé, ou un reste qui a changé depuis la lecture, est refusé en `409 OPTIMISTIC_LOCK_CONFLICT`. L'écriture, au journal OD, débite le compte de la nature (réglé dans *Administration → Facturation*), crédite la créance ; pour `discount` et `bad_debt`, elle débite aussi la **TVA due au prorata des taux** de la facture. Le solde apparaît dans la liste des règlements (`settlementType = write_off`) et s'annule comme eux. Réponse : `{ invoice, journalEntryId, amount }`.

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

**`POST /api/v1/supplier-invoices/{id}/settlement/cancel`** — écriture (`read-write`), ouverte aux clés comme `POST /supplier-invoices/{id}/pay`. Sans corps. Contre-passe l'écriture de règlement (datée du jour) et ramène la facture à `open` : `settlementType`, `settlementJournalEntryId`, `paidAt`… reviennent à `null`. Le lot de paiement confirmé qui l'a éventuellement réglée **n'est pas modifié**. *(Depuis la v0.13.0)* L'annulation défait le lettrage `document` de la facture — la dette redevient ouverte — et la contre-passation lettre le règlement avec son inverse. Réponse : `{ invoice, reversalJournalEntryId }`. Distinct de `POST /supplier-invoices/{id}/cancel`, qui annule la **facture** elle-même (ci-dessous).

`GET /supplier-invoices/{id}`, la réponse de `pay` et celle de l'annulation portent `settlementCancellable`, `settlementCancelBlockedBy` (code du motif), `settlementCancelBlockedLabel` (numéro du compte archivé) et `lastConfirmedBatch` (`{ id, confirmedAt }` du lot confirmé **le plus récent** qui contient la facture — un fait **historique**, qui ne dit pas d'où vient le règlement courant). Ailleurs, ces champs valent `null` : *non calculés*. Après `pay` ou l'annulation, la facture est **relue** avec ses champs, dans une même lecture : la réponse décrit l'état courant — celui d'une écriture concurrente éventuelle —, jamais un état qui se contredit.

**Refus de `POST /api/v1/supplier-invoices/{id}/pay` (régler une facture fournisseur)** — distinct de l'annulation : une contrepartie qui est **le compte créanciers de la facture** (celui que son écriture d'achat a crédité) — compte interne, ou compte comptable lié au compte bancaire du virement — est refusée en `400 SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`, sans rien écrire (`D 2000 / C 2000` passerait la facture « payée » sans paiement) ; `details` : `{ accountId, accountNumber, claim: "payable", role: "counterparty" }`. Les lots de paiement font le même contrôle : `POST /api/v1/payment-batches` place la facture dans `failed[]` (`200`) avec ce code et `details.bankAccountId` en plus — ou avec `SUPPLIER_INVOICE_PURCHASE_ENTRY_MALFORMED` (`details.reason = "no_credit_line_on_purchase_entry"`, `details.purchaseEntryId`) si son écriture d'achat n'a pas de ligne de crédit ; `POST /api/v1/payment-batches/{id}/confirm` refuse la confirmation entière en `400` du même code, `details.paymentBatchId` et `details.supplierInvoiceId` en plus, si le compte bancaire a été relié au compte créanciers depuis la création du lot (le lot reste `generated` : relier le compte bancaire à son propre compte de banque puis confirmer, ou annuler le lot). *(Depuis la v0.13.0.)*

*Depuis la v0.14.0* : `POST /api/v1/payment-batches` refuse plus de **500** identifiants dans `supplierInvoiceIds` — comptés sur la longueur **brute** de la liste, doublons compris — en `400 VALIDATION_ERROR`, sans rien lire ; une liste **vide** rend `200` `{ batch: null, failed: [] }`, sans lot, comme auparavant. Deux créations **simultanées** qui demandent la même facture ne la mettent plus dans deux lots `generated` (deux fichiers pain.001 pour une facture : un double paiement) : la seconde attend la première, puis place la facture dans `failed[]` (`ALREADY_IN_GENERATED_BATCH`). `failed[]` garde l'ordre de la requête.

*Depuis la v0.13.3* : un paiement **par virement** vers un compte bancaire **archivé** est refusé comme vers un compte inexistant — `404 NOT_FOUND`, sans rien écrire ; jusqu'ici, il était accepté. L'annulation d'un paiement ancien de ce compte reste possible. Un virement vers un compte bancaire **clos** est refusé en `409 BANK_ACCOUNT_CLOSED` (`details.bankAccountId`) — lu sous le verrou du compte, sans fenêtre.

Refus de l'annulation : `SUPPLIER_INVOICE_NOT_PAID` (`409`, la facture n'est pas payée), `BANK_ACCOUNT_CLOSED` (`409`, `details.bankAccountId` : la facture a été payée par virement depuis un compte bancaire **clos** — rouvrir d'abord le compte ; avant les motifs qui suivent ; *depuis la v0.13.3*), `FISCAL_YEAR_CLOSED` (`409`), `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` (`409`, le paiement est lettré avec sa facture dans une période close — un administrateur fait reculer le verrou avant la date la plus récente du lettrage et/ou rouvre les exercices clôturés, selon la cause ; *depuis la v0.13.0*), `MATCHED_BANK_TRANSACTION` (`409`, `details.documentId` = la transaction ; refus défensif — aucun geste de Kesh ne rapproche aujourd'hui l'écriture de règlement d'une facture fournisseur d'une transaction bancaire), `ACCOUNT_ARCHIVED` (`400`, `details.rejected[]`), `FISCAL_YEAR_INVALID` (`400`), `LATER_FISCAL_YEAR_CLOSED` (`400`, l'exercice du jour est suivi d'un exercice clôturé — *depuis la v0.13.0*), `PERIOD_LOCKED` (`400`), `NOT_FOUND` (`404`). ⚠️ **Angle mort assumé** : `settlementCancelBlockedBy` ne prédit pas `LATER_FISCAL_YEAR_CLOSED` ([#568](https://github.com/guycorbaz/kesh/issues/568)) — `settlementCancellable` peut valoir `true` et le clic être refusé.

### Annuler une facture fournisseur

**`POST /api/v1/supplier-invoices/{id}/cancel`** — écriture (`read-write`), ouverte aux clés comme `pay`. Sans corps. Annule la facture, **ouverte ou payée** : contre-passe l'écriture d'**achat** (datée du jour, liée par `reversesEntryId`) et passe la facture à `cancelled`. Payée, son **règlement reste au grand livre, détaché** : `settlementType`, `settlementJournalEntryId`, `paidAt`… reviennent à `null`, et l'écriture de règlement, qui n'appartient plus à aucune pièce, devient un paiement sans facture — contre-passable depuis sa fiche d'écriture. *(Depuis la v0.13.0)* L'annulation défait le lettrage `document` de la facture payée et lettre l'achat avec sa contre-passation ; le règlement détaché reste **ouvert**, lettrable à la main (`POST /letterings`), et sa contre-passation, une fois passée, le lettre avec son inverse (`reversal`). L'audit `supplier_invoice.cancelled` garde le lien (`settlementJournalEntryId`). Réponse : la facture relue, avec ses champs de lecture (Story 25-3-c).

`GET /supplier-invoices/{id}`, les réponses de `pay`, de l'annulation du règlement et de l'annulation de la facture portent aussi `cancellable`, `cancelBlockedBy` (code du motif) et `cancelBlockedLabel` (numéro du compte archivé) — même discipline : `null` = *non calculé*.

Refus, dans l'ordre de précédence : `SUPPLIER_INVOICE_CANCELLED` (`409`, déjà annulée), `BANK_ACCOUNT_CLOSED` (`409`, `details.bankAccountId` : facture **payée** par virement depuis un compte bancaire **clos** — l'annulation détacherait ce paiement du compte ; rouvrir d'abord le compte ; *depuis la v0.13.3*), `FISCAL_YEAR_CLOSED` (`409`, exercice de l'**achat** clôturé), `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` (`409`, la facture est lettrée avec son paiement dans une période close — un administrateur fait reculer le verrou avant la date la plus récente du lettrage et/ou rouvre les exercices clôturés, selon la cause ; *depuis la v0.13.0*), `MATCHED_BANK_TRANSACTION` (`409`, `details.documentId` = la transaction ; l'écriture d'**achat** est rapprochée — refus défensif, au même titre), `ACCOUNT_ARCHIVED` (`400`, `details.rejected[]`), `FISCAL_YEAR_INVALID` (`400`, aucun exercice ouvert le jour), `SUPPLIER_INVOICE_IN_PAYMENT_BATCH` (`409`, facture dans un lot de paiement en cours — dernier des motifs du geste), `LATER_FISCAL_YEAR_CLOSED` (`400`, l'exercice du jour est suivi d'un exercice clôturé — *depuis la v0.13.0* ; non prédit par `cancelBlockedBy`, angle mort assumé, [#568](https://github.com/guycorbaz/kesh/issues/568)), `PERIOD_LOCKED` (`400`), `NOT_FOUND` (`404`). ⚠️ **Changement** : une facture **payée** était refusée (`409 ILLEGAL_STATE_TRANSITION`) ; une facture ouverte dont l'achat est dans un exercice clôturé était annulée, elle est désormais refusée.

⚠️ **`FISCAL_YEAR_CLOSED` rend ici `409`**, alors que la dévalidation le rend en `400` : c'est un refus du **geste** d'annulation, qui porte sur l'exercice du **règlement** ; la contre-passation, elle, serait datée d'un exercice ouvert.

### Accepter des propositions de rapprochement

**`POST /api/v1/reconciliation/accept`** — écriture (`read-write`), rôle Comptable, ouverte aux clés comme les autres routes de réconciliation ; la clé est nommée au journal d'audit (`reconciliation.accepted`). Corps : `{ bankAccountId, proposals: [...] }`, chaque proposition portant `type` (`invoice`, `split`, `rule` ou, *depuis la v0.14.0*, `supplier_invoice`) et `bankTransactionId` — plus `invoiceId` pour une facture client, `supplierInvoiceId` pour une facture fournisseur (§ *Factures fournisseur pour un débit*, ci-dessous). Chaque élément de `accepted[]` porte `supplierInvoiceId` *(depuis la v0.14.0 ; `null` hors d'une facture fournisseur)* et `invoiceId` (`0` hors d'une facture client). Les propositions viennent de `GET /api/v1/reconciliation/proposals`, où `invoiceAmount` est le **reste dû** de la facture (le TTC tant que rien n'est réglé) et `invoiceTotalTtc` son total d'origine, présent seulement quand les deux diffèrent. L'ordre de `candidates` est une **priorité écrite** *(depuis la v0.13.4 ; l'ordre des candidates est celui des versions précédentes, sauf entre factures de même score ; au-delà de 50 factures ouvertes du même montant, l'ensemble des candidates retenues peut aussi changer, [#668](https://github.com/guycorbaz/kesh/issues/668))* : d'abord les candidates `candidateType = "invoice"` qui portent un indice — montant égal au centime (`amountScore = 1`), référence liée au numéro (`referenceScore > 0`) ou contrepartie liée au nom du client (`contactScore > 0`) —, par score décroissant (à score égal, par **date d'exigibilité** croissante — la plus tardive de la date de la facture (`invoiceDate`) et de son échéance, l'échéance, absente de la réponse, valant la date à défaut —, puis date de facture, puis identifiant ; avant la v0.13.4 : identifiant seul) ; puis la règle d'affectation, une au plus, qui se reconnaît à `candidateType = "rule"` — ses sous-scores valent 0 et son `score.total = 1.0` est **conventionnel**, non comparable au score d'une facture —, présente seulement si aucune facture n'atteint `0.5`, et absente aussi quand une facture que le paiement dépasse de `0,005` à `0,05` CHF (un demi-centime à 5 centimes) — qui n'est plus candidate, voir ci-dessous — porte une référence ou une contrepartie liée *(depuis la v0.13.4, [#665](https://github.com/guycorbaz/kesh/issues/665) ; une telle facture passait auparavant devant la règle)* : `candidates` est alors vide. L'écran affiche et accepte `candidates[0]`.

La réponse est un **succès partiel** en `200` : `{ accepted: [...], failed: [{ bankTransactionId, errorCode, details }] }` — une proposition refusée n'empêche pas les autres. Pour une facture, le score est recalculé par le serveur, et un montant supérieur au reste dû est refusé (`RECONCILIATION_OVERPAYMENT`, jamais écrit). `GET /api/v1/reconciliation/proposals` ne propose plus une facture dont le reste dû est inférieur **de plus d'un demi-centime** au montant de la transaction *(depuis la v0.13.4)* : ce refus ne se rencontre plus que pour un couple choisi hors des propositions, ou une facture modifiée entre la lecture et l'acceptation. Contrat inchangé. Une transaction de **débit** ou de montant **nul** ne règle jamais une facture **client** *(« jamais une facture » jusqu'à la v0.13.4 : depuis la v0.14.0, un débit règle une facture **fournisseur**, ci-dessous)* : la proposition `invoice` est refusée en `RECONCILIATION_TRANSACTION_NOT_CREDIT`, avec `details.reason` = `debit_transaction` ou `zero_amount_transaction` et `details.transactionAmount` (même forme que pour `RECONCILIATION_OVERPAYMENT`), **avant** la lecture de la facture — un débit proposé contre une facture inexistante rend ce code, non `INVOICE_NOT_FOUND` —, mais **après** les refus du pré-vol : une transaction introuvable, déjà rapprochée ou déjà rejetée garde son code, et une transaction rapprochée pendant l'attente du verrou du compte rend encore `RECONCILIATION_ALREADY_RECONCILED` *(depuis la v0.13.3)*. ⚠️ **Changement** : pour une facture validée sans règlement, un tel débit rendait auparavant, dans les cas exécutés, `RECONCILIATION_SCORE_TOO_LOW` quand ni sa référence ni sa contrepartie n'étaient reconnues, et **`DATABASE_ERROR`** quand sa référence et sa contrepartie ressemblaient à la facture — la base refusait l'écriture (`details.message` citait la contrainte `chk_jel_debit_credit_exclusive`), rien n'était écrit, mais le code invitait au rejeu ; d'autres montages pouvaient rendre d'autres codes. Une intégration qui rejouait ce refus doit cesser. Les propositions `split` et `rule` admettent un débit (inchangé) ; elles refusent toujours un montant nul en `VALIDATION_ERROR` avec `details.reason = "zero_amount_transaction"` (le rapprochement manuel et ventilé, en `400`) — un client qui ne teste que ce `details.reason` sous `VALIDATION_ERROR` ne verra pas le cas de la facture. Un compte de l'écriture qui serait **le compte débiteurs de la facture** est refusé, lui aussi par proposition, en `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` — `details.role` vaut `counterparty` (le compte bancaire est lié au compte débiteurs ; `details.bankAccountId` est posé, et ce refus précède le trop-perçu) ou `rounding` (le compte de différences d'arrondi des réglages est le compte débiteurs ; sans `bankAccountId`) *(depuis la v0.13.0)*. Si la facture change **pendant** l'acceptation — un règlement, un avoir, une dévalidation enregistrés au même moment —, la proposition est refusée en `RECONCILIATION_INVOICE_NOT_ELIGIBLE` (`details.reason` = `race_during_update`) sans rien écrire : relire les propositions et recommencer. Un interblocage transitoire avec une autre opération est **rejoué** par le serveur, sans être montré ; s'il persiste après trois tentatives, la requête finit en `500 INTERNAL_ERROR` et peut être renvoyée telle quelle — rien n'a été écrit. Refus de la requête entière : `400` (corps invalide), `404` (compte bancaire inconnu), `409 RECONCILIATION_ACCOUNT_LOCKED` (une autre acceptation tient le compte).

**Dates** — la date du paiement est la date de valeur de la transaction, à défaut sa date de comptabilisation. Une facture est refusée en `RECONCILIATION_INVOICE_NOT_ELIGIBLE` avec `details.reason` = `payment_date_before_invoice_date` si le paiement précède **la date de la facture** de plus d'un jour (`details.tolerance_days` = 1, champ ajouté *depuis la v0.14.0* ; 7 pour une facture fournisseur), et avec `payment_date_outside_window` (`details.window_days` = 30) s'il arrive plus de 30 jours après **la plus tardive** de sa date, de son échéance et de son dernier rappel non annulé. `GET /api/v1/reconciliation/proposals` applique la même règle de dates : une facture proposée n'est refusée pour l'une de ces raisons que si elle a changé entre-temps (facture redatée, rappel annulé). ⚠️ **Changement** *(depuis la v0.13.4)* : **seule la seconde raison change de sens** — elle comptait les 30 jours depuis la seule date de la facture, si bien qu'une acceptation autrefois refusée pour ce motif peut désormais aboutir. Codes, raisons et `window_days` sont inchangés ; les refus de montant (`RECONCILIATION_OVERPAYMENT`) aussi.

**Transaction disparue, rapprochée ou rejetée** *(depuis la v0.13.2)* — pour `accept` comme pour `reject`, une transaction du lot **introuvable** (supprimée avec son import, d'un autre compte bancaire ou d'une autre company), **déjà rapprochée** ou **déjà rejetée** est refusée **par proposition** dans `failed[]`, en `200`, et les autres propositions sont traitées. ⚠️ **Changement de contrat** : le lot entier était refusé en `400 VALIDATION_ERROR` (« bankTransactions […] n'appartiennent pas au bankAccountId ») ; un client qui traitait ce `400` reçoit désormais un `200` à lire ligne par ligne. Au **pré-vol** (avant le verrou du compte), les codes sont `BANK_TRANSACTION_NOT_FOUND` (les trois causes d'absence, indiscernables — rien ne distingue une transaction supprimée d'une transaction d'une autre company), `RECONCILIATION_ALREADY_RECONCILED` et `RECONCILIATION_ALREADY_REJECTED` (code neuf), tous trois avec `details: null`. Une transaction **changée pendant le traitement** rend d'autres codes : sous verrou, une proposition `split` ou `rule` rend `RECONCILIATION_TRANSACTION_NOT_PENDING` (inchangé) ; et `RECONCILIATION_ALREADY_RECONCILED` peut porter **`details.reason = "race_during_update"`** quand la transaction est rapprochée, rejetée ou supprimée entre la relecture et l'écriture (acceptation ; repli défensif du rejet) — un client qui teste `details == null` sur ce code se trompe. Le **rejet**, lui, reclasse sous verrou une transaction changée pendant l'attente et rend les **trois mêmes codes** qu'au pré-vol, `details: null` ; le `race_during_update` n'y est qu'un repli défensif, que rien n'atteint en pratique. **Tous ces refus appellent la même conduite** : relire `GET /api/v1/reconciliation/proposals`. Une transaction refusée en `RECONCILIATION_ALREADY_REJECTED` n'y reparaît pas tant que son rejet n'est pas annulé — par `POST /api/v1/reconciliation/transactions/{id}/unreject`, cf. § *Annuler le rejet d'une transaction bancaire* *(depuis la v0.13.3)*. Un lot dont aucune transaction n'est éligible répond `200` sans prendre le verrou du compte.

**Compte bancaire clos** *(depuis la v0.13.3)* — pour `accept` comme pour `reject`, un lot dont le `bankAccountId` désigne un compte **clos** rend `200` avec **chaque** proposition dans `failed[]`, code `BANK_ACCOUNT_CLOSED` (`details.bankAccountId`), sans rien écrire ni prendre le verrou du compte ; ce pré-vol précède tous les refus **par proposition** ci-dessus (le `400` d'un corps invalide et le `404` d'un compte inconnu, qui refusent la requête entière, le précèdent). Le rapprochement manuel (`POST /reconciliation/manual`) et ventilé (`/split`) sur un compte clos sont refusés en `409 BANK_ACCOUNT_CLOSED`. Un compte clos n'a, par construction, aucune transaction en attente (la clôture le refuse) : ce refus n'est atteint que par une course — un lot préparé avant la clôture —, ou sur un compte clos que la fenêtre de la v0.13.3 a laissé avec une transaction en attente ([#651](https://github.com/guycorbaz/kesh/issues/651)). Rouvrir le compte, puis relire les propositions.

**Plafond** *(depuis la v0.13.2)* : un lot de plus de **500** transactions (`proposals` ou `bankTransactionIds`) est refusé en `400 VALIDATION_ERROR` avant toute lecture — il sortait auparavant en `500`. Le **corps** reste borné par la limite par défaut d'Axum (2 Mo) : un lot `accept` de propositions ventilées, jusqu'à 50 lignes chacune, peut être refusé avant 500 propositions, en `400 VALIDATION_ERROR` « requête invalide (corps non-parsable) ».

Une proposition datée dans un exercice **suivi d'un exercice clôturé** (données antérieures à la v0.13.0) est refusée dans `failed[]` en `LATER_FISCAL_YEAR_CLOSED`, avec `details: { fiscalYearId, fiscalYearName }` — l'exercice clôturé le plus proche — et `projectId` quand le projet de la facture est connu (jamais pour `split` ni `rule`) *(depuis la v0.13.0)*. Le rapprochement manuel et ventilé rendent le même refus en `400`.

Pour une proposition `split` ou `rule`, un compte de contrepartie **non imputable** (compte de regroupement, de résultat ou de clôture) est refusé dans `failed[]` en `ACCOUNT_NOT_POSTABLE`, avec `details.rejected[{accountId, accountNumber}]` — la forme du `400` du § 10. Un compte inconnu, d'une autre company ou archivé reste `ACCOUNT_NOT_FOUND` et **prime**. Une règle dont le compte est devenu non imputable n'est plus proposée par `GET /api/v1/reconciliation/proposals` *(depuis la v0.13.0)*.

Une contrepartie qui est **le compte comptable du compte bancaire** lui-même — l'écriture `D banque / C banque` serait nulle — est refusée dans `failed[]` en `VALIDATION_ERROR` avec `details.reason = "counterparty_equals_bank_ledger"`, pour une proposition `split` (une ligne sur ce compte) comme pour une proposition `rule` *(pour `rule`, depuis la v0.13.0)*. Ce refus vient **avant** `ACCOUNT_NOT_FOUND` et `ACCOUNT_NOT_POSTABLE` : un compte de banque devenu non imputable, pris pour contrepartie, rend `VALIDATION_ERROR`. Une règle qui vise ce compte n'est pas proposée par `GET /api/v1/reconciliation/proposals` *(depuis la v0.13.0)*. Si le compte comptable lié au compte bancaire est **archivé**, une proposition `split` ou `rule` est refusée en `BANK_ACCOUNT_NOT_CONFIGURED` avec `details.bankAccountId` — reliez le compte bancaire à un compte actif. ⚠️ **Changement** *(depuis la v0.13.0)* : pour une proposition `rule`, ce refus rendait auparavant `DATABASE_ERROR` (règle ordinaire) ou `ACCOUNT_NOT_FOUND` (règle sur le compte de banque).

Le rapprochement manuel (**`POST /api/v1/reconciliation/manual`**) et ventilé (**`POST /api/v1/reconciliation/split`**) refusent une contrepartie égale au compte de la banque en `400 VALIDATION_ERROR` — message seul, sans `details.reason` *(pour `manual`, depuis la v0.13.0 ; auparavant l'écriture nulle était passée)* —, **avant** le `404 ACCOUNT_NOT_FOUND`, le non-imputable et l'état de la transaction. Ils refusent de même un compte de contrepartie non imputable, en `400 ACCOUNT_NOT_POSTABLE` qui nomme tous les comptes refusés de la requête — après le `404 ACCOUNT_NOT_FOUND`, qui prime *(depuis la v0.13.0)*.

Le rejet en lot (**`POST /api/v1/reconciliation/reject`**) traite et rend ses transactions **par identifiant croissant**, quel que soit l'ordre reçu : `rejected[]` et `failed[]` sortent triés — `failed[]` **en entier**, refus du pré-vol et refus sous verrou mêlés *(depuis la v0.13.2)* —, et la trace `reconciliation.rejected` prend le plus petit identifiant rejeté pour `entity_id` *(depuis la v0.13.2 ; auparavant, l'ordre reçu et le premier identifiant du lot)*.

### Factures fournisseur pour un débit

*(Depuis la v0.14.0, [#686](https://github.com/guycorbaz/kesh/issues/686).)* **Proposition** — dans `GET /api/v1/reconciliation/proposals`, une transaction de **débit** en CHF reçoit des candidates `candidateType = "supplier_invoice"`, jamais une facture client ; un crédit ne reçoit jamais de facture fournisseur, et un montant nul aucune facture. Une candidate fournisseur porte `supplierInvoiceId`, `invoiceNumber` (le numéro de la facture chez le fournisseur, nullable), `invoiceAmount` (son **total** TTC, au centime), `invoiceDate`, `contactName` (le fournisseur) ; `invoiceId` et `invoiceTotalTtc` y sont `null`. Champs additifs : `supplierInvoiceId` et `contactName` figurent sur **toutes** les candidates (`contactName` = le client d'une facture client, `null` pour une règle), et chaque proposition porte `notice` (`null` sauf le cas ci-dessous). Une facture fournisseur est candidate si elle est **ouverte** (`open`), de **total exactement égal** à la valeur absolue du débit (le règlement fournisseur est toujours du total ; un total à plus de deux décimales n'est jamais proposé, [#687](https://github.com/guycorbaz/kesh/issues/687) — *depuis la v0.17.0*, cela ne se produit plus que pour une facture enregistrée avant, une ligne sous le centime étant refusée à la création), **hors de tout lot de paiement `generated`**, datée au plus **7 jours après** le paiement et d'exigibilité — la plus tardive de sa date et de son échéance — au plus **365 jours avant** ; la date de saisie ne compte pas. Elle est notée comme une facture client, sauf la référence : `referenceScore = 1` si la référence du paiement est **égale** à sa référence de paiement (`paymentReference`) ou à son numéro, blancs retirés et casse ignorée, sur six caractères alphanumériques au moins — jamais par inclusion ni préfixe. La priorité est celle des factures client (rang des factures à indice, puis la règle) ; à score égal, l'ordre d'imputation.

**Le lot de paiement du compte** — si une facture fournisseur **ouverte** du montant **exact** est engagée dans un lot `generated` **dont le compte bancaire est celui de la transaction**, la proposition n'a **aucune** candidate, ni facture ni règle, et porte `notice = { code: "supplier_invoice_in_payment_batch", supplierInvoiceId, supplierInvoiceNumber, paymentBatchId }` (un seul code de `notice` à ce jour). Ce débit est peut-être le paiement du lot : régler une autre facture du même montant (la mensualité suivante) compterait la dépense deux fois. Si c'est le cas, annuler le lot ; ne pas le confirmer pour ce débit (le paiement confirmé resterait sans transaction, ci-dessous).

**La règle masquée** — pour un débit, la règle d'affectation n'est pas proposée quand une facture fournisseur candidate atteint `0.5` (toujours le cas : son montant est exact), ni quand une facture fournisseur **ouverte** de total à `0,05` près du débit, non candidate (hors fenêtre, ou dans le lot d'un autre compte), porte un indice — `referenceScore > 0` ou `contactScore > 0` —, ni quand 200 factures ou plus sont à `0,05` près (par prudence).

**Acceptation** — `{ "type": "supplier_invoice", "bankTransactionId": …, "supplierInvoiceId": … }`. Kesh passe le **paiement de la fiche** (`POST /supplier-invoices/{id}/pay`, virement), depuis **le compte bancaire de l'import** : écriture `D créanciers / C banque` du total, datée de la date de paiement de la transaction (valeur, à défaut comptabilisation), journal Banque ; la facture passe `paid` à cette date et est lettrée avec son achat ; la transaction passe `reconciled` sur cette écriture. Audits : `reconciliation.accepted` (`details.type = "supplier_invoice"`, `supplier_invoice_id`, `journal_entry_id`, `purchase_journal_entry_id`, `paid_amount` — le total —, `transaction_amount` — tel quel, négatif —, `score`, `batch_size`) et `supplier_invoice.paid` ; la **clé d'API** figure dans les deux et dans `lettering.created` (`documentType = "supplierInvoice"`). Refus **par proposition**, dans `failed[]` en `200` :

| `errorCode` | Cause | `details` |
|---|---|---|
| `RECONCILIATION_TRANSACTION_NOT_DEBIT` *(neuf)* | crédit ou montant nul, avant toute lecture de la facture | `reason` = `credit_transaction` ou `zero_amount_transaction`, `transactionAmount` |
| `SUPPLIER_INVOICE_NOT_FOUND` | facture absente ou d'une autre company (indiscernables) | `supplierInvoiceId` |
| `RECONCILIATION_INVOICE_NOT_ELIGIBLE` | facture non ouverte ; engagée dans un lot `generated` (tout compte) ; date hors fenêtre ; changée pendant l'acceptation | `reason` = `supplier_invoice_not_open`, `supplier_invoice_in_payment_batch` (+ `paymentBatchId`), `payment_date_before_invoice_date` (+ `tolerance_days` = 7), `payment_date_outside_window` (+ `window_days` = 365), `race_during_update` |
| `RECONCILIATION_SUPPLIER_PAYMENT_HOLD` *(neuf)* | une **autre** facture ouverte du même montant est dans un lot `generated` de ce compte — un appel direct, ou une page chargée avant la création du lot | `reason = "other_invoice_in_payment_batch"`, `supplierInvoiceId` (celle du lot), `paymentBatchId` |
| `RECONCILIATION_AMOUNT_MISMATCH` *(neuf)* | valeur absolue du débit ≠ total | `amountDue` (le total tel que stocké, **à quatre décimales** — `"10.0000"` —, à comparer en décimal et non en chaîne à `invoiceAmount`), `transactionAmount` (tel quel, au centime) |
| `BANK_ACCOUNT_NOT_CONFIGURED` | compte bancaire sans compte au grand livre | `bankAccountId` |
| `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` | le compte bancaire est lié au compte **créanciers** de la facture | `claim = "payable"`, `role = "counterparty"`, `bankAccountId` |
| `INACTIVE_OR_INVALID_ACCOUNTS` | le compte créanciers de l'achat, ou le compte de la banque, désactivé depuis | — |
| `FISCAL_YEAR_INVALID`, `PERIOD_LOCKED`, `LATER_FISCAL_YEAR_CLOSED` | comme pour une facture client | comme pour une facture client |

Aucune garde de score : le score est recalculé pour la réponse (≥ `0.5`, le montant étant exact). Il n'y a pas de garde de devise propre (aucune transaction non CHF n'est importée depuis la v0.13.1). Le refus « débit » d'une proposition `invoice` (`RECONCILIATION_TRANSACTION_NOT_CREDIT`) est inchangé. **Annuler** le rapprochement (§ suivant) retire le paiement : la facture redevient `open`.

**L'acceptation d'une règle rejoue le masquage** — ⚠️ **changement de contrat, des deux côtés** : une proposition `rule` autrefois acceptée est refusée en `RECONCILIATION_RULE_MASKED` *(neuf)* quand une facture la défend, relue au moment de l'acceptation (un appel direct, ou une page chargée avant la saisie de la facture) : pour un **débit**, `details.reason = "payment_batch"` (facture du montant exact dans un lot `generated` du compte ; `supplierInvoiceId`, `paymentBatchId`) ou `"invoice_candidate"` (une facture fournisseur candidate ; `supplierInvoiceId`) ; pour un **crédit**, `"invoice_slightly_overpaid"` (une facture client que le paiement dépasse de `0,005` à `0,05` et qui porte une référence ou une contrepartie liée ; `invoiceId`). Rien n'est écrit. Les autres formes du masquage de la proposition (facture fournisseur presque au montant ou hors fenêtre portant un indice, lot d'un autre compte, liste pleine ; côté client, une facture **candidate**) ne sont **pas** rejouées. Le geste attendu est de traiter la facture — accepter la proposition `supplier_invoice` (débit), ou enregistrer le règlement de la facture client puis rattacher la transaction (`POST /api/v1/reconciliation/transactions/{id}/link`, crédit). `POST /api/v1/reconciliation/manual` ou `/split` ne conviennent que pour une transaction qui ne règle aucune facture saisie : sinon la charge ou le produit serait compté deux fois.

⚠️ **Dépense déjà réglée par un autre chemin** — le débit d'une facture fournisseur **déjà payée** (depuis la fiche, par la confirmation d'un lot, ou payée puis annulée, son paiement restant au grand livre), ou d'une dépense passée par une écriture saisie au journal, n'est pas **proposé**, même s'il porte sa référence : la facture n'est plus ouverte. Pour ce débit, l'API propose peut-être la règle, ou une **autre** facture ouverte du même montant, et **rien n'empêche** de les accepter, ni `POST /reconciliation/manual` ni `/split` — chacun compterait la dépense ou le paiement deux fois. *Depuis la v0.17.0*, la transaction se **rattache** à l'écriture déjà passée (`GET …/entry-candidates`, puis `POST …/link-entry`, section suivante) : aucune seconde écriture. Restent sans écriture à laquelle se rattacher, à **rejeter** plutôt qu'à affecter : un lot de plusieurs factures débité en une ligne et le second côté d'un virement entre deux comptes propres ([#823](https://github.com/guycorbaz/kesh/issues/823)) ; un paiement passé sur un ancien compte du grand livre, ou sur un autre compte bancaire à compte du grand livre distinct ([#827](https://github.com/guycorbaz/kesh/issues/827)) ; un paiement qui inclut des frais de rappel ([#401](https://github.com/guycorbaz/kesh/issues/401)).

**`paidAt`** — *depuis la v0.14.0*, le `paidAt` d'une facture fournisseur payée est la **date du paiement** à minuit (`AAAA-MM-JJT00:00:00`) — le `paymentDate` de `POST /supplier-invoices/{id}/pay`, celui de `POST /payment-batches/{id}/confirm`, la date de paiement de la transaction acceptée —, et non plus l'heure de l'enregistrement. Les factures déjà payées gardent leur valeur.

### Annuler un rapprochement bancaire

**`GET /api/v1/reconciliation/transactions/{id}`** — lecture, rôle Comptable (comme les propositions). La transaction bancaire (mêmes champs que dans le détail d'un import, dont `matchedEntryId`), `kind` (`invoice_settlement` : le rapprochement a réglé une facture client — ou, *depuis la v0.13.4*, la transaction a été rattachée au règlement saisi depuis la fiche, § « Rattacher une transaction à un règlement déjà enregistré » ; *depuis la v0.14.0*, `supplier_settlement` : le rapprochement porte le paiement d'une facture **fournisseur** ; `entry` : une écriture que seule la transaction possède — éclatement, règle, rapprochement manuel, ou le paiement d'une facture fournisseur **annulée** après paiement, détaché d'elle ; `null` : la transaction n'est pas rapprochée), `invoiceId` et `invoiceNumber` (la facture **client** — `null` pour un paiement fournisseur), *depuis la v0.14.0* `supplierInvoiceId` et `supplierInvoiceNumber` (la facture fournisseur et le numéro donné par le fournisseur, `null` hors de `supplier_settlement`), et **`cancellable`** — calculé par la fonction même qui refuserait l'annulation, lu **dans un seul instantané**. Quand il vaut `false`, `cancelBlockedBy` porte le code du motif, `cancelBlockedLabel` le numéro du compte archivé et `cancelBlockedDocumentId` l'**autre** transaction qui pointe la même écriture. ⚠️ Calculé pour **une** transaction : le détail d'un import ne le porte pas. ⚠️ **Angle mort assumé** : `cancellable` ne voit pas qu'un exercice postérieur à l'exercice du jour est clôturé ([#568](https://github.com/guycorbaz/kesh/issues/568)) — le clic rend alors `400 LATER_FISCAL_YEAR_CLOSED`.

**`POST /api/v1/reconciliation/transactions/{id}/cancel`** — écriture (`read-write`), ouverte aux clés comme les autres routes de réconciliation ; la clé est nommée au journal d'audit (`reconciliation.cancelled`). Sans corps. Défait le lien, contre-passe l'écriture du rapprochement (**datée du jour**) et, pour une facture client, retire son règlement — *depuis la v0.14.0*, pour une facture fournisseur, retire son paiement : la facture revient `open`, à payer, comme par `POST /supplier-invoices/{id}/settlement/cancel` ; la transaction revient `pending` et réapparaît dans `GET /reconciliation/proposals`. Réponse : `{ bankTransaction, reversalJournalEntryId, invoiceId, supplierInvoiceId }` (`supplierInvoiceId` *depuis la v0.14.0*, `null` hors d'un paiement fournisseur). L'entrée d'audit `reconciliation.cancelled` porte `kind` et, pour un paiement fournisseur, `supplierInvoiceId` (`invoiceId` et `settlementId` y sont alors `null`). Un interblocage transitoire avec une autre opération est **rejoué** par le serveur, sans être montré.

| Refus | Code | Statut |
|---|---|---|
| La transaction n'est pas rapprochée | `BANK_TRANSACTION_NOT_RECONCILED` | `409` |
| Le compte bancaire de la transaction est **clos** — rouvrir d'abord le compte ; vient avant les motifs qui suivent *(depuis la v0.13.3)* | `BANK_ACCOUNT_CLOSED` | `409`, `details.bankAccountId` |
| Facture créditée par un avoir — son règlement reste ouvert au compte débiteurs | `INVOICE_CREDITED` | `409` |
| Écriture du rapprochement dans un exercice **clos** — l'exercice du **paiement**, jamais celui de la facture | `FISCAL_YEAR_CLOSED` | `409` |
| Rapprochement lettré avec sa facture dans une période close — un administrateur fait reculer le verrou avant la date la plus récente du lettrage et/ou rouvre les exercices clôturés jusqu'à celui de cette date, selon la cause *(depuis la v0.13.0)* ; refusé **avant** que le lien soit défait | `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` | `409` |
| Une autre transaction pointe la même écriture | `MATCHED_BANK_TRANSACTION` | `409` |
| Compte de l'écriture archivé | `ACCOUNT_ARCHIVED` | `400`, `details.rejected[]` nomme les comptes |
| Aucun exercice ouvert ne couvre la date du jour | `FISCAL_YEAR_INVALID` | `400` |
| L'exercice du jour est suivi d'un exercice clôturé — la contre-passation y changerait son bilan *(depuis la v0.13.0)* | `LATER_FISCAL_YEAR_CLOSED` | `400`, `details.fiscalYearId` / `fiscalYearName` |
| Date du jour dans une période verrouillée | `PERIOD_LOCKED` | `400` |
| Compte bancaire en cours de réconciliation par une autre opération | `RECONCILIATION_ACCOUNT_LOCKED` | `409` |
| Transaction introuvable (ou d'une autre société) | `NOT_FOUND` | `404` |

**`GET /api/v1/bank-imports/{id}`** : chaque transaction porte désormais `matchedEntryId`, l'écriture liée par son rapprochement (`null` sinon), et — *depuis la v0.13.3* — `autoMatchRejectedAt`, la date de son rejet (cf. § suivant).

### Annuler le rejet d'une transaction bancaire

*(Depuis la v0.13.3.)* **`POST /api/v1/reconciliation/transactions/{id}/unreject`** — écriture (`read-write`), rôle Comptable, ouverte aux clés comme les autres routes de réconciliation ; la clé est nommée au journal d'audit (`reconciliation.unrejected`, dont `details.previouslyRejectedAt` porte la date du rejet annulé). Sans corps ; réponse **`204`**, sans corps. Défait le rejet posé par `POST /api/v1/reconciliation/reject` : le marqueur repasse à `null` et la transaction **redevient proposable** — elle figure de nouveau dans `GET /api/v1/reconciliation/proposals`, où elle peut être acceptée, rejetée, rapprochée manuellement ou éclatée — et rattachée à un règlement déjà enregistré, ce que l'API permet aussi sans annuler le rejet *(depuis la v0.13.4 ; § suivant)*. ⚠️ Cette liste ne rend que les `limit` transactions en attente les plus récentes (100 par défaut, 500 au plus ; `hasMore` vaut `true` s'il en reste) : une transaction plus ancienne dont le rejet est annulé n'y reparaît qu'une fois les plus récentes traitées. Le geste ne passe **aucune écriture** ni aucun règlement ; le statut et le lien de la transaction ne changent pas. Il prend le verrou du compte bancaire, comme les autres gestes de réconciliation, et n'est pas rejoué sur interblocage : le seul cycle connu, avec la remise à zéro d'une installation de démonstration, rend `500` sans rien écrire — la requête peut être renvoyée. Sur un compte bancaire **clos**, le geste est refusé en `409 BANK_ACCOUNT_CLOSED` (`details.bankAccountId`) : rouvrir d'abord le compte. Une transaction rejetée comptant comme en attente, la clôture d'un tel compte est elle-même refusée (`412 BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS`) ; le `409` de l'annulation du rejet n'est donc atteint que sur un compte clos que la fenêtre de la v0.13.3 a laissé dans cet état ([#651](https://github.com/guycorbaz/kesh/issues/651), fermée en v0.13.4 ; note ⁹ du §7).

**`autoMatchRejectedAt`** — champ des transactions bancaires du détail d'un import, de `GET /api/v1/reconciliation/transactions/{id}` et de la réponse de `…/cancel` : la date du rejet, chaîne ISO 8601 UTC (suffixe `Z`), ou `null` si la transaction n'est pas rejetée. C'est la même chaîne que `previouslyRejectedAt` à l'audit de l'annulation.

| Refus | Code | Statut |
|---|---|---|
| La transaction n'est pas rejetée — jamais, ou plus (son rejet a déjà été annulé) —, ou elle est rapprochée | `RECONCILIATION_TRANSACTION_NOT_REJECTED`, `details.bankTransactionId` | `409` |
| Compte bancaire en cours de réconciliation par une autre opération | `RECONCILIATION_ACCOUNT_LOCKED` | `409` |
| Transaction introuvable, d'une autre société, ou supprimée avec son import pendant l'attente du verrou | `NOT_FOUND` | `404` |
| Défensif, inatteignable : la ligne lue sous verrou n'a pas été écrite | `OPTIMISTIC_LOCK_CONFLICT` | `409` |
| Rôle Consultation ; clé `read` | — ; `API_KEY_READ_ONLY` | `403` |

Aucun refus n'écrit ni ne trace quoi que ce soit.

### Rattacher une transaction à un règlement déjà enregistré

*(Depuis la v0.13.4, [#670](https://github.com/guycorbaz/kesh/issues/670).)* Un règlement client par virement enregistré par `POST /api/v1/invoices/{id}/settlements` passe son écriture sans toucher la transaction du relevé, qui reste `pending`. Le rattachement la passe `reconciled` en pointant l'écriture **existante** du règlement — **aucune** écriture, aucun règlement, aucun lettrage ; l'état produit est celui d'une proposition « facture » acceptée (`kind = invoice_settlement`), et `…/cancel` le défait de même, **règlement compris**.

**La règle.** Un règlement est rattachable à une transaction si et seulement si : la transaction est un **crédit** en CHF, `pending` (rejetée ou non) ; le règlement est de la même société, `settlementType = bank_transfer`, sur le **même compte bancaire** ; son écriture n'est rapprochée d'**aucune** transaction ; soit `M` le mouvement de son écriture sur le compte du grand livre **actuellement** lié au compte bancaire, `M > 0` et `0 ≤ montant de la transaction − M ≤ 0.05`. Aucune condition de date. Un règlement passé sur un ancien compte du grand livre du compte bancaire n'est pas rattachable.

**`GET /api/v1/reconciliation/transactions/{id}/link-candidates`** — lecture, rôle Comptable (comme les propositions). Réponse `200` : `{ bankTransactionId, candidates: [{ settlementId, invoiceId, invoiceNumber, contactName, settledOn, amount, gap, daysApart, journalEntryId }] }` — `amount` = `M` et `gap` = transaction − `M`, chaînes à **deux décimales** ; `daysApart` = nombre de jours, toujours positif, entre `settledOn` et la date de paiement de la transaction (date de valeur, à défaut de comptabilisation). Tri : `gap` croissant, puis `daysApart` croissant, puis `settlementId` ; **50** au plus. Une transaction de débit, nulle ou hors CHF rend `200` et `candidates: []`. Kesh ne présélectionne rien : c'est au client de choisir.

**`POST /api/v1/reconciliation/transactions/{id}/link`** — écriture (`read-write`), rôle Comptable, ouverte aux clés comme les autres routes de réconciliation ; la clé est nommée au journal d'audit (`reconciliation.linked`, dont `details` porte `bank_transaction_id`, `settlement_id`, `invoice_id`, `journal_entry_id`, `amount`, `settlement_amount`, `gap` et `was_previously_rejected`). Corps `{ "settlementId": 4 }`. Réponse `200` : `{ bankTransactionId, journalEntryId, invoiceId, settlementId }`. Le marqueur de rejet est remis à `null`. Un interblocage transitoire est **rejoué** par le serveur, sans être montré.

Refus, **dans cet ordre** — ceux que marque « `POST` » ne concernent que le rattachement ; aucun n'écrit ni ne trace quoi que ce soit. Le `GET` s'arrête après la ligne « Transaction déjà rapprochée » de son pré-vol ; le `POST` juge ensuite, **sous le verrou du compte**, la transaction, puis le règlement :

| Refus | Code | Statut |
|---|---|---|
| Clé `read` au `POST` ; rôle Consultation | `API_KEY_READ_ONLY` ; — | `403` |
| `settlementId` absent ou `≤ 0` (`POST`) | `VALIDATION_ERROR` | `400` |
| Transaction introuvable (ou d'une autre société) | `NOT_FOUND` | `404` |
| Compte bancaire de la transaction **archivé** | `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` | `404` |
| Compte bancaire de la transaction **clos** | `BANK_ACCOUNT_CLOSED`, `details.bankAccountId` | `409` |
| Compte bancaire sans compte du grand livre lié | `BANK_ACCOUNT_NOT_CONFIGURED` | `412` |
| Transaction déjà rapprochée (`GET` ; au `POST`, voir plus bas) | `RECONCILIATION_TRANSACTION_NOT_PENDING` | `404` |
| Compte bancaire en cours de réconciliation par une autre opération (`POST`) | `RECONCILIATION_ACCOUNT_LOCKED` | `409` |
| Transaction supprimée avec son import pendant l'attente du verrou (`POST`) | `NOT_FOUND` | `404` |
| Transaction déjà rapprochée (`POST`) | `RECONCILIATION_TRANSACTION_NOT_PENDING` | `404` |
| Transaction qui n'est pas un crédit en CHF — jugée **avant** le règlement, quel que soit `settlementId` (`POST`) | `RECONCILIATION_SETTLEMENT_NOT_LINKABLE`, `details.bankTransactionId`, `details.settlementId` | `409` |
| Règlement introuvable ou d'une autre société — annulé depuis la fiche, par exemple (`POST`) | `NOT_FOUND` | `404` |
| Défensif : compte bancaire clos pendant l'attente du verrou — la clôture refuse tant qu'une transaction est en attente (`POST`) | `BANK_ACCOUNT_CLOSED`, `details.bankAccountId` | `409` |
| La règle n'admet pas le règlement — autre type ou autre compte bancaire, montant, écriture déjà rapprochée ou passée sur un ancien compte du grand livre ; la liste a changé : la relire (`POST`) | `RECONCILIATION_SETTLEMENT_NOT_LINKABLE`, `details.bankTransactionId`, `details.settlementId` | `409` |
| Défensif, inatteignable : la ligne lue sous verrou n'a pas été écrite (`POST`) | `OPTIMISTIC_LOCK_CONFLICT` | `409` |

### Rattacher une transaction à une écriture déjà passée

*(Depuis la v0.17.0, [#671](https://github.com/guycorbaz/kesh/issues/671).)* Un mouvement déjà comptabilisé — facture fournisseur payée depuis la fiche (`POST /supplier-invoices/{id}/pay`) ou par un lot d'**une** facture, écriture saisie par `POST /journal-entries`, règlement client par compte interne sur le compte de la banque — laisse la transaction du relevé `pending`. Le rattachement la passe `reconciled` en pointant l'écriture **existante** — **aucune** écriture, aucun règlement, aucun lettrage. Kesh ne présélectionne rien : c'est au client de choisir. `…/cancel` défait le rattachement selon l'écriture : une écriture saisie est **contre-passée** (`kind = entry`), un règlement client retiré (`invoice_settlement`), un paiement fournisseur annulé (`supplier_settlement`).

**La règle.** Une écriture est rattachable à une transaction si et seulement si : la transaction est en CHF, de montant **non nul** (crédit ou débit), `pending` (rejetée ou non) ; l'écriture est de la même société ; son mouvement `M` sur le compte du grand livre **actuellement** lié au compte bancaire (somme des débits moins somme des crédits de ses lignes sur ce compte) **égale exactement** le montant de la transaction, signe compris ; elle n'est rapprochée d'**aucune** transaction ; elle ne contre-passe rien et n'est contre-passée par rien ; ce n'est pas l'écriture d'un règlement client **par virement sur ce compte bancaire** (celle-là se rattache par `…/link`, section précédente). Aucune condition de date. ⚠️ Deux comptes bancaires qui partagent un compte du grand livre voient chacun les écritures non rapprochées de l'autre.

**`GET /api/v1/reconciliation/transactions/{id}/entry-candidates`** — lecture, rôle Comptable (comme les propositions). Réponse `200` : `{ bankTransactionId, candidates: [{ journalEntryId, entryNumber, fiscalYearId, entryDate, journal, description, amount, daysApart, cancelBlockedBy, cancelBlockedLabel }] }` — `amount` = `M`, chaîne signée à **deux décimales** ; `journal` = code du journal (`Achats`, `Ventes`, `Banque`, `Caisse`, `OD`) ; `daysApart` = nombre de jours, toujours positif, entre `entryDate` et la date de paiement de la transaction (date de valeur, à défaut de comptabilisation). Tri : `daysApart` croissant, puis `journalEntryId` ; **50** au plus. `cancelBlockedBy` : le code du premier motif qui, **au moment de la lecture**, empêcherait d'annuler ce rattachement (`FISCAL_YEAR_CLOSED`, `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, `ACCOUNT_ARCHIVED`, `FISCAL_YEAR_INVALID`, `INVOICE_WRITTEN_OFF`, `INVOICE_CREDITED` — les codes de `GET /reconciliation/transactions/{id}`), ou `null` — un **instantané** : un motif peut apparaître après le rattachement (clôture d'exercice) ou disparaître ; `cancelBlockedLabel` : le numéro du compte archivé (`ACCOUNT_ARCHIVED`). Une transaction nulle ou hors CHF rend `200` et `candidates: []`.

**`POST /api/v1/reconciliation/transactions/{id}/link-entry`** — écriture (`read-write`), rôle Comptable, ouverte aux clés comme les autres routes de réconciliation ; la clé est nommée au journal d'audit (`reconciliation.entry_linked`, dont `details` porte `bank_transaction_id`, `journal_entry_id`, `entry_number`, `fiscal_year_id`, `amount` et `was_previously_rejected`). Corps `{ "journalEntryId": 211 }`. Réponse `200` : `{ bankTransactionId, journalEntryId }`. Le marqueur de rejet est remis à `null`. Un interblocage transitoire est **rejoué** par le serveur, sans être montré.

Refus, **dans cet ordre** — aucun n'écrit ni ne trace quoi que ce soit ; le `GET` suit l'ordre de `…/link-candidates` (introuvable, archivé, clos, non lié, déjà rapprochée). Le `POST` juge, **sous le verrou du compte**, la transaction, puis l'écriture :

| Refus | Code | Statut |
|---|---|---|
| Clé `read` au `POST` ; rôle Consultation | `API_KEY_READ_ONLY` ; — | `403` |
| `journalEntryId` absent ou `≤ 0` (`POST`) | `VALIDATION_ERROR` | `400` |
| Transaction introuvable (ou d'une autre société) | `NOT_FOUND` | `404` |
| Compte bancaire de la transaction **archivé** | `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` | `404` |
| Compte bancaire de la transaction **clos** | `BANK_ACCOUNT_CLOSED`, `details.bankAccountId` | `409` |
| Compte bancaire sans compte du grand livre lié | `BANK_ACCOUNT_NOT_CONFIGURED` | `412` |
| Transaction déjà rapprochée (`GET` ; au `POST`, voir plus bas) | `RECONCILIATION_TRANSACTION_NOT_PENDING` | `404` |
| Compte bancaire en cours de réconciliation par une autre opération (`POST`) | `RECONCILIATION_ACCOUNT_LOCKED` | `409` |
| Transaction supprimée avec son import pendant l'attente du verrou (`POST`) | `NOT_FOUND` | `404` |
| Transaction déjà rapprochée (`POST`) | `RECONCILIATION_TRANSACTION_NOT_PENDING` | `404` |
| Transaction hors CHF ou de montant nul — jugée **avant** l'écriture, quel que soit `journalEntryId` (`POST`) | `RECONCILIATION_ENTRY_NOT_LINKABLE`, `details.bankTransactionId`, `details.journalEntryId` | `409` |
| Écriture introuvable ou d'une autre société — supprimée entre-temps, par exemple (`POST`) | `NOT_FOUND` | `404` |
| Défensif : compte bancaire clos pendant l'attente du verrou (`POST`) | `BANK_ACCOUNT_CLOSED`, `details.bankAccountId` | `409` |
| La règle n'admet pas l'écriture — mouvement différent, autre compte du grand livre, déjà rapprochée, contre-passée ou contre-passation, règlement client par virement sur ce compte ; la liste a changé : la relire (`POST`) | `RECONCILIATION_ENTRY_NOT_LINKABLE`, `details.bankTransactionId`, `details.journalEntryId` | `409` |
| Défensif, inatteignable : la ligne lue sous verrou n'a pas été écrite (`POST`) | `OPTIMISTIC_LOCK_CONFLICT` | `409` |

### Importer un relevé — l'empreinte d'aperçu

*(Depuis la v0.13.2.)* **`POST /api/v1/bank-imports/preview`** rend, pour un CSV comme pour un CAMT.053, un champ **`previewFingerprint`** : 64 caractères hexadécimaux minuscules, empreinte des entrées de l'aperçu — le compte bancaire (et son IBAN), le fichier (son hash), son format et, pour un CSV, le **contenu** du profil résolu. Elle n'est pas un secret et n'authentifie rien.

**`POST /api/v1/bank-imports`** accepte ce champ en multipart, **facultatif mais conseillé** : la création recalcule l'empreinte sur ses propres entrées et, si elle diffère, refuse l'import — **`409 BANK_IMPORT_PREVIEW_STALE`**, rien n'est écrit. C'est le cas quand le profil a été modifié (colonnes, sens des débits, séparateurs, nom…), qu'un autre profil ou un autre compte est envoyé, que l'IBAN du compte a changé ou que le fichier n'est plus le même. Refaire l'aperçu, le vérifier, puis créer avec la nouvelle empreinte. Une empreinte mal formée (longueur, majuscules, caractère hors `[0-9a-f]`) ou envoyée deux fois rend `400 VALIDATION_ERROR`. Ni l'état de la base (doublons de fichier ou de lignes, recalculés à la création de toute façon), ni les cases de confirmation, ni le nom du fichier n'y entrent ; un enregistrement du profil sans changement de contenu ne la périme pas.

⚠️ **Angle mort assumé** : une création **sans** `previewFingerprint` n'est pas contrôlée — elle importe avec le profil du moment, comme avant la v0.13.2. L'écran de Kesh envoie toujours l'empreinte.

⚠️ **Envoyez aussi l'id du profil résolu.** Une empreinte envoyée **sans** `bankProfileId` laisse la création résoudre le profil par le nom du fichier : si un **autre** profil à motif concordant a été modifié ou créé entre-temps (il passe en tête, le plus récent l'emportant), ou si l'aperçu portait un `bankProfileId` explicite que la création n'envoie pas, le profil résolu peut différer — `409`, à juste titre — ou manquer — `404 BANK_CSV_NO_PROFILE_MATCH`. Renvoyer `csvProfileMatch.profileId`, l'id que l'aperçu a résolu, l'évite.

**Ordre des refus.** Les refus qui précèdent la résolution du profil restent prioritaires : `400` du multipart, `404 BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`, `409 BANK_ACCOUNT_CLOSED` (compte clos, lu au même pré-vol ; *depuis la v0.13.3*), et pour un CSV le `422 BANK_IMPORT_DUPLICATE_FILE` non confirmé puis le `404 BANK_CSV_NO_PROFILE_MATCH`. Le `409` vient ensuite, avant tout parse du fichier (ses octets sont déjà lus : le hash en est tiré) : il précède donc les refus nés du parse, dont ceux de la devise — `422 BANK_IMPORT_UNSUPPORTED_CURRENCY` et, pour un CSV, `422 BANK_CSV_MIXED_CURRENCIES` (depuis la v0.13.2). ⚠️ **Asymétrie CSV / CAMT** : pour un CAMT.053, le `409` **précède** le `422 BANK_IMPORT_DUPLICATE_FILE` ; pour un CSV, il le **suit**. Les deux refusent sans importer.

### Documents à traiter — l'import du dossier

*(Section créée par la v0.14.0, [#685](https://github.com/guycorbaz/kesh/issues/685).)* Le dossier d'import (`KESH_INBOX_DIR`, unique par instance) se lit à la demande ; ses documents entrent dans une liste de **documents à traiter**, avec ou sans QR-code. Les routes ci-dessous sont réservées au rôle **Comptable** (et Administrateur), ouvertes aux clés selon la portée (§4) ; tout est scopé à la société de l'utilisateur.

**`POST /api/v1/inbox-import`** — écriture (`read-write`), sans corps. Lit le dossier et rend **`200`** avec `{ accepted, failed, warnings }`, même si des fichiers échouent (patron *batch*). `409 INBOX_IMPORT_ALREADY_RUNNING` si un import est en cours.

- **`accepted[]`** : `{ importedSupplierInvoiceId, fileName, qr, qrError? }`. *Depuis la v0.14.0*, `qr` dit ce que le document apporte de son QR : **`present`** (données lues et stockées), **`none`** (aucun QR trouvé), **`unreadable`** (un QR trouvé mais inexploitable : le document entre **sans** les données du QR). `qrError` n'apparaît que pour `unreadable` : `INVALID_SPC_PAYLOAD` (QR qui n'est pas un payload SPC valide), `INVALID_IBAN` (IBAN du QR invalide), `PDF_RENDER_ERROR` (PDF que Kesh ne peut pas rendre — y compris quand la bibliothèque PDF manque, ou au-delà de `KESH_INBOX_MAX_PDF_PAGES` pages sans QR), `FIELD_TOO_LONG` (une valeur du QR dépasse la largeur de sa colonne, comptée en caractères), **`QR_AMOUNT_OUT_OF_RANGE`** (montant hors de `0.01..=999999999.99`, ou à plus de deux décimales significatives ; un montant vide reste admis). Un document **réactivé** — retiré de la liste puis redéposé — porte `present` ou `none` selon ce qui a été stocké (un QR inexploitable ne l'a pas été).
- **`failed[]`** : `{ fileName, errorCode, details? }`, le fichier déplacé dans `<dossier>/failed/` : `UNSUPPORTED_FILE_TYPE` (extension hors `pdf`/`png`/`jpg`/`jpeg`, octets incohérents avec l'extension, image non décodable), `FILE_TOO_LARGE` (`details.byteSize`, `details.maxBytes`), `SYMLINK_REJECTED`, `DUPLICATE` (même contenu déjà dans la liste ou complété), `FILE_READ_ERROR`. ⚠️ *Depuis la v0.14.0*, **`NO_QR_CODE_FOUND` n'existe plus**, et `INVALID_SPC_PAYLOAD`, `INVALID_IBAN`, `PDF_RENDER_ERROR` et `FIELD_TOO_LONG` ne sortent plus en `failed[]` — sauf `FIELD_TOO_LONG` et `INVALID_SPC_PAYLOAD`, gardés comme filets d'une insertion refusée par la base, inatteignables tant que le contrôle du QR est juste.
- **`warnings[]`** : messages lisibles (fichier en cours d'écriture, limite de fichiers par run).

**`GET /api/v1/imported-supplier-invoices?status=`** — lecture. `status` obligatoire, `to_complete` \| `completed` \| `discarded` (sinon `400`). Rend la liste des documents, du plus récent au plus ancien : `id`, `status`, `supplierInvoiceId`, `fileHash`, `storagePath`, `originalFilename`, `mimeType`, `byteSize`, les champs du QR, `version`, `createdAt`, `updatedAt`. ⚠️ *Depuis la v0.14.0*, sept champs du QR deviennent **nullables** — `creditorIban`, `isQrIban`, `creditorAddressType`, `creditorName`, `creditorCountry`, `referenceType`, `currency` — et sont `null` **ensemble** pour un document sans QR (contrainte « tout ou rien » en base) ; `creditorLine1`, `creditorLine2`, `creditorPostalCode`, `creditorTown`, `referenceValue`, `amount`, `unstructuredMessage` et `billingInformation` l'étaient déjà. Un client qui lit les sept premiers comme des chaînes casse. « Le document porte un QR » se lit `creditorIban !== null`.

**`POST /api/v1/imported-supplier-invoices/{id}/discard`** — écriture (`read-write`), sans corps. **Retire le document de la liste** (`to_complete` → `discarded`), `204`. Le fichier reste conservé ; le redéposer réactive la même ligne. Refus : `404 IMPORTED_INVOICE_NOT_FOUND` (inconnu ou d'une autre société), `409 IMPORT_NOT_PENDING_COMPLETION` (`details.currentStatus`). Trace `imported_supplier_invoice.discarded`, avec `iban_present` (jamais l'IBAN).

**`GET /api/v1/imported-supplier-invoices/{id}/source-document`** — lecture. Rend les octets du document, `Content-Type` = type dérivé de l'extension à l'import, `Content-Disposition: attachment` (nom d'origine, RFC 5987). `404 SOURCE_DOCUMENT_NOT_FOUND` si le document est inconnu ou d'une autre société ; **`410 SOURCE_DOCUMENT_GONE`** si la ligne existe mais que le fichier manque. *Depuis la v0.15.0*, un `.keshbackup` de format 2 emporte les documents et les restaure ([#503](https://github.com/guycorbaz/kesh/issues/503)) : le 410 frappe encore une restauration d'une sauvegarde de format 1 (0.14.x et avant) ou d'un dump de base, faite sans le dossier des documents, **et** toute pièce qu'une sauvegarde de format 2 n'a pas emportée — absente, altérée, illisible ou incohérente au moment de l'export, listée au manifeste sous `filesNotIncluded` et comptée par l'import dans `filesMissing`.

**`POST /api/v1/imported-supplier-invoices/{id}/complete`** — écriture (`read-write`) : crée la facture fournisseur du document, dans la même transaction que son passage `completed`. ⚠️ *Depuis la v0.14.0*, son corps est **celui de `POST /api/v1/supplier-invoices`** (`contactId`, `supplierInvoiceNumber`, `invoiceDate`, `dueDate`, `creditorIban`, `creditorQrIban`, `paymentReference`, `expectedPaymentAmount`, `projectId`, `lines`) — le projet est désormais honoré ; un client qui n'envoie pas les champs ajoutés reste accepté. La réponse (`200`) est la facture **relue comme par `GET /supplier-invoices/{id}`**, champs de lecture calculés.

- Document **avec** QR : IBAN ou QR-IBAN, référence et montant attendu viennent du QR. Un corps qui porte l'un de ces quatre champs (non vide) est refusé en **`400 QR_FIELDS_FROM_DOCUMENT`** — décidé sur la ligne du document verrouillée. Restent les refus `CURRENCY_NOT_SUPPORTED` (seul le CHF est admis), `IBAN_REFERENCE_MISMATCH`, puis, *depuis la v0.17.0*, les refus de forme des lignes — `400 VALIDATION_ERROR` (quantité ou prix à plus de quatre décimales), `409 ILLEGAL_STATE_TRANSITION` (quantité, prix ou produit non strictement positif), `400 SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES` (ligne qui n'est pas au centime, ci-dessous) — **avant** `AMOUNT_MISMATCH` (le total TTC des lignes doit égaler exactement le montant du QR), en `400`.
- Document **sans** QR : ni contrôle de devise (la facture est en CHF), ni contrôle de montant ; IBAN, QR-IBAN, référence et montant attendu se prennent dans le corps et se **valident comme pour `POST /supplier-invoices`** (`INVALID_PAYMENT_DETAILS`, ci-dessous).
- Communs aux deux : `404 IMPORTED_INVOICE_NOT_FOUND`, `409 IMPORT_NOT_PENDING_COMPLETION`, et les refus de `POST /supplier-invoices` (compte non imputable, exercice…).

**Les coordonnées de paiement se valident à un seul endroit.** ⚠️ *Depuis la v0.14.0*, `POST /api/v1/supplier-invoices` et la complétion d'un document sans QR refusent en **`400 INVALID_PAYMENT_DETAILS`**, `details: { field, reason }` — `field` ∈ `creditorIban`, `creditorQrIban`, `paymentReference` ; `reason` est l'un de **sept codes** :

| `reason` | `field` | Cas |
|---|---|---|
| `INVALID_IBAN` | `creditorIban` ou `creditorQrIban` | format, code pays absent de la table intégrée de Kesh, longueur pour ce pays ou somme de contrôle MOD-97 invalide |
| `QR_IBAN_IN_IBAN_FIELD` | `creditorIban` | un QR-IBAN (IBAN CH/LI dont le QR-IID est dans 30000-31999) dans `creditorIban` |
| `NOT_A_QR_IBAN` | `creditorQrIban` | un IBAN valide, mais hors de la plage QR |
| `BOTH_IBANS` | `creditorIban` | `creditorIban` et `creditorQrIban` renseignés ensemble |
| `QRR_REQUIRED` | `paymentReference` | un QR-IBAN sans référence |
| `INVALID_QRR` | `paymentReference` | avec un QR-IBAN, une référence qui n'est pas une QRR valide (27 chiffres, somme modulo 10 récursive) |
| `REFERENCE_TOO_LONG` | `paymentReference` | avec un IBAN ordinaire ou sans IBAN, une référence de plus de 64 caractères |

Sont **acceptés** : un IBAN étranger valide d'un pays de la table ; toute référence libre de 64 caractères au plus avec un IBAN ordinaire ou sans IBAN, **27 chiffres compris** (ancienne référence BVR/ESR — jamais de refus « forme QRR »). Les blancs — espace, tabulation, insécables U+00A0 et U+202F, U+0085, U+200B, U+FEFF… — sont retirés des IBAN et de la QRR ; un champ vide ou blanc vaut absent. Les valeurs sont **enregistrées normalisées** (IBAN sans blancs en majuscules, QRR sans blancs, référence libre rognée des mêmes blancs, sa longueur comptée en caractères **après** ce rognage). Les factures existantes ne changent pas. ⚠️ **`expectedPaymentAmount` n'est pas validé** — ni en signe, ni en échelle, ni contre le total des lignes : un client d'API qui le renseigne en répond (angle mort écrit de la v0.14.0, comme avant elle). ⚠️ Un IBAN valide d'un pays **absent de la table** (par exemple `CI`, `BF`, `BJ`, `ML`, `NE`, `TG`, `MG`), accepté avant la v0.14.0, est désormais refusé ; et un IBAN étranger accepté reste refusé à la création d'un lot pain.001 (`POST /payment-batches`, `failed[]` `INVALID_IBAN`), qui n'admet que des IBAN suisses ou liechtensteinois.

**Les lignes d'une facture fournisseur se saisissent au centime.** ⚠️ *Depuis la v0.17.0* ([#687](https://github.com/guycorbaz/kesh/issues/687)), `POST /api/v1/supplier-invoices` et la complétion d'un document refusent une ligne dont le montant (`quantity × unitPrice`) n'est pas au centime, et une quantité ou un prix à plus de quatre décimales :

| Code | Statut | Cas |
|---|---|---|
| `SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES` | `400` | `quantity × unitPrice` a plus de deux décimales significatives ; `details: { line, lineTotal }` — `line` le rang de la ligne (à partir de 1), `lineTotal` le produit sans zéros de fin (`"99.9999"`). La première ligne fautive l'emporte. |
| `VALIDATION_ERROR` | `400` | `quantity` ou `unitPrice` à plus de quatre décimales significatives (zéros de fin ignorés) ; le message nomme la ligne et le champ. Jugé **avant** les coordonnées de paiement. |

C'est un **changement de contrat** : un client d'API qui envoyait des lignes sous le centime — acceptées jusqu'à la v0.16.x, et enregistrées avec un total à plus de deux décimales — reçoit désormais un `400`. Un produit non strictement positif reste refusé en `409 ILLEGAL_STATE_TRANSITION`, jugé avant le centime. Les factures existantes ne sont pas modifiées : celles dont le total est sous le centime restent lisibles, payables et annulables.

**`GET /api/v1/imported-supplier-invoices/{id}/supplier-candidates`** — lecture, *depuis la v0.14.0*. Les fournisseurs **actifs** de la société dont une facture fournisseur **non annulée** porte l'IBAN ou le QR-IBAN du créancier du document (comparaison sans espaces ni insécables, sans casse), la facture la plus récente d'abord, puis par `id` ; **trois au plus**. L'IBAN est lu sur le document : il n'est ni dans l'URL ni dans la réponse. Réponse `200` : `{ candidates: Contact[] }` (même forme que `GET /contacts/{id}`), liste vide pour un document sans QR ; `404 IMPORTED_INVOICE_NOT_FOUND` pour un document inconnu ou d'une autre société. Une **proposition** : Kesh ne rattache rien de lui-même, c'est au client de choisir.

### Le document d'une facture fournisseur

*(Depuis la v0.14.0, [#685](https://github.com/guycorbaz/kesh/issues/685).)* Une facture fournisseur a **au plus un** document. Il naît de la complétion d'un document à traiter, ou se joint après coup ; il se détache.

**`sourceDocument`, à trois états.** La facture relue — `GET /supplier-invoices/{id}`, les réponses de `pay`, `cancel`, `settlement/cancel` (dans `invoice`), de la complétion d'un document et des deux routes ci-dessous — porte `sourceDocument` : `{ originalFilename, mimeType, byteSize }` si la facture a un document, **`null`** si elle n'en a pas. Dans la réponse de **`POST /supplier-invoices`** (création), le champ est **absent** : *non calculé*, jamais « sans document ». Un client lit donc l'absence du champ comme « inconnu », et seul `null` comme « aucun document ».

**`GET /api/v1/supplier-invoices/{id}/source-document`** — lecture. Rend les octets du document (mêmes en-têtes que la lecture d'un document à traiter, ci-dessus). ⚠️ *Depuis la v0.14.0*, **ouverte à tout rôle authentifié** (Consultation compris) et donc aux clés `read` de tout créateur, comme la fiche de la facture ; la lecture d'un document **à traiter** (`GET /imported-supplier-invoices/{id}/source-document`) reste réservée au rôle Comptable. `404 SOURCE_DOCUMENT_NOT_FOUND` pour une facture inconnue, d'une autre société, ou **sans document** ; `410 SOURCE_DOCUMENT_GONE` si le fichier manque (restauration d'une sauvegarde de format 1 ou d'un dump sans le dossier des documents, ou pièce qu'une sauvegarde de format 2 n'a pas emportée — `filesNotIncluded`, #503).

**`POST /api/v1/supplier-invoices/{id}/source-document`** — écriture (`read-write`), rôle Comptable. Corps : `{ "importedSupplierInvoiceId": <id> }`. Joint un document **à traiter** à une facture existante, quel que soit son statut (ouverte, payée, annulée) ; aucun champ de la facture n'est modifié, aucun montant n'est comparé. Le document passe `completed` et quitte la liste ; trace `imported_supplier_invoice.attached` (`supplier_invoice_id`, `original_filename`). Réponse `200` : la facture relue, `sourceDocument` renseigné. Refus : `404 IMPORTED_INVOICE_NOT_FOUND` (document inconnu ou d'une autre société), `404 NOT_FOUND` (facture inconnue ou d'une autre société), `409 IMPORT_NOT_PENDING_COMPLETION` (document déjà traité ou retiré, `details.currentStatus`), **`409 SOURCE_DOCUMENT_ALREADY_ATTACHED`** *(neuf)* — la facture a déjà un document : détachez-le d'abord. ⚠️ La demande n'est pas idempotente : renvoyée après un succès dont la réponse s'est perdue (coupure réseau, ou erreur entre la validation et la relecture de la facture), elle rend `409 IMPORT_NOT_PENDING_COMPLETION` alors que le document est bien joint — relisez la facture (`GET`) avant de conclure à un échec.

**`DELETE /api/v1/supplier-invoices/{id}/source-document`** — écriture (`read-write`), rôle Comptable, sans corps. Détache le document : il redevient **à traiter** (avec ses données de QR) et revient dans la liste ; la facture, ses lignes et ses écritures ne changent pas, aucun fichier n'est supprimé. Tout statut de facture. Trace `imported_supplier_invoice.detached`. Réponse `200` : la facture relue, `sourceDocument: null`. Refus : `404 SOURCE_DOCUMENT_NOT_FOUND` (facture inconnue, d'une autre société ou sans document) — c'est aussi ce que rend un détachement renvoyé après un succès dont la réponse s'est perdue : relisez la facture avant de conclure. ⚠️ Un document **avec QR** détaché peut être complété une seconde fois : Kesh ne signale pas la première facture.

**Le détail d'une écriture** (`GET /journal-entries/{id}`) porte `sourceDocument` : `{ supplierInvoiceId, supplierInvoiceNumber, originalFilename }` si l'écriture est l'**achat** ou le **règlement** d'une facture fournisseur qui a un document, `null` sinon (le champ est toujours présent sur le détail, absent de la liste). Le document se lit alors par `GET /supplier-invoices/{supplierInvoiceId}/source-document`. Une écriture de règlement que la facture ne porte plus rend `null` : règlement **annulé**, ou facture **payée puis annulée** (l'annulation détache le règlement) ; l'écriture d'achat d'une facture annulée garde son document.

### Notes du tableau des ressources

Renvois ¹, ² et ³ du tableau en tête de ce § 7.

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
| **Journal d'audit consultable dans l'interface web seulement** | Les trois routes `/api/v1/audit-log*` refusent toute clé, `read` comprise (`403 API_KEY_MANAGEMENT_FORBIDDEN`, cf. §4). | — |
| **Suppression d'un import bancaire réservée à l'interface web** | `DELETE /bank-imports/{id}` refuse toute clé : `403 API_KEY_OPERATION_FORBIDDEN` pour une clé `read-write`, `403 API_KEY_READ_ONLY` pour une clé `read` (cf. §4). *(Depuis la v0.13.2.)* | [#632](https://github.com/guycorbaz/kesh/issues/632) |
| **Pas de spécification OpenAPI** | Aucun schéma OpenAPI/Swagger n'est publié en v0.2 (la base de code n'embarque pas `utoipa`). Documentez vos appels à partir de ce guide. | v0.3 |

> ✅ **Corrigé — auto-propagation des clés Administrateur** ([KF-036 / #167](https://github.com/guycorbaz/kesh/issues/167)). Une clé `read-write` créée par un Administrateur atteignait auparavant les routes réservées aux Administrateurs : elle pouvait donc créer un compte administrateur, ce qui rendait la révocation de la clé inopérante. Ce n'est plus le cas. **Si une intégration existante appelait ces routes, elle reçoit désormais `403 API_KEY_ADMIN_FORBIDDEN`.**
>
> ⚠️ **Dans quelle version ?** Cette fermeture est livrée depuis la **v0.10.0** (entrée [#167](https://github.com/guycorbaz/kesh/issues/167) du [CHANGELOG](../CHANGELOG.md)). Jusqu'à la v0.9.0 incluse, une clé `read-write` créée par un Administrateur atteint les routes d'administration : sur une telle version, traitez une clé d'origine Administrateur comme un secret d'administrateur, et mettez Kesh à jour.

Hors périmètre (non planifié pour v0.2) : OAuth/SSO, webhooks (cf. `epic-17.md` — « Hors scope »).

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
| `403` | `API_KEY_OPERATION_FORBIDDEN` | Opération réservée à l'interface web, atteinte avec une clé `read-write` ; aujourd'hui, la suppression d'un import bancaire (`DELETE /bank-imports/{id}`). Voir §4. *(Depuis la v0.13.2.)* |
| `400` | `VALIDATION_ERROR` | Corps de requête invalide (champ manquant, valeur hors limites, …). |
| `400` | `INACTIVE_OR_INVALID_ACCOUNTS` | Un compte référencé est inconnu, archivé, d'une autre company ou d'un mauvais type (par exemple un compte de charge qui n'est pas de charge) ; anti-énumération : le compte n'est pas nommé. |
| `400` | `ACCOUNT_NOT_POSTABLE` | Un compte de la company, **actif**, n'est **pas imputable** — compte de regroupement, de résultat ou de clôture. `details.rejected[{accountId, accountNumber}]` nomme chaque compte refusé, triés par numéro. Rendu par `POST /journal-entries`, `PUT /journal-entries/{id}` (depuis la v0.13.0 — un compte devenu non imputable est refusé même sur une ligne inchangée), `POST /opening-balances`, `POST /invoices/{id}/settlements` (compte interne), `POST /supplier-invoices` (compte de charge), `POST /imported-supplier-invoices/{id}/complete` (compte de charge), `POST /supplier-invoices/{id}/pay` (compte interne), `POST /reconciliation/manual` et `POST /reconciliation/split` (compte de contrepartie), `POST /reconciliation/rules` et `PATCH /reconciliation/rules/{id}` (compte de contrepartie — au `PATCH`, seulement si le compte change ou si la règle est réactivée), `POST /bank-accounts`, `PUT /bank-accounts/{id}` et `PATCH /bank-accounts/{id}` (compte comptable lié — au `PUT` et au `PATCH`, seulement s'il change). Rendu aussi, avec le même détail, par `POST /invoices/{id}/validate` (créance, ou TVA due si la facture en porte), `POST /supplier-invoices` et `POST /imported-supplier-invoices/{id}/complete` (compte créanciers, ou TVA récupérable si la facture en porte) quand un compte **désigné dans les réglages de facturation** est devenu non imputable depuis sa désignation : le message renvoie alors à *Administration → Facturation*, où un administrateur doit le remplacer (depuis la v0.13.0 ; ce refus vient après tous les autres, exercice compris). Dans `failed[]` de `POST /reconciliation/accept`, le même code et le même détail pour une proposition `split` ou `rule`. Un compte à la fois inconnu, archivé ou d'un mauvais type **et** non imputable rend `INACTIVE_OR_INVALID_ACCOUNTS`, et un compte archivé ou invalide sur une autre ligne de la même écriture prime lui aussi sur un compte non imputable. *(Depuis la v0.13.0 ; ces refus rendaient auparavant `INACTIVE_OR_INVALID_ACCOUNTS`.)* |
| `400` | `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT` | `POST /bank-accounts`, `PUT /bank-accounts/{id}` et `PATCH /bank-accounts/{id}` (lien) : le compte comptable lié est le **compte débiteurs** ou le **compte créanciers** désigné dans les réglages de facturation — chaque encaissement ou paiement par ce compte bancaire écrirait une écriture nulle. `details` : `accountId`, `accountNumber`, `claim` (`receivable` \| `payable`). Au `PUT` et au `PATCH`, seulement si le compte lié **change** : un compte bancaire déjà lié ainsi reste modifiable dans ses autres champs. Vient après le `409` de version et après `ACCOUNT_NOT_POSTABLE`. *(Depuis la v0.13.0.)* |
| `400` | `INVALID_PAYMENT_DETAILS` | `POST /supplier-invoices` et `POST /imported-supplier-invoices/{id}/complete` (document sans QR) : IBAN, QR-IBAN ou référence refusés. `details` : `field` (`creditorIban` \| `creditorQrIban` \| `paymentReference`) et `reason` (sept codes — § *Documents à traiter*). *(Depuis la v0.14.0.)* |
| `400` | `QR_FIELDS_FROM_DOCUMENT` | `POST /imported-supplier-invoices/{id}/complete` : le document porte un QR, et le corps porte IBAN, QR-IBAN, référence ou montant attendu — ils viennent du QR. *(Depuis la v0.14.0.)* |
| `400` | `LATER_FISCAL_YEAR_CLOSED` | `PUT` et `DELETE /journal-entries/{id}` : un exercice **postérieur** à celui de l'écriture est clôturé ; son bilan, cumulatif, reprend l'écriture. `POST /fiscal-years` : un exercice postérieur à la date de début demandée est clôturé — un exercice ne se crée pas avant un exercice clôturé (message propre à la création). `details.fiscalYearId` / `details.fiscalYearName` nomment le plus proche. Depuis la Story 15-12b (#543), aussi **toute création** d'écriture dans un tel exercice — `POST /journal-entries`, contre-passation, validation, règlement et dévalidation d'une facture, factures fournisseur, soldes de départ, rapprochements et leurs annulations — et, dans `failed[]` de `POST /reconciliation/accept`, la proposition concernée (`200`). ⚠️ Angles morts assumés : les champs qui annoncent un geste possible ne le prédisent pas — `cancellable` / `cancelBlockedBy` / `settlementCancelBlockedBy` des annulations ([#568](https://github.com/guycorbaz/kesh/issues/568)) et `canComplete` / `completeReason` de `GET /opening-balances/status` ; le refus vient alors au `POST`. *(Depuis la v0.13.0.)* |
| `409` | `EARLIER_FISCAL_YEAR_OPEN` | `POST /fiscal-years/{id}/close` : un exercice **antérieur** est encore ouvert — les exercices se clôturent dans l'ordre, le bilan étant cumulatif. `details.fiscalYearId` / `details.fiscalYearName` nomment le **plus ancien** antérieur ouvert, à clôturer d'abord. ⚠️ **Changement de contrat** : une intégration qui clôturait hors d'ordre reçoit désormais ce refus. Un exercice déjà clos rend `409 ILLEGAL_STATE_TRANSITION`, qui parle d'abord. *(Depuis la v0.13.0.)* |
| `409` | `NEXT_FISCAL_YEAR_MISSING` | `POST /journal-entries/{id}/reverse-at-next-fiscal-year-start` : aucun exercice ne suit celui de l'écriture — créez l'exercice suivant, puis recommencez. `details.fiscalYearName` nomme l'exercice **de l'écriture**. *(Depuis la v0.14.0.)* |
| `409` | `NEXT_FISCAL_YEAR_CLOSED` | `POST /journal-entries/{id}/reverse-at-next-fiscal-year-start` : l'exercice qui suit celui de l'écriture est clôturé — la contre-passation ne peut pas y être datée. `details.fiscalYearId` / `details.fiscalYearName` le nomment. *(Depuis la v0.14.0.)* |
| `409` | `ENTRY_LETTERED` | `PUT` et `DELETE /journal-entries/{id}` : une ligne de l'écriture est lettrée — délettrez-la d'abord (`DELETE /letterings/{key}`). `details.letteringCode` porte le code du premier groupe (ni `documentId` ni `documentNumber`). Rendu en dernier : quand il parle, le délettrage suffit. *(Depuis la v0.13.0.)* |
| `409` | `DETACHED_SUPPLIER_SETTLEMENT` | `PUT` et `DELETE /journal-entries/{id}` : l'écriture est le paiement d'une facture fournisseur annulée — une sortie de banque réelle, qui se corrige par contre-passation. `details.documentId` est l'identifiant de la facture. *(Depuis la v0.13.0.)* |
| `409` | `BANK_ACCOUNT_CLOSED` | Écriture qui vise un compte bancaire **clos** (liste des routes : note ⁹ du §7) ; dans `failed[]`, en `200`, pour `POST /reconciliation/accept` et `/reject`. `details.bankAccountId`. Rouvrir d'abord le compte (`POST /bank-accounts/{id}/reopen`). Rendu aussi par `POST /bank-accounts/{id}/close` sur un compte déjà clos. *(Depuis la v0.13.3.)* |
| `409` | `BANK_ACCOUNT_NOT_CLOSED` | `POST /bank-accounts/{id}/reopen` : le compte n'est pas clos. *(Depuis la v0.13.3.)* |
| `409` | `RECONCILIATION_SETTLEMENT_NOT_LINKABLE` | `POST /reconciliation/transactions/{id}/link` : le règlement n'est pas rattachable à la transaction — transaction qui n'est pas un crédit en CHF, règlement qui n'est pas un virement sur ce compte bancaire (autre compte bancaire, compte interne, solde), montant qui ne correspond pas, écriture déjà rapprochée ou passée sur un ancien compte du grand livre ; un seul code pour toutes ces causes. Relire `GET …/link-candidates`. `details.bankTransactionId`, `details.settlementId`. *(Depuis la v0.13.4.)* |
| `409` | `RECONCILIATION_ENTRY_NOT_LINKABLE` | `POST /reconciliation/transactions/{id}/link-entry` : l'écriture n'est pas rattachable à la transaction — transaction hors CHF ou de montant nul, mouvement de l'écriture sur le compte du grand livre de la banque différent du montant, écriture déjà rapprochée, contre-passée ou contre-passation, règlement client par virement sur ce compte bancaire (à rattacher par `…/link`) ; un seul code pour toutes ces causes. Relire `GET …/entry-candidates`. `details.bankTransactionId`, `details.journalEntryId`. *(Depuis la v0.17.0.)* |
| `412` | `BANK_ACCOUNT_CANNOT_CLOSE_PRIMARY` | `POST /bank-accounts/{id}/close` : le compte est le compte **principal** (celui de la QR-facture) — en désigner d'abord un autre. *(Depuis la v0.13.3.)* |
| `412` | `BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS` | `POST /bank-accounts/{id}/close` : le compte porte des transactions importées non rapprochées, rejetées comprises. `details.pendingCount`. *(Depuis la v0.13.3.)* |
| `412` | `BANK_ACCOUNT_HAS_OPEN_PAYMENT_BATCHES` | `POST /bank-accounts/{id}/close` : un lot de paiement généré sur le compte n'est ni confirmé ni annulé. `details.batchCount`. *(Depuis la v0.13.3.)* |
| `400` | `LETTERING_TOO_FEW_LINES`, `LETTERING_TOO_MANY_LINES` | `POST /letterings` : moins de deux lignes distinctes, ou plus de 200. *(Depuis la v0.13.0.)* |
| `409` | `LETTERING_ACCOUNTS_DIFFER`, `LETTERING_ACCOUNT_NOT_LETTERABLE`, `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, `LETTERING_LINE_OWNED_BY_DOCUMENT`, `LETTERING_LINE_ALREADY_LETTERED`, `LETTERING_UNBALANCED`, `LETTERING_IS_DOCUMENT`, `LETTERING_CONCURRENT_CHANGE` | `POST` et `DELETE /letterings` : refus du lettrage — voir « Lettrer des lignes ». `LETTERING_ACCOUNT_NOT_LETTERABLE` est aussi le refus de `GET /accounts/{id}/open-items` et `GET /accounts/{id}/lettering-proposals` sur un compte non lettrable. `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` est aussi le refus de l'annulation d'un règlement, d'un solde, d'un rapprochement, d'un paiement ou d'une facture fournisseur dont le lettrage est figé par une période close (§ des annulations) ; le texte rendu diffère selon la route — celui du lettrage, ou celui de la famille d'annulation. *(Depuis la v0.13.0.)* |
| `422` | `LETTERING_PROPOSALS_TOO_MANY_LINES` | `GET /accounts/{id}/lettering-proposals` : plus de 2 000 lignes candidates (ouvertes et lettrables à la main) sur le compte — Kesh ne propose rien plutôt qu'une liste tronquée. Lettrer d'abord une partie à la main (`POST /letterings`). *(Depuis la v0.13.0.)* |
| `409` | `BANK_IMPORT_PREVIEW_STALE` | `POST /bank-imports` : l'empreinte d'aperçu envoyée (`previewFingerprint`) ne correspond plus au profil, au compte ou au fichier — refaire l'aperçu. Voir « Importer un relevé — l'empreinte d'aperçu » (§7). *(Depuis la v0.13.2.)* |
| `404` | `NOT_FOUND` | Ressource absente ou appartenant à une autre company (anti-énumération). Certaines ressources renvoient un code spécifique (ex. `ACCOUNT_NOT_FOUND`). |

**Interblocages : les écritures au journal sont rejouées.** Toute route qui écrit au journal comptable rejoue d'elle-même un interblocage transitoire avec une autre opération, sans le montrer. Ouvertes aux clés `read-write`, ce sont : `POST /journal-entries`, `PUT` et `DELETE /journal-entries/{id}`, `POST /journal-entries/{id}/reverse`, `POST /journal-entries/{id}/reverse-at-next-fiscal-year-start` (depuis la v0.14.0), `POST /opening-balances`, `POST /opening-balances/complete`, `POST /invoices/{id}/validate`, `POST /invoices/{id}/unvalidate`, `POST /invoices/{id}/settlements`, `POST /invoices/{id}/settlements/{settlementId}/cancel`, `POST /invoices/{id}/write-off`, `POST /credit-notes`, `POST /supplier-invoices`, `POST /supplier-invoices/{id}/pay`, `POST /supplier-invoices/{id}/cancel`, `POST /supplier-invoices/{id}/settlement/cancel`, `POST /imported-supplier-invoices/{id}/complete`, `POST /payment-batches/{id}/confirm`, `POST /reconciliation/accept`, `POST /reconciliation/manual`, `POST /reconciliation/split`, `POST /reconciliation/transactions/{id}/cancel`, `POST /letterings` et `DELETE /letterings/{key}` — ainsi que la création et la clôture d'un exercice, `POST /fiscal-years` et `POST /fiscal-years/{id}/close`, qui n'écrivent pas au journal mais forment des interblocages avec la contre-passation (depuis la v0.13.0), et le rattachement d'une transaction à un règlement, `POST /reconciliation/transactions/{id}/link`, qui n'écrit pas au journal mais verrouille le règlement et son écriture comme leur annulation (depuis la v0.13.4), et le rattachement d'une transaction à une écriture, `POST /reconciliation/transactions/{id}/link-entry`, qui n'écrit pas au journal mais verrouille l'écriture comme sa contre-passation (depuis la v0.17.0) (la restauration d'une sauvegarde, réservée à l'interface d'administration, ne rejoue pas ; l'effacement des données de démonstration, réservé lui aussi à l'interface d'administration, rejoue depuis la v0.13.0). *(Depuis la v0.13.4.)* Rejouent aussi les routes du compte bancaire, qui n'écrivent pas au journal mais peuvent former un interblocage avec un import, un lot de paiement ou une annulation en cours sur le même compte : `POST /bank-accounts`, `PUT`, `PATCH` et `DELETE /bank-accounts/{id}`, `POST /bank-accounts/{id}/close` et `POST /bank-accounts/{id}/reopen` — ainsi que la suppression d'un import, `DELETE /bank-imports/{id}` (depuis la v0.13.2), réservée à l'interface web. *Depuis la v0.14.0*, la création d'un lot de paiement, `POST /payment-batches`, rejoue elle aussi : elle n'écrit pas au journal, mais peut former un interblocage avec une autre opération sur les mêmes factures ou le même compte. Rejouent de même le rattachement et le détachement du document d'une facture fournisseur, `POST` et `DELETE /supplier-invoices/{id}/source-document` *(depuis la v0.14.0)* : ils n'écrivent pas au journal, mais le rattachement verrouille le document, la facture puis la société, et peut former un interblocage avec toute opération qui règle plusieurs factures fournisseur dans une même transaction — l'acceptation d'un lot de rapprochement, la confirmation d'un lot de paiement. L'import d'un relevé (`POST /bank-imports`) n'est pas rejoué (voir le paragraphe suivant). Si l'interblocage persiste après trois tentatives, la requête finit en `500 INTERNAL_ERROR` ; rien n'a été écrit, et elle peut être renvoyée telle quelle.

Une autre route peut, rarement, rendre `500` sur un conflit transitoire d'accès concurrent ; rien n'est alors écrit et la requête peut être renvoyée.

---

## Voir aussi

- Manuel d'administration : section « Sécurité → Clés API (PAT) ».
- Issue d'origine : [#100](https://github.com/guycorbaz/kesh/issues/100).
