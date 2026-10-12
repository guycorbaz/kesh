# Epic 27 — registre des choix autonomes, Story 27-2 (à fusionner)

*Registre propre à la branche `story/27-2-devise-import-csv`, à fusionner **par union** dans
`epic-27-choix-autonomes.md` (qui vit sur la branche de la 27-1) par l'orchestrateur. Chaque choix : contexte,
option retenue, alternatives, réversibilité.*

---

## Story 27-2 — La devise d'un relevé CSV est lue, et le contrôle de devise joue (#624)

Fiche : `27-2-devise-import-csv.md`. Choix faits à la création de la fiche (2026-10-12), en autonomie, sur
`a2743fd4` (`origin/main`, rétrospective de l'Epic 26).

### C-27-2-1 — Le mécanisme : une colonne `currency` facultative dans `ColumnMapping`, sans migration

- **Contexte** : le parseur CSV écrit `"CHF"` en dur, pour le relevé (`parser.rs:364`) et pour chaque transaction
  (`parser.rs:680`). Il faut que le profil puisse **dire où lire la devise**. Deux lieux de stockage possibles :
  le JSON `bank_profiles.column_mapping` (les indices de colonnes y vivent tous), ou une colonne SQL dédiée
  (précédent `debit_sign`, Story 26-1, choisi « parce que la 0.13.0 désérialise `column_mapping` en
  `deny_unknown_fields` » — en-tête de `20261011000001_bank_profiles_debit_sign.sql`).
- **Retenu** : `ColumnMapping.currency: Option<usize>`, `#[serde(skip_serializing_if = "Option::is_none", default)]`
  comme ses voisins `reference` / `details` / `counterparty`. **Aucune migration, aucun bump de
  `kesh_version_min_required`, aucun bump Cargo.** Raisons :
  1. l'indice entre de lui-même dans `all_indices()` et `max_index()` : la collision avec une autre colonne et le
     hors-borne au premier enregistrement sont refusés **sans code neuf** ;
  2. un profil **sans** colonne Devise se sérialise **octet pour octet** comme avant (le champ est omis) : une
     0.13.1 le lit sans rien voir ;
  3. un profil **avec** colonne Devise, relu par une 0.13.1, est **refusé bruyamment** : `ColumnMapping` y porte
     `#[serde(deny_unknown_fields)]` (vérifié aux tags : `git show v0.13.1:crates/kesh-import/src/csv/profile.rs |
     grep -n deny_unknown_fields` → `37:#[serde(deny_unknown_fields)]` ; `v0.13.0` → `29:`), donc
     `parse_column_mapping` rend `DbError::Invariant` → 500 sur la liste des profils et sur tout import par ce
     profil. **Aucun import silencieusement faux** : la politique de migration vise les *downgrades silencieux
     corrupteurs* (§ Migration breaking policy, introduction), ce retour-là est bruyant et n'écrit rien.
  4. une colonne SQL dédiée serait, elle, **ignorée en silence** par une 0.13.1 (qui forcerait CHF de nouveau) et
     imposerait donc le bump `min_required` → `0.13.2`, le bump Cargo du workspace dans la même PR (P2-bis), une
     migration (P5, P6, P7, squash `test-schema`, `test_schema_guard`), pour un résultat moins bon.
- **Écartées** : colonne SQL dédiée + bump (plus lourde, et rend le retour arrière impossible au lieu de bruyant) ;
  lecture de la devise dans le **préambule** UBS (`Évaluation en:;CHF;`) — hors périmètre, c'est #621 ; détection
  automatique d'une colonne titrée « Devise » / « Währung » — contraire à la règle du Project Lead de la 26-1
  (« un réglage explicite, jamais une déduction »).
- **Conséquence écrite** (CHANGELOG, AC12) : avant un retour à une 0.13.1, retirer la colonne Devise des profils qui
  la lisent ; sinon la 0.13.1 refuse la liste des profils.
- **Réversible** : oui (champ facultatif, jamais exigé).

### C-27-2-2 — Un relevé CSV est dans une seule devise : un fichier qui en mélange est refusé en entier

- **Contexte** : `ImportedStatement.currency` est une valeur unique (un `<Stmt>` CAMT.053 n'a qu'un compte, donc une
  devise) ; une colonne Devise lue ligne par ligne peut en rendre plusieurs.
- **Retenu** : le parseur (agnostique, `kesh-import`) exige que toutes les transactions **valides** portent la même
  devise ; sinon `CsvError::MixedCurrencies { currencies }` (devises distinctes, triées, **au plus 10**), refus
  **non confirmable** → `422 BANK_CSV_MIXED_CURRENCIES`, `details.currencies`. Le refus de signe
  (`AmountSignMismatch`, 26-1) le précède — « régler le profil d'abord » ; il précède lui-même le rejet partiel.
- **Écartées** : prendre la devise de la première ligne et mettre les autres en erreur de ligne (arbitraire, et
  confirmable : on importerait les lignes en CHF d'un relevé multi-devises sans que rien ne le dise au solde) ;
  ramener le mélange à `BANK_IMPORT_UNSUPPORTED_CURRENCY` dans le parseur (le parseur ne connaît pas le périmètre
  CHF, `kesh-core/src/bank_imports.rs:458-461`).
- **Réversible** : oui.

### C-27-2-3 — Le contrôle : `validate_currency_supported_v0_1` tel quel, appelé enfin sur le chemin CSV

- **Contexte** : le contrôle existe (`kesh-core/src/bank_imports.rs:467`) mais n'est appelé que sur le chemin
  CAMT (`routes/bank_imports.rs:961` aperçu, `:1127` import) ; `preview_csv` (`:1609`) et `create_csv` (`:1758`)
  ne l'appellent pas.
- **Retenu** : même fonction, **non modifiée**, sur le relevé CSV : à l'aperçu, avertissement
  `warnings.unsupportedCurrency` (l'écran l'affiche déjà et grise *Confirmer*, `BankImportUpload.svelte:226`, :596) ;
  à l'import, `422 BANK_IMPORT_UNSUPPORTED_CURRENCY` **avant** le traitement du rejet partiel (le relevé valide d'un
  `PartialFailure` est contrôlé avant que `confirmPartialImport` soit consulté : on ne demande pas de confirmer des
  lignes pour refuser ensuite le fichier).
- **Écartée** : étendre `validate_currency_supported_v0_1` à la devise **de chaque transaction**, pour les deux
  formats — dangereux pour CAMT : `camt053/mod.rs:168-178` pose `t.currency` depuis **tout** `<Amt Ccy>` rencontré
  sous `<TxDtls>` (y compris, si le fichier en porte, `AmtDtls/InstdAmt/Amt`), alors que le montant ne vient que de
  `TxDtls/Amt` (`:427`). Un relevé CHF porteur d'un paiement par carte en EUR pourrait être refusé à tort. Le
  chemin CAMT n'est **pas touché** (hypothèse signalée à l'orchestrateur, C-27-2-10).
