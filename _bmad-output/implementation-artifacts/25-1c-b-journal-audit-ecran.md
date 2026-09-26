# Story 25.1c-b : L'écran de consultation du journal d'audit

Status: split

⛔ **FICHE DÉCOUPÉE LE 2026-09-15 — NE PAS IMPLÉMENTER CE DOCUMENT.** Arbitrage du Project Lead après la
passe 2 de validation (sévérité MEDIUM → MEDIUM sur des défauts d'origine répartis dans cinq zones) :

- **`25-1c-b1-journal-audit-ecran.md`** — l'écran ;
- **`25-1c-b2-journal-audit-textes.md`** — les manuels, le README, le vocabulaire.

L'extraction du téléchargement (volet C) **sort** de la story, vers une issue de dette. Cette fiche reste
la **référence** de ce que les passes 1 et 2 ont établi ; les treize findings de la passe 2 sont appliqués
dans les deux fiches filles, pas ici.

⚠️ **Issue du SPLIT de la 25-1c** (arbitrage du Project Lead du 2026-09-15) :

| | objet | état |
|---|---|---|
| 25-1c-zero | la colonne `audit_log.company_id` | done — PR #437 ouverte |
| 25-1c-a | la route de consultation et son export CSV — backend | ready-for-dev, validée |
| **25-1c-b** *(celle-ci)* | **l'écran, le menu, les libellés, les E2E, les manuels** | ready-for-dev |

⛔ **Dépendance dure** : cet écran consomme `GET /api/v1/audit-log` et `/export.csv` **tels que la
25-1c-a les livre** (`25-1c-a-journal-audit-route.md`, AC 5-14). **Ne pas implémenter avant le merge de
la 25-1c-a** — la branche est empilée sur la sienne et se rebase sur `main` après.

⛔ **C'est cette story qui FERME #378** : le titre de sa PR porte `closes #378` (le dépôt merge en
squash — § *Commits qui adressent une issue*).

## Story

