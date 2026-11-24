# Story 50-6 : Dates et heures de Zurich — journal d'audit, sauvegardes, envois MCP, rejet de rapprochement (#1511)

Status: in-progress

*Fiche rédigée le 2026-11-24 (Opus 5.5), worktree `kesh-50-6`, branche `story/50-6-dates-heure-de-zurich` partie de
`origin/epic/50-corrections` à `e5654fc00`, par le workflow `bmad-create-story`, en autonomie. Epic 50 « Corrections », cinquième story ;
release **0.20.0** (coupure le vendredi 2026-11-27 au matin ; si elle n'est pas close à la coupure, elle sort de la release sans la
retarder — décision du Project Lead du 2026-11-24). **Aucune migration**, aucun code serveur touché. **Changement voulu d'un comportement
documenté** (décision du Project Lead du 2026-11-24, D0) : l'étape 0 a montré que le diagnostic de l'issue (« UTC naïf lu en heure
locale ») est faux ; ce qui change est le fuseau d'affichage, du poste à Zurich. Validation P1 (Sonnet) remédiée le 2026-11-24 ;
validation P2 (Opus) : 0/0/0/9, les neuf LOW traités à l'étape 0 du développement (Change Log).*

## Story

En tant qu'utilisateur qui relit la piste de contrôle (journal d'audit, sauvegardes, envois et prompts du serveur MCP, rejets de
rapprochement), je veux que chaque instant enregistré par Kesh s'affiche **à l'heure de Zurich**, comme ceux des autres écrans et comme
les jours que filtre le journal d'audit, quel que soit le fuseau de mon poste, afin qu'une opération faite peu après minuit en Suisse ne
paraisse pas à la veille ni à une autre heure selon la machine qui l'affiche.

## Étape 0 — les horodatages sont-ils naïfs ? (lue au code à `e5654fc00` ; confirmée par la validation P1)

L'issue affirme : « Le serveur rend des horodatages **sans fuseau** (UTC naïf) ; `new Date(iso)` les lit en heure locale du navigateur »,
avec pour effet « en Suisse, toujours » un décalage d'une ou deux heures. **C'est faux pour les quatre sites.** Champ par champ :

| Site (frontend) | Champ lu | Type serveur et sérialisation | Lecture côté écran | Instant lu juste ? |
|---|---|---|---|---|
| `routes/(app)/audit-log/+page.svelte:376` | `createdAt` | `crates/kesh-api/src/routes/audit_log.rs:91` `pub created_at: String,` rempli `:111-113` `.created_at` `.format("%Y-%m-%dT%H:%M:%S%.3fZ")` — **`Z` écrit à la main** (commentaire `:89-90` « UTC **explicite** ») | `new Date(e.createdAt)` | **oui** |
| `lib/features/reconciliation/UnrejectTransactionAction.svelte:39` | `autoMatchRejectedAt` | `bank_imports.rs:342` `pub auto_match_rejected_at: Option<chrono::DateTime<chrono::Utc>>,` rempli `:361` `t.auto_match_rejected_at.map(\|d\| d.and_utc())` | `new Date(tx.autoMatchRejectedAt)` | **oui** |
| `lib/features/admin-backup/backup-format.ts:8` | `lastAttemptAt`, `lastSuccessAt`, `nextDueAt`, `modifiedAt` | `backup_schedule.rs:62-63` `pub last_attempt_at: Option<chrono::DateTime<Utc>>,` / `pub last_success_at: …`, `:71` `pub next_due_at: Option<chrono::DateTime<Utc>>,`, `:204` `pub modified_at: Option<chrono::DateTime<Utc>>,` ; conversion `:50-51` `fn utc(t: Option<NaiveDateTime>) -> …` `t.map(\|t\| t.and_utc())` | `new Date(iso)` | **oui** |
| `lib/features/mcp-sends/mcp-sends-format.ts:41` | `createdAt`, `expiresAt`, `decidedAt` (et, par le même `dateLocale`, `creeLe`, `decideLe`, `modifieLe` des prompts MCP) | `mcp_envois.rs:58-61` `pub expires_at: NaiveDateTime,` `pub decided_at: Option<NaiveDateTime>,` `pub created_at: NaiveDateTime,` ; `mcp_prompts.rs:189-190`, `:221` `NaiveDateTime` — **naïfs** | `:40` `const iso = /(Z\|[+-]\d{2}:\d{2})$/.test(valeur) ? valeur : \`${valeur}Z\`;` — **le `Z` est ajouté** avant `new Date(iso)` | **oui** |

Sérialisation de `DateTime<Utc>` : chrono 0.4.45 (`Cargo.lock`), `~/.cargo/registry/src/*/chrono-0.4.45/src/datetime/serde.rs:45`
`write_rfc3339(f, naive, offset, SecondsFormat::AutoSi, true)` — le dernier argument est `use_z` : la chaîne finit par `Z`, avec 0 à 9
décimales (`modifiedAt`, tiré d'un `SystemTime`, en porte jusqu'à neuf). Aucun `serde(with …)` ni `serialize_with` dans les cinq fichiers
serveur (validation P1). Les tests existants le confirment : `UnrejectTransactionAction.test.ts:72` `autoMatchRejectedAt: '2026-03-04T11:00:00Z'`,
`backup-schedule.api.test.ts:37` `lastSuccessAt: '2026-10-24T01:00:00.000Z'`, `audit-log-page.test.ts:68` `createdAt: '2026-09-16T12:26:33.123Z'`.
Le JSON réel n'avait pas été relu à la rédaction : c'était la prémisse **P1**, établie à l'étape 0 du développement (2026-11-24, base
jetable `kesh_e506`, binaire de `epic/50-corrections` à `e5654fc00`, `kesh-gate-logs/e506/ac1-json.txt`) : `createdAt` (audit),
`autoMatchRejectedAt`, `lastAttemptAt`/`lastSuccessAt`/`nextDueAt`, `modifiedAt` finissent par `Z` ; `createdAt`/`expiresAt`/`decidedAt`
des envois MCP et `creeLe`/`modifieLe` des prompts n'ont **pas** de fuseau. **Prémisse confirmée, aucun écart.** Les neuf décimales de
`modifiedAt` sont réelles (`2026-11-24T18:44:17.026083080Z`).