- **Réversible** : oui.

### C-27-2-4 — Sans colonne Devise au profil : CHF supposé, et dit à chaque aperçu

- **Contexte** : l'issue propose, à défaut, de « documenter que le profil CSV suppose un compte en CHF ». Le
  contrôle « par la devise du compte bancaire » est impossible : `bank_accounts` ne porte aucune devise
  (`crates/kesh-db/src/entities/bank_account.rs:24-53`, `grep -rln currency crates/kesh-db/migrations/` ne cite que
  `bank_imports` et `imported_supplier_invoices`) — tous les comptes sont implicitement en CHF dans cette version.
- **Retenu** : profil sans colonne Devise → devise `DEFAULT_CSV_CURRENCY = "CHF"` (constante publique nommée et
  documentée de `kesh-import`, au lieu de deux littéraux), **et** avertissement informatif
  `bank_csv_currency_assumed` à chaque aperçu d'un tel profil (« Le profil ne lit aucune colonne Devise : le relevé
  est supposé en francs suisses (CHF). »). Le manuel le dit, et conseille de régler la colonne.
- **Écartées** : rendre la colonne obligatoire (casserait tous les profils existants et les banques dont le CSV n'a
  pas de colonne Devise) ; ajouter une devise au compte bancaire (périmètre multi-devises) ; documentation seule
  (l'hypothèse resterait invisible à l'import, ce qui est exactement le défaut).
- **Réversible** : oui.

### C-27-2-5 — Lecture d'une cellule Devise : forme ISO, casse normalisée, erreurs de ligne ordinaires

- **Retenu** : cellule `trim`ée ; vide → `EMPTY_MANDATORY_FIELD` (valeur `currency`) — une fois la colonne
  réglée, la devise est obligatoire, jamais remplacée par CHF ; exactement trois lettres ASCII → mise en
  majuscules (`chf` → `CHF`) ; autre chose (`Fr.`, `€`, `CH`, `CHF1`) → nouveau code de ligne `INVALID_CURRENCY`.
  Ce sont des erreurs de ligne **ordinaires** (rejet partiel, confirmables), comme une date illisible : la ligne
  écartée n'entre pas en comptabilité. Pas de contrôle contre la liste ISO 4217 : la forme suffit au parseur, et
  toute devise autre que CHF est de toute façon refusée par C-27-2-3.
