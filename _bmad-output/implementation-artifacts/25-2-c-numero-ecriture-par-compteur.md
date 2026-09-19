# Story 25.2-c : Le numéro d'écriture vient d'un compteur, non d'un `MAX + 1`

Status: done

**Issue : [#381].**

## Story

En tant que **comptable dont les livres peuvent être contrôlés**,
je veux qu'**un numéro d'écriture une fois attribué ne soit jamais réattribué à une autre écriture**,
afin qu'une **séquence comptable soit univoque dans le temps**, et pas seulement unique à un instant
donné.

## Le défaut, établi

`crates/kesh-db/src/repositories/journal_entries.rs:343-351` :

```sql
SELECT COALESCE(MAX(entry_number), 0) + 1 FROM journal_entries
 WHERE company_id = ? AND fiscal_year_id = ? FOR UPDATE
```

L'`UNIQUE (company_id, fiscal_year_id, entry_number)` de
`20260412000001_journal_entries.sql:9` garantit l'unicité **à un instant donné** — pas la
contiguïté, pas l'univocité dans le temps. #381 en tire les deux symptômes :

| Symptôme | Ce qu'il donne | Visibilité |
|---|---|---|
| supprimer l'écriture n° 42 au milieu | un **trou** définitif | **visible** — un contrôleur le voit et demande l'explication |
| supprimer la **dernière** | son numéro est **réattribué** à une écriture au contenu différent | ⛔ **muet** — rien ne le signale, jamais |

**C'est la réutilisation que cette story ferme.** Le trou, non : il subsiste, et il reste
*explicable* — il correspond à une facture dévalidée, que le journal d'audit nomme.

## Le remède existe déjà dans Kesh, et il n'est pas à inventer

`invoice_number_sequences` (`20260417000001_invoice_validation.sql:21-33`) est un **compteur
persistant** : colonne `next_number`, verrou `SELECT … FOR UPDATE`, création à la demande
(`INSERT IGNORE`), incrément vérifié à `rows_affected() == 1`
(`crates/kesh-db/src/repositories/invoice_number_sequences.rs:30-97`). Sa portée est
`UNIQUE (company_id, fiscal_year_id)` — **exactement celle d'`entry_number`**.

Un compteur ne redescend jamais : c'est précisément la propriété qui manque au `MAX + 1`.

⚠️ **Le `MAX + 1` des écritures est donc l'exception du dépôt, pas la norme.** Cette story
l'aligne ; elle n'invente pas de mécanisme.

## Acceptance Criteria

1. **Le numéro vient d'un compteur.** Une table `journal_entry_number_sequences`
   — `(company_id, fiscal_year_id)` UNIQUE, `next_number BIGINT NOT NULL DEFAULT 1`, `version`,
   `CHECK (next_number >= 1)`, FK `RESTRICT` vers `companies` et `fiscal_years` — calquée sur
   `invoice_number_sequences`, jusqu'aux noms de contraintes. `create_in_tx` la consulte au lieu du
   `MAX + 1`.

2. **Jamais de réattribution.** Test décisif : créer trois écritures (1, 2, 3), supprimer la
   **dernière**, en créer une quatrième → elle porte le **4**, pas le 3. C'est ce test qui dit si la
   story a atteint son but ; sans lui elle ne prouve rien.

3. **Le trou reste possible, et c'est assumé.** Supprimer l'écriture du milieu laisse un trou. La
   story ne le comble pas et ne renumérote rien : renuméroter réécrirait des écritures existantes,
   ce que le gel de l'Epic 24 interdit précisément.

4. **La migration amorce le compteur depuis l'existant.** Pour chaque `(company_id, fiscal_year_id)`
   ayant au moins une écriture : `next_number = MAX(entry_number) + 1`. Sans cet amorçage, une
   installation en service réattribuerait des numéros dès la première écriture suivante — l'inverse
   du but.

5. **⛔ L'amorçage est un backfill de données : P7 s'applique.** La migration écrit des lignes, donc
   elle doit être **triée** — inscrite au registre `POST_RESTORE_BACKFILLS` ou portée à
   `EXEMPT_MIGRATIONS` **avec justification écrite** (`crates/kesh-db/src/post_restore.rs`). Le test
   `every_data_backfill_migration_is_triaged` échoue sinon en nommant le fichier. ⚠️ Une exemption
   invoquant l'argument de fenêtre **doit** commencer par la chaîne `Hors fenêtre`, sans quoi elle
   échappe au contrôle symétrique.

6. **P5 — l'audit d'idempotence.** Une ligne dans `docs/migrations-idempotence-audit.md` avec son
   verdict, **et les compteurs recomptés depuis le tableau** : les deux sites du total
   (en-tête de section et ligne `Total`) et les trois compteurs de partition, dont la somme doit
   égaler le total. Recompter, jamais incrémenter de confiance.

7. **P6 — le couplage positionnel.** `grep -rn "migrations.len()\|apply_migrations_up_to" crates/`
   et **inspecter chaque site** : une migration de plus décale toute fenêtre indexée par position.
   Chaque site doit résoudre par version, ou porter son garde-fou fail-loud.

8. **P2 — pas de bump.** `CREATE TABLE` + `INSERT` est **non-breaking** : un binaire antérieur ignore
   la table. Aucun `kesh_version_min_required`, donc aucun bump Cargo.

9. **La concurrence est tenue.** Deux créations simultanées dans le même exercice n'obtiennent jamais
   le même numéro — le `FOR UPDATE` sur la ligne de séquence remplace le gap lock du `MAX + 1`. Test
   de concurrence, sur le modèle de ceux d'`invoice_number_sequences`.

10. **L'`UNIQUE` reste.** `uq` sur `(company_id, fiscal_year_id, entry_number)` n'est pas retiré : il
    devient le filet du compteur, exactement comme pour les factures.

11. **La documentation dit la vérité.** ⛔ **Le fichier de migration de 2026-04-12 qui promet
    « jamais de trou » NE DOIT PAS être modifié** — P8, `sqlx` en a le checksum, le binaire refuserait
    de démarrer. La rectification va ailleurs : c'était l'objet de **#368, déjà fermée**. Vérifier que
    ce qu'elle a écrit reste juste après cette story, et l'amender au bon endroit si besoin.

12. **#381 se ferme** — `closes #381` sur la PR, `refs` sur les commits.

## Tasks / Subtasks

- [x] **T1 — La migration** (AC 1, 4, 5, 6, 7, 8) — `20260917000001`
  - [x] `CREATE TABLE journal_entry_number_sequences`, calquée jusqu'aux noms de contraintes.
  - [x] Amorçage `INSERT … SELECT … GROUP BY`, sans lequel la migration ferait **l'inverse** de son
        but sur une installation en service : le compteur partirait de 1 et heurterait l'`UNIQUE`.
  - [x] ⛔ **La table entre dans `TABLES_TO_TRUNCATE`** — non prévu par la spec, et **décisif** : un
        compteur non sauvegardé revient vide d'une restauration, repart de 1 et réattribue. La story
        aurait échoué sur son propre objet, en silence.
  - [x] **P5** : ligne d'audit + les **cinq** valeurs recomptées depuis la source (69 fichiers,
        69 lignes, en-tête 69, Total 69, partitions 8+61+0 = 69).
  - [x] **P6** : sans objet — `migrations_before` résout **par version**, pas par position ; les
        treize montages de `invoice_lines_revenue_account_backfill` sont insensibles.
  - [x] **P7** : triage complet. Créer une table applicative **referme la fenêtre
        d'importabilité** : les deux entrées du registre passent à `RETIRED_BACKFILLS` + exemption
        `Hors fenêtre`, le registre actif redevient vide. ⚠️ **Mon exemption a d'abord invoqué
        « Hors fenêtre » par analogie avec `vat_rates` ; le contrôle symétrique l'a REFUSÉE** — cette
        migration *est* la fenêtre, et le marqueur est réservé à un fait de chronologie. Fondement
        réécrit en termes de **couverture**. *Le garde-fou a corrigé un raisonnement.*
  - [x] Squash du schéma de test régénéré **par son script** (il ne s'édite jamais) : 69 migrations,
        40 tables, rejeu vérifié.
  - [x] ⛔ **Gate complet, pas ciblé** — joué le 2026-09-18 puis le 2026-09-19, cf. Change Log.

- [x] **T2 — L'allocateur** (AC 1, 9)
  - [x] `next_number_for`, calqué jusqu'au contrôle de `rows_affected() == 1`.
  - [x] `create_in_tx` l'appelle ; le `COALESCE(MAX(...))` a disparu. ⚠️ La sérialisation **change
        de nature, pas d'existence** : gap lock d'index → verrou de ligne du compteur.

- [x] **T3 — Les tests qui prouvent** (AC 2, 3, 9)
  - [x] **Le test décisif**, `un_numero_libere_n_est_jamais_reattribue`, **prouvé par mutation** :
        `MAX + 1` rebranché → **un seul rouge sur 31**, le sien, sur son assertion. Les trois autres
        tests neufs restent verts, et c'est juste — seule la *source* du numéro change.
  - [x] Trou du milieu assumé ; concurrence ; invariant du compteur.
  - [x] ⚠️ Assertions **relationnelles** et non absolues : `setup` efface les écritures mais **pas**
        le compteur, donc exiger une valeur exacte ferait rougir pour une raison étrangère.
  - [x] Deux tests existants réécrits pour la même cause — `test_create_sequential_numbering` disait
        la bonne propriété (la contiguïté) avec la mauvaise ancre (`1..=3`).
  - [x] **AC 4 — l'amorçage sur une base portant déjà des écritures**,
        `tests/journal_entry_number_sequences_bootstrap.rs` : migrations jusqu'à juste avant
        `20260917000001` (index **par version**, P6), écritures d'avant insérées en SQL brut —
        **numéros troués** (`1, 2, 5`), sociétés et exercices **désalignés**, un exercice **sans**
        écriture —, puis `MIGRATOR.run()`. Assertion de montage : la table n'existe pas avant.
        **Prouvé par trois mutations** de l'amorçage, toutes tuées : retiré, `COUNT(*) + 1`,
        `MAX` sans `+ 1`. Inscrit à `ALLOWED_REAL_MIGRATOR_FILES`.
        ⚠️ **Le danger décrit ici était mal nommé.** Avec le plancher de l'allocateur, un amorçage
        manqué ne heurte plus l'`UNIQUE` : il **réattribue en silence** les numéros des dernières
        écritures supprimées. C'est pire — muet — et c'est ce que l'assertion (2) du test exerce.
        L'en-tête de la migration dit encore « la contrainte d'unicité refuserait l'insertion » :
        vrai au moment où il a été écrit, **figé par P8**, donc corrigé ici et non là-bas.

- [x] **T4 — Documentation** (AC 11)
  - [x] ⚠️ Le manuel n'était pas **faux**, il était devenu **incomplet** — nuance qui change le
        correctif. Sa sous-section « Numérotation » (écrite pour #368) disait juste sur les **trous** ;
        elle ne disait **rien** de la réattribution, le symptôme muet. Ajout, pas correction.
  - [x] `CHANGELOG.md` : section « Non publié » créée, unicité vérifiée. ⚠️ **Conflit prévisible**
        avec la branche 25-2-a, qui la crée au même endroit.
  - [x] PDF régénéré (63 pages avant comme après), contrôlé **à plat** — apostrophes typographiques
        et tirets cadratins, précisément où un grep naïf rend un faux négatif.
  - [x] Écartés après vérification : brochure et § 811 parlent des **factures** (autre compteur) ;
        `docs/api-external.md` ne mentionne pas la numérotation.

- [ ] **T5 — Gates complets et PR** (AC 12)

## Dev Notes

### Ordre par rapport à la 25-2-b

**Cette story devrait précéder la 25-2-b**, ou voyager avec elle. La 25-2-b rend la suppression d'une
écriture *délibérée et fréquente* ; l'y laisser arriver avant le compteur, c'est multiplier les
occasions de réattribution muette pendant l'intervalle.

### Sites exacts

| Quoi | Où |
|---|---|
| le `MAX + 1` à remplacer | `crates/kesh-db/src/repositories/journal_entries.rs:343-351` |
| l'`UNIQUE` et le commentaire intouchable | `crates/kesh-db/migrations/20260412000001_journal_entries.sql:9,20` |
| le motif à transposer | `crates/kesh-db/src/repositories/invoice_number_sequences.rs:30-97` |
| sa DDL | `crates/kesh-db/migrations/20260417000001_invoice_validation.sql:21-33` |

### References

- [Source: https://github.com/guycorbaz/kesh/issues/381] — audit du 2026-08-26, comptable § D3, fiscaliste § II.1.
- [Source: https://github.com/guycorbaz/kesh/issues/368] — la rectification du commentaire de migration, **fermée**.
- [Source: CLAUDE.md § Migration breaking policy — P2, P5, P6, P7, P8]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

| Fichier | État |
|---|---|
| `crates/kesh-db/migrations/20260917000001_journal_entry_number_sequences.sql` | **créé** — ⛔ appliqué, donc **figé par P8** |
| `crates/kesh-db/src/repositories/journal_entry_number_sequences.rs` | **créé** — l'allocateur, et son rattrapage |
| `crates/kesh-db/src/repositories/mod.rs` | modifié — déclaration du module |
| `crates/kesh-db/src/repositories/journal_entries.rs` | modifié — le `MAX + 1` remplacé ; **6 tests** (4 neufs, 2 réécrits) ; nettoyage du compteur dans deux helpers |
| `crates/kesh-db/src/repositories/invoices.rs` | modifié — nettoyage du compteur (son `.ok()` **avalait** l'échec) |
| `crates/kesh-seed/src/lib.rs` | modifié — **code livré** : suppression du compteur avant les exercices |
| `crates/kesh-db/src/backup.rs` | modifié — la table entre à `TABLES_TO_TRUNCATE` ; **sans quoi la story échouait sur son propre objet** |
| `crates/kesh-db/src/post_restore.rs` | modifié — triage P7 : registre vidé, 2 entrées retirées, 3 exemptions |
| `crates/kesh-db/migrations.sha384` | modifié — registre de checksums P8 |
| `crates/kesh-db/test-schema/0001_schema_squash.sql` | **régénéré par son script** (il ne s'édite jamais) |
| `crates/kesh-db/tests/journal_entry_number_sequences_bootstrap.rs` | **créé** — AC 4, l'amorçage sur base peuplée |
| `crates/kesh-db/tests/test_schema_guard.rs` | modifié — le test ci-dessus inscrit à `ALLOWED_REAL_MIGRATOR_FILES` |
| `crates/kesh-db/tests/migrations_upgrade_path.rs` | modifié — compteur **et** soustracteur |
| `crates/kesh-api/tests/admin_full_import_e2e.rs` | modifié — 4 tests, substitution de source ; `report_entry` supprimé |
| `crates/kesh-api/tests/admin_full_export_e2e.rs` | modifié — décompte 38 → 39 |
| `crates/kesh-api/tests/admin_backup_e2e.rs` | modifié — commentaire qui annonçait « les 23 » pour 38 réelles |
| `docs/migrations-idempotence-audit.md` | modifié — ligne + **cinq** valeurs recomptées |
| `docs/manual/fr/user-manual.tex` + `.pdf` | modifiés — la garantie neuve ; PDF contrôlé à plat |
| `CHANGELOG.md` | modifié — section « Non publié » ⚠️ **conflit prévisible avec la 25-2-a** |

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-19 | merge de `main` | `main` intégré après les merges de **#439** (25-1c-a) et **#441** (25-2-a). Conflits résolus : `CHANGELOG.md` (l'entrée #381 rejoint « Corrigé » sous la section commune), ligne E25 du `README.md` (texte de `main` gardé, phrase #381 réinsérée), fiche 25-2-c (version complète de la branche), PDF du manuel **régénéré** et contrôlé à plat (les trois textes y sont). ⛔ **Un défaut d'interaction, invisible à chaque PR prise seule** : le helper `drop_company_deep`, ajouté par la 25-2-a dans `accounts.rs`, supprimait les exercices sans les compteurs — `fk_jens_fiscal_year` l'a refusé (`le_conflit_de_version_precede_la_garde_de_retypage` rouge). Même piège que les quatre sites réparés en dev ; **inventaire refait** de tous les `DELETE FROM fiscal_years` du dépôt combiné, c'était le seul. **Gates sur la combinaison** : backend **2396/2396** (2390 + les 6 tests de la story), frontend **745/745** + build, E2E **216 / 7** (KF-029, 15:58 UTC). |
| 2026-09-19 | gate de push | **E2E complète après les correctifs de revue : 216 passés, 7 échecs, 19 ignorés, 8,4 min** — `kesh_e2e` reconstruite (`DROP` + 69 migrations), frontend et backend rebuildés, `smtpConfigured:true`, run à 13:15 UTC. Les 7 échecs sont **exactement** ceux de la KF-029 (`mode-expert:26,41`, `onboarding-path-b:65,92`, `onboarding:57,77,150`) ; la KF-045 est passée, comme attendu l'après-midi ; aucune pollution. Avec le gate backend 2336/2336 de la passe 2, **story `done`**. |
| 2026-09-19 | revue P2 — **boucle close** | **Passe 2, CIBLÉE** (une lentille, Regression Hunter sur Sonnet, prompt versionné `25-2-c-review-prompt-p2.md`), sur le seul commit de remédiation `cd11aa35`. **0 CRITICAL, 0 HIGH, 1 MEDIUM, 0 LOW**, axes 1 à 5 tous déclarés exercés — le test seul n'a pas été exécuté par la lentille, ce que le prompt interdisait. **Le MEDIUM, confirmé** : l'écriture hors bande de `le_plancher_voit_une_ecriture_validee_pendant_la_transaction` était validée hors de `tx` et jamais supprimée — purgée par le `setup()` suivant et sans effet sous le groupe sérialisé, mais c'est le résidu que la règle KF-039 demande de traiter par réflexe. Supprimée à la fin du test, **avant** l'assertion pour partir même en cas d'échec. **Mutation rejouée** (plancher sans `FOR UPDATE`) : rouge sur `rendu 40` pour un 40 pris, **et zéro résidu en base après l'échec** ; code de production restauré, `git diff` vide. Confirmé par la lentille : le verrou d'intervalle rétabli est celui que prenait déjà le `MAX + 1` de `main`, **à la même position** — le commit restaure, il n'ajoute pas ; aucun cycle entre créations. Signalé hors verdict : `delete_in_tx` verrouille `journal_entries JOIN fiscal_years` dans un ordre peut-être inverse de `create_in_tx` — **antérieur** à la story, absent du diff. **Gate complet : 2336 passés, 4 ignorés, 94 s**, base remise à zéro et contrôlée. ⛔ **Clôture** : le correctif ne touche **aucune ligne de production** (un test de `mod tests`), ce qui ferme le motif « la remédiation introduit le défaut suivant ». Trend : **P1 3 MEDIUM / 5 LOW → P2 1 MEDIUM → 0 au-dessus de LOW**. |
| 2026-09-19 | revue P1 | **Passe 1 de `bmad-code-review`, trois lentilles** (Blind Hunter et Acceptance Auditor sur Sonnet, Edge Case Hunter sur Haiku 4.5), prompt versionné `25-2-c-review-prompt-p1.md`. **0 CRITICAL, 0 HIGH, 3 MEDIUM, 5 LOW** après dédoublonnage, chacun vérifié sur le code avant d'être retenu. ⛔ **Le MEDIUM qui compte porte sur ma propre décision hors spec** — le plancher de l'allocateur, relevé par **deux lentilles indépendantes** : lu sans verrou sous `REPEATABLE READ`, il lisait l'instantané de la **première** lecture de `create_in_tx` et ratait une écriture validée entre-temps, c'est-à-dire exactement le cas pour lequel il existe. **Rendu observable avant d'être corrigé** : le test neuf `le_plancher_voit_une_ecriture_validee_pendant_la_transaction` a rougi sur le code d'origine (`rendu 41` pour un 41 déjà pris), puis a viré au vert avec `FOR UPDATE` sur le plancher. ⚠️ Mon premier montage était faux — une écriture hors bande à `compteur + 10` faisait rougir le test sans qu'aucune collision n'existe ; c'est le numéro **exact** que le compteur s'apprête à rendre qui produit le défaut. Les deux autres MEDIUM : la ligne de `docs/migrations-idempotence-audit.md` qui déclarait le fondement **`Hors fenêtre`**, celui-là même que le garde-fou avait refusé en dev (corrigé : **couverture**) ; et **huit** jetons restés à l'ancien compte dans les commentaires de `migrations_upgrade_path.rs` (`total == 68`, « les 34 restantes »…) — ⛔ *le fichier même où je racontais avoir bougé les deux nombres du même pas*. LOW corrigés : la précondition d'`INSERT IGNORE` (qui change aussi une violation de FK en avertissement) écrite au doc-comment, inatteignable en production puisque `create_in_tx` verrouille l'exercice par `id` **et** `company_id` avant ; #381 ajoutée à la ligne E25 du `README.md` ; case du gate T1 cochée. LOW non retenus : référence de ligne périmée dans `kesh-report/src/trial_balance.rs:9` (**antérieure** à la story, hors périmètre) ; suppression de la société provisoire dans `auth/bootstrap.rs:185` (jamais mouvementée, donc sans ligne de compteur). ⚠️ **Haiku a coché l'axe des manuels « vérifié » après l'avoir déclaré non exercé** ; l'axe était couvert par les deux autres lentilles, PDF aplati compris. Symptôme grepé sur le dépôt (`plancher`, `rattrapage`, jetons `33\|34\|35\|68\|69`) : aucun autre site. **Gate complet après remédiation : 2336 passés, 4 ignorés, 87 s**, base remise à zéro et contrôlée ; `fmt` et `clippy -D warnings` verts. E2E non rejouée (aucun fichier frontend, aucune route touchée) — **à rejouer au push**. |
| 2026-09-19 | gate | **Gate complet backend : 2335 tests, 2335 passés, 4 ignorés, 91 s** (2334 + le test d'amorçage), base remise à zéro et contrôlée ; `fmt` et `clippy -D warnings` verts. **E2E complète : 214 passés, 9 échecs, 19 ignorés, 8,4 min**, `kesh_e2e` reconstruite (`DROP` + migrations), frontend rebuildé, `smtpConfigured:true`. Les 9 échecs, jugés fichier par fichier, sont **tous** attendus : les 7 de la KF-029 (`mode-expert:26,41`, `onboarding-path-b:65,92`, `onboarding:57,77,150`) et les 2 de la KF-045 (`invoices:405,429`), run lancé à 09:14 UTC. Aucune pollution. |
| 2026-09-19 | AC 4 | **L'angle mort déclaré est fermé** : test d'amorçage sur base peuplée, un test neuf, trois mutations de l'amorçage toutes tuées (retiré → `[]` ; `COUNT(*)+1` → `(10,110,4)` ; `MAX` sans `+1` → `(10,110,5)`), fichier de migration restauré et vérifié par `git status`. ⚠️ Le danger était **mal nommé** dans cette fiche et dans l'en-tête de la migration : depuis le plancher, un amorçage manqué ne bloque plus la saisie, il **réattribue en silence**. ⚠️ L'assertion comportementale (2) n'est pas éprouvée **isolément** : chaque mutation tombe d'abord sur l'assertion (1). |
| 2026-09-18 | preuves | **Le rattrapage prouvé par DEUX mutations symétriques, et c'était nécessaire de les jouer à deux.** (1) **Compteur retiré** (plancher seul) → `un_numero_libere_n_est_jamais_reattribue` **rouge** : sans compteur, le numéro libéré est réattribué. (2) **Plancher retiré** (compteur seul) → `post_accept_reconciles_transaction_and_invoice` **rouge** : sans plancher, une écriture arrivée hors de l'allocateur bloque toute création. ⛔ **Aucune des deux moitiés n'est décorative** — chacune garde un défaut que l'autre ne voit pas, et une seule mutation aurait laissé croire le contraire. Chaque mutation jouée seule, restaurée, identité vérifiée avec un détecteur **borné au fichier**. ⚠️ La preuve antérieure du test décisif (rebranchement du `MAX + 1`) ne valait plus : l'allocateur avait changé depuis, et une preuve porte sur le code qu'elle a mesuré, pas sur son nom. |
| 2026-09-18 | gate | **Gate complet backend : 2334 tests, 2334 passés, 4 ignorés, 91 s**, sur une base **remise à zéro et contrôlée** (41 tables, 1 société, 1 admin, 1 exercice, 5 comptes). `fmt` et `clippy --workspace --all-targets -D warnings` verts. Gate **complet et non ciblé** : la branche touche les migrations **et** deux repositories, ce qui interdit le ciblage. ⛔ **Aucun neuvième garde-fou** : les huit réparations tiennent ensemble — c'était l'objet même de ce run, après huit corrections dont aucune n'avait été trouvée par moi. ⚠️ **E2E pas encore jouée** ; frontend non concerné, la branche n'y touche aucun fichier. |
| 2026-09-18 | dev T1→T4 | **⛔ Une décision de conception non prévue par la spec, et c'est la plus lourde : l'allocateur retient LE PLUS GRAND des deux** — son compteur, ou le plus grand numéro en service plus un. Sept tests de rapprochement l'ont imposée (`Duplicate entry '1-1-1'`) : le compteur n'avance que si l'on passe par lui, et si une écriture arrive autrement, il rend un numéro déjà pris — l'`UNIQUE` refuse alors, et **toute création d'écriture échoue indéfiniment** jusqu'à intervention manuelle. Pour une comptabilité, un blocage total de la saisie est inacceptable. ⚠️ **Le rattrapage ne coûte rien à la propriété** : il ne joue que vers le HAUT, et un numéro libéré laisse le compteur au-dessus. Établi au préalable, et contre mon premier relevé qui disait l'inverse : **aucun chemin de production n'insère une écriture hors de l'allocateur** (mon grep cherchait dans `src/`, où vivent aussi les tests de `kesh-db`). |
| 2026-09-18 | dev — garde-fous | **HUIT garde-fous réveillés, et le dépôt me les a TOUS signalés : aucun trouvé par moi.** ⚠️ Mes inventaires P5/P6/P7 en avaient manqué la moitié. (1) `migrations.sha384`, registre de checksums P8 **que je n'avais jamais rencontré** — le test donne la ligne à coller. (2)(3) `migrations_upgrade_path` : compteur 68→69 **et** soustracteur 34→35 ; j'ai fait le premier sans le second, la fenêtre a glissé d'un cran et le test a rougi sur `COUNT(accounts) : expected 4, got 2` — *un échec qui ne ressemble en rien à sa cause*. (4) décompte d'export 38→39. (5)(6)(7) les trois tests de libellé d'acteur — **substitution de source**, pas ajout : j'ai d'abord ajouté le rejeu sans changer la source, et les trois ont continué d'échouer au même point. (8) `report_entry` devenu `never used`, supprimé plutôt que masqué. ⛔ **Et la clé étrangère du compteur vers `fiscal_years` a fait tomber DIX-SEPT tests d'un coup** — `invoice_number_sequences` porte la même, mais ne crée sa ligne qu'à la validation d'une facture ; la mienne en crée une dès la première écriture. Quatre sites de nettoyage l'ignoraient, dont `kesh-seed` (code **livré**) et un `invoices.rs` dont le `.ok()` aurait **avalé** l'échec en laissant un résidu. |
| 2026-09-16 | spec | Story créée, détachée de la 25-2-b. Fait établi le même jour : le compteur monotone **existe déjà** dans le dépôt (`invoice_number_sequences`), à la portée exacte d'`entry_number` — la story l'étend au lieu de l'inventer. |
