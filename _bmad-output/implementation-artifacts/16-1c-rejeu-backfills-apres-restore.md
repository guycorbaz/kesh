# Story 16.1c : Rejeu des backfills de données après un restore d'installation

## Status

ready-for-dev

## Story

**As a** exploitant d'une instance Kesh qui restaure un `.keshbackup` produit par une version antérieure,
**I want** que les **backfills de données** portés par les migrations postérieures à ce backup soient **rejoués à la fin du restore**,
**so that** l'import ne réintroduise pas silencieusement, et **définitivement**, les bugs que ces migrations avaient fermés — le rôle des comptes, le compte créanciers par défaut, la catégorie des taux de TVA et le compte de produit par ligne revenant tous à `NULL` ou à leur valeur par défaut sans qu'aucun message ne le signale.

Issue : **#281**. Sous-story de l'Epic 16 « Facturation avancée ».

**Dépend de 16-1a-bis** (migration `20260729000001`), dont elle ferme la régression au restore. Les deux doivent partir dans **la même PR et la même v0.9.0** : publier 16-1a-bis sans cette story reviendrait à livrer la mine décrite par l'issue.

### Pourquoi cette story est dans l'Epic 16 et non dans l'Epic 17

Le code touché relève du chemin **backup/restore** (Epic 17, clos). Le rattachement à l'Epic 16 procède du **triage hors fenêtre rétrospective** de la § « Tech debt management » de `CLAUDE.md` : une dette catégorie A découverte en cours d'Epic et **critique pour l'Epic en cours** est traitée dans l'Epic en cours. Ici le critère est franchi de la manière la plus directe qui soit — 16-1a-bis **n'est pas encore publiée**, et c'est elle qui crée le cas.

