# Story 33-1a : le numéro interne des factures fournisseur — colonne, format, attribution, numérotation de l'existant (#926)

Status: ready-for-dev

*Fiche rédigée le 2026-11-06, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche
`story/33-0-kickoff`, numéros de ligne de `4ef19901`. Registre : `e33-choix-autonomes-e331.md`, choix `C-e331-1` et suivants.
Clé courte `e331`. Décisions du Project Lead : `epic-33-numerotation-et-defauts.md` § *Décisions de Guy* et les
commentaires de #926 ; elles font foi. **Seule migration de l'Epic 33.** La 33-1b (écrans) suit cette story.*

## Story

En tant que comptable qui classe ses factures papier,
je veux que chaque facture fournisseur validée reçoive de Kesh un numéro unique et lisible,
afin de retrouver le papier sans dépendre du numéro du fournisseur, facultatif et parfois identique d'un fournisseur à l'autre.

## Ce qui est établi au code

| # | Fait | Source (à `4ef19901`) |
|---|---|---|
| F1 | Le seul numéro d'une facture fournisseur est celui du fournisseur, `VARCHAR(64) NULL`, sans unicité | `20260628000001_supplier_invoices.sql:31`, `supplier_invoices.rs:41` |
| F2 | **Une table neuve rendrait toute sauvegarde 0.17.4 inimportable** : le manifeste doit porter exactement `TABLES_TO_TRUNCATE`, dans les deux sens. Elle refermerait aussi la fenêtre d'importabilité, donc interdirait l'entrée P7 de cette migration | `admin_backup/import.rs:250-263`, `post_restore.rs:21-27` |
| F3 | Une colonne neuve nullable ou pourvue d'un `DEFAULT` n'est pas exigée d'une sauvegarde ancienne ; une colonne inconnue de la destination est refusée — donc une sauvegarde 0.17.5 est refusée par une 0.17.4 | `admin_backup/import.rs:613-627` |
| F4 | `validate_in_tx` tient déjà, pour toute validation, la ligne `company_invoice_settings` `FOR UPDATE` (2 bis) et la ligne `fiscal_years` `FOR UPDATE` (3) | `supplier_invoices.rs:1421`, `:1488` ; `company_invoice_settings.rs:107-114` ; `fiscal_years.rs:758` |
| F5 | Une validation non brouillon a toujours une écriture d'achat et son exercice : `chk_supplier_invoices_entry_iff_not_draft`, `journal_entries.fiscal_year_id NOT NULL`, clés `RESTRICT` | `20261031000001_supplier_invoices_draft.sql:44`, `20260412000001_journal_entries.sql:19,28` |
| F6 | La route de validation est rejouée sur interblocage | `routes/supplier_invoices.rs:870` |
| F7 | Deux seuls libellés à changer : achat et règlement | `supplier_invoices.rs:1511-1512`, `:1909` |
| F8 | `render` existe et rend `{YEAR}` depuis l'année de `start_date` | `kesh_core::invoice_format::render` (`invoice_format.rs:206`) ; `credit_notes.rs:559-572`, `invoices.rs:2305-2322` |
| F9 | `validate_template` **accepte un format sans `{SEQ}`** : `F-{YEAR}` passe, et tous les numéros seraient identiques | `invoice_format.rs:151-199` (seul `has_placeholder` est exigé) |
| F10 | Une facture cliente dévalidée reprend son numéro sans toucher au compteur : le patron à suivre | `invoices.rs:2293-2330` |
| F11 | Deux exercices d'une société peuvent commencer la même année civile : seuls le nom et la date de début sont uniques, et `chk_fiscal_years_dates` n'impose pas une année | `20260404000001_initial_schema.sql:49-51` |
| F12 | `fiscal_years` et `supplier_invoices` portent `updated_at … ON UPDATE CURRENT_TIMESTAMP` : un `UPDATE` de backfill le changerait, sauf `SET updated_at = updated_at` | `initial_schema.sql:47`, `20260628000001_supplier_invoices.sql:51` |
| F13 | `LPAD(1000, 3, '0')` **tronque** à `100` : `LPAD` seul ferait des doublons à la millième facture d'un exercice | comportement MariaDB ; à constater au test (AC7) |
| F14 | Le registre de rejeu est vide depuis la 32-5a, et un test exige ce vide ; le manuel d'administration dit « vide depuis la 0.16.0 » | `post_restore.rs:210-227`, `tests/admin_full_import_e2e.rs:1597` (`report.is_empty()`), `admin-manual.tex:2358` |
| F15 | Le service et le test de montée utilisent `supplier_invoices::list` ; les colonnes de `SupplierInvoice` sont écrites en trois listes | `services/factures_fournisseur.rs:119` ; `supplier_invoices.rs:41-47`, `:51-56`, `:2817-2822` |
| F16 | MariaDB est en 10.11 en production comme en dev | `docker-compose.yml:4`, `docker-compose.dev.yml:56` |

