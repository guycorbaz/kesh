# Story 25.1c-a : La route de consultation du journal d'audit

Status: review

⚠️ **RÉOUVERTE le 2026-09-15 au soir** après sa validation en 5 passes : trois arbitrages du Project Lead
changent sa conception (arbitrages 2, 3 et 5 ci-dessous). ✅ **Revalidation CLOSE le 2026-09-16 en 4 passes
ciblées** (`1H/3M/3L → 2M → 1M/3L → 1L`), et les **122 libellés français arbitrés** le même jour
(`25-1c-a-libelles-proposes.md`). **Implémentation commencée le 2026-09-16**, après le merge de la PR #437
(squash `842eaac0`).

⚠️ **Issue du SPLIT de la 25-1c**, décidé le 2026-09-15 par le Project Lead avant spécification :
sept modules, donc la § *Règle de splitting préventif*. Découpage :

| | objet | état |
|---|---|---|
| 25-1c-zero | la colonne `audit_log.company_id` | done (PR #437) |
| **25-1c-a** *(celle-ci)* | **la route de consultation, son vocabulaire traduit et son export CSV — backend** | ready-for-dev, réouverte |
| 25-1c-b1 | l'écran, le menu, les gardes i18n du frontend, les E2E | ready-for-dev, réouverte |
| 25-1c-b2 | les manuels, le README, le vocabulaire « journal d'audit » | ready-for-dev, réouverte |

⛔ **Cette story ne livre AUCUN écran.** Elle livre ce qu'un écran consommera, et ce qu'un réviseur
peut déjà obtenir par un appel HTTP authentifié : la liste, l'export, et **le vocabulaire traduit** — les
libellés des types d'entité et des actions, dont elle est **la source unique** (AC 16).

⚠️ **Dépendance** : la branche est empilée sur `story/25-1c-zero-audit-company-id`. Elle se rebase sur
`main` (`git rebase --onto origin/main 42b6aac0` — par **SHA** : le nom de branche peut être supprimé au merge) **après** le squash-merge
de la PR #437, et **avant** d'ouvrir sa propre PR.

## Story

**En tant que** comptable ou administrateur d'une société tenue dans Kesh,
**je veux** lire le journal d'audit de ma société — filtré par période, par entité et par action, dans
la langue de l'interface — et l'exporter,
**afin de** pouvoir **produire** la trace des corrections apportées aux livres, ce que l'art. 958f CO
exige et qu'aucune route ne permet aujourd'hui.

**Couvre** : [#378], volet backend. **Ne ferme pas l'issue** — le `closes #378` appartient à la PR des
25-1c-b1 et b2, qui livrent l'écran et ses textes.

## ✅ Arbitrages du Project Lead — 2026-09-15 (`epic-25-vague1-suite.md`)

1. **Qui consulte** : le **Comptable** et l'**Admin**. **Ni le rôle Consultation** — les détails
   portent des e-mails et des changements de rôle —, **ni une clé API** — qu'on ne puisse pas
   extraire le journal entier par programme.
2. **Filtre strict par société.** Un import de sauvegarde restaure **la même** installation (*« si on
   importe une sauvegarde, c'est que la base de données a disparu ou est corrompue et doit être
   recréée »*). Et, au soir, sur les entrées sans société : *« le cas réel n'existe pas : le système ne
   sera pas en production avant de pouvoir faire les sauvegardes complètes. le cas théorique est …
   théorique »*. ⇒ **la route ne rend que `company_id = ?`** ; une entrée sans société n'apparaît dans
   aucune consultation.
3. **Affichage** — cité de l'epic : *« le code d'action tel quel, les types d'entité traduits, les
   détails en JSON indenté »* ; **amendé au soir** sur le CSV : *« dans l'export csv, il faut mettre la
   traduction de la langue utilisée par l'utilisateur de kesh, sinon ce sera difficile à utiliser »* ; et
   sur l'écran, à la proposition d'afficher aussi l'action traduite : *« ok »*. À la proposition d'écrire
   les libellés d'action dans une story séparée : *« non, maintenant »*.
   ⇒ **types d'entité ET actions traduits, à l'écran comme dans le CSV** ; les libellés s'écrivent
   **dans cette story** ; les **détails** restent du JSON.
4. **Vocabulaire** : « journal d'audit » — `Audit-Protokoll`, `registro di audit`, `audit log`.
5. **L'export n'écrit PAS d'entrée d'audit** — réponse *« non »* à « chaque export CSV du journal
   s'inscrit-il lui-même dans le journal ? ».

⚠️ **« La langue utilisée par l'utilisateur » est, dans Kesh, celle de l'INSTALLATION** : il n'existe
aucune langue par utilisateur. L'interface entière suit `KESH_LANG` (`config.rs:831`,
`state.config.locale`), que `routes/i18n.rs:21` sert à l'écran. **La route, le vocabulaire et le CSV
suivent la même** — et non la `accounting_language` de la société, qui est la langue des libellés
comptables.

## Acceptance Criteria

### Volet A — la lecture (repository `kesh-db`)

**1. `repositories/audit_log.rs` gagne une lecture paginée scopée** — `list_by_company_paginated(pool,
company_id, AuditLogListQuery) -> Result<AuditLogListResult, DbError>`, sur le **patron exact** de
`journal_entries::list_by_company_paginated` (`repositories/journal_entries.rs:825-895`) :

- deux `QueryBuilder` **distincts**, l'un pour `SELECT COUNT(*)`, l'autre pour les lignes, alimentés
  par **une seule** fonction `push_where_clauses` (le commentaire `:745-747` dit pourquoi un
  `QueryBuilder` ne se réutilise pas) ;
- colonnes lues par `const COLUMNS` (`:52`), **jamais** une liste recopiée ;
- `AuditLogListQuery { date_from: Option<NaiveDate>, date_to: Option<NaiveDate>, entity_type:
  Option<String>, entity_id: Option<i64>, action: Option<String>, limit: i64, offset: i64 }` ;
  `AuditLogListResult { items: Vec<AuditLogEntry>, total, offset, limit }` ;
- clamp défensif `limit.clamp(1, MAX_LIMIT)` et `offset.max(0)` (`:837-838`), `MAX_LIMIT` du module.

**2. La clause WHERE, et chacune de ses parties est une décision** :

```sql
WHERE company_id = ?
  [AND created_at >= ?]                 -- date_from à 00:00:00.000
  [AND created_at <= ?]                 -- date_to à 23:59:59.999
  [AND entity_type = ?] [AND entity_id = ?] [AND action = ?]
ORDER BY created_at DESC, id DESC
```

- ⛔ **`company_id = ?` seul, sans `OR company_id IS NULL`** (arbitrage 2). Une entrée sans société
  n'appartient à aucune consultation.
- ⛔ **Bornes INCLUSIVES à la milliseconde, sans aucune arithmétique de date** : `date_from` à
  `00:00:00.000`, `date_to` à **`23:59:59.999`**, liées comme `NaiveDateTime`. `created_at` est un
  `DATETIME(3)` : aucune valeur ne tient entre `23:59:59.999` et le lendemain, la borne est donc
  **exacte**. ⚠️ **Ni `<= date_to` nu** — `<= '2026-09-15'` exclurait tout ce qui suit minuit pile —,
  **ni « `< date_to + 1 jour` »** : cette forme, retenue d'abord, a produit **trois passes de défauts**
  (panique de `+ Days(1)` sur `+262142-12-31`, an 10000 comparé en **0 ligne** sous l'avertissement
  1292, puis une règle d'omission par `checked_add_days` qui ne se déclenchait **jamais** pour
  `9999-12-31`). La borne inclusive supprime la classe entière : elle ne calcule rien, et
  `'9999-12-31 23:59:59.999'` est une valeur valide.

  **Éprouvé par l'orchestrateur sur base jetable** (2026-09-15, passe 4) : entrées à `2026-09-15
  00:00:00.000` et `23:59:59.999` **incluses**, `2026-09-16 00:00:00.000` et `2026-09-14 23:59:59.999`
  **exclues** ; au `9999-12-31`, les entrées de `00:00:00.000` et `23:59:59.999` **incluses** ; mêmes
  résultats en **requête préparée** avec bornes `DATETIME(6)` (la forme sqlx) ; **aucun avertissement** ;
  `EXPLAIN` : `range` sur l'index.

  ⛔ **Qui fait quoi, pour qu'il n'y ait qu'UNE implémentation** : la **route** valide la plage des deux
  dates et répond 400 (AC 7) ; le **repository** construit les deux bornes horodatées dans
  **`push_where_clauses`** — `date_from.and_hms_milli_opt(0, 0, 0, 0)` et
  `date_to.and_hms_milli_opt(23, 59, 59, 999)` —, partagée par `list_by_company_paginated` et
  `list_for_export`. ⚠️ **Précondition de `push_where_clauses`, à écrire en doc-comment** : des dates
  **dans `[1000-01-01, 9999-12-31]`**. Hors plage, `push_bind` **panique**
  (`sqlx-core-0.8.6/src/query_builder.rs:158`, `expect`), et aucun `CatchPanic` n'existe dans `kesh-api`
  — c'est la raison d'être de la validation de la route.
- **UTC** : MariaDB tourne en UTC (`@@system_time_zone = UTC`, `NOW() = UTC_TIMESTAMP()`, vérifié le
  2026-09-15) et `created_at` vaut `CURRENT_TIMESTAMP(3)`. Les dates de filtre s'entendent en **jours
  UTC**. À écrire dans le doc-comment. ⚠️ **Le fait qui tient vraiment n'est pas le réglage du serveur**
  (`@@time_zone` vaut `SYSTEM`) : c'est **sqlx qui impose `time_zone='+00:00'` à chaque session**
  (`sqlx-mysql-0.8.6/src/options/mod.rs:112`). C'est ce fait-là que le doc-comment cite.
- `ORDER BY … id DESC` en second : deux entrées de la même milliseconde gardent un ordre stable. Les
  entrées fusionnées d'une archive gardent leur `created_at` d'origine, et c'est **voulu** — elles se
  rangent à leur date réelle.

**3. `list_for_export(pool, company_id, &AuditLogListQuery, max_rows) -> Result<Vec<AuditLogEntry>,
DbError>`** — **même** `push_where_clauses`, **même** ordre, `LIMIT max_rows`, sans offset. ⛔ Aucune
seconde écriture de la clause WHERE : c'est ce qui garantit que l'export et l'écran montrent **les
mêmes lignes**.

**4. Rien d'autre ne change dans le repository** : `insert_in_tx` et `find_by_entity` intacts, **pas
de méthode `delete`** (en-tête du module, `:3-29`). Le doc-comment de `find_by_entity` (« la future UI
de consultation (story 3.5 ou post-MVP) ») devient faux et est corrigé, **de même que** l'en-tête
d'`entities/audit_log.rs:20` (« Story 3.5 étendra avec … l'UI de consultation ») — ⛔ **mais pas**
`migrations/20260413000001_audit_log.sql:5`, qui dit la même chose : une migration appliquée ne se
modifie plus, pas même un commentaire (P8) : la consultation passe par
`list_by_company_paginated`.

### Volet B — les routes de consultation (`kesh-api`)

**5. Trois routes**, montées dans **`comptable_routes`** (`lib.rs:336-660`), **avant** le
`route_layer(require_comptable_role)` de `:658` — ⛔ une route chaînée après le `route_layer` compile
et échappe au RBAC (`lib.rs:310-321`) :

- `GET /api/v1/audit-log` — la liste (AC 7-9) ;
- `GET /api/v1/audit-log/export.csv` — l'export (AC 10-14) ;
- `GET /api/v1/audit-log/vocabulary` — le vocabulaire traduit (AC 17).

Handlers dans un **nouveau** module `routes/audit_log.rs` (`pub mod audit_log;` dans `routes/mod.rs`).

**6. Les refus — identiques sur les TROIS routes, et ils ne se valent pas** :

| appelant | réponse | porté par |
|---|---|---|
| sans authentification | 401 | `require_auth` |
| rôle **Consultation** | 403 | `require_comptable_role` |
| **clé API**, même `read` d'un Admin | 403 `API_KEY_MANAGEMENT_FORBIDDEN` | `ensure_not_pat(&current_user)?` en **première** instruction du handler |
| **clé API** créée par un utilisateur **Consultation** | 403, code du RBAC | `require_comptable_role`, qui passe **avant** le handler — la clé porte le rôle de son créateur |

⚠️ **Pourquoi `ensure_not_pat` dans le handler, et pas une garde de bloc** : `comptable_routes` n'a
**pas** de couche anti-clé (seul le bloc admin porte `require_not_pat`, `lib.rs:329-331`), et le test
`the_admin_guards_have_no_alias_and_no_second_consumer_crate_wide` (`admin_pat_denied_e2e.rs:427-503`)
**interdit** tout second consommateur de `require_not_pat`. `ensure_not_pat` (`routes/api_keys.rs:95-100`,
`pub(crate)`) a déjà deux consommateurs hors gestion de clés (`admin.rs`, `fiscal_years.rs`).
⚠️ Son code d'erreur parle de « gestion » ; il est **réutilisé tel quel** — un variant de plus pour
un libellé serait du bruit. À écrire dans le doc-comment du handler.

**7. Les paramètres de la liste et de l'export** — struct `ListAuditLogQuery`,
`#[serde(rename_all = "camelCase")]` : `dateFrom`, `dateTo`, `entityType`, `entityId`, `action`,
`offset`, `limit`.

- **Pagination** : défaut `limit = 50`, **ramené** dans `[1, 200]` et `offset.max(0)` — convention
  des écritures (`routes/journal_entries.rs:287-288`), pas le rejet en 400 des factures. Motif : un
  écran de consultation n'a aucune raison de faire échouer une page pour une taille hors bornes.
- **Dates** reçues en `String` et parsées dans le handler (`journal_entries.rs:291-304`) : format
  invalide ⇒ **400 `VALIDATION_ERROR`** nommant le paramètre ; `dateFrom > dateTo` ⇒ 400 ; **date hors de `[1000-01-01, 9999-12-31]` ⇒ 400**, pour **`dateFrom` comme pour `dateTo`** (AC 2). ⚠️ La
  borne basse n'est pas moins exposée : `NaiveDate::from_str("-0001-01-01")` **réussit**, et une année
  négative — comme une année au-delà de 65535 (`sqlx-mysql-0.8.6/src/types/chrono.rs:263-264`,
  `u16::try_from`) — fait **paniquer** la liaison (`push_bind`, `query_builder.rs:158`). Ce n'est pas
  une 500 : c'est une panique, qu'aucune couche ne rattrape. `0999-12-31`, lui, se lie sans erreur et
  rendrait simplement tout le journal — il est refusé pour que la plage soit la même aux deux bornes.
- **Textes** `entityType`, `action` : des **codes** (l'écran les prend dans le vocabulaire, AC 17) ;
  `trim()` ; vide ⇒ absent ; plus long que la colonne (`entity_type` 32, `action` 64 caractères) ⇒ 400 —
  une valeur plus longue ne peut rien trouver, et le dire vaut mieux qu'une liste vide muette. Un code
  **absent** du vocabulaire reste **accepté** : un code historique doit rester filtrable.
- **`entityId`** : `<= 0` ⇒ 400 ; **fourni sans `entityType` ⇒ 400** — un identifiant n'a de sens que
  rapporté à son type (`entities/audit_log.rs:87-97`, « filtrer sur le couple »).
- ⚠️ **Refus hors `VALIDATION_ERROR`, et c'est assumé** : un `entityId`, `offset` ou `limit` **non
  numérique** est rejeté par l'extracteur `Query` d'Axum en **400 texte**, avant le handler — convention
  écrite du dépôt (`journal_entries.rs:275-279`). Ne pas la « corriger » ici.

**8. La réponse de la liste** — `ListResponse<AuditLogEntryResponse>` (`routes/mod.rs:44-63`), soit
`{ items, total, offset, limit }`. ⛔ **DTO dédié, et non l'entité sérialisée** :

| champ JSON | source | pourquoi un DTO |
|---|---|---|
| `id` | `id` | — |
| `createdAt` | `created_at`, **rendu en UTC explicite** : `2026-09-15T14:26:33.123Z`, format **`%Y-%m-%dT%H:%M:%S%.3fZ`** (patron `repositories/invoices.rs:89`) — millisecondes **toujours** écrites, ce que la sérialisation serde de `DateTime<Utc>` ne fait pas | `NaiveDateTime` sérialise **sans** fuseau ; l'écran doit savoir que c'est de l'UTC |
| `actorLabel` | `actor_label` | — |
| `actorType` | **`actor_type.as_str()`** : `"user"` / `"api_key"` | ⚠️ `ActorType` dérive `Serialize` **sans renommage** et sortirait `"User"` / `"ApiKey"` (`entities/audit_log.rs:35-49`) — la valeur de la base est la seule stable |
| `actorApiKeyId` | `actor_api_key_id` | — |
| `userId` | `user_id` | — |
| `action` | code brut | l'écran filtre et construit l'URL sur le **code** |
| `actionLabel` | `audit_labels::action_label(…)` (AC 16), langue de l'interface | arbitrage 3 |
| `entityType` | code brut | idem |
| `entityTypeLabel` | `audit_labels::entity_type_label(…)` (AC 16), langue de l'interface | arbitrage 3 |
| `entityId` | `entity_id` **tel quel**, `0` compris | `AUDIT_ENTITY_ID_NONE` vaut `0` ; le filtrer ferait disparaître l'information |
| `details` | `details_json` **tel quel** (`serde_json::Value`), `null` possible | arbitrage 3 |

⚠️ **Pas de champ `companyId`** : le filtre est strict (arbitrage 2), toutes les lignes portent la
société de l'appelant — le champ n'apprendrait rien.

**9. La consultation n'écrit PAS d'entrée d'audit**, pas plus que le vocabulaire ni l'export
(arbitrage 5, AC 14). Chaque page d'écran en écrirait une, et le journal enflerait de sa propre lecture.

### Volet C — l'export CSV

**10. `GET /api/v1/audit-log/export.csv`** — **mêmes refus** que l'AC 6 (`ensure_not_pat` en tête),
**mêmes filtres et mêmes validations** que l'AC 7 — `offset` et `limit` **ignorés**, comme l'export de
l'échéancier (`invoices.rs:1225-1231`). ⛔ La validation des paramètres est **une seule fonction**
partagée par les deux handlers, qui produit l'`AuditLogListQuery`.

**11. Plafond** : `MAX_EXPORT_ROWS = 10_000`. Le handler demande `MAX_EXPORT_ROWS + 1` lignes ; au-delà
⇒ **400 `RESULT_TOO_LARGE`** (`AppError::ResultTooLarge`, `errors.rs:546,1422`), message i18n
`audit-log-export-error-too-large` avec l'argument `limit`, **dans la langue de l'interface**
(`state.config.locale`) — patron `invoices.rs:1239-1259`, ⚠️ **sauf sa langue** : le patron lit
`company.accounting_language` (`invoices.rs:1249`), ce que l'encadré sous les arbitrages interdit ici.

**12. Le fichier** :

- UTF-8 **avec BOM**, séparateur `;`, fins de ligne CRLF, `csv::WriterBuilder` — patron
  `invoices.rs:1266-1281` ;
- **dix colonnes**, en-têtes traduits dans **la langue de l'interface** (`state.config.locale`, cf.
  l'encadré sous les arbitrages), clés `audit-log-csv-header-*` avec repli français :

  | clé | repli | contenu de la cellule |
  |---|---|---|
  | `…-id` | N° | `id` |
  | `…-created-at` | Date (UTC) | `created_at`, `YYYY-MM-DD HH:MM:SS.mmm` |
  | `…-actor` | Auteur | `actor_label` |
  | `…-user-id` | Identifiant d'auteur | `user_id` |
  | `…-actor-type` | Type d'auteur | **libellé traduit** (`audit-log-actor-type-user` / `-api-key`) |
  | `…-action` | Action | **libellé traduit** de l'action |
  | `…-entity-type` | Type d'entité | **libellé traduit** du type |
  | `…-entity-id` | Identifiant d'entité | `entity_id` |
  | `…-api-key-id` | Clé API | `actor_api_key_id`, vide si absent |
  | `…-details` | Détails | `details_json` en **JSON compact**, vide si absent |

  ⚠️ **`id` et `user_id` sont exportés, et ce n'est pas du remplissage** : des **trous** dans la suite
  des `id` sont un indice d'effacement, et `actor_label` n'est qu'un **instantané** du nom — `user_id`
  relie l'entrée au compte.

  ⚠️ **Plus de colonne « Société »** : le filtre strict la rendrait constante (arbitrage 2).
- ⛔ **Les libellés viennent des fonctions de l'AC 16, et d'elles seules** — les mêmes que la liste
  JSON. Un code **inconnu** du vocabulaire sort **tel quel** dans sa cellule.
- ⛔ **toute cellule texte passe par `csv_sanitize`** : `actor_label`, les trois libellés, `details`.
  Un nom d'utilisateur ou un détail commençant par `=`, `+`, `-` ou `@` est une injection de formule —
  `actor_label` est **choisi par un utilisateur**, et le repli d'un libellé est un **code** lu en base ;
- `Content-Type: text/csv; charset=utf-8` ; `Content-Disposition` par
  **`build_content_disposition`** (`util.rs:104-116`, RFC 5987), tag de langue
  **`state.config.locale.dir_name()`** (`kesh-i18n/src/lib.rs:33`, qui rend `de-CH`) — ⛔ **pas**
  `util::map_language_to_bcp47` (`util.rs:170`), qui attend une langue **comptable** (`"FR"`, `"DE"`…) et
  replie tout autre code sur `fr-CH` avec un simple avertissement ;
  nom `kesh-journal-audit-{slug société}-{AAAA-MM-JJ}.csv` (`slugify`, `util.rs:37`).

**13. `csv_sanitize` est EXTRAITE, pas recopiée.** Elle est aujourd'hui privée à `routes/invoices.rs`
(`:1176-1199`), et c'est le **seul** échappement anti-formule du dépôt — le moteur `kesh-report` n'en
a pas. ⇒ déplacée dans `crate::util` en `pub(crate)`, `invoices.rs` l'importe (trois cellules,
`:1310,1313,1320`), **comportement identique**. (Règle DRY du `CLAUDE.md` : c'est le deuxième
consommateur.)

⛔ **Elle n'a AUCUN test aujourd'hui** — vérifié : `grep -rn "csv_sanitize" crates` ne rend que sa
définition et ses trois appels, aucun cas dans le `mod tests` d'`invoices.rs`. Une fonction de
sécurité non testée qu'on déplace est une fonction qu'on peut casser sans le voir. ⇒ l'extraction
**ajoute** ses tests unitaires dans le `mod tests` de `util.rs` : un cas par caractère déclencheur
(`=`, `+`, `-`, `@`), le **contournement par espace de tête** (`" =cmd"`), le remplacement de `\r`,
`\n`, `\t`, et une chaîne ordinaire rendue inchangée.

**14. L'export n'écrit PAS d'entrée d'audit** (arbitrage 5). ⇒ ni transaction dédiée, ni patron
`emit_report_export_audit`, ni traitement particulier des requêtes `HEAD` : l'export ne fait que lire.

### Volet D — le vocabulaire traduit et les clés i18n

**15. Les clés, dans les quatre catalogues** (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`),
en un bloc commenté `# --- Journal d'audit — route, vocabulaire et export (Story 25-1c-a) ---` :

| famille | nombre | origine |
|---|---|---|
| `audit-log-csv-header-*` | 10 | AC 12 |
| `audit-log-export-error-too-large` | 1 | AC 11 |
| `audit-log-actor-type-user`, `-api-key` | 2 | AC 12 |
| `audit-log-entity-*` | **28** à la spécification | un par code de `ENTITY_TYPES` (AC 16) |
| `audit-log-action-*` | **92** à la spécification | un par code de `ACTIONS` (AC 16) — 82 écrites en littéral, 10 par des sites indirects (AC 18) |

⚠️ **Les deux derniers nombres se RECOMPTENT** : ils sont ceux du 2026-09-15, et c'est la garde de
l'AC 18 qui les fixe, pas cette fiche. Le **total** (133 à la spécification) s'écrit avec sa ventilation au
Dev Agent Record. Vocabulaire de l'arbitrage 4 ; le test de parité `crates/kesh-i18n/src/loader.rs:693`
impose le même jeu de clés partout.

**Règles d'écriture des libellés** :

- **action** : « *Objet* *participe passé* », majuscule initiale, sans point — `invoice.validated` →
  « Facture validée », `reconciliation_rule.applied` → « Règle d'affectation appliquée »,
  `contact_person.archived` → « Personne de contact archivée » ;
- **type d'entité** : le nom au singulier, majuscule initiale — y compris pour les codes au pluriel
  (`bank_imports` → « Import bancaire ») ;
- ⛔ **les termes suivent `docs/i18n-glossaire.md`**, partie A là où elle les atteste (créé /
  `erstellt` / `creato·a` / `created`, annulé / `storniert` / `annullato·a` / `cancelled`, avoir /
  `Gutschrift` / `nota di credito` / `credit note`…) ; un terme récurrent qu'elle n'atteste pas va en
  **partie B**, avec sa valeur proposée ;
- les codes irréguliers reçoivent un libellé qui dit ce qui s'est passé, non ce que dit le préfixe :
  `books.locked` → « **Période comptable verrouillée** » *(⚠️ cette ligne disait « Livres verrouillés »,
  la forme écartée : l'arbitrage du 2026-09-16 — « 1: période comptable » — l'a remplacée au document des
  libellés, et ce jumeau-ci avait été oublié. Corrigé à T4)*, `reconciliation.split_applied` →
  « Ventilation appliquée »,
  `admin_break_glass_reset` → « Réinitialisation d'urgence de l'administrateur » ;
- **la liste française complète est recopiée au Dev Agent Record** pour relecture par le Project Lead.

⚠️ **Les clés sont écrites par le backend, et le frontend les lit pourtant** — par la route
`/api/v1/i18n/messages`, qui sert **tout** le catalogue (`routes/i18n.rs:22`, `all_messages`). La 25-1c-b1
n'en a cependant pas besoin : elle reçoit les libellés **déjà traduits** (AC 8, AC 17). ⇒ **aucun**
compteur de `frontend/src/lib/shared/i18n-keys.test.ts` ne bouge, et `audit-log-` n'entre **pas** dans
`PREFIXES_A_COUVERTURE_CLOSE` (`:388`). Le vérifier, pas le supposer.

**16. Le module `crates/kesh-api/src/audit_labels.rs` — la SOURCE UNIQUE des libellés** :

- ⛔ **module PUBLIC** — `pub mod audit_labels;` dans `lib.rs`, éléments `pub` : un fichier de `tests/` est
  une **crate externe**, qui ne voit que le `pub` de `kesh_api` (`util`, déclaré `pub(crate)` à `lib.rs:24`,
  lui reste invisible). Sans cela, la garde de l'AC 18 devrait recopier les listes, ce qui est interdit ;
- `pub const ENTITY_TYPES: &[&str]` et `pub const ACTIONS: &[&str]`, triés par code ;
- clés **dérivées** : `audit-log-entity-{code}` et `audit-log-action-{code}`, où `.` et `_` deviennent
  `-` (`invoice.reminder_sent` → `audit-log-action-invoice-reminder-sent`), par **une seule** fonction
  `pub fn message_key(prefix: &str, code: &str) -> String`, que la garde de l'AC 18 appelle — jamais une
  dérivation recopiée dans le test, qui ne verrait plus une mutation de la vraie ;
- `pub fn entity_type_label(i18n: &I18nBundle, locale: &Locale, code: &str) -> String`,
  `pub fn action_label(…)` et `pub fn actor_type_label(…)`.

⛔ **Un code inconnu rend le CODE, et ce n'est pas `format` qui le garantit.** `I18nBundle::format`
(`kesh-i18n/src/loader.rs:110-128`) replie sur le français puis rend **la clé brute** : appelé sur un code
hors liste, il afficherait `audit-log-action-foo-bar`. La fonction **teste d'abord l'appartenance** du
code à la liste, et ne traduit qu'ensuite. Motif du repli sur le code : le journal conserve les codes des
versions antérieures (`journal_entry.updated`, qu'aucun site n'écrit plus) et un code futur ne doit
rien casser.

Tests unitaires dans le module : un code connu rend son libellé dans deux locales ; un code inconnu rend
le code ; **les clés dérivées sont deux à deux distinctes** — deux codes qui ne diffèrent que par `.` et
`_` produiraient la même clé.

**17. `GET /api/v1/audit-log/vocabulary`** — **mêmes refus** que l'AC 6, aucune lecture en base, aucune
entrée d'audit. Réponse :

```json
{ "entityTypes": [{ "code": "account", "label": "Compte" }, …],
  "actions":     [{ "code": "account.archived", "label": "Compte archivé" }, …] }
```

- une entrée par code de `ENTITY_TYPES` et de `ACTIONS`, libellés par les fonctions de l'AC 16, dans la
  langue de l'interface ;
- **ordre des listes : celui des constantes** (par code). Le tri par libellé est l'affaire de l'écran,
  qui reçoit la locale de `/api/v1/i18n/messages` et la conserve (25-1c-b1 AC 3) — un tri d'octets côté serveur rangerait
  « Écriture » après « Utilisateur ».

Motif de la route : l'écran doit proposer des **listes** pour les filtres — un code d'action tapé à la
main n'a plus de sens quand l'écran n'affiche que des libellés —, et elle lui évite de recopier les deux
listes de codes.

**18. La garde des libellés — `crates/kesh-api/tests/audit_label_registry.rs`**, sur le patron de
`tests/audit_route_registry.rs` (lecture de la source, **diff ensembliste**, jamais un compteur — son
en-tête `:15-19` dit pourquoi) :

- (a) **l'ensemble des actions ÉCRITES par le code de production** — les littéraux passés à
  `NewAuditLogEntry::{user, for_actor, api_key, from_current_user}(` dans `crates/*/src`, hors modules
  `#[cfg(test)]` — **est égal** à `ACTIONS`, dans les deux sens : une action ajoutée demain sans libellé
  rougit, un libellé dont l'action a disparu aussi ;
- (b) **même égalité** pour les types d'entité et `ENTITY_TYPES` ;
- (c) **chaque clé dérivée existe dans CHACUN des quatre fichiers `.ftl`**, lus directement — ⚠️ **pas**
  par `all_messages`, qui comble les absences par le français (`loader.rs:130-131`) et rendrait vraie par
  construction une clé manquante en allemand ;
- (d) ⛔ **tout site d'écriture dont l'action ou le type n'est pas résolu, et qui n'est pas inscrit à
  l'inventaire ci-dessous, fait rougir la garde** — c'est ce qui la ferme (`CLAUDE.md` § *Inventorier les
  sites NON RÉSOLUS*). Chaque site de l'inventaire est lui-même **vérifié présent**, et **ses valeurs
  vérifiées dans `ACTIONS`** : un site renommé ne sort pas de l'inventaire en silence ;
- (e) **codes historiques — angle mort assumé** : l'égalité stricte de (a) interdit de libeller un code
  qu'aucun site n'écrit plus (`journal_entry.updated`). Sans données de production (arbitrage 2), un tel
  code s'affiche en code brut (AC 16) et n'est pas proposé dans les filtres de l'écran.

**L'extracteur, et ce qu'il doit savoir lire.**

| constructeur | argument de l'action | argument du type |
|---|---|---|
| `NewAuditLogEntry::user(user_id, action, entity_type, …)` | 2ᵉ | 3ᵉ |
| `NewAuditLogEntry::from_current_user(&user, action, entity_type, …)` | 2ᵉ | 3ᵉ |
| `NewAuditLogEntry::for_actor(user_id, api_key_id, action, entity_type, …)` | 3ᵉ | 4ᵉ |
| `NewAuditLogEntry::api_key(api_key_id, user_id, action, entity_type, …)` | 3ᵉ | 4ᵉ |

- **formes résolues** : le littéral nu, `"…".to_string()` et `"…".into()` — une quarantaine de sites de
  production écrivent `.to_string()`, et un extracteur « littéraux nus » ne rend que 43 actions —, et — ⛔ **sans quoi la garde est
  verte par construction** — une **variable locale liée par une conditionnelle** à deux littéraux
  (`let action = if … { "a" } else { "b" };`), dont les deux branches sont lues ; et une **conditionnelle écrite directement en argument**, sans
  variable intermédiaire — un seul site au 2026-09-15, `kesh-api/src/routes/projects.rs:354-358`
  (`if archived { "project.archived" } else { "project.unarchived" }`), lue de même ;
- ⛔ **une forme se reconnaît sur l'argument ENTIER**, borné aux virgules de profondeur 0 ; un argument qui
  n'y correspond pas entièrement est **non résolu** (d). Motif : un extracteur qui chercherait un littéral
  *dans* l'argument lirait `"project.archived"` seul, fixerait `ACTIONS` à 91, et `project.unarchived`
  sortirait en code brut **sans que rien ne rougisse** ;
- **commentaires masqués** avant lecture (patron `strip_line_comments` d'`admin_pat_denied_e2e.rs`) : le
  doc-comment `email_templates.rs:32` contient `NewAuditLogEntry::for_actor(…)` et serait lu comme un appel ;
- **modules `#[cfg(test)] mod … { }` exclus** par appariement d'accolades — ⚠️ **dans les seuls fichiers qui
  contiennent `NewAuditLogEntry::`** : ailleurs, une **accolade isolée dans un littéral**
  (`kesh-core/src/email_template_engine.rs:160`, test `"Texte { non fermé"`) déséquilibre le compte — un
  appariement naïf ne distingue pas une accolade de chaîne d'une accolade de code ;
- une action **sans point** reste une action : `admin_break_glass_reset` (`auth/bootstrap.rs:289`).

**L'inventaire des sites indirects, au 2026-09-15** (établi par extraction positionnelle en revalidation R1) :

| site | forme | valeurs |
|---|---|---|
| `kesh-api/src/routes/users.rs:310-318` | variable conditionnelle | `user.role_changed`, `user.updated` |
| `kesh-api/src/routes/reconciliation.rs:~1575-1583` | variable passée à `for_actor` | `invoice.paid`, `invoice.partially_settled` |
| `kesh-db/src/repositories/invoice_settlements_write.rs:248-256` | variable conditionnelle | `invoice.paid`, `invoice.partially_settled` |
| `kesh-db/src/repositories/invoices.rs:2019-2031` | variable conditionnelle, `.to_string()` | `invoice.dunning_paused`, `invoice.dunning_resumed` |
| `kesh-db/src/repositories/fiscal_years.rs:126` — helper `build_audit_entry(user_id, action, …)`, appelants `:191`, `:316`, `:412`, `:736`, `:868` | action en **2ᵉ argument du helper** | `fiscal_year.created`, `fiscal_year.updated`, `fiscal_year.closed`, `fiscal_year.reopened` |
| `kesh-api/src/audit.rs:36-52` — `from_current_user` | helper générique, qui transmet | les valeurs de ses appelants, lus directement |

⚠️ **La première rédaction comptait 82 actions** avec un script par littéraux : les **dix** actions de cet
inventaire lui échappaient, dont `invoice.paid` — l'action comptable la plus lue du journal. Elle déclarait
aussi `email_templates.rs` « sans littéral », à tort (`:217`, `:421`). **92 actions et 28 types** sont
les nombres de la revalidation ; c'est la garde qui les fixe.

### Volet E — ce que la story rend faux

**19. Le manuel d'administration** — `docs/manual/fr/admin-manual.tex:1786` affirme depuis la
25-1c-zero : *« le journal ne se **consulte** pas encore : aucune route ni aucun écran ne permet de le
lire depuis l'application »*. **La moitié devient fausse.** Réécrire : une route de consultation et
d'export existe pour le Comptable et l'Admin, l'écran reste à venir (#378).

**20. La conséquence de l'arbitrage 2, écrite dans le même manuel** : **l'import sert à restaurer la
même installation**. Importer la sauvegarde d'une autre installation est techniquement possible, et
ferait attribuer les entrées d'audit locales à la société importée. **Trois sites**, et le premier
contredit aujourd'hui la prémisse :

- `admin-manual.tex:1585` — ⛔ la sous-section s'intitule **« Importer (restauration / migration) »**.
  « Migration » laisse entendre qu'on importe vers une **autre** installation. Dire ce que le mot
  couvre — déplacer **la même** installation vers un autre serveur — ou le retirer ; ne pas laisser
  le titre promettre l'usage que l'arbitrage exclut ;
- `admin-manual.tex:1590` — l'avertissement « Opération destructrice et irréversible », cinq lignes
  sous ce titre, renvoie aux identifiants de « l'**instance importée** » et dit que « les comptes de
  l'**installation précédente** n'existent plus » : c'est la description d'un import venu d'**une autre**
  installation. À reformuler pour la restauration de la même ;
- `admin-manual.tex:1803-1810` — l'encadré « Deux réserves à connaître avant d'invoquer cette
  conformité » affirme que les entrées de l'archive sont « **fusionnées** » avec celles de
  l'installation : c'est là que la conséquence d'un import étranger doit être écrite.

**21. Régénération et contrôle** : `make fr` dans `docs/manual/`, PDF commités, **PDF aplati vérifié**
(`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) — l'ancienne phrase absente, les nouvelles présentes.
⛔ **En LaTeX le souligné s'écrit `\_`** : greper `audit(\\)?_log`, jamais `audit_log`.

⚠️ **Ce qui reste VRAI et ne doit pas être touché** : `user-manual.tex:498-503` (« ce qui manque encore,
c'est la consultation du journal d'audit **depuis l'application** ») et la feuille de route du `README.md`, lignes des epics 24 et 25 (« consultable
par aucun **écran** ») — c'est la 25-1c-b2 qui les rendra faux.

### Volet F — les tests

**22. Repository** — `#[sqlx::test(migrations = "./test-schema")]` dans `repositories/audit_log.rs`,
**identifiants désalignés** (sociétés 30 et 40, utilisateurs 501 et 602), sur le montage `seed_actor`
déjà présent (25-1c-zero). ⚠️ Ce montage ne crée que l'utilisateur 501 de la société 40 : **un helper
distinct** ajoute l'utilisateur 602 dans la société 30, **sans modifier `seed_actor`**, dont dépendent les
tests de la 25-1c-zero. Et `MAX_LIMIT` n'existe pas encore dans ce module : le **fixer à 200**, la borne
de la route (AC 7), faute de quoi le clamp de (f) est invérifiable :

- (a) **scoping** : une entrée de la société 30 n'apparaît **jamais** dans la consultation de la 40 ;
- (b) **filtre strict** : une entrée `company_id IS NULL` n'apparaît **ni** dans la consultation de la
  40, **ni** dans celle de la 30, **ni** dans `list_for_export` — posée par `UPDATE` explicite après
  insertion ;
- (c) **bornes de date** : entrée à `date_to 23:59:59.999` **incluse**, entrée au lendemain
  `00:00:00.000` **exclue**, entrée à `date_from 00:00:00.000` **incluse**, entrée de la **veille** à `23:59:59.999` **exclue**, et au
  **dernier jour possible** — `date_to = 9999-12-31` — une entrée à `9999-12-31 23:59:59.999` **incluse** — dates posées par `UPDATE`
  explicite ;
- (d) filtres `entity_type`, `entity_id`, `action`, chacun seul ;
- (e) **ordre** `created_at DESC, id DESC`, dont deux entrées de même `created_at` ;
- (f) **pagination** : `total` indépendant de `limit`/`offset`, page 2 correcte, clamp ;
- (g) `list_for_export` rend **exactement** les lignes de la consultation non paginée, dans le même
  ordre, et respecte `max_rows`.

**23. API** — nouveau `crates/kesh-api/tests/audit_log_e2e.rs`, `#[sqlx::test(migrations =
"../kesh-db/test-schema")]`, patron `spawn_app` / `seed_role` d'`admin_full_import_e2e.rs` :

- ⛔ **sur les TROIS routes** : Comptable ⇒ 200, Admin ⇒ 200, **Consultation ⇒ 403**, sans jeton ⇒ 401 ;
  **clé API `read` d'un Admin ⇒ 403 `API_KEY_MANAGEMENT_FORBIDDEN`** — l'export et le vocabulaire sont
  des chemins d'extraction comme la liste ;
- 400 `VALIDATION_ERROR` : `dateFrom` invalide, `dateFrom > dateTo`, `entityId` sans `entityType`,
  `entityId <= 0`, `action` de 65 caractères, `dateTo=%2B262142-12-31`, **`dateFrom=%2B262142-12-31`**, **`dateFrom=-0001-01-01`**,
  **`dateFrom=0999-12-31`**, **`dateTo=0999-12-31`** — ⛔ le `+` **encodé `%2B`**, ou les paramètres passés
  par `.query(&[…])` : écrit en clair, `form_urlencoded` le décode en **espace**, la valeur échoue au
  **format** et le test ne sonde plus la **plage**. L'assertion vérifie que le message nomme **la
  plage**, pas le format ; et
  **`dateTo=9999-12-31` accepté**
  (200, **avec des `items`** : une entrée datée du `9999-12-31` est rendue — un 200 à liste vide
  masquerait exactement le défaut que la borne inclusive supprime) ;
- **forme de la réponse** : `actorType` vaut **`"user"`** (jamais `"User"`), `createdAt` finit par `Z`,
  **aucun champ `companyId`** ;
- ⛔ **langue** : l'application montée **en `de-CH`** (`Config::with_locale`, `config.rs:524` ; patrons
  `spawn_app_with_locale`, `i18n_e2e.rs:65`, ou `spawn_app_with`, `admin_full_export_e2e.rs:63`) **et**
  la société en `accounting_language` **française** — pour qu'une implémentation qui lirait la langue
  comptable rougisse. Une entrée `invoice.validated` sur `invoice` rend `actionLabel` et
  `entityTypeLabel` **allemands** ; le vocabulaire et les en-têtes du CSV aussi ; le `Content-Disposition`
  porte **`filename*=UTF-8'de-CH'`** ; et le message du refus `RESULT_TOO_LARGE`, dans la même application,
  est **allemand** (AC 11) ;
- ⛔ **code inconnu** : une entrée d'action `zz.unknown_code` et de type `zz_unknown` rend, dans la liste
  **et** dans le CSV, **le code** — et non une clé `audit-log-action-…` ;
- **vocabulaire** : autant d'entrées que `ACTIONS.len()` et `ENTITY_TYPES.len()` (lus depuis le module,
  jamais recopiés), dans l'ordre des constantes, libellés traduits ;
- **export** : `Content-Type`, `Content-Disposition` (`filename*=`), **BOM**, séparateur `;`, dix en-têtes,
  une ligne par entrée, cellules action, type et type d'auteur **traduites** ;
- ⛔ **injection** : un `actor_label` posé à `=HYPERLINK("x")` sort préfixé d'une apostrophe. ⚠️
  **Relire le fichier par `csv::ReaderBuilder` (séparateur `;`) et comparer la CELLULE exacte**
  `'=HYPERLINK("x")` : le writer double les guillemets (`"'=HYPERLINK(""x"")"`), si bien qu'une
  assertion par sous-chaîne rougit à tort, et qu'une assertion négative reste verte sous la mutation ;
- **`RESULT_TOO_LARGE`** : 10 001 entrées semées par **un seul** `INSERT … SELECT` (pas 10 001
  appels) ⇒ 400 avec ce code ;
- ⛔ **aucune écriture** : le nombre de lignes d'`audit_log` est **identique** avant et après une
  consultation, un appel au vocabulaire et un export réussi.

**24. Épreuve par mutation, résultats OBSERVÉS consignés** — chaque mutation appliquée, le test visé
**vu rouge sur assertion**, puis fichier restauré et `cmp` vérifié :

| mutation | test attendu rouge |
|---|---|
| `company_id = ?` → `(company_id = ? OR company_id IS NULL)` | 22 (b) |
| borne haute `date_to 23:59:59.999` → `date_to 00:00:00.000` | 22 (c) — l'entrée de `23:59:59.999` disparaît |
| borne basse `created_at >= ?` → `created_at > ?` | 22 (c) — l'entrée de `date_from 00:00:00.000` disparaît |
| `ensure_not_pat` retiré de la route de vocabulaire | 23, clé API |
| route d'export montée dans `authenticated_routes` au lieu de `comptable_routes` | 23, Consultation sur l'export |
| `csv_sanitize` retirée de la cellule `actor_label` | 23, injection |
| `actor_type.as_str()` → sérialisation serde de l'enum | 23, forme |
| test d'appartenance retiré d'`action_label` (appel direct à `format`) | 23, code inconnu ; 16, test unitaire |
| langue lue dans `accounting_language` au lieu de `state.config.locale` | 23, langue |
| tag du `Content-Disposition` produit par `map_language_to_bcp47` | 23, langue — `filename*=UTF-8'de-CH'` |
| message `RESULT_TOO_LARGE` formé dans `accounting_language` | 23, langue — message du refus |
| un site de l'inventaire retiré de la table de la garde | 18 (d) |
| résolution des variables conditionnelles retirée de l'extracteur | 18 (a) — les dix actions indirectes deviennent des libellés « sans site » |
| cellule action écrite avec le code au lieu du libellé | 23, export |
| une entrée retirée de `ACTIONS` | 18 (a) |
| une clé `audit-log-action-*` retirée du seul `de-CH` | 18 (c) |

⛔ **Un test qui ne compile pas ne rougit pas : il se tait.** Chaque mutation doit produire un échec
d'**assertion**.

**25. Les gardes existantes restent vertes sans être modifiées** — à vérifier, pas à supposer :
`admin_pat_denied_e2e.rs` (les routes ne sont **pas** dans le bloc admin), `audit_route_registry.rs` (il ne
recense que les verbes mutants, `:181,311`), les tests de parité i18n, et les gardes i18n du frontend
(AC 15).

## Tasks / Subtasks

- [x] **T0 — Rebase** sur `main` **après** le merge de la PR #437 (ouverte au 2026-09-15 : ne pas
      implémenter avant), par SHA (cf. en-tête), puis remise à zéro
      **vérifiée** de la base de dev.
      *(2026-09-16 : PR #437 fusionnée en squash `842eaac0` ; `git rebase --onto origin/main 42b6aac0` —
      13 commits rejoués, sans conflit ; base de dev recréée en trois étapes — **40 tables, 1 Admin,
      colonne `company_id` présente**. ⚠️ Le premier essai a échoué : le conteneur répondait au `ping`
      alors qu'il **initialisait encore** son datadir tmpfs — attendre qu'un `SELECT 1` passe, pas un
      ping.)*
- [x] **T1 — Repository** (AC 1-4) et ses tests (AC 22).
      *(2026-09-16 : `MAX_LIMIT = 200`, `AuditLogListQuery`, `AuditLogListResult`, `push_where_clauses`
      partagée, `list_by_company_paginated`, `list_for_export` ; deux doc-comments périmés corrigés —
      `find_by_entity` et l'en-tête d'`entities/audit_log.rs`. **13 tests verts**, dont les 7 neufs, en
      0,43 s : scoping, filtre strict, bornes inclusives à la milliseconde jusqu'au 9999-12-31, filtres,
      ordre à égalité de milliseconde, pagination et clamp, parité export/consultation.)*
- [x] **T2 — Extraction de `csv_sanitize`** (AC 13), tests de l'échéancier verts avant d'aller plus loin.
      *(2026-09-16 : fonction déplacée dans `crate::util` en `pub(crate)`, **comportement identique** ;
      `invoices.rs` l'importe et garde un renvoi à sa place. **Quatre tests unitaires écrits** — elle n'en
      avait aucun : un cas par caractère déclencheur, le contournement par espace ou tabulation de tête,
      CR/LF/TAB remplacés, et une chaîne ordinaire intacte. 17 tests d'`util` verts.)*
- [x] **T3 — Module `audit_labels` et garde des libellés** (AC 16, 18) : **la garde d'abord**, qui fixe
      les deux listes par diff contre la source ; puis le module et ses tests unitaires.
      *(2026-09-16 : garde écrite en premier, elle a FIXÉ les listes par extraction positionnelle sur
      **cinq** formes — les quatre constructeurs plus le helper `build_audit_entry` des exercices. Ses
      trois volets structurels étaient **verts dès le premier lancement**, ce qui confirme les 92 actions
      et les 28 types par une extraction indépendante de celle du document arbitré. Inventaire clos de
      **six** sites non résolus, plus deux passe-plats d'`audit.rs` exclus nommément — ils relaient
      l'action de leurs appelants et ne sont l'angle mort de personne. Diff **bilatéral** : le sens
      « liste → code » attraperait une coupe `#[cfg(test)]` devenue trop gourmande. Les `.ftl` sont lus
      **directement**, jamais via `I18nBundle` — `format` replie sur le français et `all_messages`
      comble les trous, si bien qu'une locale vide passerait au vert. Module : test d'appartenance
      **avant** traduction, sans quoi la clé brute s'afficherait. À la livraison de T4 : garde 4/4 et
      module 4/4 ; avant T4, 3/4 de part et d'autre, le rouge portant sur les 122 clés absentes.
      ⚠️ **Depuis la passe 1 de revue, garde 7/7 et module 5/5** — trois tests ajoutés à la garde et un
      au module par la remédiation.)*
- [x] **T4 — Libellés, quatre locales** (AC 15) : les 120 libellés à la spécification, et les 13 autres clés ;
      glossaire partie B si un terme l'exige ; **liste française recopiée au Dev Agent Record**.
      *(2026-09-16 : **133 clés par locale**, recomptées depuis les fichiers et non estimées —
      ventilation `grep -c` sur `fr-CH` : 28 `entity` + 92 `action` + 2 `actor-type` + 10 `csv-header`
      + 1 `export-error` = 133, et les quatre locales rendent le même total. Français **recopié du
      document arbitré** `25-1c-a-libelles-proposes.md`, sans reformulation. Les trois langues cibles
      sont **relevées** au catalogue, jamais inventées : deux balayages ont extrait les participes et
      les noms d'entité attestés (`erstellt`/`creato`, `storniert`/`annullata`, `archiviert`,
      `validiert`/`convalidata`, `widerrufen`/`revocata`, `verworfen`/`scartata`, `Mahnstufe`,
      `API-Schlüssel`, `Bankprofil`, `Zuordnungsregel`…), et les 11 clés d'export **décalquent**
      `echeancier-csv-header-*` et `echeancier-export-error-too-large`, dont seul l'exemple de filtre
      change. Verts à la livraison de T4 : garde 4/4, module 4/4 *(7/7 et 5/5 depuis la passe 1 de
      revue)*, `kesh-i18n` 29/29 dont `parity_between_locales`,
      `lint-i18n-ownership` PASS, frontend 740/740.)*
- [x] **T5 — Routes de consultation et de vocabulaire** (AC 5-9, 17) : module, DTO, validation partagée,
      montage.
      *(2026-09-16 : `routes/audit_log.rs` créé — `ListAuditLogQuery`, `AuditLogEntryResponse`,
      `VocabularyResponse`, et **une seule** fonction `construire_requete` que la liste et l'export
      partageront : deux validations séparées dériveraient, et l'export cesserait de montrer les lignes
      de l'écran. `ensure_not_pat` est la **première instruction** de chaque handler, pas une couche de
      bloc — `comptable_routes` n'a pas de garde anti-clé et le garde-fou d'`admin_pat_denied_e2e`
      interdit un second consommateur de `require_not_pat`. Bornes de date refusées **aux deux côtés**
      (`1000-01-01` … `9999-12-31`) : une année négative se parse sans erreur et fait **paniquer** la
      liaison, ce qu'aucune couche ne rattrape. Montées dans `comptable_routes`, module déclaré à son
      rang alphabétique. `cargo check` et `clippy -D warnings` verts. ⚠️ **Non testée ici** : les tests
      d'API sont la tâche T7, et cette note ne prétend pas le contraire.)*
- [x] **T6 — Export CSV** (AC 10-14).
      *(2026-09-16 : `export_audit_log_csv` — **même** fonction de validation que la liste, donc mêmes
      lignes à l'écran et au fichier ; `offset` et `limit` **ignorés sans être refusés**, un export
      n'étant pas paginé. Plafond demandé à `MAX_EXPORT_ROWS + 1` : c'est ce `+1` qui distingue
      « exactement le plafond » de « au-delà », et le refus est un `RESULT_TOO_LARGE` **nommé** plutôt
      qu'un fichier tronqué en silence. Dix colonnes, en-têtes traduits avec **repli français** —
      `format` rend la clé brute quand elle manque, l'en-tête afficherait sinon
      `audit-log-csv-header-id`. ⛔ Toute cellule texte passe par `csv_sanitize` : `actor_label` est
      choisi par un utilisateur et le repli d'un libellé est un **code lu en base**. `Content-Disposition`
      par `build_content_disposition` avec `locale.dir_name()` — ⚠️ **pas** `map_language_to_bcp47`, qui
      attend une langue comptable et replie silencieusement sur `fr-CH`. ⚠️ Le site voisin
      (`invoices.rs:1306`) construit cet en-tête **à la main** ; la spec impose le helper, c'est lui qui
      est suivi. `clippy -D warnings` et `fmt` verts. ⚠️ **Non testé ici** — tests d'API en T7.)*
- [x] **T7 — Tests API** (AC 23). ⛔ *Une tâche qui décrit un test est une promesse ; la cocher sans
      l'avoir écrit la transforme en mensonge.*
      *(2026-09-16 : `tests/audit_log_e2e.rs`, **15 tests, 15 verts** en 4,6 s. L'application est montée
      **en `de-CH`** pendant que la société est en `accounting_language` **française** : c'est le seul
      dispositif qui fasse rougir une lecture de la mauvaise langue — les deux langues concordant, le
      test passerait au vert sur un code faux. Les refus sont éprouvés **sur les trois routes** en
      boucle : 401 sans jeton, 403 pour Consultation, 403 `API_KEY_MANAGEMENT_FORBIDDEN` pour une clé
      `read` d'**Admin** — le cas le plus favorable au refus qu'on veut prouver. Les dix cas de 400
      passent le `+` **encodé `%2B`** : écrit en clair, `form_urlencoded` le décode en espace, la valeur
      échoue au **format** et le test ne sonde plus la **plage** ; un test distinct vérifie que le
      message nomme la plage. La borne `9999-12-31` est acceptée **avec des `items`** — un 200 à liste
      vide masquerait exactement le défaut que la borne inclusive supprime. L'injection est relue par
      `csv::ReaderBuilder` et comparée **cellule à cellule** (`'=HYPERLINK("x")`) : le writer double les
      guillemets, une assertion par sous-chaîne rougirait à tort et une assertion négative resterait
      verte sous la mutation. Les 10 001 entrées du plafond sont semées par **un seul**
      `INSERT … SELECT FROM seq_1_to_10001`. Les longueurs du vocabulaire sont **lues du module**,
      jamais recopiées.)*
- [x] **T8 — Mutations** (AC 24), résultats observés.
      *(2026-09-16 : **neuf mutations, neuf rouges sur assertion** — aucune n'a échoué à la compilation,
      ce qui ne prouverait rien. Chacune appliquée **seule**, le fichier restauré et l'identité vérifiée
      par `git diff --quiet` après chaque tour ; arbre propre à la fin.)*

      | mutation | test vu rouge | ligne de l'assertion |
      |---|---|---|
      | `WHERE company_id = ?` → `(company_id IS NULL OR company_id = ?)` | `list_excludes_entries_without_a_company` | `audit_log.rs:658` |
      | borne haute `23:59:59.999` → `00:00:00.000` | `list_date_bounds_are_inclusive_to_the_millisecond` | `audit_log.rs:714` |
      | borne basse `created_at >=` → `created_at >` | idem | `audit_log.rs:710` *(assertion **différente** de la précédente — les deux bornes sont éprouvées séparément)* |
      | `ensure_not_pat` retiré du **vocabulaire** | `une_cle_api_est_refusee_sur_les_trois_routes` | `audit_log_e2e.rs:317` |
      | export monté dans `authenticated_routes` | `le_role_consultation_est_refuse_sur_les_trois_routes` | `:300` |
      | `csv_sanitize` retirée de la cellule `actor_label` | `l_export_neutralise_une_formule_dans_le_nom_de_l_auteur` | `:629` |
      | `actor_type.as_str()` → sérialisation de l'enum | `la_reponse_porte_le_code_de_base_et_jamais_la_societe` | `:447` |
      | test d'appartenance retiré d'`action_label` | `un_code_inconnu_sort_en_code_dans_la_liste_et_dans_le_csv` | `:494` |
      | idem, **rejouée** pour le test unitaire | `un_code_inconnu_rend_le_code_et_jamais_la_cle` | `audit_labels.rs:299` |
      | langue lue ailleurs que `state.config.locale` | `les_libelles_suivent_la_langue_de_l_interface_et_non_la_comptable` | `:476` |

      ⚠️ **Défaut de ma propre épreuve, relevé et corrigé.** La mutation du test d'appartenance visait
      **deux** tests ; je les avais chaînés par `&&`. Le premier ayant rougi, le `&&` a court-circuité et
      le test **unitaire n'a jamais tourné sous mutation** — la mutation aurait été déclarée complète
      sans l'être. Rejouée seule *(ligne « idem, rejouée »)* : elle rougit avec le message attendu,
      « le repli a laissé fuir une clé brute : `audit-log-action-parti-en-vacances` ». *Un enchaînement
      de commandes qui court-circuite transforme une épreuve en affirmation.*

      ⛔ **ÉCART DÉCLARÉ, relevé par la passe 1 de revue : l'AC 24 prescrit SEIZE mutations, neuf
      avaient été jouées, et le total « neuf mutations, neuf rouges » se lisait comme complet.** Ma
      fenêtre de lecture du tableau s'était arrêtée à la ligne 541 et j'avais cru la table finie ; elle
      court jusqu'à 548. *Un décompte qui ne dit pas son dénominateur n'est pas un décompte.*
      ⇒ **16 prescrites · 15 jouées · 1 sans objet, motivée** :

      | mutation (AC 24) | résultat |
      |---|---|
      | tag `Content-Disposition` replié sur `fr-CH` | ✅ rouge — `l_export_porte_son_bom…:568` |
      | message `RESULT_TOO_LARGE` formé en français | ✅ rouge — `au_dela_du_plafond…:660` |
      | un site retiré de l'inventaire de la garde | ✅ rouge — `tout_site_dont…:305` |
      | résolution des conditionnelles retirée de l'extracteur | ⚪ **SANS OBJET** — mon extracteur ne les résout **pas** : c'est l'inventaire qui les porte (cf. l'écart d'AC 18 ci-dessous). Il n'y a donc aucune résolution à retirer, et la propriété visée est éprouvée par la mutation précédente |
      | cellule action écrite avec le code au lieu du libellé | ✅ rouge — `l_export_porte_son_bom…:591` |
      | une entrée retirée d'`ACTIONS` | ✅ rouge — `les_actions_de_la_liste…:398` |
      | une clé retirée du seul `de-CH` | ✅ rouge — `chaque_code_a_son_libelle…:471` |

      ⚠️ **Les trois dernières ont été REJOUÉES contre la garde corrigée** : leur premier passage avait
      tourné contre la version d'origine (cf. l'incident ci-dessous).

      ⛔ **Incident de méthode, déclaré plutôt que tu.** Le script de mutation restaure ses cibles par
      `git checkout`. Lancé sur des correctifs **non commités**, il les a **effacés** — les six éditions
      de la remédiation ont dû être réécrites. *Commiter avant toute épreuve qui restaure par git.*
      ⚠️ Et le `git status` final le disait déjà : seuls les fichiers de documentation y figuraient,
      les fichiers Rust étant redevenus « propres ». Je l'ai lu sans le comprendre.

      ⚠️ Deux sites étaient **ambigus** — `ensure_not_pat` et `let locale = state.config.locale`
      apparaissent chacun **trois** fois, une par handler. La mutation porte donc sur un **numéro de
      ligne** (464 pour le vocabulaire, 440 pour la liste) : muter par texte aurait touché le mauvais
      handler, ou les trois, et le rouge obtenu n'aurait pas prouvé ce qu'il prétend.
- [x] **T9 — Manuel d'administration** (AC 19-21), PDF régénéré et vérifié aplati.
      *(2026-09-16 : **quatre** corrections. AC 19 — la phrase des champs disait « le journal ne se
      consulte pas encore : aucune route ni aucun écran » ; elle dit désormais que la consultation
      existe par l'API (liste, vocabulaire, export), réservée au Comptable et à l'Admin, refusée aux
      clés, **l'écran restant à venir**. AC 20, les trois sites : le titre « Importer (restauration /
      **migration**) » devient « Importer (**restaurer la même installation**) » et dit ce que le mot
      couvrait ; l'avertissement ne parle plus des « identifiants de l'**instance importée** » mais de
      ceux « tels qu'ils étaient au moment de la sauvegarde » ; et l'encadré OLICo porte la conséquence
      d'un import étranger — les entrées locales seraient rattachées à la société importée, **sans
      qu'aucun message ne le signale**. PDF régénéré sous `mem-guard` (67 pages) et **contrôlé aplati** :
      ancienne phrase absente, huit phrases neuves présentes.)*

      ⚠️ **Mon premier contrôle a rendu trois faux négatifs**, et la cause valait d'être établie plutôt
      que corrigée à l'aveugle : les trois phrases « absentes » étaient les trois qui contiennent une
      **apostrophe**. LaTeX rend `'` par `’` (U+2019), qu'un `grep -F` sur l'apostrophe droite ne trouve
      jamais. Le contrôle normalise désormais les apostrophes avant de chercher. *C'était le détecteur
      qui était faux, pas le manuel — et un détecteur mal formé coûte le même diagnostic qu'un défaut
      réel.*

      ⚠️ **Relevé sans être traité** : les `⚠️` du manuel ne sont **pas rendus** (`Missing character:
      There is no ⚠ (U+26A0)`), de même que `✓`, `✗` et `═`. C'est **antérieur à cette story** et hors
      de son périmètre ; le seul que j'allais ajouter a été écrit `\faExclamationTriangle`, la commande
      que ce fichier emploie déjà.
- [x] **T10 — Propagation** : `grep -rniE "aucune route|post-MVP|story 3\.5" crates docs/manual/fr/*.tex --exclude-dir=migrations`
      (⛔ **insensible à la casse**, sinon « Story 3.5 » échappe ; et **sans** `migrations/` — une
      migration appliquée ne se modifie plus, P8)
      et le symptôme « la consultation n'existe pas » sur le code, les manuels, le README et `website/` ;
      trier ce qui devient faux (**cette** story) de ce qui reste vrai jusqu'à la 25-1c-b2.
      *(2026-09-16 : grep prescrit exécuté ⇒ **20 occurrences, 0 résidu**. Le tri, site par site : les
      « aucune route » parlent des **PAT et de l'administration** (`admin_pat_denied_e2e`, `lib.rs:326`,
      `errors.rs:192`), de l'absence d'**update générique de société** (`companies.rs:98,232`), de la
      **non-réactivation** (`backfill.rs:169`) ou du registre lui-même (`audit_route_registry.rs:288,325`)
      — aucune ne parle de consulter le journal ; les « story 3.5 » sont les références **historiques** à
      l'origine du journal d'audit (`journal_entries.rs`, `fiscal_years.rs`, `accounts.rs`), exactes et à
      conserver ; l'unique « post-MVP » vise le **format d'erreur d'Axum** (`journal_entries.rs:279`),
      sans rapport. Second volet — le symptôme « la consultation n'existe pas » grepé sur tout le dépôt
      (`.tex`, `.md`, `.rs`, `.html`, `.svelte`, `.ts`, `website/` compris) ⇒ **deux** occurrences, toutes
      deux à CONSERVER : `README.md:218` dit « consultable par aucun **écran** », ce qui **reste vrai**
      jusqu'à la 25-1c-b2, et la spec de cette story cite l'ancienne phrase pour la corriger. ⚠️ Vérifié
      au `git diff --stat` : `user-manual.tex`, `README.md` et `website/` sont **intacts**.)*

      ⚠️ **Ce dernier constat ne vaut que pour T10, et la CLÔTURE l'a dépassé** — sur instruction du
      Project Lead (« mettre à jour la doc »), qui a du même coup tranché l'arbitrage laissé ouvert à la
      passe 3. Quatre sites de plus ont été traités :

      | site | ce qui était devenu faux |
      |---|---|
      | `docs/manual/fr/user-manual.tex:497` | « vous ne pouvez pas encore **produire** l'historique » — or l'export CSV le produit. Réécrit pour distinguer l'**API**, qui existe, de l'**écran**, qui manque. ⚠️ Le site voisin `:1613` dit « écran dédié » : il **reste vrai**, il n'a pas été touché |
      | `README.md`, ligne v0.12.1 | « la consultation depuis l'application, **qui n'existe toujours pas** » — la route existe désormais |
      | `CHANGELOG.md` | aucune section `[Unreleased]` n'existait, alors que le manuel d'administration y **renvoie** ; créée, avec l'entrée de cette story |
      | `website/` | vérifié : **aucune** de ses mentions du journal ne promet de consultation — rien à corriger |

      PDF régénérés et **contrôlés aplatis** (apostrophes normalisées) : l'ancienne phrase absente, les
      trois neuves présentes, « écran dédié » intact.