*(Arbitrage de Guy du 2026-08-01, tracé en commentaire de l'issue #281.)*

---

## Contexte

### Le mécanisme, vérifié en ground-truth

Trois protections auraient pu bloquer le cas. **Aucune ne le fait.**

1. **Le contrôle de version ne refuse qu'un backup trop récent.** `check_import_version_compat` (`crates/kesh-db/src/version.rs:281`, appelé en `routes/admin.rs:152`) ne rend `DowngradeRefused` que si le manifeste exige un binaire **plus récent** que le nôtre. Un backup **ancien** est accepté — et c'est voulu : restaurer une sauvegarde ancienne est le cas d'usage nominal.

2. **Le contrôle de schéma ne bloque pas non plus.** `check_schema_compat` (`crates/kesh-api/src/admin_backup/import.rs:195`) n'exige la présence d'une colonne destination dans la source que si `ColumnConstraint::is_required()` (`crates/kesh-db/src/backup.rs:329-331`) est vrai :

   ```rust
   pub fn is_required(&self) -> bool {
       !self.is_nullable && !self.has_default && !self.is_auto_increment && !self.is_generated
   }
   ```

   **Le `has_default` est aussi déterminant que le `is_nullable`** — et c'est le point que l'issue ne dit qu'à moitié. Une colonne `NOT NULL DEFAULT …` est tout aussi silencieusement absente du contrôle qu'une colonne nullable : elle est simplement **réinitialisée à son défaut** au lieu de tomber à `NULL`. C'est le cas d'`accounts.postable` (`NOT NULL DEFAULT TRUE`) et de `vat_rates.category` (`NOT NULL DEFAULT 'custom'`). Ne pas restreindre le raisonnement aux colonnes nullables.

3. **`_sqlx_migrations` n'est pas restaurée**, donc la migration reste marquée appliquée et ne repassera jamais. **Ancre exacte** : ce n'est pas la clause `AND TABLE_NAME NOT IN ('_sqlx_migrations', '_kesh_version')` de `backup.rs:586` — celle-ci est dans le **test** `backup_inventory_matches_schema`, qui ne fait que **documenter** l'invariant. La cause réelle est que `TABLES_TO_TRUNCATE` (`backup.rs:34-72`) ne **contient pas** ces deux tables, et que l'export (`admin_backup/export.rs:55`) comme le restore (`backup.rs:425`) n'énumèrent que cette constante. *(L'issue cite `:586` ; suivre l'ancre sans borner la fonction englobante mènerait à patcher un test — c'est le mode d'échec relevé en rétro 16-1a.)*

Le `DELETE` + `INSERT` du restore ne cite que les `columnNames` du manifeste (`backup.rs:468`) : toute colonne absente de la source prend `NULL` ou son `DEFAULT`, en silence.

### Le périmètre réel : 4 migrations, pas 1

Triage des **6** migrations du dépôt qui portent un `UPDATE` (`grep -ln "^\s*UPDATE\b" crates/kesh-db/migrations/*.sql`) :

| Migration | Ce que le restore d'un backup antérieur perd | Verdict |
|---|---|---|
| `20260613000001_vat_rates_crud.sql` (11-1) | `vat_rates.category` → `'custom'` par `DEFAULT` | **exposée** |
| `20260628000001_supplier_invoices.sql` (12-2) | `company_invoice_settings.default_payable_account_id` → `NULL` | **exposée** |
| `20260722000001_accounts_role_postable.sql` (14-3a) | `accounts.role` → `NULL` ; `accounts.postable` → `TRUE` par `DEFAULT` | **exposée** |
| `20260729000001_…_revenue_account_backfill.sql` (16-1a-bis) | `invoice_lines.revenue_account_id` et `credit_note_lines.revenue_account_id` → `NULL` | **exposée** — l'objet de l'issue |
| `20260419000002_users_company_id.sql` | `users.company_id` est `NOT NULL` **sans défaut** | **protégée** — `is_required()` vrai, `check_schema_compat` refuse le backup (400) |
| `20260714000002_email_templates_reminder.sql` | l'`UPDATE` est le bump `kesh_version_min_required`, pas un backfill | **hors sujet** — `_kesh_version` n'est de toute façon jamais restaurée |

La perte de `accounts.role` est la plus lourde : elle casse la présentation des fonds propres par rôle (Epic 14) **et** la condition (4) du backfill 16-1a-bis lui-même, qui exige `accounts.postable = TRUE`.

Aucune migration du dépôt ne porte d'`INSERT … SELECT` (vérifié : `grep -ln "INSERT INTO.*SELECT" crates/kesh-db/migrations/*.sql` → vide). Le détecteur du garde-fou (D-C6) n'a donc à couvrir que l'`UPDATE` **aujourd'hui**, mais doit être écrit pour ne pas devenir faux si ce cas apparaît.

### Ce qui n'est PAS dans cette story

- **Toute modification d'un fichier de migration existant.** Les checksums SHA-384 de sqlx rendent les `.sql` déjà appliqués **immuables** (cf. l'avertissement en tête de `docs/migrations-idempotence-audit.md`). Le rejeu se fait à côté, jamais en réécrivant l'histoire.
- **Le frontend.** Cf. D-C7 — le rapport de rejeu ne remonte pas au corps de réponse HTTP.
- **La réparation d'un parc déjà restauré avant cette version.** Un exploitant dans ce cas relance l'import du même backup une fois à jour ; le rejeu s'appliquera alors. À dire au CHANGELOG, pas à outiller.
- **Le durcissement de `check_schema_compat`** (refuser un backup dont il manque une colonne « sémantique »). Écarté : cela transformerait un cas rattrapable en refus d'import, alors que restaurer un backup ancien est le cas nominal.

---

## Décisions de conception

### D-C1 — Le rejeu est CONDITIONNÉ à l'absence de la colonne dans le manifeste source

**C'est la décision structurante de la story.**

Rejouer verbatim n'est **pas** sûr en général. Sur les 4 migrations exposées, seuls trois `UPDATE` sur seize portent une garde `IS NULL` :

| Backfill | Garde | Rejeu inconditionnel sur un backup **récent** |
|---|---|---|
| `20260729000001`, les 2 `UPDATE` | `revenue_account_id IS NULL` | no-op strict |
| `20260628000001`, l'`UPDATE` | `default_payable_account_id IS NULL` | no-op strict |
| `20260722000001`, les 10 `UPDATE` de rôle | `role IS NULL AND active = TRUE` | no-op strict |
| `20260722000001`, les 2 `UPDATE` de `postable` | **aucune** | **écrase** un `postable` posé à la main |
| `20260613000001`, l'`UPDATE … CASE` | **aucune** (le `WHERE label IN (…)` n'est pas une garde) | **écrase** une catégorie choisie par l'utilisateur |

Or `role` et `postable` sont **éditables** : `PUT /api/v1/accounts/{id}` a une sémantique **full-replace** et les exige tous deux (`routes/accounts.rs:64-69`, `:271`). `category` l'est également (CRUD admin des taux, Story 11-1). Un rejeu inconditionnel remplacerait donc une perte de donnée par une autre.

