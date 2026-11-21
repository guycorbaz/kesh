# Story 36-18b : les montants « 1.250 » des réglages, du catalogue et des filtres — prix d'article, frais de rappel, montant minimal, filtres des écritures (#1173, #1175, #1176, #1177)

> **Reprise par la 48-10 (fiche `48-10-montants-ambigus-reglages-filtres.md`).** Cette fiche est conservée telle quelle, à titre d'historique ; elle est `superseded`.

Status: backlog

*Fiche issue du découpage de la 36-18 (`36-18-montants-ambigus-autres-ecrans.md`, `Status: split`), approuvé par
l'orchestrateur le 2026-11-15 (C-36-18-10). Rédigée par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), code lu
à `fed34a9c7`. Release **0.18.0**, phase 2, décision **D6** de Guy du 2026-11-15 (#1177 en dernier). **Aucune
migration**, aucun changement serveur ni d'API. Voie **2**, bases `kesh_e3618b`, `kesh_e2e_e3618b`, `kesh_ci_e3618b`
(voie 1). **Part de la 36-18a fusionnée** (aides `amountEntryPayload`, `amountEntryProblem`, clés `common-amount-*`,
`montants-ambigus.spec.ts`), elle-même partie de la **36-15 fusionnée**. **Les décisions communes sont celles de la
36-18a** (`36-18a-montants-ambigus-livres.md`, D1 à D4 et D9 ; registre C-36-18-1 à C-36-18-10) : cette fiche y renvoie
et ne les répète pas. Inventaires bruts : `/home/gcorbaz/devel/kesh-gate-logs/e36-18/fiche-inventaires.log`. Validation
non lancée.*

## Story

En tant que titulaire qui tient ses livres réels dans Kesh, je veux que le prix d'un article, les frais d'un niveau de
rappel, le montant minimal d'une facture et les bornes du filtre des écritures se lisent comme le montant d'une écriture —
« 1.250 » refusé comme ambigu, « 1'250 » accepté —, sans que rien de juste jusqu'ici devienne refusé, afin qu'un prix
d'article mille fois trop bas ne passe pas sur mes factures, et qu'un filtre ne me montre pas une liste fausse sans le dire.

## Ce qui se lit au code (`fed34a9c7`)

La règle, le lecteur et leurs aides : fiche 36-18a, § *La règle*. Les aides génériques (`amountEntryPayload`,
`amountEntryProblem`) et les clés `common-amount-ambiguous`, `common-amount-unreadable` (`{ $max }`) n'existent qu'une
fois la 36-18a fusionnée.

**1. Prix d'un article (#1173)** — `frontend/src/lib/features/products/product-helpers.ts` :
- `:53` `const PRICE_RE = /^(0|[1-9][0-9]{0,14})(\.[0-9]{1,4})?$/;`, `:71-73` `normalizePriceInput`
  (`raw.trim().replace(',', '.')`), `:82-87` `classifyPriceInput` (`empty` / `negative` / `invalid` / `ok`) — « 1.250 » `ok`,
  « 1,250 » normalisé `1.250` `ok` ;
- `:61-65` `isValidPrice` : **aucun appelant de production** (`git grep -n "isValidPrice" -- frontend/src` : le test et deux
  doc-comments, `:59`, `:80`) ;
- page `frontend/src/routes/(app)/products/+page.svelte` : `:327-334` messages (`product-error-price-required`, `-negative`,
  `-invalid`), `:368` `unitPrice: normalizePriceInput(formPrice),`, champ `:653-662` ;
- rouvert `:306-308` `const fixed = new Big(p.unitPrice).toFixed(4);` `formPrice = fixed.replace(/(\.\d{2})0+$/, '$1');` —
  sortie à deux ou quatre décimales, jamais trois (« 1.2340 » reste `1.2340`, « 1.2500 » → `1.25`, « 0.1230 » reste
  `0.1230`) : **jamais ambiguë**, à prouver par test (AC1) ;
- serveur `crates/kesh-api/src/routes/products.rs:210` `scale_within(&unit_price, MAX_DECIMAL_SCALE)` : « 1.250 » passe, lu
  1,25, puis repris par chaque ligne de facture qui emploie l'article.

**2. Frais de rappel (#1175)** — `frontend/src/routes/(app)/settings/dunning/+page.svelte` (306 lignes) : `:104`
`if (fFee.trim() === '' || Number.isNaN(Number(fFee))) {` ; `:111`, `:116` `feeAmount: fFee` envoyé **brut** ; champ
`:284` ; rouvert `:88` `fFee = level.feeAmount;` (colonne `DECIMAL(7,2)`,
`crates/kesh-db/migrations/20260714000001_dunning_config.sql:32` : jamais ambigu). ⚠️ **L'issue dit plus que le code**
(commentée par l'orchestrateur) : le serveur refuse déjà « 1.250 » (`crates/kesh-api/src/routes/dunning_levels.rs:82-86`
`if !scale_within(fee, 2) {` … « Les frais de rappel ne peuvent avoir plus de 2 décimales. », sans `normalize()`). Le
défaut réel : ce message pousse à écrire `1.25` pour mille deux cent cinquante, « 1'250 » est refusé (`Number("1'250")` est
`NaN`), et `Number(" 1.25 ")` admet des blancs qui partent avec le brut (C-36-18-6). **Aucun test de composant** pour la page.

**3. Montant minimal d'une facture (#1176)** — `frontend/src/routes/(app)/settings/invoicing/+page.svelte` : `:276`
`minimumInvoiceAmount: minimumAmount.trim() === '' ? null : minimumAmount.trim(),` envoyé tel quel ; serveur
`crates/kesh-api/src/routes/company_invoice_settings.rs:489-496` `scale_within(&min.normalize(), 2)` : « 1.250 » normalisé
`1.25`, **accepté** ; champ `:682-688`, sans message ; rouvert `:213` et `:307` (colonne `DECIMAL(19,4)`,
`20261001000002_invoice_minimum_amount.sql:14` : quatre décimales, jamais ambigu — forme exacte relevée à l'étape 0, AC3).

**4. Filtres de montant de la liste des écritures (#1177, P4)** — `frontend/src/routes/(app)/journal-entries/+page.svelte` :
`:80-81` `amountMin: amountMin || undefined,` envoyé brut (et dans l'URL, `:179-185`, par `serializeQuery`,
`frontend/src/lib/features/journal-entries/query-helpers.ts:29-34`) ; initialisé depuis l'URL `:146-147` ; champs `:398-418`.
Le serveur lit « 1.250 » 1,25 et refuse « 1'250 » ou « 1,25 » en 400 (`crates/kesh-api/src/services/ecritures.rs:96-101`),
que la page tait (`:111-113` `entries = []; total = 0;`). Rien n'est écrit : seul le filtre est faux.

### Inventaire par le symptôme

Fait pour toute la 36-18 (fiche 36-18a, § *Inventaire*, sept motifs, sonde positive : le premier motif rend
`settings/dunning/+page.svelte`, site connu). Sites de ce lot : les quatre ci-dessus. Aucun site hors des huit issues.

## Décisions

*Lecture, aides, forme rouverte, messages communs et rognage : D1 à D4 et D9 de la 36-18a, réemployés sans modification —
D3 compris dans sa forme remédiée à la validation P1 de la 36-18a (V1-6) : tout champ fautif porte la bordure,
`aria-invalid="true"` et `aria-describedby` vers l'`id` de son message, aux quatre écrans de ce lot.*

- **D1 — Prix d'article (#1173)** (C-36-18-8) : `classifyPriceInput` lit la saisie rognée : vide → `empty` ; signe →
  `negative` (avant la lecture, message inchangé) ; puis `readJournalAmount` : `ambiguous` → `ambiguous` (neuf, message
  `common-amount-ambiguous`) ; `unreadable` → `invalid` (`product-error-price-invalid`, inchangé) ; `clear` → `ok`.
  `normalizePriceInput` rend la charge (`amountEntryPayload`, « 1'250 » → `1250`, « 10,50 » → `10.50`). `isValidPrice` et
  `PRICE_RE`, sans appelant de production, sont retirés avec leurs tests ; « 007.50 » et « 01 » deviennent acceptés.
  Rouvert : inchangé, jamais ambigu (AC1). *Écartée* : garder `PRICE_RE` et y ajouter le prédicat — une seconde grammaire.
- **D2 — Frais de rappel (#1175)** : `submitForm` lit `fFee.trim()` : vide → `dunning-form-error-fee` (inchangé) ;
  `ambiguous` / `unreadable` → message commun (`$max` 2) dans `formError` ; `clear` → `feeAmount: payload`. Bordure et
  `aria-invalid` tant que le champ est fautif. Bornes et échelle au serveur.
- **D3 — Montant minimal (#1176)** : vide → `null` (inchangé) ; `ambiguous` / `unreadable` → message sous le champ
  (`data-testid="settings-minimum-amount-ambiguous"` / `-unreadable`, `$max` 2) et l'enregistrement s'arrête par
  `notifyError` du même texte ; `clear` → `minimumInvoiceAmount: payload`.
- **D4 — Filtres (#1177, en dernier)** (C-36-18-7) : `buildQuery` envoie la charge d'une borne claire (requête et URL,
  « 1'250 » → `amountMin=1250`) et omet une borne ambiguë ou illisible ; sous le champ, le message commun (`$max` 4) suivi
  de `journal-entries-filter-amount-not-applied`, `data-testid="journal-entries-filter-amount-min-problem"` /
  `-max-problem`. Une URL `amountMin=1.250` (signet ancien) affiche le message et n'applique pas la borne.
  *Écartées* : ne pas recharger (filtre périmé tu) ; envoyer quand même (le défaut).
- **D5 — Texte** (quatre locales, à la suite de `journal-entries-filter-amount-max`, fr-CH `:408`, autres `:414`) :
  `journal-entries-filter-amount-not-applied` — fr-CH « Ce filtre n'est pas appliqué tant que le montant n'est pas
  corrigé. » ; de-CH « Dieser Filter wird erst angewendet, wenn der Betrag korrigiert ist. » ; it-CH « Questo filtro non è
  applicato finché l'importo non è corretto. » ; en-CH « This filter is not applied until the amount is corrected. »
  Fallback = texte fr-CH.
- **D6 — Ce qui ne change pas** : le serveur et l'API ; les aides et clés de la 36-18a ; les écrans de la 36-18a et des
  36-3, 36-3b, 36-2, 35-6.

## Critères d'acceptation

1. **AC1 — Prix d'article** (`product-helpers.test.ts`, `products-page.test.ts`) : `classifyPriceInput` — « 1.250 »,
   « 1,250 » → `ambiguous` ; « 1'250 », « 10,50 », « 0 », « 007.50 » → `ok` ; « -1 » → `negative` ; « abc », « 1.23456 »,
   « .5 » → `invalid` ; page : « 1.250 » → message ambigu, aucun appel ; « 1'250 » → `unitPrice: '1250'` ; rouvert aux prix
   serveur `1.2340`, `1.2500`, `1500.0000`, `0.1230` → champs `1.2340`, `1.25`, `1500.00`, `0.1230`, **sans message**.
2. **AC2 — Frais de rappel** (test neuf `frontend/src/routes/(app)/settings/dunning/dunning-page.test.ts`) : « 1.250 » →
   message ambigu, `createDunningLevel` non appelé ; « 1'250 » → `feeAmount: '1250'` ; «  12.50  » → `'12.50'` ; « 12'34 »
   → « … 2 décimales au plus » ; un niveau rouvert à `10.00` s'enregistre sans message. **Étape 0** : constater par un
   `curl` (base jetable) ou un test serveur que `PUT /dunning-levels/{id}` refuse `"1.250"` en 400, journal
   `kesh-gate-logs/e36-18b/etape0-dunning.log`.
3. **AC3 — Montant minimal** (`settings-invoicing-page.test.ts`) : « 1.250 » → message sous le champ,
   `updateInvoiceSettings` non appelé ; « 1'250 » → `minimumInvoiceAmount: '1250'` ; vide → `null` (test `:176-184` vert
   sans modification) ; rouvert à `5.00` et `1.2500` → aucun message ; **étape 0** : forme réelle relevée par
   `GET /api/v1/company/invoice-settings` (`etape0-minimum.log`).
4. **AC4 — Filtres** (`journal-entries-list-page.test.ts`) : « 1.250 » en *Montant min* → message et phrase « pas
   appliqué », `fetchJournalEntries` appelé **sans** `amountMin`, URL sans `amountMin` ; « 1'250 » → `amountMin: '1250'` et
   `?amountMin=1250` ; URL `?amountMin=1.250` → message au chargement, borne non envoyée ; « 100 » → `amountMin: '100'`.
5. **AC5 — Rien de juste n'est refusé** : pour chacun des quatre sites, un test par forme — « 1.25 », « 1250 », « 1'250.50 »,
   « 1’250 », « 1,25 », « 0.50 » — sans message, envoyée normalisée ; une saisie bordée de blancs («  1250  ») sans
   message, envoyée `1250` (les quatre sites rognent, D4 de la 36-18a) ; « 1250.0000 » (zéros de fin) accepté à l'article,
   au minimum et aux filtres ; les formes rouvertes d'AC1 à AC3 sans message. Seules deviennent refusées la forme de la
   règle et ce que la grammaire du journal ne lit pas — `1e3`, `.5`, `5.`, signe, blanc intérieur, **plus de quatre
   décimales, même nulles** — : un test le constate aux frais (« 10.01000 », que `Number` lisait aujourd'hui). *(Propagé
   de la validation P1 de la 36-18a, V1-1 et V1-5.)*
6. **AC6 — E2E** (`frontend/tests/e2e/montants-ambigus.spec.ts`, créé par la 36-18a : un `test` de plus par écran) :
   « 1.250 » → message visible ; l'écouteur `page.on('request')`, armé avant le clic, compte les seules requêtes
   d'écriture de l'écran, par **méthode et route** — `POST /api/v1/products` et `PUT /api/v1/products/{id}`
   (`products.api.ts:40`, `:47`), `POST /api/v1/dunning-levels` et `PUT /api/v1/dunning-levels/{id}` (`dunning.api.ts:29`,
   `:41`, `:49`), `PUT /api/v1/company/invoice-settings` (`invoices.api.ts:106`) — et en attend **zéro** ; la preuve est
   la relecture par `authedApiContext` de l'état d'avant. Puis « 1'250 » → le corps porte `1250`, réponse 2xx, relecture
   comparée **en valeur** (`new Big(lu).eq(1250)`), jamais en chaîne (`1250.0000` article et minimum, `1250.00` frais).
   Filtres : la requête `GET /api/v1/journal-entries` (`journal-entries.api.ts:28`) porte `amountMin=1250` pour « 1'250 »,
   aucune borne pour « 1.250 ». *(Propagé de V1-7 de la 36-18a.)*
7. **AC7 — Textes** : une clé × quatre locales (D5) ; `lint-i18n-ownership` vert sans entrée neuve ; `sitesTotal` de
   `i18n-keys.test.ts` **recompté à la source** après la fusion de la 36-18a, sans addition.
8. **AC8 — Manuels** (`.tex` ; PDF à l'intégration) : une phrase par écran, qui renvoie à la règle de *Saisie d'une écriture
   en partie double* (`user-manual.tex:559`), porte la chaîne exacte `refusé comme ambigu`, l'exemple `1.250`, la forme
   `1'250` et la limite de décimales — `user-manual.tex` item *Prix unitaire HT* (`:1126`), § *Le montant minimum*
   (`:1197`), § *Recherche d'écritures*, item *Montant* (`:881`, la borne non appliquée) ; `admin-manual.tex` item *Frais*
   (`:1338`) et § *Montant minimum* (`:3856`). **Contrôle de clôture de la story, sur le `.tex`** :
   `/usr/bin/grep -n 'refusé comme ambigu' docs/manual/fr/user-manual.tex docs/manual/fr/admin-manual.tex` rend une ligne
   dans chacune des cinq sections citées. **Contrôle d'intégration seulement** (PDF régénéré après l'union) :
   `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -o 'refusé comme ambigu' | wc -l` pour chacun des deux
   manuels, comparé à sa valeur à l'étape 0 — `grep -c` sur un flux aplati ne rend que 0 ou 1. *(Propagé de V1-2 de la
   36-18a.)*
9. **AC9 — Rien d'autre ne change** : `git diff <base> --stat` ne nomme ni `crates/` hors `kesh-i18n/locales/`, ni
   `balance.ts`, `unit-price.ts`, `supplier-invoice-form.ts`, `SettleInvoiceDialog.svelte`, `opening-balances`, ni les écrans
   des 36-3, 36-3b, 36-2 ; leurs tests verts sans modification.
10. **AC10 — Captures** (thème clair, base jetable `kesh_e2e_e3618b`, `kesh-gate-logs/e36-18b/captures/`) : chaque écran à
    « 1.250 », à « 12'34 », et la valeur « 1'250 » enregistrée ; un article rouvert à `1.2340` ; le filtre ambigu et sa phrase.

## Tests à écrire, et la preuve du rouge

Écrits d'abord, sur le code non corrigé (36-18a fusionnée), journal `/home/gcorbaz/devel/kesh-gate-logs/e36-18b/rouges.log`.

- **Rouges attendus** : AC1 (`classifyPriceInput('1.250') === 'ok'` ; « 1'250 » `invalid`) ; AC2 (`feeAmount: '1.250'` part ;
  « 1'250 » refusé) ; AC3 (`minimumInvoiceAmount: '1.250'`) ; AC4 (`amountMin: '1.250'`) ; AC6 (chaque requête part avec
  `1.250`).
- **Verts au départ, et dits** : le prix d'article rouvert (jamais ambigu par construction) ; les montants rouverts aux
  frais et au minimum ; « 1.25 » et « 1250 » partout. Les mutations M2, M3 et M5 vérifient qu'ils rougiraient.
- **E2E rouge** : sources remises à la base par `git restore --source=<base> --worktree -- <fichiers>` après commit,
  frontend construit, spec rouge, sources restaurées (`git restore --worktree` + `touch`), journal `e2e-rouges.log`.

**Mutations** (code commité d'abord ; scripts dans `/home/gcorbaz/devel/kesh-gate-logs/e36-18b/mut/` ; restauration
`git restore --worktree -- <fichier>` puis `touch` ; journaux `mut/M*.log`, résumé `mut/resume.log` ; Vitest seul) :

| # | Mutation | Tuée par |
|---|---|---|
| M1 | article : `classifyPriceInput` sans le cas `ambiguous` | AC1 |
| M2 | article : `normalizePriceInput` reprend `replace(',', '.')` | AC1 (`1250`), AC5 |
| M3 | article : `openEdit` passe par `readableAmount(…, 2)` | AC1 (rouvert `1.234` → message) |
| M4 | frais : `feeAmount: fFee` brut | AC2 |
| M5 | frais : la lecture sans `trim()` | AC2 (« 12.50 » rogné) |
| M6 | minimum : `minimumAmount.trim()` envoyé brut | AC3 |
| M7 | filtres : `buildQuery` envoie la borne brute | AC4 |
| M8 | filtres : la phrase « pas appliqué » retirée | AC4 |
| M9 | minimum : la lecture sans `trim()` | AC5 (« 1250 » bordé de blancs) |

Une mutation équivalente se dit au Dev Agent Record.

## Tâches

- [ ] **Étape 0** : partir de `epic/36-0.18.0` **avec la 36-18a fusionnée** (et donc la 36-15) ; relire les lignes citées ;
      rejouer les motifs d'inventaire (`kesh-gate-logs/e36-18b/etape0-inventaires.log`) ; AC2 (`curl` des frais) et AC3
      (forme du minimum) ; valeurs de départ du contrôle PDF (AC8) ; relire les LOW du dernier rapport de validation.
- [ ] Tests d'abord (AC1 à AC5) ; `product-helpers.ts` et la page (D1) ; `settings/dunning` (D2) ; `settings/invoicing` (D3) ;
      `journal-entries/+page.svelte` (D4, en dernier) ; quatre `messages.ftl` (D5) ; `i18n-keys.test.ts` recompté (AC7).
- [ ] E2E (AC6), rouge d'abord ; manuels (AC8), `.tex` seuls, leur contrôle `.tex` à la clôture ; captures (AC10) ;
      mutations M1 à M9.
- [ ] Gates (voie 2, `e35-dev-commun.md` § *Quels gates, quand*) : en cours, Vitest ciblé par `.gate-front.lock`,
      `npm run check`, `lint-i18n-ownership` ; **passage en `review`** : bases `kesh_e3618b` remises à neuf (voie 2),
      `scripts/test-fast.sh --ci`, `scripts/gate-ci.sh kesh_ci_e3618b` (voie 1, son verrou), frontend complet, pas d'E2E ;
      **remédiations** : `test-fast.sh --ci` et frontend complet ; **clôture** : `gate-ci` et E2E complet sur
      `kesh_e2e_e3618b` recréée, au dernier commit de code, échecs nommés contre `docs/testing.md` § « Les échecs attendus ».
      Une commande lourde à la fois ; journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e36-18b/`.
- [ ] Fin du développement : `Status: review`, clé `review` au `sprint-status.yaml`, prompt de revue P1
      (`36-18b-montants-ambigus-reglages-filtres-review-prompt-p1.md`) qui nomme les deux manuels, les PDF et les captures.

## Note de version

*Rubrique* **Corrigé** (C-e360-11) :

- Le prix d'un article, les frais d'un niveau de rappel, le montant minimal d'une facture et les bornes du filtre des
  écritures lisent désormais un montant comme une écriture : « 1.250 » n'est plus lu 1,25 franc en silence ; il est refusé
  comme ambigu (le filtre ne s'applique pas, et l'écran le dit), et le message dit comment l'écrire (`1'250` ou `1.25`).
  L'apostrophe des milliers et la virgule décimale sont acceptées ; la saisie à exposant (`1e3`), sans chiffre avant ou
  après le point (`.5`, `5.`) ou à plus de quatre décimales, même nulles, n'est plus lue. Les frais de rappel ne disent
  plus seulement « deux décimales au plus » d'un montant ambigu. **Avant la 0.18.0**, un prix d'article ou un montant
  minimal tapé avec trois chiffres après le point ou la virgule (« 1.250 ») a été lu sans avertissement comme un franc
  vingt-cinq : relisez vos articles et votre montant minimal saisis ainsi, et les factures qui ont repris un tel prix.
  (refs #1173, refs #1175, refs #1176, refs #1177)

## Dev Notes

### Hors périmètre

- Soldes de départ, facture fournisseur, règlement : **36-18a**.
- Le serveur et l'API (C-e350-19) ; les taux de TVA ; les quantités.
- La convergence des messages des 36-3, 36-3b et 36-2 vers les clés communes : **#1202**.

### Splitting — quatre modules

(1) articles (`features/products`, `routes/(app)/products`) ; (2) réglages (`routes/(app)/settings/dunning`,
`settings/invoicing`) ; (3) `features/journal-entries` (la page de liste, même module métier) ; (4)
`crates/kesh-i18n/locales`. S'y ajoutent les manuels, `montants-ambigus.spec.ts`, `i18n-keys.test.ts` et le test neuf
`dunning-page.test.ts`. **Quatre : la règle ne s'applique pas.**

### Fichiers communs avec les autres stories de la phase 2

- `settings/invoicing/+page.svelte`, `journal-entries/+page.svelte` : la fiche 36-17 les nomme ; sa branche ne les modifie
  pas à `fed34a9c7` (`git diff --name-only origin/epic/36-0.18.0...story/36-17-assistant-fin-configuration`) ; à
  recontrôler à l'étape 0.
- `messages.ftl` ×4, manuels, `i18n-keys.test.ts` : comme la 36-18a.
- `montants-ambigus.spec.ts` : créé par la 36-18a.
- `CHANGELOG.md` : non écrit par la branche.

### Ce qui doit rester vrai

- Un montant juste ne devient jamais refusé (AC5) ; un montant rouvert jamais (AC1 à AC3).
- Le serveur reste juge ; big.js partout ; fixtures synthétiques seulement.

### E2E

Chaque montant part tel que tapé, et le filtre traverse la frontière HTTP : AC6 exigé.

### Références

- Issues #1173, #1175 (et son commentaire de l'orchestrateur), #1176, #1177, #1202 ; fiches `36-18a-montants-ambigus-livres.md`
  et `36-18-montants-ambigus-autres-ecrans.md` (`split`) ; Stories 36-3, 36-3b ; C-36-18-1 à C-36-18-10.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — Fiche créée (Opus 5.5) par découpage de la 36-18 (C-36-18-10), à `fed34a9c7` : quatre sites, décisions D1 à
  D6 (les communes renvoyées à la 36-18a), AC1 à AC10, mutations M1 à M8 ; quatre modules ; aucune migration ; `backlog`
  jusqu'à la fusion de la 36-18a. Validation non lancée.
- 2026-11-15 — Propagation de la validation P1 de la 36-18a (`kesh-gate-logs/e36-18/validation-36-18a-p1.md`) : `aria-invalid`
  et `aria-describedby` (V1-6) ; AC5 : blancs, zéros de fin, plus de quatre décimales (V1-1, V1-5), M9 ; AC6 : requêtes
  comptées par méthode et route, relecture en valeur (V1-7, routes lues `products.api.ts:40`, `:47`, `dunning.api.ts:29`,
  `invoices.api.ts:106`, `journal-entries.api.ts:28`) ; AC8 : contrôle `.tex` à la clôture, PDF à l'intégration par
  `grep -o | wc -l` (V1-2) ; note de version : relire les montants saisis avant la 0.18.0 (V1-10). V1-3, V1-4, V1-8 et
  V1-9 ne visent que la 36-18a.
