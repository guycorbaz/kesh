# Story 35-6 : le solde d'ouverture d'un compte bancaire ne lit plus « 1.234 » comme un franc en silence, et l'écran et le serveur ont la même grammaire du montant (#1026, #1028, #1025)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story`. Release **0.17.7**. **Aucune migration.** Code lu sur **`release/0.17.6`** (`4e1c768a`), qui porte la
34-2 (#533) et la passe de textes de la 0.17.6 (`4e1c768a`). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -15) ;
`e35-choix-autonomes-e356.md` pour la story, à créer au premier choix. Validation à lancer par l'orchestrateur.*

## Story

En tant que titulaire qui saisit le solde d'ouverture d'un compte bancaire, je veux qu'une saisie qui peut se lire de deux
façons (« 1.234 » à l'allemande, « 1,234 » à l'anglaise) soit refusée avec la forme à écrire, et qu'une saisie que Kesh
ne sait pas lire (« 1_000 ») soit refusée par l'écran comme par le serveur, afin qu'un solde d'ouverture mille fois trop
petit ne s'écrive jamais dans mes livres.

## Ce qui se lit au code (`release/0.17.6`)

**L'écran** — `frontend/src/lib/features/bank-accounts/format.ts` :
- `openingAmountPayload` (`:236-239`) retire les séparateurs suisses (`['’\s  ]`) et lit **la première** virgule comme un
  point (`.replace(',', '.')`) ; il n'ôte ni `+` ni `_`.
- `openingAmountAmbiguous` (`:251-253`) : `/^\d+,\d{3}$/` sur la saisie sans séparateurs. Seule la **virgule** suivie de
  trois chiffres est ambiguë ; **`1.234` ne l'est pas** (#1026) ; un `+` en tête la rend non ambiguë (`+1,234` → envoyé
  `+1.234`, lu 1,234 franc, #1026) ; **`0,500`** l'est à tort (#1025 : aucun millier ne commence par 0).
- `openingAmountPreview` (`:264-269`) : rien si ambigu, ou si la charge ne suit pas `/^\d+(\.\d{1,4})?$/` — `1_000` n'a
  donc pas d'aperçu ; mais rien ne **refuse** la saisie à l'écran.
- `formatChfExact` (`:286-288`) passe la chaîne à `Intl.NumberFormat` : `formatChfExact('1_000')` rend « NaN CHF ».
- La page `frontend/src/routes/(app)/bank-accounts/+page.svelte` : à la création (`submitCreate`, `:450-…`), seule
  l'ambiguïté est arrêtée (`:456`), puis la charge part (`:482-485`) ; l'avis de succès formate la charge envoyée
  (`formatChfExact(payload.openingBalance?.amount ?? '0')`, `:499`, `:510`) — d'où « NaN CHF » pour `1_000` (#1028).
  Même chose pour un compte existant (`confirmOpeningBalance`, `:275-308`, avis `:299`). Le message d'ambiguïté
  (`ambiguousAmountMessage`, `:253-258`, clé `bank-accounts-opening-error-amount-ambiguous`, fr-CH `messages.ftl:1713`)
  conseille d'écrire **`1.234` pour un franc vingt-trois** : il devient faux.

**Le serveur** — `crates/kesh-api/src/routes/opening_balances.rs:515-530`, `parse_complement_amount` : `Decimal::from_str`
sur la chaîne rognée, puis positif, quatre décimales, borne. `rust_decimal` accepte le souligné (`1_000` → 1000, #1028) et
un `+` en tête. Appelée par le solde d'ouverture d'un compte bancaire (`routes/bank_accounts.rs:749`, création `:982`,
compte existant `:1249`) **et** par le complément des *Soldes de départ* (`opening_balances.rs:553-554`).

**Les docs** — `docs/manual/fr/user-manual.tex:2456` : « Une virgule suivie d'exactement trois chiffres (`1,234`) est
refusée comme ambiguë … `1.234` est toujours lu ainsi, jamais comme un millier à l'allemande … (\#1026). Le refus touche
aussi quelques montants qui n'étaient pas ambigus, comme `0,500` … (\#1025). » `docs/api-external.md:272` : « `amount`
chaîne décimale positive d'au plus quatre décimales ».

**Tests existants** — `frontend/src/lib/features/bank-accounts/format.test.ts:141-185` : `1.234` déclaré **non** ambigu
(`:165`), `1.005` attendu dans l'aperçu (`:181`) ; tous deux changent. Côté serveur :
`crates/kesh-api/tests/bank_account_opening_balance_e2e.rs`, `opening_balances_e2e.rs`.

## Décisions (C-e350-15)

- **D1 — La grammaire de l'écran**, après retrait des séparateurs suisses et d'un `+` en tête : des chiffres, au plus un
  séparateur décimal (point **ou** virgule), au plus quatre décimales. Trois issues pour une saisie :

  | Issue | Exemples |
  |---|---|
  | **ambiguë**, refusée avec la forme à écrire : un séparateur (point ou virgule) suivi d'exactement trois chiffres, partie entière de 1 à 3 chiffres sans zéro de tête, **ou** la règle de la virgule d'aujourd'hui | `1,234`, `12,500`, `1'000,000`, ` 9,999 `, `1.234`, `12.500`, `999.999`, `+1,234`, `+1.234` |
  | **claire**, aperçu et envoi | `1,23`, `1,2345`, `1'234,50`, `1'234.567`, `1234`, `1.2345`, `1.23`, `1234.567`, `0,500`, `0.500`, `+12.50` |
  | **non lisible**, refusée (message propre, aucun envoi, aucun aperçu) | `1_000`, `1.234.567`, `1,234,567`, `1e3`, `-5`, `abc`, `1.2.3`, `1.00001` |

  Le tableau est la spécification : le test l'écrit tel quel (AC1).
- **D2 — Le message d'ambiguïté** ne recommande plus `1.234` : « Montant ambigu : un point ou une virgule suivis de trois
  chiffres peuvent séparer les milliers. Écrivez 1'234 pour mille deux cent trente-quatre, ou 1.2340 pour un franc
  vingt-trois et quatre dixièmes de centime. » Un message neuf pour la saisie non lisible : « Montant non lisible : des
  chiffres, au plus un point ou une virgule décimale et quatre décimales ; les milliers s'écrivent avec une apostrophe
  (1'234). »
- **D3 — Le serveur refuse le souligné** (et toute forme hors `^\+?\d+(\.\d+)?$` après rognage) avant
  `Decimal::from_str`, par le refus existant `OPENING_COMPLEMENT_INVALID_AMOUNT`. Il garde le point seul comme séparateur
  décimal (forme d'API) et le `+` en tête. Il ne juge pas l'ambiguïté : un appelant de l'API envoie un nombre, non une
  saisie. Cela vaut aussi pour les *Soldes de départ* (même fonction) : changement de contrat, écrit à `api-external.md`.
- **D4 — Aucun « NaN CHF »** : l'avis de succès formate le montant **lu par l'écran** (la charge validée par D1), jamais une
  chaîne que `Intl` ne comprend pas ; si D1 a refusé, rien ne part.
- **D5 — #1025 entre dans la story** (C-e350-15) : même fonction, même règle ; `0,500` et `0.500` deviennent clairs.

## Critères d'acceptation

1. **AC1 — La table de D1**, dans `format.test.ts` : chaque exemple, dans sa colonne (fonction de classement unique,
   `openingAmountKind(raw) → 'ambiguous' | 'clear' | 'unreadable'` ou nom voisin ; `openingAmountAmbiguous` et
   `openingAmountPreview` s'y adossent).
2. **AC2 — L'écran refuse** une saisie ambiguë ou non lisible, à la création comme pour un compte existant, avec le
   message de D2, **sans appel au serveur** (Vitest de la page : `createBankAccount` / `enterOpeningBalance` non appelés).
3. **AC3 — L'aperçu** : présent pour une saisie claire, au format suisse, jusqu'à quatre décimales (`1.2345`, `0.500`) ;
   absent sinon ; jamais « NaN ».
4. **AC4 — L'avis de succès** ne contient jamais « NaN » (mutation M4).
5. **AC5 — Le serveur** : `parse_complement_amount` refuse `1_000`, `1__0`, `_1`, `1e3`, `1,5`, `- 1` ; accepte `1000`,
   `+1000`, `1.2345`, `0`, ``. Test unitaire (`opening_balances.rs`, `mod tests`, sans état i18n : il n'asserte que
   `Ok`/`Err` et le code), et un test HTTP par route : `POST /bank-accounts` avec `openingBalance.amount = "1_000"` → `400
   OPENING_COMPLEMENT_INVALID_AMOUNT`, rien n'est écrit (ni compte ni écriture) ; le complément des *Soldes de départ* avec
   `debit = "1_000"` → même refus.
6. **AC6 — Quatre langues.** La clé d'ambiguïté réécrite et la clé « non lisible » dans `fr-CH`, `de-CH`, `it-CH`, `en-CH`,
   à côté de `bank-accounts-opening-*` (C-e350-8) ; aucun sélecteur.
7. **AC7 — Manuel et API.** `user-manual.tex:2456` : la règle de D1 (point **et** virgule ambigus, `1.2340`, `0,500` clair,
   saisie non lisible refusée) ; les renvois à #1026 et #1025 retirés. `docs/api-external.md:272` : `amount` « chiffres,
   point décimal, `+` en tête admis ; ni souligné ni exposant » ; même phrase pour le complément des *Soldes de départ*.
   PDF régénéré, contrôlé aplati. CHANGELOG `## [0.17.7] — Non publié`, *Corrigé*, un point par issue (C-e350-8), dont le
   changement de contrat de D3.
8. **AC8 — Captures** (thème clair, base jetable, `kesh-gate-logs/e356/captures/`) : `1.234` refusé ; `1_000` refusé ;
   `0,500` accepté avec son aperçu.

## Tests à écrire, et la preuve du rouge

`format.test.ts` (AC1, AC3) et Vitest de la page (AC2, AC4) ; `opening_balances.rs` `mod tests` et les deux tests HTTP
(AC5) — écrits d'abord, rouges sur le code de départ (`1.234` clair, `1_000` accepté, « NaN CHF »), journal
`kesh-gate-logs/e356/rouges.log`. Les tests existants `format.test.ts:165` (`1.234` non ambigu) et `:181` (`1.005` dans
l'aperçu) changent d'attendu : `1.234` et `1.005` sont ambigus ; la preuve des quatre décimales passe à `1.0050`.

**Mutations** : M1 la règle du point retirée (`1.234` redevient clair) ; M2 `0,500` redevient ambigu ; M3 le `+` en tête
contourne la règle ; M4 l'avis formate la chaîne brute ; M5 le contrôle du souligné retiré du serveur ; M6 la saisie non
lisible part au serveur.

## Tâches

- [ ] Étape 0 : relire les lignes citées.
- [ ] Tests, rouges consignés ; `format.ts`, page, clés ×4 ; `parse_complement_amount` ; manuel, API, CHANGELOG ; captures.
- [ ] Gates : `test-fast.sh --ci` (le serveur change), frontend complet, E2E complet au dernier commit de code.

## Dev Notes

### Hors périmètre, issues à demander à l'orchestrateur (une par écran)

- *Soldes de départ* (`routes/(app)/settings/opening-balances/+page.svelte:199-200`, `:245-246`) : `1.234` y part tel quel,
  lu 1,234 franc, et `1'234` est refusé par le serveur — même risque d'un solde mille fois trop petit.
- *Écriture manuelle* (`features/journal-entries/JournalEntryForm.svelte:208-209`, `balance.ts:70`) : même lecture.

### Fichiers communs — règle d'union C-e350-8

`crates/kesh-api/src/routes/opening_balances.rs` avec la **35-5a** (ici `:515-530` ; la 35-5a, `:244`, `:604` : hunks
distincts ; la 35-6, plus courte, fusionne d'abord si les deux sont en vol) ; `messages.ftl` ×4 ; `user-manual.tex`
(ici `:2456`) ; `docs/api-external.md` (ici la grammaire du montant ; la 35-5a, la phrase « UTC » de la même ligne `:272` :
**même ligne**, à unir à la main) ; `CHANGELOG.md`.

### Ce qui doit rester vrai

- Les séparateurs suisses (`1'234.50`) et la virgule décimale (`75,25`) restent acceptés à l'écran. Le sens présélectionné,
  l'aperçu du montant nul (#1027) et les refus du serveur (compte mouvementé, exercice) ne changent pas. Fixtures
  synthétiques seulement.

### Références

- Issues #1026, #1028, #1025 ; Story 34-2 (#533), revue P1 B-1.
- `CLAUDE.md` : § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `release/0.17.6`) ; validation à lancer.

### File List

## Change Log

- 2026-11-12 — Fiche rédigée (Epic 35). Validation non lancée.
