# Story 40-9 : solde de la dette de l'Epic 40 (Stripe)

Status: ready-for-dev

*Fiche rédigée le 2026-11-22, branche `story/40-9`, partie d'`epic/40-stripe` à `b321f24ad` (fusion de la 40-8b).
Story de solde de l'Epic 40 (`CLAUDE.md`, § *Priorités des défauts* : « une story de solde à la fin de chaque epic ») ;
**plafond de 5 issues** de la dette des stories de l'epic, fixé au kickoff (`planning-artifacts/epic-40-stripe.md`,
ligne 40-9 et § *Stories* : « plafond de 5 issues »). **Aucune migration, aucun changement de contrat de l'API.**
Documentation et commentaires sont hors plafond ; les défauts du code comptent.*

## Story

En tant que responsable de l'Epic 40, je veux solder, avant la PR de l'epic, la dette la moins risquée laissée par ses
stories (40-7a à 40-8b, et le correctif `fix-40-destination-rejugee`), pour que l'epic parte avec des commentaires et une
documentation justes et cinq défauts de moins à ouvrir en issues.

## Inventaire de la dette candidate (sources lues)

Sources : ligne `40-9-solde-dette` de `sprint-status.yaml` ; notes « 40-9 » de `kesh-gate-logs/consignes-correctifs.md`
(revue P1 du correctif `fix-40`, LOW-1, 3, 4 ; LOW-C de la revue P2 de la 39-11d2 ; LOW-2 de la revue de fusion
11d2/40) ; entrées « (40-…) », « (fix-40) », « (intégration 38/40) » de `kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md` ;
LOW des Dev Agent Records des 40-7a à 40-8b (dont les LOW 1 à 5 de la revue P2 de la 40-7b, versés « en dette pour la
story de solde 40-9 »). Chaque prémisse a été relue au code de `b321f24ad` :

| # | Dette | Nature | Relu au code | Décision |
|---|---|---|---|---|
| D1 | `stripe_reconciliation_e2e.rs:1072-1073` : « une destination CHF ferait refuser par `counterparty_wrong_currency` » — faux depuis `fix-40` (refus `STRIPE_PAYOUT_ACCOUNT_INVALID`/`currency`) | commentaire | oui | **traité** (hors plafond) |
| D2 | `stripe_reconciliation_e2e.rs:1797-1799` : « la route Wise n'est pas sur cette branche » — deviendra faux à la reprise d'`epic/40` par l'Epic 39 | commentaire | oui | **traité** : phrase rendue indépendante de la branche (hors plafond) |
| D3 | `docs/api-external.md:1074` : l'écran Stripe envoie des « propositions renvoyées telles que la revue les montre » — faux : `stripeAcceptBody` (`stripe-review.ts:20`) n'envoie que les champs du corps | doc | oui | **traité** (hors plafond) |
| D4 | `docs/api-external.md:1066` : « Angle mort connu : la destination n'est pas protégée… la relecture à l'usage viendra avec les écritures de versement » — périmé : `fix-40` rejuge la destination à la revue et à l'acceptation | doc | oui | **traité** (hors plafond) |
| D5 | `docs/api-external.md:1399` (`BANK_ACCOUNT_FIELD_NOT_ALLOWED`) et `:1476` (`ACCOUNT_CURRENCY_PROVIDER_LINKED`) disent « compte PayPal » ; le code juge tout prestataire (40-7b P2 L4) | doc | à relire au code (étape 0) | **traité** si le code juge `is_provider()` / `provider IS NOT NULL` (hors plafond) |
| D6 | Fiche `fix-40-destination-rejugee.md` en `review`, sans Change Log de sa revue P1 (0/0/0/4), sans clé au registre | fiche | oui | **traité** (hors plafond) |
| C1 | `find_payout_destinations` et `list_provider_balances` (`kesh-db/src/repositories/bank_accounts.rs:1652`, `:1686`) lisent la devise du compte du plan sans `a.company_id = bank_accounts.company_id` (fix-40 P1 LOW-3) | code, défense | oui | **plafond 1/5** |
| C2 | Libellé d'une transaction Stripe : la catégorie n'est pas bornée, une catégorie longue coupe la fin `) - <versement>` (`stripe_imports.rs:188-209`, 40-7b P2 L1) | code | oui | **plafond 2/5** |
| C3 | Page des prestataires : exercice par défaut au jour **local** (`ProviderReviewPage.svelte:178`, `todayLocal()`), période de téléchargement au jour de **Zurich** (intégration 38/40) | code | oui | **plafond 3/5** |
| C4 | Page des prestataires : le lien « Comptes bancaires » du panneau d'erreur d'import s'affiche pour tout refus (`ProviderReviewPage.svelte:373`, 40-7d P1 L7) | code | oui | **plafond 4/5** |
| C5 | Comptes bancaires : destination de versement absente de la liste lue affichée en blanc (`bank-accounts/+page.svelte:1137-1153`, 40-7d P4 L1) ; le test du repli de `payoutPool` manque (40-7d P4 L2, même chemin, « lié au point précédent ») | code + test | oui | **plafond 5/5** (L2 soldé par le test de L1, un seul chemin) |