## Décisions de conception (registre pour le détail)

- **Le compteur est une colonne de `fiscal_years`, non une table de séquence** (`C-e331-1`). La mécanique de
  `credit_note_number_sequences` (`credit_note_number_sequences.rs:17-76`) se réemploie dans son principe — lecture sous verrou,
  incrément, annulé par le rollback, jamais redescendu — mais pas dans sa forme : F2 l'interdit. Le compteur sur l'exercice ne
  prend **aucun verrou neuf** (F4) et se restaure avec l'exercice.
- **Concurrence** (`C-e331-2`) : la ligne de réglages sérialise les validations d'une société, celle de l'exercice porte le
  compteur, la lecture du compteur et le contrôle de collision sont **verrouillants** (sous REPEATABLE READ, une lecture ordinaire
  rendrait un instantané périmé). L'`UNIQUE` reste le filet.
- **Un exercice dont le nom n'est pas une année** (`C-e331-3`) : `{YEAR}` vaut l'année de `start_date`, comme pour les factures
  clientes et les avoirs (exercice « 2025/2026 » commencé le 1.7.2025 : `FF-2025-001`). `{FY}` reste offert.
- **« Configurable »** (`C-e331-4`) : le format réglable y répond (préfixe, année, largeur du compteur). Ni valeur de départ, ni
  choix du mode de remise à zéro. Le format fournisseur **exige `{SEQ}`** (F9).
- **Brouillon numéroté supprimé** (`C-e331-5`) : suppression permise, numéro perdu, **jamais réattribué** ; le trou est accepté
  et l'audit garde l'instantané.
- **Hors de tout exercice** (`C-e331-6`) : un brouillon hors exercice n'a pas de numéro et sa validation est refusée comme
  aujourd'hui (`FiscalYearInvalid`) ; à la migration, l'exercice se lit sur l'écriture d'achat, jamais sur la date : aucune
  facture n'en manque (F5).
- **Changement de format en cours d'exercice** (`C-e331-7`) : vaut pour les numéros à venir ; les numéros attribués ne se
  réécrivent pas, le compteur continue.
- **Collision** (`C-e331-8`) : même rendu pour deux exercices de même année civile (F11), ou format sans année. La validation
  **saute** les numéros pris ; la migration numérote ces exercices à la suite l'un de l'autre. Le seul cas où la séquence ne
  repart pas à 1.
- **Brouillon numéroté déplacé dans un autre exercice** (`C-e331-9`) : garde son numéro, le compteur du nouvel exercice ne
  bouge pas. Décision de Guy appliquée à la lettre : le papier est déjà classé.
- **Libellé** (`C-e331-10`) : `Facture fournisseur FF-2026-001 (ABC-123) - Dupont SA`, et `Règlement fournisseur …` de même ; sans
  numéro du fournisseur, `Facture fournisseur FF-2026-001 - Dupont SA`. Le numéro du fournisseur reste cherchable au journal.
- **Pas de `CHECK` « non brouillon ⇒ numéro »** (`C-e331-11`) : il ferait échouer la restauration d'une sauvegarde 0.17.4 avant le
  rejeu, et toute fixture qui insère une facture ouverte en SQL. L'absence se contrôle par test.
