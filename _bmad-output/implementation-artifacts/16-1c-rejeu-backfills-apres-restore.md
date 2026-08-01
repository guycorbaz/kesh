# Story 16.1c : Rejeu des backfills de données après un restore d'installation

## Status

ready-for-dev

## Story

**As a** exploitant d'une instance Kesh qui restaure un `.keshbackup` produit par une version antérieure,
**I want** que les **backfills de données** portés par les migrations postérieures à ce backup soient **rejoués à la fin du restore**,
**so that** l'import ne réintroduise pas silencieusement, et **définitivement**, les bugs que ces migrations avaient fermés — le rôle des comptes, les comptes de TVA, le compte créanciers par défaut, la catégorie des taux de TVA et le compte de produit par ligne disparaissant tous sans qu'aucun message ne le signale.

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

   **Et le contrôle ne porte que sur les colonnes.** Une migration qui a inséré des **lignes** manquantes ne laisse aucune trace détectable par ce chemin — cf. `20260614000001` ci-dessous.

3. **`_sqlx_migrations` n'est pas restaurée**, donc la migration reste marquée appliquée et ne repassera jamais. **Ancre exacte** : ce n'est pas la clause `AND TABLE_NAME NOT IN ('_sqlx_migrations', '_kesh_version')` de `backup.rs:586` — celle-ci est dans le **test** `backup_inventory_matches_schema`, qui ne fait que **documenter** l'invariant. La cause réelle est que `TABLES_TO_TRUNCATE` (`backup.rs:34-72`) ne **contient pas** ces deux tables, et que l'export (`admin_backup/export.rs:55`) comme le restore (`backup.rs:425`) n'énumèrent que cette constante. *(L'issue cite `:586` ; suivre l'ancre sans borner la fonction englobante mènerait à patcher un test — c'est le mode d'échec relevé en rétro 16-1a.)*

Le `DELETE` + `INSERT` du restore ne cite que les `columnNames` du manifeste (`backup.rs:468`) : toute colonne absente de la source prend `NULL` ou son `DEFAULT`, en silence.

### Le périmètre réel : 5 migrations exposées sur 7 porteuses d'un backfill

**Recompte à la source** (ne pas relire ces nombres, les recompter — cf. § « Décomptes » des Dev Notes) :

```sh
grep -c "^\s*UPDATE\b" crates/kesh-db/migrations/*.sql            # 6 fichiers, 18 statements
perl -0777 -ne 'print "$ARGV\n" if /INSERT\s+INTO[^;]*?\bSELECT\b/si' crates/kesh-db/migrations/*.sql
```

⚠️ **Le second grep DOIT être multi-ligne.** Un `grep "INSERT INTO.*SELECT"` mono-ligne rend **vide** et rate `20260614000001`, dont le `SELECT` est sur la ligne suivante. *(C'est l'erreur qu'a commise la première rédaction de cette spec, corrigée en passe 1 de `validate` — finding BH-1 CRITICAL.)*

| Migration | Ce que le restore d'un backup antérieur perd | Verdict |
|---|---|---|
| `20260613000001_vat_rates_crud.sql` (11-1) | `vat_rates.category` → `'custom'` par `DEFAULT` | **exposée** |
| `20260614000001_vat_accounts_config.sql` (18-1a) | les **lignes** `accounts` 1171 « Impôt préalable » et 2206 « Décompte TVA » de chaque société | **exposée** |
| `20260628000001_supplier_invoices.sql` (12-2) | `company_invoice_settings.default_payable_account_id` → `NULL` | **exposée** |
| `20260722000001_accounts_role_postable.sql` (14-3a) | `accounts.role` → `NULL` ; `accounts.postable` → `TRUE` par `DEFAULT` | **exposée** |
| `20260729000001_…_revenue_account_backfill.sql` (16-1a-bis) | `invoice_lines.revenue_account_id` et `credit_note_lines.revenue_account_id` → `NULL` | **exposée** — l'objet de l'issue |
| `20260419000002_users_company_id.sql` | `users.company_id` est `NOT NULL` **sans défaut** | **protégée** — `is_required()` vrai, `check_schema_compat` refuse le backup (400) |
| `20260714000002_email_templates_reminder.sql` | l'`UPDATE` porte sur `_kesh_version`, table système jamais restaurée | **hors sujet** |

La perte de `accounts.role` est la plus lourde : elle casse la présentation des fonds propres par rôle (Epic 14) **et** la condition (4) du backfill 16-1a-bis lui-même, qui exige `accounts.postable = TRUE`.

### Ce qui n'est PAS dans cette story

- **Toute modification d'un fichier de migration existant.** Les checksums SHA-384 de sqlx rendent les `.sql` déjà appliqués **immuables** (cf. l'avertissement en tête de `docs/migrations-idempotence-audit.md`). Le rejeu se fait à côté, jamais en réécrivant l'histoire.
- **Le frontend.** Cf. D-C7 — le rapport de rejeu ne remonte pas au corps de réponse HTTP.
- **La réparation d'un parc déjà restauré avant cette version.** Un exploitant dans ce cas relance l'import du même backup une fois à jour ; le rejeu s'appliquera alors. À dire au CHANGELOG, pas à outiller.
- **Le durcissement de `check_schema_compat`** (refuser un backup dont il manque une colonne « sémantique »). Écarté : cela transformerait un cas rattrapable en refus d'import, alors que restaurer un backup ancien est le cas nominal.

---

## Décisions de conception

### D-C1 — DEUX CLASSES d'entrées, et le déclencheur n'est pas le même

**C'est la décision structurante de la story, et c'est celle que la passe 1 de `validate` a fait refondre** (findings BH-1 et ECH-1, tous deux CRITICAL).

#### Le point de départ : rejouer verbatim n'est pas sûr partout

Sur les **18** statements de backfill des 5 migrations exposées, **3 seulement** sont dépourvus de garde contre l'écrasement d'une valeur posée par l'utilisateur — mais ce sont eux qui interdisent un rejeu inconditionnel généralisé :

