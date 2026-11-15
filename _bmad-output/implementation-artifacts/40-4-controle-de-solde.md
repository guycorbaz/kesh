# Story 40-4 : contrôle de solde Stripe — versements en route, `available` et `pending`, créance hors trésorerie

Status: ready-for-dev

- **Epic** : 40 (`_bmad-output/planning-artifacts/epic-40-stripe.md`, § *40-4*, G1, G5, G10, G12, C-40-0, C-40-2 (c),
  C-40-6) ; décisions de Beat **B13** (créance « Débiteurs Stripe ») et **B20** (le relevé est la somme de contrôle),
  `epics-38-40-cadrage.md`, qui priment ; décision de l'orchestrateur **P40-1** (clés `rk_test_` admises, mode montré).
- **Dépend de** : **40-3** (versements : `movement_kind = 'withdrawal'`, écriture datée de `value_date`, rattachement à
  la ligne UBS), **40-1** (`StripeConnector::balances`, « Tester »), **38-9a** (route et module du contrôle (b) :
  `GET /api/v1/bank-accounts/{id}/provider-balance-check`, `kesh-db::repositories::provider_balance_check`, AC-8),
  **38-10** (contrôle K9 `ATTACHED_STATEMENT_GAP`), **38-6c2** (fragment `LINKED_TRANSACTIONS_OF(e)`), **38-3b**
  (`ProviderBalance`, instantanés `reported`), **38-2** (bloc `provider` de `GET /bank-accounts`, AC-8), **37-4**
  (`account_balances::balances_as_of` et fragment de somme en devise, AC-1 quater).
