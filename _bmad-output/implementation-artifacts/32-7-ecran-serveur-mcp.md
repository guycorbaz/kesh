# Story 32.7 : l'écran « Serveur MCP » (Administration) et la section du manuel

Status: ready-for-dev

*Fiche rédigée le 2026-10-25, en autonomie (Opus 5.5), worktree `kesh-32-7`, branche `story/32-7` partie de `main`
`966baed5` (toute la 32-5 fusionnée ; 32-6, 32-8, 32-9 et 32-10a non fusionnées). Registre :
`epic-32-choix-autonomes-327.md` (C-32-7-1 à C-32-7-20). Clé courte : `327`. Issue de l'epic : #723 (CR « Serveur MCP
intégré à Kesh »), en `refs` seulement. Les numéros de ligne cités sont ceux de `966baed5`, sauf mention d'une fiche
voisine (lue dans son worktree, en lecture seule, à la date de rédaction).*

⛔ **Développement APRÈS la fusion des 32-6a, 32-6b, 32-8 (a, b0, b, c), 32-9a, 32-9b et 32-10a** (C-32-7-3). L'écran
seul ne dépend que de la 32-5 ; mais la section du manuel décrit **ce que fait le code** de ces stories — le comportement
de `/mcp`, le journal des appels, la validation avant envoi, les outils et ce que chaque portée démasque, la portée
`prompts:write` —, et elle **range** sous elle les sous-sections que la 32-8 (`sec:mcp-envois`) et la 32-9
(`sec:mcp-outils`) auront écrites. **Tout ce qui, dans cette fiche, dépend de ces stories se recontrôle à l'étape 0**
(§ *À recontrôler à l'étape 0*), ligne lue citée au Dev Agent Record ; une phrase prescrite ici que le code ne confirme
pas **se réécrit selon le code**, elle ne se garde pas.

⚠️ **Ordre de rechange, si l'orchestrateur veut l'écran plus tôt** (C-32-7-3) : les AC sont rangés en **trois groupes**
— A (l'écran, AC-1 à AC-17), B (le manuel et les textes, AC-18 à AC-24), C (les gardes de cohérence, AC-25). Le
groupe A se développe sur la **seule 32-5** (les six routes existent, fait 1), **à trois exceptions près**, nommées :
la seconde moitié de l'E2E (AC-16 (c), qui présente la clé à `/mcp` : 32-6a), le lien vers *Envois du serveur MCP*
(AC-13 (d) : 32-8c) et la vérification au code des textes de portée (AC-10 : 32-9, 32-10a) — qui passent alors au
groupe B. Le découpage serait mécanique : `32-7a` (groupe A, sans les trois exceptions) puis `32-7b` (groupe B, C et
les trois exceptions). **Il n'est pas retenu par défaut** : quatre modules en tout, et un écran publié avant que le
manuel le décrive ferait une fusion où l'écran parle d'un serveur que `main` n'a pas encore.

## Story

En tant que Guy, Administrateur d'une installation qui tiendra des livres réels, je veux, dans **Administration →
Serveur MCP**, activer ou désactiver le serveur, régler son filtre d'adresse, et créer, voir **une seule fois**,
révoquer et régler les **clés MCP** de mes agents — avec, écrit à l'écran et au manuel, **ce que ce filtre ne garantit
pas** —, afin d'ouvrir ce canal en sachant exactement ce qu'il expose, et de pouvoir le refermer d'un geste.

## Origine

- **Kickoff** (`planning-artifacts/epic-32-serveur-mcp.md` § *32-7*, l. 405-429) : activation (désactivé par défaut),
  filtre d'adresse et mandataires de confiance, clés MCP (création avec portées, affichage unique, révocation,
  expiration, dernière utilisation), ce que le filtre ne garantit pas **à l'écran et au manuel** ; section *Le serveur
  MCP* du manuel d'administration (activer, créer une clé, le filtre, ce qui est journalisé) ; manuel utilisateur :
  rien encore (32-11). Modules annoncés : frontend, `kesh-i18n`, manuel, `kesh-api` « seulement si une route manque » —
  **aucune ne manque** (fait 1). Risque : un secret affiché deux fois, ou conservé dans l'état du composant.
- **Reports de la 32-5** (epic l. 419-429 ; fiche `32-5-cles-et-reglages-mcp.md` l. 59, 572-573, 768-774) : section
  du manuel ; `100.64.0.0/10` absente des défauts, à l'écran et au manuel (C-32-5-10) ; restauration d'un serveur
  activé (Q7, C-32-5-11) ; renvois depuis les passages de la 32-5d (admin l. 1414, 1416, 2467, 2472 ; utilisateur
  l. 190, 3204-3205, 3214-3215, annotation C-32-5-48) ; la clé donnée au client en portée `local`/`user`, jamais dans
  un `.mcp.json` versionné sauf `${VARIABLE}` (spike § 6).
- **#738** (LOW de la revue de la 32-5c tracés par l'orchestrateur, `gh issue view 738`) : **B7 = E6** (horodatages
  des clés en `NaiveDateTime` sans `Z`, des réglages en `DateTime<Utc>` avec `Z` — « l'écran de la 32-7 doit le
  savoir ») ; **B3 = E5** (`/0` accepté) ; **B2 = E4** (`010.0.0.0/8` stocké `10.0.0.0/8` sans signal) ; **B5 = E2**
  (corps mal formé : refus par défaut d'axum, non traduit) ; **B8** (`version: -1` au `PUT` des réglages → 409).
- **Fiches voisines** (inventaire complet au § *Inventaire des reports*) : 32-6 (l. 1002, 1018, 1231-1283, 1310, 1402,
  1469, 1558), 32-8 (l. 916-918, 1034, 1397, 1406-1408, 1507), 32-9 (l. 914, 1315-1325, 1339-1341, 1373, 1429),
  32-10a (l. 998-1008, 1386-1387) ; spike § 6 (l. 945-948).
- **Réponses de Guy (2026-10-24, soir et 2026-10-25)** : création des clés par l'**Administrateur** seul (Q3) ; les cinq
  portées (Q5) ; validation **par clé**, `required` par défaut (Q12) ; serveur **désactivé** par défaut ; filtre
  **désactivable**, et derrière Traefik ou la passerelle Docker **il ne filtre rien — l'écran et le manuel le disent** ;
  réglages **restaurés tels quels** (Q7) ; **pas de limite de débit** ; la sécurité relève de l'infrastructure — Kesh
  n'est jamais exposé sur Internet, l'accès externe passe par un VPN ; **Q11 (a)** — aucun second utilisateur MariaDB,
  donc rien à écrire au manuel sur une création à la main sur le NAS (spike l. 946-947, sans objet) ; **Q10 (c)** — la
  poignée, donc « au-delà de 2 min Claude Code poursuit sans le résultat » (spike l. 947-948) est **sans objet** : un
  outil rend aussitôt sa poignée.

## Ce que le code dit (relevé au code le 2026-10-25, `966baed5`)

1. **Les routes existent toutes** (`crates/kesh-api/src/lib.rs:321-340`) : `GET`/`POST /api/v1/admin/mcp/keys`,
   `DELETE /api/v1/admin/mcp/keys/{id}`, `PUT /api/v1/admin/mcp/keys/{id}/send-validation`, `GET`/`PUT
   /api/v1/admin/mcp/settings`, dans le bloc `admin_routes` (Administrateur, refus d'un PAT, une clé MCP non reconnue —
   `routes/mcp_keys.rs:15-21`). **Aucune route neuve** n'est donc demandée à `kesh-api`.
2. **La forme d'une clé** (`routes/mcp_keys.rs:56-73`) : `id`, `name`, `scopes` (connues, ordre canonique),
   `unknownScopes` (stockées, inconnues de la version, **jamais accordées**, l. 63-65), `sendValidation`
   (`required`|`exempt`), `createdByUserId`, `createdAt`, `lastUsedAt`, `revokedAt`, `expiresAt`, `version` — les
   quatre dates en **`NaiveDateTime`** (l. 68-71), sérialisées **sans fuseau** (`2026-10-25T10:00:00.123`), en UTC
   (#738 B7 = E6). La liste est triée `created_at DESC, id DESC` (`kesh-db/src/repositories/mcp_keys.rs:208-209`),
   révoquées comprises (`mcp_keys.rs:264`).
3. **La création** (`routes/mcp_keys.rs:93-116`, `270-338`) : corps `name`, `scopes` (au moins une, doublons
   normalisés), `expiresAt` facultatif (**RFC 3339 avec fuseau**, `DateTime<Utc>`, l. 101-102), `sendValidation`
   facultatif (absent → `required`, l. 277-280). Réponse **201** : la clé **aplatie** (`#[serde(flatten)]`, l. 112-113)
   **plus** `key`, le secret `kesh_mcp_…` (l. 114-115) — un seul objet porte donc les deux. Refus en 400 traduits : nom
   vide ou trop long (rogné des invisibles, l. 138-153), portée inconnue (citée bornée, l. 182-202), expiration passée
   ou au-delà du 9999-12-31 (l. 284-299). Audit `mcp_key.created` sans secret ni condensat (l. 314-327 ; testé,
   `tests/mcp_admin_routes_e2e.rs:803`).
4. **La révocation** (`routes/mcp_keys.rs:340-372`) : corps `{ "version": n }`, 204 ; 404 (absente ou d'une autre
   société) ; 409 (version périmée **ou déjà révoquée**). **La politique** (l. 374-411) : `PUT` `{ sendValidation,
   version }`, rend la clé relue ; 409 si révoquée (C-32-5-30). `version < 1` → 400 (l. 215-225).
5. **Les réglages** (`routes/mcp_server_settings.rs:63-102`) : `enabled`, `addressFilterEnabled`, `allowedNetworks`,
   `trustedProxies`, `allowedOrigins`, `version` (**0** = aucune ligne, défauts du code, l. 72-73), `updatedAt` en
   `DateTime<Utc>` **avec `Z`** (l. 74-75). `PUT` = remplacement complet ; `version: 0` crée la ligne, 409 si elle
   existe ; `version ≥ 1` modifie, 409 si périmée (l. 7-11, 230-232) ; `version: -1` → 409 aussi (#738 B8). Chaque
   entrée est **rognée** (l. 112, 141) puis mise en forme canonique : une adresse devient `/32` ou `/128` (l. 113-115) ;
   un réseau à bits d'hôte est refusé avec la forme attendue (l. 126-135) ; une entrée **vide** est refusée comme
   invalide (l. 116 : `"".parse::<IpNet>()` échoue) ; 64 entrées de 255 caractères au plus (l. 57-61, 175-188) ;
   doublons retirés (l. 190-192). Origines : `http(s)://hôte[:port]` sans chemin (l. 140-167).
6. **Les défauts** (`kesh-db/src/entities/mcp_server_settings.rs:17-40`) : désactivé ; filtre **activé** ; plages
   admises `127.0.0.0/8`, `::1/128`, `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, `fc00::/7`, `fe80::/10` —
   **sans** `100.64.0.0/10` (l. 24-25, C-32-5-10) ; aucun mandataire ; aucune origine.
7. **Les portées** (`kesh-db/src/entities/mcp_key.rs:33-63`) : `books:read`, `invoices:read`, `contacts:read`,
   `documents:read`, `prompts:write`, `ALL` dans cet ordre ; **politique** `required` (défaut) | `exempt` (l. 256-272).
8. **Le motif de l'écran le plus proche** : la page des clés d'API (`frontend/src/routes/(app)/settings/api-keys/
   +page.svelte`, 428 lignes) — liste, création inline, encart du secret, copie robuste hors HTTPS (`copyToClipboard`,
   `shared/utils/clipboard.ts`, le `textarea` de repli retiré en `finally`, l. 44-47), révocation inline. Le secret y
   **survit à la fermeture du formulaire et au rechargement de la liste** jusqu'au clic sur *Fermer* (l. 40-42 — c'est
   la même règle ici). Deux points à **ne pas** recopier : l'objet de création **entier**, secret compris, circule
   dans le code de la page (`created`, l. 154, d'où l'encart tire `name` et `key`, l. 155) — rien ne sépare le secret
   de la clé à la frontière de l'API ; et l'expiration est envoyée
   `${date}T23:59:59Z` (l. 150-153) puis affichée par `slice(0, 10)` d'une date naïve (l. 73-77), en UTC — la date
   choisie est la dernière journée **UTC**, non suisse.
9. **La garde de route Administrateur** : `frontend/src/routes/(app)/admin/backup/+page.ts:11-17` (`ssr = false`,
   redirection 302 vers `/` hors `Admin`) ; la page elle-même est mince (`+page.svelte`, un composant de feature).
10. **La navigation** : groupe *Administration*, `adminOnly` (`frontend/src/routes/(app)/+layout.svelte:147-170`), la
    dernière entrée étant `nav-admin-restore` (l. 166) ; l'ouverture automatique du groupe compare le chemin **à
    l'égalité** (l. 212-215).
11. **i18n** : l'appartenance d'une clé à une feature à nom composé se juge par le préfixe `${feature}-`
    (`frontend/scripts/lint-i18n-ownership.js:177-186`) — feature `mcp-server`, clés `mcp-server-*`. Les messages
    d'erreur du serveur (`admin-mcp-keys-error-*`, `admin-mcp-settings-error-*`, `crates/kesh-i18n/locales/fr-CH/
    messages.ftl:1755-1767`) arrivent **déjà traduits** dans `message`. `i18nMsg` remplace `{ $x }` après résolution
    (`shared/utils/i18n.svelte.ts:18-28`). Quatre catalogues (`crates/kesh-i18n/locales/{fr,de,it,en}-CH`).
12. **Les boutons rouges** : la variante `destructive` du `Button` partagé est la seule source admise du rouge ; un
    rouge posé à la main s'inscrit à la table fermée de `frontend/src/lib/components/ui/button/
    boutons-rouges-inventory.test.ts` (précédent : `api-keys/+page.svelte#api-keys-revoke-{k.id}`, l. 207).
13. **Les sélecteurs E2E** ne se figent jamais sur un libellé traduit (`frontend/src/lib/shared/
    e2e-selecteurs-traduits.test.ts`) ; la spec des clés d'API sert de motif (`frontend/tests/e2e/api-keys.spec.ts`,
    `login`, `seedTestState`).
14. **Rien ne journalise un corps de réponse** côté serveur : `grep -rn 'TraceLayer\|on_response\|make_span_with'
    crates/kesh-api/src/lib.rs crates/kesh-api/src/main.rs crates/kesh-api/src/middleware/` → **rien**. Côté
    navigateur, `errorMessageOf` journalise l'erreur reçue (`shared/utils/error-message.ts:48`) — jamais une réponse
    réussie ; `api-client.ts` ne journalise qu'aux l. 159-169 (renouvellement de session).
15. **Le masqueur de commentaires** réutilisable par une garde lexicale : `masquerCommentaires`
    (`frontend/src/lib/shared/i18n-literal-reader.js:261`), qui blanchit `//`, `/* */` et `<!-- -->` hors littéraux.
16. **Le manuel d'administration** (`docs/manual/fr/admin-manual.tex`, classe `article` : `\section` est le niveau
    le plus haut) : `\section{Sécurité}` l. 2862, `\subsection{Clés API (PAT)…}\label{sec:api-keys}` l. 2898, son renvoi
    à `docs/api-external.md` l. 2925, `\subsection{RBAC…}` l. 2927, le compte « 122 des 128 routes à verbe mutant »
    l. 2935, `\subsection{Sécurité réseau (firewall)}\label{sec:firewall}` l. 2990, `\section{Monitoring et logs}`
    l. 3017. Passages MCP de la 32-5 : l. 1414 (remise à zéro : clés MCP conservées, **réglages effacés, serveur
    désactivé**), 1416 (#528), 2230, 2237 (sauvegarde : clés et réglages **restaurés tels quels**, une clé révoquée
    depuis redevient active), 2249 (compatibilité), 2324, 2467 et 2472 (réparation). **Aucune** section ne décrit
    encore le serveur (`grep -n 'section.*MCP' docs/manual/fr/admin-manual.tex` → rien).
17. **Le manuel utilisateur** : l. 190, 3029-3030, 3204-3205, 3214-3215 nomment les clés du serveur MCP, sans renvoi.
18. **`docs/api-external.md`** : l. 29-35 (deux types de clé, réécrit par la 32-5c), l. 183-185 (§ 6.4, « pas de
    serveur MCP Kesh-natif en v0.2 »), l. 831 (« Hors périmètre (non planifié pour v0.2) : … serveur MCP
    Kesh-natif »).
19. **CHANGELOG** : `## [0.16.0] — Non publié` (l. 11), section `### Modifié` seule ; aucune `### Ajouté` encore à
    `966baed5` (les 32-8 et 32-9 en ajoutent une).
20. **Le garde des textes** : `crates/kesh-api/tests/textes_coherents.rs`, gardes G2 à G22 (l. 12-27) ; G22 (l. 1673)
    est le précédent d'une cohérence **manuel ↔ écran ↔ quatre locales**. Aucune fiche voisine ne réserve G23
    (`grep -n 'G23' kesh-32-*/…/32-*.md` → rien au 2026-10-25).

## Le découpage (règle de découpage préventif)

Modules, au grain du kickoff : **frontend** (`features/mcp-server/` neuf, `routes/(app)/admin/mcp/` neuf,
`+layout.svelte`, `tests/e2e/mcp-server.spec.ts`), **`kesh-i18n`** (quatre catalogues), **docs** (`admin-manual`,
`user-manual`, leurs PDF, `CHANGELOG.md`), **`kesh-api`** (le seul fichier de test `tests/textes_coherents.rs`, garde
G23, AC-25) — **4**, au plus cinq : **pas de découpage** (C-32-7-1). Aucune migration, aucun dépôt, `kesh-db` non
touché : l'exception `kesh-db` de *Pendant une boucle de revue* ne s'applique pas. L'ordre de rechange (deux clés) est
décrit en tête de fiche.

## Critères d'acceptation

Conventions : feature `frontend/src/lib/features/mcp-server/` — `mcp-server.api.ts`, `mcp-server.types.ts`,
`mcp-time.ts`, `secret-unique.svelte.ts`, `McpServerPanel.svelte`, `McpSettingsSection.svelte`,
`McpKeysSection.svelte`, `McpKeySecretNotice.svelte` (noms indicatifs ; la répartition en composants est libre, **les
propriétés ne le sont pas**) ; tests Vitest à côté (`*.test.ts`) ; route `frontend/src/routes/(app)/admin/mcp/`. Tous
les libellés par `i18nMsg('mcp-server-…', '<repli fr-CH>')`. Tous les `data-testid` préfixés `mcp-server-`. « Vert »
s'entend `npm run test:unit -- <fichier>` puis le gate complet (§ *Gates*).

⛔ **Règle commune des gardes lexicales** (reprise de la fiche 32-6, l. 172-184) : pour chaque garde, la fiche fixe la
**propriété**, le **périmètre** et les **formes** ; un motif écrit ici n'est qu'un **exemple** et ne vaut qu'avec sa
**mesure**. Au développement : (1) **à l'étape 0**, chaque motif retenu est greppé sur le **dépôt réel**, dans son
périmètre, et rend **vide** hors des sites écrits en liste blanche (fichier, ligne réduite de ses blancs) ; (2) chaque
garde a **un témoin rouge** (une forme qu'elle doit attraper) et **un témoin vert** (une forme **légitime et existante
au dépôt** qu'elle ne doit pas attraper), sur source factice, par une fonction pure ; (3) commandes et sorties citées
au Dev Agent Record. Il en va de même d'un test qui s'adosse à un outillage (un espion, un fuseau) : il porte un
**témoin** qui prouve qu'il ne passe pas à vide.

### Groupe A — l'écran

1. **AC-1 — la route et sa garde.** `frontend/src/routes/(app)/admin/mcp/+page.ts`, copie de
   `admin/backup/+page.ts:11-17` (`ssr = false`, `redirect(302, '/')` hors rôle `Admin`) ; `+page.svelte` mince, qui
   monte `McpServerPanel`. Test Vitest du `load` : `Comptable` et `Consultation` → redirection 302 vers `/` ;
   `Admin` → aucune. Le serveur reste l'autorité (fait 1) : la garde est une garde d'écran.
2. **AC-2 — la navigation.** Entrée `adminOnly` `{ i18nKey: 'nav-admin-mcp', fallback: 'Serveur MCP', href:
   '/admin/mcp' }` au groupe *Administration* (`+layout.svelte:147-170`), **après** `nav-admin-restore` (l. 166) ; si la
   32-8c a posé `nav-admin-mcp-sends` (constaté à l'étape 0), l'entrée de la 32-7 se place **immédiatement avant**
   elle. Un Comptable ne la voit pas (E2E, AC-16 (e)).
3. **AC-3 — la couche d'API sépare le secret de la clé** (C-32-7-5). `mcp-server.api.ts` expose `listMcpKeys`,
   `createMcpKey`, `revokeMcpKey(id, version)` (`DELETE` avec corps `{ version }`, motif `api-keys.api.ts:41-43`),
   `setMcpKeySendValidation(id, sendValidation, version)`, `getMcpServerSettings`, `updateMcpServerSettings`.
   - (a) `createMcpKey` rend **`{ cle: McpKey; secret: string }`**, où `cle` est construite **champ par champ** depuis
     les onze champs de `McpKeyResponse` (fait 2), **jamais** par décomposition (`{ ...res }`) ni par `delete res.key` ;
     le type `McpKey` n'a **pas** de champ `key`. Test `createMcpKey_separe_le_secret` : `apiClient.post` simulé rend
     un objet portant `key: 'kesh_mcp_TEMOIN…'` et un champ en trop `zzz` ; `'key' in cle === false`,
     `'zzz' in cle === false`, aucune valeur de `cle` (`Object.values`, récursivement) n'égale le secret,
     `secret === 'kesh_mcp_TEMOIN…'`.
   - (b) Les corps envoyés, **égaux** à l'attendu (`toEqual`) : création `{ name, scopes, sendValidation,
     expiresAt? }` — `sendValidation` **toujours présent** (`'required'` par défaut, C-32-7-7) ; révocation
     `{ version }` ; politique `{ sendValidation, version }` ; réglages, les **six** champs de
     `UpdateMcpServerSettingsRequest` (fait 5), rien d'autre.
4. **AC-4 — les dates** (#738 B7 = E6 ; C-32-7-8). `mcp-time.ts` :
   - (a) `depuisUtcNaif(s)` lit une date de clé (`NaiveDateTime`, fait 2) **comme de l'UTC** ; une chaîne qui porte
     déjà un fuseau (`Z`, `±hh:mm`) est lue telle quelle ; `null` → `null` ; une chaîne illisible → `null` (jamais
     `Invalid Date` affiché). `updatedAt` des réglages (avec `Z`, fait 5) passe par la même fonction.
   - (b) Affichage dans le **fuseau du navigateur**, par `Intl.DateTimeFormat(i18nLocale(), …)` ; jamais par
     `slice` d'une chaîne.
   - (c) L'expiration saisie (`<input type="date">`, `AAAA-MM-JJ`) devient la **fin de cette journée dans le fuseau
     du navigateur** (`23:59:59.999` local), envoyée en `toISOString()` ; le `min` du champ est le lendemain local,
     recalculé à l'ouverture du formulaire. Réaffichée, elle redonne **la même date**.
   - Tests `mcp-time.test.ts`, sous `process.env.TZ = 'Europe/Zurich'` posé dans le fichier et rétabli après, avec un
     **témoin** que le fuseau a pris (`new Date(2026, 6, 1, 12).getTimezoneOffset() === -120`) — sans lui, la CI en UTC
     ferait passer à vide la mutation M7 : `depuisUtcNaif('2026-10-25T10:00:00.123')` égale
     `Date.UTC(2026, 9, 25, 10, 0, 0, 123)` ; l'aller-retour `'2026-12-31'` → ISO → date affichée `31.12.2026`
     (locale `fr-CH`) ; une chaîne à `Z` n'est pas décalée deux fois.
5. **AC-5 — les réglages : lire, enregistrer, et dire la forme retenue.** Une section *Réglages du serveur* : case
   *Serveur activé*, case *Filtre d'adresse activé*, trois zones de texte (*Plages admises*, *Mandataires de
   confiance*, *Origines admises*), **une entrée par ligne**, la date de dernière modification (ou « jamais modifiés —
   valeurs par défaut » si `version: 0`), un bouton *Enregistrer*.
   - (a) À l'envoi, chaque ligne est rognée et **les lignes vides sont retirées** (fait 5 : le serveur refuse une
     entrée vide) ; rien d'autre n'est transformé côté écran — la validation est au serveur.
   - (b) `version` envoyée = celle **lue** (0 pour les défauts) ; jamais une valeur fabriquée.
   - (c) Succès → les champs sont **remplacés par la réponse** (formes canoniques : une adresse saisie `192.168.1.10`
     réaffichée `192.168.1.10/32`, `010.0.0.0/8` réaffiché `10.0.0.0/8` (#738 B2 = E4), un doublon retiré), message de
     succès. Une phrase fixe sous les zones le dit : « Kesh enregistre chaque entrée sous sa forme canonique. »
   - (d) 400 → le `message` du serveur dans la section (`role="alert"`), saisies **conservées**, rien d'enregistré.
   - (e) 409 → message de conflit (« les réglages ont changé entre-temps ; ils sont rechargés, vos modifications ne
     sont pas enregistrées »), puis rechargement.
   - (f) Erreur hors `ApiError` → le générique traduit (motif `api-keys-page.test.ts`, cas B6/B7), ni le message
     technique ni `[object Object]`.
   - Tests `McpSettingsSection.test.ts` : un cas par lettre, corps envoyé comparé **en entier**.
6. **AC-6 — ce que le filtre ne garantit pas, écrit à l'écran** (décision de Guy, 2026-10-25). Un encadré
   **permanent** de la section, quatre phrases au moins, dans les quatre locales :
   - (1) le filtre juge l'**adresse que voit Kesh** ; derrière **Traefik** non déclaré parmi les mandataires, ou
     derrière une **publication de port Docker**, tous les clients paraissent venir d'une adresse privée : **le filtre
     n'arrête alors personne** ;
   - (2) il ne remplace **ni le pare-feu ni le VPN** : Kesh ne doit jamais être exposé sur Internet ; l'accès de
     l'extérieur passe par un VPN ;
   - (3) la plage **`100.64.0.0/10`** (VPN de type Tailscale) **n'est pas admise par défaut** : l'ajouter soi-même aux
     plages admises (C-32-5-10) ;
   - (4) **aucune limite de débit** n'est posée sur `/mcp`.
   Puis des **avertissements conditionnels**, informatifs, jamais bloquants (C-32-7-12), calculés sur les champs
   **tels qu'affichés** :
   - (a) filtre désactivé → « toute adresse est admise » ;
   - (b) une plage admise de préfixe `/0` (`0.0.0.0/0`, `::/0`, ou toute entrée dont le préfixe, rogné, vaut 0) →
     « cette plage admet toute adresse, comme un filtre désactivé » (#738 B3 = E5) ;
   - (c) un **mandataire** de préfixe `/0` → « tout pair est cru sur son en-tête `X-Forwarded-For` » ;
   - (d) filtre activé et aucune plage admise → « personne n'est admis ».
   Tests : chaque avertissement **présent** quand sa condition l'est, **absent** sinon (témoin) ; l'encadré (1)-(4)
   présent dans tous les états ; `100.64.0.0/10` dans le texte rendu.
7. **AC-7 — l'activation, la restauration, la remise à zéro et l'adresse.** Sous la case *Serveur activé* :
   - (a) désactivé, « `/mcp` répond comme une adresse qui n'existe pas » ; activé, « le serveur répond aux clés MCP de
     cette société, depuis les adresses admises » ;
   - (b) l'**adresse du serveur**, `new URL('/mcp', window.location.origin).href`, en `<code>`, avec la mention
     « exactement `/mcp`, sans barre finale » (32-6 E8) — c'est l'adresse **que voit le navigateur** : derrière un
     mandataire, elle peut différer de celle qu'un agent du réseau local emploie (C-32-7-18) ;
   - (c) « une **restauration** rend ces réglages tels qu'ils étaient dans la sauvegarde : un serveur actif dans la
     sauvegarde l'est après l'import » (Q7, C-32-5-11 ; `admin-manual.tex:2237`) ;
   - (d) « la **réinitialisation** des données de démonstration efface ces réglages (serveur désactivé) et conserve
     les clés » (`admin-manual.tex:1414`).
   Test : (a) suit la case **enregistrée** (non la case cochée avant l'envoi) ; (b) rend `http://localhost:3000/mcp`
   sous `window.location.origin` simulé ; (c) et (d) présents.
8. **AC-8 — la liste des clés.** Colonnes : nom ; portées (le **code** `books:read` tel quel, chacune) ; politique
   (*Requise* / *Dispensée*) ; créée le ; expire le (ou « jamais ») ; **dernière utilisation** ; statut (*Active*,
   *Expirée le …*, *Révoquée le …*) ; actions. Ordre du serveur (fait 2). Une clé révoquée ou expirée est atténuée.
   - (a) `unknownScopes` non vides → chaque portée affichée avec « inconnue de cette version — jamais accordée » ; une
     clé qui n'a **que** des portées inconnues porte « n'ouvre aucun accès » (C-32-5-37).
   - (b) **La dernière utilisation** porte une note (infobulle ou phrase sous le tableau) : « posée chaque fois que la
     clé est présentée et acceptée, même si l'appel n'est pas servi ensuite » — jamais le libellé « dernier appel »
     (32-6, l. 1015-1018 ; la formulation exacte se recontrôle à l'étape 0 contre `services::cles_mcp::authentifier`
     et la 32-6a).
   - (c) État vide : « Aucune clé MCP. » ; échec du chargement : le message de l'`ApiError`, sinon le générique
     (motif `api-keys-page.test.ts`, B6).
   - (d) Le créateur de la clé n'est **pas** affiché (la réponse ne porte que `createdByUserId`, fait 2 ; C-32-7-16).
   - Tests `McpKeysSection.test.ts` : (a), (b), (c), et une clé expirée (`expiresAt` passé) marquée *Expirée*.
9. **AC-9 — créer une clé.** Formulaire inline : nom (requis, 255 points de code au plus, compté `[...nom]`), **cinq
   cases de portée, aucune cochée par défaut** (C-32-7-6), chacune avec sa phrase (AC-10) ; politique en deux boutons
   radio, **`Requise` cochée par défaut**, chacun avec sa phrase (AC-13) ; expiration facultative (AC-4 (c)).
   - (a) Refus à l'écran, sans appel : nom vide (après rognage), aucune portée.
   - (b) 400 du serveur → son `message` dans le formulaire, saisies conservées.
   - (c) Double soumission impossible (garde avant le `disabled`, motif `api-keys/+page.svelte:128-130`).
   - (d) Succès → encart du secret (AC-11), formulaire fermé et **remis à zéro**, liste **rechargée par `GET`**
     (jamais complétée par l'objet de création).
   - Tests : (a), (b), (c), (d), et le corps envoyé (AC-3 (b)).
10. **AC-10 — la phrase de chaque portée** (Q5 ; décision de Guy sur les tiers, 2026-10-25 ; fiches 32-9 l. 1339-1341
    et 32-10a l. 1386-1387). Sous la case, dans les quatre locales, **telle que le code des 32-9a, 32-9b et 32-10a la
    rend vraie** — formulation de départ, à confronter au code à l'étape 0 (ligne lue citée ; une phrase que le code ne
    confirme pas se réécrit) :
    - `books:read` — « le plan comptable, les exercices, les écritures, le grand livre, la balance, le bilan, le compte
      de résultat et les postes ouverts » ;
    - `invoices:read` — « les factures clientes et fournisseur et la balance âgée ; les **noms des tiers** sont
      lisibles, mais l'IBAN, le QR-IBAN du créancier et l'adresse d'envoi sont **masqués** sans `contacts:read` » ;
    - `contacts:read` — « le carnet d'adresses — des **données personnelles** — ; elle **démasque** aussi, dans les
      factures, l'IBAN et le QR-IBAN du créancier et l'adresse d'envoi ; les noms des tiers se lisent sans elle » (et
      l'IBAN n'est pas une donnée du contact, `contacts.rs:380-381` selon la 32-9 : à recontrôler) ;
    - `documents:read` — « la liste des pièces et de leurs rattachements, **sans leur contenu** » ;
    - `prompts:write` — « **proposer** une version d'un prompt, qui attend l'approbation d'un administrateur ; elle ne
      permet ni d'approuver ni de relire une proposition ; une clé à validation requise peut proposer ».
    Et, au-dessus des cases : « Aucune portée n'écrit dans les livres ni n'administre Kesh. »
    Test : les cinq phrases rendues, une par case, et l'ordre des cases égal à `McpScope::ALL` (fait 7) — l'ordre est
    tenu par un tableau unique `PORTEES_MCP` de `mcp-server.types.ts`, que la garde G23 (AC-25 (b)) **ne** lit **pas** :
    le test Vitest compare ce tableau à la liste littérale des cinq codes, recopiée de `mcp_key.rs:48-54` avec la
    ligne citée.
11. **AC-11 — le secret, une seule fois** (la propriété de la story ; C-32-7-5). Après un 201 :
    - l'encart (`role="alert"`) dit « Clé « {nom} » créée », « copiez-la maintenant : elle ne sera plus jamais
      affichée », et « donnez-la au client MCP en portée locale ou utilisateur — jamais dans un `.mcp.json` versionné »
      (renvoi `sec:mcp-client`) ; le secret dans **un seul** élément `<code data-testid="mcp-server-secret-value">` —
      ni `<input>`, ni attribut (`value`, `title`, `data-*`, `aria-*`) ; boutons *Copier* (`copyToClipboard`, fait 8)
      et *Fermer* ;
    - le secret est tenu par **une instance** de la classe `SecretUnique` (`secret-unique.svelte.ts` : `montrer(s)`,
      `lire()`, `oublier()`), créée **dans le `<script>` du composant** — jamais au niveau d'un module ; *Fermer*
      appelle `oublier()` ; une **nouvelle création** appelle `oublier()` avant `montrer()` ; quitter la page détruit
      le composant et son instance.
    Les cinq « ni », chacun son test :
    - (i) **le DOM** — pendant l'affichage, `document.body` contient le secret dans **exactement un** nœud texte ;
      après *Fermer*, **aucun** (`document.body.outerHTML` ne le contient pas), ni après une nouvelle création qui a
      échoué, ni après un rechargement de la liste ;
    - (ii) **l'état** — `secret-unique.test.ts` : après `oublier()`, `lire() === null` et ni `JSON.stringify` de
      l'instance ni aucune propriété propre (`Object.getOwnPropertyNames`, valeurs lues) ne contient le secret ; le test
      du composant, la classe espionnée, vérifie que *Fermer* appelle `oublier()` ;
    - (iii) **la liste** — AC-3 (a), et aucune ligne du tableau ne contient le secret après création ;
    - (iv) **un journal** — `console.log`, `info`, `warn`, `error`, `debug`, `trace` espionnés pendant tout le
      parcours (création, copie, fermeture, échec simulé d'une seconde création) : aucun argument, converti en texte
      (`String` et `JSON.stringify`), ne contient le secret ; aucun appel de `toast.*` non plus ; `document.title` non
      plus ; **témoin** : un `console.info('kesh_mcp_TEMOIN')` posé par le test lui-même est bien vu par l'espion ;
    - (v) **le stockage et l'adresse** — `Storage.prototype.setItem` (donc `localStorage` et `sessionStorage`)
      espionné : jamais appelé avec le secret ; `indexedDB.open` jamais appelé ; `history.pushState`/`replaceState`
      jamais appelés avec une URL qui le contient.
    Côté serveur, déjà établi par la 32-5c et cité au Dev Agent Record : la liste sans secret ni condensat
    (`mcp_admin_routes_e2e.rs:371`), l'audit sans l'un ni l'autre (`:803`), aucun journal de corps (fait 14).
12. **AC-12 — révoquer.** Offerte pour **toute clé non révoquée**, expirée comprise (C-32-7-9). Confirmation inline :
    « L'agent qui emploie cette clé perd l'accès immédiatement. La révocation est définitive ; seule la restauration
    d'une sauvegarde antérieure rendrait la clé de nouveau active. » ; bouton `<Button variant="destructive">`, **sans
    rouge posé à la main** (fait 12 : la table de `boutons-rouges-inventory.test.ts` reste inchangée) ; 204 → message
    et rechargement ; 404 ou 409 → message (« la clé a changé ou n'existe plus ; liste rechargée ») et rechargement ;
    hors `ApiError` → le générique. Tests : chaque issue ; le bouton absent sur une clé révoquée ; présent sur une
    expirée ; `revokeMcpKey` appelé avec la `version` de la ligne.
13. **AC-13 — la politique de validation d'une clé** (Q12 ; fiche 32-8 l. 1406-1408). Pour une clé non révoquée, un
    contrôle *Passer en dispensée* / *Passer en requise* :
    - (a) vers `exempt` : **confirmation** — « les données partiront à l'agent **sans vous être montrées** ; chaque
      appel reste journalisé » ; vers `required` : sans confirmation (C-32-7-10) ;
    - (b) succès → la ligne **remplacée par la clé rendue** ; 409 ou 404 → message et rechargement ; hors `ApiError`
      → le générique ;
    - (c) les deux phrases de politique, au formulaire de création et en note du tableau : *Requise* — « chaque réponse
      d'outil qui porte des données attend votre validation dans *Envois du serveur MCP* avant de partir » (délai,
      plafonds et exceptions **tels que le code de la 32-8 les fixe**, recontrôlés à l'étape 0 : 24 h, 20 envois en
      attente par clé, une erreur d'outil partant sans validation, fiche 32-8 l. 916-930) ; *Dispensée* — « les
      données partent sans validation ; chaque appel reste journalisé » ;
    - (d) un lien vers *Envois du serveur MCP* (`/admin/mcp/sends`), **si** la route existe à l'étape 0 (32-8c) ;
      sinon, pas de lien et un écart déclaré.
    Tests : (a) confirmation présente vers `exempt` et absente vers `required` ; (b) chaque issue ; pas de contrôle sur
    une clé révoquée ; (d) le lien et sa cible.
14. **AC-14 — i18n.** Toutes les clés `mcp-server-*` et `nav-admin-mcp` dans les **quatre** catalogues ;
    `npm run lint-i18n-ownership` vert sans entrée neuve à `KNOWN_VIOLATIONS` ; repli **égal** à la valeur fr-CH (garde
    frontend existante, nommée à l'étape 0) ; `i18n-keys.test.ts` vert ; aucune collision avec `mcp-sends-*` (32-8) ni
    avec les messages serveur `admin-mcp-*` (fait 11). Les noms propres (`Traefik`, `Docker`, `100.64.0.0/10`,
    `X-Forwarded-For`, `.mcp.json`) et les codes de portée **ne se traduisent pas**.
15. **AC-15 — deux gardes lexicales** (règle commune ; fichier `frontend/src/lib/features/mcp-server/
    mcp-server-gardes.test.ts`, `// @vitest-environment node`) :
    - **G-a — ni stockage, ni journal, ni HTML brut.** *Propriété* : aucune source **hors tests** de
      `features/mcp-server/` ni de `routes/(app)/admin/mcp/` — **à l'exclusion** des sous-dossiers d'autres stories
      (`sends/` de la 32-8c, `prompts/` de la 32-10b, constatés à l'étape 0) — n'emploie un stockage du navigateur, un
      journal ou du HTML brut. *Formes* : `localStorage`, `sessionStorage`, `indexedDB`, `caches.`, `document.cookie`,
      `console.`, `{@html`, après `masquerCommentaires` (fait 15). *Mesure à la rédaction*, sur les dossiers
      comparables (le périmètre n'existe pas encore) : `grep -rnE 'localStorage|sessionStorage|indexedDB|caches\.|
      document\.cookie|console\.|\{@html' frontend/src/lib/features/api-keys "frontend/src/routes/(app)/settings/
      api-keys" frontend/src/lib/features/admin-backup | grep -v '\.test\.ts'` → **1** ligne,
      `routes/(app)/settings/api-keys/+page.svelte:41`, un **commentaire** (« pas de localStorage ») — que le masquage
      doit écarter : c'est le **témoin vert**. *Témoin rouge* : `sessionStorage.setItem('k', s)` sur source factice.
    - **G-b — l'instance du secret ne vit que dans un composant.** *Propriété* : `new SecretUnique` n'apparaît que dans
      le `<script>` d'un `.svelte` du périmètre de G-a, jamais dans un `.ts`/`.svelte.ts` hors tests (une instance de
      module survivrait à la fermeture et à la navigation). *Mesure à la rédaction* : la classe n'existe pas
      (`grep -rn 'SecretUnique' frontend/src` → rien) ; la mesure se refait à l'étape 0 **et** après T3. *Témoin
      rouge* : `export const s = new SecretUnique();` dans un `.ts` factice ; *témoin vert* :
      `const secret = new SecretUnique();` dans le `<script>` d'un `.svelte` factice.
    Chaque garde lit les **vrais** fichiers du périmètre, et porte une assertion de **cardinalité** du périmètre lu
    (nombre de fichiers ≥ celui que la story crée, recompté) — sans elle, un chemin faux la ferait passer à vide.
16. **AC-16 — l'E2E** (`frontend/tests/e2e/mcp-server.spec.ts`, sélecteurs par `data-testid` seulement, fait 13 ; motif
    `api-keys.spec.ts`). Préalable lu à l'étape 0 : ce que `seedTestState` vide (`mcp_keys`, `mcp_server_settings`) ;
    si l'une n'est pas vidée, un `afterAll` remet le serveur **désactivé** et révoque les clés de la spec.
    - (a) **activer** : Administrateur, menu *Administration* → *Serveur MCP* (par `data-testid`) ; état initial
      désactivé ; cocher, *Enregistrer* ; `GET /api/v1/admin/mcp/settings` (contexte authentifié) rend
      `enabled: true` ;
    - (b) **créer et voir une fois** : nom « agent E2E 327 », `books:read`, politique laissée à *Requise* ; le secret
      lu dans `mcp-server-secret-value` répond à `^kesh_mcp_[0-9A-Za-z]{27}$` ; *Fermer* ; `page.content()` ne le
      contient plus ; après `page.reload()`, non plus ; `page.url()` ne le contient pas ; `localStorage` et
      `sessionStorage` (toutes valeurs, par `page.evaluate`) ne le contiennent pas ;
    - (c) **la clé montrée est la vraie** (32-6a fusionnée) : `POST /mcp` `initialize` (en-tête `Authorization: Bearer
      <secret>`, sans cookie, `Content-Type: application/json`) → **200** ;
    - (d) **régler la politique** : *Passer en dispensée*, confirmer ; la liste (`GET`) rend `sendValidation:
      "exempt"` ;
    - (e) **révoquer** : confirmer ; le statut devient *Révoquée* ; le même `POST /mcp` → **401** (corps égal au 401
      fixe de la 32-6a) ;
    - (f) **un Comptable** ne voit pas l'entrée de menu, et `/admin/mcp` le redirige vers `/`.
    La suite complète tourne au dernier commit de code ; les échecs se jugent **fichier par fichier** contre
    `docs/testing.md` § *Les échecs attendus* (7 à 10 attendus).
17. **AC-17 — la recette visuelle** (`CLAUDE.md` § *Recette visuelle*, thème clair seulement), sur une base de
    démonstration, captures hors du dépôt (`/home/gcorbaz/devel/kesh-gate-logs/327-captures/`), jointes au Dev Agent
    Record et au prompt de la revue de code, **une par état touché** :
    (1) l'écran à l'état initial (désactivé, défauts, aucune clé, encadré AC-6) ; (2) serveur activé et enregistré,
    date de modification ; (3) filtre désactivé, avertissement (a) ; (4) une plage `0.0.0.0/0`, avertissement (b) ; (5)
    un mandataire `/0`, avertissement (c) ; (6) filtre activé sans plage, avertissement (d) ; (7) une entrée refusée
    par le serveur (bits d'hôte) ; (8) une forme canonique réaffichée après enregistrement (`192.168.1.10` →
    `/32`) ; (9) le conflit de version ; (10) le formulaire de création, les cinq phrases de portée, les deux de
    politique ; (11) un refus du formulaire (aucune portée) ; (12) l'encart du secret ; (13) la liste après
    *Fermer* ; (14) la confirmation de révocation ; (15) une clé révoquée ; (16) une clé expirée ; (17) une clé à
    portée inconnue (posée par SQL sur la base de démonstration du worktree) ; (18) la confirmation du passage en
    dispensée ; (19) une clé dispensée dans la liste ; (20) l'entrée de menu ; (21) l'échec du chargement.
    ⚠️ La capture (12) montre un **secret réel de la démonstration** : la clé est **révoquée** juste après, et la
    capture reste hors du dépôt.

### Groupe B — le manuel et les textes

18. **AC-18 — la section *Le serveur MCP* du manuel d'administration** (C-32-7-4). Une `\section{Le serveur
    MCP}\label{sec:mcp}` placée **après** la `\section{Sécurité}` et **avant** `\section{Monitoring et logs}` (l. 3017 à
    `966baed5`, recomptée), dont les sous-sections disent, **tel que le code le fait** (chaque affirmation confrontée au
    code à l'étape 0, ligne citée) :
    1. `\subsection{Activer le serveur}\label{sec:mcp-activer}` — *Administration → Serveur MCP*, Administrateur
       seul ; **désactivé par défaut** ; désactivé, `/mcp` répond **comme une route absente** (C-32-6-2), pas un 404
       littéral ; l'activation est par société, et une clé d'une société qui ne sert pas reçoit le 401 d'une clé
       inconnue ; l'adresse est **exactement** `/mcp` (`/mcp/` rend la page de l'application) ; une **restauration** rend
       les réglages tels quels — serveur actif s'il l'était (renvoi au § sauvegarde, l. 2237) ; la **réinitialisation**
       les efface (renvoi l. 1414).
    2. `\subsection{Les clés du serveur MCP}\label{sec:mcp-cles}` — création par l'**Administrateur** seul ; le secret
       `kesh_mcp_…` **affiché une seule fois**, seul son condensat conservé ; les **portées** en tableau entre les
       lignes `% mcp-portees:debut` et `% mcp-portees:fin` (une ligne par portée : code, ce qu'elle ouvre — dont ce que
       `contacts:read` **démasque** —, en reprenant les phrases de l'AC-10) ; **aucune** portée d'écriture des livres
       ni d'administration ; expiration ; révocation immédiate et définitive ; « dernière utilisation » n'est pas
       « dernier appel servi » ; une clé MCP **n'ouvre pas** `/api/v1` (401), et `/mcp` **ne lit que**
       `Authorization: Bearer <clé MCP>` — ni un PAT ni le navigateur connecté ; au journal d'audit, un appel paraît
       **au nom du créateur de la clé**, sans la marque « clé API » (point 4 de la fiche 32-6, accepté) ; restauration
       (une clé révoquée depuis redevient active), remise à zéro (clés conservées), réparation (#528 : révoquées).
    3. `\subsection{La politique de validation}\label{sec:mcp-validation}` — `requise` par défaut, `dispensée` désignée
       par l'administrateur ; ce que chacune fait, **sans** la phrase « une clé `required` ne peut appeler aucun outil »
       (devenue fausse avec la 32-8, fiche 32-8 l. 1397, 1408) ; renvoi à `sec:mcp-envois`.
    4. `\subsection{Le filtre d'adresse}\label{sec:mcp-filtre}` — porte sur le **pair TCP** ; `X-Forwarded-For` lu des
       seuls **mandataires déclarés**, de droite à gauche ; `Forwarded` et `X-Real-IP` **toujours ignorés** ; refusé,
       le client voit une route absente ; filtre activé et liste vide = personne ; une entrée invalide des **plages**
       est ignorée, une entrée invalide des **mandataires** coupe le serveur MCP de la société (route absente, erreur au
       journal du serveur), **filtre activé ou non** (P7-H3 de la 32-6) — l'écran n'en laisse enregistrer aucune, une
       sauvegarde ou une écriture SQL le peut ; les **défauts**, listés exactement entre `% mcp-reseaux-defaut:debut` et
       `% mcp-reseaux-defaut:fin` (fait 6) ; **`100.64.0.0/10` absente**, à ajouter pour un VPN de type Tailscale ; un
       `0.0.0.0/0` ou `::/0` admet tout client (#738) ; une adresse IPv4 mappée en IPv6 est reconnue ; puis un
       `keshwarning` **Ce que le filtre ne garantit pas** : Traefik à déclarer parmi les mandataires, faute de quoi tous
       les clients paraissent venir de lui ; la publication de port Docker par le mandataire en espace utilisateur, où
       tous peuvent paraître venir de la passerelle (`172.17.0.1`), adresse privée admise par défaut — **le filtre ne
       filtre alors rien** ; il ne remplace ni le pare-feu (renvoi `sec:firewall`) ni le VPN ; Kesh n'est **jamais**
       exposé sur Internet ; **pas de limite de débit** ; et, **si la 32-6a l'a posée** (point 2 bis de sa fiche,
       constaté à l'étape 0), la garde qui refuse un pair non déclaré porteur d'un en-tête de mandataire — sinon, la
       phrase « un tel en-tête venu d'un pair non déclaré est ignoré ».
    5. `\subsection{L'en-tête Origin}\label{sec:mcp-origin}` — ne protège que **d'un navigateur** (DNS rebinding) ; la
       vraie barrière est la clé ; `Host` n'est pas contrôlé ; origines propres (`KESH_PUBLIC_BASE_URL`,
       `localhost`/`127.0.0.1`/`[::1]` au port de Kesh, port par défaut normalisé) ; un `KESH_PUBLIC_BASE_URL` invalide
       retire **en silence** l'origine qui en viendrait (avertissement au seul journal du serveur), un
       `KESH_PUBLIC_BASE_URL` à sous-chemin donne son origine ; **pas de CORS** (`OPTIONS` → 405) : une origine admise
       n'ouvre `/mcp` à aucune page web étrangère.
    6. `\subsection{Ce qui est journalisé}\label{sec:mcp-journal}` — au journal d'audit : `mcp_key.created`,
       `mcp_key.revoked`, `mcp_key.send_validation_changed`, `mcp_server_settings.updated`, et **chaque** `tools/call`
       (`mcp.tool_called` : clé, outil — ou « (inconnu) » —, issue, durée, politique ; **jamais** les arguments ni les
       données) ; si le journal ne s'écrit pas, **rien ne part** ; ce qui n'y est pas (`initialize`, `ping`,
       `tools/list`, les notifications, les refus du transport — au journal du serveur seulement), et ce que les
       stories suivantes y ajoutent (consultation d'un rapport, 32-9 ; décisions de validation, 32-8 ; `prompts/get`,
       32-10a — chacun **constaté** à l'étape 0, sinon omis) ; un appel MCP **écrit** (dernière utilisation, journal) —
       à ne pas présenter comme « une lecture n'écrit rien » (#730) ; un panic n'est pas journalisé.
    7. `\subsection{Ce que voit un client mal configuré}\label{sec:mcp-erreurs}` — `Content-Type: application/json`
       exigé (415) ; `MCP-Protocol-Version` autre que `2025-11-25`/`2025-06-18` refusée (400) ; corps limité à 1 Mio ;
       délai de 30 s par outil ; en-têtes de contrôle répétés refusés ; toute notification reçoit 202 ; un message de
       type réponse reçoit 400 (32-6 B4) ; les erreurs JSON-RPC du transport n'ont pas d'`id` ; chaque appel d'outil
       ouvre une connexion MariaDB neuve — chaque valeur **relue au code** de la 32-6a à l'étape 0.
    8. `\subsection{Configurer un client}\label{sec:mcp-client}` (C-32-7-13) — pour **Claude Code** :
       `claude mcp add --transport http --scope user kesh <adresse>/mcp --header "Authorization: Bearer <clé>"` (ou
       `--scope local`) ; la clé y est enregistrée **en clair** dans `~/.claude.json` — protéger ce fichier ; **jamais**
       `--scope project`, qui écrit un `.mcp.json` **versionné**, sauf avec `"Authorization": "Bearer
       ${KESH_MCP_KEY}"` et la variable hors du dépôt ; la commande passe par l'historique du shell : la précéder d'une
       espace (avec `HISTCONTROL=ignorespace`) ou lire la clé par `read -rs` ; la limite de **25 000 jetons** d'une
       réponse dans Claude Code ; pour **Hermes**, le bloc `mcp_servers` de `~/.hermes/config.yaml` (spike § 2.4). Une
       phrase dit que ces gestes **ne sont pas éprouvés** contre un client réel par cette story, et renvoie au parcours
       pas à pas de la 32-11 (*Brancher un agent*).
    9. **Rangées sous `sec:mcp`**, déplacées sans changer leurs étiquettes : `\subsection{Les outils et leurs
       portées}\label{sec:mcp-outils}` (32-9, et son bloc `% matrice-mcp:debut`/`fin`, dont le test de la 32-9 doit
       rester vert — son ancrage relu à l'étape 0) ; `\subsection{Les envois du serveur MCP}\label{sec:mcp-envois}`
       (32-8c) ; et, **si elle existe à l'étape 0**, la section *Les prompts du serveur MCP* (32-10b) — sinon la fiche
       32-10b est avisée de s'y ranger (demande à l'orchestrateur).
    Aucune donnée réelle, aucun IBAN : exemples factices (`kesh_mcp_` suivi de `…`).
19. **AC-19 — les renvois** (report (4) de la 32-5) : `admin-manual.tex` l. 1414 et 1416 (remise à zéro), 2237
    (sauvegarde), 2467 et 2472 (réparation) gagnent `\S\ref{sec:mcp-cles}` (ou `sec:mcp-activer` pour les réglages) ;
    la `\subsection{Clés API (PAT)…}` (l. 2898) gagne une phrase : « les clés du serveur MCP sont un autre type de clé,
    \S\ref{sec:mcp-cles} » ; `user-manual.tex` l. 190, 3204-3205 et 3214-3215 gagnent « (manuel d'administration,
    § *Le serveur MCP*) ». Lignes **recomptées** à l'étape 0.
20. **AC-20 — ce que la 32-7 ne touche pas, et qui le porte** (C-32-7-14) : `admin-manual.tex:2935` (le compte « N des
    M routes à verbe mutant » — la 32-7 n'ajoute aucune route : **32-10b**, qui le recompte après ses routes, fiche
    32-10a l. 1008 et 1384, puis **32-11** au dernier état, fiche 32-6 A3 l. 1279-1281) ; `docs/api-external.md:185` et
    `:831` (**32-11**, fiches 32-6 l. 1282-1283, 32-9 l. 1328) ; `admin-manual.tex:2925`, la brochure
    (`marketing-brochure.tex:480`), `README.md:233` et `website/roadmap.html:375` (**32-11**). À l'étape 0, la 32-7 relit
    chacun et **écrit au Dev Agent Record** s'il est faux à cette date ; elle ne le corrige pas.
21. **AC-21 — le PDF.** `admin-manual.pdf` et `user-manual.pdf` régénérés (`make fr` dans `docs/manual/`) et commités ;
    contrôle **aplati** cité au Dev Agent Record : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s
    ' ' | grep -oiE '.{0,60}(Le serveur MCP|100\.64\.0\.0/10|ne filtre alors rien|\.mcp\.json|25 000 jetons).{0,80}'`
    — chaque motif trouvé au moins une fois ; et les renvois de l'AC-19 rendus (aucun `??`).
22. **AC-22 — CHANGELOG.** Sous `## [0.16.0] — Non publié`, `### Ajouté` (créée si absente, ordre Keep a Changelog) :
    « **Administration → Serveur MCP** : activer le serveur, régler son filtre d'adresse, créer, révoquer et régler les
    clés du serveur MCP ; manuel d'administration, § *Le serveur MCP* » — sans date (la release la pose).
23. **AC-23 — README et site.** Aucune fonctionnalité annoncée *(à venir)* ne devient livrée par la seule 32-7 (le
    serveur entier se publie à la 0.16.0) ; la 32-7 le **constate** par `grep -n 'MCP' README.md website/*.html` (sortie
    citée) et ne change rien, sauf une phrase qui nommerait l'écran comme absent.
24. **AC-24 — aucune donnée réelle.** Ni secret, ni IBAN, ni libellé de `~/kesh-test/` dans un fichier versionné ;
    exemples du manuel factices.

### Groupe C — la garde de cohérence

25. **AC-25 — la garde G23** (`crates/kesh-api/tests/textes_coherents.rs`, ligne ajoutée au tableau d'en-tête l. 12-27 ;
    numéro relu à l'étape 0 : si une story fusionnée a pris G23, le suivant libre). Elle échoue en **nommant** l'écart :
    - (a) le bloc `% mcp-reseaux-defaut:debut`/`fin` du manuel liste **exactement** `DEFAULT_ALLOWED_NETWORKS`
      (`kesh_db::entities::mcp_server_settings`, fait 6), ensemble égal dans les deux sens ; et la chaîne
      `100.64.0.0/10` paraît dans `sec:mcp-filtre` **hors** de ce bloc ;
    - (b) le bloc `% mcp-portees:debut`/`fin` nomme **exactement** les cinq `McpScope::ALL` (`as_str`), dans cet ordre ;
    - (c) dans les **quatre** catalogues, la clé de l'AC-6 (3) contient `100.64.0.0/10`, et celle de l'AC-6 (1)
      contient `Traefik` **et** `Docker` (noms des clés fixés au développement, écrits dans la garde) ;
    - chaque volet a son **témoin** : la fonction pure qui extrait un bloc rend une erreur sur un bloc absent (sans quoi
      un marqueur supprimé ferait passer (a) et (b) à vide).

## Tâches

- [ ] **T0 — étape 0** (§ *À recontrôler à l'étape 0*) : chaque point relu, ligne citée au Dev Agent Record ; les
  mesures des gardes G-a et G-b refaites ; l'état des 32-6a/b, 32-8, 32-9, 32-10a et 32-10b constaté.
- [ ] **T1 — la couche d'API et les dates** : `mcp-server.types.ts`, `mcp-server.api.ts`, `mcp-time.ts` et leurs tests
  (AC-3, AC-4).
- [ ] **T2 — les réglages** : `McpSettingsSection` et ses tests (AC-5, AC-6, AC-7).
- [ ] **T3 — les clés** : `secret-unique.svelte.ts`, `McpKeysSection`, `McpKeySecretNotice` et leurs tests (AC-8 à
  AC-13).
- [ ] **T4 — la route, la navigation, l'i18n, les gardes lexicales** (AC-1, AC-2, AC-14, AC-15).
- [ ] **T5 — l'E2E et la recette visuelle** (AC-16, AC-17).
- [ ] **T6 — le manuel, les renvois, les PDF, le CHANGELOG, README et site** (AC-18 à AC-24).
- [ ] **T7 — la garde G23** (AC-25).
- [ ] **T8 — mutations, gates, Dev Agent Record, prompt de la revue de code P1.**

## Tests et mutations prévus

| Fichier | Tests (noms indicatifs) | AC |
|---|---|---|
| `features/mcp-server/mcp-server.api.test.ts` | séparation du secret ; corps de chaque appel | 3 |
| `features/mcp-server/mcp-time.test.ts` | naïf UTC ; fuseau déjà porté ; illisible ; fin de journée locale ; témoin du fuseau | 4 |
| `features/mcp-server/McpSettingsSection.test.ts` | (a) à (f) ; avertissements (a) à (d) et leurs témoins ; encadré ; AC-7 | 5, 6, 7 |
| `features/mcp-server/secret-unique.test.ts` | `oublier` efface tout ; `montrer` remplace | 11 (ii) |
| `features/mcp-server/McpKeysSection.test.ts` | liste ; création ; les cinq « ni » ; révocation ; politique | 8 à 13 |
| `features/mcp-server/mcp-server-gardes.test.ts` | G-a, G-b, témoins, cardinalité | 15 |
| `routes/(app)/admin/mcp/admin-mcp-page.test.ts` | garde du `load` | 1 |
| `tests/e2e/mcp-server.spec.ts` | (a) à (f) | 16 |
| `crates/kesh-api/tests/textes_coherents.rs` | G23 (a) à (c), témoin du bloc absent | 25 |

**Mutations** — chacune jouée sur une copie, le fichier restauré **et touché** ensuite (mémoire *Mutation restaurée,
binaire périmé*), journal `kesh-gate-logs/327-mut-<id>.log`, verdict au Dev Agent Record :

| Id | Mutation | Doit tuer |
|---|---|---|
| M1 | *Fermer* masque l'encart (`visible = false`) sans appeler `oublier()` | AC-11 (i)/(ii) — test composant |
| M2 | `oublier()` remet un drapeau sans effacer la valeur | `secret-unique.test.ts` |
| M3 | `createMcpKey` rend `cle = { ...res }` | `createMcpKey_separe_le_secret` |
| M4 | l'objet de création ajouté à la liste au lieu d'un `GET` | AC-9 (d) / AC-11 (iii) |
| M5 | `sessionStorage.setItem('mcp-secret', secret)` à la création | AC-11 (v) **et** G-a |
| M6 | `console.info('créée', res)` | AC-11 (iv) **et** G-a |
| M7 | `depuisUtcNaif` sans ajout du `Z` | `mcp-time.test.ts` (sous TZ Zurich, témoin) |
| M8 | expiration envoyée `${date}T23:59:59Z` (motif PAT) | aller-retour de date, AC-4 (c) |
| M9 | `sendValidation` omis du corps de création | AC-3 (b) |
| M10 | avertissement `/0` retiré | AC-6 (b) |
| M11 | lignes vides non retirées avant l'envoi | AC-5 (a) |
| M12 | 409 des réglages sans rechargement | AC-5 (e) |
| M13 | contrôle de politique offert sur une clé révoquée | AC-13 |
| M14 | confirmation du passage en dispensée retirée | AC-13 (a) |
| M15 | `+page.ts` sans la redirection | AC-1 |
| M16 | `export const secret = new SecretUnique()` dans un `.ts` du périmètre | G-b |
| M17 | `10.0.0.0/8` retiré du bloc `mcp-reseaux-defaut` du manuel | G23 (a) |
| M18 | `100.64.0.0/10` retiré de la clé fr-CH de l'AC-6 (3) | G23 (c) |
| M19 | les marqueurs `% mcp-portees:` retirés | G23 témoin (bloc absent) |

## Gates

- **Frontend** (depuis `frontend/`, sous `scripts/mem-guard.sh`) : `npm run check`, `npm run lint-i18n-ownership`,
  `npm run test:unit`, `npm run build`.
- **Backend** : la garde G23 et les catalogues (`kesh-i18n` : parité des quatre locales) imposent `cargo fmt --all --
  --check`, `cargo clippy --workspace --all-targets -- -D warnings`, puis, en boucle, le gate ciblé
  `cargo nextest run -E 'binary(textes_coherents)'` **et** les tests de `kesh-i18n` ; le **gate complet**
  (`scripts/test-fast.sh --ci`, base du worktree remise à zéro juste avant) au dernier commit de code et avant tout push.
- **E2E complet** au **dernier commit de code** (montage de `docs/testing.md` § *Prérequis Playwright local*, base
  `kesh_e2e_327`, dossiers `/tmp/kesh-e2e-327/`, port propre au worktree, binaire reconstruit après le commit) —
  « rejouée au push » interdit.
- **Manuel** : `make fr`, PDF commités, contrôle aplati (AC-21).
- Le Dev Agent Record n'affirme que ce qui a tourné (gate ciblé ou complet, nommé).

## Ce que les gardes ne voient pas

- **Le presse-papiers** : *Copier* y dépose le secret ; un gestionnaire d'historique du presse-papiers le garde.
  Aucune garde ne le voit ; le manuel le dit (copier, coller, puis vider).
- **Le panneau réseau des outils du navigateur** montre le corps de la réponse 201, secret compris ; une extension du
  navigateur le peut aussi. Inhérent à un affichage unique dans une page web.
- **La mémoire du navigateur** : `oublier()` retire la référence ; la chaîne vit jusqu'au ramasse-miettes. Les tests
  prouvent l'absence d'une référence **atteignable par l'écran**, non l'effacement de la mémoire.
- **L'outillage de développement** (HMR de Vite, extension Svelte) peut conserver un état de composant ; hors
  production.
- **Une écriture indirecte** dans un stockage (par une bibliothèque, par `window['local' + 'Storage']`) échappe à G-a ;
  les espions de l'AC-11 (v) couvrent les chemins **exécutés** par les tests seulement.
- **Ce que l'utilisateur fait du secret** : l'historique du shell, `~/.claude.json` en clair, un `.mcp.json` versionné
  malgré l'avertissement. Le manuel le dit ; rien ne le contrôle.
- **La topologie réelle** (Traefik, Docker, NAS) : l'écran et le manuel **décrivent** ce que le filtre ne garantit pas ;
  rien ne détecte qu'on est derrière un mandataire non déclaré (non vérifié sur le NAS, point 2 de la fiche 32-6).
- **L'adresse affichée** (AC-7 (b)) est celle du navigateur ; un agent du réseau local peut devoir en employer une autre.
- **Les textes de portée et de politique** (AC-10, AC-13 (c)) sont tenus vrais par une **relecture** au code à l'étape 0
  et en revue, non par un test : seul l'ordre des portées l'est (AC-10), et les deux listes du manuel (G23).
- **L'E2E ne tourne qu'en `fr-CH`** ; la justesse des trois autres langues n'est vérifiée que par la parité des clés.
- **Les dates** s'affichent dans le fuseau du navigateur ; un administrateur hors de Suisse verra l'heure de son poste.

## Inventaire des reports

Méthode : `grep -n '32-7'` sur l'epic, la fiche 32-5 (worktree de la 32-7), les fiches 32-6, 32-8, 32-9, 32-10a (leurs
worktrees), la spike (l. 945-948), et `gh issue view 738` (la phrase « l'écran de la 32-7 » et les points qui touchent
l'écran). Chaque report est rattaché à un AC, ou écarté avec sa raison.

| # | Source | Report | Où |
|---|---|---|---|
| R1 | epic l. 420-421, fiche 32-5 l. 59, 769-771 | section *Le serveur MCP* : création, affichage unique, portées, politique, filtre | AC-18 (1)-(4) |
| R2 | epic l. 421-422, spike l. 927-928 | clé en portée `local`/`user`, jamais en `.mcp.json` versionné sauf `${VARIABLE}` | AC-11, AC-18 (8) |
| R3 | epic l. 422-423, C-32-5-10 | `100.64.0.0/10` absente des défauts, à l'écran et au manuel | AC-6 (3), AC-18 (4), AC-25 |
| R4 | epic l. 423-424, C-32-5-11, Q7 | un serveur actif dans la sauvegarde l'est après l'import | AC-7 (c), AC-18 (1) |
| R5 | epic l. 424-429, C-32-5-48, fiche 32-5 l. 572-573 | renvois depuis admin 1414, 1416, 2467, 2472 ; utilisateur 190, 3204-3205, 3214-3215 | AC-19 |
| R6 | #738 B7 = E6 | horodatages des clés sans `Z`, des réglages avec | AC-4 |
| R7 | #738 B3 = E5, fiche 32-6 l. 1558 | `/0` accepté, admet tout client | AC-6 (b), (c), AC-18 (4) |
| R8 | #738 B2 = E4 | `010.0.0.0/8` mis en forme sans signal | AC-5 (c) (forme réaffichée) |
| R9 | #738 B5 = E2 | corps mal formé non traduit | **écarté** : l'écran n'envoie que des corps bien formés (AC-3 (b)) ; le défaut reste sur #738 |
| R10 | #738 B8 | `version: -1` → 409 | **écarté** : l'écran n'envoie que la version lue (AC-5 (b)) ; reste sur #738 |
| R11 | fiche 32-6 l. 1231-1252 (« Ce qu'elle laisse à la 32-7 ») | route absente, activation par société, filtre et mandataires, `Origin`, seul `Bearer` lu, 1 Mio, 30 s, journal, appel qui écrit, acteur au nom du créateur, `last_used_at`, `KESH_PUBLIC_BASE_URL`, ligne illisible, requêtes anonymes, `Forwarded`/`X-Real-IP`, 415, 400, connexion neuve | AC-18 (1), (2), (4)-(7) ; « une clé `required` ne peut appeler aucun outil » **écartée** (fausse après la 32-8, R16) ; « ligne de réglages illisible → désactivée » : AC-18 (1), à constater à l'étape 0 |
| R12 | fiche 32-6 l. 1002, 1310 (point 2) | Traefik et Docker devant le filtre | AC-6 (1), AC-18 (4) — tranché par Guy (2026-10-25) ; NAS non vérifié (*Ce que les gardes ne voient pas*) |
| R13 | fiche 32-6 l. 1311 (point 2 bis) | garde d'un pair non déclaré porteur d'en-tête | AC-18 (4), selon ce que la 32-6a a fait (étape 0) |
| R14 | fiche 32-6 l. 1015-1018 | `last_used_at` n'est pas « dernier appel » | AC-8 (b), AC-18 (2) |
| R15 | fiche 32-6 l. 1253-1257 | `README.md:233`, `website/roadmap.html:375` (clé dispensée) | **écarté** vers la 32-11 (fiches 32-6, 32-8 l. 1397-1401, 32-10a l. 1390) ; constaté, AC-20 |
| R16 | fiche 32-6 l. 1258-1263 | pas de CORS, `OPTIONS` → 405 | AC-18 (5) |
| R17 | fiche 32-6 l. 1267-1271 (B4) | message de type réponse → 400 | AC-18 (7) |
| R18 | fiche 32-6 l. 1272-1278 (E8) | route absente, 401 unique (dont `Origin`), cookie et PAT non lus, JSON, 1 Mio, `kesh_version`, pas de CORS, `/mcp` exact, en-têtes répétés, 202, erreurs sans `id`, `/0`, sous-chemin | AC-18 (1), (2), (4), (5), (7) ; AC-7 (b) |
| R19 | fiche 32-6 l. 1279-1281 (A3) | `admin-manual.tex:2935`, 128 → 129 | **écarté** : 32-10b puis 32-11 (AC-20) |
| R20 | fiche 32-6 l. 1282-1283, 32-9 l. 1328 | `api-external.md:185`, `:831` | **écarté** : 32-11 (AC-20) |
| R21 | fiche 32-6 l. 1402, 1469 (P7-H3) | entrée invalide des mandataires coupe le serveur, filtre activé ou non | AC-18 (4) |
| R22 | fiche 32-8 l. 917-918, 1406-1408 | montrer la politique, lien vers les envois, ranger `sec:mcp-envois`, retirer la phrase fausse | AC-13, AC-18 (3), (9) |
| R23 | fiche 32-8 l. 1034, 1574 | la 32-8c relit l'état de la 32-7 | **sans objet** ici (ordre : la 32-8c avant la 32-7) |
| R24 | fiche 32-8 l. 1507 (Q12) | compteur d'envois au menu | **écarté** : à Guy (point 3), non fait (C-32-7-15) |
| R25 | fiche 32-9 l. 914, 1315-1325, 1373, 1429 | ranger `sec:mcp-outils` sous *Le serveur MCP* | AC-18 (9) |
| R26 | fiche 32-9 l. 1339-1341, 1373 | à l'écran de création, ce que `contacts:read` démasque ; noms lisibles sans elle ; l'IBAN n'est pas une donnée du contact | AC-10, AC-18 (2) |
| R27 | fiche 32-10a l. 1006-1008, 1386-1387 | portée `prompts:write` (proposer, jamais approuver, ni lire ; une clé `required` peut proposer) | AC-10, AC-18 (2) |
| R28 | fiche 32-10a l. 1008, 1384 | le compte de l'admin l. 2935 à la 32-10b | AC-20 (constaté, non porté) |
| R29 | spike l. 945-946 | configuration des deux clients, limite de 25 000 jetons, `Origin` ne protège que d'un navigateur | AC-18 (5), (8) |
| R30 | spike l. 946-947 (Q11 (b)/(c)) | création à la main d'un utilisateur `SELECT` | **sans objet** : Q11 (a) |
| R31 | spike l. 947-948 (Q10 (b)) | au-delà de 2 min Claude Code poursuit | **sans objet** : Q10 (c), la poignée |
| R32 | epic l. 192 (*Points d'attention*) | le manuel dit ce que le filtre ne garantit pas | AC-18 (4) |
| R33 | consigne de l'orchestrateur | pas de limite de débit, écrit | AC-6 (4), AC-18 (4) |

## Dev Notes

### Le secret : où il est, où il n'est jamais

```
POST /api/v1/admin/mcp/keys ──201 { …clé, key }──► mcp-server.api.ts
                                                   ├─ cle    (champ par champ, sans key) ──► (rien : la liste se recharge par GET)
                                                   └─ secret ──► McpKeysSection : secret.montrer(s)   [instance SecretUnique, locale au composant]
                                                                   └─ McpKeySecretNotice : <code> unique
                                                                        Fermer ──► secret.oublier()  ; démontage ──► instance perdue
```

Jamais : la liste, un `$state` de module, un stockage du navigateur, l'URL, `document.title`, un `toast`, un
`console.*`, un attribut HTML. La classe, indicative :

```ts
// secret-unique.svelte.ts — une instance par composant, jamais au niveau d'un module.
export class SecretUnique {
	#valeur = $state<string | null>(null);
	#nom = $state<string | null>(null);
	montrer(nom: string, valeur: string) { this.#nom = nom; this.#valeur = valeur; }
	lire() { return this.#valeur === null ? null : { nom: this.#nom!, valeur: this.#valeur }; }
	oublier() { this.#valeur = null; this.#nom = null; }
}
```

⚠️ Les champs privés `#` ne paraissent ni à `JSON.stringify` ni à `Object.getOwnPropertyNames` : le test (ii) de
l'AC-11 passerait **à vide** sur eux. Il lit donc l'état **par `lire()`** après `oublier()` (doit rendre `null`), et la
mutation M2 (un drapeau `visible` sans effacement) le fait rougir parce que `lire()` rendrait encore la valeur. Si le
développement choisit des champs publics, les deux contrôles s'appliquent ; s'il choisit des privés, `lire()` est
l'unique preuve, et le Dev Agent Record le dit.

### Les dates (#738 B7 = E6)

`createdAt` d'une clé : `"2026-10-25T10:00:00.123"` (UTC, sans fuseau) ; `updatedAt` des réglages :
`"2026-10-25T10:00:00.123Z"`. `new Date("2026-10-25T10:00:00.123")` est lu **en heure locale** par les navigateurs :
dix heures UTC deviendraient dix heures à Zurich, soit une ou deux heures d'écart. D'où `depuisUtcNaif`, qui ajoute
`Z` à une chaîne sans fuseau (`/[zZ]|[+-]\d{2}:\d{2}$/`). L'expiration va dans l'autre sens : la date choisie est la
**dernière journée de validité en heure locale**, envoyée en UTC par `toISOString()` (le serveur exige un fuseau,
fait 3) ; le refus « expiration passée » du serveur (l. 284-291) ne peut pas toucher le lendemain local.

### Les réglages : ce que l'écran ne fait pas

L'écran **ne valide pas** les adresses : le serveur le fait et traduit ses refus (fait 5, fait 11). Il retire les lignes
vides (sinon chaque ligne vide finale ferait un 400) et calcule des avertissements **informatifs** sur le texte
affiché ; un préfixe se lit après le dernier `/` de l'entrée rognée. Une adresse sans préfixe (`192.168.1.10`) n'est
jamais `/0`. Le remplacement des champs par la réponse (AC-5 (c)) est ce qui rend visible la mise en forme du serveur
— la seule réponse de l'écran à #738 B2 = E4.

### Le manuel : place et forme

`admin-manual.tex` est un `article` : `\section` y est le plus haut niveau. *Le serveur MCP* devient une `\section`
(C-32-7-4), ce qui laisse à la 32-8 (`sec:mcp-envois`) et à la 32-9 (`sec:mcp-outils`) leur niveau `\subsection` :
on les **déplace** sous elle sans les rétrograder, et leurs `\label` ne changent pas — leurs renvois restent justes.
Le déplacement de la sous-section de la 32-9 emporte son bloc `% matrice-mcp:debut`/`fin` ; si le test de la 32-9
ancre ce bloc sur une section (par exemple « sous `\section{Sécurité}` »), il se corrige **dans le même commit**, et
l'écart est déclaré.

### Lignes de code lues et citées

`crates/kesh-api/src/lib.rs:321-340` ; `crates/kesh-api/src/routes/mcp_keys.rs:15-26`, `:52`, `:56-73`, `:93-116`,
`:138-153`, `:182-202`, `:215-225`, `:260-266`, `:270-338`, `:340-372`, `:374-411` ;
`crates/kesh-api/src/routes/mcp_server_settings.rs:4-38`, `:57-61`, `:63-102`, `:111-137`, `:140-167`, `:171-195`,
`:230-232` ; `crates/kesh-db/src/entities/mcp_server_settings.rs:17-40` ; `crates/kesh-db/src/entities/mcp_key.rs:33-63`,
`:256-272` ; `crates/kesh-db/src/repositories/mcp_keys.rs:197-209` ; `crates/kesh-api/tests/mcp_admin_routes_e2e.rs:371`,
`:803`, `:920` ; `crates/kesh-api/tests/textes_coherents.rs:12-27`, `:1673` ;
`crates/kesh-i18n/locales/fr-CH/messages.ftl:1755-1767` ; `frontend/src/routes/(app)/settings/api-keys/+page.svelte:40-42`,
`:73-77`, `:128-130`, `:150-155` ; `frontend/src/lib/features/api-keys/api-keys.api.ts:41-43` ;
`frontend/src/routes/(app)/admin/backup/+page.ts:11-17` ; `frontend/src/routes/(app)/+layout.svelte:147-170`, `:212-215` ;
`frontend/scripts/lint-i18n-ownership.js:177-186` ; `frontend/src/lib/shared/utils/i18n.svelte.ts:18-28` ;
`frontend/src/lib/shared/utils/clipboard.ts:44-47` ; `frontend/src/lib/shared/utils/error-message.ts:48` ;
`frontend/src/lib/shared/i18n-literal-reader.js:261` ;
`frontend/src/lib/components/ui/button/boutons-rouges-inventory.test.ts:207` ;
`docs/manual/fr/admin-manual.tex:1414`, `:1416`, `:2237`, `:2463`, `:2467`, `:2472`, `:2862`, `:2898`, `:2925`,
`:2935`, `:2990`, `:3017` ; `docs/manual/fr/user-manual.tex:190`, `:3029-3030`, `:3204-3205`, `:3214-3215` ;
`docs/api-external.md:29-35`, `:183-185`, `:831` ; `CHANGELOG.md:11`.

### Fichiers touchés (prévus)

`frontend/src/lib/features/mcp-server/**` (neuf) ; `frontend/src/routes/(app)/admin/mcp/{+page.ts, +page.svelte,
admin-mcp-page.test.ts}` (neufs) ; `frontend/src/routes/(app)/+layout.svelte` ; `frontend/tests/e2e/mcp-server.spec.ts`
(neuf) ; `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` ; `crates/kesh-api/tests/textes_coherents.rs` ;
`docs/manual/fr/{admin-manual,user-manual}.{tex,pdf}` ; `CHANGELOG.md` ; et, si le déplacement l'exige, le test de la
matrice de la 32-9 (écart déclaré).

### Références

`planning-artifacts/epic-32-serveur-mcp.md` (§ *32-7*, *Points d'attention*, *Réponses de Guy* des 2026-10-24 et
2026-10-25) ; `planning-artifacts/epic-32-spike-mcp.md` (§ 2.1, 2.4, 6, *Réponses*) ; fiches `32-5-cles-et-reglages-mcp.md`,
`32-6-endpoint-mcp.md`, `32-8-validation-avant-envoi.md`, `32-9-outils-de-lecture.md`, `32-10a-prompts-en-base.md` ;
registres `-325.md` (C-32-5-10, 11, 30, 37, 48) ; #738 ; `CLAUDE.md` (*Recette visuelle*, *Review Iteration Rule*, *La
passe ciblée*, *Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites NON RÉSOLUS*, *Règle de splitting
préventif*, *E2E*, *Recompter*) ; `docs/agents-orchestration.md` ; `docs/testing.md`.

## Points à soumettre à Guy

1. **Aucune portée cochée par défaut** à la création (C-32-7-6) : chaque clé se compose consciemment. *Alternative* :
   `books:read` coché d'office. *Recommandation* : aucune.
2. **La révocation est offerte aussi pour une clé expirée** (C-32-7-9) : une clé expirée n'authentifie plus, mais
   l'écrire révoquée la ferme aussi contre une restauration qui changerait la date. *Recommandation* : l'offrir.
3. **Un compteur des envois en attente au menu** (question 12 de la fiche 32-8, sans réponse) : la 32-7 n'en pose pas
   (C-32-7-15). *Recommandation* : en décider après la recette de la 0.16.0.
4. **Le créateur d'une clé n'est pas affiché** (la réponse n'en porte que le numéro, C-32-7-16) ; une seule personne
   administre aujourd'hui. *Recommandation* : l'accepter ; l'ajouter demanderait de toucher la route.
5. **La configuration d'un client au manuel** (C-32-7-13) : la 32-7 écrit les gestes (Claude Code, Hermes) **sans les
   éprouver** ; la 32-11 les rejoue. La commande de Claude Code enregistre la clé **en clair** dans `~/.claude.json` :
   c'est le fonctionnement de Claude Code, que le manuel dit.
6. **Pour information** : la capture du secret (recette visuelle) montre une clé de démonstration, révoquée aussitôt,
   et reste hors du dépôt.

## Demandes à l'orchestrateur

1. **Le propriétaire du compte de `admin-manual.tex:2935`** : la fiche 32-6 (A3) le donne à la 32-11, la fiche 32-10a à
   la 32-10b. La 32-7 n'ajoute aucune route et ne le porte pas (AC-20). *Proposition* : la 32-10b le recompte après
   ses routes, la 32-11 au dernier état ; l'inscrire aux deux sections de l'epic.
2. **La fiche 32-10b** : ranger *Les prompts du serveur MCP* sous `\section{Le serveur MCP}` (`sec:mcp`) si la 32-7 a
   fusionné avant elle.
3. **Le point 2 bis de la fiche 32-6** (garde du pair non déclaré porteur d'en-tête) : la 32-7 décrit ce que la 32-6a a
   fait ; si Guy ne l'a pas tranché, le lui reposer avant la 32-7.
4. **Le numéro G23** de `textes_coherents.rs`, réservé à la 32-7 par cette fiche.
5. **L'ordre** : la 32-7 en dernier des stories citées (C-32-7-3) ; si l'écran doit venir plus tôt, découper selon
   l'ordre de rechange (32-7a, 32-7b), sans réécrire les AC.

## À recontrôler à l'étape 0 du développement

- **Fusions** : 32-6a, 32-6b, 32-8 (a, b0, b, c), 32-9a, 32-9b, 32-10a — `git log --oneline origin/main` ; état de la
  32-10b. Ce que chaque story a **réellement** livré, contre ce que cette fiche prescrit d'en écrire (AC-10, AC-13 (c),
  AC-18) : chaque phrase, ligne lue citée ; une phrase non confirmée se réécrit.
- **Les routes de la 32-5c** (fait 1 à 5) inchangées ; un champ ajouté à `McpKeyResponse` par une story suivante entre
  à l'AC-3 (a).
- **`/admin/mcp/sends`** (32-8c) et `nav-admin-mcp-sends` : présents ? (AC-2, AC-13 (d)) ; les sous-dossiers de
  `routes/(app)/admin/mcp/` d'autres stories, exclus de G-a.
- **`seedTestState`** : vide-t-il `mcp_keys` et `mcp_server_settings` ? Quel utilisateur Comptable le préréglage
  fournit-il ? (AC-16).
- **Le 401 fixe** de la 32-6a (constante de `mcp/textes.rs`) pour l'AC-16 (e) ; la réponse d'`initialize` (200) sans
  `Origin`, depuis `127.0.0.1`.
- **`last_used_at`** : ce que pose `services::cles_mcp::authentifier` et quand (AC-8 (b)).
- **Le point 2 bis** de la 32-6 (AC-18 (4)) ; la « ligne de réglages illisible » (fiche 32-6, AC-a2) ; les valeurs de
  l'AC-18 (7) (415, 400, 1 Mio, 30 s, 202, 400 d'une réponse).
- **La place de `sec:mcp-outils` et `sec:mcp-envois`** dans le manuel, et l'ancrage du test de la matrice de la 32-9.
- **Les lignes du manuel** (fait 16, 17 ; AC-19, AC-20) : recomptées, la 32-8c ayant amendé l. 1414, 2230, 2237, 2249
  et `user-manual.tex:3027-3033`.
- **La garde frontend** qui contrôle qu'un repli égale la valeur fr-CH (AC-14) : la nommer.
- **G23** libre (AC-25) ; la mesure de G-a et G-b refaite (AC-15).
- **Les échecs E2E attendus** : `docs/testing.md` § *Les échecs attendus*, à la date de l'étape 0.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

| Date | Étape | Résumé |
|---|---|---|
| 2026-10-25 | création | Fiche rédigée en autonomie (Opus 5.5), contre `966baed5`. 25 AC (A : 17, B : 7, C : 1), 9 tâches, 19 mutations, gardes G-a, G-b, G23 ; 33 reports inventoriés (R1 à R33 : 24 rattachés à un AC, 9 écartés ou sans objet avec leur raison — R9, R10, R15, R19, R20, R23, R24, R30, R31) ; choix C-32-7-1 à C-32-7-20 ; 4 modules, pas de découpage. Prompt de validation P1 : `327-validate-prompt-p1.md`. |