- [x] **T11 — Gates** : backend complet via `scripts/test-fast.sh`, base remise à zéro **et vérifiée** ;
      frontend : `npm run test:unit` (gardes i18n, AC 15) ; **E2E complète avant le push**, `kesh_e2e`
      reconstruite.
      *(2026-09-16 : les quatre volets ont tourné. **Aucune régression imputable à la story** — les neuf
      échecs E2E sont tous qualifiés un par un ci-dessous.)*

      | volet | résultat observé |
      |---|---|
      | base de gate | remise à zéro **inconditionnelle** puis **vérifiée** : `SELECT 1` réel après 4 s — jamais un `ping`, le datadir tmpfs s'initialise encore —, migrations rejouées, seed appliqué, **40 tables, 1 Admin** |
      | backend (`scripts/test-fast.sh`) | ✅ **2365 tests, 2365 passés**, 4 ignorés, 86,7 s — `fmt` et `clippy -D warnings` compris |
      | frontend | ✅ `check` **0 erreur** (27 avertissements, tous **préexistants** — aucun de mes fichiers) · `lint-i18n-ownership` PASS · `test:unit` **740/740** · `build` ✔ |
      | E2E Playwright | ✅ **214 passés / 9 échoués / 19 ignorés** en 8,3 min — `kesh_e2e` **recréée** (DROP/CREATE + migrations, 40 tables) |

      **Les neuf échecs, qualifiés un par un** — un rouge ne se juge pas au nombre :

      | échec | verdict |
      |---|---|
      | `mode-expert:26` et `:41`, `onboarding-path-b:65` et `:92`, `onboarding:57`, `:77`, `:150` | **les 7 fixes de la KF-029 (#97)**, au complet |
      | `invoices.spec.ts:405` et `:429` | **absents, et c'est correct** : la KF-045 ne rougit qu'**avant 12:00 UTC** ; le run a tourné à 14:34 UTC. Leur présence aurait été une régression |
      | `sidebar-navigation:75` | **pollution d'état** — rejoué **seul**, il **passe** (2,0 s). Ce n'est donc pas la KF-046, qui échoue rejouée seule. La liste nominative donne deux causes sur ce test ; seul le rejeu tranche |
      | `inbox-import.spec.ts:106` | ⛔ **défaut de MON montage, pas du code** — voir ci-dessous |

      ⛔ **Le seul échec hors liste venait de ma propre recette, et le dépôt l'avait documenté.**
      `inbox-import:106` échouait **rejoué seul** — donc pas de la pollution. Diagnostic **vérifié au
      log**, non supposé : `internal: inbox import: racine inbox: Permission denied (os error 13)`.
      `docs/testing.md:198-207` décrit ce symptôme mot pour mot — sans `KESH_INBOX_DIR` ni
      `KESH_DOCUMENTS_DIR`, le fichier rend « 1 échec et 2 tests skippés » et Playwright ne trouve pas
      `getByTestId('inbox-import-report')`. J'avais monté le backend depuis la recette du `CLAUDE.md`
      **sans lire la recette complète** de `docs/testing.md`. Backend relancé avec les deux variables,
      spec rejouée **dans les conditions exactes du run complet** ⇒ **passe en 1,5 s**, et plus aucune
      erreur au log. *C'est la faute que ce dépôt documente déjà pour `KESH_COOKIE_SECURE` : chercher la
      doc du dépôt AVANT de diagnostiquer. Elle s'est reproduite sur une autre variable.*

      ⚠️ **Le rouge attendu se juge fichier par fichier, jamais au nombre.** Run lancé à **14:34 UTC**,
      donc **après 12:00 UTC** : les deux échecs KF-045 (`invoices.spec.ts:405` et `:429`) **ne doivent
      pas** apparaître — ils ne rougissent que le matin. Attendu ici : les **7 fixes** de la KF-029
      (`mode-expert:26,41`, `onboarding-path-b:65,92`, `onboarding:57,77,150`), plus éventuellement
      `sidebar-navigation:75` (KF-046) et **1 à 2 échecs de pollution à identité variable**. Tout test
      hors de cette liste sera **rejoué seul** : sur `sidebar-navigation:75`, la liste donne deux causes
      possibles et **seul le rejeu isolé tranche** — la KF-046 échoue seule, la pollution passe seule.

## Dev Notes

### Ce que la story touche

| fichier | nature |
|---|---|
| `crates/kesh-db/src/repositories/audit_log.rs` | `AuditLogListQuery`, `AuditLogListResult`, `push_where_clauses`, deux lectures, doc de `find_by_entity`, tests AC 22 |
| `crates/kesh-api/src/routes/audit_log.rs` | **nouveau** : DTO, validation, trois handlers |
| `crates/kesh-api/src/audit_labels.rs` | **nouveau** : les deux listes et les trois fonctions de libellé (AC 16) |
| `crates/kesh-api/src/routes/mod.rs`, `src/lib.rs` | `pub mod audit_log;` (routes), **`pub mod audit_labels;`** (lib) ; trois `.route(…)` dans `comptable_routes`, **avant** `:658` |
| `crates/kesh-api/src/util.rs` | `csv_sanitize` déplacée |
| `crates/kesh-api/src/routes/invoices.rs` | `csv_sanitize` importée, plus définie |
| `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` | 133 clés par locale à la spécification (AC 15) |
| `docs/i18n-glossaire.md` | partie B, si un terme des libellés l'exige |
| `crates/kesh-api/tests/audit_log_e2e.rs`, `tests/audit_label_registry.rs` | **nouveaux** (AC 18, 23) |
| `docs/manual/fr/admin-manual.tex` + PDF | AC 19-21 |

**Trois crates, le glossaire et un manuel, aucun frontend** — sous le seuil de splitting. ⚠️ **Le volume
change de nature** : 133 clés × 4 langues, dont 120 libellés à rédiger. C'est du texte, gardé par une
seule garde ensembliste, et non de la logique ; le Project Lead a refusé de le sortir en story séparée
(arbitrage 3).

### Faits établis à la spécification, et non supposés

- **Aucune lecture scopée n'existe** : `find_by_entity` (`audit_log.rs:140-158`) n'est ni scopée ni
  paginée, et n'est appelée par **aucune** route (`crates/kesh-api/src` : 0 appel).
- **Fuseau** : UTC — `@@system_time_zone = UTC`, `NOW()` = `UTC_TIMESTAMP()` ; et surtout, **sqlx impose
  `time_zone='+00:00'` à chaque session** (`sqlx-mysql-0.8.6/src/options/mod.rs:112`).
- **Langue** : aucune langue par utilisateur ; `KESH_LANG` → `config.locale` (`config.rs:831-832`), servie
  à l'écran par `routes/i18n.rs:21-22`. `companies.instance_language` existe (« langue de l'interface »,
  `entities/company.rs:164-165`) mais **aucune route ne la lit** : la langue réellement affichée est
  celle de la configuration. `accounting_language` n'est lue par les rapports que pour **étiqueter** le
  nom du fichier (`reports.rs:1228-1233`).