- **Suivie de** : **40-6** (écrans : ventilation, `available`/`pending` à « Tester », détail de K9 ; manuels).
- **Migration** : **aucune** (C-40-0, NFR-40-4-2).
- **Modules** (≤ 5, règle de splitting) : `kesh-providers` (champ additif de `ProviderBalance`), `kesh-db` (décalages
  de date, contrôle (b), K9), `kesh-api` (réponses de « Tester », du contrôle (b), bloc `provider`), `kesh-i18n`,
  `frontend` (tuile de l'accueil seule).

## Story

En tant que **Comptable** de la société de Beat,
je veux que le contrôle de solde d'un compte Stripe compare le solde que rapporte Stripe (`available + pending`) au
compte « Débiteurs Stripe » de la devise, et que l'écart se ventile en transactions non rapprochées, **versements en
route** et reste inexpliqué — et que la créance sur Stripe ne soit pas additionnée à mes liquidités sans le dire,
afin qu'un versement parti de Stripe et pas encore arrivé en banque ne passe jamais pour une erreur, et qu'à la clôture
je lise le solde de 1101–1103 pour ce qu'il est : une créance (B13).

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-40-4-1` | L'écart entre solde comptable et solde rapporté se ventile en non rapprochées, versements en route et reste inexpliqué. | AC-1, AC-2, AC-3 |
| `FR-40-4-2` | « Tester » montre `available` et `pending` ; l'écran du compte montre le solde rapporté (voir *Écarts à l'epic*, 2). | AC-5 |
| `FR-40-4-3` | Un compte prestataire Stripe n'est pas compté comme liquidité là où Kesh additionne la trésorerie (ou l'est en le disant), selon le recensement. | AC-6, AC-7 |
| `FR-40-4-4` *(ajoutée)* | Le contrôle préalable à la clôture K9 dit, pour un compte en écart, la part des versements en route à la date du relevé joint ; son statut est inchangé. | AC-4 |
| `NFR-40-4-1` | Une seule définition de « versement en route », testée aux bornes (`as_of` = `booked_at`, date d'écriture = date de `as_of`). | AC-1, AC-3 |
| `NFR-40-4-2` | Contrôle sans écriture ; aucune migration. | AC-8 |
| `NFR-40-4-3` *(ajoutée)* | Additif, non régressif (G18) : PayPal et les comptes ordinaires gardent leurs réponses JSON à l'identique quand rien n'est en route ; quatre locales ; captures de la tuile. | AC-2, AC-4, AC-5, AC-7, AC-9 |

## Ce que le code dit (prémisses vérifiées le 2026-11-15 sur `a2b1da9c`)

- **Une transaction rapprochée pointe son écriture** : `bank_transactions.status` ∈ `pending`, `reconciled`
  (`crates/kesh-db/migrations/20260504000001_bank_imports.sql:70`, `:84`), `matched_entry_id` FK `journal_entries`
  `ON DELETE SET NULL` (`:71`, `:83`) ; `value_date DATE NULL` existe déjà (`:61`) ; l'écriture porte `entry_date`
  (`20260412000001_journal_entries.sql:21`).
- **La correction des dates de valeur existe déjà, en CHF, pour l'écart du relevé** : `BALANCES_SQL`
  (`crates/kesh-db/src/repositories/bank_accounts.rs:1867-1928`) somme, par compte bancaire, la ligne **de l'écriture
  sur le compte du grand livre lié** (non `amount`) des transactions `reconciled` dont la date bancaire et la date
  d'écriture tombent de part et d'autre de `period_to` : `booked_before_entered_after` (`booking_date <= period_to`,
  `entry_date > period_to`, `:1878-1888`) et `booked_after_entered_before` (`:1890-1900`) ; `balances_from_row` en
  tire `ledger_at + plus − minus` (`:1972-2002`) ; doc de la règle et de son invariant « une transaction, une
  écriture » `:1796-1826`. Le contrôle K3 la lit (`closing_checks.rs:714`), et le manuel l'explique
  (`docs/manual/fr/user-manual.tex:337-356`, « L'écart tient compte des dates de valeur »). **C'est exactement le
  versement en route** : un `payout` sorti de Stripe le jour J (`booking_date`), dont l'écriture `D 1020 / C 1101` est
  datée de l'arrivée (40-3, G10).
- **Le contrôle (b) de 38-9a ne corrige pas les dates** (38-9a AC-8 b) : `ledger` = `balances_as_of` à la date de
  Zurich de `asOf` (37-4, en devise), `unreconciled` = Σ `amount` des `pending` à `booked_at ≤ asOf`, `unexplained =
  reported − ledger − unreconciled`. Un versement rapproché dont l'écriture est datée après `asOf` y tombe donc en
  « inexpliqué ». Le contrôle K9 de 38-10 (C-38-10-4) compare de même le solde saisi à `balances_as_of` à `period_to`,
  sans correction.
- **Le solde d'un instantané `reported` s'arrête avant son instant** : la complétude de 38-3b somme les mouvements
  `booked_at < as_of` du `reported` (38-3b AC-7, fenêtre demi-ouverte) ; Stripe : `balances(Some(as_of))` = solde
  courant − Σ `net` des transactions `created ≥ as_of` (G5). 38-9a AC-8 b compte en revanche les `pending` à
  **`booked_at ≤ asOf`** (voir *Questions*, Q-40-4-2).
- **Graphiques de l'accueil** : revenus et charges seulement (`a.account_type IN ('Revenue', 'Expense')`,
  `crates/kesh-report/src/homepage_summary.rs:252-292`, `:295-334`) — aucun compte bancaire lu.
- **Tuile « Comptes bancaires » de l'accueil** : `ledgerTotal` additionne `currentBalance` de **tous** les comptes
  bancaires liés, un compte du grand livre une fois (`frontend/src/lib/features/homepage/homepage.ts:33-50`) ; total
  affiché « Total (solde comptable) » (`BankAccountsCard.svelte:18`, `:91-112` ; clé
  `homepage-bank-total-ledger`, `crates/kesh-i18n/locales/fr-CH/messages.ftl:1688`). `currentBalance` est en devise
  principale (38-2 *Dev Notes* : « reste calculé comme aujourd'hui »). Un compte Stripe lié à 1101 y serait compté
  comme une liquidité.
- **Rendement par projet** : « coût investi » = `Expense` + `Asset` dont le numéro ne commence pas par `10`
  (`crates/kesh-report/src/project_report.rs:392`, `:458`) — règle par numéro de compte, indépendante du lien
  bancaire.
- **Règlement d'une facture client** : le dialogue propose les comptes bancaires hors cartes et hors compte débiteurs
  (`frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte:113-114`) — une facture payée par Stripe peut s'y
  régler sur le compte Stripe (`D 1101 / C 1100`), ce qui est juste.
- **Aucun outil MCP ne lit les soldes bancaires** (`crates/kesh-api/src/mcp/outils/` : aucun module de comptes
  bancaires).
- **Contrôles de clôture** : toute requête de `closing_checks` est une constante de `ALL_SQL` (`closing_checks.rs:550`),
  les fonctions externes sont l'angle mort assumé de T-a2, listées au doc du module (`:51-60`).

## Écarts à l'epic

1. **La règle du versement en route n'est pas « `booked_at < as_of ≤ value_date` »** (G12) mais la **date de
   l'écriture rapprochée** : un versement compte en route quand il est sorti du solde Stripe à `asOf` et que
   l'écriture à laquelle il est rattaché est datée **après** la date de Zurich de `asOf` ; le montant est la ligne de
   cette écriture sur le compte lié (règle de `BALANCES_SQL`, prémisse). *Raison* : le solde comptable comparé se lit
   par `entry_date`, non par `value_date`. En voie (b) de 40-3 les deux coïncident (écriture datée de `value_date`) ;
   en voie (a), l'écriture existante peut être datée autrement (livres 2024, Q-40-4) — la règle de G12 compterait alors
   en route un versement déjà au grand livre. Corollaires : un versement **non rapproché** est compté dans
   `unreconciled`, jamais en route (aucun double compte, risque de l'epic) ; un versement **sans `value_date`** se
   juge à son écriture comme les autres ; à date d'écriture égale à la date de `as_of`, il n'est pas en route (borne
   inclusive de `balances_as_of`).
2. **« L'écran du compte montre `available` et `pending` »** (G12, FR-40-4-2) : impossible sans les stocker — l'instantané
   ne garde que le total (`provider_balance_snapshots.kind` est fermé par un `CHECK BINARY IN ('opening','reported')`,
   38-3a ; une valeur neuve serait une migration, C-40-0). Ils sont rendus par « Tester » (AC-5), lus à l'instant ;
   l'écran du compte montre le solde rapporté total de l'instantané (38-9a AC-8 b).
3. **Décalages de date autres que les versements** : la même règle, appliquée à toutes les transactions rapprochées
   (comme `BALANCES_SQL`), donne une seconde ligne de ventilation, `otherDateShifts` (un rattachement en lot des
   livres 2024, 38-6c1, n'a pas de fenêtre de date). Sans elle, ces décalages tomberaient en « inexpliqué ».
4. **K9 gagne un détail** (FR-40-4-4) alors que l'epic le dit « inchangé » : son **statut** l'est, son `details` gagne
   deux champs facultatifs. *Raison* : le rapport Stripe de fin d'année (40-5) arrête le solde Stripe au 31.12, versements
   partis compris ; 1101 les porte jusqu'à leur arrivée : K9 serait en écart à **chaque** clôture qui a un versement en
   route, sans dire pourquoi.
5. **Modules** : `kesh-providers` (champ de `ProviderBalance`) et `frontend` (tuile) s'ajoutent à la liste de l'epic ;
   cinq au total, dans la règle.

## Choix

- **C-40-4-1 — Une fonction, deux appelants.** `kesh_db::repositories::provider_balance_check::date_shifts(conn,
  company_id, bank_account_id, ledger_account_id, cut: Cut) -> DateShifts { in_transit_payouts, other }` (module de
  38-9a AC-8 b), où `Cut::AtInstant(as_of)` (contrôle (b)) ou `Cut::AtDate(d)` (K9). Pour une coupe, avec `D` = la date
  de la coupe (date de Zurich de `as_of`, ou `d`) :
  - *comptée au solde rapporté* : `AtInstant` → le prédicat de `unreconciled` de 38-9a AC-8 b, **le même code** (une
    aide partagée : `booked_at ≤ as_of`, ou `booking_date ≤ D` si `booked_at` est nul) ; `AtDate` → `booking_date ≤ d` ;
  - `plus` = Σ des lignes, **sur le compte du plan lié**, des écritures `e` rapprochées
    (`LINKED_TRANSACTIONS_OF(e)` de 38-6c2, `status = 'reconciled'`) à une transaction du compte **comptée**, et
    `e.entry_date > D` ; `minus` = de même pour une transaction **non comptée** et `e.entry_date ≤ D` ;
  - montant d'une ligne : en devise pour un compte du plan en devise, par le **fragment de somme en devise de 37-4**
    (débit − crédit, jamais une seconde somme `amount_currency`), en CHF sinon (`debit − credit`, convention de
    `BALANCES_SQL`) ;
  - `in_transit_payouts` = part de `plus` dont la transaction porte `movement_kind = 'withdrawal'` ; `other` =
    `plus − minus − in_transit_payouts`.
  Sans verrou, sur l'exécuteur reçu ; **une** requête par appel (sous-requêtes séparées, jamais
  `bank_transactions` joint aux lignes dans un même `FROM` : règle de `BALANCES_SQL`, `bank_accounts.rs:1830-1834`).
- **C-40-4-2 — Pourquoi pas `BALANCES_SQL` lui-même** : il est en CHF, par compte et pour le relevé importé ; le
  contrôle (b) est en devise et à un instant. La même règle s'écrit donc une seconde fois **en devise** — et un test
  de concordance la tient : pour un compte CHF, `date_shifts(…, AtDate(d))` rend exactement
  `booked_before_entered_after − booked_after_entered_before` de `BALANCES_SQL` borné à `d` (AC-3).
- **C-40-4-3 — Créance hors total** (Q-40-10, défaut proposé en attendant Guy) : la nature du solde d'un compte
  prestataire se dit au bloc `provider` de `GET /bank-accounts` par un champ `balanceNature` ∈ `liquidity`,
  `receivable`, posé par **une** fonction de `kesh-api` sur le prestataire (`stripe` → `receivable` ; `paypal`, `wise`
  et tout autre → `liquidity`). La tuile de l'accueil **n'additionne pas** une créance à son total et la montre à part.
  Si Guy tranche autrement, seule la fonction pure de l'accueil change (AC-7).

## Critères d'acceptation

1. **AC-1 — Versements en route : la fonction (FR-40-4-1, NFR-40-4-1).** `date_shifts` (C-40-4-1) dans
   `kesh-db/src/repositories/provider_balance_check.rs`. Doc-comment : la règle, le renvoi à `BALANCES_SQL`, l'invariant
   « une transaction, une écriture » (et les groupes de 38-6c2 : toute écriture du groupe compte, par
   `LINKED_TRANSACTIONS_OF`), la limite « une transaction `reconciled` à `matched_entry_id` nul (écriture supprimée) ne
   compte nulle part ». Tests d'intégration (`#[sqlx::test]` sur le squash), compte prestataire USD lié à un compte du
   plan USD (1103), versement de 300.00 : (a) sorti à `booked_at` = `as_of` exact, rapproché d'une écriture datée D + 1
   → `in_transit_payouts = −300.00` ; (b) même versement, `booked_at` = `as_of` + 1 ms → rien ; (c) écriture datée D
   → rien (au grand livre à D) ; (d) écriture datée D + 1, transaction `pending` → rien (elle est à `unreconciled`) ;
   (e) rattaché en voie (a) à une écriture datée D − 2 (avant la sortie) avec `booking_date` = D → rien ; (f) vente
   rapprochée d'une écriture datée D + 1 → `other = +115.80`, `in_transit_payouts` nul ; (g) transaction comptée après
   la coupe, écriture datée ≤ D → `other` négatif ; (h) membre de groupe (38-6c2) daté D + 1 → compté ; (i) compte CHF,
   montants en CHF ; (j) `booked_at` nul (transaction de fichier, 38-2 AC-3) → `booking_date` ; (k) autre société →
   rien (KF-002). Mutation déclarée au Change Log : prédicat `>` changé en `>=` sur `entry_date` → (c) rougit.
2. **AC-2 — Contrôle (b) ventilé (FR-40-4-1, NFR-40-4-3 ; C-40-2 c).** `GET /api/v1/bank-accounts/{id}/provider-balance-check`
   (38-9a AC-8 b ; droits, refus et 404 inchangés) rend en plus `inTransitPayouts` et `otherDateShifts` (chaînes
   décimales signées, devise de la réponse), **absents quand nuls** (`skip_serializing_if`, patron C-37-9 : la réponse
   d'un compte PayPal sans décalage reste identique, G18) ; `unexplained = reported − ledger − unreconciled −
   inTransitPayouts − otherDateShifts` (termes absents = 0), calculé par la **même** fonction que l'écran ne recalcule
   jamais. Appel de `date_shifts(…, Cut::AtInstant(asOf))` avec l'`asOf` de l'instantané. Doc-comment de la réponse :
   le sens de chaque terme (« Selon Stripe = livres + non rapproché + en route + autres décalages + inexpliqué »).
   Tests de route : compte USD, ventes rapprochées, versement rapproché daté de l'arrivée postérieure à `asOf` →
   `inTransitPayouts: "-300.00"`, `unexplained` absent de l'écart (`"0.00"`) ; même cas sans la ventilation (mutation)
   → `unexplained: "-300.00"` ; compte PayPal sans décalage → ensemble exact des clés de 38-9a ; Consultation → 403,
   clé d'API → 403 (inchangé).
3. **AC-3 — Concordance avec `BALANCES_SQL` (NFR-40-4-1).** Test : compte bancaire CHF ordinaire, relevé CAMT au 31.12,
   deux transactions rapprochées de part et d'autre (écriture postérieure, écriture antérieure) → `date_shifts(…,
   AtDate(31.12))` égale `booked_before_entered_after − booked_after_entered_before` de `balances_by_bank_account(…,
   Some(31.12))` ; écrit comme garde de la seconde écriture de la règle (C-40-4-2).
4. **AC-4 — K9 : part des versements en route (FR-40-4-4, NFR-40-4-3).** Dans `closing_checks` (38-10 AC-6), chaque
   compte de `details.accounts[]` en écart gagne `inTransitPayouts` et `otherDateShifts` (même sens, même absence
   quand nuls), lus par `date_shifts(…, Cut::AtDate(periodTo))` du document retenu, sur la connexion de l'instantané ;
   **statut, `count`, `gap` et règle de K9 inchangés** (Q-40-4-1). `date_shifts` entre à la liste de l'angle mort de
   T-a2 au doc du module (`closing_checks.rs:51-60`), son SQL relu sans verrou en revue ; aucune constante neuve à
   `ALL_SQL` (la requête vit dans `provider_balance_check`). Tests (`crates/kesh-db/tests/closing_checks.rs`) : rapport
   joint au 31.12 dont le solde est le solde Stripe, versement parti le 30.12 arrivé le 3.1 → K9 `attention`, `gap`
   = 300.00 (livres − relevé), `inTransitPayouts = "-300.00"` ; sans versement → champs absents ; compte ordinaire →
   absents. Type TS `fiscal-years.types.ts` gagne les deux champs facultatifs (affichage : 40-6).
5. **AC-5 — `available` et `pending` à « Tester » (FR-40-4-2 ; Écart 2).** `kesh_providers::ProviderBalance` (38-3b
   AC-2) gagne `parts: Option<BalanceParts { available: Decimal, pending: Decimal }>` (additif ; `None` pour PayPal, le
   factice et tout solde à une date passée) ; `StripeConnector::balances(None)` (40-1) le pose par devise, avec le
   contrôle `available + pending == amount` (sinon `InvalidResponse { reason: "balance_parts" }`). La route `POST
   …/provider-connections/{id}/test` (38-7 AC-8 : `balances: [{ currency, total }]`) rend en plus `available` et
   `pending` (chaînes décimales) **quand présents**, absents sinon (réponse PayPal inchangée, G18). Le passage (38-3b)
   n'en écrit rien. Si 40-1 a déjà posé ce champ, rien n'est à refaire : le dire au Change Log. Tests : fixture
   `balance.json` anonymisée (40-1, G16) → USD `available 369.89`, `pending 28.72`, total `398.61` ; parties
   incohérentes → `InvalidResponse` ; PayPal → clés exactes de 38-7.
6. **AC-6 — Recensement des lecteurs « liquidité » (FR-40-4-3 ; G1).** Tableau tenu au doc de la fonction de C-40-4-3 et
   recopié au Change Log, relevé par `grep -rn "currentBalance\|current_balance\|ledgerTotal\|bank_accounts::list"
   crates/*/src frontend/src` et `grep -rnE "trésorerie|liquidit|LIKE '10" crates/*/src frontend/src` (relancés au
   développement, résultat cité) :
   | Lecteur | Décision |
   |---|---|
   | Tuile « Comptes bancaires » de l'accueil (`homepage.ts:33-50`, `BankAccountsCard.svelte`) | **changé** : créance hors total, dite (AC-7) |
   | Graphiques de l'accueil (`homepage_summary.rs:252-334`) | non concerné : produits et charges seuls |
   | Rendement par projet (`project_report.rs:458`) | non concerné : règle par numéro (1101 compté « investi » comme 1100) ; constat noté au Change Log pour l'issue de C-40-7 si la revue le juge défaut |
   | Liste des comptes bancaires (`bank-accounts/+page.svelte:407`, `:981`) | non concerné : soldes par compte, aucun total |
   | Règlement d'une facture (`SettleInvoiceDialog.svelte:113-114`) | admis : une créance Stripe éteint une créance client |
   | Lot de paiement pain.001 | déjà exclu (`PAYMENT_BATCH_ACCOUNT_IS_PROVIDER`, 38-2 AC-7) |
   | K3 `BANK_STATEMENT_AT_YEAR_END` | inchangé : un compte Stripe sans CAMT y est `NO_BALANCE_STATEMENT` ; K9 le contrôle |
   | Bilan, balance, grand livre | non concernés : classe du plan (1101 = actif circulant, créance) |
   | Outils MCP | aucun ne lit les soldes bancaires |
   Tout lecteur trouvé au grep et absent de ce tableau y entre avec sa décision avant la revue.
7. **AC-7 — La créance à part sur l'accueil (FR-40-4-3, NFR-40-4-3 ; C-40-4-3, Q-40-10).** Bloc `provider` de `GET
   /api/v1/bank-accounts` (38-2 AC-8) : `balanceNature` (`liquidity` | `receivable`) par la fonction unique de
   C-40-4-3 ; type TS `BankAccountSummary` à jour. `ledgerTotal` (fonction pure, `homepage.ts`) n'ajoute plus au total un
   compte dont `provider?.balanceNature === 'receivable'` et rend en plus `receivables: Big` et `anyReceivable: boolean`
   (une valeur inconnue de `balanceNature` → comptée comme aujourd'hui, liquidité, et non masquée). La tuile garde la
   ligne du compte et son solde, y ajoute « Créance sur Stripe — hors total » sous le nom (`data-testid`
   `homepage-bank-receivable-{id}`), et sous le total « Hors total : créances sur prestataires, X » quand
   `anyReceivable` (`homepage-bank-receivables`). Clés `homepage-bank-receivable-note`,
   `homepage-bank-total-excludes-receivables` aux **quatre** locales (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`),
   écrites en toutes lettres (`i18n-keys.test.ts`). Vitest (`homepage.test.ts`, `HomepageCards.test.ts`) : compte
   Stripe exclu du total et montré à part ; compte PayPal compté ; compte ordinaire inchangé ; `balanceNature` inconnu
   compté ; compte du grand livre partagé inchangé. Test Rust du bloc : `stripe` → `receivable`, `paypal` →
   `liquidity`, compte ordinaire → bloc absent (JSON inchangé).
8. **AC-8 — Ce que la story n'écrit pas (NFR-40-4-2).** Aucune écriture comptable (contrôle, et clôture : le solde de
   1101–1103 au 31.12 **est** la créance, `fiscal_years::close` inchangé — test de caractérisation existant
   `closing_a_year_writes_no_entry_and_changes_nothing` non touché) ; aucune migration, aucun relèvement de
   `kesh_version_min_required` ; aucune route neuve (`LIB_ROUTES` et `TEST_ENDPOINT_ROUTES` : delta 0, C-40-6) ;
   aucune action d'audit (lectures) ; **aucun outil MCP** (G17, E23 : contrôles et connexions ne sont pas exposés).
   `docs/api-external.md` : si 38-9a et 38-10 y documentent le contrôle (b) et K9, les deux champs y sont ajoutés avec
   leur sens ; manuels à 40-6.
