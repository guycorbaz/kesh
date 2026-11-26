# Story 39-14 : l'import Wise branché sur le cœur commun des prestataires

Status: done

*Fiche rédigée le 2026-11-22 au gabarit de `bmad-create-story`, branche `story/39-14` partie d'`epic/39-wise`
(`1f03f180`, qui porte la 39-11e et `epic/38-paypal` à `8b8cd338`, fusionnée par `aaaf7875`). Epic 39. **Dépend
de** : 39-11b (route `POST /api/v1/wise-imports`), 39-11e (zip), 38-12a (cœur extrait `routes/provider_imports.rs`),
toutes `done`. **Aucune migration**, aucune route neuve, **aucun comportement visible changé**. Liste « À raccorder »
de la fiche 39-11b (§ Dev Agent Record), points 1 à 10.*

## Story

En tant que **développeur de Kesh**, je veux que l'import Wise passe par le cœur commun des imports de prestataire
(`provider_imports::import_lines`) comme PayPal, afin qu'il n'existe qu'**un** cœur (verrous, relecture, empreinte,
doublons, entrée `bank_imports`, traces, rejeu) et que la copie de la 39-11b, restée dans `paypal_imports.rs`,
disparaisse.

## État de départ (relevé au code, `1f03f180`)

- PayPal (fichier et téléchargement) passe déjà par `provider_imports::import_lines` (38-12a).
- `routes/paypal_imports.rs` porte **en plus** le cœur de la 39-11b, que seul Wise appelle (l. 454-826) :
  `file_currency_hash`, `resolve_currency_balances`, `LockedTarget`, `WriteFuture`, `ProviderWrite`,
  `run_provider_import`, `import_under_locks`, `recheck_targets_locked`, `write_under_locks`, `CurrencyImport`,
  `insert_currency_import` — doublon de `provider_imports.rs`. Les bornes `DETAILS_MAX_CHARS`, `NAME_MAX_CHARS`,
  `PROVIDER_TYPE_MAX_CHARS`, `TXID_COLUMN_MAX_CHARS` et `truncate_chars`/`truncate_bytes` y sont `pub(crate)` pour
  Wise : elles restent (Wise s'en sert pour ses lignes).
- `routes/wise_imports.rs` : `create` appelle `resolve_currency_balances` puis `run_provider_import` avec un
  `WiseWriter` (doublons par (identifiant, sens), continuité, entrée par fichier, trace `wise_import.created` avec
  `files` et `ignoredEntries`).
- `provider_imports::AUDIT_ACTIONS` ne porte pas `wise_import.created` (point 6) : `import_lines` le refuserait en
  500. `audit_labels::ACTIONS` et les quatre `.ftl` le portent déjà (39-11b) ; **`SITES_INDIRECTS`
  d'`audit_label_registry.rs` non** (son entrée `provider_imports.rs` ne liste que les deux codes PayPal) : une fois le
  littéral retiré de `wise_imports.rs`, trois tests de ce registre rougiraient (validation P1, H1).

## Ce que le cœur ne sait pas faire, et que Wise exige

1. **Un fichier par devise** : nom (`bank_imports.filename`) et octets (empreinte `SHA-256(octets ‖ ":" ‖ devise)`)
   propres à chaque devise ; `ImportRequest` n'a qu'un `filename` et une `ImportSource::File` pour toutes.
2. **Période, ouverture et clôture du fichier entier** à l'entrée `bank_imports` (point 8 : la continuité de la 39-11c
   relit `period_from` et `opening_balance` stockés) ; le cœur prend la période des lignes **gardées**, `opening_balance:
   None` et la clôture de `balance_after`.
