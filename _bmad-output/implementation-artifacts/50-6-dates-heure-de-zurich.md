# Story 50-6 : Dates et heures de Zurich — journal d'audit, sauvegardes, envois MCP, rejet de rapprochement (#1511)

Status: drafted

*Fiche rédigée le 2026-11-24 (Opus 5.5), worktree `kesh-50-6`, branche `story/50-6-dates-heure-de-zurich` partie de
`origin/epic/50-corrections` à `e5654fc00`, par le workflow `bmad-create-story`, en autonomie. Epic 50 « Corrections », cinquième story ;
release visée **0.20.0** (coupure le vendredi 2026-11-27 au matin ; si elle n'est pas close à la coupure, elle sort de la release sans la
retarder — décision du Project Lead du 2026-11-24). **Aucune migration**, aucun code serveur touché. Statut `drafted` et non
`ready-for-dev` : **l'étape 0, menée dès la rédaction, dément la prémisse de l'issue** (§ *Étape 0*), et la story ne vaut que si le
Project Lead tranche D0 (Q1).*

## Story

En tant qu'utilisateur qui relit la piste de contrôle (journal d'audit, sauvegardes, envois du serveur MCP, rejets de rapprochement),
je veux que chaque date et chaque heure s'affichent **à l'heure de Zurich**, comme celles des autres écrans de Kesh et comme les jours
que filtre le journal d'audit, quel que soit le fuseau de mon poste, afin qu'une opération faite peu après minuit en Suisse ne paraisse
pas à la veille ni à une autre heure selon la machine qui l'affiche.

## Étape 0 — les horodatages sont-ils naïfs ? (lue au code à `e5654fc00`)

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
décimales (`modifiedAt`, tiré d'un `SystemTime`, en porte jusqu'à neuf). Les tests existants le confirment : `UnrejectTransactionAction.test.ts:72`
`autoMatchRejectedAt: '2026-03-04T11:00:00Z'`, `backup-schedule.api.test.ts:37` `lastSuccessAt: '2026-10-24T01:00:00.000Z'`,
`audit-log-page.test.ts:68` `createdAt: '2026-09-16T12:26:33.123Z'`. Le JSON réel n'a pas été relu (aucun serveur lancé à la rédaction) :
c'est la prémisse **P1**, à confirmer au développement par `curl` (§ *Prémisses à confirmer*).

Lecture JS vérifiée (Node 22.22.1, poste à `Europe/Zurich`) : `Date.parse("2026-11-24T22:15:00.123456789Z")` → `22:15:00.123Z` (les neuf
décimales passent) ; `"2026-11-24T22:15:00.123"` (naïf) → `21:15:00.123Z` (lu local : c'est le défaut que l'issue décrit, **absent** des quatre
sites).

**Ce qui reste vrai du défaut.** Les quatre sites lisent le bon instant, mais le **formatent dans le fuseau du poste**
(`Intl.DateTimeFormat` sans `timeZone` : `audit-log/+page.svelte:65`, `UnrejectTransactionAction.svelte:37`, `backup-format.ts:10`,
`mcp-sends-format.ts:43`). Sur un poste réglé sur la Suisse (celui du Project Lead, et le navigateur des specs : `playwright.config.ts:59`
`timezoneId: 'Europe/Zurich'`), l'affichage est **juste**. Il diverge seulement sur un poste réglé ailleurs (voyage, machine ou VM en UTC) :
`2026-11-24T23:30:00Z` s'y lit « 24 nov. 2026, 23:30 » (UTC) ou « 24 nov. 2026, 18:30 » (New York) au lieu de « 25 nov. 2026, 00:30 ».
Or les autres écrans affichent déjà Zurich (`formatSwissDate`/`formatSwissDateTime`, `reports.api.ts:258-273` : utilisateurs, clés d'API,
factures, rapports), et **les filtres du journal d'audit choisissent des jours de Zurich** (0.18.0) : sur un poste en UTC, une entrée de
23 h 30 UTC paraît datée du 24 sous le filtre du 25. C'est une incohérence, non une piste fausse en Suisse.