**Décision** : chaque entrée du registre porte une ou plusieurs **colonnes sentinelles** `(table, colonne)`. Le backfill n'est rejoué que si **au moins une** sentinelle est **absente des `columnNames` du manifeste source** — information déjà disponible à l'import via `parsed.tables[table].column_names` (`admin_backup/import.rs:28-33`).

- Sentinelle présente → le backup portait la colonne, **sa donnée fait foi** → **skip strict**. Coût nul, risque nul.
- Sentinelle absente → le backup **précède** la migration ; il n'existe **aucune intention utilisateur** à écraser sur cette colonne → **rejeu intégral**, y compris des statements non gardés.

Une table entièrement absente du manifeste compte comme sentinelle absente (choix conservateur ; en flux réel `parse_and_verify` garantit la couverture).

**Ce que cette décision achète** : c'est le seul critère qui rend 14-3a et 11-1 rattrapables. Un critère « on ne rejoue que les backfills gardés `IS NULL` » aurait été plus simple à raisonner mais aurait laissé les rôles de comptes cassés après restore — c'est-à-dire l'exposition la plus grave des quatre.

### D-C2 — Le rejeu s'exécute par ORDRE DE VERSION CROISSANTE — c'est un invariant, pas un détail de style

Le rejeu doit reproduire **exactement** ce qu'aurait fait une montée de version depuis le binaire source. Les backfills ne sont pas indépendants :

- `20260722000001` pose `accounts.postable` ;
- la condition (4) de `20260729000001` **exige** `accounts.postable = TRUE` sur le compte candidat (`INNER JOIN accounts a ON … a.postable = TRUE`).

Rejouer `20260729000001` **avant** `20260722000001` le ferait tourner sur une table où `postable` vaut `TRUE` partout par défaut, donc sur un ensemble de candidats **plus large** que celui qu'aurait vu une montée de version réelle — et écrirait des comptes de regroupement dans `invoice_lines`, exactement ce que la condition (4) existe pour interdire.

**Décision** : le registre est **ordonné par version croissante**, l'itération suit cet ordre, et un test le verrouille (`versions strictement croissantes`). Un `HashMap` ou un ordre d'écriture « logique » est interdit.

### D-C3 — Le SQL rejoué est celui du dépôt, jamais une paraphrase

Deux formes selon la migration :

- **`20260729000001` est du backfill pur** (deux `UPDATE`, aucun DDL — c'est la seule migration du dépôt classée `yes` à l'audit d'idempotence pour cette raison). Son fichier est rejouable **en entier** : `include_str!("../migrations/20260729000001_invoice_lines_revenue_account_backfill.sql")`. **Zéro duplication**, et un renommage du fichier casse la **compilation** plutôt que de dégrader en échec runtime.
- **Les trois autres mêlent DDL et données.** Les rejouer en bloc échouerait (`1060 duplicate column` dès le premier `ALTER`). Elles exigent un **extrait** : un `.sql` dédié ne contenant que les statements de backfill, embarqué par `include_str!`.

**Les trois extraits sont des copies VERBATIM** — vérifié statement par statement à la rédaction de cette spec, aucune adaptation n'est nécessaire (c'est le conditionnement de D-C1 qui rend sûrs les statements non gardés, pas une réécriture). Un test asserte que chaque statement de l'extrait est un **sous-texte de la migration source** telle qu'embarquée dans le `MIGRATOR`. Les migrations étant immuables, ce test ne protège pas d'une dérive future mais d'une **erreur de copie à l'écriture** — c'est là qu'est le risque réel.

**Ne PAS** réécrire, reformater, ni « améliorer » un statement en le copiant. Un `<>` introduit à la place d'un `<=>` en recopiant reproduirait le piège D-B3 de 16-1a-bis, indiscernable du succès.

### D-C4 — Le rejeu vit dans la transaction de restore, entre le restore et l'audit

**Emplacement** : nouveau module `crates/kesh-db/src/post_restore.rs`, exposant le registre et une fonction

```rust
pub async fn replay_post_restore_backfills(
    tx: &mut Transaction<'_, MySql>,
    tables: &BTreeMap<String, TableRestore>,
) -> Result<Vec<ReplayedBackfill>, DbError>
```

appelée depuis `run_backup_and_restore` (`routes/admin.rs:264-267`) **immédiatement après** `restore_tables_in_tx` et **avant** l'insertion de l'audit.

