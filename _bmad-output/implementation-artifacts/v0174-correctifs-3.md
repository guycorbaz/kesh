# Story v0174-correctifs-3 : les écrans — toutes les factures d'un lot (#888), la date du jour à l'heure suisse (#889), le projet archivé d'un brouillon (#872), la date du verrou au repassage en brouillon (#869)

Status: ready-for-dev

*Fiche sœur de `v0174-correctifs.md` (l'index), rédigée le 2026-11-05 en autonomie (Opus 5.5), worktree
`/home/gcorbaz/devel/kesh-0174c`, branche `story/v0174-correctifs` partie de `7bd4d5c0`. Release **0.17.4**, **sans
migration**. Registre : `v0174-choix-autonomes-v0174c.md`, choix C-v0174c-12 à 19. Clé de statut `v0174-correctifs-3`.
Issues **#888** (P3), **#889** (P4), **#872** (P3), **#869** (P3, côté écran), en `refs` seulement. Numéros de ligne de
`7bd4d5c0` (chemins relatifs à `frontend/src/` sauf mention). **Se développe après la fiche 2** (#869 lit
`details.lockedThrough`, que la fiche 2 pose).*

## Story

En tant que Comptable qui prépare un lot de virements, je veux voir toutes mes factures validées, non les deux cents
premières ; en tant qu'utilisateur qui travaille après minuit, je veux que « aujourd'hui » soit le jour de Zurich, non la
veille ; en tant que Comptable qui rouvre un brouillon, je veux voir le projet qui lui est imputé même archivé ; en tant que
Comptable à qui le repassage en brouillon est refusé, je veux savoir jusqu'à quelle date les livres sont verrouillés.

## Contrainte

Modules : `frontend` (features `payment-batches`, `supplier-invoices`, et le *rollout* de #889), `kesh-i18n` (quatre locales).
Aucun changement serveur dans cette fiche. #889 est un **rollout mécanique** d'un helper (Règle de splitting préventif :
revu fichier par fichier).

---

## 3.1 — #888 : le lot de paiement propose toutes les factures validées

### Ce que le code dit

1. `loadOpenInvoices` (`routes/(app)/payment-batches/+page.svelte:53-56`) lit **une** page :
   `listSupplierInvoices({ limit: 200, status: ['open'] })`, puis filtre `status === 'open'` en défense. **Prémisse
   confirmée.** Le serveur écrête à 200 (`LIMITE_MAX`, `crates/kesh-api/src/services/factures_fournisseur.rs:30`, `clamp`
   `:116`), rend `total` filtré par statut et trie `si.invoice_date DESC, si.id DESC`
   (`crates/kesh-db/src/repositories/supplier_invoices.rs:252`) : un ordre total, la lecture par `offset` ne saute ni ne
   double une facture tant que la liste ne change pas.
2. La création d'un lot refuse plus de `MAX_IN_CLAUSE_IDS` = 500 identifiants (`routes/payment_batches.rs:173-178`,
   `crates/kesh-db/src/repositories/reconciliation.rs:1083`), en `400 VALIDATION_ERROR` affiché par le formulaire
   (`formError = err.message`, `+page.svelte:99`).
3. **Défaut voisin, non traité** : la liste des lots ne lit que les cent premiers (`listPaymentBatches({ limit: 100 })`,
   `:43`) — issue demandée (C-v0174c-12).

### Conception (C-v0174c-12)

