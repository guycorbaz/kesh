# Story 27.2 : La devise d'un relevé CSV est lue, et le contrôle de devise joue

## Status

**ready-for-dev** — fiche créée le 2026-10-12 en autonomie (skill `bmad-create-story`), sur `a2743fd4`
(`origin/main`, rétrospective de l'Epic 26). Validation P1 faite (Sonnet, lentilles R et F : 3 MEDIUM, 14 LOW),
remédiée le 2026-10-12 — périmètre élargi par l'orchestrateur au formulaire qui perd la contrepartie (AC8-bis) et à
la devise des transactions CAMT.053 (AC19). Validation P2 faite (Opus, lentilles R et F : 3 MEDIUM, 18 LOW), remédiée
le 2026-10-12. Validation P3 faite (Sonnet, lentilles R et F : 1 MEDIUM distinct, 13 LOW distincts), remédiée le
2026-10-12 ; validation P4 **ciblée** à lancer : prompt `27-2-validate-prompt-p4-ciblee.md` (Haiku, une lentille).
Choix tranchés : **C-27-2-1 à C-27-2-28**, registre `_bmad-output/implementation-artifacts/epic-27-choix-autonomes-27-2.md`
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
| (i) | `reconciliation.rs:631`, `:780`, `:2728` — garde « transaction non CHF » des **propositions automatiques** et de l'**acceptation d'une règle** seulement ; le rapprochement manuel (`post_manual_once`, `:3532`), l'éclatement (`post_split_once`, `:3984`, `accept_one_split`, `:2119`) et l'acceptation d'une proposition de facture (`accept_one_invoice`, `:1428`) ne contrôlent pas la devise (`reconciliation_e2e.rs:2241-2262` le documente ; validation P3, F1 = R1) | défense en aval, **partielle** | inchangé (C-27-2-20) ; devient effective pour le CSV dont le profil lit la colonne Devise (une ligne non CHF n'atteint plus la base) |

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
- ⛔ **`deny_unknown_fields` est la prémisse de C-27-2-1, et il est figé** (validation P2, F1 = R10 ; C-27-2-15) : le
  doc-comment `///` de `ColumnMapping` le dit, à l'intention de qui ajoutera la clé suivante — « ajouter un champ rend
  les profils qui s'en servent illisibles par les binaires antérieurs : refus bruyant, sans bump (C-27-2-1, Story
  27-2). Retirer `deny_unknown_fields` rendrait **silencieux** le retour depuis une version ultérieure — un champ
  inconnu serait ignoré, et c'est le downgrade corrupteur que la politique de migration interdit. » ; et un test le
  garde (5, ci-dessous), que tue la mutation M23.
- `all_indices()` le pousse sous le nom `"currency"` → la collision (`validate()`) et le hors-borne au premier
  enregistrement (`parse_csv_collect_inner`, étape 6.b) le couvrent **sans code neuf**.
- Doc-module de `profile.rs` et de `parser.rs` : la colonne Devise est citée.
- Tests (`profile.rs`, `mod tests`) : (1) collision `currency` = `date` → `ProfileMisconfigured` nommant
  `column_mapping.currency` ; (2) **un mapping sans devise se sérialise sans la clé** :
  `!serde_json::to_string(&m).unwrap().contains("currency")` — c'est ce qui garde le retour à une 0.13.1 indolore pour
  les profils qui ne lisent pas la devise (C-27-2-1) ; (3) `{"date":0,"amount":1,"currency":2}` se désérialise en
  `Some(2)` ; (4) l'ancien JSON `{"date":2,"debit_credit_split":[5,6]}` donne `currency: None` ; (5)
  `unknown_key_is_rejected_so_older_binaries_refuse_loudly` : `serde_json::from_str::<ColumnMapping>(
  r#"{"date":0,"amount":1,"bogus":2}"#)` → `Err`, avec un commentaire qui renvoie à C-27-2-1 (tue M23).

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
  deux bras** du `match` (`AllValid`, et `PartialFailure` — bras ouvert `:1825` — sur `valid` **avant**
  `if !fields.confirm_partial_import`, `:1831`) ; un appel unique placé après le `match` ne convient pas (validation P1, R5-1) → `Err(AppError::BankImportUnsupportedCurrency(c))`, `422 BANK_IMPORT_UNSUPPORTED_CURRENCY`,
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
  `JSON.stringify` omet. Le test (v3) l'établit.
- **Vitest** (`BankProfileForm.test.ts`, patron existant). ⚠️ Le fichier porte **déjà** des cas `(a)` à `(e)`
  (26-1, l. 66-109) : les cas neufs sont libellés `(v1)`… `(v9)`, en tête du titre `it('(v1) …')`, et AC17 les cite
  ainsi (validation P2, F5). (v1) devise 4 en mode séparé → requête
  `columnMapping.currency === 4` ; (v2) idem en colonne unique ; (v3) champ laissé vide →
  `expect(req.columnMapping.currency).toBeUndefined()` **et** `JSON.stringify(req)` ne contient pas `"currency"` (la clé
  est présente en JS, de valeur `undefined` : un `'currency' in …` serait vrai — validation P1, F2) ; (v4) profil
  existant `currency: 4` → le champ s'ouvre à 4, et l'enregistrer sans y toucher le renvoie ; (v5) profil existant sans
  devise → champ vide.

**AC8-bis — Le formulaire ne perd aucune clé qu'il ne montre pas** (validation P1, F1 ; périmètre élargi par
l'orchestrateur, C-27-2-12). Aujourd'hui `columnMapping` est reconstruit des seuls champs du formulaire
(`BankProfileForm.svelte:56-69`) et le `PUT` est un remplacement complet : un `counterparty` posé par l'API est
**perdu** au premier enregistrement à l'écran — chemin que AC11 et le CHANGELOG prescrivent justement d'emprunter pour
ajouter la colonne Devise.

- Le mapping envoyé part du mapping existant **privé des clés que le formulaire possède** (`date`, `amount`,
  `debit_credit_split`, `reference`, `details`, `currency`), puis reçoit les valeurs du formulaire,
  `{ ...keepUnownedKeys(existing?.columnMapping), ...champsDuFormulaire }`, dans les **deux** branches.
- ⛔ **Une seule source de vérité des clés possédées** (validation P2, F4 = R2 ; C-27-2-17) : la constante, et non une
  déstructuration qui récrirait la liste dans son motif. Forme prescrite, typée sans `any` ni cast :

  ```ts
  /** Clés de `columnMapping` que ce formulaire écrit ; toute autre clé du profil existant est gardée telle quelle
   *  (Story 27-2, AC8-bis). Indispensable pour les clés que le formulaire n'écrit que dans l'un de ses deux modes
   *  (`amount`, `debit_credit_split`) : absentes d'ici, la valeur de l'ancien profil survivrait au changement de mode
   *  (refus `422`). Les clés écrites dans les deux modes y figurent aussi, par sûreté : le formulaire les écrase
   *  aujourd'hui, mais ne plus les écrire dans un mode les ferait survivre. Une clé neuve montrée par le formulaire
   *  s'ajoute ICI. */
  const OWNED_COLUMN_MAPPING_KEYS = [
  	'date', 'amount', 'debit_credit_split', 'reference', 'details', 'currency',
  ] as const satisfies readonly (keyof ColumnMapping)[];

  /** Le mapping existant privé des clés que le formulaire possède. */
  function keepUnownedKeys(existing?: ColumnMapping): Partial<ColumnMapping> {
  	const rest: Partial<ColumnMapping> = { ...existing };
  	for (const key of OWNED_COLUMN_MAPPING_KEYS) delete rest[key];
  	return rest;
  }
  ```

  Toutes les propriétés de `Partial<ColumnMapping>` sont facultatives, ce qui rend le `delete` licite en `strict`
  (`frontend/tsconfig.json`) ; le spread suivant fournit `date`, d'où un type compatible avec
  `CreateOrUpdateBankProfileRequest.columnMapping`. **Import** : `ColumnMapping` n'est pas importé aujourd'hui par
  `BankProfileForm.svelte` ; il s'ajoute au bloc **`import type { … } from './bank-profile.types'`** existant (l. 15-19),
  jamais à un import de valeur — `verbatimModuleSyntax` est actif (`.svelte-kit/tsconfig.json:18`, fichier engendré
  par `svelte-kit sync`), un import de valeur d'un type y est refusé (validation P3, F6). **`satisfies`
  dans un `.svelte`** : le dépôt ne l'emploie que dans des `.test.ts` (aucun `.svelte` de production) ; le compilateur
  Svelte 5.55.1 retire `TSAsExpression` et `TSSatisfiesExpression`
  (`node_modules/svelte/src/compiler/phases/1-parse/remove_typescript_nodes.js:86`, `:89`), la forme est donc admise
  par lecture, sans précédent au dépôt (validation P3, R9). *Non compilé à la rédaction : `npm run check` **et**
  `npm run build` le confirment en T4 ; si l'un refuse, garder la constante et adapter la fonction, jamais revenir à
  deux listes.*
