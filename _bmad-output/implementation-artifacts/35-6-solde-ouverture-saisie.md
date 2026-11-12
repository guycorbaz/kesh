# Story 35-6 : la saisie du solde d'ouverture d'un compte bancaire ne lit plus « 1.234 » comme un franc en silence, et l'écran et le serveur s'accordent sur ce qu'ils lisent (#1026, #1028, #1025)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story`. Release **0.17.7**. **Aucune migration.** Code lu sur **`release/0.17.6`** (`4e1c768a`), qui porte la
34-2 (#533) et la passe de textes de la 0.17.6. Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -15, -17) ;
`e35-choix-autonomes-e356.md` pour la story, à créer au premier choix. Validation P1 remédiée (Change Log).*

## Story

En tant que titulaire qui saisit le solde d'ouverture d'un compte bancaire, je veux qu'une saisie qui peut se lire de deux
façons (« 1.234 » à l'allemande, « 1,234 » à l'anglaise) soit refusée avec la forme à écrire, qu'une saisie sans ambiguïté
(« 0,500 », « 1'234,567 ») passe, et qu'une saisie que Kesh ne sait pas lire (« 1_000 », « 12'34 ») soit refusée par
l'écran comme par le serveur, afin qu'un solde d'ouverture mille fois trop petit ne s'écrive jamais dans mes livres.

## Ce qui se lit au code (`release/0.17.6`)

**L'écran** — `frontend/src/lib/features/bank-accounts/format.ts` :
- `openingAmountPayload` (`:236-239`) retire les séparateurs suisses (`['’\s  ]`) et lit **la première** virgule comme un
  point (`.replace(',', '.')`) ; il n'ôte ni `+` ni `_`. `12'34` devient `1234` sans signal ; `.5` part tel quel.
- `openingAmountAmbiguous` (`:251-253`) : `/^\d+,\d{3}$/` sur la saisie **sans séparateurs**. Seule la virgule suivie de trois
  chiffres est ambiguë ; **`1.234` ne l'est pas** (#1026) ; un `+` en tête la rend non ambiguë (`+1,234` → `+1.234`, lu 1,234
  franc, #1026) ; `0,500`, `1'234,567`, `1 234,567` le sont **à tort** (#1025 : un zéro de tête, ou un groupement des milliers
  déjà écrit, exclut la lecture « milliers »).
- `openingAmountPreview` (`:264-269`) : rien si ambigu, ou si la charge ne suit pas `/^\d+(\.\d{1,4})?$/` — mais rien ne
  **refuse** la saisie : elle part au serveur.
- `formatChfExact` (`:286-288`) passe la chaîne à `Intl.NumberFormat` : `formatChfExact('1_000')` rend « NaN CHF ».
- La page `frontend/src/routes/(app)/bank-accounts/+page.svelte` : à la création (`submitCreate`), seule l'ambiguïté est
  arrêtée (`:456`) ; la charge part (`:482-485`) ; l'avis formate la charge envoyée (`:499`, `:510`) — d'où « NaN CHF »
  (#1028). Même chose pour un compte existant (`confirmOpeningBalance`, `:275-308`, avis `:299`). L'aperçu du montant nul
  teste `/^0+(\.0+)?$/` sur la charge (`:722`). Le message d'ambiguïté (`ambiguousAmountMessage`, `:253-258`, repli littéral
  `:255-256`, clé `bank-accounts-opening-error-amount-ambiguous`, fr-CH `messages.ftl:1713`) conseille d'écrire **`1.234`
  pour un franc vingt-trois** : il devient faux. `bank-accounts-opening-error-amount-required` (« Saisissez un montant
  positif. », `:1724`) existe.

**Le serveur** — `crates/kesh-api/src/routes/opening_balances.rs:515-530`, `parse_complement_amount` : `Decimal::from_str`
sur la chaîne rognée, puis positif, au plus quatre décimales **après normalisation** (`:524`, `1.50000` passe), borne de
quinze chiffres entiers. `rust_decimal` accepte le souligné (`1_000` → 1000, #1028), un `+` en tête, `.5`, `5.`. Appelée par
le solde d'ouverture d'un compte bancaire (`routes/bank_accounts.rs:749`, création `:982`, compte existant `:1249`) **et** par
le complément des *Soldes de départ* (`opening_balances.rs:553-554`).

**Les autres lectures d'un montant saisi** (V1-4, recompté par `git grep` : virgule → point sans règle, ou
`Decimal::from_str` direct) — **hors périmètre**, parce qu'elles ne passent pas par `parse_complement_amount` ni par
`format.ts` :
- écran : `JournalEntryForm.svelte:208-209` et `balance.ts:70` (#1051) ; `settings/opening-balances/+page.svelte:199-200`,
  `:245-246` (#1050) ; `products/product-helpers.ts:72` (`normalizePriceInput`, prix d'article, à quatre décimales :
  `1.234` y est un prix légitime) ;
- serveur : `opening_balances.rs:363-366` (`POST /opening-balances`, génération des *Soldes de départ*, #1050) et
  `journal_entries.rs:542-545` (écriture manuelle, #1051) lisent par `Decimal::from_str` : ils continueront d'accepter
  `1_000`. Après cette story, l'écran *Soldes de départ* refusera donc `1_000` en mode *Compléter* et l'acceptera en mode
  *Générer* : écart écrit aux commentaires de #1050 et #1051.

**Les docs** — `docs/manual/fr/user-manual.tex:2456` : « Une virgule suivie d'exactement trois chiffres (`1,234`) est refusée
comme ambiguë … `1.234` est toujours lu ainsi … (\#1026). Le refus touche aussi … `0,500` … (\#1025). » ; `:2462` (compte
existant). `docs/api-external.md:272` (`POST /bank-accounts`, `amount` « chaîne décimale positive d'au plus quatre
décimales ») ; la note ¹¹ (`:270`, `:274`) nomme le complément des *Soldes de départ* sans dire la forme du montant ;
`POST /bank-accounts/{id}/opening-balance` n'en dit rien non plus.

**Tests existants** — `frontend/src/lib/features/bank-accounts/format.test.ts:141-185` : `1'000,000` ambigu (`:162`), `1.234`
non ambigu (`:165`), `1.005` dans l'aperçu (`:181`) ; tous trois changent. Côté serveur :
`crates/kesh-api/tests/bank_account_opening_balance_e2e.rs`, `opening_balances_e2e.rs`. `textes_coherents.rs` ne garde aucune
clé `bank-accounts-opening-*` (grep).

## Décisions (C-e350-15, C-e350-17)

- **D1 — Classer avant de retirer les séparateurs** (V1-1). La saisie est rognée ; un `+` en tête est retiré ; un `-` renvoie
  au message « montant positif ». Puis, sur `[0-9]` seulement (jamais `\d`, qui est Unicode) :
  - **groupement** : la partie entière est soit sans séparateur, soit groupée par trois avec `'`, `’`, une espace, U+00A0 ou
    U+202F (`1'234`, `12 345 678`) ; un groupement mal placé (`12'34`) est **non lisible** ;
  - **séparateur décimal** : au plus un, point ou virgule ; l'autre ne peut pas servir de groupement (`1,234.56`,
    `1.234,56` : **non lisibles**) ;
  - **ambiguë** : partie entière **non groupée**, de 1 à 3 chiffres, sans zéro de tête, puis un séparateur suivi d'exactement
    trois chiffres ; **claire** sinon.
- **D2 — La table complète** (V1-2), que le test écrit telle quelle (AC1). « Charge » : ce qui part au serveur.

  | Saisie | Issue | Charge |
  |---|---|---|
  | `1,234` · `12,500` · ` 9,999 ` · `1.234` · `12.500` · `999.999` · `+1,234` · `+1.234` · `1.005` | ambiguë | — |
  | `1'000,000` · `1'234,567` · `1 234,567` · `1'234.567` | claire (#1025) | `1000.000`, `1234.567`, `1234.567`, `1234.567` |
  | `0,500` · `0.500` · `0,123` | claire (#1025) | `0.500`, `0.500`, `0.123` |
  | `1234,567` · `1234.567` | claire | `1234.567` |
  | `1,23` · `1,2345` · `1'234,50` · `1234` · `1.2345` · `1.23` · `75,25` · `1'234.50` | claire | `1.23`, `1.2345`, `1234.50`, `1234`, `1.2345`, `1.23`, `75.25`, `1234.50` |
  | `+12.50` · `+0` · `0` · `0,00` · `0.000` | claire (`+0`, `0`… : montant nul, #1027) | `12.50`, `0`, `0`, `0.00`, `0.000` |
  | `.5` · `,5` · `5.` | claire (acceptée aujourd'hui : rien de nouveau n'est refusé) | `0.5`, `0.5`, `5` |
  | `1.50000` | claire (décimales nulles au-delà de quatre, que le serveur accepte aujourd'hui) | `1.5` |
  | `1.23456` | non lisible (cinq décimales significatives ; le serveur la refuse aujourd'hui) | — |
  | `1_000` · `1.234.567` · `1,234,567` · `1,234.56` · `1.234,56` · `12'34` · `1e3` · `abc` · `1.2.3` · `+` · `١٢٣` (chiffres arabo-indiens) | non lisible | — |
  | `-5` · `-0,5` | montant positif requis (`amount-required`) | — |
  | seize chiffres entiers ou plus | claire à l'écran ; le serveur la refuse par sa borne, comme aujourd'hui | la saisie |
- **D3 — Saisies acceptées aujourd'hui et refusées désormais, annoncées** (V1-2) : celles de la ligne « ambiguë » (dont
  `1.234`, `1.005`), `12'34`, `1_000`. Elles sont dites au manuel et au CHANGELOG. Aucune autre saisie aujourd'hui acceptée
  n'est refusée : `.5`, `5.`, `1.50000`, `+12.50` restent acceptées.
- **D4 — La charge** ne porte ni `+`, ni séparateur de groupement, ni virgule ; le zéro de tête manquant est ajouté (`.5` →
  `0.5`) ; un point final est retiré (`5.` → `5`) ; les zéros au-delà de quatre décimales sont retirés (V1-3). L'aperçu,
  l'avis de succès et le test du montant nul (`:722`) lisent cette charge : jamais « NaN », et `+0` donne l'avis du montant nul.
- **D5 — Les messages** : ambiguïté réécrite (« un point ou une virgule suivis de trois chiffres peuvent séparer les milliers.
  Écrivez 1'234 pour mille deux cent trente-quatre, ou 1.2340 pour un franc vingt-trois et quatre dixièmes de centime ») ;
  une clé neuve « non lisible » (« Montant non lisible : un nombre positif, au plus un point ou une virgule décimale et quatre
  décimales ; les milliers s'écrivent avec une apostrophe (1'234) ») ; `-5` → `amount-required` (V1-7).
- **D6 — Le serveur** (V1-2) : `parse_complement_amount` refuse, avant `Decimal::from_str`, tout caractère hors de
  `[0-9]`, d'un point et d'un `+` en tête — donc le souligné, l'exposant et les chiffres non ASCII — par le refus existant
  `OPENING_COMPLEMENT_INVALID_AMOUNT`. Il garde ce qu'il accepte aujourd'hui (`+`, `.5`, `5.`, `1.50000`) : le seul
  changement de contrat est le refus de `1_000` (et des formes non ASCII, à constater au rouge). Il ne juge pas
  l'ambiguïté : un appelant de l'API envoie un nombre, non une saisie. Le texte du refus ne change pas.
- **D7 — #1025 est fermée en entier** (V1-1) : `0,500`, `1'234,567` et `1 234,567` deviennent clairs ; la PR porte
  `closes #1025`.

## Critères d'acceptation

1. **AC1 — La table de D2**, dans `format.test.ts`, par **une** fonction de classement (`openingAmountKind(raw) →
   { kind: 'ambiguous' | 'clear' | 'unreadable' | 'negative', payload?: string }`, nom au choix ; `openingAmountAmbiguous`,
   `openingAmountPayload` et `openingAmountPreview` s'y adossent) : chaque ligne, son issue **et sa charge**.
2. **AC2 — L'écran refuse** une saisie ambiguë, non lisible ou négative, **dans les deux formulaires** (création : `formError` ;
   compte existant : `openingError`), avec le message de D5, **sans appel au serveur** (Vitest de la page :
   `createBankAccount` / `enterOpeningBalance` non appelés).
3. **AC3 — L'aperçu** : présent pour une saisie claire, au format suisse, jusqu'à quatre décimales (`1.2345`, `0.500`,
   `1'234,567`) ; l'avis du montant nul pour `+0`, `0,00` ; absent sinon ; jamais « NaN ».
4. **AC4 — L'avis de succès** formate la charge de D4 ; jamais « NaN ».
5. **AC5 — Le serveur** (`opening_balances.rs`, `mod tests`, sans état i18n : `Ok`/`Err` et le code seuls) : refuse `1_000`,
   `1__0`, `_1`, `١٢٣` ; accepte `1000`, `+1000`, `1.2345`, `.5`, `5.`, `1.50000`, `0`, ``. Rouges au départ : les cas du
   souligné (et des chiffres non ASCII si `from_str` les accepte : à constater) ; `1e3`, `1,5`, `- 1` sont déjà refusés
   (pas de faux rouge, V1-6). **Parité** : un tableau Rust des **charges** des lignes claires de D2 (`1234.567`, `0.500`,
   `0.5`, `5`, `1.5`, `12.50`, `0`…) est accepté par `parse_complement_amount`, et le test Vitest exige que chaque charge
   claire suive `^[0-9]+(\.[0-9]{1,4})?$`. Un test HTTP par route : `POST /bank-accounts` avec `openingBalance.amount =
   "1_000"`, `POST /bank-accounts/{id}/opening-balance` idem, et le complément des *Soldes de départ* avec `debit = "1_000"`
   → `400 OPENING_COMPLEMENT_INVALID_AMOUNT`, rien n'est écrit.
6. **AC6 — Quatre langues et replis.** La clé d'ambiguïté réécrite et la clé neuve (`bank-accounts-opening-error-…`, préfixe
   du lint d'ownership) dans `fr-CH`, `de-CH`, `it-CH`, `en-CH`, à côté de `bank-accounts-opening-*` (C-e350-8) ; aucun
   sélecteur ; appels `i18nMsg` à clé littérale, **replis littéraux égaux au fr-CH** (`:255-256` et le neuf), contrôlés par
   les gardes de replis du frontend (`harvest-i18n-fallbacks`, `i18n-keys.test.ts`). G9 n'est pas concernée (aucune clé
   `bank-accounts-opening-*` côté Rust, V1-8).
7. **AC7 — Manuel, API, CHANGELOG.**
   - `user-manual.tex:2456` : la règle de D1 (point **et** virgule ambigus, `1.2340`, `0,500` et `1'234,567` clairs, saisie
     non lisible refusée, montant positif), les renvois à #1026 et #1025 retirés ; contrôle au PDF aplati de `1.2340`,
     `0,500`, `1'234,567` et de l'absence de « (\#1026) » et « (\#1025) » (V1-9). PDF régénéré.
   - `docs/api-external.md` : **une** phrase de grammaire, à la note ¹¹ (« `amount`, `debit`, `credit` : chiffres ASCII, au
     plus un point décimal, `+` en tête admis ; ni souligné, ni exposant, ni virgule ; refus `400
     OPENING_COMPLEMENT_INVALID_AMOUNT` »), renvoyée depuis `POST /bank-accounts` (`:272`),
     `POST /bank-accounts/{id}/opening-balance` et le complément des *Soldes de départ* (V1-5).
   - CHANGELOG `## [0.17.7] — Non publié`, *Corrigé*, un point par issue (C-e350-8), et sous #1028 une **sous-puce *API***
     explicite : les trois routes refusent désormais `1_000` (`400 OPENING_COMPLEMENT_INVALID_AMOUNT`). Les saisies que
     l'écran refuse désormais (D3) sont dites sous #1026.
8. **AC8 — Captures** (thème clair, base jetable, `kesh-gate-logs/e356/captures/`) : `1.234` refusé ; `1_000` refusé ;
   `1'234,567` accepté avec son aperçu.

## Tests à écrire, et la preuve du rouge

`format.test.ts` (AC1, AC3) et Vitest de la page (AC2, AC4) ; `opening_balances.rs` `mod tests` et les tests HTTP (AC5) —
écrits d'abord, rouges sur le code de départ (`1.234` clair, `1'234,567` ambigu, `1_000` accepté, « NaN CHF »), journal
`kesh-gate-logs/e356/rouges.log`, en disant quels cas étaient déjà verts. Les tests existants `format.test.ts:162`
(`1'000,000`), `:165` (`1.234`) et `:181` (`1.005`) changent d'attendu ; la preuve des quatre décimales passe à `1.0050`.

**Mutations** : M1 la règle du point retirée (`1.234` redevient clair) ; M2 `0,500` redevient ambigu ; M3 le `+` en tête
contourne la règle ; M4 l'avis formate la chaîne brute ; M5 le contrôle du souligné retiré du serveur ; M6 la saisie non
lisible part au serveur — jouée sur **chacun** des deux formulaires ; M7 la borne « 1 à 3 chiffres » retirée (`1234.567`
devient ambigu) ; M8 le classement fait après le retrait des séparateurs (`1'234,567` redevient ambigu) ; M9 `\d` au lieu de
`[0-9]` (`١٢٣` passe).

## Tâches

- [ ] Étape 0 : relire les lignes citées ; constater `Decimal::from_str` sur `_1`, `.5`, `5.`, `١٢٣` (une ligne de test).
- [ ] Tests, rouges consignés ; `format.ts`, page, clés ×4 ; `parse_complement_amount` ; manuel, API, CHANGELOG ; captures.
- [ ] Gates : `test-fast.sh --ci` (le serveur change), frontend complet, E2E complet au dernier commit de code.

## Dev Notes

### Hors périmètre, une issue par écran

- **#1050** — *Soldes de départ* : l'écran (`settings/opening-balances/+page.svelte:199-200`, `:245-246`) et la route de
  génération (`opening_balances.rs:363-366`, `Decimal::from_str` direct).
- **#1051** — *Écriture manuelle* : l'écran (`JournalEntryForm.svelte:208-209`, `balance.ts:70`) et la route
  (`journal_entries.rs:542-545`).
- Le prix d'un article (`product-helpers.ts:72`) : `1.234` y est un prix légitime (quatre décimales) ; aucune issue ouverte
  par cette fiche, à juger par l'orchestrateur.

### Fichiers communs — règle d'union C-e350-8

`crates/kesh-api/src/routes/opening_balances.rs` avec la **35-5a** (ici `:515-530` ; la 35-5a, `:244`, `:604` : hunks
distincts ; la 35-6, plus courte, fusionne d'abord si les deux sont en vol) ; `messages.ftl` ×4 ; `user-manual.tex`
(ici `:2456`) ; `docs/api-external.md` (ici la note ¹¹ et le renvoi à `:272` ; la 35-5a, la phrase « UTC » de la même ligne
`:272` : **même ligne**, à unir à la main) ; `CHANGELOG.md`.

### Ce qui doit rester vrai

- Les séparateurs suisses (`1'234.50`) et la virgule décimale (`75,25`) restent acceptés à l'écran. Le sens présélectionné,
  l'aperçu du montant nul (#1027) et les refus du serveur (compte mouvementé, exercice) ne changent pas. Fixtures
  synthétiques seulement.

### Références

- Issues #1026, #1028, #1025 ; #1050, #1051 ; Story 34-2 (#533), revue P1 B-1 ; validation P1
  `kesh-gate-logs/e35-6-validation-p1.md`.
- `CLAUDE.md` : § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `release/0.17.6`).

### File List

## Change Log

- 2026-11-12 — Fiche rédigée (Epic 35). Validation non lancée.
- 2026-11-12 — **Validation P1** (Sonnet) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 4 LOW (`kesh-gate-logs/e35-6-validation-p1.md`).
  Prémisses relues au code de `release/0.17.6` ; aucune réfutée (#1025 relue : titre « 0,500 ou 1'234,567 » ;
  `opening_balances.rs:355-366` ; `journal_entries.rs:536-545` ; `product-helpers.ts:68-74`). Appliqués :
  - V1-1 : classement **avant** le retrait des séparateurs (D1) ; #1025 fermée en entier (D7, `closes #1025`) ;
  - V1-2 : table complète (D2), `[0-9]`, `.5`/`5.`/`1.50000` restent acceptés ; seules les saisies de D3 sont refusées
    désormais, et annoncées ; le serveur ne refuse de neuf que le souligné (et les chiffres non ASCII) ;
  - V1-3 : la charge sans `+` (D4), `+0` donne l'avis du montant nul ;
  - V1-4 : `opening_balances.rs:363` et `journal_entries.rs:542` ne passent pas par `parse_complement_amount` : **hors
    périmètre**, écrits à #1050 et #1051 (commentaires rédigés pour l'orchestrateur) ; prix d'article noté ; titre reformulé ;
  - V1-5 : une phrase de grammaire à la note ¹¹, renvoyée par les trois routes ; sous-puce *API* au CHANGELOG ;
  - LOW : V1-6 (cas déjà verts dits, parité écran/serveur, M7–M9, deux formulaires) ; V1-7 (`-5` → montant positif) ; V1-8
    (replis égaux au fr-CH, G9 non concernée) ; V1-9 (contrôles au PDF aplati).
  - Symptômes greppés dans la fiche, le registre et le fichier d'epic : « par la règle de la virgule d'aujourd'hui »,
    « `0,500` et `0.500` deviennent clairs », `\d`, « même phrase pour le complément » — remplacés.