**Pourquoi dans la même transaction** : un échec de rejeu doit annuler le restore entier. Un restore committé suivi d'un rejeu échoué laisserait précisément l'état que la story existe pour empêcher, en plus difficile à diagnostiquer.

**Pourquoi une fonction séparée et non dans `restore_tables_in_tx`** : cette dernière rétablit `FOREIGN_KEY_CHECKS = 1` en sortie, systématiquement (`backup.rs:403-416`). Le rejeu doit tourner **FK actives** — ses `UPDATE` posent des FK (`revenue_account_id → accounts.id`, `default_payable_account_id → accounts.id`) et doivent être contrôlés. L'inclure dans le corps du restore le ferait tourner sous `FK = 0`.

**Pourquoi `tables` et non `parsed`** : le module vit dans `kesh-db`, qui ne connaît pas `ParsedBackup` (type `kesh-api`). `TableRestore.column_names` porte toute l'information nécessaire et le crate reste sans dépendance montante.

### D-C5 — Le rapport de rejeu : logs + audit, rien d'autre

Chaque rejeu produit un `ReplayedBackfill { version: i64, label: &'static str, missing_sentinels: Vec<String>, rows_affected: u64 }`.

- **`tracing::info!` par backfill rejoué**, et un `tracing::debug!` par backfill sauté — l'exploitant doit pouvoir lire dans le journal serveur *pourquoi* un rejeu a eu lieu (quelle colonne manquait) et *ce qu'il a touché*.
- **`audit_log`** : le détail JSON de l'entrée `admin.full_import` existante (`routes/admin.rs:298-304`) reçoit une clé supplémentaire `backfills_replayed`. Pas de nouvelle action d'audit, pas de nouvelle table.

**`rows_affected` est informatif et ne fonde AUCUNE assertion de succès.** Zéro ligne touchée est un résultat parfaitement normal : le backfill 16-1a-bis est délibérément incomplet (son AC-B2), et une base restaurée peut n'avoir aucune ligne éligible. Ne **pas** écrire de garde « le rejeu doit avoir touché au moins une ligne » — ce serait la post-condition fausse contre laquelle 16-1a-bis met explicitement en garde.

### D-C6 — Le garde-fou fail-loud : toute future migration portant un backfill DOIT être triée

Sans cela, le registre redérivera au fil des Epics — exactement comme le compteur `tracked-by-sqlx` de `docs/migrations-idempotence-audit.md`, qui avait accumulé **7 de dérive** sur les Epics 20-21 et a survécu à sept passes adversariales.

**Décision** : un test de `kesh-db` parcourt `MIGRATOR.migrations` et exige que **toute** migration dont le SQL contient un statement de backfill de données figure soit dans le **registre**, soit dans une **liste d'exemption explicite** portant une justification écrite. Une migration nouvelle non triée fait **échouer le test**, avec un message qui nomme le fichier et les deux issues possibles.