- Ce n'est pas une recopie de tout le mapping existant : `amount` et `debit_credit_split` s'excluent (`validate()`), et
  un passage de la colonne unique au mode séparé laisserait sinon les deux clés — refus `422` ; **le passage inverse
  aussi** (mode séparé → colonne unique : `debit_credit_split` resterait à côté du nouvel `amount`).
- **Vitest** : (v6) profil existant `counterparty: 3`, enregistré sans y toucher, en colonne unique **et** en mode séparé
  → `req.columnMapping.counterparty === 3` ; (v7) profil existant en colonne unique (`amount: 1`, `counterparty: 3`),
  passé au mode séparé → la requête porte `debit_credit_split` et `counterparty: 3`, et `amount` y est `undefined`
  (absent de `JSON.stringify(req)`) ; (v8) **sens inverse** (validation P2, R1) : profil existant en mode séparé
  (`debit_credit_split: [5, 6]`, `counterparty: 3`), case décochée → la requête porte `amount` et `counterparty: 3`, et
  `JSON.stringify(req)` ne contient pas `debit_credit_split` ; (v9) création (pas de profil existant) → la requête ne
  porte que les clés du formulaire.

**AC9 — Aperçu d'import (`BankImportUpload.svelte`).** Rien de neuf à afficher : `warning-unsupported-currency` et
`warning-informational` existent (l. 596, 617). Seul le texte de l'avertissement change (AC10). `csv-line-error-label.ts`
gagne `INVALID_CURRENCY: ['invalid-currency', '<repli fr>']`, et **`csv-line-error-label.test.ts`** sa ligne dans la
table `attendu` : ce test compare l'ensemble des clés à l'égalité **exacte**, il rougit dès le code ajouté (validation
P1, R3-1).

### Les textes

**AC10 — Quatre locales** (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`), clés présentes dans les
**quatre** fichiers ; `frontend/src/lib/shared/i18n-keys.test.ts` (dont `sitesTotal`, l. 575, **recompté** par le
test, jamais incrémenté de confiance ; la famille `'bank-import-info-'` l. 628 et son compte l. 680 ; la famille
`'bank-import-csv-errors-'` l. 630), `i18n-un-repli-par-cle.test.ts` (`MOTIFS_DYNAMIQUES`, cf. commentaire de
`csv-line-error-label.ts`) et `npm run lint-i18n-ownership` verts. Les deux gestes qu'impose `i18n-keys.test.ts`
(validation P2, R6) : la famille `'bank-import-info-'` gagne `'bank-csv-currency-assumed'` (trois valeurs) **et** son
compte écrit passe de `2` à `3` (l. 680) ; la famille `'bank-import-csv-errors-'`, énumérée en dur (l. 630-633), gagne
`'invalid-currency'`, faute de quoi elle rougit contre sa cardinalité lue de `CSV_LINE_ERROR_LABELS` (l. 682).

**Garde G9 des replis Rust** (`crates/kesh-api/tests/textes_coherents.rs`, `les_replis_rust_suivent_le_catalogue`,
l. 679 — table fermée ; validation P2, R6) : l'entrée `crates/kesh-api/src/errors.rs` gagne
`("bank-import-errors-unsupported-currency", 1, &[])` et `("bank-import-csv-errors-mixed-currencies", 1, DEVISES)`,
avec `const DEVISES: &[(&str, &str)] = &[("{ $currencies }", "{currencies}")];` — le repli de la seconde est un
`format!` dont la variable s'appelle `currencies` (patron de `NOM`). C'est ce qui donne un test à l'affirmation
« replis identiques aux textes `fr-CH` » ci-dessous.

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
*devise* → le nouveau texte de l'avertissement). S'y ajoutent (validation P2, R7 ; C-27-2-19) le message du même
refus côté `kesh-core`, qui va au journal — `crates/kesh-core/src/errors.rs:93`
(`#[error("devise non supportée v0.1 : {0} (seul CHF est accepté)")]` → `"devise non supportée : {0} (seul CHF est
accepté)"`) et son doc-comment `:91` — et les doc-comments qui décrivent ce refus : `crates/kesh-import/src/error.rs:5`,
`crates/kesh-core/src/bank_imports.rs:25` et `:457` (« v0.1 » → « cette version »). Le **nom**
`validate_currency_supported_v0_1` reste (C-27-2-3), et les mentions historiques « v0.1 mono-CHF » des tests et de la
réconciliation restent : elles décrivent une convention toujours vraie, non ce refus. **Liste fermée des sites gardés**
(validation P3, R2 ; C-27-2-21), c'est-à-dire ce que rend `grep -rnF "v0.1" crates --include=*.rs | grep -i
"devise\|currency\|chf"` une fois le patch appliqué — rien de plus, rien de moins :
`crates/kesh-api/src/routes/bank_imports.rs:1125`, `crates/kesh-api/src/routes/reconciliation.rs:628`,
`crates/kesh-api/tests/bank_imports_e2e.rs:443`, `crates/kesh-api/tests/reconciliation_e2e.rs:15`, `:1974`, `:1975`,
`:1981`, `:2253`, `crates/kesh-db/src/repositories/reconciliation.rs:9` et `:104` (colonne `invoices.currency` absente,
filtre absent : conventions vraies), `crates/kesh-core/src/bank_imports.rs:725` (message d'un `expect` de `mod tests`,
gardé). Le dev **recopie la sortie** du grep au Dev Agent Record ; une ligne hors de cette liste est un site oublié.
⚠️ **`reconciliation_e2e.rs:2253-2254` se récrit en partie** (C-27-2-21) : sa ligne « (le parser CSV rejette EUR/USD,
cf. AC bank_imports `BANK_IMPORT_UNSUPPORTED_CURRENCY`) » est **fausse** aujourd'hui — c'est le défaut de #624 — et ne
devient vraie que pour un profil qui lit la colonne Devise : la parenthèse devient « (le parseur CSV refuse un relevé
non CHF quand le profil lit la colonne Devise — Story 27-2 ; sans elle, CHF supposé) » ; « v0.1 mono-CHF » y reste.
**Greper la valeur** après coup : `grep -rnF "v0.1" frontend/src/lib/features/bank-import crates/kesh-i18n/locales
docs/i18n-glossaire.md` ne rend plus `unsupported-currency` ni `Devise non supportée`, et `grep -rniF "devise non
supportée v0.1" crates` ne rend rien.

**Gardes des replis côté écran** (validation P3, R7) : `frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts`
croise chaque repli `i18nMsg` avec `fr-CH` et rougit à la moindre divergence — c'est lui qui attrape le repli Svelte de
`bank-import-warnings-unsupported-currency` (`BankImportUpload.svelte:604`) et ceux des clés neuves du formulaire ; il
reste vert sans entrée neuve à son allowlist `TOLEREES`. **Apostrophe** : les textes `fr-CH` neufs ou modifiés et leurs
replis (Rust et Svelte) emploient l'apostrophe **droite** `'`, comme les clés voisines
(`bank-import-csv-errors-encoding-mismatch`, `-partial-failure`, `-header-skips-data`) ; G9 compare octet pour octet.

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
  la phrase neuve, **fixée ici** (validation P2, F9) : la note remplacée contient mot pour mot « le relevé est supposé
  en CHF » ; même aplatissement, `grep -oE "le relevé est supposé en CHF" | wc -l` → **0 avant**, **≥ 1 après** (la
  même phrase peut revenir dans une autre puce : un compte exact échouerait sans défaut).

**AC12 — CHANGELOG** (`CHANGELOG.md`, section `## [0.13.2] — Non publié`, créée ici sous `---` en tête si absente — la
branche de la 27-1 la porte déjà : l'orchestrateur fusionnera). Le texte **à publier** est rangé dans les rubriques du
patron de la 0.13.1 — un bloc « ⚠️ **À faire en passant à la 0.13.2** » en tête de section, liste numérotée, puis
`### Modifié` et `### Corrigé` — et ne porte **aucun** renvoi de traçabilité (validation P3, R8 ; C-27-2-24) : ceux-ci
sont regroupés dans les *Notes pour le dev*, qui suivent et **ne se publient pas**.