**Le manuel le décrit tel quel, comme un choix** : `docs/manual/fr/user-manual.tex:4688-4689` « Le tableau affiche l'heure de votre
\textbf{poste} --- pour un poste réglé sur la Suisse, la même. » (journal d'audit). Le doc-comment de `mcp-time.ts:28` dit de même « Date et
heure dans le fuseau du navigateur ».

**Conséquences, à trancher par l'orchestrateur et le Project Lead (Q1, Q2)** : la priorité P2 (« piste de contrôle incomplète ») ne
tient pas — rien n'est faux sur un poste suisse ; le défaut relève de **P3** (incohérence d'affichage, duplication de la lecture des
instants). Passer à Zurich **change un comportement documenté** : c'est D0.

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
  du poste** (`:32`, `:39`, sans `timeZone`). Ce n'est pas un site « qui traite déjà le défaut » au sens de l'issue : il lit juste et affiche au
  poste, comme les quatre sites.
- `lib/shared/utils/local-date.ts:21` `const JOUR_DE_ZURICH = new Intl.DateTimeFormat('en-CA', {` — le jour de Zurich (`todayInZurich`).

### Les quatre sites de l'issue

- `routes/(app)/audit-log/+page.svelte:64-66` `let dateFormat = $derived(` `new Intl.DateTimeFormat(i18nLocale(), { dateStyle: 'medium', timeStyle: 'short' }),`
  puis `:376` `<td class="py-2 whitespace-nowrap">{dateFormat.format(new Date(e.createdAt))}</td>`.
- `lib/features/reconciliation/UnrejectTransactionAction.svelte:37` `const dateFormat = $derived(new Intl.DateTimeFormat(i18nLocale(), { dateStyle: 'medium' }));`,
  `:38-40` `const rejectedOn = $derived(` `tx.autoMatchRejectedAt != null ? dateFormat.format(new Date(tx.autoMatchRejectedAt)) : '',`.
- `lib/features/mcp-sends/mcp-sends-format.ts:38` `export function dateLocale(valeur: string | null): string {` (`:39` `—` si vide, `:42` valeur
  brute si illisible), appelée par `McpSendsList.svelte:98-99`, `McpSendDetail.svelte:195`, `:197`, `:205` **et** `mcp-prompts/McpPromptDetail.svelte:350`,
  `:427`, `:508`, `:514` (les prompts MCP changent donc aussi, sans que leur module soit touché).
- `lib/features/admin-backup/backup-format.ts:6` `export function formatBackupDate(iso: string | null, locale: string): string {` (`''` si absent
  ou illisible), appelée par `AutoBackupSection.svelte:411`, `:435`, `:446`, `:535` et `BackupScheduleBanner.svelte:39`.

### Inventaire par le symptôme (tout `frontend/src`, hors tests)

Commandes et sortie complète : `/home/gcorbaz/devel/kesh-gate-logs/e506/inv-symptome.txt` (68 lignes) — `git grep` de `new Date(`,
`toLocale*String`, `.getHours/Minutes/Seconds/TimezoneOffset(`, `.getFullYear/Month/Date/Day(`, `DateTimeFormat`, `Date.parse`, hors
`*.test.ts`, `*.spec.ts`, `*.test-helper.ts`. R2 (`toLocale…String`) et R3 (heures, minutes, décalage) : **aucune ligne**. Aucune
bibliothèque de dates (`date-fns`, `dayjs`, `luxon`…) dans `package.json`. Classement :

**En défaut (fuseau du poste à l'affichage d'un instant du serveur) — dans le périmètre**

| # | Site | Pourquoi |
|---|---|---|
| E1 | `audit-log/+page.svelte:65`, `:376` | `Intl` sans `timeZone` ; `createdAt` porte `Z` |
| E2 | `UnrejectTransactionAction.svelte:37`, `:39` | idem, date seule |
| E3 | `mcp-sends-format.ts:40-43` | `Z` ajouté, `Intl` sans `timeZone` (couvre aussi les prompts MCP) |
| E4 | `backup-format.ts:8-10` | `Intl` sans `timeZone` ; champs `DateTime<Utc>` |

**En défaut, hors périmètre (même classe, autre module) — issue à ouvrir (I2)**

| # | Site | Pourquoi hors périmètre |
|---|---|---|
| H1 | `mcp-server/mcp-time.ts:24`, `:32`, `:39` (`dateHeureAffichee`, `dateAffichee` : clés MCP, réglages MCP) | sixième module (D1) ; choix écrit au doc-comment (`:28`, `:35` « fuseau du navigateur ») ; sa lecture `depuisUtcNaif` est juste |

**Justes**