9. **AC-9 — Recette visuelle (NFR-40-4-3).** Captures (thème clair, base de démonstration, `kesh-gate-logs/`, hors
   dépôt) de la tuile de l'accueil avec un compte Stripe en créance et un compte PayPal, et sans compte Stripe ; jointes
   au Dev Agent Record et au prompt de revue. E2E : aucun neuf ici (la tuile est couverte par Vitest ; le parcours
   Stripe est à 40-6) ; la suite E2E complète tourne au dernier commit de code (`homepage*.spec.ts` lisent la tuile).

## Tâches

- [ ] T1 — `date_shifts` et `Cut`, aide partagée « comptée au solde rapporté » avec `unreconciled` (AC-1).
- [ ] T2 — Contrôle (b) : deux champs, formule de `unexplained`, tests de route (AC-2) ; concordance (AC-3).
- [ ] T3 — K9 : détail additif, angle mort de T-a2, tests (AC-4).
- [ ] T4 — `ProviderBalance.parts`, `StripeConnector`, réponse de « Tester » (AC-5).
- [ ] T5 — Recensement (AC-6) ; `balanceNature`, `ledgerTotal`, tuile, i18n, Vitest, captures (AC-7, AC-9).
- [ ] T6 — Gate complet **même en cours de boucle** (dépôts `kesh-db` touchés) ; E2E complet au dernier commit de
      code ; `api-external.md` (AC-8).