`loadOpenInvoices` lit les pages de 200 (`offset` 0, 200, 400…) jusqu'à avoir `total` factures, et s'arrête sur une page vide ;
le filtre client reste. Un seul site (le montage et la relecture après un lot l'appellent déjà, `:60`, `:97`).

### Acceptance Criteria (3.1)

**AC-3.1** — `payment-batches-page.test.ts` : `total = 450`, trois pages simulées (200, 200, 50) → trois appels
`{ limit: 200, offset: 0|200|400, status: ['open'] }`, **450** cases à cocher. **Rouge avant** (200 cases, un appel).
**AC-3.2** — `total = 450` mais la deuxième page revient vide : deux appels, 200 cases, aucun troisième appel (pas de boucle).
**AC-3.3** — `total = 0` : un appel, « Aucune facture fournisseur validée à payer. ». Les tests existants du fichier (`:55-`,
AC-27 de la v017-4b) restent verts, leurs simulacres complétés de `total` si besoin.
**AC-3.4 — E2E** (`frontend/tests/e2e/`, spec des lots existante ou voisine) : 205 factures validées créées par l'API
(fixture synthétique) → le formulaire en propose 205. Capture.

**Mutation (P3)** : **MC-888** — la boucle s'arrête après la première page : AC-3.1 rougit.

---

## 3.2 — #889 : la date du jour en heure locale, un helper unique

### Ce que le code dit — inventaire clos

4. Le motif de l'issue, rejoué : `git grep -nE 'toISOString\(\)' -- frontend/src ':!*.test.ts' ':!*.test-helper*'` → **20**
   lignes, dont **15** de la forme `.toISOString().slice(0, 10)` (les quinze sites de l'issue, recalés) et 5 qui ne la portent
   pas (`mcp-server.types.ts:53` et `mcp-time.ts:9`, commentaires ; `mcp-time.ts:50`, instant RFC 3339 voulu ;
   `reminder-sent-at.ts:19`, commentaire ; `reminder-sent-at.ts:27`, `.slice(0, 19)`, instant UTC voulu). Formes voisines
   cherchées : `git grep -nE "toJSON\(\)|split\('T'\)|substring\(0, ?10\)|substr\(0, ?10\)|toLocaleDateString|toDateString\(\)"`
   → une seule, `routes/(app)/users/+page.svelte:274` (§ 6) ; `getUTC*` / `setUTC*` : deux validations de date pure
   (`manual-statement.ts:107`, `open-items.ts:43`) et les deux arithmétiques des sites 5 et 13 (`form-helpers.ts:62`,
   `settings/+page.svelte:219`).
5. **Les quinze sites, triés** (le numéro de l'issue, puis la ligne à `7bd4d5c0`) :

| # | Site | Ce qu'il calcule | Sort |
|---|---|---|---|
| 1 | `lib/components/invoices/InvoiceForm.svelte:115-118` (`todayIso`) | date par défaut d'une facture | **corrigé** |
| 2 | `lib/features/invoices/SettleInvoiceDialog.svelte:76-78` | date par défaut d'un règlement | **corrigé** |
| 3 | `lib/features/invoices/WriteOffDialog.svelte:63-65` | date par défaut d'un solde | **corrigé** |
| 4 | `lib/features/invoices/invoice-helpers.ts:124` (`addDaysIso`) | arithmétique sur les composants, en UTC | conforme |
| 5 | `lib/features/journal-entries/form-helpers.ts:63` (`nextDay`) | arithmétique sur les composants, en UTC | conforme |
| 6 | `lib/features/reminders/ManualReminderDialog.svelte:45-47` | date par défaut et `max` d'un rappel manuel | **corrigé** |
| 7 | `lib/features/reminders/reminder-sent-at.ts:24` | « aujourd'hui ? » du `sent_at` | **corrigé**, avec le 6 (C-v0174c-15) |
| 8 | `lib/features/reports/reports.api.ts:383` | nom du fichier `balance-agee-{date}.csv` | **corrigé** |
| 9 | `routes/(app)/invoices/[id]/+page.svelte:106` | date de l'avoir créé | **corrigé** |
| 10 | `routes/(app)/invoices/due-dates/+page.svelte:297` | nom du fichier `echeancier-{date}.csv` | **corrigé** |
| 11 | `routes/(app)/payment-batches/+page.svelte:37` | date d'exécution par défaut | **corrigé** |
| 12 | `routes/(app)/payment-batches/[id]/+page.svelte:26` | date de paiement par défaut (confirmation) | **corrigé** |
| 13 | `routes/(app)/settings/+page.svelte:217-221` (`maxLockDate`) | veille, borne du verrou | conforme (C-v0174c-14) |
| 14 | `routes/(app)/settings/api-keys/+page.svelte:53-55` (`computeMinDate`) | lendemain, `min` de l'expiration | **corrigé** (`addDaysIso(todayLocal(), 1)`) |
| 15 | `routes/(app)/supplier-invoices/[id]/+page.svelte:98` | date de paiement par défaut | **corrigé** |

   Soit **12 corrigés et 3 conformes** (= 15). S'y ajoute, hors inventaire de l'issue et sans défaut (déjà local), la copie
   `todayISO` de `lib/features/journal-entries/JournalEntryForm.svelte:84-90`, remplacée par le helper (DRY, C-v0174c-13).
   Le site 16 du relevé de la fiche 0.17.2 (`SupplierInvoiceForm.svelte`) emploie déjà `todayLocal` (`:157`).
6. **Les affichages d'instants** (second commentaire de l'issue, douze sites ; `git grep -nE "formatSwissDate\(.*\.slice\(0, ?10\)"`
   rejoué : les douze, recalés, plus `api-keys/+page.svelte:102` et `supplier-invoice-unvalidate.ts:65`). Depuis la 0.17.2,
   `formatSwissDate` rend la date à Zurich d'un horodatage (`lib/features/reports/reports.api.ts:237-241`) : retirer le
   `.slice(0, 10)` suffit. Lu au code serveur (C-v0174c-16) :

| # | Site | Champ | Nature (Rust) | Sort |
|---|---|---|---|---|
| a | `routes/(app)/invoices/[id]/+page.svelte:1162` | `paidAt` | date du règlement à minuit (`invoice_settlements_write.rs:323-324`) | conforme |
| b | `routes/(app)/invoices/[id]/+page.svelte:1171` | `pdfFrozenAt` | `NOW(3)` (`repositories/invoices.rs:2753`, `:2823`) | **corrigé** |
| c | `routes/(app)/invoices/[id]/+page.svelte:1179` | `emailedAt` | `NOW(6)` (`invoices.rs:2654`) | **corrigé** |
| d | `lib/features/reminders/ReminderHistory.svelte:58` | `sentAt` | instant (UTC, ou midi UTC d'une date passée, C-v0174c-15) | **corrigé** |
| e | `lib/features/reminders/ReminderHistory.svelte:76` | `cancelledAt` | `UTC_TIMESTAMP(6)` (`invoice_reminders.rs:176`) | **corrigé** |
| f | `routes/(app)/invoices/due-dates/+page.svelte:472` | `paidAt` | date à minuit | conforme |
| g | `routes/(app)/invoices/reminders/+page.svelte:362` | `lastReminderAt` | `MAX(sent_at)` (`dunning_eligibility.rs:80`) | **corrigé** |
| h | `routes/(app)/payment-batches/[id]/+page.svelte:110` | `confirmedAt` | `CURRENT_TIMESTAMP(3)` (`payment_batches.rs:624`) | **corrigé** |
| i | `routes/(app)/supplier-invoices/[id]/+page.svelte:152` | `paidAt` | date à minuit (`supplier_invoices.rs:1922-1923`) | conforme |
| j | `routes/(app)/supplier-invoices/[id]/+page.svelte:241` | `batch.confirmedAt` | idem h | **corrigé** |
| k | `routes/(app)/supplier-invoices/[id]/+page.svelte:350` | `batch.confirmedAt` | idem h | **corrigé** |
| l | `lib/features/imported-supplier-invoices/imported-supplier-invoices.types.ts:65` | `createdAt` | instant | **corrigé** (et son commentaire « angle mort assumé », `:62`) |
| — | `routes/(app)/settings/api-keys/+page.svelte:102` | `expiresAt` | date de calendrier envoyée `T23:59:59Z` | conforme (exception 0.17.2) |
| — | `lib/features/supplier-invoices/supplier-invoice-unvalidate.ts:65` | dates `AAAA-MM-JJ` | date pure | conforme |

   Soit **9 corrigés, 3 conformes** sur les douze de l'issue (prémisse corrigée : les trois `paidAt` ne sont pas des instants),
   plus deux conformes hors liste. **Site hors inventaire, corrigé** : `formatDate` de `routes/(app)/users/+page.svelte:271-279`
   lit un horodatage UTC sans décalage par `new Date(iso)` (heure **locale**) — remplacé par `formatSwissDate(iso)` (le `'—'`
   d'une date illisible gardé).
7. Le helper existe : `todayLocal` (`lib/features/open-items/open-items.ts:26-31`, testé `open-items.test.ts:30-35`),
   déjà importé hors de sa feature (`SupplierInvoiceForm.svelte:80`, `routes/(app)/bank-accounts/+page.svelte:41`). Le patron
   de test qui épingle le fuseau existe : `lib/shared/utils/pin-time-zone.test-helper.ts` (`pinTimeZone`).

### Conception (C-v0174c-13 à 17)

- `todayLocal` déménage dans `lib/shared/utils/local-date.ts` (son test avec lui), réexporté par `open-items.ts` ; les sites
  corrigés l'importent de `$lib/shared/utils/local-date`.
- Sites 1-3, 6, 9, 11, 12, 15 : `todayLocal()` ; sites 8, 10 : `todayLocal()` dans le nom du fichier ; site 14 :
  `addDaysIso(todayLocal(), 1)` ; site 7 : `if (sentAt === todayLocal(now))` (l'envoi reste l'instant UTC, C-v0174c-15) ;
  `JournalEntryForm` : `todayLocal()`.
- Affichages b-e, g, h, j-l : `formatSwissDate(x)` sans `.slice(0, 10)` ; `users/+page.svelte` : `formatSwissDate`.
- **Garde** (C-v0174c-17) : `lib/shared/utils/local-date-guard.test.ts` lit les sources `src/**/*.{ts,svelte}` hors tests
  (`readdirSync` / `readFileSync`, patron de `lib/shared/i18n-un-repli-par-cle.test.ts:45`, `:240-247` ; filtre des
  `*.test.ts`, `*.test-helper*`) et relève `toISOString().slice(0, 10)` et `formatSwissDate(….slice(0, 10))` ; chaque relevé
  doit figurer à une allowlist nominative `{ fichier, motif }` — les sites 4, 5, 13 et les deux conformes hors liste (`api-keys`,
  `supplier-invoice-unvalidate`) ; les trois `paidAt` (a, f, i) y entrent aussi, motif « date du règlement à minuit » —, et
  aucune entrée de l'allowlist ne doit être orpheline. Sonde positive au Dev Agent Record (MC-889-g).

### Acceptance Criteria (3.2)

Tous les tests de cette section épinglent `pinTimeZone('Europe/Zurich')` et l'horloge
(`vi.useFakeTimers({ toFake: ['Date'] })`, `vi.setSystemTime(new Date('2026-11-04T23:30:00Z'))` — 00 h 30 le 5 novembre à
Zurich) :

**AC-3.5** — `todayLocal()` = `'2026-11-05'` (test déplacé, plus ce cas).
**AC-3.6 — Par site corrigé** (un test par site, dans le fichier de test voisin existant ou neuf) : la valeur par défaut
affichée ou envoyée vaut `2026-11-05` — facture (1), règlement (2), solde (3), rappel manuel et son `max` (6), avoir envoyé
`{ date: '2026-11-05' }` (9), date d'exécution (11), date de paiement d'un lot (12), `min` de l'expiration d'une clé
`2026-11-06` (14), date de paiement d'une facture fournisseur (15) ; noms de fichier `balance-agee-2026-11-05.csv` (8),
`echeancier-2026-11-05.csv` (10) ; `JournalEntryForm` (date par défaut `2026-11-05`). **Rouges avant correctif** (sauf
`JournalEntryForm`, déjà local : vert avant, il fige le remplacement).
**AC-3.7 — Le rappel manuel** (`reminder-sent-at.test.ts`) : `reminderSentAtDateTime('2026-11-05', now)` rend l'instant courant
(`2026-11-04T23:30:00`, ≤ now) ; `('2026-11-04', now)` rend `2026-11-04T12:00:00` (passé). Les tests existants (`:4-34`) restent
verts. **Rouge avant** (le premier rend `2026-11-05T12:00:00`, futur, que le serveur refuse — `dunning_reminders.rs:261`).
**AC-3.8 — Les affichages** : sous `pinTimeZone('Europe/Zurich')`, un horodatage `2026-11-04T23:30:00` s'affiche `05.11.2026`
pour b, c, d, e, g, h, j, k, l et `users` ; les tests existants qui figent ces affichages (`ReminderHistory.test.ts:36-45` : à
`08:00`/`10:00` UTC, même date) restent verts. Les `paidAt` (a, f, i) : `2026-11-04T00:00:00` → `04.11.2026`. **Rouges avant**
(sites corrigés).
**AC-3.9 — La garde** : verte après correctif ; rouge si l'on réintroduit `new Date().toISOString().slice(0, 10)` dans un
fichier hors allowlist (sonde, MC-889-g) ; rouge si une entrée de l'allowlist ne correspond plus à rien.

**Mutations (P4)** : **MC-889-a** — `todayLocal` rend l'UTC (`toISOString().slice(0, 10)`) : AC-3.5, AC-3.6 rougissent.
**MC-889-b** — `reminderSentAtDateTime` compare de nouveau à l'UTC : AC-3.7 rougit. **MC-889-c** — un `.slice(0, 10)`
rétabli sur un affichage (b) : AC-3.8 et la garde rougissent. **MC-889-g** — sonde de la garde (AC-3.9).

---

## 3.3 — #872 : le projet archivé d'un brouillon reste affiché

### Ce que le code dit

8. **Prémisse corrigée** : le projet est celui de l'**en-tête** de la facture fournisseur (`fProjectId`, un seul `<select>`,
   `lib/features/supplier-invoices/SupplierInvoiceForm.svelte:724-740`), non celui d'une ligne. La page d'édition lit les
   projets archivés compris (`routes/(app)/supplier-invoices/[id]/edit/+page.svelte:82`) et ne garde, des archivés, que celui
   du brouillon (`:114`) ; le formulaire ne rend que les racines (`parentId === null`) et leurs enfants (`:733-737`). Un
   sous-projet archivé dont la racine est archivée — donc écartée — n'est rendu nulle part : le `<select>` (lié à
   `fProjectId`) n'a pas d'option à sa valeur et s'affiche sans sélection ; la valeur repart au `PUT`. **Prémisse confirmée
   pour le reste.** Le test existant (`edit-page.test.ts:261-`, « les projets archivés ne sont offerts que s’ils sont celui du
   brouillon ») ne monte qu'un archivé **racine**.
9. Un enfant actif sous une racine archivée ne se forme pas par l'API (C-v0174c-18 : `routes/projects.rs:161-165`, `:329`,
   `:338-340`) ; les sélecteurs qui ne lisent que les actifs ne sont pas touchés.

### Conception (C-v0174c-18)

`SupplierInvoiceForm.svelte` rend, après l'arbre, chaque projet de `projects` dont `parentId` n'est pas `null` et dont le
parent n'est pas dans `projects` (orphelin), à plat ; toute option d'un projet `archived` porte la marque « (archivé) »
(clé neuve `supplier-invoices-project-archived-option` = « { $label } (archivé) », quatre locales). La page d'édition ne
change pas.

### Acceptance Criteria (3.3)

**AC-3.10** — `edit-page.test.ts` : projets `R` (racine archivée), `R/E` (enfant archivé), `A` (actif) ; brouillon sur `R/E`
→ l'option de `R/E` existe, porte « (archivé) », `sel.value === String(id de R/E)` ; `R` n'est pas offerte. **Rouge avant.**
**AC-3.11** — le cas existant (`:261-`, archivé racine) : son option porte désormais « (archivé) » (assertion ajoutée) ;
les actifs ne portent pas la marque. La création (`supplier-invoices/new`) ne change pas (elle ne lit que les actifs).

**Mutation (P3)** : **MC-872** — les orphelins ne sont pas rendus : AC-3.10 rougit.

---

## 3.4 — #869 : le refus du repassage en brouillon dit la date du verrou

### Ce que le code dit

10. `supplierInvoiceUnvalidateMessage` (`lib/features/supplier-invoices/supplier-invoice-unvalidate.ts:73-`) : le cas
    `PERIOD_LOCKED` (`:154-158`) rend une phrase sans date ; `SupplierInvoiceUnvalidateDetails` (`:54-61`) n'a pas de champ
    pour le verrou ; le patron « nommé, avec repli » existe (`LATER_FISCAL_YEAR_CLOSED`, `:135-149`, `fiscalYearName`). La
    page passe `err.details` (`routes/(app)/supplier-invoices/[id]/+page.svelte:432-437`).

### Conception (C-v0174c-19)

`lockedThrough?: string | null` à l'interface ; avec lui, clé neuve `supplier-invoices-unvalidate-blocked-period-locked-named`
— « L’écriture d’achat de cette facture est datée sous le verrou de période, posé jusqu’au { $date } : pour pouvoir la repasser
en brouillon, un administrateur doit faire reculer le verrou avant sa date. » —, date en forme suisse (`swissDate`, `:64-66`) ;
sans lui, la clé d'aujourd'hui. Quatre locales ; le repli en dur dit le FTL fr-CH mot pour mot (règle de l'en-tête du fichier,
`:24`).

### Acceptance Criteria (3.4)

**AC-3.12** — `supplier-invoice-unvalidate.test.ts` (existant) : `('PERIOD_LOCKED', { lockedThrough: '2026-03-31' })`
→ le texte contient `31.03.2026` ; `('PERIOD_LOCKED', null)` → la phrase d'aujourd'hui. **Rouge avant.**
**AC-3.13 — E2E** (spec du cycle du brouillon fournisseur, `tests/e2e/supplier-invoice-draft-cycle.spec.ts` ou voisine) :
facture validée, verrou posé après sa date, *Repasser en brouillon* → le message nomme la date en forme suisse. Capture.

**Mutation (P3)** : **MC-869** — le cas nommé n'est jamais pris : AC-3.12 rougit.

---

## Tâches

1. Rouges d'abord : AC-3.1, 3.5-3.8, 3.10, 3.12 (constatés, notés).
2. #888 ; #889 (helper, douze sites, `JournalEntryForm`, neuf affichages et `users`, garde) ; #872 ; #869.
3. i18n : deux clés neuves, quatre locales (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`) ;
   `npm run lint-i18n-ownership` ; gardes `i18n-keys.test.ts` et `i18n-un-repli-par-cle.test.ts` (décomptes **recomptés**,
   avant → après, au Dev Agent Record).
4. Manuel utilisateur (fiche-index, § *Manuel*).
5. Captures (thème clair, base de démonstration jetable `kesh_v0174c`, `kesh-gate-logs/v0174c/captures/`) : formulaire d'un lot
   avec plus de 200 factures (pied de la liste visible) ; édition d'un brouillon au projet archivé sous racine archivée ;
   refus du repassage en brouillon nommant la date du verrou.
6. Mutations MC-888, MC-889-a/b/c/g, MC-872, MC-869 ; gates (fiche-index).

## Dev Notes

- **#889** : ne pas toucher aux valeurs **envoyées** autrement que par la date par défaut (une date choisie par l'utilisateur
  part telle quelle) ; ne pas toucher aux sites conformes. Le fuseau du runner de la CI est UTC : sans `pinTimeZone`, les
  tests de la section se tromperaient de jour selon le poste.
- **#888** : 450 factures, 3 requêtes ; aucune borne haute n'est posée (une société à 10 000 factures validées non payées ferait
  50 requêtes) — déclaré, non traité.
- **#872** : la validation refuse toujours un projet archivé (`user-manual.tex:1705`) ; la marque prévient.

## Change Log

- 2026-11-05 — fiche créée (Opus 5.5, autonomie). Prémisses : #888 **confirmée** (§ 1) ; #889 **confirmée** (15 sites : 12
  corrigés, 3 conformes ; § 5) et son commentaire **corrigé en partie** (les trois `paidAt` sont des dates, 9 affichages
  corrigés sur 12 ; § 6), un site hors inventaire (`users`) ; #872 **corrigée** (projet d'en-tête, non de ligne ; § 8) ;
  #869 côté écran (§ 10). Défaut voisin : la liste des lots limitée à 100 (issue demandée).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