**Non retenu (le plafond est atteint ; reste en issue à ouvrir, raison écrite à `issues-a-ouvrir.md`)** :

- Extraction commune de la boucle de lecture, de la collecte plafonnée et de la troncature des lecteurs CSV (dette
  inscrite au registre, 40-7a P1 M2) : sa condition, « une fois les trois lecteurs en place », n'est pas remplie sur
  `epic/40-stripe` — le lecteur Wise (39-11a) est sur `epic/39-wise` (`ls crates/kesh-import/src/` : `paypal.rs`,
  `stripe.rs`, pas de `wise.rs`). L'extraire ici sur deux lecteurs forcerait un troisième remaniement à la fusion
  40 → 39. Elle va à l'intégration des Epics 38 à 40 (issue).
- Copies privées des aides dans `paypal_imports.rs` et `bounded` sans appelant (40-7b P2 L5) : remaniement d'un fichier
  de l'Epic 38, à faire sur la branche d'intégration (issue).
- Verrou de la destination à l'acceptation (fix-40, `technical-debt` P3), type de compte de la destination (40-7b, P3),
  libellé passé d'un prestataire à l'autre (40-7d P1 L1), textes du remboursement en CHF (40-7d P1 L8), compte rendu CSV
  resté affiché après un téléchargement, six clés `provider-*` sans site, retour au compte du plan d'origine (40-7d P4
  L3), E2E de versements en devise (40-7d P1 L3), `bank-import.spec.ts:130` : restent en issues.