- **`I18nBundle::format`** rend la clé brute pour une clé absente partout, après repli sur le français
  (`loader.rs:110-128`) ; `all_messages` comble les absences par le français (`:130-131`).
- **Vocabulaire** : **92 actions** et **28 types d'entité**, comptés par extraction **positionnelle** en
  revalidation R1 : 82 résolues directement (dont `admin_break_glass_reset`, sans point), et **10** portées
  par des variables conditionnelles ou par le helper `build_audit_entry` de `fiscal_years.rs` (inventaire de
  l'AC 18). La première rédaction en comptait 82 : un script « par littéraux » ne voit pas les dix autres.
- **Langue du nom de fichier** : `Locale::dir_name()` rend `de-CH` (`kesh-i18n/src/lib.rs:33`) ;
  `util::map_language_to_bcp47` attend `"FR"`/`"DE"`/`"IT"`/`"EN"` (`util.rs:170`).
- **Visibilité** : `lib.rs` déclare `pub mod audit;` et `pub(crate) mod util;` (`:11`, `:24`) — un test
  d'intégration ne voit que le premier genre.
- **`ActorType`** sérialise `"User"`/`"ApiKey"` ; la base stocke `user`/`api_key` (`as_str`).
- **Clé API** : une clé `read` passe tout GET hors bloc admin (`middleware/auth.rs:141-143,172-177`) —
  d'où `ensure_not_pat`.
- **`admin_pat_denied_e2e.rs`** porte un tuple codé en dur `(5, 22)` (`:814-819`) pour le **bloc admin**
  seulement ; une route de `comptable_routes` ne le touche pas.
- **Aucun test ne compte les routes** de `comptable_routes`.
- **Échappement anti-formule** : seul `invoices.rs:1176` en a un.
- **Index** : `idx_audit_log_company_date (company_id, created_at)` sert le filtre principal ; aucun
  index sur `action` — le volume d'une PME ne le justifie pas, et `EXPLAIN` le montrera au dev si le
  doute naît.
- **Irrégularités des codes**, que la route **ne corrige pas** : types d'entité au pluriel
  (`reconciliation_rules`, `bank_imports`, `bank_profiles`), préfixe d'action ≠ type (`books.*` →
  `company`, `reconciliation.*` → `bank_transaction`), `admin_break_glass_reset` sans point,
  `journal_entry.updated` historique qu'aucun site n'écrit plus. `details_json` mêle camelCase et
  snake_case. ⚠️ **Le libellé, lui, les corrige** (AC 15) : c'est ce qui les rend lisibles.
