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
- **Conséquence écrite** (CHANGELOG, AC12 ; manuel d'administration, AC12-bis) : avant un retour à une 0.13.1,
  retirer la colonne Devise de **tous** les profils qui la lisent.
- **Portée réelle de la dégradation** (amendé en validation P1, lentilles R et F, vérifié aux tags) : **un seul** profil
  à colonne Devise fait échouer `GET /api/v1/bank-profiles` (500) pour **toute la société** sous la 0.13.1 — la liste
  désérialise chaque profil (`BankProfileResponse::try_from_db`, `routes/bank_profiles.rs:66` de la 0.13.1). L'écran
  *Profils bancaires* est inutilisable ; l'écran d'import **avale** l'erreur (`git show
  v0.13.1:frontend/src/lib/features/bank-import/BankImportUpload.svelte`, l. 66-71 : `catch { availableProfiles = []; }`)
  et affiche un sélecteur de profil **vide, sans message** ; l'import CAMT n'est pas touché. Le profil ne se corrige
  plus par l'écran sous la 0.13.1 (le `GET` par identifiant échoue aussi) : la colonne se retire **avant** le retour,
  sous la 0.13.2 — oubliée, elle se retire encore **par l'API** de la 0.13.1 (`PUT` complet sans
  `columnMapping.currency`, ou `DELETE` : ni l'un ni l'autre ne désérialise le profil enregistré, `git show
  v0.13.1:crates/kesh-db/src/repositories/bank_profiles.rs` l. 264 compare la chaîne brute ; amendé en validation P2,
  F7), le SQL restant le dernier recours. Une sauvegarde d'installation 0.13.2 s'importe dans une 0.13.1 (pas de bump au
  manifeste), avec la même conséquence. Rien n'est importé à tort, aucune écriture : bruyant. ⚠️ Tout cela suppose
  qu'aucune migration de la 0.13.2 ne relève `kesh_version_min_required` (vrai pour 27-1, 27-2, 27-3 au 2026-10-12 ;
  validation P2, F10).
- **La prémisse est figée** (validation P2, F1 = R10 ; C-27-2-15) : ce raisonnement tient tout entier au
  `deny_unknown_fields` de `ColumnMapping` dans le binaire **antérieur**. La 0.13.2 sera le binaire antérieur de la
  prochaine clé : l'attribut y est gardé par un test (AC1-(5), mutation M23) et justifié au doc-comment de la struct,
  là où une revue future lira l'exception — et non plus dans ce seul registre.
- **Lecture de P1** : la définition de P1 (« un état du schéma qu'un binaire antérieur ne peut plus consommer
  correctement »), prise à la lettre, couvrirait ce cas. Elle est lue **à la lumière de l'objet de la politique** — sa
  phrase d'ouverture : « protège contre les downgrades **silencieux corrupteurs** » — et de sa forme (P2 prescrit un
  `UPDATE` **dans une migration**, P3 énumère des DDL). Ici, aucun DDL, et le binaire antérieur **refuse** au lieu
  d'importer faux. C'est une exception **assumée**, par analogie à P4 (une opération qui reste techniquement sans
  danger pour un binaire antérieur), écrite ici pour qu'une revue ne la classe pas CRITICAL « sans bump » par lecture
  littérale de P3 — qui ne vise d'ailleurs que des migrations. Le précédent `debit_sign` (26-1) est le cas inverse et
  confirme la ligne : la 0.13.0 aurait **ignoré** la colonne et inversé les débits **en silence** → bump.
- **Reste, assumé** (validation P1, F5-b) : un onglet de la 0.13.1 resté ouvert pendant la mise à jour, ou un client
  d'API, qui fait un `PUT` sans `currency` ramène le profil au CHF supposé, sans erreur (remplacement complet) ; seul
  l'avertissement `bank_csv_currency_assumed` le signale ensuite. Poste unique : risque faible.
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
  **contrôle** CAMT n'est pas touché. *(Amendé en validation P1 : l'hypothèse sur `:168-178` est confirmée au code par
  les deux lentilles ; le **parseur** CAMT est corrigé à l'ancrage de la devise — C-27-2-11 —, ce qui rend la
  transaction d'un paiement par carte CHF de nouveau. L'extension du contrôle à chaque transaction reste écartée :
  elle n'apporte rien tant qu'aucun compte ne porte de devise, C-27-2-4.)*
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
- **Étendu en validation P1** (R4-1) : les autres textes du même refus qui disent « v0.1 » hors des `.ftl` suivent —
  `crates/kesh-api/src/errors.rs:673-674` (doc-comment et message `thiserror`), `:2141` (repli de
  `bank-import-errors-unsupported-currency`, aligné sur son texte `fr-CH`), `docs/i18n-glossaire.md:161` (exemple).
- **Réversible** : oui.

### C-27-2-7 — L'écran du profil : un champ de plus, traduit ; le reste du formulaire n'est pas repris

- **Retenu** : champ « devise (facultative) » (`data-testid="col-currency-input"`), vide par défaut, initialisé depuis
  `existing.columnMapping.currency`, envoyé seulement s'il est rempli, avec une aide traduite. Libellé et aide
  passent par `i18nMsg` (clés neuves, quatre locales).
- **Non repris, signalé** : les libellés voisins (`date`, `débit`, `crédit`, `montant`, `référence`, `détails`,
  `BankProfileForm.svelte:259-283`) sont du français en dur. *(Amendé en validation P1 : la perte du `counterparty`
  posé par l'API, d'abord signalée ici avec une issue demandée, est **corrigée dans la story** — C-27-2-12, AC8-bis —
  sur décision de l'orchestrateur ; plus d'issue à ouvrir. Le formulaire n'a toujours pas de **champ** contrepartie.)*
- **Réversible** : oui.

### C-27-2-8 — Le profil UBS recommandé lit la colonne 4

- **Retenu** : le tableau du manuel (*Le relevé CSV d'UBS*) gagne « Colonne devise : 4 (« Devise ») » ; le test
  local sur le vrai relevé (`crates/kesh-import/tests/ubs_fichier_reel.rs`, ignoré par défaut) mappe la colonne 4 et
  vérifie que **chaque** transaction et le relevé sont en CHF — contrôle rendu « conforme / non conforme »
  seulement, lancé par l'orchestrateur (aucune donnée réelle dans le dépôt).
- **Réversible** : oui.

### C-27-2-9 — Fixtures : dérivées de `ubs_synthetique.csv`, jamais du fichier réel

- **Retenu** : les cas EUR et mélange se **dérivent dans le test** de `crates/kesh-import/tests/fixtures/csv/ubs_synthetique.csv`
  (remplacement de `;CHF;` sur les **seules** lignes de données : le préambule porte aussi `;CHF;`, `Évaluation
  en:;CHF;` ligne 7 — 12 occurrences pour 11 lignes ; le helper saute préambule et en-tête et asserte le nombre de
  remplacements, validation P1 R9-1 ; il saute `header_row_count` lignes **non vides** — la fixture a dix lignes
  physiques avant la première donnée pour `header_row_count = 9`, la ligne 9 étant vide et avalée par le lecteur `csv`,
  validation P2 R5) — une seule source ; les cas limites (cellule vide, `chf`,
  `Fr.`, collision, hors-borne) en petits CSV écrits dans le test. Aucun fichier neuf sous `fixtures/csv/`, rien lu
  sous `~/kesh-test/`. *(La seule fixture neuve de la story est CAMT, synthétique : C-27-2-11.)*
- **Réversible** : oui.

### C-27-2-10 — Hors périmètre, signalés et non traités

- ~~**CAMT.053, devise par transaction**~~ — *sorti de cette liste en validation P1* : hypothèse confirmée au code par
  les deux lentilles (`camt053/mod.rs:168-178` contre `:372`, `:427`), corrigée dans la story (C-27-2-11, AC19). Plus
  d'issue à ouvrir.
- **Le préambule UBS** (`Évaluation en:`) : #621.
- **Semver de `kesh-import`** : un champ public ajouté à `ColumnMapping`, un variant à `CsvError` et à
  `CsvLineErrorCode` — même nature que les ajouts de la 26-1 (`CsvProfile.debit_sign`, `HeaderSkipsData`,
  `AmountSignMismatch`), sans `#[non_exhaustive]` : le crate n'est pas publié, le `--dry-run` reste le seul
  contrôle exigé.

### C-27-2-11 — CAMT.053 : la devise d'une transaction se lit sur l'élément qui porte son montant (validation P1)

- **Contexte** : les deux lentilles de P1 confirment au code l'hypothèse de C-27-2-3 : à l'ouverture de **tout**
  `<Amt Ccy>`, `camt053/mod.rs:168-178` écrase la devise de la transaction (sous `<TxDtls>`) ou de l'entrée (sinon),
  le dernier rencontré gagnant, alors que le montant ne se lit que sur `TxDtls/Amt` (`:427`) et `Ntry/Amt` (`:372`). Un
  `<AmtDtls><InstdAmt><Amt Ccy="EUR">`, des frais `<Chrgs>` ou des intérêts `<Intrst>` en devise étrangère donnent à
  une transaction d'un compte CHF la devise EUR, au montant en francs ; la réconciliation l'écarte ensuite. Aucune
  fixture du dépôt ne porte `<AmtDtls>` ni `<Chrgs>` (`grep -n "Chrgs\|AmtDtls" crates/kesh-import/tests/fixtures/camt053/*.xml`
  → aucune sortie), d'où l'absence de test qui l'aurait montré.
- **Retenu** (périmètre élargi par l'orchestrateur, même sujet que #624 : la devise des imports ; pas d'issue neuve) :
  la devise n'est retenue que si l'`Amt` est l'enfant **direct** de `<TxDtls>` ou de `<Ntry>` (`path.last()` à
  l'ouverture, `Amt` n'étant poussé qu'après) — le même élément que le montant. Fixture synthétique
  `v04_amtdtls_foreign_currency.xml`, tests écrits et vus **rouges** avant la correction, test d'intégration, mutations
  M19 (bras `TxDtls`) et M20 (bras `Ntry`).
- **Écartées** : contrôler chaque transaction contre CHF (C-27-2-3 : refuserait à tort un relevé CHF à paiement par
  carte en EUR) ; préférer `AmtDtls/TxAmt` quand il existe (le montant enregistré est celui de `TxDtls/Amt` : la devise
  doit être la sienne) ; laisser en l'état avec une issue (le défaut est dans le même sujet, et le nombre de bugs doit
  baisser à la release).
