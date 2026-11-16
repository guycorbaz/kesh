# Story 35-6 : la saisie du solde d'ouverture d'un compte bancaire ne lit plus « 1.234 » comme un franc en silence, et ne montre plus « NaN CHF » (#1026, #1028, #1025)

Status: done

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story`. Release **0.17.7**. **Aucune migration** ; depuis la validation P2, **aucun changement du code
serveur ni d'API** (C-e350-19) : la story est de l'écran, plus un test de parité côté serveur (C-e350-33, test seul). Code lu
sur **`release/0.17.6`** (`4e1c768a`). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -15, -17, -18, -19, -30 à -35) ;
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
`Decimal::from_str` (`rust_decimal` 1.43.0), puis positif, quatre décimales après normalisation (`:526`), borne de quinze
chiffres. D'après la source du crate (validation P2, relue en P3 : `str.rs:245`, `b'_' if HAS`, le souligné est permis dès
qu'un chiffre a été lu, avant comme après le point ; `:325`, refusé avant) : accepte `1_000`, `1__0`, `1.2_5`, `1_`, `1_.5`,
`1._5`, `.5`, `5.`, `-0` et `-0.00` (zéro, le signe effacé) ; refuse `_1`, `+_5`, `._5`, `+` et `.` seuls (`str.rs:244`,
`:478`, `tail_no_has`), les chiffres non ASCII, `1,5`, tout négatif non nul. **Il accepte la notation à exposant** :
`Decimal::from_str` se rabat sur `from_scientific_lossy` dès que la saisie porte un `e` ou un `E` (`decimal.rs:2108-2117`,
`:687-735`) — `1e3` et `1E3` valent 1000, `1.5e2` 150, `5e-1` 0,5 (validation P4). Au-delà de 28 décimales, il arrondit
(`maybe_round`, `str.rs`). Il rogne la saisie (`str::trim`, `:517`). Aujourd'hui, l'écran envoie `1e3` tel quel : 1000 est
écrit, sans aperçu.

**Les docs** — `docs/manual/fr/user-manual.tex:2456` (`1,234` ambigu ; `1.234` « toujours lu ainsi … (\#1026) » ; `0,500` refusé
« (\#1025) ») ; `:2462` (compte existant). Le PDF aplati porte « (#1026) », « (#1025) » et des ligatures (« aﬀiché »).

**Tests existants** — `format.test.ts:141-185` : `openingAmountPayload('   ')` → `null` (`:155-156`) ; `1'000,000` ambigu
(`:162`) ; `1.234` non ambigu (`:165`) ; `1.005` dans l'aperçu (`:181`). `bank-accounts-page.test.ts:917-998`.
`frontend/src/lib/shared/i18n-keys.test.ts:972` : `sitesTotal`.

## Décisions (C-e350-15, -17, -19, -30 à -35)