- **Données personnelles** dans `details_json` : e-mails d'utilisateurs, coordonnées de contacts et de
  la société — motif de l'exclusion du rôle Consultation.

### Ce qui ne bouge PAS

- `insert_in_tx`, `find_by_entity` (hors doc-comment), les 106 sites d'écriture — **aucun code d'action
  n'est renommé** pour faciliter les libellés.
- Le bloc `admin_routes` et ses gardes.
- Le frontend, dans son entier.
- `user-manual.tex:498-503` et les lignes des epics 24 et 25 de la feuille de route du `README.md`, qui
  restent vraies jusqu'à la 25-1c-b2.

### Intelligence des stories précédentes

- **25-1c-zero** : un test qui **exécute** un montage ne prouve pas ce qu'il **démontre** — deux
  assertions vraies par construction ont traversé quatre passes de validation. Pour chaque assertion
  des AC 22-23, se demander **ce qui la rendrait fausse**. Identifiants **désalignés** partout.
- **25-1c-zero** : une mutation « `NULL` nu » laisse un paramètre lié en trop et échoue sur **erreur**,
  pas sur assertion.
- **25-1a / 25-1c-zero** : grep des manuels avec `\_` ; contrôle du **PDF aplati**.
- **25-1b** : *deux grandeurs différentes portant le même nombre sont indétectables à la relecture* —
  le nombre de clés i18n (AC 15) se recompte. Et le registre des routes : **diff ensembliste**, jamais un
  compteur (AC 18).
