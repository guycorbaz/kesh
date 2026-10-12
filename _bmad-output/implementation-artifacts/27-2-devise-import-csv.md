# Story 27.2 : La devise d'un relevé CSV est lue, et le contrôle de devise joue

## Status

**ready-for-dev** — fiche créée le 2026-10-12 en autonomie (skill `bmad-create-story`), sur `a2743fd4`
(`origin/main`, rétrospective de l'Epic 26). Validation P1 faite (Sonnet, lentilles R et F : 3 MEDIUM, 14 LOW),
remédiée le 2026-10-12 — périmètre élargi par l'orchestrateur au formulaire qui perd la contrepartie (AC8-bis) et à
la devise des transactions CAMT.053 (AC19). Validation P2 à lancer : prompt `27-2-validate-prompt-p2.md`. Choix
tranchés : **C-27-2-1 à C-27-2-14**, registre `_bmad-output/implementation-artifacts/epic-27-choix-autonomes-27-2.md`
(à fusionner par union dans `epic-27-choix-autonomes.md`, qui vit sur la branche de la 27-1).

**Issue** : [#624](https://github.com/guycorbaz/kesh/issues/624) (**P1**, `bug` — montant faux en comptabilité).
**PR : `closes #624`** — texte prescrit au § *Pour la PR* (AC12). Epic 27 (recette de la v0.13.1, cible v0.13.2) : le Project Lead a décidé le 2026-10-12 que la
v0.13.2 fermera aussi #624, #626 et #634, pour que le nombre de bugs baisse.

## Story

**As a** utilisateur qui importe le relevé CSV d'un compte bancaire,
**I want** que Kesh lise la devise que le fichier porte, et refuse un relevé qui n'est pas en francs suisses,
**so that** les montants d'un compte en euros ou en dollars n'entrent jamais en comptabilité comme des francs, sans un
mot — exactement comme le fait déjà l'import CAMT.053.

## Le défaut, en une phrase

Le parseur CSV écrit `"CHF"` en dur — pour le relevé (`crates/kesh-import/src/csv/parser.rs:364`) et pour chaque
transaction (`:680`) — sans lire aucune colonne ; et le contrôle `validate_currency_supported_v0_1`
(`crates/kesh-core/src/bank_imports.rs:467`), qui refuse un CAMT.053 dans une autre devise
(`routes/bank_imports.rs:961`, `:1127`), **n'est même pas appelé** sur le chemin CSV (`preview_csv` `:1609`,
`create_csv` `:1758`). Un CSV d'un compte en EUR s'importe donc en CHF, au montant nominal. *(L'issue cite les lignes
338 et 497 : elles ont bougé avec la 26-1 ; les numéros ci-dessus sont ceux de `a2743fd4`.)*

## Inventaire des sites non résolus — où la devise est fixée ou supposée sur le chemin CSV

⛔ Inventaire **clos**, refait depuis le symptôme (le littéral et les lecteurs de `currency`), pas depuis l'issue. Commandes
exécutées sur `a2743fd4` :

```text
$ grep -rnF '"CHF"' crates/kesh-import/src crates/kesh-core/src/bank_imports.rs crates/kesh-api/src/routes/bank_imports.rs \
    crates/kesh-api/src/routes/bank_profiles.rs crates/kesh-db/src/repositories/bank_imports.rs crates/kesh-db/src/repositories/bank_transactions.rs
crates/kesh-import/src/types.rs:62:    /// Code devise ISO 4217 (`"CHF"`, `"EUR"`).
crates/kesh-import/src/types.rs:137:            currency: "CHF".to_string(),        ← mod tests (l. 128)
crates/kesh-import/src/types.rs:151:            currency: "CHF".to_string(),        ← mod tests
crates/kesh-core/src/bank_imports.rs:466:/// devise rencontré pour toute devise autre que `"CHF"`.
crates/kesh-core/src/bank_imports.rs:468:    if stmt.currency != "CHF" {          ← LE contrôle (légitime)
crates/kesh-core/src/bank_imports.rs:486/505/554/864                     ← mod tests (l. 477)
crates/kesh-import/src/csv/parser.rs:364:        currency: "CHF".to_string(),   ← SITE (a) devise du relevé
crates/kesh-import/src/csv/parser.rs:680:            currency: "CHF".to_string(), ← SITE (b) devise de chaque ligne
crates/kesh-import/src/csv/parser.rs:801:        assert_eq!(stmt.currency, "CHF");  ← mod tests (l. 749)

$ grep -rnE "\.currency\b" crates/kesh-api/src/routes/bank_imports.rs crates/kesh-core/src/bank_imports.rs
crates/kesh-core/src/bank_imports.rs:71:            currency: tx.currency,        ← from_imported : recopie (pas de choix)
crates/kesh-core/src/bank_imports.rs:88:            currency: tx.currency.clone(),
crates/kesh-api/src/routes/bank_imports.rs:266:   currency: t.currency,      ← détail d'un import, lu en base
crates/kesh-api/src/routes/bank_imports.rs:1007:  currency: tx.currency.clone(), ← aperçu CAMT
crates/kesh-api/src/routes/bank_imports.rs:1031:  currency: stmt.currency.clone(), ← aperçu CAMT
crates/kesh-api/src/routes/bank_imports.rs:1239:  currency: t.currency,      ← import CAMT, recopie
crates/kesh-api/src/routes/bank_imports.rs:1730:  currency: t.currency.clone(), ← aperçu CSV, recopie
crates/kesh-api/src/routes/bank_imports.rs:1745:  currency: stmt.currency,   ← aperçu CSV, recopie
crates/kesh-api/src/routes/bank_imports.rs:1966:  currency: d.currency,      ← import CSV, recopie
```

| # | Site | Nature | Sort |
|---|---|---|---|
| (a) | `parser.rs:364` — `ImportedStatement.currency = "CHF"` | **supposition** | résolu : devise lue (AC2), ou `DEFAULT_CSV_CURRENCY` nommée si la colonne n'est pas réglée (AC4) |
| (b) | `parser.rs:680` — `ImportedTransaction.currency = "CHF"` | **supposition** | résolu : cellule de la ligne (AC2), ou la même constante (AC4) |
| (c) | `preview_csv` (`bank_imports.rs:1609-1756`) — **aucun** appel au contrôle | **contrôle absent** | résolu : AC5 (avertissement) |
| (d) | `create_csv` (`:1758-1999`) — **aucun** appel au contrôle | **contrôle absent** | résolu : AC5 (refus `422`) |
| (e) | `ColumnMapping` (`profile.rs:37-60`) — aucun champ pour la devise | **réglage absent** | résolu : AC1 |
| (f) | `BankProfileForm.svelte:56-69` — reconstruit `columnMapping` de ses seuls champs | **perte à l'édition** | résolu : `currency` porté (AC8), et toute clé que le formulaire ne montre pas — `counterparty` aujourd'hui — recopiée du profil existant (AC8-bis, C-27-2-12) |
| (g) | `bank_accounts` — aucune colonne devise (`entities/bank_account.rs:24-53`) | **supposition structurelle** (tous les comptes en CHF) | **non résolu, assumé** (C-27-2-4) : le manuel l'écrit (AC11) |
| (h) | `camt053/mod.rs:168-178` — devise de transaction prise sur **tout** `<Amt Ccy>` sous `<TxDtls>` (et, hors `<TxDtls>`, sur tout `<Amt Ccy>` sous `<Ntry>`) ; le dernier rencontré gagne | **supposition** (hors chemin CSV, même sujet) | résolu : AC19 (C-27-2-11) — confirmé au code par les deux lentilles de P1 |
| (i) | `reconciliation.rs:631`, `:2728` — garde « transaction non CHF » | défense en aval | inchangé ; devient effective pour le CSV (une ligne non CHF n'atteint plus la base) |

Les sites de **recopie** (`from_imported`, DTO d'aperçu, `NewBankTransaction`) ne décident rien : ils transportent la
valeur que (a)/(b) produisent. Aucun autre lieu du chemin CSV ne fixe de devise. *(Validation P1 : inventaire refait et
élargi par la lentille R — `kesh-seed`, `test_endpoints.rs`, `post_restore.rs`, `backup.rs`, `frontend/tests`, export
global — aucun site supplémentaire sur le chemin CSV ; `reconciliation.rs:777-780`, saut d'un candidat non CHF, est de
la famille (i).)*

## Critères d'acceptation

### Le réglage

**AC1 — `ColumnMapping.currency` (`crates/kesh-import/src/csv/profile.rs`).**

- Champ `pub currency: Option<usize>`, doc-comment « index de la colonne Devise (facultatif) ; sans elle, le relevé est
  supposé en [`DEFAULT_CSV_CURRENCY`] », attributs **exactement** ceux de `reference` / `details` / `counterparty` :
  `#[serde(skip_serializing_if = "Option::is_none", default)]`. `deny_unknown_fields` **reste** sur la struct.
- `all_indices()` le pousse sous le nom `"currency"` → la collision (`validate()`) et le hors-borne au premier
  enregistrement (`parse_csv_collect_inner`, étape 6.b) le couvrent **sans code neuf**.
- Doc-module de `profile.rs` et de `parser.rs` : la colonne Devise est citée.
- Tests (`profile.rs`, `mod tests`) : (1) collision `currency` = `date` → `ProfileMisconfigured` nommant
  `column_mapping.currency` ; (2) **un mapping sans devise se sérialise sans la clé** :
  `!serde_json::to_string(&m).unwrap().contains("currency")` — c'est ce qui garde le retour à une 0.13.1 indolore pour
  les profils qui ne lisent pas la devise (C-27-2-1) ; (3) `{"date":0,"amount":1,"currency":2}` se désérialise en
  `Some(2)` ; (4) l'ancien JSON `{"date":2,"debit_credit_split":[5,6]}` donne `currency: None`.

**AC1-bis — Aucune migration.** Le JSON `bank_profiles.column_mapping` porte la clé ; `CHECK (JSON_VALID(…))` suffit.
⛔ **Une migration au diff signalerait une dérive** : pas de fichier sous `crates/kesh-db/migrations/`, pas de
changement de `crates/kesh-db/test-schema/`, pas de bump `kesh_version_min_required` ni de version Cargo, aucune
ligne à `docs/migrations-idempotence-audit.md` — P5, P6, P7 et P2-bis sans objet. Contrôle : `git diff --stat
origin/main -- crates/kesh-db/` vide.

### Le parseur (`crates/kesh-import`, publiable, zéro dépendance interne)

**AC2 — La devise est lue quand la colonne est réglée.** Dans `parse_row`, **après** le montant (ordre : longueur,
date, montant, devise — C-27-2-5) :

- cellule `trim`ée (le `get_cell` existant) ;
- vide → `CsvLineError` `EMPTY_MANDATORY_FIELD`, valeur `"currency"` ;
- exactement trois caractères ASCII alphabétiques → `to_ascii_uppercase()` (`"chf"` → `"CHF"`) ;
- autre chose → `CsvLineError` du **nouveau** code `INVALID_CURRENCY` (`CsvLineErrorCode::InvalidCurrency`), valeur
  = la cellule brute (tronquée par le mécanisme existant `MAX_CSV_LINE_ERROR_VALUE_CHARS`).
- `ImportedTransaction.currency` = la devise lue. Ces erreurs sont des erreurs de ligne **ordinaires** (rejet partiel,
  confirmable par `confirmPartialImport`).
- ⛔ **Une erreur de devise garde l'observation de débit déjà faite** (validation P1, R1-1). La devise est lue après le
  montant : quand la ligne échoue sur sa devise, `parse_row` rend `RowParse { result: Err(e), debit }` avec le `debit`
  calculé au stade du montant (`Consistent` / `Zero`), **jamais** `RowParse::err(e)`, qui le remet à
  `DebitObservation::NotRead` (`parser.rs:439-442`). Sinon un débit cohérent à cellule Devise vide sort du dénominateur
  de `whole_debit_column`, et le refus de signe conseillerait à tort de régler le sens des débits du profil — le conseil
  que C-26-1-25 interdit. Test AC13-12, mutation M18.
- `CsvLineErrorCode::InvalidCurrency` : ajouté à `all()`, `as_str()` → `"INVALID_CURRENCY"`, `i18n_key()` →
  `"bank-import-csv-errors-invalid-currency"` ; le test `all_is_complete` (`error.rs:379`) doit rester vert, et un test
  `invalid_currency_i18n_key` sur le patron de `amount_sign_mismatch_i18n_key` (`error.rs:402`) fige la clé.

**AC3 — Un relevé, une devise.** Après la boucle des lignes et **après** le refus de signe (étape 7.b, inchangée et
prioritaire) :

- si la colonne est réglée, l'ensemble des devises des transactions **valides** est calculé ; plus d'une →
  `Err(CsvError::MixedCurrencies { currencies })`, devises distinctes **triées**, **au plus 10** (constante nommée,
  documentée) — refus **non confirmable**, même s'il y a aussi des erreurs de ligne. Les 10 rapportées sont **les 10
  premières de l'ensemble complet trié** (un `BTreeSet` tronqué), pas les 10 premières rencontrées ; aucun indicateur de
  troncature (C-27-2-13) ;
- ⚠️ **Seules les lignes valides révèlent leur devise** (validation P1, R7-1/F4 ; C-27-2-14) : une ligne écartée pour sa
  date ou son montant n'atteint jamais la lecture de la devise. Un fichier CHF dont les lignes EUR sont toutes invalides
  pour une autre raison n'est donc pas refusé pour mélange : confirmé en partiel, il importe ses lignes CHF, exactes, et
  les lignes EUR restent listées comme invalides. Écrit au doc-comment de l'étape et au manuel (AC11) ;
- sinon `ImportedStatement.currency` = cette devise unique ; s'il n'y a **aucune** transaction valide (rejet partiel à
  zéro ligne valide), `DEFAULT_CSV_CURRENCY` (l'import est de toute façon refusé, `no_valid_lines_to_commit`).
  L'invariant « chaque transaction valide porte la devise du relevé » est écrit au doc-comment de la construction du
  relevé et vérifié par AC13-1, -2, -5 et -9, qui assertent **chaque** `tx.currency` (validation P1, F7 ; pas de
  `debug_assert!`, absent en release).
- `CsvError::MixedCurrencies` : doc-comment sur le patron de `AmountSignMismatch` (pourquoi non confirmable :
  importer les seules lignes d'une devise ferait mentir le solde sans un mot) ; message `thiserror` en français.
- Doc-module de `parser.rs` (le pipeline numéroté) : étapes mises à jour.

**AC4 — Sans colonne réglée : la supposition est nommée.** Nouvelle constante publique
`pub const DEFAULT_CSV_CURRENCY: &str = "CHF";` (dans `csv/parser.rs` ou `csv/profile.rs`, réexportée par
`csv/mod.rs` et `lib.rs`), doc-comment : « devise supposée d'un relevé dont le profil ne lit aucune colonne Devise ».
Les deux littéraux de (a) et (b) disparaissent au profit de cette constante. Contrôle :
`grep -nF '"CHF"' crates/kesh-import/src/csv/parser.rs` ne rend plus que des lignes de `mod tests`.

### L'API (`crates/kesh-api`)

**AC5 — Le contrôle joue sur le chemin CSV (`routes/bank_imports.rs`).**

- `preview_csv` : après l'obtention du relevé (`AllValid` ou `PartialFailure.valid`, y compris après le second essai
  `EncodingMismatch`), `core_bank_imports::validate_currency_supported_v0_1(&stmt)` ; `Err(BankImportUnsupportedCurrency(c))`
  → `warnings.unsupported_currency = Some(UnsupportedCurrencyPayload { currency: c })` ; toute autre variante →
  `AppError::Internal`, **sur le patron exact** du chemin CAMT (`:961-971`). **Factoriser** ce bloc (CAMT + CSV) dans
  un helper plutôt que le copier (règle DRY).
- `create_csv` : même appel sur le relevé, **avant** le test de `confirm_partial_import` — appelé **dans chacun des
  deux bras** du `match` (`AllValid`, et `PartialFailure` sur `valid` **avant** `if !fields.confirm_partial_import`,
  `:1825`) ; un appel unique placé après le `match` ne convient pas (validation P1, R5-1) → `Err(AppError::BankImportUnsupportedCurrency(c))`, `422 BANK_IMPORT_UNSUPPORTED_CURRENCY`,
  `details.currency`, **rien d'écrit** (la transaction est abandonnée). Le refus de fichier déjà importé
  (`BANK_IMPORT_DUPLICATE_FILE`), qui précède le parse, garde sa place.
- `map_csv_error` : bras **explicite** `CsvError::MixedCurrencies { currencies } => AppError::BankCsvMixedCurrencies
  { currencies }` (le repli `other =>` rendrait le message masqué, cf. commentaire `:654-655`).
- `crates/kesh-api/src/errors.rs` : variant `BankCsvMixedCurrencies { currencies: Vec<String> }` à côté de
  `BankCsvHeaderSkipsData` (`:806`) ; réponse `422`, `code: "BANK_CSV_MIXED_CURRENCIES"`, message
  `t_args("bank-import-csv-errors-mixed-currencies", <repli fr>, { currencies: "CHF, EUR" })`,
  `details: { currencies: [...] }` — patron de `:2395-2411`.

**AC6 — L'avertissement « devise supposée ».** `preview_csv` pousse `"bank_csv_currency_assumed"` dans
`warnings.informational` **si et seulement si** `csv_profile.column_mapping.currency.is_none()`. (`create_csv` ne
l'émet pas : le corps de `201` n'a pas d'avertissements.)

**AC7 — Routes des profils (`routes/bank_profiles.rs`).** Rien à coder : `ColumnMapping` traverse
`CreateOrUpdateBankProfileRequest` et `BankProfileResponse`. Doc-module : la colonne Devise facultative citée. Test
d'intégration (`crates/kesh-api/tests/bank_profiles_e2e.rs`) : `POST` avec `columnMapping.currency = 4` → `201`,
`GET` le rend ; collision (`currency` = `date`) → `422 BANK_CSV_PROFILE_INVALID`. L'export global
(`exports/csv_tables.rs`, colonne `column_mapping_json`) porte la clé sans code neuf ; la sauvegarde d'installation
aussi (JSON brut).

### L'écran

**AC8 — Formulaire du profil (`frontend/src/lib/features/bank-import/BankProfileForm.svelte`) et type.**

- `bank-profile.types.ts` : `ColumnMapping.currency?: number`.
- Nouveau champ numérique `col-currency-input` (`data-testid="col-currency-input"`, `min="0"`), dans la grille des
  colonnes, après « détails » ; état `currencyIdx = $state<number | null>(existing?.columnMapping.currency ?? null)` ;
  envoyé `currency: currencyIdx ?? undefined` **dans les deux branches** (séparé et colonne unique). Libellé
  `i18nMsg('bank-import-profile-labels-col-currency', 'devise (facultative)')` et aide
  `i18nMsg('bank-import-profile-labels-col-currency-help', …)` reliés au champ (`for` / `aria-describedby`) — la spec
  axe du formulaire reste à zéro violation.
- ⚠️ Un champ `number` vidé par l'utilisateur rend `null` en Svelte 5 (`to_number('') = null`,
  `node_modules/svelte/src/internal/client/dom/elements/bindings/input.js:288-289`, Svelte 5.55.1 — lu par la lentille F
  en P1 ; l'hypothèse « ou `''` » était fausse) : `null ?? undefined` donne une clé de valeur `undefined`, que
  `JSON.stringify` omet. Le test (c) l'établit.
- **Vitest** (`BankProfileForm.test.ts`, patron existant) : (a) devise 4 en mode séparé → requête
  `columnMapping.currency === 4` ; (b) idem en colonne unique ; (c) champ laissé vide →
  `expect(req.columnMapping.currency).toBeUndefined()` **et** `JSON.stringify(req)` ne contient pas `"currency"` (la clé
  est présente en JS, de valeur `undefined` : un `'currency' in …` serait vrai — validation P1, F2) ; (d) profil
  existant `currency: 4` → le champ s'ouvre à 4, et l'enregistrer sans y toucher le renvoie ; (e) profil existant sans
  devise → champ vide.

**AC8-bis — Le formulaire ne perd aucune clé qu'il ne montre pas** (validation P1, F1 ; périmètre élargi par
l'orchestrateur, C-27-2-12). Aujourd'hui `columnMapping` est reconstruit des seuls champs du formulaire
(`BankProfileForm.svelte:56-69`) et le `PUT` est un remplacement complet : un `counterparty` posé par l'API est
**perdu** au premier enregistrement à l'écran — chemin que AC11 et le CHANGELOG prescrivent justement d'emprunter pour
ajouter la colonne Devise.

- Le mapping envoyé part du mapping existant **privé des clés que le formulaire possède** (`date`, `amount`,
  `debit_credit_split`, `reference`, `details`, `currency`), puis reçoit les valeurs du formulaire : déstructuration
  `const { date, amount, debit_credit_split, reference, details, currency, ...rest } = existing?.columnMapping ?? {}`
  (ou équivalent), `{ ...rest, ...champsDuFormulaire }`, dans les **deux** branches. La liste des clés possédées est
  **une** constante nommée et documentée (JSDoc), partagée par les deux branches (DRY).
- Ce n'est pas une recopie de tout le mapping existant : `amount` et `debit_credit_split` s'excluent (`validate()`), et
  un passage de la colonne unique au mode séparé laisserait sinon les deux clés — refus `422`.
- **Vitest** : (f) profil existant `counterparty: 3`, enregistré sans y toucher, en colonne unique **et** en mode séparé
  → `req.columnMapping.counterparty === 3` ; (g) profil existant en colonne unique (`amount: 1`, `counterparty: 3`),
  passé au mode séparé → la requête porte `debit_credit_split` et `counterparty: 3`, et `amount` y est `undefined`
  (absent de `JSON.stringify(req)`) ; (h) création (pas de profil existant) → la requête ne porte que les clés du
  formulaire.

**AC9 — Aperçu d'import (`BankImportUpload.svelte`).** Rien de neuf à afficher : `warning-unsupported-currency` et
`warning-informational` existent (l. 596, 617). Seul le texte de l'avertissement change (AC10). `csv-line-error-label.ts`
gagne `INVALID_CURRENCY: ['invalid-currency', '<repli fr>']`, et **`csv-line-error-label.test.ts`** sa ligne dans la
table `attendu` : ce test compare l'ensemble des clés à l'égalité **exacte**, il rougit dès le code ajouté (validation
P1, R3-1).

### Les textes

**AC10 — Quatre locales** (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`), clés présentes dans les
**quatre** fichiers ; `frontend/src/lib/shared/i18n-keys.test.ts` (dont `sitesTotal`, l. 575, **recompté** par le
test, jamais incrémenté de confiance ; la famille `'bank-import-info-'` l. 628 et son compte l. 680 ; la famille
`'bank-import-csv-errors-'` l. 631), `i18n-un-repli-par-cle.test.ts` (`MOTIFS_DYNAMIQUES`, cf. commentaire de
`csv-line-error-label.ts`) et `npm run lint-i18n-ownership` verts.

| Clé | Rôle | Texte fr-CH (indicatif) |
|---|---|---|
| `bank-import-profile-labels-col-currency` | neuve, libellé du champ | devise (facultative) |
| `bank-import-profile-labels-col-currency-help` | neuve, aide | Colonne qui porte la devise de chaque ligne. Sans elle, le relevé est supposé en francs suisses (CHF). Relevé CSV d'UBS : 4. |
| `bank-import-csv-errors-invalid-currency` | neuve, message par ligne | Devise illisible dans une ligne CSV (trois lettres attendues, par exemple CHF). |
| `bank-import-csv-errors-mixed-currencies` | neuve, refus global, `{ $currencies }` | Le relevé porte plusieurs devises ({ $currencies }) : un relevé CSV doit être dans une seule devise. Rien n'a été importé. |
| `bank-import-info-bank-csv-currency-assumed` | neuve, information d'aperçu | Le profil ne lit aucune colonne Devise : le relevé est supposé en francs suisses (CHF). |
| `bank-import-warnings-unsupported-currency` | **modifiée** (C-27-2-6) | Devise non supportée : seul le franc suisse (CHF) est pris en charge. Devise du relevé : |

Les replis français passés à `i18nMsg` / `t` / `t_args` sont **identiques** aux textes `fr-CH`. Le refus
`BANK_IMPORT_UNSUPPORTED_CURRENCY` devient celui du chemin le plus employé : ses textes qui disent « v0.1 » hors des
`.ftl` sont réécrits dans le même patch (validation P1, R4-1) — `crates/kesh-api/src/errors.rs:673-674` (doc-comment et
`#[error("Devise non supportée v0.1 : {0}")]` → `"Devise non supportée : {0}"`), `:2141` (repli de
`bank-import-errors-unsupported-currency` → le texte `fr-CH` de la clé, `messages.ftl:1057` : « Devise non supportée.
Seul le franc suisse (CHF) est pris en charge dans cette version. »), `docs/i18n-glossaire.md:161` (exemple de la ligne
*devise* → le nouveau texte de l'avertissement). **Greper la valeur** après coup : `grep -rnF "v0.1"
frontend/src/lib/features/bank-import crates/kesh-i18n/locales docs/i18n-glossaire.md` ne rend plus
`unsupported-currency` ni `Devise non supportée`, et `grep -nF "Devise non supportée v0.1" crates/kesh-api/src/errors.rs`
ne rend rien.

### La documentation

**AC11 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, **PDF régénéré et commité**, `make user` sous
`flock`) :

- § *Profils banque* (l. 1727-1740) : la liste du formulaire cite « la **colonne devise**, facultative » ;
- la `keshnote` l. 1753 (« **La colonne Devise n'est pas lue** : tout import CSV est enregistré en CHF. ») est
  **remplacée** : avec la colonne réglée, Kesh lit la devise de chaque ligne et refuse un relevé qui n'est pas en CHF,
  ou qui mélange des devises ; sans elle, le relevé est **supposé en CHF** et l'aperçu le dit — un relevé d'un compte
  en euros importé par un tel profil serait enregistré en francs : réglez la colonne ; une cellule vide ou illisible
  est une ligne invalide (rejet partiel) ; le mélange de devises n'est contrôlé que sur les lignes retenues — une ligne
  écartée pour sa date ou son montant ne révèle pas sa devise (AC3, C-27-2-14) ;
- § *Le relevé CSV d'UBS* (tableau l. 1761-1777) : ligne « Colonne devise & 4 (« Devise ») » ; une puce : les profils
  UBS créés avant la 0.13.2 n'ont pas cette colonne — l'ajouter ;
- § *Import dans Kesh* (l. 1717) : « Une devise non supportée bloque l'import » précisé (CAMT.053, ou CSV dont le
  profil lit la colonne Devise) ;
- § *Rejet partiel* (l. 1824-1828) : l'exception « non confirmable » vaut aussi pour un relevé à plusieurs devises
  **et pour un relevé dont la devise n'est pas le CHF** (refusé case cochée, AC14-(3a) — validation P1, R8-1), dans la même
  phrase ;
- ⚠️ Contrôler le **PDF**, aplati — et avec un motif qui **peut** rougir (validation P1, R2-1 : le PDF porte l'apostrophe
  typographique, un motif en `'` droit rend 0 avant comme après le correctif) :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oE "colonne Devise n.est pas lue" | wc -l`
  → **1 avant** le correctif (à relever et écrire au Dev Agent Record), **0 après** ; et une assertion **positive** sur
  la phrase neuve, même aplatissement, motif tolérant à l'apostrophe (`.` à la place de `'`/`’`) → **1 après**.

**AC12 — CHANGELOG** (`CHANGELOG.md`, section `## [0.13.2] — Non publié`, créée ici sous `---` en tête si absente — la
branche de la 27-1 la porte déjà : l'orchestrateur fusionnera) :

- `### Corrigé` — **La devise d'un relevé CSV est lue, et un relevé qui n'est pas en francs suisses est refusé
  ([#624])**. Avant, tout import CSV était enregistré en CHF, au montant du fichier, sans avertissement. Profil :
  colonne Devise facultative ; aperçu : avertissement de devise non supportée, *Confirmer* grisé ; import :
  `422 BANK_IMPORT_UNSUPPORTED_CURRENCY` ; relevé à plusieurs devises : `422 BANK_CSV_MIXED_CURRENCIES`
  (`details.currencies`), non confirmable ; cellule vide ou illisible : ligne invalide (`EMPTY_MANDATORY_FIELD`,
  `INVALID_CURRENCY`). Sans colonne Devise, le relevé reste supposé en CHF et l'aperçu le dit.
- `### Corrigé` — **La devise d'une transaction CAMT.053 est celle de son montant** (AC19). Avant, la devise d'un
  montant d'information — montant d'origine d'un paiement par carte (`<AmtDtls>`), frais (`<Chrgs>`) — remplaçait celle
  de la transaction : sur un compte en CHF, un paiement en euros était enregistré « en EUR » au montant en francs, et
  la réconciliation l'écartait.
- `### Corrigé` — **Modifier un profil bancaire à l'écran ne perd plus sa colonne contrepartie** (AC8-bis), réglable
  par l'API seulement : le formulaire la supprimait au premier enregistrement.
- **À faire** : ajouter la colonne Devise (4 pour UBS) aux profils existants dont le fichier en porte une ; sans elle,
  le relevé reste **supposé en CHF**, comme avant, et l'aperçu le dit.
- **Retour à une 0.13.1** (validation P1, C-27-2-1 — portée réelle) : aucune migration ne l'interdit, mais **un seul**
  profil qui lit la colonne Devise fait échouer `GET /api/v1/bank-profiles` (500) pour **toute la société** sous la
  0.13.1 : l'écran *Profils bancaires* est inutilisable, et l'écran d'import affiche un sélecteur de profil **vide,
  sans message** ; les imports par ce profil sont refusés (rien n'est importé à tort). Le profil ne se corrige plus à
  l'écran sous la 0.13.1 : **retirer la colonne Devise de tous les profils avant le retour** (sous la 0.13.2). Une
  sauvegarde d'installation prise sous la 0.13.2 s'importe dans une 0.13.1, avec la même conséquence.
- **API** : `columnMapping.currency` (facultatif) dans `POST`/`PUT`/`GET /api/v1/bank-profiles` ; la colonne
  `column_mapping_json` de l'export global peut porter la clé `currency`.

**Pour la PR** (texte prescrit, à reprendre par l'orchestrateur — validation P1, R10-1) : `closes #624` ferme le défaut
pour les profils qui lisent la colonne Devise ; **les profils sans colonne Devise — tous les profils existants à la
mise à jour — gardent le CHF supposé**, désormais **signalé** à chaque aperçu (`bank_csv_currency_assumed`) et dit au
manuel, non refusé : un relevé en euros importé par un tel profil est encore enregistré en francs. La fermeture ne se
lit donc pas « le défaut n'existe plus », mais « la devise est lue quand le profil la désigne, et supposée à voix
haute sinon » (C-27-2-4). La PR nomme aussi les deux correctifs entrés en cours de validation (AC8-bis, AC19), sans
issue propre.

**AC12-bis — Manuel d'administration** (`docs/manual/fr/admin-manual.tex`, **PDF régénéré et commité** sous `flock`,
contrôlé aplati) :

- nouvelle `\subsubsection{Passer à la 0.13.2 : la colonne Devise des profils CSV}\label{sec:maj-0-13-2}`, **avant**
  celle de la 0.13.1 (`:1972`), sur son patron : ni compose ni `.env` neufs ; **ajouter la colonne Devise** aux profils
  existants dont le fichier en porte une — **le profil UBS en colonne 4** (renvoi au manuel utilisateur, *Profils banque*
  et *Le relevé CSV d'UBS*) — sans quoi le relevé reste supposé en CHF ; modifier un profil à l'écran garde désormais sa
  colonne contrepartie ; un paragraphe **« Retour à la 0.13.1 »** qui porte la réserve de l'AC12 (un profil à colonne
  Devise rend la liste des profils de toute la société en erreur sous la 0.13.1, sélecteur d'import vide sans message :
  retirer la colonne de tous les profils **avant** le retour ; sinon par SQL) ;
- § *Rollback en cas d'échec* (`:2223`, « le retour à la version précédente n'est possible que si aucune autre migration
  de la release ne relève la version minimale exigée ») : y porter la réserve, par renvoi à `\S\ref{sec:maj-0-13-2}` —
  pour la 0.13.2, le retour exige en plus que les profils CSV ne lisent plus la colonne Devise ;
- contrôle PDF aplati (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`) : `grep -oE "Passer à la
  0.13.2"` et `grep -oE "colonne Devise"` rendent chacun au moins 1.
- ⚠️ Fusion : la branche de la 27-1 modifie le même fichier (`:1993`, `:2270`, `:2305`, `:2463`) sans créer cette
  section ; l'orchestrateur réunit les apports de la 27-1 et de la 27-3 sous la même `\subsubsection` à la fusion.

### Les preuves

**AC13 — Tests du parseur** (`crates/kesh-import/src/csv/parser.rs`, `mod tests` ; fixture
`include_bytes!(…"/tests/fixtures/csv/ubs_synthetique.csv")`, dont la colonne 4 vaut `CHF` sur les 11 lignes ;
variantes **dérivées dans le test** par remplacement de `;CHF;` **sur les seules lignes de données** — C-27-2-9. Le
préambule porte aussi `;CHF;` (`Évaluation en:;CHF;`, ligne 7 : 12 occurrences pour 11 lignes de données) : le helper
de dérivation saute les lignes du préambule et de l'en-tête (`header_row_count` du profil UBS) et asserte qu'il a
remplacé exactement le nombre de cellules voulu — validation P1, R9-1) :

1. profil UBS (26-1) + `currency: Some(4)` → `AllValid`, `stmt.currency == "CHF"`, les 11 `tx.currency == "CHF"` ;
2. même profil, fixture passée en `EUR` sur **toutes** les lignes → `AllValid`, `stmt.currency == "EUR"`, chaque
   `tx.currency == "EUR"` (le parseur reste agnostique ; le refus est en AC14) ;
3. **une seule** ligne passée en `EUR` → `HardFailure(MixedCurrencies { currencies: ["CHF", "EUR"] })` ;
4. une ligne en `eur` sur un fichier tout `eur` → `"EUR"` (casse) ;
5. une cellule vide → `PartialFailure`, une erreur `EMPTY_MANDATORY_FIELD` valeur `"currency"`, 10 valides en CHF ;
6. une cellule `Fr.` → `PartialFailure`, une erreur `INVALID_CURRENCY` ; `€`, `CHF1`, **`ÉUR`** et **`ÉU`** idem —
   `ÉUR` (trois caractères, quatre octets) tue un contrôle « alphabétique Unicode sur trois caractères », `ÉU` (deux
   caractères, trois octets) un contrôle « alphabétique Unicode sur trois octets », que `Fr.`, `€` et `CHF1` laisseraient
   vivre (validation P1, R6-1) ;
7. mélange **et** erreur de ligne dans le même fichier → `MixedCurrencies` l'emporte ;
8. un débit de signe contraire **et** une ligne `EUR` → `AmountSignMismatch` l'emporte (ordre de l'AC3) ;
9. `currency: None` → `stmt.currency == DEFAULT_CSV_CURRENCY` et chaque `tx.currency` aussi (comportement d'avant,
   désormais nommé) ;
10. `currency` hors borne (`Some(40)`) → `ProfileMisconfigured` nommant `column_mapping.currency` ;
11. plus de 10 devises distinctes (les 11 lignes de données, 11 codes distincts) → `currencies.len() == 10`, égales aux
    10 premières de l'ensemble **trié** ; le code qui se trie **en dernier** est placé sur la **première** ligne de
    données, si bien qu'une troncature « 10 premières rencontrées » le garderait et rougirait (validation P1, R6-1) ;
12. **l'erreur de devise garde l'observation de débit** (AC2, R1-1) : profil UBS (mode séparé, `debit_sign`
    `negative`) + `currency: Some(4)`, fixture dérivée où **un** débit (négatif, donc cohérent) a sa cellule Devise
    vidée et les **huit** autres débits de la fixture (elle en porte neuf, et deux crédits) sont passés en positif →
    `HardFailure(AmountSignMismatch { whole_debit_column: false, total_errors: 8, .. })` (`d` = 9, `e` = 8). Sous la
    mutation M18 (`RowParse::err` sur l'erreur de devise), `d` = 8 = `e` et `whole_debit_column` devient `true` : le
    test rougit.

**AC14 — Tests d'intégration** (`crates/kesh-api/tests/bank_imports_e2e.rs`, sur le patron de
`post_import_rejects_eur_currency` l. 442 et des tests CSV l. 1093-1263, `#[sqlx::test(migrations =
"../kesh-db/test-schema")]`) :

1. aperçu CSV, profil avec devise, fichier EUR → `200`, `warnings.unsupportedCurrency.currency == "EUR"`,
   `selectedStatement.currency == "EUR"` ;
2. import du même → `422 BANK_IMPORT_UNSUPPORTED_CURRENCY`, `details.currency == "EUR"`, **zéro** ligne ajoutée à
   `bank_imports`, `bank_transactions` et `audit_log` (compter avant/après) ;
3. import EUR **avec** erreurs de ligne, deux assertions **nommées** (validation P1, R5-1) : (3a) `confirmPartialImport=true`
   → `422 BANK_IMPORT_UNSUPPORTED_CURRENCY`, rien d'écrit ; (3b) `confirmPartialImport=false` → **même** code, et
   **pas** `BANK_CSV_PARTIAL_FAILURE` — c'est (3b), et elle seule, qui tue M5 (sous M5, (3a) rend le même `422`) ;
4. fichier mélangé → aperçu **et** import `422 BANK_CSV_MIXED_CURRENCIES`, `details.currencies == ["CHF","EUR"]`,
   rien d'écrit ;
5. profil avec devise, fichier CHF → `201`, transactions en base `currency = 'CHF'` ; aperçu sans
   `bank_csv_currency_assumed` ;
6. profil **sans** devise → aperçu porte `"bank_csv_currency_assumed"` dans `warnings.informational` ; import `201`
   comme avant (non-régression).
7. `bank_profiles_e2e.rs` : AC7.
8. **CAMT.053** (AC19) : aperçu puis import de la fixture `v04_amtdtls_foreign_currency.xml` sur un compte CHF →
   `200` sans `warnings.unsupportedCurrency`, puis `201` ; **chaque** ligne de `bank_transactions` de l'import en
   `currency = 'CHF'` — la valeur traverse la frontière HTTP et la base.

**AC15 — E2E Playwright** (`frontend/tests/e2e/bank-csv-import.spec.ts`, patron des tests UBS de la 26-1, l. 260-400 ;
`createProfileViaUi` gagne la colonne `col-currency-input`) :

1. profil UBS avec devise 4 créé **par l'écran**, `ubs_synthetique.csv` importé → succès, transactions `currency ==
   "CHF"` par l'API, et `warning-informational` ne porte **pas** `data-info-code="bank_csv_currency_assumed"` ;
2. même profil, fichier EUR **dérivé en mémoire** (`setInputFiles({ name, mimeType, buffer })`) → aperçu
   `warning-unsupported-currency` visible, `bank-import-confirm` désactivé, `importTotal` inchangé ;
3. le test existant « imports the UBS CSV as is » (profil sans devise) asserte en plus
   `data-info-code="bank_csv_currency_assumed"` présent.

**AC16 — Contrôle local sur le vrai relevé** (`crates/kesh-import/tests/ubs_fichier_reel.rs`, ignoré par défaut) : le
profil recommandé gagne `currency: Some(4)` ; le test vérifie `stmt.currency == "CHF"` et chaque `tx.currency ==
"CHF"` **sans imprimer de valeur** (même discipline que l'en-tête du fichier : « conforme » ou le nom de l'assertion).
Le `match` de son message de refus (`ubs_fichier_reel.rs:97-105`) gagne un bras `MixedCurrencies { currencies } =>`
nommant les codes de devise (ce ne sont pas des données du relevé) : le repli `_ => "refus avant la lecture des
lignes"` serait faux pour ce refus, qui vient après (validation P1, R3-1).
⛔ **Lancé par l'orchestrateur seulement** (`KESH_UBS_REAL_CSV=…`), rapporté « conforme / non conforme » ; l'agent
de dev ne lit rien sous `~/kesh-test/`.

### Les mutations et les gates

**AC17 — Mutations à jouer** (chacune doit faire rougir au moins un test nommé ; restaurer puis **toucher** le
fichier — mémoire « mutation restaurée, binaire périmé ») :

| # | Mutation | Test qui doit rougir |
|---|---|---|
| M1 | `parse_row` ignore la cellule, `currency = DEFAULT_CSV_CURRENCY` | AC13-2, AC14-1, AC15-2 |
| M2 | `stmt.currency` reste la constante quand la colonne est réglée | AC13-2, AC14-1/2 |
| M3 | `preview_csv` n'appelle pas le contrôle | AC14-1, AC15-2 |
| M4 | `create_csv` n'appelle pas le contrôle | AC14-2 |
| M5 | contrôle placé **après** le test `confirm_partial_import` | AC14-(3b) (seule ; (3a) rend le même `422`) |
| M6 | contrôle du mélange retiré | AC13-3, AC14-4 |
| M7 | pas de mise en majuscules | AC13-4 |
| M8 | cellule vide acceptée en CHF | AC13-5 |
| M9 | forme non contrôlée (`Fr.` acceptée), ou contrôle alphabétique Unicode (`ÉUR` accepté) / sur la longueur en octets (`ÉU` accepté) | AC13-6 |
| M10 | `currency` absent de `all_indices()` | AC1-(1), AC13-10 |
| M11 | `skip_serializing_if` retiré du champ | AC1-(2) |
| M12 | `MixedCurrencies` évalué avant le refus de signe | AC13-8 |
| M13 | `bank_csv_currency_assumed` toujours poussé / jamais poussé | AC14-5 / AC14-6, AC15-1 / AC15-3 |
| M14 | le formulaire n'envoie pas `currency` | AC8-(a)(b), AC15-1 |
| M15 | le formulaire ne relit pas `existing.columnMapping.currency` | AC8-(d) |
| M16 | `map_csv_error` sans bras `MixedCurrencies` (repli `other`) | AC14-4 (code ≠ `BANK_CSV_MIXED_CURRENCIES`) |
| M17 | plafond des devises rapportées retiré, ou troncature aux 10 premières **rencontrées** | AC13-11 |
| M18 | l'erreur de devise rendue par `RowParse::err` (observation de débit perdue) | AC13-12 |
| M19 | CAMT : la devise de transaction reprise de **tout** `<Amt Ccy>` sous `<TxDtls>` (code d'avant) | AC19-1 |
| M20 | CAMT : la devise d'entrée reprise de tout `<Amt Ccy>` sous `<Ntry>` hors `<TxDtls>` (code d'avant) | AC19-2 |
| M21 | le formulaire ne recopie pas les clés qu'il ne montre pas (`...rest` retiré) | AC8-(f) |
| M22 | le formulaire recopie **tout** le mapping existant, sans en retirer ses propres clés | AC8-(g) |

**AC18 — Gates** (au **dernier commit de code**, sous `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`) :
`scripts/test-fast.sh` complet (base remise à zéro avant — base `kesh_272`), `cargo publish --dry-run -p kesh-import`
(zéro dépendance interne, à **exécuter**, il est aussi dans la CI), `npm run check`, `npm run lint-i18n-ownership`,
`npm run test:unit`, `npm run build`, **E2E complet** (base `kesh_e2e_272`, port 3035 ; échecs jugés fichier par
fichier contre `docs/testing.md` § « Les échecs attendus »). Le Dev Agent Record n'affirme que ce qui a tourné, avec
ses chiffres recomptés.

### La devise des transactions CAMT.053 (périmètre élargi en validation P1)

**AC19 — La devise d'une transaction CAMT.053 est lue sur l'élément qui porte son montant**
(`crates/kesh-import/src/camt053/mod.rs`, C-27-2-11). Aujourd'hui, à l'ouverture de **tout** `<Amt Ccy="…">`
(`:168-178`), la devise écrase `t.currency` tant qu'un `<TxDtls>` est ouvert, et `n.currency` sinon — le dernier
rencontré gagne ; or le montant ne se lit que sur `TxDtls/Amt` (`handle_txdtls_text`, `ends_with(path, &["TxDtls",
"Amt"])`, `:427`) et `Ntry/Amt` (`:372`). Un `<AmtDtls><InstdAmt><Amt Ccy="EUR">` (montant d'origine d'un paiement par
carte), un `<Chrgs>…<Amt Ccy="EUR">` ou un `<Intrst>` posés après le montant donnent donc à une transaction d'un compte
CHF la devise EUR, au montant en francs ; `validate_currency_supported_v0_1` ne contrôle que `Acct/Ccy` (`:325-327`),
et la réconciliation écarte ensuite la transaction (`reconciliation.rs:631`, `:2728`).

- Correction : à l'ouverture d'un `Amt`, `path` ne contient pas encore `Amt` (il est poussé après le `match`) ; la
  devise est retenue **seulement** si `path.last()` vaut `"TxDtls"` (pour `t.currency`) ou `"Ntry"` (pour
  `n.currency`) — exactement l'élément dont le montant est lu. Tout autre `<Amt Ccy>` est ignoré pour la devise. Un
  commentaire au site dit pourquoi (la devise d'un montant se lit sur l'élément du montant ; les montants d'information
  portent la devise d'origine) et renvoie aux deux ancrages du montant, qu'il ne faut pas laisser diverger.
- Repli inchangé : un `<TxDtls>` sans `Amt` propre prend le montant **et** la devise de l'entrée (`:603`, `:618`).
- **Fixture synthétique** neuve `crates/kesh-import/tests/fixtures/camt053/v04_amtdtls_foreign_currency.xml`, sur le
  patron de `v04_with_subtxs.xml` (même IBAN de test, compte `CHF`, soldes d'ouverture et de clôture cohérents avec les
  entrées, ordre des éléments du schéma 001.04 — `AmtDtls` après `CdtDbtInd`, `Chrgs` après `BkTxCd`) : entrée 1 avec un
  `<TxDtls>` `<Amt Ccy="CHF">` suivi de `<AmtDtls><InstdAmt><Amt Ccy="EUR">` (et son `CcyXchg`) et de
  `<Chrgs><Rcrd><Amt Ccy="EUR">` ; entrée 2 **sans** `<TxDtls>`, `<Amt Ccy="CHF">` suivi de
  `<AmtDtls><InstdAmt><Amt Ccy="EUR">`. Valeurs inventées.
- **Tests** (`crates/kesh-import/tests/camt053_tests.rs`) : (1) la transaction de l'entrée 1 a `currency == "CHF"` et le
  montant de `TxDtls/Amt` ; (2) celle de l'entrée 2 a `currency == "CHF"` et le montant de `Ntry/Amt` ; (3)
  non-régression : `parse_eur_currency_preserved` (compte et transaction EUR) et
  `parse_with_subtxs_extracts_individual_transactions` restent verts sans modification. ⛔ (1) et (2) sont écrits
  **avant** la correction et lancés sur le code d'avant : ils doivent rougir (`"EUR"` au lieu de `"CHF"`) — le rouge est
  noté au Dev Agent Record, sinon le test ne prouve rien. Test d'intégration : AC14-8.
- L'en-tête de `camt053_tests.rs` (« Les 10 fixtures couvrent », pour 13 fichiers à `a2743fd4`) est **recompté** depuis
  `ls crates/kesh-import/tests/fixtures/camt053/ | wc -l` et gagne la ligne de la fixture neuve.
- Ni `validate_currency_supported_v0_1` ni le contrôle par transaction ne changent (C-27-2-3, alternative toujours
  écartée).

## Tâches

- [ ] **T0** — Brancher dans le worktree `/home/gcorbaz/devel/kesh-27-2` (fait : `story/27-2-devise-import-csv`) ;
  `node_modules` par `cp -al` depuis le dépôt principal ; `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-27-2/target` ;
  bases `kesh_272` / `kesh_e2e_272` créées, migrées, seedées.
- [ ] **T1** — `kesh-import` : AC1, AC2, AC3, AC4, AC19 ; tests AC1, AC13 et AC19 (AC19-1/2 rouges d'abord) ; mutations
  M1, M2, M6-M12, M17-M20.
- [ ] **T2** — `kesh-api` : AC5, AC6, AC7 (helper de contrôle factorisé, variant d'erreur, bras explicite) ; tests
  AC14 (dont AC14-8, CAMT) ; mutations M3-M5, M13, M16 ; textes « v0.1 » de `errors.rs` (AC10).
- [ ] **T3** — i18n : AC10 (quatre locales, gardes Vitest et lint).
- [ ] **T4** — Frontend : AC8, AC8-bis, AC9 ; Vitest (dont `csv-line-error-label.test.ts`) ; mutations M14, M15,
  M21, M22.
- [ ] **T5** — E2E : AC15.
- [ ] **T6** — `ubs_fichier_reel.rs` : AC16 (compilé, **non lancé** par l'agent).
- [ ] **T7** — Manuel utilisateur (AC11) et manuel d'administration (AC12-bis), PDF régénérés + contrôles aplatis ;
  `docs/i18n-glossaire.md` (AC10) ; CHANGELOG (AC12) ; `README.md` « Feuille de route » vérifié
  (aucun changement attendu) ; `website/` vérifié (aucune promesse sur la devise CSV attendue : `grep -rni "devise\|currency" website/`).
- [ ] **T8** — Propagation : `grep -rnF '"CHF"' crates/kesh-import/src` (AC4) ; `grep -rnE "n.est pas lue" docs/manual/fr
  CHANGELOG.md` (la phrase de 0.13.1 dans le CHANGELOG **reste** : c'est l'histoire) ; `grep -rnF "v0.1"` sur les
  clés et sites modifiés (AC10) ; `grep -rn "counterparty" frontend/src/lib/features/bank-import` (AC8-bis) ; gates
  AC18 ; mutations M1-M22 ; Dev Agent Record.

## Dev Notes

### État actuel des fichiers modifiés (lus le 2026-10-12 sur `a2743fd4`)

- `crates/kesh-import/src/csv/profile.rs` (571 l.) — `ColumnMapping` (l. 37-60) en `deny_unknown_fields`, champs
  facultatifs en `skip_serializing_if` + `default` ; `all_indices` (l. 65-84), `max_index` (l. 88-94) ; `validate`
  (l. 190-310). **À préserver** : l'ordre et les messages des contrôles existants, `DebitSign` intact.
- `crates/kesh-import/src/csv/parser.rs` (2056 l.) — `parse_csv_collect_inner` (l. 157-392) : validate, encodage, saut
  des lignes, garde de surcomptage, premier enregistrement, hors-borne, boucle, refus de signe (7.b, l. 303-315),
  construction du relevé (l. 357-367), discrimination (l. 369-391). `parse_row` (l. 519-690). `RowAccumulator`
  (l. 451-514). **À préserver** : la priorité du refus de signe, `whole_debit_column`, la sentinelle de période, les
  plafonds `MAX_CSV_LINE_ERRORS`.
- `crates/kesh-import/src/error.rs` — `CsvError` (l. 59-198), `CsvLineErrorCode` (l. 204-265), `all_is_complete`
  (l. 379).
- `crates/kesh-api/src/routes/bank_imports.rs` (1999 l.) — `map_csv_error` (l. 624-673), `db_profile_to_csv_profile`
  (l. 1590-1607, inchangé : il passe le `ColumnMapping` entier), `preview` CAMT (l. 896-1043, bloc devise
  l. 961-971 à factoriser), `create` CAMT (l. 1048-…, l. 1126-1130), `preview_csv` (l. 1609-1756), `create_csv`
  (l. 1758-1999 ; branche `PartialFailure` l. 1825-1861).
- `crates/kesh-api/src/errors.rs` — `BankImportUnsupportedCurrency` (l. 675, réponse l. 2135-2147),
  `BankCsvHeaderSkipsData` (l. 806, réponse l. 2395-2411).
- `frontend/src/lib/features/bank-import/BankProfileForm.svelte` (304 l.) — état l. 26-47, `columnMapping` l. 56-69,
  grille des colonnes l. 257-285 ; libellés de colonnes en français en dur (non repris, C-27-2-7).
- `frontend/src/lib/features/bank-import/BankImportUpload.svelte` — `buttonDisabled` l. 217-228 (grise *Confirmer* sur
  `unsupportedCurrency`), bloc l. 596-608, informations l. 616-625 (`data-info-code`).
- `docs/manual/fr/user-manual.tex` — § *Import CSV multi-encodage* l. 1721-1807, *Rejet partiel* l. 1824-1828.
- `docs/manual/fr/admin-manual.tex` — § *Mise à jour de Kesh* (l. 1920), *Passer à la 0.13.1* (l. 1972, patron de la
  section neuve), *Rollback en cas d'échec* (phrase sur la version minimale, l. 2223).
- `crates/kesh-import/src/camt053/mod.rs` — `Event::Start` de `Amt` (l. 168-178, `path.push` l. 183), ancrages du
  montant `Ntry/Amt` (l. 372) et `TxDtls/Amt` (l. 427), replis montant/devise d'un `<TxDtls>` (l. 603, 618).
  **À préserver** : tout le reste du parseur (signes, soldes, contreparties, multi-relevés).

### Points de vigilance

- **29 littéraux `ColumnMapping { … }`** à compléter de `currency: None` (`grep -rn "ColumnMapping {" crates` : 23 dans
  `parser.rs`, 5 dans `profile.rs`, 1 dans `tests/ubs_fichier_reel.rs`) — le compilateur les trouve, mais
  `ubs_fichier_reel.rs` est un test **ignoré** : `cargo build --workspace --all-targets` le compile, `nextest` ne
  l'exécute pas.
- **Ordre des refus**, à tenir exactement : fichier déjà importé (avant parse, `create_csv`) → erreurs fatales du
  parseur → refus de signe → mélange de devises → [API] devise non supportée → rejet partiel (`confirmPartialImport`)
  → zéro ligne valide.
- Le contrôle de devise au chemin CSV se fait sur le relevé **après** le second essai `EncodingMismatch` (aperçu) /
  la reprise `confirm_encoding_mismatch` (import) — pas sur le premier `HardFailure`.
- `DEFAULT_CSV_CURRENCY` est la **seule** supposition restante, et elle est dite à l'aperçu (AC6). Ne pas ajouter de
  repli silencieux ailleurs (une cellule vide n'est **pas** CHF, AC2).
- **Le contrôle de devise n'est pas touché** (C-27-2-3) : `validate_currency_supported_v0_1` garde sa signature et son
  corps. Le parseur CAMT ne change qu'à l'ancrage de la devise (AC19) — rien d'autre.
- **Zéro ligne valide, colonne Devise réglée** (validation P1, F3) : l'aperçu annonce `selectedStatement.currency =
  "CHF"` (`DEFAULT_CSV_CURRENCY`, AC3) sans l'avoir lu, et aucun avertissement de devise. Sans dommage — l'import est
  refusé (`no_valid_lines_to_commit`) et le contrôle de devise ne refuse pas ce cas à tort — mais à garder en tête si
  l'aperçu est un jour affiné ; pas de test exigé.
- **Un `PUT` aveugle** (validation P1, F5-b) : un onglet de la 0.13.1 resté ouvert pendant la mise à jour, ou un client
  d'API, qui renvoie un profil sans `currency` le ramène au CHF supposé (remplacement complet), sans erreur ; seul
  l'avertissement `bank_csv_currency_assumed` le signale ensuite. Risque faible (poste unique), assumé (C-27-2-1).
- Fusion avec la 27-1 (orchestrateur) : conflits attendus sur `CHANGELOG.md` (§ 0.13.2), `sprint-status.yaml`,
  `i18n-keys.test.ts` (`sitesTotal` : la 27-1 en ajoute 9), les quatre `messages.ftl`, le registre des choix,
  `admin-manual.tex` (la 27-1 y touche la section de la 0.13.1, AC12-bis).

### Ce qui reste hors périmètre

La lecture du préambule UBS (#621) ; une devise par compte bancaire (multi-devises) ; un **contrôle** de la devise de
chaque transaction contre le périmètre CHF (C-27-2-3) ; un **champ** contrepartie au formulaire (la clé est désormais
gardée, pas montrée — C-27-2-12) et les libellés non traduits du formulaire (C-27-2-7).

### Architecture et conventions à suivre

- `kesh-import` reste **sans dépendance interne** et ne connaît pas le périmètre CHF : le refus « non CHF » vit dans
  `kesh-core` (doc de `validate_currency_supported_v0_1`, l. 456-461). Le parseur sait seulement lire une devise,
  exiger qu'elle soit unique, et nommer celle qu'il suppose.
- Erreurs : bras explicites dans `map_csv_error`, codes `SCREAMING_SNAKE_CASE`, messages traduits côté serveur
  (`t` / `t_args`), `details` structurés.
- Doc-comments `///` sur tout ajout public ; JSDoc sur la fonction du formulaire si elle grossit.

### Leçons des stories précédentes à appliquer ici

- 26-1 : le site qui fait passer le réglage du formulaire à l'import est `db_profile_to_csv_profile` — ici le
  `ColumnMapping` y passe entier, mais l'E2E AC15-1 reste la preuve que la valeur traverse la frontière HTTP.
- 26-1 : les bras explicites de `map_csv_error`, les codes de ligne lus depuis la production par
  `i18n-keys.test.ts`.
- « Un détecteur part du symptôme » : l'inventaire ci-dessus est refait par grep du littéral **et** des lecteurs.
- « Tests qui prouvent moins » : AC14-2 compte les lignes **avant et après** au lieu de seulement lire le statut.
- Données réelles : rien du relevé de Guy dans le dépôt, ni dans un rapport ; fixtures synthétiques dérivées.

### Project Structure Notes

Aucun fichier neuf hors tests et artefacts BMAD ; aucune migration ; **une** fixture neuve, synthétique :
`crates/kesh-import/tests/fixtures/camt053/v04_amtdtls_foreign_currency.xml` (AC19). Aucune fixture CSV neuve.

### References

- Issue #624 ; fiche 26-1 (`26-1-import-csv-ubs-natif.md`, § Le fichier d'UBS, AC10-AC14) ; `CLAUDE.md`
  § Migration breaking policy (P1, P2-bis, P5-P8), § Review Iteration Rule (sites non résolus, propagation),
  § Test Locally First.
- `crates/kesh-db/migrations/20261011000001_bank_profiles_debit_sign.sql` (en-tête : le précédent de la colonne
  dédiée et sa raison).

## Dev Agent Record

### Agent Model Used

*(à remplir par `bmad-dev-story`)*

### Debug Log References

### Completion Notes List

- Fiche créée le 2026-10-12 (Opus 5.5, `bmad-create-story`, autonomie) — analyse du code au sol sur `a2743fd4`.

### File List

## Change Log

- 2026-10-12 — Fiche créée (19 AC : AC1, AC1-bis, AC2-AC18 ; tâches T0-T8 ; mutations M1-M17 ; choix C-27-2-1 à
  C-27-2-10). Validation P1 à lancer (`27-2-validate-prompt-p1.md`).

### Validation P1

2026-10-12. Passe **complète**, **Sonnet**, deux lentilles en contexte frais et en lecture seule (prompt
`27-2-validate-prompt-p1.md`) : **R** (relecture contre le code au sol) — 0 CRITICAL, 0 HIGH, **2 MEDIUM**, 8 LOW ;
**F** (faisabilité et chemins de la décision) — 0 CRITICAL, 0 HIGH, **1 MEDIUM**, 6 LOW. Total **3 MEDIUM, 14 LOW**. Les
deux lentilles confirment C-27-2-1 (aucune migration, aucun bump) aux tags `v0.13.1` et `v0.13.0`. Remédiation
(Opus 5.5, sur consignes de l'orchestrateur, qui a **élargi le périmètre** à F1 et F6 sans issue neuve — le nombre de
bugs doit baisser à la release) :

| Finding | Sév. | Décision |
|---|---|---|
| R1-1 | MEDIUM | **Appliqué** : AC2 (l'erreur de devise garde `debit`), test AC13-12, mutation M18. |
| R2-1 | MEDIUM | **Appliqué** : AC11, motif `n.est` (1 avant / 0 après) + assertion positive sur la phrase neuve ; T8 aligné. |
| F1 | MEDIUM | **Appliqué, périmètre élargi** : AC8-bis (le formulaire garde toute clé qu'il ne montre pas), Vitest (f)(g)(h), mutations M21, M22 ; inventaire (f) résolu ; C-27-2-7 amendé, C-27-2-12. |
| R3-1 | LOW | **Appliqué** : `csv-line-error-label.test.ts` (AC9), `invalid_currency_i18n_key` (AC2), bras `MixedCurrencies` de `ubs_fichier_reel.rs` (AC16). |
| R4-1 | LOW | **Appliqué** : AC10 — `errors.rs:673-674`, `:2141`, `docs/i18n-glossaire.md:161`, grep élargi. |
| R5-1 | LOW | **Appliqué** : AC14-(3a)/(3b) nommées, M5 tuée par (3b) seule ; AC5 — contrôle dans chaque bras du `match`. |
| R6-1 | LOW | **Appliqué** : AC13-6 (`ÉUR`, et `ÉU` pour le contrôle en octets), AC13-11 et AC3 (10 premières du tri, sans indicateur — C-27-2-13), M9 et M17 élargies. |
| R7-1 / F4 | LOW | **Appliqué** (même constat) : AC3 et AC11 l'écrivent ; C-27-2-14. |
| R8-1 | LOW | **Appliqué** : AC11, exception « non confirmable » étendue au relevé non CHF. |
| R9-1 | LOW | **Appliqué** : AC13 et C-27-2-9 — dérivation limitée aux lignes de données, nombre de remplacements asserté. |
| R10-1 | LOW | **Appliqué** : § *Pour la PR* (AC12) — les profils sans colonne Devise gardent le CHF supposé, signalé. |
| F2 | LOW | **Appliqué** : AC8 — Svelte 5 rend `null` ; (c) asserte `toBeUndefined()` et l'absence au `JSON.stringify`. |
| F3 | LOW | **Écrit** aux Dev Notes (aperçu à zéro ligne valide), sans test : aucun dommage. |
| F5 | LOW | (a) **Appliqué** : la portée réelle du retour à la 0.13.1 (C-27-2-1, AC12, AC12-bis) ; (b) **écrit** aux Dev Notes et à C-27-2-1 (PUT aveugle, assumé). |
| F6 | LOW | **Appliqué, périmètre élargi** : AC19 (ancrage de la devise CAMT), fixture synthétique, tests rouges d'abord, AC14-8, mutations M19, M20 ; inventaire (h) résolu ; C-27-2-10 amendé, C-27-2-11. Plus d'issue à ouvrir. |
| F7 | LOW | **Appliqué autrement** : invariant écrit au doc-comment (AC3), déjà asserté par AC13-1/-2/-5/-9 sur chaque transaction ; pas de `debug_assert!` (absent en release). |
| Question C-27-2-1 | — | **Confirmé**, et amendé : portée réelle (500 sur la liste des profils de toute la société, sélecteur d'import vide sans message, retirer la colonne avant le retour) à AC12, au manuel d'administration (AC12-bis, section *Passer à la 0.13.2* et réserve de `:2223`) et à C-27-2-1 (P1 lu à la lumière de son objet). |

Décomptes après remédiation, recomptés dans la fiche : **22 AC** (AC1, AC1-bis, AC2-AC8, AC8-bis, AC9-AC12, AC12-bis,
AC13-AC19) ; tâches **T0-T8** ; **12** tests de parseur AC13 ; **8** items AC14 ; **8** cas Vitest AC8/AC8-bis
((a)-(h)) ; **3** items AC19 ; mutations **M1-M22** ; choix **C-27-2-1 à C-27-2-14**. Symptômes grepés après patch
(fiche, registre, prompt P2) : `colonne Devise n'est pas lue` (motif à apostrophe droite : ne subsiste que dans la
citation du texte actuel du manuel, AC11, 2e puce), `counterparty` / « perdu »,
« hypothèse non établie », `M1-M17`, `C-27-2-10)`, `19 AC`, « aucune fixture neuve ». Validation P2 à lancer
(`27-2-validate-prompt-p2.md`, **Opus**, lentilles R et F).