**En tant que** comptable ou administrateur,
**je veux** consulter le journal d'audit de ma société dans l'application — le filtrer, déplier le
détail d'une entrée, l'exporter,
**afin de** **produire** la trace des corrections apportées aux livres sans accès direct à la base —
« *"apparent" et "archivé dans une table que personne ne peut lire" ne sont pas la même chose* » (#378).

**Couvre** : [#378], volet écran. **Ferme** l'issue.

## ✅ Arbitrages du Project Lead — 2026-09-15 (`epic-25-vague1-suite.md`)

1. **Qui consulte** : **Comptable** et **Admin** ; ni Consultation, ni clé API.
2. **Filtre strict par société** (porté par la route).
3. **Affichage sobre** — cité de l'epic : *« le code d'action tel quel, les types d'entité traduits, les
   détails en JSON indenté »*.
4. **Vocabulaire** : **« journal d'audit »** — `Audit-Protokoll`, `registro di audit`, `audit log`.

⚠️ **Hérités de la 25-1c-a, NON arbitrés — à confirmer en revue** : une entrée **sans société** est
affichée (AC 5 : « société indéterminée ») ; l'**export écrit une entrée d'audit** ; les cellules du
**CSV** gardent les codes bruts.

## Acceptance Criteria

### Volet A — la feature `frontend/src/lib/features/audit-log/`

**1. Les types** — `audit-log.types.ts` : `AuditLogEntry` en miroir **exact** du DTO de la 25-1c-a
(AC 8) — `id`, `createdAt` (chaîne ISO UTC finissant par `Z`), `actorLabel`, `actorType: 'user' |
'api_key'`, `actorApiKeyId: number | null`, `userId`, `action`, `entityType`, `entityId`,
`companyId: number | null`, `details: unknown` ; `AuditLogQuery` (`dateFrom`, `dateTo`, `entityType`,
`entityId`, `action`, `offset`, `limit`). Liste : l'enveloppe `{ items, total, offset, limit }` déjà
typée (`src/lib/shared/types/user.ts:22-27`).

**2. L'API** — `audit-log.api.ts`, via `apiClient` :

- `listAuditLog(query)` → `GET /api/v1/audit-log` ; **un paramètre vide ou absent n'est pas envoyé** ;
- `exportAuditLogCsv(query)` → `apiClient.getBlob(...)` (`api-client.ts:526-529`), nom de fichier lu
  dans `Content-Disposition` par `parseContentDispositionFilename`, téléchargement par
  `triggerDownload` — **tous deux importés du module partagé de l'AC 11**, jamais recopiés.

**3. Les libellés des 28 types d'entité** — `entity-type-label.ts`, **sur le patron exact de
`imported-supplier-invoices/error-label.ts:28-39`** — **la CARTE seulement** : sa fonction
`importErrorLabel` (`:41-48`) a un repli **différent** de celui qu'exige cet AC (cf. ci-dessous). Une carte exportée
`ENTITY_TYPE_LABELS: Record<string, [string, string]>` (code → [suffixe de clé, repli français]) et une
fonction `entityTypeLabel(code)`. Clés `audit-log-entity-<suffixe>`, suffixe = code avec `_` → `-`.

| code | repli | code | repli |
|---|---|---|---|
| `account` | Compte | `export` | Export |
| `api_key` | Clé API | `fiscal_year` | Exercice comptable |
| `bank_account` | Compte bancaire | `imported_supplier_invoice` | Facture importée |
| `bank_imports` | Import bancaire | `installation` | Installation |
| `bank_profiles` | Profil bancaire | `invoice` | Facture |
| `bank_transaction` | Transaction bancaire | `journal_entry` | Écriture |
| `company` | Société | `payment_batch` | Lot de paiement |
| `company_dunning_settings` | Réglages de recouvrement | `product` | Produit |
| `company_invoice_settings` | Réglages de facturation | `project` | Projet |
| `contact` | Contact | `reconciliation_rules` | Règle d'affectation |
| `contact_person` | Personne de contact | `report` | Rapport |
| `credit_note` | Avoir | `supplier_invoice` | Facture fournisseur |
| `dunning_level` | Niveau de rappel | `user` | Utilisateur |
| `email_template` | Modèle d'e-mail | `vat_rate` | Taux de TVA |

- ⚠️ **Les codes au pluriel** (`bank_imports`, `bank_profiles`, `reconciliation_rules`) sont les codes
  **réels** : ne pas les « corriger » — la carte doit coïncider avec ce que la base contient.
- ⛔ **Un code INCONNU s'affiche tel quel**, sans erreur — **et c'est ici que le patron s'arrête** :
  `importErrorLabel` rend pour un code inconnu un message **traduit et interpolé**
  (`imported-supplier-invoices-error-unknown`, `error-label.ts:45-47`) ; `entityTypeLabel` rend le **code
  brut**, sans appel à `i18nMsg`. Motif : le journal conserve les codes des versions
  antérieures et des archives fusionnées, et un code futur ne doit rien casser.
- **La liste est recomptée depuis la source**, pas recopiée (cf. Dev Notes : 28 types littéraux, trois
  sites non littéraux vérifiés à la main).

**4. Les paramètres d'URL** — `query-helpers.ts` sur le patron de
`journal-entries/query-helpers.ts:19-121` : sérialisation qui **omet** les valeurs par défaut,
lecture qui **ignore** les valeurs invalides (date mal formée, `entityId` non numérique ou `<= 0`,
`entityType` hors carte **conservé** — un code historique doit rester filtrable).

### Volet B — l'écran `frontend/src/routes/(app)/audit-log/`

**5. La page** — `+page.svelte` :

- **filtres** : période (`Input type="date"`, du / au), type d'entité (`Select` : « Tous » + les 28
  libellés), identifiant d'entité (numérique, **désactivé tant qu'aucun type n'est choisi** — la route
  répond 400 sinon), action (texte, anti-rebond 300 ms) ; **synchronisés dans l'URL** et relus au
  montage (patron `journal-entries/+page.svelte:138-183`) ; bouton « Réinitialiser » ;
- **tableau**, colonnes : **Date** (heure **locale** du navigateur, `Intl.DateTimeFormat` de la locale
  courante, depuis `createdAt` UTC) · **Auteur** (`actorLabel`, avec la mention « clé API » si
  `actorType === 'api_key'`) · **Action** (code **brut**, police à chasse fixe) · **Type d'entité**
  (`entityTypeLabel`) · **N°** (`entityId`, « — » s'il vaut `0`) · **Société** (« société
  indéterminée » si `companyId` est `null`, **rien d'autre sinon** — la société est celle de
  l'utilisateur) · **Détails** (bouton qui déplie un `<pre>` de `JSON.stringify(details, null, 2)` ;
  absent si `details` est `null`) ;