| Statement | Garde contre l'intention utilisateur | Rejeu inconditionnel sur un backup **récent** |
|---|---|---|
| `20260729000001`, les 2 `UPDATE` | `revenue_account_id IS NULL` | no-op strict |
| `20260628000001`, l'`UPDATE` | `default_payable_account_id IS NULL` | no-op strict |
| `20260722000001`, les 10 `UPDATE` de rôle | `role IS NULL AND active = TRUE` | no-op strict |
| `20260614000001`, les 2 `INSERT … SELECT` | `NOT EXISTS (… number = '1171' \| '2206')` | no-op strict — **voir la justification ci-dessous, elle est portante** |
| `20260722000001`, les 2 `UPDATE` de `postable` | **aucune** | **écrase** un `postable` posé à la main |
| `20260613000001`, l'`UPDATE … CASE` | **aucune** | **écrase** une catégorie choisie par l'utilisateur |

⚠️ **Le backfill `postable` #1 contient un `NOT EXISTS` et n'est pourtant PAS gardé.** Son `NOT EXISTS (… journal_entry_lines …)` est un **prédicat structurel** qui décrit quels comptes viser, pas une garde d'idempotence : rejoué, il repose `postable = FALSE` sur un compte que l'utilisateur venait de rendre imputable. **Conséquence directe pour l'implémentation** : un test qui classerait les entrées d'après la présence textuelle de `IS NULL` / `NOT EXISTS` se tromperait sur ce statement précis. La classe est **déclarée explicitement par entrée**, jamais devinée — cf. AC-C6.

Or `role`, `postable` et `category` sont **éditables** : `PUT /api/v1/accounts/{id}` a une sémantique **full-replace** et exige les deux premiers (`routes/accounts.rs:64-69`, `:271`) ; `category` l'est par le CRUD admin des taux (Story 11-1).

#### Classe A — auto-gardée : rejeu INCONDITIONNEL

Une entrée dont **tous** les statements sont gardés contre l'intention utilisateur est rejouée **systématiquement**, sans condition. Le rejeu est un no-op strict sur une base à jour, donc il n'y a rien à conditionner.

| Entrée | Justification écrite de l'appartenance à la classe A |
|---|---|
| `20260614000001` | `NOT EXISTS (… accounts.number = '1171' \| '2206')`. **L'absence de ces lignes ne peut pas être une intention utilisateur** : il n'existe aucune route de suppression individuelle de compte — vérifié, `crates/kesh-api/src/lib.rs` n'expose que `POST /accounts`, `PUT /accounts/{id}`, `PUT …/archive`, `PUT …/reactivate`, et l'archivage **conserve la ligne** (`active = FALSE`), donc `NOT EXISTS` reste faux. Le seul `DELETE FROM accounts` du dépôt est `delete_all_by_company` (`repositories/accounts.rs:903`), un remplacement de plan comptable entier. |
| `20260628000001` | `default_payable_account_id IS NULL` — un `NULL` n'est l'expression d'aucun choix. |
| `20260729000001` | `revenue_account_id IS NULL` sur les deux `UPDATE`, critère déterministe, idempotence **prouvée par test** en 16-1a-bis (D-B6, `backfill_is_idempotent`). |

#### Classe B — sentinelle : rejeu CONDITIONNÉ à l'absence de la colonne

Une entrée contenant au moins un statement non gardé porte une ou plusieurs **colonnes sentinelles** `(table, colonne)`. Elle n'est rejouée que si **au moins une** sentinelle est **absente des `columnNames` du manifeste source** — information disponible via `parsed.tables[table].column_names` (`admin_backup/import.rs:28-33`).

- Sentinelle présente → le backup portait la colonne, **sa donnée fait foi** → **skip strict**.
- Sentinelle absente → le backup **précède** la migration ; il n'existe **aucune intention utilisateur** à écraser → **rejeu intégral**, statements non gardés compris.

| Entrée | Sentinelles |
|---|---|
| `20260613000001` | `(vat_rates, category)` |
| `20260722000001` | `(accounts, role)`, `(accounts, postable)` |

**La condition de validité de la classe B, et elle est stricte** : le raisonnement « colonne présente ⇒ backfill appliqué » n'est vrai **que si le DDL et le backfill sont dans le MÊME fichier de migration**, donc dans la même transaction. C'est le cas des deux entrées ci-dessus (`ADD COLUMN category` ligne 31 de `20260613000001` ; `ADD COLUMN role` ligne 75 et `ADD COLUMN postable` ligne 77 de `20260722000001`), et **un test le verrouille** (AC-C6).

C'est précisément ce qui **interdisait** de traiter `20260729000001` en classe B : sa colonne est créée par une **autre** migration (`20260727000001`, DDL pur). Un backup pris entre les deux porte la colonne, entièrement `NULL`, et une sentinelle l'aurait déclaré « à jour » — rouvrant le bug de l'issue par un second chemin. *(Finding ECH-1, CRITICAL.)*

#### Pourquoi pas la classe A partout

Parce que les 3 statements non gardés causeraient une perte de donnée réelle sur le cas nominal — restaurer un backup **récent**, qui est de loin le plus fréquent. Et pourquoi pas la classe B partout : parce qu'elle repose sur une coïncidence structurelle (DDL et données dans le même fichier) que le dépôt ne garantit pas, et qui est déjà fausse une fois.

### D-C2 — Le rejeu s'exécute par ORDRE DE VERSION CROISSANTE — c'est un invariant, pas un détail de style

Le rejeu doit reproduire **exactement** ce qu'aurait fait une montée de version depuis le binaire source. Les backfills ne sont pas indépendants :

- `20260722000001` pose `accounts.postable` ;
- la condition (4) de `20260729000001` **exige** `accounts.postable = TRUE` sur le compte candidat (`INNER JOIN accounts a ON … a.postable = TRUE`, ligne 255).

Rejouer `20260729000001` **avant** `20260722000001` le ferait tourner sur une table où `postable` vaut `TRUE` partout par défaut, donc sur un ensemble de candidats **plus large** que celui qu'aurait vu une montée de version réelle — et écrirait des comptes de regroupement dans `invoice_lines`, exactement ce que la condition (4) existe pour interdire.