## Dev Notes

- **Signes** : `unreconciled` est Σ `amount` (un versement est négatif) ; `inTransitPayouts` est la ligne de
  l'écriture sur 1101 (`C 1101` → négatif) — même signe que le mouvement du solde Stripe. Identité tenue par les tests :
  `reported = ledger + unreconciled + inTransitPayouts + otherDateShifts + unexplained`. K9 garde son sens propre
  (`gap` = livres − relevé) : un versement en route y donne `gap = +300.00` et `inTransitPayouts = −300.00` ; 40-6
  écrit « dont versements en route » sans changer de signe à l'écran (le manuel dit le sens).
- **Piège** : ne pas lire `value_date` pour dire « en route » (Écart 1) ; ne pas compter `amount` mais la ligne de
  l'écriture (une écriture posée sur un ancien compte lié sort d'elle-même, comme dans `BALANCES_SQL`).
- **Piège** : `pending` exclu de la ventilation des décalages (sinon double compte avec `unreconciled`).
- **Le passage** de 38-3b lit `balances(Some(fin de fenêtre))` : `parts` y est `None`, rien n'est stocké.
- **Références** : epic G1, G5, G10, G12, C-40-2 (c), Q-40-10 ; 38-9a AC-8 (b), AC-9 ; 38-10 C-38-10-4, C-38-10-5,
  AC-6 ; 38-6c2 (`LINKED_TRANSACTIONS_OF`) ; 38-3b AC-2, AC-7 ; 38-7 AC-8 ; 37-4 AC-1 quater ; 25-6-a (tuile) ; 29-1a
  (K3, `BALANCES_SQL`).