- **pagination** Précédent / Suivant, « X–Y sur N », `limit` 50, garde `if (loading) return` (patron
  `journal-entries/+page.svelte:245-265`) ;
- **trois états distincts** — chargement, **erreur** (message, jamais une liste vide silencieuse —
  patron `supplier-invoices/+page.svelte:429-438`), vide ;
- **export CSV** : bouton qui transmet **les filtres affichés**, désactivé pendant l'export ; un 400
  `RESULT_TOO_LARGE` affiche un message dédié (« Trop de résultats : affinez les filtres »), toute
  autre erreur passe par `notifyError`.

**6. La garde** — `+page.ts`, patron **exact** de `supplier-invoices/import/+page.ts` : `ssr = false`,
redirection 302 vers `/` si le rôle n'est ni `Admin` ni `Comptable`. ⚠️ Le masquage d'interface ne
protège rien : c'est le 403 de la route qui fait foi.

**7. Le menu** — entrée `{ i18nKey: 'nav-audit-log', fallback: "Journal d'audit", href: '/audit-log' }`
dans la liste **`comptableOnly`** du groupe `administration` (`routes/(app)/+layout.svelte:130-136`).
⚠️ **`'audit-log'` s'ajoute à `FAMILLES_RESOLUES['nav-']`** (`i18n-keys.test.ts:321-327`) : une clé de
menu est une donnée lue par `getItemLabel`, et c'est ce trou qui avait laissé quatre entrées en
français dans les quatre langues (commentaire `:303-307`).

**8. Les sélecteurs** — `data-testid` sur tout ce que les tests visent : `audit-log-table`,
`audit-log-row`, `audit-log-empty`, `audit-log-error`, `audit-log-loading`, `audit-log-export`,
`audit-log-filter-date-from`, `-date-to`, `-entity-type`, `-entity-id`, `-action`, `-reset`,
`audit-log-details-toggle`, `audit-log-details`, `audit-log-prev`, `audit-log-next`. ⛔ **Aucun
sélecteur par libellé** — `e2e-selecteurs-traduits.test.ts` le refuserait, et `DETTE_CONNUE` ne
s'allonge pas.

### Volet C — le téléchargement partagé

**9. `frontend/src/lib/shared/utils/download.ts`** reçoit `triggerDownload` et
`parseContentDispositionFilename`, **déplacés** depuis `features/export/exports.api.ts:52-110`, avec
leurs tests. **Comportement identique**.

⛔ **`parseContentDispositionFilename` n'a PAS la même distribution que `triggerDownload`, et l'oublier
casse la compilation** :

| site | aujourd'hui | après |
|---|---|---|
| `features/export/exports.api.ts:52` | **définition d'origine**, exportée | supprimée — importée de `download.ts` |
| `features/admin-backup/admin-backup.api.ts:55` | **seconde copie, exportée** — son commentaire `DC-B1` (`:10-14`) planifiait déjà l'extraction « vers `lib/shared/utils/download.ts` » | supprimée — importée de `download.ts` |
| `features/imported-supplier-invoices/imported-supplier-invoices.api.ts:11` | **import croisé** depuis `$lib/features/export/exports.api` | **repointé** sur `$lib/shared/utils/download` — sans quoi il ne compile plus |

Les tests des deux copies (`exports.api.test.ts`, `admin-backup.api.test.ts`) se **fusionnent** dans
celui de `download.ts`, sans perdre un cas ; les deux copies étant identiques, un cas présent d'un seul
côté est conservé.

**10. Les quatre copies existantes de `triggerDownload`** l'importent désormais —
`reports.api.ts:348`, `exports.api.ts:98`, `imported-supplier-invoices.api.ts:71`,
`admin-backup.api.ts:92`. Motif : la règle DRY du `CLAUDE.md`, et le commentaire même de
`exports.api.ts:89-92`, qui reportait l'extraction « si > 2 features dupliquent » — elles sont
quatre, l'écran serait la cinquième.

