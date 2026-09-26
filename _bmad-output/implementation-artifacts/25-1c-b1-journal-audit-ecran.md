# Story 25.1c-b1 : L'écran de consultation du journal d'audit

Status: ready-for-dev

⚠️ **RÉOUVERTE le 2026-09-15 au soir**, après sa validation en 7 passes : trois arbitrages du Project Lead
changent le contrat de la 25-1c-a, donc ce que l'écran consomme et affiche (cf. « Réouverture » au Change
Log). **Revalidation requise** avant implémentation.

⚠️ **Issue du SPLIT de la 25-1c-b** (arbitrage du Project Lead du 2026-09-15, après la passe 2 de
validation — sévérité MEDIUM → MEDIUM sur des défauts d'origine répartis dans cinq zones) :

| | objet | état |
|---|---|---|
| 25-1c-zero | la colonne `audit_log.company_id` | done — PR #437 ouverte |
| 25-1c-a | la route de consultation, son vocabulaire traduit et son export CSV — backend | ready-for-dev, réouverte |
| **25-1c-b1** *(celle-ci)* | **l'écran : feature, page, garde, menu, gardes i18n, tests** | ready-for-dev, réouverte |
| 25-1c-b2 | les textes : manuels, README, vocabulaire « journal d'audit » | ready-for-dev, réouverte |

La fiche parente `25-1c-b-journal-audit-ecran.md` (statut `split`) reste la référence des passes 1 et
2 ; **ne pas l'implémenter**.

⛔ **Dépendance dure** : cet écran consomme `GET /api/v1/audit-log`, `/export.csv` et `/vocabulary`
**tels que la 25-1c-a les livre** (`25-1c-a-journal-audit-route.md`, AC 5-17 — le comportement HTTP, les
libellés traduits, et la clé du message de refus). **Ne pas implémenter avant le merge de la 25-1c-a** —
la branche est empilée sur la sienne et se rebase sur `main` après.

⛔ **Livrée dans la MÊME PR que la 25-1c-b2**, et c'est cette PR qui **ferme #378** : son titre porte
`closes #378` (le dépôt merge en squash — § *Commits qui adressent une issue*). Sans la b2, le manuel
continuerait d'affirmer que l'écran n'existe pas.

⚠️ **Hors de cette story, par arbitrage** : le regroupement des sept copies existantes de la logique
de téléchargement → **issue de dette [#438]**. Cette story crée le module partagé et ne migre aucune
copie (AC 9).

## Story

**En tant que** comptable ou administrateur,
**je veux** consulter le journal d'audit de ma société dans l'application — le filtrer, déplier le
détail d'une entrée, l'exporter —, dans la langue de l'interface,
**afin de** **produire** la trace des corrections apportées aux livres sans accès direct à la base —
« *"apparent" et "archivé dans une table que personne ne peut lire" ne sont pas la même chose* » (#378).

**Couvre** : [#378], volet écran.

## ✅ Arbitrages du Project Lead — 2026-09-15 (`epic-25-vague1-suite.md`)

1. **Qui consulte** : **Comptable** et **Admin** ; ni Consultation, ni clé API.
2. **Filtre strict par société**, porté par la route — et il n'existe **pas** d'entrée sans société à
   montrer (*« le cas réel n'existe pas […] le cas théorique est … théorique »*).
3. **Affichage** — cité de l'epic : *« le code d'action tel quel, les types d'entité traduits, les
   détails en JSON indenté »* ; **amendé au soir** : à la proposition d'afficher l'action traduite à
   l'écran comme dans le CSV, *« ok »*. ⇒ **actions et types d'entité affichés traduits**, détails en JSON
   indenté.
4. **Vocabulaire** : **« journal d'audit »** — `Audit-Protokoll`, `registro di audit`, `audit log`.
5. **Découpage** (soir du 2026-09-15) : écran (b1) et textes (b2) dans la même PR ; le téléchargement
   partagé sort vers une issue de dette.
6. **L'export n'est pas inscrit au journal** — sans effet sur l'écran, sinon qu'aucun type d'entité
   `audit_log` n'existe.

⛔ **L'écran ne traduit RIEN du vocabulaire lui-même.** La 25-1c-a est la source unique des libellés
(son AC 16) : la liste rend `actionLabel` et `entityTypeLabel` (son AC 8), la route de vocabulaire rend les
deux listes traduites (son AC 17). Aucune carte de codes, aucune clé `audit-log-entity-*` ni
`audit-log-action-*` n'est appelée par le frontend.

## Acceptance Criteria

### Volet A — la feature `frontend/src/lib/features/audit-log/`

**1. Les types** — `audit-log.types.ts` :

- `AuditLogEntry` en miroir **exact** du DTO de la 25-1c-a (AC 8) — `id`, `createdAt` (chaîne ISO UTC
  finissant par `Z`), `actorLabel`, `actorType: 'user' | 'api_key'`, `actorApiKeyId: number | null`,
  `userId`, `action`, `actionLabel`, `entityType`, `entityTypeLabel`, `entityId`, `details: unknown`.
  ⚠️ **Pas de `companyId`** : la route ne le rend plus ;
- `AuditLogVocabularyItem { code: string; label: string }` et `AuditLogVocabulary { entityTypes:
  AuditLogVocabularyItem[]; actions: AuditLogVocabularyItem[] }` (25-1c-a AC 17) ;
- `AuditLogQuery` (`dateFrom`, `dateTo`, `entityType`, `entityId`, `action`, `offset`, `limit`) ;
- liste : l'enveloppe `{ items, total, offset, limit }` déjà typée (`src/lib/shared/types/user.ts:22-27`).

**2. L'API** — `audit-log.api.ts`, via `apiClient` :

- `listAuditLog(query)` → `GET /api/v1/audit-log` ; **un paramètre vide ou absent n'est pas envoyé** ;
- `getAuditLogVocabulary()` → `GET /api/v1/audit-log/vocabulary` ;
- `exportAuditLogCsv(query)` → `apiClient.getBlob(url)` (`api-client.ts:526-529`), **l'URL portant les
  mêmes filtres que la liste, `offset` et `limit` exceptés** ; nom de fichier lu dans
  `Content-Disposition` par `parseContentDispositionFilename`, téléchargement par `triggerDownload` —
  **tous deux importés du module partagé de l'AC 9**, jamais recopiés.

**3. Les listes des filtres** — `vocabulary-options.ts`, fonction pure
`toSelectOptions(items, locale, current)` :

- **tri par libellé** avec `Intl.Collator(locale, { sensitivity: 'base' })` — la route rend l'ordre des
  **codes** (25-1c-a AC 17), et un tri d'octets rangerait « Écriture » après « Utilisateur » ;
- ⛔ **un code de l'URL absent du vocabulaire est ajouté comme option, libellée par son code** : un code
  historique doit rester filtrable **et visible** dans la liste, sans quoi la liste afficherait « Tous »
  alors que la page filtre sur ce code ;
- `locale` est celle de l'interface — ⛔ **et le frontend la JETTE aujourd'hui** : `loadI18nMessages`
  (`src/lib/shared/utils/i18n.svelte.ts:25-35`) reçoit `{ locale, messages }` et ne garde que `messages`.
  ⇒ la story ajoute **`i18nLocale()`** au même module, qui conserve `data.locale` (`'fr-CH'` avant tout
  chargement) ; le mock de `$lib/shared/utils/i18n.svelte` des tests de page l'exporte aussi (patron
  `contacts-page.test.ts:21-25`). Sans cela, un développeur fixerait `'fr-CH'` en dur ou prendrait
  `navigator.language`, et l'écran d'une installation allemande afficherait des dates françaises.

**4. Les paramètres d'URL** — `query-helpers.ts` sur le patron de
`journal-entries/query-helpers.ts:19-121` : sérialisation qui **omet** les valeurs par défaut, lecture qui
**ignore** les valeurs invalides — date mal formée, `entityId` non numérique ou `<= 0`. `entityType` et
`action` **hors vocabulaire sont conservés** (un code historique doit rester filtrable).

⛔ **`entityId` n'a de sens qu'avec un `entityType`** — la route répond 400 sinon (25-1c-a). Donc :

- à la **lecture** de l'URL, un `entityId` **sans** `entityType` est **ignoré** (un lien retouché à la
  main ne doit pas produire un écran en erreur) ;
- à la **sérialisation**, un `entityId` sans `entityType` **n'est pas écrit**.

### Volet B — l'écran `frontend/src/routes/(app)/audit-log/`

**5. La page** — `+page.svelte` :

- **au montage**, le vocabulaire et la première page de la liste se chargent ; ⛔ un **échec du
  vocabulaire** affiche l'**état d'erreur** de la page — des filtres vides laisseraient croire qu'il n'y a
  rien à filtrer ;
- **filtres** :
  - **période** (`Input type="date"`, du / au), **libellée comme des jours UTC** — « Du (jour UTC) » /
    « Au (jour UTC) », ou un libellé de groupe « Période (jours UTC) » ;
  - **type d'entité** : liste « Tous » + les types du vocabulaire, triés par libellé (AC 3) ;
  - **identifiant d'entité** (numérique, **désactivé tant qu'aucun type n'est choisi**) ;
  - **action** : liste « Toutes » + les actions du vocabulaire, triées par libellé (AC 3) ;
  - ⛔ **les deux listes sont des `<select>` NATIFS**, avec `data-testid` et un **`<label for>`** chacune,
    sur le patron `invoice-paused-filter` de `routes/(app)/invoices/+page.svelte:314-318`, piloté par
    `invoices.spec.ts:167`. **Motif** : un `<select>` natif se pilote par **valeur**,
    `selectOption({ value: code })`, sans dépendre du texte traduit. Le `Select` de bits-ui pourrait l'être
    aussi (il pose `data-value` sur ses options, `bits-ui/dist/bits/select/select.svelte.js:909`), mais **aucun
    E2E du dépôt ne le fait** : les précédents le pilotent par libellé (`accounts.spec.ts:136`) ou par
    position. ⚠️ **C'est la spec, et non la garde, qui interdit le pilotage par libellé** :
    `e2e-selecteurs-traduits.test.ts` laisse passer `selectOption({ label: … })` et un
    `getByRole('option', { name })` sans accent ;
  - tous **synchronisés dans l'URL** et relus au montage (patron `journal-entries/+page.svelte:138-183`) ;
    bouton « Réinitialiser ».

  ⛔ **Revenir à « Tous » VIDE l'identifiant d'entité** : désactiver le champ ne suffit pas, sa valeur
  resterait dans l'état et partirait à la route, qui répondrait 400.

  ⛔ **Pourquoi « jours UTC »** : la route borne les dates sur des **jours UTC**
  (`date_from 00:00:00.000` à `date_to 23:59:59.999`, en UTC — 25-1c-a), alors que le tableau affiche
  l'heure **locale**. Une modification faite à Zurich le 16 à 00:30 s'affiche le 16, mais tombe dans le
  jour UTC du **15** : sans le libellé, le filtre « du 16 au 16 » l'écarterait sans explication. Le
  contrat de la route est arbitré et ne se change pas ici ; l'écran **dit** ce qu'il filtre.
- **tableau**, colonnes :
  - **Date** : heure **locale** du navigateur,
    `Intl.DateTimeFormat(i18nLocale(), { dateStyle: 'medium', timeStyle: 'short' })` (AC 3), depuis
    `createdAt` UTC — la forme change réellement avec la langue : `16 sept. 2026, 14:26` en `fr-CH`,
    `16.09.2026, 14:26` en `de-CH` (Node 22, ICU 78 ; **sans** ces options, `16.09.2026` et `16.9.2026`) ;
  - **Auteur** : `actorLabel`, avec la mention « clé API » si `actorType === 'api_key'` ;
  - **Action** : **`actionLabel`** ;
  - **Type d'entité** : **`entityTypeLabel`** ;
  - **N°** : `entityId`, « — » s'il vaut `0` ;
  - **Détails** : bouton qui déplie un `<pre>` de `JSON.stringify(details, null, 2)`, absent si
    `details` est `null` ;
  - chaque ligne porte **`data-action`** et **`data-entity-type`** avec les **codes** : c'est par eux, et
    non par le texte traduit, que les tests identifient une ligne.

  ⚠️ **Plus de colonne « Société »** : le filtre est strict (arbitrage 2).
- **pagination** Précédent / Suivant, « X–Y sur N », `limit` 50, garde `if (loading) return` (patron
  `journal-entries/+page.svelte:245-265`) ;
- **trois états distincts** — chargement, **erreur** (message, jamais une liste vide silencieuse —
  patron `supplier-invoices/+page.svelte:429-438`), vide. ⚠️ **L'erreur de la LISTE remplace le tableau
  seul** : les filtres et « Réinitialiser » restent visibles, pour qu'on sorte d'une erreur de filtre. Des
  valeurs valides en forme mais refusées par la route — une plage inversée, une date hors
  `[1000-01-01, 9999-12-31]` dans l'URL, une action de plus de 64 caractères — produisent un 400 dont le
  **message du serveur** s'affiche ; c'est **assumé**, et `query-helpers` ne les écarte pas ;
- **export CSV** : bouton qui transmet **les filtres affichés**, désactivé pendant l'export. Le fichier
  est traduit par le serveur, dans la langue de l'interface (25-1c-a AC 12). Le traitement des erreurs :
  - un 400 dont `code === 'RESULT_TOO_LARGE'` affiche, **dans la page** (`audit-log-export-error`),
    **le message rendu par le serveur** — la route le traduit déjà (`audit-log-export-error-too-large`,
    argument `limit`, 25-1c-a AC 11 ; clé inscrite au catalogue par son AC 15), et `parseErrorResponse` le
    transmet tel quel dans `ApiError.message` (`api-client.ts:227-229`). **Aucune clé frontend neuve**
    pour ce message ;
  - toute autre erreur passe par `notifyError`.

**6. La garde** — `+page.ts`, patron **exact** de `supplier-invoices/import/+page.ts` : `ssr = false`,
redirection 302 vers `/` si le rôle n'est ni `Admin` ni `Comptable`. ⚠️ Le masquage d'interface ne
protège rien : c'est le 403 de la route qui fait foi.

**7. Le menu** — entrée `{ i18nKey: 'nav-audit-log', fallback: "Journal d'audit", href: '/audit-log' }`
dans la liste **`comptableOnly`** du groupe `administration` (`routes/(app)/+layout.svelte:130-136`).
⚠️ **`'audit-log'` s'ajoute à `FAMILLES_RESOLUES['nav-']`** (`i18n-keys.test.ts`, bloc `FAMILLES_RESOLUES`, `:428-436` au 2026-09-26) : une clé de
menu est une donnée lue par `getItemLabel`, et c'est ce trou qui avait laissé quatre entrées en
français dans les quatre langues (commentaire `i18n-keys.test.ts:311-314`).

**8. Les sélecteurs** — `data-testid` sur tout ce que les tests visent : `audit-log-table`,
`audit-log-row`, `audit-log-row-action`, `audit-log-empty`, `audit-log-error`, `audit-log-loading`,
`audit-log-export`, `audit-log-export-error`, `audit-log-filter-date-from`, `-date-to`, `-entity-type`,
`-entity-id`, `-action`, `-reset`, `audit-log-details-toggle`, `audit-log-details`, `audit-log-prev`,
`audit-log-next`. Le lien de menu reçoit `nav-link-audit-log` du gabarit existant
(`+layout.svelte:258-261`). ⛔ **Aucun sélecteur par libellé**, `selectOption({ label })` compris — la
spec l'interdit, même là où `e2e-selecteurs-traduits.test.ts` ne le voit pas (AC 5) ; `DETTE_CONNUE` ne
s'allonge pas.

### Volet C — le module de téléchargement partagé

**9. `frontend/src/lib/shared/utils/download.ts`** (neuf) exporte `triggerDownload(blob, filename)` et
`parseContentDispositionFilename(header)`, reprises **à l'identique** de
`features/export/exports.api.ts:52-110` — la forme `try/finally` de `triggerDownload`, **pas** la
libération différée des trois copies écrites dans les pages.

- ⛔ **Aucune copie existante n'est migrée** : c'est l'objet de [#438], qui recense les sept copies
  (quatre fonctions dans des `.api.ts`, trois écrites en ligne dans des pages, au comportement
  différent) et les deux définitions de `parseContentDispositionFilename`.
- **Les commentaires qui parlent de l'extraction sont mis à jour** pour dire que le module existe et que
  la migration est suivie par #438 — sans quoi ils continueraient d'annoncer une extraction « reportée » à
  un module déjà présent :
  - **deux la planifient**, et se réécrivent : `admin-backup.api.ts:10-14` (DC-B1) et `exports.api.ts:89-92` ;
  - **un la justifie par renvoi** à la décision d'`exports.api.ts` : `imported-supplier-invoices.api.ts:66-69`,
    qui renvoie désormais à `download.ts` et à #438 ;
  - `reports.api.ts:338-346` ne parle **pas** d'extraction (il justifie le `finally`) : **il ne se touche pas**.

  **Le commentaire seulement** : aucune ligne de code de ces fichiers ne bouge.

**Tests** — `download.test.ts` :

- `parseContentDispositionFilename` : les cas de `exports.api.test.ts` (`describe
  'parseContentDispositionFilename'`), **recopiés sans modification de leurs assertions** — leur
  suppression dans `exports.api.test.ts` relève de #438 ;
- **`triggerDownload`, testé directement** — ce qu'aucune des copies actuelles n'a : l'ancre reçoit
  `download = filename` et un `href` d'URL objet, `click()` est appelé, puis l'ancre est retirée et
  `revokeObjectURL` appelé **même si `click()` jette** (espions `URL.createObjectURL` /
  `revokeObjectURL`, patron `exports.api.test.ts:82-100`).

### Volet D — les libellés de l'écran

**10. Les clés i18n de l'écran**, dans les quatre catalogues
(`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`), en un bloc
`# --- Journal d'audit — écran (Story 25-1c-b1) ---` :

- `nav-audit-log` ;
- titre et sous-titre de page ;
- libellés des filtres — **dont la mention « jour(s) UTC » de la période** (AC 5), et les options
  « Tous » / « Toutes » — et du bouton de réinitialisation ;
- en-têtes de colonnes ;
- mention « clé API » ;
- bouton de détail ;
- états vide et erreur ;
- bouton d'export.

⚠️ **Les clés du vocabulaire ne sont PAS de cette story** : les clés `audit-log-*` de la 25-1c-a (en-têtes du CSV,
message de refus, types d'auteur, 28 types d'entité et les actions — **138** clés et **97** actions au 2026-09-26, 133 et 92 à la spécification : cinq stories mergées depuis en ont ajouté, et le nombre **dérivera encore**, d'où un décompte à refaire au développement et jamais figé ici) **existent déjà** et
ne se recréent ni ne s'appellent ici. Le **total** de cette story se recompte depuis la source à
l'implémentation et s'écrit avec sa ventilation.

⛔ **Préfixe partagé** : les clés de l'écran commencent par `audit-log-`, comme celles du backend. Ne
**jamais** ajouter `audit-log-` à `PREFIXES_A_COUVERTURE_CLOSE` (`i18n-keys.test.ts:497` au 2026-09-26) : les clés `audit-log-*`
servies par le backend y deviendraient des orphelines, alors qu'elles ont un lecteur, la route.

Les termes suivent `docs/i18n-glossaire.md` : partie A là où elle les atteste — notamment « journal
d'audit », **dont l'inscription en partie A est le travail de la 25-1c-b2** (règle 3) ; un terme
récurrent qu'elle n'atteste pas va en **partie B**, avec sa valeur proposée.

**11. Les gardes i18n du frontend, recomptées et ventilées.** L'écran n'appelle plus aucune clé à gabarit
ni aucune carte de libellés : les compteurs dynamiques **ne bougent pas**, et c'est à **vérifier**, pas à
supposer.

`src/lib/shared/i18n-keys.test.ts` :

| site | attendu |
|---|---|
| `MOTIFS_DYNAMIQUES` (`:343`), `CARDINALITES` (`:401-410`) | **inchangés** — aucun préfixe dynamique neuf |
| `ATTENDU.sitesGabarit` (`:319`), `SITES_GABARIT_ATTENDUS` (`:474-485`) | **inchangés** — 10 |
| `FAMILLES_RESOLUES['nav-']` (`:428-436`) | **+ `'audit-log'`** (AC 7) |
| `ATTENDU.sitesTotal`, `sitesNonResolus`, `relais` (`:316-318`) | **recomptés** — les appels littéraux de la page ajoutent des sites —, historique commenté prolongé |
| `PREFIXES_A_COUVERTURE_CLOSE` (`:497`) | **inchangé** (AC 10) |

`src/lib/shared/i18n-libelle-en-dur.test.ts` : `CANDIDATES_ATTENDUES` (`:144`) et la partition (`:661`)
**inchangés** — sauf si le code déclare une fonction ou une variable dont le nom finit par `Label`,
`Text` ou `Display` (`SUFFIXES`, `:72`), y compris un `$derived` (`:245`). ⚠️ `toSelectOptions` n'en est
pas une ; un `let actionLabel = $derived(…)` en serait une. **Dans ce cas, recompter et ventiler.**

⛔ **Si un compteur bouge sans que le code l'explique, c'est la forme du code qui diffère de la spec** — la
corriger, ou écrire l'écart : **jamais** ajuster un compteur pour le faire passer.

### Volet E — les tests

**12. Vitest** :

- `i18n.svelte` (`src/lib/shared/utils/i18n.svelte.test.ts`) : ⛔ **`i18nLocale()` rend la locale servie** —
  le helper `servir` (`:28`), qui ne sert aujourd'hui que des messages, est étendu à la locale ; `'de-CH'`
  servi ⇒ `i18nLocale()` rend `'de-CH'` ; avant tout chargement ⇒ `'fr-CH'` — ⛔ **lu sur un module FRAIS** (`vi.resetModules()` puis
  `await import('./i18n.svelte')`) : ce fichier ne remet jamais son module à zéro, et l'état d'un chargement
  précédent rendrait l'assertion vraie ou fausse **selon sa place dans le fichier** (exécuté en R3). ⚠️ **C'est le seul test qui
  exerce la vraie fonction** : les tests de page mockent le module entier ;
- `vocabulary-options` :
  - tri par libellé : l'entrée arrive **dans l'ordre des codes**, comme la route la rend
    (`invoice.validated` « Facture validée », puis `journal_entry.created` « Écriture créée ») ; la sortie
    range « Écriture créée » **avant** « Facture validée ». ⚠️ Une entrée déjà rangée par libellé laisserait
    passer l'absence de tri ;
  - ⛔ **le paramètre `locale` se prouve par le résultat, à condition de choisir une locale et des libellés
    qui trient autrement** : les quatre locales du projet rangent « Öffnung » avant « Zahlung », `sv-SE` fait
    l'inverse (Node 22, ICU 78). ⇒ `toSelectOptions` avec `'sv-SE'` et ces deux libellés rend « Zahlung »
    d'abord. *(Un espion `vi.spyOn(Intl, 'Collator')`, prescrit en R1, casse `.compare()` et a été retiré en
    R2 ; l'angle mort écrit alors est levé en R3.)* ;
  - ⛔ un code courant **absent** du vocabulaire est ajouté, libellé par son code ; un code présent n'est
    pas dupliqué ;
- `query-helpers` :
  - aller-retour URL ;
  - valeurs invalides ignorées ;
  - `entityType` et `action` hors vocabulaire conservés ;
  - ⛔ **un `entityId` sans `entityType` est ignoré à la lecture et omis à la sérialisation** (AC 4) ;
- `audit-log.api` :
  - un paramètre vide n'est pas envoyé ;
  - `getAuditLogVocabulary` appelle la route de vocabulaire ;
  - ⛔ **l'URL passée à `getBlob` porte les filtres** : `dateFrom`, `dateTo`, `entityType`,
    `entityId`, `action`, sans `offset` ni `limit`. Aucun test du dépôt n'observe aujourd'hui l'URL de
    `getBlob` ; patron `exports.api.test.ts:102-120`, qui intercepte `fetch` par `vi.stubGlobal` et lit
    l'URL appelée ;
  - le nom vient de `Content-Disposition` ;
- `download.ts` : AC 9 ;
- la page :
  - `load()` **redirige** un rôle Consultation (patron `routes/(app)/users/users-page.test.ts:26-53`) ;
  - API mockée (patron `contacts/contacts-page.test.ts:17-40`) : rendu des trois états, de « — » pour
    `entityId = 0`, et du dépliage du détail ;
  - ⛔ **la colonne Action affiche `actionLabel`, et non `action`** : l'entrée mockée porte des valeurs
    **différentes** pour les deux, faute de quoi l'assertion serait vraie par construction ; même chose
    pour `entityTypeLabel` ;
  - ⛔ **un échec du vocabulaire affiche l'état d'erreur** ;
  - ⛔ **la page passe réellement par `toSelectOptions`** : URL `?action=zz.legacy`, vocabulaire mocké sans
    ce code et rendu **dans l'ordre des codes**. Le `<select>` `audit-log-filter-action` vaut `zz.legacy` et
    contient une option de texte `zz.legacy` ; la première option après « Toutes » est la première **par
    libellé**, pas par code. Sans ce test, une page qui rendrait `vocabulary.actions` directement passerait
    tous les autres ;
  - ⛔ **la page transmet `i18nLocale()`**, le mock de ce module rendant une locale choisie :
    - à `'sv-SE'`, avec deux actions libellées « Öffnung » et « Zahlung », la liste « Action » propose
      « Zahlung » d'abord — une locale écrite en dur rougit ;
    - à `'de-CH'`, la date d'une entrée `2026-09-16T12:26:33.123Z` contient `16.09.2026` et **pas** `sept.` —
      `'fr-CH'` en dur (`16 sept. 2026`) et `navigator.language` (`Sep 16, 2026` sous jsdom) rougissent. Seule
      la **date** est vérifiée : l'heure dépend du fuseau de la machine de test ;
  - l'erreur de la **liste** laisse les filtres et « Réinitialiser » visibles (AC 5) ;
  - ⛔ **revenir à « Tous » vide l'identifiant d'entité** : l'appel suivant à `listAuditLog` ne porte
    plus `entityId` ;
  - ⛔ **le bouton d'export transmet les filtres affichés** : `exportAuditLogCsv` mocké, appelé avec la
    requête courante ;
  - un rejet `{ code: 'RESULT_TOO_LARGE', message: '…', status: 400 }` — le `status` est exigé par
    `isApiError` (`api-client.ts:19-28`) — affiche ce message dans
    `audit-log-export-error`, sans `notifyError`.

**13. E2E Playwright** — `frontend/tests/e2e/audit-log.spec.ts`, patron `journal-entries.spec.ts:1-40` :

- ⚠️ **Le seed E2E ne contient ni Comptable ni Consultation** : les deux utilisateurs se créent par
  l'API en Admin (`ctx.post('/api/v1/users', { data: { username, password, role } })`, patrons
  `company-contact-details.spec.ts:236-246` pour le Comptable et `reports.spec.ts:428` pour la
  Consultation), noms suffixés par un `uniqSuffix()` **redéfini localement** — aucun helper ne
  l'exporte, trois specs le recopient (`company-contact-details.spec.ts:51`) —, contexte libéré par `disposeContextSafe`.
- ⚠️ **Le groupe « Administration » est replié** : cliquer
  `[data-testid="nav-group-administration"] summary` **avant** de viser un lien qu'il contient
  (`sidebar-navigation.spec.ts:48`, `users.spec.ts:45`).
- ⚠️ **Les filtres se pilotent par valeur** : `getByTestId('audit-log-filter-entity-type').selectOption({
  value: 'contact' })` (AC 5).

Les scénarios :

1. le Comptable créé **se connecte** (`clearAuthStorage` puis `login(page, username, password)`, sur le patron
   de `company-contact-details.spec.ts:39-49`, appelé à `:249` — ⚠️ **pas** le `login` à un seul paramètre de
   `journal-entries.spec.ts:28` ou `reports.spec.ts:40`, qui connecterait l'Admin en silence) ; ⚠️
   `authedApiContext(page)` reprend **la session de la page** (`test-state.ts:174-191`) :
   sans cette connexion, tout le scénario tournerait sous l'Admin du seed, que la route autorise aussi, et
   rien ne rougirait. Il crée un contact par `authedApiContext`, ouvre l'écran **par le menu**
   (`nav-link-audit-log`) et voit la ligne **`data-action="contact.created"`** — **isolée par le filtre
   type `contact` et l'identifiant du contact créé**, jamais par sa position dans la liste (la suite
   partage la base). ⛔ La cellule `audit-log-row-action` de cette ligne est **égale au `label` de
   `contact.created`**, lu par `authedApiContext` sur `/api/v1/audit-log/vocabulary` : c'est la preuve que
   l'écran affiche la traduction, exacte et indépendante de la langue. *(« Non vide et différente du code »,
   écrit d'abord, laissait passer une colonne rendue avec `entityTypeLabel`.)* ;
2. l'URL survit au rechargement ;
3. un filtre sur une période **passée** (`2000-01-01` → `2000-01-02`) l'écarte ;
4. ⛔ la période qui la **garde** se calcule en **jour UTC** — `new Date().toISOString().slice(0, 10)` —
   et non en jour local. Le navigateur de test est à `Europe/Zurich` (`playwright.config.ts:59`) : entre
   00:00 et 02:00 locales, le jour local précède le jour UTC d'un cran, et un test écrit en jour local
   rougirait deux heures par nuit ;
5. le détail se déplie et contient le JSON ;
6. l'export déclenche un téléchargement dont `suggestedFilename()` commence par `kesh-journal-audit-`
   (nom fixé par la 25-1c-a, `kesh-journal-audit-{slug}-{AAAA-MM-JJ}.csv` ; patron
   `invoices_echeancier.spec.ts:186-191`) ;
7. ⛔ un utilisateur **Consultation** ne voit **pas** l'entrée de menu — `nav-link-audit-log` à `toHaveCount(0)`, **précédé** de
   `nav-link-settings` à `toHaveCount(1)` (entrée du même groupe, visible de tous les rôles,
   `+layout.svelte:126`), sans quoi un sondage antérieur au rendu de la barre latérale passerait à vide
   (patron `invoice-send-email.spec.ts:154-157`) ; et `/audit-log` le **redirige**. Le Consultation se
   crée et se connecte sur le patron `reports.spec.ts:421-439`. ⚠️ **La redirection n'a AUCUN précédent E2E dans le dépôt** : ce bloc
   n'ouvre aucune route réservée, et aucune spec ne vérifie qu'un rôle insuffisant est renvoyé. Elle
   s'écrit donc sans patron : `page.goto('/audit-log')` puis `expect(page).toHaveURL('/')`, qui porte **à
   lui seul** la preuve. Une page qui ne se charge pas garde l'URL `/audit-log` ; et rien d'autre que la
   garde ne mène un Consultation sur `/` — `(app)/+layout.ts` ne redirige que vers `/login` et
   `/onboarding`, `api-client.ts` que vers `/login` et `/setup`, et un 403 ne redirige nulle part. Un
   complément : l'accueil est réellement rendu, `homepage-card-open-invoices` visible (patron
   anti-vacuité `homepage-reminders.spec.ts:83-84`).

   ⚠️ **Où se trouve la garde n'est PAS l'affaire de l'E2E.** Qu'elle jette dans `load()`, avant tout
   montage, est prouvé par le **test unitaire** de l'AC 12 (`load()` redirige un Consultation), qui
   l'observe directement. *(Un compteur de requêtes vers `/api/v1/audit-log` a été prescrit puis retiré
   en passe 6 : son résultat dépendait de l'ordre de déclaration des effets du composant, et trois
   corrections successives n'ont pas su le rendre discriminant.)*

   ⛔ **Ne PAS asserter `audit-log-table` à `toHaveCount(0)`** : une fois l'URL `/` atteinte, elle est
   vraie par construction, et elle ne distingue même pas la mutation « garde retirée » — la route
   répondrait 403, la page afficherait son état d'erreur, et le tableau resterait absent.

**14. Épreuve par mutation, résultats OBSERVÉS consignés** :

| mutation | site | test attendu rouge |
|---|---|---|
| garde retirée | `+page.ts` | 12, redirection Consultation ; 13, scénario 7 — `toHaveURL('/')` |
| colonne Action rendue avec `action` au lieu de `actionLabel` | `+page.svelte` | 12, page — libellé ; 13, scénario 1 — cellule traduite |
| tri des options par code au lieu du libellé | `vocabulary-options.ts` | 12, `vocabulary-options` — tri |
| code courant hors vocabulaire non ajouté aux options | `vocabulary-options.ts` | 12, `vocabulary-options` — code conservé |
| échec du vocabulaire avalé (filtres vides, pas d'erreur) | `+page.svelte` | 12, page — état d'erreur |
| page qui rend `vocabulary.actions` sans `toSelectOptions` | `+page.svelte` | 12, page — options de la liste « Action » |
| `i18nLocale()` qui rend toujours `'fr-CH'` | `i18n.svelte.ts` | 12, `i18n.svelte` — la locale servie (le test de page, qui mocke le module, ne la voit pas) |
| valeur initiale d'`i18nLocale()` vide au lieu de `'fr-CH'` | `i18n.svelte.ts` | 12, `i18n.svelte` — module frais |
| page qui passe `'fr-CH'` en dur à `toSelectOptions` | `+page.svelte` | 12, page — tri sous `'sv-SE'` |
| colonne Date formatée avec `navigator.language` ou `'fr-CH'` en dur | `+page.svelte` | 12, page — date sous `'de-CH'` |
| `nav-audit-log` retirée des quatre catalogues | `messages.ftl` | 11, « toute clé demandée existe » — ⚠️ ne rougit que si `'audit-log'` est bien dans `FAMILLES_RESOLUES` : c'est ce qui rend cet ajout observable, puisque son **seul** oubli ne fait rougir aucune garde |
| entrée de menu passée de `comptableOnly` à `items` | `+layout.svelte` | 13, Consultation voit le menu |
| `entityId` conservé au retour à « Tous » | `+page.svelte`, gestionnaire de la liste des types | 12, page — `entityId` vidé |
| `entityId` sans type relu depuis l'URL | `query-helpers.ts`, lecture | 12, `query-helpers` |
| filtres non sérialisés dans l'URL d'export | `audit-log.api.ts`, `exportAuditLogCsv` | 12, URL de `getBlob` |
| export appelé avec `{}` au lieu des filtres affichés | `+page.svelte`, gestionnaire du bouton d'export | 12, page — export |
| `revokeObjectURL` sorti du `finally` | `download.ts` | 12, `triggerDownload` qui jette |

⛔ **Un test qui ne compile pas ne rougit pas : il se tait.**

## Tasks / Subtasks

- [ ] **T0 — Rebase** sur `main` après le merge de la 25-1c-a (ne pas implémenter avant) ; vérifier que
      les **trois** routes répondent (`curl` authentifié) sur le backend local, et que les libellés du
      vocabulaire sortent dans la langue de `KESH_LANG`.
- [ ] **T1 — Module de téléchargement** (AC 9) et ses tests ; les trois commentaires de l'AC 9 (`reports.api.ts` intact).
- [ ] **T2 — Locale conservée (`i18nLocale()`), types, API, options des listes, paramètres d'URL** (AC 1-4)
      et leurs tests (AC 12).
- [ ] **T3 — Page, garde, menu, sélecteurs** (AC 5-8) et tests de page (AC 12).
- [ ] **T4 — i18n de l'écran, quatre locales** (AC 10) — ⚠️ recompte depuis la source.
- [ ] **T5 — Gardes i18n** (AC 11), compteurs **recomptés** et ventilés.
- [ ] **T6 — E2E** (AC 13). ⛔ *Une tâche qui décrit un test est une promesse ; la cocher sans l'avoir
      écrit la transforme en mensonge.*
- [ ] **T7 — Mutations** (AC 14), résultats observés.
- [ ] **T8 — Gates**, **après** l'implémentation de la 25-1c-b2 sur la même branche :
  - frontend complet : `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` ;
  - backend : `cargo test -p kesh-i18n` (parité des catalogues), puis `scripts/test-fast.sh` ;
  - **E2E complète**, `kesh_e2e` reconstruite, **build frontend postérieur au dernier patch**.

## Dev Notes

### Ce que la story touche

| zone | fichiers |
|---|---|
| feature neuve | `frontend/src/lib/features/audit-log/` — `audit-log.types.ts`, `audit-log.api.ts`, `vocabulary-options.ts`, `query-helpers.ts`, tests |
| écran | `frontend/src/routes/(app)/audit-log/+page.svelte`, `+page.ts`, test de page |
| menu | `frontend/src/routes/(app)/+layout.svelte` |
| téléchargement | `frontend/src/lib/shared/utils/download.ts` + test (neufs) ; **commentaires seulement** dans `reports.api.ts`, `exports.api.ts`, `imported-supplier-invoices.api.ts`, `admin-backup.api.ts` |
| store i18n | `frontend/src/lib/shared/utils/i18n.svelte.ts` — `i18nLocale()` (AC 3) ; les mocks de ce module dans les tests de page qui en ont besoin |
| gardes | `frontend/src/lib/shared/i18n-keys.test.ts` (`FAMILLES_RESOLUES`, compteurs de sites) |
| i18n | `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (clés de l'écran) ; `docs/i18n-glossaire.md` partie B si un terme l'exige |
| E2E | `frontend/tests/e2e/audit-log.spec.ts` |

### Faits établis à la spécification, et non supposés

- **Langue** : Kesh n'a pas de langue par utilisateur ; l'interface suit `KESH_LANG`, servie par
  `/api/v1/i18n/messages` (`routes/i18n.rs:21-22`), et la 25-1c-a traduit le vocabulaire dans la même.
- **Listes** : le composant `Select` du dépôt est bits-ui (`src/lib/components/ui/select/`) ; il pose
  `data-value` sur ses options, mais les E2E existants le pilotent par libellé (`accounts.spec.ts:136`) ou
  par position — `journal-entries.spec.ts:112` vise l'autocomplétion de compte, pas un `Select`. Des
  `<select>` natifs existent dans **treize** pages de `routes/` ; le patron « natif + `data-testid` » est
  `invoices/+page.svelte:314-318`, piloté par `invoices.spec.ts:167` (les filtres de `contacts/+page.svelte`
  n'ont pas de `data-testid`).
- **Locale** : `i18n.svelte.ts:25-35` jette `data.locale` ; aucun `Intl.DateTimeFormat` ni `Intl.Collator`
  dans le frontend ; `app.html:2` porte `lang="fr"` en dur. Un store qui conserve la locale ne change aucun
  compteur de garde (sonde de la revalidation R1).
- **Erreurs d'API** : `parseErrorResponse` (`api-client.ts:208-240`) recopie `error.code` et
  `error.message` du corps JSON ; le message d'un 400 de la route arrive donc **déjà traduit** par le
  serveur. **Aucune page du frontend ne traite encore `RESULT_TOO_LARGE`** (grep vide).
- **Menu** : `navGroups` (`+layout.svelte:62-163`), listes `comptableOnly` (`:130-136`) et `adminOnly`
  (`:138-161`), `isComptablePlus` (`:46-48`), testid `nav-link-<href slugifié>` (`:258-261`).
- **Téléchargement** : aucun test du dépôt n'observe l'URL passée à `getBlob`, ni `triggerDownload`
  directement. Inventaire des copies : [#438].
- **E2E** : `workers: 1`, `locale: 'fr-CH'`, `timezoneId: 'Europe/Zurich'` (`playwright.config.ts:53,58-59`)
  — la suite ne tourne qu'en français, d'où l'exigence de `data-testid` et des attributs de code.
- **Pas de composant partagé** de pagination ni de datepicker : `Input type="date"`, pagination écrite
  dans la page.

### Ce qui ne bouge PAS

- Le backend, hors catalogues FTL : les routes et le vocabulaire sont la 25-1c-a.
- Le **nom** d'une clé API (la route ne renvoie que `actorApiKeyId`).
- Le code des sept copies de téléchargement ([#438]).
- Les manuels, le README, le glossaire partie A et les clés existantes au vocabulaire divergent : **25-1c-b2**.

### Intelligence des stories précédentes

- **25-1c-a** (validation en 5 passes) : *une forme qui casse à chaque correction se retire, elle ne se
  rapièce pas* ; *un arbitrage résumé est un arbitrage réécrit* — citer, ne pas paraphraser.
- **25-1c-zero** : une assertion **vraie par construction** a traversé quatre passes ; pour chaque
  assertion, se demander ce qui la rendrait fausse — d'où les valeurs **différentes** de `action` et
  `actionLabel` dans les données de test (AC 12).
- **Cette story, passes 3 à 7** : trois corrections successives d'une même assertion E2E n'ont pas su la
  rendre discriminante ; elle a été retirée au profit d'un test unitaire qui observait la chose.
- **23-3b / 23-1** : les clés de menu échappent aux gardes si `FAMILLES_RESOLUES` n'est pas tenu.
- **Gate** : un build frontend **en retard d'un périmètre** a déjà faussé un gate E2E (24-4c, 25-1b).

### Hors périmètre

- Recherche plein texte dans `details`, nom des clés API.
- Conversion des jours locaux en bornes UTC : le contrat de la route prend des dates, il est arbitré.
- Toute traduction du vocabulaire côté frontend : la 25-1c-a en est la source unique.
- **[#438]** (regroupement du téléchargement), **[#386]** (export de souveraineté, 25-5), **[#431]**,
  **[#434]**, **[#435]**.

### References

- `25-1c-a-journal-audit-route.md` — **le contrat des routes** (AC 5-17)
- `25-1c-b-journal-audit-ecran.md` — la fiche parente, passes 1 et 2
- `epic-25-vague1-suite.md` § *Arbitrages du 2026-09-15* et § *fin de soirée*
- `frontend/src/lib/features/journal-entries/query-helpers.ts`, `routes/(app)/journal-entries/+page.svelte` — **patron de la liste filtrée**
- `frontend/src/routes/(app)/invoices/+page.svelte:314-318` et `tests/e2e/invoices.spec.ts:167` — **patron du `<select>` natif de filtre, avec `data-testid`**
- `frontend/src/lib/shared/utils/i18n.svelte.ts` — **store i18n, où conserver la locale**
- `frontend/src/routes/(app)/supplier-invoices/import/+page.ts` — **patron de la garde Comptable+**
- `frontend/src/lib/features/export/exports.api.ts:52-110` et `exports.api.test.ts` — **source du module de téléchargement et de ses tests**
- `frontend/src/lib/shared/i18n-keys.test.ts:206-216,234,292-301,311-327,359-376,388` · `i18n-libelle-en-dur.test.ts:72,115,245,632`
- `CLAUDE.md` § *Test Locally First*, § *Propagation post-patch*

[#378]: https://github.com/guycorbaz/kesh/issues/378
[#438]: https://github.com/guycorbaz/kesh/issues/438

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-09-15** — **Fiche créée par le split de la 25-1c-b** (arbitrage du Project Lead), après la passe 2
  de validation de la parente (Opus : 6 MEDIUM, 7 LOW, tous vérifiés au sol et retenus). Les AC de
  l'écran, du menu, des gardes et des tests viennent de la parente ; les treize findings y sont appliqués
  **ici** (M1-M4, L1, L3-L7) ou dans la **25-1c-b2** (M5, M6) :
  - **M1** — le 29ᵉ type `audit_log`, créé par l'export de la 25-1c-a, manquait à la carte → ajouté
    (AC 3), recompte en T0 ;
  - **M2** — filtre en jours UTC, tableau en heure locale → libellé « jours UTC » (AC 5), E2E calculé en
    jour UTC (AC 13, scénario 4) ;
  - **M3** — `entityId` résiduel au retour à « Tous », et relu sans type depuis l'URL → vidé et ignoré
    (AC 4, 5), tests et mutations ;
  - **M4** — rien ne prouvait que l'export transmet les filtres → test de l'URL de `getBlob` et test de
    page, sites de mutation nommés (AC 12, 14) ;
  - **L1** — renvoi « module partagé de l'AC 11 » faux → AC 9 ;
  - **L2** — sept copies de téléchargement et non quatre → **le volet d'extraction sort** (arbitrage) :
    module neuf sans migration (AC 9), issue de dette **#438**, quatre commentaires mis à jour ;
  - **L3** — conséquences exactes sur les gardes (`sitesGabarit` 11, `SITES_GABARIT_ATTENDUS`,
    `CANDIDATES_ATTENDUES` 44, `conforme` 37) et trois textes devenus faux → tableau de l'AC 11 ;
  - **L4** — message de `RESULT_TOO_LARGE` → **message du serveur réutilisé**, aucune clé neuve (AC 5) ;
  - **L5** — E2E : groupe « Administration » replié, seed sans Comptable ni Consultation → AC 13 ;
  - **L6** — les clés de la 25-1c-a sont 11 + 1 = 12 → AC 10 ;
  - **L7** — renvoi de commentaire `:311-314` ; aucun test direct de `triggerDownload` → ajouté (AC 9) ;
    le message d'erreur arrive traduit par le serveur → Dev Notes.

### Passe 3 de `bmad-create-story validate` — une lentille Sonnet, contexte frais

Prompt versionné : `25-1c-b1-validate-prompt-p3.md`. Base : `git diff 0b029e24 f764004d`.

**Rendu : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW — tous vérifiés au sol et retenus. Après reclassement :
1 MEDIUM, 3 LOW.**

- **M1** — la dépendance à la 25-1c-a citait « AC 5-14 », alors que l'écran consomme aussi la clé
  `audit-log-export-error-too-large` de son AC 15 → « AC 5-15 ».
  *Reclassé LOW à la vérification* : les References disaient déjà 5-15.
- **M2** — l'argument `limit` du refus `RESULT_TOO_LARGE` était attribué à l'AC 15 de la 25-1c-a ; il est
  défini par son **AC 11** (« Plafond », `25-1c-a…:203-205`) → « AC 11 », au site de l'AC 5 de cette
  fiche. L'autre mention
  (`AC 10`, « 11 + 1, 25-1c-a AC 15 ») porte sur les **clés**, et reste juste.
- **L3** — le scénario E2E 7 citait `reports.spec.ts:421-446` pour la redirection d'un Consultation ; ce
  bloc n'ouvre aucune route réservée. **Aucune spec du dépôt ne teste une telle redirection** (grep
  inventaire corrigé en passe 4 : les `goto` vers des routes gardées se font tous en
  Admin, et les trois specs qui créent un Consultation — `reports.spec.ts:428`,
  `homepage-reminders.spec.ts:66`, `invoice-send-email.spec.ts:144` — n'ouvrent que des routes ouvertes à
  tous) → patron restreint à la création et connexion (`:421-439`),
  assertion écrite, et une seconde assertion exigée pour que la redirection ne soit pas vraie par
  construction.
- **L4** — `uniqSuffix()` n'est exporté d'aucun helper (trois copies locales) → précisé.

**Axe déclaré non exercé par la lentille, repris par l'orchestrateur** : la partition de
`i18n-libelle-en-dur.test.ts`. `candidatesDe` (la fonction réelle, copiée dans le scratchpad) a été
**exécutée** sur un `entity-type-label.ts` synthétique conforme à l'AC 3 et sur l'`error-label.ts` réel :
une candidate chacun, `analysee: true`, `retours: []` ⇒ **conforme**, comme son modèle. Les valeurs 44 et
37 de l'AC 11 sont confirmées par exécution. La lentille avait, elle, exécuté le moissonneur de gabarits
(1 site, préfixe `audit-log-entity-` ⇒ `sitesGabarit` 11).

Remédiation : texte de spec uniquement, aucune ligne de code.

### Passe 4 CIBLÉE de `bmad-create-story validate` — une lentille Opus, contexte frais

Prompt versionné : `25-1c-b1-validate-prompt-p4.md`. Base : `git diff f764004d 637c6508`.

**Rendu : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW — tous sur le scénario E2E 7 écrit par la passe 3, tous
vérifiés au sol et retenus.**

- **M1** — l'assertion `audit-log-table` à `toHaveCount(0)`, ajoutée « pour que la redirection ne soit pas
  vraie par construction », **l'était elle-même**. Une page qui ne charge pas garde l'URL `/audit-log`, donc
  `toHaveURL('/')` suffit, et aucun autre chemin ne mène un Consultation sur `/` (vérifié :
  `(app)/+layout.ts:11,40,43`). → retirée, remplacée par deux compléments qui discriminent : l'accueil
  rendu, et aucune requête vers la route.
- **L2** — le volet menu (`toHaveCount(0)`) pouvait passer avant le rendu de la barre latérale, alors que
  le dépôt a deux patrons anti-vacuité → ancrage `nav-link-settings` à `toHaveCount(1)`.
- **L3** — la preuve de « aucun précédent » (un grep sur le mot `redirect`) n'établissait pas une absence
  → remplacée par l'inventaire des `goto` et des specs qui créent un Consultation.

⛔ *Une assertion ajoutée pour empêcher un test vrai par construction peut l'être à son tour* — la leçon
de la 25-1c-zero, reproduite une passe plus tard sur une spec.

La passe confirme par ailleurs les références neuves de la passe 3 (AC 11, AC 15, lignes de
`reports.spec.ts` et `company-contact-details.spec.ts`, `uniqSuffix`) et la cohérence des décomptes.
Remédiation : texte de spec uniquement.

### Passe 5 CIBLÉE de `bmad-create-story validate` — une lentille Sonnet, contexte frais

Prompt versionné : `25-1c-b1-validate-prompt-p5.md`. Base : `git diff 637c6508` (remédiation de la passe 4).

**Rendu : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 0 LOW — vérifié au sol et retenu.**

- **M1** — le complément « aucune requête » ne disait pas que le listener doit être attaché **avant** le
  `goto`. Posé après `toHaveURL('/')` — l'ordre dans lequel le texte présentait les choses —, il passe à
  vide contre une garde déplacée après le montage : la page monte, appelle la route, puis redirige.
  Vérifié : le patron cité attache le listener (`homepage-reminders.spec.ts:77-80`) avant son `goto`
  (`:82`), et c'est la seule occurrence de `page.on('request'` de la suite. → ordre exigé ; **la mutation
  qui prouve ce complément est ajoutée à l'AC 14**, pour que l'implémentation montre qu'il rougit.

⛔ **Deuxième passe de suite où l'assertion ajoutée pour garantir une autre était elle-même muette**
(passe 4 : le tableau absent, écrit en passe 3 ; passe 5 : le compteur de requêtes, écrit en passe 4).
*(Écrit « troisième » à la rédaction, en comptant deux fois la même assertion — corrigé en passe 6. Le
prompt `p6` reprend l'erreur et reste tel qu'il a été envoyé.)* Le geste qui
ferme le motif n'est pas une meilleure phrase : c'est **une mutation par assertion ajoutée**, observée
rouge à l'implémentation.

La passe confirme par ailleurs : `homepage-card-open-invoices` n'est pas conditionnée au rôle ;
`nav-link-settings` est unique et reste dans le DOM d'un `<details>` replié ; `nav-link-audit-log` est
retiré par un `{#if}` réel ; le préchargement au survol (`app.html:10`) est sans effet, le lien étant
absent pour ce rôle.

Remédiation : texte de spec uniquement.

### Passe 6 CIBLÉE de `bmad-create-story validate` — une lentille Opus, contexte frais

Prompt versionné : `25-1c-b1-validate-prompt-p6.md`. Base : `git diff c5d14f32` (remédiation de la passe 5).

**Rendu : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW — vérifiés et retenus.**

- **M1** — la mutation ajoutée en passe 5 (« garde déplacée dans le composant ») ne rougit l'assertion
  qu'elle devait prouver que sous **une seule** des trois formes qu'on peut lui donner. Le patron de page
  imposé par l'AC 5 déclenche au montage un `onMount` qui charge, **puis** un `$effect` qui fait
  `goto(url, { replaceState: true })` ; SvelteKit abandonne une navigation dès qu'une plus récente la
  suit (`client.js:1690-1692`). Donc :
  - une garde écrite en tête de composant voit son `goto('/')` annulé, et c'est `toHaveURL('/')` qui
    rougit — le compteur n'est jamais lu ;
  - seule une garde déclarée **après** le `$effect` fait rougir le compteur.

  « Scénario 7 rouge » aurait été consigné sans que l'assertion visée ait tourné.
- **L1** — « une page montée puis renvoyée ailleurs en ferait une [requête] » était trop général.
- **L2** — « troisième passe de suite » comptait deux fois la même assertion → « deuxième ».

**Décision de l'orchestrateur : le complément et sa mutation sont RETIRÉS, pas rapiécés.** C'est la
troisième correction sur le même objet (écrit en passe 4, corrigé en passe 5, mis en défaut en passe 6),
et la leçon de la 25-1c-a s'applique à la lettre : *une forme qui casse à chaque correction se retire*.
Ce qu'il prétendait prouver — la garde jette dans `load()`, avant montage — est **déjà observé
directement** par le test unitaire de l'AC 12, sans dépendre de l'ordre des effets d'un composant.
L'E2E garde `toHaveURL('/')` et l'accueil rendu, qui rougissent tous deux quand la garde est retirée.
L1 disparaît avec le complément.

Remédiation : **retrait** de texte de spec ; aucune assertion ajoutée.

### Passe 7 CIBLÉE de `bmad-create-story validate` — une lentille Sonnet, contexte frais — **BOUCLE CLOSE**

Prompt versionné : `25-1c-b1-validate-prompt-p7.md`. Base : `git diff 32af4148` (remédiation de la passe 6).

**Rendu : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW.** Les quatre axes sont déclarés exercés, avec leurs
preuves :
- le patron unitaire `users-page.test.ts:26-53` exige une redirection 302 vers `/`, et toute mutation qui
  sort la garde de `load()` le fait rougir ;
- la ligne fusionnée de l'AC 14 annonce des tests rouges réels ;
- le retrait est complet hors Change Log ;
- le Change Log de la passe 6 est fidèle au diff.

**Vérification de l'orchestrateur** (un zéro se vérifie comme un finding) : le point porteur, relu à la
source. Dans `users-page.test.ts:26-53`, `load()` est appelé, et `expect.unreachable` jette s'il ne jette
pas lui-même. Cette erreur est rattrapée, puis l'assertion `status === 302` rougit. L'assertion est bien
discriminante.

**Critère de clôture** : 0 finding au-dessus de LOW, et la dernière remédiation (passe 6) est un **retrait**
de texte de spec, sans assertion ajoutée ni ligne de code.

**Trend de la boucle** (la parente pour les passes 1-2, cette fiche ensuite) :

| passe | modèle | rendu (après reclassement) |
|---|---|---|
| 2026-09-26 | revalidation R6 ciblée | **Une lentille Haiku 4.5**, contexte frais, braquée sur la seule remédiation R5 (`git diff bc4fee4b 6358639e`), prompt `25-1c-b1-validate-prompt-r6-ciblee.md`, axes déclarés. **0 finding** : 28 types, 97 actions, 138 clés recomptés ; chaque numéro de ligne vérifié par `grep` ; « 133 » et « 92 » ne subsistent que dans l'historique. La remédiation ne touchait que la fiche ⇒ **boucle close**, fiche revalidée contre `main` à `0e4c2682`. |
| 2026-09-26 | revalidation R5 « dérive » | Reprise sur `main` à `0e4c2682` après dix jours sur une branche locale jamais poussée. **Une lentille Sonnet**, prompt `25-1c-b-validate-prompt-r5-derive.md`, axes déclarés (non exercés : manuels — délégués à la b2 —, contenu détaillé des specs E2E non modifiées depuis). **Contrat de la route mergée (PR #439) confirmé champ par champ** ; rien de ce que la fiche prescrit n'a été livré entre-temps ; sept copies de téléchargement toujours sept. **1 MEDIUM** : « 133 clés / 92 actions » périmé — **138 / 97** au 2026-09-26 (`account.retyped`, `invoice.unvalidated`, `invoice.settlement_cancelled`, `supplier_invoice.settlement_cancelled`, `reconciliation.cancelled`) ; la fiche ne fige plus le nombre, à recompter au développement. **3 LOW** : lignes de `i18n-keys.test.ts` (+109 à +120) et de `i18n-libelle-en-dur.test.ts` rafraîchies ; décalages d'une dizaine de lignes dans les patrons cités (`invoices/+page.svelte`, `invoices_echeancier.spec.ts`, commentaires de l'AC 9), contenu intact — non repris, T0 relit par contenu. #386 citée hors périmètre : désormais fermée, sans effet. |
| 1 | Sonnet + Haiku | 2 M, 2 L |
| 2 | Opus | 6 M, 7 L → **split** |
| 3 | Sonnet | 1 M, 3 L |
| 4 | Opus, ciblée | 1 M, 2 L |
| 5 | Sonnet, ciblée | 1 M |
| 6 | Opus, ciblée | 1 M, 2 L → **retrait** |
| 7 | Sonnet, ciblée | **0** |

Aucun défaut de conception d'origine après le split. **Chacun des MEDIUM des passes 3 à 6 a été introduit
par la remédiation précédente**, et les trois derniers portaient sur **le même objet** : l'assertion qui
devait garantir la redirection d'un Consultation. La boucle ne s'est pas close par une meilleure phrase,
mais par le **retrait** de cette assertion au profit d'un test qui observait déjà la chose directement.

---

## Réouverture du 2026-09-15 (soir) — le contrat de la 25-1c-a a changé

Trois arbitrages du Project Lead, rendus après la clôture de la boucle (`epic-25-vague1-suite.md`
§ *fin de soirée*), réécrivent le contrat que l'écran consomme :

| arbitrage | ce qui change ici |
|---|---|
| filtre strict — *« le cas réel n'existe pas »* | plus de `companyId` (AC 1), plus de colonne « Société » ni de mention « société indéterminée » (AC 5, 10, 12) |
| l'export n'écrit pas d'entrée — *« non »* | le type `audit_log` n'existe plus ; l'ancien M1 de la parente (29ᵉ type) devient sans objet |
| actions et types traduits, écran et CSV, dans la langue de l'interface — *« ok »*, *« non, maintenant »* | **l'écran ne traduit plus rien lui-même** : il affiche `actionLabel` et `entityTypeLabel` (AC 5), charge le **vocabulaire** traduit (AC 2), et ses filtres type et action deviennent des **listes** triées par libellé (AC 3, 5) ; la carte `entity-type-label.ts`, ses 29 clés et ses conséquences sur les gardes **disparaissent** (AC 3, 10, 11, 12, 14) |

**Choix de spécification qui en découlent, et qui sont à contester en revalidation** :

- **`<select>` natifs** pour les deux listes, et non le `Select` de bits-ui : le pilotage E2E par valeur
  évite un sélecteur par libellé (AC 5) ;
- **attributs `data-action` et `data-entity-type`** sur chaque ligne, pour identifier une ligne par son
  code alors qu'elle affiche un texte traduit (AC 5, 13) ;
- **un code d'URL hors vocabulaire devient une option** libellée par son code (AC 3) ;
- **l'échec du vocabulaire est un état d'erreur** de la page (AC 5).

**Ce qui tombe avec la réécriture** : les valeurs de gardes établies en passes 2 et 3 (`sitesGabarit` 11,
`CANDIDATES_ATTENDUES` 44, `conforme` 37), qui ne valaient que pour la carte retirée — **les gardes
restent désormais à leurs valeurs** (AC 11). Les AC cités dans le Change Log ci-dessus sont ceux de la
version validée, et ne sont pas réécrits.

**Revalidation requise** : passe complète sur les volets A, B, D et E, puis ciblée.

### Revalidation R1 — une lentille Opus, contexte frais

Prompt versionné : `25-1c-b1-validate-prompt-r1.md`. Base : `git diff 5760bab7 90a427bf`. Sondes : `Intl`
sous Node 22 (ICU 78), copie jetable des gardes i18n avec une page synthétique conforme, regex de la garde
E2E, source de bits-ui et de Svelte.

**Rendu : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 8 LOW — tous vérifiés au sol et retenus.**

- **M1 — la « locale de l'interface » n'existe pas dans le frontend** : `i18n.svelte.ts` jette
  `data.locale`. Vérifié. → `i18nLocale()` (AC 3), format de date sur elle (AC 5), mock, mutation ; la
  phrase correspondante de la 25-1c-a (AC 17) est corrigée dans sa fiche.
- **M2 — aucun test ne vérifiait que la page passe par `toSelectOptions`** : une page rendant le vocabulaire
  brut passait tous les tests. → test de page (code d'URL hors vocabulaire, ordre par libellé), mutation.
- **L1** — le test de tri laissait passer l'absence de tri et une locale ignorée → entrée dans l'ordre des
  codes, espion `Intl.Collator`.
- **L2** — oublier `'audit-log'` dans `FAMILLES_RESOLUES` ne fait rougir aucune garde (exécuté) → mutation
  « `nav-audit-log` retirée des catalogues », qui rend l'ajout observable.
- **L3** — les faits qui justifiaient les `<select>` natifs étaient inexacts : bits-ui est pilotable par
  valeur, la garde E2E laisse passer `selectOption({ label })`, les natifs sont dans treize pages et le vrai
  patron avec `data-testid` est `invoices`. Vérifié. → **choix maintenu**, justification réécrite, `<label
  for>` exigé, interdiction portée par la spec et non attribuée à la garde.
- **L4** — le rejet mocké sans `status` n'aurait pas passé `isApiError` → `status: 400`.
- **L5** — « non vide et différente du code » laissait passer une colonne rendue avec le type → égalité au
  `label` lu sur `/vocabulary`.
- **L6** — deux des « quatre commentaires » ne planifiaient pas l'extraction → deux à réécrire, un à
  rediriger, `reports.api.ts` intact.
- **L7** — des valeurs valides en forme mènent au 400, et l'erreur de liste n'était pas cadrée → 400
  assumé, filtres et réinitialisation visibles.
- **L8** — le tableau d'en-tête de la 25-1c-a ne disait pas la b1 et la b2 rouvertes → corrigé dans sa fiche.

**Reçu de la revalidation R1 de la 25-1c-a** : les comptes du vocabulaire passent de 123 à **133** clés et
de 82 à **92** actions (AC 10).

Remédiation : texte de spec uniquement. **Revalidation R2 requise** (MEDIUM).

### Revalidation R2 CIBLÉE — une lentille Sonnet, contexte frais

Prompt versionné : `25-1c-b1-validate-prompt-r2.md`. Base : `git diff 2efbc76c 6cda8b89`. Sondes : copies
d'`i18n.svelte.ts` correcte et mutée, composant Svelte 5 à `<select>` natif sous jsdom, espion `Intl.Collator`,
tous exécutés sous vitest 4.1.3.

**Rendu : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 0 LOW — vérifiés au sol et retenus.** ⛔ **Deux des trois sont le motif
de cette fiche** : une preuve ajoutée par la remédiation qui ne prouve rien.

- **M1** — la mutation « `i18nLocale()` rend toujours `'fr-CH'` » était attribuée à deux tests qui **ne voient
  pas la vraie fonction** : `toSelectOptions` reçoit la locale en paramètre, et le test de page mocke le module
  entier. Exécuté : le test de page passe à l'identique sur la source correcte et sur la mutée. → **test direct**
  dans `i18n.svelte.test.ts`, `servir` étendu à la locale, ligne de mutation réalignée.
- **M2** — l'espion `vi.spyOn(Intl, 'Collator')` prescrit **casse `.compare()`** sous vitest, même sur une
  implémentation correcte (exécuté). → **retiré, pas rapiécé** : la locale ne se prouve pas par le tri entre les
  quatre langues du projet, et le contournement (`Reflect.construct`) ajouterait une assertion fragile à une
  fiche où trois ont déjà dû être retirées. Angle mort écrit.
- **M3** — le scénario E2E 1 ne connectait pas le Comptable : `authedApiContext` reprend la session de la page
  (vérifié, `test-state.ts:174-186`), et tout le scénario aurait tourné sous l'Admin, autorisé lui aussi. →
  connexion exigée.

La passe confirme par ailleurs, par exécution : le test de page « la page passe par `toSelectOptions` » rougit
sous sa mutation et reste vert sur une implémentation correcte ; aucun des 23 mocks existants du module i18n
n'est cassé par l'ajout d'`i18nLocale` ; les faits réécrits en R1 (lignes, « treize pages ») sont exacts.

Remédiation : texte de spec uniquement. **Revalidation R3 ciblée requise** (MEDIUM).

### Revalidation R3 CIBLÉE — une lentille Opus, contexte frais

Prompt versionné : `25-1c-b1-validate-prompt-r3.md`. Base : `git diff 0bbefc51 e9b15645`. Sondes exécutées
sous vitest 4.1.3 : copies d'`i18n.svelte.ts` et de son test, trois emplacements du test, deux mutations ;
page-sonde pour la date et le tri.

**Rendu : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW, et 1 LOW hors périmètre — vérifiés et retenus.**

- **M1** — l'assertion « avant tout chargement ⇒ `'fr-CH'` » ne discriminait que **placée en tête du
  fichier** : le module garde son état d'un test à l'autre. En fin de fichier, elle restait verte sous la
  mutation « valeur initiale vide », ou rougissait à tort. → **module frais** (`vi.resetModules()`), mutation
  ajoutée.
- **M2** — l'angle mort écrit en R2 était **trop étroit et évitable** : la colonne Date n'était plus observée
  par aucun test, et une locale discriminante (`sv-SE`, « Öffnung » / « Zahlung ») prouve le tri sans espion.
  Vérifié par l'orchestrateur sous Node 22 (`fr-CH` → « Öffnung » d'abord, `sv-SE` → « Zahlung » d'abord ;
  `de-CH` en `dateStyle: 'medium'` → `16.09.2026, 14:26`). → deux tests de page, deux mutations, **angle mort
  levé**.
- **L1** — « comme au scénario 7 » renvoyait à un `login` à un seul paramètre → patron
  `company-contact-details.spec.ts:39-49,249`, vérifié.
- **L2** — `test-state.ts:174-186` → `:174-191`.
- **L3** — les bases de la R2 (`2efbc76c`, `6cda8b89`) ne sont plus ancêtres de la branche après rebase ; leurs
  équivalents `dd3cedab`, `5dfbafc2` donnent le même diff. Le prompt `r2` reste tel qu'il a été envoyé.
- **L4** — « deux des trois » MEDIUM de la R2 relevaient du motif ; le troisième aussi (un scénario qui tourne
  sous l'Admin sans que rien ne rougisse) → c'étaient **les trois**.
- **Hors périmètre** — les exemples de date de l'AC 5 n'étaient justes qu'avec `dateStyle: 'medium'`, option que
  la fiche ne prescrivait pas → option prescrite, exemples mesurés.

⛔ **Le motif de la fiche, une fois de plus** : la remédiation R2 avait retiré un espion qui cassait, et écrit un
angle mort **à la place d'un test qui existait** — il suffisait d'une locale qui trie autrement. *Retirer une
assertion fragile n'oblige pas à renoncer à la preuve.*

Remédiation : texte de spec uniquement. **Revalidation R4 ciblée requise** (MEDIUM).

### Revalidation R4 CIBLÉE — une lentille Sonnet, contexte frais — **BOUCLE CLOSE**

Prompt versionné : `25-1c-b1-validate-prompt-r4.md`. Base : `git diff 717f7763 44d638f9` — commits
« docs(25-1c-b1): revalidation R2 ciblée » et « docs(25-1c-b1): revalidation R3 ciblée ».

**Rendu : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW.** Les cinq axes sont déclarés exercés, **par exécution** sous
vitest 4.1.3 et Node 22 :
- le test d'`i18nLocale()` sur module frais rougit sous « valeur initiale vide », **quelle que soit sa place**, et
  le contrôle négatif sans `resetModules` reproduit le défaut que la R3 a corrigé ;
- le tri sous `sv-SE` inverse bien l'ordre des quatre locales du projet, et la locale en dur rougit ; la CI épingle
  Node 22 officiel, à ICU complète ;
- la date `de-CH` rend `16.09.2026, 14:26` sous jsdom, et les mutations « `'fr-CH'` en dur » et
  « `navigator.language` » rougissent ; la date reste le 16 en UTC comme à Zurich ;
- le `login` à trois arguments et `test-state.ts:174-191` sont exacts ;
- le Change Log R3 est fidèle, décomptes justes.

**Vérification de l'orchestrateur** : la lentille a d'abord écrit ses sondes **dans le dépôt**
(`frontend/src/lib/shared/utils/_r4probe_*`), en violation de l'interdit, puis les a retirées et refaites dans le
scratchpad. Contrôlé avant clôture : aucun fichier de sonde, `git diff` vide, seuls les prompts non suivis. ⛔ *Un
interdit écrit n'empêche pas l'écriture* : le motif de la passe 4 de la 24-5, reproduit en plus bénin.

**Critère de clôture** : 0 finding, et la dernière remédiation ne touche que du texte de spec.

**Trend de la boucle rouverte** (après la clôture en 7 passes de la version d'avant les arbitrages du soir) :

| passe | modèle | rendu |
|---|---|---|
| R1 | Opus, complète | 2 M, 8 L |
| R2 | Sonnet, ciblée | 3 M |
| R3 | Opus, ciblée | 2 M, 5 L |
| R4 | Sonnet, ciblée | **0** |

**Ce que la boucle rouverte apprend** : les deux MEDIUM de R1 étaient **de conception** — la locale que le frontend
jette, et une page que rien n'obligeait à passer par `toSelectOptions`. Tous les suivants venaient de la
remédiation, sous la forme propre à cette fiche : **une preuve ajoutée qui ne prouvait rien** (un espion qui casse,
un test dont le résultat dépend de sa place, un scénario joué sous le mauvais rôle). La boucle ne s'est close
qu'une fois chaque preuve **exécutée** par la lentille sur sa mutation, et non plus relue.