## Questions ouvertes

- **Q-40-4-1 (Beat, Guy)** — K9 doit-il **corriger** le solde comptable des décalages de date, comme K3 le fait pour les
  relevés importés (le manuel le promet pour l'écart du relevé, `user-manual.tex:337`) ? Alors un versement en route
  ne mettrait plus K9 en attention. Proposé ici : statut inchangé, détail ajouté (AC-4) ; la correction serait un
  amendement de 38-10 (C-38-10-4).
- **Q-40-4-2 (inter-fiches 38-9a / 38-3b)** — borne de l'instant : 38-9a compte `booked_at ≤ asOf`, l'instantané
  `reported` s'arrête à `booked_at < asOf` (38-3b AC-7 ; G5). Un mouvement Stripe à la seconde exacte de `asOf` ferait un
  écart d'un mouvement. 40-4 suit le prédicat de 38-9a (le même code) ; l'aligner sur `<` relève de 38-9a.
- **Q-40-10 (Guy)** — créance dans le total de trésorerie ou hors total : défaut « hors total, dit » (C-40-4-3).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — rédaction (Opus) : 9 AC ; 4 FR (3 de l'epic + 1 ajoutée), 3 NFR (2 + 1) ; aucune migration ; aucune route
  neuve ; écarts : règle d'« en route » par la date de l'écriture rapprochée (non `value_date`), `available`/`pending` à
  « Tester » seulement (non stockables), décalages de date autres (`otherDateShifts`), détail additif de K9, cinq
  modules. Relu au code : `20260504000001_bank_imports.sql:61-84`, `20260412000001_journal_entries.sql:21`,
  `bank_accounts.rs:1761-2025`, `closing_checks.rs:1-66`, `:550`, `:714`, `homepage.ts:1-65`,
  `BankAccountsCard.svelte:1-116`, `homepage_summary.rs:252-334`, `project_report.rs:392`, `:440-470`,
  `SettleInvoiceDialog.svelte:113-114`, `user-manual.tex:318-356`, `messages.ftl:1679-1693`.
