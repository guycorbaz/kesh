# Story 33-2 : l'import CAMT refuse le relevé qu'il ne sait pas relire « à la 0.17.3 », et l'alias ne signale plus un faux doublon

Status: ready-for-dev

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`,
numéros de ligne de `4ef19901`. Epic 33, release **0.17.5**, **sans migration**. Registre :
`e33-choix-autonomes-e332.md`, choix `C-e332-k`. Issues **#922** (faux négatif) et **#915** (faux positif), en `refs` dans les
commits, `closes` dans la seule PR. Modules : `kesh-import` (un test), `kesh-core`, `kesh-api`, `kesh-i18n`, `frontend`,
manuels : la logique tient dans deux crates, le reste suit.*

## Story

En tant que Comptable qui importe un relevé CAMT.053 dans des livres réels, je veux que Kesh **refuse ce relevé, en me disant
quoi faire,** lorsqu'il ne peut pas vérifier qu'il ne reprend pas une ligne déjà importée, et qu'il **ne prenne plus une ligne
neuve pour un doublon** sur la seule foi de la référence tronquée que lisait la 0.17.3 — sans qu'un relevé déjà importé cesse
d'être reconnu.

## Contrainte — production

⚠️ **Guy a des relevés réels déjà importés en 0.17.x, et la clé de doublon protège ses livres.** Aucun essai sur ses fichiers ni
sur le NAS : fixtures synthétiques, XML écrit dans les tests (`memory/donnees-reelles-hors-depot-public.md`). Aucune
transaction réécrite, aucune migration. Aucune ligne réelle dans les captures.

## Ce que le code dit (4ef19901)

1. **Le lieu.** `legacy_0173_aliases` est dans `crates/kesh-api/src/routes/bank_imports.rs:1980-2006`, non dans `kesh-import`. La
   relecture est `kesh_import::camt053::parse_legacy_0173` (`crates/kesh-import/src/camt053/mod.rs:89-96`) ; la comparaison,
   `detect_duplicate_lines_with_aliases` (`crates/kesh-core/src/bank_imports.rs:446-498`). Deux appelants : l'aperçu
   (`bank_imports.rs:1100`, derrière `if !stmt.transactions.is_empty()`, `:1096`) et la création, sous le verrou du compte
   (`:1348`, appelée sans garde de vide).
2. **Le défaut #922.** `legacy_0173_aliases` a trois sorties vides (`:1986-1995`) : lecture en échec, sélection par IBAN en
   échec, nombre de lignes différent. L'import suit alors son cours sans alias. Le test
   `camt_import_without_alias_when_the_legacy_reading_fails_819` (`crates/kesh-api/tests/bank_imports_e2e.rs:4074`) et son jumeau
   de sélection (`:4089`), par `imports_without_alias_819` (`:4048-4070`), **figent ce défaut** : aperçu sans doublon, trois
   transactions créées. Ils seront récrits, non conservés.
3. **Ce qui fait échouer la relecture — inventaire clos.** La lecture corrigée accepte le fichier (elle passe avant :
   `:1050`, `:1288`). L'écart entre les deux lectures vient du seul `TextMode` (`camt053/mod.rs:202-207`, `:313-355`) : la lecture
   héritée ne voit ni référence d'entité ni CDATA, et distribue chaque morceau de texte seul.
   - **E1, lecture.** Un champ obligatoire entièrement en CDATA ou en référence : date de comptabilisation
     (`emit_transactions`, `MissingRequiredField`), montant, période du relevé, IBAN, devise du compte
     (`finalize_statement`) ; ou coupé en deux par une référence, dont le dernier morceau ne se lit pas (`parse_date`,
     `InvalidDate`, `Decimal::from_str`, `InvalidAmount`).
   - **E2, sélection.** L'IBAN du compte écrit avec une référence : lu tronqué, il ne correspond plus (`select_statement_by_iban`,
     `:932-963`).
   - **E3, nombre de lignes.** Atteignable, contrairement à ce que laisse croire l'absence de test (#923) : la structure
     (`Ntry`, `TxDtls`) ne dépend pas du texte, donc une seule forme y mène, **le fichier à plusieurs relevés du même compte**.
     Si le premier `Stmt` a son IBAN à référence, la lecture corrigée le sélectionne, la lecture héritée l'écarte et sélectionne
     le second (`:939-949`, « le premier gagne »).
   - **E4, désalignement — défaut neuf, le LOW de #923.** La même forme, avec le même nombre de lignes : les deux lectures
     rendent deux relevés différents, et le `zip` (`:1996-2004`) fabrique des alias de lignes sans rapport. Aucune sortie vide,
     aucun signal. **À traiter comme les trois autres** : une paire dont la date de comptabilisation ou le montant diffère veut
     dire que les deux lectures ne parlent pas de la même ligne.

   Il n'y a pas d'autre site : une entité inconnue est refusée **avant**, par la lecture corrigée (`camt053/mod.rs:336-352`) ; la lecture héritée
   l'ignore.
4. **Les relevés ordinaires ne sont pas touchés.** Un fichier sans entité ni CDATA donne « exactement les appels de la lecture
   héritée » (doc-comment `camt053/mod.rs:178-181`) ; la caractérisation le tient
   (`crates/kesh-import/tests/camt053_golden.rs:232`, `legacy_reading_matches_references_produced_under_quick_xml_039`). Un
   relevé d'une banque que la 0.17.3 lisait ne peut donc pas échouer à la relecture. C'est l'argument à prouver par AC-N3.
5. **Le défaut #915.** Un alias ne compare que `(date, montant, référence normalisée, compte)` (`DuplicateKey`,
   `kesh-core/bank_imports.rs:295-300`), et `index` ne garde que le premier `id` par clé (`or_insert`, `:471-474`). Une ligne neuve
   « REF & Co <1> » égale ainsi une ligne existante de référence « 1 », et, par CDATA, toute ligne **sans référence** du même jour
   et du même montant (#915, second commentaire). Elle est écartée par défaut (`apply_duplicate_lines_filter`, `bank_imports.rs:1779-`).
6. **La lecture héritée donne plus que la clé.** Une ligne importée par la 0.17.3 porte en base les **textes** de cette lecture :
   `value_date`, `reference`, `end_to_end_id`, `transaction_id`, `counterparty_iban`, `counterparty_name`, `details`
   (`BankTransactionDraft::from`, `kesh-core/bank_imports.rs:85-100`, puis `new_txs`, `bank_imports.rs:1408-1424`, sans
   transformation). **Ces colonnes ne changent jamais après l'import** : les 17 sites `UPDATE bank_transactions` de production ne
   touchent que `status`, `matched_entry_id`, `auto_match_rejected_at`, `updated_at`, `version`
   (`git grep -n -A5 "UPDATE bank_transactions" -- crates/kesh-api/src crates/kesh-db/src`, `SET` relus : aucun ne nomme une colonne de texte, de date ni de montant). **Et leur calcul n'a pas changé
   entre les versions** : `git log -- crates/kesh-import/src/camt053/mod.rs` ne porte que 076ac863 (création), dc51300e
   (dépendances), e04122dc (devise : d'où `currency` **hors** de la comparaison), 910e5d5a (0.17.0, tenue par la
   caractérisation), c2051805 (0.17.4, #819). Une ligne qu'a réellement importée la 0.17.x a donc en base exactement les textes que
   recalcule `parse_legacy_0173`. C'est ce qui ferme #915 sans rouvrir #922 (C-e332-3).
7. **Le refus n'écrit rien, à condition de rester là où il est.** `create_camt_under_lock` n'écrit qu'à
   `create_import_or_roll_back` (`:1426`) et à la trace (`:1428`) ; tout `Err` avant abandonne la transaction (`:1261-1266`, M-4).
   L'appel `:1348` précède ces deux écritures.
8. **L'écran rend déjà le message.** Toute erreur d'API s'affiche dans `bank-import-error` (`BankImportUpload.svelte:172-173`,
   `:460-464`) : `err.message` vient du serveur (`t(...)`, `errors.rs:2405-2418`), traduit par `kesh-i18n` (quatre locales,
   `bank-import-errors-no-matching-statement`, ligne 1161 en fr, 1104 ailleurs). Aucun code d'écran pour le refus lui-même.
9. **#923.** Sa « Fiche 1 » porte cinq points, tous de cette zone, tous repris ici : voir § *Ce que cette story retire de #923*.
   Les fiches 2 et 3 de #923 restent ouvertes, sans rapport.

## Décisions

Décision de Guy (#922, 2026-11-06, epic) : **refus de ce relevé, message clair**. Le reste est au registre :

- **C-e332-1** : refus inconditionnel dès qu'une des quatre causes survient, sans considérer si le compte a des lignes dans la
  fenêtre, et sans contournement par `confirm*`.
- **C-e332-2** : le désalignement (E4) est une cause de refus.
- **C-e332-3** : l'alias ne reconnaît une ligne que si **tous les textes** stockés égalent ceux de la lecture héritée.
- **C-e332-4** : clé d'alias égale mais textes différents = **ni doublon, ni silence** : la ligne s'importe, et l'aperçu la signale.
- **C-e332-5** : un relevé sans ligne n'est pas refusé.
- **C-e332-6** : 422 `BANK_IMPORT_LEGACY_REREAD_FAILED`, avec la cause en `details`.
- **C-e332-7** : aucune borne dans le temps pour l'alias.

## Critères d'acceptation

### Refus (#922)

- **AC-R1** — Lecture héritée en échec (E1 : date en CDATA) : `POST /bank-imports/preview` **et** `POST /bank-imports` rendent
  `422`, code `BANK_IMPORT_LEGACY_REREAD_FAILED`, `details.cause = "reading"`, message i18n de AC-R7.
- **AC-R2** — Sélection héritée en échec (E2) : idem, `cause = "selection"`.
- **AC-R3** — Nombre de lignes différent (E3 : deux relevés du même IBAN, le premier à référence de caractère, trois lignes contre
  deux) : idem, `cause = "line_count"`. **Cette branche est exercée pour la première fois.**
- **AC-R4** — Même nombre, lignes différentes (E4 : second relevé de trois lignes à d'autres dates) : idem, `cause = "misaligned"`.
  Une paire est désalignée si sa date de comptabilisation **ou** son montant normalisé diffère.
- **AC-R5** — Aucun contournement : `confirmDuplicateLines=import`, `confirmBalanceMismatch=true` et `confirmDuplicateFile=true`
  ensemble ne lèvent pas le refus.
- **AC-R6** — **Rien n'est écrit** : après le refus de la création, `COUNT(*)` de `bank_imports`, `bank_transactions` et `audit_log`
  sont égaux à leur valeur d'avant, et un import valide du même compte, juste après, réussit (ni verrou ni transaction restés).
- **AC-R7** — Message, quatre locales, clé `bank-import-errors-legacy-reread-failed`. Français :
  « Kesh ne peut pas vérifier que ce relevé ne reprend pas des lignes déjà importées par une version antérieure : il n'a rien
  importé. Demandez à votre banque un nouvel export de ce relevé ; à défaut, vérifiez dans la liste des transactions du compte
  quelles lignes y figurent déjà, et saisissez les autres avec un relevé saisi à la main. » Fallback Rust : la version française,
  comme les voisines.
- **AC-R8** — Un relevé **sans ligne** dont la relecture échoue n'est pas refusé : aperçu `200`, création `201`, zéro transaction.

### Pas de refus à tort

- **AC-N1** — Un relevé sans entité : aperçu `200`, création `201`, comme avant.
- **AC-N2** — Un relevé à entités et CDATA dans ses textes, relisible (`first_file_819`) : aperçu `200`, création `201`.
- **AC-N3** — **Toutes** les fixtures du dépôt que la lecture corrigée accepte
  (`crates/kesh-import/tests/fixtures/camt053/*.xml`, 19 fichiers, `frontend/tests/e2e/fixtures/camt053_*.xml`, 4 fichiers), pour
  l'IBAN de leur premier relevé : `legacy_0173_aliases` rend `Ok`. Une exception ne s'admet que nommée, cause dite, au Change Log,
  et si sa construction relève d'E1 à E4. Le test échoue s'il a lu moins de 23 fichiers (pas de test muet).

### Reconnaissance (la protection des livres)

- **AC-S1** — `camt_import_of_0173_stays_recognised_after_the_819_fix` (`bank_imports_e2e.rs:3894`) et
  `camt_import_of_0173_overlap_forced_imports_all_three_819` passent **sans aucune modification de leur corps** : relevé
  enregistré comme la 0.17.3, même fichier refusé par son empreinte, autre fichier reconnu ligne à ligne, deux lignes écartées,
  une importée. La sortie `git diff` de ces deux fonctions est vide, et le Dev Agent Record le dit.
- **AC-S2** — Même preuve avec une référence **en CDATA** (lue vide en 0.17.3) : la ligne enregistrée comme la 0.17.3 reste
  reconnue.
- **AC-S3** — Plusieurs lignes existantes de même clé d'alias, seule la seconde a les mêmes textes : la ligne est reconnue, et
  `existingTransactionId` est l'`id` de la **seconde** (le `or_insert` n'y fait plus obstacle).
- **AC-S4** — Le chemin de clé directe ne change pas : une ligne dont la clé égale une ligne existante est un doublon quels que
  soient ses textes (« le libellé n'en fait pas partie », `user-manual.tex:2716`). Le CSV et la saisie manuelle ne passent aucun
  alias et rendent ce qu'ils rendaient (`detect_duplicate_lines`).

### Faux doublon (#915)

- **AC-F1** — L'exemple de #915 : ligne existante de référence « 1 » au libellé A ; ligne neuve de référence
  `REF &amp; Co &lt;1&gt;`, mêmes date, montant et compte, libellé B. Aperçu : absente de `duplicateLines`, présente dans
  `uncertainLegacyLines` (`{ newIndex, existingTransactionId }`). Création : importée avec le mode par défaut `skip`, et la trace
  `bank_import.created` porte `details.uncertain_legacy_lines_imported = 1`.
- **AC-F2** — Le cas du second commentaire : ligne neuve à référence en CDATA ; ligne existante sans référence, autre libellé,
  même jour, même montant. Même résultat.
- **AC-F3** — Écran : `uncertainLegacyLines` non vide affiche le panneau `warning-uncertain-legacy-lines` (nombre, phrase, tableau
  index et ID existant), **informatif** : il ne désactive pas *Confirmer l'import*. Liste vide : aucun panneau. Quatre locales.
- **AC-F4** — Les **sept** colonnes comparées sont exactement `value_date`, `reference`, `end_to_end_id`, `transaction_id`,
  `counterparty_iban`, `counterparty_name`, `details`, à égalité stricte, **sans** `currency` (e04122dc).

### Reliquat de #923

- **AC-L1** — Test : une entité inconnue dans un élément que Kesh ne lit pas (`<Stmt><Foo>&inconnue;</Foo>…`) rend
  `CamtError::MalformedXml` en lecture corrigée. Le manuel dit déjà « un relevé qui porte une référence d'entité inconnue »
  (`user-manual.tex:2419-2421`) : ajouter « où qu'elle soit dans le fichier ».

## Tâches

### T0 — Étape 0

Relire au code les prémisses 1 à 8 sur le dernier `HEAD` (d'autres stories de l'epic fusionnent) ; citer les lignes relues au
Change Log. `git grep -n "detect_duplicate_lines_with_aliases"` : inventorier les appelants et les tests.

### T1 — Tests rouges d'abord (`crates/kesh-api/tests/bank_imports_e2e.rs`, à côté de `:4040-4098`)

Remplacer `imports_without_alias_819` et ses deux tests par des tests de **refus**, en réutilisant `camt_819`, `montage_819`,
`post_819`, `LINE*_819`. XML synthétique, IBAN du montage.

| Test | AC | Rouge sur `4ef19901` parce que |
|---|---|---|
| `camt_refused_when_the_legacy_reading_fails_922` | R1, R6 | 200/201 au lieu de 422 |
| `camt_refused_when_the_legacy_selection_fails_922` | R2 | idem |
| `camt_refused_when_line_counts_differ_922` | R3 | idem |
| `camt_refused_when_readings_are_misaligned_922` | R4 | idem |
| `camt_refusal_has_no_bypass_922` | R5 | idem |
| `camt_refusal_writes_nothing_then_a_valid_import_succeeds_922` | R6 | idem |
| `camt_without_lines_is_not_refused_922` | R8 | **vert à la création d'origine ; rouge dès T2 sans la garde** |
| `camt_cdata_reference_stays_recognised_922` | S2 | vert : **témoin**, à garder vert à travers T3 |
| `camt_alias_finds_the_row_with_equal_texts_915` | S3 | rend le premier `id` |
| `camt_truncated_reference_is_not_a_false_duplicate_915` | F1 | ligne écartée |
| `camt_cdata_reference_is_not_a_false_duplicate_915` | F2 | ligne écartée |

Dans le `mod` des alias de `routes/bank_imports.rs` (`:2570-` : `aliases_of`, `no_alias_when_*`, récrits) :
`every_repository_fixture_rereads_922` (N3), `line_count_branch_is_reachable_922`, `misaligned_branch_is_reachable_922`.

Dans `kesh-core/src/bank_imports.rs` : tests purs de la comparaison (F4 : **une ligne de test par colonne**, chaque colonne qui
diffère fait passer de doublon à incertain ; S3 ; S4).

Dans `crates/kesh-import/tests/` : `unknown_entity_in_an_unread_element_is_refused` (L1).

### T2 — Le refus (`routes/bank_imports.rs`, `errors.rs`, `kesh-i18n`)

- `legacy_0173_aliases(...) -> Result<Vec<Option<LegacyLine>>, LegacyRereadFailure>`, `enum LegacyRereadFailure { Reading,
  Selection, LineCount, Misaligned }`, avec `new_txs.is_empty()` → `Ok(vec![])` en tête. Doc-comment récrit : l'ancien dit « Jamais
  d'erreur » et « un tel fichier n'a pas pu être importé par la 0.17.3 », ce qui a été le défaut.
- `AppError::BankImportLegacyRereadFailed { cause: &'static str }`, mappé à côté de `BankImportNoMatchingStatement`
  (`errors.rs:2405`), `422`, `tracing::warn!` de la cause seule (aucun contenu du fichier).
- Les deux appels (`:1100`, `:1348`) propagent par `?`. L'aperçu reste derrière sa garde (`:1096`), la fonction porte la sienne.
- ⚠️ Dans l'aperçu, le refus prime sur `warnings.duplicate_file` déjà calculé (`:1086`) ; à la création, `BANK_IMPORT_DUPLICATE_FILE`
  reste premier (`:1276`). Asymétrie sans effet sur les livres, écrite au doc-comment.
- Clé `bank-import-errors-legacy-reread-failed`, quatre locales, à côté des voisines.

### T3 — La comparaison (`kesh-core`, `routes/bank_imports.rs`)

- `kesh-core` : `LineTexts` (les sept champs d'AC-F4, `PartialEq`), `LegacyLine { key: DuplicateKey, texts: LineTexts }` ;
  `detect_duplicate_lines_with_aliases` prend `existing: &[(i64, DuplicateKey, LineTexts)]` et `aliases: &[Option<LegacyLine>]`, et
  rend `DuplicateLines { duplicates: Vec<DuplicateLine>, uncertain: Vec<UncertainLegacyLine> }`. L'index par clé porte **toutes** les
  lignes (`HashMap<&DuplicateKey, Vec<(i64, &LineTexts)>>`). `detect_duplicate_lines` garde sa signature et son résultat, et n'a pas
  à fabriquer de textes (DRY : extraire la partie commune sans lui imposer le nouveau type ; `bank_statements_manual.rs` ne change pas).
- `kesh-api` : `existing_dedup_keys` reste pour le manuel ; ajouter la variante à textes pour le CAMT. Le CSV passe `&[]`.
- `PreviewWarnings.uncertain_legacy_lines: Vec<UncertainLegacyLine>` (`#[serde(skip_serializing_if = "Vec::is_empty")]` n'est **pas**
  posé : champ toujours présent), rempli dans l'aperçu ; à la création, `details_extra["uncertain_legacy_lines_imported"]` quand la
  liste n'est pas vide. **Aucun** nouveau `modifier` : `duplicate_lines_*` reste ce qu'il est.

### T4 — Écran

`bank-import.types.ts` (`uncertainLegacyLines: UncertainLegacyLineWarning[]`) ; panneau après `warning-duplicate-lines`
(`BankImportUpload.svelte:567`), mêmes classes ; clés `bank-import-warnings-uncertain-legacy-lines-summary`,
`bank-import-labels-uncertain-col-index`, `-existing-id`, quatre locales. Français du résumé : « ressemblent à une ligne déjà importée
par une version antérieure (même date, même montant, même référence lue à l'ancienne), mais leur libellé ou leur contrepartie
diffèrent. Kesh les importe : vérifiez-les dans la liste des transactions du compte. » Fixture `BankImportUpload.test.ts:105` étendue ;
tests Vitest : panneau présent, absent, *Confirmer* non désactivé.

### T5 — Manuels et CHANGELOG

- `docs/manual/fr/user-manual.tex`, § *Détection des doublons* (`:2718-2723`) : retirer les « deux limites » et dire ce qui est vrai
  à présent : le refus, ses causes en mots simples, quoi faire ; et le signalement d'une ligne qui ressemble sans être identique.
  Limite restante, dite : une ligne déjà importée en 0.17.3 dont la référence porte un caractère spécial **et** dont la banque
  aurait changé le libellé dans un nouvel export s'importe une seconde fois, **signalée** à l'aperçu. § *Import dans Kesh*
  (`:2419-2421`) : le refus, et AC-L1. FAQ (`:3779`) : une entrée « Kesh refuse mon relevé… ».
- `docs/manual/fr/admin-manual.tex:2553`, *Passer à la 0.17.4* : section d'historique, **non réécrite** ; ajouter une phrase datée
  « Depuis la 0.17.5, ces deux limites sont levées : … \S … ».
- `docs/api-external.md` : le code d'erreur et `uncertainLegacyLines`, au tableau des imports (`:223`, renvoi ⁸).
- `make fr` dans `docs/manual/` ; contrôle des PDF aplatis : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`, phrases cherchées citées
  au Change Log (le refus présent ; « sans aucun signal » absent du manuel utilisateur).
- `CHANGELOG.md`, `## [0.17.5] — Non publié`, *Corrigé* : deux puces, `refs` #922 et #915, qui disent le refus et le signalement
  avec leurs conditions (le reproche de #923 : CHANGELOG plus bref que le manuel). Le paragraphe de la 0.17.4 ne se récrit pas.
- Greper le symptôme : `git grep -nE "sans aucun signal|n'est pas reconnue|Jamais d'erreur|legacy_0173_aliases" -- docs crates`.

### T6 — Captures

Base jetable, thème clair, `/home/gcorbaz/devel/kesh-gate-logs/e332/captures/` : le refus (`bank-import-error`), le panneau
`warning-uncertain-legacy-lines`, un aperçu à entités lues en entier (la capture #819 annoncée et absente, #923). Jointes au Dev
Agent Record et au prompt de revue.

## Ce que cette story retire de #923

Les **cinq** points de la « Fiche 1 » ; l'orchestrateur édite l'issue en le disant :

| Point | Sort |
|---|---|
| Entité inconnue, portée ni écrite ni testée | AC-L1 |
| Alignement des alias fondé sur le seul nombre de lignes | AC-R4, C-e332-2 — c'était plus qu'un LOW : des alias sans rapport |
| Branche « nombre de lignes différent » sans test | AC-R3 |
| CHANGELOG plus bref que le manuel | T5 ; la phrase de la 0.17.4 reste, la 0.17.5 la remplace |
| Capture #819 absente | T6 |

## Mutations (`kesh-gate-logs/e332/vmut.sh`, sous le verrou, restauration `git checkout --` puis `touch`)

| Id | Mutation | Tuée par |
|---|---|---|
| M1 | `Reading` → `Ok(vec![])` | `..._legacy_reading_fails_922` |
| M2 | `Selection` → `Ok(vec![])` | `..._legacy_selection_fails_922` |
| M3 | `LineCount` → `Ok(vec![])` | `..._line_counts_differ_922` |
| M4 | contrôle de désalignement retiré | `..._misaligned_922` |
| M5 | désalignement sur la date seule | une paire de même date et d'autre montant, dans le même test |
| M6 | site de l'aperçu : `.unwrap_or_default()` au lieu de `?` | le volet aperçu de R1 |
| M7 | site de la création : idem | le volet création de R1, et R6 |
| M8 | garde `new_txs.is_empty()` retirée | `camt_without_lines_is_not_refused_922` |
| M9 | comparaison des textes retirée (alias par clé seule) | `..._not_a_false_duplicate_915` |
| M10-M16 | une colonne retirée de `LineTexts::eq`, sept mutations | les tests purs par colonne |
| M17 | index à la première ligne seule | `camt_alias_finds_the_row_with_equal_texts_915` |
| M18 | incertaine écartée au lieu d'importée | F1 volet création |
| M19 | `uncertain` non rendu à l'aperçu | F1 volet aperçu |
| M20 | `details.uncertain_legacy_lines_imported` retiré | F1 volet trace |
| M21 | alias appliqué sans contrôle à la clé directe (S4 inversé) | test pur S4 |

Une survivante se traite ou se justifie au Change Log ; un fichier muté restauré se `touch` (cargo garde sinon le binaire muté).

## Gates

Commun : `e33-commun.md`, chaque commande lourde sous le double `flock` et `mem-guard`.

- Entre passes : `cargo fmt --all -- --check`, clippy du workspace, `cargo nextest run -E 'binary(bank_imports_e2e)'`.
- Au dernier commit de code : `scripts/test-fast.sh --ci`, `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`,
  `npm run build`, **E2E complet** (`bank-import*.spec.ts` compris), rouges jugés fichier par fichier contre `docs/testing.md`.
- La story ne touche ni `crates/kesh-db/migrations/`, ni `post_restore.rs`, ni un repository : gate ciblé permis en boucle.
- Spec E2E à ajouter à `frontend/tests/e2e/bank-import.spec.ts` (fixture XML écrite par le test) : dépôt d'un relevé dont la date est
  en CDATA → `bank-import-error` de code `BANK_IMPORT_LEGACY_REREAD_FAILED`, aucun import listé. Sélecteurs `data-testid`, jamais de
  libellé traduit.

## Dev Notes

- **Ce qui reste de l'alias.** C-e332-7 : il ne se borne pas dans le temps. `imported_at` ne dit pas quelle version a lu la ligne
  (aucune trace de la date de montée, `_kesh_version` sans historique), et la borne serait fausse dans un sens ou dans l'autre.
- **Le coût du refus.** Il est réel pour un fichier qu'accepte la 0.17.4 et dont la banque écrirait une date en CDATA : la 0.17.5 le
  refuse, y compris sur une installation neuve, et tant que l'alias existe. Aucune fixture ni aucune banque connue n'y mène (prémisse
  4). C'est la décision de Guy, prise à ce prix ; C-e332-1 dit pourquoi on n'a pas tenté de la restreindre.
- **Ne pas toucher** : l'empreinte de fichier (`compute_sha256_hex`, test `file_hash_is_sha256_of_received_bytes`), `DuplicateKey`,
  l'ordre des refus de la création.
- **Interdit** : ouvrir un relevé réel. Un contrôle sur les fichiers réels de Guy, s'il est voulu, revient à l'orchestrateur, en local,
  et se rapporte en « conforme / non conforme » seulement.

## Change Log

- 2026-11-06 — Fiche rédigée en autonomie (Opus 5.5).

## Dev Agent Record

*À remplir au développement.*