*Texte à publier* (reformulable, le sens est prescrit) :

- Bloc **⚠️ À faire en passant à la 0.13.2** (renvoi au manuel d'administration, *Passer à la 0.13.2 : la colonne
  Devise des profils CSV*) :
  1. **Ajouter la colonne Devise** (4 pour UBS) aux profils existants dont le fichier en porte une : *Administration →
     Profils bancaires*. Sans elle, le relevé reste **supposé en CHF**, comme avant, et l'aperçu le dit.
  2. **Transactions déjà importées** : la mise à jour ne les corrige pas, et réimporter le même fichier ne les remplace
     pas (fichier signalé en doublon, lignes signalées en doublon : l'empreinte de ligne ignore la devise). Remède :
     **supprimer l'import** (fonction de la 0.13.2, permise tant qu'aucune de ses transactions n'est rapprochée), puis
     le réimporter. Si l'import a des transactions rapprochées, la suppression est refusée
     (`409 BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS`) : annuler d'abord leur rapprochement (*Annuler le rapprochement*,
     qui contre-passe l'écriture ou annule le règlement qu'il avait produit), puis supprimer et réimporter. CAMT : une
     transaction restée « en EUR » n'était proposée ni au rapprochement automatique ni par une règle, mais le
     rapprochement manuel, l'éclatement et l'acceptation d'une facture l'admettaient — si l'une d'elles a été
     rapprochée ainsi, la même règle s'applique : annuler d'abord son rapprochement. CSV d'un compte en devise : vérifier
     les montants ; réimporté avec la colonne Devise réglée, le relevé est **refusé** (devise non supportée) — il n'a
     pas sa place dans Kesh, qui ne tient que des comptes en CHF.
  3. **Retour à une 0.13.1** : aucune migration ne l'interdit, mais **un seul** profil qui lit la colonne Devise fait
     échouer `GET /api/v1/bank-profiles` (500) pour **toute la société** sous la 0.13.1 : l'écran *Profils bancaires*
     est inutilisable, et l'écran d'import affiche un sélecteur de profil **vide, sans message** ; les imports par ce
     profil sont refusés (rien n'est importé à tort). Le profil ne se corrige plus à l'écran sous la 0.13.1 :
     **retirer la colonne Devise de tous les profils avant le retour** (sous la 0.13.2). Oublié, il se répare encore
     **par l'API** de la 0.13.1 : un `PUT /api/v1/bank-profiles/{id}` complet sans `columnMapping.currency`, ou un
     `DELETE`. L'identifiant et les réglages du profil, que la 0.13.1 ne rend plus par `GET`, se lisent dans **l'export
     global** de la 0.13.1 (`bank_profiles.csv`, colonnes `id` et `column_mapping_json`), qui recopie le profil sans
     le lire. Le SQL reste le dernier recours. Une sauvegarde d'installation prise sous la 0.13.2 s'importe dans une
     0.13.1, avec la même conséquence.
- `### Modifié` — **API des profils bancaires** : `columnMapping.currency` (facultatif) dans `POST`/`PUT`/`GET
  /api/v1/bank-profiles` ; la colonne `column_mapping_json` de l'export global peut porter la clé `currency`.
- `### Corrigé` — **La devise d'un relevé CSV est lue, et un relevé qui n'est pas en francs suisses est refusé
  ([#624](https://github.com/guycorbaz/kesh/issues/624))**. Avant, tout import CSV était enregistré en CHF, au montant du fichier, sans avertissement. Profil :
  colonne Devise facultative ; aperçu : avertissement de devise non supportée, *Confirmer* grisé ; import :
  `422 BANK_IMPORT_UNSUPPORTED_CURRENCY` ; relevé à plusieurs devises : `422 BANK_CSV_MIXED_CURRENCIES`
  (`details.currencies`), non confirmable ; cellule vide ou illisible : ligne invalide (`EMPTY_MANDATORY_FIELD`,
  `INVALID_CURRENCY`). Sans colonne Devise, le relevé reste supposé en CHF et l'aperçu le dit.
- `### Corrigé` — **La devise d'une transaction CAMT.053 est celle de son montant.** Avant, la devise d'un montant
  d'information — montant d'origine d'un paiement par carte (`<AmtDtls>`), frais (`<Chrgs>`) — remplaçait celle de la
  transaction quand il était le **dernier** montant de la transaction à porter une devise : sur un compte en CHF, un
  paiement en euros **pouvait** ainsi être enregistré « en EUR » au montant en francs, et la réconciliation ne le
  proposait plus d'elle-même.
- `### Corrigé` — **Modifier un profil bancaire à l'écran ne perd plus sa colonne contrepartie**, réglable par l'API
  seulement : le formulaire la supprimait au premier enregistrement.

*Notes pour le dev* (à ne pas publier) :

- Traçabilité : bloc 1 et paragraphe API — AC1, AC7, C-27-2-1 ; bloc 2 — validation P2, F2, et P3, F1 = R1 ;
  C-27-2-16, C-27-2-20 ; bloc 3 — validation P1 (portée réelle de C-27-2-1), P2, F7, et P3, F2 (C-27-2-22) ; CAMT —
  AC19, validation P2, R8 (un `<AmtDtls>` complet, qui finit par `<TxAmt>` en devise du compte, n'exposait pas au
  défaut : d'où « pouvait » et « dernier ») ; contrepartie — AC8-bis.
- Preuves du bloc 3 : `git show v0.13.1:crates/kesh-db/src/repositories/bank_profiles.rs`, l. 264 (`PUT` : comparaison
  de la chaîne brute ; `DELETE` idem) ; `git show v0.13.1:crates/kesh-api/src/exports/csv_tables.rs`, l. 1041-1075
  (`serialize_bank_profiles_csv` recopie `column_mapping_json` tel quel, `id` en première colonne) et
  `exports/global.rs:320` (`bank_profiles::list_all_by_company`, sans désérialisation) — lus en validation P3, non
  exécutés.
- ⚠️ Le bloc 3 ne vaut que si **aucune** migration de la 0.13.2 ne relève `kesh_version_min_required` (validation P2,
  F10) : vrai pour les branches 27-1, 27-2 et 27-3 au 2026-10-12, à revérifier par l'orchestrateur à la fusion. La
  27-1 porte déjà un `## [0.13.2]` : l'orchestrateur réunit les deux blocs « À faire » en une seule liste numérotée.
- Lien d'issue **en ligne**, patron de la 0.13.1 : `([#624](https://github.com/guycorbaz/kesh/issues/624))`. Le
  CHANGELOG n'a aucune définition de lien de référence (`grep -nE "^\[#[0-9]+\]:" CHANGELOG.md` → aucune sortie,
  112 liens en ligne) : un `([#624])` nu y serait un lien mort.

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
  colonne contrepartie ; un paragraphe **« Transactions déjà importées »** sur le patron de celui de la 0.13.1
  (`admin-manual.tex:1993`), qui porte le remède de l'AC12 (validation P2, F2 ; C-27-2-16) : la mise à jour ne corrige
  rien, le réimport seul non plus ; supprimer l'import non rapproché puis le réimporter ; s'il est rapproché, annuler
  d'abord le rapprochement de ses transactions (renvoi au manuel utilisateur, *Annuler le rapprochement*) ; une
  transaction CAMT restée « en EUR » n'était pas proposée au rapprochement automatique ni par une règle, mais le
  rapprochement manuel, l'éclatement et l'acceptation d'une facture l'admettaient : rapprochée ainsi, elle suit la
  même règle (validation P3, F1 = R1 ; C-27-2-20) ; un CSV d'un compte en devise réimporté avec la colonne Devise est
  refusé ; un paragraphe **« Retour à la 0.13.1 »** qui porte la réserve de l'AC12 (un profil à colonne Devise rend la
  liste des profils de toute la société en erreur sous la 0.13.1, sélecteur d'import vide sans message : retirer la
  colonne de tous les profils **avant** le retour ; oublié, le profil se répare par l'API de la 0.13.1 — `PUT` complet
  sans `columnMapping.currency`, ou `DELETE` —, identifiant et réglages lus dans l'export global de la 0.13.1
  (`bank_profiles.csv`), le SQL en dernier recours ; validation P2, F7, et P3, F2 ; C-27-2-22) ;
- § *Rollback en cas d'échec* (`:2223`, « le retour à la version précédente n'est possible que si aucune autre migration
  de la release ne relève la version minimale exigée ») : y porter la réserve, par renvoi à `\S\ref{sec:maj-0-13-2}` —
  pour la 0.13.2, le retour exige en plus que les profils CSV ne lisent plus la colonne Devise. ⚠️ Cette réserve et le
  paragraphe « Retour à la 0.13.1 » supposent qu'**aucune** migration de la 0.13.2 ne relève
  `kesh_version_min_required` (vrai pour les branches 27-1, 27-2 et 27-3 au 2026-10-12 : `git diff --stat a2743fd4
  <branche> -- crates/kesh-db/migrations` vide) ; si une story de la release en ajoute une, le retour devient
  impossible et le paragraphe se récrit — à revérifier par l'orchestrateur à la fusion (validation P2, F10) ;
- contrôle PDF aplati (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`) : `grep -oE "Passer à la
  0.13.2"`, `grep -oE "colonne Devise"` et `grep -oE "supprimer l.import"` rendent chacun au moins 1 (le troisième :
  le remède des transactions déjà importées).
- ⚠️ Fusion : la branche de la 27-1 modifie le même fichier (`:1993`, `:2270`, `:2305`, `:2463`) sans créer cette
  section ; l'orchestrateur réunit les apports de la 27-1 et de la 27-3 sous la même `\subsubsection` à la fusion.

### Les preuves

**AC13 — Tests du parseur** (`crates/kesh-import/src/csv/parser.rs`, `mod tests` ; fixture
`include_bytes!(…"/tests/fixtures/csv/ubs_synthetique.csv")`, dont la colonne 4 vaut `CHF` sur les 11 lignes ;
variantes **dérivées dans le test** par remplacement de `;CHF;` **sur les seules lignes de données** — C-27-2-9. Le
préambule porte aussi `;CHF;` (`Évaluation en:;CHF;`, ligne 7 : 12 occurrences pour 11 lignes de données) : le helper
de dérivation saute les lignes du préambule et de l'en-tête et asserte qu'il a remplacé exactement le nombre de
cellules voulu — validation P1, R9-1). ⚠️ **Sauter `header_row_count` lignes NON VIDES, pas lignes physiques**
(validation P2, R5) : le profil UBS porte `header_row_count = 9`, mais la fixture a **10** lignes physiques avant la
première donnée — la ligne 9 est vide, et le lecteur `csv` l'avale sans la compter (doc-module de `parser.rs`, l. 14-17 :
« une ligne vide […] n'entre pas dans le compte (relevé UBS : 9) »). Un helper qui sauterait 9 lignes physiques prendrait
la ligne de titres pour la première donnée et décalerait d'une unité les indices de AC13-11 et AC13-12. Le helper
applique donc la règle du lecteur (ignorer les lignes vides, compter les autres). ⚠️ **« Vide » au sens du lecteur =
zéro caractère** une fois le terminateur (`\n` ou `\r\n`) retiré — `line.is_empty()`, **jamais** `line.trim().is_empty()`
(validation P3, R4 ; C-27-2-23) : une ligne d'espaces ou de seuls `;` est un enregistrement pour `csv::ReaderBuilder`
(`parser.rs:191-195`) et compte dans `header_row_count`, comme le dit le manuel (*Profils banque*). La fixture n'a pas
une telle ligne, aucun test ne rougirait : la règle s'écrit au doc-comment du helper. L'assertion du nombre de
remplacements reste :

1. profil UBS (26-1) + `currency: Some(4)` → `AllValid`, `stmt.currency == "CHF"`, les 11 `tx.currency == "CHF"` ;
2. même profil, fixture passée en `EUR` sur **toutes** les lignes → `AllValid`, `stmt.currency == "EUR"`, chaque
   `tx.currency == "EUR"` (le parseur reste agnostique ; le refus est en AC14) ;
3. **une seule** ligne passée en `EUR` → `HardFailure(MixedCurrencies { currencies: ["CHF", "EUR"] })` ;
4. une ligne en `eur` sur un fichier tout `eur` → `"EUR"` (casse) ;
5. une cellule vide → `PartialFailure`, une erreur `EMPTY_MANDATORY_FIELD` valeur `"currency"`, 10 valides en CHF ;
6. une cellule `Fr.` → `PartialFailure`, une erreur `INVALID_CURRENCY` ; `€`, `CHF1`, **`ÉUR`**, **`ÉU`**, **`CH1`**
   et **`CH`** idem — `ÉUR` (trois caractères, quatre octets) tue un contrôle « alphabétique Unicode sur trois
   caractères », `ÉU` (deux caractères, trois octets) un contrôle « alphabétique Unicode sur trois octets », que `Fr.`,
   `€` et `CHF1` laisseraient vivre (validation P1, R6-1) ; `CH1` tue un contrôle « trois caractères ASCII
   **alphanumériques** », `CH` une borne « **au plus** trois lettres » (validation P2, F3 = R3) ;
7. mélange **et** erreur de ligne dans le même fichier → `MixedCurrencies` l'emporte ;
8. un débit de signe contraire **et** une **autre** ligne, **valide**, passée en `EUR` → `AmountSignMismatch`
   l'emporte (ordre de l'AC3). ⚠️ Pas la même ligne (validation P2, R4) : une ligne de signe contraire sort avant la
   lecture de sa devise (`parser.rs:597-602` pour un débit, `:629` pour un crédit — et non `:578-583`, cité à tort
   par le rapport R de P2) : il n'y aurait plus de mélange parmi les lignes valides, et M12 ne rougirait pas ;
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
   `200` sans `warnings.unsupportedCurrency`, puis `201` ; les lignes de `bank_transactions` de l'import sont **trois**
   (une par entrée — compte asserté, sinon « chaque » passerait sur une liste vide) et **chacune** en
   `currency = 'CHF'` — la valeur traverse la frontière HTTP et la base.

