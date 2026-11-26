# Story 58-1 : Format de numéro de facture — textes, tests et miroir (#1115, #1117, #1118, #1119, #1120, #1122, #1123, #1124)

Status: drafted

*Fiche rédigée le 2026-11-26 (Sonnet 5.5, en autonomie), workflow `bmad-create-story`. Worktree `/home/gcorbaz/devel/kesh-58-1`, branche `story/58-1-numero-facture-textes-tests`, partie de `epic/58-nettoyage` = `ee684a69f`. Première story du premier epic de nettoyage (fichier d'epic : `_bmad-output/planning-artifacts/epic-58-nettoyage.md`). Release visée : vendredi 2026-12-04. **Aucune migration, aucune règle métier changée, aucun changement d'API** : le Project Lead a dit le 2026-11-26 que changer des phrases de refus, sans changer ni le code d'erreur ni les champs, n'est pas un changement d'API. La story touche l'usage réel du Project Lead : il émet ses factures client sous le format par défaut `F-{YEAR}-{SEQ:04}`.*

## Story

En tant qu'administrateur de Kesh qui règle le format de numérotation des factures, je veux que les refus de numéro disent la vérité sur leur cause, que l'écran des réglages juge un format comme le serveur, et que les tests tiennent ce que les textes promettent, afin de ne jamais suivre une piste fausse ni découvrir à la validation d'une facture ce que l'écran avait laissé passer.

## Ce qui se lit au code (`ee684a69f`)

Les huit défauts existent encore ; aucune issue n'est déjà corrigée, aucune ne révèle une migration ni un changement d'API. Une seule demande de l'issue sort du lot (#1119, voir D1).

### #1115 (P4) — les phrases `too_long` et `duplicate` présument leur cas typique

- `crates/kesh-api/src/errors.rs:2364-2369` (cause `TooLong`) : « … donne un numéro trop long (plus de 64 octets ; une lettre accentuée en compte deux) : un administrateur doit raccourcir le format dans Administration → Facturation, **ou le nom de l’exercice** ». Le nom de l'exercice n'entre dans le numéro que si le format porte `{FY}` (`invoice_format.rs:361`, jeton `Token::Fy`) ; un format posé par SQL (`'x'×70 + {SEQ}`) est trop long sans lui.
- `errors.rs:2378-2383` (cause `Duplicate`) : « … il doit distinguer chaque pièce — un compteur (`{SEQ}`), qui repart à chaque exercice, **et l'exercice (`{FY}`)** ; l'année (`{YEAR}`) ne suffit pas si deux exercices commencent la même année ». Le doublon vient aussi de numéros attribués sous un **format précédent** (`into_number_duplicate`, `kesh-db/src/errors.rs:2528-2556` : la violation d'unicité dit seulement que le numéro rendu existe dans la société) : la phrase prescrit alors une piste inadaptée.
- Les mêmes phrases vivent dans les quatre catalogues : `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl:364` (`error-number-format-unrenderable-too-long`) et `:366` (`…-duplicate`) ; la garde G9 (`textes_coherents.rs`, `les_replis_rust_suivent_le_catalogue`) exige que le repli Rust égale le catalogue fr-CH.
- Le refus est juste, seule la piste est inadaptée : aucun effet sur les comptes (P4).

### #1117 (P4) — aucun test du format par défaut sur deux exercices commencés la même année

- Le seul test de la cause `duplicate` sur deux exercices est `ac4_format_without_fiscal_year_collides_on_second_fiscal_year` (`crates/kesh-api/tests/number_format_unrenderable_e2e.rs:691`), au format `FACT{SEQ}`. Aucun test ne monte `F-{YEAR}-{SEQ:04}` sur deux exercices dont l'année de début est la même (`grep -nF "SEQ:04" number_format_unrenderable_e2e.rs` : aucun test de ce scénario).
- Le mécanisme est lu : `year` est l'année de **début** de l'exercice (`kesh-db/src/repositories/invoices.rs:2477-2489`, `fy.start_date.format("%Y")`), le compteur repart à 1 par exercice (`next_number_for(&mut tx, company_id, fy.id)`, `:2489`), l'unicité vaut pour la société (`uq_invoices_number`). Un exercice court `2026-01-01 → 2026-06-30` suivi de `2026-07-01 → 2027-06-30` ne se chevauche pas (`find_overlapping`, `fiscal_years.rs:1046`) et donne deux fois `F-2026-0001`. C'est le cas où le Project Lead, au format par défaut, tomberait sur le refus : il mérite son test.

### #1118 (P4) — `MAX_RENDERED_LEN` et `RenderedTooLong` parlent de caractères pour une borne en octets

- `crates/kesh-core/src/invoice_format.rs:27` : « Longueur max du rendu final (identique à la VARCHAR(64) DB) » ; la colonne compte en caractères, la borne en **octets** (`out.len()`, `:386`).
- `:92-95` : `RenderedTooLong(n)` s'affiche « un numéro de {n} caractères (max {MAX_RENDERED_LEN}) ». `n` est un nombre d'octets sur les deux chemins qui l'émettent (`:253-254` pire cas, `worst_case_len` en `s.len()` ; `:386-387` `out.len()`).
- Ce `Display` n'est visible que par la route des réglages (`company_invoice_settings.rs:377-381`, `e.to_string()` dans un `400 VALIDATION_ERROR`) ; au rendu d'une pièce, la même erreur devient le refus typé (`from_render_error`, `kesh-db/src/errors.rs:1030`).
- **Site voisin, laissé** : `TemplateTooLong(n)` (`:65-68`) dit aussi « caractères » pour un `template.len()` en octets (`:214-215`), mais la variante est **partagée** avec `validate_description_template`, qui compte réellement en caractères (`:415-418`) : la corriger est une affaire de deux variantes, hors de cette story (D2).

### #1119 (P4) — `{SEQ:+4}` : refusé à l'écran, accepté par le serveur

- `invoice_format.rs:149-152` : le padding se lit par `n_str.parse::<u32>()`, que Rust admet précédé d'un `+` ; `{SEQ:+4}` est donc accepté par `validate_template` et rendu `0042`. Le miroir exige `^\d+$` (`invoice-number-format.ts:246-249`) et refuse. Le commentaire du miroir (« comme `str::parse::<u32>` », `:246`) est inexact.
- Refuser le `+` côté serveur changerait une règle de validation (un format `{SEQ:+4}` déjà enregistré deviendrait illisible : cause `invalid_format` à la prochaine validation). C'est **hors de cette story** (D1).

### #1120 (P4) — `\s` admet tout blanc Unicode, `length` compte en UTF-16

- `invoice-number-format.ts:14` : `ALLOWED_CHAR_RE = /^[A-Za-z0-9\-_/.#\s{}:]+$/`. `\s` admet U+00A0, U+2003, `\n`, `\r`, `\v`, `\f` ; le serveur n'admet que l'espace et la tabulation (`invoice_format.rs:108-114`, `is_allowed_char`). Un format qui porte une espace insécable passe à l'écran, puis est refusé par le serveur (`IllegalCharacter`).
- `:193-199` : `template.length` (unités UTF-16) borne et est affiché comme « actuel : n » ; le serveur compte en octets (`:215`, `template.len()`), **avant** le contrôle des caractères. Pour 33 « é » + `{SEQ}` : 38 unités, 71 octets ; l'écran dit « caractères non autorisés », le serveur « trop long ». Même refus, autre motif.
- Antérieur à la 35-1.

### #1122 (P4) — la borne de 64 octets présentée comme imposée par la QR-facture

- Prémisse relue : `crates/kesh-qrbill/src/generator.rs:73` `const SIX_PAYLOAD_MAX_BYTES: usize = 997;` est vraie ; mais le numéro n'entre dans la charge que par `unstructured_message`, **tronqué à 140 caractères** (`crates/kesh-api/src/routes/invoice_pdf_service.rs:479-482`, `msg.chars().take(140)`). Un numéro de 64 octets plus son préfixe ne menace pas la charge : la borne de 64 n'est pas ce qui la protège. C'est un choix de **prudence** (plus stricte que la colonne en caractères, jamais plus lâche).
- Sites qui présentent la borne comme imposée par la QR-facture (ensemble clos, `grep -rnF "C-e351-7"` hors `target` et `node_modules` : six fichiers) :
  1. `invoice_format.rs:345-347` (doc-comment de `render`) ;
  2. `crates/kesh-api/tests/number_format_unrenderable_e2e.rs` (doc-comment de `ac3_multibyte_fiscal_year_name_counts_bytes`, « la QR-facture borne sa charge en octets ») ;
  3. `_bmad-output/implementation-artifacts/e35-choix-autonomes-e351.md:77-91` (le registre, C-e351-7) ;
  4. `35-1-numerotation-sans-seq.md:475` (fiche de la 35-1) ;
  5. `35-1-numerotation-sans-seq-review-prompt-p2-ciblee.md` et `epic-58-nettoyage.md` : citations, **à laisser** (le prompt est un document daté ; l'epic cite l'issue).
- Les entrées passées d'un registre ou d'une fiche ne se réécrivent pas : sites 3 et 4 reçoivent un **rectificatif daté** (CLAUDE.md, § *Recompter*).

### #1123 (P4) — « une lettre accentuée en compte deux » sous-dit le poids des caractères

- Ensemble clos des sites (`grep -rnE "en compte deux|counts as two|zählt zwei|ne conta due" crates docs frontend/src`, hors fiches `_bmad-output` historiques et hors `mcp-sends-*` — autre sujet, la borne d'un envoi MCP, où « la plupart des symboles trois » est déjà dit) :
  1. `errors.rs:2368` (repli) ;
  2. `fr-CH/messages.ftl:364`, `de-CH:364` (« zählt zwei »), `it-CH:364` (« ne conta due »), `en-CH:364` (« counts as two ») ;
  3. `docs/api-external.md:527` et `:1417` (« un caractère accentué en compte deux ») ;
  4. `docs/manual/fr/user-manual.tex:1602` (« qui comptent chacune deux ») ; **les manuels DE/IT/EN n'existent pas** (`docs/manual/{de,en,it}/` ne portent qu'un `README.md`) ;
  5. `CHANGELOG.md:123` (entrée **publiée** de la 0.17.7) : voir D3.
- Le fait : « é » pèse deux octets, « – » (tiret demi-cadratin) et « € » trois. Aucun test ne couvre un caractère de trois octets.

### #1124 (P4) — le compteur n'est pas asserté pour deux causes

- `ac3_multibyte_fiscal_year_name_counts_bytes` (`number_format_unrenderable_e2e.rs:597-626`) et `ac3_unreadable_stored_format_is_a_typed_refusal` (`:581-593`) n'appellent pas `counter(...)`, alors que les textes disent « aucun numéro n'a été consommé » pour **toutes** les causes (les tests de `too_long`, `brace` et `duplicate` l'assertent : `:435-465`, `:630-657`). Dans les deux, aucune facture n'a été validée avant : la ligne du compteur n'existe pas, il faut en créer une avant d'asserter (une première validation au format par défaut, comme `ac3_invoice_number_too_long_is_a_typed_refusal`, `:437-439`).
- Le mécanisme qui tient le compteur est la transaction annulée (`invoices.rs:2489-2503` : le numéro est tiré, puis le rendu échoue, la transaction est abandonnée avec lui).

## Décisions

- **D1 — #1119 : texte seulement, le serveur reste souple.** Le serveur continue d'admettre `{SEQ:+4}` ; l'écran continue de le refuser, ce qui est sans danger (personne n'écrit un `+` ; refuser est le côté sûr). Le commentaire du miroir dit l'écart au lieu de le nier, et un test de chaque côté le fixe (le jour où le serveur se resserre, le test rougit et appelle la mise à jour du miroir). La demande « refuser le `+` côté Rust aussi » est une **règle neuve** : elle sort de la story. **Action de l'orchestrateur : ouvrir une issue P4 pour elle (une issue par problème) ou en décider le non-lieu** ; la fiche n'ouvre rien (`gh` en lecture seule).
- **D2 — #1118 : le `Display` de `RenderedTooLong` dit « octets » ; `TemplateTooLong` reste.** Voir le site voisin ci-dessus : la variante sert deux validations, dont une compte vraiment en caractères. À signaler comme issue P4 (« TemplateTooLong dit caractères pour des octets sur le format de numéro »), non traité ici. Le catalogue de l'écran `invoices-format-error-rendered-too-long` (« caractères », ×4) **reste** : le miroir ne juge qu'un gabarit déjà filtré par `ALLOWED_CHAR_RE` (ASCII), où octets et caractères coïncident ; le changer imposerait quatre catalogues pour une phrase exacte.
- **D3 — CHANGELOG : l'entrée publiée de la 0.17.7 n'est pas réécrite.** Elle était inexacte (« une lettre accentuée en comptant deux ») mais datée d'une release ; la correction va aux *Notes de version* de la fiche, d'où elle passera à l'entrée de la release suivante, **sans écrire le CHANGELOG dans la story** (règle des fichiers communs). Si le Project Lead veut aussi corriger l'entrée 0.17.7, ce sera une annotation datée, à son initiative. `docs/api-external.md` et le manuel, qui se tiennent à jour, sont corrigés.
- **D4 — formulation neutre, aucune clé ni branche de code ajoutée** (#1115). Le texte conditionne la piste au lieu de la présumer ; aucune nouvelle clé i18n (`sitesTotal` de `i18n-keys.test.ts` inchangé, `lint-i18n-ownership` sans effet), aucun choix de variante par le code. Propositions fr (le développeur écrit de, it, en dans le même registre que le reste du catalogue) :
  - `too-long` : « Le format « { $format } » donne, dans l’exercice « { $fiscalYear } », un numéro trop long (plus de 64 octets ; certains caractères en comptent deux ou trois) : un administrateur doit raccourcir le format dans Administration → Facturation, et le nom de l’exercice s’il reprend {FY}. » ;
  - `duplicate` : « Le format « { $format } » produit, dans l’exercice « { $fiscalYear } », un numéro déjà attribué dans la société — dans cet exercice, dans un autre, ou sous un format précédent : chaque pièce doit avoir un numéro distinct. Un compteur ({SEQ}) repart à chaque exercice ; pour distinguer les exercices il faut {FY}, ou {YEAR} si deux exercices ne commencent jamais la même année. Un administrateur doit le corriger dans Administration → Facturation. »
- **D5 — pas de capture à part pour les trois autres causes** : seuls `too_long` et `duplicate` changent de texte visible.

## Critères d'acceptation

- **AC1 (#1115)** — Le refus `too_long` ne prescrit plus le nom de l'exercice sans condition : il dit qu'il ne compte que si le format reprend `{FY}`. Le refus `duplicate` nomme, parmi ses causes possibles, « un format précédent », et ne prescrit plus `{SEQ}` et `{FY}` comme une obligation unique. Quatre catalogues + repli Rust identiques au fr-CH (G9).
- **AC2 (#1117)** — Sous `F-{YEAR}-{SEQ:04}` (le défaut, **sans** `set_format`), deux exercices ouverts qui commencent la même année civile (`2026-01-01 → 2026-06-30`, puis `2026-07-01 → 2027-06-30`) : la première facture du premier exercice reçoit `F-2026-0001` ; la première du second rend `400 NUMBER_FORMAT_UNRENDERABLE`, cause `duplicate`, reste brouillon, sans numéro, compteur du second exercice inchangé (assertion `counter`).
- **AC3 (#1118)** — `FormatError::RenderedTooLong(n)` s'affiche en **octets** ; le commentaire de `MAX_RENDERED_LEN` dit que la borne est en octets, plus stricte que la colonne `VARCHAR(64)` en caractères, jamais plus lâche.
- **AC4 (#1119)** — Le commentaire du miroir dit que le padding est strictement décimal **et que le serveur admet aussi un `+` initial** (écart connu, #1119) ; un test Rust fixe que `validate_template("F-{SEQ:+4}")` est accepté et rendu `F-0042` pour `seq = 42` ; un test vitest fixe que le miroir refuse `{SEQ:+4}` (code absent, message « Padding »).
- **AC5 (#1120)** — Le miroir n'admet comme blancs que l'espace et la tabulation : `F-{YEAR}-{SEQ}` suivi de U+00A0, de U+2003, de `\n` est refusé « caractères non autorisés » (comme `IllegalCharacter` du serveur, fixé par un test Rust) ; `F-{YEAR}\t-{SEQ}` reste accepté. La longueur du gabarit se compte en **octets UTF-8** (`new TextEncoder().encode(t).length`), pour le refus et pour le « actuel : n » : `'é'.repeat(33) + '{SEQ}'` (71 octets, 38 unités UTF-16) est refusé comme **trop long**, `actual` 71.
- **AC6 (#1122)** — Les sites 1 et 2 de la liste close ne disent plus que la QR-facture impose la borne : ils la disent **choix de prudence** (`unstructured_message` tronqué à 140 caractères, `invoice_pdf_service.rs:479-482`). Les sites 3 et 4 reçoivent un rectificatif daté, sans réécrire l'entrée d'origine. Contrôle : `grep -rnF "borne sa charge en octets" crates frontend/src` ne rend plus rien ; `grep -rnF "C-e351-7" crates` ne rend que des renvois au registre.
- **AC7 (#1123)** — Les huit textes (repli Rust, quatre catalogues, `api-external.md` ×2, manuel fr) disent « deux ou trois octets » ; un test d'API avec un nom d'exercice fait de « – » (22 caractères, 66 octets) sous `{FY}-{SEQ:04}` est refusé `too_long`, avec le message nouveau (`deux ou trois`), et prouve qu'un caractère de trois octets pèse trois.
- **AC8 (#1124)** — `ac3_multibyte_fiscal_year_name_counts_bytes`, le nouveau test de l'AC7 et `ac3_unreadable_stored_format_is_a_typed_refusal` assertent que le compteur d'exercice est resté à sa valeur d'avant (une première facture validée au format par défaut le fait exister, à 2).
- **AC9 (transverse)** — Aucune migration, aucun code d'erreur ni aucun champ d'API changé, aucune nouvelle clé i18n ; le nombre de clés des quatre catalogues et `sitesTotal` ne bougent pas ; aucune facture, aucun numéro, aucun PDF existant ne change.

## Tests à écrire, et la preuve du rouge

Les tests de texte et de miroir rougissent **d'abord** sur le code actuel. Les tests d'AC2, AC4 (Rust) et AC8 décrivent un comportement **déjà** en place : ils sont verts au départ, et leur rouge se prouve **par mutation**, jamais par réputation (note à la fiche et au Dev Agent Record).

| AC | Test | Rouge au départ | Mutation (restaurer puis **toucher le fichier**, les binaires périmés de cargo gardant la mutation) |
|---|---|---|---|
| AC1 | `number_format_unrenderable_e2e.rs` : `too_long` posé par SQL (`'x'×70 + {SEQ}`, aucun `{FY}`) : le message contient « s’il reprend » `{FY}` ; `duplicate` : contient « format précédent » | oui | M1 : remettre l'ancienne phrase dans `errors.rs` **et** le catalogue : les deux assertions rougissent ; M1b : changer le catalogue sans le repli : G9 rougit |
| AC2 | `ac4_default_format_collides_on_two_fiscal_years_starting_the_same_year` | non (vert) | M2 : `names_key` toujours faux dans `into_number_duplicate` : 409/500 au lieu de 400 ; M2b : `year` pris sur `end_date` : les numéros diffèrent, le refus disparaît |
| AC3 | test unitaire `invoice_format.rs` : `RenderedTooLong(65).to_string()` contient « octets » et pas « caractères » | oui | M3 : remettre « caractères » |
| AC4 | `invoice_format.rs` : `{SEQ:+4}` accepté et rendu `0042` ; vitest : `{SEQ:+4}` refusé, message « Padding » | Rust : non ; vitest : non (c'est le comportement actuel) | M4 : refuser le `+` côté Rust → le test Rust rougit ; M4b : `^\+?\d+$` au miroir → le test vitest rougit |
| AC5 | vitest `invoice-number-format.test.ts` : NBSP, U+2003, `\n` refusés, tabulation acceptée, `'é'.repeat(33)+'{SEQ}'` trop long avec `actual` 71 ; Rust : `F-{SEQ}` + U+00A0 → `IllegalCharacter` | oui (côté TS) | M5 : remettre `\s` ; M5b : remettre `template.length` |
| AC6 | grep de l'ensemble clos (étape de tâche, pas un test : un test qui lirait un commentaire serait vrai par construction) | sans objet | sans objet — déclaré |
| AC7 | `ac3_three_byte_fiscal_year_name_counts_bytes` (« – ×22 », 66 octets, `{FY}-{SEQ:04}`, posé par SQL) ; `deux ou trois` dans le message | oui | M7 : remettre « en compte deux » |
| AC8 | compteur asserté dans les trois tests | non (le compteur est tenu) | M8 : valider la transaction avant de rendre le refus de rendu (`invoices.rs:2493`, `commit` avant le `?`) : les trois tests rougissent, comme les tests des autres causes déjà asserteurs |

Sondes positives : avant chaque assertion d'absence (« ne contient pas »), une assertion qui prouve que le texte lu est bien celui de la cause (le code d'erreur et `Administration → Facturation` sont déjà asserted par `assert_unrenderable`).

## Tâches

1. **Étape 0** : relire `errors.rs:2364-2383`, `invoice_format.rs:27`, `:92-95`, `:149-152`, `:345-347`, `invoice-number-format.ts:14`, `:193-203`, `:246-249`, `invoice_pdf_service.rs:479-482` et le registre ; consigner les lignes lues.
2. **Tests rouges** (AC1, AC3, AC5 TS, AC7), puis les tests verts-à-prouver (AC2, AC4, AC8) ; un commit.
3. **Textes** (AC1, AC7) : `errors.rs` + quatre catalogues dans le même commit ; `invoice_format.rs` (AC3, AC6 site 1) ; test de l'AC6 site 2.
4. **Miroir** (AC4, AC5) : regex, longueur en octets, commentaire.
5. **Documentation** : `docs/api-external.md:527` et `:1417` (« deux ou trois octets » ; cause `duplicate` : ajouter « ou numéro attribué sous un format précédent ») ; `docs/manual/fr/user-manual.tex:1602` (« qui comptent chacune deux **ou trois** ») ; rectificatifs datés au registre (site 3) et à la fiche de la 35-1 (site 4). **Aucun PDF commité** ; `\keshVersion` ne bouge pas (à la release).
6. **Propagation** : grep de la valeur, pas de la formulation (`en compte deux`, `counts as two`, `zählt zwei`, `ne conta due`, `deux octets`, `une lettre accentuée`) sur code, catalogues, manuel, `docs/`, `frontend/`, fiches vivantes ; chaque site restant est justifié (les `mcp-sends-*` le sont : autre borne).
7. **Mutations** M1 à M8, chacune restaurée, fichier touché.
8. **Recette visuelle** puis gates.

## Gates

- **Pendant** (gate ciblé) : `cargo fmt --all -- --check` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo nextest run -E 'binary(number_format_unrenderable_e2e)'` ; `-p kesh-core` (tests de `invoice_format`) ; `binary(textes_coherents)` ; `-p kesh-i18n` (parité des locales) ; `npm --prefix frontend run test:unit -- invoice-number-format`. Chaque commande lourde sous `scripts/mem-guard.sh` et `flock <kesh-gate-logs>/.gate.lock`.
- **Gate complet** (avant déclarer `done` et au dernier commit de code) : base remise à zéro (recette de `CLAUDE.md` § *Base de gate*, depuis le dépôt principal ; un agent travaille sur sa base `kesh_<clé>`), `cargo build --workspace --all-targets`, fmt, clippy, `cargo test --workspace` (ou `scripts/test-fast.sh`), puis frontend : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`. **`scripts/gate-ci.sh`** une fois à la clôture (l'état de processus se voit là, nextest l'ignore).
- **E2E Playwright complet** au dernier commit de code (montage de `docs/testing.md`, `LANG=fr_CH.UTF-8`) ; « vert » = les échecs attendus seulement. Aucun parcours ne change : aucun nouveau spec ; le spec existant des factures sert de filet.
- **Modules** (grain du domaine) : `kesh-core`, `kesh-api` (production : `errors.rs` ; tests), `kesh-i18n` (catalogues), `frontend/src/lib/features/invoices` — **quatre**, sous la borne de cinq. `kesh-db` n'est **pas** touché (aucun repository, aucune migration, ni `post_restore.rs` : pas de gate complet imposé en cours de boucle).

## Recette visuelle

Texte visible modifié : captures de chaque état touché, sur une base de démonstration, thème clair, hors dépôt (`kesh-gate-logs/s581/captures/`), jointes au Dev Agent Record et au prompt de la revue de code :

1. refus `too_long` à la validation d'une facture (nom d'exercice long sous `{FY}`) ;
2. refus `duplicate` (deux exercices de la même année, format par défaut) ;
3. page *Administration → Facturation* : format avec espace insécable → « caractères non autorisés » (AC5) ; format de 33 « é » → motif « dépasse 64 ».

## Note de version

- **Numéros de facture et d'avoir : les refus disent la bonne piste et le bon poids des caractères.** Le refus « numéro trop long » ne conseille plus de raccourcir le nom de l'exercice quand le format ne le reprend pas, et dit que certains caractères comptent deux ou trois octets ; le refus « numéro déjà attribué » nomme aussi le cas d'un format précédent. L'écran des réglages juge les blancs et la longueur d'un format comme le serveur (espace insécable refusée dès l'écran). **Correction** : la note de la 0.17.7, qui disait « une lettre accentuée en compte deux », sous-disait le poids de certains caractères (« – », « € » : trois octets). Aucun changement d'API (même code `NUMBER_FORMAT_UNRENDERABLE`, mêmes champs). Pour la page « Passer à la 0.21.0 » : rien à faire.

## Dev Notes

- **Règle de l'epic** : aucune règle métier, aucune migration, aucun changement d'API. Si une issue révèle autre chose en cours de route, elle sort et suit son chemin (issue, priorité).
- **Journal des sorties** : #1119 (partie « refuser `+` côté Rust », D1) et `TemplateTooLong` (D2) sont à ouvrir en issues par l'orchestrateur ; aucune issue n'est fermée par cette fiche.
- **Commits** : `refs #1115` etc. sur chacun ; aucun mot-clé de fermeture (`closes`, `fixes`, `resolves` et variantes), pas même en prose. Contrôle avant la PR : `git log origin/epic/58-nettoyage..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`.
- **Fichiers communs** : `messages.ftl` (quatre locales) — un seul bloc touché (`error-number-format-unrenderable-*`, lignes 364 et 366), à l'écart des blocs des autres epics ; la 58-4 touche d'autres clés du même fichier (conflit résolu en gardant les deux côtés).
- **Une base propre n'est pas requise pour les tests unitaires** ; les tests `#[sqlx::test]` montent le schéma de test.
- **Pas de CHANGELOG ni de PDF dans la story.**

### Project Structure Notes

Fichiers touchés : `crates/kesh-core/src/invoice_format.rs` ; `crates/kesh-api/src/errors.rs`, `crates/kesh-api/tests/number_format_unrenderable_e2e.rs` ; `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` ; `frontend/src/lib/features/invoices/invoice-number-format.ts` et son test ; `docs/api-external.md` ; `docs/manual/fr/user-manual.tex` ; `_bmad-output/implementation-artifacts/e35-choix-autonomes-e351.md` et `35-1-numerotation-sans-seq.md` (rectificatifs datés) ; `sprint-status.yaml`.

## Dev Agent Record

### Agent Model Used

(à remplir)

### Debug Log References

(à remplir)

### Completion Notes List

(à remplir)

### File List

(à remplir)

## Change Log

- 2026-11-26 — Fiche rédigée (`bmad-create-story`, Sonnet 5.5) ; statut `drafted` ; validation P1 (Opus, contexte frais) à lancer : prompt `58-1-numero-facture-textes-tests-validation-prompt-p1.md`.