- **Angle mort assumé** (validation P2, F8) : une entrée dont l'`Amt` direct n'a pas de `Ccy`, mais dont un montant
  d'information en porte un, en prenait la devise ; elle est désormais refusée (`MissingRequiredField(currency)`,
  `camt053/mod.rs:572-575`), et avec elle tout le relevé. Le XSD rend `Ccy` obligatoire : seul un exporteur non conforme
  produit ce cas ; le refus est bruyant ; reprendre la devise d'un montant d'information serait le défaut même. Pas de
  test exigé.
- **Réversible** : oui (un ancrage dans le parseur, aucune donnée persistée autrement qu'avant pour un fichier sans
  montant d'information).

### C-27-2-12 — Le formulaire de profil garde toute clé qu'il ne montre pas (validation P1, F1)

- **Contexte** : `BankProfileForm.svelte:56-69` reconstruit `columnMapping` de ses seuls champs ; le `PUT` remplace le
  profil entier ; un `counterparty` posé par l'API est donc **perdu** au premier enregistrement à l'écran — chemin que
  la story prescrit justement (ajouter la colonne Devise aux profils existants).
- **Retenu** (périmètre élargi par l'orchestrateur, pas d'issue neuve) : le mapping envoyé part du mapping existant
  **privé des clés que le formulaire possède** (une constante nommée : `date`, `amount`, `debit_credit_split`,
  `reference`, `details`, `currency`), puis reçoit les valeurs du formulaire. Garde `counterparty` aujourd'hui, et
  toute clé future que le formulaire ne montrerait pas. Forme de la constante et de la fonction : C-27-2-17 ; les deux
  sens du changement de mode sont testés (AC8-bis (v7) et (v8), validation P2, R1).
- **Écartées** : recopier `counterparty` seul (le défaut renaîtrait à la prochaine clé) ; recopier tout le mapping
  existant (un passage colonne unique → mode séparé laisserait `amount` **et** `debit_credit_split` : refus `422`) ;
  ajouter un champ contrepartie (hors sujet, et rien ne l'exige).
- **Réversible** : oui.

### C-27-2-13 — Le plafond de 10 devises rapportées : les 10 premières du tri, sans indicateur (validation P1, R6-1)

- **Retenu** : `MixedCurrencies.currencies` porte les 10 premières de l'ensemble **trié** des devises distinctes des
  lignes valides (un `BTreeSet`), jamais les 10 premières rencontrées ; aucun indicateur de troncature — le message
  « le relevé porte plusieurs devises (…) » reste vrai, et un relevé à plus de 10 devises n'est pas un relevé de compte.
  Le test AC13-11 place le code qui se trie en dernier sur la **première** ligne, pour départager.
- **Écartée** : un champ `truncated` / `total` (patron des erreurs de ligne) — un champ d'API de plus pour un cas que
  rien ne produit.
- **Réversible** : oui (ajout d'un champ facultatif).

### C-27-2-14 — Le mélange de devises ne se juge que sur les lignes valides (validation P1, R7-1 / F4)

- **Contexte** : la devise est lue **après** la date et le montant (C-27-2-5) : une ligne écartée pour une autre raison
  ne révèle jamais sa devise.
- **Retenu** : assumé et écrit (AC3, manuel § *Rejet partiel*). Un fichier CHF dont les lignes EUR sont toutes
  invalides pour une autre raison se confirme en partiel et importe ses lignes CHF, **exactes** ; les lignes EUR restent
  listées comme invalides — aucun montant faux n'entre en comptabilité.
- **Écartée** : lire la devise avant la date et le montant (une ligne illisible donnerait un refus de mélange sur une
  cellule peut-être décalée — moins lisible que l'erreur de ligne d'aujourd'hui, et sans gain comptable).
- **Réversible** : oui.

### C-27-2-15 — `deny_unknown_fields` de `ColumnMapping` : figé par un test et justifié au code (validation P2, F1 = R10)

- **Contexte** : C-27-2-1 (aucune migration, aucun bump) repose entièrement sur le fait qu'un binaire **antérieur**
  refuse une clé inconnue de `ColumnMapping` au lieu de l'ignorer. Aucun test ne gardait l'attribut
  (`grep -rn "deny_unknown\|unknown field" crates/kesh-import/src crates/kesh-db/src crates/kesh-api/tests` → la seule
  ligne `profile.rs:37`), et l'exception n'était écrite qu'ici. Or la 0.13.2 sera le binaire antérieur de la prochaine
  clé : l'attribut retiré, ce retour-là deviendrait **silencieux** — le downgrade corrupteur que la politique vise.
- **Retenu** : un test AC1-(5) (`unknown_key_is_rejected_so_older_binaries_refuse_loudly`, clé `bogus` → `Err`) que tue
  la mutation M23 (attribut retiré) ; un doc-comment `///` sur `ColumnMapping` qui dit le contrat (ajouter un champ =
  refus bruyant par les binaires antérieurs, sans bump ; retirer l'attribut = retour silencieux depuis la version
  suivante) et renvoie à C-27-2-1.
- **Écartées** : un garde-fou dans `textes_coherents.rs` ou un test d'architecture (plus lourd ; le test sérde suffit à
  faire rougir) ; ne rien faire (l'exception resterait invérifiable par qui lit le code).
- **Réversible** : oui.

### C-27-2-16 — Les transactions déjà importées avec une devise fausse : remède écrit, rien de corrigé d'office (validation P2, F2)

- **Contexte** : les deux défauts corrigés ont pu écrire des données fausses — un CSV d'un compte en devise importé en
  `CHF` au montant du fichier (#624), un paiement CAMT par carte resté « en EUR » au montant en francs (AC19), que la
  réconciliation écarte (`reconciliation.rs:631`, `:2728`). La mise à jour ne les touche pas ; le réimport seul ne les
  remplace pas (fichier en doublon, `BANK_IMPORT_DUPLICATE_FILE` ; empreinte de ligne sans la devise,
  `routes/bank_imports.rs:1532-1539`).
- **Retenu** : écrire le remède au CHANGELOG (AC12) et au manuel d'administration (AC12-bis, paragraphe
  « Transactions déjà importées », patron de `admin-manual.tex:1993`) : **supprimer l'import non rapproché** (Story 27-1,
  même release) puis le réimporter ; s'il a des transactions rapprochées, la suppression est refusée
  (`409 BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS`, fiche 27-1, AC5) — annuler d'abord leur rapprochement (*Annuler le
  rapprochement* contre-passe l'écriture propre ou annule le règlement, `reconciliation_cancel.rs`, doc-module), puis
  supprimer et réimporter. Les transactions CAMT restées « en EUR » ne sont pas rapprochables : leur import est
  supprimable, sauf si une autre de ses transactions est rapprochée. Un CSV d'un compte en devise réimporté avec la
  colonne Devise est refusé : il n'a pas sa place dans Kesh (C-27-2-4).
- **Écartées** : une migration de correction (impossible : rien en base ne dit quelle transaction CSV venait d'un compte
  en devise, et une transaction CAMT ne garde pas son XML) ; un outil de réparation dédié (hors périmètre ; la
  suppression d'import de la 27-1 suffit) ; le silence (le précédent de la 0.13.1 écrit ce paragraphe pour le sens des
  débits).
- **Dépendance** : le remède suppose la 27-1 dans la même release. Si la 27-1 n'y était pas, le paragraphe se récrit
  (SQL ou restauration) — à revérifier par l'orchestrateur à la fusion.
- **Réversible** : oui (texte).

### C-27-2-17 — Les clés possédées par le formulaire : une constante, une fonction, aucune liste répétée (validation P2, F4 = R2)

- **Contexte** : la remédiation P1 prescrivait à la fois une déstructuration (qui écrit la liste des clés dans son
  motif) et « une constante nommée partagée » — deux sources de vérité ; et la déstructuration de
  `existing?.columnMapping ?? {}` (type `ColumnMapping | {}`) risquait l'erreur TS2339 sous `strict: true`
  (non compilé, déduit par les deux lentilles).
- **Retenu** : `OWNED_COLUMN_MAPPING_KEYS = [...] as const satisfies readonly (keyof ColumnMapping)[]` et
  `keepUnownedKeys(existing?: ColumnMapping): Partial<ColumnMapping>` qui copie le mapping puis `delete` chaque clé
  possédée — sans `any` ni cast ; `satisfies` est déjà employé au dépôt (TypeScript 5.9.3). Mutations M22 (tout
  recopier) et M24 (constante privée d'une clé) tuées par (v7)/(v8).
- **Écartées** : la déstructuration seule (pas de constante, liste dans le motif, typage incertain) ; un filtre
  `Object.fromEntries(Object.entries(…))` (perd le typage, cast nécessaire).
- **Réversible** : oui. *Non compilé à la rédaction : `npm run check` le confirme en T4.*

### C-27-2-18 — Hypothèse R9 (P2) : un `<TxDtls>` sans `Amt` propre prend le montant de l'entrée entière — non établie comme défaut, rapportée

- **Hypothèse** (lentille R, P2, R9) : en camt.053.001.**04**, `EntryTransaction4` n'aurait pas d'`Amt` direct ; un vrai
  relevé à plusieurs `<TxDtls>` par `<Ntry>` donnerait alors à chacun le montant de l'entrée (`t.amount.unwrap_or(
  ntry_amount)`, `camt053/mod.rs:603`).
- **Ce que le code établit** (lu) : le repli `:603` donne à **tout** `<TxDtls>` sans `TxDtls/Amt` direct le montant de
  l'entrée, quelle que soit la version (`v04.rs` et `v08.rs` ne font que dispatcher vers le même parseur). Avec **un**
  `<TxDtls>`, c'est juste ; avec **deux ou plus** dont l'un n'a pas d'`Amt` direct, la somme des transactions dépasse le
  montant de l'entrée. Le contrôle de solde le signale (`BankImportBalanceMismatch`), mais il est **confirmable**
  (`confirmBalanceMismatch`, `routes/bank_imports.rs:433`) : bruyant, pas bloquant.
- **Ce que le schéma établit** : dans le seul XSD versionné (001.08,
  `docs/six-references/ig-cash-managment-xml-schemas-v2.0.2-en/camt.053.001.08.xsd:705`), `EntryTransaction10/Amt` est
  `minOccurs="0"` — l'absence est donc **conforme** en 001.08 aussi. Aucun XSD 001.04 au dépôt : l'affirmation
  « `EntryTransaction4` n'a pas d'`Amt` » n'est **ni établie ni réfutée** ici (à ma connaissance elle est fausse —
  `Amt` et `CdtDbtInd` figurent, facultatifs, dans le `TxDtls` de la 001.04, et les fixtures v04 du dépôt les portent —,
  mais une connaissance non sourcée n'est pas une preuve).
- **Ce que les fixtures établissent** (comptage par `<Ntry>`, `awk` sur les 13 fichiers) : deux fixtures portent
  plusieurs `<TxDtls>` par entrée — `v04_with_subtxs.xml` (3, tous avec `Amt` direct) et `v04_ntry_sign_fallback.xml`
  (2, tous avec `Amt` direct) ; les treize autres entrées
  à `<TxDtls>` (dont celle de `v04_prefixed_namespace.xml`, comptée à part, ses balises étant préfixées ;
  `v04_truncated.xml`, coupé, exclu) en ont **un seul**, sans `Amt` direct (repli juste).
  **Aucune fixture n'exerce le cas litigieux** (≥ 2 `<TxDtls>` dont un sans `Amt` direct) : aucun test ne le couvre, ni
  dans un sens ni dans l'autre.
- **Conclusion** : comportement **établi au code** pour une forme d'entrée conforme au XSD 001.08 mais **dont l'existence
  chez une banque n'est pas établie** ; ce n'est donc pas un défaut établi. Pour l'établir, il manque : (1) une source
  qui montre la forme en usage — un relevé réel d'une banque suisse (contrôlé en local, rapporté « présent / absent »),
  ou le guide SPS camt.053 (SIX) qui dirait `TxDtls/Amt` facultatif et non rempli pour les lots ; (2) une fixture
  synthétique à deux `<TxDtls>` dont un sans `Amt`, et un test qui montre la somme fausse. Le comportement correct
  serait alors à trancher (refuser l'entrée, ou répartir via `AmtDtls/TxAmt`).
- **Retenu** : **hors périmètre** de la 27-2 (AC19 ne change rien à ce repli : la devise y suit l'entrée, ce qui est
  juste) ; rapporté à l'orchestrateur, qui décide d'une issue.
- **Réversible** : sans objet.

### C-27-2-19 — Les « v0.1 » restants du refus de devise : réécrits là où ils décrivent ce refus, gardés là où ils racontent l'histoire (validation P2, R7)

- **Contexte** : le grep prescrit en P1 laissait hors champ trois textes du même refus — `kesh-core/src/errors.rs:93`
  (message au journal), `kesh-import/src/error.rs:5` et `kesh-core/src/bank_imports.rs:25`, `:457` (doc-comments).
- **Retenu** : les réécrire (« v0.1 » → « cette version », ou retiré du message) dans le même patch qu'AC10, et élargir
  le grep de contrôle à `crates/` (`grep -rnF "v0.1" crates --include=*.rs | grep -i "devise\|currency"`). Gardés : le
  nom `validate_currency_supported_v0_1` (C-27-2-3) et les mentions « v0.1 mono-CHF » des tests et de la réconciliation
  (`reconciliation.rs:628`, `reconciliation_e2e.rs`, `bank_imports_e2e.rs:443`, `bank_imports.rs:1125`), qui décrivent
  une convention toujours vraie.
- **Écartée** : renommer la fonction (bruit de diff sans gain ; C-27-2-3).
- **Réversible** : oui.
