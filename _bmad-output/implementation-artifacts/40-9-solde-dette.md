# Story 40-9 : solde de la dette de l'Epic 40 (Stripe)

Status: done

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
| D5 | `docs/api-external.md:1399` (`BANK_ACCOUNT_FIELD_NOT_ALLOWED`) et `:1476` (`ACCOUNT_CURRENCY_PROVIDER_LINKED`) disent « compte PayPal » ; le code juge tout prestataire (40-7b P2 L4) | doc | oui (`bank_accounts.rs:2539`, `:2613` : `is_provider()` ; `accounts.rs:2743` : `provider IS NOT NULL`) | **traité** (`c0990f8e7`, hors plafond) |
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
  L3), E2E de versements en devise (40-7d P1 L3), `bank-import.spec.ts:130` : restent en issues. *(Annoté le 2026-11-25 :
  ouvertes le 2026-11-24 — verrou de la destination #1541, type de compte de la destination #1542, libellé passé d'un
  prestataire à l'autre #1535, textes du remboursement en CHF #1537, compte rendu CSV resté affiché #1546, retour au
  compte du plan d'origine #1540, E2E de versements en devise #1536.)*
- Un compte ordinaire déjà importé en CAMT peut recevoir des réglages de prestataire (40-7c P1 L4, `bug` P3) : demande
  une décision (refuser le réglage, ou dater par `value_date`), non un correctif mécanique ; reste en issue.
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
  doc-comment de `label` dit la borne. Test unitaire : catégorie de 1000 caractères, avec une **assertion de montage**
  posée sur la donnée (`row.category.chars().count() > DETAILS_MAX_CHARS` : sans correctif, le budget de la
  description est nul et la fin serait coupée ; sans cette garde, la sonde ne prouverait rien) → le libellé de la transaction et celui des frais (`provider_lines_of_row`, champ `details`)
  finissent par `) - <versement>` et tiennent dans `DETAILS_MAX_CHARS` ; rougit sans le correctif.
- **AC-3 (C3)** : l'exercice par défaut de la page des prestataires se choisit au jour de Zurich (`todayInZurich()`),
  comme la période proposée au téléchargement. Test Vitest d'une page qui monte `ProviderReviewPage` (fichier neuf à côté
  de `frontend/src/routes/(app)/reconciliation/stripe/stripe-page.test.ts`, avec ses propres simulations : deux exercices calendaires 2025 et
  2026) : fuseau du processus posé explicitement à `UTC` (`process.env.TZ`, rendu après le test, comme
  `settings/api-keys/api-keys-page.test.ts:115-128`), `vi.useFakeTimers({ toFake: ['Date'] })` à
  `2025-12-31T23:30:00Z` (jour local 31.12.2025, jour de Zurich 01.01.2026) → la revue est demandée pour l'exercice 2026.
  Rougit sans le correctif quel que soit le fuseau du poste (le fuseau est posé par le test). Le test de
  `defaultFiscalYearId`, pure, ne suffit pas : le défaut est dans l'appel de `onMount`.