- **Migration non breaking** (`C-e331-14`) : `ADD COLUMN` nullable ou avec défaut, `ADD UNIQUE` ; ni `DROP`, ni `MODIFY`, ni
  `RENAME` (P1, P3). `kesh_version_min_required` reste à `0.17.0`, **aucun bump Cargo ici** (la release s'en charge). Comme toute
  migration, elle empêche la 0.17.4 de démarrer sur la base migrée (P1).

### Dérogation à la règle de splitting

La story touche **huit modules** (`kesh-core/invoice_format`, `kesh-db/supplier_invoices`, `kesh-db/company_invoice_settings`,
`kesh-db/post_restore`, `kesh-api/routes/supplier_invoices`, `kesh-api/routes/company_invoice_settings`, `kesh-api/exports`,
`kesh-api/mcp`), au-dessus du seuil de cinq. **Dérogation assumée** (`C-e331-18`) : la migration, l'attribution, le réglage et le
rejeu forment un seul invariant, et les séparer laisserait dans `main` une colonne remplie à la montée que la validation
n'alimenterait pas, ou l'inverse. Le découpage en 33-1a (le patron) et 33-1b (le déploiement) est déjà celui que la règle
prescrit. Risque accepté : une revue plus large ; à la revue, s'en tenir aux lentilles nommées ci-dessous.

## Acceptance Criteria

### La migration (`20261106000001_supplier_invoice_internal_number.sql`)

**AC1 — Le schéma.** Après montée, et dans cet ordre d'instructions : `supplier_invoices.internal_number VARCHAR(64) NULL` ;
`fiscal_years.supplier_invoice_next_number BIGINT NOT NULL DEFAULT 1` avec `CHECK (… >= 1)` ;
`company_invoice_settings.supplier_internal_number_format VARCHAR(64) NOT NULL DEFAULT 'FF-{YEAR}-{SEQ:03}'` avec `CHECK` non vide
(patron de `chk_cis_cn_format_nonempty`) ; les deux `UPDATE` de l'AC2 et l'AC3 ; **en dernier**
`CONSTRAINT uq_supplier_invoices_internal_number UNIQUE (company_id, internal_number)`. Aucune table neuve : `TABLES_TO_TRUNCATE`
inchangé, et le test le dit (patron `draft_migration_adds_no_table`, `migrations_upgrade_path.rs:1679`).

**AC2 — La numérotation de l'existant.** Toute facture `open`, `paid` ou `cancelled` reçoit un numéro ; une facture `draft` n'en
reçoit pas. L'exercice est celui de son écriture d'achat (`journal_entries.fiscal_year_id`). Le rang se calcule par société et par
année de début d'exercice, dans l'ordre `fy.start_date, si.invoice_date, si.id`. Forme à essayer au développement :

```sql
UPDATE supplier_invoices si
  JOIN (
    SELECT s.id, YEAR(f.start_date) AS annee,
           ROW_NUMBER() OVER (PARTITION BY s.company_id, YEAR(f.start_date)
                              ORDER BY f.start_date, s.invoice_date, s.id) AS rang
      FROM supplier_invoices s
      JOIN journal_entries je ON je.id = s.purchase_journal_entry_id AND je.company_id = s.company_id
      JOIN fiscal_years f ON f.id = je.fiscal_year_id AND f.company_id = s.company_id
     WHERE s.status <> 'draft'
  ) n ON n.id = si.id
   SET si.internal_number = CONCAT('FF-', n.annee, '-', LPAD(n.rang, GREATEST(3, CHAR_LENGTH(n.rang)), '0')),
       si.updated_at = si.updated_at
 WHERE si.internal_number IS NULL;
```

*Si MariaDB 10.11 refuse la table cible dans la table dérivée, matérialiser autrement et le dire au Change Log ; la sémantique ne
change pas.* `ROW_NUMBER` a un précédent au dépôt (`20261010000001_lettering_documents_backfill.sql:162`).

**AC3 — Le compteur.** Chaque exercice reçoit `supplier_invoice_next_number` = plus grand rang de ses factures + 1, par la **même**
partition et le même ordre, `GREATEST` avec la valeur en place, `updated_at` préservé. Un exercice sans facture reste à 1.

**AC4 — Rien d'autre ne bouge.** Ni `version` ni `updated_at` de la facture ; **aucune ligne de `journal_entries`** : la description et
`updated_at` des écritures passées sont identiques à l'octet. Décision de Guy.

**AC5 — Montée sur une base 0.17.4 peuplée** (`migrations_upgrade_path.rs`, `#[sqlx::test(migrations = false)]`, montage
`migrations_before(20261106000001, …)`). ⚠️ Peupler en **SQL brut** et `journal_entries::create_in_tx` (patron `:1270-1301`), jamais
par `supplier_invoices::*` : `FIND_SCOPED_SQL` lit désormais une colonne absente de la base partielle. Le jeu couvre, en un seul
montage : deux sociétés ; exercices 2025 et 2026 ; **trois factures à la même date** (ordre par `id`) ; une facture de date
antérieure insérée après (ordre par date) ; `open`, `paid`, `cancelled`, `draft` ; numéro fournisseur présent, absent, identique
chez deux fournisseurs ; un exercice « 2025/2026 » commencé le 1.7.2025 ; **deux exercices commençant la même année civile**
(numérotés à la suite, aucun échec d'`UNIQUE`) ; un exercice sans facture. Assertions : numéro exact de chaque ligne, compteurs
exacts, AC4 ligne à ligne, aucune facture non brouillon sans numéro, brouillon à `NULL`. Assertion de montage : la base partielle
n'a pas la colonne.

**AC6 — Montée à vide.** Une base sans facture fournisseur monte sans erreur (`migrations_fresh_install`) ; le squash est régénéré
(AC21).

**AC7 — La millième.** Sur un exercice à 1 001 factures (`INSERT` en SQL, l'écriture d'achat peut être partagée : aucune unicité sur
`purchase_journal_entry_id`), le rang 1000 rend `FF-2026-1000` et non `FF-2026-100`.

### La validation

**AC8 — L'attribution.** `validate_in_tx`, sur une facture sans numéro, attribue le numéro **après** `check_written` (dernier refus de
forme) et **avant** la construction du libellé : prochain numéro lu en lecture verrouillante sur `fiscal_years` (aucun verrou neuf,
F4), `render(format, année de fy.start_date, fy.name, seq)`, contrôle de collision, écriture du compteur (`seq + 1`,
`updated_at = updated_at`) ; le numéro part dans l'`UPDATE` de l'étape (6), celui qui passe à `open`. Le doc-comment des verrous
(`:1259-1324`) porte l'étape (3 bis).

**AC9 — Le numéro survit.** `unvalidate` le garde : validée puis brouillon, annulée puis brouillon. Revalidée, la facture **reprend
le même numéro sans lire ni écrire le compteur** (patron F10), et ce même si le format a changé entre-temps.

**AC10 — Un refus ne consomme rien.** Validation refusée (exercice absent ou clos, compte archivé, compte non imputable, réglages
incomplets, conflit de version) : numéro `NULL`, compteur inchangé. Échec **après** l'attribution (`create_in_tx`) : idem, par le
rollback.

**AC11 — Séquence par exercice.** Validations entrelacées sur 2026 et 2027 : `FF-2026-001`, `FF-2027-001`, `FF-2026-002`.

**AC12 — Concurrence.** N validations simultanées de la même société et du même exercice, par des transactions réelles : numéros tous
distincts, suite sans trou, compteur à N+1, aucun échec sinon un interblocage rejoué. Test piloté par le patron de
`rejeu_interblocage_e2e.rs`.

**AC13 — Le saut.** Numéro déjà pris : deux exercices de même année civile ; retour à un format antérieur ; format sans année. La
validation saute au premier libre, le compteur finit au-delà. Saut borné : au-delà de 100 000 essais, `DbError::Invariant`. Le
rendu impossible (`FormatError`, nom d'exercice trop long pour le format) rend `DbError::ConfigurationRequired` (`errors.rs:4483`,
400) et ne consomme rien (`C-e331-17`).

**AC14 — Personne d'autre n'écrit le numéro.** `create_draft`, `update_draft` et la complétion d'un document (`POST
/imported-supplier-invoices/{id}/complete`) l'ignorent, même si le corps porte `internalNumber`. Un `update_draft` le préserve. La
suppression d'un brouillon numéroté réussit, le numéro figure à l'instantané d'audit, et le compteur ne redescend pas : la facture
validée suivante prend le numéro d'après, jamais celui du supprimé (`C-e331-5`).

### Les libellés

**AC15 — Les nouvelles écritures.** Selon `C-e331-10`, dans les deux sites de F7, par **une** fonction pure testée (numéro interne
seul ; avec numéro du fournisseur ; numéro du fournisseur blanc, qui vaut absent — règle de `:956-959` ; sans numéro interne, ce qui
ne se produit que par une insertion directe : repli sur le numéro du fournisseur). Une facture numérotée par la migration et réglée
après porte son numéro au règlement. Revalidée, la nouvelle écriture d'achat aussi. `journal_entries.description` (`VARCHAR(500)`)
tient au pire cas : 64 + 64 + 255 plus le texte fixe.

### Le réglage, l'API, l'export

**AC16 — Le format.** `CompanyInvoiceSettings`, `COLUMNS`, l'instantané d'audit, la comparaison de non-changement, le `SET` et le
`SELECT` de repli de `company_invoice_settings.rs:37,53,247,350,1159` portent `supplier_internal_number_format`. `GET` et `PUT
/company/invoice-settings` portent `supplierInternalNumberFormat` ; **absent du corps : préservé** (patron #216,
`routes/company_invoice_settings.rs:295-302`), le formulaire ne l'envoyant qu'à partir de la 33-1b. Validation par une fonction neuve
`invoice_format::validate_numbering_template`, qui exige en plus `{SEQ}` (`FormatError::NoSeqPlaceholder`, message FR comme ses
voisines). Refus 400 pour `FF-{YEAR}`, pour un format vide, pour `FF-{YEAR}-{SEQ:11}`. Les formats client et avoir gardent
`validate_template` (`C-e331-16`).

**AC17 — L'API de la facture.** `internalNumber` (`string | null`) dans `SupplierInvoiceResponse` et `SupplierInvoiceListItemResponse`
(`routes/supplier_invoices.rs:75,303`), donc dans toutes les réponses qui les réemploient. `supplierInvoiceNumber` ne change ni de nom
ni de valeur. Les trois listes de F15 et `SupplierInvoiceListItem` suivent. `snapshot_json` porte `internalNumber`.

**AC18 — MCP.** `kesh_factures_fournisseur` et sa fiche (`mcp/outils/factures_fournisseur.rs:54,101,167,225`) ajoutent `numero_interne`,
`TexteNonFiable` comme `numero`, et `numero` garde son sens. `mcp_outils_e2e` et le test de contrat suivent. La pièce source de
`kesh_ecriture` relève de la 33-1b.

**AC19 — L'export.** `serialize_supplier_invoices_csv` (`exports/csv_tables.rs:1206,1232`) gagne `internal_number` juste après
`supplier_invoice_number` ; `company_invoice_settings` gagne `supplier_internal_number_format` (`:961,985`). Le compteur n'est pas
exporté en CSV, comme les autres compteurs (`exports/global.rs:172-183`) : `serialize_fiscal_years_csv` ne change pas.
`exports_global_e2e` suit.

### La sauvegarde

**AC20 — P7, classe B.** Entrée au registre `POST_RESTORE_BACKFILLS`, version `20261106000001`, déclencheur
`Sentinels(&[("supplier_invoices", "internal_number")])`, SQL dans `post_restore/20261106000001_supplier_invoice_internal_number.sql`,
copie **verbatim** des deux `UPDATE` (tests `extract_statements_are_verbatim_substrings_of_source_migration`,
`extract_carries_every_write_statement_of_its_source_migration`, `registry_sql_contains_no_ddl`, `registry_sql_has_no_literal_hazard`,
`class_b_sentinel_column_is_added_by_its_own_migration`, `registry_entries_are_within_import_window`). Le commentaire « VIDE depuis la
Story 32-5a » est remplacé. Dans `admin_full_import_e2e.rs` :

- **Une sauvegarde 0.17.4 reçoit les numéros** : export, `strip_column` (`:931`) de `supplier_invoices.internal_number`,
  `fiscal_years.supplier_invoice_next_number`, `company_invoice_settings.supplier_internal_number_format`, import par la **vraie route**
  et le **registre de production** ; numéros et compteurs ceux de la migration, format au défaut, entrée `REPLAYED_SENTINELS_ABSENT`
  ; la validation suivante prend le numéro d'après.
- **Une sauvegarde 0.17.5 garde les siens** : numéros non contigus (un brouillon numéroté supprimé), compteur au-delà du plus grand ;
  après import, numéros et compteur **inchangés**, entrée `SKIPPED`.
- **La sauvegarde 0.17.5 refusée par une 0.17.4** se constate à la répétition (AC22), non par un test du dépôt.
- `full_import_report_mirrors_the_production_registry` (`:1597`) perd son `report.is_empty()` : une entrée, `SKIPPED`, zéro ligne. La
  boucle n'est plus vacuement verte.

**AC21 — P5, P6, schéma de test.** P5 : ligne au tableau de `docs/migrations-idempotence-audit.md`, verdict `tracked-by-sqlx` (erreur
1060 au rejeu) ; **90** aux deux sites du total, `yes` 11, `tracked-by-sqlx` 79, `no` 0, recomptés. P6 : `grep -rn
"migrations.len()\|apply_migrations_up_to" crates/` inspecté et chaque site listé au Dev Agent Record ; à ce jour, à toucher :
`migrations_upgrade_path.rs:127` (`total` 89 → 90), `:219` (`total - 55` → `total - 56`, frontière figée à 34) et les commentaires
`:46,:124,:139,:149,:212`. Par version, sans changement : `pre_upgrade_backup/tests.rs:184,191,506`, `sauvegarde_avant_migration.rs:28,43`.
Squash : `scripts/regen-test-schema.sh`, **jamais édité à la main**, `test_schema_guard` vert. P8 : la migration ne se modifie plus
après son premier passage sur une base persistante, ou l'on reconstruit ces bases et on l'annonce.

**AC22 — La répétition réelle, sur données synthétiques.** Aucun test `#[sqlx::test]` ne démarre le binaire sur une base persistante
(P8). Base jetable `kesh_e331rep` : migrations jusqu'à `20261104000001`, seed de démonstration, factures fournisseur de tous statuts,
`last_applied = '0.17.4'`. (a) **Montée** : démarrage du binaire de la branche : `backup/pre-upgrade/` porte une sauvegarde, la
migration passe, `/health` répond, numéros présents. (b) **Reprise** : cette sauvegarde importée dans une seconde base au schéma à jour
redonne les mêmes numéros. (c) **Retour** : l'image `gcorbaz/kesh:0.17.4` refuse de démarrer sur la base migrée ; sur une base neuve, elle
importe la sauvegarde `pre-upgrade/` et démarre ; elle refuse une sauvegarde prise après la montée. Commandes et résultats au Dev
Agent Record, journal dans `kesh-gate-logs/e331/`. Ce que le manuel affirme du retour ne s'écrit qu'après ce passage (AC23).
⚠️ **Jamais le NAS, jamais `~/kesh/`.**

### Les textes

**AC23 — Manuel d'administration** (`.tex` et PDF, PDF aplati contrôlé) : `\subsubsection{Passer à la 0.17.5}\label{sec:maj-0-17-5}` avant
`:2524` : une migration, non breaking, seuil inchangé ; **la première sauvegarde automatique réelle** (`backup/pre-upgrade/`, #847) ; **prendre
aussi une sauvegarde manuelle complète** — base, `.keshbackup`, dossier des documents ; **le retour à la 0.17.4 passe uniquement par la
sauvegarde d'avant la mise à jour** : la 0.17.4 ne démarre plus sur la base migrée et refuse une sauvegarde de la 0.17.5 ; les factures déjà
validées reçoivent un numéro par ordre de date, et les écritures passées ne changent pas. `:2283` (*Compatibilité*) : une sauvegarde 0.17.4
s'importe dans la 0.17.5 et y reçoit les numéros ; pas l'inverse. **`:2358`, phrase « Depuis la 0.16.0, aucune de ces reprises n'est plus
rejouée » et « vide depuis la 0.16.0 » : récrites**, la numérotation est rejouée. `:2527` (« la prochaine version qui migre ») : relu, renvoi à
`sec:maj-0-17-5`. Le manuel utilisateur et le réglage à l'écran sont à la 33-1b.

**AC24 — CHANGELOG et API.** `## [0.17.5] — Non publié` (union) : *Ajouté*, le numéro interne, avec l'avertissement de migration et la
sauvegarde, `refs #926`. `docs/api-external.md` : `internalNumber`, `supplierInternalNumberFormat`, cycle d'une facture.

## Tests qui rougissent avant le correctif

À écrire d'abord et à constater rouges, note au Dev Agent Record : AC2/AC3/AC5 (colonne absente), AC8, AC9, AC13, AC15, AC16 (`FF-{YEAR}`
accepté), AC20. Repository : `crates/kesh-db/tests/supplier_invoices_internal_number.rs`, sur le squash. Unitaire : `invoice_format.rs`
(`validate_numbering_template`) ; la fonction de libellé.

## Mutations nommées (sous le verrou, restauration `git checkout -- <fichier>` puis `touch`)

| # | Mutation | Tuée par |
|---|---|---|
| M1 | Rang sans tri par date (`ORDER BY s.id`) | AC5 |
| M2 | Sans `s.id` en dernier tri | AC5 (mêmes dates) |
| M3 | `LPAD(n.rang, 3, '0')` | AC7 |
| M4 | Partition par exercice au lieu de l'année civile | AC5 (même année : l'`UNIQUE` échoue) |
| M5 | `status <> 'draft'` retiré ou réduit à `open` | AC5 |
| M6 | Exercice lu par la date au lieu de l'écriture | AC5, ou le dire équivalente, preuve à l'appui (F5) |
| M7 | `updated_at = updated_at` retiré, facture ou exercice | AC4 |
| M8 | Compteur non initialisé par la migration | AC5, AC20 |
| M9 | Revalidation qui tire un numéro neuf | AC9 |
| M10 | Saut de collision retiré | AC13 |
| M11 | Lecture du compteur sans `FOR UPDATE` | AC12, ou équivalente sous le verrou de l'exercice : à prouver |
| M12 | Attribution avant `check_written` | AC10 : indiscernable par le rollback ⇒ à déclarer équivalente si c'est le cas |
| M13 | Libellé : le numéro du fournisseur à la place | AC15 |
| M14 | Sentinelle sur une autre colonne | AC20 |
| M15 | `validate_numbering_template` remplacé par `validate_template` | AC16 |
| M16 | Format absent du corps écrasé par le défaut | AC16 |
| M17 | `update_draft` qui efface le numéro | AC14 |
| M18 | Compteur décrémenté à la suppression | AC14 |