Lecture JS vérifiée (Node 22.22.1, poste à `Europe/Zurich`) : `Date.parse("2026-11-24T22:15:00.123456789Z")` → `22:15:00.123Z` (les neuf
décimales passent) ; `"2026-11-24T22:15:00.123"` (naïf) → `21:15:00.123Z` (lu local : le défaut que l'issue décrit, **absent** des quatre sites).

**Ce qui change donc.** Les quatre sites lisent le bon instant, mais le **formatent dans le fuseau du poste** (`Intl.DateTimeFormat` sans
`timeZone` : `audit-log/+page.svelte:65`, `UnrejectTransactionAction.svelte:37`, `backup-format.ts:10`, `mcp-sends-format.ts:43`). Sur un poste
réglé sur la Suisse (celui du Project Lead, et le navigateur des specs : `playwright.config.ts:59` `timezoneId: 'Europe/Zurich'`), l'affichage
est juste et le restera à l'identique. Sur un poste réglé ailleurs (voyage, machine ou VM en UTC), `2026-11-24T23:30:00Z` s'y lit aujourd'hui
« 24 nov. 2026, 23:30 » (UTC) ou « 24 nov. 2026, 18:30 » (New York) ; il se lira « 25 nov. 2026, 00:30 ». Les autres écrans affichent déjà
Zurich (`formatSwissDate`/`formatSwissDateTime`, `reports.api.ts:258-273` : utilisateurs, clés d'API, factures, rapports), et **les filtres du
journal d'audit choisissent des jours de Zurich** (0.18.0) : sur un poste en UTC, une entrée de 23 h 30 UTC paraît aujourd'hui datée du 24
sous le filtre du 25.

Le manuel documentait le choix inverse : `docs/manual/fr/user-manual.tex:4688-4689` « Le tableau affiche l'heure de votre \textbf{poste} ---
pour un poste réglé sur la Suisse, la même. » ; le doc-comment de `mcp-time.ts:28` dit de même « Date et heure dans le fuseau du navigateur ».

## Ce qui se lit au code (`e5654fc00`)

### Les fonctions qui traitent déjà le fuseau

- `frontend/src/lib/features/reports/reports.api.ts:219` `const TIMESTAMP_RE = /^(\d{4}-\d{2}-\d{2})[T ](\d{2}:\d{2}(?::\d{2}(?:\.\d+)?)?)(Z|[+-]\d{2}:\d{2})?$/;`,
  `:222` `const ZURICH_PARTS = new Intl.DateTimeFormat('fr-CH', {` (`timeZone: 'Europe/Zurich'`, `:223`), `:237` `function zurichParts(iso: string)`
  (`:240` `const ms = Date.parse(\`${m[1]}T${m[2]}${m[3] ?? 'Z'}\`);` — naïf lu UTC, décalage honoré), `:258` `export function formatSwissDate`,
  `:270` `export function formatSwissDateTime`. Forme fixe `JJ.MM.AAAA HH:mm`, indépendante de la langue. **57 fichiers** importent de
  `reports.api.ts` (`grep -rln "features/reports/reports.api" frontend/src | wc -l`), dont `users/+page.svelte` et `settings/api-keys/+page.svelte` :
  c'est, de fait, le module commun des dates.
- `lib/features/mcp-server/mcp-time.ts:21` `export function depuisUtcNaif(s: string | null | undefined): Date | null {` (copie de la même
  lecture, `:14` `AVEC_FUSEAU` accepte aussi `z` minuscule et `±hhmm`), `:29` `dateHeureAffichee` et `:36` `dateAffichee` formatent **au fuseau
  du poste** (`:32`, `:39`, sans `timeZone`). `dateAffichee` sert à deux choses : la date d'**expiration** d'une clé, **saisie** au poste
  (`:8` « dernière journée de validité en heure locale du navigateur », `finDeJourneeLocale` `:46-51`, `lendemainLocal` `:54-58`, aller-retour
  figé par `mcp-time.test.ts:66-89` et écrit à `admin-manual.tex:3584` « à la fin de ce jour, à l'heure du poste »), et la date de
  **révocation**, un instant (`McpKeysSection.svelte:412`).
- `lib/shared/utils/local-date.ts:21` `const JOUR_DE_ZURICH = new Intl.DateTimeFormat('en-CA', {` — le jour de Zurich (`todayInZurich`).

### Les quatre sites de l'issue

- `routes/(app)/audit-log/+page.svelte:64-66` `let dateFormat = $derived(` `new Intl.DateTimeFormat(i18nLocale(), { dateStyle: 'medium', timeStyle: 'short' }),`
  puis `:376` `<td class="py-2 whitespace-nowrap">{dateFormat.format(new Date(e.createdAt))}</td>` (cellule **sans** `data-testid`).
- `lib/features/reconciliation/UnrejectTransactionAction.svelte:37` `const dateFormat = $derived(new Intl.DateTimeFormat(i18nLocale(), { dateStyle: 'medium' }));`,
  `:38-40` `const rejectedOn = $derived(` `tx.autoMatchRejectedAt != null ? dateFormat.format(new Date(tx.autoMatchRejectedAt)) : '',`.
- `lib/features/mcp-sends/mcp-sends-format.ts:38` `export function dateLocale(valeur: string | null): string {` (`:39` `—` si vide, `:42` valeur
  brute si illisible), appelée par `McpSendsList.svelte:98-99`, `McpSendDetail.svelte:195`, `:197`, `:205` **et**, par le réexport de
  `mcp-prompts/mcp-prompts-format.ts:23`, `McpPromptDetail.svelte:350`, `:427`, `:508`, `:514` (les prompts MCP changent donc aussi, sans que
  leur code change).
- `lib/features/admin-backup/backup-format.ts:6` `export function formatBackupDate(iso: string | null, locale: string): string {` (`''` si absent
  ou illisible), appelée par `AutoBackupSection.svelte:411`, `:435`, `:446`, `:535` et `BackupScheduleBanner.svelte:39` (le bandeau).

### Inventaire par le symptôme (tout `frontend/src`, hors tests)

Commandes et sortie complète : `/home/gcorbaz/devel/kesh-gate-logs/e506/inv-symptome.txt` (68 lignes) — `git grep` de `new Date(`,
`toLocale*String`, `.getHours/Minutes/Seconds/TimezoneOffset(`, `.getFullYear/Month/Date/Day(`, `DateTimeFormat`, `Date.parse`, hors
`*.test.ts`, `*.spec.ts`, `*.test-helper.ts`. `toLocale…String`, heures, minutes, décalage : **aucune ligne**. Aucune bibliothèque de dates
dans `package.json`. La validation P1 a élargi la recherche (`toISOString|toDateString|toUTCString|RelativeTimeFormat|formatToParts|.slice(11|Temporal`,
gabarits `{…At}` `{…Le}`) sans trouver de site omis. Classement :

**Dans le périmètre (instant formaté au fuseau du poste)**

| # | Site | Pourquoi |
|---|---|---|
| E1 | `audit-log/+page.svelte:65`, `:376` | `Intl` sans `timeZone` ; `createdAt` porte `Z` |
| E2 | `UnrejectTransactionAction.svelte:37`, `:39` | idem, date seule |
| E3 | `mcp-sends-format.ts:40-43` | `Z` ajouté, `Intl` sans `timeZone` (couvre aussi les prompts MCP) |
| E4 | `backup-format.ts:8-10` | `Intl` sans `timeZone` ; champs `DateTime<Utc>` (écran et bandeau) |

**Hors périmètre, écrit comme limite (issue I2)**

| # | Site | Pourquoi |
|---|---|---|
| H1 | `mcp-server/mcp-time.ts:24`, `:32`, `:39` — instants de l'écran *Serveur MCP* : création et dernier usage d'une clé (`McpKeysSection.svelte:403`, `:408`), révocation (`:412`), dernière modification des réglages (`McpSettingsSection.svelte:111`) | sixième module (D1, D2) ; restent au fuseau du poste jusqu'à I2 |

**Justes**

| Site | Raison |
|---|---|
| `reports.api.ts:222`, `:240`, `:243` | `timeZone: 'Europe/Zurich'` ; naïf lu UTC — le module de référence |
| `mcp-time.ts:39` (pour `expiresAt`), `:46-58` (`finDeJourneeLocale`, `lendemainLocal`) | **date saisie** (D0, exception) : saisie et affichée au poste, aller-retour voulu |
| `settings/api-keys/+page.svelte:105` (`formatSwissDate(value.slice(0, 10))`) | **date saisie** d'expiration d'une clé d'API, envoyée `T23:59:59Z` — déjà exception de la garde existante |
| `local-date.ts:13-16` (`todayLocal`), `:21` (`JOUR_DE_ZURICH`), `:37`, `:46` | date du **poste** voulue pour une saisie ; Zurich pour une borne ; `Date.UTC` sur une date pure |
| `settings/api-keys/+page.svelte:72` | `new Date(utc).getTime() < Date.now()` : comparaison d'instants, `Z` ajouté `:71` ; indépendante du fuseau (mais copie de la lecture : I1) |
| `users/+page.svelte:276` | `isNaN(new Date(iso).getTime())` : test de validité seul ; l'affichage passe par `formatSwissDate` |
| `journal-entries/form-helpers.ts:82` | `` new Date(`${iso}T00:00:00Z`) `` : date pure, arithmétique UTC |
| `fiscal-years.helpers.ts:31-32` | `new Date(input.startDate)` sur `AAAA-MM-JJ` : lu UTC minuit par la norme, comparé à l'autre — ordre seul |
| `fiscal-years.helpers.ts:52`, `onboarding/premier-exercice.ts:18-31`, `:50` | année / date **du poste** voulue (saisie) |
| `exchange-rates-grid.ts:167-195`, `homepage-charts.ts:206-216`, `bank-import/manual-statement.ts:106`, `invoice-helpers.ts:123`, `journal-entries/fx.ts:163`, `shared/utils/date-filtre.ts:23` | `Date.UTC` et `timeZone: 'UTC'` sur des dates pures ou des mois : rien ne glisse |
| `reminders/reminder-sent-at.ts:28` | `now: Date = new Date()` : instant courant, envoyé en ISO UTC |

**Hors sujet** : `invoice-helpers.ts:116`, `mcp-time.ts:4`, `users/+page.svelte:272`, `i18n.svelte.ts:14` (commentaires) ;
`pin-clock.test-helper.ts:21` (test). Les specs E2E tournent sous `timezoneId: 'Europe/Zurich'` ; le CSV de l'audit (`audit_log.rs:373-375`,
UTC, manuel `:4724`) n'est pas un affichage d'écran.

**Angles morts assumés de l'inventaire** : un instant formaté par une fonction d'un autre module sans `new Date(` ni `DateTimeFormat` sur la
même ligne ; un affichage brut d'une chaîne d'horodatage dans un gabarit — un second `grep` (`\{[^}]*[a-z](At|Le)\b[^}]*\}` hors formateurs)
n'en trouve aucun.

### Manuels

- `docs/manual/fr/user-manual.tex:4688-4689` (journal d'audit) : la phrase « Le tableau affiche l'heure de » / « votre \textbf{poste} --- pour un
  poste réglé sur la Suisse, la même. » est **coupée sur deux lignes** : un `git grep` d'une ligne ne la voit pas. Elle devient fausse.
- `user-manual.tex:790-791` « Les \textbf{heures} enregistrées (journal d'audit, envoi d'un rappel) restent en UTC » : vraie du stockage ; à
  compléter de l'affichage et de ses exceptions.
- `admin-manual.tex:3584` (expiration d'une clé MCP « à l'heure du poste ») : **reste vraie** (exception de D0).
- `user-manual.tex:3661` (« Rejetée le~\dots »), `admin-manual.tex:2507` (« Ce que l'écran montre » de la sauvegarde automatique), `:2509`
  (bandeau « depuis le … »), § `sec:mcp-envois` : aucun fuseau nommé — rien ne devient faux.
- `admin-manual.tex:2717`, `:2745` : notes des versions 0.18.0 et 0.17.7, historiques — inchangées.
- Seul le français existe (`docs/manual/{de,en,it}/` ne portent qu'un `README.md`).
- **PDF** : la phrase du tableau y est. `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "heure de votre poste" | wc -l`
  rend **1** à `e5654fc00` ; un grep qui inclut « affiche » échoue sur la **ligature « ﬀ »** (« aﬀiche ») et l'apostrophe typographique : on
  cherche donc `heure de votre poste`. Les PDF ne se régénèrent pas dans la story (règle des fichiers communs) ; le contrôle PDF se rejoue
  après `make fr`, à la release.

## Décisions

- **D0 — l'heure de Zurich partout où Kesh affiche un instant** (décision du Project Lead du 2026-11-24). Un **instant** est un moment
  enregistré par Kesh (création, décision, rejet, sauvegarde, dernier usage). **Exception : les dates saisies** — la date d'expiration d'une
  clé du serveur MCP (`mcp-time.ts`, saisie et affichée au poste, `admin-manual.tex:3584`) et celle d'une clé d'API (date de calendrier) —
  restent saisies et affichées telles quelles (arbitrage de l'orchestrateur sur V1-3, que le Project Lead peut revoir). **Limite** : les
  instants de l'écran *Serveur MCP* (H1) restent au fuseau du poste jusqu'à I2, faute d'un sixième module ; la note de version et le
  manuel le disent. C'est un changement voulu d'un comportement documenté : note de version « Modifié ». *Écartée* : garder le fuseau du
  poste (comportement documenté de la 0.18.0).
- **D1 — un seul formateur d'instants, `formatInstant`, logé à côté de `zurichParts` dans `lib/features/reports/reports.api.ts`.** Il
  reprend la lecture de `zurichParts` (une fonction privée `instantDuServeur(iso): Date | null` extraite de `:237-241`, que `zurichParts`
  appelle désormais : une seule regex, un seul « naïf = UTC ») et formate par `Intl.DateTimeFormat(locale, { …style, timeZone: ZONE_SUISSE })`,
  la constante `ZONE_SUISSE = 'Europe/Zurich'` étant partagée avec `ZURICH_PARTS`. Signature :
  `formatInstant(iso: string | null | undefined, style: 'date' | 'dateHeure', locale: string): string | null` — `date` =
  `{ dateStyle: 'medium' }`, `dateHeure` = `{ dateStyle: 'medium', timeStyle: 'short' }` (les styles actuels des quatre sites : **aucun
  changement visible sur un poste suisse**) ; `null` si absent ou illisible, chaque appelant garde son repli (`''`, `—`, valeur brute).
  **Cache** (V1-10) : les formateurs sont gardés dans une `Map` de module, clé `` `${locale}|${style}` ``, construits au premier appel —
  le journal d'audit (50 lignes) ne régresse pas par rapport à son `$derived` actuel.
  *Écartée 1* : un module neuf `lib/shared/utils/dates.ts` — la bonne place à terme, mais six modules ; le déménagement, pur, va à **I1**.
  *Écartée 2* : `formatSwissDateTime` (forme fixe `JJ.MM.AAAA HH:mm`) — change la forme visible des quatre écrans en Suisse. *Écartée 3* :
  un `formatInstant` avec sa propre regex — une copie de plus. **Copies restantes après la story** (V1-9) : `mcp-time.ts:14-26`
  (`depuisUtcNaif`, I2) et `settings/api-keys/+page.svelte:70-71` (lecture « naïf = UTC » avant comparaison, I1) ; elles sont suivies par ces
  issues et inscrites à la garde.
- **D2 — `mcp-time.ts` (écran *Serveur MCP*) reste hors périmètre** (sixième module). **I2** se borne aux **instants** de cet écran
  (création et dernier usage d'une clé, révocation, modification des réglages) : les passer par `formatInstant` et retirer `depuisUtcNaif`, en
  **séparant** `dateAffichee` en deux usages — la révocation (instant, à Zurich) et l'expiration (date saisie, inchangée, aller-retour de
  `mcp-time.test.ts:66-89` conservé). *Écartée* : l'inclure (6 modules).
- **D3 — la garde étend `lib/shared/utils/local-date-guard.test.ts`** et son auditeur (`auditer` : une entrée ↔ un relevé ; « entrée
  orpheline », « relevé sans entrée », « fragment à plusieurs lignes », « relevé à plusieurs entrées » rougissent). Deux motifs neufs, relevés
  **par ligne**, sans masque des commentaires, dans les sources hors tests (`estSource`) :
  - `MOTIF_FORMAT_D_INSTANT` = `/DateTimeFormat\(|toLocale(Date|Time)?String\(|\.get(Hours|Minutes|Seconds|TimezoneOffset)\(/` ;
  - `MOTIF_LECTURE_D_INSTANT` = `/new (?:globalThis\.)?Date\((?!\)|Date\.UTC\()|Date\.parse\(/`.
  Ces motifs **n'attrapent pas** `new Date()` (sans argument) ni `new Date(Date.UTC(…))` : ces lignes n'ont **pas** d'entrée (sans quoi elles
  seraient orphelines). Relevés à `e5654fc00` (commande ci-dessous, sortie `kesh-gate-logs/e506/garde-releves-avant.txt`) : **30**, dont 8 sur
  les quatre sites corrigés (`audit-log/+page.svelte:65`, `:376` ; `UnrejectTransactionAction.svelte:37`, `:39` ; `backup-format.ts:8`, `:10` ;
  `mcp-sends-format.ts:41`, `:43`), qui disparaissent. Restent **22 relevés**, d'où l'allowlist `ALLOWLIST_INSTANTS`, **une entrée par ligne relevée**
  (`{ fichier, fragment, motif }`, le fragment étant un extrait exact de la ligne), sauf les deux lignes identiques de `premier-exercice.ts`
  (une entrée à `lignes: 2`) : **22 relevés, 21 entrées** (V2-2) :

  | # | Fichier : ligne (à `e5654fc00`) | Motif de l'entrée |
  |---|---|---|
  | 1 | `routes/(app)/settings/api-keys/+page.svelte:72` | comparaison d'instants (`Z` ajouté) ; copie de lecture, I1 |
  | 2 | `routes/(app)/users/+page.svelte:272` | commentaire |
  | 3 | `routes/(app)/users/+page.svelte:276` | test de validité seul |
  | 4 | `lib/features/journal-entries/form-helpers.ts:82` | date pure, `Z` littéral |
  | 5-6 | `lib/features/onboarding/premier-exercice.ts:29`, `:50` | composants d'une date du poste — **deux lignes identiques** (voir ci-dessous) |
  | 7-9 | `lib/features/reports/reports.api.ts:222`, `:240`, `:243` | module de référence (lignes déplacées par le correctif : se recomptent) |
  | 10 | `lib/features/invoices/invoice-helpers.ts:116` | commentaire |
  | 11 | `lib/features/mcp-server/mcp-time.ts:24` | lecture naïf = UTC (copie, I2) |
  | 12 | `lib/features/mcp-server/mcp-time.ts:32` | instants au fuseau du poste — limite de D0, I2 |
  | 13 | `lib/features/mcp-server/mcp-time.ts:39` | expiration (date saisie, exception de D0) et révocation (instant, I2) |
  | 14 | `lib/features/mcp-server/mcp-time.ts:49` | fin de journée locale de l'expiration saisie |
  | 15 | `lib/features/mcp-server/mcp-time.ts:55` | lendemain local (borne de saisie) |
  | 16-17 | `lib/features/homepage/homepage-charts.ts:211`, `:216` | `timeZone: 'UTC'` sur un mois |
  | 18-19 | `lib/features/exchange-rates/exchange-rates-grid.ts:186`, `:194` | idem |
  | 20-21 | `lib/features/fiscal-years/fiscal-years.helpers.ts:31`, `:32` | dates pures comparées |
  | 22 | `lib/shared/utils/local-date.ts:21` | `timeZone: 'Europe/Zurich'` (`JOUR_DE_ZURICH`) |

  Plus les lignes neuves de `reports.api.ts` (au moins le constructeur du formateur mis en cache ; `Date.parse` déplacé dans
  `instantDuServeur`), chacune avec son entrée : **les deux totaux se recomptent** au dernier commit et s'écrivent au Dev Agent Record (relevés 22 + n, entrées 21 + n).
  **Lignes identiques** : `premier-exercice.ts:29` et `:50` portent le même texte (`const d = new Date(2000, 0, 1);`) ; l'auditeur actuel ne sait
  pas les admettre (une entrée → « fragment à plusieurs lignes » ; deux entrées → « relevé à plusieurs entrées »). L'entrée gagne un champ
  facultatif `lignes?: number` (défaut 1), et `auditer` exige `rs.length === (e.lignes ?? 1)` ; la sonde vérifie qu'une troisième ligne
  identique rougit. Commande de recompte (celle de la validation P1, à rejouer par la P2) :
  `cd frontend/src && grep -rnP "DateTimeFormat\(|toLocale(Date|Time)?String\(|\.get(Hours|Minutes|Seconds|TimezoneOffset)\(|new (?:globalThis\.)?Date\((?!\)|Date\.UTC\()|Date\.parse\(" --include=*.ts --include=*.svelte . | grep -vE '\.test\.(ts|svelte):|\.test-helper|\.spec\.ts:' | wc -l`
  → 30 avant, 22 + n après (le compte des **relevés** ; celui des entrées est le nombre d'objets de `ALLOWLIST_INSTANTS`, une de moins).
  **Angles morts assumés de la garde** (V1-11) : un alias (`const F = Intl.DateTimeFormat; new F(…)`, `const D = Date; new D(x)`), un jeton
  coupé (`Intl.DateTime\nFormat(`), `Reflect.construct(Date, [x])`, `Temporal`, `new window.Date(x)` et `new self.Date(x)` (le motif n'admet que `globalThis.`),
  les composants calendaires du poste `.getFullYear(`, `.getMonth(`, `.getDate(`, `.getDay(` (R1-4 : le jour du poste d'un instant venu d'ailleurs, ou de `new Date()`, passerait ; ceux d'aujourd'hui sont des dates du poste voulues : `premier-exercice.ts`, `fiscal-years.helpers.ts`, `mcp-time.ts`, `local-date.ts`), `new Date (x)` (espace avant la parenthèse ; Prettier l'efface ; aucune de ces trois formes n'existe aujourd'hui, `grep -rnE
  "new window\.Date|new self\.Date|Reflect\.construct|new Date \(" --include=*.ts --include=*.svelte frontend/src` hors tests : vide), une date construite dans un autre module et passée à un
  formateur légitime ; deux lectures sur une même ligne ne font qu'un relevé (la ligne rougit quand même si elle n'est pas admise). Sont,
  eux, attrapés et sondés : l'appel sur plusieurs lignes (`new Intl.DateTimeFormat(\n locale, {…})`, le jeton `DateTimeFormat(` étant sur la
  première) et `new globalThis.Date(x)`.
  *Écartée* : la garde de l'issue (« `new Date(` sur un champ `*At` ») — elle part d'une liste de noms : `backup-format.ts:8` lit
  `new Date(iso)` et lui échappait, et `Intl` sans `timeZone`, le vrai symptôme, n'y était pas.
- **D4 — `mcp-sends-format.ts` garde `dateLocale`** (nom et contrat : `—` si vide, valeur brute si illisible), réécrit en une ligne sur
  `formatInstant(valeur, 'dateHeure', i18nLocale()) ?? valeur` ; son commentaire (`:34-37` « à l'heure **locale**, comme le journal
  d'audit ») est réécrit. Pareil pour `formatBackupDate` (`?? ''`) ; le journal d'audit prend la valeur brute (`?? e.createdAt`, V2-5). Les appelants ne changent pas. *Écartée* : renommer — touche `mcp-prompts`.
- **D5 — commentaires rendus faux hors des cinq fichiers** (V1-8) : `mcp-prompts/mcp-prompts-format.ts:21-22` (« UTC de la base montrée à
  l'heure locale ») et `UnrejectTransactionAction.test.ts:67-69` (« La date suit le fuseau du navigateur… ») se réécrivent. Une correction de
  **commentaire seul** ne fait pas de `mcp-prompts` un module touché (règle du `CLAUDE.md` : code de production) ; AC14 l'admet.
- **D6 — la cellule de date du journal d'audit reçoit `data-testid="audit-log-row-date"`** (V1-12) : AC11 la désigne sans dépendre de
  l'ordre des colonnes.

## Critères d'acceptation

1. **AC1 — étape 0 rejouée au JSON réel** : sur la base jetable `kesh_e506`, `curl` de `GET /api/v1/audit-log`, `GET /api/v1/bank-imports/{id}`
   (transaction rejetée), `GET /api/v1/admin/backup-schedule`, `GET /api/v1/admin/backups`, `GET /api/v1/admin/mcp/sends` **et** `GET /api/v1/admin/mcp/prompts/{id}` : les champs
   `createdAt`, `autoMatchRejectedAt`, `lastSuccessAt`/`lastAttemptAt`/`nextDueAt`, `modifiedAt` finissent par `Z` ; ceux des envois MCP
   (`createdAt`, `expiresAt`, `decidedAt`) et des prompts (`creeLe`, `modifieLe`) n'ont pas de fuseau. Sur une base neuve, l'envoi se
   dépose et le prompt se propose comme à `mcp-sends.spec.ts:118-134` et `mcp-prompts.spec.ts` (`proposer_prompt`), la liste étant vide
   sans cela (V2-9 ; le script de recette `kesh-gate-logs/e506/` le fait). Sorties dans `kesh-gate-logs/e506/ac1-json.txt`. Un écart arrête le développement et se dit à l'orchestrateur.
2. **AC2 — le jour suivant, l'hiver** : processus épinglé à `America/New_York` puis à `UTC` (`pinTimeZone`),
   `formatInstant('2026-11-24T23:30:00Z', 'dateHeure', 'fr-CH')` = `'25 nov. 2026, 00:30'` ; avec `'date'` = `'25 nov. 2026'` ; la forme
   naïve `'2026-11-24T23:30:00.123'` et la forme à neuf décimales `'2026-11-24T23:30:00.123456789Z'` donnent le même résultat ; en `de-CH`,
   `'25.11.2026, 00:30'`.
3. **AC3 — l'été et les deux bascules** : `'2026-07-14T22:30:00Z'` → `'15 juil. 2026, 00:30'` ; `'2026-03-29T00:59:00Z'` →
   `'29 mars 2026, 01:59'` et `'2026-03-29T01:00:00Z'` → `'29 mars 2026, 03:00'` ; `'2026-10-25T00:30:00Z'` et `'2026-10-25T01:30:00Z'` →
   tous deux `'25 oct. 2026, 02:30'` (l'heure répétée de l'automne, assumée).
4. **AC4 — l'absent et l'illisible** : `formatInstant(null|undefined|''|'abc'|'2026-13-45T99:99:99Z', …)` = `null`. Le **repli du
   journal d'audit** (V2-5 ; aujourd'hui la page lève un `RangeError` sur une date illisible) est la **valeur brute**, comme pour les envois
   MCP : `formatInstant(e.createdAt, 'dateHeure', i18nLocale()) ?? e.createdAt`.
5. **AC5 — journal d'audit** : `audit-log-page.test.ts`, processus à `America/New_York`, une entrée `createdAt: '2026-11-24T23:30:00.000Z'` →
   la cellule `audit-log-row-date` vaut `'25 nov. 2026, 00:30'` ; une entrée `createdAt: 'illisible'` rend la cellule `'illisible'`
   (repli, sans exception).
6. **AC6 — rejet de rapprochement** : `UnrejectTransactionAction.test.ts`, processus à `America/New_York`,
   `autoMatchRejectedAt: '2026-03-04T23:30:00Z'` → mention `'Rejetée le 5 mars 2026'` ; le test existant (`:90`, `'Rejetée le 4 mars 2026'`
   pour `11:00Z`) reste vert.
7. **AC7 — sauvegardes** : test de `backup-format.ts`, processus à `America/New_York` : `formatBackupDate('2026-11-24T23:30:00Z', 'fr-CH')` =
   `'25 nov. 2026, 00:30'` ; `formatBackupDate(null, …)` = `''`, `formatBackupDate('abc', …)` = `''` (contrat inchangé).
8. **AC8 — envois MCP** : test de `mcp-sends-format.ts`, processus à `America/New_York` : `dateLocale('2026-11-24T23:30:00.000')` =
   `'25 nov. 2026, 00:30'` ; `dateLocale(null)` = `'—'` ; `dateLocale('abc')` = `'abc'` (contrat inchangé).
9. **AC9 — rien ne bouge sur un poste suisse** : les mêmes entrées d'AC5 à AC8, processus à `Europe/Zurich`, rendent la même chose qu'avant
   la story (filet de non-régression, vert avant et après, déclaré comme tel). Les tests de `mcp-time.test.ts` (expiration) restent verts
   sans modification.
10. **AC10 — la garde (D3)** : les deux motifs relevés ; `ALLOWLIST_INSTANTS` close, une entrée par ligne relevée — **sauf** les deux lignes identiques de `premier-exercice.ts`, qui n'ont qu'**une**
    entrée à `lignes: 2` (V2-2) : les **deux** nombres s'écrivent, **relevés** (22 + n, ce que rend la commande de recompte) et **entrées**
    (21 + n, une de moins), au Dev Agent Record avec la sortie de la commande ; **sonde positive** en mémoire : les lignes d'avant la story
    (`dateFormat.format(new Date(e.createdAt))`, `new Intl.DateTimeFormat(i18nLocale(), { dateStyle: 'medium' })`, `const date = new Date(iso);`),
    une forme multiligne (`new Intl.DateTimeFormat(\n\tlocale,`) et `new globalThis.Date(x)` sont relevées sans entrée ; une troisième ligne
    identique à celles de `premier-exercice.ts` rougit. **Sonde réelle** : la ligne `:376` d'avant rétablie dans `audit-log/+page.svelte`
    fait rougir la garde (journal `kesh-gate-logs/e506/mut-M4.txt` ; M4 rétablit la mise en forme d'avant, sans fuseau, **en ligne** dans la cellule — variante équivalente de la ligne d'avant, dont la forme exacte est relevée par la sonde en mémoire).
11. **AC11 — un E2E** : `frontend/tests/e2e/audit-log.spec.ts` gagne un `describe` sous `test.use({ timezoneId: 'America/New_York' })` : une
    action qui écrit une entrée d'audit, la lecture de son `createdAt` par l'API, et l'assertion que `getByTestId('audit-log-row-date')` de
    la ligne égale `Intl.DateTimeFormat('fr-CH', { dateStyle: 'medium', timeStyle: 'short', timeZone: 'Europe/Zurich' }).format(new Date(createdAt))`
    **calculé dans le navigateur** par `page.evaluate` (l'ICU de Chromium, qui met la cellule en forme ; le corps d'un test tourne dans Node, dont
    l'ICU pourrait différer — V2-6) — et diffère de la même mise en forme à New York (`timeZone: 'America/New_York'`, aussi par `page.evaluate`). Il rougit au code d'avant (journal `rouge-e2e.txt` ; au code
    d'avant, le `data-testid` manque : le rouge se constate avec `locator('td').first()`, noté au journal).
12. **AC12 — manuel** : `user-manual.tex:4688-4689` dit, **en ces termes** (V2-3, texte fixé) : « Le tableau affiche l'heure de Zurich, quel que
    soit le fuseau de votre poste. \emph{Depuis la version 0.20.0~; avant, l'heure du poste.} » (apostrophe droite, sans `\textbf` sur `Zurich`) ; `:790-791` ajoute que les instants s'affichent à l'heure de Zurich,
    sauf les dates que l'on saisit (expiration d'une clé du serveur MCP, à l'heure du poste ; d'une clé d'API) et, pour l'instant, les dates de
    création, de dernier usage et de révocation des clés du serveur MCP et de la modification de ses réglages, encore à l'heure du poste.
    **Contrôles qui rougissent avant** (le texte est aplati, la phrase étant coupée) :
    `tr '\n' ' ' < docs/manual/fr/user-manual.tex | tr -s ' ' | grep -o "affiche l'heure de votre .textbf{poste}" | wc -l` → **1** à
    `e5654fc00`, **0** après ; `… | grep -o "affiche l'heure de Zurich, quel que soit le fuseau de votre poste" | wc -l` → **0** avant, **1**
    après (le texte aplati tient sur une ligne : `grep -c` ne rendrait que 0 ou 1 par ligne, `grep -o | wc -l` compte les occurrences).
    `admin-manual.tex:3584` inchangé. Aucun PDF commité ; à la release, après `make fr`,
    `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "heure de votre poste" | wc -l` → 0 (1 aujourd'hui).
13. **AC13 — recette visuelle** (règle du `CLAUDE.md`, chaque état touché) : captures, thème clair, base de démonstration `kesh_e506b` (la base `kesh_e2e_e506` est celle de l'E2E complet), de :
    - *Journal d'audit* (liste) ;
    - détail d'un import bancaire avec une transaction rejetée (« Rejetée le … ») ;
    - *Administration → Sauvegarde complète* : section automatique (dernière réussite, dernière tentative, prochaine) et liste des sauvegardes ;
    - le **bandeau** « Aucune sauvegarde automatique réussie depuis le … » : **voie principale, l'interception de route** de
      `backup-schedule.spec.ts` (`poser(page, etat({ overdue: true, lastSuccessAt: '2026-11-20T23:30:00.000Z' }))`, instant de fin de soirée
      UTC, pour que le jour change à Zurich), reproductible à coup sûr. Le repli `UPDATE _kesh_backup_schedule SET last_success_at =
      '2026-11-20 23:30:00.000'` sur la base jetable **ne tient pas** : le planificateur tourne toutes les cinq minutes
      (`auto_backup.rs:58`) et écrit une réussite neuve, qui éteint le bandeau (V2-4) ;
    - *Envois du serveur MCP* : la liste **et** le détail d'un envoi (créé, échéance, décision), envoi déposé comme dans `mcp-sends.spec.ts:118-134` ;
    - le détail d'un **prompt du serveur MCP** (`McpPromptDetail` : modification, versions proposées, **et une version décidée** : colonne *Décision* datée) ;
    - un **rejet de rapprochement daté de fin de soirée UTC** (interception de la réponse du détail d'import, `autoMatchRejectedAt: '2026-11-24T23:30:00Z'`) : « Rejetée le 24 nov. » avant, « 25 nov. » après, à New York (R1-5).
    Les chemins et numéros de ligne s'entendent **à la tête de `epic/50-corrections` au moment de la recette** : la 50-5 déplace l'écran
    `/admin/backup` vers `/backup-export?onglet=sauvegarder` (V2-1). Chacun **avant** (code de `e5654fc00`, navigateur à `America/New_York`) et **après** (même fuseau), plus **après** à `Europe/Zurich`
    (identique à l'avant suisse). Rangées dans `kesh-gate-logs/e506/captures/`, jointes au Dev Agent Record et au prompt de la revue de code.
14. **AC14 — périmètre tenu** : `git diff --stat e5654fc00..HEAD -- frontend/src ':!*.test.ts'` ne nomme que `reports.api.ts`,
    `audit-log/+page.svelte`, `UnrejectTransactionAction.svelte`, `backup-format.ts`, `mcp-sends-format.ts`, et `mcp-prompts-format.ts` pour
    ses seules lignes de commentaire (`git diff -U0` le montre) ; aucun fichier de `crates/`, aucune clé i18n neuve (`git diff` des quatre
    `.ftl` vide).

## Tests à écrire, et la preuve du rouge

Les tests se commitent **avant** le correctif (commit « test rouge »), journaux dans `kesh-gate-logs/e506/` :

| Test | Fichier | Rouge sans le correctif ? | Journal |
|---|---|---|---|
| AC2-AC4 | `lib/features/reports/format-instant.test.ts` (neuf) | oui : `formatInstant` n'existe pas (échec d'import), puis, sans `timeZone` (mutation M1), AC2/AC3 rougissent sous New York | `rouge-format-instant.txt`, `mut-M1.txt` |
| AC5 | `routes/(app)/audit-log/audit-log-page.test.ts` | oui : « 24 nov. 2026, 18:30 » | `rouge-audit.txt` |
| AC6 | `lib/features/reconciliation/UnrejectTransactionAction.test.ts` | oui : « Rejetée le 4 mars 2026 » | `rouge-unreject.txt` |
| AC7 | `lib/features/admin-backup/backup-format.test.ts` (neuf) | oui | `rouge-backup.txt` |
| AC8 | `lib/features/mcp-sends/mcp-sends-format.test.ts` (neuf, ou bloc d'un test existant du module) | oui | `rouge-mcp-sends.txt` |
| AC9 | les mêmes, sous `Europe/Zurich` | **non, voulu** : filet | — |
| AC10 | `lib/shared/utils/local-date-guard.test.ts` | oui : les huit lignes des quatre sites sont relevées sans entrée ; sonde réelle M4 | `rouge-garde.txt`, `mut-M4.txt` |
| AC11 | `tests/e2e/audit-log.spec.ts` | oui, au code d'avant | `rouge-e2e.txt` |
| AC12 | les deux `grep -c` du texte aplati | oui : 1 et 0 avant | `manuel-avant.txt`, `manuel-apres.txt` |

**Piège du `$derived`** (P3) : un formateur créé avant l'épinglage du fuseau ne se recalcule pas ; monter le composant **après** le
`beforeEach` de `pinTimeZone`.

**Mutations** (code commité avant ; restauration par `git restore --worktree` puis `touch`) : **M1** retirer `timeZone` de `formatInstant`
→ AC2, AC3, AC5-AC8 rougissent (journal `mut-M1-bis.txt` ; AC5 et AC6 n'y rougissent que parce que leurs fichiers de test épinglent le fuseau à New York **pour tout le fichier** : le formateur mis en cache naît sous New York, quels que soient la station et l'ordre des tests — revue P1, R1-1 ; sans cela, `mut-M1.txt`, M1 les laissait verts sur une station à Zurich) ; **M2** dans `instantDuServeur`, ne plus ajouter `Z` à une forme naïve → AC2 (forme naïve) et AC8 rougissent ;
**M3** `formatBackupDate` rendu à `new Date(iso)` + `Intl` sans zone → AC7 et la garde rougissent ; **M4** la ligne `:376` d'avant rétablie →
AC5 et la garde rougissent. Un journal par mutation.

## Tâches

1. **Étape 0** (AC1) : base `kesh_e506`, `curl`, journal ; arrêt si écart.
2. **Test rouge** : AC2-AC12 écrits ou constatés rouges, commit `test(50-6) : … (refs #1511)`, push de `story/50-6-…`.
3. **Correctif** : `ZONE_SUISSE`, `instantDuServeur`, cache et `formatInstant` dans `reports.api.ts`, `zurichParts` les appelle
   (`formatSwissDate`/`formatSwissDateTime` inchangés, leurs tests verts) ; les quatre sites convertis (D4) ; `data-testid` (D6) ;
   commentaires d'en-tête réécrits (`backup-format.ts:1-3`, `mcp-sends-format.ts:34-37`) et D5. Commit, push.
4. **Garde** (D3) : motifs, `lignes?`, allowlist, sondes, recompte ; commit, push.
5. **Manuel** (AC12) ; **propagation** : `git grep -nE "heure locale|l'heure \*\*locale|fuseau du navigateur|heure du poste"` sur
   `frontend/src`, `docs/`, et le grep du texte aplati de chaque `.tex` ; chaque site traité, ou laissé et dit (exception D0, limite I2).
6. **Mutations** M1-M4, journaux.
7. **Recette visuelle** (AC13).
8. **Gates** (ci-dessous), Dev Agent Record, statut `review`.

## Gates

- Pendant la boucle : `cd frontend && npm run check && npm run test:unit -- <fichiers touchés>` sous `scripts/mem-guard.sh`.
- Au dernier commit de code : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build` (sous `mem-guard`) ;
  **E2E complet** (`docs/testing.md` § *Prérequis Playwright local*, base `kesh_e2e_e506`) — vert = les seuls échecs attendus, jugés fichier
  par fichier. Aucun fichier de `crates/` : le gate Rust n'est pas requis par la story ; `scripts/gate-ci.sh` reste celui du push si la
  branche d'epic l'exige.

## Note de version

**Modifié** — Les dates et heures du journal d'audit, des sauvegardes (écran et bandeau), des envois et des prompts du serveur MCP, et la date
d'un rejet de rapprochement, s'affichent désormais à l'heure de Zurich, quel que soit le fuseau du poste (jusqu'ici : l'heure du poste), comme
celles des autres écrans et comme les jours que filtre le journal d'audit ; sur un poste réglé sur la Suisse, rien ne change. Les dates que
vous saisissez — l'expiration d'une clé du serveur MCP ou d'une clé d'API — restent telles que saisies ; à l'écran *Serveur MCP*, la création,
le dernier usage et la révocation des clés et la modification des réglages restent pour l'instant à l'heure du poste. (refs #1511)

## Dev Notes

### Modules touchés (règle de splitting) — cinq

Au grain du `CLAUDE.md` (un crate, ou un domaine `frontend/src/lib/features/<domaine>`, dont le **code de production** change ; V2-7) :
`lib/features/reports` (`reports.api.ts`), `lib/features/reconciliation`, `lib/features/admin-backup`, `lib/features/mcp-sends` : **quatre**.
La route `routes/(app)/audit-log/+page.svelte` n'est pas un domaine de `lib/features/` (la 50-5 applique le même grain) ; la compter par
prudence fait **cinq**, et le commentaire seul de `mcp-prompts-format.ts` (D5, hors code de production) un sixième lu au plus large — **la
conclusion tient sous les trois lectures** : quatre, cinq, ou six au plus large pour un commentaire, sans changement de règle. Tests (garde
comprise), fiche et manuel ne comptent pas. **Pas de découpage**, à condition de tenir D1 et D2 ; toute extension (shared, mcp-server)
ajoute un module et renvoie à I1/I2.

### Prémisses à confirmer à l'étape 0

- **P1** — les formes JSON réelles (AC1).
- **P2** — les chaînes d'ICU : les valeurs d'AC2/AC3/AC6 viennent de Node 22.22.1 (`fr-CH` medium = « 25 nov. 2026, 00:30 », bascules
  « 29 mars 2026, 01:59 » / « 03:00 », « 25 oct. 2026, 02:30 », `de-CH` = « 25.11.2026, 00:30 », date seule « 5 mars 2026 ») ; Vitest tourne
  sous le même Node ; l'E2E calcule son attendu par `Intl` **dans le navigateur** (`page.evaluate`, ICU de Chromium) ; les valeurs de Vitest viennent de l'ICU de Node.
- **P3** — `process.env.TZ` changé en cours de processus est relu par `Intl` (vérifié : 23:30 → 18:30 → 00:30 en changeant `TZ` dans le même
  processus) ; `pinTimeZone` (`lib/shared/utils/pin-time-zone.test-helper.ts`) suffit, avec le piège du `$derived`.

### Ce qui doit rester vrai

- Sur un poste à `Europe/Zurich`, les écrans touchés rendent **exactement** ce qu'ils rendaient (AC9, AC13).
- `formatSwissDate`, `formatSwissDateTime` et leurs 57 importeurs ne changent pas de comportement (tests existants verts sans modification).
- Les contrats de repli : `''` (sauvegardes, rejet), `—` et valeur brute (envois MCP).
- L'expiration d'une clé MCP : saisie et affichage au poste, aller-retour de `mcp-time.test.ts:66-89` (AC9).
- Le CSV du journal d'audit reste en UTC (manuel `:4724`) ; aucun code serveur ne change ; aucune migration.

### Issues à ouvrir (par l'orchestrateur ; aucune `gh` qui écrit ici)

- **I1** (dette de code) — déménager `ZONE_SUISSE`, `instantDuServeur`, `zurichParts`, `formatSwissDate`, `formatSwissDateTime`,
  `formatInstant` de `reports.api.ts` vers `lib/shared/utils/dates.ts` (réexport transitoire), repointer les importeurs, et faire passer la
  lecture de `settings/api-keys/+page.svelte:70-71` par `instantDuServeur` (dernière copie hors MCP).
- **I2** — écran *Serveur MCP* : afficher à Zurich ses **instants** (création et dernier usage d'une clé, révocation, modification des
  réglages : `McpKeysSection.svelte:403`, `:408`, `:412`, `McpSettingsSection.svelte:111`) par `formatInstant`, retirer `depuisUtcNaif` ;
  **ne pas** toucher l'expiration (date saisie : `finDeJourneeLocale`, `lendemainLocal`, son affichage par `dateAffichee`, `admin-manual.tex:3584`),
  ce qui oblige à séparer les deux usages de `dateAffichee`. Ferme la limite écrite par la note de version et le manuel.

### Fichiers communs avec les autres stories de l'Epic 50

À `e5654fc00`, la 48-11 est fusionnée dans `epic/50-corrections` ; le recouvrement de `reports.api.ts` qu'elle touchait est donc intégré.
**La 50-5** (`story/50-5-page-sauvegarde-export`) touche le même domaine `lib/features/admin-backup` (`AutoBackupSection.svelte`,
`BackupScheduleBanner.svelte` et leurs tests, mais **pas** `backup-format.ts`), `backup-schedule.spec.ts` (`:149`, `:222`, `:275`, dont
l'interception que la recette d'AC13 emprunte) et `admin-manual.tex:2509` (le lien du bandeau) : pas de conflit de texte attendu avec
la 50-6, mais le chemin de l'écran et les numéros de ligne changent si la 50-5 entre la première (V2-1) ; AC13 suit l'adresse en vigueur.
`docs/manual/fr/user-manual.tex` est commun à toutes les stories (sections distinctes) ; `local-date-guard.test.ts` n'est touché par aucune
autre story de l'epic à cette tête. À recontrôler à l'intégration, comme le veut le `CLAUDE.md`.

### Charge estimée

**Développement** : environ une journée d'agent — étape 0 et tests rouges 1 h 30 ; correctif 1 h ; garde, `lignes?` et sondes 2 h ; manuel et
mutations 1 h ; recette visuelle 1 h 30 ; E2E complet et gates 1 h 30. **Boucles** : la validation de la fiche (P1 faite, P2 à jouer, plus si
un MEDIUM revient) et deux ou trois passes de revue de code, chacune de 30 min à 1 h avec sa remédiation. Du mardi soir à la coupure du
vendredi matin, c'est tenable si la validation converge en P2 ou P3 ; sinon la story sort de la 0.20.0, comme décidé.

### Références

- Issue #1511 ; antécédents #880 (dates des écrans comptables, 0.17.2 : `formatSwissDate`, « naïf = UTC ») et #889 (date du jour en UTC,
  0.17.4 : `todayLocal`, `local-date-guard.test.ts`).
- `/home/gcorbaz/devel/kesh-gate-logs/dry-audit/rapport.md` § D1 (il laissait « à confirmer au payload » : l'étape 0 l'a fait).
- Rapport de la validation P1 : `/home/gcorbaz/devel/kesh-gate-logs/e506/validation-p1.md`.
- `CLAUDE.md` § *Qualité du code*, § *Recette visuelle*, § *E2E (Playwright)*, § *Règle de splitting préventif*, § *Review Iteration Rule*.

## Dev Agent Record

### Agent Model Used

Sonnet 5.5 (`bmad-dev-story`, autonomie), 2026-11-24. Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e506/`.

### Debug Log References

- **Étape 0 / AC1** : `ac1-json.txt` (base jetable `kesh_e506`, binaire `kesh-api` copié de `epic/50-corrections` à `e5654fc00` dans
  `e506/bin/` — aucun code serveur ne change ; script `backend.sh`). Quatre champs à `Z`, envois et prompts MCP naïfs : prémisse P1 confirmée.
- **Rouges** (commit `8c029046b`) : `rouge-format-instant.txt` (9 échecs sur 9), `rouge-audit.txt` (3 : la cellule sans `data-testid`
  rougit aussi le filet Zurich, attendu), `rouge-unreject.txt` (1), `rouge-backup.txt` (1), `rouge-mcp-sends.txt` (1), `rouge-garde.txt`
  (les 8 relevés des quatre sites sans entrée), `rouge-e2e.txt` (1) : (1) spec telle que commitée, le `data-testid` manque ; (2) variante
  `locator('td').first()`, cellule « 24 nov. 2026, 13:48 » (New York) au lieu de « 19:48 » (Zurich).
- **Mutations** (script `mut.sh`, restauration par `git restore --worktree` puis `touch`, arbre propre constaté) : `mut-M1.txt` (11 tests
  rouges, **mais AC5 et AC6 restaient verts** sur une station à Zurich : le formateur en cache, construit par un test non épinglé, était
  réemployé par les cas New York — revue P1, R1-1) puis **`mut-M1-bis.txt`** après épinglage du fuseau à New York pour tout le fichier de
  test d'`audit-log-page.test.ts` et d'`UnrejectTransactionAction.test.ts` : 15 tests rouges, dont **AC5 et AC6** (les filets Zurich
  rougissent aussi, le formateur en cache gardant New York) ; `reports.api.ts` restauré (`git restore --worktree`, `touch`), `mut-M2.txt` (5 : AC2 naïf,
  AC8, et `formatSwissDateTime` de `reports.api.test.ts`), `mut-M3.txt` (2 : AC7 et la garde), `mut-M4.txt` (3 : AC5, V2-5 et la garde ; M4 rétablit la mise en forme d'avant, sans fuseau, **en ligne** dans la cellule — une
  variante équivalente de la ligne d'avant, dont la forme exacte est relevée par la sonde en mémoire).
- **Manuel** : `manuel-avant.txt` (1 / 0 / PDF 1), `manuel-apres.txt` (0 / 1).
- **Recette visuelle** : `captures/{avant-ny,apres-ny,apres-zurich}/` (dix captures chacune, thème clair, base `kesh_e506b`, script
  `recette-scripts/recette.spec.ts`) ; les n° 09 (rejet de fin de soirée UTC : « Rejetée le 24 nov. » avant, « 25 nov. » après, à New York)
  et 10 (version de prompt décidée, colonne *Décision* datée) ont été ajoutées à la revue P1 (R1-5), et toutes les captures rejouées.
- **Gates** (voir « Completion Notes ») : `front-gate.log` ; `testfast.log` ; `e2e-complet.log`.

### Completion Notes List

- **AC1** établie (étape 0). **AC2-AC9** : `format-instant.test.ts` (neuf), `audit-log-page.test.ts`, `UnrejectTransactionAction.test.ts`,
  `backup-format.test.ts` (neuf), `mcp-sends-format.test.ts` (neuf), chacun avec un `describe` épinglé à New York (rouge avant) et un à
  `Europe/Zurich` (filet AC9, vert avant et après, déclaré tel).
- **Code** : `reports.api.ts` gagne `ZONE_SUISSE`, `instantDuServeur` (la seule lecture « naïf = UTC » du module, `zurichParts` l'appelle),
  `formatInstant` (formateurs en `Map` de module, clé `locale|style`) ; `formatSwissDate`/`formatSwissDateTime` inchangés. Quatre sites
  convertis (D4) ; le journal d'audit prend la valeur brute en repli (V2-5) et sa cellule porte `data-testid="audit-log-row-date"` (D6).
  Commentaires d'en-tête réécrits (`backup-format.ts`, `mcp-sends-format.ts`, `mcp-prompts-format.ts`, `UnrejectTransactionAction.svelte`,
  `audit-log/+page.svelte`, `UnrejectTransactionAction.test.ts`).
- **AC10 — garde** : `ALLOWLIST_INSTANTS` close. **Relevés : 22** (22 + n avec n = 0 : les trois lignes de `reports.api.ts` — `ZURICH_PARTS`,
  `Date.parse`, `formatToParts(new Date(ms))` — sont devenues `ZURICH_PARTS`, `new Date(` de `instantDuServeur`, constructeur du formateur de
  `formatInstant`). **Entrées : 21** (dont une à `lignes: 2`, `premier-exercice.ts`). Recompte, commande de la fiche (D3) rejouée au dernier
  commit de code : `cd frontend/src && grep -rnP "DateTimeFormat\(|toLocale(Date|Time)?String\(|…" … | wc -l` → **22** (30 avant ;
  sortie `garde-releves-apres.txt`), et `awk` sur `ALLOWLIST_INSTANTS` : 21 `fichier:`. Sonde positive en mémoire (sept lignes d'avant la
  story, multiligne, `globalThis`, troisième ligne identique de `premier-exercice.ts`) et sonde réelle M4 : vertes/rouges comme prévu.
- **AC11** : spec `audit-log.spec.ts` (bloc `New York`), attendus calculés par `page.evaluate` ; rouge constaté ; **vert à constater** dans l'E2E complet (voir
  ci-dessous ; rien n'est affirmé avant que son journal tombe).
- **AC12** : phrase du tableau fixée (« Le tableau affiche l'heure de Zurich, quel que soit le fuseau de votre poste. »), paragraphe de
  `:790-796` complété (exception et limite) ; contrôles du texte aplati 1 → 0 et 0 → 1. Aucun PDF commité ; après `make fr` à la release :
  `pdftotext … | grep -o "heure de votre poste" | wc -l` → 0 (1 aujourd'hui).
- **AC13** : recette visuelle faite (voir journaux), complétée à la revue P1 (R1-5). Bandeau par interception de route (V2-4). Les captures « après » à New York et à Zurich
  montrent la même heure (par exemple le bandeau : « 20 nov. 2026, 18:30 » avant, « 21 nov. 2026, 00:30 » après, aux deux fuseaux).
- **AC14** : `git diff --stat e5654fc00..HEAD -- frontend/src ':!*.test.ts'` ne nomme que `reports.api.ts`, `audit-log/+page.svelte`,
  `UnrejectTransactionAction.svelte`, `backup-format.ts`, `mcp-sends-format.ts`, et `mcp-prompts-format.ts` (une ligne de commentaire) ;
  aucun fichier de `crates/`, aucun `.ftl`.
- **Propagation (tâche 5)** : le grep du symptôme ne laisse que les sites voulus — `mcp-time.ts` (limite D0, I2) et son test, `api-keys/+page.svelte:65`
  (commentaire historique 0.17.2), `admin-manual.tex:3584` (expiration saisie), `crates/` (hors sujet), `CHANGELOG.md` (historique).
- **Gate frontend complet**, commit `5671704ce` (journal `front-gate.log`) : `npm run check` 0 erreur, `lint-i18n-ownership` vert, `test:unit`
  **294 fichiers / 5098 tests verts**, `npm run build` vert.
- **Gate Rust complet** (`scripts/test-fast.sh --ci`, base `kesh_e506`, verrous `station-heavy` puis `.gate.lock`) et **E2E complet** : **lancés,
  en attente des verrous de la voie** (journaux `testfast.log`, `e2e-complet.log`, scripts `testfast.sh`, `e2e-complet.sh`) ; **non terminés à
  l'écriture de cette ligne** — leurs résultats s'ajoutent ici, ligne par ligne, quand ils tombent.
- **Hors fiche, dit** : le backend de l'E2E et de la recette est le binaire de `epic/50-corrections` à `e5654fc00` (aucun code serveur ne change
  dans la story), copié dans `e506/bin/`, non reconstruit ; les scripts de recette sont dans `e506/recette-scripts/` (le dossier
  `frontend/.recette-e506/` du worktree est exclu par `.git/info/exclude`).

### File List

- `frontend/src/lib/features/reports/reports.api.ts` ; `frontend/src/lib/features/admin-backup/backup-format.ts` ;
  `frontend/src/lib/features/mcp-sends/mcp-sends-format.ts` ; `frontend/src/lib/features/mcp-prompts/mcp-prompts-format.ts` (commentaire) ;
  `frontend/src/lib/features/reconciliation/UnrejectTransactionAction.svelte` ; `frontend/src/routes/(app)/audit-log/+page.svelte`.
- Tests : `lib/features/reports/format-instant.test.ts` (neuf), `lib/features/admin-backup/backup-format.test.ts` (neuf),
  `lib/features/mcp-sends/mcp-sends-format.test.ts` (neuf), `lib/features/reconciliation/UnrejectTransactionAction.test.ts`,
  `routes/(app)/audit-log/audit-log-page.test.ts`, `lib/shared/utils/local-date-guard.test.ts`, `frontend/tests/e2e/audit-log.spec.ts`.
- `docs/manual/fr/user-manual.tex` (aucun PDF).
- Fiche et prompt de revue P1 : `_bmad-output/implementation-artifacts/50-6-dates-heure-de-zurich.md`, `…-review-prompt-p1.md`.

## Change Log

- **2026-11-24 — création** (Opus 5.5, `bmad-create-story`, autonomie) : fiche `drafted` ; étape 0 menée au code serveur — les quatre sites lisent
  le bon instant (trois champs portent `Z`, les envois MCP le reçoivent du code), la prémisse « UTC naïf lu en local » de #1511 est
  démentie ; reste l'affichage au fuseau du poste ; inventaire par le symptôme (`kesh-gate-logs/e506/inv-symptome.txt`, 68 lignes) ; cinq
  modules ; prompt de validation P1 versionné.
- **2026-11-24 — validation P1 remédiée** (P1 Sonnet : 0 CRITICAL / 0 HIGH / 5 MEDIUM / 9 LOW ; remédiation Opus 5.5). Décisions du Project
  Lead du 2026-11-24 écrites comme tranchées : D0 = l'heure de Zurich partout où l'on affiche un instant, story dans la 0.20.0, changement
  voulu (« Modifié ») ; exception des dates saisies (expiration des clés MCP et d'API), arbitrée par l'orchestrateur sur V1-3. Sort des
  findings : **V1-1** (MEDIUM) contrôle du manuel sur le texte aplati, 1 avant / 0 après (AC12) ; **V1-2** (MEDIUM) note « Modifié » ;
  **V1-3** (MEDIUM) exception et limite écrites (D0, D2, note, AC12), I2 réécrite aux seuls instants ; **V1-4** (MEDIUM) D3 réécrite : 30
  relevés recomptés (`garde-releves-avant.txt`), 22 entrées une par ligne, lignes `Date.UTC`/`new Date()` hors promesse, deux commentaires
  inscrits, champ `lignes?` pour les deux lignes identiques de `premier-exercice.ts` (relevé à la remédiation, non vu par la P1) ; **V1-5**
  (MEDIUM) AC13 complété (bandeau et sa provocation, détail d'un envoi, prompts) ; **V1-6** statut `ready-for-dev`, conditions levées ;
  **V1-7** phrase présente au PDF (ligature « ﬀ »), contrôle sur `heure de votre poste` ; **V1-8** D5, AC14 amendé ; **V1-9** copies
  restantes nommées, `api-keys:70-71` ajoutée à I1 ; **V1-10** cache des formateurs (D1) ; **V1-11** angles morts de la garde écrits,
  `globalThis` et multiligne sondés ; **V1-12** `data-testid` (D6) ; **V1-13** prescription sur #1511 retirée (requalification par
  l'orchestrateur) ; **V1-14** charge avec les boucles, mention périmée de la 48-11 retirée. Prompt de validation P2 versionné
  (`50-6-dates-heure-de-zurich-validation-prompt-p2.md`, Opus, passe complète).
- **2026-11-24 — étape 0 du développement** (Sonnet 5.5, `bmad-dev-story`, autonomie) : prémisses rejouées au code à `e5654fc00` — les 30
  relevés de la garde se recomptent à l'identique (`kesh-gate-logs/e506/garde-releves-etape0.txt`, même ensemble que `garde-releves-avant.txt`,
  seul l'ordre diffère) ; **P1 établie** par le JSON réel (`ac1-json.txt`) : quatre champs à `Z`, envois et prompts MCP naïfs, aucun écart.
  Sort des neuf LOW de la P2 : **V2-1** fiche 50-5 nommée aux fichiers communs, AC13 suit l'adresse en vigueur ; **V2-2** « 22 relevés, 21
  entrées » (AC10, D3), les deux nombres au Dev Agent Record ; **V2-3** phrase du manuel fixée en entier (AC12), contrôles `grep -o … | wc -l`
  (1 → 0, 0 → 1) ; **V2-4** interception de route voie principale pour le bandeau (AC13), l'`UPDATE` écrit comme ne tenant pas ; **V2-5** repli du
  journal = valeur brute (D4, AC4, AC5) ; **V2-6** attendu de l'E2E calculé par `page.evaluate` (AC11, P2) ; **V2-7** modules comptés au grain du
  `CLAUDE.md` (quatre, cinq ou six selon la lecture, conclusion inchangée) ; **V2-8** `window.Date`, `self.Date`, `new Date (` écrits aux angles
  morts, grep vide cité ; **V2-9** AC1 renvoie au dépôt d'un envoi et d'un prompt, prompts ajoutés.
- **2026-11-24 — développement** (Sonnet 5.5, `bmad-dev-story`) : étape 0 rejouée et P1 établie (JSON réel) ; neuf LOW de la P2 traités à
  l'étape 0 ; tests rouges commités (`8c029046b`), correctif (`88d7afe18`), garde (`5d233d0f4`), manuel (`5671704ce`) ; mutations M1 à M4
  rougies et restaurées ; recette visuelle faite ; gate frontend complet vert ; **gate Rust complet et E2E complet en attente des verrous
  de la voie** — le statut passe à `review` quand ils sont verts (le prompt de la revue P1, Opus, est versionné à côté de la fiche).
- **2026-11-24 — remédiation de la revue de code P1** (Sonnet 5.5 ; Opus : 0 CRITICAL / 0 HIGH / 1 MEDIUM / 5 LOW, rapport `e506/review-p1.md`),
  un seul commit, **aucun code de production** : **R1-1** (MEDIUM) fuseau épinglé à New York pour tout le fichier dans
  `audit-log-page.test.ts` et `UnrejectTransactionAction.test.ts` ; M1 rejouée (`mut-M1-bis.txt`) : AC5 et AC6 rougissent ; § Mutations et
  Dev Agent Record corrigés ; **R1-2** AC11 « vert à constater » tant que le journal de l'E2E n'est pas tombé ; **R1-3** journal cité
  `mut-M4.txt`, M4 dit variante en ligne ; **R1-4** angle mort `.getFullYear/.getMonth/.getDate/.getDay` écrit (garde et D3) ; **R1-5** captures 09
  (rejet de fin de soirée) et 10 (version décidée) ajoutées, base alignée (`kesh_e506b`) ; **R1-6** phrase de `user-manual.tex:790-799` en phrases
  courtes (« Kesh affiche tous les instants à l'heure de Zurich… », deux exceptions nommées). Gate ciblé (vitest des deux fichiers) vert, `npm run check` vert (`check-r1.log`). Prompt de la passe ciblée P2 : `…-review-prompt-p2.md`.