- **AC-4 (C4)** : le lien « Comptes bancaires » du panneau d'erreur d'import ne s'affiche que pour les refus qui se
  règlent à l'écran *Comptes bancaires* : `PROVIDER_ACCOUNT_NOT_FOUND`, `PROVIDER_CURRENCY_ACCOUNT_MISSING`,
  `PROVIDER_CURRENCY_DUPLICATE` et `BANK_ACCOUNT_CLOSED` (refus du cœur d'import, `provider_imports.rs:7-10` ; un
  compte clos se rouvre à cet écran). Liste blanche : une erreur réseau ou inconnue n'a pas de lien. *(Annoté le 2026-11-25 : le dialogue de téléchargement
  par l'API n'offre pas ce lien, #1545.)* La page garde le **code**
  du refus (état neuf, remis à zéro avec `importErrors` au changement de compte ou d'exercice et à chaque import) :
  `importErrorLines` ne rend que des phrases. Règle hors composant (`importErrorConcernsAccounts(code)`,
  `provider-review.ts`), testée seule code par code ; et, sur la page Stripe, le panneau avec le lien pour
  `PROVIDER_ACCOUNT_NOT_FOUND` et sans lien pour `STRIPE_CSV_INVALID`.
- **AC-5 (C5)** : quand la destination réglée n'est pas parmi les candidates affichées (lecture des comptes archivés et
  clos en échec) — c'est-à-dire absente de `payoutSource`, non seulement des candidates filtrées par devise —, le
  sélecteur montre une option « Destination actuelle (introuvable) » portant l'identifiant, choisie ;
  la valeur envoyée reste inchangée. Texte aux quatre locales ; `sitesTotal` recompté par la garde. Test Vitest du cas
  dégradé (lecture `listBankAccounts(true, true)` en échec) : destination absente de la liste de l'écran → « Destination
  actuelle (introuvable) » choisie ; **et**, dans le même cas dégradé, une candidate active de la liste de l'écran reste
  offerte, nommée (c'est ce qui tue la mutation du repli `payoutPool`, `… : null` → `… : []`, qui afficherait aussi
  « introuvable »). Limite, tracée en issue : une destination présente dans la liste mais écartée par le filtre (devenue
  Stripe ou PayPal, autre devise) reste affichée en blanc. *(Annoté le 2026-11-25 : #1543.)*
- **AC-6 (docs)** : D1 à D6 faits ; D4 réécrit, non retiré : le `PUT` ne protège toujours pas la destination
  (décision 5 de `fix-40`), seule la fin « la relecture à l'usage viendra » est périmée. Grep des symptômes sur le
  dépôt (code, quatre locales, manuels `.tex`, docs, fiches) ; la formule de `POST /stripe/accept`
  (`api-external.md:1072`, « les propositions de la revue, renvoyées telles quelles ») décrit le contrat de la route,
  qui tolère les champs en trop : gardée, et dite au Change Log. Manuels : aucun changement attendu (lus :
  `user-manual.tex:3466-3510`), à confirmer au grep.
- **AC-7** : entrées corrigées barrées dans `kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md` avec le commit ; entrées
  non retenues annotées de la raison.

## Tâches

1. Étape 0 : relire D5 au code (`is_provider()` du `PATCH`/`PUT`, `provider IS NOT NULL` d'`accounts.rs`).
2. C1 (kesh-db) + test ; C2 (kesh-api) + test.
3. C3 (import de `todayLocal` retiré de `ProviderReviewPage.svelte`), C4, C5 (frontend) + tests ; i18n (C5) aux quatre locales ; garde `sitesTotal`.
4. D1 à D6.
5. Mutations hors dépôt, une à la fois, tuées sur assertion nommée.
6. Gates : fmt, clippy workspace, nextest complet, frontend complet (`check`, `lint-i18n-ownership`, Vitest, `build`) ;
   E2E : aucune spec ajoutée par la story — aucun E2E (règle de la nuit du 22.11).
7. `issues-a-ouvrir.md` ; ligne `40-9-solde-dette` de `sprint-status.yaml` et lignes 40-9 de `epic-40-stripe.md`
   (`:93`, `:221`) : contenu réel de la story et report de l'extraction des lecteurs, avec sa raison ; statut `review` ;
   prompt de revue P1.

## Dev Notes

- `PayoutDestination::of(account, ledger_currency)` lit `None` comme CHF : avec la garde, une donnée incohérente (compte
  du plan d'une autre société) se lit comme à l'acceptation (`counterparty_account`, filtrée par société, rend
  `Ok(None)`, donc une devise `None`, CHF ; `payout_destination_locked`, déjà gardée, de même) : la revue et
  l'acceptation ne divergent plus sur la devise lue.
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
- 2026-11-22 — validation P1 (complète, une lentille, Claude Sonnet, contexte frais ; rapport
  `kesh-gate-logs/s40-9/validation-p1-rapport.md`) : 0 C, 0 H, 2 M, 5 L, remédiés : M1 (liste du lien élargie aux refus
  du cœur d'import qui se règlent aux comptes bancaires ; code du refus gardé par la page), M2 (fuseau du test posé, page
  montée nommée), L1 (Dev Note de C1), L2 (D4 réécrit, non retiré), L3 (formule de `POST /stripe/accept` gardée, dite),
  L4 (option « introuvable » jugée contre `payoutSource`), L5 (registre et fichier d'epic). Passe P2 à lancer.
- 2026-11-22 — validation P2 (complète, une lentille, Claude Opus, contexte frais ; rapport
  `kesh-gate-logs/s40-9/validation-p2-rapport.md`) : 0 C, 0 H, 1 M, 5 L, remédiés : M1 (sonde de C2 : catégorie de 1000
  caractères et assertion de montage), L1 (`PROVIDER_LEDGER_REQUIRED` retiré de la liste : refus du réglage, non de
  l'import), L2 (test de C5 qui tue la mutation du repli ; limite du filtre tracée), L3 (`api-external.md:1066` : motifs
  d'acceptation exacts), L4 (40-7c P1 L4 nommée), L5 (registre `in-progress`, D5 marqué traité). Passe P3 à lancer.
- 2026-11-22 — validation P3 (complète, une lentille, Claude Sonnet, contexte frais ; rapport
  `kesh-gate-logs/s40-9/validation-p3-rapport.md`) : **0 C, 0 H, 0 M, 3 L** — boucle close. LOW remédiés dans la fiche :
  L1 (registre et fichier d'epic mis à l'état réel au début du développement), L2 (assertion de montage posée sur la
  donnée, `compose` étant privée), L3 (chemin du test d'AC-3, retrait de l'import de `todayLocal`). Trend : P1 0/0/2/5 →
  P2 0/0/1/5 → P3 0/0/0/3. Statut `in-progress` (développement).
- 2026-11-22 — revue d'interaction de la fusion de la 40-8b (orchestrateur, rapport
  `kesh-gate-logs/integration/epic-40-fusion-40-8b-revue.md`, 0 au-dessus de LOW) : son L1 (`api-external.md:1066`,
  « Angle mort connu… la relecture à l'usage viendra ») est le D4 de cette fiche, déjà corrigé (`c0990f8e7`, précisé
  par la remédiation de la validation P2) ; son L2 (le binaire du téléchargement ne joue pas la revue) noté, sans suite ici.
- 2026-11-22 — développement (`d9e642777`) ; gates verts ; statut `review`, revue de code P1 à lancer (prompt
  `kesh-gate-logs/s40-9/revue-p1-prompt.md`).

- 2026-11-22 — revue de code P1 (Claude Opus, contexte frais, rapport `kesh-gate-logs/s40-9/revue-p1-rapport.md`) :
  **0 C, 0 H, 0 M, 5 L** — boucle close sans code de production. L1 : test du lien pour `PROVIDER_ACCOUNT_NOT_FOUND`
  ajouté sur la page Stripe (AC-4) ; L2 : limite du témoin des mutations dite au Dev Agent Record ; L3 : commentaire de
  `bank-accounts/+page.svelte` aux deux causes ; L4 : conflits de fusion vers `epic/39-wise` notés pour l'intégration
  (`provider-review.test.ts` add/add, `api-external.md` « PayPal, Stripe, Wise ») ; L5 : ergonomie de l'option
  « introuvable », hors AC, notée par l'orchestrateur *(annoté le 2026-11-25 : #1544)*. Trend validation P1 0/0/2/5 → P2 0/0/1/5 → P3 0/0/0/3 ; revue
  de code P1 0/0/0/5. Statut `done`.

## Dev Agent Record

Journaux : `kesh-gate-logs/s40-9/`. Dernier commit de code : `d9e642777`.

### Fait

- **C1** — `kesh-db/src/repositories/bank_accounts.rs` : `AND a.company_id = bank_accounts.company_id` dans les
  sous-requêtes de devise de `list_provider_balances` et de `find_payout_destinations` ; doc-comments.
- **C2** — `kesh-api/src/routes/stripe_imports.rs`, `label` : catégorie bornée à `PROVIDER_TYPE_MAX_CHARS` ; doc-comment.
- **C3** — `ProviderReviewPage.svelte` : `todayInZurich()` au lieu de `todayLocal()` (import retiré).
- **C4** — `provider-review.ts` : `IMPORT_ERRORS_CONCERNING_ACCOUNTS`, `importErrorCode`, `importErrorConcernsAccounts` ;
  la page garde le code du refus (`importErrorCodeValue`, remis à zéro avec `importErrors`) et n'affiche le lien
  (`{testId}-import-error-accounts-link`) que pour les quatre refus du cœur d'import.
- **C5** — `bank-accounts/+page.svelte` : `payoutMissing` (destination absente de `payoutSource`) et option
  « Destination actuelle (introuvable) » (`{prefix}-payout-missing`) ; clé `bank-accounts-labels-payout-missing` aux
  quatre locales ; `sitesTotal` 3741 → **3742**, relevé par la garde (« expected 3742 to be 3741 »).
- **D1–D6** — commentaires de `stripe_reconciliation_e2e.rs`, `api-external.md` (`:1066`, `:1074`, `:1399`, `:1476`),
  fiche `fix-40-destination-rejugee.md` close et sa clé au registre (`c0990f8e7` et suivants). `api-external.md:1072`
  (« renvoyées telles quelles », contrat de `POST /stripe/accept`, qui tolère les champs en trop) gardé. Manuels : aucun
  changement (grep de `todayLocal`, « introuvable », « Comptes bancaires » : rien que C3 à C5 rende faux).
- Registre (`40-9-solde-dette`) et fichier d'epic : état réel, extraction des lecteurs reportée.
- `kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md` : quatre entrées barrées « CORRIGÉ par la 40-9 (`d9e642777`) »
  (40-7d P1 L7, 40-7d P4 L1, 40-7d P4 L2, intégration 38/40 exercice), trois « TRAITÉ par fix-40 (`0638cb1c0`) »
  (40-7b archivage, P1 L2, P2 LOW 3), la dette de la boucle de lecture annotée (reportée) ; deux entrées ajoutées
  (destination écartée par le filtre affichée en blanc — validation P2 L2 ; copies privées de `paypal_imports.rs` —
  40-7b P2 L5). Copie d'avant : `kesh-gate-logs/s40-9/issues-a-ouvrir.avant-40-9.md`.

### Tests ajoutés (de `b321f24ad` à `d9e642777`, recomptés au diff)

- Rust : **2** — `bank_accounts_repository::devise_du_compte_du_plan_lue_dans_la_seule_societe` (intégration, témoin
  cohérent et lien incohérent, revue et import) ; `stripe_imports::tests::categorie_longue_bornee_dans_le_libelle`
  (assertion de montage sur la donnée).
- Vitest : **7** blocs `it`/`it.each` (14 cas) — `provider-review.test.ts` (neuf : 4 + 5 + 1 cas),
  `stripe-page-exercice.test.ts` (neuf, `TZ=UTC`, horloge au 31.12.2025 23 h 30 UTC), `stripe-page.test.ts` (+1 cas, +1
  assertion dans le refus de format), `bank-accounts-page-stripe.test.ts` (+2).

### Gates (au dernier commit de code `d9e642777`)

- `cargo fmt --check` et clippy workspace `-D warnings` verts (`fmt-1.log`, `clippy-1.log`).
- `scripts/test-fast.sh --ci` (base `kesh_s409` remise à zéro) : **6553 exécutés, 6553 passés, 15 ignorés**
  (`gate-backend-2.log` ; le premier passage, `gate-backend-1.log`, n'a pas démarré : bus systemd utilisateur refusé,
  relancé avec `KESH_NO_MEMGUARD=1`).
- Frontend : `check` 0 erreur, `lint-i18n-ownership`, `build` verts ; Vitest **5088 / 5088, 298 fichiers**
  (`front-1-*.log`).
- E2E : aucune spec ajoutée par la story — aucun E2E (règle de la nuit du 22.11). E2E complet et `gate-ci.sh` à
  l'intégration de fin. Les captures ont été prises par une spec **temporaire**, jamais commitée
  (`kesh-gate-logs/s40-9/zz-captures-40-9.spec.ts`, retirée du worktree par son script ; `e2e-cap.log` : 1 passé).

### Mutations (copies `git archive` hors dépôt, cibles propres, une à la fois ; témoin non muté vert d'abord)

- Rust (`mut-rust-all.console`) : témoin 2/2 verts ; garde de société de `list_provider_balances` retirée → tuée
  (`bank_accounts_repository.rs:3236`, « import : compte du plan d'une autre société ») ; de `find_payout_destinations`
  → tuée (`:3225`, « revue : … ») ; borne de la catégorie retirée → tuée (`stripe_imports.rs:727`).
- Vitest (`mut-front-all.console`) : témoin vert ; `todayInZurich` → jour local → tuée (« expected 25 to be 26 ») ;
  `importErrorConcernsAccounts` → toujours vrai → tuée (lien présent sur un refus de format) ; `payoutMissing` neutralisé
  → tuée (« expected '' to be '4' ») ; repli `payoutPool` `null` → `[]` → tuée (« expected [ '', '4' ] to include '6' »).

- **Limite (revue P1, L2)** : le témoin non muté des mutations Vitest n'a tourné que sur `stripe-page-exercice.test.ts`
  (`mut-f-temoin.log`, 1 passé) ; `stripe-page.test.ts` et `bank-accounts-page-stripe.test.ts` n'ont pas eu de témoin
  dans la copie. Chaque mutation est tuée par une assertion nommée, et les fichiers passent à HEAD (revue P1 : 86/86).

### Captures (thème clair, après correctif ; `kesh-gate-logs/s40-9/captures/`)

- `40-9-01-refus-de-format-sans-lien.png`, `40-9-02-refus-de-compte-avec-lien.png` (page Stripe) ;
  `40-9-03-destination-introuvable.png` (comptes bancaires, lecture des comptes archivés et clos en échec simulée).
- Pas de capture « avant » : l'état d'avant est celui des captures de la 40-7d (`kesh-gate-logs/s40-7d/captures/`).

### Points relevés

- L2 de la revue de fusion 40-8b : `stripe_download_e2e` ne joue pas la revue (une revue qui ne rejugerait plus la
  destination y survit) — couvert par `stripe_reconciliation_e2e`, noté.