## Tâches

1. **Étape 0** : relire F1–F16 au code, citer les lignes ayant bougé ; `git grep -n "struct SupplierInvoice\b\|SupplierInvoice {"`.
2. Tests rouges (ci-dessus).
3. Migration ; `regen-test-schema.sh` ; P5, P6, P7.
4. `invoice_format`, entité, listes de colonnes, attribution, libellés.
5. Réglages, routes, MCP, CSV.
6. Rejeu et tests d'import.
7. Mutations.
8. Gate complet, E2E complet, répétition (AC22).
9. Manuel (AC23, après AC22), PDF, CHANGELOG, `docs/api-external.md`.

## Gates

⛔ **Gate complet même en cours de boucle** : `crates/kesh-db/migrations/`, `post_restore.rs` et un repository sont touchés (§ *Test Locally
First*, exception `kesh-db`). Commandes lourdes : `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock
scripts/mem-guard.sh <commande>`, une par prise, `free -g` avant. Bases du worktree recréées à neuf avant chaque gate complet,
`DATABASE_URL` forcée.

`cargo fmt --all -- --check` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `scripts/test-fast.sh --ci` ; `cargo build --workspace
--all-targets` ; côté frontend `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` (rien ne devrait bouger : à constater). **E2E complet au
dernier commit de code**, 7 à 10 échecs attendus, jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus*. Vérification
après démarrage réel : `kesh_e2e_*` à la migration comprise (P8).

**Aucune capture** : aucun contrôle visible n'est ajouté. Les libellés du journal changent ; `supplier-documents.spec.ts:476` lit le lien de
la pièce source, qui ne bouge qu'à la 33-1b.

## Revue

Le prompt nomme : le **manuel d'administration** contre le code et le PDF aplati (§ *Le prompt d'une passe doit NOMMER le manuel*) ; la
migration sur une base peuplée ; la cohérence **migration / extrait de rejeu** ; les verrous de `validate_in_tx` ; les sites non résolus
(tout `format!` de libellé, toute liste de colonnes). Axes exercés et non exercés exigés au rapport ; `scripts/prepare-release.sh` interdit.

## Hors périmètre

Écrans, recherche, tri, autres sites d'affichage du numéro : 33-1b. Bump Cargo, macros des manuels, tag : release. Format client ou avoir sans
`{SEQ}` : issue demandée (`C-e331-16`).

## Dev Agent Record

*À compléter : sites P6 inspectés, tests rouges constatés, mutations, gates et chiffres, journal de la répétition.*

## Change Log

- 2026-11-06 : fiche créée, `ready-for-dev`.
