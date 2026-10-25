# Story 32.7 : l'écran « Serveur MCP » (Administration) et la section du manuel

Status: ready-for-dev

*Fiche rédigée le 2026-10-25, en autonomie (Opus 5.5), worktree `kesh-32-7`, branche `story/32-7` partie de `main`
`966baed5` (toute la 32-5 fusionnée ; 32-6, 32-8, 32-9 et 32-10a non fusionnées). Registre :
`epic-32-choix-autonomes-327.md` (C-32-7-1 à C-32-7-25 ; C-32-7-21 et 22 ajoutés en validation P1, C-32-7-23 et 24 en validation P2, C-32-7-25 en validation P3). Clé courte : `327`. Issue de l'epic : #723 (CR « Serveur MCP
intégré à Kesh »), en `refs` seulement. Les numéros de ligne cités sont ceux de `966baed5`, sauf mention d'une fiche
voisine (lue dans son worktree, en lecture seule, à la date de rédaction).*

⛔ **Développement APRÈS la fusion de ses prérequis, dans l'ordre nominal : 32-6a, 32-6b, 32-8a, 32-8b0, 32-8b, 32-8c,
32-9a, 32-9b, 32-10a, puis la 32-7** (C-32-7-3, récrit en validation P1). Le code de l'écran ne dépend que de la 32-5 ;
mais ses textes et la section du manuel décrivent **ce que fait le code** de ces stories — le comportement de `/mcp`,
le 401 fixe, le journal des appels, la validation avant envoi et son écran, les outils et ce que chaque portée
démasque, la portée `prompts:write` —, et la section **range** sous elle les sous-sections que la 32-8
(`sec:mcp-envois`) et la 32-9 (`sec:mcp-outils`) auront écrites. **L'étape 0 le vérifie** : `git log --oneline
origin/main` porte **chacune** des neuf stories ; s'il en manque une, le développement **ne commence pas** et l'agent le
dit à l'orchestrateur — aucun AC de cette fiche n'a de variante « si la story n'a pas fusionné ». **Tout ce qui, dans
cette fiche, décrit le code de ces stories se recontrôle à l'étape 0** (§ *À recontrôler à l'étape 0*), ligne lue citée
au Dev Agent Record ; une phrase prescrite ici que le code ne confirme pas **se réécrit selon le code**, elle ne se garde
pas.

Les AC sont rangés en **trois groupes** pour la lecture — A (l'écran, AC-1 à AC-17), B (le manuel et les textes, AC-18 à
AC-24), C (les gardes de cohérence, AC-25) — et se livrent ensemble, en une seule clé (C-32-7-1). *(L'ordre de rechange
écrit à la création — un `32-7a` sur la seule 32-5 — est retiré en validation P1, décision de l'orchestrateur : F1 avait
montré que ses trois exceptions n'étaient pas les seules dépendances du groupe A.)*

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
  (corps mal formé : refus par défaut d'axum, non traduit) ; **B8** (`version: -1` au `PUT` des réglages → 409 ; **une
  ligne de réglages corrompue rend 500 au `GET` comme au `PUT`** — seconde moitié, rattachée en validation P2, R3).
- **Fiches voisines** (inventaire complet au § *Inventaire des reports*) : 32-6 (l. 1002, 1018, 1231-1283, 1310, 1402,
  1469, 1558 à la rédaction ; à `dd4b8398`, la liste de la 32-7 est aux l. 1252-1300 et P7-H3 aux l. 1422-1428 —
  validation P2, R13), 32-8 (l. 916-918, 1034, 1397, 1406-1408, 1507), 32-9 (l. 914, 1315-1325, 1339-1341, 1373, 1429),
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
19. **CHANGELOG** : `## [0.16.0] — Non publié` (l. 11), sections `### Modifié` (l. 13) puis `### Corrigé` (l. 24, #714) ;
    aucune `### Ajouté` encore à `966baed5` (les 32-8 et 32-9 en ajoutent une). L'ordre du dépôt est celui de la 0.15.0 :
    `### Ajouté` (l. 59), `### Modifié` (l. 111), `### Corrigé` (l. 134). *(Corrigé en validation P1, R3 : la fiche disait
    « `### Modifié` seule ».)*
20. **Le garde des textes** : `crates/kesh-api/tests/textes_coherents.rs`, gardes G2 à G22 (l. 12-27) ; G22 (l. 1673)
    est le précédent d'une cohérence **manuel ↔ écran ↔ quatre locales**. Aucune fiche voisine ne réserve G23
    (`grep -n 'G23' kesh-32-*/…/32-*.md` → rien au 2026-10-25).
21. **La garde des clés demandées** (`frontend/src/lib/shared/i18n-keys.test.ts` ; ajouté en validation P2, F1 = R1) :
    - les clés de menu sont des **données** (`{ i18nKey, fallback }`), résolues par `i18nMsg(item.i18nKey,
      item.fallback)` (`+layout.svelte:174-176`) — un site **non littéral**, que `replisParCle()` de
      `i18n-repli-divergent-actif.test.ts` écarte (`site.arg?.kind !== 'literal'`, l. 122 — « l. 121 » jusqu'en
      validation P3, F3) ; leurs clés sont déclarées
      à `FAMILLES_RESOLUES['nav-']` (`i18n-keys.test.ts:858-868`, dont `'admin-backup'`, `'admin-restore'`,
      `'audit-log'`) ;
    - ce que cette famille **vérifie**, et seulement cela : chaque clé déclarée existe au catalogue **fr-CH**
      (`clesDemandees`, l. 1008-1016, et le test « toute clé demandée existe au catalogue », l. 1037-1045), et une
      famille n'est pas vide (l. 1127-1135). Elle ne lit **aucun repli** : rien n'y compare `'Serveur MCP'` à la valeur
      fr-CH ; et `'nav-'` n'est pas un préfixe à couverture close (`PREFIXES_A_COUVERTURE_CLOSE = ['contact-duplicate-',
      'settings-invoicing-']`, l. 932 ; test l. 1153-1163), si bien qu'une valeur oubliée à `FAMILLES_RESOLUES` ne
      rougit pas non plus ;
    - le même fichier tient le compte **exact** des sites d'appel : `sitesTotal: 2334` (l. 724), assertion
      `expect(releve.sitesTotal).toBe(ATTENDU.sitesTotal)` (l. 1034), chaque story ajoutant son décompte, commenté et
      recompté aux deux bornes (précédent : la 32-1b, l. 715-723) ; `sitesNonResolus: 31` (l. 725) ne bouge que si un
      site non littéral apparaît — une entrée de menu n'en ajoute aucun (précédent de l'audit, l. 253-256) ;
    - **aucune clé `mcp-server-` n'existe encore** côté frontend, ni n'est résolue par une table : `grep -rnF
      'mcp-server-' frontend/src` → rien ; `MOTIFS_DYNAMIQUES` et `FAMILLES_RESOLUES` n'ont pas de préfixe
      `mcp-server-`. ⚠️ Le catalogue porte en revanche **deux** clés qui **contiennent** la chaîne sans **commencer**
      par elle : `audit-log-entity-mcp-server-settings` (fr-CH l. 2639) et `audit-log-action-mcp-server-settings-updated`
      (l. 2752), dans les quatre locales.

## Le découpage (règle de découpage préventif)

Modules, au grain du kickoff : **frontend** (`features/mcp-server/` neuf, `routes/(app)/admin/mcp/` neuf,
`+layout.svelte`, `tests/e2e/mcp-server.spec.ts`), **`kesh-i18n`** (quatre catalogues), **docs** (`admin-manual`,
`user-manual`, leurs PDF, `CHANGELOG.md`), **`kesh-api`** (le seul fichier de test `tests/textes_coherents.rs`, garde
G23, AC-25) — **4**, au plus cinq : **pas de découpage** (C-32-7-1). Aucune migration, aucun dépôt, `kesh-db` non
touché : l'exception `kesh-db` de *Pendant une boucle de revue* ne s'applique pas. Les deux gardes frontend partagées
qu'amende la story (`i18n-repli-divergent-actif.test.ts`, AC-14 (a) ; `i18n-keys.test.ts`, AC-14 (d) — validation P2,
F15) et le commentaire de G22 restent dans ces modules (frontend, `kesh-api/tests`). L'ordre de développement est nominal (tête de fiche) ; aucune rechange.

## Critères d'acceptation

Conventions : feature `frontend/src/lib/features/mcp-server/` — `mcp-server.api.ts`, `mcp-server.types.ts`,
`mcp-time.ts`, `secret-unique.svelte.ts`, `McpServerPanel.svelte`, `McpSettingsSection.svelte`,
`McpKeysSection.svelte`, `McpKeySecretNotice.svelte` (noms indicatifs ; la répartition en composants est libre, **les
propriétés ne le sont pas**) ; tests Vitest à côté (`*.test.ts`) ; route `frontend/src/routes/(app)/admin/mcp/`. Tous
les libellés par `i18nMsg('mcp-server-…', '<repli fr-CH>')`, **clé et repli littéraux** (jamais une clé bâtie par
gabarit : la garde de l'AC-14 ne verrait pas une clé dynamique). Tous les `data-testid` de la feature préfixés
`mcp-server-` — **sauf** le lien de menu, dont le `data-testid` est dérivé de son `href` par `navTestid`
(`+layout.svelte:267-270`) : `nav-link-admin-mcp`, distinct de `nav-link-admin-mcp-sends` (32-8c). « Vert »
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
   monte `McpServerPanel`. Test Vitest du `load`, **sous `vi.mock('$app/environment', () => ({ browser: true }))`**
   (le `load` ne redirige que si `browser` est vrai, `admin/backup/+page.ts:14` ; précédent
   `contacts/contacts-page.test.ts:17`) : `Comptable` et `Consultation` → redirection 302 vers `/` ; `Admin` → aucune.
   **Témoin** : dans le même fichier, le même `load` lève bien la redirection pour un Comptable — sans quoi le cas
   `Admin` passerait à vide sous un `browser` resté faux. Le serveur reste l'autorité (fait 1) : la garde est une garde
   d'écran.
2. **AC-2 — la navigation.** Entrée `adminOnly` `{ i18nKey: 'nav-admin-mcp', fallback: 'Serveur MCP', href:
   '/admin/mcp' }` au groupe *Administration* (`+layout.svelte:147-170`), **immédiatement avant** l'entrée
   `nav-admin-mcp-sends` que la 32-8c a posée (prérequis fusionné ; sa place relue à l'étape 0). Un Comptable ne la
   voit pas (E2E, AC-16 (f)).
3. **AC-3 — la couche d'API sépare le secret de la clé** (C-32-7-5). `mcp-server.api.ts` expose `listMcpKeys`,
   `createMcpKey`, `revokeMcpKey(id, version)` (`DELETE` avec corps `{ version }`, motif `api-keys.api.ts:41-43`),
   `setMcpKeySendValidation(id, sendValidation, version)`, `getMcpServerSettings`, `updateMcpServerSettings`.
   - (a) `createMcpKey` rend **`{ cle: McpKey; secret: string }`**, où `cle` est construite **champ par champ** depuis
     les onze champs de `McpKeyResponse` (fait 2), **jamais** par décomposition (`{ ...res }`) ni par `delete res.key` ;
     le type `McpKey` n'a **pas** de champ `key`. Test `createMcpKey_separe_le_secret` : `apiClient.post` simulé rend
     un objet portant `key: 'kesh_mcp_SECRET…'` et un champ en trop `zzz` ; `'key' in cle === false`,
     `'zzz' in cle === false`, aucune valeur de `cle` (`Object.values`, récursivement) n'égale le secret,
     `secret === 'kesh_mcp_SECRET…'`. *(Le secret simulé s'écrit `kesh_mcp_SECRET…` dans tous les tests de la story, et
     les témoins d'espion `kesh_mcp_TEMOIN` — deux littéraux dont aucun ne contient l'autre ; validation P2, F8.)*
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
   - Tests `mcp-time.test.ts`, sous `process.env.TZ = 'Europe/Zurich'` posé dans le fichier et rétabli après
     (précédent au dépôt : `routes/(app)/bank-accounts/bank-accounts-page.test.ts:701-720`, `try`/`finally`), avec un
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
   - (g) **Le chargement des réglages** (validation P1, R12 = F6) : pendant le `GET`, *Enregistrer* est désactivé ;
     échec du `GET` → le `message` de l'`ApiError` dans la section (`role="alert"`), sinon le générique, et
     *Enregistrer* **reste désactivé** — sans `version` lue, rien ne s'envoie (b). L'état d'échec porte en outre une
     **phrase fixe** qui dit ce qu'il signifie (validation P2, R3 ; C-32-7-24) : « Les réglages n'ont pas pu être lus :
     tant qu'ils ne le sont pas, rien ne s'enregistre depuis cet écran. Si l'erreur persiste, voir le manuel
     d'administration, § *Activer le serveur*. » — c'est le cas d'une **ligne de réglages corrompue** : un tableau qui
     ne contient pas que des chaînes (`[1]`) — la base garantit un tableau JSON **valide** (`CHECK (JSON_VALID(…) AND
     JSON_TYPE(…) = 'ARRAY')`, migration `20261025000001_mcp_keys_and_server_settings.sql:81-83`), mais `decode_list`
     attend des chaînes (`kesh-db/src/repositories/mcp_server_settings.rs:40-45`) et rend `DbError::Invariant`,
     traduit en **500** `INTERNAL_ERROR`, `kesh-api/src/errors.rs:4270-4276` ; #738 B8 ; validation P3, F6). Test : la
     phrase présente à l'échec du `GET` (500 simulé), absente au succès (témoin).
   - (h) **Une phrase fixe sous *Origines admises*** (validation P2, F14 ; fiche 32-6 à `dd4b8398`, l. 1279-1283) :
     « Une origine admise n'ouvre le serveur à aucune page web : elle ne sert qu'aux clients qui envoient un en-tête
     `Origin` sans être des navigateurs. » Test : la phrase rendue, cherchée par son repli fr-CH.
   - Tests `McpSettingsSection.test.ts` : un cas par lettre, corps envoyé comparé **en entier**.
   - ⚠️ **Ce que l'écran ne répare pas** — deux états de la ligne de réglages posés hors de l'écran :
     - une ligne à `version = 0` **réelle** (SQL à la main, sauvegarde fabriquée ; C-32-5-53,
       `routes/mcp_server_settings.rs:35-38`) est rendue telle quelle par le `GET`, et chaque `PUT` à `version: 0` rend
       alors 409 — (e) recharge, et l'enregistrement suivant rend encore 409. La boucle n'est pas automatique (elle suit
       chaque clic) ;
     - une ligne **corrompue** (un tableau qui ne contient pas que des chaînes, (g)) rend 500 au `GET` : (g) s'affiche, *Enregistrer* reste
       désactivé (validation P2, R3 ; #738 B8, seconde moitié) ; et même un `PUT` qui porterait la bonne `version`
       rendrait 500 : la route relit l'« avant » de l'audit par `get_or_defaults` avant d'écrire
       (`routes/mcp_server_settings.rs:245`) — la ligne ne se réécrit donc pas non plus par l'API.
     Ni l'un ni l'autre ne se répare **depuis l'écran**, ni par une action ciblée de Kesh, sur une installation de
     production (C-32-7-24, annoté en validation P3, F2) : la remise à zéro
     efface bien la ligne (`kesh-db/src/backup.rs:105-116`, `mcp_server_settings` hors de `RESET_PRESERVED_TABLES`),
     mais elle est **refusée** au-delà de l'étape 2 de l'assistant (`kesh-api/src/routes/onboarding.rs:255-268` :
     « Une installation de production n'est jamais remise à zéro au-delà de l'étape 2 ») ; la réparation au démarrage
     (#528, 32-5d) ne touche pas `mcp_server_settings` (`grep -rn 'mcp_server_settings' crates --include=*.rs` hors
     entité, dépôt, route et tests : `lib.rs`, `audit_labels.rs`, `exports/global.rs`, `backup.rs`, `post_restore.rs`
     seulement — aucun dans `kesh-seed` ni dans la réparation). Restent, **hors de l'écran**, dans cet ordre
     (validation P3, F2) : d'abord **supprimer la ligne par SQL**, la voie la moins coûteuse (le `GET` rend alors les
     défauts à `version: 0`, fait 5 ; serveur désactivé) ; en dernier recours, **restaurer une sauvegarde antérieure**,
     qui remplace **toute** l'installation, non la seule ligne (`admin-manual.tex:2262`), et refuse une sauvegarde
     d'une version qui ne porte pas les mêmes tables — celles de la 0.15.0 et d'avant (`admin-manual.tex:2249`). Le
     manuel le dit (AC-18 (1)) ; l'écran n'a pas de cas propre au-delà de (e) et (g).
     Un troisième état, lui, **se répare à l'écran** (validation P3, F7) : une entrée invalide de *Mandataires de
     confiance* posée hors de l'écran (sauvegarde, SQL) coupe le serveur de la société, filtre activé ou non (P7-H3,
     AC-18 (4)), alors que la case *Serveur activé* reste cochée — le `GET` la rend telle quelle (une chaîne se décode).
     Tant qu'elle est dans la zone, *Enregistrer* rend le 400 du serveur qui la nomme (`validate_settings`,
     `routes/mcp_server_settings.rs:198-208`, (d)) ; la retirer et enregistrer rétablit le serveur. L'écran ne
     signale pas l'entrée avant l'enregistrement (*Ce que les gardes ne voient pas*).
6. **AC-6 — ce que le filtre ne garantit pas, écrit à l'écran** (décision de Guy, 2026-10-25). Un encadré
   **permanent** de la section, quatre phrases au moins, dans les quatre locales :
   - (1) le filtre juge l'**adresse que voit Kesh** ; derrière **Traefik** non déclaré parmi les mandataires, tous les
     clients paraissent venir de lui ; derrière une publication de port **Docker** par le mandataire en espace
     utilisateur, tous peuvent paraître venir de la passerelle Docker — dans les deux cas une adresse privée, admise par
     défaut : **le filtre n'arrête alors personne** (même **sens** que le manuel, AC-18 (4) — « le filtre ne filtre alors
     rien » —, et que la fiche 32-6, point 2 ; les formulations diffèrent, et seule G23 (c) en tient une part,
     `Traefik` et `Docker` ; validation P1, R5 — une publication de port ordinaire conserve l'adresse source ;
     validation P2, R6) ;
   - (2) il ne remplace **ni le pare-feu ni le VPN** : Kesh ne doit jamais être exposé sur Internet ; l'accès de
     l'extérieur passe par un VPN ;
   - (3) la plage **`100.64.0.0/10`** (VPN de type Tailscale) **n'est pas admise par défaut** : l'ajouter soi-même aux
     plages admises (C-32-5-10) ;
   - (4) **aucune limite de débit** n'est posée sur `/mcp`.
   Puis des **avertissements conditionnels**, informatifs, jamais bloquants (C-32-7-12), calculés sur les champs
   **tels qu'affichés** :
   - (a) filtre désactivé → « toute adresse est admise » ;
   - (b) une plage admise de préfixe `/0` (toute entrée dont le préfixe, rogné, vaut 0) → un avertissement qui
     **nomme la famille** de l'entrée (IPv6 si elle contient `:`, IPv4 sinon) : « `0.0.0.0/0` admet tout client
     IPv4 » ou « `::/0` admet tout client IPv6 », suivi de « pour admettre les deux familles, il faut les deux
     entrées » (#738 B3 = E5 ; validation P3, R1 — la formulation « toute adresse, comme un filtre désactivé » était
     fausse). *Fait sourcé* (code de la 32-6a, `kesh-32-6`, branche `story/32-6`, PR #760) : le pair est ramené à sa
     forme canonique avant tout contrôle (`transport.rs:81`, `reseau.rs:64` et `:73` pour les entrées de
     `X-Forwarded-For` — `IpAddr::to_canonical`, doc `reseau.rs:9-12` : une adresse IPv4 mappée `::ffff:a.b.c.d`
     devient IPv4), puis chaque réseau est comparé par `IpNet::contains` (`reseau.rs:57`, `:128`), qui rend **faux**
     entre familles différentes (`ipnet` 2.12.0, version de `Cargo.lock:1872-1873`, `src/ipnet.rs:1419-1426`). Une
     adresse IPv4 mappée compte donc comme **IPv4** : `::/0` ne l'admet pas, `0.0.0.0/0` l'admet ;
   - (c) un **mandataire** de préfixe `/0` → même règle de famille : « `0.0.0.0/0` : tout pair IPv4 est cru sur son
     en-tête `X-Forwarded-For` » ou « `::/0` : tout pair IPv6 … » (validation P3, R1) ;
   - (d) filtre activé et aucune plage admise → « personne n'est admis ».
   Tests : chaque avertissement **présent** quand sa condition l'est, **absent** sinon (témoin) ; pour (b) et (c),
   `0.0.0.0/0` seul rend la formulation IPv4 et non l'IPv6, `::/0` seul l'IPv6 et non l'IPv4, et aucune ne dit
   « toute adresse » sans famille (validation P3, R1) ; l'encadré (1)-(4)
   présent dans tous les états, **chacune des quatre phrases** cherchée par son repli fr-CH (mutation M25) ;
   `100.64.0.0/10` dans le texte rendu.
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
   Test : (a) suit la case **enregistrée** (non la case cochée avant l'envoi : case cochée, aucun envoi, la phrase
   reste celle du serveur désactivé — mutation M24) ; (b) rend `http://localhost:3000/mcp`
   sous `window.location.origin` simulé ; (c) et (d) présents.
