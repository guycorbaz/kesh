# Story 35-6 : la saisie du solde d'ouverture d'un compte bancaire ne lit plus « 1.234 » comme un franc en silence, et ne montre plus « NaN CHF » (#1026, #1028, #1025)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story`. Release **0.17.7**. **Aucune migration** ; depuis la validation P2, **aucun changement serveur ni
d'API** (C-e350-19) : la story est de l'écran seul. Code lu sur **`release/0.17.6`** (`4e1c768a`). Registres :
`e35-choix-autonomes-e350.md` (C-e350-8, -15, -17, -18, -19) ; `e35-choix-autonomes-e356.md` pour la story, à créer au
premier choix. Validations P1 et P2 remédiées (Change Log).*

## Story

En tant que titulaire qui saisit le solde d'ouverture d'un compte bancaire, je veux qu'une saisie que Kesh lirait autrement
que je ne l'entends (« 1.234 » à l'allemande) soit refusée avec la forme à écrire, que les saisies sans ambiguïté
(« 0,500 », « 1'234,567 ») passent, et que l'écran montre toujours le montant tel qu'il sera écrit, afin qu'un solde
d'ouverture mille fois trop petit ne s'écrive jamais dans mes livres.

## Le principe (C-e350-19)

**Ce qui est accepté aujourd'hui reste accepté**, avec la même valeur, **sauf** une saisie que Kesh lit aujourd'hui avec une
valeur différente de celle que l'utilisateur voulait : le cas « 1.234 ». Chaque saisie nouvellement refusée est listée une
par une, justifiée, et annoncée (CHANGELOG, manuel). Le serveur ne change pas : il n'y a donc aucun changement de contrat
d'API à annoncer.

## Ce qui se lit au code (`release/0.17.6`)

**L'écran** — `frontend/src/lib/features/bank-accounts/format.ts` :
- `openingAmountPayload` (`:236-239`) retire les séparateurs `THOUSANDS_SEPARATORS = /['’\s  ]/g` (`:242` ; `\s` est
  **Unicode** en JavaScript : espace, U+00A0, U+202F, U+2009, tabulation…) et lit **la première** virgule comme un point ; il
  n'ôte ni `+` ni `_`. Vide → `null`.
- `openingAmountAmbiguous` (`:251-253`) : `/^\d+,\d{3}$/` sur la saisie **sans séparateurs**. Seule la virgule suivie de trois
  chiffres est ambiguë ; **`1.234` ne l'est pas** (#1026) ; un `+` en tête la rend non ambiguë (`+1,234` → `+1.234`, lu 1,234
  franc, #1026) ; `0,500`, `1'234,567`, `1 234,567` le sont **à tort** (#1025).
- `openingAmountPreview` (`:264-269`) : rien si ambigu, ou si la charge ne suit pas `/^\d+(\.\d{1,4})?$/`.
- `formatChfExact` (`:286-288`) passe la chaîne à `Intl.NumberFormat` : `formatChfExact('1_000')` rend « NaN CHF » (#1028).
- La page `frontend/src/routes/(app)/bank-accounts/+page.svelte` : à la création, la garde n'est jouée que si un compte lié est
  choisi (`:456`, `formJournalAccountId !== null`) ; un montant vide n'envoie rien (`:481-485`) ; l'avis formate la charge
  envoyée (`:499`, `:510`). Compte existant (`confirmOpeningBalance`, `:275-308`) : vide → `amount-required` (`:278-281`), avis
  `:299`. Aperçu du montant nul : `/^0+(\.0+)?$/` sur la charge (`:722`). Message d'ambiguïté (`:253-258`, repli littéral
  `:255-256`, clé `bank-accounts-opening-error-amount-ambiguous`, fr-CH `messages.ftl:1713`) : il conseille `1.234` pour un franc
  vingt-trois — faux après la story ; `bank-accounts-page.test.ts:924`, `:998` attendent son préfixe « Montant ambigu ».

**Le serveur, inchangé** — `parse_complement_amount` (`crates/kesh-api/src/routes/opening_balances.rs:515-530`) :
`Decimal::from_str` (`rust_decimal` 1.43.0), puis positif, quatre décimales après normalisation (`:524`), borne de quinze
chiffres. D'après la source du crate (validation P2) : accepte `1_000`, `1__0`, `1.2_5` (souligné **après** un chiffre), `+`,
`.5`, `5.`, `-0` et `-0.00` (zéro, le signe effacé) ; refuse `_1`, `1e3`, les chiffres non ASCII, `1,5`, tout négatif non nul.

**Les docs** — `docs/manual/fr/user-manual.tex:2456` (`1,234` ambigu ; `1.234` « toujours lu ainsi … (\#1026) » ; `0,500` refusé
« (\#1025) ») ; `:2462` (compte existant). Le PDF aplati porte « (#1026) », « (#1025) » et des ligatures (« aﬀiché »).

**Tests existants** — `format.test.ts:141-185` : `openingAmountPayload('   ')` → `null` (`:155-156`) ; `1'000,000` ambigu
(`:162`) ; `1.234` non ambigu (`:165`) ; `1.005` dans l'aperçu (`:181`). `bank-accounts-page.test.ts:917-998`.
`frontend/src/lib/shared/i18n-keys.test.ts:972` : `sitesTotal`.

## Décisions (C-e350-15, -17, -19)

- **D1 — Les séparateurs restent ceux d'aujourd'hui, plus le souligné.** L'ensemble retiré est celui de `:242` (apostrophes
  `'` `’`, tout blanc `\s`) **et** `_` : aucune saisie acceptée aujourd'hui avec ces caractères ne change de valeur (`1'2345`,
  `1234'567`, `1  234`, `1<TAB>234`, U+2009 restent acceptés, lus comme aujourd'hui) ; `1_000` est lu 1000, comme le serveur le
  lit, et l'aperçu le montre (#1028 : écran et serveur d'accord).
- **D2 — Ambiguë** (refusée avec la forme à écrire) : la partie entière **ne porte aucun séparateur**, compte 1 à 3 chiffres
  `[0-9]` sans zéro de tête, puis **un** point ou **une** virgule, puis exactement trois chiffres — après retrait d'un `+` en
  tête. Une partie entière qui porte un séparateur (`1'234,567`) rend le séparateur suivant décimal (#1025).
- **D3 — La table**, que le test écrit telle quelle (AC1). Les espaces spéciales s'écrivent par échappement dans le test
  (`'1 234,50'`, `'1 234.50'`).

  | Saisie | Issue | Charge |
  |---|---|---|
  | `''` · `'   '` | vide : création sans solde ; compte existant `amount-required` (inchangé) | — |
  | `1.234` · `12.500` · `999.999` · `1.005` · `+1.234` | **ambiguë — nouvellement refusée** (lue aujourd'hui 1,234 franc, etc.) | — |
  | `1,234` · `12,500` · ` 9,999 ` · `+1,234` | ambiguë (déjà refusée aujourd'hui à l'écran, sauf `+1,234`, lue aujourd'hui 1,234 franc : **nouvellement refusée**) | — |
  | `1'000,000` · `1'234,567` · `1 234,567` (U+0020 et U+00A0) · `1'234.567` · `12'34,567` | claire (#1025) | `1000.000`, `1234.567`, `1234.567`, `1234.567`, `1234.567` |
  | `0,500` · `0.500` · `0,123` · `.500` · `,500` | claire (#1025) | `0.500`, `0.500`, `0.123`, `0.500`, `0.500` |
  | `1234,567` · `1234.567` | claire | `1234.567` |
  | `1 234,50` (U+0020) · `1 234,50` (U+00A0) · `1 234.50` (U+202F) · `1'234.50` · `1'234,50` | claire | `1234.50` |
  | `12.50` · `12,5` · `1,23` · `1,2345` · `1234` · `1.2345` · `75,25` · `+12.50` | claire | `12.50`, `12.5`, `1.23`, `1.2345`, `1234`, `1.2345`, `75.25`, `12.50` |
  | `1'2345` · `1234'567` · `1  234` · `1<TAB>234` · `1 234` (U+2009) · `12'34` | claire (acceptées aujourd'hui, même valeur) | `12345`, `1234567`, `1234`, `1234`, `1234`, `1234` |
  | `1_000` · `1__0` · `1.2_5` | claire (#1028 ; acceptées aujourd'hui par le serveur, même valeur) | `1000`, `10`, `1.25` |
  | `.5` · `,5` · `5.` | claire | `0.5`, `0.5`, `5` |
  | `1.50000` | claire (zéros au-delà de la quatrième décimale) | `1.5000` |
  | `0` · `0,00` · `0.000` · `+0` · `-0` · `-0.00` | claire, montant nul (#1027 : l'avis « aucune écriture ») | `0`, `0.00`, `0.000`, `0`, `0`, `0.00` |
  | `-5` · `-0,5` | négative : `amount-required` (refusée aujourd'hui par le serveur) | — |
  | `1.23456` · `1.234.567` · `1,234,567` · `1,234.56` · `1.234,56` · `1e3` · `_1` · `١٢٣` · `abc` · `1.2.3` · `+` · `1-2` | non lisible (toutes refusées aujourd'hui par le serveur : rien de nouveau) | — |
  | `1234567890123456` (seize chiffres) | claire à l'écran ; refusée par le serveur (borne), comme aujourd'hui | `1234567890123456` |

- **D4 — Nouvellement refusées**, la liste close : la ligne « ambiguë » au point (`1.234`, `12.500`, `999.999`, `1.005`,
  `+1.234`) et `+1,234`. Justification : Kesh les lit aujourd'hui avec une valeur mille fois plus petite que celle d'une
  saisie à l'allemande ou à l'anglaise. Annoncées au CHANGELOG et au manuel (AC6).
- **D5 — La charge** : séparateurs de D1 et `+` retirés, virgule lue comme point, `0` ajouté devant un séparateur en tête
  (`.5` → `0.5`), point final retiré (`5.` → `5`), zéros **au-delà de la quatrième décimale** retirés (`1.50000` → `1.5000`),
  signe d'un zéro retiré (`-0.00` → `0.00`). L'aperçu, l'avis de succès et le test du montant nul (`:722`) lisent cette charge.
- **D6 — Les messages** : ambiguïté réécrite, **commençant toujours par « Montant ambigu : »** (« un point ou une virgule
  suivis de trois chiffres peuvent séparer les milliers. Écrivez 1'234 pour mille deux cent trente-quatre, ou 1.2340 pour un
  franc vingt-trois et quatre dixièmes de centime ») ; une clé neuve « non lisible » ; une saisie négative non nulle →
  `amount-required`.
- **D7 — #1025 est fermée en entier** (`closes #1025`) ; #1028 est fermée par D1 (l'écran lit `1_000` comme le serveur et
  l'affiche juste) ; #1026 par D2.

## Critères d'acceptation

1. **AC1 — La table de D3**, dans `format.test.ts`, par **une** fonction de classement (`openingAmountKind(raw) → { kind:
   'empty' | 'ambiguous' | 'clear' | 'unreadable' | 'negative', payload?: string }`, nom au choix ; `openingAmountAmbiguous`,
   `openingAmountPayload` et `openingAmountPreview` s'y adossent) : chaque ligne, son issue **et sa charge**. Chaque charge
   claire suit `^[0-9]+(\.[0-9]{1,4})?$`, sauf la ligne des seize chiffres (hors de ce motif seulement par la borne serveur).
2. **AC2 — L'écran refuse** une saisie ambiguë, non lisible ou négative, **dans les deux formulaires** (création :
   `formError` ; compte existant : `openingError`), avec le message de D6, sans appel au serveur. À la création, le
   classement n'est joué **que si un compte lié est choisi**, comme aujourd'hui ; compte lié et montant vide →
   `createBankAccount` appelé sans `openingBalance` (Vitest de la page).
3. **AC3 — L'aperçu** : présent pour une saisie claire (`1_000` → 1'000.00 au format suisse ; `1 234.50` ; `0.500`) ;
   l'avis du montant nul pour `+0`, `-0.00`, `0,00` ; absent sinon ; jamais « NaN ».
4. **AC4 — L'avis de succès** formate la charge ; jamais « NaN ».
5. **AC5 — Quatre langues et gardes.** La clé d'ambiguïté réécrite et la clé neuve (`bank-accounts-opening-error-…`) dans
   `fr-CH`, `de-CH`, `it-CH`, `en-CH`, à côté de `bank-accounts-opening-*` (C-e350-8) ; aucun sélecteur ; appels `i18nMsg` à
   clé littérale, **replis littéraux égaux au fr-CH** (`:255-256` et le neuf) ; `i18n-keys.test.ts` (`sitesTotal`) recompté à
   la source (C-e350-18). G9 non concernée (aucune clé `bank-accounts-opening-*` côté Rust).
6. **AC6 — Manuel et CHANGELOG.**
   - `user-manual.tex:2456` : la règle (point **et** virgule ambigus après une partie entière de 1 à 3 chiffres sans
     séparateur ; `1.2340` ; `0,500` et `1'234,567` clairs ; `1_000` lu 1000) ; la liste de D4 dite ; les renvois
     « (\#1026) » et « (\#1025) » retirés. PDF régénéré ; contrôle au PDF aplati : **absence** de « (#1026) » et « (#1025) »
     (sans barre oblique), **présence** de « 1.2340 » (sonde positive) ; greper des mots sans `ff`/`fi`/`fl` (ligatures).
   - CHANGELOG `## [0.17.7] — Non publié`, *Corrigé*, un point par issue (C-e350-8) ; sous #1026, la liste de D4.
   - `docs/api-external.md` : **inchangé** (le serveur ne change pas).
7. **AC7 — Captures** (thème clair, base jetable, `kesh-gate-logs/e356/captures/`) : `1.234` refusé ; `1_000` accepté avec
   son aperçu ; `1'234,567` accepté avec son aperçu.

## Tests à écrire, et la preuve du rouge

`format.test.ts` (AC1, AC3) et Vitest de la page (AC2, AC4) — écrits d'abord, journal `kesh-gate-logs/e356/rouges.log`.
Rouges au départ : `1.234` et `+1,234` clairs, `1'234,567` et `0,500` ambigus, `1_000` sans aperçu (« NaN » à l'avis), `-0`
sans l'avis du montant nul, les lignes « non lisible » envoyées au serveur au lieu d'être refusées à l'écran. Verts au
départ, et dits comme tels : le vide (AC2), les lignes claires d'aujourd'hui qui gardent leur valeur. Tests existants
qui changent d'attendu : `format.test.ts:162` (`1'000,000`), `:165` (`1.234`), `:181` (`1.005`, la preuve des quatre décimales
passe à `1.0050`).

**Mutations** : M1 la règle du point retirée (`1.234` redevient clair) ; M2 `0,500` redevient ambigu ; M3 le `+` en tête
contourne la règle ; M4 l'avis formate la chaîne brute ; M5 `_` retiré de l'ensemble des séparateurs (`1_000` non lisible,
AC1) ; M6 la saisie non lisible part au serveur — sur **chacun** des deux formulaires ; M7 la borne « 1 à 3 chiffres » retirée
(`1234.567` devient ambigu) ; M8 le classement fait après le retrait des séparateurs (`1'234,567` redevient ambigu) ; M9
l'ensemble des séparateurs réduit à l'espace ASCII et aux apostrophes (U+00A0, U+202F, tabulation : AC1) ; M10 le vide
classé non lisible (AC2, création sans solde).

## Tâches

- [ ] Étape 0 : relire les lignes citées sur `release/0.17.6`.
- [ ] Tests, rouges consignés ; `format.ts`, page, clés ×4, gardes i18n ; manuel, CHANGELOG ; captures.
- [ ] Gates : frontend complet ; backend complet (`test-fast.sh --ci` : les catalogues changent) ; E2E complet au dernier
      commit de code.

## Dev Notes

### Hors périmètre, une issue par écran

- **#1050** — *Soldes de départ* : l'écran (`settings/opening-balances/+page.svelte:199-200`, `:245-246`) et la route de
  génération (`opening_balances.rs:363-366`).
- **#1051** — *Écriture manuelle* : l'écran (`JournalEntryForm.svelte:208-209`, `balance.ts:70`) et la route
  (`journal_entries.rs:542-545`).
- Le prix d'un article (`product-helpers.ts:72`) : `1.234` y est un prix légitime ; aucune issue ouverte par cette fiche.
- Le serveur (`parse_complement_amount`) n'est pas touché : sa grammaire (`1_000`, `-0` acceptés) est celle que l'écran suit.

### Fichiers communs — règle d'union C-e350-8

`messages.ftl` ×4 ; `user-manual.tex` (ici `:2456`) ; `CHANGELOG.md` ; `frontend/src/lib/shared/i18n-keys.test.ts`
(`sitesTotal`). Plus aucun fichier commun avec la 35-5a (le serveur n'est plus touché).

### Ce qui doit rester vrai

- Aucune saisie acceptée aujourd'hui n'est refusée, hors la liste de D4 ; aucune ne change de valeur. Le sens présélectionné,
  l'aperçu du montant nul (#1027) et les refus du serveur ne changent pas. Fixtures synthétiques seulement.

### Références

- Issues #1026, #1028, #1025 ; #1050, #1051 ; Story 34-2 (#533) ; validations P1 et P2 `kesh-gate-logs/e35-6-validation-p*.md`.
- `CLAUDE.md` : § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel* ; § *Règle de splitting préventif*.

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
- 2026-11-12 — **Validation P2** (Opus) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 5 LOW (`kesh-gate-logs/e35-6-validation-p2.md`).
  - **Signal de découpage** (`CLAUDE.md`, § *Règle de splitting préventif*) : P1 5 MEDIUM → P2 5 MEDIUM, sévérité égale.
    Constat : **recyclage** — V2-2, V2-3, V2-4 et V2-5 naissent de la grammaire écrite à la remédiation P1 (contrôle du
    serveur, groupement fermé, `\d`) ; V2-1 (le vide) est d'origine. Réponse : non pas découper, mais **réduire** par le
    principe C-e350-19 (décision de l'orchestrateur) : ce qui est accepté aujourd'hui reste accepté avec la même valeur, hors
    le cas « 1.234 » ; le serveur n'est plus touché ; la story devient de l'écran seul (trois modules : `features/bank-accounts`,
    la page, les catalogues). Signal à déclarer à Guy par l'orchestrateur.
  - Prémisses relues ; la lecture de `rust_decimal` 1.43.0 est celle du rapport (non rejouée ici), cohérente avec le code
    d'appel (`opening_balances.rs:515-530`). Appliqués : V2-1 (vide, D3, AC2, M10) ; V2-2 (le serveur ne change pas : `-0`,
    `1_000` restent acceptés, l'écran les lit comme lui ; `_1`, `١٢٣` déjà refusés, retirés des rouges) ; V2-3 (les
    séparateurs d'aujourd'hui restent, plus `_`) ; V2-4 (U+00A0, U+202F, `12.50`, `12,5` dans la table, par échappement) ;
    V2-5 (M9 remplacée : l'ensemble des séparateurs réduit) ; LOW : V2-6 (`1.5000`) ; V2-7 (seize chiffres hors parité, `.500`,
    `,500`, `1-2`) ; V2-8 (PDF : `(#1026)` sans barre oblique, sonde positive, ligatures) ; V2-9 (`sitesTotal` ; plus de fichier
    commun avec la 35-5a ; message « Montant ambigu : » gardé) ; V2-10 (sans objet : `api-external.md` inchangé).
  - Symptômes greppés (fiche, registre, fichier d'epic) : « le serveur refuse le souligné », « Changement de contrat »,
    « sous-puce *API* », `opening_balances.rs` dans les fichiers communs — remplacés ou annotés.