**AC15 — E2E Playwright** (`frontend/tests/e2e/bank-csv-import.spec.ts`, patron des tests UBS de la 26-1, l. 260-400 ;
`createProfileViaUi` gagne la colonne `col-currency-input`) :

1. profil UBS avec devise 4 créé **par l'écran** — nom de banque **propre** `UBS synthétique — devise`, motif **propre**
   `^ubs_synthetique_devise\.csv$` —, fichier **propre** importé : `ubs_synthetique_devise.csv`, **dérivé en mémoire**
   (`setInputFiles({ name, mimeType, buffer })`) de `ubs_synthetique.csv` en décalant d'une année les dates des **seules
   lignes de données** (`2026-` → `2025-` dans les quatre colonnes de date) → aperçu `preview-tx-count` = `11`, succès,
   puis par l'API, l'import retrouvé **par ce nom de fichier** : `expect(transactions).toHaveLength(11)` **et** chaque
   `currency === "CHF"` ; `warning-informational` ne porte **pas** `data-info-code="bank_csv_currency_assumed"`.
   ⚠️ **Pourquoi pas `ubs_synthetique.csv` tel quel** (validation P3, R5 ; C-27-2-25) : le test voisin « imports the UBS
   CSV as is » (`bank-csv-import.spec.ts:315-356`) l'importe déjà, sur le même compte (`ensureBankAccount`) et dans la
   même base (`seedTestState` une fois, `beforeAll`, l. 28) — mêmes octets → avertissement de fichier en doublon,
   *Confirmer* grisé tant que la case n'est pas cochée ; mêmes lignes → doublons ignorés, import à **0** transaction, et
   une assertion « chaque `currency == "CHF"` » sur une liste vide passe **à vide** ; même motif de profil → deux
   profils candidats, le plus récent gagne. Décaler les dates change l'empreinte de chaque ligne
   (`dedup_key_scalar`, `bank_imports.rs:1532-1539`, porte la date) et les octets du fichier. Si une contrainte de période
   refusait 2025, choisir un autre décalage : l'exigence est qu'**aucune** ligne n'ait l'empreinte d'une ligne du test
   voisin, et le `toHaveLength(11)` la contrôle ;
