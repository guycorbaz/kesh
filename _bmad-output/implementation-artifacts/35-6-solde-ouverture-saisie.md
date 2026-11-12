# Story 35-6 : la saisie du solde d'ouverture d'un compte bancaire ne lit plus « 1.234 » comme un franc en silence, et ne montre plus « NaN CHF » (#1026, #1028, #1025)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story`. Release **0.17.7**. **Aucune migration** ; depuis la validation P2, **aucun changement du code
serveur ni d'API** (C-e350-19) : la story est de l'écran, plus un test de parité côté serveur (C-e350-33, test seul). Code lu
sur **`release/0.17.6`** (`4e1c768a`). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -15, -17, -18, -19, -30 à -33) ;
`e35-choix-autonomes-e356.md` pour la story, à créer au premier choix. Validations P1, P2 et P3 remédiées (Change Log).*

## Story

En tant que titulaire qui saisit le solde d'ouverture d'un compte bancaire, je veux qu'une saisie que Kesh lirait autrement
que je ne l'entends (« 1.234 » à l'allemande) soit refusée avec la forme à écrire, que les saisies sans ambiguïté
(« 0,500 », « 1'234,567 ») passent, et que l'écran montre toujours le montant tel qu'il sera écrit, afin qu'un solde
d'ouverture mille fois trop petit ne s'écrive jamais dans mes livres.

## Le principe (C-e350-19)

**Ce qui est accepté aujourd'hui reste accepté**, avec la même valeur, **sauf** une saisie que Kesh lit aujourd'hui avec une
valeur différente de celle que l'utilisateur voulait : le cas « 1.234 ». Les saisies nouvellement refusées forment des
**familles décrites par une règle décidable** (D4), justifiées, avec leurs exemples et leurs bornes, et annoncées
(CHANGELOG, manuel). Le serveur ne change pas : il n'y a donc aucun changement de contrat d'API à annoncer.

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
chiffres. D'après la source du crate (validation P2, relue en P3 : `str.rs:245`, `b'_' if HAS`, le souligné est permis dès
qu'un chiffre a été lu, avant comme après le point ; `:325`, refusé avant) : accepte `1_000`, `1__0`, `1.2_5`, `1_`, `1_.5`,
`1._5`, `+`, `.5`, `5.`, `-0` et `-0.00` (zéro, le signe effacé) ; refuse `_1`, `+_5`, `._5`, `1e3`, les chiffres non ASCII,
`1,5`, tout négatif non nul. Il rogne la saisie (`str::trim`, `:517`).

**Les docs** — `docs/manual/fr/user-manual.tex:2456` (`1,234` ambigu ; `1.234` « toujours lu ainsi … (\#1026) » ; `0,500` refusé
« (\#1025) ») ; `:2462` (compte existant). Le PDF aplati porte « (#1026) », « (#1025) » et des ligatures (« aﬀiché »).

**Tests existants** — `format.test.ts:141-185` : `openingAmountPayload('   ')` → `null` (`:155-156`) ; `1'000,000` ambigu
(`:162`) ; `1.234` non ambigu (`:165`) ; `1.005` dans l'aperçu (`:181`). `bank-accounts-page.test.ts:917-998`.
`frontend/src/lib/shared/i18n-keys.test.ts:972` : `sitesTotal`.

## Décisions (C-e350-15, -17, -19, -30 à -33)

- **D1 — Les séparateurs.** Séparateurs **de groupe** : ceux de `:242` d'aujourd'hui (apostrophes `'` `’`, tout blanc `\s`,
  Unicode en JavaScript : espace, U+00A0, U+202F, U+2009, tabulation). Ils sont permis **dans la partie entière seulement**,
  en nombre quelconque (`1'2345`, `1234'567`, `1  234`, `1<TAB>234`, U+2009 restent acceptés, lus comme aujourd'hui).
  **Le souligné** suit la règle du serveur (`rust_decimal` 1.43.0 `str.rs:245`, `b'_' if HAS`) : permis **dès qu'un chiffre a
  été lu**, retiré, transparent pour le classement ; avant le premier chiffre, il rend la saisie non lisible (`_1`, `+_5`,
  `._5`, que le serveur refuse aussi, `str.rs:325`). `1_000` est lu 1000 et l'aperçu le montre (#1028). *(C-e350-31 : la règle
  « entre deux chiffres » proposée en P3 aurait refusé `1_` et `1_.5`, que le serveur lit 1 et 1,5 aujourd'hui.)*
- **D2 — La lecture, un pipeline ordonné** (C-e350-30). Une fonction, `openingAmountKind(raw)`, rend `{ kind, payload? }` ; les
  étapes s'appliquent dans cet ordre, la première qui conclut l'emporte :
  1. **L1 — Rognage** : blancs Unicode de tête et de queue retirés (`String.prototype.trim`). Reste vide → `empty`.
  2. **L2 — Signe** : un `+` ou un `-` en premier caractère est ôté et retenu, puis le reste est **rogné à nouveau**
     (`+ 1,234` → `1,234`). Reste vide (`+`, `-`), ou portant encore un `+` ou un `-` (`++1`, `+-1`, `1+2`, `1-2`) →
     `unreadable`.
  3. **L3 — Souligné** (D1) : un `_` avant le premier chiffre → `unreadable` ; les autres sont retirés.
  4. **L4 — Découpe** : plus d'un `.` ou `,` au total (`1.234.567`, `1,234,567`, `1,234.56`, `1.234,56`, `1.2.3`) →
     `unreadable`. Le séparateur décimal, s'il y en a un, coupe la partie entière **E** et la fraction **F**.
  5. **L5 — Fraction** : F ne contient que des chiffres `[0-9]` (vide permis : `5.`). Un blanc ou une apostrophe dans F
     (`1,23 4`, `1. 234`, `1.2 5`, `1.5'0`) → `unreadable` : un séparateur de groupe n'a pas de lecture sûre après la décimale.
  6. **L6 — Partie entière** : E est dite **groupée** si un ou plusieurs séparateurs de groupe y sont placés **entre deux
     chiffres** ; les séparateurs de groupe de E sont retirés ; E ne doit plus contenir que `[0-9]` (vide permis si F ne l'est
     pas : `.5`) — sinon → `unreadable` (`1e3`, `abc`, `١٢٣`, `.`).
  7. **L7 — Classement**, dans cet ordre :
     - F, ses zéros au-delà de la quatrième décimale retirés, garde plus de quatre chiffres (`1.23456`, `-1.23456`) →
       `unreadable` ;
     - le signe est `-` et la valeur n'est pas nulle (`-5`, `-0,5`, `-1.234`) → `negative` ;
     - **ambiguë** (la règle décidable de D4) : un séparateur décimal, F (telle que saisie) d'exactement trois chiffres, E non
       groupée, et les chiffres de E, zéros de tête ôtés, au nombre de 1 à 3 — soit, sur E et F ainsi lus, le motif
       `^0*[1-9][0-9]{0,2}[.,][0-9]{3}$` → `ambiguous` ;
     - sinon → `clear`, avec la charge de D5.

  « Clair » se définit ainsi : `kind === 'clear'` ; sa charge suit `^[0-9]+(\.[0-9]{1,4})?$` (AC1).
- **D3 — La table**, que le test écrit telle quelle (AC1), chaque ligne vérifiée contre le pipeline de D2 (validation P3) ;
  « aujourd'hui » désigne l'écran et le serveur de `release/0.17.6`. Les espaces spéciales s'écrivent par échappement dans le test
  (`'1 234,50'`, `'1 234.50'`).

  | Saisie | Issue | Charge |
  |---|---|---|
  | `''` · `'   '` | vide (L1) : création sans solde ; compte existant `amount-required` (inchangé) | — |
  | `1.234` · `12.500` · `999.999` · `1.005` · `1.000` · `100.250` · `+1.234` · `01.234` · `'1.234` · `1 .234` · `1_.234` | **ambiguë — nouvellement refusée** (D4, famille A ; lues aujourd'hui 1,234 franc, 12,5, …, 1, 100,25) | — |
  | `+1,234` · `+ 1,234` · `+01,234` | **ambiguë — nouvellement refusée** (D4, famille B : le `+` masquait la virgule ; lues aujourd'hui 1,234 franc) | — |
  | `1,234` · `12,500` · `100,000` · ` 9,999 ` · `01,234` · `012,345` · `1 ,234` | ambiguë (déjà refusées aujourd'hui à l'écran par `^\d+,\d{3}$` : rien de nouveau) | — |
  | `1. 234` · `1.2 5` · `1.5'0` | **non lisible — nouvellement refusée** (D4, famille C ; lues aujourd'hui 1,234, 1,25, 1,5) | — |
  | `1,23 4` | non lisible (déjà refusée aujourd'hui : après retrait, `1,234` était ambiguë) | — |
  | `1'000,000` · `1'234,567` · `1 234,567` (U+0020 et U+00A0) · `1'234.567` · `12'34,567` | claire (#1025 ; E groupée) | `1000.000`, `1234.567`, `1234.567`, `1234.567`, `1234.567` |
  | `0,500` · `0.500` · `0,123` · `0.999` · `.500` · `,500` · `00,500` | claire (#1025 ; E nulle ou vide ; `00,500` refusée aujourd'hui, C-e350-32) | `0.500`, `0.500`, `0.123`, `0.999`, `0.500`, `0.500`, `00.500` |
  | `1234,567` · `1234.567` · `1000.000` | claire (E de quatre chiffres) | `1234.567`, `1234.567`, `1000.000` |
  | `1 234,50` (U+0020) · `1 234,50` (U+00A0) · `1 234.50` (U+202F) · `1'234.50` · `1'234,50` | claire | `1234.50` |
  | `12.50` · `12,5` · `1,23` · `1,2345` · `1234` · `1.2345` · `1.2340` · `75,25` · `+12.50` | claire | `12.50`, `12.5`, `1.23`, `1.2345`, `1234`, `1.2345`, `1.2340`, `75.25`, `12.50` |
  | `1'2345` · `1234'567` · `1  234` · `1<TAB>234` · `1 234` (U+2009) · `12'34` | claire (acceptées aujourd'hui, même valeur) | `12345`, `1234567`, `1234`, `1234`, `1234`, `1234` |
  | `1_000` · `1__0` · `1.2_5` · `1_` · `1_.5` | claire (#1028 ; acceptées aujourd'hui par le serveur, même valeur) | `1000`, `10`, `1.25`, `1`, `1.5` |
  | `.5` · `,5` · `5.` | claire | `0.5`, `0.5`, `5` |
  | `1.50000` · `1.23450` | claire (zéros au-delà de la quatrième décimale) | `1.5000`, `1.2345` |
  | `0` · `0,00` · `0.000` · `+0` · `-0` · `-0.00` | claire, montant nul (#1027 : l'avis « aucune écriture ») | `0`, `0.00`, `0.000`, `0`, `0`, `0.00` |
  | `-5` · `-0,5` · `-1.234` | négative : `amount-required` (refusées aujourd'hui par le serveur) | — |
  | `1.23456` · `-1.23456` · `1.234.567` · `1,234,567` · `1,234.56` · `1.234,56` · `1e3` · `_1` · `+_5` · `١٢٣` · `abc` · `1.2.3` · `+` · `1-2` · `1+2` · `++1` | non lisible (toutes refusées aujourd'hui, par le serveur : rien de nouveau) | — |
  | `1234567890123456` (seize chiffres) | claire à l'écran ; refusée par le serveur (borne), comme aujourd'hui | `1234567890123456` |

- **D4 — Nouvellement refusées : trois familles, par règle** (C-e350-30, -32), non une liste finie. Justification commune :
  Kesh les lit aujourd'hui avec une valeur que l'utilisateur n'a peut-être pas voulue (mille fois plus petite pour une saisie
  à l'allemande ou à l'anglaise), sans signal.
  - **A — un point suivi de trois chiffres après une partie entière courte** : après L1 à L3, la partie entière sans
    séparateur de groupe entre deux chiffres, et `^0*[1-9][0-9]{0,2}\.[0-9]{3}$` sur ses chiffres, le point et la fraction.
    Exemples : `1.234`, `1.000`, `100.250`, `999.999`, `+1.234`, `01.234`. **Bornes, claires** : `1000.000` (quatre chiffres),
    `0.999` et `0.500` (partie entière nulle : le zéro seul n'est pas un zéro de tête), `1'234.567` (groupée), `1.2340`, `1.23`.
  - **B — la même règle avec la virgule, masquée par un `+`** (`+1,234`, `+ 1,234`, `+01,234`) ; sans `+`, la virgule était déjà
    refusée.
  - **C — un blanc ou une apostrophe après le séparateur décimal** (`1. 234`, `1.2 5`, `1.5'0`) : non lisibles.

  Le CHANGELOG et le manuel annoncent **la règle** (point comme virgule), avec `1.234` et `1.000` en exemples et `1000.000`,
  `0.500` en bornes (AC6). Les saisies citées aux rouges de départ (« Tests à écrire ») ne sont **que** des rouges, non
  l'ensemble du refus.
- **D5 — La charge** (saisie claire) : E, séparateurs de groupe et soulignés retirés, `0` si E est vide (`.5` → `0.5`) ; puis,
  si F n'est pas vide, `.` et F, ses zéros au-delà de la quatrième décimale retirés (`1.50000` → `1.5000`) ; point final
  sans fraction retiré (`5.` → `5`) ; aucun signe (`+12.50` → `12.50`, `-0.00` → `0.00`) ; les zéros de tête restent, comme
  aujourd'hui (`00,500` → `00.500`, que le serveur lit 0,5). L'aperçu, l'avis de succès et le test du montant nul (`:722`)
  lisent cette charge. **`openingAmountPayload(raw)`** rend la charge si `clear`, `null` sinon (le vide compris : les tests
  `format.test.ts:154-156` restent verts) ; la page classe **avant** de prendre la charge, si bien qu'un `null` n'y signifie
  « vide » qu'après un `kind === 'empty'`.
- **D6 — Les messages** : ambiguïté réécrite, **commençant toujours par « Montant ambigu : »** (« un point ou une virgule
  suivis de trois chiffres peuvent séparer les milliers. Écrivez 1'234 pour mille deux cent trente-quatre, ou 1.2340 pour un
  franc vingt-trois et quatre dixièmes de centime ») ; une clé neuve, **`bank-accounts-opening-error-amount-unreadable`**
  (« Montant illisible : écrivez un nombre positif, par exemple 1'234.50. ») ; `negative` →
  `bank-accounts-opening-error-amount-required`. L'ordre des issues est celui de L7 : `-1.23456` est non lisible, `-1.234`
  négative.
- **D7 — Fermetures, et les motifs à reprendre dans la PR et au commentaire de fermeture** :
  - **#1025** fermée en entier (`closes #1025`) : `0,500` et `1'234,567` passent ; ne restent refusées que les formes de D4 et
    celles que l'écran refusait déjà.
  - **#1026** fermée (`closes #1026`) — motif : *« Corrigé à l'écran du solde d'ouverture (création et compte existant) :
    `1.234` et `+1,234` sont refusés comme ambigus (règle de D4). L'API garde sa lecture : un client qui envoie `"1.234"`
    écrit toujours 1,234 franc, ce qui est le sens d'un nombre décimal, et l'API ne lit pas la virgule. Les autres écrans de
    saisie d'un montant sont suivis à #1050 et #1051. »*
  - **#1028** fermée (`closes #1028`) — motif : *« L'écran lit `1_000` comme le serveur (1000) et l'affiche juste, jamais
    « NaN CHF ». Écran et serveur ne partagent pas pour autant une grammaire : l'écran retire les apostrophes et les blancs de
    la partie entière, que le serveur refuse, et refuse `1.234`, que le serveur lit 1,234 ; mais ce qui part au serveur est
    une charge qu'il lit à la valeur que l'écran a montrée (test de parité, C-e350-33). »*

## Critères d'acceptation

1. **AC1 — La table de D3**, écrite **une fois** dans la fixture `frontend/src/lib/features/bank-accounts/opening-amount-cases.json`
   (`{ raw, kind, payload?, serverRejects? }` par saisie ; espaces spéciales par échappement JSON ` `, ` `,
   ` `, `\t`), et lue par `format.test.ts` et par le test de parité (AC8). Une seule fonction de classement,
   **`openingAmountKind(raw) → { kind: 'empty' | 'ambiguous' | 'clear' | 'unreadable' | 'negative', payload?: string }`**,
   suit le pipeline de D2 ; `openingAmountPayload` (D5), `openingAmountPreview` s'y adossent, et `openingAmountAmbiguous`
   disparaît ou s'y adosse. Chaque ligne : son issue **et sa charge** ; les bornes de D4 (`1.000`, `100.250` ambiguës ;
   `1000.000`, `0.999` claires) y sont. Chaque charge claire suit `^[0-9]+(\.[0-9]{1,4})?$` ; la ligne des seize chiffres porte
   `serverRejects: true` (borne du serveur, non de l'écran).
2. **AC2 — L'écran refuse** une saisie ambiguë, non lisible ou négative, **dans les deux formulaires** (création :
   `formError` ; compte existant : `openingError`), avec le message de D6, sans appel au serveur. À la création, le
   classement n'est joué **que si un compte lié est choisi**, comme aujourd'hui ; compte lié et montant vide →
   `createBankAccount` appelé sans `openingBalance` (Vitest de la page).
3. **AC3 — L'aperçu** : présent pour une saisie claire (`1_000` → 1'000.00 au format suisse ; `1 234.50` ; `0.500`) ;
   l'avis du montant nul pour `+0`, `-0.00`, `0,00` ; absent sinon ; jamais « NaN ».
4. **AC4 — L'avis de succès** formate la charge ; jamais « NaN ».
5. **AC5 — Quatre langues et gardes.** La clé d'ambiguïté réécrite et la clé neuve `bank-accounts-opening-error-amount-unreadable`
   (D6) dans `fr-CH`, `de-CH`, `it-CH`, `en-CH`, à côté de `bank-accounts-opening-*` (C-e350-8) ; aucun sélecteur ; appels
   `i18nMsg` à clé littérale, **replis littéraux égaux au fr-CH** (`:255-256` et le neuf) ; `i18n-keys.test.ts` (`sitesTotal`)
   recompté à la source (C-e350-18). G9 non concernée : aucune des deux clés touchées n'est lue côté Rust (la seule clé
   `bank-accounts-opening-*` du serveur, `bank-accounts-opening-entry-description`, `bank_accounts.rs:788`, n'est pas touchée).
6. **AC6 — Manuel et CHANGELOG.**
   - `user-manual.tex:2456` : **la règle de D4** (un point ou une virgule suivis d'exactement trois chiffres, après une partie
     entière de un à trois chiffres sans séparateur de milliers — zéros de tête non comptés, zéro seul exclu —, est ambigu ;
     exemples `1.234`, `1.000` ; bornes claires `1000.000`, `0.500`, `1'234,567` ; écrire `1'234` ou `1.2340`) ; un blanc ou une
     apostrophe après le séparateur décimal est refusé ; `1_000` lu 1000 ; les renvois « (\#1026) » et « (\#1025) » retirés.
     PDF régénéré ; contrôle au PDF aplati : **absence** de « (#1026) » et « (#1025) » (sans barre oblique), **présence** de
     « 1.2340 » et de « 1000.000 » (sondes positives) ; greper des mots sans `ff`/`fi`/`fl` (ligatures).
   - CHANGELOG `## [0.17.7] — Non publié`, *Corrigé*, un point par issue (C-e350-8) ; sous #1026, **la règle de D4** (familles
     A, B, C), avec ses exemples et ses bornes, non une liste.
   - `docs/api-external.md` : **inchangé** (le serveur ne change pas).
7. **AC7 — Captures** (thème clair, base jetable, `kesh-gate-logs/e356/captures/`) : `1.234` refusé ; `1_000` accepté avec
   son aperçu ; `1'234,567` accepté avec son aperçu.
8. **AC8 — Parité écran/serveur** (C-e350-33, test seul, `kesh-api`) : dans le `mod tests` de
   `crates/kesh-api/src/routes/opening_balances.rs` (`:630`), un test lit la fixture d'AC1 **à l'exécution**
   (`env!("CARGO_MANIFEST_DIR")` puis `std::fs::read_to_string`, pour ne pas lier la compilation au frontend) et vérifie :
   (a) pour chaque ligne `clear` sans `serverRejects`, `parse_complement_amount(Some(payload))` vaut `Decimal::from_str(payload)`
   — la valeur que l'aperçu a montrée ; (b) pour la ligne `serverRejects`, le refus ; (c) pour chaque ligne `clear` dont la
   saisie brute n'emploie que l'alphabet du serveur (`[0-9._+-]`, rognée : `1_000`, `1__0`, `1.2_5`, `1_`, `1_.5`, `-0`,
   `+12.50`, `.5`, `5.`, `1.50000`), `parse_complement_amount(Some(raw))` vaut la même valeur que la charge ; (d) pour chaque
   ligne `unreadable` ou `negative` de cet alphabet (`_1`, `+_5`, `1+2`, `++1`, `1.23456`, `1.2.3`, `+`, `1-2`, `-5`), le refus
   du serveur. Aucune ligne de production du serveur ne change.

## Tests à écrire, et la preuve du rouge

`format.test.ts` (AC1, AC3) et Vitest de la page (AC2, AC4) — écrits d'abord, journal `kesh-gate-logs/e356/rouges.log`.
Rouges au départ : `1.234`, `1.000`, `01.234`, `+1,234` et `1. 234` clairs, `1'234,567` et `0,500` ambigus, `1_000` sans aperçu (« NaN » à l'avis), `-0`
sans l'avis du montant nul, les lignes « non lisible » envoyées au serveur au lieu d'être refusées à l'écran. Verts au
départ, et dits comme tels : le vide (AC2), les lignes claires d'aujourd'hui qui gardent leur valeur. Le test de parité
(AC8) est vert au départ (le serveur ne change pas) : sa preuve est une mutation (M17). Tests existants, liste complète
(validation P3, relue sur `release/0.17.6`) :
- `format.test.ts:154-156` (`' 12 000 '` → `12000`, `'   '` et `''` → `null`) : inchangés, verts (D5) ;
- `:160-167` : `1'000,000` (`:162`) passe d'ambigu à clair ; `1.234` (`:165`) de clair à ambigu ; les autres entrées de `:165`
  (`'1,23'`, `'1,2345'`, `"1'234,50"`, `'1234'`, `''`, `'1,2'`) restent non ambiguës — si `openingAmountAmbiguous` disparaît,
  ces deux boucles sont ré-adossées à `openingAmountKind` (même attendu, `kind !== 'ambiguous'`) ;
- `:181` (`1.005`) : la preuve des quatre décimales passe à `1.0050` ;
- `bank-accounts-page.test.ts:924` (`1,234` à la création) et `:998` (`12,500`, compte existant) : leur attendu « Montant
  ambigu » reste vrai (D6), cités pour mémoire ;
- E2E : aucun touché (`bank-account-opening-balance.spec.ts` n'emploie que `1'234.50` et `500`).

**Mutations** : M1 la règle du point retirée (`1.234` redevient clair) ; M2 `0,500` redevient ambigu ; M3 le `+` en tête
contourne la règle ; M4 l'avis formate la chaîne brute ; M5 `_` retiré de l'ensemble des séparateurs (`1_000` non lisible,
AC1) ; M6 la saisie non lisible part au serveur — sur **chacun** des deux formulaires ; M7 la borne « 1 à 3 chiffres » retirée
(`1234.567` devient ambigu) ; M8 le classement fait après le retrait des séparateurs (`1'234,567` redevient ambigu) ; M9
l'ensemble des séparateurs réduit à l'espace ASCII et aux apostrophes (U+00A0, U+202F, tabulation : AC1) ; M10 le vide
classé non lisible (AC2, création sans solde) ; M11 le rognage retiré (` 9,999 ` cesse d'être ambiguë) ; M12 les zéros
au-delà de la quatrième décimale non retirés (`1.50000` devient non lisible) ; M13 un `+` hors de la tête accepté (`1+2`) ;
M14 un `_` avant le premier chiffre accepté (`_1`, `+_5`) ; M15 un séparateur de groupe dans la fraction retiré au lieu de
refusé (`1. 234` devient ambiguë ou claire) ; M16 les zéros de tête comptés (`01,234` devient claire) ; M17 (AC8) la charge
de `1_000` portée à `100` dans la fixture : l'assertion (c) du test de parité rougit. Le test de parité affirme en dur le
nombre de lignes qu'exerce chacune de ses assertions (a) à (d), pour qu'un filtre d'alphabet trop étroit ne le fasse pas
passer à vide.

## Tâches

- [ ] Étape 0 : relire les lignes citées sur `release/0.17.6`.
- [ ] Tests, rouges consignés ; fixture de la table ; `format.ts`, page, clés ×4, gardes i18n ; test de parité (AC8) ;
      manuel, CHANGELOG ; captures.
- [ ] Gates : frontend complet ; backend complet (`test-fast.sh --ci` : les catalogues changent) ; E2E complet au dernier
      commit de code.

## Dev Notes

### Hors périmètre, une issue par écran

- **#1050** — *Soldes de départ* : l'écran (`settings/opening-balances/+page.svelte:199-200`, `:245-246`) et la route de
  génération (`opening_balances.rs:363-366`).
- **#1051** — *Écriture manuelle* : l'écran (`JournalEntryForm.svelte:208-209`, `balance.ts:70`) et la route
  (`journal_entries.rs:542-545`).
- Le prix d'un article (`product-helpers.ts:72`) : `1.234` y est un prix légitime ; aucune issue ouverte par cette fiche.
- Le code du serveur (`parse_complement_amount`) n'est pas touché ; l'écran suit sa lecture du souligné et de `-0`, et
  AC8 le vérifie (test seul).

### Fichiers communs — règle d'union C-e350-8

`messages.ftl` ×4 ; `user-manual.tex` (ici `:2456`) ; `CHANGELOG.md` ; `frontend/src/lib/shared/i18n-keys.test.ts`
(`sitesTotal`). Avec la 35-5a, un seul fichier commun : `crates/kesh-api/src/routes/opening_balances.rs`, où la 35-6
n'ajoute qu'un test au `mod tests` (`:630` et suivantes), hunk distinct de la date du jour (`:244`, `:604`) (C-e350-33).

### Ce qui doit rester vrai

- Aucune saisie acceptée aujourd'hui n'est refusée, hors les familles de D4 ; aucune ne change de valeur. Le sens présélectionné,
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
- 2026-11-12 — **Validation P3** : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 7 LOW (`kesh-gate-logs/e35-6-validation-p3.md`) ; série
  5 → 5 → 3 MEDIUM. Remédiée par l'agent de relève (Opus 5.5). Prémisses relues sur `release/0.17.6` (`5e2a5818`) :
  `format.ts:236-253`, `format.test.ts:141-185`, page `:250-310`, `:450-515`, `:722`, `opening_balances.rs:515-530`,
  `bank_accounts.rs:788`, `rust_decimal` 1.43.0 `str.rs:245` (`b'_' if HAS`), `:325`. Toutes confirmées.
  - V3-1 (MEDIUM) appliqué en partie : le `+` n'est ôté qu'en tête, une fois (L2) ; `_` avant le premier chiffre non lisible
    (L3). **Réfuté en partie** : la règle « `_` entre deux chiffres » aurait refusé `1_` et `1_.5`, que le serveur lit 1 et 1,5
    aujourd'hui (`str.rs:245` : le souligné est permis dès qu'un chiffre a été lu) — contraire à C-e350-19 ; l'écran suit la
    règle du serveur (C-e350-31). Lignes ajoutées : `1_`, `1_.5` (claires), `+_5`, `1+2`, `++1` (non lisibles).
  - V3-2 (MEDIUM) : « liste close » remplacée par la règle décidable (D4, familles A, B, C ; motif
    `^0*[1-9][0-9]{0,2}[.,][0-9]{3}$`, zéro seul exclu) dans D4, AC1, AC6 (manuel, CHANGELOG) ; bornes `1.000`, `100.250`
    (ambiguës), `1000.000`, `0.999` (claires), `100,000` ajoutées (C-e350-30).
  - V3-3 (MEDIUM) : la lecture écrite en pipeline ordonné L1-L7 (D2) ; un séparateur de groupe dans la fraction rend la saisie
    non lisible (famille C, nouvellement refusée : `1. 234`, `1.2 5`, `1.5'0`) ; chaque ligne de la table revérifiée contre
    lui ; ajoutées : `1,23 4`, `1. 234`, `+ 1,234`, `1 ,234`, `'1.234`, `1 .234`, `1_.234`, `1.23450`, `-1.234`, `-1.23456`.
  - LOW : V3-4 tranché (C-e350-32 : zéros de tête non comptés — `01,234`, `012,345` restent ambiguës comme aujourd'hui,
    `00,500` devient claire, `01.234` nouvellement refusée, famille A) ; V3-5 (définition de « clair », ordre des issues en L7,
    `openingAmountPayload` → `null` hors `clear`, clé `bank-accounts-opening-error-amount-unreadable`, M11-M16) ; V3-6
    (registre C-e350-8 annoté) ; V3-7 (AC5 reformulé, `bank_accounts.rs:788` cité) ; V3-8 (motifs de fermeture de #1026 et
    #1028 écrits en D7) ; V3-9 (test de parité AC8 sur la fixture commune, C-e350-33, M17) ; V3-10 (liste complète des tests
    existants).
  - Symptômes greppés (fiche, fichier d'epic, registre) : « liste close », « six saisies », « la liste de D4 », la liste des
    six refusées de C-e350-19 — remplacés dans la fiche, annotés au registre (C-e350-19, C-e350-8) ; fichier d'epic : la
    colonne des fichiers complétée.
