# Story 27.2 : La devise d'un relevé CSV est lue, et le contrôle de devise joue

## Status

**ready-for-dev** — fiche créée le 2026-10-12 en autonomie (skill `bmad-create-story`), sur `a2743fd4`
(`origin/main`, rétrospective de l'Epic 26). Validation P1 à lancer : prompt `27-2-validate-prompt-p1.md`. Choix
tranchés : **C-27-2-1 à C-27-2-10**, registre `_bmad-output/implementation-artifacts/epic-27-choix-autonomes-27-2.md`
(à fusionner par union dans `epic-27-choix-autonomes.md`, qui vit sur la branche de la 27-1).

**Issue** : [#624](https://github.com/guycorbaz/kesh/issues/624) (**P1**, `bug` — montant faux en comptabilité).
**PR : `closes #624`.** Epic 27 (recette de la v0.13.1, cible v0.13.2) : le Project Lead a décidé le 2026-10-12 que la
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
| (f) | `BankProfileForm.svelte:56-69` — reconstruit `columnMapping` de ses seuls champs | **perte à l'édition** | résolu pour `currency` (AC8) ; ⚠️ `counterparty` reste perdu — **non résolu, signalé** (C-27-2-7) |
| (g) | `bank_accounts` — aucune colonne devise (`entities/bank_account.rs:24-53`) | **supposition structurelle** (tous les comptes en CHF) | **non résolu, assumé** (C-27-2-4) : le manuel l'écrit (AC11) |
| (h) | `camt053/mod.rs:168-178` — devise de transaction prise sur **tout** `<Amt Ccy>` sous `<TxDtls>` | **hors chemin CSV** | **non résolu, signalé** (C-27-2-3, C-27-2-10) — hypothèse non établie |
| (i) | `reconciliation.rs:631`, `:2728` — garde « transaction non CHF » | défense en aval | inchangé ; devient effective pour le CSV (une ligne non CHF n'atteint plus la base) |

Les sites de **recopie** (`from_imported`, DTO d'aperçu, `NewBankTransaction`) ne décident rien : ils transportent la
valeur que (a)/(b) produisent. Aucun autre lieu du chemin CSV ne fixe de devise.

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
- `CsvLineErrorCode::InvalidCurrency` : ajouté à `all()`, `as_str()` → `"INVALID_CURRENCY"`, `i18n_key()` →
  `"bank-import-csv-errors-invalid-currency"` ; le test `all_is_complete` (`error.rs:379`) doit rester vert.

**AC3 — Un relevé, une devise.** Après la boucle des lignes et **après** le refus de signe (étape 7.b, inchangée et
prioritaire) :

- si la colonne est réglée, l'ensemble des devises des transactions **valides** est calculé ; plus d'une →
  `Err(CsvError::MixedCurrencies { currencies })`, devises distinctes **triées**, **au plus 10** (constante nommée,
  documentée) — refus **non confirmable**, même s'il y a aussi des erreurs de ligne ;
- sinon `ImportedStatement.currency` = cette devise unique ; s'il n'y a **aucune** transaction valide (rejet partiel à
  zéro ligne valide), `DEFAULT_CSV_CURRENCY` (l'import est de toute façon refusé, `no_valid_lines_to_commit`).
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
- `create_csv` : même appel sur le relevé, **avant** le test de `confirm_partial_import` (donc sur `valid` dans la
  branche `PartialFailure`) → `Err(AppError::BankImportUnsupportedCurrency(c))`, `422 BANK_IMPORT_UNSUPPORTED_CURRENCY`,
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
- ⚠️ Un champ `number` vidé par l'utilisateur rend `null` (ou `''` selon Svelte) : le test (c) ci-dessous l'établit, la
  requête ne doit alors **pas** porter `currency`.
- **Vitest** (`BankProfileForm.test.ts`, patron existant) : (a) devise 4 en mode séparé → requête
  `columnMapping.currency === 4` ; (b) idem en colonne unique ; (c) champ laissé vide → `'currency' in
  req.columnMapping` faux (ou valeur `undefined`, sérialisée absente) ; (d) profil existant `currency: 4` → le champ
  s'ouvre à 4, et l'enregistrer sans y toucher le renvoie ; (e) profil existant sans devise → champ vide.

**AC9 — Aperçu d'import (`BankImportUpload.svelte`).** Rien de neuf à afficher : `warning-unsupported-currency` et
`warning-informational` existent (l. 596, 617). Seul le texte de l'avertissement change (AC10). `csv-line-error-label.ts`
gagne `INVALID_CURRENCY: ['invalid-currency', '<repli fr>']`.

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

Les replis français passés à `i18nMsg` / `t` / `t_args` sont **identiques** aux textes `fr-CH`. **Greper la valeur**
après coup : `grep -rnF "v0.1" frontend/src/lib/features/bank-import crates/kesh-i18n/locales` ne rend plus
`unsupported-currency`.

### La documentation

**AC11 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, **PDF régénéré et commité**, `make user` sous
`flock`) :

- § *Profils banque* (l. 1727-1740) : la liste du formulaire cite « la **colonne devise**, facultative » ;
- la `keshnote` l. 1753 (« **La colonne Devise n'est pas lue** : tout import CSV est enregistré en CHF. ») est
  **remplacée** : avec la colonne réglée, Kesh lit la devise de chaque ligne et refuse un relevé qui n'est pas en CHF,
  ou qui mélange des devises ; sans elle, le relevé est **supposé en CHF** et l'aperçu le dit — un relevé d'un compte
  en euros importé par un tel profil serait enregistré en francs : réglez la colonne ; une cellule vide ou illisible
  est une ligne invalide (rejet partiel) ;
- § *Le relevé CSV d'UBS* (tableau l. 1761-1777) : ligne « Colonne devise & 4 (« Devise ») » ; une puce : les profils
  UBS créés avant la 0.13.2 n'ont pas cette colonne — l'ajouter ;
- § *Import dans Kesh* (l. 1717) : « Une devise non supportée bloque l'import » précisé (CAMT.053, ou CSV dont le
  profil lit la colonne Devise) ;
- § *Rejet partiel* (l. 1824-1828) : l'exception « non confirmable » vaut aussi pour un relevé à plusieurs devises.
- ⚠️ Contrôler le **PDF**, aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o
  "colonne Devise n'est pas lue"` → **aucune** sortie ; la phrase neuve s'y trouve.

**AC12 — CHANGELOG** (`CHANGELOG.md`, section `## [0.13.2] — Non publié`, créée ici sous `---` en tête si absente — la
branche de la 27-1 la porte déjà : l'orchestrateur fusionnera) :

- `### Corrigé` — **La devise d'un relevé CSV est lue, et un relevé qui n'est pas en francs suisses est refusé
  ([#624])**. Avant, tout import CSV était enregistré en CHF, au montant du fichier, sans avertissement. Profil :
  colonne Devise facultative ; aperçu : avertissement de devise non supportée, *Confirmer* grisé ; import :
  `422 BANK_IMPORT_UNSUPPORTED_CURRENCY` ; relevé à plusieurs devises : `422 BANK_CSV_MIXED_CURRENCIES`
  (`details.currencies`), non confirmable ; cellule vide ou illisible : ligne invalide (`EMPTY_MANDATORY_FIELD`,
  `INVALID_CURRENCY`). Sans colonne Devise, le relevé reste supposé en CHF et l'aperçu le dit.
- **À faire** : ajouter la colonne Devise (4 pour UBS) aux profils existants dont le fichier en porte une.
- **Retour à une 0.13.1** : un profil qui lit la colonne Devise rend la liste des profils et ses imports en erreur
  sous la 0.13.1 (rien n'est importé à tort) ; retirer la colonne de ces profils avant de revenir.
- **API** : `columnMapping.currency` (facultatif) dans `POST`/`PUT`/`GET /api/v1/bank-profiles` ; la colonne
  `column_mapping_json` de l'export global peut porter la clé `currency`.
- La section *Passer à la 0.13.2* du manuel d'administration relève de la release, pas de cette story (à signaler à
  l'orchestrateur).

### Les preuves

**AC13 — Tests du parseur** (`crates/kesh-import/src/csv/parser.rs`, `mod tests` ; fixture
`include_bytes!(…"/tests/fixtures/csv/ubs_synthetique.csv")`, dont la colonne 4 vaut `CHF` sur les 11 lignes ;
variantes **dérivées dans le test** par remplacement de `;CHF;` sur les lignes de données — C-27-2-9) :

1. profil UBS (26-1) + `currency: Some(4)` → `AllValid`, `stmt.currency == "CHF"`, les 11 `tx.currency == "CHF"` ;
2. même profil, fixture passée en `EUR` sur **toutes** les lignes → `AllValid`, `stmt.currency == "EUR"`, chaque
   `tx.currency == "EUR"` (le parseur reste agnostique ; le refus est en AC14) ;
3. **une seule** ligne passée en `EUR` → `HardFailure(MixedCurrencies { currencies: ["CHF", "EUR"] })` ;
4. une ligne en `eur` sur un fichier tout `eur` → `"EUR"` (casse) ;
5. une cellule vide → `PartialFailure`, une erreur `EMPTY_MANDATORY_FIELD` valeur `"currency"`, 10 valides en CHF ;
6. une cellule `Fr.` → `PartialFailure`, une erreur `INVALID_CURRENCY` ; `€` et `CHF1` idem (ASCII, longueur) ;
7. mélange **et** erreur de ligne dans le même fichier → `MixedCurrencies` l'emporte ;
8. un débit de signe contraire **et** une ligne `EUR` → `AmountSignMismatch` l'emporte (ordre de l'AC3) ;
9. `currency: None` → `stmt.currency == DEFAULT_CSV_CURRENCY` et chaque `tx.currency` aussi (comportement d'avant,
   désormais nommé) ;
10. `currency` hors borne (`Some(40)`) → `ProfileMisconfigured` nommant `column_mapping.currency` ;
11. plus de 10 devises distinctes → `currencies.len() == 10`, triées.

**AC14 — Tests d'intégration** (`crates/kesh-api/tests/bank_imports_e2e.rs`, sur le patron de
`post_import_rejects_eur_currency` l. 442 et des tests CSV l. 1093-1263, `#[sqlx::test(migrations =
"../kesh-db/test-schema")]`) :

1. aperçu CSV, profil avec devise, fichier EUR → `200`, `warnings.unsupportedCurrency.currency == "EUR"`,
   `selectedStatement.currency == "EUR"` ;
2. import du même → `422 BANK_IMPORT_UNSUPPORTED_CURRENCY`, `details.currency == "EUR"`, **zéro** ligne ajoutée à
   `bank_imports`, `bank_transactions` et `audit_log` (compter avant/après) ;
3. import EUR **avec** erreurs de ligne et `confirmPartialImport=true` → toujours `422 BANK_IMPORT_UNSUPPORTED_CURRENCY`
   (le contrôle précède la confirmation) ; sans `confirmPartialImport` → même code (pas `BANK_CSV_PARTIAL_FAILURE`) ;
4. fichier mélangé → aperçu **et** import `422 BANK_CSV_MIXED_CURRENCIES`, `details.currencies == ["CHF","EUR"]`,
   rien d'écrit ;
5. profil avec devise, fichier CHF → `201`, transactions en base `currency = 'CHF'` ; aperçu sans
   `bank_csv_currency_assumed` ;
6. profil **sans** devise → aperçu porte `"bank_csv_currency_assumed"` dans `warnings.informational` ; import `201`
   comme avant (non-régression).
7. `bank_profiles_e2e.rs` : AC7.

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
| M5 | contrôle placé **après** le test `confirm_partial_import` | AC14-3 |
| M6 | contrôle du mélange retiré | AC13-3, AC14-4 |
| M7 | pas de mise en majuscules | AC13-4 |
| M8 | cellule vide acceptée en CHF | AC13-5 |
| M9 | forme non contrôlée (`Fr.` acceptée) | AC13-6 |
| M10 | `currency` absent de `all_indices()` | AC1-(1), AC13-10 |
| M11 | `skip_serializing_if` retiré du champ | AC1-(2) |
| M12 | `MixedCurrencies` évalué avant le refus de signe | AC13-8 |
| M13 | `bank_csv_currency_assumed` toujours poussé / jamais poussé | AC14-5 / AC14-6, AC15-1 / AC15-3 |
| M14 | le formulaire n'envoie pas `currency` | AC8-(a)(b), AC15-1 |
| M15 | le formulaire ne relit pas `existing.columnMapping.currency` | AC8-(d) |
| M16 | `map_csv_error` sans bras `MixedCurrencies` (repli `other`) | AC14-4 (code ≠ `BANK_CSV_MIXED_CURRENCIES`) |
| M17 | plafond des devises rapportées retiré | AC13-11 |

**AC18 — Gates** (au **dernier commit de code**, sous `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`) :
`scripts/test-fast.sh` complet (base remise à zéro avant — base `kesh_272`), `cargo publish --dry-run -p kesh-import`
(zéro dépendance interne, à **exécuter**, il est aussi dans la CI), `npm run check`, `npm run lint-i18n-ownership`,
`npm run test:unit`, `npm run build`, **E2E complet** (base `kesh_e2e_272`, port 3035 ; échecs jugés fichier par
fichier contre `docs/testing.md` § « Les échecs attendus »). Le Dev Agent Record n'affirme que ce qui a tourné, avec
ses chiffres recomptés.

## Tâches

- [ ] **T0** — Brancher dans le worktree `/home/gcorbaz/devel/kesh-27-2` (fait : `story/27-2-devise-import-csv`) ;
  `node_modules` par `cp -al` depuis le dépôt principal ; `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-27-2/target` ;
  bases `kesh_272` / `kesh_e2e_272` créées, migrées, seedées.
- [ ] **T1** — `kesh-import` : AC1, AC2, AC3, AC4 ; tests AC1 et AC13 ; mutations M1, M2, M6-M12, M17.
- [ ] **T2** — `kesh-api` : AC5, AC6, AC7 (helper de contrôle factorisé, variant d'erreur, bras explicite) ; tests
  AC14 ; mutations M3-M5, M13, M16.
- [ ] **T3** — i18n : AC10 (quatre locales, gardes Vitest et lint).
- [ ] **T4** — Frontend : AC8, AC9 ; Vitest ; mutations M14, M15.
- [ ] **T5** — E2E : AC15.
- [ ] **T6** — `ubs_fichier_reel.rs` : AC16 (compilé, **non lancé** par l'agent).
- [ ] **T7** — Manuel (AC11) + PDF + contrôle aplati ; CHANGELOG (AC12) ; `README.md` « Feuille de route » vérifié
  (aucun changement attendu) ; `website/` vérifié (aucune promesse sur la devise CSV attendue : `grep -rni "devise\|currency" website/`).
- [ ] **T8** — Propagation : `grep -rnF '"CHF"' crates/kesh-import/src` (AC4) ; `grep -rn "n'est pas lue" docs/manual/fr
  CHANGELOG.md` (la phrase de 0.13.1 dans le CHANGELOG **reste** : c'est l'histoire) ; `grep -rnF "v0.1"` sur les
  clés modifiées ; gates AC18 ; mutations ; Dev Agent Record.

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
- **CAMT.053 n'est pas touché** (C-27-2-3) : ni `validate_currency_supported_v0_1` (sa signature et son corps restent),
  ni le parseur CAMT.
- Fusion avec la 27-1 (orchestrateur) : conflits attendus sur `CHANGELOG.md` (§ 0.13.2), `sprint-status.yaml`,
  `i18n-keys.test.ts` (`sitesTotal` : la 27-1 en ajoute 9), les quatre `messages.ftl`, le registre des choix.

### Ce qui reste hors périmètre

La lecture du préambule UBS (#621) ; une devise par compte bancaire (multi-devises) ; la devise par transaction du
CAMT.053 (C-27-2-10) ; le champ contrepartie et les libellés non traduits du formulaire (C-27-2-7).

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

Aucun fichier neuf hors tests et artefacts BMAD ; aucune migration ; aucune fixture neuve sous `tests/fixtures/`.

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
