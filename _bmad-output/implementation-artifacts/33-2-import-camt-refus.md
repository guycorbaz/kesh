# Story 33-2 : l'import CAMT refuse le relevé qu'il ne sait pas relire « à la 0.17.3 », et l'alias ne signale plus un faux doublon

Status: review

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`,
numéros de ligne de `4ef19901`. Epic 33, release **0.17.5**, **sans migration**. Registre :
`e33-choix-autonomes-e332.md`, choix `C-e332-k`. Issues **#922** (faux négatif) et **#915** (faux positif, **réduit**, non supprimé : AC-F5), en `refs` dans les
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
     (`emit_transactions`, `MissingRequiredField`), montant, sens (`CdtDbtInd` d'une `Ntry` sans `TxDtls`, `:726`), période du relevé, IBAN, devise du compte
     (`finalize_statement`) ; montant ou sens d'un solde dont le code se lit (`apply_balance`, `:671-676` ; un code illisible laisse le solde ignoré, `:668-670`) ; ou coupé en deux par une référence, dont le dernier morceau ne se lit pas (`parse_date`,
     `InvalidDate`, `Decimal::from_str`, `InvalidAmount`, `parse_sign`, `:638`). Le sens perdu d'un `TxDtls` retombe sur celui de la `Ntry` (`:748-756`), et finit au besoin en E4, le montant signé différant.
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

   **Une date coupée ne rend pas toujours `InvalidDate`.** `parse_date` (`camt053/mod.rs:863-873`) passe par chrono, dont le `%Y` sans signe
   accepte de un à quatre chiffres (`chrono-0.4.45/src/format/parse.rs:381`, `:416`, lu, non exécuté : AC-S5 le jouera). `<Dt>&#x32;026-10-10</Dt>`
   se lit donc, en lecture héritée, `026-10-10` : l'an 26, sans erreur. Sur une date de comptabilisation, c'est E4 ; sur une date de valeur ou une
   borne de période (`:472`, `:476`, `:532`), rien n'échoue, et la lecture rend ce que rendait la 0.17.3. L'inventaire reste clos **à condition que
   la lecture héritée reste sans borne d'année** : la 33-4 en pose une sur les dates reçues, et `parse_legacy_0173` passe par le même `parse_date`.
   Avec cette borne, l'inventaire gagnerait une cause, et un relevé à borne de période coupée serait refusé à tort. AC-S5 le fixe.
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
   transformation). **Ces colonnes ne changent jamais après l'import** : les 11 requêtes `UPDATE bank_transactions` de production (17 occurrences de la chaîne, dont 6 commentaires) ne
   touchent que `status`, `matched_entry_id`, `auto_match_rejected_at`, `updated_at`, `version`
   (`git grep -n -A5 "UPDATE bank_transactions" -- crates/kesh-api/src crates/kesh-db/src`, `SET` relus : aucun ne nomme une colonne de texte, de date ni de montant). **Et leur calcul n'a pas changé
   entre les versions** : `git log -- crates/kesh-import/src/camt053/mod.rs` ne porte que 076ac863 (création), dc51300e
   (dépendances), e04122dc (devise : d'où `currency` **hors** de la comparaison), 910e5d5a (0.17.0, tenue par la
   caractérisation), c2051805 (0.17.4, #819) ; `v04.rs` et `v08.rs` n'ont changé qu'à c2051805 (`TextMode::Joined` passé) ; quick-xml passe de 0.39.4 à 0.42.0 à la 0.17.0, ce que 910e5d5a conserve octet pour octet (`camt053_golden.rs:232`). Une ligne importée par la 0.17.3 ou une version antérieure a donc en base exactement les textes que
   recalcule `parse_legacy_0173` (une ligne de la 0.17.4 porte ceux de la lecture corrigée, et se retrouve par la clé directe), **à une réserve près : la troncature à l'écriture**. Le pool de production (`kesh-api/src/main.rs:91-94`) ne pose aucun `sql_mode` : `create_pool`, qui en pose un strict (`kesh-db/src/pool.rs:42`), n'a aucun appelant. Reste donc celui du serveur, auquel sqlx n'ajoute que `PIPES_AS_CONCAT,NO_ENGINE_SUBSTITUTION`
   (`sqlx-mysql-0.8.6/src/options/connect.rs:53-66`). `docker-compose.yml:33-37` n'en fixe aucun, et l'image `mariadb:10.11` est stricte par défaut (`STRICT_TRANS_TABLES`, relevé sur `kesh-mariadb-dev`, même image, sans réglage non plus) ; mais `docker-compose.prod.yml:7-9` laisse la base à l'utilisateur, et le manuel ne fait que recommander ce mode (`admin-manual.tex:1022`). **Le dépôt ne l'établit donc pas pour le NAS, et la conception n'en dépend pas** (AC-F4). C'est ce qui réduit #915 sans rouvrir #922 (C-e332-3). **Réduit, et non supprimé** : une ligne neuve dont les sept textes,
   lus à l'ancienne, égalent ceux d'une ligne existante reste signalée comme doublon (typiquement une ligne qui ne porte que sa référence). Rien
   ne la distingue d'une ligne vraiment déjà importée : AC-F5.
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
- **C-e332-21** : à l'aperçu, un fichier déjà importé dont la relecture échoue reçoit le refus « déjà importé », non le conseil d'un nouvel
  export : l'ordre de l'aperçu s'aligne sur celui de la création.
- **C-e332-22** : la relecture héritée ne porte pas la borne d'années de la 33-4.

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
  enregistré. Demandez à votre banque un nouvel export de ce relevé ; à défaut, vérifiez dans la liste des transactions du compte
  quelles lignes y figurent déjà, et saisissez les autres avec un relevé saisi à la main. » Fallback Rust : la version française,
  comme les voisines.
- **AC-R8** — Un relevé **sans ligne** dont la relecture échoue n'est pas refusé : aperçu `200`, création `201`, zéro transaction.
- **AC-R9** — **Un fichier déjà importé ne reçoit pas de faux conseil** (C-e332-21). Fichier dont la relecture échoue et dont l'empreinte est
  déjà en base (ce qu'a pu faire la 0.17.4) : l'aperçu rend `422 BANK_IMPORT_DUPLICATE_FILE`, `existingImportId` et `existingFilename` en `details`,
  message « Ce fichier a déjà été importé. », et **non** `BANK_IMPORT_LEGACY_REREAD_FAILED` ; la création sans confirmation, de même (`:1276`,
  inchangé) ; la création avec `confirmDuplicateFile=true` rend `BANK_IMPORT_LEGACY_REREAD_FAILED` (AC-R5). Relecture réussie : l'aperçu reste
  `200` avec `warnings.duplicateFile` (AC-S1, volet (b)).

### Pas de refus à tort

- **AC-N1** — Un relevé sans entité : aperçu `200`, création `201`, comme avant.
- **AC-N2** — Un relevé à entités et CDATA dans ses textes, relisible (`first_file_819`) : aperçu `200`, création `201`.
- **AC-N3** — Le test parcourt les fixtures du dépôt (`crates/kesh-import/tests/fixtures/camt053/*.xml`, 19 fichiers,
  `frontend/tests/e2e/fixtures/camt053_*.xml`, 4 fichiers) et les range en deux classes. **Refusés** : la lecture corrigée rend une erreur : rien à relire. **Acceptés** : pour l'IBAN de leur premier relevé, `legacy_0173_aliases` rend `Ok`. Une
  exception ne s'admet que nommée, cause dite, au Change Log, et si sa construction relève d'E1 à E4. **Pas de test muet** : le test échoue si
  `acceptés + refusés ≠ 23`, et si `acceptés` diffère d'une constante en dur, relevée au développement et écrite au Change Log. `v04_multi_stmt.xml`
  (deux IBAN distincts, donc sans E3 ni E4) et `v04_characterization.xml` (entités et CDATA, IBAN simple) sont nommés dans le test et passent, sans exception.

### Reconnaissance (la protection des livres)

- **AC-S1** — `camt_import_of_0173_stays_recognised_after_the_819_fix` (`bank_imports_e2e.rs:3894`) et
  `camt_import_of_0173_overlap_forced_imports_all_three_819` passent **sans aucune modification de leur corps** : relevé
  enregistré comme la 0.17.3, même fichier refusé par son empreinte, autre fichier reconnu ligne à ligne, deux lignes écartées,
  une importée. La sortie `git diff` de ces deux fonctions est vide, et le Dev Agent Record le dit.
- **AC-S2** — Même preuve avec une référence **en CDATA** (lue vide en 0.17.3) : la ligne enregistrée comme la 0.17.3 reste
  reconnue.
- **AC-S3** — Plusieurs lignes existantes de même clé d'alias, seule la seconde a les mêmes textes : la ligne est reconnue, et
  `existingTransactionId` est l'`id` de la **seconde** (le `or_insert` n'y fait plus obstacle). Si aucune n'a les mêmes textes, ou si plusieurs les ont, l'`id` rendu, à `uncertainLegacyLines` comme à `duplicateLines`, est le **plus petit** de la clé.
- **AC-S4** — Le chemin de clé directe ne change pas : une ligne dont la clé égale une ligne existante est un doublon quels que
  soient ses textes (« le libellé n'en fait pas partie », `user-manual.tex:2716`). Le CSV et la saisie manuelle ne passent aucun
  alias et rendent ce qu'ils rendaient (`detect_duplicate_lines`).
- **AC-S5** — **La relecture héritée reste celle de la 0.17.3, sans borne d'années** (C-e332-22). (a) `parse_legacy_0173` lit
  `<Dt>0999-01-15</Dt>` en l'an 999 et `<Dt>&#x32;026-10-10</Dt>` en l'an 26, en `Ok` ; ce test ne dit rien de la lecture corrigée, qui relève de la
  33-4. (b) Borne de période écrite `&#x32;026-10-01T00:00:00`, lignes ordinaires : aperçu `200`, création `201`, aucun refus, et la reconnaissance d'une
  ligne déjà importée reste entière. (c) Date de comptabilisation écrite `&#x32;026-10-10` : refus, `cause = "misaligned"`, et non `"reading"`.
  Les volets (a) et (b) sont verts sur `4ef19901` ; le (c) y est rouge, faute de refus. Tous les trois rougissent si une borne d'années entre dans le chemin de la lecture héritée, quel que soit
  l'ordre de fusion de la 33-2 et de la 33-4.

### Faux doublon (#915)

- **AC-F1** — L'exemple de #915 : ligne existante de référence « 1 » au libellé A ; ligne neuve de référence
  `REF &amp; Co &lt;1&gt;`, mêmes date, montant et compte, libellé B. Aperçu : absente de `duplicateLines`, présente dans
  `uncertainLegacyLines` (`{ newIndex, existingTransactionId }`). Création : importée avec le mode par défaut `skip`, et la trace
  `bank_import.created` porte `details.uncertain_legacy_lines_imported = 1`.
- **AC-F2** — Le cas du second commentaire : ligne neuve à référence en CDATA ; ligne existante sans référence, autre libellé,
  même jour, même montant. Même résultat.
- **AC-F3** — Écran : `uncertainLegacyLines` non vide affiche le panneau `warning-uncertain-legacy-lines` (phrase chiffrée, au singulier ou au pluriel, tableau
  index et ID existant), **informatif** : il ne désactive pas *Confirmer l'import*. Liste vide : aucun panneau. Quatre locales.
- **AC-F4** — Les **sept** colonnes comparées sont exactement `value_date`, `reference`, `end_to_end_id`, `transaction_id`,
  `counterparty_iban`, `counterparty_name`, `details`, à égalité stricte **modulo la largeur de colonne**, **sans** `currency` (e04122dc). Le texte de la lecture héritée se coupe une fois, à la largeur de sa colonne (255 caractères ; `counterparty_iban`, 34 ; `details`, 65 535 octets, à une frontière de caractère ; `20260504000001_bank_imports.sql:64-69`). **Quel que soit le `sql_mode` du serveur** (prémisse 6), un texte qu'un serveur sans mode strict a tronqué à l'écriture reste donc reconnu : 300 caractères lus et leurs 255 premiers stockés, doublon ; leurs 254 premiers stockés, incertaine. Seule exception, inchangée par cette story : une `reference` elle-même tronquée change la clé, et la ligne s'importe sans signal, comme en 0.17.4. En mode strict, un texte trop long n'a pas pu être stocké : le cas n'existe pas. Aucun test ne dépend du mode : ceux de cet AC sont purs.
- **AC-F5** — **La limite qui reste, caractérisée** : ligne neuve de référence `REF &amp; Co &lt;1&gt;` et ligne existante de référence « 1 », mêmes date,
  montant et compte, et **mêmes sept textes** après lecture héritée : la ligne est dans `duplicateLines`, écartée par défaut, importable par
  *Importer quand même*. Elle est indiscernable d'une ligne vraiment déjà importée. Elle se **voit** à l'aperçu : le faux doublon est réduit à ce cas,
  non supprimé (C-e332-3).

### Reliquat de #923

- **AC-L1** — Test : une entité inconnue dans le **texte** d'un élément que Kesh ne lit pas (`<Stmt><Foo>&inconnue;</Foo>…`) rend
  `CamtError::MalformedXml` en lecture corrigée (`Event::GeneralRef`, `camt053/mod.rs:336-352`). Second test, qui fixe la portée : dans un **attribut** non
  lu (`<Stmt Foo="&inconnue;">`), elle n'est pas refusée, seul `Ccy` étant lu (`:259`). Le manuel dit déjà « un relevé qui porte une référence
  d'entité inconnue » (`user-manual.tex:2419-2421`) : ajouter « où qu'elle soit dans le texte du fichier ».

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
| `camt_identical_texts_stay_a_duplicate_915` | F5 | vert : **témoin** de la limite, à garder vert à travers T3 |
| `camt_refused_file_already_imported_says_so_922` | R9 | aperçu `200` au lieu de `422` ; montage : `montage_819` avec `kesh_import::camt053::parse`, la lecture de la 0.17.4 |
| `camt_short_year_in_period_is_not_refused_922` | S5 (b) | vert : **témoin** ; rouge si la borne d'années atteint la relecture ; montage : une ligne enregistrée comme la 0.17.3, à reconnaître |
| `camt_short_year_on_booking_date_is_misaligned_922` | S5 (c) | 200/201 au lieu de 422 ; `cause = "reading"` si la borne atteint la relecture |

Dans le `mod` des alias de `routes/bank_imports.rs` (`:2570-` : `aliases_of`, `no_alias_when_*`, récrits) :
`every_repository_fixture_rereads_922` (N3), `line_count_branch_is_reachable_922`, `misaligned_branch_is_reachable_922`.

Dans `kesh-core/src/bank_imports.rs` : tests purs de la comparaison (F4 : **une ligne de test par colonne**, chaque colonne qui
diffère fait passer de doublon à incertain ; la largeur de colonne ; S3, plus petit `id` compris ; S4).

Dans `crates/kesh-import/tests/` : `unknown_entity_in_an_unread_element_is_refused`, `unknown_entity_in_an_unread_attribute_is_not_refused` (L1) ;
`legacy_reading_has_no_year_bound_922` (S5 (a), vert : **témoin**).

### T2 — Le refus (`routes/bank_imports.rs`, `errors.rs`, `kesh-i18n`)

- `legacy_0173_aliases(...) -> Result<Vec<Option<LegacyLine>>, LegacyRereadFailure>`, `enum LegacyRereadFailure { Reading,
  Selection, LineCount, Misaligned }`, avec `new_txs.is_empty()` → `Ok(vec![])` en tête. Doc-comment récrit : l'ancien dit « Jamais
  d'erreur » et « un tel fichier n'a pas pu être importé par la 0.17.3 », ce qui a été le défaut.
- `AppError::BankImportLegacyRereadFailed { cause: &'static str }`, mappé à côté de `BankImportNoMatchingStatement`
  (`errors.rs:2405`), `422`, `tracing::warn!` de la cause seule (aucun contenu du fichier).
- La création (`:1348`) propage par `?`. L'aperçu reste derrière sa garde (`:1096`), la fonction porte la sienne.
- **L'aperçu suit l'ordre de la création** (C-e332-21). La lecture de l'empreinte (`:1086`) garde `existing` (`id`, `filename`), et un échec de la
  relecture (`:1100`) rend `AppError::BankImportDuplicateFile` si `existing` est posé, `BankImportLegacyRereadFailed` sinon. Une relecture réussie ne
  change rien : `200`, `warnings.duplicate_file`. Le message passe par `setError`, sans code d'écran (`BankImportUpload.svelte:172-173`).
- Doc-comment de `parse_legacy_0173` (`camt053/mod.rs:87`) : ajouter « **aucune borne d'année** : une borne sur les dates reçues se pose à la lecture
  corrigée » (AC-S5). La 33-2 ne modifie pas `parse_date`.
- Clé `bank-import-errors-legacy-reread-failed`, quatre locales, à côté des voisines.

### T3 — La comparaison (`kesh-core`, `routes/bank_imports.rs`)

- `kesh-core` : `LineTexts` (les sept champs d'AC-F4, `PartialEq` **écrit à la main**, par destructuration exhaustive `let Self { .. } = self` et une clause par
  champ : un champ ajouté ne compile pas sans être comparé, et une clause se retire sans toucher à la forme du type, ce qu'exigent M10 à M16),
  `LegacyLine { key: DuplicateKey, texts: LineTexts }`, dont la construction coupe les textes à la largeur de leur colonne (AC-F4), `eq` restant strict ;
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
(`BankImportUpload.svelte:567`), mêmes classes ; **deux clés choisies par le composant**, `bank-import-warnings-uncertain-legacy-lines-summary-one` et `-other`, le motif de `supplier-invoices-count-one` / `-other` (un sélecteur Fluent ne jouerait pas : `kesh-i18n/src/loader.rs:158`, messages pré-résolus sans arguments),
`bank-import-labels-uncertain-col-index`, `-existing-id`, quatre locales. Français du résumé, nombre compris : « 1 ligne ressemble à une ligne déjà importée
par une version antérieure (même date, même montant, même référence lue à l'ancienne), mais ses textes diffèrent (libellé, contrepartie, identifiants ou date de valeur). Kesh l'importe : vérifiez-la dans la liste des transactions du compte. » ; « { $count } lignes ressemblent à une ligne déjà importée
par une version antérieure (même date, même montant, même référence lue à l'ancienne), mais leurs textes diffèrent (libellé, contrepartie, identifiants ou date de valeur). Kesh les importe : vérifiez-les dans la liste des transactions du compte. » Fixture `BankImportUpload.test.ts:105` étendue ;
tests Vitest : panneau à une ligne et à deux lignes, absent, *Confirmer* non désactivé.

### T5 — Manuels et CHANGELOG

- `docs/manual/fr/user-manual.tex`, § *Détection des doublons* (`:2718-2723`) : retirer les « deux limites » et dire ce qui est vrai
  à présent : le refus, ses causes en mots simples, quoi faire ; et le signalement d'une ligne qui ressemble sans être identique.
  **Limites restantes, dites toutes les deux** : (1) C-e332-4 : une ligne qui ressemble à une ligne existante sans porter les mêmes textes s'importe
  et se **signale** ; c'est ainsi qu'une ligne déjà importée en 0.17.3, dont la référence porte un caractère spécial **et** dont la banque aurait changé le
  libellé dans un nouvel export, s'importe une seconde fois, signalée à l'aperçu ; (2) AC-F5 : une ligne neuve dont tous les textes, lus à l'ancienne,
  égalent ceux d'une ligne existante du même jour et du même montant reste signalée comme doublon ; l'aperçu la montre, et *Importer quand même* la récupère. Et, pour un fichier
  déjà importé, le message reste « déjà importé » (AC-R9). § *Import dans Kesh* (`:2419-2421`) : le refus, et AC-L1. FAQ (`:3779`) : une entrée « Kesh refuse mon relevé… ».
- `docs/manual/fr/admin-manual.tex:2553`, *Passer à la 0.17.4* : section d'historique, **non réécrite** ; ajouter une phrase datée
  « Depuis la 0.17.5, la première limite est levée (Kesh refuse le relevé qu'il ne sait pas relire) ; la seconde est réduite à une ligne dont tous les textes, lus à l'ancienne, égalent ceux d'une ligne existante : … \S … ».
- `docs/api-external.md`, paragraphe **Ordre des refus** (`:771`, un tableau de ressources à `:223` n'étant pas un lieu de contrat) : le
  `422 BANK_IMPORT_LEGACY_REREAD_FAILED`, `details.cause` et ses quatre valeurs ; son rang : à la création, après, dans l'ordre (`routes/bank_imports.rs:1276-1348`), le `422 BANK_IMPORT_DUPLICATE_FILE` non confirmé, la lecture du fichier, la sélection, le `422 BANK_IMPORT_UNSUPPORTED_CURRENCY`
  et le refus de solde non confirmé, avant toute écriture, sans contournement par `confirm*` ; à l'aperçu, d'emblée, solde et devise n'y étant que des avertissements, ou le `422 BANK_IMPORT_DUPLICATE_FILE` s'il est déjà importé (AC-R9). Dire l'asymétrie : un client qui envoie un fichier à solde faux n'obtient ce refus, à la création, qu'après `confirmBalanceMismatch=true`.
  Une phrase sur `warnings.uncertainLegacyLines` à côté, avec son sens : la ligne s'importe, sans choix.
- `make fr` dans `docs/manual/` ; contrôle des PDF aplatis : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`, phrases cherchées citées
  au Change Log (le refus présent ; les deux limites présentes ; « sans aucun signal » absent du manuel utilisateur).
- `CHANGELOG.md`, `## [0.17.5] — Non publié`, *Corrigé* : deux puces, `refs` #922 et #915, qui disent le refus et le signalement
  avec leurs conditions, et, pour #915, « réduit » : la limite d'AC-F5 y est écrite (le reproche de #923 : CHANGELOG plus bref que le manuel). Le paragraphe de la
  0.17.4 ne se récrit pas.
- Greper le symptôme : `git grep -nE "sans aucun signal|n'est pas reconnue|Jamais d'erreur|ces deux limites|legacy_0173_aliases" -- docs crates CHANGELOG.md`.

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
| M6 | site de l'aperçu : l'échec ignoré, `existing` absent (`200` au lieu du refus) | le volet aperçu de R1 |
| M7 | site de la création : `.unwrap_or_default()` au lieu de `?` | le volet création de R1, et R6 |
| M8 | garde `new_txs.is_empty()` retirée | `camt_without_lines_is_not_refused_922` |
| M9 | comparaison des textes retirée (alias par clé seule) | `..._not_a_false_duplicate_915` |
| M10-M16 | une clause retirée de `LineTexts::eq`, écrit à la main (T3), sept mutations | les tests purs par colonne |
| M17 | index à la première ligne seule | `camt_alias_finds_the_row_with_equal_texts_915` |
| M18 | incertaine écartée au lieu d'importée | F1 volet création |
| M19 | `uncertain` non rendu à l'aperçu | F1 volet aperçu |
| M20 | `details.uncertain_legacy_lines_imported` retiré | F1 volet trace |
| M21 | alias appliqué sans contrôle à la clé directe (S4 inversé) | test pur S4 |
| M22 | borne 1000-9999 ajoutée dans `parse_date` (une 33-4 mal placée) | `legacy_reading_has_no_year_bound_922`, `camt_short_year_in_period_is_not_refused_922` |
| M23 | aperçu : le refus de relecture rendu même si `existing` est posé | `camt_refused_file_already_imported_says_so_922` |
| M24 | aperçu : `existing` posé, relecture en échec, `200` au lieu d'un refus | idem |
| M25 | textes de `LegacyLine` non coupés à la largeur de colonne | test pur de largeur (AC-F4) |

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
- **Ce qui reste de #915.** AC-F5. Rien, à la lecture du fichier et de la base, ne sépare cette ligne d'une ligne vraiment déjà importée : la « réduction » est le
  résultat honnête, et la PR, le CHANGELOG et le commentaire de fermeture disent « réduit ». Issue du reliquat demandée à l'orchestrateur (« on trace toujours »).
- **Ne pas toucher** : l'empreinte de fichier (`compute_sha256_hex`, test `file_hash_is_sha256_of_received_bytes`), `DuplicateKey`,
  l'ordre des refus de la création.
- **Interdit** : ouvrir un relevé réel. Un contrôle sur les fichiers réels de Guy, s'il est voulu, revient à l'orchestrateur, en local,
  et se rapporte en « conforme / non conforme » seulement.

## Recoupements

**33-4** (`kesh-import/src/camt053/mod.rs`) : elle borne les années des dates reçues, et `parse_date` est commun aux deux lectures. AC-S5 fixe que la relecture héritée n'en
hérite pas ; la 33-2 ne touche pas à `parse_date` (T2). Ordre de fusion indifférent : la seconde fusion rejoue `camt053_golden`, `bank_imports_e2e` et les tests d'AC-S5.

## Change Log

- 2026-11-06 — Fiche rédigée en autonomie (Opus 5.5).
- 2026-11-06 — Remédiation de la validation P1 (rapport `kesh-gate-logs/e33-validate-p1-e332.md`, Opus 5.5, aucune compilation ni aucun test). Sept constats portent sur cette
  fiche, tous remédiés, aucun réfuté : R-3, R-4, R-5, R-6, F-2, F-4 (LOW), et M-R4-2 (MEDIUM, rapport `e33-validate-p1-e334.md`). Prémisses relues : `routes/bank_imports.rs:1085-1090`,
  `:1100`, `:1276`, `:1980-2006` ; `camt053/mod.rs:85-96`, `:259`, `:336-352`, `:472`, `:476`, `:530-532`, `:863-873` ; `chrono-0.4.45/src/format/parse.rs:381`, `:416` ;
  `errors.rs:2419-2435` ; `BankImportUpload.svelte:172-173` ; `api-external.md:223`, `:771` ; 19 + 4 fixtures listées, `v04_multi_stmt.xml:17,77` (deux IBAN), `v04_characterization.xml:19`.
  Conception : l'aperçu suit l'ordre de la création (C-e332-21) ; la relecture héritée reste sans borne (C-e332-22) ; `LineTexts::eq` écrit à la main.
  Décomptes de cette fiche après remédiation, recomptés : 23 AC, 15 tests de route au tableau de T1, 24 mutations (M10-M16 comptant pour sept).
- 2026-11-06 — Remédiation de la validation P2 (rapport `kesh-gate-logs/e33-validate-p2-e332.md`, Opus 5.5, aucune compilation ni aucun test, aucun commit). Quatre constats de cette fiche, tous LOW, tous remédiés, aucun réfuté : R-12, F-1, F-2, F-3. Prémisses relues : `camt053/mod.rs:638`, `:700-711`, `:722-727`, `:748-756` (R-12) ; `BankImportUpload.svelte:567-578`, `kesh-i18n/src/loader.rs:158` (F-1) ; `routes/bank_imports.rs:1056-1112`, `:1276-1320` (F-2).
  **Question restée ouverte en P1 et P2, le `sql_mode`** (C-e332-3) : relevée sans accès au NAS, à `main.rs:91-94`, `pool.rs:39-49`, `sqlx-mysql-0.8.6/src/options/connect.rs:53-66`, `docker-compose.yml:33-37`, `docker-compose.prod.yml:7-9`, `admin-manual.tex:1022`, et, en lecture seule, `SELECT @@GLOBAL.sql_mode` sur `kesh-mariadb-dev` (MariaDB 10.11.19 : `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`). ⚠️ Le doc-comment de `create_pool` semble la trancher, mais cette fonction n'a aucun appelant : le dépôt ne l'établit pas pour une base fournie par l'utilisateur. AC-F4 ne dépend donc plus du mode : la conséquence, sur un serveur sans mode strict, est celle de C-e332-4. La troncature ne joue que sur une ligne passant par l'alias (une référence à entité ou CDATA, AC-S4) et portant un autre texte de plus de 255 caractères ; une référence elle-même tronquée change la clé, comme aujourd'hui (AC-F4).
  Décomptes de cette fiche, recomptés : 23 AC, 15 tests de route au tableau de T1, 24 mutations, inchangés (M6 et M7 récrites).
- 2026-11-06 — Remédiation de la validation P3 (rapport `kesh-gate-logs/e33-validate-p3-e332.md`, aucune compilation ni aucun test, aucun commit). Huit constats de cette fiche, tous remédiés, aucun réfuté : F-1 (MEDIUM), R-2 à R-7, F-4 (LOW). Prémisses relues : `git grep -i "UPDATE bank_transactions"` (17 lignes ; les 11 requêtes et 6 commentaires sont le relevé du rapport, non recompté) ; `git log` de `v04.rs`, `v08.rs` ; `Cargo.lock` de `v0.16.0` et `v0.17.0` ; `camt053_tests.rs:239-244`, `fixtures/README.md:37` ; `camt053/mod.rs:663-676` ; `20260504000001_bank_imports.sql:61-69` ; `kesh-core/bank_imports.rs:446-498`.
  **Conception changée (R-6, arbitrage)** : la largeur de colonne entre à AC-F4, coupée une fois à la construction de `LegacyLine` : `eq` reste symétrique, M10 à M16 tiennent. Une ligne dont un texte a été tronqué par un serveur sans mode strict n'est plus réimportée. La coupe de MariaDB sur `details` (octets, frontière de caractère) est lue, non jouée sur un serveur non strict. C-e332-3 révisé. AC-N3 ne nomme plus d'exemple : `v04_invalid_iban` n'y est pas refusé. L'admin-manuel dit désormais ce que dit le manuel utilisateur (F-1).
  Décomptes de cette fiche, recomptés : 23 AC, 15 tests de route au tableau de T1, 25 mutations (M25 ajoutée).
- 2026-11-06 — Développement (Opus 5.5, worktree `/home/gcorbaz/devel/kesh-e332`, branche `story/33-2-import-camt`, base `bdc3faa4`). **Étape 0** : `git diff --stat 4ef19901 bdc3faa4 -- crates frontend docs` vide, les numéros de ligne de la fiche valent donc ; relues : `routes/bank_imports.rs:1085-1112`, `:1276`, `:1338-1360`, `:1980-2006`, `kesh-core/bank_imports.rs:446-498`, `camt053/mod.rs:85-96`, `:863-873`, `errors.rs:2405-2435`, `BankImportUpload.svelte:172-173`, `:567-632`. LOW restants : ceux de la P3 (`e33-validate-p3-e332.md`, R-2 à R-7, F-4) sont déjà portés par la fiche, relus un à un (prémisse 6, E1, AC-N3, AC-F4, AC-S3, AC-R7) ; la P4 (`e33-validate-p4.md`) ne rend rien sur la 33-2 (« aucun écart entre la prose et le code lu »). **Constante d'AC-N3** : 23 fixtures, **19 acceptées**, 4 refusées par la lecture corrigée, aucune exception. **Écart à la fiche, T5** : `admin-manual.tex:2553` énonce ses deux limites dans l'ordre inverse de celui que supposait la fiche (la ligne neuve d'abord, l'échec de relecture ensuite) ; la phrase datée dit donc « la seconde est levée ; la première est réduite ». **Écart à la fiche, T1** : `camt_identical_texts_stay_a_duplicate_915` n'était pas vert sur la base, faute du champ `uncertainLegacyLines` qu'il lit ; il l'est depuis T3. Choix nouveaux : C-e332-25, C-e332-26.

## Dev Agent Record

**Agent** : Opus 5.5. **Commits** : `1fa33649` (rouges), `c278714e` (code, manuels `.tex`, API, CHANGELOG), `5596684e` (PDF, sites i18n, registre), `261b5e81` (clippy, un test), puis ce compte rendu. Dernier commit de code : `261b5e81`.

**Rouges** (`kesh-gate-logs/e332/rouges.log`, sur le code d'origine, périmètre `binary(bank_imports_e2e)` + `binary(camt053_tests)`) : 110 tests, 12 rouges : les sept refus (`reading`, `selection`, `line_counts`, `misaligned`, `no_bypass`, `writes_nothing`, `already_imported`), `short_year_on_booking_date`, `finds_the_row`, `truncated_reference`, `cdata_reference_is_not`, `identical_texts` (ce dernier faute du champ, voir Change Log). Verts comme la fiche l'annonce : `without_lines`, `cdata_reference_stays`, `short_year_in_period`, `legacy_reading_has_no_year_bound`, les deux `unknown_entity`. Les tests purs de `kesh-core` et de `routes/` ne compilent pas sur la base (types neufs) : leur rouge est celui de la compilation, et leur preuve, les mutations.

**AC-S1** : corps de `camt_import_of_0173_stays_recognised_after_the_819_fix` et `camt_import_of_0173_overlap_forced_imports_all_three_819` **inchangés** : empreintes SHA-256 des deux fonctions identiques entre `bdc3faa4` et `HEAD` (`7cad908e3b4e`, `cba598e13434`) ; verts.

**Mutations** (`kesh-gate-logs/e332/vmut.py`, `mut/summary.log`, `mut/M*.log`) : **25 sur 25 tuées**, aucune équivalente. Dans `summary.log`, les premières lignes de M9 (`SURVIVANTE rc=2`) et M10 (`ERREUR-COMPILATION`) sont des erreurs du script, aucun test n'ayant tourné (`--lib` passé deux fois ; un bloc en tête d'expression) : rejouées, les lignes suivantes les disent tuées. M22 tuée par `legacy_reading_has_no_year_bound_922`, `camt_short_year_in_period_is_not_refused_922` et `camt_short_year_on_booking_date_is_misaligned_922`.

**Gates au dernier commit de code** (`261b5e81`, bases recréées) : `scripts/test-fast.sh --ci` vert, fmt et clippy compris : **4840 exécutés, 4840 passés, 12 ignorés, 1 flaky** (`mcp_transport_e2e::abandon_au_delai_le_permis_reste_pris`, passé au second essai, sans rapport avec cette story) ; `cargo build --workspace --all-targets` vert. **Frontend** joué à `5596684e`, le dernier commit n'en touchant rien : `npm run check` 0 erreur (29 avertissements), `lint-i18n-ownership` vert, `npm run test:unit` **2699 tests, 218 fichiers**, `npm run build` vert. **E2E complet** (`e2e-r1/`, base recréée vide, binaire de `261b5e81`) : **353 passés, 7 échoués, 18 ignorés**, 16,0 min ; les sept sont exactement ceux de la KF-029 (#97) à `docs/testing.md` § *Les échecs attendus* : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`. La spec neuve `refuses a CAMT.053 statement…` est verte. Aucun backend ni port 3182 restant (`ss` avant et après).

**Décomptes**, périmètre `bdc3faa4..261b5e81` : `#[sqlx::test]` de `bank_imports_e2e.rs` 68 → 81 (+15 de T1, −2 retirés) ; `#[test]` de `kesh-core/bank_imports.rs` 33 → 40 ; `camt053_tests.rs` +3 ; Vitest de l'écran +3 ; sites i18n 2832 → 2836 ; cinq clés aux quatre locales (20 lignes).

**Manuels** : `make fr`, PDF aplatis contrôlés (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`). `user-manual.pdf` : « Le relevé que Kesh ne sait pas relire » 1, « s’importe une seconde fois, signalée » 1, « reste signalée comme doublon » 1, « où qu’elle soit dans le texte du fichier » 1, « Kesh refuse mon relevé » 2 (table des matières et titre), « sans aucun signal » 0, « n’est pas reconnue » 0. `admin-manual.pdf` : « Depuis la 0.17.5, la seconde de ces limites est levée » 1, « la première est réduite » 1. `marketing-brochure.pdf`, seulement régénéré, restauré.

**Captures** (`kesh-gate-logs/e332/captures/`, thème clair, base de démonstration, XML synthétique) : `e332-01-refus.png`, `e332-02-lignes-incertaines.png`, `e332-03-entites-lues-en-entier.png`. Vues : le refus et le panneau se lisent, *Confirmer l'import* reste actif. Vu aussi, hors de cette story : les colonnes *Montant* et *Détails* de l'aperçu se touchent (« 100.00 CHFDupont & Fils »).

**Écarts de méthode** : deux écritures de `.rs` ont été faites hors de l'outil Edit, contre la consigne 14 (le raccord initial des tests de T1 dans `bank_imports_e2e.rs`, par script ; l'ajout en fin de `camt053_tests.rs`, par redirection) ; `cargo fmt` et le gate sont passés dessus. Les `.ftl`, `.tex`, `.md` l'ont été aussi, motif vérifié unique. **Limites qui restent** : AC-F5, et C-e332-4 (une ligne de la 0.17.3 au libellé changé par la banque s'importe une seconde fois, signalée). La coupe de MariaDB sur `details` sans mode strict est lue, non jouée.