3. **IBAN de la contrepartie** (`ProviderLine` n'en a pas ; `into_new_bank_transaction` pose `None`).
4. **Doublons par (identifiant, sens)** (39-11a2) : le cœur juge par identifiant seul (un recrédit au même
   identifiant qu'un débit déjà en base serait ignoré à tort) — absent de la liste « À raccorder », relevé ici.
5. **Avertissements de continuité** par devise (`bank_imports::continuity_gap`, sous les verrous, avant l'insertion).
6. **Ordre du formulaire** : le cœur parcourt les devises dans l'ordre de `BTreeMap` (alphabétique) ; Wise juge les
   refus `PROVIDER_CURRENCY_DUPLICATE`, `BANK_ACCOUNT_CLOSED`, `BANK_IMPORT_DUPLICATE_FILE`, crée ses entrées (donc
   leurs identifiants) et rend `files` **dans l'ordre du formulaire** (zips développés à leur place).
7. **La trace** `wise_import.created` porte `files` (les comptes rendus de fichier, tels que la réponse) et
   `ignoredEntries`, sans `currencies` (point 7).

## Décisions

- **D1 — Une variante de source, non un champ de plus** : `ImportSource::Statements(&[CurrencyStatement])`, un relevé
  par devise, dans l'ordre de l'appelant. `CurrencyStatement { currency, filename, bytes, period_from, period_to,
  opening_balance, closing_balance, trace }` — `bytes: &'a [u8]`, période en `NaiveDate`, soldes en `Decimal` (non
  facultatifs : `continuity_gap` les exige, la réponse les rend), `trace: serde_json::Map` (validation P2, L5). Pour cette source, le cœur : parcourt les devises dans l'ordre des
  relevés (refus et écritures) ; prend nom, empreinte (des octets du relevé, même formule que `File`), période et
  soldes du relevé (même pour un relevé à doublons partiels) ; refuse l'empreinte déjà importée comme `File` ; juge la
  continuité et rend `warnings` ; écrit à la trace de la source `files` (pour chaque devise, `trace` du relevé complété
  du compte rendu de la devise) au lieu de `currencies`. `ImportRequest` ne change pas de forme (pas de champ neuf :
  les constructions de PayPal et de Stripe, epic/40, restent valides) ; son `filename` est ignoré pour cette source.
  Les devises de `by_currency` et celles des relevés doivent coïncider, une fois chacune : sinon `AppError::Internal`
  (faute de Kesh), testé. **Ordre dans l'écriture, par devise** (celui de l'ancien `WiseWriter`) : lecture des
  transactions existantes, doublons, **continuité jugée pour tout relevé, avant de savoir s'il reste une ligne** (un
  relevé tout en doublons rend aussi ses avertissements), puis l'entrée s'il reste une ligne.
- **D2 — `ProviderLine` gagne deux champs** : `counterparty_iban: Option<String>` et `direction:
  Option<LineDirection>` (`Debit`/`Credit`). `None` (PayPal, Stripe) : doublon par identifiant, comme aujourd'hui.
  `Some` : doublon par (identifiant, sens) — une transaction en base vaut débit si son montant est ≤ 0, crédit s'il est
  ≥ 0 (zéro : les deux), règle de la 39-11a2 déplacée telle quelle ; dans la source, l'unité du doublon est
  (`group_id`, sens). Coût assumé : les littéraux `ProviderLine { … }` d'epic/40 (Stripe) ne compileront plus à sa
  fusion — correction mécanique, `counterparty_iban: None, direction: None` (à noter au registre des choix).
- **D3 — Doublons** : le cœur appelle `dedupe_by_keys(groups, &ExistingKeys)` ; `dedupe_groups(groups,
  &HashSet<String>)` garde sa signature (tests de PayPal et de Stripe, epic/40) et délègue par
  `ExistingKeys::from_ids` (validation P2, M2.3) ; `ExistingKeys::of(&[ExistingTransaction])`, lue par
  `bank_transactions::existing_transactions_by_ids` (même filtre SQL que `existing_transaction_ids`, montant en plus)
  pour tous les prestataires. Une ligne **sans** sens est présente si une transaction de même identifiant existe, quel
  que soit son montant (ensemble des identifiants, distinct des deux ensembles par sens) : doublons inchangés pour
  PayPal. `existing_transaction_ids` (`kesh-db`) perd son dernier appelant de `kesh-api` ; il reste (fonction publique,
  lue par d'autres branches d'epic), et ses commentaires et ceux d'`existing_transactions_by_ids`, qui nomment leurs
  appelants, deviennent inexacts : **à tracer en issue** par l'orchestrateur (validation P2, L2). Les lignes d'un
  relevé s'insèrent dans l'ordre de `csv.rows`, comme avant (L6).
- **D4 — `CurrencyReport.warnings: Option<Vec<ImportWarning>>`**, omis quand `None` : la réponse PayPal reste octet
  pour octet ; `Some` (éventuellement vide) pour `Statements`. `ImportWarning` = l'ancien `FileWarning` de Wise
  (`code`, `expected`, `found`, `previousImportId`), déplacé au cœur avec `WARNING_GAP` ; `wise_imports` les
  ré-exporte sous leurs noms.
- **D5 — Wise construit ses lignes et son compte rendu** : `line_of_row(row) -> ProviderLine` (un groupe = une ligne,
  `group_id = transaction_id = id`, `direction` du lecteur, `rank` = rang de la ligne, `balance_after: None` — les
  soldes du relevé priment) ; `transaction_of_row` devient `line_of_row(row).into_new_bank_transaction(…)` (API et
  tests inchangés). `FileReport` = la partie fichier (nom, archive, lignes lues, soldes) + le `CurrencyReport` de la
  devise ; la `trace` du relevé porte ces mêmes champs fichier, si bien que `files` de la trace égale `files` de la
  réponse.
- **D6 — Point 2e (point d'accroche `rules` de la 39-12a3) hors périmètre** : la 39-12a3 est sortie de l'epic
  (branche `futur/wise-api`) ; rien n'est écrit qui l'empêche.
- **D7 — Textes du journal serveur** : ceux du cœur 38-12a (« import : libération des verrous en échec », « import :
  verrous des comptes non acquis », « import : trop de transactions ») remplacent pour Wise ceux de la copie
  39-11b (« import de prestataire : … », « import wise : trop de transactions »). Journal seul : ni réponse, ni trace.

## Critères d'acceptation

- **AC-1 — Un seul cœur** : `paypal_imports.rs` ne porte plus aucune des onze fonctions et types du cœur 39-11b ;
  `wise_imports.rs` appelle `provider_imports::load_balances` et `import_lines` ; `grep -rn
  "run_provider_import\|insert_currency_import\|ProviderWrite\|LockedTarget\|resolve_currency_balances\|file_currency_hash\|WriteFuture\|CurrencyImport\b\|WiseWriter\|fn existing_keys("
  crates/` ne rend que des mentions historiques datées (fiches, commentaires « avant la 39-14 »).
- **AC-2 — Réponses inchangées** : pour Wise, mêmes statuts, codes, corps (`files` dans l'ordre du formulaire,
  `rows`, `inserted`, `duplicates`, `importId`, `openingBalance`, `closingBalance`, `warnings` toujours présent,
  `archive`, `ignoredEntries`) et même ordre des refus (en-tête du module `wise_imports`) ; pour PayPal, fichier et
  téléchargement, corps octet pour octet (pas de clé `warnings`). Tenu par les suites existantes
  (`wise_import_e2e`, `paypal_import_e2e`, `paypal_download*`) vertes **sans modification de leurs assertions**, et
  par un test neuf (AC-5) ; pour PayPal, l'ensemble exact des clés d'une entrée de `currencies` (`bankAccountId`,
  `currency`, `duplicates`, `importId`, `inserted`) est vérifié dans la réponse et la trace, fichier
  (`paypal_import_e2e::import_ventile_par_devise_puis_doublons`) et téléchargement (`paypal_download_e2e`), sonde
  « `warnings` sérialisé pour PayPal » (validation P2, M1).
- **AC-3 — Écritures inchangées** : par fichier Wise, une entrée `bank_imports` (nom du fichier, empreinte octets ‖
  devise, période et soldes du **fichier entier**, `CSV`) s'il reste une ligne ; transactions avec IBAN de la
  contrepartie ; doublons par (identifiant, sens) ; traces `bank_import.created` (mêmes `details`) et
  `wise_import.created` (`provider`, `provider_account`, `files`, `ignoredEntries`, sans `currencies`) ; identifiants
  des entrées croissants dans l'ordre du formulaire ; rejeu sur interblocage sous l'opération `wise_imports::create`.
- **AC-4 — Registre des actions** : `AUDIT_ACTIONS` porte `wise_import.created`, et l'entrée `provider_imports.rs` de
  `SITES_INDIRECTS` (`tests/audit_label_registry.rs`) aussi ; verts : `action_de_trace_inscrite_ou_refusee` (cœur),
  `les_actions_de_la_liste_sont_exactement_celles_que_le_code_ecrit`,
  `les_actions_du_coeur_d_import_sont_inventoriees_et_libellees`,
  `les_codes_de_l_inventaire_sont_verifies_dans_le_fichier_de_leur_site`.
- **AC-5 — Test de non-régression neuf** (`wise_import_e2e`) : deux relevés envoyés dans l'ordre USD puis CHF, le
  second à doublons partiels et à écart de continuité : corps entier comparé à une valeur attendue écrite en dur,
  `files` de la trace égal à `files` de la réponse, lignes `bank_imports` (nom, période, ouverture, clôture) du fichier
  entier, identifiant de l'entrée USD inférieur à celui de CHF ; et deux comptes clos, envoyés USD puis EUR : le refus
  nomme le compte USD ; un relevé CHF **tout en doublons** à écart de continuité : `importId` nul, avertissement rendu
  et tracé, aucune entrée ; empreinte stockée = SHA-256(octets ‖ `:CHF`). **Vert avant le raccord** (joué sur `1f03f180`) **et après** ; sondes par mutation hors dépôt
  (ordre alphabétique rétabli, période des lignes gardées, `direction` ignorée, `warnings` omis, continuité sautée pour
  un relevé tout en doublons, `warnings` sérialisé pour PayPal) tuées par une
  assertion nommée.
- **AC-6 — Doc** : en-têtes de `wise_imports.rs` et `provider_imports.rs` à jour ; `audit_route_registry.rs` point
  (vi) : l'enveloppe de rejeu de Wise vit dans `provider_imports::import_lines` (comptes de routes inchangés — aucune
  route ajoutée ; recompte déclaré) ; `docs/api-external.md` inchangé (aucun contrat ne bouge).

## Liste « À raccorder » de la 39-11b

| Point | Sort |
|---|---|
| 1 | T3 (cœur 39-11b retiré de `paypal_imports.rs` ; PayPal y était déjà branché à la fusion `aaaf7875` et avant) |
| 2a–2d | D1, D2, D4 |
| 2e | écarté, D6 (39-12a3 sortie de l'epic) |
| 3 | D5, T2 |
| 4 | AC-6 (commentaire du point (vi)) ; comptes de routes et phrase du manuel **déjà recomptés** à la fusion d'`epic/38` (gate vert de la 39-11e à `ac3e1fe4`) — rien à recompter, aucune route ajoutée |
| 5, 9 | résolus aux fusions précédentes d'`epic/38` dans `epic/39` (blocs i18n et conflits de texte présents des deux côtés) ; rien ici |
| 6 | AC-4 |
| 7 | D1, D5 : `files` gardé, sans `currencies` |
| 8 | D1 (période et soldes du relevé entier), AC-5 |
| 10 | hors périmètre (Epic 40 : `served()`, et `json.paypal.or(json.wise)`, `kesh-db/src/entities/bank_account.rs:215`) : **à tracer en issue** par l'orchestrateur (cette story n'a pas le droit d'ouvrir d'issue) |

## Tâches

1. T1 — Cœur : `LineDirection`, deux champs de `ProviderLine`, `ExistingKeys`, `CurrencyStatement`,
   `ImportSource::Statements`, ordre des cibles, empreinte, période et soldes, continuité, `warnings`, trace `files` ;
   `wise_import.created` à `AUDIT_ACTIONS` ; tests unitaires (empreinte d'un relevé = celle de `File`, doublons par
   sens, zéro des deux sens).
2. T2 — Wise : `line_of_row`, `create` par `load_balances` + `import_lines`, `FileReport` depuis `CurrencyReport` ;
   retrait de `WiseWriter`, `Target`, `existing_keys` (son test passe au cœur).
3. T3 — PayPal : retrait du cœur 39-11b et des imports devenus inutiles ; littéraux `ProviderLine` complétés.
4. T4 — Test AC-5 écrit **d'abord**, joué vert sur le code d'avant ; assertions des clés exactes de `currencies` dans
   `paypal_import_e2e::import_ventile_par_devise_puis_doublons` et dans le test du téléchargement de
   `paypal_download_e2e` qui relit la trace `paypal_download.created` (AC-2) ; `les_cles_existantes_lisent_le_sens_au_signe`
   (cœur) couvre la ligne sans sens au débit, au crédit et à zéro.
5. T5 — Doc (AC-6), grep des symboles retirés, fiche.

## Stratégie de tests

Non-régression avant tout : suites e2e Wise et PayPal inchangées + test AC-5 joué des deux côtés du raccord ; tests
unitaires du cœur. Gates : fmt, clippy workspace, nextest complet ; aucun écran, aucune spec E2E neuve, donc aucun E2E
(règle de la nuit du 22.11) ; frontend non touché.

## Modules touchés (1)

`kesh-api` : `routes/provider_imports.rs`, `routes/wise_imports.rs`, `routes/paypal_imports.rs`,
`tests/wise_import_e2e.rs`, `tests/paypal_import_e2e.rs` et `tests/paypal_download_e2e.rs` (clés exactes du compte
rendu, AC-2 ; T4), `tests/audit_label_registry.rs` (inventaire), `tests/audit_route_registry.rs`
(commentaire). E2E partiel : aucun — aucune spec neuve, aucun écran (règle de la nuit du 22.11) ; selon la règle du
22.11 00:30, les specs des écrans d'import Wise et PayPal auraient été retenues.

## Fichiers communs (39-11c en remédiation sur `story/39-11c`)

`sprint-status.yaml` (chacun sa ligne ; `tests/audit_route_registry.rs` est touché des deux côtés, sans conflit à la
fusion à blanc). La 39-11c ne touche ni `wise_imports.rs`, ni `provider_imports.rs`, ni `paypal_imports.rs` ; son
`date_coverage` relit période et ouverture stockées, que D1 garde au fichier entier (compatible, L7).

**`epic/40-stripe`** (`73fa1573`, validation P2, M2) change aussi `provider_imports.rs` ; qui fusionne en second :
(1) `AUDIT_ACTIONS` et l'entrée `SITES_INDIRECTS` : garder `stripe_import.created` **et** `wise_import.created` ;
(2) `ProviderLine` : l'union des champs (`value_date` de Stripe, `counterparty_iban` et `direction` d'ici) — **trois
sites de conflit de texte** (le quatrième conflit de cette story est `AUDIT_ACTIONS`, point (1)) : les champs du type, le littéral du test `line()` de `provider_imports.rs`, et le
littéral de `provider_lines_of_row` dans `paypal_imports.rs` (~l. 309 ; non annoncé jusqu'à la revue P1, M1) ; ceux
de Wise sont à compléter de `value_date: None` (sa valeur d'avant), ceux de Stripe et de PayPal de
`counterparty_iban: None, direction: None`, et `into_new_bank_transaction` recopie les trois ; (3) `dedupe_groups`
garde sa signature (D3) : les tests de `stripe_imports.rs` compilent ; (4) Stripe **copie** les bornes (`DETAILS_MAX_CHARS`…),
`FEE_SUFFIX`, `bounded` et `truncate_*` en `pub` dans `provider_imports.rs` sans retirer celles de `paypal_imports.rs`
(commentaire de Stripe : « à replier à l'intégration ») : rien ne casse à la fusion, mais **deux définitions
restent**, à unifier par celui qui fusionne en second (celles de `paypal_imports.rs` retirées, les `use` de Wise
**et** les usages propres de PayPal repointés) ; `parse_form` et `ProviderImportForm` s'ajoutent sans conflit.
Les autres conflits d'une fusion à blanc d'`epic/40-stripe` viennent d'autres stories de l'Epic 39 (notamment la 39-11b :
`crates/kesh-db/src/entities/bank_account.rs`, `routes/bank_accounts.rs`, `tests/bank_account_opening_balance_e2e.rs`,
`errors.rs`, `lib.rs`, registres, catalogues i18n, `sprint-status.yaml`, documents) ; ils se listent à la fusion réelle,
non ici. Le littéral `ProviderLine` de `wise_imports.rs` ne conflictue pas mais devra recevoir `value_date: None`.

## Note de version

Aucune ligne au CHANGELOG : réorganisation interne, sans effet visible (journal serveur seul, D7).

## Dev Agent Record

### Étape 0 — mise en place et recontrôle

- Fusion **non squash** d'`epic/38-paypal` (`8b8cd338`) dans `story/39-14` : « Already up to date » — sa tête était
  déjà fusionnée dans `epic/39-wise` par `aaaf7875` ; rien de neuf, aucun conflit, aucun compteur à recompter.
- Recontrôle du MEDIUM de la P3 (grep de l'AC-1), au code final : `grep -rn
  "run_provider_import\|insert_currency_import\|ProviderWrite\|LockedTarget\|resolve_currency_balances\|file_currency_hash\|WriteFuture\|CurrencyImport\b\|WiseWriter\|fn existing_keys(" crates/`
  ne rend qu'une ligne, `tests/audit_route_registry.rs:367`, mention historique (« avant, la copie de la 39-11b,
  `paypal_imports::run_provider_import` »).

### Ce qui est livré (`35e69ea6`)

- `routes/provider_imports.rs` : `LineDirection` ; `ProviderLine.counterparty_iban` et `.direction` ;
  `ExistingKeys` (`of`, `from_ids`, `contains`) ; `dedupe_by_keys` (le cœur), `dedupe_groups` gardé (délègue) ;
  `ImportWarning`, `WARNING_GAP`, `CurrencyReport.warnings` (omis si `None`) ; `CurrencyStatement`,
  `ImportSource::Statements` ; `currency_order` (ordre des relevés, garde `Internal`) ; `file_hash(Option<&[u8]>, …)` ;
  refus d'empreinte pour fichier et relevés ; continuité par relevé avant le saut « tout en doublons » ; période et
  soldes du relevé ; trace `files` (relevé + compte rendu) au lieu de `currencies` ; lecture des existants par
  `existing_transactions_by_ids` ; `wise_import.created` à `AUDIT_ACTIONS`.
- `routes/wise_imports.rs` : `line_of_row` (et `transaction_of_row` par elle) ; `create` par `load_balances` +
  `import_lines` ; `FilePart` (partie fichier du compte rendu, et sa trace) ; retirés `Target`, `existing_keys`,
  `WiseWriter`, `FileWarning` local (ré-exporté depuis le cœur, comme `WARNING_GAP`).
- `routes/paypal_imports.rs` : la version d'`epic/38-paypal` (`8b8cd338`), plus les bornes et `truncate_*` en
  `pub(crate)` (Wise s'en sert) et les deux champs neufs au littéral `ProviderLine` : le cœur 39-11b (onze éléments,
  374 lignes) disparaît.
- Tests : `audit_label_registry.rs` (`SITES_INDIRECTS`), `audit_route_registry.rs` (commentaire (vi)),
  `paypal_import_e2e.rs` et `paypal_download_e2e.rs` (clés exactes du compte rendu, réponse et trace).
- **Réponses de l'API** : inchangées (AC-2, tenu par les suites inchangées et le test AC-5) ; seuls les textes du
  journal serveur de D7 changent.

### Tests (recomptés, `#[test]` + `#[sqlx::test]`, de `1f03f180` à `35e69ea6`)

`provider_imports.rs` 3 → 7 (+4 : `les_cles_existantes_lisent_le_sens_au_signe`, reprise du test d'`existing_keys` de
Wise, et trois neufs : `dans_la_source_le_doublon_tient_compte_du_sens`, `l_empreinte_d_un_releve_est_celle_de_ses_octets`,
`l_ordre_des_releves_et_leur_couverture`) ; `wise_imports.rs` 11 → 10 (le test d'`existing_keys`, passé au cœur) ;
`paypal_imports.rs` 10 → 10 ; `wise_import_e2e.rs` 14 → 15 (`import_raccord_reponses_traces_et_ordre_inchanges`).
Soit **4 tests neufs, 1 déplacé**, et des assertions ajoutées à deux tests existants (PayPal fichier et téléchargement).

Le test AC-5 a été joué **vert sur le code d'avant** (`36b8a23b` = `1f03f180` + le test,
`kesh-gate-logs/s39-14/ac5-avant.log`, 1 passé), puis vert après.

### Gates (au dernier commit de code `35e69ea6`)

- `cargo fmt --check` vert ; `cargo clippy --workspace --all-targets -D warnings` vert (`clippy.log`).
- Gate ciblé (wise, paypal fichier et téléchargement, deux registres, tests unitaires des trois routes) : 92 passés
  (`cible-3.log` ; `cible-2.log` arrêté par l'orchestrateur, mémoire de la station, rejoué).
- **nextest complet** (`--workspace --profile ci`, base `kesh_wt39n` remise à zéro, migrée, seedée par
  `s39-14/run.sh`) : **6504 passés, 17 ignorés** (`gate-complet.log`).
- Frontend non touché : non joué. **E2E partiel : aucun** (aucune spec ajoutée, règle de la nuit du 22.11) ; E2E
  complet et `gate-ci.sh` à la fin de l'epic.

### Sondes par mutation (hors dépôt, `kesh-gate-logs/s39-14/mutations.py`, copie `git archive` de `35e69ea6`, cible distincte `kesh-targets/kesh-wt39n-mut`, supprimée)

Filtre : `wise_import_e2e`, `paypal_import_e2e`, `paypal_download_e2e` (39 tests). Témoin non muté vert (39/39).
« Tuée » = un test nommé en échec sur une assertion (`panicked at`), relevé à chaque journal `mut-<nom>.log` :

| Mutation | Tuée par |
|---|---|
| ordre alphabétique rétabli pour les relevés | `import_raccord_reponses_traces_et_ordre_inchanges` |
| période des lignes gardées au lieu du relevé | `import_raccord_reponses_traces_et_ordre_inchanges` |
| sens ignoré (`ExistingKeys::contains` par identifiant seul) | `import_meme_identifiant_au_debit_puis_au_credit`, `import_raccord_…` |
| `warnings` omis du compte rendu Wise | `import_continuite_et_doublons_de_lignes`, `import_raccord_…` |
| continuité sautée pour un relevé tout en doublons | `import_raccord_reponses_traces_et_ordre_inchanges` |
| `warnings` sérialisé pour PayPal (`skip_serializing_if` retiré) | `paypal_download_e2e::quatre_mois_ventiles_par_devise`, `paypal_import_e2e::import_ventile_par_devise_puis_doublons` |

Les deux premières jouées dans un même lancement (`mutations-1.log`), arrêté par l'orchestrateur (mémoire de la
station) ; les quatre autres une à une, chacune dans son appel de `heavy.sh`, `CARGO_BUILD_JOBS=4`
(`mutations-<nom>.run.log`). Synthèse : `mutations-synthese.log`.

### Fichiers communs et points relevés

- Avec `story/39-11c` : `tests/audit_route_registry.rs` (commentaire du point (vi) seul) et `sprint-status.yaml`.
- Avec `epic/40-stripe` : `provider_imports.rs` et `paypal_imports.rs` (§ *Fichiers communs*).
- **À tracer en issue par l'orchestrateur** (cette story n'ouvre pas d'issue) : point 10 de la 39-11b
  (`json.paypal.or(json.wise)`, `bank_account.rs:215`, et `served()` pour l'Epic 40) ; commentaires
  d'`existing_transaction_ids` et d'`existing_transactions_by_ids` (`kesh-db`) devenus inexacts, la première sans
  appelant dans `kesh-api`.

### Remédiation de la revue de code P1 (`revue-p1-rapport.md`)

- M1 : § *Fichiers communs* corrigée (trois sites `ProviderLine` dont `paypal_imports.rs`, plus `AUDIT_ACTIONS` ; bornes et `truncate_*`
  **copiés** par `epic/40-stripe`, deux définitions à unifier), fusion à blanc rejouée dans un clone hors dépôt
  (supprimé) : les conflits de cette story sont bien ceux-là, les autres (`errors.rs`, `lib.rs`, `bank_accounts.rs`,
  registres, i18n, documents) viennent d'autres stories.
- L3 : doc-comments d'`ImportRequest::source`, `ImportRequest::audit_details` et `CurrencyReport` à jour.
- L4 : le test AC-5 tient aussi `entity_id` de la trace `wise_import.created` (plus petit des trois comptes).
- L1 (`dedupe_groups` sans appelant de production, gardée pour Stripe, D3) et L2 (ré-export `WARNING_GAP`/`FileWarning`
  de `wise_imports.rs`, reliquat D4) : **non traités**, à voir à la fusion avec `epic/40-stripe` (retirer ce que
  Stripe n'utilise pas).

## Change Log

- 2026-11-22 — **Clôture** : revue P3 ciblée (Claude Sonnet 5.5, `revue-p3-rapport.md`, sur le commit de fiche `e7ce9519`) :
  0 C, 0 H, 0 M, 2 L. Boucle close : validation P1 → P3, revue de code P1 (Sonnet 0/0/1/4) → P2 (Haiku ciblée 0/0/2/2)
  → P3 (Sonnet ciblée 0/0/0/2), la dernière remédiation ne touchant aucune ligne de production. Story `done`, fusionnée
  sans squash dans `epic/39-wise`.
- 2026-11-22 — Revue de code P1 (Claude Sonnet 5.5, `revue-p1-rapport.md`) : 0 C, 0 H, 1 M, 4 L ; aucune différence de
  comportement trouvée. M1 (fiche : quatre sites, copie des bornes par Stripe), L3, L4 remédiés ; L1, L2 notés au
  Dev Agent Record. Remédiation : fiche, commentaires, une assertion de test ; aucune ligne de production. Passe P2
  ciblée : `revue-p2-prompt.md`.
- 2026-11-22 — Développement (`35e69ea6`) : raccord fait ; gates fmt, clippy, nextest complet 6504/17 verts ; six
  sondes tuées ; story en `review`, prompt de revue P1 `kesh-gate-logs/s39-14/revue-p1-prompt.md`.
- 2026-11-22 — Validation P3 (Claude Sonnet 5.5, ciblée sur la remédiation `41cb5018`, `validation-p3-rapport.md`) :
  0 C, 0 H, 1 M, 3 L. M-A (grep de l'AC-1 insatisfiable : `fn existing_keys` rendait `existing_keys_with_texts`) :
  motif corrigé en `fn existing_keys(`. L-A : le test nommé existe dans le développement (le relecteur lisait
  `8b17e53b`), cité en T4 ; L-B : fichiers PayPal aux modules touchés et en T4 ; L-C : point 4 d'epic/40 complété.
  **Validation close au plafond de trois passes fixé pour cette story**, sur un MEDIUM corrigé sans passe de plus
  (remédiation de fiche seule) : recontrôle à l'étape 0 du développement, déclaré au Dev Agent Record. Tendance :
  P1 1 H / 3 M → P2 2 M → P3 1 M (ciblée).
- 2026-11-22 — Validation P2 (Claude Opus 5.5, contexte frais, lecture seule, `validation-p2-rapport.md`) : 0 C, 0 H,
  2 M, 7 L. Remédiations de P1 tenues. M1 : clés exactes du compte rendu PayPal (réponse, trace, fichier et
  téléchargement) en AC-2 ; M2 : `epic/40-stripe` aux fichiers communs, `dedupe_groups` gardé (D3). LOW : L1
  (empreinte stockée en AC-5), L2 (issue à tracer), L5, L6, L7 écrits ; L3 (grep : `WriteFuture`, `CurrencyImport`,
  `WiseWriter`, `existing_keys` ajoutés à l'AC-1) ; L4 (test `les_cles_existantes_lisent_le_sens_au_signe` couvre la
  ligne sans sens au débit, au crédit et à zéro).
- 2026-11-22 — Validation P1 (Claude Sonnet 5.5, contexte séparé, lecture seule, `kesh-gate-logs/s39-14/validation-p1-rapport.md`) :
  0 C, 1 H, 3 M, 7 L. H1 (`SITES_INDIRECTS`) : prémisse corrigée, AC-4 ; M1 : tableau des points 1 à 10 ; M2 : D3
  précisé ; M3 : ordre de la continuité en D1, cas « tout en doublons » ajouté à AC-5. LOW : `flatten` écarté (D5
  construit `FileReport` champ à champ, ordre des clés gardé), cas `Internal` testé, E2E dit ; les autres sans suite
  (clone des groupes, grep de l'AC-1 qui vise l'état final).
- 2026-11-22 — Fiche rédigée (raccord, liste « À raccorder » de la 39-11b, points 1 à 10 ; points 4 et 6 relevés au
  code ; doublon par sens ajouté).

### Revue P2 ciblée (Haiku, `7e234beb`) et remédiation de l'orchestrateur

- P2 : 0/0/2/2. M1 (« quatre sites » pour trois énumérés) et M2 (liste des autres conflits incomplète :
  `bank_account.rs`, `bank_account_opening_balance_e2e.rs`) corrigés dans § *Fichiers communs* : trois sites
  `ProviderLine` plus `AUDIT_ACTIONS`, autres conflits attribués aux autres stories de l'Epic 39 et listés à la fusion
  réelle. L1 (aucun conflit à blanc sur `audit_route_registry.rs` avec la 39-11c) et L2 (`value_date: None` à ajouter au
  littéral de `wise_imports.rs`) intégrés à la même section. Fiche seule, aucune ligne de code ; passe P3 ciblée à suivre.