**Décision** : le registre est **ordonné par version croissante**, l'itération suit cet ordre, et un test le verrouille. Un `HashMap` ou un ordre d'écriture « logique » est interdit. L'ordre est donc `20260613000001` → `20260614000001` → `20260628000001` → `20260722000001` → `20260729000001`, **classes mêlées** : la classe ne change rien à l'ordre.

### D-C3 — Le SQL rejoué est celui du dépôt, jamais une paraphrase

Deux formes selon la migration :

- **`20260729000001` est du backfill pur** (deux `UPDATE`, aucun DDL — c'est la seule migration du dépôt classée `yes` à l'audit d'idempotence pour cette raison). Son fichier est rejouable **en entier** : `include_str!("../migrations/20260729000001_invoice_lines_revenue_account_backfill.sql")`. **Zéro duplication**, et un renommage du fichier casse la **compilation** plutôt que de dégrader en échec runtime.
- **Les quatre autres mêlent DDL et données.** Les rejouer en bloc échouerait (`1060 duplicate column` ou `1050 table exists` dès le premier `ALTER`/`CREATE`). Elles exigent un **extrait** : un `.sql` dédié ne contenant que les statements de backfill, embarqué par `include_str!`.

**Les quatre extraits sont des copies VERBATIM.** Aucune adaptation n'est nécessaire : c'est la classe (D-C1) qui rend sûrs les statements non gardés, pas une réécriture. Un test asserte que chaque statement d'extrait est un **sous-texte de la migration source** telle qu'embarquée dans le `MIGRATOR`. Les migrations étant immuables, ce test ne protège pas d'une dérive future mais d'une **erreur de copie à l'écriture** — c'est là qu'est le risque réel.

**Ne PAS** réécrire, reformater, ni « améliorer » un statement en le copiant. Un `<>` introduit à la place d'un `<=>` en recopiant reproduirait le piège D-B3 de 16-1a-bis, indiscernable du succès.

### D-C4 — Le rejeu vit dans la transaction de restore, après la garde de comptage

**Emplacement** : nouveau module `crates/kesh-db/src/post_restore.rs`, exposant le registre et une fonction

```rust
pub async fn replay_post_restore_backfills(
    tx: &mut Transaction<'_, MySql>,
    tables: &BTreeMap<String, TableRestore>,
) -> Result<Vec<ReplayedBackfill>, DbError>
```

appelée depuis `run_backup_and_restore` (`routes/admin.rs`) **après la garde de cohérence de comptage** `rows_restored == expected_rows` (`:269-282`) et **avant** l'insertion de l'audit (`:298`).

**Pourquoi après la garde de comptage** : si le restore est déjà incohérent, la transaction va être annulée — rejouer des backfills par-dessus serait du travail perdu et brouillerait le diagnostic. *(Point laissé ambigu par la première rédaction, relevé par deux lentilles ; l'ordre inverse est sans conséquence fonctionnelle, mais la spec doit en désigner un.)*

**Pourquoi dans la même transaction** : un échec de rejeu doit annuler le restore entier. Un restore committé suivi d'un rejeu échoué laisserait précisément l'état que la story existe pour empêcher, en plus difficile à diagnostiquer.

**Pourquoi une fonction séparée et non dans `restore_tables_in_tx`** : cette dernière rétablit `FOREIGN_KEY_CHECKS = 1` en sortie, systématiquement (`backup.rs:403-416`). Le rejeu doit tourner **FK actives** — ses statements posent des FK (`revenue_account_id → accounts.id`, `default_payable_account_id → accounts.id`, `accounts.parent_id`) et doivent être contrôlés. L'inclure dans le corps du restore le ferait tourner sous `FK = 0`.

**Pourquoi `tables` et non `parsed`** : le module vit dans `kesh-db`, qui ne connaît pas `ParsedBackup` (type `kesh-api`). `TableRestore.column_names` porte toute l'information nécessaire et le crate reste sans dépendance montante.

### D-C5 — Le rapport de rejeu : logs + audit, rien d'autre

Chaque entrée produit un `ReplayedBackfill { version, label, trigger, rows_affected }`, où `trigger` distingue « classe A, inconditionnel » de « classe B, sentinelles absentes : … ».

- **`tracing::info!` par backfill rejoué**, `tracing::debug!` par backfill sauté — l'exploitant doit pouvoir lire dans le journal serveur *pourquoi* un rejeu a eu lieu et *ce qu'il a touché*.
- **`audit_log`** : le détail JSON de l'entrée `admin.full_import` existante (`routes/admin.rs:298-304`) reçoit une clé supplémentaire `backfills_replayed`. Pas de nouvelle action d'audit, pas de nouvelle table.

**`rows_affected` est informatif et ne fonde AUCUNE assertion de succès.** Zéro ligne touchée est un résultat parfaitement normal : le backfill 16-1a-bis est délibérément incomplet (son AC-B2), et une base restaurée peut n'avoir aucune ligne éligible. Ne **pas** écrire de garde « le rejeu doit avoir touché au moins une ligne » — ce serait la post-condition fausse contre laquelle 16-1a-bis met explicitement en garde.

### D-C6 — Le garde-fou fail-loud : toute future migration portant un backfill DOIT être triée

Sans cela, le registre redérivera au fil des Epics — exactement comme le compteur `tracked-by-sqlx` de `docs/migrations-idempotence-audit.md`, qui avait accumulé **7 de dérive** sur les Epics 20-21 et a survécu à sept passes adversariales. Le finding BH-1 de la passe 1 en est la démonstration immédiate : une migration exposée avait échappé au triage manuel de la rédaction initiale.

**Décision** : un test de `kesh-db` parcourt `MIGRATOR.migrations` et exige que **toute** migration dont le SQL contient un statement de backfill de données figure soit dans le **registre**, soit dans une **liste d'exemption explicite** portant une justification écrite. Une migration nouvelle non triée fait **échouer le test**, avec un message qui nomme le fichier et les deux issues possibles.

**Détection** — sur le SQL du `MIGRATOR`, après retrait des commentaires (`--` en fin de ligne comme en ligne entière ; aucune migration du dépôt n'utilise `/* */`, vérifié) : un statement dont le premier mot-clé est `UPDATE`, ou un `INSERT INTO … SELECT` **quelle que soit sa mise en forme sur plusieurs lignes**.

**Le retrait des commentaires n'est pas cosmétique** : les migrations du dépôt sont très commentées et plusieurs commentaires contiennent le mot `UPDATE` en prose (`20260722000001` : « Les douze UPDATE de… », plus au moins `20260418000001:17`, `20260522000001:12`, `20260614000001:47`). Un détecteur naïf sur la sous-chaîne `UPDATE` produirait des faux positifs — et la réaction naturelle serait de les exempter, ce qui viderait le garde-fou de son sens. Attention en particulier à `ON UPDATE CURRENT_TIMESTAMP`, présent dans une vingtaine de migrations : c'est du **DDL**, pas un statement `UPDATE`.

**Le découpage en statements doit être multi-ligne**, sans quoi le détecteur reproduit l'angle mort exact qui a coûté le finding BH-1.

**Les deux exemptions à écrire d'emblée** :

| Migration | Justification d'exemption |
|---|---|
| `20260419000002_users_company_id.sql` | `users.company_id` finit `NOT NULL` sans défaut ⇒ `is_required()` vrai ⇒ un backup qui ne la porte pas est **refusé** par `check_schema_compat` (400). Le cas ne peut pas se produire. |
| `20260714000002_email_templates_reminder.sql` | L'`UPDATE` porte sur `_kesh_version`, table système **jamais restaurée** (absente de `TABLES_TO_TRUNCATE`). Ce n'est pas un backfill de données applicatives. |

**Deux invariants supplémentaires à verrouiller par test**, parce qu'ils portent la sûreté des deux classes :

- **classe B ⇒ DDL dans le même fichier** : pour chaque entrée de classe B, le SQL de la migration source contient un `ADD COLUMN <sentinelle>`. Sans cela, la sentinelle ment (cas ECH-1).
- **classe A ⇒ no-op au rejeu, PROUVÉ À L'EXÉCUTION** : rejouer deux fois de suite le SQL d'une entrée de classe A sur une base représentative ne change rien au second passage. **Ne pas** substituer à ce test une détection textuelle de `IS NULL` / `NOT EXISTS` : elle classerait à tort le backfill `postable` #1, dont le `NOT EXISTS` est structurel (cf. D-C1).

### D-C7 — Le corps de réponse HTTP de `full-import` reste INCHANGÉ

Le corps `{ backupCreated, tablesRestored, rowsRestored, sourceVersion, sessionInvalidated }` (`routes/admin.rs:184-190`) n'est **pas** étendu.

**Motif** : le frontend consomme ce contrat (`frontend/src/lib/features/admin-restore/admin-restore.api.ts:12-14`, `AdminRestorePanel`). Ajouter un champ *utile* impliquerait de l'afficher, donc d'ouvrir le frontend, ses tests unitaires et ses quatre locales — pour une information de diagnostic que l'exploitant d'un restore lit dans le journal serveur, opération qu'il conduit de toute façon depuis la machine. Le rapport est donc **loggé et audité** (D-C5), pas retourné.

**Conséquence assumée** : un administrateur qui n'a pas accès aux logs ne voit pas que des backfills ont été rejoués. Acceptable pour v0.9.0 ; si l'exposition en UI est souhaitée, elle relève d'un CR distinct.

### D-C8 — Ce que le rejeu ne prétend PAS être

Le rejeu **n'est pas** un mécanisme général de « migration de données à l'import ». Il ne rattrape qu'un cas précis : *le backup précède une migration qui a rempli une colonne ou inséré des lignes*. Il ne détecte ni ne corrige :

- une colonne remplie par du **code applicatif** et non par une migration ;
- une donnée dont la sémantique a **changé** sans changement de colonne ;
- un backup **plus récent** que le binaire (refusé en amont, 409 si `min_required` l'exige, 400 `IMPORT_SCHEMA_MISMATCH` si une colonne inconnue apparaît).

À écrire dans le doc-comment du module. Un mainteneur qui croirait le mécanisme plus général qu'il ne l'est y verserait des rattrapages qui n'y ont pas leur place.

---

## Acceptance Criteria

- **AC-C1 — Registre** : `crates/kesh-db/src/post_restore.rs` expose un registre **ordonné par version croissante** de **5** entrées, chacune portant sa **classe déclarée explicitement** :

  | Version | Classe | Sentinelles | Source du SQL |
  |---|---|---|---|
  | `20260613000001` | **B** | `(vat_rates, category)` | extrait |
  | `20260614000001` | **A** | — | extrait |
  | `20260628000001` | **A** | — | extrait |
  | `20260722000001` | **B** | `(accounts, role)`, `(accounts, postable)` | extrait |
  | `20260729000001` | **A** | — | migration **entière** via `include_str!` |

- **AC-C2 — Déclencheur** : une entrée de **classe A** est rejouée **inconditionnellement**. Une entrée de **classe B** est rejouée **si et seulement si** au moins une de ses sentinelles est absente des `column_names` de la table correspondante dans le manifeste source (table absente = sentinelle absente) ; sinon **skip strict**, aucun statement n'est envoyé à la base.
- **AC-C3 — Ordre** : le rejeu suit l'ordre **croissant des versions**, classes mêlées. Un test asserte que le registre est strictement croissant.
- **AC-C4 — Transactionnalité** : le rejeu s'exécute dans la **transaction de restore**, après la garde de comptage (donc `FOREIGN_KEY_CHECKS = 1`) et avant l'audit. Toute erreur d'un statement remonte en `AppError::AdminFullImportFailed` et **annule le restore entier**.
- **AC-C5 — Fidélité du SQL** : chaque statement d'extrait est un **sous-texte verbatim** de la migration source telle qu'embarquée dans `MIGRATOR`, vérifié par test. L'entrée `20260729000001` est le fichier de migration **lui-même**.
- **AC-C6 — Garde-fous fail-loud**, trois tests distincts :
  1. une migration du `MIGRATOR` portant un statement de backfill (`UPDATE`, ou `INSERT … SELECT` **multi-ligne**) et absente à la fois du registre et de la liste d'exemption fait **échouer** le test, avec un message nommant le fichier et les deux issues ;
  2. toute entrée de **classe B** a son `ADD COLUMN <sentinelle>` dans **la même** migration ;
  3. toute entrée de **classe A** est un **no-op au second passage**, prouvé en rejouant son SQL deux fois sur une base représentative.
- **AC-C7 — Restitution** : chaque rejeu émet un `tracing::info!` portant la version, le déclencheur et le nombre de lignes touchées ; chaque skip émet un `tracing::debug!`. Le détail JSON de l'audit `admin.full_import` porte une clé `backfills_replayed`, **assertée** par au moins un cas de AC-C10.
- **AC-C8 — Contrat HTTP inchangé** : le corps de réponse de `POST /api/v1/admin/full-import` conserve exactement ses cinq clés. Aucun fichier de `frontend/` n'est modifié par cette story.
- **AC-C9 — Aucune migration nouvelle, aucune migration modifiée.** Donc aucun bump de `kesh_version_min_required` ni de version Cargo (P1/P2/P2-bis), et **aucune ligne ajoutée** à `docs/migrations-idempotence-audit.md`. Ses compteurs, **recomptés le 2026-08-01**, valent **57 / 5 / 52 / 0** (total / `yes` / `tracked-by-sqlx` / `no`) et ne doivent pas bouger. *(Le garde-fou P5 impose une ligne d'audit par migration **ajoutée** ; cette story n'en ajoute aucune. Ne pas « mettre à jour les compteurs par précaution » : ce serait les casser. **Recompter à l'implémentation** plutôt que reprendre ce nombre — la première rédaction de cette spec avait écrit 57/4/53/0, valeur *relue* dans un état de session périmé, jamais recomptée.)*

### AC-C10 — Post-conditions testées de bout en bout

Le test de bout en bout est **indispensable et non substituable** par des tests unitaires : le défaut naît de l'**interaction** entre l'import et des migrations que la PR ne touche pas.

Montage : `export_backup` → `unzip` → retirer la ou les colonnes visées des `columnNames` du manifeste (et/ou vider des lignes) → `rezip` → importer → observer. Cf. Dev Notes pour l'ancre exacte.

| Cas | Attendu | Ce qu'il discrimine |
|---|---|---|
| **C1** — backup **sans** `invoice_lines.revenue_account_id`, facture validée canonique | les lignes portent le compte crédité par leur écriture | cas nominal de l'issue #281 |
| **C1-bis** — backup **portant** la colonne mais **entièrement `NULL`** (fenêtre entre `20260727000001` et `20260729000001`) | les lignes sont **quand même** backfillées | **classe A / rejeu inconditionnel** — tombe si l'entrée est traitée en classe B (finding ECH-1) |
| **C2** — backup sans `accounts.role` **ni** `postable` | rôles réattribués, `postable` recalculé | classe B, déclenchement |
| **C2-bis** — backup portant `accounts.role` mais **pas** `postable` | le rejeu se déclenche **quand même** | **sémantique OU des sentinelles** — tombe si le dev implémente un ET (finding AA-4) |
| **C3** — backup complet, avec `role` / `postable` / `category` posés **à la main** sur des valeurs non standard | la donnée du backup est **intacte** | **classe B / conditionnement** — tombe si la classe B rejoue inconditionnellement |
| **C4** — échec injecté pendant le rejeu | transaction annulée, destination **inchangée** | transactionnalité |
| **C5** — backup sans `accounts.role` **et** sans `revenue_account_id`, monté pour que le candidat du backfill 16-1a-bis soit un compte que 14-3a rend **non imputable** | la ligne reste `NULL` | **ordre croissant** — tombe si le registre est parcouru à l'envers |
| **C6** — backup antérieur à `20260614000001` (comptes `1171` / `2206` absents des lignes `accounts`) | les deux comptes sont recréés pour chaque société ayant un plan | **classe A sur un `INSERT`** — le cas que la sentinelle-colonne ne pouvait pas voir (finding BH-1) |

**Sur C1-bis, C2-bis, C3, C5 et C6, une note de montage explicite est exigée dans le test** (§ « Ce que ce test discrimine ») : ce sont les seuls tests qui tombent si l'une des décisions structurantes est mal implémentée, et un montage qui se décale les rendrait muets — le mode d'échec exact subi par `backfill_skips_archived_accounts` en 16-1a.

**C1 ou C2 asserte en outre la clé `backfills_replayed`** de l'audit (AC-C7), en relisant la ligne `audit_log` après import.

---

## Tasks / Subtasks

- [ ] **T1 — Registre et mécanique de rejeu** (AC-C1, AC-C2, AC-C3)
  - [ ] Créer `crates/kesh-db/src/post_restore.rs` ; le déclarer dans `lib.rs` (ordre alphabétique des `pub mod`).
  - [ ] Doc-comment de module : le mécanisme, les **deux classes** de D-C1 et pourquoi aucune ne suffit seule, l'invariant d'ordre D-C2, et **ce que le rejeu n'est pas** (D-C8).
  - [ ] Types : `BackfillTrigger { Unconditional, Sentinels(&[(&str, &str)]) }`, `PostRestoreBackfill { version, label, trigger, sql }`, `ReplayedBackfill`.
  - [ ] Registre `POST_RESTORE_BACKFILLS: &[PostRestoreBackfill]`, **5** entrées, **triées par version croissante**, chacune commentée avec la justification écrite de sa classe (celles de D-C1, pas une paraphrase).
  - [ ] `replay_post_restore_backfills(tx, tables)` : évaluation du déclencheur, exécution `sqlx::raw_sql`, collecte du rapport.
- [ ] **T2 — Extraits SQL** (AC-C5)
  - [ ] `crates/kesh-db/src/post_restore/20260613000001_vat_rates_category.sql` — l'`UPDATE … CASE`, verbatim, `ELSE category END` **compris**.
  - [ ] `…/20260614000001_vat_accounts.sql` — les **2** `INSERT INTO accounts … SELECT … FROM companies c WHERE EXISTS … AND NOT EXISTS …`, verbatim, avec leurs `CASE c.accounting_language` complets.
  - [ ] `…/20260628000001_default_payable_account.sql` — l'`UPDATE … INNER JOIN accounts`, verbatim.
  - [ ] `…/20260722000001_accounts_role_postable.sql` — les **12** `UPDATE` (10 rôles + 2 `postable`), verbatim, **dans l'ordre du fichier source**.
  - [ ] En-tête de chaque extrait : de quelle migration il provient, pourquoi un extrait plutôt que le fichier entier (DDL), sa classe, et l'interdiction de le reformater.
  - [ ] `20260729000001` : `include_str!` de la migration, **aucun extrait**.
- [ ] **T3 — Câblage dans le restore** (AC-C4, AC-C7)
  - [ ] Appeler `replay_post_restore_backfills` dans `run_backup_and_restore`, **après** la garde de comptage et **avant** l'audit ; mapper l'erreur en `AppError::AdminFullImportFailed`.
  - [ ] `tracing::info!` / `debug!` par entrée ; clé `backfills_replayed` dans le détail de l'audit.
  - [ ] Vérifier que le corps de réponse HTTP est **inchangé** (AC-C8).
- [ ] **T4 — Garde-fous** (AC-C3, AC-C5, AC-C6)
  - [ ] Fonction de retrait des commentaires SQL + découpage **multi-ligne** en statements, unitairement testée sur les pièges réels : `ON UPDATE CURRENT_TIMESTAMP`, le mot `UPDATE` en prose de commentaire, un `INSERT … SELECT` à cheval sur deux lignes.
  - [ ] Liste d'exemption avec justification écrite (les 2 entrées de D-C6).
  - [ ] `every_data_backfill_migration_is_triaged`, message d'échec actionnable.
  - [ ] `registry_versions_are_strictly_increasing`.
  - [ ] `extract_statements_are_verbatim_substrings_of_source_migration`.
  - [ ] `class_b_sentinel_column_is_added_by_its_own_migration`.
  - [ ] `class_a_backfills_are_strict_noop_on_replay` — **à l'exécution**, pas par détection textuelle.
- [ ] **T5 — Tests de bout en bout** (AC-C10)
  - [ ] Écrire un helper `strip_column(manifest, data, table, column)` opérant sur le couple `(Value, BTreeMap<String, Vec<u8>>)` rendu par `unzip` — c'est le geste que C1, C2, C2-bis, C5 et C6 partagent.
  - [ ] Les 8 cas C1, C1-bis, C2, C2-bis, C3, C4, C5, C6 dans `crates/kesh-api/tests/admin_full_import_e2e.rs`, en réutilisant `spawn_app` / `export_backup` / `unzip` / `rezip` / `post_import` déjà présents.
  - [ ] Note de montage « ce que ce test discrimine » sur C1-bis, C2-bis, C3, C5, C6.
  - [ ] Assertion `backfills_replayed` de l'audit sur C1 ou C2.
- [ ] **T6 — Preuve par mutation** (geste 16-1a-bis, à reproduire)
  - [ ] Classe A rendue conditionnelle → **C1-bis et C6** doivent rougir.
  - [ ] Sentinelles en ET au lieu de OU → **C2-bis** doit rougir.
  - [ ] Classe B rendue inconditionnelle → **C3** doit rougir.
  - [ ] Registre parcouru en ordre décroissant → **C5** doit rougir.
  - [ ] Consigner les quatre résultats dans le Dev Agent Record. Si un test attendu **ne** rougit pas, le montage est muet : le corriger avant d'aller plus loin.
- [ ] **T7 — Documentation** (AC-C9)
  - [ ] **CHANGELOG** : amender le paragraphe de diagnostic de 16-1a-bis. Il dit aujourd'hui *« si vous avez restauré une sauvegarde antérieure à cette version, le chiffre remonté n'a pas la cause annoncée ici […] suivi dans l'issue #281 »* — ce n'est plus vrai. Le remplacer par la mention que la reprise **se rejoue** après un import, et signaler qu'un parc restauré **avant** cette version se répare en relançant le même import.
  - [ ] `docs/manual/fr/admin-manual.tex` — section restore : mentionner le rejeu, son ordre et le fait qu'il ne touche pas une donnée que le backup portait. Régénérer le PDF (`make fr` dans `docs/manual/`) et le commiter. **Ne PAS** toucher les macros de version (`kesh-style.sty`) — gate 4-bis, réservé au tag de release.
  - [ ] `CLAUDE.md` — garde-fou **P7** sous § « Migration breaking policy » : toute PR ajoutant une migration porteuse d'un backfill de données doit la trier (registre ou exemption justifiée) ; un manquement est un finding **MEDIUM** en `bmad-code-review`. Mentionner que le grep de détection doit être **multi-ligne** et renvoyer aux tests qui l'outillent.
  - [ ] **Ne PAS** ajouter de ligne ni toucher aux compteurs de `docs/migrations-idempotence-audit.md` (AC-C9).
- [ ] **T8 — Gate** : `scripts/test-fast.sh` complet (fmt + clippy `-D warnings` + nextest workspace) sur l'**état final**, exit 0 exigé, non présumé d'un run antérieur. `npm run check` inutile — aucun fichier frontend touché.

---

## Dev Notes

### Fichiers à lire AVANT d'écrire quoi que ce soit

| Fichier | Ce qu'il faut en retenir |
|---|---|
| `crates/kesh-db/src/backup.rs:375-486` | `restore_tables_in_tx` : `FOREIGN_KEY_CHECKS` posé à 0 puis **rétabli à 1 systématiquement**, y compris sur erreur. Le rejeu tourne donc FK actives — voulu (D-C4). |
| `crates/kesh-api/src/routes/admin.rs:221-328` | `run_backup_and_restore` : verrou `_kesh_version FOR UPDATE` → backup pré-import → restore → **garde de comptage** → audit → `force_onboarding_done_if_eligible` → commit. Point d'insertion : entre la garde et l'audit. |
| `crates/kesh-api/src/admin_backup/import.rs:28-33`, `:195-236` | `ParsedBackup.tables` et `check_schema_compat`. Les `column_names` proviennent du **manifeste**. |
| `crates/kesh-db/tests/invoice_lines_revenue_account_backfill.rs:1128-1140` | Le patron `sqlx::raw_sql(&sql)` sur le SQL réellement embarqué. Le rejeu utilise le même mécanisme. |
| `crates/kesh-api/tests/admin_full_import_e2e.rs:186-276` | **Le harnais réel** : `export_backup` (export HTTP) → `unzip` → édition du `Value` JSON → `rezip` → `post_import`. Les tests `full_import_refuses_schema_mismatch_400` (`:474`) et `full_import_refuses_missing_required_column_400` (`:504`) montrent déjà comment retirer/ajouter une entrée de `manifest["tables"][t]["columnNames"]`. |
| `crates/kesh-db/tests/common/mod.rs` | Résolution **par version** des fenêtres de migration (garde-fou P6). À ne pas confondre avec le registre de cette story, qui n'applique pas de migrations. |

### ⚠️ `build_test_backup` n'est PAS le harnais à utiliser

Cette fonction existe bien, mais dans `crates/kesh-api/src/admin_backup/import.rs:246`, à l'intérieur d'un `#[cfg(test)] mod tests` **privé de la lib**. Un fichier `tests/*.rs` est une **crate d'intégration séparée** : ce code n'y est même pas compilé, l'appeler ne compile pas. Utiliser `unzip` / `rezip` comme indiqué ci-dessus.

*(Ancre fausse dans la première rédaction de cette spec, relevée indépendamment par les trois lentilles de la passe 1 de `validate`.)*

### Décomptes — à RECOMPTER, jamais à relire

Les nombres de cette spec ont été recomptés le 2026-08-01. Les revérifier à l'implémentation :

```sh
ls crates/kesh-db/migrations/*.sql | wc -l                       # 57
grep -c '^| `20' docs/migrations-idempotence-audit.md            # 57
grep -lc "^\s*UPDATE\b" crates/kesh-db/migrations/*.sql          # 6 fichiers
perl -0777 -ne 'print "$ARGV\n" if /INSERT\s+INTO[^;]*?\bSELECT\b/si' crates/kesh-db/migrations/*.sql   # 1 fichier
grep -c "^\s*UPDATE\b" crates/kesh-db/migrations/20260722000001_accounts_role_postable.sql              # 12
```

⇒ **7** migrations porteuses d'un backfill, **5** exposées, **2** exemptes, **18** statements dont **3** non gardés.

### La garde de comptage du restore ne couvre PAS le rejeu

`run_backup_and_restore` vérifie `rows_restored == expected_rows` (`admin.rs:272-282`). Cette garde porte sur les `INSERT` du restore et **n'a aucun rapport** avec les statements du rejeu. Ne pas chercher à y intégrer `rows_affected`, ne pas s'en inspirer pour écrire une garde symétrique — cf. D-C5, zéro ligne touchée est normal.

### Pièges de recopie des extraits

- `20260613000001` : le `CASE … ELSE category END` doit être recopié **entier**. Le tronquer à ses quatre `WHEN` changerait la sémantique des lignes hors liste.
- `20260614000001` : les deux `INSERT` portent chacun un `CASE c.accounting_language` à quatre branches et un `parent_id` en sous-requête (`number = '10'` / `'20'`). Recopier le statement **entier**, du `INSERT` au `;`.
- `20260722000001` : **12** statements, pas 10. Les deux derniers portent sur `postable` et sont ceux qui placent l'entrée en classe B ; les omettre viderait C2 de sa moitié la plus consultée (`postable` gouverne l'imputabilité).
- `20260722000001`, backfill `postable` #1 : l'`EXISTS` corrélé porte sur la table cible `accounts`. MariaDB l'autorise dans un `UPDATE` (vérifié 10.11, commentaire du fichier source) — ne pas « corriger » en table dérivée en croyant contourner l'ER 1093.
- `20260628000001` : `UPDATE … INNER JOIN accounts a ON a.company_id = cis.company_id AND a.number = '2000'` — multi-société par construction ; ne pas ajouter de filtre.

### Pourquoi l'état de la base destination n'entre pas dans le raisonnement

Le restore **remplace intégralement** les tables de `TABLES_TO_TRUNCATE` (`DELETE` puis `INSERT`, `backup.rs:424-484`) **avant** le rejeu. L'état antérieur de la destination — fraîche, peuplée, multi-société, comptes archivés, exercice clos — est donc sans effet : seule compte la **source**. C'est ce qui rend le déclencheur analysable à partir du seul manifeste.

### Ce qui rend ce défaut invisible en revue de diff

Le mode d'échec ne naît ni du code écrit ni de la spec, mais de l'**interaction** entre une migration ajoutée par une story et un chemin de restore que cette story ne touche pas. C'est le même profil que le garde-fou **P6** (couplage positionnel des migrations), codifié en 16-1a après que trois tests ont changé de sens sans qu'aucune ligne de leur fichier ne bouge — dont un **passé à vide**. D'où l'exigence T6 : prouver par **mutation** que les cas discriminants discriminent, plutôt que de constater qu'ils sont verts.

### Project Structure Notes

- Modules touchés : `kesh-db` (nouveau module + tests), `kesh-api` (câblage + tests E2E), `docs/` + `CHANGELOG.md` + `CLAUDE.md`. **3 modules** — sous le seuil de 5 de la règle de splitting préventif.
- Le sous-répertoire `crates/kesh-db/src/post_restore/` ne contient que des `.sql` embarqués par `include_str!`. Il ne doit **pas** être confondu avec `crates/kesh-db/migrations/` : rien de ce qu'il contient n'est jamais vu par `sqlx::migrate!("./migrations")`. Le dire dans le doc-comment du module.
- Aucun fichier `frontend/`. Aucune migration.

### References

- Issue **#281** — [Restaurer un backup antérieur au backfill 16-1a-bis rouvre le bug définitivement](https://github.com/guycorbaz/kesh/issues/281), et son commentaire d'arbitrage du 2026-08-01 (D-1 / D-2 / D-3).
- Story **16-1a-bis** — `_bmad-output/implementation-artifacts/16-1a-bis-backfill-parc-existant.md`, décisions **D-B1** (source de vérité = l'écriture), **D-B2** (critère d'unicité), **D-B3** (`<=>` NULL-safe), **D-B5** (portée), **D-B6** (idempotence intrinsèque), **D-B7** (le backfill enregistre, il ne répare pas).
- Story **17-3c** — import transactionnel : `crates/kesh-api/src/admin_backup/import.rs`, `crates/kesh-api/src/routes/admin.rs`.
- `CLAUDE.md` § « Migration breaking policy » (P1-P6), § « Tech debt management » (triage hors rétrospective), § « Review Iteration Rule » (§ Propagation post-patch).
- `docs/migrations-idempotence-audit.md` — verdicts et invariant des compteurs (**57 / 5 / 52 / 0** au 2026-08-01, à recompter).

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

### Passe 1 de `bmad-create-story validate`

**2026-08-01 — Sonnet, 3 lentilles (BlindHunter / EdgeCaseHunter / AcceptanceAuditor), contexte frais.** 8 findings retenus : **2 CRITICAL, 2 HIGH, 2 MEDIUM, 2 LOW**. Aucun faux positif — les 8 ont été vérifiés en ground-truth par l'orchestrateur avant application.

**Les deux CRITICAL portent sur la même décision et l'ont fait refondre.** D-C1 conditionnait tout rejeu à l'absence d'une colonne sentinelle dans le manifeste source. Ce critère est faux dans deux directions :

- **BH-1** — une **5ᵉ migration exposée** avait échappé au triage : `20260614000001_vat_accounts_config.sql` insère les comptes de TVA `1171` / `2206` manquants. Elle avait été manquée parce que le grep de vérification de la spec était **mono-ligne** (`INSERT INTO.*SELECT`) alors que le `SELECT` est sur la ligne suivante. Et la sentinelle-colonne ne s'y généralise pas : ce sont des **lignes** qui manquent, `accounts.number` est présente dans tous les backups depuis l'origine.
- **ECH-1** — la sentinelle suppose « colonne présente ⇒ backfill appliqué ». Vrai seulement si le DDL et le backfill sont dans le **même fichier**. Or `20260727000001` crée `revenue_account_id` et `20260729000001` la remplit : un backup pris entre les deux porte la colonne, entièrement `NULL`, et aurait été déclaré « à jour » — rouvrant le bug de l'issue par un second chemin.

**Refonte** : deux classes d'entrées. **Classe A** (tous statements gardés contre l'intention utilisateur) → rejeu **inconditionnel**, no-op strict ; couvre les deux cas ci-dessus. **Classe B** (contient un statement non gardé) → sentinelle, dont la validité est désormais **verrouillée par un test** exigeant que le `ADD COLUMN` soit dans la même migration.

**Un défaut qu'aucune lentille n'a vu, trouvé au recompte de l'orchestrateur** : le backfill `postable` #1 contient un `NOT EXISTS` mais n'est **pas** gardé — c'est un prédicat structurel, pas une garde d'idempotence. Un garde-fou qui aurait classé les entrées par présence textuelle de `IS NULL` / `NOT EXISTS` l'aurait rangé à tort en classe A et aurait écrasé un `postable` posé à la main. Le garde-fou est donc un **no-op prouvé à l'exécution** (AC-C6.3), et la classe est **déclarée**, jamais devinée.

**Triple convergence des trois lentilles** (BH-2 / ECH-2 / AA-3, HIGH) : les Dev Notes désignaient `build_test_backup` comme harnais de test. La fonction vit dans `admin_backup/import.rs:246`, en `#[cfg(test)]` **privée de la lib** — inaccessible depuis une crate de test d'intégration, l'appeler ne compile pas. Le harnais réel est `export_backup` / `unzip` / `rezip`, déjà présent dans le fichier cible et déjà utilisé pour retirer une colonne d'un manifeste. Ancre corrigée + sous-tâche `strip_column` ajoutée.

**BH-3 / AA-1 (HIGH)** : AC-C9 annonçait les compteurs de l'audit d'idempotence à `57 / 4 / 53 / 0`. Recomptés : **57 / 5 / 52 / 0**. La valeur fausse venait d'un état de session **relu** et non recompté, antérieur au reclassement d'une ligne en `yes` par la passe 2 de revue de 16-1a-bis. Le mode de dérive exact que le garde-fou P5 existe pour empêcher, commis dans le document qui met en garde contre lui, à deux endroits.

**AA-2 (MEDIUM)** : la phrase introduisant le tableau des gardes disait « seuls trois `UPDATE` sur seize portent une garde `IS NULL` » quand le tableau qu'elle introduit, trois lignes plus bas, en montre treize gardés sur seize. Sens inversé.

**AA-4 (MEDIUM)** : la sémantique **OU** des sentinelles multiples n'était discriminée par aucun cas de test — `20260722000001` est la seule entrée à deux sentinelles, et le seul cas qui la touchait les omettait **toutes les deux**, montage où un **ET** erroné donne le même résultat vert. Cas **C2-bis** ajouté (présence partielle).

**AA-5 (MEDIUM)** : AC-C7 (logs + clé d'audit `backfills_replayed`) n'avait aucun test nommé. Assertion ajoutée sur C1 ou C2.

**AA-6 / ECH-3 (LOW)** : le point d'insertion du rejeu était décrit « entre le restore et l'audit », intervalle qui contient déjà la garde de comptage. Sans conséquence fonctionnelle, mais la spec doit désigner un ordre : **après** la garde.

**Effet sur le périmètre** : registre de 4 → **5** entrées, extraits de 3 → **4**, cas de test de 5 → **8**, mutations de 2 → **4**, garde-fous de 3 → **5**.

**Prochaine** : passe 2 de `validate`, LLM ≠ Sonnet, contexte frais.