⚠️ **Ce volet est un rollout mécanique, séparable** : s'il résiste (un test d'une autre feature qui
casse pour une raison sans rapport), **l'AC 9 reste obligatoire** et l'AC 10 peut sortir en issue de
dette — à écrire, pas à taire.

### Volet D — les libellés et le vocabulaire

**11. Les clés i18n**, dans les quatre catalogues (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`),
en un bloc `# --- Journal d'audit — écran (Story 25-1c-b) ---` : `nav-audit-log`, titre et sous-titre de
page, libellés des filtres et du bouton de réinitialisation, en-têtes de colonnes, mentions « clé API »
et « société indéterminée », bouton de détail, états vide / erreur / trop de résultats, bouton d'export,
et les **28** `audit-log-entity-*`. **Le nombre total se recompte depuis la source** à l'implémentation
(`grep -c '^audit-log-\|^nav-audit-log' …`) et s'écrit avec sa ventilation ; les 12 clés
`audit-log-csv-header-*` et `audit-log-export-error-too-large` de la 25-1c-a **existent déjà** et ne se
recréent pas.

**12. Les gardes i18n du frontend, mises à jour avec leur ventilation** (`src/lib/shared/i18n-keys.test.ts`) :

- `MOTIFS_DYNAMIQUES['audit-log-entity-']` et **`CARDINALITES['audit-log-entity-'] =
  Object.keys(ENTITY_TYPE_LABELS).length`** — **lu depuis la production**, sur le précédent
  `imported-supplier-invoices-error-` (`:299`), et non recopié ;
- `FAMILLES_RESOLUES['nav-']` (AC 7) ;
- les compteurs d'`ATTENDU` (`:206-213`) — `sitesTotal`, et selon la forme des appels `sitesGabarit`,
  `sitesNonResolus`, `relais` — **recomptés**, leur historique commenté prolongé ;
- `CANDIDATES_ATTENDUES` d'`i18n-libelle-en-dur.test.ts:115` si une fonction `*Label` est ajoutée
  (`entityTypeLabel` en est une).

**13. Le vocabulaire, porté au glossaire** — `docs/i18n-glossaire.md`, **partie A** (terme arbitré) :
« journal d'audit » | `Audit-Protokoll` | `registro di audit` | `audit log` | arbitrage du 2026-09-15, clé
`nav-audit-log`. ⛔ **Règle 3 du glossaire : un terme de la partie A s'installe en mettant à jour TOUTES
ses occurrences déjà livrées.** Deux clés disent aujourd'hui autre chose :

| clé | fr | de | it | en |
|---|---|---|---|---|
| `fiscal-year-reopen-confirmation-body` | « piste d'audit » | « Audit-Protokoll » | « pista di audit » | « audit trail » |
| `export-global-content-excludes` | « journal d'audit interne » | « internes Audit-Log » | « registro audit interno » | « internal audit log » |

⇒ **alignées** sur les quatre termes. Le grep `grep -rniE "audit" crates/kesh-i18n/locales/*/messages.ftl`
se rejoue pour trouver une troisième occurrence. ⚠️ **Greper le mot `audit` seul, jamais la locution** :
le catalogue français écrit l'apostrophe **typographique** (`’`) — un motif `piste d'audit` rend zéro
ligne sur `fr-CH:766`, vérifié à la spécification. Les **libellés des 28 types** suivent la partie A là où
elle les atteste ; un terme récurrent qu'elle n'atteste pas va en **partie B**, avec sa valeur proposée.

### Volet E — ce que l'écran rend faux

**14. Le manuel utilisateur, `user-manual.tex:497-503`** — l'encadré « Ce que le logiciel ne fait pas
encore à votre place » affirme : *« Ce qui manque encore, c'est la consultation du journal d'audit depuis
l'application »*. **Devient faux.** ⚠️ **Le même encadré porte un second énoncé DÉJÀ faux**, constaté à
la spécification : *« Le verrou reste par ailleurs annuel : aucun verrou de période plus fin n'existe
encore »* — la Story **24-4c** a livré le verrou de période (PR #425). Les deux se corrigent **ensemble** ;
si l'encadré n'a plus rien à dire, il disparaît.

**15. Le manuel utilisateur, `user-manual.tex:1611-1616`** — *« n'est pas encore consultable via un écran
dédié dans l'interface (page de consultation prévue pour une version ultérieure) »*. → décrire l'écran :
menu **Administration → Journal d'audit**, réservé au Comptable et à l'Admin, filtres, détail, export CSV
(dates en **UTC** dans le fichier, en heure locale à l'écran).

**16. Le manuel d'administration, `admin-manual.tex:1786`** — après la 25-1c-a, il dira qu'une route
existe et que l'écran reste à venir : **l'écran existe**.

**17. Le README**, feuille de route : `README.md:218` (« le journal d'audit n'est consultable par aucun
écran ») et `:219` (« la consultation depuis l'application ([#378]), qui n'existe toujours pas »). →
mis à jour **dans le même commit** (§ *Synchroniser le planning du README*).

**18. Régénération et contrôle** : `make fr`, PDF commités, **PDF aplati vérifié**
(`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) — anciennes phrases absentes, nouvelles présentes. ⛔ En
LaTeX le souligné s'écrit `\_`. `website/index.html:106` et `roadmap.html:105` (« an audit log that a
backup import no longer replaces ») **restent vrais** et ne se touchent pas.