2. même profil, fichier EUR **dérivé en mémoire** du buffer de (1) (même nom `ubs_synthetique_devise.csv`, `;CHF;` →
   `;EUR;` sur les seules lignes de données) → aperçu `warning-unsupported-currency` visible, `bank-import-confirm`
   désactivé, `importTotal` inchangé (aperçu seul, rien d'importé) ;
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
| M9 | forme non contrôlée (`Fr.` acceptée), ou contrôle alphabétique Unicode (`ÉUR` accepté) / sur la longueur en octets (`ÉU` accepté) / ASCII alphanumérique (`CH1` accepté) / longueur `≤ 3` (`CH` accepté) | AC13-6 |
| M10 | `currency` absent de `all_indices()` | AC1-(1), AC13-10 |
| M11 | `skip_serializing_if` retiré du champ | AC1-(2) |
| M12 | `MixedCurrencies` évalué avant le refus de signe | AC13-8 |
| M13 | `bank_csv_currency_assumed` toujours poussé / jamais poussé | AC14-5 / AC14-6, AC15-1 / AC15-3 |
| M14 | le formulaire n'envoie pas `currency` | AC8-(v1)(v2), AC15-1 |
| M15 | le formulaire ne relit pas `existing.columnMapping.currency` | AC8-(v4) |
| M16 | `map_csv_error` sans bras `MixedCurrencies` (repli `other`) | AC14-4 (code ≠ `BANK_CSV_MIXED_CURRENCIES`) |
| M17 | plafond des devises rapportées retiré, ou troncature aux 10 premières **rencontrées** | AC13-11 |
| M18 | l'erreur de devise rendue par `RowParse::err` (observation de débit perdue) | AC13-12 |
| M19 | CAMT : la devise de transaction reprise de **tout** `<Amt Ccy>` sous `<TxDtls>` (code d'avant) | AC19-1 |
| M20 | CAMT : la devise d'entrée reprise de tout `<Amt Ccy>` sous `<Ntry>` hors `<TxDtls>` (code d'avant) | AC19-2 |
| M21 | le formulaire ne recopie pas les clés qu'il ne montre pas (`keepUnownedKeys` retiré du spread) | AC8-bis-(v6) |
| M22 | le formulaire recopie **tout** le mapping existant, sans en retirer ses propres clés | AC8-bis-(v7), (v8) |
| M23 | `#[serde(deny_unknown_fields)]` retiré de `ColumnMapping` | AC1-(5) |
| M24 | `OWNED_COLUMN_MAPPING_KEYS` privée de `debit_credit_split` (resp. de `amount`) | AC8-bis-(v8) (resp. (v7)) |
| M25 | CAMT : la **première** devise rencontrée sous `<TxDtls>` gagne (n'écrase que si `None`), au lieu de l'ancrage sur `path.last()` | AC19-(1-bis) (seule : (1) a son `Amt` direct CHF en tête et resterait verte) |

⚠️ **Mutants équivalents, déclarés** (validation P3, F6 = R3 ; C-27-2-26) : retirer `date`, `reference`, `details` ou
`currency` de `OWNED_COLUMN_MAPPING_KEYS` est **sans effet observable** — le formulaire écrit ces quatre clés dans ses
**deux** modes (`clé: valeur ?? undefined`, `BankProfileForm.svelte:56-69`), et le spread qui suit écrase la valeur
recopiée, `undefined` compris. Aucun cas (v1)-(v9) ne peut les tuer, et aucun ne le doit : ce ne sont pas des
survivants. Seules `amount` et `debit_credit_split`, écrites dans un seul mode, rendent la constante nécessaire — d'où
M24, qui ne vise qu'elles.

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
et les propositions automatiques de la réconciliation (`reconciliation.rs:631`, `:780`) comme l'acceptation d'une
règle (`:2728`) l'écartent ensuite ; le rapprochement manuel, l'éclatement et l'acceptation d'une proposition de facture,
eux, ne contrôlent pas la devise et l'admettent (validation P3, F1 = R1 ; inventaire (i), C-27-2-20).

- Correction : à l'ouverture d'un `Amt`, `path` ne contient pas encore `Amt` (il est poussé après le `match`) ; la
  devise est retenue **seulement** si `path.last()` vaut `"TxDtls"` (pour `t.currency`) ou `"Ntry"` (pour
  `n.currency`) — exactement l'élément dont le montant est lu. Tout autre `<Amt Ccy>` est ignoré pour la devise. Un
  commentaire au site dit pourquoi (la devise d'un montant se lit sur l'élément du montant ; les montants d'information
  portent la devise d'origine) et renvoie aux deux ancrages du montant, qu'il ne faut pas laisser diverger.
- Repli inchangé : un `<TxDtls>` sans `Amt` propre prend le montant **et** la devise de l'entrée (`:603`, `:618`).
- **Fixture synthétique** neuve `crates/kesh-import/tests/fixtures/camt053/v04_amtdtls_foreign_currency.xml`, sur le
  patron de `v04_with_subtxs.xml` (même IBAN de test, compte `CHF`, soldes d'ouverture et de clôture cohérents avec les
  entrées ; ordre des éléments de `EntryTransaction10` dans le seul XSD versionné, celui de la 001.08 —
  `docs/six-references/ig-cash-managment-xml-schemas-v2.0.2-en/camt.053.001.08.xsd:705` : `Refs`, `Amt`, `CdtDbtInd`,
  `AmtDtls`, `Avlbty`, `BkTxCd`, `Chrgs`… —, que le parseur lit sans distinction de version ; aucun XSD 001.04 n'est au
  dépôt, la fiche n'en affirme donc pas l'ordre — validation P2, R9) : entrée 1 avec un
  `<TxDtls>` `<Amt Ccy="CHF">` suivi de `<AmtDtls><InstdAmt><Amt Ccy="EUR">` (et son `CcyXchg`) et de
  `<Chrgs><Rcrd><Amt Ccy="EUR">` ; entrée 2 **sans** `<TxDtls>`, `<Amt Ccy="CHF">` suivi de
  `<AmtDtls><InstdAmt><Amt Ccy="EUR">` ; **entrée 3** (validation P3, F3 ; C-27-2-27) : `<Ntry>` à `<Amt Ccy="CHF">`
  direct, et un seul `<TxDtls>` **sans** `Amt` propre qui porte `<AmtDtls><InstdAmt><Amt Ccy="EUR">` — la forme
  **dominante** au dépôt (les entrées à un seul `<TxDtls>` des fixtures existantes n'ont pas d'`Amt` direct ; seules
  `v04_with_subtxs.xml` et `v04_ntry_sign_fallback.xml` en portent un) et le cas exact du défaut. Valeurs inventées.
- **Tests** (`crates/kesh-import/tests/camt053_tests.rs`) : (1) la transaction de l'entrée 1 a `currency == "CHF"` et le
  montant de `TxDtls/Amt` ; (1-bis) celle de l'entrée 3 a `currency == "CHF"` (repli sur l'entrée, `:618`) et le
  montant de l'entrée (`:603`) — elle seule tue M25 ; (2) celle de l'entrée 2 a `currency == "CHF"` et le montant de
  `Ntry/Amt` ; (3)
  non-régression : `parse_eur_currency_preserved` (compte et transaction EUR) et
  `parse_with_subtxs_extracts_individual_transactions` restent verts sans modification. ⛔ (1), (1-bis) et (2) sont écrits
  **avant** la correction et lancés sur le code d'avant : ils doivent rougir (`"EUR"` au lieu de `"CHF"`) — le rouge est
  noté au Dev Agent Record, sinon le test ne prouve rien. Test d'intégration : AC14-8.
- L'en-tête de `camt053_tests.rs` (« Les 10 fixtures couvrent », pour 13 fichiers à `a2743fd4`) est **recompté** depuis
  `ls crates/kesh-import/tests/fixtures/camt053/ | wc -l` et gagne la ligne de la fixture neuve. **Même geste** au
  catalogue `crates/kesh-import/tests/fixtures/README.md:25-39` (validation P2, R6) : « Les 10 fichiers » recompté, une
  ligne pour la fixture neuve, et une pour chacun des trois fichiers qui y manquent déjà (`v04_ntry_sign_fallback.xml`,
  `v04_overlap.xml`, `v04_truncated_after_stmt_close.xml`) ; la ligne de `v04_minimal.xml` (« 1 Ntry sans TxDtls »,
  `README.md:30`) est **corrigée** : le fichier porte un `<TxDtls>` (`grep -c "<TxDtls>"` → 1 ; validation P3, R6).
- ⚠️ **Angle mort assumé** (validation P2, F8 ; C-27-2-11) : une entrée dont l'`Amt` **direct** n'a pas de `Ccy`, mais
  dont un montant d'information en porte un, prenait jusqu'ici la devise de ce dernier ; elle est désormais refusée
  (`MissingRequiredField(currency)`, `camt053/mod.rs:572-575`) — et avec elle **tout le relevé**. Le XSD rend `Ccy`
  obligatoire (exporteur non conforme), le refus est bruyant, et prendre la devise d'un montant d'information serait le
  défaut même. Pas de test exigé.
- Ni `validate_currency_supported_v0_1` ni le contrôle par transaction ne changent (C-27-2-3, alternative toujours
  écartée).

## Tâches

- [ ] **T0** — Brancher dans le worktree `/home/gcorbaz/devel/kesh-27-2` (fait : `story/27-2-devise-import-csv`) ;
  `node_modules` par `cp -al` depuis le dépôt principal ; `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-27-2/target` ;
  bases `kesh_272` / `kesh_e2e_272` créées, migrées, seedées.
- [ ] **T1** — `kesh-import` : AC1, AC2, AC3, AC4, AC19 ; tests AC1, AC13 et AC19 (AC19-1/2 rouges d'abord) ; mutations
  M1, M2, M6-M12, M17-M20, M23, M25 ; catalogue `tests/fixtures/README.md` (AC19).
- [ ] **T2** — `kesh-api` : AC5, AC6, AC7 (helper de contrôle factorisé, variant d'erreur, bras explicite) ; tests
  AC14 (dont AC14-8, CAMT) ; mutations M3-M5, M13, M16 ; textes « v0.1 » de `errors.rs` et de `kesh-core`, parenthèse fausse de `reconciliation_e2e.rs:2253-2254` (AC10) ;
  garde G9 de `textes_coherents.rs` (AC10).
- [ ] **T3** — i18n : AC10 (quatre locales, gardes Vitest et lint).
- [ ] **T4** — Frontend : AC8, AC8-bis, AC9 ; Vitest (dont `csv-line-error-label.test.ts`) ; mutations M14, M15,
  M21, M22, M24.
- [ ] **T5** — E2E : AC15.
- [ ] **T6** — `ubs_fichier_reel.rs` : AC16 (compilé, **non lancé** par l'agent).
- [ ] **T7** — Manuel utilisateur (AC11) et manuel d'administration (AC12-bis), PDF régénérés + contrôles aplatis ;
  `docs/i18n-glossaire.md` (AC10) ; CHANGELOG (AC12) ; `README.md` « Feuille de route » vérifié
  (aucun changement attendu) ; `website/` vérifié (aucune promesse sur la devise CSV attendue : `grep -rni "devise\|currency" website/`).
- [ ] **T8** — Propagation : `grep -rnF '"CHF"' crates/kesh-import/src` (AC4) ; `grep -rnE "n.est pas lue" docs/manual/fr
  CHANGELOG.md` (la phrase de 0.13.1 dans le CHANGELOG **reste** : c'est l'histoire) ; `grep -rnF "v0.1"` sur les
  clés et sites modifiés (AC10) ; `grep -rn "counterparty" frontend/src/lib/features/bank-import` (AC8-bis) ; gates
  AC18 ; mutations M1-M25 ; sortie du grep « v0.1 » recopiée (liste fermée, AC10) ; Dev Agent Record.

## Dev Notes

### État actuel des fichiers modifiés (lus le 2026-10-12 sur `a2743fd4`)

- `crates/kesh-import/src/csv/profile.rs` (571 l.) — `ColumnMapping` (l. 37-60) en `deny_unknown_fields`, champs
  facultatifs en `skip_serializing_if` + `default` ; `all_indices` (l. 65-84), `max_index` (l. 88-94) ; `validate`
  (l. 190-310). **À préserver** : l'ordre et les messages des contrôles existants, `DebitSign` intact.
- `crates/kesh-import/src/csv/parser.rs` (2056 l.) — `parse_csv_collect_inner` (l. 157-392) : validate, encodage, saut
  des lignes, garde de surcomptage, premier enregistrement, hors-borne, boucle, refus de signe (7.b, l. 303-315),
  construction du relevé (l. 357-367), discrimination (l. 369-391). `parse_row` (l. 519-690, doc l. 501-518).
  `RowAccumulator` (struct l. 451-461, `impl` l. 463-500). **À préserver** : la priorité du refus de signe, `whole_debit_column`, la sentinelle de période, les
  plafonds `MAX_CSV_LINE_ERRORS`.
- `crates/kesh-import/src/error.rs` — `CsvError` (l. 59-198), `CsvLineErrorCode` (l. 204-265), `all_is_complete`
  (l. 379).
- `crates/kesh-api/src/routes/bank_imports.rs` (1999 l.) — `map_csv_error` (l. 624-673), `db_profile_to_csv_profile`
  (l. 1590-1607, inchangé : il passe le `ColumnMapping` entier), `preview` CAMT (l. 896-1043, bloc devise
  l. 961-971 à factoriser), `create` CAMT (l. 1048-…, l. 1126-1130), `preview_csv` (l. 1609-1756), `create_csv`
  (l. 1758-1999 ; branche `PartialFailure` l. 1825-1861, `if !fields.confirm_partial_import` l. 1831).
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

- **27 littéraux `ColumnMapping { … }`** à compléter de `currency: None` (`grep -rn "ColumnMapping {" crates` rend
  29 lignes, dont la définition `profile.rs:38` et le bloc `impl` `:62` : 23 littéraux dans `parser.rs`, 3 dans
  `profile.rs` — l. 329, 538, 557 —, 1 dans `tests/ubs_fichier_reel.rs` ; recompté en validation P2, F6) — le
  compilateur les trouve, mais
  `ubs_fichier_reel.rs` est un test **ignoré** : `cargo build --workspace --all-targets` le compile, `nextest` ne
  l'exécute pas.
- **Ordre des refus**, à tenir exactement (CSV, `create_csv`) : fichier déjà importé (avant parse) → profil introuvable
  (`404 BANK_CSV_NO_PROFILE_MATCH`, résolution du profil) → **[27-3]** `409 BANK_IMPORT_PREVIEW_STALE` (après la
  résolution du profil, **avant tout parse** — fiche 27-3, AC8) → erreurs fatales du parseur → refus de signe → mélange
  de devises → [API] devise non supportée → rejet partiel (`confirmPartialImport`) → zéro ligne valide. Côté CAMT
  (`create`), le `409` de la 27-3 précède au contraire le `422 BANK_IMPORT_DUPLICATE_FILE` (asymétrie assumée par la
  27-3, C-27-3-24) ; le contrôle de devise CAMT n'en bouge pas. Validation P3, F4 ; C-27-2-28.
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
- Fusion avec la 27-3 (`story/27-3-profil-apres-apercu` ; validation P2, F10) : elle touche les mêmes sites —
  `preview_csv` / `create_csv` (champ `previewFingerprint`, refus `409 BANK_IMPORT_PREVIEW_STALE`, **inséré à sa place**
  dans l'ordre des refus ci-dessus, que la 27-2 tient), `i18n-keys.test.ts` (`sitesTotal`), les `messages.ftl`, le manuel
  utilisateur aux lignes voisines (`:1794-1807`). Son empreinte couvre `column_mapping_json` en chaîne brute : la clé
  `currency` y entre d'elle-même, sans interaction défectueuse. Aucune migration portée par la 27-1 ni la 27-3 au
  2026-10-12 — condition du paragraphe « Retour à la 0.13.1 » (AC12, AC12-bis).

### Ce qui reste hors périmètre

La lecture du préambule UBS (#621) ; une devise par compte bancaire (multi-devises) ; un **contrôle** de la devise de
chaque transaction contre le périmètre CHF (C-27-2-3) ; une garde de devise sur le rapprochement manuel, l'éclatement et
l'acceptation d'une proposition de facture (C-27-2-20 — défaut connu du dépôt, `reconciliation_e2e.rs:2241-2262`) ; un **champ** contrepartie au formulaire (la clé est désormais
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

### Validation P2

2026-10-12. Passe **complète**, **Opus**, deux lentilles en contexte frais et en lecture seule (prompt
`27-2-validate-prompt-p2.md`) : **R** — 0 CRITICAL, 0 HIGH, **1 MEDIUM**, 10 LOW ; **F** — 0 CRITICAL, 0 HIGH,
**2 MEDIUM**, 8 LOW. Total **3 MEDIUM, 18 LOW** (dont trois paires convergentes : F1 = R10, F3 = R3, F4 = R2 — soit
**18 constats distincts**). Les deux lentilles confirment de nouveau C-27-2-1 aux tags `v0.13.1` et `v0.13.0`. Écart
déclaré par F : un script `python3 -I` en lecture seule exécuté malgré l'interdit général d'exécuter, constat revérifié
au `grep` ; le prompt P3 nomme `python3` parmi les interdits. Remédiation (Opus 5.5, consignes de l'orchestrateur) :

| Finding | Sév. | Décision |
|---|---|---|
| F1 = R10 | MEDIUM (R : LOW) | **Appliqué** : AC1 — doc-comment de `ColumnMapping` (contrat de `deny_unknown_fields`), test AC1-(5), mutation M23 ; C-27-2-1 amendé, C-27-2-15. |
| F2 | MEDIUM | **Appliqué** : AC12 et AC12-bis — paragraphe « Transactions déjà importées » (supprimer l'import non rapproché puis réimporter ; import rapproché : annuler d'abord le rapprochement ; CSV en devise : refusé au réimport), contrôle PDF `supprimer l.import` ; C-27-2-16. |
| R1 | MEDIUM | **Appliqué** : AC8-bis — passage inverse (mode séparé → colonne unique) écrit, cas Vitest (v8), mutation M24 ; C-27-2-12 amendé. |
| F3 = R3 | LOW | **Appliqué** : AC13-6 gagne `CH1` et `CH` ; M9 élargie. |
| F4 = R2 | LOW | **Appliqué** : AC8-bis — une constante `OWNED_COLUMN_MAPPING_KEYS` (`satisfies`) et `keepUnownedKeys` typée `Partial<ColumnMapping>`, plus de déstructuration ; C-27-2-17. Typage non compilé : `npm run check` en T4. |
| F5 | LOW | **Appliqué** : cas Vitest neufs libellés (v1)-(v9) (le fichier porte déjà (a)-(e)) ; AC17 aligné. |
| F6 | LOW | **Appliqué** : recompté — `grep -rn "ColumnMapping {" crates` rend 29 lignes, dont la définition (`profile.rs:38`) et l'`impl` (`:62`) : **27** littéraux (23 / 3 / 1). La lentille F a raison, R s'était fiée au total brut. |
| F7 | LOW | **Appliqué** : AC12, AC12-bis, C-27-2-1 — réparation par l'API de la 0.13.1 (`PUT` complet sans `currency`, ou `DELETE`), SQL en dernier recours. |
| F8 | LOW | **Écrit** comme angle mort assumé (AC19, C-27-2-11) ; pas de test. Site corrigé : `camt053/mod.rs:572-575` (F citait `:575-578`). |
| F9 | LOW | **Appliqué** : AC11 — phrase fixée (« le relevé est supposé en CHF », 0 sur le PDF actuel, vérifié), seuil `≥ 1`. |
| F10 | LOW | **Appliqué** : *Points de vigilance* (fusion avec la 27-3, ordre des refus) ; AC12 et AC12-bis (le retour à la 0.13.1 suppose qu'aucune migration de la release ne relève `min_required`). |
| R4 | LOW | **Appliqué** : AC13-8 — la ligne `EUR` est une **autre** ligne, valide. Le site cité par R (`parser.rs:578-583`) est faux : le retour de signe contraire est à `:597-602` (débit) et `:629` (crédit), vérifié au `sed -n` ; la fiche cite les bons. |
| R5 | LOW | **Appliqué** : AC13 et C-27-2-9 — le helper saute `header_row_count` lignes **non vides** (10 lignes physiques pour 9 comptées, ligne 9 vide ; doc-module de `parser.rs` l. 14-17, vérifié à la fixture masquée). |
| R6 | LOW | **Appliqué** : AC19 (catalogue `tests/fixtures/README.md`, et trois fichiers qui y manquaient), AC10 (garde G9 de `textes_coherents.rs`, gestes de `i18n-keys.test.ts`) ; T1, T2 alignées. |
| R7 | LOW | **Appliqué** : AC10 — `kesh-core/src/errors.rs:91`/`:93`, `kesh-import/src/error.rs:5`, `kesh-core/src/bank_imports.rs:25`/`:457` ; grep élargi à `crates/` ; C-27-2-19. |
| R8 | LOW | **Appliqué** : AC12 (CHANGELOG, CAMT) — « pouvait », et la condition (dernier montant d'information en devise étrangère). |
| R9 | LOW | **Non corrigé dans la fiche** (consigne) ; texte de la fixture corrigé (AC19 : ordre de l'XSD 001.08, seul versionné) ; hypothèse instruite au registre, C-27-2-18 : **non établie** comme défaut — comportement du repli `:603` établi au code, forme d'entrée conforme au XSD 001.08 mais non montrée chez une banque, aucune fixture ne l'exerce. Rapportée à l'orchestrateur. |
| R11 | LOW | **Appliqué** : AC5 (`:1825` bras, `:1831` test), Dev Notes (`RowAccumulator` 451-461 / 463-500, `parse_row` doc 501-518, `create_csv` 1831), AC10 (famille l. 630). |

**Signal D5** (§ *Règle de splitting préventif*, amendement du 2026-10-06) — **déclaré au Project Lead**. La passe P2 rend
autant de MEDIUM que la P1 (3 → 3, aucun HIGH), ce qui lève le signal « sévérité égale ». Constat, MEDIUM par MEDIUM :

- les trois MEDIUM de P1 (R1-1 observation de débit perdue sur une erreur de devise ; R2-1 motif PDF à apostrophe
  droite ; F1 formulaire qui perd `counterparty`) ne reviennent **sous aucune forme** en P2 : les patchs R1-1 et R2-1
  ont été revérifiés au code par les deux lentilles (AC13-12 recalculé, `pdftotext` exécuté) ;
- **F1 = R10** (`deny_unknown_fields` non figé) porte sur la prémisse de C-27-2-1, **présente dès la création** de la
  fiche : défaut d'origine, non né d'une remédiation ;
- **F2** (transactions déjà importées) : pour moitié défaut d'origine (le CSV de #624 dès la création), pour moitié
  **né de l'élargissement P1** (la moitié CAMT n'existe que depuis AC19) — un manque de la remédiation, non un défaut
  qu'elle aurait introduit dans le code prescrit ;
- **R1** (passage inverse du formulaire non testé) est **né de la remédiation P1** : AC8-bis a été écrit en P1, avec un
  seul sens du changement de mode testé. C'est une lacune du correctif F1 de P1, non une réapparition de F1.

**Conclusion : pas de recyclage au sens de D5.** Aucun défaut ne revient sous une autre forme ; un MEDIUM et demi sur
trois naissent de la remédiation P1 — c'est le motif connu (« la sévérité se déplace vers ce qu'on vient d'écrire »),
mais ce qui en naît reste des **lacunes** de texte neuf, pas des régressions de l'existant ; la sévérité ne monte pas.
Pas de découpage. **Critère de bascule écrit pour P3** : si P3 trouve encore un MEDIUM **né de la remédiation P2** dans
AC8-bis ou AC19, le signal devient recyclage, et l'orchestrateur propose au Project Lead de sortir ces deux AC — ajoutés
en P1 sans issue propre — dans une story à part. Dispersion : la story touche `kesh-import`, `kesh-api`, `kesh-core`
(textes seulement), `kesh-i18n`, `frontend/…/bank-import` et les manuels ; à surveiller, inchangée depuis la P1.

Décomptes après remédiation, recomptés dans la fiche (`grep`) : **22 AC** (inchangé) ; tâches **T0-T8** ; **5** tests
AC1 ; **12** tests de parseur AC13 ; **8** items AC14 ; **9** cas Vitest AC8/AC8-bis ((v1)-(v9)) ; **3** items AC19 ;
mutations **M1-M24** (T1 : 14, T2 : 5, T4 : 5) ; choix **C-27-2-1 à C-27-2-19** (19 au registre). Symptômes grepés après
patch (fiche, registre, prompt P3) : `29 littéraux`, `sinon par SQL`, `était enregistré « en EUR »`, `AC8-(`, `(f)`,
`(g)`, `(h)` (ne subsistent qu'aux lignes de l'inventaire, autre série, et au tableau historique de P1), `M1-M22`,
`C-27-2-14` comme borne, `:1825` comme ligne du test, `l. 631`, `schéma 001.04`, `Devise non supportée v0.1`.
Validation P3 à lancer (`27-2-validate-prompt-p3.md`, **Sonnet**, lentilles R et F).

### Validation P3

2026-10-12. Passe **complète**, **Sonnet**, deux lentilles en contexte frais et en lecture seule (prompt
`27-2-validate-prompt-p3.md`) : **R** — 0 CRITICAL, 0 HIGH, **1 MEDIUM**, 9 LOW ; **F** — 0 CRITICAL, 0 HIGH,
**1 MEDIUM**, 5 LOW. Total brut **2 MEDIUM, 14 LOW** ; deux paires convergentes (F1 = R1, F6 = R3), soit **1 MEDIUM et
13 LOW distincts**. Les deux lentilles confirment une troisième fois C-27-2-1 (aucune migration, `deny_unknown_fields`
présent aux tags), le remède des transactions déjà importées contre la fiche 27-1 (hors la clause CAMT), AC13-12
recalculé et M18 tuée, et ne trouvent aucun MEDIUM dans AC8-bis ni dans AC19. Écart déclaré par F : trois extraits de
`git show` redirigés vers son répertoire temporaire de session (hors dépôt). Remédiation (Opus 5.5, consignes de
l'orchestrateur) :

| Finding | Sév. | Décision |
|---|---|---|
| F1 = R1 | MEDIUM | **Appliqué** : « les transactions CAMT restées en EUR ne sont pas rapprochables » est faux — seules les propositions automatiques (`reconciliation.rs:631`, `:780`) et l'acceptation d'une règle (`:2728`) filtrent la devise ; le rapprochement manuel, l'éclatement et l'acceptation d'une facture ne la contrôlent pas (constaté au `grep`, et documenté par `reconciliation_e2e.rs:2241-2262`). Corrigé à AC12 (bloc 2 et puce CAMT, « la réconciliation ne le proposait plus d'elle-même »), AC12-bis, AC19 (contexte), inventaire (i), C-27-2-11, C-27-2-16 ; garde non ajoutée, hors périmètre (C-27-2-20). |
| R5 | LOW | **Appliqué** : AC15-1 importe `ubs_synthetique_devise.csv`, dérivé en mémoire (dates des lignes de données décalées d'un an), banque et motif propres, `toHaveLength(11)` avant les devises ; AC15-2 dérive du même buffer (C-27-2-25). |
| F2 | LOW | **Appliqué** : AC12 bloc 3, AC12-bis, C-27-2-1 — identifiant et réglages lus dans l'export global de la 0.13.1 (`bank_profiles.csv`, chaîne brute ; `csv_tables.rs` l. 1041-1075 et `global.rs:320` de `v0.13.1`, lus) ; C-27-2-22. |
| F3 | LOW | **Appliqué** : AC19 — entrée 3 (`<TxDtls>` sans `Amt` propre, `AmtDtls` en EUR), test (1-bis), mutation **M25** « première devise gagne », qu'elle seule tue ; AC14-8 asserte trois transactions ; C-27-2-27. |
| F4 | LOW | **Appliqué** : *Points de vigilance* — le `409 BANK_IMPORT_PREVIEW_STALE` de la 27-3 après la résolution du profil, avant tout parse (CSV) ; avant `DUPLICATE_FILE` (CAMT), d'après la fiche 27-3, AC8 ; C-27-2-28. |
| F5 | LOW | **Appliqué** : C-27-2-18 — contrôle de solde confirmable à `bank_imports.rs:1141` ; `:433` est la lecture du champ (vérifié au `sed -n`). |
| F6 = R3 | LOW | **Appliqué** : doc-comment de `OWNED_COLUMN_MAPPING_KEYS` reformulé (nécessaire pour les clés d'un seul mode) ; AC17 déclare équivalents les mutants d'omission de `date`, `reference`, `details`, `currency` ; `ColumnMapping` ajouté à l'`import type` existant (`verbatimModuleSyntax`) ; C-27-2-26. |
| R2 | LOW | **Appliqué** : AC10 — liste fermée des « v0.1 » gardés (les trois sites non nommés y entrent), sortie du grep recopiée au Dev Agent Record ; la parenthèse fausse de `reconciliation_e2e.rs:2253-2254` (« le parser CSV rejette EUR/USD ») est récrite, T2 ; C-27-2-21. |
| R4 | LOW | **Appliqué** : AC13 — « vide » = zéro caractère (`line.is_empty()`, jamais `trim()`) ; C-27-2-23. |
| R6 | LOW | **Appliqué** : C-27-2-18 — `v04_ntry_sign_fallback.xml` porte 3 `<TxDtls>` (`grep -c` → 3), non 2 ; AC19 — la ligne de `v04_minimal.xml` du catalogue (« sans TxDtls ») est corrigée. |
| R7 | LOW | **Appliqué** : AC10 — `i18n-repli-divergent-actif.test.ts` nommé ; apostrophe **droite** pour les textes neufs et leurs replis (C-27-2-21). |
| R8 | LOW | **Appliqué** : AC12 — texte à publier rangé dans les rubriques de la 0.13.1 (bloc « À faire » numéroté, `### Modifié`, `### Corrigé`), sans renvoi de traçabilité ; *Notes pour le dev* séparées ; lien d'issue en ligne (aucune définition de lien de référence au CHANGELOG) ; C-27-2-24. |
| R9 | LOW | **Appliqué** : AC8-bis et C-27-2-17 — `satisfies` n'existe au dépôt que dans des `.test.ts` ; admis par lecture du compilateur Svelte (`remove_typescript_nodes.js:89`), `npm run check` **et** `npm run build` tranchent en T4. |
| R10 | LOW | **Appliqué** : Change Log P2 — « trois paires convergentes ». |

**Trend** : P1 3 MEDIUM → P2 3 MEDIUM → P3 **1 MEDIUM** (distincts), 0 HIGH et 0 CRITICAL à chaque passe ; LOW
distincts 13 → 15 → 13 (P1 : 14 bruts, une paire R7-1 = F4 ; P2 : 18 constats distincts dont 3 MEDIUM, recompte de
la lentille R de P3).

**Signal D5** (critère de bascule écrit en P2) — **pas de bascule**. Le MEDIUM F1 = R1 naît bien de la remédiation P2
(phrase écrite au remède F2 de P2), mais il ne siège **ni dans AC8-bis ni dans AC19** : c'est une **phrase fausse** sur
le comportement de la réconciliation, dans le texte prescrit d'AC12 et dans C-27-2-16, corrigeable en une phrase et
corrigée sans toucher au code prescrit. Aucun défaut de P1 ou de P2 ne revient sous une autre forme ; la sévérité
baisse (3 → 1). Les deux lentilles le qualifient de même. Pas de découpage ; signal déclaré au Project Lead par ce
constat.

Décomptes après remédiation, recomptés dans la fiche (`grep`) : **22 AC** (`grep -cE '^\*\*AC[0-9]+(-bis)?'`) ;
tâches **T0-T8** ; **5** tests AC1 ; **12** tests de parseur AC13 ; **8** items AC14 ; **9** cas Vitest ((v1)-(v9)) ;
**4** items AC19 ((1), (1-bis), (2), (3)) ; mutations **M1-M25** (25 lignes du tableau ; T1 : 15, T2 : 5, T4 : 5) ;
choix **C-27-2-1 à C-27-2-28** (28 titres au registre). Symptômes grepés après patch (fiche et registre) :
`rapprochable`, `écarte`, `l'écart`, `deux paires`, `déjà employé`, `:433`, `(2, tous`, `à côté de la valeur saisie`,
`insérer dans`, `ne rend plus que les sites` — ne subsistent que dans les tableaux historiques et dans les choix qui
racontent la correction. Validation P4 **ciblée** à lancer : `27-2-validate-prompt-p4-ciblee.md` (**Haiku**, une
lentille, le seul commit de cette remédiation).