- **25-1c-b1** (validation) : *une assertion ajoutée pour en garantir une autre peut être muette à son
  tour* — la langue de l'AC 23 se teste avec **deux** langues différentes, sans quoi elle est vraie par
  construction.
- **Gate** : remise à zéro de base **vérifiée** ; build frontend et gate E2E **postérieurs** au dernier
  patch.

### Hors périmètre

- **L'écran**, le menu, le tri des listes par libellé → **25-1c-b1** ; les manuels utilisateur, le README
  et le glossaire partie A → **25-1c-b2**.
- **[#386]** — l'export de souveraineté qui omet `audit_log` → 25-5. ⚠️ Même donnée, périmètre
  différent : la table entière dans un ZIP, contre un export filtré ici.
- Le **nom** d'une clé API (jointure sur `api_keys`) — la route renvoie `actorApiKeyId`.
- **[#431]** (attribution par clé API incomplète), **[#434]**, **[#435]**.
- Un **marqueur d'origine** des entrées importées — écarté par l'arbitrage 2.
- Une **langue par utilisateur** : Kesh n'en a pas, et cette story n'en crée pas.

### References

- `_bmad-output/planning-artifacts/epic-25-vague1-suite.md` § *Arbitrages du 2026-09-15* et § *fin de soirée*
- `25-1c-zero-audit-company-id.md` (colonne, caractérisation du restore)
- Issue [#378] · voisines [#386], [#431], [#434], [#435]
- `repositories/journal_entries.rs:745-895` — **patron de la lecture paginée**
- `routes/journal_entries.rs:238-404` — **patron du handler de liste**
- `routes/invoices.rs:812,1158-1340` — **patron de l'export CSV**
- `tests/audit_route_registry.rs` — **patron de la garde ensembliste sur la source**
- `routes/i18n.rs`, `config.rs:524,831-832`, `kesh-i18n/src/loader.rs:110-131` — **langue et traduction**
- `routes/mod.rs:44-63` (`ListResponse`), `routes/api_keys.rs:95-100` (`ensure_not_pat`),
  `util.rs:37,104-116`, `errors.rs:546,1422`, `lib.rs:310-321,336-660`
- `25-1c-a-libelles-proposes.md` — **les 122 libellés français, arbitrés le 2026-09-16** (AC 15, T4)
- `docs/i18n-glossaire.md` § A, B, *Comment s'en servir*
- `CLAUDE.md` § *Test Locally First*, § *Propagation post-patch*, § *Le prompt d'une passe doit NOMMER
  le manuel*, § *Inventorier les sites NON RÉSOLUS*

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context) — implémentation du 2026-09-16.

### Debug Log References

- **Remise à zéro de la base (T0)** : le premier essai a échoué en `ERROR 1045 Access denied for
  'root'@'localhost'`. Le conteneur répondait déjà au `mariadb-admin ping` alors qu'il **initialisait
  encore** son datadir tmpfs. ⇒ attendre qu'un `SELECT 1` passe, jamais un ping. Deuxième essai : 40
  tables, 1 Admin, colonne `company_id` présente.
- **T1** : `cargo test -p kesh-db --lib repositories::audit_log` ⇒ **13 passed, 0 failed** (0,43 s), après
  correction d'un `unused_mut` que `clippy -D warnings` aurait refusé au gate.

### Completion Notes List

- **T1 — repository.** La clause WHERE vit dans **une seule** fonction, `push_where_clauses`, appelée sur
  deux `QueryBuilder` distincts (count et items) puis par l'export : c'est ce qui garantit que l'export et
  l'écran montrent les mêmes lignes. Les bornes de date sont **inclusives à la milliseconde** et ne
  calculent rien — le test les éprouve jusqu'au `9999-12-31 23:59:59.999`, la valeur qui faisait paniquer
  la forme abandonnée.
- **T3 — la garde a précédé le module, et c'est ce qui a servi.** Écrite d'abord, elle a **fixé** les
  deux listes par extraction positionnelle sur cinq formes, et ses trois volets structurels étaient
  verts au premier lancement : les 92 actions et 28 types du module sont donc confirmés par une
  extraction **indépendante** de celle qui avait produit le document arbitré. Deux acquis qui ne se
  devinent pas : `audit.rs` est un **passe-plat** (il relaie l'action de ses appelants et n'en produit
  aucune — l'inventorier eût rapporté deux faux angles morts), et le helper `build_audit_entry` des
  exercices écrit `"fiscal_year"` **en dur**, d'où une forme sans position de type.
- **T4 — liste française.** Elle n'est pas recopiée ici mais dans
  `25-1c-a-libelles-proposes.md`, **versionné et arbitré** le 2026-09-16 (commit `4639944a`) : la
  dupliquer créerait deux sources pour une même décision, et c'est l'écart entre les deux qu'on
  découvrirait plus tard. ⚠️ **Deux points laissés au Project Lead** : le catalogue dit « Éclater »
  là où le document arbitré dit « Ventilation appliquée » (`reconciliation.split_applied`) — le
  français arbitré est conservé, les trois cibles suivent l'attesté ; et `Mahneinstellungen` /
  `Fakturierungseinstellungen` sont **composés** à partir de termes attestés, non relevés tels quels.
  ⚠️ Relevé au passage sans être corrigé ici : « Règles d'affectation » porte **deux** formes
  allemandes au catalogue (`Zuordnungsregeln` en navigation, `Zuweisungsregeln` en titre de page) ;
  la première est retenue. Et `fiscal-year-close-button` reste **KF-041** — il dit `Schliessen`, le
  mot de « fermer » ; le libellé d'audit écrit donc `abgeschlossen`, conformément au glossaire.

### File List

- `crates/kesh-db/src/repositories/audit_log.rs` — modifié (T1)
- `crates/kesh-db/src/entities/audit_log.rs` — modifié (doc-comment, AC 4)
- `crates/kesh-api/src/util.rs` — modifié (T2 : `csv_sanitize` et ses quatre tests)
- `crates/kesh-api/src/routes/invoices.rs` — modifié (T2 : import et renvoi, définition retirée)
- `crates/kesh-api/src/audit_labels.rs` — **créé** (T3 : les deux listes, `message_key`, les trois
  fonctions de libellé et leurs **cinq** tests)
- `crates/kesh-api/src/lib.rs` — modifié (T3 : `pub mod audit_labels;` — la garde est une crate externe)
- `crates/kesh-api/tests/audit_label_registry.rs` — **créé** (T3 : la garde, quatre tests)
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — modifiés (T4 : **133** clés par locale)
- `crates/kesh-api/src/routes/audit_log.rs` — **créé** (T5 : DTO, validation partagée, liste et
  vocabulaire)
- `crates/kesh-api/src/routes/mod.rs` — modifié (T5 : `pub mod audit_log;`)
- `crates/kesh-api/src/lib.rs` — modifié (T5, T6 : montage des **trois** routes dans `comptable_routes`)
- `crates/kesh-api/tests/audit_log_e2e.rs` — **créé** (T7 : 15 tests HTTP, application montée en `de-CH`)
- `docs/manual/fr/admin-manual.tex` — modifié (T9 : AC 19 et les trois sites de l'AC 20)
- `docs/manual/fr/admin-manual.pdf` — régénéré et commité (T9, convention du dépôt)

## Change Log

### Gate de clôture — après les sept passes de revue

⛔ **Relancé en entier, et non repris de T11** : depuis, la garde a été réécrite **cinq fois**, la
cellule d'export du CSV a changé, `derive(Default)` a été retiré, quatre documents et un PDF ont été
modifiés. *Les verts consignés à T11 ne valaient plus pour cet état.*

| volet | résultat observé |
|---|---|
| base de gate | remise à zéro **inconditionnelle** puis **vérifiée** : `SELECT 1` réel après 4 s, migrations rejouées, seed appliqué — **40 tables, 1 Admin** |
| backend (`scripts/test-fast.sh`) | ✅ **2372 tests, 2372 passés**, 4 ignorés, 100,8 s — `fmt` et `clippy -D warnings` compris |
| frontend | ✅ `check` **0 erreur** (27 avertissements, tous préexistants) · `lint-i18n-ownership` PASS · `test:unit` **740/740** · `build` ✔ |
| E2E Playwright | ✅ **215 passés / 8 échoués / 19 ignorés** en 8,7 min — `kesh_e2e` reconstruite, montage **complet** |

**Les huit échecs, qualifiés un par un** :

| échec | verdict |
|---|---|
| `mode-expert:26` et `:41`, `onboarding-path-b:65` et `:92`, `onboarding:57`, `:77`, `:150` | **les 7 fixes de la KF-029 (#97)**, au complet |
| `sidebar-navigation:75` | **KF-046 (#424)** — rejoué **seul**, il **échoue** : c'est donc la KF déterministe, et non la pollution, qui passe rejouée seule |
| `invoices.spec.ts:405` et `:429` | **absents, et c'est correct** — la KF-045 ne rougit qu'avant 12:00 UTC ; le run a tourné à 16:53 |
| `inbox-import.spec.ts:106` | **absent** — la confirmation que l'échec de l'après-midi venait de **mon montage**, non du code |

⇒ **ZÉRO régression**, et zéro pollution : les huit sont tous des échecs connus et nommés.

⚠️ **Le montage E2E porte cette fois `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR`**, omises cet
après-midi, ce qui m'avait fait prendre un défaut de ma propre recette pour un échec inconnu. Les trois
contrôles préalables sont verts avant lancement : `/health` en 1 s, `smtpConfigured: true`, **aucune**
erreur `racine inbox` au log.

⚠️ Suite lancée à **16:53 UTC**, donc **après 12:00** : les deux échecs KF-045 (`invoices.spec.ts:405`
et `:429`) **ne doivent pas** apparaître — leur présence serait une régression, non un échec connu.

### Passe 7 — CIBLÉE : **1 MEDIUM, zéro CRITICAL ni HIGH** — une première depuis la passe 1

Prompt **versionné** : `25-1c-a-review-prompt-p7-ciblee.md`. Modèle : Sonnet.

⛔ **Le MEDIUM : la distinction lifetime / littéral introduite à la passe 6 n'était gardée par RIEN.**
Prouvé par mutation **exécutée** : une régression plausible de cette branche restait invisible aux dix
tests, sur les quatre entrées synthétiques **comme sur les 218 fichiers réels**. ⚠️ La lentille note
elle-même la nuance qui abaisse la sévérité : mon message de commit l'annonçait comme lacune assumée.

Elle confirme au sol les chiffres de la passe 6 — 218 fichiers, 99 portant l'attribut, 27 lignes de
chaînes brutes, le littéral de `loader.rs:386` — et vérifie que mes deux autres mutations sont bien
attrapées.

⛔ **Ma première tentative de correctif NE GARDAIT RIEN, et c'est l'épreuve qui l'a montré.** La
cinquième entrée synthétique est restée **verte** sous la mutation. La cause était ma construction :
sur `impl<'a> Chose<'a> {`, le saut « large » va d'une apostrophe à l'autre **sans jamais enjamber
d'accolade** — la régression rend donc le même compte. *Aucune de mes lignes ne portait le cas
discriminant : une accolade située **entre** deux apostrophes.*

⇒ **La propriété se teste directement sur `accolades_hors_chaines`**, non par le détour de son
appelant : `impl<'a> Foo { fn g<'b>() {} }` doit rendre `(2, 2)`, là où un saut « de la prochaine
apostrophe » enjamberait l'accolade de `Foo {` et rendrait `(1, 2)`. S'y ajoutent les lifetimes
ordinaires, `'{'`, `'}'`, l'apostrophe échappée, le guillemet-caractère et l'accolade de chaîne.

⚠️ *C'est la leçon de la passe 6, une couche plus bas : **tester une fonction par le détour de son
appelant laisse passer ce que l'appelant ne distingue pas**.*

| épreuve | verdict |
|---|---|
| mutation « saut large », **première** tentative (entrée synthétique) | ⛔ **restée verte** — l'entrée ne gardait rien |
| mutation « saut large », après correctif (test unitaire, cas discriminant) | ✅ **rouge** — « une lifetime n'ouvre ni ne ferme rien, et n'enjambe aucune accolade » |

### Passe 6 — CIBLÉE, et elle CONCLUT : simplifier plutôt que raffiner

Prompt **versionné** : `25-1c-a-review-prompt-p6-ciblee.md`. Modèle : Opus.

**Rendu : 3 HIGH, 2 MEDIUM, 1 LOW** — tous sur ma garde, aucun sur le code de production.

⛔ **Ma garde symétrique était VERTE PAR VACUITÉ, et c'est mesuré.** Sur les **99** fichiers portant un
`#[cfg(test)]`, **95** n'ont aucun item reconnu en colonne 0 après l'attribut ; et retirer le correctif
qu'elle surveillait ne changeait **rien** sur le dépôt entier — **0 fichier sur 218**. Elle ne
détectait donc pas son propre scénario nominal.

⛔ **Et la VISIBILITÉ décidait de la protection** : `est_item_de_production` énumérait douze préfixes
où manquaient `pub(crate) fn` (**28** occurrences réelles), `pub mod` (**202**), `pub use` (**85**).
Un `pub fn` était surveillé, le même code écrit `pub(crate) fn` ne l'était plus. *Le travers que
D4-ter proscrit, commis dans la garde censée l'appliquer.*

⛔ **Mon doc-comment affirmait « vérifié » et disait FAUX.** `kesh-i18n/src/loader.rs:386` porte
`contains('{')` **dans** son bloc de test (attribut l.195) : la profondeur n'y retombait jamais à zéro,
et ce bloc n'était masqué correctement que **par accident**, parce qu'il court jusqu'à la dernière
ligne du fichier. Toute production ajoutée après lui aurait été avalée en silence. *Une limite
déclarée sans être mesurée vaut moins qu'une limite tue : elle dissuade d'aller voir.*

⛔ **MEDIUM — septième décompte faux**, introduit par le commit qui fermait les décomptes faux :
« les trois lecteurs » alors qu'il y en a **cinq**, ce commit ayant lui-même ajouté le cinquième.

**⇒ La remédiation SIMPLIFIE au lieu de corriger**, sur recommandation de la lentille et conformément
à ce que j'avais annoncé au Project Lead. Le balayage du dépôt est remplacé par **quatre entrées
synthétiques écrites en dur** — `mod` déclaré sans bloc, bloc ordinaire, bloc piégé par une accolade
de chaîne **et** un littéral de caractère, attribut sur une méthode. Décidables, **indépendantes de ce
que le dépôt contient ce jour-là**, et elles exercent les quatre cas que les passes 3 à 6 ont mis au
jour. `accolades_hors_chaines` traite désormais les littéraux de caractère, sans confondre avec une
lifetime. Le nombre de lecteurs n'est plus écrit.

**Éprouvé — et cette fois les tests DÉTECTENT, cas par cas** :

| mutation | test vu rouge |
|---|---|
| sortie 2 retirée | « mod déclaré sans bloc » — `production_apres_mod_declare` a disparu |
| traitement des littéraux de caractère retiré | « bloc piégé » — `production_apres_accolades_piegees` a disparu |

⚠️ **Verdict de la lentille, que je fais mien** : « élargir l'énumération ne ferait que déplacer la
frontière ; une garde dont la sensibilité dépend du dépôt est verte par vacuité. Le coût est réel —
deux passes — et le rendement mesuré est nul. » *C'est la réponse à la question de conduite portée au
Project Lead à la passe 5.*

### Passe 5 — CIBLÉE, une seule lentille sur le seul commit `b7a06a29`

Prompt **versionné** : `25-1c-a-review-prompt-p5-ciblee.md`. Modèle : Sonnet (la passe 4 était Opus).

**Rendu : 1 CRITICAL, 1 MEDIUM, 2 LOW** — le CRITICAL est **l'angle que je lui avais donné**, et
elle l'a confirmé par mutation compilée.

⛔ **Mon appariement d'accolades ne sortait que sur `ouvert && profondeur <= 0`.** Un item gardé qui
n'ouvre **aucune** accolade — `use …;`, `mod fixtures;`, `const … = …;` — ne le déclenchait jamais :
le saut courait jusqu'à la **prochaine accolade du fichier, quelle qu'elle soit**, avalant tout ce qui
se trouvait entre les deux ; et si le fichier n'en portait plus, **jusqu'à sa fin**. Plus profond que
la troncature que la passe 4 avait remplacée. ⚠️ Dormant — aucun site de ce genre aujourd'hui — mais
`#[cfg(test)] mod fixtures;` est un idiome courant.

⛔ **Et mon garde-fou restait VERT PAR EXCÈS.** Il ne vérifiait que l'absence d'attribut résiduel : or
si le masquage avale tout, il ne reste évidemment plus aucun attribut à trouver. *Un détecteur qui ne
surveille qu'un sens de sa propre erreur ne surveille rien* — c'est la faute de l'inventaire qui
affirmait au lieu de vérifier, à deux passes de distance.

⇒ **Deux correctifs** : une seconde sortie de boucle (l'item se termine sur son point-virgule sans
avoir ouvert de bloc), et une garde **symétrique** — le masquage ne retire aucun item de production,
reconnu à sa colonne 0.

⛔ **MEDIUM — sixième décompte faux de cette boucle**, et dans le test censé fermer cette classe : mon
message annonçait « elles sont trois » là où le grep en trouve **cinq**, hors de l'en-tête. ⇒ le
message ne chiffre plus rien : *un message qui annonce « elles sont N » est lui-même un décompte.*

**Éprouvé DANS LES DEUX SENS — une première dans cette boucle** :

| épreuve | verdict |
|---|---|
| correctif **en place**, sonde `#[cfg(test)] const …;` suivie de production | ✅ rouge — l'action neuve est vue, la production n'est plus avalée |
| correctif **retiré**, même sonde | ✅ rouge — la garde symétrique nomme l'item perdu |

⚠️ La remédiation ne touche **aucune ligne de code de production**. Sévérité toujours au-dessus de
LOW ⇒ passe 6, **sous réserve de l'arbitrage de conduite porté au Project Lead** : la sévérité ne
décroît pas (HIGH en P4, CRITICAL en P5), le code de production n'a plus été pris en défaut depuis la
passe 2, et les cinq passes se sont jouées sur **un seul fichier de test**. *Ce détecteur vaut-il ce
qu'il coûte ?*

### Passe 4 — CIBLÉE, une seule lentille sur le seul commit `9ff606de`

Prompt **versionné** : `25-1c-a-review-prompt-p4-ciblee.md`. Modèle : Opus (la passe 3 était Sonnet).

**Rendu : 1 HIGH, 2 MEDIUM, 3 LOW.**

⛔ **Le HIGH : ma coupe fermait une variante du défaut et laissait l'autre — la plus coûteuse.**
Elle s'arrêtait à la première ligne valant `#[cfg(test)]`, **quel que soit l'item gardé**. Or un
attribut posé sur une **méthode**, ou un `mod tests` placé **au milieu** d'un fichier, sont deux
arrangements Rust ordinaires. Vérifié au sol :

| fichier | coupe | production sortie du balayage |
|---|---|---|
| `routes/invoice_email.rs` | l.901, sur `fn code(&self)` | **686 lignes**, dont `send_reminder_batch` |
| `kesh-db/src/version.rs` | l.97, `mod tests` au milieu | **256 lignes**, dont `check_downgrade_protection` — *le garde-fou P2-bis* — et `record_boot_version` |
| `kesh-db/src/entities/user.rs` | l.178 | **75 lignes**, dont `UserUpdate` |

⚠️ Et `invoice_email.rs` **écrit déjà de l'audit** : la zone perdue **appelait** cette écriture. Le
trou était sous le pied du prochain code d'audit du lot de rappels.

⛔ **Mais le défaut n'était pas la coupe seule : c'était mon DOC-COMMENT**, qui déclarait la classe
fermée — « la coupe porte sur une LIGNE ENTIÈRE, jamais sur une sous-chaîne » — alors qu'une seule de
ses deux variantes l'était. *Le lecteur suivant aurait lu cette assurance et ne serait pas allé voir.*
Même reproche au « filet » annoncé dans `relever()` (MEDIUM-1) : il ne prend que si le code perdu était
l'**unique** site d'une action **déjà** déclarée ; une action **neuve** n'entre dans aucun ensemble.

⇒ **Correctif : masquer le bloc par appariement d'accolades, comptées hors chaînes** — c'est ce que
l'AC 18 demandait dès l'origine. Je l'avais simplifié en `split`, en comptant sur le diff bilatéral
pour rattraper.

⛔ **MEDIUM-2 — la règle de propagation enfreinte dans le commit qui corrigeait ce défaut.** J'avais
corrigé « six » à **un** site en écrivant que c'était un décompte faux, et j'en ai laissé **deux
autres dans le même fichier** (l.20, l.250). Le grep qui suffisait :
`grep -niE '\b(six|sept)\b'`. ⇒ les nombres en prose sont désormais **doublés d'une assertion**, qui
ne se périme pas en silence.

**Deux gardes ajoutées** : le masqueur ne laisse aucun attribut derrière lui — sur **tout** le
workspace, ensemble clos et décidable — et l'inventaire compte ce que le fichier annonce.

**Les correctifs sont ÉPROUVÉS, dans les DEUX variantes du défaut** :

| épreuve | verdict |
|---|---|
| action neuve après un `#[cfg(test)]` sur une **méthode** | ✅ rouge — `invoice.reminder_batch_sent` |
| action neuve après un `mod tests` **au milieu** | ✅ rouge — `installation.sonde_neuve` |
| accolade non appariée **dans un littéral** de test | ✅ reste verte — le comptage hors chaînes tient |

⚠️ **Renseignement en soi** : les neuf tests passent alors qu'environ **mille lignes de production**
supplémentaires sont désormais balayées — aucune action d'audit ne s'y cachait.

⚠️ La remédiation ne touche **aucune ligne de code de production**, mais la sévérité reste au-dessus
de LOW : **passe 5 requise**.

### Passe 3 — CIBLÉE, une seule lentille sur le seul commit `9b47e3b8`

Prompt **versionné** : `25-1c-a-review-prompt-p3-ciblee.md`, comme l'exige la § *La passe ciblée* —
« une passe non vérifiable ne vaut pas mieux qu'une passe non faite ». Modèle : Sonnet.

**Rendu : 2 CRITICAL, 1 MEDIUM, 2 LOW — les deux CRITICAL démontrés par REPRODUCTION**, en harnais
isolé, sans écrire dans le dépôt. Tous deux causés par **mon patch de la passe 2**.

| défaut | ce qu'il rouvrait |
|---|---|
| le tamis appariait les guillemets **sans suivre les échappements**, quand `args_de` les suit vingt lignes plus haut | un seul `\"` en nombre impair désynchronise **tout le reste du fichier** : l'ensemble devient vide, la boucle d'assertion ne s'exécute plus, et le trou que ce tamis existe pour fermer se rouvre — la garde restant **verte**. Reproduit : 7 littéraux → 0 |
| la coupe `#[cfg(test)]` portait sur la source **brute**, avant tout retrait de commentaire | `config.rs:406` porte cette chaîne dans un **doc-comment**, bien avant son vrai attribut : **83 %** du fichier sortait du balayage. Défaut **dormant** tant que la garde ne lisait qu'un fichier, **activé** par mon élargissement |

⛔ **Propagation faite dans le même patch** : `relever()` — l'extracteur principal, qui balaie **toutes**
les crates — portait le même défaut de coupe. La lentille le signalait en « axes non exercés » sans
l'auditer. Les trois lecteurs passent désormais par **une seule** fonction, `source_assainie`, qui coupe
sur une **ligne entière** et non sur une sous-chaîne cherchée dans la prose. *Trois copies d'une ligne
qui s'est révélée fausse, c'était le défaut suivant.*

**MEDIUM** : « borné aux **six** fichiers inventoriés » — il y en a **sept** depuis l'ajout d'`audit.rs`.
Un décompte faux dans le commentaire même qui corrigeait un défaut de comptage voisin.

**Les deux correctifs sont ÉPROUVÉS** :

| épreuve | verdict |
|---|---|
| dérivation posée dans `config.rs` **après** le `#[cfg(test)]` en prose | ✅ rouge — la garde nomme le fichier |
| guillemet échappé impair **suivi** d'un code neuf, dans `reconciliation.rs` | ✅ rouge — le tamis voit le code malgré l'échappement |

⚠️ **La seconde épreuve a d'abord rendu un FAUX NÉGATIF, et la cause était mon épreuve, pas mon
patch.** J'avais posé la sonde **en fin de fichier** ; or `reconciliation.rs` porte son `#[cfg(test)]`
à la ligne 3523 sur 3616, si bien que la sonde tombait dans les 93 lignes **légitimement coupées**.
J'avais pris cette précaution pour `config.rs` et l'avais oubliée ici. *Une épreuve mal construite rend
un verdict qui ressemble à s'y méprendre à un correctif défaillant — et aurait fait « corriger » du
code sain.*

⚠️ **Critère de clôture** : cette remédiation ne touche **aucune ligne de code de production** — c'est
la condition qui permettra de clore la boucle. Mais la sévérité, elle, reste au-dessus de LOW :
**passe 4 requise**.

### Passe 2 de `bmad-code-review` — braquée sur la REMÉDIATION (`8599035c..HEAD`)

⛔ **Périmètre délibérément restreint au patch, non à la conception.** C'est le motif que ce dépôt a
mesuré : sur les Epics 22 et 23, **sept passes sur huit** ont trouvé une régression du patch précédent
et **aucune** un défaut d'origine. Modèles tournés par rapport à la passe 1.

| lentille | modèle | axe | rendu |
|---|---|---|---|
| La garde remédiée tient-elle ? | Opus | garde, extracteur | **1 H, 3 M, 2 L** |
| Production et documents | Sonnet | code, docs | **1 M** — le reste prouvé au sol : équivalence stricte de la cellule, `Default` retirable, documents exacts, aucune propagation résiduelle |
| Les comptes rendus disent-ils vrai ? | Haiku 4.5 | fiche, sprint-status | **1 H, 3 M** — toutes sur les statistiques git |

⛔ **Le HIGH : mon correctif de la passe 1 fermait le RENOMMAGE et laissait ouvert l'AJOUT.** Une
branche neuve — `else if définitif { "invoice.dunning_cancelled" }` — passait les sept tests : le site
restait non résolu donc inventorié, les deux codes déclarés étaient toujours là, et le code neuf
n'était **ni** relevé (une variable n'est pas un littéral) **ni** déclaré, donc n'entrait dans aucun
ensemble. *Le défaut de la passe 1, déplacé d'un cran.* ⇒ contrôle rendu **bilatéral**.

Les trois autres défauts de ma remédiation : le test lisait la source **brute** là où l'extracteur
assainit — un code en simple commentaire le satisfaisait ; la garde du chemin ne couvrait qu'**un**
fichier alors qu'elle invoque une règle de dépôt ; et mes commentaires attribuaient la protection
d'`audit.rs` au « volet (a) », **ce qui est faux** — elle vient du retrait de l'exclusion, et un
lecteur trompé aurait pu rouvrir le trou en croyant bien faire.

**Le tamis du sens 2 a d'abord rendu six faux positifs**, tous des colonnes SQL qualifiées. Plutôt que
six exceptions, un **critère** ferme la famille — un préfixe d'au moins trois caractères écarte les
alias `i.`, `c.`, aucun type d'entité de Kesh ne descendant sous quatre — et **une** exception déclarée
avec son motif couvre le reste.

**Les trois correctifs sont ÉPROUVÉS par mutation**, et non seulement verts :

| mutation | test vu rouge |
|---|---|
| branche neuve portant un code non déclaré | `les_codes_de_l_inventaire…` — « contient le littéral `invoice.dunning_cancelled` » |
| dérivation posée dans **`routes/users.rs`** | `aucune_route_ne_derive…` — la garde nomme le fichier fautif |
| code renommé mais laissé en **commentaire** | `les_codes_de_l_inventaire…` — l'assainissement mord |

⚠️ **Signal à porter au Project Lead : la sévérité maximale ne décroît pas** — `HIGH → HIGH` entre les
passes 1 et 2. La § *Règle de splitting préventif* en fait un critère. ⚠️ **Mais les deux HIGH portent
sur MA remédiation, non sur la conception** : c'est le comportement attendu d'une boucle dans ce
dépôt, pas le signe d'une story trop large. Arbitrage laissé à qui il revient.

### Passe 1 de `bmad-code-review` — trois lentilles, trois modèles, contexte frais

| lentille | modèle | axe | rendu brut | après vérification au sol |
|---|---|---|---|---|
| Correction et sécurité | Sonnet | code de production | 0C/0H/0M/2L | **2 L** — un `derive(Default)` divergent (corrigé), et `entityId=0`, qui est une **décision de spec** que la lentille a elle-même retrouvée : écartée |
| Les tests mentent-ils ? | Haiku 4.5 | tests et garde | 1C/1H/1M/1L | **1 H** (reclassé du C : le code livré est juste, c'est la garde qui était perméable), 1 M, 1 L — tous vérifiés **au sol** avant traitement, comme la § *Haiku-specific guardrails* l'impose |
| Le code tient-il ses textes ? | Opus | spec, manuel, propagation | 0C/1H/5M/3L | **1 H, 5 M, 3 L** — 23 AC sur 25 tenus, les 120 libellés français comparés **programmatiquement** au document arbitré (0 écart) |

**Ce que la passe a trouvé de plus coûteux, et c'était sur la garde elle-même** : son inventaire
**affirmait au lieu de vérifier**. Les codes de `SITES_INDIRECTS` entraient dans l'ensemble attendu
**inconditionnellement** ; un site qui aurait renommé son code aurait laissé l'inventaire **et**
`ACTIONS` sur l'ancienne valeur — diff bilatéral vide, test vert, et la production écrivant un code
**sans libellé**. L'AC 18 (d) demandait « ses valeurs **vérifiées** » ; je n'avais vérifié que leur
présence dans `ACTIONS`, ce qui va de soi. *C'est l'assertion vraie par construction que le
`CLAUDE.md` décrit, les deux côtés sortant de la même source.*

**Remédiation** (commit `919dabcc` et le suivant) : le test qui confronte chaque code déclaré au
**texte** du fichier de son site ; `PASSE_PLATS` retiré au profit d'une entrée d'inventaire à codes
vides pour `audit.rs` — *exclure un fichier échangeait une ligne de bruit contre un trou* ; la cellule
« type d'auteur » du CSV repassée par `actor_type_label` ; un code contenant `/` désormais refusé ;
`derive(Default)` retiré ; `api-external.md` et la section « Clés API » du manuel disent enfin que le
journal est fermé aux clés, **avec un autre code** que les routes d'administration ;
`Fakturierungseinstellungen` remonté en partie B du glossaire.

⛔ **Un finding que j'ai trouvé sur moi-même en éprouvant mes propres correctifs** : après avoir
corrigé la cellule du CSV, je l'ai remutée — **les quinze tests HTTP sont restés verts**. Aucun test
ne protégeait le correctif, et le rendu étant identique par les deux chemins, aucun test de sortie ne
le pourrait. D'où une garde qui vérifie le **chemin** : aucune route ne dérive une clé de code hors du
module source unique. Éprouvée à son tour — verte sur le code correct, rouge sur la dérivation
manuelle.

⚠️ **Reste ouvert, porté au Project Lead** : `user-manual.tex:498-503` affirme qu'on ne peut « pas
encore **produire** » l'historique des corrections. L'AC 21 le déclarait « vrai et à ne pas toucher » ;
la lentille conteste, le verbe *produire* étant exactement celui que l'export CSV livre. L'arbitrage
tient si « depuis l'application » se lit « depuis l'écran » — auquel cas il gagnerait à être écrit,
son voisin `:1613` disant, lui, « écran dédié ».

- **2026-09-16** — **Implémentation close** (`bmad-dev-story`, Opus 5) : les **onze** tâches cochées,
  rien de poussé. ⛔ **Périmètre FIGÉ `main..8599035c`** — et non `main..HEAD`, qui **bouge** : c'est
  la clôture de l'implémentation, avant la revue. **25 commits, 29 fichiers, +5217/−41**, **+35 tests**
  (7 dépôt, 4 `util`, 5 module, 4 garde, 15 E2E), **133 clés** par locale × 4.
  ⛔ **L'état courant n'est PAS répété ici** — ni commits, ni fichiers, ni lignes, ni total de tests.
  Une telle mesure se périme **à chaque commit de revue**, et celle-ci l'avait déjà fait deux fois :
  elle annonçait « 27 commits, 31 fichiers, +5417/−44, +42 tests, la garde passée de 4 à 7 » quand le
  dépôt en était à 34, 33, +5980 et une garde à 9. *J'avais tiré cette leçon pour la mesure figée
  ci-dessus et l'avais oubliée sur la phrase suivante.* L'état courant est tenu dans
  `sprint-status.yaml`, **daté à chaque passe**.

  ⚠️ **Deux de ces nombres étaient FAUX dès l'écriture**, relevé par la passe 2 : « 24 commits » pour
  **25**, et « +5206 » pour **+5217** — repris d'un `git diff --stat` antérieur au dernier commit de la
  série, dans la ligne même qui annonçait « décomptes recomptés depuis la source ». Les deux autres,
  29 fichiers et −41, étaient exacts et n'ont péri que du périmètre mouvant. *D'où le périmètre figé
  ci-dessus : rafraîchir les nombres les aurait repérimés au commit suivant.*

  Gates : backend **2365/2365**, frontend **740/740** + build, E2E **214/9/19** avec les neuf
  échecs qualifiés un par un ⇒ **0 régression**. Épreuve par mutation : **9 mutations, 9 rouges sur
  assertion**. ⚠️ Trois points laissés au Project Lead, écrits plutôt que tranchés en silence : l'**écart
  entre la garde et la spec** sur le découpage de l'inventaire (`audit.rs` exclu comme passe-plat et
  `projects.rs` inventorié, là où la spec fait l'inverse — six sites de part et d'autre, mais pas les
  mêmes six) ; le libellé de `reconciliation.split_applied` (« Ventilation appliquée » au document
  arbitré, « Éclater » au catalogue — le français arbitré est conservé) ; et les `⚠️`, `✓`, `✗` du manuel
  qui **ne s'impriment pas**, défaut antérieur à cette story. Prochaine étape : `bmad-code-review`.
- **2026-09-15** — Spécification créée (`bmad-create-story`), après le split de la 25-1c et les
  arbitrages du Project Lead. Quatre explorations du code (conventions backend, patrons d'écran,
  sémantique des entrées, données disponibles après import). Faits vérifiés au sol : fuseau UTC de
  MariaDB, sérialisation d'`ActorType`, emplacement de `ensure_not_pat`, unicité de `csv_sanitize`.
  **Deux choix de conception laissés à confirmer en revue** : l'inclusion des entrées sans société, et
  l'audit de l'export.
- **2026-09-15** — Contrôle checklist : `csv_sanitize` n'a **aucun** test aujourd'hui — l'AC 13 exige
  désormais ses tests unitaires à l'extraction.

### Passe 1 de `bmad-create-story validate` — deux lentilles, contexte frais

Prompt versionné : `25-1c-a-validate-prompt-p1.md`.

| Lentille | Modèle | Rendu brut | Après vérification au sol |
|---|---|---|---|
| Sonnet | Sonnet | 1 C, 1 M, 3 L | **1 C, 1 M, 2 L retenus**, 1 L écarté |
| Haiku | Haiku 4.5 | 1 H, 1 L | H **écarté**, L retenu (doublon du M de Sonnet) |

**Bilan retenu : 1 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW.**

- **C1 — la sonde des parenthèses ne sondait rien** (Sonnet). L'AC 19 (c) posait une entrée **sans
  société** d'action `Y` ; or `AND` liant plus fort que `OR`, la clause sans parenthèses se lit
  `company_id = ? OR (company_id IS NULL AND … AND action = ?)` : la branche `NULL` garde ses filtres,
  et c'est une entrée **de la société ciblée** qui fuit. La mutation « parenthèses retirées » serait
  restée **verte**, et l'AC 21 exigeait de la consigner rouge. Reproduit par la lentille sur base
  jetable, et confirmé par la priorité des opérateurs. → sonde réécrite, tableau de mutations mis en
  accord. ⛔ *Le défaut était dans le texte même qui expliquait le mécanisme* — l'AC 2 disait « le `OR`
  absorberait tous les filtres suivants », sans dire **quelle branche** y échappe.
- **M2 — référence fausse** (Sonnet, convergent avec le LOW de Haiku) : le commentaire du patron
  `QueryBuilder` est à `journal_entries.rs:745-747`, non `:737-740` (qui désigne les champs de
  `JournalEntryListResult`). Vérifié par `grep -nF "CRITIQUE"`.
- **L3 — l'AC 17 visait le mauvais endroit, et c'était pire que signalé** (Sonnet). La sous-section
  s'intitule « **Importer (restauration / migration)** » (`admin-manual.tex:1585`) : son titre même
  contredit l'arbitrage 2. → deux sites nommés, dont le titre.
- **L4 — branche morte de `for_actor`** (Sonnet) : fondé, mais le choix est **maintenu** pour défense
  en profondeur, et désormais justifié dans l'AC 14.

**Écartés, sur preuve** :

- **« 109 sites d'écriture et non 106 »** (Sonnet) — son motif `insert_in_tx(` compte aussi
  `invoice_reminders::insert_in_tx` (`dunning_reminders.rs:296`, `invoice_email.rs:586`) et sa
  définition (`invoice_reminders.rs:130`). Le motif de la spec, restreint à `audit_log(_repo)?::`,
  rend **106** : la spec est juste.
- **HIGH « le manuel ne dit pas que l'import restaure la même installation »** (Haiku) — c'est le
  travail que l'AC 17 **prescrit**, pour une story `ready-for-dev`. Constat vrai, défaut inexistant.

⚠️ **La lentille Haiku a déclaré « aucun axe non exercé » en laissant l'axe 7 de côté** (« pas
d'implémentation attendue » — alors que l'axe porte sur la conception des tests écrite dans la spec).
C'est la lentille Sonnet qui l'a exercé, et c'est là qu'était le CRITICAL.

### Passe 2 de `bmad-create-story validate` — lentille unique (Opus), contexte frais

Prompt versionné : `25-1c-a-validate-prompt-p2.md`. Base déclarée : `git diff 25a61a53 -- …`. Sondes
exécutées : base jetable (parenthèses, trois bornes de date, année 10000), crate Rust de sonde
(chrono, sérialisation serde, writer csv).

**Rendu : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 8 LOW — tous retenus après vérification au sol.** Sévérité
maximale **CRITICAL → MEDIUM** : la boucle converge. La remédiation de la passe 1 **tient** — la sonde
des parenthèses tranche, rejouée.

- **M1 — la borne « `date_to` + 1 jour » pouvait paniquer ou vider la liste en silence.**
  `"+262142-12-31"` se parse puis panique à l'incrément, sans `CatchPanic` ; `9999-12-31` + 1 jour
  donne l'an 10000, que MariaDB compare en rendant 0 ligne. ⛔ **Né du choix même de l'AC 2** — la
  route des écritures n'incrémente pas. → bornes `[1000-01-01, 9999-12-31]`, `checked_add_days`, borne
  omise au dernier jour ; tests.
- **M2 — le grep de T9 était sensible à la casse** et manquait `entities/audit_log.rs:20` ; insensible,
  il atteignait une **migration**, que P8 interdit de modifier. → `-i`, `--exclude-dir=migrations`, site
  nommé à l'AC 4.
- **M3 — un troisième site du manuel**, `admin-manual.tex:1590`, décrit l'import d'une **autre**
  installation (« l'instance importée », « l'installation précédente »). → ajouté à l'AC 17.
- **M4 — l'arbitrage 3 était MAL CITÉ** : l'epic dit « les types d'entité traduits », la spec écrivait
  « aucune traduction côté serveur » — contredite par ses propres en-têtes CSV traduits. → citation mot
  pour mot ; les **codes bruts dans le CSV** deviennent un **troisième choix non arbitré**. ⛔ *Un
  arbitrage résumé est un arbitrage réécrit.*
- **M5 — le refus Consultation n'était exigé que sur la liste** : un export monté dans
  `authenticated_routes` passait toute la suite, alors que c'est **le** chemin d'extraction que
  l'arbitrage 1 ferme. → matrice sur les deux routes, mutation ajoutée.
- **L1** assertion d'injection par sous-chaîne, faussée par le doublement des guillemets → relecture par
  `csv::ReaderBuilder`. **L2** `createdAt` : format `%.3fZ` imposé ; motif UTC = sqlx, non le serveur.
  **L3** `HEAD` et `RESULT_TOO_LARGE` n'écrivent pas d'audit. **L4** helper de montage distinct de
  `seed_actor`, `MAX_LIMIT = 200`. **L5** deux refus précisés (clé API d'un Consultation ; paramètres
  non numériques en 400 texte). **L6** `id` et `user_id` exportés — les trous d'`id` sont un indice
  d'effacement — ⇒ **onze colonnes, douze clés**. **L7** rebase par SHA ; T0 aligné sur l'état ouvert
  de #437. **L8** références corrigées ; `from_current_user`, convention du dépôt, remplace `for_actor`.

**Grep de propagation** : les nombres de colonnes et de clés (9 → 11, 10 → 12) corrigés à leurs quatre
sites de la spec (AC 12, AC 15 ×3, Dev Notes) ; « Deux choix non arbitrés » → trois dans le suivi. Les
prompts des passes 1 et 2, qui citent « 10 clés, 9 colonnes », sont des **artefacts datés** et ne sont
pas réécrits.

### Passe 3 de `bmad-create-story validate` — lentille unique (Sonnet), contexte frais

Prompt versionné : `25-1c-a-validate-prompt-p3.md`. Base déclarée : `git diff 49ff49d7 -- …`. Sondes :
base jetable (parenthèses, an 10000), crate chrono (panique, `checked_add_days`, `NaiveDate::MAX`),
round-trip CSV, source d'Axum 0.8.9 (`HEAD` servi par le handler `GET`).

**Rendu : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 1 LOW — les deux retenus.** Tout le reste de la remédiation de la
passe 2 est **confirmé sur pièces** : `from_current_user`, `HEAD`, les trois sites du manuel, les
décomptes (22 AC, 11 tâches, 11 colonnes, 12 clés).

- **M1 — la borne basse n'était ni exigée ni testée.** L'AC 20 ne portait que `dateTo`, et la narration
  de l'AC 2 ne parlait que de lui ; or `0999-12-31` se parse, et une année au-delà de 65535 échoue à
  l'encodage sqlx en **500**. → la plage vaut explicitement pour les deux paramètres, **deux** cas de test ajoutés (le troisième
  cité, `dateTo=+262142-12-31`, existait depuis la passe 2 — décompte corrigé en passe 4).
- **L2 — la couche qui calcule la borne haute n'était pas nommée** : la route « calcule la borne », mais
  la clause vit dans `push_where_clauses`. → la route valide, le repository calcule — **une seule**
  implémentation, partagée par la liste et l'export.

⚠️ **Sévérité maximale MEDIUM → MEDIUM : signal littéral de la § *Règle de splitting préventif*.** Il ne
traduit pas une story trop large : le seul MEDIUM est **né de la remédiation de la passe 2** (les bornes
de date qu'elle a introduites), et la lentille n'a rien trouvé d'autre. C'est la condition d'emploi de la
§ *La passe ciblée* ⇒ **passe 4 ciblée** sur ce seul correctif, précédent du Project Lead sur la 25-1c-zero
(« continue »).

### Passe 4 de `bmad-create-story validate` — PASSE CIBLÉE, lentille unique (Opus)

Prompt versionné : `25-1c-a-validate-prompt-p4.md`. Base déclarée : `git diff 63e1ce4d -- …`. Sondes :
crate aux versions du `Cargo.lock` (chrono 0.4.45, sqlx 0.8.6, serde_urlencoded 0.7.1), base jetable.

**Rendu : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW — tous retenus. La remédiation de la passe 3 NE TENAIT
PAS.**

- **M1 — la règle d'omission ne se déclenchait jamais pour `9999-12-31`** : `checked_add_days` y rend
  `Some(+10000-01-01)`, et `None` n'apparaît qu'à `NaiveDate::MAX` (`+262142-12-31`), déjà refusé par la
  route. La liste revenait vide, sans signal — le M1 de la passe 2, **ramené**. Et le test « 200 » de
  l'AC 20 restait vert sur une liste vide.
- **M2 — les cas `+262142-12-31` étaient muets** : `form_urlencoded` décode un `+` en clair en espace,
  la valeur échouait au **format**, et le test ne sondait plus la **plage**.
- **L1** « une 500 » était faux : `push_bind` **panique** ; l'exposition de la borne basse est l'année
  **négative**, non `0999-12-31`. **L2** `dateTo=0999-12-31` non testé, et « trois cas » au Change Log
  de la passe 3 pour deux ajoutés.

⛔ **Sévérité maximale MEDIUM → MEDIUM → MEDIUM, et les trois fois sur la même famille** — les bornes de
date, chaque passe trouvant le défaut de la correction précédente. **La leçon de la 25-1b s'applique à
la lettre** : *quand une passe ne trouve plus que les défauts de son propre patch, ajouter une passe
reconduit le motif — changer le geste, et le faire soi-même.* ⇒ **le geste a changé** : la forme
« `< date_to + 1 jour` » est **abandonnée** pour une borne **inclusive `23:59:59.999`**, qui ne calcule
rien — la classe de défauts disparaît au lieu d'être rapiécée. L'orchestrateur l'a **éprouvée lui-même**
sur base jetable avant de l'écrire (cf. AC 2). M2, L1 et L2 corrigés dans l'AC 20, l'AC 7 et le présent
Change Log ; AC 19 (d) et la mutation de l'AC 21 alignés sur la nouvelle forme.

⚠️ **Signal de la § *Règle de splitting préventif* franchi deux fois de suite** — signalé au Project
Lead. Ce n'est pas la largeur de la story qui résiste, c'est **un choix de conception** — et il est
désormais retiré. Passe 5 ciblée sur ce changement.

### Passe 5 de `bmad-create-story validate` — PASSE CIBLÉE, lentille unique (Sonnet)

Prompt versionné : `25-1c-a-validate-prompt-p5.md`, écrit pour **prendre en défaut** la borne inclusive.
Base déclarée : `git diff c25999f7 -- …`. Sondes aux versions du `Cargo.lock` et base jetable.

**Rendu : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW — retenu.** La borne inclusive **tient**, et chaque fait de
l'AC 2 a été **reproduit indépendamment** : `created_at` est `DATETIME(3)` dans la migration **et** le
squash ; une valeur à **sept décimales est tronquée** à `.999`, jamais arrondie au lendemain, sous le
`sql_mode` strict de `pool.rs:39` ; `and_hms_milli_opt` existe et rend `Some` ; la liaison en **protocole
binaire** réel (`push_bind`) rend exactement les lignes attendues, sans avertissement ; les deux bornes
extrêmes se lient sans panique, l'année −1 panique au message exact cité ; `-0001-01-01` passe `Query` et
le parsing ; le `+` en clair est décodé en espace par `serde_urlencoded`, `%2B` non.

- **L1** — la sonde de borne **basse** ajoutée à l'AC 19 (d) n'avait pas de mutation dans l'AC 21. →
  mutation `>=` → `>` ajoutée.

---

## Boucle de validation — close en 5 passes

| Passe | Modèle | CRITICAL | HIGH | MEDIUM | LOW | Retenus |
|---|---|---|---|---|---|---|
| 1 | Sonnet · Haiku 4.5 | 1 | 0 | 1 | 2 | 4 (2 écartés) |
| 2 | Opus | 0 | 0 | 5 | 8 | 13 |
| 3 | Sonnet | 0 | 0 | 1 | 1 | 2 |
| 4 | Opus — **ciblée** | 0 | 0 | 2 | 2 | 4 |
| 5 | Sonnet — **ciblée** | 0 | 0 | 0 | 1 | 1 |

**Critère d'arrêt atteint** : aucun finding au-dessus de LOW, et la remédiation de la passe 5 ne touche
**aucune ligne de code** — c'est une spec.

**24 findings retenus, 2 écartés sur preuve** (« 109 sites », et un HIGH reprochant au manuel l'absence du
texte que l'AC 17 prescrit).

**Ce que la boucle apprend, pour la rétrospective de l'Epic 25** :

1. ⛔ *Une forme de calcul qui produit un défaut à chaque correction se retire, elle ne se rapièce pas.*
   La borne « `< date_to + 1 jour` » a coûté les passes 2, 3 et 4 — panique, an 10000, puis une règle
   d'omission qui ne s'appliquait jamais. La borne inclusive, qui ne calcule rien, a clos la famille en
   une passe. C'est la leçon de la 25-1b (« changer le geste, et le faire soi-même ») appliquée — **mais
   deux passes trop tard** : le signal de stagnation était lisible dès la passe 3.
2. ⛔ *Un arbitrage résumé est un arbitrage réécrit* — « aucune traduction côté serveur » pour « les
   types d'entité traduits » (passe 2, M4).
3. ⛔ *Un test d'URL écrit à la main ment sur le `+`* : décodé en espace, il fait échouer le format au
   lieu de la plage (passe 4, M2).
4. *La sonde a été le seul outil décisif* : les défauts des passes 2 à 5 ont été établis ou réfutés par
   exécution (base jetable, crate aux versions du `Cargo.lock`), aucun par lecture seule.

---

## Réouverture du 2026-09-15 (soir) — trois arbitrages changent la conception

Après la clôture de la boucle, le Project Lead a répondu aux trois choix de conception laissés à confirmer
(`epic-25-vague1-suite.md` § *fin de soirée*) :

| choix par défaut de la spec validée | réponse | ce qui change |
|---|---|---|
| une entrée **sans société** est incluse dans toute consultation | *« le cas réel n'existe pas […] le cas théorique est … théorique »* | **filtre strict** `company_id = ?` (AC 2) ; plus de `companyId` au DTO (AC 8), plus de colonne « Société » au CSV (AC 12) ; test 22 (b) inversé ; la sonde et la mutation des **parenthèses** disparaissent avec le `OR` |
| l'**export écrit une entrée d'audit** | *« non »* | AC 14 réécrit : l'export ne fait que lire ; tests et limitation `HEAD` retirés ; le type d'entité `audit_log` n'existe plus |
| les cellules **action** et **type** du CSV restent des **codes bruts** | *« il faut mettre la traduction de la langue utilisée par l'utilisateur de kesh »* ; écran : *« ok »* ; story séparée : *« non, maintenant »* | **source unique des libellés** côté serveur (AC 16) ; libellés dans la liste JSON (AC 8) ; **route de vocabulaire** pour les filtres de l'écran (AC 17) ; **110 libellés** *(120 depuis la revalidation R1)* à écrire dans les quatre langues (AC 15) ; **garde ensembliste** sur la source (AC 18) ; langue = **celle de l'interface**, et non `accounting_language` (AC 12) |

**Numérotation** : les AC 1-13 gardent leur numéro, pour que les renvois des 25-1c-b1 et b2 restent
valables ; l'AC 14 change de sens ; les clés restent à l'AC 15 ; le module, la route de vocabulaire et la
garde prennent les AC 16-18 ; le manuel passe aux AC 19-21 et les tests aux AC 22-25. **Les AC cités dans
le Change Log ci-dessus sont ceux de la version validée**, et ne sont pas réécrits.

**Faits vérifiés au sol pour la réécriture** : aucune langue par utilisateur (`KESH_LANG`,
`config.rs:831`, `routes/i18n.rs:21`) ; `I18nBundle::format` rend la clé brute pour une clé absente
(`loader.rs:110-128`) ; 82 codes d'action *(92 depuis la revalidation R1, qui a trouvé dix actions indirectes)* et 28 types par script ; `admin_break_glass_reset` sans point
(`bootstrap.rs:289`) ; patron `audit_route_registry.rs` ; `PREFIXES_A_COUVERTURE_CLOSE` ne contient pas
`audit-log-` (`i18n-keys.test.ts:388`).

**Revalidation requise** : la réécriture touche la conception (trois routes au lieu de deux, un module, une
garde, cent dix libellés *(cent vingt depuis la revalidation R1)*). Une passe complète, puis ciblées jusqu'à zéro au-dessus de LOW.

### Revalidation R1 — une lentille Opus, contexte frais

Prompt versionné : `25-1c-a-validate-prompt-r1.md`. Base : `git diff 529ab035 46a3d54b`. Sondes : extraction
positionnelle en Python sur `crates/*/src`, dérivation des clés sur les codes réels, copie jetable des
gardes i18n du frontend avec 133 clés injectées.

**Rendu : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 3 LOW — tous vérifiés au sol et retenus.**

- **H1 — dix actions réellement écrites manquaient aux 82**, et l'inventaire « clos » de l'AC 18 était
  faux : `user.role_changed`/`updated`, `invoice.paid`/`partially_settled` (deux sites),
  `invoice.dunning_paused`/`resumed`, et quatre `fiscal_year.*` par le helper `build_audit_entry`. Toutes
  passent par une **variable** ou un **helper**. Vérifié à la source (`users.rs:310-318`,
  `invoice_settlements_write.rs:248-256`, `invoices.rs:2019-2023`, `fiscal_years.rs:126,191,316,412,736,868`).
  Un extracteur écrit selon la fiche aurait rendu une égalité **verte par construction**, et ces dix
  actions seraient sorties en code brut. → **92 / 120 / 133** partout, inventaire des sites indirects,
  table des positions, formes résolues, rouge sur tout site non résolu non inventorié (AC 18 d), mutations.
- **M1 — des constantes `pub(crate)` sont illisibles depuis `tests/`** (crate externe). → module `pub`, une
  seule `message_key`.
- **M2 — le patron du refus lit la langue comptable** (`invoices.rs:1249`), et le seul helper BCP 47
  attend une langue comptable. → `state.config.locale` pour le message, `dir_name()` pour le tag, test et
  mutations.
- **M3 — la source des arbitrages ne portait pas « ok » ni « non, maintenant »**, et gardait un paragraphe
  contradictoire. → **déjà corrigé** dans l'epic par la revalidation de la 25-1c-b2 (commit `9c2eedd1`,
  branche `story/25-1c-b-journal-audit-ecran`), avant la lecture de ce rapport.
- **L1** — `AppConfig` n'existe pas → `Config::with_locale`, patrons `spawn_app_with(_locale)`.
- **L2** — l'extracteur était sous-spécifié (positions, `.to_string()`, commentaires, `cfg(test)`) → écrit.
- **L3** — l'égalité stricte interdit de libeller un code historique → **angle mort assumé** (AC 18 e).

**Reçus d'autres passes et appliqués ici** : la revalidation R1 de la **b1** a établi que le frontend **jette**
la locale reçue de `/api/v1/i18n/messages` (`i18n.svelte.ts:28-31`) — la phrase de l'AC 17 « l'écran, qui
connaît la locale d'affichage » était fausse, corrigée ; et le tableau d'en-tête ne disait pas la b1 et la
b2 rouvertes (L8 de la b1).

Remédiation : texte de spec uniquement. **Revalidation R2 requise** (HIGH).

### Revalidation R2 CIBLÉE — une lentille Sonnet, contexte frais

Prompt versionné : `25-1c-a-validate-prompt-r2.md`. Base : `git diff 46a3d54b 02d3d45f`. Sonde : extraction
positionnelle rejouée sur `crates/*/src`, avec la seule table des formes écrite dans la fiche.

**Rendu : 0 CRITICAL, 1 HIGH, 1 MEDIUM, 0 LOW — vérifiés au sol et retenus. Après reclassement : 2 MEDIUM.**

- **H1 → MEDIUM** — les formes résolues de l'AC 18 ne couvraient pas une **conditionnelle écrite
  directement en argument** : `projects.rs:354-358` (`project.archived` / `project.unarchived`), seul site de
  cette forme (vérifié par script sur les quatre constructeurs). Un extracteur écrit selon la fiche ne l'aurait
  ni résolu ni trouvé à l'inventaire, et la règle (d) aurait rougi dès sa construction. → forme ajoutée.
  *Reclassé* : le **total de 92 était juste** — les deux actions y figuraient déjà, l'extraction de la R1
  résolvant cette forme sans l'avoir écrite —, et l'échec aurait été **bruyant**, non muet.
- **M2** — deux anciens comptes restaient écrits sans marque de péremption dans la section de réouverture
  (« 110 libellés », « 82 codes d'action ») → annotés, sur le modèle des autres mentions historiques.

La passe confirme par ailleurs : les six sites de l'inventaire aux lignes dites, 28 types, le module public
sans conflit, `dir_name()` → `de-CH`, `build_content_disposition` qui accepte ce tag,
`map_language_to_bcp47` sur les langues comptables, et l'absence de `syn` en dépendance directe — la garde
se construit par lecture de texte, sur le patron `strip_line_comments`.

Remédiation : texte de spec uniquement. **Revalidation R3 ciblée requise** (MEDIUM).

### Revalidation R3 CIBLÉE — une lentille Opus, contexte frais

Prompt versionné : `25-1c-a-validate-prompt-r3.md`. Base : `git diff 02d3d45f 8715effb` — commits
« revalidation R1 » et « revalidation R2 ciblée » de la 25-1c-a. Sonde : extraction rejouée avec les **seules**
formes et l'inventaire écrits dans la fiche.

**Rendu : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW — vérifiés et retenus.** La conception tient : **92 actions et
28 types**, sites non résolus exactement ceux de l'inventaire, aucune autre forme (`match`, bloc, `const`,
macro) dans le code de production.

- **M1** — un troisième compte périmé, écrit **en toutes lettres** (« cent dix libellés »), avait échappé à la
  R2, qui cherchait « 110 » en chiffres : *greper la valeur, pas la formulation* (`CLAUDE.md`). Le Change Log
  R2 disait donc « deux anciens comptes » pour trois. → annoté.
- **L1** — l'AC 18 ne disait pas qu'une forme couvre l'argument **entier** : un extracteur cherchant un littéral
  *dans* l'argument aurait lu une seule branche de la conditionnelle de `projects.rs`, sans rien faire rougir.
  → écrit ; le « échec bruyant » du reclassement de la R2 ne valait que pour une implémentation fidèle.
- **L2** — `projects.rs:351-357` désignait l'appel ; la conditionnelle est à `:354-358` → corrigé aux deux sites.
- **L3** — « environ soixante sites » `.to_string()` : il y en a une **quarantaine**, et la phrase, déplacée par
  l'insertion de la R2, semblait justifier la conditionnelle → corrigé et replacé.
- **Hors périmètre, intégré** — l'appariement d'accolades échoue sur `email_template_engine.rs` → restreint aux
  fichiers qui contiennent `NewAuditLogEntry::`. *(La cause d'abord écrite ici, « des gabarits `{{` », était
  fausse : le fichier n'en contient aucun ; c'est une accolade isolée dans un littéral de test, `:160` —
  corrigé en R4.)*

Remédiation : texte de spec uniquement. **Revalidation R4 ciblée requise** (MEDIUM).

### Revalidation R4 CIBLÉE — une lentille Sonnet, contexte frais — **BOUCLE CLOSE**

Prompt versionné : `25-1c-a-validate-prompt-r4.md`. Base : `git diff 8715effb 360f2b4e` — commits
« docs(25-1c-a): revalidation R2 ciblée » et « docs(25-1c-a): revalidation R3 ciblée ». Sondes : appariement
d'accolades rejoué sur les 42 fichiers de production qui contiennent `NewAuditLogEntry::`, extraction
positionnelle bornée aux virgules de profondeur 0 sur les 108 sites d'appel.

**Rendu : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW — vérifié et appliqué.** Les trois axes sont déclarés exercés.

- **L1** — la cause citée pour restreindre l'appariement d'accolades était fausse : `email_template_engine.rs` ne
  contient aucun `{{` ; le déséquilibre vient d'une **accolade isolée dans un littéral de test** (`:160`, vérifié par
  l'orchestrateur). La conclusion — restreindre aux fichiers concernés — était juste. → cause corrigée à l'AC 18 et
  annotée au Change Log R3 ; une implémentation qui ne chercherait que des `{{` aurait raté l'accolade seule.

La passe confirme : les 42 fichiers concernés ferment à profondeur 0 ; les 108 sites d'appel se répartissent en
60 littéraux, 39 `.to_string()`, 7 variables, 1 conditionnelle en argument et 1 `action.to_string()`, sans `match`,
bloc ni macro ; `projects.rs:354-358` exact ; aucun compte périmé non annoté.

**Critère de clôture** : aucun finding au-dessus de LOW, et la dernière remédiation ne touche que du texte de spec.

**Trend de la boucle rouverte** (après la clôture en 5 passes de la version d'avant les arbitrages du soir) :

| passe | modèle | rendu (après reclassement) |
|---|---|---|
| R1 | Opus, complète | 1 H, 3 M, 3 L |
| R2 | Sonnet, ciblée | 2 M |
| R3 | Opus, ciblée | 1 M, 3 L |
| R4 | Sonnet, ciblée | **1 L** |

**Ce que la boucle rouverte apprend** : le HIGH de R1 était **de conception** — dix actions écrites par variable ou
par helper échappaient au comptage, dont `invoice.paid`. Il n'a été trouvé que par une **extraction exécutée**, et
chaque passe suivante a trouvé ce que la précédente avait écrit sans l'exécuter : une forme d'argument non décrite,
un compte en toutes lettres oublié par un grep en chiffres, une cause d'échec affirmée sans lecture du fichier.
⛔ *Un inventaire se prouve en le rejouant avec ce qui est écrit, et seulement ce qui est écrit.*

### Après la clôture — les libellés français, écrits et arbitrés (2026-09-16)

Pendant l'attente du merge de la PR #437, les **122 libellés français** (92 actions, 28 types d'entité, 2 types
d'auteur) ont été rédigés et soumis au Project Lead : `25-1c-a-libelles-proposes.md`. Cinq lui ont été posés en
question, cinq ont été tranchés le jour même ; deux autres l'avaient été **par le code** — `books.*` désigne le
verrou de période (`lock_books` écrit `companies.books_locked_through`), et `installation.ui_mode_changed` le
mode d'utilisation, mot déjà employé par le catalogue.

⚠️ **Cet ajout ne touche aucun AC** : il ne fait qu'ajouter un renvoi aux References, et il alimente la tâche
T4, qui demandait précisément que la liste française soit relue par le Project Lead. Aucune revalidation n'est
requise pour un renvoi ; la fiche reste close à `1 LOW`.