### Volet F — les tests

**19. Vitest** :

- `entity-type-label` : les 28 codes rendent leur clé ; `_` → `-` ; **un code inconnu rend le code
  brut** ;
- `query-helpers` : aller-retour URL, valeurs invalides ignorées, `entityType` inconnu conservé ;
- `audit-log.api` : un paramètre vide n'est pas envoyé ; l'export passe par `getBlob` et le nom de
  `Content-Disposition` ;
- `download.ts` : les tests déplacés passent **sans modification de leurs assertions** ;
- la page : `load()` **redirige** un rôle Consultation (patron `routes/(app)/users/users-page.test.ts:26-53`) ;
  rendu des trois états, de « société indéterminée », de « — » pour `entityId = 0`, et du dépliage du
  détail, API mockée (patron `contacts/contacts-page.test.ts:17-40`).

**20. E2E Playwright** — `frontend/tests/e2e/audit-log.spec.ts`, patron `journal-entries.spec.ts:1-40` :

- un Comptable crée une entité auditée par `authedApiContext` (un contact), ouvre l'écran **par le
  menu** (`nav-link-audit-log`) et voit la ligne `contact.created` ;
- le filtre par type d'entité la garde, un filtre sur une autre période l'écarte ; l'URL survit au
  rechargement ;
- le détail se déplie et contient le JSON ;
- l'export déclenche un téléchargement dont `suggestedFilename()` commence par `kesh-journal-audit-`
  (patron `invoices_echeancier.spec.ts:186-191`) ;
- ⛔ un utilisateur **Consultation** ne voit **pas** l'entrée de menu (`toHaveCount(0)`) et `/audit-log`
  le **redirige** (patron `reports.spec.ts:421-446`).

**21. Épreuve par mutation, résultats OBSERVÉS consignés** :

| mutation | test attendu rouge |
|---|---|
| garde de `+page.ts` retirée | 19, redirection Consultation |
| repli « code inconnu » retiré de `entityTypeLabel` | 19, code inconnu |
| une entrée retirée de `ENTITY_TYPE_LABELS` sans toucher le catalogue | la cardinalité de `i18n-keys.test.ts` |
| entrée de menu passée de `comptableOnly` à `items` | 20, Consultation voit le menu |
| filtres non transmis à l'export | 19, API d'export |

⛔ **Un test qui ne compile pas ne rougit pas : il se tait.**

## Tasks / Subtasks

- [ ] **T0 — Rebase** sur `main` après le merge de la 25-1c-a (ne pas implémenter avant) ; vérifier que
      les deux routes répondent (`curl` authentifié) sur le backend local.
- [ ] **T1 — Téléchargement partagé** (AC 9, 10), `npm run test:unit` vert **avant** d'aller plus loin.
- [ ] **T2 — Types, API, libellés, paramètres d'URL** (AC 1-4) et leurs tests (AC 19).
- [ ] **T3 — Page, garde, menu, sélecteurs** (AC 5-8) et tests de page (AC 19).
- [ ] **T4 — i18n, quatre locales** (AC 11) — ⚠️ recompte depuis la source.
- [ ] **T5 — Gardes i18n** (AC 12), compteurs **recomptés** et ventilés.
- [ ] **T6 — Glossaire et alignement des deux clés** (AC 13).
- [ ] **T7 — E2E** (AC 20). ⛔ *Une tâche qui décrit un test est une promesse ; la cocher sans l'avoir
      écrit la transforme en mensonge.*
