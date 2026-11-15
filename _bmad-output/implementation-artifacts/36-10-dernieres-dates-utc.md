# Story 36-10 : les dernières dates en UTC — les filtres du journal d'audit et la fenêtre de la réconciliation lisent le jour de Zurich (#1147, #1053)

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-10`, branche
`story/36-10-dernieres-dates-utc`, par le workflow `bmad-create-story`. Phase 2, release **0.18.0**, **sans migration**.
Rang 10 du fichier d'epic (« une story pour les deux », Q11) ; prolonge les 35-5a0 à 35-5c (le jour de Zurich par une
horloge injectable). Code lu à la tête du worktree, **`0bf3c3b4b`** (`origin/epic/36-0.18.0`, kickoff de la 0.18.0 sur la
0.17.8). Registres : `e36-choix-autonomes-e360.md` (aucune étape 0 pour cette story : le kickoff n'a lu que la phase 1 ;
C-e360-11 pour le CHANGELOG) ; `e36-choix-autonomes-36-10.md` pour la story (C-36-10-1 à -5, créé avec la fiche).
Validation non lancée.*

## Story

En tant que titulaire qui tient ses livres en Suisse, je veux que le journal d'audit filtre et exporte par **jour de
Zurich**, et que la fenêtre de 30 jours de la réconciliation parte du **jour de Zurich** du dernier rappel, afin qu'une
opération faite ou un rappel envoyé entre minuit et 1 h (2 h en été) ne soit plus rangé à la veille — comme tout ce que
Kesh date du jour depuis la 0.17.7.

**L'effet visible, la nuit.** (a) Une opération faite le 15 septembre à 0 h 30 (22 h 30 UTC le 14) n'apparaît **pas**
quand on filtre le journal d'audit sur le 15 septembre, mais sur le 14 — alors que le tableau l'affiche au 15 à 0 h 30 —, et
l'export CSV suit le même filtre. (b) Un rappel envoyé le 16 mai à 0 h 30 (22 h 30 UTC le 15) fait partir la fenêtre de la
réconciliation du 15 : le paiement reçu le 15 juin, trentième jour, n'est ni proposé ni accepté
(`payment_date_outside_window`). Ni l'un ni l'autre ne dépend de l'heure où l'on **regarde** : seulement de l'heure où
l'opération ou le rappel a eu lieu.

## Ce qui se lit au code (`0bf3c3b4b`)

### Le journal d'audit (#1147)

- **La route** ne fait que lire les dates : `crates/kesh-api/src/routes/audit_log.rs:157-158`
  (`let date_from = parse_date(params.date_from.as_deref(), "dateFrom")?;`), validées dans `[1000-01-01, 9999-12-31]`
  (`:226`, `if !annee_stockable(date) {`), puis passées telles quelles au dépôt (`:192-194`, `Ok(AuditLogListQuery {`
  `date_from,` `date_to,`). **Une seule fonction pour la liste et l'export** (`construire_requete`, `:154-201` ; l'export
  l'appelle avant `list_for_export`, `:325`). Le commentaire à réécrire : `:420-421`
  (`// Le nom porte la date du jour de Zurich (Story 35-5b, #929) ; les lignes et` / `// les filtres restent en UTC (#669).`).
- **Le dépôt** pose les bornes en jours UTC : `crates/kesh-db/src/repositories/audit_log.rs:224-240`, dans
  `push_where_clauses` (`:216`), partagée par `list_by_company_paginated` (`:267`) et `list_for_export` (`:314`) :
  ```
  let borne = date_from
      .and_hms_milli_opt(0, 0, 0, 0)
  …
  qb.push(" AND created_at >= ");
  …
  let borne = date_to
      .and_hms_milli_opt(23, 59, 59, 999)
  …
  qb.push(" AND created_at <= ");
  ```
  `created_at` est un `DATETIME(3)` en UTC (doc-comment `:171-176` : « ⚠️ **Les dates s'entendent en JOURS UTC.** … **sqlx**,
  qui impose `time_zone='+00:00'` à chaque session ») ; la session est fixée par `crates/kesh-db/src/pool.rs:43`
  (`time_zone = '+00:00'",`). Le doc-comment de `:211-215` (« ⛔ **Bornes de date INCLUSIVES à la milliseconde, sans
  arithmétique** ») explique pourquoi la forme `< date_to + 1 jour` a été écartée : elle paniquait au dernier jour.
- **L'écran** envoie le jour choisi, sans conversion (`frontend/src/routes/(app)/audit-log/+page.svelte:70-71`,
  `if (dateFrom) q.dateFrom = dateFrom;`), et affiche l'heure **du poste** (`:63-64`, `new Intl.DateTimeFormat(i18nLocale(),
  { dateStyle: 'medium', timeStyle: 'short' })`, `:375`, `{dateFormat.format(new Date(e.createdAt))}`) ; son en-tête le dit
  (`:8-10`, « ⚠️ Les dates filtrent sur des jours **UTC** (contrat de la route) ; le tableau affiche l'heure **locale** — d'où
  les libellés « jour UTC » des filtres »). Libellés : `:237` `{i18nMsg('audit-log-filter-date-from', 'Du (jour UTC)')}`,
  `:256` `{i18nMsg('audit-log-filter-date-to', 'Au (jour UTC)')}` ; catalogues : `fr-CH/messages.ftl:3437-3438`
  (`Du (jour UTC)`, `Au (jour UTC)`), `de-CH:3358-3359` (`Von (UTC-Tag)`, `Bis (UTC-Tag)`), `en-CH:3364-3365`
  (`From (UTC day)`, `To (UTC day)`), `it-CH:3359-3360` (`Dal (giorno UTC)`, `Al (giorno UTC)`). Types :
  `frontend/src/lib/features/audit-log/audit-log.types.ts:44` (`/** Jour UTC, \`AAAA-MM-JJ\`. */`), `:46`.
- **Le CSV** : en-tête `audit-log-csv-header-created-at = Date (UTC)` (`fr-CH:3414` ; repli `routes/audit_log.rs:279`,
  `"Date (UTC)",`) ; chaque ligne porte l'instant UTC **sans fuseau écrit**, l'unité n'étant dite que par l'en-tête
  (`:372-375`, `.created_at` `.format("%Y-%m-%d %H:%M:%S%.3f")` → `2026-09-14 22:00:00.000`). La forme à `T…Z` est celle
  de la réponse **JSON** (`AuditLogEntryResponse::depuis`, `:113`, `.format("%Y-%m-%dT%H:%M:%S%.3fZ")`), non du CSV.
- **Le serveur MCP** ne lit pas le journal d'audit (`git grep -n "audit_log::list\|list_for_export\b\|AuditLogListQuery" --
  'crates/*/src/*'` : seuls `routes/audit_log.rs` et des commentaires de `exports/csv_tables.rs:1521-1522`) ; l'export
  global (`exports/global.rs:395`, `list_all_by_company`) n'a pas de filtre de date. Les routes du journal sont **fermées aux
  clés d'API** (`docs/api-external.md:85`) : aucun contrat d'API externe ne change de ce côté.
- **Tests dont l'attendu change** : `kesh-db/src/repositories/audit_log.rs` `list_date_bounds_are_inclusive_to_the_millisecond`
  (`:715-780` : entrées à `2026-09-14 23:59:59.999`, `2026-09-15 00:00:00.000`, `23:59:59.999`, `2026-09-16 00:00:00.000`,
  `9999-12-31 23:59:59.999`, bornes UTC) ; `kesh-api/tests/audit_log_e2e.rs` `la_borne_haute_9999_est_acceptee_et_rend_bien_l_entree`
  (`:469-494` : entrée posée à `'9999-12-31 23:59:59.999'`, `dateTo=9999-12-31` doit la rendre — elle est, à Zurich, le
  `10000-01-01` : voir D3).
- **La spec E2E** : `frontend/tests/e2e/audit-log.spec.ts:56-59` (`/** Le jour UTC courant, … la route filtre sur des jours
  UTC. */ function todayUtc() { return new Date().toISOString().slice(0, 10); }`), employée au pas 6 (`:130-135`). Son
  commentaire est **faux de sens** (`:130-131`, « entre 00:00 et 02:00 à Zurich, le jour local **précède** le jour UTC ») : à
  cette heure, le jour de Zurich est le **lendemain** du jour UTC. La garde frontend l'admet nommément
  (`frontend/src/lib/shared/utils/local-date-guard.test.ts:272-275`, `fichier: 'audit-log.spec.ts'`, `motif: 'todayUtc —
  le filtre du journal d’audit compare des horodatages UTC (#669)'`), et deux sondes s'appuient sur cette entrée
  (`:346-357`, `:359-373`). Le helper du jour suisse existe : `frontend/tests/e2e/helpers/api-fixtures.ts:392-394`
  (`export function todayInZurich(): string {` … `timeZone: 'Europe/Zurich'`).

### La fenêtre de la réconciliation (#1053)

- **L'ancre**, `A = max(F, E, R)` : `crates/kesh-db/src/repositories/reconciliation.rs:231-241`, macro
  `invoice_payment_anchor_sql!`, dont le terme du rappel est `:237`
  (`"COALESCE(DATE(rmd.last_sent_at), i.date)",`), `rmd` étant la dérivée du dernier rappel non annulé (`:159-164`,
  `MAX(sent_at) AS last_sent_at FROM invoice_reminders WHERE cancelled_at IS NULL`). `DATE()` d'un `DATETIME` lu en session
  `+00:00` rend le **jour UTC**. Le doc-comment l'écrit en angle mort (`:221-224`, « l'écart de fuseau d'un jour au plus est
  un angle mort écrit (C-29-2-3), sans effet mesurable sur une borne de 30 jours »).
- **Deux lecteurs, une expression** (`:226-229`) : la **proposition** (`unpaid_invoices_in_window_sql`, `:722-746`, dont
  `:738` `AND {INVOICE_PAYMENT_ANCHOR_SQL} >= ? \`, liée à `window.earliest_anchor_date` par
  `find_unpaid_invoices_for_window`, `:646`/`:661`, et `find_invoices_slightly_overpaid_for_window`, `:693`/`:708`) et
  l'**acceptation** (`invoice_payment_anchor_date`, `:570-583`, requête scalaire `INVOICE_PAYMENT_ANCHOR_SCALAR_SQL`,
  `:253-258`), appelée par `kesh-api/src/routes/reconciliation.rs:2491` puis comparée par `window.check(invoice.date,
  anchor_date)` (`:2520`). Le côté fournisseur (`:834`) n'a pas de rappel : hors du symptôme.
- **Qui écrit `sent_at`** : l'envoi par e-mail, l'instant UTC (`kesh-api/src/routes/invoice_email.rs:667`,
  `sent_at: chrono::Utc::now().naive_utc(),`) ; le rappel manuel, la valeur reçue (`routes/dunning_reminders.rs:122`,
  `pub sent_at: NaiveDateTime,`), que l'écran construit en instant UTC pour le jour même et à midi pour un jour passé
  (`frontend/src/lib/features/reminders/reminder-sent-at.ts:29-34`, `return now.toISOString().slice(0, 19);` …
  `` return `${sentAt}T12:00:00`; ``). Dans les deux cas, `sent_at` est un **instant UTC** : son jour de Zurich est le bon
  jour, et c'est ce que l'éligibilité aux rappels lit déjà depuis la 35-5b
  (`kesh-db/src/repositories/dunning_eligibility.rs:136`,
  `today >= jour_suisse(lr.and_utc()) + Duration::days(delay_of(nl) as i64)`). Aujourd'hui, **un même rappel a deux jours** :
  celui de Zurich pour le rappel suivant, celui d'UTC pour la réconciliation — le mélange que la 35-5b a laissé écrit en
  limite (fiche 35-5b, Dev Notes, « Hors périmètre »).
- **Tests existants** : `kesh-db/tests/reconciliation_repository.rs:1485` (`the_anchor_is_the_latest_of_date_due_date_and_last_reminder`),
  `:1645` (`a_reminder_moves_only_the_anchor_of_its_own_invoice`), `:1947` (`exigibility_date_sql_and_rust_agree`, sur `X`
  seule) ; `kesh-api/tests/reconciliation_e2e.rs:3239` (`proposal_and_acceptance_share_the_anchor_of_due_date_and_reminders`,
  par HTTP, `D = 2026-06-15`), dont le helper pose le rappel à midi et demi (`:3204`,
  `let sent_at = sent_on.and_hms_opt(12, 30, 0).unwrap();`) : aucun cas de nuit.

### L'horloge et sa garde

- `crates/kesh-core/src/horloge.rs:58-62` : `pub fn jour_suisse(instant: DateTime<Utc>) -> NaiveDate` (« le fuseau est une
  constante (C-e350-16), jamais `TZ` ni `chrono::Local` »), seul sens non ambigu (`:50-52`). **Le sens inverse — les instants
  qui bornent un jour de Zurich — n'existe pas** au dépôt : c'est ce que le filtre du journal demande.
- `crates/kesh-api/tests/date_du_jour_garde.rs` inventorie les formes qui tirent **la date du jour** d'ailleurs que de
  l'horloge (`FORMES`, `:128-139` : `.date_naive(`, `UTC_DATE(`, `CURDATE(`, `DATE(NOW…`, …) en liste close à égalité
  stricte (`EXEMPTIONS`, `:178-220` : deux entrées permanentes, `kesh-core/src/horloge.rs` 1 et `kesh-api/src/routes/admin.rs` 1).
  **Elle dit encore vrai** pour sa classe (relevé ci-dessous, I1). **Elle ne voit pas** les deux défauts de cette story,
  qui ne lisent pas la date du jour : `DATE(rmd.last_sent_at)` n'est pas `DATE(NOW…` (`FORMES`, `:138`), et des bornes posées
  par `and_hms_milli_opt` ne sont aucune forme ; son doc-comment l'écrit en angle mort pour Rust (`:23-24`, « un jour tiré
  d'une variable d'instant (`let n = Utc::now().naive_utc(); n.date()`) n'est pas relevé »). D5 l'étend.

### Inventaire par le symptôme

Le symptôme : **un jour civil tiré d'un instant, ou des instants tirés d'un jour civil, en UTC**. Aucun détecteur par
exécution ne s'offre (le défaut ne dépend pas de l'horloge : `KESH_TEST_INSTANT` ne le fait pas rougir) ; détecteurs par
lecture, partis du symptôme, sur tout `crates/*/src` et `frontend/src`, commandes lancées à `0bf3c3b4b`.

**I1 — la date du jour (la classe de la garde), recompte à la main** :
```
$ git grep -nP "\.date_naive\s*\(|(?i)\bUTC_DATE\s*\(|(?i)\bCURDATE\s*\(" -- 'crates/*/src/*.rs' 'crates/*/tests/*.rs' ':!crates/kesh-api/tests/date_du_jour_garde.rs'
crates/kesh-core/src/horloge.rs:61:        .date_naive()
crates/kesh-core/src/horloge.rs:105:    /// `.date_naive()`).
crates/kesh-report/src/aged_receivables.rs:27://! - **`as_of` bindé** (pas `UTC_DATE()` en dur) pour la testabilité.
$ git grep -nP "Utc::now\s*\(\s*\)\s*\.\s*(year|month|day|format)\s*\(" -- 'crates/*/src/*.rs' 'crates/*/tests/*.rs'
crates/kesh-api/src/routes/admin.rs:724:        Utc::now().format("%Y%m%dT%H%M%S%3f"),
```
Hors commentaires : `horloge.rs:61` et `admin.rs:724`, les deux entrées permanentes. Recompte de lecture (il ne réduit pas
les blancs autour des `.` comme la garde) : **la garde elle-même se relance à l'étape 0**, journal cité.

**I2 — SQL : le jour d'un horodatage** (`DATE(` suivi d'autre chose que `)`) :
```
$ git grep -nP "\bDATE\s*\((?!\s*\))" -- 'crates/*/src/*.rs' | /usr/bin/grep -vE ":\s*///?"
crates/kesh-db/src/repositories/reconciliation.rs:237:            "COALESCE(DATE(rmd.last_sent_at), i.date)",
$ git grep -nP "(?i)\bDATE\s*\((?!\s*\))" -- 'crates/kesh-db/migrations/*.sql' 'crates/kesh-db/test-schema/*'
crates/kesh-db/migrations/20260419000001_invoice_paid_at.sql:29:-- de comparer sur `DATE(paid_at)` pour des bornes strictement cohérentes
crates/kesh-db/migrations/20260419000001_invoice_paid_at.sql:33:    CHECK (paid_at IS NULL OR DATE(paid_at) >= date - INTERVAL 1 DAY);
$ git grep -nP "(?i)CONVERT_TZ|time_zone" -- 'crates/*/src/*'
crates/kesh-db/src/pool.rs:15:/// - `time_zone = '+00:00'` pour garantir la cohérence UTC avec `NaiveDateTime`
crates/kesh-db/src/pool.rs:43:                         time_zone = '+00:00'",
crates/kesh-db/src/repositories/audit_log.rs:173:/// serveur qui le garantit (`@@time_zone` vaut `SYSTEM`) mais **sqlx**, qui
crates/kesh-db/src/repositories/audit_log.rs:174:/// impose `time_zone='+00:00'` à chaque session
```
→ `reconciliation.rs:237` **dans le périmètre** (#1053). La contrainte `chk_invoices_paid_at_after_date` (migration
`20260419000001`, `:33`) **hors périmètre** : `paid_at` n'est pas un instant mais la date du règlement posée à minuit
(C-e355b-6 ; écrivains `kesh-db/src/repositories/invoice_settlements_write.rs:394`, `:748`,
`settled_on.and_hms_opt(0, 0, 0)`), et la changer demanderait une migration (C-e350-25, règle d'arrêt).

**I3 — Rust : le jour d'un instant** (`.date()` sur un `NaiveDateTime`), hors modules de tests (tri : la ligne après le
premier `#[cfg(test)]` du fichier compte pour test) :
```
$ git grep -nE "\.date\(\)" -- 'crates/*/src/*.rs' | /usr/bin/grep -vE ":\s*//"      # 3 lignes, dont 2 hors tests :
crates/kesh-api/src/routes/dunning_reminders.rs:272:    borner(body.sent_at.date(), "sentAt")?;
crates/kesh-api/src/routes/invoices.rs:1573:                    .map(|d| format_date(&d.date()))
crates/kesh-api/src/routes/payment_batches.rs:355:            requested_execution_date: moment.date(),   # mod tests (#[cfg(test)] :333)
```
→ `dunning_reminders.rs:272` **hors symptôme** : `borner` ne contrôle que la plage d'années stockable (Story 33-4), un jour
UTC ou suisse y donne le même verdict sauf la nuit du 31 décembre 9999. `invoices.rs:1573` **hors symptôme** : `paid_at`
du CSV de l'échéancier, une date à minuit (C-e355b-6, revue P1 de la 35-5b). `payment_batches.rs:355` est dans le
`mod tests` du fichier (`#[cfg(test)]` à `:333`) : une fixture, hors symptôme. Liste close de D5 (b) : ces **trois**.

**I4 — Rust : les instants qui bornent un jour** (un jour civil monté en instant par `and_hms*`, `and_time`,
`NaiveTime::MIN`, puis comparé à un horodatage), hors modules de tests — 54 lignes au total, 19 hors tests :
```
$ git grep -nE "and_hms|and_time\(|NaiveTime::MIN|and_utc\(\)|\.and_local_timezone" -- 'crates/*/src/*.rs' | /usr/bin/grep -vE ":\s*//"   # puis tri par #[cfg(test)]
crates/kesh-api/src/auth/api_key.rs:201  .map(|dt| dt.and_utc().timestamp())          — un instant sérialisé
crates/kesh-api/src/routes/auth.rs:56, :530, :858  ….and_utc().timestamp()            — instants (jetons)
crates/kesh-api/src/routes/backup_schedule.rs:51, bank_imports.rs:345, :1790, mcp_server_settings.rs:87,
  reconciliation.rs:3043, :3055, :4784, :6215  ….and_utc()                              — instants sérialisés
crates/kesh-api/src/routes/reconciliation.rs:2585  .and_time(… 0, 0, 0 …)              — paid_at, date à minuit
crates/kesh-db/src/repositories/invoice_settlements_write.rs:394, :748  settled_on.and_hms_opt(0, 0, 0) — idem
crates/kesh-db/src/repositories/supplier_invoices.rs:2175  .and_time(… 0, 0, 0 …)       — idem (facture fournisseur)
crates/kesh-db/src/repositories/dunning_eligibility.rs:136  jour_suisse(lr.and_utc())  — déjà le jour de Zurich (35-5b)
crates/kesh-db/src/repositories/audit_log.rs:228, :236  .and_hms_milli_opt(0,0,0,0) / (23,59,59,999)
```
→ **seul `audit_log.rs:228`, `:236` présente le symptôme** (#1147, dans le périmètre). Les autres sont des instants
sérialisés, ou la date d'un règlement posée à minuit (la même convention des deux côtés, aucun fuseau en jeu).

**I5 — l'écran** (un horodatage coupé à son jour UTC, ou un calendrier en UTC) :
```
$ git grep -nE "(At|_at|At\))\??\.(slice|substring)\(0, ?10\)|At\.split\('T'\)|getUTC(Date|FullYear|Month)|timeZone: ?'UTC'" -- frontend/src | /usr/bin/grep -v "\.test\.ts"
src/lib/features/bank-import/manual-statement.ts:107   getUTC…   — validité d'une date civile saisie
src/lib/features/homepage/homepage-charts.ts:211, :216 timeZone: 'UTC' — nom d'un mois civil
src/lib/features/journal-entries/form-helpers.ts:62    setUTCDate — arithmétique de date civile
src/lib/shared/utils/date-filtre.ts:24, local-date.ts:47-49 getUTC… — idem
src/routes/(app)/invoices/[id]/+page.svelte:1291, invoices/due-dates/+page.svelte:610,
  supplier-invoices/[id]/+page.svelte:210   paidAt.slice(0, 10)    — paid_at, date à minuit
```
Et, hors de ce motif (validation P1, V1-6) : `src/lib/features/fiscal-years/fiscal-years.helpers.ts:52`
(`const year = new Date().getFullYear();`, dans `currentYearDefaults`, `:51`) — l'**année du poste**, pré-remplie dans le
formulaire d'un exercice et modifiable : le jour local assumé de l'écran (manuel, `user-manual.tex:752-754`, « il propose
la date \textbf{de votre poste} »), hors du symptôme (aucun jour UTC). Les autres `new Date()` de `frontend/src` sont des
instants (sessions, `reminder-sent-at.ts:31`, instant UTC voulu) ou ce même jour local.
→ **aucun site dans le périmètre** : arithmétique sur des dates civiles, ou `paid_at`. Le jour UTC de la seule spec
(`audit-log.spec.ts:56-59`) suit le correctif (D6). La garde `local-date-guard.test.ts` couvre le reste de la forme
`toISOString().slice(0, 10)` (src et specs, #889, 35-5c).

**Angles morts de l'inventaire** : I2 ne voit pas un `date(` SQL en minuscules ni un jour tiré par `CAST(x AS DATE)` (aucun
au dépôt, `git grep -niP "CAST\([^)]* AS DATE\)" -- 'crates/*/src/*'` ne rend que les `CAST(GREATEST(…) AS DATE)` de
`reconciliation.rs`, sur des `DATE`) ; I3 et I4 ne voient pas un jour tiré par `format("%Y-%m-%d")` d'un instant (relevé
`git grep -nE "_at[a-z_]*\)?\.?\s*\.?format\(\"%(d|Y)[^\"]*\"\)" -- 'crates/*/src/*.rs'` hors `%H` : vide), ni un instant
porté par une variable au nom neutre.

## Décisions

- **D1 — Le sens inverse de `jour_suisse` vit dans `kesh-core/src/horloge.rs`, défini par lui** (C-36-10-1). Deux
  fonctions pures : `debut_du_jour_suisse(jour: NaiveDate) -> DateTime<Utc>`, le **premier** instant (à la milliseconde)
  dont `jour_suisse` vaut `jour`, et `fin_du_jour_suisse(jour) -> DateTime<Utc>`, le **dernier** (`debut(jour + 1) − 1 ms`,
  ou équivalent). Le contrat se dit et se teste par `jour_suisse` : `jour_suisse(debut(j)) == j`,
  `jour_suisse(debut(j) − 1 ms) == j − 1`, de même pour `fin`. **L'algorithme** : `Europe::Zurich.from_local_datetime(
  j 00:00:00.000)` ; `Single(t)` → `t` ; `Ambiguous(a, b)` → `min(a, b)` ; `None` (minuit n'existe pas : le seul cas est le
  passage de l'heure de Berne, `+00:29:46`, à l'heure d'Europe centrale le 1894-06-01) → **recherche dichotomique à la
  milliseconde** du premier instant `t` de `[j 00:00Z − 3 h, j 00:00Z + 1 h]` tel que `jour_suisse(t) >= j` — exacte parce
  que `jour_suisse` est croissante au sens large et que tout décalage de Zurich est compris entre `0` et `+2 h` ; une
  avance par minutes est écartée (elle tomberait jusqu'à 59 s après le vrai premier instant, et la propriété à − 1 ms
  rougirait). `fin(j) = debut(j + 1) − 1 ms`. **Précondition** écrite au doc-comment des deux fonctions, comme celle de
  `push_where_clauses` (`audit_log.rs:202-205`) : `jour` dans `[1000-01-01, 9999-12-31]`, plage que la route garantit
  (`routes/audit_log.rs:226`, `if !annee_stockable(date) {`) ; hors d'elle (`NaiveDate::MIN`, `MAX`), le résultat n'est pas
  défini. Dans ce domaine, aucun `expect`, aucune panique. Le jour de 23 heures (fin mars) et celui de
  25 heures (fin octobre) sortent juste par construction. — *Écartée* : écrire les bornes `+01:00`/`+02:00` à la main dans `kesh-db` : un
  second calcul du fuseau, qui dériverait de `jour_suisse` (c'est le mélange de deux horloges que `jour_suisse` a fermé).
- **D2 — Le journal d'audit filtre en jours de Zurich, dans `push_where_clauses` seul** (C-36-10-2). `date_from` borne à
  `created_at >= debut_du_jour_suisse(date_from)`, `date_to` à `created_at <= fin_du_jour_suisse(date_to)`, liés en
  `NaiveDateTime` UTC (`.naive_utc()`) ; `AuditLogListQuery` garde ses `NaiveDate`, la route et l'écran n'envoient rien de
  neuf ; liste et export restent une seule clause (doc-comment de `list_for_export`, `:311-313`). Les bornes restent
  **inclusives à la milliseconde**, sans `<` sur le lendemain (raison écrite `:211-215`). — *Écartée* : convertir dans la
  route et passer des instants au dépôt : deux types pour la même requête, et le doc-comment « Les dates s'entendent en
  JOURS » perdrait son seul lecteur.
- **D3 — Les bornes extrêmes restent servies** (C-36-10-2). `dateFrom=1000-01-01` donnerait une borne au `0999-12-31` UTC
  (l'heure locale moyenne de 1000, `+00:34:08` dans la base de fuseaux), hors de la plage documentée de `DATETIME` : la
  borne basse liée est **toujours plafonnée** à `1000-01-01 00:00:00.000` (`max(debut, 1000-01-01 00:00:00.000)`), ce qui
  ne change aucun résultat (aucune ligne ne peut la précéder) et ne laisse rien dépendre de la tolérance du MariaDB de
  chaque installation (`push_bind` ne panique pas sur l'an 999, `sqlx-mysql-0.8.6/src/types/chrono.rs:262-265`,
  `u16::try_from(date.year())` : le plafond est une défense de la base, non de `sqlx`). **Le calcul des deux bornes est une
  fonction pure de `kesh-db`**, dans `repositories/audit_log.rs` (aucun module neuf ; le plafond est une affaire de plage
  `DATETIME`, non de fuseau, d'où sa place hors de `kesh-core`) : `bornes_du_filtre(date_from: Option<NaiveDate>, date_to:
  Option<NaiveDate>) -> (Option<NaiveDateTime>, Option<NaiveDateTime>)`, appelée par `push_where_clauses`, qui ne fait plus
  que lier ; c'est elle que teste `bornes_du_filtre_jours_de_zurich_et_plafond` (tableau des tests, M7). `dateTo=9999-12-31` donne
  `9999-12-31T22:59:59.999Z` : une entrée posée à `9999-12-31 23:59:59.999` UTC est, à Zurich, le `10000-01-01`, et
  **aucun filtre de date ne la rend plus** ; le test `la_borne_haute_9999_est_acceptee_et_rend_bien_l_entree`
  (`audit_log_e2e.rs:469`) se réécrit sur l'entrée `22:59:59.999` (rendue) et `23:00:00.000` (non rendue). Une date de l'an
  9999 n'est pas une date réelle : la perte est assumée et écrite au doc-comment.
- **D4 — L'ancre de la réconciliation lit le jour de Zurich du dernier rappel, calculé hors du SQL** (C-36-10-3).
  - **Proposition** : le prédicat `A >= e` se réécrit sans `DATE()`, par l'équivalence
    `max(X, R, F) >= e ⟺ X >= e OR R >= e` (`X >= F` toujours ; `R` absent ne compte pas) et
    `jour_suisse(r) >= e ⟺ r >= debut_du_jour_suisse(e)` : `AND ({INVOICE_EXIGIBILITY_DATE_SQL} >= ? OR rmd.last_sent_at >= ?)`,
    liés à `earliest_anchor_date` puis à `debut_du_jour_suisse(earliest_anchor_date).naive_utc()`. Un `NULL` de
    `last_sent_at` rend le second terme faux, ce qui est le sens voulu ; l'ordre des `?` change dans les deux lecteurs
    (`:661`, `:708`) et dans le doc-comment de `unpaid_invoices_in_window_sql` (`:718-721`, « Les trois premiers `?` sont… »).
  - **Acceptation** : `invoice_payment_anchor_date` lit `X` et `rmd.last_sent_at` et rend `max(X, jour_suisse(r))` en Rust,
    même signature (`Option<NaiveDate>`) ; `routes/reconciliation.rs:2491`, `:2520` ne changent pas.
  - La macro `invoice_payment_anchor_sql!` et `INVOICE_PAYMENT_ANCHOR_SQL` (et la const scalaire) disparaissent ou ne
    gardent que ce qu'un lecteur emploie ; le doc-comment `:221-224` (angle mort C-29-2-3) et celui des « deux lecteurs, une
    expression » (`:226-229`) se réécrivent : **deux écritures** d'une même règle, tenues d'accord par un test (AC6).
  - *Écartées* : (a) `CONVERT_TZ(rmd.last_sent_at, '+00:00', 'Europe/Zurich')` — juste sur le MariaDB de dev
    (`SELECT CONVERT_TZ('2026-03-10 23:30:00','+00:00','Europe/Zurich')` rend `2026-03-11 00:30:00` sur
    `kesh-mariadb-dev2`), mais la justesse dépendrait des tables de fuseaux du serveur de chaque installation, et leur
    absence rend `NULL`, que le `COALESCE` changerait **en silence** en date de facture ; (b) le décalage d'été recalculé
    en SQL (dernier dimanche de mars et d'octobre) : une seconde écriture du fuseau ; (c) un pré-filtre SQL élargi d'un
    jour puis un tri en Rust : le `LIMIT 50` (`:741`) s'appliquerait avant le tri et pourrait écarter une vraie candidate.
- **D5 — La garde s'étend au jour d'un instant enregistré** (C-36-10-4). Une famille de plus dans
  `date_du_jour_garde.rs`, sur `crates/*/src` (texte sans lignes `//`), en liste close à égalité stricte comme les autres :
  (a) SQL : `\bDATE\s*\(` **seul** (casse respectée : le SQL du dépôt s'écrit en capitales, et `date(` minuscule est un
  helper de test courant en Rust, `reconciliation.rs:1289`) — **0 partout** après le correctif. La garde emploie la crate
  `regex` (`date_du_jour_garde.rs:42`, `use regex::Regex;` ; `kesh-api/Cargo.toml:92`, `regex = "1.10"`), **sans
  lookahead** : l'exclusion de `DATE()` que portait la commande `git grep -P` d'I2 ne s'y écrit pas, et n'a aucun usage au
  dépôt (`DATE_FORMAT(`, `STR_TO_DATE(`, `AS DATE)` ne correspondent pas au motif, borné par `\b` et suivi de `(`). Le motif
  se compile avec `regex` : une sonde le construit par `re()` et compte 1 sur `"COALESCE(DATE(rmd.last_sent_at), i.date)"` ;
  (b) Rust : `\.date\s*\(\s*\)` — liste close comptée depuis l'exécution, **trois** entrées aujourd'hui (I3 :
  `dunning_reminders.rs` 1, borne d'années ; `routes/invoices.rs` 1, `paid_at` ; `routes/payment_batches.rs` 1, `mod
  tests`), chaque entrée avec son motif. Sondes positives (le texte ci-dessus, une forme coupée sur deux lignes, une
  `DATE(x)` ajoutée à un fichier propre) ; angles morts écrits au doc-comment
  (`date(` minuscule, `CAST(… AS DATE)`, un jour tiré par `format`, des bornes `and_hms*` — la classe de #1147, que seul le
  test de `bornes_du_filtre` tient —, et le SQL hors `.rs` : les `.sql` de `crates/kesh-db/src/post_restore/` et la
  migration que `post_restore.rs:378` inclut, exécutés à l'import). — *Écartée* : ne rien étendre et écrire l'angle mort : #1053 est née de ce trou, et
  un `DATE(` SQL est un motif d'une ligne.
- **D6 — L'écran et les catalogues cessent de dire « jour UTC »** (C-36-10-5). Libellés `Du` / `Au`, `Von` / `Bis`,
  `From` / `To`, `Dal` / `Al` (les quatre `messages.ftl` et les deux replis de `+page.svelte`, égaux à la valeur fr-CH) ;
  commentaires `+page.svelte:8-10` et `audit-log.types.ts:44`, `:46` réécrits (« jour de Zurich »). **Le CSV garde ses
  heures UTC**, **son format** et son en-tête `Date (UTC)` : ses lignes sont des instants (#669, « les horodatages restent
  en UTC »), écrits `AAAA-MM-JJ HH:MM:SS.mmm` sans fuseau (`routes/audit_log.rs:374`), l'unité étant dite par l'en-tête
  (`fr-CH:3414`, repli `:279`) ; **aucun changement du contenu de l'export** ; seul le filtre change. Aucune clé i18n ajoutée ni retirée (`i18n-keys.test.ts` non touché). — *Écartée* :
  « Du (jour suisse) » : une précision que plus rien ne contredit à l'écran, puisque le tableau affiche l'heure du poste ;
  et écrire les lignes du CSV à l'heure de Zurich : changement du contenu d'un export sans demande, hors de #1147.
- **D7 — Aucun Playwright neuf ; la spec existante passe au jour de Zurich**. Le filtre se joue à la frontière HTTP de
  façon déterministe dans `kesh-api/tests/audit_log_e2e.rs` (entrées posées à l'instant voulu par `UPDATE audit_log SET
  created_at`, comme `:474`, puis `GET /api/v1/audit-log` et `GET /api/v1/audit-log/export.csv` par HTTP), et la fenêtre
  dans `reconciliation_e2e.rs` (proposition et acceptation par HTTP, `spawn_app`). Le navigateur ne porte aucune valeur
  neuve : le jour choisi part tel quel (`+page.svelte:70-71`), la liste des propositions s'affiche telle que reçue. Le
  serveur servi ne lit pas `KESH_TEST_INSTANT` (`kesh-core/src/horloge.rs:21-22`) et `created_at` est posé par le serveur :
  **un Playwright ne peut pas jouer la nuit à coup sûr**. La spec `audit-log.spec.ts` prend `todayInZurich()` (sans second
  helper) au pas 6 — faute de quoi, après le correctif, elle rougirait la nuit —, son commentaire faux de sens est réécrit,
  et l'entrée de la garde frontend (`local-date-guard.test.ts:272-275`) disparaît, les deux sondes `:346-373` se réécrivant
  sur une autre entrée ou sur un texte synthétique ; les trois mentions du compte de l'allowlist des specs
  (`local-date-guard.test.ts:29`, « close de trois entrées », `:263`, « Les trois sites admis », `:318`, « les trois sites
  de l’allowlist ») passent à « deux ». **Elle ne joue la nuit que lancée entre 22 h ou 23 h et minuit UTC** ;
  c'est écrit dans la spec et ici.

## Critères d'acceptation

1. **AC1 — Les bornes d'un jour de Zurich** (D1) : `debut_du_jour_suisse` et `fin_du_jour_suisse` rendent, en dates fixes :
   `2026-01-15` → `2026-01-14T23:00:00.000Z` / `2026-01-15T22:59:59.999Z` ; `2026-09-15` → `2026-09-14T22:00:00.000Z` /
   `2026-09-15T21:59:59.999Z` ; `2026-03-29` (23 h) → `2026-03-28T23:00:00.000Z` / `2026-03-29T21:59:59.999Z` ;
   `2026-10-25` (25 h) → `2026-10-24T22:00:00.000Z` / `2026-10-25T22:59:59.999Z` ; et, pour tout jour d'un balayage
   (chaque jour de 2025 à 2027, plus `1000-01-01`, `1894-06-01`, `9999-12-31`), la propriété de `jour_suisse` à ± 1 ms ;
   aucune panique.
2. **AC2 — Le filtre du journal la nuit** (#1147, D2), par HTTP : entrées posées à `2026-09-14 21:59:59.999`,
   `22:00:00.000`, `22:30:00.000` (00 h 30 le 15 à Zurich), `2026-09-15 21:59:59.999`, `22:00:00.000` (UTC).
   `dateFrom=dateTo=2026-09-15` rend exactement les trois du milieu (`22:00:00.000` et `22:30` le 14, `21:59:59.999` le 15) ;
   le squelette rend les deux du 15 UTC (`21:59:59.999`, `22:00:00.000`). Même jeu en hiver (`2026-01-15`, décalage d'une
   heure). `dateFrom` seul et `dateTo` seul bornent chacun leur côté. Le **total** (`total`) suit les lignes.
3. **AC3 — L'export suit le même filtre** : `GET /api/v1/audit-log/export.csv?dateFrom=2026-09-15&dateTo=2026-09-15` porte
   les mêmes trois lignes, chacune avec son instant UTC **au format inchangé** (`2026-09-14 22:00:00.000`,
   `2026-09-14 22:30:00.000`, `2026-09-15 21:59:59.999`, sans `T` ni `Z`) et l'en-tête `Date (UTC)` inchangé ; le nom reste
   celui de la 35-5b (date de Zurich de l'horloge).
4. **AC4 — Les bornes extrêmes** (D3) : `dateFrom=1000-01-01` et `dateTo=9999-12-31` répondent 200 sans panique ; l'entrée à
   `9999-12-31 22:59:59.999` est rendue par `dateTo=9999-12-31`, celle à `23:00:00.000` ne l'est pas ; une entrée à
   `1000-01-01 00:00:00.000` est rendue par `dateFrom=1000-01-01`, la borne liée valant `1000-01-01 00:00:00.000` (D3,
   plafond ; constaté par le test unitaire `bornes_du_filtre_jours_de_zurich_et_plafond`, sur la valeur calculée). Le refus de `0999-12-31` reste 400
   (`une_date_hors_plage_est_refusee_pour_la_plage_et_non_le_format`, `:449`, vert sans changement).
5. **AC5 — La fenêtre de la réconciliation la nuit** (#1053, D4), par HTTP, `D = 2026-06-15` (été), `e = D − 30 =
   2026-05-16` : facture `F = D − 90`, `E = D − 60`, dernier rappel non annulé posé à **`2026-05-15T22:30:00Z`** (00 h 30 le
   16 à Zurich) → **proposée et acceptée** (le squelette : refus `payment_date_outside_window`) ; à `2026-05-15T21:59:59Z`
   (23 h 59 le 15) → refusée ; à `2026-05-15T22:00:00Z` → proposée et acceptée. Même jeu en hiver (`D = 2026-02-15`,
   `e = 2026-01-16`, rappels à `2026-01-15T22:59:59Z` refusé, `23:00:00Z` et `23:30:00Z` acceptés). Proposition et
   acceptation disent **la même chose** pour chaque cas (l'assertion `proposed, accepted` du test existant, `:3311`).
   Les onze cas existants (`a` à `k`) restent tels quels.
6. **AC6 — Deux écritures, une règle** (D4) : un test de `kesh-db` met d'accord le prédicat de la proposition et la valeur
   de `invoice_payment_anchor_date` sur un balayage d'instants de rappel autour du passage de `e − 1` à `e` à Zurich :
   **été**, `e = 2026-05-16`, de `2026-05-15T20:30:00Z` à `2026-05-16T00:30:00Z` inclus par pas de 30 min, plus
   `2026-05-15T21:59:59Z` et `2026-05-15T22:00:01Z` (de part et d'autre de `debut(e) = 2026-05-15T22:00:00Z`) ; **hiver**,
   `e = 2026-01-16`, de `2026-01-15T21:30:00Z` à `2026-01-16T01:30:00Z` inclus par pas de 30 min, plus `2026-01-15T22:59:59Z`
   et `2026-01-15T23:00:01Z` (`debut(e) = 2026-01-15T23:00:00Z`). Facture `F = e − 60`, `E = e − 30` (l'ancre ne tient qu'au
   rappel). Pour chaque instant, la facture est candidate **si et seulement si** `anchor >= e`, **et** le verdict est écrit
   en clair, sans appeler `debut_du_jour_suisse` dans le test : en été, candidate et ancre `e` (`2026-05-16`) pour
   `r >= 2026-05-15T22:00:00Z`, non candidate et ancre `e − 1` (`2026-05-15`) avant ; en hiver, de même pour
   `r >= 2026-01-15T23:00:00Z`, ancres `2026-01-16` et `2026-01-15`. (Un défaut commun aux deux écritures passerait l'accord
   seul ; il ne passe pas l'attendu fixe.) `the_anchor_is_the_latest_of_date_due_date_and_last_reminder` gagne deux cas de nuit.
7. **AC7 — Un même rappel, un même jour** : un rappel posé à `2026-03-10T23:30:00Z` compte du **11 mars** pour le rappel
   suivant (`dunning_eligibility.rs:136`, inchangé) **et** pour l'ancre de la réconciliation (`invoice_payment_anchor_date`
   rend `2026-03-11` pour `F = E = 2026-03-01`).
8. **AC8 — La garde** (D5) : la famille neuve est verte à `0` pour `DATE(` dans `crates/*/src` et à sa liste close pour
   `.date()` ; ses sondes rougissent (forme coupée, ajout dans un fichier propre) ; les familles 1 à 3 inchangées, vertes ;
   le doc-comment nomme les angles morts. **Le squelette** (D4 non appliqué) rougit sur `reconciliation.rs` 1.
9. **AC9 — L'écran et la spec** (D6, D7) : les libellés des deux filtres ne disent plus « UTC » dans aucune des quatre
   langues, et les replis du composant égalent la valeur fr-CH ; captures en thème clair des filtres (base de démonstration
   jetable, `kesh-gate-logs/e36-10/captures/`) ; `audit-log.spec.ts` pas 6 sur `todayInZurich()`, importé de
   `./helpers/api-fixtures` (`:8`), l'entrée
   `audit-log.spec.ts` de `ALLOWLIST_SPECS` retirée, ses deux sondes réécrites et toujours rouges sur leur cas, les
   mentions « trois » de `local-date-guard.test.ts:29`, `:263`, `:318` passées à « deux » (valeur recomptée à
   l'intégration avec la 36-7a, Fichiers communs) ;
   `npm run check`, `lint-i18n-ownership`, `test:unit` et `build` verts.
10. **AC10 — Docs** : le manuel ne dit plus que les filtres du journal sont en jours UTC ni que la fenêtre compte du jour
    UTC du dernier rappel (sites de la Tâche 5) ; il dit que les **heures** du tableau, du détail et du CSV restent celles
    qu'elles étaient (heure du poste à l'écran, UTC au CSV) ; `docs/api-external.md:681` dit « son dernier rappel non annulé
    (le jour de son envoi, à Zurich, depuis la v0.18.0) » ; PDF régénéré à l'intégration, contrôlé aplati (E6).
11. **AC11 — Rien d'autre ne bouge** : de jour (instant de midi), toutes les suites restent vertes sans changement
    d'attendu hors des tests nommés ici ; **aucune migration** (`git diff --stat origin/epic/36-0.18.0 -- crates/kesh-db/migrations`
    vide) ; aucun changement de schéma, de code d'erreur ni de champ de réponse.

## Tests à écrire, et la preuve du rouge

Le défaut ne dépend pas de l'horloge : les tests de nuit **rougissent sur le code d'aujourd'hui** sans squelette, d'une
date fixe. Ordre : (1) les tests neufs, commités **seuls**, rouges contre le code de `0bf3c3b4b`, journal
`kesh-gate-logs/e36-10/rouges.log` (AC1 ne compile pas avant D1 : commiter D1 d'abord avec ses seuls tests unitaires — ils
ne mentent pas sur le jour —, puis AC2 à AC7 rouges contre lui) ; (2) le correctif.

| Test | Fichier | Rouge attendu sur le code d'avant |
|---|---|---|
| bornes d'un jour (AC1) | `kesh-core/src/horloge.rs`, `mod tests` | (après D1, mutation M1/M4) |
| filtre de nuit, été et hiver (AC2) | `kesh-api/tests/audit_log_e2e.rs` | rend `21:59:59.999` et `22:00` du 15, non celles du 14 |
| export de nuit (AC3) | idem | idem, dans le CSV |
| bornes extrêmes (AC4) | idem, et `la_borne_haute_9999_…` réécrit | l'entrée `23:00:00.000` rendue |
| `bornes_du_filtre_jours_de_zurich_et_plafond` (AC2, AC4 ; `#[test]` sans base) | `kesh-db/src/repositories/audit_log.rs`, `mod tests` | (fonction neuve ; tue M1, M2, M5, M7) : `1000-01-01` → `1000-01-01 00:00:00.000` (début) ; `2026-09-15` → `2026-09-14 22:00:00.000` / `2026-09-15 21:59:59.999` ; `2026-01-15` → `2026-01-14 23:00:00.000` / `2026-01-15 22:59:59.999` ; `9999-12-31` → `9999-12-31 22:59:59.999` (fin) ; `None` → `None` |
| bornes de `push_where_clauses` (AC2, dépôt) | `kesh-db/src/repositories/audit_log.rs`, `list_date_bounds_are_inclusive_to_the_millisecond` réécrit en jour de Zurich | ses nouveaux attendus |
| fenêtre de nuit, été et hiver (AC5) | `kesh-api/tests/reconciliation_e2e.rs` (helper de rappel à **instant** à côté de `insert_reminder_292`) | refus `payment_date_outside_window` aux cas `22:00Z`, `22:30Z` |
| accord des deux écritures (AC6) | `kesh-db/tests/reconciliation_repository.rs` | désaccord, ou ancre au 15 |
| un rappel, un jour (AC7) | idem | ancre `2026-03-10` |
| famille neuve de la garde (AC8) | `kesh-api/tests/date_du_jour_garde.rs` | `reconciliation.rs : 1 occurrence, 0 écrite` |

**Mutations** (scripts versionnés dans `kesh-gate-logs/e36-10/mut/`, code commité avant, restauration par
`git restore --worktree -- <fichier>` puis `touch`, E5) : **M1** `debut_du_jour_suisse` rendu à minuit UTC → AC1, AC2 ;
**M2** `fin_du_jour_suisse` d'une milliseconde de plus → AC1, AC2 (`22:00:00.000` du 15) ; **M3** le second terme du
prédicat lié à `earliest_anchor_date` en minuit UTC → AC5, AC6 ; **M4** l'acceptation revenue à `DATE(rmd.last_sent_at)` →
AC5 (proposée mais refusée), AC6 ; **M5** `bornes_du_filtre` revenue à `and_hms_milli_opt` → AC2, AC3,
`bornes_du_filtre_jours_de_zurich_et_plafond` ; **M7** le plafond de D3 retiré →
`bornes_du_filtre_jours_de_zurich_et_plafond` (`1000-01-01` rend `0999-12-31 23:25:52.000` au lieu de
`1000-01-01 00:00:00.000` ; aucun test HTTP ne la voit, l'entrée du `1000-01-01` étant rendue avec ou sans plafond) ; **M8** le repli de `debut_du_jour_suisse` sur `None` remplacé par une avance
par minutes → AC1 (`1894-06-01`) ; **M6** un `DATE(x)`
réintroduit dans une chaîne SQL de `src` → AC8. Une mutation équivalente se dit. Journaux `mut/M1.log` à `mut/M8.log`.

## Tâches

- [ ] **Étape 0** : recompter I1 à I5 à la tête de la branche (commandes ci-dessus, sorties au Change Log) ; relancer
      `date_du_jour_garde` (gate ciblé, journal) ; relire les numéros de ligne cités.
- [ ] **D1** : `debut_du_jour_suisse`, `fin_du_jour_suisse` et leurs tests (AC1) ; doc-comment du module
      (`horloge.rs:1-15`) complété.
- [ ] Tests AC2 à AC8 commités, rouges consignés (`rouges.log`).
- [ ] **D2, D3** : `bornes_du_filtre` (fonction pure, plafond compris) et son test unitaire ; `push_where_clauses` qui
      l'appelle, et son doc-comment (`audit_log.rs:171-176`, `:211-215`) ; commentaire
      `routes/audit_log.rs:420-421`.
- [ ] **D4** : proposition, acceptation, doc-comments de `reconciliation.rs` (module `:11-15`, `:116-125`, `:195-229`,
      `:251-258`, `:559-569`, `:593-597`, `:718-721`) ; `/usr/bin/grep -n "UTC\|DATE(" ` sur le fichier, sortie au Change Log.
- [ ] **D5** : famille de la garde, sondes, angles morts — dont les `.sql` de `crates/kesh-db/src/post_restore/` (quatre
      fichiers, `git ls-files 'crates/*/src/*' | /usr/bin/grep -v "\.rs$"`) et celui que `post_restore.rs:378` inclut depuis
      `../migrations/` : du SQL exécuté à l'import, que la garde ne lit pas (elle ne balaie que les `.rs`) ; aucun ne
      contient `DATE(` aujourd'hui (`/usr/bin/grep -nP "\bDATE\s*\(" crates/kesh-db/src/post_restore/*.sql` vide).
- [ ] **D6, D7** : quatre `messages.ftl` (à côté du bloc `audit-log-filter-*`, jamais en fin de fichier), `+page.svelte`,
      `audit-log.types.ts`, `audit-log.spec.ts` (**ajouter `todayInZurich` à l'import** de `./helpers/api-fixtures`,
      `audit-log.spec.ts:8`, qui ne porte aujourd'hui que `createUser, USER_PASSWORD` ; retirer `todayUtc`, `:56-59`),
      `local-date-guard.test.ts` (entrée retirée, deux sondes réécrites, « trois » → « deux » à `:29`, `:263`, `:318`) ;
      captures.
- [ ] **Tâche 5 — docs** : `docs/manual/fr/user-manual.tex:754-758` (la note « date du jour » : la fenêtre compte désormais
      du jour de Zurich du dernier rappel), `:3063` (« son dernier rappel » : son jour, à Zurich), `:4004-4007` (la puce « Les
      dates filtrent sur des jours UTC » devient : les dates filtrent sur des jours de Zurich, comme le tableau qui affiche
      l'heure du poste), `:4039-4041` (l'export : heures en UTC, filtres en jours de Zurich) ; `admin-manual.tex:2558`
      (retirer « que ses filtres et son export lisent encore en jours UTC » et la phrase de #1053, ajouter « depuis la
      0.18.0 »), `:3633` (« les dates filtrent sur des jours UTC ») ; `docs/api-external.md:681`. `git grep -n -i "jour UTC\|jours
      UTC\|UTC-Tag\|UTC day\|giorno UTC\|#1053\|\\\\#1053\|1147" -- docs/ crates/kesh-i18n frontend/src frontend/tests` à
      la fin, sortie au Change Log (le README et le CHANGELOG de la 0.17.7, publiés, ne se réécrivent pas).
- [ ] **Gates** (`e35-dev-commun.md` § *Quels gates, quand* ; voie 2) — **`kesh-db` est touché** : gate complet en cours
      de boucle, non ciblé. Passage en `review` : bases `kesh_e3610` remises à neuf sur `kesh-mariadb-dev2:3307`,
      `scripts/test-fast.sh --ci` sous `flock /home/gcorbaz/devel/kesh-gate-logs/.gate-base2.lock scripts/mem-guard.sh`,
      `TMPDIR=/tmp/kesh-voie2`, `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3307/kesh_e3610` ; `scripts/gate-ci.sh
      kesh_ci_e3610` en **voie 1** (son verrou) ; frontend complet. Une commande lourde à la fois (E7). Clôture : `gate-ci`
      et E2E complet sur `kesh_e2e_e3610` (voie 2) recréée, au dernier commit de code ; les échecs nommés contre
      `docs/testing.md` § « Les échecs attendus ».
- [ ] Note de version, Dev Agent Record, prompt de la revue P1.

## Note de version

*Pour l'orchestrateur, qui l'assemble dans `## [0.18.0] — Non publié` (C-e360-11) ; cette branche n'écrit pas
`CHANGELOG.md`.*

**Corrigé**

- Les **filtres du journal d'audit** — à l'écran et à l'export CSV — lisent désormais le jour de Zurich : une opération faite
  entre minuit et une heure du matin (deux en été) apparaît au jour où elle a eu lieu, et non plus à la veille. Les
  libellés « Du (jour UTC) » et « Au (jour UTC) » deviennent « Du » et « Au ». Les heures du fichier CSV restent en UTC,
  comme son en-tête le dit. (refs #1147)
- La **fenêtre de 30 jours de la réconciliation** part du jour de Zurich du dernier rappel, comme le délai du rappel
  suivant : un paiement reçu le trentième jour après un rappel envoyé peu après minuit est de nouveau proposé et accepté.
  La règle de l'API (`payment_date_outside_window`) ne change ni de code ni de champ ; elle admet ce jour de plus.
  (refs #1053)

## Dev Notes

### Hors périmètre

- La contrainte `chk_invoices_paid_at_after_date` (`DATE(paid_at) >= date - INTERVAL 1 DAY`) : `paid_at` est une date à
  minuit, et la changer demande une migration (C-e350-25). Les `paidAt.slice(0, 10)` de l'écran, de même.
- Les horodatages eux-mêmes — colonnes, CSV, détail, `createdAt` de la réponse — restent en UTC (#669).
- La garde frontend pour d'autres formes que `toISOString().slice(0, 10)` (#889, 35-5c).
- Le refus d'un rappel manuel « dans le futur » (`dunning_reminders.rs:267`, `body.sent_at > Utc::now().naive_utc()`) compare
  deux instants : hors du symptôme.
- **Le splitting** : cinq modules — `kesh-core` (horloge), `kesh-db` (dépôts `audit_log`, `reconciliation`), `kesh-api`
  (commentaire de route, trois fichiers de tests), `kesh-i18n` (quatre catalogues), `frontend` (`audit-log`, spec, garde) —
  plus les manuels. **À la limite, non au-delà** : la story reste une (Q11). Si une validation recycle un défaut, le
  découpage naturel suit les deux issues (36-10a journal d'audit, `kesh-core` + `kesh-db/audit_log` + écran ; 36-10b
  réconciliation, `kesh-db/reconciliation` + garde), D1 dans la première.

### Fichiers communs avec les autres stories de la 0.18.0

Le fichier d'epic ne tient de table des fichiers communs que pour la phase 1. Recoupements probables, à constater à
l'intégration : `crates/kesh-db/src/repositories/reconciliation.rs` et `kesh-api/tests/reconciliation_e2e.rs` avec la
**36-7** (désancrage des tests de 2026 : les dates fixes `2026-06-15` de ces tests sont des dates de **données**, non le jour
de l'horloge ; si la 36-7 les touche, hunks distincts) ; `messages.ftl` ×4 et `docs/manual/fr/user-manual.tex` avec la
**36-15** (« compte analytique ») et la **36-17** — paragraphes et blocs distincts (`audit-log-filter-*` ici) ;
`date_du_jour_garde.rs` avec toute story qui ajoute un site de date (aucune annoncée).
⚠️ `frontend/src/lib/shared/utils/local-date-guard.test.ts` avec la **36-7a**, parallèle : elle **ajoute** une entrée à
l'allowlist des specs et fait passer les mentions de `:29`, `:263`, `:318` de « trois » à « quatre », quand celle-ci en
**retire** une (« trois » → « deux ») ; après les deux fusions, la valeur juste est le nombre d'entrées réellement
présentes (attendu : trois), **recompté à la source à l'intégration**, sans additionner les deux diffs (C-e350-18). `CHANGELOG.md` : aucun (C-e360-11).

### Ce qui doit rester vrai

- Une seule fonction du fuseau : `jour_suisse` et ses deux inverses de D1, dans `kesh-core` (C-e350-16) ; `kesh-db` les
  appelle, ne recalcule rien.
- Liste et export du journal : une seule clause `WHERE` (`push_where_clauses`).
- Proposition et acceptation de la réconciliation : la même règle (assertion `proposed == accepted`, AC6).
- La lecture seule du MCP et les contrats d'erreur, inchangés. Fixtures synthétiques seulement.

### Références

- Issues #1147, #1053 ; voisines #669 (horodatages UTC), #889 (garde frontend), #929 (35-5b).
- Fiches 35-5a0, 35-5a, 35-5b (Hors périmètre : #1053), 35-5c ; registre `e35-choix-autonomes-e350.md` C-e350-16, -25 ;
  `e35-choix-autonomes-e355b.md` C-e355b-6.
- `CLAUDE.md` : § *Inventorier les sites NON RÉSOLUS* ; § *Exception `kesh-db`* ; § *Le prompt d'une passe doit NOMMER le
  manuel*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — Fiche rédigée (Epic 36, phase 2, 0.18.0) par l'agent de rédaction des fiches (Opus 5.5), code lu à
  `0bf3c3b4b`. Inventaire I1 à I5 lancé (commandes et sorties ci-dessus) : deux sites dans le périmètre
  (`kesh-db/src/repositories/audit_log.rs:228`, `:236` ; `reconciliation.rs:237`), les autres exclus ligne lue citée.
  Choix C-36-10-1 à -5 (`e36-choix-autonomes-36-10.md`). Aucune migration. Validation non lancée.
  **Écart à la consigne Edit** : onze corrections de numéros de ligne (relus après rédaction : `audit_log.rs` `:216`,
  `:267`, `:314`, `:211-215`, `:204`, `:113`, `:311-313`, `:715-780` ; `reconciliation.rs` `:741`, `:195-229` ;
  `reconciliation_e2e.rs:3311`), faites dans la fiche neuve par un court script Python qui contrôlait l'unicité de chaque
  chaîne avant de la remplacer, au lieu de onze appels de l'outil Edit. Rien d'autre.
- 2026-11-15 — **Validation P1** (0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW ; `kesh-gate-logs/e36-10/validation-p1.md`),
  remédiée par l'agent de rédaction (Opus 5.5), par l'outil Edit seul. Prémisses relues à `0bf3c3b4b`, toutes confirmées :
  `routes/audit_log.rs:374` (`.format("%Y-%m-%d %H:%M:%S%.3f")`), `:279` (`"Date (UTC)",`), `:113` (JSON) ;
  `date_du_jour_garde.rs:42` (`use regex::Regex;`), `kesh-api/Cargo.toml:92` (`regex = "1.10"`) ;
  `routes/payment_batches.rs:333` (`#[cfg(test)]`), `:355` (`requested_execution_date: moment.date(),`) ;
  `fiscal-years.helpers.ts:52` (`const year = new Date().getFullYear();`) ; `audit-log.spec.ts:8`
  (`import { createUser, USER_PASSWORD } from './helpers/api-fixtures';`).
  - V1-1 (MEDIUM) : le CSV n'écrit pas de `Z` ; « Le CSV », D6 et AC3 disent le format réel, **inchangé** (aucun changement
    de contrat de l'export) ; registre C-36-10-5 annoté.
  - V1-2 (MEDIUM) : motif de D5 (a) ramené à `\bDATE\s*\(` (la crate `regex` n'a pas de lookahead), sonde de compilation et
    de comptage ; registre C-36-10-4 annoté.
  - V1-3 (LOW) : D1 nomme l'algorithme (`Single`, `Ambiguous` → min, `None` → dichotomie à la milliseconde) ; D3 plafonne
    **toujours** la borne basse à `1000-01-01 00:00:00.000` ; AC4 le constate ; mutations M7, M8 ; registre C-36-10-2
    annoté ; tâche de l'étape 0 allégée.
  - V1-4 (LOW) : `payment_batches.rs:355` nommé dans I3 (`mod tests`) ; liste close de D5 (b) à trois.
  - V1-5 (LOW) : AC6 borne son balayage par instants de début et de fin, été et hiver.
  - V1-6 (LOW) : `fiscal-years.helpers.ts:52` trié dans I5 (année du poste, hors symptôme).
  - AC9 / tâche : `todayInZurich` à importer dans `audit-log.spec.ts:8`.
  - Symptômes greppés dans la fiche et le registre : `…Z`, `22:00:00.000Z` (CSV), `non suivi de`, `plafonnée sinon`,
    `occurrence de test`, `mut/M6.log` — remplacés ; les `…Z` restants désignent la réponse JSON ou des instants UTC
    d'AC1/AC5, à juste titre.
- 2026-11-15 — **Validation P2** (0 CRITICAL, 0 HIGH, 1 MEDIUM, 4 LOW ; `kesh-gate-logs/e36-10/validation-p2.md`),
  remédiée par l'agent de rédaction (Opus 5.5), outil Edit seul. Prémisses relues, toutes confirmées :
  `local-date-guard.test.ts:29` (« close de trois entrées »), `:263` (« Les trois sites admis »), `:318` (« les trois sites
  de l’allowlist ») ; `ls crates/kesh-db/src/post_restore/` (quatre `.sql`), `post_restore.rs:378`
  (`include_str!("../migrations/20261010000001_lettering_documents_backfill.sql")`), aucun `DATE(` dans ces `.sql` ;
  `audit_log.rs:202-205` (précondition de `push_where_clauses`). La preuve de V2-1 (`push_bind` ne panique pas sur l'an
  999) est reprise du rapport, sans relecture de la source de `sqlx`.
  - V2-1 (MEDIUM) : le calcul des bornes, plafond compris, devient la fonction pure `bornes_du_filtre` de
    `kesh-db/src/repositories/audit_log.rs` (aucun module neuf), testée par `bornes_du_filtre_jours_de_zurich_et_plafond`,
    qui tue M7 (et M1, M2, M5) ; « ou par une mutation » retiré d'AC4 ; table des tests et mutations à jour ; D3 corrige sa
    justification (« ni de `push_bind` ») ; registre C-36-10-2 annoté.
  - V2-2 (LOW) : précondition `[1000-01-01, 9999-12-31]` écrite à D1.
  - V2-3 (LOW) : AC6 porte le verdict en instants fixes (`22:00Z` été, `23:00Z` hiver, ancres `e` et `e − 1`).
  - V2-4 (LOW) : `local-date-guard.test.ts:29`, `:263`, `:318` (« trois » → « deux ») nommés dans D7, AC9 et la tâche ;
    fichier commun avec la 36-7a écrit (« trois » → « quatre » de son côté, recompte à l'intégration).
  - V2-5 (LOW) : angle mort des `.sql` de `post_restore/` écrit à D5 et à la tâche D5.