- Côté Wise du L3 (chaîne versement Stripe → compte Wise → `POST /wise/accept`) : de l'Epic 39 ; entrée ajoutée à
  `issues-a-ouvrir.md` si absente (l'entrée « (intégration 39/40) » existe déjà).
- Les entrées « (40-7b) Archiver un compte désigné… », « (40-7b, revue P1 L2) dérives ultérieures » et « (40-7b, revue
  P2 LOW 3) changer son compte du plan » sont **déjà traitées par `fix-40`** (`0638cb1c0` : rejugement à la revue et à
  l'acceptation ; refus au `PUT` non retenu, décision 5 de sa fiche) : barrées comme telles, hors plafond.

## Critères d'acceptation

- **AC-1 (C1)** : les deux sous-requêtes de devise du compte du plan portent `AND a.company_id =
  bank_accounts.company_id`. Test d'intégration `kesh-db` : un compte bancaire dont `journal_account_id` désigne un
  compte en EUR **d'une autre société** (donnée incohérente posée en SQL) est rendu par `find_payout_destinations` et
  `list_provider_balances` avec `ledger_currency = None` ; il rougit sans la garde (mutation).
- **AC-2 (C2)** : la catégorie, dans le libellé, est bornée à `PROVIDER_TYPE_MAX_CHARS` (comme `provider_type`) ; le
  doc-comment de `label` dit la borne. Test unitaire : catégorie de 300 caractères → le libellé (et celui des frais)
  finit par `) - <versement>` et tient dans `DETAILS_MAX_CHARS` ; rougit sans le correctif.
- **AC-3 (C3)** : l'exercice par défaut de la page des prestataires se choisit au jour de Zurich (`todayInZurich()`),
  comme la période proposée au téléchargement. Test Vitest qui rougit sans le correctif (horloge figée à une heure où les
  deux jours diffèrent, p. ex. 31 décembre 23 h 30 UTC, exercices calendaires).
- **AC-4 (C4)** : le lien « Comptes bancaires » du panneau d'erreur d'import ne s'affiche que pour les refus
  `PROVIDER_ACCOUNT_NOT_FOUND` et `PROVIDER_CURRENCY_ACCOUNT_MISSING`. Règle hors composant (`provider-review.ts`),
  testée seule ; et le panneau sans lien pour un refus de format (test du composant ou d'une page).
- **AC-5 (C5)** : quand la destination réglée n'est pas parmi les candidates affichées (lecture des comptes archivés et
  clos en échec), le sélecteur montre une option « Destination actuelle (introuvable) » portant l'identifiant, choisie ;
  la valeur envoyée reste inchangée. Texte aux quatre locales ; `sitesTotal` recompté par la garde. Test Vitest du cas
  dégradé (lecture `listBankAccounts(true, true)` en échec), qui tue aussi la mutation du repli `payoutPool` (`… : null`
  → `… : []`).
- **AC-6 (docs)** : D1 à D6 faits ; grep des symptômes sur le dépôt (code, quatre locales, manuels `.tex`, docs, fiches).
- **AC-7** : entrées corrigées barrées dans `kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md` avec le commit ; entrées
  non retenues annotées de la raison.

## Tâches

1. Étape 0 : relire D5 au code (`is_provider()` du `PATCH`/`PUT`, `provider IS NOT NULL` d'`accounts.rs`).
2. C1 (kesh-db) + test ; C2 (kesh-api) + test.
3. C3, C4, C5 (frontend) + tests ; i18n (C5) aux quatre locales ; garde `sitesTotal`.
4. D1 à D6.
5. Mutations hors dépôt, une à la fois, tuées sur assertion nommée.
6. Gates : fmt, clippy workspace, nextest complet, frontend complet (`check`, `lint-i18n-ownership`, Vitest, `build`) ;
   E2E : aucune spec ajoutée par la story — aucun E2E (règle de la nuit du 22.11).
7. `issues-a-ouvrir.md` ; statut `review` ; prompt de revue P1.

## Dev Notes

- `PayoutDestination::of(account, ledger_currency)` lit `None` comme CHF : avec la garde, une donnée incohérente (compte
  du plan d'une autre société) se lit comme à l'acceptation (`counterparty_account`, filtrée par société), qui rend
  alors `ACCOUNT_NOT_FOUND` ; la revue et l'acceptation ne divergent plus sur la devise lue.
- C2 : aucune catégorie Stripe réelle ne dépasse 64 caractères (liste fermée) ; le libellé d'une ligne réelle ne
  change pas.
- C3 : `defaultFiscalYearId(fiscalYears, today)` (`provider-review.ts`) reste pure ; seul le jour passé change.
- Pas de PDF ni de CHANGELOG commités ; aucun nom de personne, de société, de chemin personnel ni de logiciel tiers.

## Notes de version

- **Fixed** — Prestataires (PayPal, Stripe) : l'exercice proposé par défaut est celui du jour à Zurich, comme la période
  de téléchargement ; le lien « Comptes bancaires » d'un refus d'import ne paraît plus que si le refus porte sur les
  comptes.
- **Fixed** — Comptes bancaires : un compte de versement Stripe que l'écran ne retrouve pas s'affiche « Destination
  actuelle (introuvable) » au lieu d'un sélecteur vide.

## Change Log

- 2026-11-22 — fiche rédigée (`story/40-9`, depuis `b321f24ad`).

## Dev Agent Record