- [ ] **T8 — Mutations** (AC 21), résultats observés.
- [ ] **T9 — Manuels et README** (AC 14-18), PDF régénérés et vérifiés aplatis.
- [ ] **T10 — Gates** : frontend complet (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) ;
      backend `cargo test -p kesh-i18n` (parité des catalogues) puis `scripts/test-fast.sh` ; **E2E
      complète**, `kesh_e2e` reconstruite, **build frontend postérieur au dernier patch**.

## Dev Notes

### Ce que la story touche

| zone | fichiers |
|---|---|
| feature neuve | `frontend/src/lib/features/audit-log/` — `audit-log.types.ts`, `audit-log.api.ts`, `entity-type-label.ts`, `query-helpers.ts`, tests |
| écran | `frontend/src/routes/(app)/audit-log/+page.svelte`, `+page.ts`, test de page |
| menu | `frontend/src/routes/(app)/+layout.svelte` |
| téléchargement | `frontend/src/lib/shared/utils/download.ts` (neuf) ; `reports.api.ts`, `exports.api.ts`, `imported-supplier-invoices.api.ts`, `admin-backup.api.ts` |
| gardes | `frontend/src/lib/shared/i18n-keys.test.ts`, `i18n-libelle-en-dur.test.ts` |
| i18n | `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` |
| E2E | `frontend/tests/e2e/audit-log.spec.ts` |
| docs | `docs/i18n-glossaire.md`, `docs/manual/fr/{user,admin}-manual.tex` + PDF, `README.md` |

⚠️ **Largeur** : un paquet frontend, un crate i18n, la documentation. Le volet C touche quatre features
de façon **mécanique** — c'est lui qui sort en premier si la story résiste (AC 10).

### Faits établis à la spécification, et non supposés

- **28 types d'entité littéraux** — recomptés par script sur les appels
  `NewAuditLogEntry::(user|for_actor|api_key|from_current_user)(` de `crates/*/src` hors tests. Le script
  a rendu 29 littéraux, dont **`admin_break_glass_reset`, qui est une ACTION sans point**
  (`auth/bootstrap.rs`, type `user`) ; et **trois sites sans type littéral** —
  `repositories/email_templates.rs:32` (helper, type `email_template` porté ailleurs) et
  `kesh-api/src/audit.rs:42,51` (le trait générique). ⇒ la carte a 28 entrées.
- **Menu** : `navGroups` (`+layout.svelte:62-163`), listes `comptableOnly` (`:130-136`) et `adminOnly`
  (`:138-161`), `isComptablePlus` (`:46-48`), testid `nav-link-<href slugifié>` (`:258-261`).
- **`triggerDownload`** : quatre copies, et le commentaire d'`exports.api.ts:89-92` qui planifiait
  l'extraction.
- **Glossaire** : partie A (attestés), partie B (sans précédent, arbitrage requis), et la règle 3 — un
  terme de A ne s'installe qu'avec toutes ses occurrences.
- **Pas de composant partagé** de pagination ni de datepicker : `Input type="date"`, pagination écrite
  dans la page.
- **E2E** : `workers: 1`, `locale: 'fr-CH'` (`playwright.config.ts:53,58`) — la suite ne tourne qu'en
  français, d'où l'exigence de `data-testid`.

### Ce qui ne bouge PAS