| Site | Raison |
|---|---|
| `reports.api.ts:222`, `:240`, `:243` | `timeZone: 'Europe/Zurich'` ; naïf lu UTC — le module de référence |
| `local-date.ts:13-16` (`todayLocal`), `:21` (`JOUR_DE_ZURICH`), `:37`, `:46` | date du **poste** voulue pour une saisie (garde `local-date-guard.test.ts`) ; Zurich pour une borne ; `Date.UTC` sur une date pure |
| `settings/api-keys/+page.svelte:72` | `new Date(utc).getTime() < Date.now()` : comparaison d'instants, `Z` ajouté `:71` ; indépendante du fuseau |
| `users/+page.svelte:276` | `isNaN(new Date(iso).getTime())` : test de validité seul ; l'affichage passe par `formatSwissDate` |
| `journal-entries/form-helpers.ts:82` | `` new Date(`${iso}T00:00:00Z`) `` : date pure, arithmétique UTC |
| `fiscal-years.helpers.ts:31-32` | `new Date(input.startDate)` sur `AAAA-MM-JJ` : lu UTC minuit par la norme, comparé à l'autre — ordre seul |
| `fiscal-years.helpers.ts:52`, `onboarding/premier-exercice.ts:18-31`, `mcp-time.ts:49`, `:54-58` | année / date **du poste** voulue (saisie, fin de journée locale de l'expiration choisie) |
| `exchange-rates-grid.ts:167-195`, `homepage-charts.ts:206-216`, `bank-import/manual-statement.ts:106`, `invoice-helpers.ts:123`, `journal-entries/fx.ts:163`, `shared/utils/date-filtre.ts:23` | `Date.UTC` et `timeZone: 'UTC'` sur des dates pures ou des mois : rien ne glisse |
| `reminders/reminder-sent-at.ts:28` | `now: Date = new Date()` : instant courant, envoyé en ISO UTC |

**Hors sujet** : `invoice-helpers.ts:116`, `mcp-time.ts:4`, `users/+page.svelte:272`, `i18n.svelte.ts:14` (commentaires) ;
`pin-clock.test-helper.ts:21` (test). Les specs E2E tournent sous `timezoneId: 'Europe/Zurich'` (gardées par `local-date-guard.test.ts`,
volet specs) ; le serveur (CSV de l'audit en UTC, `audit_log.rs:373-375`, documenté au manuel `:4724`) n'est pas un affichage d'écran.

**Angles morts assumés de l'inventaire** : un instant formaté par une fonction importée d'un autre module sans passer par `new Date(`
ni `DateTimeFormat` sur la même ligne (`d.toISOString()` affiché brut, aucun relevé aujourd'hui) ; un affichage brut d'une chaîne
d'horodatage (`{x.createdAt}` dans un gabarit) — relevé par un second `grep` sur les gabarits Svelte (`\{[^}]*[a-z](At|Le)\b[^}]*\}`
hors formateurs) : aucun affichage brut, seulement des tests `{#if …At}`.

### Manuels

- `docs/manual/fr/user-manual.tex:4686-4691` (journal d'audit, filtres et tableau) : la phrase « Le tableau affiche l'heure de votre
  \textbf{poste} --- pour un poste réglé sur la Suisse, la même. » devient **fausse** avec la story.
- `user-manual.tex:790-791` « Les \textbf{heures} enregistrées (journal d'audit, envoi d'un rappel) restent en UTC » : vraie du
  stockage, reste vraie ; à compléter de l'affichage.
- `user-manual.tex:3661` (« Rejetée le~\dots »), `admin-manual.tex:2507` (« Ce que l'écran montre » de la sauvegarde automatique),
  `:2509` (bandeau « depuis le … »), § `sec:mcp-envois` : **aucun fuseau nommé** — rien ne devient faux.
- `admin-manual.tex:2717`, `:2745` : notes des versions 0.18.0 et 0.17.7, historiques — inchangées.
- Seul le français existe (`docs/manual/{de,en,it}/` ne portent qu'un `README.md`). Le PDF FR (`pdftotext … | tr '\n' ' ' | tr -s ' '`)
  porte « Les heures enregistrées (journal d'audit, envoi d'un rappel) restent en UTC » ; la phrase du tableau n'y a pas été retrouvée
  telle quelle (césure probable) : la validation le contrôle. Les PDF ne se régénèrent pas dans la story (règle des fichiers communs).

## Décisions

- **D0 — le fuseau d'affichage d'un instant est celui de Zurich, partout** (sous réserve de Q1). Motif : cohérence avec les écrans qui
  affichent déjà Zurich et avec les jours de Zurich des filtres de l'audit ; une piste de contrôle se lit pareil sur toute machine.
  *Écartée* : garder le fuseau du poste (comportement documenté de la 0.18.0) et fermer #1511 comme non reproductible en Suisse, en
  n'ouvrant qu'une dette de duplication. Si Q1 retient cette option, la story est retirée (statut `superseded`) et rien n'est écrit.
- **D1 — un seul formateur d'instants, `formatInstant`, logé à côté de `zurichParts` dans `lib/features/reports/reports.api.ts`.** Il
  reprend la lecture de `zurichParts` (une fonction privée `instantDuServeur(iso): Date | null` extraite de `:237-241`, que `zurichParts`
  appelle désormais : une seule regex, un seul « naïf = UTC ») et formate par `Intl.DateTimeFormat(locale, { …style, timeZone: 'Europe/Zurich' })`,
  la zone étant une constante unique `ZONE_SUISSE` partagée avec `ZURICH_PARTS`. Signature :
  `formatInstant(iso: string | null | undefined, style: 'date' | 'dateHeure', locale: string): string | null` — `date` =
  `{ dateStyle: 'medium' }`, `dateHeure` = `{ dateStyle: 'medium', timeStyle: 'short' }` (les styles actuels des quatre sites : **aucun
  changement visible sur un poste suisse**) ; `null` si absent ou illisible, chaque appelant garde son repli (`''`, `—`, valeur brute).
  *Écartée 1* : un module neuf `lib/shared/utils/dates.ts` où déménageraient `zurichParts`, `formatSwissDate`, `formatSwissDateTime`
  (réexportés par `reports.api.ts`) — c'est la bonne place à terme, mais elle fait **six modules** (shared, reports, audit-log,
  reconciliation, admin-backup, mcp-sends) et déclenche la règle de splitting ; le déménagement, pur (aucun comportement), se fait dans une
  story de dette à part (**I1**). *Écartée 2* : formater les quatre sites par `formatSwissDateTime` (forme fixe `JJ.MM.AAAA HH:mm`) — change
  la forme visible des quatre écrans en Suisse, ce que la story ne demande pas. *Écartée 3* : un `formatInstant` neuf avec sa propre
  regex — une troisième copie de la lecture (avec `zurichParts` et `depuisUtcNaif`).
- **D2 — `mcp-time.ts` (écran *Serveur MCP*) reste hors périmètre** : sixième module ; ouvrir **I2** (aligner `dateHeureAffichee` /
  `dateAffichee` sur `formatInstant`, retirer `depuisUtcNaif` ou le faire déléguer). D'ici là, les clés MCP affichent l'heure du poste et les
  envois MCP celle de Zurich ; le manuel ne nommant pas le fuseau des clés, rien n'y devient faux, et l'écart se dit dans l'issue I2. *Écartée* :
  l'inclure (6 modules).
- **D3 — la garde statique étend `lib/shared/utils/local-date-guard.test.ts`**, qui tient déjà « l'inventaire clos des dates du jour et des
  affichages d'instants » avec son auditeur (`relever`, `auditer`, allowlist `{ fichier, fragment, motif }`, une entrée ↔ un relevé). Deux
  motifs neufs, relevés par ligne dans toutes les sources hors tests :
  - `MOTIF_FORMAT_D_INSTANT` = `/DateTimeFormat\(|toLocale(Date|Time)?String\(|\.get(Hours|Minutes|Seconds|TimezoneOffset)\(/` ;
  - `MOTIF_LECTURE_D_INSTANT` = `/new Date\((?!\)|Date\.UTC\()|Date\.parse\(/` (une construction depuis un argument autre que rien ou `Date.UTC`).
  L'allowlist neuve `ALLOWLIST_INSTANTS` est **close** : chaque ligne « juste » de l'inventaire y entre avec son motif, `mcp-time.ts` avec
  « hors périmètre, I2 », et `reports.api.ts` (le module) avec « formateur de référence ». Après le correctif, **aucune** ligne des quatre
  sites n'y figure : `formatInstant` est le seul chemin. Le fichier de garde est un test : il ne compte pas comme module (règle du
  `CLAUDE.md`). *Écartée* : la garde de l'issue (« `new Date(` sur un champ `*At` ») — elle part d'une liste de noms : `backup-format.ts:8`
  lit `new Date(iso)` et lui échappait, et `Intl` sans `timeZone`, le vrai symptôme, n'y était pas.
- **D4 — `mcp-sends-format.ts` garde `dateLocale`** (nom et contrat : `—` si vide, valeur brute si illisible), réécrit en une ligne sur
  `formatInstant(valeur, 'dateHeure', i18nLocale()) ?? valeur` ; son commentaire (`:34-37` « à l'heure **locale**, comme le journal
  d'audit ») est réécrit. Pareil pour `formatBackupDate` (`?? ''`). Les appelants ne changent pas. *Écartée* : renommer (`dateLocale` →
  …) — touche `mcp-prompts` (un module de plus) pour un nom.

## Critères d'acceptation

1. **AC1 — étape 0 rejouée au JSON réel** : sur la base jetable `kesh_e506`, `curl` de `GET /api/v1/audit-log`, `GET /api/v1/bank-imports/{id}`
   (transaction rejetée), `GET /api/v1/admin/backup-schedule`, `GET /api/v1/admin/backups`, `GET /api/v1/admin/mcp/sends` : les champs
   `createdAt`, `autoMatchRejectedAt`, `lastSuccessAt`/`lastAttemptAt`/`nextDueAt`, `modifiedAt` finissent par `Z` ; ceux des envois MCP
   n'ont pas de fuseau. Sorties copiées dans `kesh-gate-logs/e506/ac1-json.txt`, citées au Dev Agent Record. Un écart avec le tableau de
   l'étape 0 arrête le développement et se dit à l'orchestrateur.
2. **AC2 — le jour suivant, l'hiver** : processus épinglé à `America/New_York` puis à `UTC` (`pinTimeZone`),
   `formatInstant('2026-11-24T23:30:00Z', 'dateHeure', 'fr-CH')` = `'25 nov. 2026, 00:30'` ; avec `'date'` = `'25 nov. 2026'` ; la forme
   naïve `'2026-11-24T23:30:00.123'` et la forme à neuf décimales `'2026-11-24T23:30:00.123456789Z'` donnent le même résultat ; en `de-CH`,
   `'25.11.2026, 00:30'`.
3. **AC3 — l'été et les deux bascules** : `'2026-07-14T22:30:00Z'` → `'15 juil. 2026, 00:30'` ; `'2026-03-29T00:59:00Z'` →
   `'29 mars 2026, 01:59'` et `'2026-03-29T01:00:00Z'` → `'29 mars 2026, 03:00'` ; `'2026-10-25T00:30:00Z'` et `'2026-10-25T01:30:00Z'` →
   tous deux `'25 oct. 2026, 02:30'` (l'heure répétée de l'automne, assumée).
4. **AC4 — l'absent et l'illisible** : `formatInstant(null|undefined|''|'abc'|'2026-13-45T99:99:99Z', …)` = `null`.
5. **AC5 — journal d'audit** : `audit-log-page.test.ts`, processus à `America/New_York`, une entrée `createdAt: '2026-11-24T23:30:00.000Z'` →
   la première cellule de la ligne vaut `'25 nov. 2026, 00:30'`.
6. **AC6 — rejet de rapprochement** : `UnrejectTransactionAction.test.ts`, processus à `America/New_York`,
   `autoMatchRejectedAt: '2026-03-04T23:30:00Z'` → mention `'Rejetée le 5 mars 2026'` ; le test existant (`:90`, `'Rejetée le 4 mars 2026'`
   pour `11:00Z`) reste vert.
7. **AC7 — sauvegardes** : test de `backup-format.ts`, processus à `America/New_York` : `formatBackupDate('2026-11-24T23:30:00Z', 'fr-CH')` =
   `'25 nov. 2026, 00:30'` ; `formatBackupDate(null, …)` = `''`, `formatBackupDate('abc', …)` = `''` (contrat inchangé).
8. **AC8 — envois MCP** : test de `mcp-sends-format.ts`, processus à `America/New_York` : `dateLocale('2026-11-24T23:30:00.000')` =
   `'25 nov. 2026, 00:30'` ; `dateLocale(null)` = `'—'` ; `dateLocale('abc')` = `'abc'` (contrat inchangé).
9. **AC9 — rien ne bouge sur un poste suisse** : les mêmes entrées d'AC5 à AC8, processus à `Europe/Zurich`, rendent la même chose
   qu'avant la story (tests verts avant et après : filet de non-régression déclaré comme tel, non comme preuve du correctif).
10. **AC10 — la garde (D3)** : `local-date-guard.test.ts` relève les deux motifs neufs ; l'allowlist `ALLOWLIST_INSTANTS` est close (entrée
    orpheline, relevé sans entrée, fragment à deux lignes rougissent) ; **sonde positive** en mémoire : les lignes d'avant la story
    (`dateFormat.format(new Date(e.createdAt))`, `new Intl.DateTimeFormat(i18nLocale(), { dateStyle: 'medium' })`, `const date = new Date(iso);`)
    sont relevées sans entrée. **Sonde réelle** : la ligne `:376` d'avant rétablie dans `audit-log/+page.svelte` fait rougir la garde
    (journal `kesh-gate-logs/e506/mut-garde.txt`).
11. **AC11 — un E2E** : `frontend/tests/e2e/audit-log.spec.ts` gagne un test sous `test.use({ timezoneId: 'America/New_York' })` : une
    action qui écrit une entrée d'audit, la lecture de son `createdAt` par l'API, et l'assertion que la cellule de date de la ligne égale
    `Intl.DateTimeFormat('fr-CH', { dateStyle: 'medium', timeStyle: 'short', timeZone: 'Europe/Zurich' })` de cet instant (calculé dans la
    spec) — et diffère de la même mise en forme à New York. Il rougit au code d'avant (journal).
12. **AC12 — manuel** : `user-manual.tex:4688-4689` dit que le tableau affiche l'heure de Zurich, quel que soit le fuseau du poste, avec
    « \emph{Depuis la version 0.20.0~; avant, l'heure du poste.} » ; `:790-791` ajoute que ces heures s'affichent à l'heure de Zurich.
    `git grep -n "heure de votre" docs/manual/fr` ne rend plus la phrase du tableau. Aucun PDF commité.
13. **AC13 — recette visuelle** (règle du `CLAUDE.md`) : captures, thème clair, base de démonstration `kesh_e2e_e506`, des quatre écrans —
    *Journal d'audit*, détail d'un import bancaire avec une transaction rejetée (« Rejetée le … »), *Administration → Sauvegarde complète*
    (section automatique et liste), *Envois du serveur MCP* (liste) —, chacun **avant** (code d'avant, navigateur à `America/New_York`) et
    **après** (même fuseau), plus **après** à `Europe/Zurich` (identique à l'avant suisse). Rangées dans `kesh-gate-logs/e506/captures/`,
    jointes au Dev Agent Record et au prompt de la revue de code.
14. **AC14 — périmètre tenu** : `git diff --stat e5654fc00..HEAD -- frontend/src ':!*.test.ts'` ne nomme que `reports.api.ts`,
    `audit-log/+page.svelte`, `UnrejectTransactionAction.svelte`, `backup-format.ts`, `mcp-sends-format.ts` ; aucun fichier de `crates/`,
    aucune clé i18n neuve (`git diff` des quatre `.ftl` vide).

## Tests à écrire, et la preuve du rouge

Les tests se commitent **avant** le correctif (commit « test rouge »), journaux dans `kesh-gate-logs/e506/` :

| Test | Fichier | Rouge sans le correctif ? | Journal |
|---|---|---|---|
| AC2-AC4 | `lib/features/reports/format-instant.test.ts` (neuf) | oui : `formatInstant` n'existe pas (échec d'import), puis, la fonction écrite **sans** `timeZone` (mutation M1), AC2/AC3 rougissent sous New York | `rouge-format-instant.txt`, `mut-M1.txt` |
| AC5 | `routes/(app)/audit-log/audit-log-page.test.ts` | oui : « 24 nov. 2026, 18:30 » | `rouge-audit.txt` |
| AC6 | `lib/features/reconciliation/UnrejectTransactionAction.test.ts` | oui : « Rejetée le 4 mars 2026 » | `rouge-unreject.txt` |
| AC7 | `lib/features/admin-backup/backup-format.test.ts` (neuf) | oui | `rouge-backup.txt` |
| AC8 | `lib/features/mcp-sends/mcp-sends-format.test.ts` (neuf, ou bloc d'un test existant du module) | oui | `rouge-mcp-sends.txt` |
| AC9 | les mêmes, sous `Europe/Zurich` | **non, voulu** : filet | — |
| AC10 | `lib/shared/utils/local-date-guard.test.ts` | oui : les quatre sites d'avant sont relevés sans entrée ; sonde réelle M4 | `rouge-garde.txt`, `mut-garde.txt` |
| AC11 | `tests/e2e/audit-log.spec.ts` | oui, au code d'avant | `rouge-e2e.txt` |

**Mutations** (code commité avant ; restauration par `git restore --worktree` puis `touch`) : **M1** retirer `timeZone` de `formatInstant`
→ AC2, AC3, AC5-AC8 rougissent ; **M2** dans `instantDuServeur`, ne plus ajouter `Z` à une forme naïve → AC2 (forme naïve) et AC8 rougissent ;
**M3** `formatBackupDate` rendu à `new Date(iso)` + `Intl` sans zone → AC7 et la garde rougissent ; **M4** la ligne `:376` d'avant rétablie →
AC5 et la garde rougissent. Un journal par mutation.

## Tâches

1. **Étape 0** (AC1) : base `kesh_e506`, `curl`, journal ; arrêt si écart.
2. **Test rouge** : AC2-AC11 écrits, rouges constatés, commit `test(50-6) : … (refs #1511)`, push de `story/50-6-…`.
3. **Correctif** : `instantDuServeur` et `ZONE_SUISSE` extraits dans `reports.api.ts`, `zurichParts` les appelle (`formatSwissDate`/`formatSwissDateTime`
   inchangés, leurs tests existants verts) ; `formatInstant` ; les quatre sites convertis (D4) ; commentaires d'en-tête réécrits
   (`backup-format.ts:1-3`, `mcp-sends-format.ts:34-37`). Commit, push.
4. **Garde** (D3) et sondes ; commit, push.
5. **Manuel** (AC12) ; **propagation** : `git grep -n "heure locale\|l'heure \*\*locale\|fuseau du navigateur\|heure de votre poste"` sur
   `frontend/src`, `docs/`, fiches de l'epic — chaque site traité ou dit hors périmètre (I2).
6. **Mutations** M1-M4, journaux.
7. **Recette visuelle** (AC13).
8. **Gates** (ci-dessous), Dev Agent Record, statut `review`.

## Gates

- Pendant la boucle : `cd frontend && npm run check && npm run test:unit -- <fichiers touchés>` sous `scripts/mem-guard.sh`.
- Au dernier commit de code : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build` (sous `mem-guard`) ;
  **E2E complet** (`docs/testing.md` § *Prérequis Playwright local*, base `kesh_e2e_e506`) — vert = les seuls échecs attendus,
  jugés fichier par fichier ; le gate Rust n'est pas requis (aucun fichier de `crates/`), mais `scripts/gate-ci.sh` reste celui du push si
  la branche d'epic l'exige.

## Note de version

**Corrigé** — Les dates et heures du journal d'audit, des sauvegardes (écran et bandeau), des envois et des prompts du serveur MCP, et la
date d'un rejet de rapprochement, s'affichent désormais à l'heure de Zurich, quel que soit le fuseau du poste, comme celles des autres
écrans et comme les jours que filtre le journal d'audit ; sur un poste réglé sur la Suisse, rien ne change. (refs #1511)

## Dev Notes

### Modules touchés (règle de splitting) — cinq

`lib/features/reports` (`reports.api.ts`), domaine *audit-log* (`routes/(app)/audit-log/+page.svelte` — compté comme module bien qu'hors de
`lib/features/`, par prudence), `lib/features/reconciliation`, `lib/features/admin-backup`, `lib/features/mcp-sends`. Tests (garde comprise),
fiche et manuel ne comptent pas. `mcp-prompts` change d'affichage sans changer de code. **Cinq : pas de découpage**, à condition de tenir D1
et D2 ; toute extension (shared, mcp-server) fait six et renvoie à I1/I2.

### Prémisses à confirmer à l'étape 0

- **P1** — les formes JSON réelles (AC1).
- **P2** — les chaînes d'ICU : les valeurs d'AC2/AC3 viennent de Node 22.22.1 (`fr-CH` medium = « 25 nov. 2026, 00:30 », `de-CH` =
  « 25.11.2026, 00:30 ») ; Vitest tourne sous le même Node, mais l'E2E (Chromium) peut différer d'un séparateur : c'est pourquoi AC11
  calcule l'attendu par `Intl` dans la spec au lieu d'une chaîne figée.
- **P3** — `process.env.TZ` changé en cours de processus est relu par `Intl` (vérifié : 23:30 → 18:30 → 00:30 en changeant `TZ` dans le
  même processus) ; `pinTimeZone` (`lib/shared/utils/pin-time-zone.test-helper.ts`) suffit. Un `$derived` créé **avant** l'épinglage ne
  se recalcule pas : monter le composant **après** `beforeEach`.
- **P4** — D0 tranchée par le Project Lead (Q1).

### Ce qui doit rester vrai

- Sur un poste à `Europe/Zurich`, les quatre écrans rendent **exactement** ce qu'ils rendaient (AC9, AC13).
- `formatSwissDate`, `formatSwissDateTime` et leurs 57 importeurs ne changent pas de comportement (leurs tests existants restent verts
  sans modification).
- Les contrats de repli : `''` (sauvegardes, rejet), `—` et valeur brute (envois MCP).
- Le CSV du journal d'audit reste en UTC (manuel `:4724`) ; aucun code serveur ne change ; aucune migration.

### Issues à ouvrir (par l'orchestrateur ; aucune `gh` qui écrit ici)

- **I1** (`technical-debt`, P3) — déménager `zurichParts`, `formatSwissDate`, `formatSwissDateTime`, `formatInstant` de
  `reports.api.ts` vers `lib/shared/utils/dates.ts` (réexport transitoire), puis repointer les importeurs.
- **I2** (`bug`, P3) — l'écran *Serveur MCP* (`mcp-time.ts:28-40`) affiche les dates des clés et des réglages au fuseau du poste ;
  l'aligner sur `formatInstant` et retirer la copie `depuisUtcNaif`.
- Recalibrer **#1511** : P2 → P3 et corriger son diagnostic (horodatages non naïfs), si Q2 le confirme.

### Fichiers communs avec les autres stories de l'Epic 50

`reports.api.ts` : la 48-11 (arrondi au centime) touche `formatReportAmount`/`formatCurrencyFigure`, plus bas dans le même fichier
(`:280` `formatReportAmount`, `:316` `formatCurrencyFigure`) — conflit textuel improbable, à résoudre en gardant les deux. `docs/manual/fr/user-manual.tex` : sections distinctes.
`local-date-guard.test.ts` : aucune autre story de l'epic ne le touche à `e5654fc00`.

### Charge estimée

Environ **une journée d'agent**, boucles de revue comprises : étape 0 et tests rouges 1 h 30 ; correctif 1 h ; garde et sondes 1 h 30 ;
manuel, mutations 1 h ; recette visuelle 1 h ; E2E complet et gates 1 h 30 ; deux ou trois passes de revue.

### Références

- Issue #1511 ; antécédents #880 (dates ISO des écrans comptables, 0.17.2 : `formatSwissDate`, « naïf = UTC »), #889 (date du jour en UTC,
  0.17.4 : `todayLocal`, `local-date-guard.test.ts`).
- `/home/gcorbaz/devel/kesh-gate-logs/dry-audit/rapport.md` § D1 (son « vérifié par grep » ne lisait que l'absence de `timeZone` et de `Z`
  dans le **code du site**, non le payload ; il laissait « à confirmer au payload » — l'étape 0 l'a fait).
- `CLAUDE.md` § *Qualité du code*, § *Recette visuelle*, § *E2E (Playwright)*, § *Règle de splitting préventif*, § *Review Iteration Rule*,
  § *Priorités des défauts*.

### Questions au Project Lead (par l'orchestrateur)

- **Q1** — D0 : afficher les instants à l'heure de Zurich partout (la story), ou garder l'heure du poste documentée en 0.18.0 (la story
  est retirée, #1511 se ferme comme non reproductible en Suisse, I1/I2 restent) ?
- **Q2** — #1511 en P3 plutôt qu'en P2 : rien n'est faux sur un poste suisse.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-11-24 — création** (Opus 5.5, `bmad-create-story`, autonomie) : fiche `drafted` ; étape 0 menée au code serveur — les quatre sites lisent
  le bon instant (trois champs portent `Z`, les envois MCP le reçoivent du code), la prémisse « UTC naïf lu en local » de #1511 est
  démentie ; reste l'affichage au fuseau du poste (D0, Q1) ; inventaire par le symptôme (`kesh-gate-logs/e506/inv-symptome.txt`, 68 lignes) :
  4 sites en défaut dans le périmètre, 1 hors périmètre (`mcp-time.ts`, I2), le reste juste ou hors sujet ; cinq modules ; prompt de
  validation P1 versionné (`50-6-dates-heure-de-zurich-validation-prompt-p1.md`).
