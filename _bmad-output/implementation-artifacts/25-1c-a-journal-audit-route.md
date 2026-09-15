# Story 25.1c-a : La route de consultation du journal d'audit

Status: ready-for-dev

⚠️ **Issue du SPLIT de la 25-1c**, décidé le 2026-09-15 par le Project Lead avant spécification :
sept modules, donc la § *Règle de splitting préventif*. Découpage :

| | objet | état |
|---|---|---|
| 25-1c-zero | la colonne `audit_log.company_id` | done (PR #437) |
| **25-1c-a** *(celle-ci)* | **la route de consultation et son export CSV — backend** | ready-for-dev |
| 25-1c-b | l'écran, le menu, les libellés, les E2E, les manuels utilisateur | backlog |

⛔ **Cette story ne livre AUCUN écran.** Elle livre ce qu'un écran consommera, et ce qu'un réviseur
peut déjà obtenir par un appel HTTP authentifié. **Rien d'anticipé sur la 25-1c-b** : ni composant,
ni clé i18n de libellé d'action ou de type d'entité, ni entrée de menu.

⚠️ **Dépendance** : la branche est empilée sur `story/25-1c-zero-audit-company-id`. Elle se rebase sur
`main` (`git rebase --onto origin/main 42b6aac0` — par **SHA** : le nom de branche peut être supprimé au merge) **après** le squash-merge
de la PR #437, et **avant** d'ouvrir sa propre PR.

## Story

**En tant que** comptable ou administrateur d'une société tenue dans Kesh,
**je veux** lire le journal d'audit de ma société — filtré par période, par entité et par action — et
l'exporter,
**afin de** pouvoir **produire** la trace des corrections apportées aux livres, ce que l'art. 958f CO
exige et qu'aucune route ne permet aujourd'hui.

**Couvre** : [#378], volet backend. **Ne ferme pas l'issue** — le `closes #378` appartient à la PR de
la 25-1c-b, qui livre l'écran.

## ✅ Arbitrages du Project Lead — 2026-09-15 (`epic-25-vague1-suite.md`)

1. **Qui consulte** : le **Comptable** et l'**Admin**. **Ni le rôle Consultation** — les détails
   portent des e-mails et des changements de rôle —, **ni une clé API** — qu'on ne puisse pas
   extraire la piste entière par programme.
2. **Filtre strict par société.** Un import de sauvegarde restaure **la même** installation (*« si on
   importe une sauvegarde, c'est que la base de données a disparu ou est corrompue et doit être
   recréée »*) : les identifiants de société ne changent pas. Les deux sous-cas mesurés par la
   caractérisation de la 25-1c-zero ne naissent que d'un **usage hors intention** — importer la
   sauvegarde d'une **autre** installation.
3. **Affichage sobre** — cité de l'epic : *« le code d'action tel quel, les types d'entité traduits, les
   détails en JSON indenté »*. ⇒ la **route JSON** renvoie les codes bruts `action` et `entity_type`
   et le détail tel quel : la traduction des types d'entité se fait **à l'écran** (25-1c-b). Pour le
   **CSV**, produit côté serveur, cf. le troisième choix non arbitré de l'AC 12.
4. **Vocabulaire** : « journal d'audit » — `Audit-Protokoll`, `registro di audit`, `audit log`.

⚠️ **Choix par défaut, NON arbitré explicitement — à confirmer en revue** : une entrée **sans
société** (`company_id IS NULL` : auteur inexistant à l'écriture, ou fusionnée d'un backup antérieur
à la colonne) **est incluse** dans la consultation de toute société. Motif : dans une installation à
une seule société — la seule réalité du code, aucune route n'en créant une seconde —, elle ne peut
appartenir qu'à celle-là ; l'exclure la rendrait invisible à tous, défaut que la 25-1a a combattu.

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
WHERE (company_id = ? OR company_id IS NULL)
  [AND created_at >= ?]                 -- date_from à 00:00:00.000
  [AND created_at <  ?]                 -- date_to + 1 jour à 00:00:00.000
  [AND entity_type = ?] [AND entity_id = ?] [AND action = ?]
ORDER BY created_at DESC, id DESC
```

- ⛔ **`OR company_id IS NULL`** tient le choix par défaut ci-dessus. **Les parenthèses sont
  obligatoires** : sans elles, le `OR` absorberait tous les filtres suivants.
- ⛔ **Borne haute exclusive au lendemain**, et non `<= date_to` : `created_at` est un
  `DATETIME(3)`, et `<= '2026-09-15'` exclurait tout ce qui est postérieur à minuit pile.
  ⛔ **Et ce « + 1 jour » a deux modes d'échec que la route des écritures n'a pas** (elle lie
  `date_to` sans l'incrémenter) : `NaiveDate::from_str("+262142-12-31")` réussit, puis `+ Days(1)`
  **panique** — aucun `CatchPanic` dans `kesh-api` —, et `9999-12-31` + 1 jour donne l'an 10000, que
  MariaDB compare en **rendant 0 ligne** (avertissement 1292). ⇒ la route **refuse en 400** toute date
  hors de `[1000-01-01, 9999-12-31]` (AC 7), calcule la borne par **`checked_add_days`**, et **omet la
  borne haute** quand `date_to` vaut `9999-12-31`. *(Relevé en passe 2 de validation, par sonde.)*
  ⛔ **Qui fait quoi, pour qu'il n'y ait qu'UNE implémentation** : la **route** valide la plage des deux
  dates et répond 400 (AC 7) ; le **repository** calcule la borne dans **`push_where_clauses`** —
  `date_to.checked_add_days(Days::new(1))`, clause **omise** si le résultat est `None` —, si bien que
  `list_by_company_paginated` et `list_for_export` la partagent. Le repository reçoit des dates déjà
  bornées et ne renvoie jamais d'erreur de validation ; le handler ne calcule aucune borne.
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

### Volet B — la route de consultation (`kesh-api`)

**5. `GET /api/v1/audit-log`**, montée dans **`comptable_routes`** (`lib.rs:336-660`), **avant** le
`route_layer(require_comptable_role)` de `:658` — ⛔ une route chaînée après le `route_layer` compile
et échappe au RBAC (`lib.rs:310-321`). Handler dans un **nouveau** module `routes/audit_log.rs`
(`pub mod audit_log;` dans `routes/mod.rs`).

**6. Les refus — et ils ne se valent pas** :

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

**7. Les paramètres** — struct `ListAuditLogQuery`, `#[serde(rename_all = "camelCase")]` :
`dateFrom`, `dateTo`, `entityType`, `entityId`, `action`, `offset`, `limit`.

- **Pagination** : défaut `limit = 50`, **ramené** dans `[1, 200]` et `offset.max(0)` — convention
  des écritures (`routes/journal_entries.rs:287-288`), pas le rejet en 400 des factures. Motif : un
  écran de consultation n'a aucune raison de faire échouer une page pour une taille hors bornes.
- **Dates** reçues en `String` et parsées dans le handler (`journal_entries.rs:291-304`) : format
  invalide ⇒ **400 `VALIDATION_ERROR`** nommant le paramètre ; `dateFrom > dateTo` ⇒ 400 ; **date hors de `[1000-01-01, 9999-12-31]` ⇒ 400**, pour **`dateFrom` comme pour `dateTo`** (AC 2). ⚠️ La
  borne basse n'est pas moins exposée : `NaiveDate::from_str("0999-12-31")` réussit, et une année
  au-delà de 65535 échoue à l'**encodage** sqlx (`sqlx-mysql-0.8.6/src/types/chrono.rs:263-264`,
  `u16::try_from`) — une **500**, pas une 400.
- **Textes** `entityType`, `action` : `trim()` ; vide ⇒ absent ; plus long que la colonne
  (`entity_type` 32, `action` 64 caractères) ⇒ 400 — une valeur plus longue ne peut rien trouver, et
  le dire vaut mieux qu'une liste vide muette.
- **`entityId`** : `<= 0` ⇒ 400 ; **fourni sans `entityType` ⇒ 400** — un identifiant n'a de sens que
  rapporté à son type (`entities/audit_log.rs:87-97`, « filtrer sur le couple »).
- ⚠️ **Refus hors `VALIDATION_ERROR`, et c'est assumé** : un `entityId`, `offset` ou `limit` **non
  numérique** est rejeté par l'extracteur `Query` d'Axum en **400 texte**, avant le handler — convention
  écrite du dépôt (`journal_entries.rs:275-279`). Ne pas la « corriger » ici.

**8. La réponse** — `ListResponse<AuditLogEntryResponse>` (`routes/mod.rs:44-63`), soit
`{ items, total, offset, limit }`. ⛔ **DTO dédié, et non l'entité sérialisée** :

| champ JSON | source | pourquoi un DTO |
|---|---|---|
| `id` | `id` | — |
| `createdAt` | `created_at`, **rendu en UTC explicite** : `2026-09-15T14:26:33.123Z`, format **`%Y-%m-%dT%H:%M:%S%.3fZ`** (patron `repositories/invoices.rs:89`) — millisecondes **toujours** écrites, ce que la sérialisation serde de `DateTime<Utc>` ne fait pas | `NaiveDateTime` sérialise **sans** fuseau ; l'écran doit savoir que c'est de l'UTC |
| `actorLabel` | `actor_label` | — |
| `actorType` | **`actor_type.as_str()`** : `"user"` / `"api_key"` | ⚠️ `ActorType` dérive `Serialize` **sans renommage** et sortirait `"User"` / `"ApiKey"` (`entities/audit_log.rs:35-49`) — la valeur de la base est la seule stable |
| `actorApiKeyId` | `actor_api_key_id` | — |
| `userId` | `user_id` | — |
| `action`, `entityType` | codes bruts | arbitrage 3 |
| `entityId` | `entity_id` **tel quel**, `0` compris | `AUDIT_ENTITY_ID_NONE` vaut `0` ; le filtrer ferait disparaître l'information |
| `companyId` | `company_id`, `null` possible | `null` = « société indéterminée » |
| `details` | `details_json` **tel quel** (`serde_json::Value`), `null` possible | arbitrage 3 |

**9. La consultation elle-même n'écrit PAS d'entrée d'audit.** Chaque page d'écran en écrirait une, et
le journal enflerait de sa propre lecture. ⚠️ **Asymétrie assumée avec l'export** (AC 13).

### Volet C — l'export CSV

**10. `GET /api/v1/audit-log/export.csv`**, dans `comptable_routes` **avant** le `route_layer`,
**mêmes refus** que l'AC 6 (`ensure_not_pat` en tête), **mêmes filtres et mêmes validations** que
l'AC 7 — `offset` et `limit` **ignorés**, comme l'export de l'échéancier (`invoices.rs:1225-1231`).
⛔ La validation des paramètres est **une seule fonction** partagée par les deux handlers, qui produit
l'`AuditLogListQuery`.

**11. Plafond** : `MAX_EXPORT_ROWS = 10_000`. Le handler demande `MAX_EXPORT_ROWS + 1` lignes ; au-delà
⇒ **400 `RESULT_TOO_LARGE`** (`AppError::ResultTooLarge`, `errors.rs:546,1422`), message i18n
`audit-log-export-error-too-large` avec l'argument `limit` — patron `invoices.rs:1239-1259`.

**12. Le fichier** :

- UTF-8 **avec BOM**, séparateur `;`, fins de ligne CRLF, `csv::WriterBuilder` — patron
  `invoices.rs:1266-1281` ;
- **onze colonnes**, en-têtes traduits dans la `accounting_language` de la société
  (`get_company_for`), clés `audit-log-csv-header-*` avec repli français :

  | clé | repli |
  |---|---|
  | `…-id` | N° |
  | `…-created-at` | Date (UTC) |
  | `…-actor` | Auteur |
  | `…-user-id` | Identifiant d'auteur |
  | `…-actor-type` | Type d'auteur |
  | `…-action` | Action |
  | `…-entity-type` | Type d'entité |
  | `…-entity-id` | Identifiant d'entité |
  | `…-company-id` | Société |
  | `…-api-key-id` | Clé API |
  | `…-details` | Détails |

  ⚠️ **`id` et `user_id` sont exportés, et ce n'est pas du remplissage** : des **trous** dans la suite
  des `id` sont un indice d'effacement, et `actor_label` n'est qu'un **instantané** du nom — `user_id`
  relie l'entrée au compte.

- `created_at` écrit `YYYY-MM-DD HH:MM:SS.mmm` (UTC, annoncé par l'en-tête) ; `details_json` écrit en
  **JSON compact** ; valeurs absentes ⇒ cellule vide ;
- ⚠️ **Troisième choix de conception, NON arbitré — à confirmer en revue** : les cellules `action` et
  `entity_type` restent des **codes bruts**. L'arbitrage 3 dit « les types d'entité traduits » ; pour la
  route JSON, la traduction se fait à l'écran (25-1c-b), mais le CSV **sort du serveur** — le traduire
  ici imposerait les 28 libellés dans les quatre langues, que la 25-1c-b doit écrire, et les ferait
  vivre en deux endroits. Les **en-têtes**, eux, sont traduits ;
- ⛔ **toute cellule texte passe par `csv_sanitize`** : `actor_label`, `action`, `entity_type`,
  `details`. Un nom d'utilisateur ou un détail commençant par `=`, `+`, `-` ou `@` est une injection de
  formule — et `actor_label` est **choisi par un utilisateur** ;
- `Content-Type: text/csv; charset=utf-8` ; `Content-Disposition` par
  **`build_content_disposition`** (`util.rs:104-116`, RFC 5987), nom
  `kesh-journal-audit-{slug société}-{AAAA-MM-JJ}.csv` (`slugify`, `util.rs:37`).

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

**14. L'export ÉCRIT une entrée d'audit** — action **`audit_log.exported`**, `entity_type = "audit_log"`,
`entity_id = AUDIT_ENTITY_ID_NONE`, via `NewAuditLogEntry::from_current_user(&current_user, …)` (trait `AuditActor`, `kesh-api/src/audit.rs:24`
— la convention du dépôt quand un `CurrentUser` est à portée) ; détails
**snake_case** : `row_count`, `date_from`, `date_to`, `entity_type`, `entity_id`, `action`. En
**best-effort** — transaction dédiée, `tracing::warn!` en cas d'échec, réponse 200 malgré tout — patron
`emit_report_export_audit` (`routes/reports.rs:1463-1510`).

⚠️ **`from_current_user` et non `::user`, bien que sa branche « clé API » soit morte ici** : `ensure_not_pat`
refuse toute clé avant d'atteindre l'écriture, donc `api_key_id` vaut toujours `None` à cet endroit.
Le choix est délibéré — **défense en profondeur** : si la garde devait un jour être levée, l'entrée
attribuerait encore correctement l'export à la clé, là où `::user` écrirait un fait faux (limitation L2
de `reports.rs:1423-1426`). À écrire en commentaire au site.

⛔ **Deux cas où l'export n'écrit PAS d'entrée** — rien ne sort, rien ne se trace :

- un refus **`RESULT_TOO_LARGE`** ;
- une requête **`HEAD`** : Axum la sert par le handler `GET` (`admin_pat_denied_e2e.rs:826-835`), qui
  l'exécuterait jusqu'au bout. Le handler lit la méthode (`axum::http::Method`) et **n'écrit l'entrée
  que pour `GET`**.

⚠️ **Choix de conception, NON arbitré — à confirmer en revue** : *qui a extrait la piste* est
précisément ce qu'une piste doit dire ; la consultation à l'écran ne l'est pas (AC 9), l'extraction
d'un fichier qui quitte l'application l'est.

### Volet D — i18n

**15. Douze clés, dans les quatre catalogues** (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`),
en un bloc commenté `# --- Journal d'audit — export CSV (Story 25-1c-a) ---` : les **onze** en-têtes
de l'AC 12 et `audit-log-export-error-too-large` (11 + 1). ⚠️ Le compte se **recompte depuis la
source** à l'implémentation — `grep -c '^audit-log-' crates/kesh-i18n/locales/*/messages.ftl` doit
rendre 12 dans chacune des quatre locales. Vocabulaire de l'arbitrage 4 ;
le test de parité `crates/kesh-i18n/src/loader.rs:693` impose le même jeu de clés partout.

⚠️ Ces clés sont **backend** : aucun appel `i18nMsg` n'est ajouté au frontend, donc **aucun** compteur
de `frontend/src/lib/shared/i18n-keys.test.ts` ne bouge. Le vérifier, pas le supposer.

### Volet E — ce que la story rend faux

**16. Le manuel d'administration** — `docs/manual/fr/admin-manual.tex:1786` affirme depuis la
25-1c-zero : *« le journal ne se **consulte** pas encore : aucune route ni aucun écran ne permet de le
lire depuis l'application »*. **La moitié devient fausse.** Réécrire : une route de consultation et
d'export existe pour le Comptable et l'Admin, l'écran reste à venir (#378).

**17. La conséquence de l'arbitrage 2, écrite dans le même manuel** : **l'import sert à restaurer la
même installation**. Importer la sauvegarde d'une autre installation est techniquement possible, et
ferait attribuer les entrées d'audit locales à la société importée. **Deux sites**, et le premier
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

**18. Régénération et contrôle** : `make fr` dans `docs/manual/`, PDF commités, **PDF aplati vérifié**
(`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) — l'ancienne phrase absente, les nouvelles présentes.
⛔ **En LaTeX le souligné s'écrit `\_`** : greper `audit(\\)?_log`, jamais `audit_log`.

⚠️ **Ce qui reste VRAI et ne doit pas être touché** : `user-manual.tex:498-503` (« ce qui manque encore,
c'est la consultation du journal d'audit **depuis l'application** ») et la feuille de route du `README.md`, lignes des epics 24 et 25 (« consultable
par aucun **écran** ») — c'est la 25-1c-b qui les rendra faux.

### Volet F — les tests

**19. Repository** — `#[sqlx::test(migrations = "./test-schema")]` dans `repositories/audit_log.rs`,
**identifiants désalignés** (sociétés 30 et 40, utilisateurs 501 et 602), sur le montage `seed_actor`
déjà présent (25-1c-zero). ⚠️ Ce montage ne crée que l'utilisateur 501 de la société 40 : **un helper
distinct** ajoute l'utilisateur 602 dans la société 30, **sans modifier `seed_actor`**, dont dépendent les
tests de la 25-1c-zero. Et `MAX_LIMIT` n'existe pas encore dans ce module : le **fixer à 200**, la borne
de la route (AC 7), faute de quoi le clamp de (g) est invérifiable :

- (a) **scoping** : une entrée de la société 30 n'apparaît **jamais** dans la consultation de la 40 ;
- (b) **entrée sans société** : une entrée `company_id IS NULL` apparaît dans la consultation de la 40
  **et** dans celle de la 30 ;
- (c) ⛔ **les parenthèses** : filtre `action = X` posé, une entrée **de la société ciblée** (40)
  d'action `Y` **n'apparaît pas**. ⚠️ **C'est elle, et non une entrée sans société, qui tranche** :
  `AND` liant plus fort que `OR`, la clause sans parenthèses se lit
  `company_id = 40 OR (company_id IS NULL AND … AND action = X)` — la branche `company_id = 40` y
  échappe à **tous** les filtres, alors que la branche `NULL` les garde. Une entrée `NULL` d'action `Y`
  reste donc **absente dans les deux cas**, et ne peut pas servir de sonde. *(Reproduit sur base jetable
  en passe 1 de validation.)* Garder l'assertion sur l'entrée `NULL` d'action `Y` absente, mais comme
  **propriété de l'AC 19 (b)** — une entrée sans société reste soumise aux filtres —, non comme sonde
  des parenthèses ;
- (d) **bornes de date** : entrée à `date_to 23:59:59.999` **incluse**, entrée au lendemain
  `00:00:00.000` **exclue**, entrée à `date_from 00:00:00.000` **incluse** — dates posées par `UPDATE`
  explicite ;
- (e) filtres `entity_type`, `entity_id`, `action`, chacun seul ;
- (f) **ordre** `created_at DESC, id DESC`, dont deux entrées de même `created_at` ;
- (g) **pagination** : `total` indépendant de `limit`/`offset`, page 2 correcte, clamp ;
- (h) `list_for_export` rend **exactement** les lignes de la consultation non paginée, dans le même
  ordre, et respecte `max_rows`.

**20. API** — nouveau `crates/kesh-api/tests/audit_log_e2e.rs`, `#[sqlx::test(migrations =
"../kesh-db/test-schema")]`, patron `spawn_app` / `seed_role` d'`admin_full_import_e2e.rs` :

- ⛔ **sur les deux routes** : Comptable ⇒ 200, Admin ⇒ 200, **Consultation ⇒ 403**, sans jeton ⇒ 401 —
  l'export est précisément le chemin d'extraction que l'arbitrage 1 ferme ;
- ⛔ **clé API `read` d'un Admin ⇒ 403 `API_KEY_MANAGEMENT_FORBIDDEN`**, sur **les deux** routes ;
- 400 `VALIDATION_ERROR` : `dateFrom` invalide, `dateFrom > dateTo`, `entityId` sans `entityType`,
  `entityId <= 0`, `action` de 65 caractères, `dateTo=+262142-12-31`, **`dateFrom=+262142-12-31`**, **`dateFrom=0999-12-31`** ; et
  **`dateTo=9999-12-31` accepté**
  (200, sans borne haute) ;
- forme de la réponse : `actorType` vaut **`"user"`** (jamais `"User"`), `createdAt` finit par `Z`,
  `companyId` `null` pour une entrée sans société ;
- export : `Content-Type`, `Content-Disposition` (`filename*=`), **BOM**, séparateur `;`, en-têtes,
  une ligne par entrée ;
- ⛔ **injection** : un `actor_label` posé à `=HYPERLINK("x")` sort préfixé d'une apostrophe. ⚠️
  **Relire le fichier par `csv::ReaderBuilder` (séparateur `;`) et comparer la CELLULE exacte**
  `'=HYPERLINK("x")` : le writer double les guillemets (`"'=HYPERLINK(""x"")"`), si bien qu'une
  assertion par sous-chaîne rougit à tort, et qu'une assertion négative reste verte sous la mutation ;
- **`RESULT_TOO_LARGE`** : 10 001 entrées semées par **un seul** `INSERT … SELECT` (pas 10 001
  appels) ⇒ 400 avec ce code ;
- **audit de l'export** : après un export, **une** entrée `audit_log.exported` dont `row_count` est
  exact ; **aucune** entrée après une consultation, ni après un `HEAD` sur l'export, ni après un refus
  `RESULT_TOO_LARGE`.

**21. Épreuve par mutation, résultats OBSERVÉS consignés** — chaque mutation appliquée, le test visé
**vu rouge sur assertion**, puis fichier restauré et `cmp` vérifié :

| mutation | test attendu rouge |
|---|---|
| `OR company_id IS NULL` retiré | 19 (b) |
| parenthèses retirées | 19 (c) — l'entrée de la société ciblée d'action `Y` apparaît |
| borne haute `< date_to + 1 j` → `<= date_to` | 19 (d) |
| `ensure_not_pat` retiré de la route de liste | 20, clé API |
| `csv_sanitize` retirée de la cellule `actor_label` | 20, injection |
| route d'export montée dans `authenticated_routes` au lieu de `comptable_routes` | 20, Consultation sur l'export |
| `actor_type.as_str()` → sérialisation serde de l'enum | 20, forme |

⛔ **Un test qui ne compile pas ne rougit pas : il se tait.** Chaque mutation doit produire un échec
d'**assertion**.

**22. Les gardes existantes restent vertes sans être modifiées** — à vérifier, pas à supposer :
`admin_pat_denied_e2e.rs` (la route n'est **pas** dans le bloc admin), `audit_route_registry.rs` (il ne
recense que les verbes mutants, `:181,311`), les tests de parité i18n.

## Tasks / Subtasks

- [ ] **T0 — Rebase** sur `main` **après** le merge de la PR #437 (ouverte au 2026-09-15 : ne pas
      implémenter avant), par SHA (cf. en-tête), puis remise à zéro
      **vérifiée** de la base de dev.
- [ ] **T1 — Repository** (AC 1-4) et ses tests (AC 19).
- [ ] **T2 — Extraction de `csv_sanitize`** (AC 13), tests de l'échéancier verts avant d'aller plus loin.
- [ ] **T3 — Route de consultation** (AC 5-9) : module, DTO, validation partagée, montage.
- [ ] **T4 — Export CSV** (AC 10-12, 14).
- [ ] **T5 — i18n** (AC 15), **recompte depuis la source**.
- [ ] **T6 — Tests API** (AC 20). ⛔ *Une tâche qui décrit un test est une promesse ; la cocher sans
      l'avoir écrit la transforme en mensonge.*
- [ ] **T7 — Mutations** (AC 21), résultats observés.
- [ ] **T8 — Manuel d'administration** (AC 16-18), PDF régénéré et vérifié aplati.
- [ ] **T9 — Propagation** : `grep -rniE "aucune route|post-MVP|story 3\.5" crates docs/manual/fr/*.tex --exclude-dir=migrations`
      (⛔ **insensible à la casse**, sinon « Story 3.5 » échappe ; et **sans** `migrations/` — une
      migration appliquée ne se modifie plus, P8)
      et le symptôme « la consultation n'existe pas » sur le code, les manuels, le README et `website/` ;
      trier ce qui devient faux (**cette** story) de ce qui reste vrai jusqu'à la 25-1c-b.
- [ ] **T10 — Gates** : backend complet via `scripts/test-fast.sh`, base remise à zéro **et vérifiée** ;
      frontend non touché ; **E2E complète avant le push**, `kesh_e2e` reconstruite.

## Dev Notes

### Ce que la story touche

| fichier | nature |
|---|---|
| `crates/kesh-db/src/repositories/audit_log.rs` | `AuditLogListQuery`, `AuditLogListResult`, `push_where_clauses`, deux lectures, doc de `find_by_entity`, tests AC 19 |
| `crates/kesh-api/src/routes/audit_log.rs` | **nouveau** : DTO, validation, deux handlers, audit de l'export |
| `crates/kesh-api/src/routes/mod.rs` | `pub mod audit_log;` |
| `crates/kesh-api/src/lib.rs` | deux `.route(…)` dans `comptable_routes`, **avant** `:658` |
| `crates/kesh-api/src/util.rs` | `csv_sanitize` déplacée |
| `crates/kesh-api/src/routes/invoices.rs` | `csv_sanitize` importée, plus définie |
| `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` | douze clés |
| `crates/kesh-api/tests/audit_log_e2e.rs` | **nouveau** (AC 20) |
| `docs/manual/fr/admin-manual.tex` + PDF | AC 16-18 |

**Trois crates et un manuel, aucun frontend** — sous le seuil de splitting.

### Faits établis à la spécification, et non supposés

- **Aucune lecture scopée n'existe** : `find_by_entity` (`audit_log.rs:140-158`) n'est ni scopée ni
  paginée, et n'est appelée par **aucune** route (`crates/kesh-api/src` : 0 appel).
- **Fuseau** : UTC — `@@system_time_zone = UTC`, `NOW()` = `UTC_TIMESTAMP()` ; et surtout, **sqlx impose
  `time_zone='+00:00'` à chaque session** (`sqlx-mysql-0.8.6/src/options/mod.rs:112`).
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
- **Irrégularités des codes**, que la route **ne corrige pas** (arbitrage 3) : types d'entité au pluriel
  (`reconciliation_rules`, `bank_imports`, `bank_profiles`), préfixe d'action ≠ type (`books.*` →
  `company`, `reconciliation.*` → `bank_transaction`), `admin_break_glass_reset` sans point,
  `journal_entry.updated` historique qu'aucun site n'écrit plus. `details_json` mêle camelCase et
  snake_case.
- **Données personnelles** dans `details_json` : e-mails d'utilisateurs, coordonnées de contacts et de
  la société — motif de l'exclusion du rôle Consultation.

### Ce qui ne bouge PAS

- `insert_in_tx`, `find_by_entity` (hors doc-comment), les 106 sites d'écriture.
- Le bloc `admin_routes` et ses gardes.
- Le frontend, dans son entier.
- `user-manual.tex:498-503` et les lignes des epics 24 et 25 de la feuille de route du `README.md`, qui
  restent vraies.

### Intelligence des stories précédentes

- **25-1c-zero** : un test qui **exécute** un montage ne prouve pas ce qu'il **démontre** — deux
  assertions vraies par construction ont traversé quatre passes de validation. Pour chaque assertion
  des AC 19-20, se demander **ce qui la rendrait fausse**. Identifiants **désalignés** partout.
- **25-1c-zero** : une mutation « `NULL` nu » laisse un paramètre lié en trop et échoue sur **erreur**,
  pas sur assertion.
- **25-1a / 25-1c-zero** : grep des manuels avec `\_` ; contrôle du **PDF aplati**.
- **25-1b** : *deux grandeurs différentes portant le même nombre sont indétectables à la relecture* —
  le nombre de clés i18n (AC 15) se recompte.
- **Gate** : remise à zéro de base **vérifiée** ; build frontend et gate E2E **postérieurs** au dernier
  patch.

### Hors périmètre

- **L'écran**, le menu, la traduction des 28 types d'entité, le glossaire → **25-1c-b**.
- **[#386]** — l'export de souveraineté qui omet `audit_log` → 25-5. ⚠️ Même donnée, périmètre
  différent : la table entière dans un ZIP, contre un export filtré ici.
- Le **nom** d'une clé API (jointure sur `api_keys`) — la route renvoie `actorApiKeyId`.
- **[#431]** (attribution par clé API incomplète), **[#434]**, **[#435]**.
- Un **marqueur d'origine** des entrées importées — écarté par l'arbitrage 2.

### References

- `_bmad-output/planning-artifacts/epic-25-vague1-suite.md` § *Arbitrages du 2026-09-15*
- `25-1c-zero-audit-company-id.md` (colonne, caractérisation du restore)
- Issue [#378] · voisines [#386], [#431], [#434], [#435]
- `repositories/journal_entries.rs:745-895` — **patron de la lecture paginée**
- `routes/journal_entries.rs:238-404` — **patron du handler de liste**
- `routes/invoices.rs:812,1158-1340` — **patron de l'export CSV**
- `routes/reports.rs:1463-1510` — **patron de l'audit best-effort d'un export**
- `routes/mod.rs:44-63` (`ListResponse`), `routes/api_keys.rs:95-100` (`ensure_not_pat`),
  `util.rs:37,104-116`, `errors.rs:546,1422`, `lib.rs:310-321,336-660`
- `CLAUDE.md` § *Test Locally First*, § *Propagation post-patch*, § *Le prompt d'une passe doit NOMMER
  le manuel*

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

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
  l'encodage sqlx en **500**. → la plage vaut explicitement pour les deux paramètres, trois cas de test.
- **L2 — la couche qui calcule la borne haute n'était pas nommée** : la route « calcule la borne », mais
  la clause vit dans `push_where_clauses`. → la route valide, le repository calcule — **une seule**
  implémentation, partagée par la liste et l'export.

⚠️ **Sévérité maximale MEDIUM → MEDIUM : signal littéral de la § *Règle de splitting préventif*.** Il ne
traduit pas une story trop large : le seul MEDIUM est **né de la remédiation de la passe 2** (les bornes
de date qu'elle a introduites), et la lentille n'a rien trouvé d'autre. C'est la condition d'emploi de la
§ *La passe ciblée* ⇒ **passe 4 ciblée** sur ce seul correctif, précédent du Project Lead sur la 25-1c-zero
(« continue »).