- Le backend, hors catalogues FTL : la route est la 25-1c-a.
- Les libellés **par action** — hors arbitrage 3 (« le code d'action tel quel »).
- Le **nom** d'une clé API (la route ne renvoie que `actorApiKeyId`).
- `website/`.

### Intelligence des stories précédentes

- **25-1c-a** (validation en 5 passes) : *une forme qui casse à chaque correction se retire, elle ne se
  rapièce pas* ; *un arbitrage résumé est un arbitrage réécrit* — citer, ne pas paraphraser.
- **25-1c-zero** : une assertion **vraie par construction** a traversé quatre passes ; pour chaque
  assertion, se demander ce qui la rendrait fausse.
- **23-3b / 23-1** : les clés de menu échappent aux gardes si `FAMILLES_RESOLUES` n'est pas tenu ; les
  cartes de libellés se **lisent** depuis la production.
- **Gate** : un build frontend **en retard d'un périmètre** a déjà faussé un gate E2E (24-4c, 25-1b).

### Hors périmètre

- Libellés par action, recherche plein texte dans `details`, nom des clés API.
- **[#386]** (export de souveraineté) → 25-5.
- **[#431]**, **[#434]**, **[#435]**.

### References

- `25-1c-a-journal-audit-route.md` — **le contrat de la route** (AC 5-14)
- `epic-25-vague1-suite.md` § *Arbitrages du 2026-09-15*
- `frontend/src/lib/features/imported-supplier-invoices/error-label.ts:28-39` — **patron de la carte de libellés** (la fonction `:41-48` a un repli différent)
- `frontend/src/lib/features/journal-entries/query-helpers.ts`, `routes/(app)/journal-entries/+page.svelte` — **patron de la liste filtrée**
- `frontend/src/routes/(app)/supplier-invoices/import/+page.ts` — **patron de la garde Comptable+**
- `frontend/src/lib/shared/i18n-keys.test.ts:206-213,234-301,319-327` · `docs/i18n-glossaire.md` § A, B, *Comment s'en servir*
- `CLAUDE.md` § *Test Locally First*, § *Propagation post-patch*, § *Le prompt d'une passe doit NOMMER le manuel*

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-09-15** — Spécification créée (`bmad-create-story`), après la validation de la 25-1c-a. Faits
  recomptés depuis la source : **28 types d'entité** (29 littéraux dont une action sans point, et trois
  sites non littéraux vérifiés), quatre copies de `triggerDownload`, deux clés i18n au vocabulaire
  divergent. **Défaut préexistant constaté** : `user-manual.tex:502-503` affirme qu'aucun verrou de
  période n'existe, alors que la 24-4c l'a livré — corrigé dans le même encadré (AC 14).
- **2026-09-15** — Contrôle checklist, quatre références vérifiées au sol : `error-label.ts`,
  `exports.api.ts`, `CANDIDATES_ATTENDUES` (`:115`), et l'apostrophe **typographique** du catalogue
  français, qui rendait muet un grep sur la locution.

### Passe 1 de `bmad-create-story validate` — deux lentilles, contexte frais

Prompt versionné : `25-1c-b-validate-prompt-p1.md`.

| Lentille | Modèle | Rendu brut | Après vérification au sol |
|---|---|---|---|
| Sonnet | Sonnet | 0 C, 0 H, 2 M, 2 L | **4 retenus** |
| Haiku | Haiku 4.5 | 0 | 0 — deux axes survolés, couverts par Sonnet |

**Bilan : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW.**

- **M1 — l'extraction oubliait la distribution propre de `parseContentDispositionFilename`** : une
  **seconde copie exportée** dans `admin-backup.api.ts:55` (sous un commentaire `DC-B1` qui planifiait la
  même extraction), et un **import croisé** dans `imported-supplier-invoices.api.ts:11` qui aurait cessé
  de compiler au déplacement. → tableau des trois sites à l'AC 9, fusion des tests des deux copies.
- **M2 — le « patron exact » contredisait l'exigence qui le suit** : `importErrorLabel` rend un message
  traduit pour un code inconnu, l'AC 3 exige le code brut. → citation restreinte à la **carte**
  (`:28-39`), divergence écrite.
- **L3** — `exports.api.ts` fait 110 lignes (`:52-111` → `:52-110`).
- **L4** — « Pièce importée » n'est attesté nulle part ; le catalogue dit « Facture importée »
  (`imported-supplier-invoices-err-not-found`) → aligné.

⚠️ **La lentille Haiku a déclaré ses neuf axes exercés**, mais a compté les 28 types **dans la spec**
(14 lignes × 2) au lieu de les recompter **dans le code**, et n'a vérifié que l'existence des gardes i18n,
pas leur mécanique. Ces deux axes ont été exercés par Sonnet : recompte par script et `diff` → **ensembles
identiques, 28/28** ; lecture intégrale d'`error-label.ts`, `i18n-keys.test.ts` et
`lint-i18n-ownership.js`.