8. **AC-8 — la liste des clés.** Colonnes : nom ; portées (le **code** `books:read` tel quel, chacune) ; politique
   (*Requise* / *Dispensée*) ; créée le ; expire le (ou « jamais ») ; **dernière utilisation** ; statut (*Active*,
   *Expirée le …*, *Révoquée le …*) ; actions. Ordre du serveur (fait 2). Une clé révoquée ou expirée est atténuée.
   - (a) `unknownScopes` non vides → chaque portée affichée avec « inconnue de cette version — jamais accordée » ; une
     clé qui n'a **que** des portées inconnues porte « n'ouvre aucun accès » (C-32-5-37).
   - (b) **La dernière utilisation** porte une note (infobulle ou phrase sous le tableau) : « posée chaque fois que la
     clé est **reconnue**, même si l'appel est ensuite refusé ou n'est pas servi » (validation P2, F13 : une clé valide
     d'une société qui ne sert pas, ou présentée sous une origine que sa société n'admet pas, reçoit le 401 d'une clé
     inconnue **et** voit sa dernière utilisation posée — fiche 32-6 à `dd4b8398`, C-32-6-52 « Ce qui reste ») —
     jamais le libellé « dernier appel »
     (32-6, l. 1015-1018 ; la formulation exacte se recontrôle à l'étape 0 contre `services::cles_mcp::authentifier`
     et la 32-6a). « Reconnue » s'entend après les contrôles du créateur : une clé dont le créateur est désactivé ou
     n'est plus Administrateur est refusée **avant** la pose (`services/cles_mcp.rs:94-100`).
     ⚠️ **Le statut *Active* ne dit que « ni révoquée ni expirée »** (validation P3, F1 ; C-32-7-25) : une clé n'authentifie
     que tant que son créateur est **actif et Administrateur**, relus à chaque requête (`services/cles_mcp.rs:20-21`,
     `:47` `ROLE_CLE_MCP = Role::Admin`, `:94-99`) ; désactiver ce compte ou lui retirer le rôle coupe ses clés sans
     les révoquer — l'agent reçoit le 401 d'une clé inconnue, la dernière utilisation ne bouge plus, et **l'écran montre
     toujours la clé *Active*** : la liste ne porte ni l'état ni le rôle du créateur (`list_by_company`,
     `kesh-db/src/repositories/mcp_keys.rs:197-217`, sans jointure ; fait 2). L'écran ne le dit pas ; le manuel le
     dit (AC-18 (2)), et *Ce que les gardes ne voient pas* l'écrit.
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
     (jamais complétée par l'objet de création). *Assertion* (validation P1, R10) : `listMcpKeys` est appelé **deux**
     fois (au montage, puis après le 201), et la liste affichée est celle du **second** `GET`, simulé pour rendre une
     clé (`id`, `name`) que le `POST` n'a **pas** rendue — l'objet de création ajouté à la liste (mutation M4) ferait
     paraître le nom du `POST`, non celui du `GET`.
   - (e) **Les valeurs par défaut décidées** (Q12, C-32-7-6 ; validation P1, F4) — test
     `formulaire_a_l_ouverture_requise_et_aucune_portee` : à l'ouverture, le bouton *Requise* est coché et *Dispensée*
     ne l'est pas, **aucune** des cinq cases de portée n'est cochée ; puis, une portée cochée et un nom saisi sans
     toucher à la politique, le corps envoyé porte `sendValidation: 'required'` (comparé en entier). Le même test,
     rouvrant le formulaire après une création réussie, retrouve ces défauts (remise à zéro de (d)). Mutations M20
     (`exempt` coché par défaut) et M21 (`books:read` précochée).
   - Tests : (a), (b), (c), (d), (e), et le corps envoyé (AC-3 (b)).
10. **AC-10 — la phrase de chaque portée** (Q5 ; décision de Guy sur les tiers, 2026-10-25 ; fiches 32-9 l. 1339-1341
    et 32-10a l. 1386-1387). Sous la case, dans les quatre locales, **telle que le code des 32-9a, 32-9b et 32-10a la
    rend vraie** — formulation de départ, à confronter au code à l'étape 0 (ligne lue citée ; une phrase que le code ne
    confirme pas se réécrit) :
    - `books:read` — « le plan comptable, les exercices, les écritures, le grand livre, la balance, le bilan, le compte
      de résultat et les postes ouverts » ;
    - `invoices:read` — « les factures clientes et fournisseur, l'échéancier et la balance âgée ; les **noms des
      tiers** sont lisibles, mais l'IBAN, le QR-IBAN du créancier et l'adresse d'envoi sont **masqués** sans
      `contacts:read` » (l'échéancier : `kesh_echeancier`, fiche 32-9 l. 700 ; validation P1, R4 (b)) ;
    - `contacts:read` — « le carnet d'adresses — des **données personnelles** — ; elle **démasque** aussi, dans les
      factures, l'IBAN et le QR-IBAN du créancier et l'adresse d'envoi ; les noms des tiers se lisent sans elle.
      L'IBAN n'est pas une donnée du carnet d'adresses. » — les **trois** phrases que la fiche
      32-9 (l. 1339-1341) demande à l'écran, la troisième **écrite à l'écran** et non seulement vérifiée
      (`contacts.rs:380-381` selon la 32-9, relu à l'étape 0 ; validation P1, R4 (c)) ;
    - `documents:read` — « le nom du fichier, le type et l'état de chaque pièce importée, **sans son contenu** »
      (`kesh_pieces` rend `nomFichier`, `typeMime`, `statut` et `qrLu`, ni rattachement ni contenu, fiche 32-9
      l. 328-332 ; validation P1, R4 (a) — « et de leurs rattachements » retiré) ;
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
    - le secret est tenu par **une instance** de la classe `SecretUnique` (`secret-unique.svelte.ts`), de signature
      **fixée** (validation P1, F9) : `montrer(nom: string, valeur: string)`, `lire(): { nom: string; valeur: string }
      | null`, `oublier()` — l'encart a besoin du nom (« Clé « {nom} » créée ») ; l'instance est créée **dans le bloc
      `<script>` d'instance du composant** — jamais au niveau d'un module, ni dans un `<script module>` (G-b) ; *Fermer*
      appelle `oublier()` ; une **nouvelle soumission** du formulaire appelle `oublier()` **avant l'envoi du `POST`**
      — non au seul succès, avant `montrer()` : après un 400, l'ancien secret ne reste pas affiché (validation P2,
      F9) ; quitter la page détruit le composant et son instance.
    Les cinq « ni », chacun son test :
    - (i) **le DOM** — pendant l'affichage, `document.body` contient le secret dans **exactement un** nœud texte ;
      après *Fermer*, **aucun** (`document.body.outerHTML` ne le contient pas), ni après une nouvelle création qui a
      échoué, ni après un rechargement de la liste ;
    - (ii) **l'état** — `secret-unique.test.ts` : après `oublier()`, `lire() === null` et ni `JSON.stringify` de
      l'instance ni aucune propriété propre (`Object.getOwnPropertyNames`, valeurs lues) ne contient le secret ; le test
      du composant, la classe espionnée, vérifie que *Fermer* appelle `oublier()` — **c'est cette sonde, et elle
      seule, qui tue M1** : un encart masqué qui sort du DOM laisse (i) vert (validation P1, F9) ;
    - (iii) **la liste** — AC-3 (a), et aucune ligne du tableau ne contient le secret après création ;
    - (iv) **un journal** — `console.log`, `info`, `warn`, `error`, `debug`, `trace` espionnés pendant tout le
      parcours (création, copie, fermeture, échec simulé d'une seconde création) : aucun argument, converti en texte
      (`String` et `JSON.stringify`), ne contient le secret ; **`toast` lui-même** espionné, et non ses seules méthodes
      (validation P2, R5 : `svelte-sonner` exporte `toast` comme une fonction appelable **et** porteuse de méthodes —
      espionner `toast.success`, `toast.info`… laisserait passer `toast(secret)`) : `vi.mock('svelte-sonner', …)` rend
      un `toast` qui est un `vi.fn()` portant **toutes les entrées de l'export réel**, chacune un `vi.fn()` (validation
      P3, R3 = F4 : la liste fermée de neuf méthodes omettait la dixième) — à la rédaction, `success`, `info`,
      `warning`, `error`, `custom`, `message`, `promise`, `dismiss`, `loading` et `getActiveToasts` (simulée
      `vi.fn(() => [])` ; elle ne reçoit aucun argument et ne peut porter le secret), `svelte-sonner` 1.1.0,
      `dist/toast-state.svelte.js:213-226`, `Object.assign(basicToast, {…})`, lu dans les `node_modules` du dépôt
      principal ; l'ensemble des entrées se **relit au paquet du worktree à l'étape 0**, et le simulacre les reprend
      toutes ; un toast reste **permis** (succès de la
      copie, de la révocation), mais **aucun argument** d'aucun de ces appels ne contient le secret (tranché en
      validation P1) ; `document.title` non plus ; **témoins** : `console.info('kesh_mcp_TEMOIN')`,
      `toast('kesh_mcp_TEMOIN')` et `toast.info('kesh_mcp_TEMOIN')`, posés par le test lui-même, sont bien vus par
      leurs espions, **puis chaque espion est remis à zéro** (`mockClear()`) avant le parcours (validation P2, R4 = F8) ;
    - (v) **le stockage et l'adresse** (validation P1, R1 — `indexedDB` n'existe pas sous jsdom 29.0.2,
      `vite.config.ts:58`) — `vi.spyOn(Storage.prototype, 'setItem')` (donc `localStorage` et `sessionStorage`) :
      jamais appelé avec le secret ; `vi.stubGlobal('indexedDB', { open: vi.fn() })`, rétabli par
      `vi.unstubAllGlobals()` : `open` jamais appelé ; `vi.spyOn(history, 'pushState')` et `'replaceState'` : jamais
      appelés avec une URL qui le contient. **Chaque espion a son témoin**, un appel posé par le test lui-même et
      **vu** par l'espion avant le parcours : `sessionStorage.setItem('k', 'kesh_mcp_TEMOIN')`,
      `indexedDB.open('t')`, `history.pushState({}, '', '/kesh_mcp_TEMOIN')` (puis l'adresse rétablie) — sans eux, un
      espion posé sur le mauvais objet ferait passer M5 à vide. **Chaque espion est remis à zéro (`mockClear()`) après
      son témoin vu** (validation P2, R4 = F8) : sans quoi `open`, appelé par le témoin, ferait rougir l'assertion
      « jamais appelé », et les assertions « jamais appelé avec le secret » porteraient aussi sur l'appel du test.
    Côté serveur, déjà établi par la 32-5c et cité au Dev Agent Record : la liste sans secret ni condensat
    (`mcp_admin_routes_e2e.rs:371`), l'audit sans l'un ni l'autre (`:803`), aucun journal de corps (fait 14).
12. **AC-12 — révoquer.** Offerte pour **toute clé non révoquée**, expirée comprise (C-32-7-9). Confirmation inline :
    « L'agent qui emploie cette clé perd l'accès immédiatement. La révocation est définitive ; seule la restauration
    d'une sauvegarde antérieure rendrait la clé de nouveau active. » ; bouton `<Button variant="destructive">`, **sans
    rouge posé à la main** (fait 12 : la table de `boutons-rouges-inventory.test.ts` reste inchangée) — y compris le
    **déclencheur** *Révoquer* de chaque ligne, qui ouvre la confirmation : un `Button` partagé (`variant="destructive"`
    ou une variante neutre), **jamais** un `<button>` brut à `text-red-600` comme le précédent `api-keys` (validation P1,
    F10) ; 204 → message
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
      attente par clé, et **les issues d'outil** qui partent sans validation sous une clé `requise` — « introuvable »,
      « trop volumineux », erreur —, fiche 32-8 l. 916-930, AC-C10 l. 924-925 ; validation P2, R8) ; *Dispensée* — « les
      données partent sans validation ; chaque appel reste journalisé » ;
    - (d) un lien vers *Envois du serveur MCP* (`/admin/mcp/sends`, posé par la 32-8c, prérequis fusionné ; la route
      relue à l'étape 0).
    Tests : (a) confirmation présente vers `exempt` et absente vers `required` ; (b) chaque issue ; pas de contrôle sur
    une clé révoquée ; (d) le lien et sa cible.
14. **AC-14 — i18n.** Toutes les clés `mcp-server-*` et `nav-admin-mcp` dans les **quatre** catalogues ;
    `npm run lint-i18n-ownership` vert sans entrée neuve à `KNOWN_VIOLATIONS` ; `i18n-keys.test.ts` vert, amendé (AC-14 (d)) ; aucune
    collision avec `mcp-sends-*` (32-8) ni avec les messages serveur `admin-mcp-*` (fait 11). Les noms propres
    (`Traefik`, `Docker`, `100.64.0.0/10`, `X-Forwarded-For`, `.mcp.json`) et les codes de portée **ne se traduisent
    pas**.
    - (a) **Le repli égal à la valeur fr-CH, par une garde réelle** (validation P1, F2 ; C-32-7-21). Aucune garde du
      dépôt ne le tient aujourd'hui pour un domaine entier : `i18n-un-repli-par-cle.test.ts` compare les sites d'une clé
      **entre eux** (`parTexte.size > 1`, l. 216), sur ses seuls `PREFIXES` (l. 51-72), jamais au catalogue ; G13
      de `i18n-repli-divergent-actif.test.ts` compare bien le repli au catalogue (`valeurDuCatalogueFr`,
      `les_replis_frontend_a_site_unique_suivent_le_catalogue`), mais sur la **liste fermée** `REPLIS_A_SITE_UNIQUE`
      (treize clés, l. 216-235 ; boucle l. 248-252). La story **étend G13 par préfixe**, dans ce même fichier : une liste
      `PREFIXES_REPLI_EGAL_AU_CATALOGUE = ['mcp-server-']` et un test
      `les_replis_mcp_server_suivent_le_catalogue` qui, sur le relevé `replisParCle()` du fichier, exige pour **chaque**
      clé du préfixe des replis égaux à `[valeurDuCatalogueFr(cle)]` ; la fonction de comparaison est **pure**
      (`ecartsAuCatalogue(releve, cles, valeurDe)` → liste des clés en écart, nommées). *Fermeture de la classe* :
      l'ensemble des clés `mcp-server-*` du relevé est **égal**, dans les deux sens, à celui des clés `mcp-server-*` du
      FTL fr-CH — une clé du catalogue sans site littéral (bâtie par gabarit, ou morte) rougit, une clé du relevé
      absente du catalogue aussi. ⚠️ **L'appartenance au préfixe se juge par `cle.startsWith(prefixe)`**, jamais par
      une recherche de sous-chaîne (validation P2, fait 21) : le FTL porte `audit-log-entity-mcp-server-settings` et
      `audit-log-action-mcp-server-settings-updated`, qui **contiennent** `mcp-server-` sans relever de l'écran — une
      recherche de sous-chaîne les ferait entrer dans l'ensemble du FTL et rougir l'égalité ; *témoin vert* : un FTL
      factice portant `audit-log-entity-mcp-server-settings` n'ajoute rien à l'ensemble. **`nav-admin-mcp` n'entre
      pas dans cette extension** (validation P2, F1 = R1 ; C-32-7-23) : c'est une clé de **donnée** du menu, résolue
      par un site non littéral (fait 21), que `replisParCle()` ne relève jamais — l'y nommer rendait le test rouge par
      construction (relevé vide contre `['Serveur MCP']`) ou, la clé retirée en cours de route, la perdait en silence.
      Elle est tenue par l'AC-14 (d). *Témoin rouge* : `ecartsAuCatalogue` sur un relevé factice dont un repli diffère de sa
      valeur → la clé est rendue ; *témoin vert* : relevé factice égal → `[]` ; *anti-test-muet* : le nombre de clés
      `mcp-server-*` relevées est **écrit en dur** (`toBe(N)`, N recompté depuis le FTL au développement). Mutation M22.
    - (b) **Les autres gardes frontend que la story touche**, à garder vertes et à citer au Dev Agent Record
      (validation P1, F10) : `erreurs-en-texte.test.ts` (les liaisons de `catch` — AC-5 (f), AC-9 (b), AC-12,
      AC-13 (b)) ; `i18n-repli-divergent-actif.test.ts`, garde générique (une clé partagée par deux sites — les phrases
      de politique de l'AC-13 (c) servent le formulaire **et** la note du tableau : un seul repli, ou deux clés) ;
      `i18n-entrees-a-variables.test.ts` (toute clé à `{ $x }`) ; `e2e-selecteurs-traduits.test.ts` (fait 13).
    - (c) **Le commentaire de G22 est faux, et la story le corrige** (`textes_coherents.rs:1688`, doc de
      `la_sauvegarde_automatique_dit_vrai`) : il dit le repli Svelte de `admin-backup-auto-reminder` « tenu égal au fr-CH
      par le Vitest `i18n-un-repli-par-cle` », qui ne compare pas au catalogue et dont les préfixes n'incluent pas
      `admin-backup-`. Nouvelle phrase : « **aucune** garde ne tient le repli Svelte égal au fr-CH (angle mort écrit ;
      `i18n-un-repli-par-cle` compare les sites entre eux, sur ses seuls préfixes ; #758) ». La story ne l'étend pas à
      `admin-backup-` (hors périmètre ; la classe est tracée par #758, ouverte par l'orchestrateur — demande 7 close en
      validation P2, F3).
    - (d) **`nav-admin-mcp` et le compte des sites** (validation P2, F1 = R1 ; C-32-7-23) — dans
      `frontend/src/lib/shared/i18n-keys.test.ts` :
      - `'admin-mcp'` ajouté à `FAMILLES_RESOLUES['nav-']` (l. 858-868), à côté de `'admin-mcp-sends'` si la 32-8c
        l'y a mis (constaté à l'étape 0 ; sinon l'écart se déclare), avec un commentaire daté de la story ;
      - `ATTENDU.sitesTotal` (l. 724) **recompté depuis la source** aux deux bornes (base de l'étape 0 → HEAD de la
        branche ; `grep -oE '(i18nMsg|[^a-zA-Z]msg)\('` par fichier touché, comme la 32-1b, l. 715-723), et le
        commentaire qui le ventile écrit fichier par fichier ; `sitesNonResolus`, `relais` et `sitesGabarit`
        **inchangés** — l'entrée de menu est une donnée, et toutes les clés `mcp-server-` sont littérales (fait 21) ;
        un compte qui bougerait est un site non littéral apparu, à résoudre, jamais à recompter (l. 1137-1145) ;
      - **ce que cette garde ne tient pas, écrit** : elle exige que `nav-admin-mcp` existe au catalogue fr-CH, non que
        son repli `'Serveur MCP'` égale la valeur fr-CH, et elle ne voit pas l'oubli de `'admin-mcp'` à la famille
        (`'nav-'` n'est pas à couverture close, fait 21). Angle mort de la classe tracée par **#758** (*Ce que les
        gardes ne voient pas*) ; la relecture en revue le couvre.
      - Mutation M26 : `nav-admin-mcp` retiré des **quatre** catalogues, `'admin-mcp'` présent à la famille → le test
        « toute clé demandée existe au catalogue » (l. 1037-1045) rougit en la nommant.
15. **AC-15 — deux gardes lexicales** (règle commune ; fichier `frontend/src/lib/features/mcp-server/
    mcp-server-gardes.test.ts`, `// @vitest-environment node`) :
    - **G-a — ni stockage, ni journal, ni HTML brut.** *Propriété* : aucune source **hors tests** de
      `features/mcp-server/` (récursivement) ni des fichiers **directement** dans `routes/(app)/admin/mcp/` — **sans
      descente** dans ses sous-dossiers, ce qui écarte par construction ceux d'autres stories (`sends/` de la 32-8c,
      `prompts/` d'une 32-10b fusionnée avant ou après la 32-7 ; validation P2, F12) — n'emploie un stockage du
      navigateur, un journal ou du HTML brut. *Formes* : `localStorage`, `sessionStorage`, `indexedDB`, `caches.`, `document.cookie`,
      `console.`, `{@html`, après `masquerCommentaires` (fait 15). *Mesure à la rédaction*, sur les dossiers
      comparables (le périmètre n'existe pas encore) : `grep -rnE 'localStorage|sessionStorage|indexedDB|caches\.|
      document\.cookie|console\.|\{@html' frontend/src/lib/features/api-keys "frontend/src/routes/(app)/settings/
      api-keys" frontend/src/lib/features/admin-backup | grep -v '\.test\.ts'` → **1** ligne,
      `routes/(app)/settings/api-keys/+page.svelte:41`, un **commentaire** (« pas de localStorage ») — que le masquage
      doit écarter : c'est le **témoin vert**. *Témoin rouge* : `sessionStorage.setItem('k', s)` sur source factice.
    - **G-b — l'instance du secret ne vit que dans un composant.** *Propriété* : `new SecretUnique` n'apparaît que dans
      le bloc `<script>` **d'instance** d'un `.svelte` du périmètre de G-a — jamais dans un `.ts`/`.svelte.ts` hors
      tests, **ni dans un `<script module>` ou un `<script context="module">` d'un `.svelte`** (validation P1, F3 = R2) :
      une instance de module, qu'elle vive dans un `.ts` ou dans le bloc de module d'un composant, est évaluée une fois
      par module et survit à la fermeture et à la navigation. La garde découpe donc chaque `.svelte` en ses blocs
      `<script …>` et classe chacun par ses attributs (`module`, `context="module"`) ; une occurrence hors de tout
      bloc `<script>` (gabarit) compte aussi comme refusée. **La recherche se fait après `masquerCommentaires`**
      (fait 15), comme G-a (validation P2, R12) : un doc-comment de `secret-unique.svelte.ts` qui écrirait l'exemple
      `const s = new SecretUnique()` ne rougit pas. *Mesure à la rédaction* : la classe n'existe pas
      (`grep -rn 'SecretUnique' frontend/src` → rien) ; la mesure se refait à l'étape 0 **et** après T3. *Témoins
      rouges* : `export const s = new SecretUnique();` dans un `.ts` factice ; `<script module> export const s = new
      SecretUnique(); </script>` dans un `.svelte` factice ; la même ligne sous `<script context="module">` ; *témoin
      verts* : `const secret = new SecretUnique();` dans le `<script lang="ts">` d'instance d'un `.svelte` factice ;
      `/** Exemple : const s = new SecretUnique(); */` dans un `.ts` factice (commentaire masqué).
      Mutations M16 (`.ts`) et M23 (`<script module>` de `McpKeysSection.svelte`).
    Chaque garde lit les **vrais** fichiers du périmètre, et porte une assertion de **cardinalité** du périmètre lu,
    **écrite en dur** (`toBe(N)`, N = nombre de fichiers hors tests que la story crée dans le périmètre, recompté au
    développement et écrit avec sa liste ; validation P1, R13) — un « au moins » laisserait passer un chemin faux qui
    lirait un sous-ensemble.
16. **AC-16 — l'E2E** (`frontend/tests/e2e/mcp-server.spec.ts`, sélecteurs par `data-testid` seulement, fait 13 ; motif
    `api-keys.spec.ts`). **Fait** (validation P1, R8 = F5) : `seedTestState` appelle `truncate_all`
    (`routes/test_endpoints.rs:172`), qui vide toute `TABLES_TO_TRUNCATE` (`kesh-db/src/test_fixtures.rs:392-402`), où
    figurent `mcp_keys` (`backup.rs:78`) et `mcp_server_settings` (`:83`) : chaque spec repart d'un serveur désactivé et
    sans clé — **aucun `afterAll`** de remise en état. Aucun préréglage ne fournit de Comptable
    (`tests/e2e/helpers/test-state.ts:33-41`, type `Preset`) : (f) le crée sur le motif de `backup-schedule.spec.ts:200-218`
    (`POST /api/v1/users` rôle `Comptable`, `clearAuthStorage`, connexion sous ce compte). Le groupe *Administration*
    est **replié** par défaut (`+layout.svelte:225`) : (a) l'ouvre par `nav-group-administration` (l. 366) avant de
    suivre `nav-link-admin-mcp`.
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
    date de modification ; (3) filtre désactivé, avertissement (a) ; (4) une plage `0.0.0.0/0` et une `::/0`, avertissement (b) dans ses deux familles ; (5)
    un mandataire `/0`, avertissement (c) ; (6) filtre activé sans plage, avertissement (d) ; (7) une entrée refusée
    par le serveur (bits d'hôte) ; (8) une forme canonique réaffichée après enregistrement (`192.168.1.10` →
    `/32`) ; (9) le conflit de version ; (10) le formulaire de création, les cinq phrases de portée, les deux de
    politique ; (11) les refus du formulaire (aucune portée ; nom vide — validation P2, F11) ; (12) l'encart du secret ; (13) la liste après
    *Fermer* ; (14) la confirmation de révocation ; (15) une clé révoquée ; (16) une clé expirée ; (17) une clé à
    portée inconnue (posée par SQL sur la base de démonstration du worktree) ; (18) la confirmation du passage en
    dispensée ; (19) une clé dispensée dans la liste ; (20) l'entrée de menu ; (21) l'échec du chargement de la liste
    des clés ; et, ajoutées en validation P1 (F7) : (22) le chargement puis l'échec du chargement des **réglages**,
    *Enregistrer* désactivé (AC-5 (g)) ; (23) le message de succès des réglages (AC-5 (c)) ; (24) l'erreur générique
    hors `ApiError` (AC-5 (f)) ; (25) un 400 du serveur à la création d'une clé (AC-9 (b)) ; (26) le message d'un 404
    ou 409 de révocation (AC-12) ; (27) le message d'un 409 de changement de politique (AC-13 (b)) ; (28) une clé dont
    la **dernière utilisation** est renseignée, avec sa note (AC-8 (b) ; la clé que l'E2E (c) présente à `/mcp`, ou
    une présentation équivalente sur la démonstration) ; et, ajoutée en validation P2 (F11) : (29) le toast de succès
    de la copie du secret. La capture (22) montre la phrase fixe de l'échec de lecture (AC-5 (g)), la capture (1) celle
    des *Origines admises* (AC-5 (h)), la capture (4) l'avertissement (b) dans ses deux familles — une plage
    `0.0.0.0/0` **et** une `::/0` (validation P3, R1). *Recette de la capture (22)* (validation P3, F6) : sur la base de
    démonstration du worktree, `UPDATE mcp_server_settings SET allowed_networks = '[1]' WHERE company_id = <la société
    de démonstration>` (un tableau JSON valide que `decode_list` refuse), la page rechargée ; puis la ligne rétablie
    (valeur relevée avant l'`UPDATE`, ou la ligne supprimée — défauts) et le rétablissement constaté à l'écran.
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
       les efface (renvoi l. 1414) ; une **ligne de réglages illisible** fait paraître le serveur de la société
       **désactivé** (fiche 32-6, AC-a2, liste de la 32-7 l. 1247-1248 à la date de lecture, l. 1268-1269 à `dd4b8398` : relu au code de la 32-6a à l'étape 0 ; validation P1, R6 (a)) ;
       une ligne à `version = 0` posée hors de l'écran (SQL, sauvegarde fabriquée) ne s'enregistre plus par l'écran —
       chaque enregistrement rend un conflit (C-32-5-53, `routes/mcp_server_settings.rs:35-38` ; validation P1, R12) ;
       une ligne **corrompue** — c'est la ligne illisible qui fait paraître le serveur désactivé — empêche aussi l'écran
       de lire les réglages : il affiche une erreur de lecture et n'enregistre rien (AC-5 (g) ; #738 B8 ; validation P2,
       R3). Phrase du manuel, pour les deux cas (récrite en validation P3, R2 et F2) : « Ces deux états ne se
       réparent pas depuis l'écran, et la réinitialisation, qui efface les réglages, n'est jamais permise sur une
       installation de production (\S\ref{sec:env-vars}, variable \texttt{KESH\_PRODUCTION\_RESET}). Supprimez la ligne
       de la société dans la table \texttt{mcp\_server\_settings} : le serveur revient à ses réglages par défaut,
       désactivé, et l'écran les relit. En dernier recours seulement, restaurez une sauvegarde antérieure
       (\S\ref{sec:backup-ui-keshbackup}) : la restauration remplace toute l'installation, non la seule ligne, et refuse
       une sauvegarde d'une version qui ne porte pas les mêmes tables, dont celles de la 0.15.0 et d'avant. » — voies
       relues au code à l'étape 0 (C-32-7-24 ; AC-5, ⚠️ *Ce que l'écran ne répare pas*). Le renvoi va à la ligne qui
       porte l'information : la rangée `KESH\_PRODUCTION\_RESET` du tableau des variables (`admin-manual.tex:767`,
       « une installation de production ne l'est jamais »), sous `\label{sec:env-vars}` (l. 705) — la sous-section
       *Configuration de la company via l'onboarding* (l. 1404), citée jusqu'en P3, ne le dit pas (R2) ;
       `sec:backup-ui-keshbackup` existe (l. 2177 ; remplacement de toute l'installation l. 2262, compatibilité
       l. 2249). La commande SQL exacte, si le manuel la donne, suit les conventions de ses autres procédures SQL
       (relues à l'étape 0), sinon il n'en donne pas.
    2. `\subsection{Les clés du serveur MCP}\label{sec:mcp-cles}` — création par l'**Administrateur** seul ; le secret
       `kesh_mcp_…` **affiché une seule fois**, seul son condensat conservé ; les **portées** en tableau entre les
       lignes `% mcp-portees:debut` et `% mcp-portees:fin` (une ligne par portée : code, ce qu'elle ouvre — dont ce que
       `contacts:read` **démasque** —, en reprenant les phrases de l'AC-10) ; **aucune** portée d'écriture des livres
       ni d'administration ; expiration ; révocation immédiate et définitive ; « dernière utilisation » n'est pas
       « dernier appel servi » ; une clé MCP **n'ouvre pas** `/api/v1` (401), et `/mcp` **ne lit que**
       `Authorization: Bearer <clé MCP>` — ni un PAT ni le navigateur connecté ; au journal d'audit, un appel paraît
       **au nom du créateur de la clé**, sans la marque « clé API » (point 4 de la fiche 32-6, accepté) ; restauration
       (une clé révoquée depuis redevient active), remise à zéro (clés conservées), réparation (#528 : révoquées).
       Et (validation P3, F1 ; C-32-7-25 ; AC-8 (b)), phrase du manuel : « Une clé n'authentifie que tant que son
       créateur est actif et Administrateur, relus à chaque requête : désactiver ce compte, ou lui retirer le rôle
       Administrateur, coupe ses clés sans les révoquer --- l'agent reçoit le refus d'une clé inconnue, et l'écran
       \emph{Serveur MCP} les montre toujours actives. Rendre au compte son état ou son rôle rouvre celles qui ne sont ni révoquées ni expirées ; pour les
       fermer, révoquez-les. » (`services/cles_mcp.rs:20-21`, `:47`, `:94-99`, relus au code à l'étape 0 ; le
       précédent des clés d'API est `admin-manual.tex:2910`.)
    3. `\subsection{La politique de validation}\label{sec:mcp-validation}` — `requise` par défaut, `dispensée` désignée
       par l'administrateur ; ce que chacune fait, **sans** la phrase « une clé `required` ne peut appeler aucun outil »
       (devenue fausse avec la 32-8, fiche 32-8 l. 1397, 1408) ; renvoi à `sec:mcp-envois`.
    4. `\subsection{Le filtre d'adresse}\label{sec:mcp-filtre}` — porte sur le **pair TCP** ; `X-Forwarded-For` lu des
       seuls **mandataires déclarés**, de droite à gauche ; `Forwarded` et `X-Real-IP` **toujours ignorés** — phrase du
       manuel : « derrière un mandataire qui ne pose que \texttt{X-Real-IP}, le client vu par Kesh est le mandataire
       lui-même » (fiche 32-6 à `dd4b8398`, l. 1270-1271 ; validation P2, F2 = R9) ; refusé,
       le client voit une route absente ; filtre activé et liste vide = personne ; une entrée invalide des **plages**
       est ignorée, une entrée invalide des **mandataires** coupe le serveur MCP de la société (route absente, erreur au
       journal du serveur), **filtre activé ou non** (P7-H3 de la 32-6) — l'écran n'en laisse enregistrer aucune, une
       sauvegarde ou une écriture SQL le peut ; phrase du manuel (validation P3, F7) : « l'écran affiche alors le
       serveur activé, sans autre signe ; retirer l'entrée fautive des mandataires de confiance et enregistrer suffit
       à le rétablir --- tant qu'elle y est, l'enregistrement est refusé et le message la nomme » (AC-5, ⚠️, troisième
       état ; `validate_settings`, `routes/mcp_server_settings.rs:198-208`) ; sur une installation à **plusieurs** sociétés, la clé d'une société
       ainsi coupée reçoit le 401 d'une clé inconnue — et non une route absente — quand elle vient d'une adresse qu'une
       **autre** société servie admet (P7-H3, second volet, fiche 32-6 à `dd4b8398`, l. 1427-1428 ; une seule société
       aujourd'hui ; validation P2, R9) ; les **défauts**, listés exactement entre `% mcp-reseaux-defaut:debut` et
       `% mcp-reseaux-defaut:fin` (fait 6) ; **`100.64.0.0/10` absente**, à ajouter pour un VPN de type Tailscale ;
       phrase du manuel (#738 ; validation P3, R1 — « un `0.0.0.0/0` ou `::/0` admet tout client » était faux) :
       « \texttt{0.0.0.0/0} admet tout client IPv4, \texttt{::/0} tout client IPv6 : pour admettre les deux familles,
       il faut les deux entrées. Il en va de même d'un mandataire de confiance de préfixe \texttt{/0}, qui fait croire
       l'en-tête \texttt{X-Forwarded-For} de tout pair de sa famille. Une adresse IPv4 écrite sous sa forme mappée en
       IPv6 (\texttt{::ffff:192.168.1.10}) est ramenée à sa forme IPv4 et compte comme IPv4. » (fait sourcé à l'AC-6
       (b) : `transport.rs:81`, `reseau.rs:9-12`, `:57`, `:64`, `:73`, `:128`, `ipnet` 2.12.0) ; puis un
       `keshwarning` **Ce que le filtre ne garantit pas** : Traefik à déclarer parmi les mandataires, faute de quoi tous
       les clients paraissent venir de lui ; la publication de port Docker par le mandataire en espace utilisateur, où
       tous peuvent paraître venir de la passerelle (`172.17.0.1`), adresse privée admise par défaut — **le filtre ne
       filtre alors rien** ; il ne remplace ni le pare-feu (renvoi `sec:firewall`) ni le VPN ; Kesh n'est **jamais**
       exposé sur Internet ; **pas de limite de débit**, et **toute requête vers `/mcp`, même anonyme, lit la base**
       (les réglages, pour savoir si le serveur est servi et à qui ; fiche 32-6 à `dd4b8398`, l. 1269-1270 ; validation
       P2, F2 = R9) ; et « un en-tête `X-Forwarded-For` venu d'un pair non déclaré
       est ignoré » (le pair est jugé sur sa propre adresse ; la garde « 2 bis » de la fiche 32-6 n'est pas retenue —
       décision de l'orchestrateur, validation P1, R7 ; la phrase relue au code de la 32-6a à l'étape 0).
    5. `\subsection{L'en-tête Origin}\label{sec:mcp-origin}` — ne protège que **d'un navigateur** (DNS rebinding) ; la
       vraie barrière est la clé ; `Host` n'est pas contrôlé ; origines propres (`KESH_PUBLIC_BASE_URL`,
       `localhost`/`127.0.0.1`/`[::1]` au port de Kesh, port par défaut normalisé) ; un `KESH_PUBLIC_BASE_URL` invalide
       retire **en silence** l'origine qui en viendrait (avertissement au seul journal du serveur), un
       `KESH_PUBLIC_BASE_URL` à sous-chemin donne son origine ; **pas de CORS** (`OPTIONS` → 405) : une origine admise
       n'ouvre `/mcp` à aucune page web étrangère. **Ce que voit un client dont l'`Origin` est refusée** (validation P2,
       R2 = F2 ; fiche 32-6 à `dd4b8398`, ordre des contrôles l. 1070-1085, C-32-6-52 et C-32-6-53 du registre
       `-326`), phrases du manuel :
       - « Une origine qu'aucune société servie n'admet reçoit **403**, avant tout examen de la clé ; un en-tête
         \texttt{Origin} envoyé **deux fois** reçoit aussi 403. Ajoutez l'origine du client aux \emph{Origines
         admises} de la société (écran \emph{Serveur MCP}), ou vérifiez \texttt{KESH\_PUBLIC\_BASE\_URL} si le client
         devrait employer l'origine de Kesh ; un en-tête répété se corrige dans le client. » (étape 4 ;
         `transport::unique`, C-32-6-53 ; la voie utile ajoutée en validation P3, F8 : l'étape 4 compare aux origines
         propres et aux `allowed_origins` des sociétés servies, `kesh-32-6/…/transport.rs:105-111`,
         `reseau.rs:182-205`) ;
       - « Une origine que la société **de la clé** n'admet pas — admise par une autre société servie — reçoit le
         **401** d'une clé inconnue : le client ne peut pas distinguer ce refus d'une clé fausse. Avant de soupçonner la
         clé, vérifiez l'origine que le client envoie, les \emph{Origines admises} de la société de la clé (écran
         \emph{Serveur MCP}) et \texttt{KESH\_PUBLIC\_BASE\_URL}. » (étape 9, C-32-6-52, `transport.rs:151-162` ; une
         seule société aujourd'hui, si bien que le cas courant est le 403 ; *Origines admises* ajoutées en validation
         P3, F8) ;
       - un client qui n'envoie pas d'`Origin` n'est pas concerné (absente, admise ; fiche 32-6 l. 44 et 388).
       Statuts et étapes **relus au code de la 32-6a** à l'étape 0 (`kesh-api/src/mcp/transport.rs`, `reponse.rs:136`,
       `textes.rs:35` à `dd4b8398`).
    6. `\subsection{Ce qui est journalisé}\label{sec:mcp-journal}` — au journal d'audit : `mcp_key.created`,
       `mcp_key.revoked`, `mcp_key.send_validation_changed`, `mcp_server_settings.updated`, et **chaque** `tools/call`
       (`mcp.tool_called` : clé, outil — ou « (inconnu) » —, issue, durée, politique ; **jamais** les arguments ni les
       données) ; si le journal ne s'écrit pas, **rien ne part** ; ce qui n'y est pas (`initialize`, `ping`,
       `tools/list`, les notifications, les refus du transport — au journal du serveur seulement), et ce que les
       32-8, 32-9 et 32-10a, prérequises, y ont ajouté (décisions de validation, 32-8 ; consultation d'un rapport,
       32-9 ; `prompts/get`, 32-10a) — chaque entrée relue au code à l'étape 0, ligne citée ; une entrée que le code ne
       pose pas se retire de la phrase, et l'écart se déclare au Dev Agent Record (validation P2, F5) ; un appel MCP **écrit** (dernière utilisation, journal) —
       à ne pas présenter comme « une lecture n'écrit rien » (#730) ; un panic n'est pas journalisé.
    7. `\subsection{Ce que voit un client mal configuré}\label{sec:mcp-erreurs}` — `Content-Type: application/json`
       exigé (415) ; `MCP-Protocol-Version` autre que `2025-11-25`/`2025-06-18` refusée (400) ; corps limité à 1 Mio ;
       délai de 30 s par outil ; en-têtes de contrôle répétés refusés (`Origin` → 403, `Authorization` → 401,
       `Content-Type` → 415, `MCP-Protocol-Version` → 400, C-32-6-53) ; une `Origin` refusée → 403 ou 401 selon
       l'étape, renvoi à `sec:mcp-origin` (validation P2, R2 = F2) ; toute notification reçoit 202 ; un message de
       type réponse reçoit 400 (32-6 B4) ; les erreurs JSON-RPC du transport n'ont pas d'`id` ; chaque appel d'outil
       ouvre une connexion MariaDB neuve — chaque valeur **relue au code** de la 32-6a à l'étape 0.
    8. `\subsection{Configurer un client}\label{sec:mcp-client}` (C-32-7-13, **propriété de la 32-7** — C-32-7-22 ;
       la 32-11 n'ouvre pas de seconde section de configuration : son parcours *Brancher un agent* y renvoie et, une
       fois les gestes éprouvés, **amende celle-ci en place**). Chaque geste porte sa source, et rien n'est écrit qui ne
       soit sourcé (validation P1, R11 ; décision acceptée de Guy) :
       - **Claude Code**, d'après sa documentation : `claude mcp add --transport http --scope local kesh
         <adresse>/mcp --header "Authorization: Bearer <clé>"` (la forme `--transport http … --header` : spike § 2.1,
         l. 211-212 ; l'option `--scope local` : spike § 2.4, l. 320 — validation P2, R7) ; la portée `local` (défaut)
         écrit la clé **en clair** dans `~/.claude.json` (spike l. 212, l. 302) — protéger ce fichier ; la portée `user`
         est nommée, **sans** dire où elle stocke la clé (la spike ne le dit pas) ; **jamais** `--scope project`, qui
         écrit un `.mcp.json` à la racine du projet, **versionné** (spike l. 212-213), sauf **par expansion de variable**
         `${KESH_MCP_KEY}`, la variable restant hors du dépôt (spike l. 302-304) — le manuel **n'écrit pas** la forme
         JSON de l'en-tête dans `.mcp.json` : la spike ne la source pas (validation P2, R7 = F10) ; la limite de **25 000 jetons** d'une réponse
         (`MAX_MCP_OUTPUT_TOKENS`, spike l. 252). Le conseil sur l'historique du shell (précéder la commande d'une espace
         sous `HISTCONTROL=ignorespace`, ou lire la clé par `read -rs`) est un geste **général de Bash**, présenté
         comme tel, non comme un comportement de Claude Code.
       - **Hermes Agent (Nous Research)**, le client que documente la spike (§ 2.2, l. 255-263) : le bloc
         `mcp_servers.<nom>` de `~/.hermes/config.yaml`, avec `url` et `headers` (l. 260) ; sa référence dit qu'un
         `transport: sse` choisit SSE « instead of Streamable HTTP » et **ne nomme pas de transport par défaut**
         (l. 261-263) — le manuel le dit tel quel, ajoute que Kesh parle **Streamable HTTP** à réponses JSON, sans flux SSE (fiche 32-6 à `dd4b8398`,
         l. 33 et 44 ; relu au code à l'étape 0), et
         n'affirme **pas** qu'un bloc sans `transport` suffise : c'est une lecture, non sourcée, à éprouver par la 32-11
         (validation P2, F10 — « et rien d'autre » retiré) ; la Q9 de la
         spike (« s'agit-il bien de Hermes Agent ? », l. 1027) n'ayant pas de réponse, le manuel nomme le client par son
         nom complet et n'affirme rien de plus que sa documentation.
       - Une phrase dit que **la configuration de ces deux clients n'est pas éprouvée** contre un client réel (spike,
         l. 206-207 : « non éprouvé tant que Guy ou la 32-11 n'a pas rejoué le § 2.4 »), et renvoie au parcours pas à
         pas de la 32-11 (*Brancher un agent*).
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
    32-10a l. 1008 et 1384, puis **32-11** au dernier état, fiche 32-6 A3 l. 1279-1281, l. 1300-1302 à `dd4b8398`) ; `docs/api-external.md:185` et
    `:831` (**32-11**, fiches 32-6 l. 1282-1283, 32-9 l. 1328) ; `admin-manual.tex:2925`, la brochure
    (`marketing-brochure.tex:480`), `README.md:233` et `website/roadmap.html:375` (**32-11**). À l'étape 0, la 32-7 relit
    chacun et **écrit au Dev Agent Record** s'il est faux à cette date ; elle ne le corrige pas.
21. **AC-21 — le PDF.** `admin-manual.pdf` et `user-manual.pdf` régénérés (`make fr` dans `docs/manual/`) et commités ;
    contrôle **aplati** cité au Dev Agent Record : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s
    ' ' | command grep -oiE '.{0,30}(Le serveur MCP|100\.64\.0\.0/10|ne filtre alors rien|\.mcp\.json|25 000
    jetons).{0,30}'` — chaque motif trouvé au moins une fois ; puis, pour les phrases neuves des validations P2 et P3
    (validation P3, F5), le même contrôle aplati avec le motif `(mcp_server_settings|X-Real-IP|aucune société
    servie|tout client IPv6|Origines admises|son créateur est actif)` — chacun trouvé au moins une fois ;
    `mcp\_server\_settings` doit paraître **`mcp_server_settings`** au PDF (soulignés rendus, non `mcp\_…`) ; et les renvois de l'AC-19 **et** de l'AC-18 (1) (`sec:env-vars`, `sec:backup-ui-keshbackup`)
    rendus (aucun `??`).
    ⚠️ **`command grep`, et des fenêtres de 30** (validation P1, F11) : sur ce poste, `grep` est une fonction qui lance
    `ugrep`, qui refuse `.{0,80}` (« exceeds complexity limits », constaté). Le nombre s'écrit `25\,000` au `.tex`
    (précédent `10\,000`, `admin-manual.tex:1297`), que `pdftotext` rend `25 000` à espace ASCII — non `25~000`.
22. **AC-22 — CHANGELOG.** Sous `## [0.16.0] — Non publié`, **compléter** la `### Ajouté` que la 32-8c a créée (sa
    fiche, AC-C10, l. 929-930 à `81cbd3f5` : « entrée `Ajouté` de la version en cours » ; prérequis fusionné) — son **absence** à
    l'étape 0 est un écart d'un prérequis, à déclarer à l'orchestrateur, non une branche de cette fiche (validation P2,
    R14 ; ordre du dépôt, fait 19) : « **Administration → Serveur MCP** : activer le serveur, régler son filtre
    d'adresse, créer, révoquer et régler les clés du serveur MCP ; une réinitialisation des données de démonstration
    conserve ces clés et efface les réglages du serveur, qui repart désactivé ; manuel d'administration, § *Le serveur
    MCP* » — sans date (la release la pose). La demi-phrase de la réinitialisation est le report de C-32-5-49 (registre
    `-325`, l. 652-654 : « la 32-5c (création des clés) ou la 32-7 (écran) écrira l'entrée qui annonce les clés du
    serveur MCP, et la conservation à la remise à zéro peut y figurer » ; validation P2, R10), relue au code à
    l'étape 0 (`RESET_PRESERVED_TABLES`, `kesh-db/src/backup.rs:121-129`).
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
    - (b) le bloc `% mcp-portees:debut`/`fin` nomme **exactement** les cinq `McpScope::ALL` (`as_str`), dans cet ordre —
      en lisant **le premier `\texttt{…}` de chaque rangée** du tableau — le texte du bloc découpé sur `\\`, non sur
      les sauts de ligne du source, une rangée pouvant s'étendre sur plusieurs lignes (validation P2, F7) —, non tous
      les codes qu'elle cite (la phrase d'`invoices:read` cite `contacts:read`) ; *témoins* : une rangée factice qui
      cite une seconde portée n'ajoute rien à la liste extraite (validation P1, R13) ; une rangée factice écrite sur
      deux lignes de source, dont la seconde commence par `\texttt{contacts:read}`, n'en ajoute rien non plus ;
    - (c) dans les **quatre** catalogues, la clé de l'AC-6 (3) contient `100.64.0.0/10`, et celle de l'AC-6 (1)
      contient `Traefik` **et** `Docker` (noms des clés fixés au développement, écrits dans la garde) ;
    - **l'extraction des blocs se fait sur le source brut** (`lire(...)`), jamais après `normaliser`
      (`tests/common/manuel.rs:32-34` : les sauts de ligne y disparaissent, et les marqueurs `% …:debut`/`fin` avec
      eux) — précédent `sections_synology` (même fichier, l. 14-16), **non** G22, qui normalise (validation P1, R13) ;
      `DEFAULT_ALLOWED_NETWORKS` est `pub const` (`kesh-db/src/entities/mcp_server_settings.rs:26`) d'un module `pub`,
      et `kesh-db` est une dépendance de `kesh-api` : l'accès est direct (G22 n'en est pas le précédent) ;
    - chaque volet a son **témoin** : la fonction pure qui extrait un bloc rend une erreur sur un bloc absent (sans quoi
      un marqueur supprimé ferait passer (a) et (b) à vide).

## Tâches

- [ ] **T0 — étape 0** (§ *À recontrôler à l'étape 0*) : chaque point relu, ligne citée au Dev Agent Record ; les
  mesures des gardes G-a et G-b refaites ; la fusion des neuf prérequis (32-6a, 32-6b, 32-8a, 32-8b0, 32-8b, 32-8c,
  32-9a, 32-9b, 32-10a) **constatée** par `git log --oneline origin/main` — s'il en manque une, arrêt et rapport à
  l'orchestrateur ; l'état de la 32-10b constaté (non prérequis).
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
| `features/mcp-server/McpSettingsSection.test.ts` | (a) à (h), dont la phrase de l'échec de lecture et son témoin ; avertissements (a) à (d) et leurs témoins, (b) et (c) dans chaque famille (`0.0.0.0/0`, `::/0`) ; encadré, ses quatre phrases ; AC-7 | 5, 6, 7 |
| `features/mcp-server/secret-unique.test.ts` | `oublier` efface tout ; `montrer` remplace | 11 (ii) |
| `features/mcp-server/McpKeysSection.test.ts` | liste ; création, dont `formulaire_a_l_ouverture_requise_et_aucune_portee` et le double `GET` ; les cinq « ni » et les témoins de leurs espions ; révocation ; politique | 8 à 13 |
| `features/mcp-server/mcp-server-gardes.test.ts` | G-a (route sans descente), G-b (blocs `<script>` d'instance et de module, après `masquerCommentaires`), témoins, cardinalité exacte | 15 |
| `lib/shared/i18n-repli-divergent-actif.test.ts` (amendé) | `les_replis_mcp_server_suivent_le_catalogue`, `ecartsAuCatalogue` et ses témoins, égalité des ensembles relevé ↔ FTL (par `startsWith`, témoin `audit-log-entity-mcp-server-settings`), compte exact | 14 (a) |
| `lib/shared/i18n-keys.test.ts` (amendé) | `'admin-mcp'` à `FAMILLES_RESOLUES['nav-']`, `sitesTotal` recompté et commenté | 14 (d) |
| `routes/(app)/admin/mcp/admin-mcp-page.test.ts` | garde du `load` sous `browser` simulé, et son témoin | 1 |
| `tests/e2e/mcp-server.spec.ts` | (a) à (f) | 16 |
| `crates/kesh-api/tests/textes_coherents.rs` | G23 (a) à (c), témoin du bloc absent, témoins du premier code de rangée (seconde portée citée, rangée sur deux lignes de source) ; commentaire de G22 corrigé | 25, 14 (c) |

**Mutations** — chacune jouée sur une copie, le fichier restauré **et touché** ensuite (mémoire *Mutation restaurée,
binaire périmé*), journal `kesh-gate-logs/327-mut-<id>.log`, verdict au Dev Agent Record :

| Id | Mutation | Doit tuer |
|---|---|---|
| M1 | *Fermer* masque l'encart (`visible = false`) sans appeler `oublier()` | AC-11 (ii) — la sonde sur `oublier()` du test composant (le DOM seul, (i), ne la tue pas) |
| M2 | `oublier()` remet un drapeau sans effacer la valeur | `secret-unique.test.ts` |
| M3 | `createMcpKey` rend `cle = { ...res }` | `createMcpKey_separe_le_secret` |
| M4 | l'objet de création ajouté à la liste au lieu d'un `GET` | AC-9 (d) seul — deux `GET`, la liste du second affichée |
| M5 | `sessionStorage.setItem('mcp-secret', secret)` à la création | AC-11 (v) (espion à témoin) **et** G-a |
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
| M20 | le bouton *Dispensée* (`exempt`) coché par défaut au formulaire | AC-9 (e) |
| M21 | la case `books:read` précochée au formulaire | AC-9 (e) |
| M22 | le repli fr-CH de la clé de l'AC-6 (1) modifié dans le `.svelte` (« Traefik » retiré), le catalogue inchangé | AC-14 (a) |
| M23 | `export const secret = new SecretUnique()` dans un `<script module>` de `McpKeysSection.svelte` | G-b |
| M24 | la phrase de l'AC-7 (a) calculée sur la case **cochée** au lieu de la valeur enregistrée | AC-7 (a) |
| M25 | la phrase (4) de l'encadré (« aucune limite de débit ») retirée du composant | AC-6, encadré |
| M26 | `nav-admin-mcp` retiré des quatre catalogues, `'admin-mcp'` gardé à `FAMILLES_RESOLUES['nav-']` | AC-14 (d) — « toute clé demandée existe au catalogue » |

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
  les espions de l'AC-11 (v) couvrent les chemins **exécutés** par les tests seulement — et `indexedDB`, absent de
  jsdom, n'y est vu que par le substitut `vi.stubGlobal` : une écriture réelle dans le navigateur ne passe que par G-a.
- **Ce que l'utilisateur fait du secret** : l'historique du shell, `~/.claude.json` en clair, un `.mcp.json` versionné
  malgré l'avertissement. Le manuel le dit ; rien ne le contrôle.
- **La topologie réelle** (Traefik, Docker, NAS) : l'écran et le manuel **décrivent** ce que le filtre ne garantit pas ;
  rien ne détecte qu'on est derrière un mandataire non déclaré (non vérifié sur le NAS, point 2 de la fiche 32-6).
- **L'adresse affichée** (AC-7 (b)) est celle du navigateur ; un agent du réseau local peut devoir en employer une autre.
- **Les textes de portée et de politique** (AC-10, AC-13 (c)) sont tenus vrais par une **relecture** au code à l'étape 0
  et en revue, non par un test : seul l'ordre des portées l'est (AC-10), et les deux listes du manuel (G23).
- **L'E2E ne tourne qu'en `fr-CH`** ; la justesse des trois autres langues n'est vérifiée que par la parité des clés.
- **Une copie du secret dans un autre `$state`** du composant parent (`McpKeysSection`, `McpServerPanel`), hors de
  l'instance `SecretUnique` : aucune garde lexicale ne la voit, et aucune mutation ne la joue ; seuls le test du DOM
  après *Fermer* (AC-11 (i)) et la relecture en revue la limitent — une copie non rendue survivrait à *Fermer* sans que
  rien ne rougisse (validation P1, R, LOW non numéroté ; écrit plutôt que muté).
- **Le repli des clés hors `mcp-server-`** : l'AC-14 (a) ne tient égal au catalogue que le préfixe `mcp-server-` ; les
  autres domaines restent sous G13 et ses treize clés, ou sous aucune garde (G22 : AC-14 (c)) — classe tracée par
  **#758**. Dont **`nav-admin-mcp`** (validation P2, F1 = R1 ; C-32-7-23) : `FAMILLES_RESOLUES['nav-']` exige qu'elle
  existe au catalogue fr-CH, non que son repli `'Serveur MCP'` égale la valeur fr-CH, et ne voit pas l'oubli de
  `'admin-mcp'` à la famille (fait 21 ; AC-14 (d)) ; seule la relecture en revue le couvre.
- **Une ligne de réglages à `version = 0` posée hors de l'écran** (C-32-5-53) : l'écran ne la répare pas, chaque
  enregistrement rend un conflit ; seul le manuel le dit (AC-18 (1)).
- **Une ligne de réglages corrompue** (#738 B8 ; validation P2, R3 ; C-32-7-24) : `GET` en 500, l'écran affiche
  l'échec de lecture et n'enregistre rien (AC-5 (g)) ; aucune voie de réparation depuis l'écran sur une installation
  de production (AC-5, ⚠️ *Ce que l'écran ne répare pas* ; « dans Kesh » jusqu'en validation P3, F2 — la restauration
  est une fonction de Kesh) ; seul le manuel dit les voies hors de l'écran (AC-18 (1)).
- **Une entrée invalide des mandataires de confiance posée hors de l'écran** (validation P3, F7 ; P7-H3) : le serveur
  de la société est coupé alors que la case *Serveur activé* reste cochée ; l'écran ne signale rien avant
  l'enregistrement, qui rend le 400 nommant l'entrée. La réparation se fait à l'écran (AC-5, troisième état) ; seuls
  le manuel (AC-18 (4)) et le journal du serveur le disent.
- **Une clé dont le créateur est désactivé ou n'est plus Administrateur** (validation P3, F1 ; C-32-7-25) :
  elle n'authentifie plus, et l'écran la montre *Active*, sa dernière utilisation figée (AC-8 (b)) — la liste ne
  porte ni l'état ni le rôle du créateur. Seul le manuel le dit (AC-18 (2)) ; aucun test ne le voit.
- **Un réseau mappé de préfixe 96** (`::ffff:0:0/96`, validation P3, R1) : la 32-6a le ramène à `0.0.0.0/0`
  (`reseau_canonique`, `kesh-32-6/…/reseau.rs:34-43`), qui admet tout client IPv4 ; l'avertissement (b), qui lit le
  préfixe écrit (96), ne le signale pas. Cas improbable, écrit plutôt que traité.
- **Les dates** s'affichent dans le fuseau du navigateur ; un administrateur hors de Suisse verra l'heure de son poste.

## Inventaire des reports

Méthode : `grep -n '32-7'` sur l'epic, la fiche 32-5 (worktree de la 32-7), les fiches 32-6, 32-8, 32-9, 32-10a (leurs
worktrees), la spike (l. 945-948), et `gh issue view 738 --comments` — les points B2 à B8 sont dans un **commentaire** (« Relevés à la revue de code P1
de la Story 32-5c »), le corps de #738 étant celui de la 32-3 (validation P1, R) — (la phrase « l'écran de la 32-7 » et les points qui touchent
l'écran). Chaque report est rattaché à un AC, ou écarté avec sa raison.

| # | Source | Report | Où |
|---|---|---|---|
| R1 | epic l. 420-421, fiche 32-5 l. 59, 769-771 | section *Le serveur MCP* : création, affichage unique, portées, politique, filtre | AC-18 (1)-(4) |
| R2 | epic l. 421-422, spike l. 927-928 | clé en portée `local`/`user`, jamais en `.mcp.json` versionné sauf `${VARIABLE}` | AC-11, AC-18 (8) |
| R3 | epic l. 422-423, C-32-5-10 | `100.64.0.0/10` absente des défauts, à l'écran et au manuel | AC-6 (3), AC-18 (4), AC-25 |
| R4 | epic l. 423-424, C-32-5-11, Q7 | un serveur actif dans la sauvegarde l'est après l'import | AC-7 (c), AC-18 (1) |
| R5 | epic l. 424-429, C-32-5-48, fiche 32-5 l. 572-573 | renvois depuis admin 1414, 1416, 2467, 2472 ; utilisateur 190, 3204-3205, 3214-3215 | AC-19 |
| R6 | #738 B7 = E6 | horodatages des clés sans `Z`, des réglages avec | AC-4 |
| R7 | #738 B3 = E5, fiche 32-6 l. 1558 | `/0` accepté, admet tout client **de sa famille** (IPv4 ou IPv6 ; précisé en validation P3, R1) | AC-6 (b), (c), AC-18 (4) |
| R8 | #738 B2 = E4 | `010.0.0.0/8` mis en forme sans signal | AC-5 (c) (forme réaffichée) |
| R9 | #738 B5 = E2 | corps mal formé non traduit | **écarté** : l'écran n'envoie que des corps bien formés (AC-3 (b)) ; le défaut reste sur #738 |
| R10 | #738 B8 | `version: -1` → 409 ; une ligne de réglages corrompue → 500 au `GET` et au `PUT` | `version: -1` **écarté** : l'écran n'envoie que la version lue (AC-5 (b)) ; reste sur #738. Ligne corrompue **rattachée** (validation P2, R3) : AC-5 (g) (phrase de l'échec de lecture) et ⚠️ *Ce que l'écran ne répare pas*, AC-18 (1) (les voies hors de l'écran), C-32-7-24 |
| R11 | fiche 32-6 l. 1231-1252 (« Ce qu'elle laisse à la 32-7 » ; l. 1252-1271 à `dd4b8398`) | route absente, activation par société, filtre et mandataires, `Origin`, seul `Bearer` lu, 1 Mio, 30 s, journal, appel qui écrit, acteur au nom du créateur, `last_used_at`, `KESH_PUBLIC_BASE_URL`, ligne illisible, requêtes anonymes, `Forwarded`/`X-Real-IP`, 415, 400, connexion neuve | AC-18 (1), (2), (4)-(7) — dont, écrits en validation P2 (F2 = R9), « requête anonyme qui lit la base » et la conséquence de `X-Real-IP` à l'AC-18 (4) (fiche 32-6 à `dd4b8398`, l. 1269-1271) ; « une clé `required` ne peut appeler aucun outil » **écartée** (fausse après la 32-8, R16) ; « ligne de réglages illisible → désactivée » : **écrite** à l'AC-18 (1) (validation P1, R6 (a)), relue au code à l'étape 0 |
| R12 | fiche 32-6 l. 1002, 1310 (point 2) | Traefik et Docker devant le filtre | AC-6 (1), AC-18 (4) — tranché par Guy (2026-10-25) ; NAS non vérifié (*Ce que les gardes ne voient pas*) |
| R13 | fiche 32-6 l. 1311 (point 2 bis) | garde d'un pair non déclaré porteur d'en-tête | **sans objet** : garde non retenue (décision de l'orchestrateur, validation P1, R7) ; la phrase « en-tête ignoré » est à l'AC-18 (4) |
| R14 | fiche 32-6 l. 1015-1018 | `last_used_at` n'est pas « dernier appel » | AC-8 (b), AC-18 (2) |
| R15 | fiche 32-6 l. 1253-1257 | `README.md:233`, `website/roadmap.html:375` (clé dispensée) | **écarté** vers la 32-11 (fiches 32-6, 32-8 l. 1397-1401, 32-10a l. 1390) ; constaté, AC-20 |
| R16 | fiche 32-6 l. 1258-1263 | pas de CORS, `OPTIONS` → 405 | AC-18 (5) |
| R17 | fiche 32-6 l. 1267-1271 (B4) | message de type réponse → 400 | AC-18 (7) |
| R18 | fiche 32-6 l. 1272-1278 (E8 ; l. 1293-1299 à `dd4b8398`) | route absente, 401 unique (dont `Origin` — écrit en validation P2, R2 = F2, à l'AC-18 (5) et (7), avec le 403 d'une origine qu'aucune société servie n'admet ou répétée, C-32-6-52, C-32-6-53), cookie et PAT non lus, JSON, 1 Mio, `kesh_version`, pas de CORS, `/mcp` exact, en-têtes répétés, 202, erreurs sans `id`, `/0`, sous-chemin | AC-18 (1), (2), (4), (5), (7) ; AC-7 (b) ; « `kesh_version` seul outil, qui rend aussi le délai d'instruction » (32-6 l. 1275) **n'est pas porté** : il cesse d'être vrai avec la 32-9, et ce que rendent les outils revient à la 32-9 (`sec:mcp-outils`) et à la 32-11 (validation P1, R6 (b)) |
| R19 | fiche 32-6 l. 1279-1281 (A3 ; l. 1300-1302 à `dd4b8398`) | `admin-manual.tex:2935`, 128 → 129 | **écarté** : 32-10b puis 32-11 (AC-20) |
| R20 | fiche 32-6 l. 1282-1283, 32-9 l. 1328 | `api-external.md:185`, `:831` | **écarté** : 32-11 (AC-20) |
| R21 | fiche 32-6 l. 1402, 1469 (P7-H3 ; l. 1422-1428 à `dd4b8398`) | entrée invalide des mandataires coupe le serveur, filtre activé ou non ; second volet : 401 fixe de la clé d'une société écartée, depuis un pair qu'une autre société admet | AC-18 (4) (second volet écrit en validation P2, R9) |
| R22 | fiche 32-8 l. 917-918, 1406-1408 | montrer la politique, lien vers les envois, ranger `sec:mcp-envois`, retirer la phrase fausse | AC-13, AC-18 (3), (9) |
| R23 | fiche 32-8 l. 1034, 1574 | la 32-8c relit l'état de la 32-7 | **sans objet** ici (ordre : la 32-8c avant la 32-7) |
| R24 | fiche 32-8 l. 1507 (Q12) | compteur d'envois au menu | **écarté** : à Guy (point 3), non fait (C-32-7-15) |
| R25 | fiche 32-9 l. 914, 1315-1325, 1373, 1429 | ranger `sec:mcp-outils` sous *Le serveur MCP* | AC-18 (9) |
| R26 | fiche 32-9 l. 1339-1341, 1373 | à l'écran de création, ce que `contacts:read` démasque ; noms lisibles sans elle ; l'IBAN n'est pas une donnée du contact | AC-10, AC-18 (2) |
| R27 | fiche 32-10a l. 1006-1008, 1386-1387 | portée `prompts:write` (proposer, jamais approuver, ni lire ; une clé `required` peut proposer) | AC-10, AC-18 (2) |
| R28 | fiche 32-10a l. 1008, 1384 | le compte de l'admin l. 2935 à la 32-10b | **écarté** : 32-10b puis 32-11, comme R19 (AC-20, constaté, non porté ; reclassé en validation P1, R6 (c)) |
| R29 | spike l. 945-946 | configuration des deux clients, limite de 25 000 jetons, `Origin` ne protège que d'un navigateur | AC-18 (5), (8) |
| R30 | spike l. 946-947 (Q11 (b)/(c)) | création à la main d'un utilisateur `SELECT` | **sans objet** : Q11 (a) |
| R31 | spike l. 947-948 (Q10 (b)) | au-delà de 2 min Claude Code poursuit | **sans objet** : Q10 (c), la poignée |
| R32 | epic l. 192 (*Points d'attention*) | le manuel dit ce que le filtre ne garantit pas | AC-18 (4) |
| R33 | consigne de l'orchestrateur | pas de limite de débit, écrit | AC-6 (4), AC-18 (4) |
| R34 | registre `-325`, C-32-5-49 (l. 652-654) | l'entrée CHANGELOG des clés MCP « peut » dire leur conservation à la remise à zéro | AC-22 (ajouté en validation P2, R10) |

## Dev Notes

### Le secret : où il est, où il n'est jamais

```
POST /api/v1/admin/mcp/keys ──201 { …clé, key }──► mcp-server.api.ts
                                                   ├─ cle    (champ par champ, sans key) ──► (rien : la liste se recharge par GET)
                                                   └─ secret ──► McpKeysSection : secret.montrer(nom, s)   [instance SecretUnique, <script> d'instance]
                                                                   └─ McpKeySecretNotice : <code> unique
                                                                        Fermer ──► secret.oublier()  ; démontage ──► instance perdue
```

Jamais : la liste, un `$state` de module ou d'un `<script module>`, un stockage du navigateur, l'URL, `document.title`,
l'argument d'un `toast`, un `console.*`, un attribut HTML. La classe, indicative :

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
`docs/manual/fr/{admin-manual,user-manual}.{tex,pdf}` ; `CHANGELOG.md` ; `frontend/src/lib/shared/
i18n-repli-divergent-actif.test.ts` (extension de G13, AC-14 (a)) ; `frontend/src/lib/shared/i18n-keys.test.ts`
(`FAMILLES_RESOLUES['nav-']`, `sitesTotal`, AC-14 (d) ; validation P2, F1) ; et, si le déplacement l'exige, le test de la
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
   administre aujourd'hui. *Recommandation* : l'accepter ; l'ajouter demanderait de toucher la route. *Conséquence
   écrite en validation P3 (F1, C-32-7-25)* : une clé dont le créateur est désactivé ou n'est plus Administrateur
   n'authentifie plus et reste affichée *Active* ; le manuel le dit, l'écran non. Un état « inopérante » à l'écran
   demanderait, lui aussi, que la route joigne le créateur.
5. ~~**La configuration d'un client au manuel**~~ — **réglé** (validation P1, R11 ; décision acceptée de Guy relayée par
   l'orchestrateur) : la configuration de Claude Code reste marquée « non éprouvée » au manuel, et le manuel n'écrit rien
   de Hermes qui ne soit sourcé (AC-18 (8)). La **Q9** de la spike (« s'agit-il bien de Hermes Agent (Nous
   Research) ? ») reste **sans réponse** : à poser à Guy avec la 32-11, qui éprouvera le client.
6. **Pour information** : la capture du secret (recette visuelle) montre une clé de démonstration, révoquée aussitôt,
   et reste hors du dépôt.

## Demandes à l'orchestrateur

*(Demandes 1, 3, 4 et 5 de la création **closes** en validation P1, tranchées par l'orchestrateur : (1) le compte de
`admin-manual.tex:2935` à la 32-10b, puis à la 32-11 au dernier état (AC-20) ; (3) la garde « 2 bis » de la 32-6 n'est
pas retenue (AC-18 (4)) ; (4) G23 réservé à la 32-7 (AC-25) ; (5) l'ordre nominal, sans rechange (tête de fiche,
C-32-7-3). Elles restent ici barrées pour l'historique.)*

1. ~~**Le propriétaire du compte de `admin-manual.tex:2935`**~~ — close (voir ci-dessus).
2. **La fiche 32-10b** : ranger *Les prompts du serveur MCP* sous `\section{Le serveur MCP}` (`sec:mcp`) si la 32-7 a
   fusionné avant elle.
3. ~~**Le point 2 bis de la fiche 32-6**~~ — close.
4. ~~**Le numéro G23**~~ — close.
5. ~~**L'ordre**~~ — close.
6. **La propriété de `sec:mcp-client`** (validation P1, F8 ; C-32-7-22) : la 32-7 l'écrit et en est propriétaire ; la
   32-11 **ne rouvre pas** de section de configuration — *Brancher un agent* y renvoie et, une fois les gestes éprouvés,
   amende `sec:mcp-client` en place (retire « non éprouvée », corrige ce que l'essai réfute). *Demande* : l'inscrire à la
   section 32-11 de l'epic (`epic-32-serveur-mcp.md:491-493`, « manuel d'administration (*Brancher un agent*,
   configuration de Claude Code) ») **et à la fiche 32-9** (`32-9-outils-de-lecture.md:1326-1327`, « Ce que la 32-11 y
   écrit : *Brancher un agent* (configuration de Claude Code, la clé en portée `local`/`user`, …) » ; validation P2,
   R11), que la 32-7 ne touche ni l'une ni l'autre.
7. ~~**Le repli de `admin-backup-auto-reminder`**~~ — close en validation P2 (F3) : l'orchestrateur a ouvert **#758**
   (P3, `technical-debt`) pour la classe entière ; l'AC-14 (c) y renvoie.
8. **Le graphe de l'epic** (validation P2, F6 = R15) : `epic-32-serveur-mcp.md:214-222` place la 32-7 après la seule
   32-6 et l'écrit « en parallèle possible » avec la 32-8 et la 32-10a, contre l'ordre nominal acquis (C-32-7-3 : après
   les neuf prérequis). *Demande* : amender le graphe et la ligne « en parallèle » de l'epic, que la 32-7 ne touche pas.
9. **Le corps de #758** (validation P2, F1 = R1 ; C-32-7-23) : il écrit que la 32-7 étend G13 à « `mcp-server-`,
   `nav-admin-mcp` » ; `nav-admin-mcp` en est retirée (clé de donnée, hors de portée de `replisParCle()`), et reste
   un cas de la classe que #758 trace (seule `FAMILLES_RESOLUES['nav-']` la tient, sans comparer son repli).
   *Demande* : le dire en commentaire de #758 (écriture `gh` réservée à l'orchestrateur).
10. **La fiche 32-6** (validation P3, R1) : `32-6-endpoint-mcp.md` l. 1299 et 1580 (worktree `kesh-32-6`) écrivent
    « un `0.0.0.0/0` dans les plages admet tout client » ; son code admet tout client **IPv4** (`reseau.rs:57`, `:128` ;
    `IpNet::contains` faux entre familles), comme son registre l'écrit (`epic-32-choix-autonomes-326.md:597`).
    *Demande* : préciser « IPv4 » dans la fiche 32-6, que la 32-7 ne touche pas.

## À recontrôler à l'étape 0 du développement

- **Fusions** : 32-6a, 32-6b, 32-8a, 32-8b0, 32-8b, 32-8c, 32-9a, 32-9b, 32-10a — `git log --oneline origin/main` porte
  **chacune**, sinon **arrêt** (tête de fiche) ; état de la 32-10b (non prérequis). Ce que chaque story a **réellement** livré, contre ce que cette fiche prescrit d'en écrire (AC-10, AC-13 (c),
  AC-18) : chaque phrase, ligne lue citée ; une phrase non confirmée se réécrit.
- **Les routes de la 32-5c** (fait 1 à 5) inchangées ; un champ ajouté à `McpKeyResponse` par une story suivante entre
  à l'AC-3 (a).
- **`/admin/mcp/sends`** (32-8c) et `nav-admin-mcp-sends` : leur place relue (AC-2, AC-13 (d)) — présents par
  prérequis ; les sous-dossiers de `routes/(app)/admin/mcp/` d'autres stories, exclus de G-a.
- **`seedTestState`** (AC-16) : *tranché à la rédaction* (validation P1, R8 = F5) — `truncate_all` vide `mcp_keys` et
  `mcp_server_settings` ; aucun préréglage Comptable. *À recontrôler* : que `TABLES_TO_TRUNCATE` les porte toujours
  (`grep -nF '"mcp_keys"' crates/kesh-db/src/backup.rs`) ; *correction attendue si non* : rétablir un `afterAll` qui
  désactive le serveur et révoque les clés de la spec, et le déclarer.
- **Le 401 fixe** de la 32-6a (constante de `mcp/textes.rs`) pour l'AC-16 (e) ; la réponse d'`initialize` (200) sans
  `Origin`, depuis `127.0.0.1`.
- **`last_used_at`** : ce que pose `services::cles_mcp::authentifier` et quand (AC-8 (b)).
- La phrase « un en-tête `X-Forwarded-For` venu d'un pair non déclaré est ignoré » (AC-18 (4)) contre le code de la
  32-6a ; la « ligne de réglages illisible » (fiche 32-6, AC-a2 ; AC-18 (1)) ; les valeurs de l'AC-18 (7) (415, 400,
  1 Mio, 30 s, 202, 400 d'une réponse).
- **La place de `sec:mcp-outils` et `sec:mcp-envois`** dans le manuel, et l'ancrage du test de la matrice de la 32-9.
- **Les lignes du manuel** (fait 16, 17 ; AC-19, AC-20) : recomptées, la 32-8c ayant amendé l. 1414, 2230, 2237, 2249
  et `user-manual.tex:3027-3033`.
- **Le repli égal au fr-CH** (AC-14 (a)) : *tranché à la rédaction* — aucune garde ne le tenait, la story étend G13 de
  `i18n-repli-divergent-actif.test.ts` par préfixe. *À recontrôler* : que `REPLIS_A_SITE_UNIQUE`, `replisParCle` et
  `valeurDuCatalogueFr` existent encore sous ces noms (`grep -nF`) ; *correction attendue si non* : ancrer l'extension
  sur leur successeur et le dire. Et le commentaire de G22 (`textes_coherents.rs:1688`), à corriger (AC-14 (c)).
- **Les LOW de la validation P1 qui relèvent de l'étape 0**, chacun avec sa correction attendue :
  - les lignes de la fiche 32-6 citées par l'AC-6 (1), l'AC-18 (1) et l'inventaire (R11 à R21) — *si décalées* : citer
    la ligne relue ;
  - la phrase de chaque portée (AC-10, R4) contre `kesh_pieces`, `kesh_echeancier` et `contacts.rs:380-381` — *si le
    code dit autre chose* : réécrire la phrase, aux quatre locales et au bloc `% mcp-portees:` ;
  - la `### Ajouté` du CHANGELOG (AC-22, R3 ; amendé en validation P2, R14) — créée par la 32-8c : la compléter ;
    *si absente* : écart d'un prérequis, à déclarer à l'orchestrateur ;
  - la commande de l'AC-21 (F11) — *si `grep` reste la fonction `ugrep`* : `command grep` et fenêtres de 30, comme écrit ;
  - `navTestid` et `nav-group-administration` (AC-16 (a), F5) — *si renommés* : suivre le nom relu ;
  - le motif Comptable de `backup-schedule.spec.ts:200-218` (AC-16 (f)) — *si déplacé* : citer sa nouvelle place ;
  - le `browser` de `admin/backup/+page.ts:14` (AC-1, R9) — *si la garde ne le lit plus* : retirer la simulation et le
    dire ;
  - l'absence d'`indexedDB` sous jsdom (AC-11 (v), R1) — *si une version de jsdom ou un `fake-indexeddb` l'apporte* :
    espionner l'objet réel, témoin conservé.
- **G23** libre (AC-25) ; la mesure de G-a et G-b refaite (AC-15).
- **Ajoutés en validation P2**, chacun avec sa correction attendue :
  - les refus d'`Origin` (AC-18 (5), (7) ; R2 = F2) : 403 à l'étape 4 (aucune société servie ne l'admet, ou en-tête
    répété) et 401 fixe à l'étape 9 (la société de la clé ne l'admet pas), contre `kesh-api/src/mcp/transport.rs`,
    `reponse.rs` et `textes.rs` de la 32-6a — *si le code dit autre chose* : réécrire les deux phrases du manuel ;
  - « toute requête, même anonyme, lit la base » et la conséquence de `X-Real-IP` (AC-18 (4) ; F2 = R9), et le second
    volet de P7-H3 — *si le code dit autre chose* : réécrire ;
  - les voies de réparation d'une ligne de réglages corrompue ou à `version = 0` (AC-5, AC-18 (1) ; R3 ; C-32-7-24) :
    la remise à zéro toujours refusée au-delà de l'étape 2 (`routes/onboarding.rs`), la réparation au démarrage
    toujours sans `mcp_server_settings` (`grep -rn 'mcp_server_settings' crates --include=*.rs`), une ligne supprimée
    toujours rendue en défauts à `version: 0` — *si l'un change* : réécrire la phrase du manuel selon le code ;
  - `FAMILLES_RESOLUES['nav-']` et `ATTENDU.sitesTotal` de `i18n-keys.test.ts` (AC-14 (d) ; F1 = R1) — *si la 32-8c a
    ajouté `'admin-mcp-sends'` ou bougé `sitesTotal`* : partir de la valeur relue, déclarer la borne ; *si renommés* :
    suivre le nom relu ;
  - les clés du FTL qui contiennent `mcp-server-` sans le préfixer (fait 21) : `grep -nE '^[a-z-]*mcp-server-'
    crates/kesh-i18n/locales/fr-CH/messages.ftl` — *si une story fusionnée en ajoute* : le témoin `startsWith` de
    l'AC-14 (a) les couvre, rien à changer ;
  - les méthodes de `toast` (`svelte-sonner`, AC-11 (iv) ; R5) — *si la version installée en ajoute* : les simuler
    aussi ;
  - `sitesTotal` et le décompte des sites recomptés **au dernier commit de code**, non à l'étape 0 seulement.
- **Ajoutés en validation P3**, chacun avec sa correction attendue :
  - la famille d'adresses (AC-6 (b), (c), AC-18 (4) ; R1) : `to_canonical` du pair et des entrées de
    `X-Forwarded-For`, `IpNet::contains` entre familles, `reseau_canonique` d'un réseau mappé, contre le code fusionné
    de la 32-6a (`kesh-api/src/mcp/reseau.rs`, `transport.rs`) et la version d'`ipnet` du `Cargo.lock` — *si le code
    admet entre familles* : réécrire les phrases de l'écran et du manuel selon le code ;
  - la condition de vie d'une clé (AC-8 (b), AC-18 (2) ; F1) : `services/cles_mcp.rs` (créateur actif, rôle au moins
    `ROLE_CLE_MCP`, refus avant `touch_last_used`) et `list_by_company` — *si la route rend désormais l'état du
    créateur* : le dire à l'orchestrateur (un état à l'écran redeviendrait possible), sans l'ajouter d'office ;
  - la réparation d'un mandataire invalide à l'écran (AC-5, troisième état ; F7) : `validate_settings` refuse toujours
    l'entrée au `PUT` — *sinon* : réécrire la phrase du manuel ;
  - les entrées de l'export de `toast` (AC-11 (iv) ; R3 = F4) : toutes reprises par le simulacre ;
  - les étiquettes `sec:env-vars` et `sec:backup-ui-keshbackup` et la rangée `KESH\_PRODUCTION\_RESET` (AC-18 (1) ; R2,
    F2) — *si renommées ou déplacées* : suivre le nom relu.
- **Les échecs E2E attendus** : `docs/testing.md` § *Les échecs attendus*, à la date de l'étape 0.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

| Date | Étape | Résumé |
|---|---|---|
| 2026-10-25 | création | Fiche rédigée en autonomie (Opus 5.5), contre `966baed5`. 25 AC (A : 17, B : 7, C : 1), 9 tâches, 19 mutations, gardes G-a, G-b, G23 ; 33 reports inventoriés (R1 à R33 : 24 rattachés à un AC, 9 écartés ou sans objet avec leur raison — R9, R10, R15, R19, R20, R23, R24, R30, R31) ; choix C-32-7-1 à C-32-7-20 ; 4 modules, pas de découpage. Prompt de validation P1 : `327-validate-prompt-p1.md`. |
| 2026-10-25 | validation P1 | **Sonnet ×2** (lentilles R et F, contexte frais, prompt `327-validate-prompt-p1.md` ; rapports `kesh-gate-logs/327-validate-p1-R.md` 0/0/1/12 et `-F.md` 0/0/4/7) ⇒ **trend P1 0/0/5/19** (doublons R2 = F3 et R8 ≈ F5 comptés une fois par lentille). Prémisses relues au dépôt avant application ; **aucune réfutée**. Corrigés à la lecture : deux numéros de ligne des rapports (R6 (a) — la ligne illisible est à la fiche 32-6 l. 1247-1248, non 1241-1243 ; R12 — `mcp_server_settings.rs:35-38`, non `:36-39`) ; l'option préférée de l'orchestrateur pour F2 (ajouter `mcp-server-` aux `PREFIXES` de `i18n-un-repli-par-cle`) **écartée** sur lecture : ce test compare les replis d'une clé entre eux (`parTexte.size > 1`, l. 216), jamais au catalogue. **Décisions de l'orchestrateur appliquées d'abord** : F1 — ordre de rechange retiré, ordre nominal des neuf prérequis vérifié à l'étape 0 (arrêt sinon), AC-2 et AC-13 (d) sans branche conditionnelle, C-32-7-3 récrit et C-32-7-1 annoté ; R7 — branche « 2 bis » de l'AC-18 (4) retirée (une seule phrase), R13 passé « sans objet », Demandes 1, 3, 4, 5 closes ; F2 — G13 de `i18n-repli-divergent-actif.test.ts` étendue par préfixe (AC-14 (a), C-32-7-21, témoins rouge et vert, égalité relevé ↔ FTL, compte exact), commentaire de G22 corrigé (AC-14 (c)) ; F3 = R2 — G-b couvre `<script module>` et `context="module"` (deux témoins rouges `.svelte`, M23) ; F4 — AC-9 (e), test `formulaire_a_l_ouverture_requise_et_aucune_portee`, M20, M21 ; R1 — AC-11 (v) par `vi.stubGlobal('indexedDB', …)`, chaque espion (`Storage.prototype`, `history`, `indexedDB`) avec son appel témoin. **LOW intégrés** : R3 (fait 19, AC-22 avant `### Modifié`), R4 (a)-(c) (AC-10), R5 (AC-6 (1) aligné sur le manuel et la 32-6), R6 (ligne illisible à l'AC-18 (1), `kesh_version` à la 32-9/32-11, R28 écarté comme R19), R8 = F5 (fait `seedTestState`, aucun `afterAll`, motif Comptable, `nav-link-admin-mcp`, groupe replié), R9 (`browser` simulé et témoin), R10 (M4 à l'AC-9 (d) seul, deux `GET`), R11 (gestes sourcés un à un, Hermes Agent sans rien de plus que la spike, Claude Code « non éprouvée » — décision acceptée de Guy), R12 = F6 (AC-5 (g), version 0 au manuel), R13 (cardinalité exacte, premier code de ligne, extraction sur le source brut), LOW non numérotés de R (`gh issue view 738 --comments`, copie du secret dans un autre `$state` écrite en angle mort, `toast.*` tranché — permis, sans le secret, avec témoin —, M24, M25, précédent TZ cité), F7 (sept captures, 28 au total), F8 (propriété de `sec:mcp-client` à la 32-7, C-32-7-22, demande 6), F9 (signature de `SecretUnique` fixée, M1 à la sonde de (ii)), F10 (gardes nommées, déclencheur *Révoquer* sans rouge à la main), F11 (`command grep`, fenêtres de 30, `25\,000`). Relevé en passant : l'AC-2 renvoyait le Comptable à l'AC-16 (e), lire (f). **Symptômes grepés** (valeurs) sur la fiche, le registre et les fiches voisines (`rechange`, `32-7a`, `2 bis`, `afterAll`, `Hermes`, `rattachements`, `i18n-un-repli-par-cle`, `sec:mcp-client`, `{0,80}`, `repli … égal`) : résidus de la fiche tous historiques ; voisines — note à l'orchestrateur (rapport). **Décomptes** recomptés à la source : 25 AC (A : 17, B : 7, C : 1), 9 tâches, **25 mutations** (M1 à M25), 33 reports (**22 rattachés, 11 écartés ou sans objet** — R9, R10, R13, R15, R19, R20, R23, R24, R28, R30, R31), **22 choix** (C-32-7-1 à C-32-7-22 ; 1, 3, 13, 19 annotés), 28 captures, 4 modules. Remédiation sur la fiche et le registre seulement ; **1 passe sur 8** ; passe suivante **complète** (P2, Opus, lentilles R et F, `327-validate-prompt-p2.md`). |
| 2026-10-25 | validation P2 | **Opus ×2** (lentilles R et F, contexte frais, prompt `327-validate-prompt-p2.md` ; rapports `kesh-gate-logs/327-validate-p2-R.md` 0/0/3/12 et `-F.md` 0/0/2/13) ⇒ **trend P1 0/0/5/19 → P2 0/0/5/25** (doublons R1 = F1, R2 ⊂ F2, R9 ⊂ F2, R4 ≈ F8, R7 ≈ F10, R15 = F6, comptés une fois par lentille). ⚠️ **Recyclage déclaré** : **F1 = R1 naît de la remédiation P1** (C-32-7-21 avait nommé `nav-admin-mcp` dans l'extension de G13) ; de même les LOW F3 (renvoi à la demande 4 close), F4 (C-32-7-14 non annoté), F5 (« sinon omis ») et R14 (« sinon créée »). R2 = F2 et R3 sont des **omissions d'origine** (le 403/401 de l'`Origin`, #738 B8 seconde moitié), non des régressions. Sévérité MEDIUM → MEDIUM : signal de découpage de `CLAUDE.md` (amendement D5) — un MEDIUM recyclé sur trois ; **déclaré au Project Lead** par le rapport. **Prémisses relues au dépôt avant application ; aucune réfutée** ; précisées à la lecture : F1 proposait une mutation « `'admin-mcp'` retiré de `FAMILLES_RESOLUES` » à vérifier — elle **ne rougirait pas** (`'nav-'` hors de `PREFIXES_A_COUVERTURE_CLOSE`, `i18n-keys.test.ts:932`), écrite en angle mort, et la mutation retenue est M26 (clé retirée des catalogues) ; R3 — le 500 au `PUT` confirmé au code (`routes/mcp_server_settings.rs:245`, l'« avant » relu par `get_or_defaults`) ; R5 — les méthodes de `toast` relues dans `svelte-sonner` 1.1.0 (`toast-state.svelte.js:213-225`, dépôt principal en lecture) ; R13 — lignes de la fiche 32-6 relues à `dd4b8398`. **Décisions de l'orchestrateur appliquées** : R1 = F1 — `nav-admin-mcp` retirée de l'extension (AC-14 (a)), tenue par `FAMILLES_RESOLUES['nav-']` (AC-14 (d), fait 21 neuf), qui **ne compare pas** le repli au fr-CH : angle mort écrit, #758 cité ; `sitesTotal` (l. 724, assertion l. 1034) recompté et commenté par la story, `i18n-keys.test.ts` au § *Fichiers touchés* ; fait sourcé : aucune clé `mcp-server-` n'existe ni n'est résolue par une table (`grep -rnF 'mcp-server-' frontend/src` → rien), et le FTL porte deux clés `audit-log-…-mcp-server-…` qui contiennent la chaîne — l'appartenance au préfixe se juge par `startsWith`, avec un témoin ; C-32-7-23. R2 = F2 — AC-18 (5) : 403 (aucune société servie n'admet l'origine, ou `Origin` répétée) et 401 fixe (la société de la clé ne l'admet pas), phrases du manuel ; AC-18 (7) y renvoie, avec les quatre en-têtes répétés ; F2 = R9 — requête anonyme qui lit la base, conséquence de `X-Real-IP`, second volet de P7-H3, à l'AC-18 (4). R3 — AC-5 (g) : phrase fixe de l'état d'échec de lecture ; ⚠️ *Ce que l'écran ne répare pas* : ligne à `version = 0` et ligne corrompue ; **aucune voie de réparation dans Kesh** sur une installation de production (remise à zéro refusée au-delà de l'étape 2, `routes/onboarding.rs:252-268` ; réparation #528 sans `mcp_server_settings`) — le manuel (AC-18 (1)) donne les voies hors de l'écran : restaurer une sauvegarde antérieure, ou supprimer la ligne par SQL ; C-32-7-24 ; R10 de l'inventaire rattaché. **LOW intégrés** : R4 = F8 (`mockClear()` après chaque témoin ; secret simulé `kesh_mcp_SECRET…` distinct des témoins `kesh_mcp_TEMOIN`), R5 (`toast` lui-même simulé, ses neuf méthodes), R6 (« même sens que »), R7 = F10 (`--scope local` sourcé à la spike l. 320 ; forme JSON de `.mcp.json` retirée ; Hermes : « et rien d'autre » retiré, transport par défaut non nommé par sa référence, Kesh en Streamable HTTP), R8 (les trois issues d'outil), R10 (C-32-5-49 rattaché à l'AC-22, inventaire R34), R12 (G-b après `masquerCommentaires`, témoin vert commentaire), R13 (lignes de la 32-6 à `dd4b8398`), R14 (AC-22 sans branche « sinon »), F3 (#758 au lieu de la demande 4), F4 (C-32-7-14 annoté), F5 (AC-18 (6) sans « sinon omis »), F7 (G23 (b) par rangée, découpée sur `\\`, témoin d'une rangée sur deux lignes), F9 (`oublier()` avant l'envoi du `POST`), F11 (capture (11) étendue au nom vide, capture (29) du toast de copie), F12 (G-a sans descente dans `admin/mcp/`), F13 (« reconnue, même si l'appel est ensuite refusé ou n'est pas servi »), F14 (AC-5 (h), phrase sous *Origines admises*), F15 (les deux gardes nommées). **Aux demandes à l'orchestrateur** : 6 étendue à la fiche 32-9 (R11), 7 close (#758), 8 neuve — le graphe de l'epic (F6 = R15), 9 neuve — un commentaire sur #758 (`nav-admin-mcp` retirée). **Symptômes grepés** (valeurs) sur la fiche et le registre : `nav-admin-mcp` (restent l'AC-2, l'AC-14, la donnée de menu, C-32-7-2 et l'historique de C-32-7-21, annoté), `403` (les seuls sites neufs), `demande 1` (la seule annotation de C-32-7-14), `demande 4`, `sinon omis`, `sinon créée`, `acceptée, même` → rien. **Décomptes** recomptés à la source : 25 AC (A : 17, B : 7, C : 1), 9 tâches, **26 mutations** (M1 à M26), **34 reports** (**24 rattachés, 10 écartés ou sans objet** — R9, R13, R15, R19, R20, R23, R24, R28, R30, R31), **24 choix** (C-32-7-1 à C-32-7-24 ; 14 et 21 annotés), **29 captures**, **21 faits**, 4 modules. Remédiation sur la fiche et le registre seulement ; **2 passes sur 8** ; passe suivante **complète** (P3, **Sonnet**, lentilles R et F, `327-validate-prompt-p3.md`). |
| 2026-10-25 | validation P3 | **Sonnet ×2** (lentilles R et F, contexte frais, prompt `327-validate-prompt-p3.md` ; rapports `kesh-gate-logs/327-validate-p3-R.md` 0/0/1/2 et `-F.md` 0/0/1/7) ⇒ **trend P2 0/0/5/25 → P3 0/0/2/9** (doublon R3 = F4 compté une fois par lentille). **Les deux MEDIUM sont des omissions d'origine**, non des régressions : R1 (`0.0.0.0/0` et `::/0` n'admettent chacun que leur famille ; la fiche disait « tout client » depuis la création, en étendant la phrase de la fiche 32-6 à `::/0`) et F1 (une clé dont le créateur est désactivé ou n'est plus Administrateur n'authentifie plus et reste affichée *Active*). **Les LOW naissent de la remédiation P2** (R2, R3 ; F2 à F8, déclarés tels par leur lentille). Sévérité MEDIUM → MEDIUM : défauts **distincts** et **non issus d'une remédiation** — selon l'amendement D5 de `CLAUDE.md`, ce n'est pas un signal de découpage ; **déclaré au Project Lead** par le rapport. **Prémisses relues au code avant application ; aucune réfutée.** Précisées à la lecture : R1 cite `ipnet-2.12.2` — le `Cargo.lock` verrouille **2.12.0** (l. 1872-1873), dont `Contains<&IpAddr> for IpNet` (`src/ipnet.rs:1419-1426`) rend le même `false` entre familles ; F2 situe `sec:backup-ui-keshbackup` à `admin-manual.tex:1926` — elle est à **l. 2177** ; R3 et F4 situent `getActiveToasts` aux l. 226-228 et 225-227 — elle est aux **l. 223-225**, l'objet de `Object.assign` aux l. 213-226 ; F7 cite `routes/mcp_server_settings.rs:205-206` — `validate_settings` occupe les l. 198-208 (`trusted_proxies` l. 205). **Fait sourcé (R1)**, code de la 32-6a (`kesh-32-6`, branche `story/32-6`, PR #760) : le pair est ramené à sa forme canonique (`transport.rs:81`, `reseau.rs:64`, entrées de `X-Forwarded-For` l. 73 ; `IpAddr::to_canonical`, doc l. 9-12) — une adresse IPv4 mappée compte comme IPv4 —, puis comparé par `IpNet::contains` (l. 57, 128). **Décisions de l'orchestrateur appliquées** : R1 — AC-6 (b) et (c) nomment la famille (« `0.0.0.0/0` admet tout client IPv4 », « `::/0` tout client IPv6 », « pour admettre les deux familles, il faut les deux entrées »), même règle pour un mandataire `/0` ; tests de chaque famille et de l'absence de « toute adresse » sans famille ; phrase de l'AC-18 (4) récrite ; capture (4) dans les deux familles ; inventaire R7 précisé ; relevé en passant : un réseau mappé `::ffff:0:0/96` devient `0.0.0.0/0` à la 32-6a (`reseau_canonique`, l. 34-43) sans avertissement à l'écran — écrit à *Ce que les gardes ne voient pas* ; demande 10 neuve (fiche 32-6 l. 1299 et 1580). F1 — l'écran **ne peut pas** le dire sans élargir la story (`list_by_company`, `kesh-db/src/repositories/mcp_keys.rs:197-217`, sans jointure ; `McpKeyResponse` sans état du créateur) : phrase au manuel (AC-18 (2)), mention à l'AC-8 (b) (dont « reconnue » précisé : le refus du créateur précède la pose de la dernière utilisation, `services/cles_mcp.rs:94-100`), ligne à *Ce que les gardes ne voient pas*, point 4 à Guy complété ; **C-32-7-25** neuf. **LOW intégrés** : F2 (suppression par SQL d'abord, restauration en dernier recours — elle remplace toute l'installation et refuse une sauvegarde d'une version qui ne porte pas les mêmes tables, `admin-manual.tex:2249`, `:2262`, renvoi `sec:backup-ui-keshbackup` ; « dans Kesh » → « depuis l'écran » à l'AC-5 et aux gardes ; C-32-7-24 **annoté**, non réécrit), R2 (renvoi à la rangée `KESH\_PRODUCTION\_RESET`, `admin-manual.tex:767`, sous `sec:env-vars`, l. 705), R3 = F4 (« toutes les entrées de l'export réel », dont `getActiveToasts`), F3 (l. 122), F5 (second motif du contrôle aplati, rendu de `mcp_server_settings`, renvois de l'AC-18 (1) sans `??`), F6 (« un tableau qui ne contient pas que des chaînes », la base garantissant un tableau JSON valide, migration `20261025000001`, l. 81-83 ; recette SQL de la capture (22)), F7 (troisième état à l'AC-5, qui **se répare à l'écran** ; phrase à l'AC-18 (4) ; ligne aux gardes), F8 (voie utile du 403 et du 401 : *Origines admises* de l'écran ; lignes `transport.rs:105-111` et `:151-162` relues). Ajouts au § *À recontrôler* (cinq points de la P3). **Symptômes grepés** (valeurs) sur la fiche, le registre, l'epic, le manuel et le FTL fr-CH : `0.0.0.0/0`, `::/0`, `tout client`, `toute adresse`, `filtre désactivé`, `aucune voie`, `neuf méthodes` — restent les sites neufs de la P3, l'entrée P2 de ce Change Log et le texte d'origine de C-32-7-24 (annoté) ; hors du worktree, la fiche 32-6 (demande 10). **Décomptes** recomptés à la source : 25 AC (A : 17, B : 7, C : 1), 9 tâches, 26 mutations (M1 à M26, aucune neuve), 34 reports (24 rattachés, 10 écartés ou sans objet — inchangé), **25 choix** (C-32-7-1 à C-32-7-25 ; 14, 21 et 24 annotés), 29 captures, 21 faits, 4 modules. Remédiation sur la fiche et le registre seulement ; **3 passes sur 8** ; passe suivante **complète** (P4, **Opus**, lentilles R et F, `327-validate-prompt-p4.md`). |