**Détection** — sur le SQL du `MIGRATOR`, après retrait des commentaires (`--` en fin de ligne comme en ligne entière ; aucune migration du dépôt n'utilise `/* */`) : un statement dont le premier mot-clé est `UPDATE` ou qui est un `INSERT INTO … SELECT`. Le second cas n'existe pas aujourd'hui, mais l'omettre rendrait le garde-fou faux le jour où il apparaîtra.

**Le retrait des commentaires n'est pas cosmétique** : les migrations du dépôt sont très commentées et plusieurs commentaires contiennent le mot `UPDATE` en prose (`20260722000001` : « Les douze UPDATE de… », `20260628000001` : `ON UPDATE CURRENT_TIMESTAMP(3)` dans une définition de colonne). Un détecteur naïf sur la sous-chaîne `UPDATE` produirait des faux positifs — et la réaction naturelle serait de les exempter, ce qui viderait le garde-fou de son sens. Attention en particulier à `ON UPDATE CURRENT_TIMESTAMP`, qui est du **DDL** et n'est pas un statement `UPDATE`.

**Les deux exemptions à écrire d'emblée** :

| Migration | Justification d'exemption |
|---|---|
| `20260419000002_users_company_id.sql` | `users.company_id` finit `NOT NULL` sans défaut ⇒ `is_required()` vrai ⇒ un backup qui ne la porte pas est **refusé** par `check_schema_compat` (400). Le cas ne peut pas se produire. |
| `20260714000002_email_templates_reminder.sql` | L'`UPDATE` porte sur `_kesh_version`, table système **jamais restaurée** (absente de `TABLES_TO_TRUNCATE`). Ce n'est pas un backfill de données applicatives. |

### D-C7 — Le corps de réponse HTTP de `full-import` reste INCHANGÉ

Le corps `{ backupCreated, tablesRestored, rowsRestored, sourceVersion, sessionInvalidated }` (`routes/admin.rs:184-190`) n'est **pas** étendu.

**Motif** : le frontend consomme ce contrat (`frontend/src/lib/features/admin-restore/admin-restore.api.ts:12-14`, `AdminRestorePanel`). Ajouter un champ *utile* impliquerait de l'afficher, donc d'ouvrir le frontend, ses tests unitaires et ses quatre locales — pour une information de diagnostic que l'exploitant d'un restore lit dans le journal serveur, opération qu'il conduit de toute façon depuis la machine. Le rapport est donc **loggé et audité** (D-C5), pas retourné.

**Conséquence assumée** : un administrateur qui n'a pas accès aux logs ne voit pas que des backfills ont été rejoués. Acceptable pour v0.9.0 ; si l'exposition en UI est souhaitée, elle relève d'un CR distinct.

### D-C8 — Ce que le rejeu ne prétend PAS être

Le rejeu **n'est pas** un mécanisme général de « migration de données à l'import ». Il ne rattrape qu'un cas précis : *le backup précède une migration qui a rempli une colonne*. Il ne détecte ni ne corrige :

- une colonne remplie par du **code applicatif** et non par une migration ;
- une donnée dont la sémantique a **changé** sans changement de colonne ;
- un backup **plus récent** que le binaire (refusé en amont, 409).

À écrire dans le doc-comment du module. Un mainteneur qui croirait le mécanisme plus général qu'il ne l'est y verserait des rattrapages qui n'y ont pas leur place.

---

## Acceptance Criteria

- **AC-C1 — Registre** : `crates/kesh-db/src/post_restore.rs` expose un registre **ordonné par version croissante** des backfills à rejouer, contenant les **4** entrées du tableau du § Contexte, chacune avec ses colonnes sentinelles :

  | Version | Sentinelles `(table, colonne)` | Source du SQL |
  |---|---|---|
  | `20260613000001` | `(vat_rates, category)` | extrait |
  | `20260628000001` | `(company_invoice_settings, default_payable_account_id)` | extrait |
  | `20260722000001` | `(accounts, role)`, `(accounts, postable)` | extrait |
  | `20260729000001` | `(invoice_lines, revenue_account_id)`, `(credit_note_lines, revenue_account_id)` | migration **entière** via `include_str!` |

- **AC-C2 — Déclencheur conditionnel** : un backfill est rejoué **si et seulement si** au moins une de ses sentinelles est absente des `column_names` de la table correspondante dans le manifeste source (table absente = sentinelle absente). Sinon, **skip strict** : aucun statement n'est envoyé à la base.
- **AC-C3 — Ordre** : le rejeu suit l'ordre **croissant des versions**. Un test asserte que le registre est strictement croissant et échoue si une entrée est insérée au mauvais endroit.
- **AC-C4 — Transactionnalité** : le rejeu s'exécute dans la **transaction de restore**, après `restore_tables_in_tx` (donc `FOREIGN_KEY_CHECKS = 1`) et avant l'audit. Toute erreur d'un statement remonte en `AppError::AdminFullImportFailed` et **annule le restore entier**.
- **AC-C5 — Fidélité du SQL** : chaque statement d'extrait est un **sous-texte verbatim** de la migration source telle qu'embarquée dans `MIGRATOR`, vérifié par test. L'entrée `20260729000001` est le fichier de migration **lui-même**.
- **AC-C6 — Garde-fou de triage** : un test échoue si une migration du `MIGRATOR` porte un statement de backfill de données (`UPDATE`, ou `INSERT … SELECT`) sans figurer ni au registre ni à la liste d'exemption justifiée. Le message d'échec nomme le fichier et énonce les deux issues.
- **AC-C7 — Restitution** : chaque rejeu émet un `tracing::info!` portant la version, les sentinelles manquantes et le nombre de lignes touchées ; chaque skip émet un `tracing::debug!`. Le détail JSON de l'audit `admin.full_import` porte une clé `backfills_replayed`.
- **AC-C8 — Contrat HTTP inchangé** : le corps de réponse de `POST /api/v1/admin/full-import` conserve exactement ses cinq clés. Aucun fichier de `frontend/` n'est modifié par cette story.
- **AC-C9 — Aucune migration nouvelle, aucune migration modifiée.** Donc aucun bump de `kesh_version_min_required` ni de version Cargo (P1/P2/P2-bis), et **aucune ligne ajoutée** à `docs/migrations-idempotence-audit.md` — dont les compteurs restent à **57 / 4 / 53 / 0**. *(Le garde-fou P5 impose une ligne d'audit par migration **ajoutée** ; cette story n'en ajoute aucune. Ne pas « mettre à jour les compteurs par précaution » : ce serait les casser.)*

### AC-C10 — Post-conditions testées de bout en bout

Le test de bout en bout est **indispensable et non substituable** par des tests unitaires : le défaut naît de l'**interaction** entre l'import et des migrations que la PR ne touche pas.

Montage : construire un `.keshbackup` en mémoire (patron `build_test_backup`, `admin_full_import_e2e.rs:246`) dont les `columnNames` **omettent** les colonnes cibles, l'importer sur une base à jour, puis observer.

| Cas | Attendu |
|---|---|
| **C1** — backup sans `invoice_lines.revenue_account_id`, données de facture validée canoniques | après import, les lignes portent le compte crédité par leur écriture. **C'est le cas nominal de l'issue #281.** |
| **C2** — backup sans `accounts.role` / `postable` | après import, les rôles sont réattribués et `postable` recalculé |
| **C3** — backup **portant** toutes les colonnes, avec un `accounts.role` **délibérément non standard** (rôle posé à la main sur un numéro inattendu, `postable` forcé) | après import, la donnée du backup est **intacte** — le rejeu n'a pas tourné. **C'est le test qui discrimine D-C1** : sans le conditionnement, il tombe. |
| **C4** — échec injecté pendant le rejeu | la transaction est annulée : la base destination est **inchangée**, y compris les tables restaurées avant l'échec |
| **C5** — backup sans `accounts.role` **et** sans `invoice_lines.revenue_account_id`, monté de sorte que le candidat du backfill 16-1a-bis soit un compte que 14-3a rend **non imputable** | la ligne reste `NULL`. **C'est le test qui discrimine D-C2** : inverser l'ordre du registre le fait tomber, en écrivant le compte de regroupement. |

**Sur C3 et C5, une note de montage explicite est exigée dans le test** (§ « Ce que ce test discrimine ») : ce sont les deux seuls tests qui tombent si l'une des deux décisions structurantes est mal implémentée, et un montage qui se décale les rendrait muets — le mode d'échec exact subi par `backfill_skips_archived_accounts` en 16-1a.

---

## Tasks / Subtasks

- [ ] **T1 — Registre et mécanique de rejeu** (AC-C1, AC-C2, AC-C3)
  - [ ] Créer `crates/kesh-db/src/post_restore.rs` ; le déclarer dans `lib.rs` (ordre alphabétique des `pub mod`).
  - [ ] Doc-comment de module : le mécanisme, le déclencheur D-C1, l'invariant d'ordre D-C2, et **ce que le rejeu n'est pas** (D-C8).
  - [ ] Types : `PostRestoreBackfill { version, label, sentinels: &[(&str, &str)], sql: &'static str }`, `ReplayedBackfill`.
  - [ ] Registre `POST_RESTORE_BACKFILLS: &[PostRestoreBackfill]`, 4 entrées, **triées par version croissante**.
  - [ ] `replay_post_restore_backfills(tx, tables)` : filtrage par sentinelle, exécution `sqlx::raw_sql`, collecte du rapport.
- [ ] **T2 — Extraits SQL** (AC-C5)
  - [ ] `crates/kesh-db/src/post_restore/20260613000001_vat_rates_category.sql` — l'`UPDATE … CASE`, copié verbatim.
  - [ ] `…/20260628000001_default_payable_account.sql` — l'`UPDATE … INNER JOIN accounts`, verbatim.
  - [ ] `…/20260722000001_accounts_role_postable.sql` — les **12** `UPDATE` (10 rôles + 2 `postable`), verbatim, **dans l'ordre du fichier source**.
  - [ ] En-tête de chaque extrait : de quelle migration il provient, pourquoi un extrait plutôt que le fichier entier (DDL), et l'interdiction de le reformater.
  - [ ] `20260729000001` : `include_str!` de la migration, **aucun extrait**.
- [ ] **T3 — Câblage dans le restore** (AC-C4, AC-C7)
  - [ ] Appeler `replay_post_restore_backfills` dans `run_backup_and_restore`, entre le restore et l'audit ; mapper l'erreur en `AppError::AdminFullImportFailed`.
  - [ ] `tracing::info!` / `debug!` par entrée ; clé `backfills_replayed` dans le détail de l'audit.
  - [ ] Vérifier que le corps de réponse HTTP est **inchangé** (AC-C8).
- [ ] **T4 — Garde-fou de triage** (AC-C6)
  - [ ] Fonction de retrait des commentaires SQL + découpage en statements (unitairement testée sur les pièges réels : `ON UPDATE CURRENT_TIMESTAMP`, le mot `UPDATE` en prose de commentaire).
  - [ ] Liste d'exemption avec justification écrite (les 2 entrées de D-C6).
  - [ ] Test `every_data_backfill_migration_is_triaged` avec message d'échec actionnable.
  - [ ] Test `registry_versions_are_strictly_increasing` (AC-C3).
  - [ ] Test `extract_statements_are_verbatim_substrings_of_source_migration` (AC-C5).
- [ ] **T5 — Tests de bout en bout** (AC-C10)
  - [ ] Les 5 cas C1-C5 dans `crates/kesh-api/tests/admin_full_import_e2e.rs` (réutiliser `build_test_backup` / `spawn_app` — ne pas dupliquer le harnais).
  - [ ] Note de montage « ce que ce test discrimine » sur C3 et C5.
- [ ] **T6 — Preuve par mutation** (geste 16-1a-bis, à reproduire)
  - [ ] Muter le conditionnement (rejeu inconditionnel) → **C3 seul** doit rougir. Muter l'ordre du registre (décroissant) → **C5 seul** doit rougir. Consigner les deux résultats dans le Dev Agent Record. Si un test attendu **ne** rougit pas, le montage est muet : le corriger avant d'aller plus loin.
- [ ] **T7 — Documentation** (AC-C9)
  - [ ] **CHANGELOG** : amender le paragraphe de diagnostic de 16-1a-bis. Il dit aujourd'hui *« si vous avez restauré une sauvegarde antérieure à cette version, le chiffre remonté n'a pas la cause annoncée ici […] suivi dans l'issue #281 »* — ce n'est plus vrai. Le remplacer par la mention que la reprise **se rejoue** après un import.
  - [ ] `docs/manual/fr/admin-manual.tex` — section restore : mentionner le rejeu et l'ordre. Régénérer le PDF (`make fr` dans `docs/manual/`) et le commiter. **Ne PAS** toucher les macros de version (`kesh-style.sty`) — gate 4-bis, réservé au tag de release.
  - [ ] `CLAUDE.md` — garde-fou **P7** sous § « Migration breaking policy » : toute PR ajoutant une migration porteuse d'un backfill de données doit la trier (registre ou exemption justifiée) ; un manquement est un finding **MEDIUM** en `bmad-code-review`. Renvoyer au test qui l'outille.
  - [ ] **Ne PAS** ajouter de ligne ni toucher aux compteurs de `docs/migrations-idempotence-audit.md` (AC-C9).
- [ ] **T8 — Gate** : `scripts/test-fast.sh` complet (fmt + clippy `-D warnings` + nextest workspace) sur l'**état final**, exit 0 exigé, non présumé d'un run antérieur. `npm run check` inutile — aucun fichier frontend touché.

---

## Dev Notes

### Fichiers à lire AVANT d'écrire quoi que ce soit

| Fichier | Ce qu'il faut en retenir |
|---|---|
| `crates/kesh-db/src/backup.rs:375-486` | `restore_tables_in_tx` : `FOREIGN_KEY_CHECKS` posé à 0 puis **rétabli à 1 systématiquement**, y compris sur erreur. Le rejeu tourne donc FK actives — voulu (D-C4). |
| `crates/kesh-api/src/routes/admin.rs:221-328` | `run_backup_and_restore` : verrou `_kesh_version FOR UPDATE` → backup pré-import → restore → **garde de cohérence de comptage** → audit → `force_onboarding_done_if_eligible` → commit. Le point d'insertion est juste après le restore. |
| `crates/kesh-api/src/admin_backup/import.rs:28-33`, `:195-236` | `ParsedBackup.tables` et `check_schema_compat`. Les `column_names` proviennent du **manifeste**. |
| `crates/kesh-db/tests/invoice_lines_revenue_account_backfill.rs:1128-1140` | Le patron `sqlx::raw_sql(&sql)` sur le SQL réellement embarqué. Le rejeu utilise le même mécanisme. |
| `crates/kesh-api/tests/admin_full_import_e2e.rs:246+` | `build_test_backup(format_version_override, tables, include_all_required, files_extra)` — c'est lui qui permet d'omettre des colonnes du manifeste. |
| `crates/kesh-db/tests/common/mod.rs` | Résolution **par version** des fenêtres de migration (garde-fou P6). À ne pas confondre avec le registre de cette story, qui n'applique pas de migrations. |

### La garde de comptage du restore ne couvre PAS le rejeu

`run_backup_and_restore` vérifie `rows_restored == expected_rows` (`admin.rs:272-282`). Cette garde porte sur les `INSERT` du restore et **n'a aucun rapport** avec les `UPDATE` du rejeu. Ne pas chercher à y intégrer `rows_affected`, ne pas s'en inspirer pour écrire une garde symétrique sur le rejeu — cf. D-C5, zéro ligne touchée est normal.

### Pièges de recopie des extraits

- `20260613000001` : le `CASE … ELSE category END` doit être recopié **entier**. Le tronquer à ses quatre `WHEN` changerait la sémantique des lignes hors liste.
- `20260722000001` : **12** statements, pas 10. Les deux derniers portent sur `postable` et sont ceux qui rendent le conditionnement de D-C1 indispensable ; les omettre viderait le cas C2 de sa moitié la plus consultée (`postable` gouverne l'imputabilité).
- `20260722000001`, backfill `postable` #1 : l'`EXISTS` corrélé porte sur la table cible `accounts`. MariaDB l'autorise dans un `UPDATE` (vérifié 10.11, commentaire du fichier source) — ne pas « corriger » en table dérivée en croyant contourner l'ER 1093.
- `20260628000001` : `UPDATE … INNER JOIN accounts a ON a.company_id = cis.company_id AND a.number = '2000'` — multi-société par construction ; ne pas ajouter de filtre.

### Ce qui rend ce défaut invisible en revue de diff

Le mode d'échec ne naît ni du code écrit ni de la spec, mais de l'**interaction** entre une migration ajoutée par une story et un chemin de restore que cette story ne touche pas. C'est le même profil que le garde-fou **P6** (couplage positionnel des migrations), codifié en 16-1a après que trois tests ont changé de sens sans qu'aucune ligne de leur fichier ne bouge — dont un **passé à vide**. D'où l'exigence T6 : prouver par **mutation** que C3 et C5 discriminent, plutôt que de constater qu'ils sont verts.

### Project Structure Notes

- Modules touchés : `kesh-db` (nouveau module + tests), `kesh-api` (2 lignes de câblage + tests E2E), `docs/` + `CHANGELOG.md` + `CLAUDE.md`. **3 modules** — sous le seuil de 5 de la règle de splitting préventif.
- Le sous-répertoire `crates/kesh-db/src/post_restore/` ne contient que des `.sql` embarqués par `include_str!`. Il ne doit **pas** être confondu avec `crates/kesh-db/migrations/` : rien de ce qu'il contient n'est jamais vu par `sqlx::migrate!`. Le dire dans le doc-comment du module.
- Aucun fichier `frontend/`. Aucune migration.

### References

- Issue **#281** — [Restaurer un backup antérieur au backfill 16-1a-bis rouvre le bug définitivement](https://github.com/guycorbaz/kesh/issues/281), et son commentaire d'arbitrage du 2026-08-01 (D-1 / D-2 / D-3).
- Story **16-1a-bis** — `_bmad-output/implementation-artifacts/16-1a-bis-backfill-parc-existant.md`, décisions **D-B1** (source de vérité = l'écriture), **D-B2** (critère d'unicité), **D-B3** (`<=>` NULL-safe), **D-B5** (portée), **D-B6** (idempotence intrinsèque), **D-B7** (le backfill enregistre, il ne répare pas).
- Story **17-3c** — import transactionnel : `crates/kesh-api/src/admin_backup/import.rs`, `crates/kesh-api/src/routes/admin.rs`.
- `CLAUDE.md` § « Migration breaking policy » (P1-P6), § « Tech debt management » (triage hors rétrospective), § « Review Iteration Rule » (§ Propagation post-patch).
- `docs/migrations-idempotence-audit.md` — verdicts et invariant des compteurs (57 / 4 / 53 / 0).

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log