- **Ordre dans `parse_row`** : longueur, date, montant, **puis devise** — un montant illisible ou de signe
  contraire masque la devise de la ligne (la ligne est écartée ou le fichier refusé de toute façon).
- **Écartées** : CHF pour une cellule vide (rouvrirait #624 ligne par ligne) ; refus du fichier entier sur une
  cellule illisible (sans motif : la ligne n'est pas importée).
- **Réversible** : oui.

### C-27-2-6 — L'avertissement « Devise non supportée v0.1. » est réécrit

- **Contexte** : la clé `bank-import-warnings-unsupported-currency` dit « v0.1 » dans les quatre locales
  (`fr-CH:1065`, `de-CH:1007`, `it-CH:1007`, `en-CH:1007`) ; le correctif la rend atteignable par le CSV, chemin le
  plus employé.
- **Retenu** : « Devise non supportée : seul le franc suisse (CHF) est pris en charge. Devise du relevé : » (la devise
  suit, comme aujourd'hui), quatre locales, repli Svelte identique.
- **Réversible** : oui.

### C-27-2-7 — L'écran du profil : un champ de plus, traduit ; le reste du formulaire n'est pas repris

- **Retenu** : champ « devise (facultative) » (`data-testid="col-currency-input"`), vide par défaut, initialisé depuis
  `existing.columnMapping.currency`, envoyé seulement s'il est rempli, avec une aide traduite. Libellé et aide
  passent par `i18nMsg` (clés neuves, quatre locales).
- **Non repris, signalé** : les libellés voisins (`date`, `débit`, `crédit`, `montant`, `référence`, `détails`,
  `BankProfileForm.svelte:259-283`) sont du français en dur ; et le formulaire **n'a pas de champ contrepartie** —
  il reconstruit `columnMapping` à partir de ses seuls champs (`:56-69`), si bien qu'un `counterparty` posé par
  l'API est **perdu** à la première modification par l'écran. Les deux sont hors du défaut #624 ; une issue est
  demandée à l'orchestrateur pour le second (perte silencieuse d'un réglage).
- **Réversible** : oui.

### C-27-2-8 — Le profil UBS recommandé lit la colonne 4

- **Retenu** : le tableau du manuel (*Le relevé CSV d'UBS*) gagne « Colonne devise : 4 (« Devise ») » ; le test
  local sur le vrai relevé (`crates/kesh-import/tests/ubs_fichier_reel.rs`, ignoré par défaut) mappe la colonne 4 et
  vérifie que **chaque** transaction et le relevé sont en CHF — contrôle rendu « conforme / non conforme »
  seulement, lancé par l'orchestrateur (aucune donnée réelle dans le dépôt).
- **Réversible** : oui.

### C-27-2-9 — Fixtures : dérivées de `ubs_synthetique.csv`, jamais du fichier réel

- **Retenu** : les cas EUR et mélange se **dérivent dans le test** de `crates/kesh-import/tests/fixtures/csv/ubs_synthetique.csv`
  (remplacement de `;CHF;` sur les lignes de données) — une seule source ; les cas limites (cellule vide, `chf`,
  `Fr.`, collision, hors-borne) en petits CSV écrits dans le test. Aucun fichier neuf sous `fixtures/`, rien lu
  sous `~/kesh-test/`.
- **Réversible** : oui.

### C-27-2-10 — Hors périmètre, signalés et non traités

- **CAMT.053, devise par transaction** (C-27-2-3) : hypothèse, non établie faute de fixture portant `<AmtDtls>` —
  aucune fixture de `crates/kesh-import/tests/fixtures/camt053/` n'en contient. Si elle se vérifie, la transaction
  serait enregistrée avec la devise d'un montant d'information, et le garde-fou de réconciliation
  (`routes/reconciliation.rs:631`, `:2728`) la rendrait irrapprochable. Issue « à vérifier » proposée.
- **Le préambule UBS** (`Évaluation en:`) : #621.
- **Semver de `kesh-import`** : un champ public ajouté à `ColumnMapping`, un variant à `CsvError` et à
  `CsvLineErrorCode` — même nature que les ajouts de la 26-1 (`CsvProfile.debit_sign`, `HeaderSkipsData`,
  `AmountSignMismatch`), sans `#[non_exhaustive]` : le crate n'est pas publié, le `--dry-run` reste le seul
  contrôle exigé.