- **D1 — Les séparateurs.** Séparateurs **de groupe** : ceux de `:242` d'aujourd'hui (apostrophes `'` `’`, tout blanc `\s`,
  Unicode en JavaScript : espace, U+00A0, U+202F, U+2009, tabulation). Ils sont permis **dans la partie entière seulement**,
  en nombre quelconque (`1'2345`, `1234'567`, `1  234`, `1<TAB>234`, U+2009 restent acceptés, lus comme aujourd'hui).
  **Le souligné** suit la règle du serveur (`rust_decimal` 1.43.0 `str.rs:245`, `b'_' if HAS`) : permis **dès qu'un chiffre a
  été lu**, retiré, transparent pour le classement ; avant le premier chiffre, il rend la saisie non lisible (`_1`, `+_5`,
  `._5`, que le serveur refuse aussi, `str.rs:325`). `1_000` est lu 1000 et l'aperçu le montre (#1028). *(C-e350-31 : la règle
  « entre deux chiffres » proposée en P3 aurait refusé `1_` et `1_.5`, que le serveur lit 1 et 1,5 aujourd'hui.)*
- **D2 — La lecture, un pipeline ordonné** (C-e350-30). Une fonction, `openingAmountKind(raw)`, rend `{ kind, payload? }` ; les
  étapes s'appliquent dans cet ordre, la première qui conclut l'emporte :
  1. **L1 — Rognage** : blancs Unicode **et apostrophes** (`'`, `’`) de tête et de queue retirés (C-e350-35 : `'+5`, `'-0`,
     `1.5'` restent acceptées, comme aujourd'hui où l'écran retirait toute apostrophe). Reste vide → `empty`.
  2. **L2 — Signe** : un `+` ou un `-` en premier caractère est ôté et retenu, puis le reste est **rogné à nouveau**, comme
     en L1 (`+ 1,234` → `1,234`). Reste vide (`+`, `-`), ou portant encore un `+` ou un `-` (`++1`, `+-1`, `1+2`, `1-2`) →
     `unreadable`.
  3. **L3 — Souligné** (D1) : un `_` avant le premier chiffre → `unreadable` ; les autres sont retirés.
  4. **L4 — Découpe** : plus d'un `.` ou `,` au total (`1.234.567`, `1,234,567`, `1,234.56`, `1.234,56`, `1.2.3`) →
     `unreadable`. Le séparateur décimal, s'il y en a un, coupe la partie entière **E** et la fraction **F**.
  5. **L5 — Fraction** : F ne contient que des chiffres `[0-9]` (vide permis : `5.`). Un blanc ou une apostrophe dans F
     (`1,23 4`, `1. 234`, `1.2 5`, `1.5'0`) → `unreadable` : un séparateur de groupe n'a pas de lecture sûre après la décimale.
  6. **L6 — Partie entière** : E est dite **groupée** si un ou plusieurs séparateurs de groupe y sont placés **entre deux
     chiffres** ; les séparateurs de groupe de E sont retirés ; E ne doit plus contenir que `[0-9]` (vide permis si F ne l'est
     pas : `.5`) — sinon → `unreadable` (`1e3`, `abc`, `١٢٣`, `.`). Un `e` ou un `E` n'est donc jamais lu (famille D) : il
     tombe en L2 (`5e-1`), L5 (`1.5e2`) ou L6 (`1e3`).
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
  (`'1\u00a0234,50'`, `'1\u202f234.50'`, `'1\u2009234'`) ; la table ci-dessous les nomme entre parenthèses (U+00A0, …).

  | Saisie | Issue | Charge |
  |---|---|---|
  | `''` · `'   '` | vide (L1) : création sans solde ; compte existant `amount-required` (inchangé) | — |
  | `1.234` · `12.500` · `999.999` · `1.005` · `1.000` · `100.250` · `+1.234` · `01.234` · `'1.234` · `1 .234` · `1_.234` | **ambiguë — nouvellement refusée** (D4, famille A ; lues aujourd'hui 1,234 franc, 12,5, …, 1, 100,25) | — |
  | `+1,234` · `+ 1,234` · `+01,234` · `1_,234` · `1,234_` · `1,00_0` | **ambiguë — nouvellement refusée** (D4, famille B : le `+` ou le `_` masquait la virgule à l'écran d'aujourd'hui ; lues 1,234 franc, 1,234, 1) | — |
  | `1,234` · `12,500` · `100,000` · ` 9,999 ` · `01,234` · `012,345` · `1 ,234` | ambiguë (déjà refusées aujourd'hui à l'écran par `^\d+,\d{3}$` : rien de nouveau) | — |
  | `1. 234` · `1.2 5` · `1.5'0` | **non lisible — nouvellement refusée** (D4, famille C ; lues aujourd'hui 1,234, 1,25, 1,5) | — |
  | `1,23 4` | non lisible (déjà refusée aujourd'hui : après retrait, `1,234` était ambiguë) | — |
  | `1'000,000` · `1'234,567` · `1 234,567` (U+0020 et U+00A0) · `1'234.567` · `12'34,567` | claire (#1025 ; E groupée) | `1000.000`, `1234.567`, `1234.567`, `1234.567`, `1234.567` |
  | `0,500` · `0.500` · `0,123` · `0.999` · `.500` · `,500` · `00,500` | claire (#1025 ; E nulle ou vide ; `00,500` refusée aujourd'hui, C-e350-32) | `0.500`, `0.500`, `0.123`, `0.999`, `0.500`, `0.500`, `00.500` |
  | `1234,567` · `1234.567` · `1000.000` | claire (E de quatre chiffres) | `1234.567`, `1234.567`, `1000.000` |
  | `1 234,50` (U+0020) · `1 234,50` (U+00A0) · `1 234.50` (U+202F) · `1'234.50` · `1'234,50` | claire | `1234.50` |
  | `12.50` · `12,5` · `1,23` · `1,2345` · `1234` · `1.2345` · `1.2340` · `75,25` · `+12.50` | claire | `12.50`, `12.5`, `1.23`, `1.2345`, `1234`, `1.2345`, `1.2340`, `75.25`, `12.50` |
  | `1'2345` · `1234'567` · `1  234` · `1<TAB>234` · `1 234` (U+2009) · `12'34` | claire (acceptées aujourd'hui, même valeur) | `12345`, `1234567`, `1234`, `1234`, `1234`, `1234` |
  | `1 2.345` · `12'3.456` | claire (E groupée de moins de quatre chiffres : seul le groupement l'écarte de la règle ; acceptées aujourd'hui, même valeur ; ligne ajoutée au développement, M8) | `12.345`, `123.456` |
  | `1_000` · `1__0` · `1.2_5` · `1_` · `1_.5` | claire (#1028 ; acceptées aujourd'hui par le serveur, même valeur) | `1000`, `10`, `1.25`, `1`, `1.5` |
  | `.5` · `,5` · `5.` | claire | `0.5`, `0.5`, `5` |
  | `1.50000` · `1.23450` | claire (zéros au-delà de la quatrième décimale) | `1.5000`, `1.2345` |
  | `0` · `0,00` · `0.000` · `+0` · `-0` · `-0.00` | claire, montant nul (#1027 : l'avis « aucune écriture ») | `0`, `0.00`, `0.000`, `0`, `0`, `0.00` |
  | `-5` · `-0,5` · `-1.234` | négative : `amount-required` (refusées aujourd'hui par le serveur) | — |
  | `1.23456` · `-1.23456` · `1.234.567` · `1,234,567` · `1,234.56` · `1.234,56` · `_1` · `+_5` · `١٢٣` · `abc` · `1.2.3` · `+` · `.` · `1-2` · `1+2` · `++1` | non lisible (toutes refusées aujourd'hui, par le serveur : rien de nouveau) | — |
  | `1e3` · `1E3` · `1,5e2` · `5e-1` | **non lisible — nouvellement refusée, à l'écran seulement** (D4, famille D, C-e350-34 ; lues aujourd'hui 1000, 1000, 150, 0,5 ; le serveur les accepte toujours : `serverAccepts`) | — |
  | `0.` suivi de 28 zéros et d'un `1` (29 décimales) | **non lisible — nouvellement refusée** (D4, famille E, C-e350-35 ; le serveur l'arrondit aujourd'hui à 0 : `serverAccepts`) | — |
  | `'+5` · `''+5` · `'-0` · `1.5'` | claire (L1 rogne les apostrophes ; acceptées aujourd'hui, même valeur) | `5`, `5`, `0`, `1.5` |
  | `1234567890123456` (seize chiffres) | claire à l'écran ; refusée par le serveur (borne), comme aujourd'hui | `1234567890123456` |

- **D4 — Nouvellement refusées : cinq familles, par règle** (C-e350-30, -32, -34, -35), non une liste finie. Justification
  commune à A, B, C : Kesh les lit aujourd'hui avec une valeur que l'utilisateur n'a peut-être pas voulue (mille fois plus
  petite pour une saisie à l'allemande ou à l'anglaise), sans signal. D et E ont leur propre justification, ci-dessous.
  - **A — un point suivi de trois chiffres après une partie entière courte** : après L1 à L3, la partie entière sans
    séparateur de groupe entre deux chiffres, et `^0*[1-9][0-9]{0,2}\.[0-9]{3}$` sur ses chiffres, le point et la fraction.
    Exemples : `1.234`, `1.000`, `100.250`, `999.999`, `+1.234`, `01.234`. **Bornes, claires** : `1000.000` (quatre chiffres),
    `0.999` et `0.500` (partie entière nulle : le zéro seul n'est pas un zéro de tête), `1'234.567` (groupée), `1.2340`, `1.23`.
  - **B — la même règle avec la virgule, que l'écran d'aujourd'hui ne voyait pas à cause d'un `+` ou d'un `_`** (`+1,234`,
    `+ 1,234`, `+01,234`, `1_,234`, `1,234_`, `1,00_0`) ; sans eux, la virgule était déjà refusée.
  - **C — un blanc ou une apostrophe après le séparateur décimal**, avant le dernier chiffre (`1. 234`, `1.2 5`, `1.5'0`) :
    non lisibles. Une apostrophe ou un blanc **en queue** est rogné (L1) : `1.5'` reste claire.
  - **D — la notation à exposant** : toute saisie qui porte un `e` ou un `E` (`1e3`, `1E3`, `1,5e2`, `5e-1`), non lisible
    **à l'écran seulement** ; le serveur, inchangé, continue de l'accepter pour un client d'API (C-e350-34, décision de
    l'orchestrateur). **Écart assumé au principe C-e350-19** : la valeur lue n'est pas fausse, mais une telle saisie est
    improbable à l'écran d'un titulaire suisse, l'aperçu ne l'a jamais montrée, et l'accepter ferait passer `1.234e0` à côté
    de la règle d'ambiguïté.
  - **E — plus de 28 décimales** dont l'une, au-delà de la quatrième, n'est pas nulle (`0.` suivi de 28 zéros et d'un `1`) :
    non lisible ; le serveur l'arrondit aujourd'hui (`maybe_round`) et l'accepte. Limite assumée, pathologique (C-e350-35).

  Le CHANGELOG et le manuel annoncent **la règle** (point comme virgule), avec `1.234` et `1.000` en exemples et `1000.000`,
  `0.500` en bornes, puis les familles C et D (E n'est dite qu'au CHANGELOG, d'une phrase) (AC6). Les saisies citées aux rouges de départ (« Tests à écrire ») ne sont **que** des rouges, non
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
    `1.234` et `+1,234` sont refusés comme ambigus (règle de D4, familles A et B) ; un blanc ou une apostrophe après la
    décimale (famille C) et plus de 28 décimales (famille E) sont refusés comme illisibles. L'API garde sa lecture : un client qui envoie `"1.234"`
    écrit toujours 1,234 franc, ce qui est le sens d'un nombre décimal, et l'API ne lit pas la virgule. La notation à
    exposant (`1e3`) est désormais refusée à l'écran ; l'API l'accepte toujours. Les autres écrans de
    saisie d'un montant sont suivis à #1050 et #1051. »*
  - **#1028** fermée (`closes #1028`) — motif : *« L'écran lit `1_000` comme le serveur (1000) et l'affiche juste, jamais
    « NaN CHF ». Écran et serveur ne partagent pas pour autant une grammaire : l'écran retire les apostrophes et les blancs de
    la partie entière, que le serveur refuse, et refuse `1.234`, que le serveur lit 1,234 ; mais ce qui part au serveur est
    une charge qu'il lit à la valeur que l'écran a montrée (test de parité, C-e350-33). »*

## Critères d'acceptation

1. **AC1 — La table de D3**, écrite **une fois** dans la fixture `frontend/src/lib/features/bank-accounts/opening-amount-cases.json`
   (`{ raw, kind, payload?, serverRejects?, serverAccepts? }` par saisie ; espaces spéciales par échappement JSON `\u00a0`,
   `\u202f`, `\u2009`, `\t`), et lue par `format.test.ts` et par le test de parité (AC8). Une seule fonction de classement,
   **`openingAmountKind(raw) → { kind: 'empty' | 'ambiguous' | 'clear' | 'unreadable' | 'negative', payload?: string }`**,
   suit le pipeline de D2 ; `openingAmountPayload` (D5), `openingAmountPreview` s'y adossent, et `openingAmountAmbiguous`
   disparaît ou s'y adosse. Chaque ligne : son issue **et sa charge** ; les bornes de D4 (`1.000`, `100.250` ambiguës ;
   `1000.000`, `0.999` claires) y sont. Chaque charge claire suit `^[0-9]+(\.[0-9]{1,4})?$` ; la ligne des seize chiffres porte
   `serverRejects: true` (borne du serveur, non de l'écran).
2. **AC2 — L'écran refuse** une saisie ambiguë, non lisible ou négative, **dans les deux formulaires** (création :
   `formError` ; compte existant : `openingError`), avec le message de D6, sans appel au serveur. À la création, le
   classement n'est joué **que si un compte lié est choisi**, comme aujourd'hui ; compte lié et montant vide →
   `createBankAccount` appelé sans `openingBalance` (Vitest de la page).
3. **AC3 — L'aperçu** : présent pour une saisie claire (`1_000` → `formatChfBalance('1000')`, « 1 000.00 CHF » à l'espace fine U+202F, non une apostrophe ; `1\u202f234.50` ; `0.500`) ;
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
     apostrophe après le séparateur décimal est refusé ; la notation à exposant (`1e3`) est refusée à l'écran (famille D) ;
     `1_000` lu 1000 ; les renvois « (\#1026) » et « (\#1025) » retirés. PDF régénéré ; contrôle au PDF aplati : **absence**
     de « (#1026) », « (#1025) » (sans barre oblique), « toujours lu ainsi » et « écrivez-les avec un point » (les deux phrases
     devenues fausses), **présence** de « 1.2340 » et de « 1000.000 » (sondes positives) ; greper des mots sans
     `ff`/`fi`/`fl` (ligatures).
   - CHANGELOG `## [0.17.7] — Non publié`, *Corrigé*, un point par issue (C-e350-8) ; sous #1026, **la règle de D4** (familles
     A à E), avec ses exemples et ses bornes, non une liste ; la famille D dite « refusée à l'écran, toujours acceptée par
     l'API ».
   - `docs/api-external.md` : **inchangé** (le serveur ne change pas).
7. **AC7 — Captures** (thème clair, base jetable, `kesh-gate-logs/e356/captures/`) : `1.234` refusé ; `1_000` accepté avec
   son aperçu ; `1'234,567` accepté avec son aperçu.
8. **AC8 — Parité écran/serveur** (C-e350-33, test seul, `kesh-api`) : dans le `mod tests` de
   `crates/kesh-api/src/routes/opening_balances.rs` (`:630`), un test lit la fixture d'AC1 **à l'exécution**
   (`env!("CARGO_MANIFEST_DIR")` puis `std::fs::read_to_string`, pour ne pas lier la compilation au frontend) et vérifie :
   (a) pour chaque ligne `clear` sans `serverRejects`, `parse_complement_amount(Some(payload))` réussit et vaut
   `Decimal::from_str(payload)` (l'acceptation de la charge ; la parité elle-même est portée par (c)) ; (b) pour la ligne
   `serverRejects`, le refus ; (c) pour chaque ligne `clear` **sans `serverRejects`** dont la saisie brute n'emploie que
   l'alphabet du serveur (`[0-9._+-]`, rognée des seuls blancs, comme `str::trim` ; une apostrophe sort de l'alphabet ; par exemple `1_000`, `1__0`, `1.2_5`, `1_`, `1_.5`, `-0`, `+12.50`, `.5`, `5.`,
   `1.50000`), `parse_complement_amount(Some(raw))` vaut la même valeur que la charge ; (d) pour chaque ligne `unreadable` ou
   `negative` de cet alphabet **sans `serverAccepts`** (par exemple `_1`, `+_5`, `1+2`, `++1`, `1.23456`, `1.2.3`, `+`, `.`,
   `1-2`, `-5`), le refus du serveur ; (e) pour chaque ligne `serverAccepts` dont la saisie brute n'emploie que
   `[0-9._+eE-]` (familles D et E, `1,5e2` exclue par sa virgule), l'acceptation par le serveur et sa valeur, **épinglée**
   (`1e3` → 1000, `1E3` → 1000, `5e-1` → 0,5, la ligne E → 0) : une montée de `rust_decimal` qui changerait cette lecture
   rougit. Nombres attendus, sur la table de la validation P4 : (c) 24, (d) 13, (e) 4 — recomptés au développement et
   affirmés en dur. Aucune ligne de production du
   serveur ne change.

## Tests à écrire, et la preuve du rouge

`format.test.ts` (AC1, AC3) et Vitest de la page (AC2, AC4) — écrits d'abord, journal `kesh-gate-logs/e356/rouges.log`.
Pour que chaque rouge prouve un comportement et non une absence d'export, `openingAmountKind` est d'abord écrit comme
**adaptateur de la lecture d'aujourd'hui** (`ambiguous` si `openingAmountAmbiguous(raw)`, `empty` si
`openingAmountPayload(raw) === null`, sinon `clear` avec `openingAmountPayload(raw)`) ; les rouges se consignent contre lui,
ligne par ligne, puis il est remplacé par le pipeline de D2.
Rouges au départ : `1.234`, `1.000`, `01.234`, `+1,234` et `1. 234` clairs, `1'234,567` et `0,500` ambigus, `1_000` sans aperçu (« NaN » à l'avis), `-0`
sans l'avis du montant nul, les lignes « non lisible » envoyées au serveur au lieu d'être refusées à l'écran. Verts au
départ, et dits comme tels : le vide (AC2), les lignes claires d'aujourd'hui qui gardent leur valeur. Le test de parité
(AC8) est vert au départ (le serveur ne change pas) : sa preuve est une mutation (M17). Tests existants, liste complète
(validation P3, relue sur `release/0.17.6`) :
- `format.test.ts:154-156` (`' 12 000 '` → `12000`, `'   '` et `''` → `null`) : inchangés, verts (D5) ;
- `:160-167` : `1'000,000` (`:162`) passe d'ambigu à clair ; `1.234` (`:165`) de clair à ambigu ; les autres entrées de `:165`
  (`'1,23'`, `'1,2345'`, `"1'234,50"`, `'1234'`, `''`, `'1,2'`) restent non ambiguës — si `openingAmountAmbiguous` disparaît,
  ces deux boucles sont ré-adossées à `openingAmountKind` (même attendu, `kind !== 'ambiguous'`) ;
- `:170-177` (aperçu `null` pour `''`, `'1,234'`, `'abc'`, `'-5'`, `'1.00001'`, `'1.2.3'`) : inchangé, vert au pipeline ;
- `:181` (`1.005`) : la preuve des quatre décimales passe à `1.0050` ;
- `bank-accounts-page.test.ts:924` (`1,234` à la création) et `:998` (`12,500`, compte existant) : leur attendu « Montant
  ambigu » reste vrai (D6), cités pour mémoire ;
- E2E : aucun touché (`bank-account-opening-balance.spec.ts` n'emploie que `1'234.50` et `500`).

**Mutations** : M1 la règle du point retirée (`1.234` redevient clair) ; M2 `0,500` redevient ambigu ; M3 le `+` en tête
contourne la règle ; M4 l'avis formate la chaîne brute ; M5 `_` retiré de l'ensemble des séparateurs (`1_000` non lisible,
AC1) ; M6 la saisie non lisible part au serveur — sur **chacun** des deux formulaires ; M7 la borne « 1 à 3 chiffres » retirée
(`1234.567` devient ambigu) ; M8 le classement fait après le retrait des séparateurs (`1 2.345` et `12'3.456` deviennent ambigus ; *corrigé au développement : `1'234,567`, annoncé ici, garde ses quatre chiffres et reste clair sous la borne de un à trois*) ; M9
l'ensemble des séparateurs réduit à l'espace ASCII et aux apostrophes (U+00A0, U+202F, tabulation : AC1) ; M10 le vide
classé non lisible (AC2, création sans solde) ; M11 le rognage retiré (` 9,999 ` cesse d'être ambiguë) ; M12 les zéros
au-delà de la quatrième décimale non retirés (`1.50000` devient non lisible) ; M13 un `+` hors de la tête accepté (`1+2`) ;
M14 un `_` avant le premier chiffre accepté (`_1`, `+_5`) ; M15 un séparateur de groupe dans la fraction retiré au lieu de
refusé (`1. 234` devient ambiguë ou claire) ; M16 les zéros de tête comptés (`0,500`, `00,500`, `0.999` deviennent
ambiguës) ; M16-bis le `0*` retiré du motif (`01,234` devient claire) ; M17 (AC8) la charge de `1_000` portée à `100` dans
la fixture : l'assertion (c) du test de parité rougit ; M18 l'exposant lu (`1e3` devient claire, famille D) ; M19 les
apostrophes de tête non rognées (`'+5` devient non lisible). Le test de parité affirme en dur le nombre de lignes
qu'exerce chacune de ses assertions (a) à (e), pour qu'un filtre d'alphabet trop étroit ne le fasse pas
passer à vide.

## Tâches

- [x] Étape 0 : relire les lignes citées sur `release/0.17.6`.
- [x] Tests, rouges consignés ; fixture de la table ; `format.ts`, page, clés ×4, gardes i18n ; test de parité (AC8) ;
      manuel, CHANGELOG ; captures.
- [x] Gates : frontend complet ; backend complet (`test-fast.sh --ci` : les catalogues changent) ; E2E complet au dernier
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

- Aucune saisie acceptée aujourd'hui n'est refusée, hors ce que L7 classe ambigu (familles A et B) et les familles C, D et E
  de D4 ; aucune ne change de valeur. Le sens présélectionné, l'aperçu du montant nul (#1027) et les refus et acceptations du
  serveur ne changent pas (la famille D reste acceptée par l'API). Fixtures synthétiques seulement.

### Références

- Issues #1026, #1028, #1025 ; #1050, #1051 ; Story 34-2 (#533) ; validations P1 et P2 `kesh-gate-logs/e35-6-validation-p*.md`.
- `CLAUDE.md` : § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel* ; § *Règle de splitting préventif*.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`claude-opus-5-5`), workflow `bmad-dev-story`, worktree `kesh-e356`, branche `story/35-6-solde-ouverture-saisie`
(partie de `epic-35/base2`), 2026-11-12/13.

### Debug Log References

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e356/` : `rouges.log` (et `rouges-brut.log`), `verts-cible.log`,
`mutations.log`, `front-check.log`, `front-test-unit.log`, `front-build.log`, `make-fr.log`, `test-fast-ci.log`,
`gate-ci.out` et `gate-ci-journal.log`, `e2e-r1.log` ; captures dans `captures/`.

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `release/0.17.6`).
- **Étape 0** : lignes relues sur la base de la branche (`epic-35/base2` = la 0.17.6 publiée + les fiches) — `format.ts:236-288`
  (`openingAmountPayload`, `THOUSANDS_SEPARATORS`, `openingAmountAmbiguous`, `openingAmountPreview`, `formatChfExact`),
  page `:253-258` (message d'ambiguïté), `:277-288` (compte existant), `:456` (garde de la création), `:482-485`, `:499`,
  `:510`, `:718-729` (aperçu), `format.test.ts:141-185`, `bank-accounts-page.test.ts:917-998`,
  `opening_balances.rs:515-530` et `:630` (`mod tests`), `messages.ftl` fr-CH `:1713`, `user-manual.tex:2456`. Toutes
  conformes à la fiche.
- **Rouges** (`rouges.log`) : la table écrite d'abord dans `opening-amount-cases.json`, `openingAmountKind` d'abord
  adaptateur de la lecture d'aujourd'hui ; **96 rouges sur 229 tests** des deux fichiers (`format.test.ts`,
  `bank-accounts-page.test.ts`), dont toutes les lignes annoncées par la fiche (`1.234`, `1.000`, `01.234`, `+1,234`,
  `1. 234` clairs ; `1'234,567`, `0,500` ambigus ; `1_000` sans aperçu ; `-0` sans l'avis nul ; les non lisibles envoyées) ;
  verts au départ et dits tels : les deux vides, les ambiguës déjà refusées, 29 lignes claires de même valeur (liste au
  journal). Puis le pipeline D2 : **toutes les lignes vertes au premier essai** ; aucune ligne de la table ne donnait autre
  chose que l'issue annoncée.
- **AC8** : test de parité `solde_ouverture_parite_ecran_serveur` dans le `mod tests` de `opening_balances.rs`, fixture lue
  à l'exécution ; lignes exercées affirmées en dur **(a) 58, (b) 1, (c) 24, (d) 13, (e) 4** — (c), (d), (e) égaux aux
  nombres de la fiche ; (a) 56 au premier passage, 58 après l'ajout des deux lignes de M8 (hors alphabet du serveur).
  Valeurs épinglées confirmées : `1e3`, `1E3` → 1000, `5e-1` → 0,5, la ligne E → 0 ; `-0`, `-0.00` acceptés à zéro.
- **Mutations** (`mutations.log`) : M1 à M19 et M16-bis jouées, M4 et M6 sur **chacun** des deux formulaires, M17 côté
  Vitest et côté Rust — **toutes tuées**, M8 après correction de la table (voir Change Log). Lanceur : motif vérifié unique,
  restauration `git checkout --` puis `touch`.
- **Captures** (thème clair, base jetable `kesh_e356` remise au preset de test `with-company`, compte 1020 créé par l'API) :
  `35-6-1-1_000-apercu.png` (« Montant qui sera écrit : 1 000.00 CHF », espace fine), `35-6-2-1234,567-apercu.png`
  (« 1 234.567 CHF »), `35-6-3-1.234-refuse.png` (message d'ambiguïté), `35-6-4-illisible-refuse.png` (`1. 234`).
- **Manuel** : `user-manual.tex:2456` réécrit et scindé en deux paragraphes ; PDF régénéré (`make fr`), contrôlé aplati :
  absences de « (#1026) », « (#1025) », « toujours lu ainsi », « écrivez-les avec un point » (0 chacune) ; présences de
  « 1.2340 », « 1000.000 », « 1_000 », « 1e3 », « 1.23456 » (1 chacune). Les PDF d'administration et de la brochure,
  régénérés sans changement de source, ne sont pas commités (C-e356-5). `docs/api-external.md` inchangé.
- **i18n** : clé d'ambiguïté réécrite et `bank-accounts-opening-error-amount-unreadable` ajoutée dans les quatre
  catalogues, à côté de `bank-accounts-opening-*` ; replis littéraux égaux au fr-CH ; `sitesTotal` 3026 → 3027, recompté
  par `grep -o 'i18nMsg('` (page 112 → 113) et par la garde. `openingAmountAmbiguous` retiré (plus aucun appelant).

### File List

- `frontend/src/lib/features/bank-accounts/opening-amount-cases.json` (neuf)
- `frontend/src/lib/features/bank-accounts/format.ts`
- `frontend/src/lib/features/bank-accounts/format.test.ts`
- `frontend/src/routes/(app)/bank-accounts/+page.svelte`
- `frontend/src/routes/(app)/bank-accounts/bank-accounts-page.test.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `crates/kesh-api/src/routes/opening_balances.rs` (`mod tests` seulement)
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`
- `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/35-6-solde-ouverture-saisie.md`
- `_bmad-output/implementation-artifacts/e35-choix-autonomes-e356.md` (neuf)
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

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
- 2026-11-12 — **Validation P4** (Opus) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 9 LOW (`kesh-gate-logs/e35-6-validation-p4.md`) ; série
  5 → 5 → 3 → 1 MEDIUM. Remédiée par l'agent de relève (Opus 5.5). Prémisses relues : `rust_decimal` 1.43.0
  `decimal.rs:2108-2117` (`from_str` se rabat sur `from_scientific_lossy` quand la saisie porte `e`/`E`), `str.rs:244`, `:478`
  (`+` seul refusé) — confirmées.
  - V4-1 (MEDIUM), **décision de l'orchestrateur** (C-e350-34) : la notation à exposant devient la **famille D**,
    nouvellement refusée à l'écran seulement ; le serveur l'accepte toujours. Écart assumé à C-e350-19, motivé. Corrigés :
    « Ce qui se lit au code » (le serveur lit l'exposant), la ligne des non lisibles (`1e3` en sort), « Ce qui doit rester
    vrai » ; ligne de table propre (`1e3`, `1E3`, `1,5e2`, `5e-1`) ; AC6 (manuel, CHANGELOG) ; AC8 (e) épingle la lecture du
    serveur ; mutation M18 ; motif de fermeture de #1026 complété.
  - V4-2 : famille B élargie au `_` (`1_,234`, `1,234_`, `1,00_0`) ; L1 rogne aussi les apostrophes de bord (`'+5`, `''+5`,
    `'-0`, `1.5'` restent claires, M19) ; **famille E**, plus de 28 décimales, limite assumée (C-e350-35).
  - LOW : V4-3 (`+` et `.` seuls rangés aux refus du serveur) ; V4-4 (AC8 : « par exemple », sans `serverRejects` en (c),
    sans `serverAccepts` en (d), nombres attendus 24, 13 et 4, champ `serverAccepts`) ; V4-5 (AC3 : `formatChfBalance('1000')`,
    espace fine U+202F) ; V4-6 (M16 récrite, M16-bis) ; V4-7 (échappements écrits en clair, `\u00a0`, `\u202f`, `\u2009`) ;
    V4-8 (sondes d'absence « toujours lu ainsi », « écrivez-les avec un point ») ; V4-9 (adaptateur de la lecture
    d'aujourd'hui avant le pipeline ; `format.test.ts:170-177` cité). V4-10 (fiche 35-5a, autre branche) : **non traité
    ici**, transmis par l'orchestrateur.
  - **Écart à la consigne Edit** : trois lignes portant des espaces spéciales brutes (U+00A0, U+202F, U+2009 : D3 `:92`, AC1
    `:158-159`, AC3) n'ont pas pu être visées par l'outil Edit, qui ne retrouvait pas la chaîne malgré la copie exacte ; elles
    ont été remplacées par un court script Python qui contrôle le début de chaque ligne avant de l'écrire (artefact
    `_bmad-output`, non du code). L'outil manque d'une façon de viser une espace insécable ; en sens inverse, il convertit
    en caractère une séquence d'échappement Unicode écrite dans le texte de remplacement (constaté sur la ligne V4-7 de
    cette entrée, corrigée par le même script avant le commit).
  - Symptômes greppés (fiche, registre, fichier d'epic) : `1e3`, « trois familles », « A, B, C », espaces spéciales brutes
    (aucune ne reste) ; registre C-e350-30 annoté ; entrées passées du Change Log et du registre laissées telles quelles.
- 2026-11-12 — **Validation P5, passe ciblée** (Sonnet) sur la remédiation P4 : 0 finding au-dessus de LOW, 3 LOW
  (`kesh-gate-logs/e35-6-validation-p5-ciblee.md`). Appliqués : V5-1 (normalisation à `opening_balances.rs:526`, non `:524`) ;
  V5-2 (AC8 (c) : « rognée des seuls blancs ; une apostrophe sort de l'alphabet ») ; V5-3 (mutations dans l'ordre M17, M18,
  M19 ; « (a) à (e) » ; familles C et E nommées au motif de fermeture de #1026). Rédaction seule, aucune règle changée.
- 2026-11-12 — **Validation terminée.** Série des findings au-dessus de LOW : P1 5 MEDIUM → P2 5 → P3 3 → P4 1 → P5 0
  (CRITICAL et HIGH : 0 à chaque passe). Modèles : Sonnet (P1), Opus (P2), Sonnet (P3), Opus (P4), Sonnet en passe ciblée
  (P5). Le **signal de découpage** levé en P2 (5 → 5, recyclage) a été traité par **réduction** plutôt que par découpage
  (C-e350-19 : ce qui est accepté aujourd'hui reste accepté ; serveur non touché ; story de l'écran seul), à déclarer à Guy
  par l'orchestrateur. La boucle se clôt sur une passe sans finding au-dessus de LOW ; la remédiation de P5 ne touche que la
  rédaction. Choix retenus : **C-e350-19**, **C-e350-30** (pipeline et règle), **C-e350-31** (souligné), **C-e350-32** (zéros
  de tête), **C-e350-33** (test de parité), **C-e350-34** (exposant, écran seul ; décision de l'orchestrateur), **C-e350-35**
  (souligné devant la virgule, apostrophes de bord, plus de 28 décimales). Statut : `ready-for-dev`.
- 2026-11-12/13 — **Développement** (Opus 5.5, `bmad-dev-story`, worktree `kesh-e356`). Commits `1adf413c` (table en
  fixture, 96 rouges contre la lecture d'aujourd'hui), `b09b5f9f` (pipeline D2, page, clés ×4, `sitesTotal`, test de
  parité), `8b83e65f` (manuel et CHANGELOG), `d64520db` (lignes de M8), puis ce compte rendu.
  - **Fiche corrigée au développement — M8.** La mutation M8 (« classement après le retrait des séparateurs ») **survivait**
    (229 tests verts) : la fiche annonçait que `1'234,567` redeviendrait ambigu, ce que la borne « un à trois chiffres »
    (posée en P3, C-e350-30) exclut — `1234` a quatre chiffres, groupé ou non. Aucune ligne de la table ne faisait dépendre
    l'issue du seul groupement. Ajoutées à D3 et à la fixture : `1 2.345` → `12.345`, `12'3.456` → `123.456`, claires,
    **acceptées aujourd'hui à la même valeur** (C-e350-19 tenu) ; M8 récrite (`1 2.345`, `12'3.456` deviennent ambiguës)
    et désormais tuée (4 rouges) ; (a) du test de parité 56 → 58. Corollaire, LOW, non traité : D4 cite `1'234.567` comme
    borne « groupée » de la famille A, alors qu'il est clair par ses quatre chiffres déjà.
  - Aucune autre ligne de la table n'a donné une autre issue que l'annoncée ; les nombres (c) 24, (d) 13, (e) 4 de AC8
    sont ceux du code.
  - **Gates au dernier commit de code `d64520db`** : `scripts/test-fast.sh --ci` vert, **5308 exécutés, 0 échec, 13
    ignorés** (5307 à la 0.17.6 + le test de parité) ; `scripts/gate-ci.sh kesh_ci_e356` vert, **5315 passés, 0 échoué,
    17 ignorés** ; frontend : `npm run check` 0 erreur (29 avertissements, aucun dans les fichiers de la story),
    `lint-i18n-ownership` vert, `test:unit` **3325 tests, 232 fichiers**, `npm run build` vert ; **E2E complet**, recette
    entière (base `kesh_e2e_e356` recréée, port 3406, dossiers sous `target/e2e-r1/`) : **403 passés, 7 échecs, 18 ignorés**
    — les sept de la KF-029 (#97), nommés contre `docs/testing.md` (`mode-expert.spec.ts:26`, `:41`,
    `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) ; `bank-account-opening-balance.spec.ts`
    vert (deux tests).
  - **Écart à la consigne Edit** (deux fois, une fixture et deux lignes de code) : (1) la fixture écrite par l'outil Write
    a vu ses échappements Unicode (U+00A0, U+202F, U+2009, U+0661 à U+0663) changés en caractères bruts ; un court script Python
    les a rétablis (contrôle du nombre de chaque caractère) ; (2) l'outil Edit ne retrouvait pas les deux lignes de
    `format.ts` portant U+00A0 et U+202F bruts (`:237`, `:242`), même copiées exactement — même constat qu'en validation
    P4 ; un script a remplacé ces deux seules lignes, après contrôle de leur contenu exact, par un marqueur ASCII, que
    l'outil Edit a ensuite remplacé avec le reste. L'outil manque d'une façon de viser une espace insécable, et convertit
    les échappements Unicode du texte écrit.
  - Choix : registre `e35-choix-autonomes-e356.md`, **C-e356-1 à C-e356-6**.
  - Statut : `review`.
- **2026-11-13 — Revue de code, close en P1** (prompt versionné `35-6-solde-ouverture-saisie-review-prompt-p1.md`, `f72fbd42` ;
  diff aplati `1ebe1240..d64520db` ; rapports hors dépôt `kesh-gate-logs/e356-review-p1-{B,E,A}.md`).
  - Trend : **P1 (Opus, trois lentilles) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 8 LOW** → boucle close (aucune remédiation, donc
    aucune passe ciblée à relire).
  - Axes : tous les huit exercés par au moins une lentille (déclarés par rapport) ; l'orchestrateur a relu lui-même la fidélité
    des traductions (axe 5, exercé en A par le seul recompte) : les quatre locales disent la même règle et la même forme à écrire.
  - LOW, une issue par problème : #1069 (`1,5e2` et `serverAccepts`), #1070 (saut de ligne et U+FEFF lus comme séparateurs),
    #1071 (retour silencieux inatteignable, à fermer par le type), #1072 (erreur qui reste après correction), #1073 (saisies
    absentes de la fixture), #1074 (manuel et CHANGELOG : apostrophe de queue), #1075 (`verts-cible.log` porte un rouge),
    #1076 (`1'234,567` donné pour exemple de groupement). LOW du développement : #1068.
  - Écart de la lentille : une redirection `> /dev/null` en fin de pipeline `pdftotext`, sans fichier créé, déclarée et rejouée
    sans redirection.
  - Statut : `done`.
