# Story 16.1c : Rejeu des backfills de données après un restore d'installation

## Status

ready-for-dev

## Story

**As a** exploitant d'une instance Kesh qui restaure un `.keshbackup` produit par une version antérieure,
**I want** que les **backfills de données** portés par les migrations postérieures à ce backup soient **rejoués à la fin du restore**,
**so that** l'import ne réintroduise pas silencieusement, et **définitivement**, les bugs que ces migrations avaient fermés — le rôle des comptes et le compte de produit par ligne disparaissant sans qu'aucun message ne le signale.

Issue : **#281**. Sous-story de l'Epic 16 « Facturation avancée ».

**Dépend de 16-1a-bis** (migration `20260729000001`), dont elle ferme la régression au restore. Les deux doivent partir dans **la même PR et la même v0.9.0** : publier 16-1a-bis sans cette story reviendrait à livrer la mine décrite par l'issue.

### Pourquoi cette story est dans l'Epic 16 et non dans l'Epic 17

Le code touché relève du chemin **backup/restore** (Epic 17, clos). Le rattachement à l'Epic 16 procède du **triage hors fenêtre rétrospective** de la § « Tech debt management » de `CLAUDE.md` : une dette catégorie A découverte en cours d'Epic et **critique pour l'Epic en cours** est traitée dans l'Epic en cours. Ici le critère est franchi de la manière la plus directe qui soit — 16-1a-bis **n'est pas encore publiée**, et c'est elle qui crée le cas.

*(Arbitrage de Guy du 2026-08-01, tracé en commentaire de l'issue #281.)*

---

## Contexte

### Le mécanisme, vérifié en ground-truth

Quatre protections auraient pu bloquer le cas. **Trois ne le font pas — la quatrième borne le problème et c'est elle qui en fixe le périmètre.**

1. **Le contrôle de version ne refuse qu'un backup trop récent.** `check_import_version_compat` (`crates/kesh-db/src/version.rs:281`, appelé en `routes/admin.rs:152`) ne rend `DowngradeRefused` que si le manifeste exige un binaire **plus récent** que le nôtre. Un backup **ancien** est accepté — et c'est voulu : restaurer une sauvegarde ancienne est le cas d'usage nominal.

2. **Le contrôle de colonnes ne bloque pas non plus.** `check_schema_compat` (`crates/kesh-api/src/admin_backup/import.rs:195`) n'exige la présence d'une colonne destination dans la source que si `ColumnConstraint::is_required()` (`crates/kesh-db/src/backup.rs:329-331`) est vrai :

   ```rust
   pub fn is_required(&self) -> bool {
       !self.is_nullable && !self.has_default && !self.is_auto_increment && !self.is_generated
   }
   ```

   **Le `has_default` est aussi déterminant que le `is_nullable`** — et c'est le point que l'issue ne dit qu'à moitié. Une colonne `NOT NULL DEFAULT …` est tout aussi silencieusement absente du contrôle qu'une colonne nullable : elle est simplement **réinitialisée à son défaut**. C'est le cas d'`accounts.postable` (`NOT NULL DEFAULT TRUE`).

3. **`_sqlx_migrations` n'est pas restaurée**, donc la migration reste marquée appliquée et ne repassera jamais. **Ancre exacte** : ce n'est pas la clause `AND TABLE_NAME NOT IN ('_sqlx_migrations', '_kesh_version')` de `backup.rs:586` — celle-ci est dans le **test** `backup_inventory_matches_schema`, qui ne fait que **documenter** l'invariant. La cause réelle est que `TABLES_TO_TRUNCATE` (`backup.rs:34-72`) ne **contient pas** ces deux tables, et que l'export (`admin_backup/export.rs:55`) comme le restore (`backup.rs:425`) n'énumèrent que cette constante. *(L'issue cite `:586` ; suivre l'ancre sans borner la fonction englobante mènerait à patcher un test.)*

4. **Mais le contrôle de COUVERTURE DES TABLES, lui, bloque — et c'est ce qui borne toute cette story.** `parse_and_verify` exige l'égalité **exacte**, dans les deux sens, entre l'ensemble des tables du manifeste et `TABLES_TO_TRUNCATE` du binaire courant (`import.rs:117-136`) : toute table attendue absente du manifeste → 400, toute table du manifeste hors inventaire → 400. Et `TABLES_TO_TRUNCATE` est verrouillé sur le schéma réel par `backup_inventory_matches_schema` (`backup.rs:581-606`).

   Comme **aucune migration du dépôt ne supprime de table** (`grep -c "DROP TABLE" crates/kesh-db/migrations/*.sql` → aucun), l'ensemble des tables ne fait que croître. Un backup n'est donc importable **que s'il provient d'un binaire postérieur ou égal à la dernière migration créatrice de table applicative** — aujourd'hui `20260715000001_invoice_reminders.sql`.

   **C'est la fenêtre d'importabilité, et elle détermine à elle seule quels backfills sont rattrapables.**

Le `DELETE` + `INSERT` du restore ne cite que les `columnNames` du manifeste (`backup.rs:468`) : dans cette fenêtre, toute colonne absente de la source prend `NULL` ou son `DEFAULT`, en silence.

### Le périmètre réel : 2 migrations rattrapables sur 9 porteuses d'un backfill

**Recompte à la source** (ne pas relire ces nombres — cf. § « Décomptes » des Dev Notes) :

```sh
grep -n "^\s*UPDATE\b" crates/kesh-db/migrations/*.sql      # 6 fichiers, 18 statements
grep -n "^\s*INSERT"    crates/kesh-db/migrations/*.sql      # 3 fichiers, 7 statements
grep -ln "^\s*DELETE\|^\s*REPLACE\|ON DUPLICATE" crates/kesh-db/migrations/*.sql   # aucun
```

⚠️ **Deux pièges de recompte, tous deux tombés en revue** : un `grep "INSERT INTO.*SELECT"` **mono-ligne** rend vide (le `SELECT` est à la ligne suivante) ; et un motif `INSERT\s+INTO` rate les quatre `INSERT **IGNORE** INTO … SELECT` de `20260428000001`. Chercher `^\s*INSERT` tout court, puis trier à la main.

| Migration | Backfill | Statut |
|---|---|---|
| `20260722000001_accounts_role_postable.sql` (14-3a) | `accounts.role` / `postable` | **RATTRAPABLE** — postérieure à la fenêtre |
| `20260729000001_…_revenue_account_backfill.sql` (16-1a-bis) | `invoice_lines` / `credit_note_lines.revenue_account_id` | **RATTRAPABLE** — l'objet de l'issue |
| `20260419000002_users_company_id.sql` | `users.company_id` | exempte — colonne finale `NOT NULL` sans défaut ⇒ `is_required()` vrai ⇒ backup refusé (400) |
| `20260428000001_vat_rates.sql` | 4 taux de TVA par société | exempte — **crée la table `vat_rates`** ; un backup sans ces lignes est un backup sans la table ⇒ refusé à la couverture |
| `20260522000001_kesh_version.sql` | ligne singleton `_kesh_version` | exempte — table système, jamais restaurée |
| `20260613000001_vat_rates_crud.sql` | `vat_rates.category` | exempte — **hors fenêtre** : un backup sans la colonne précède `20260628000001`, donc n'a pas les tables `supplier_invoices` ⇒ refusé |
| `20260614000001_vat_accounts_config.sql` | comptes `1171` / `2206` | exempte — **hors fenêtre**, même raisonnement |
| `20260628000001_supplier_invoices.sql` | `default_payable_account_id` | exempte — **autoréfutante** : la migration crée elle-même `supplier_invoices` et `supplier_invoice_lines` ; un backup sans la colonne n'a pas les tables |
| `20260714000002_email_templates_reminder.sql` | `UPDATE _kesh_version` (bump `min_required`) | exempte — table système, et ce n'est pas un backfill applicatif |

**9 migrations porteuses, 2 au registre, 7 exemptes.**

La perte de `accounts.role` est la plus lourde : elle casse la présentation des fonds propres par rôle (Epic 14) **et** la condition (4) du backfill 16-1a-bis, qui exige `accounts.postable = TRUE`.

⚠️ **La fenêtre est mouvante, et c'est un invariant à outiller, pas une note de bas de page.** Toute future migration créant une table applicative **referme la fenêtre** et rend injoignables toutes les entrées de registre antérieures. Le garde-fou AC-C6.6 recalcule la fenêtre depuis le `MIGRATOR` et fait échouer le test si une entrée du registre s'est retrouvée hors fenêtre — sans quoi le registre garderait du code mort qui *paraît* fonctionner.

### Ce qui n'est PAS dans cette story

- **Toute modification d'un fichier de migration existant.** Les checksums SHA-384 de sqlx rendent les `.sql` déjà appliqués **immuables** (cf. l'avertissement en tête de `docs/migrations-idempotence-audit.md`). Le rejeu se fait à côté, jamais en réécrivant l'histoire.
- **Le frontend.** Cf. D-C7.
- **La réparation d'un parc déjà restauré avant cette version.** Un exploitant dans ce cas relance l'import du même backup une fois à jour ; le rejeu s'appliquera alors. À dire au CHANGELOG, pas à outiller.
- **Le durcissement de `check_schema_compat`.** Écarté : cela transformerait un cas rattrapable en refus d'import.
- **Les 7 migrations exemptes.** Elles ne sont pas « à faire plus tard » : leur backfill est **inatteignable** par le chemin d'import, définitivement tant que la fenêtre reste où elle est. L'exemption est un résultat, pas un report.

---

## Décisions de conception

### D-C1 — DEUX CLASSES d'entrées, et le déclencheur n'est pas le même

Sur les **14** statements de backfill des 2 migrations rattrapables, **2 seulement** sont dépourvus de garde contre l'écrasement d'une valeur posée par l'utilisateur — mais ce sont eux qui interdisent un rejeu inconditionnel généralisé :

| Statement | Garde contre l'intention utilisateur | Rejeu inconditionnel sur un backup **récent** |
|---|---|---|
| `20260729000001`, les 2 `UPDATE` | `revenue_account_id IS NULL` **sur des factures `validated`** | no-op strict — cf. justification ci-dessous |
| `20260722000001`, les 10 `UPDATE` de rôle | `role IS NULL AND active = TRUE` | no-op strict |
| `20260722000001`, les 2 `UPDATE` de `postable` | **aucune** | **écrase** un `postable` posé à la main |

⚠️ **Le backfill `postable` #1 contient un `NOT EXISTS` et n'est pourtant PAS gardé.** Son `NOT EXISTS (… journal_entry_lines …)` est un **prédicat structurel** qui décrit quels comptes viser, pas une garde d'idempotence : rejoué, il repose `postable = FALSE` sur un compte que l'utilisateur venait de rendre imputable. **Conséquence directe** : un test qui classerait les entrées d'après la présence textuelle de `IS NULL` / `NOT EXISTS` se tromperait sur ce statement précis. La classe est **déclarée explicitement par entrée**, jamais devinée.

`role` et `postable` sont bien **éditables** : `PUT /api/v1/accounts/{id}` a une sémantique **full-replace** et les exige tous deux (`routes/accounts.rs:64-69`, `:271`).

#### Classe A — auto-gardée : rejeu INCONDITIONNEL

Une entrée dont **tous** les statements sont gardés contre l'intention utilisateur est rejouée **systématiquement**. Le rejeu est un no-op strict sur une base à jour, donc il n'y a rien à conditionner.

| Entrée | Justification écrite de l'appartenance à la classe A |
|---|---|
| `20260729000001` | Les deux `UPDATE` sont gardés `revenue_account_id IS NULL` **et restreints aux pièces `validated` / `issued`**. C'est la conjonction qui porte la sûreté : une facture validée n'est plus modifiable (`update` rejette tout statut ≠ `draft`), donc un `NULL` y subsistant **ne peut pas** être un choix utilisateur — contrairement à un `NULL` sur un brouillon, que le `PUT` produit dès qu'un client omet `revenueAccountId` (c'est le CR #278). Idempotence par ailleurs **prouvée par test** en 16-1a-bis (D-B6, `backfill_is_idempotent`). |

⚠️ **« Un `NULL` n'est l'expression d'aucun choix » est un critère FAUX en général — ne pas le réutiliser tel quel pour une entrée future.** Il a été écrit dans une rédaction antérieure de cette spec pour justifier le classement en A de `20260628000001` (`default_payable_account_id IS NULL`), et il était faux : le `PUT` des réglages de facturation lie ce champ en full-replace (`repositories/company_invoice_settings.rs:177`, `:190`), donc envoyer `null` l'efface **délibérément** ; et le lazy-create n'insère que `(company_id)` (`:60`, `:92`, `:141`), donc toute société créée après 2026-06-28 a ce champ `NULL` **de façon nominale**. Un rejeu inconditionnel lui aurait réattribué le compte n° 2000 à chaque import d'un backup **courant**. L'entrée est aujourd'hui hors fenêtre, mais **le critère doit être vérifié route par route** pour toute entrée de classe A future.

#### Classe B — sentinelle : rejeu CONDITIONNÉ à l'absence de la colonne

Une entrée contenant au moins un statement non gardé porte une ou plusieurs **colonnes sentinelles** `(table, colonne)`. Elle n'est rejouée que si **au moins une** sentinelle est **absente des `columnNames` du manifeste source** — information disponible via `parsed.tables[table].column_names` (`admin_backup/import.rs:28-33`).

- Sentinelle présente → le backup portait la colonne, **sa donnée fait foi** → **skip strict**.
- Sentinelle absente → le backup **précède** la migration ; il n'existe **aucune intention utilisateur** à écraser → **rejeu intégral**, statements non gardés compris.

| Entrée | Sentinelles |
|---|---|
| `20260722000001` | `(accounts, role)`, `(accounts, postable)` |

**La condition de validité de la classe B, et elle est stricte** : le raisonnement « colonne présente ⇒ backfill appliqué » n'est vrai **que si le DDL et le backfill sont dans le MÊME fichier de migration**, donc dans la même transaction. C'est le cas ici (`ADD COLUMN role` ligne 75 et `ADD COLUMN postable` ligne 77 de `20260722000001`), et **un test le verrouille** (AC-C6.2).

C'est précisément ce qui **interdisait** de traiter `20260729000001` en classe B : sa colonne est créée par une **autre** migration (`20260727000001`, DDL pur). Un backup pris entre les deux porte la colonne, entièrement `NULL`, et une sentinelle l'aurait déclaré « à jour » — rouvrant le bug de l'issue par un second chemin.

*(Cette fenêtre n'a jamais existé en version publiée : `20260727000001` et `20260729000001` partent dans la même v0.9.0. Elle est atteignable sur un build intermédiaire de `main`, ce qui suffit à justifier la classe A, et le raisonnement vaut surtout comme **règle pour l'avenir** : rien n'oblige une future story à garder DDL et backfill ensemble.)*

### D-C2 — Le rejeu s'exécute par ORDRE DE VERSION CROISSANTE

Le rejeu doit reproduire **exactement** ce qu'aurait fait une montée de version depuis le binaire source. Les deux entrées ne sont pas indépendantes :

- `20260722000001` attribue `role = 'CurrentYearResult'` au compte n° `2979` (ligne 137) puis rend non imputable tout compte portant ce rôle (ligne 177) ;
- la condition (4) de `20260729000001` **exige** `accounts.postable = TRUE` sur le compte candidat (ligne 255).

Rejoué **avant** `20260722000001`, le backfill 16-1a-bis voit `postable = TRUE` **partout** (valeur `DEFAULT` posée par le restore) et retient un compte que 14-3a s'apprête à rendre non imputable — le **compte de résultat de l'exercice**.

⚠️ **Ce n'est PAS « un compte de regroupement », contrairement à ce qu'affirmait une rédaction antérieure.** L'autre statement `postable` (ligne 161) vise les comptes de regroupement, mais il porte `NOT EXISTS (SELECT 1 FROM journal_entry_lines l WHERE l.account_id = a.id)` — or le candidat de 16-1a-bis est **par construction** le compte d'une ligne d'écriture (`INNER JOIN journal_entry_lines jel … a.id = jel.account_id`). Les deux ensembles sont **disjoints par construction** : ce statement ne peut jamais affecter un candidat. Le seul chemin réel passe par le rôle `CurrentYearResult`. **Monter C5 sur un compte de regroupement produit un cas impossible et un test muet.**

**Décision** : le registre est **ordonné par version croissante**, l'itération suit cet ordre, et deux tests le verrouillent — l'un sur la **déclaration** (AC-C6.5), l'autre sur l'**exécution** (C5).

### D-C3 — Le SQL rejoué est celui du dépôt, jamais une paraphrase

- **`20260729000001` est du backfill pur** (deux `UPDATE`, aucun DDL — c'est la seule migration du dépôt classée `yes` à l'audit d'idempotence *pour cette raison* ; les 4 autres `yes` le sont pour des `IF NOT EXISTS` sur du DDL). Son fichier est rejouable **en entier** : `include_str!("../migrations/20260729000001_invoice_lines_revenue_account_backfill.sql")`. **Zéro duplication**, et un renommage du fichier casse la **compilation** plutôt que de dégrader en échec runtime.
- **`20260722000001` mêle DDL et données.** La rejouer en bloc échouerait (`1060 duplicate column` dès le premier `ALTER`). Elle exige un **extrait** : un `.sql` dédié ne contenant que les 12 `UPDATE`, embarqué par `include_str!`.

**L'extrait est une copie VERBATIM.** Aucune adaptation n'est nécessaire : c'est la classe qui rend sûrs les statements non gardés, pas une réécriture. Un test asserte que chaque statement d'extrait est un **sous-texte de la migration source** telle qu'embarquée dans le `MIGRATOR` — il ne protège pas d'une dérive future (les migrations sont immuables) mais d'une **erreur de copie à l'écriture**, qui est le risque réel.

**Ne PAS** réécrire, reformater, ni « améliorer » un statement en le copiant, **ni juger de l'utilité d'une clause**. Un extrait n'est pas un lieu où l'on nettoie : le test de sous-texte le refuserait, et un `<>` introduit à la place d'un `<=>` reproduirait le piège D-B3 de 16-1a-bis, indiscernable du succès.

### D-C4 — Le rejeu vit dans la transaction de restore, après la garde de comptage

**Emplacement** : nouveau module `crates/kesh-db/src/post_restore.rs`, exposant le registre et

```rust
pub async fn replay_post_restore_backfills(
    tx: &mut Transaction<'_, MySql>,
    tables: &BTreeMap<String, TableRestore>,
) -> Result<Vec<ReplayedBackfill>, DbError>
```

appelée depuis `run_backup_and_restore` (`routes/admin.rs`) **après la garde de cohérence de comptage** `rows_restored == expected_rows` (`:269-282`) et **avant** l'insertion de l'audit (`:298`).

**Pourquoi après la garde de comptage** : si le restore est déjà incohérent, la transaction va être annulée — rejouer par-dessus serait du travail perdu et brouillerait le diagnostic.

**Pourquoi dans la même transaction** : un échec de rejeu doit annuler le restore entier. Un restore committé suivi d'un rejeu échoué laisserait précisément l'état que la story existe pour empêcher.

**Pourquoi une fonction séparée et non dans `restore_tables_in_tx`** : cette dernière rétablit `FOREIGN_KEY_CHECKS = 1` en sortie, systématiquement (`backup.rs:403-416`). Le rejeu doit tourner **FK actives**. L'inclure dans le corps du restore le ferait tourner sous `FK = 0`.

**Pourquoi `tables` et non `parsed`** : le module vit dans `kesh-db`, qui ne connaît pas `ParsedBackup` (type `kesh-api`). `TableRestore.column_names` porte toute l'information nécessaire et le crate reste sans dépendance montante.

### D-C5 — Le rapport de rejeu : logs + audit, rien d'autre

Chaque entrée produit un `ReplayedBackfill { version, label, trigger, rows_affected }`, où `trigger` distingue « classe A, inconditionnel » de « classe B, sentinelles absentes : … » et de « sauté ».

- **`tracing::info!` par backfill rejoué**, `tracing::debug!` par backfill sauté.
- **`audit_log`** : le détail JSON de l'entrée `admin.full_import` existante (`routes/admin.rs:298-304`) reçoit une clé supplémentaire `backfills_replayed`. Pas de nouvelle action d'audit, pas de nouvelle table.

**`rows_affected` est informatif et ne fonde AUCUNE assertion de succès.** Zéro ligne touchée est un résultat parfaitement normal : le backfill 16-1a-bis est délibérément incomplet (son AC-B2). Ne **pas** écrire de garde « le rejeu doit avoir touché au moins une ligne » — ce serait la post-condition fausse contre laquelle 16-1a-bis met explicitement en garde. *(La non-vacuité est prouvée là où elle doit l'être : sur les **fixtures de test**, cf. AC-C6.4 et T6, jamais sur la donnée de production.)*

### D-C6 — Le garde-fou fail-loud : toute future migration écrivant des données DOIT être triée

Sans cela, le registre redérivera au fil des Epics — comme le compteur `tracked-by-sqlx` de `docs/migrations-idempotence-audit.md`, qui avait accumulé **7 de dérive** sur les Epics 20-21 et a survécu à sept passes adversariales. Les deux passes de revue de cette story en ont fourni la démonstration immédiate : **deux migrations porteuses de backfill ont échappé au triage manuel**, chacune à cause d'un grep trop étroit.

**Décision** : un test de `kesh-db` parcourt `MIGRATOR.migrations` et exige que **toute** migration dont le SQL contient un statement d'écriture de données figure soit au **registre**, soit dans une **liste d'exemption explicite** portant une justification écrite. Une migration non triée fait **échouer** le test, avec un message nommant le fichier et les deux issues.

**Détection — large par conception.** Sur le SQL du `MIGRATOR`, après retrait des commentaires et découpage **multi-ligne** en statements : tout statement dont le premier mot-clé est `UPDATE`, `INSERT` (**toutes formes** : `INTO … SELECT`, `INTO … VALUES`, `IGNORE`, `LOW_PRIORITY`, `HIGH_PRIORITY`, `ON DUPLICATE KEY UPDATE`), `REPLACE` ou `DELETE`.

**Pourquoi si large alors que seules 3 formes existent aujourd'hui** : parce que le coût d'une forme en trop est une exemption d'une ligne, et le coût d'une forme manquante est un garde-fou **muet**. C'est déjà arrivé deux fois sur cette seule story — d'abord le mono-ligne, puis `INSERT IGNORE`. Semer une ligne de référence par migration (nouveau taux de TVA officiel, nouveau niveau de rappel) est un geste banal qui serait exposé au même trou de restore, sans qu'aucune colonne ne manque.

**Le retrait des commentaires n'est pas cosmétique** : les migrations sont très commentées et plusieurs commentaires contiennent le mot `UPDATE` en prose (`20260722000001` « Les douze UPDATE de… », `20260418000001:17`, `20260522000001:12`, `20260614000001:47`). Attention surtout à `ON UPDATE CURRENT_TIMESTAMP`, présent dans une vingtaine de migrations : c'est du **DDL**. Aucune migration n'utilise `/* */` (vérifié).

**Les 7 exemptions à écrire d'emblée** — reprendre les justifications du tableau du § Contexte, telles quelles.

**Cinq invariants supplémentaires verrouillés par test**, parce qu'ils portent la sûreté des deux classes :

- **classe B ⇒ DDL dans le même fichier** : le SQL de la migration source contient un `ADD COLUMN <sentinelle>`. Sans cela, la sentinelle ment.
- **classe A ⇒ no-op sur une base NOMINALE À JOUR** : monter une base par `MIGRATOR` complet + fixture d'usage courant, rejouer l'entrée, exiger **`rows_affected == 0`**. ⚠️ **Ce n'est PAS la même propriété que « idempotent au second passage »**, et c'est la distinction qui compte : `20260628000001` satisfaisait la seconde (le 1er passage pose le compte, le 2e ne fait rien) et violait la première. Une rédaction antérieure spécifiait le test d'idempotence, qui n'aurait rien attrapé.
- **classe A ⇒ non-vacuité** : sur une base montée **en amont** de la migration cible, le premier passage doit toucher **> 0** ligne. Sans quoi le test précédent est vrai à vide.
- **registre ordonné** par version strictement croissante.
- **registre dans la fenêtre** : chaque entrée est postérieure à la dernière migration créatrice de table applicative, fenêtre **recalculée depuis le `MIGRATOR`** et jamais codée en dur. Une entrée hors fenêtre est du code mort qui paraît fonctionner.

### D-C7 — Le corps de réponse HTTP de `full-import` reste INCHANGÉ

Le corps `{ backupCreated, tablesRestored, rowsRestored, sourceVersion, sessionInvalidated }` (`routes/admin.rs:184-190`) n'est **pas** étendu.

**Motif** : le frontend consomme ce contrat (`frontend/src/lib/features/admin-restore/admin-restore.api.ts:12-14`). Ajouter un champ *utile* impliquerait de l'afficher, donc d'ouvrir le frontend, ses tests unitaires et ses quatre locales — pour une information de diagnostic que l'exploitant lit dans le journal serveur, opération qu'il conduit de toute façon depuis la machine.

**Conséquence assumée** : un administrateur sans accès aux logs ne voit pas qu'un backfill a été rejoué. Acceptable pour v0.9.0 ; l'exposition en UI relèverait d'un CR distinct.

### D-C8 — Ce que le rejeu ne prétend PAS être

Le rejeu **n'est pas** un mécanisme général de « migration de données à l'import ». Il ne rattrape qu'un cas précis : *le backup est dans la fenêtre d'importabilité et précède une migration qui a rempli une colonne*. Il ne détecte ni ne corrige :

- une colonne remplie par du **code applicatif** et non par une migration ;
- une donnée dont la sémantique a **changé** sans changement de colonne ;
- un backup **hors fenêtre** (refusé en amont, 400) ou plus récent que le binaire (409, ou 400 si une colonne inconnue apparaît).

À écrire dans le doc-comment du module. Un mainteneur qui croirait le mécanisme plus général qu'il ne l'est y verserait des rattrapages qui n'y ont pas leur place.

---

## Acceptance Criteria

- **AC-C1 — Registre** : `crates/kesh-db/src/post_restore.rs` expose un registre **ordonné par version croissante** de **2** entrées, chacune portant sa **classe déclarée explicitement** :

  | Version | Classe | Sentinelles | Source du SQL |
  |---|---|---|---|
  | `20260722000001` | **B** | `(accounts, role)`, `(accounts, postable)` | extrait (12 `UPDATE`) |
  | `20260729000001` | **A** | — | migration **entière** via `include_str!` |

- **AC-C2 — Déclencheur** : une entrée de **classe A** est rejouée **inconditionnellement**. Une entrée de **classe B** est rejouée **si et seulement si** au moins une de ses sentinelles est absente des `column_names` de la table correspondante dans le manifeste source ; sinon **skip strict**, aucun statement n'est envoyé à la base. *(La branche « table absente du manifeste » compte comme sentinelle absente ; elle est **défensive seulement** — `parse_and_verify` refuse en 400 tout manifeste incomplet en tables, donc l'état est inatteignable. **Ne pas écrire de test dédié** : il ne serait constructible qu'en court-circuitant le parseur.)*
- **AC-C3 — Ordre** : le rejeu suit l'ordre **croissant des versions**. Verrouillé en déclaration (AC-C6.5) **et** en exécution (C5).
- **AC-C4 — Transactionnalité** : le rejeu s'exécute dans la **transaction de restore**, après la garde de comptage (donc `FOREIGN_KEY_CHECKS = 1`) et avant l'audit. Toute erreur d'un statement remonte en `AppError::AdminFullImportFailed` et **annule le restore entier**.
- **AC-C5 — Fidélité du SQL** : chaque statement d'extrait est un **sous-texte verbatim** de la migration source telle qu'embarquée dans `MIGRATOR`, vérifié par test. L'entrée `20260729000001` est le fichier de migration **lui-même**.
- **AC-C6 — Garde-fous fail-loud**, six tests distincts :
  1. une migration du `MIGRATOR` portant un statement d'écriture de données (`UPDATE` / `INSERT` toutes formes / `REPLACE` / `DELETE`, découpage **multi-ligne**, commentaires retirés) et absente à la fois du registre et de la liste d'exemption fait **échouer** le test, avec un message nommant le fichier et les deux issues ;
  2. toute entrée de **classe B** a son `ADD COLUMN <sentinelle>` dans **la même** migration ;
  3. toute entrée de **classe A** est un **no-op sur une base nominale à jour** (`MIGRATOR` complet + fixture d'usage courant) : `rows_affected == 0` ;
  4. toute entrée de **classe A** est **non vacue** : sur une base montée en amont de sa migration, le premier passage touche **> 0** ligne ;
  5. le registre est **strictement croissant** en version ;
  6. chaque entrée du registre est **postérieure à la dernière migration créatrice de table applicative**, fenêtre recalculée depuis le `MIGRATOR`.
- **AC-C7 — Restitution** : chaque rejeu émet un `tracing::info!` portant la version, le déclencheur et le nombre de lignes touchées ; chaque skip émet un `tracing::debug!`. Le `trigger` et le `rows_affected` de chaque `ReplayedBackfill` sont **assertés sur la valeur de retour** de `replay_post_restore_backfills` dans au moins un cas de AC-C10 — capter `tracing` en test est fragile, verrouiller le contenu du rapport ne l'est pas. Le détail JSON de l'audit `admin.full_import` porte une clé `backfills_replayed`, **assertée** en relisant `audit_log` après import.
- **AC-C8 — Contrat HTTP inchangé** : le corps de réponse de `POST /api/v1/admin/full-import` conserve exactement ses cinq clés. Aucun fichier de `frontend/` n'est modifié — **contrôle de revue** (`git diff --stat -- frontend/` vide), pas un test.
- **AC-C9 — Aucune migration nouvelle, aucune migration modifiée.** Donc aucun bump de `kesh_version_min_required` ni de version Cargo (P1/P2/P2-bis), et **aucune ligne ajoutée** à `docs/migrations-idempotence-audit.md`. Ses compteurs, **recomptés le 2026-08-01**, valent **57 / 5 / 52 / 0** (total / `yes` / `tracked-by-sqlx` / `no`) et ne doivent pas bouger. *(Le garde-fou P5 impose une ligne d'audit par migration **ajoutée** ; cette story n'en ajoute aucune. Ne pas « mettre à jour les compteurs par précaution » : ce serait les casser. **Recompter à l'implémentation** — la première rédaction de cette spec avait écrit 57/4/53/0, valeur *relue* dans un état de session périmé.)*
- **AC-C11 — Documentation, critères opposables** :
  1. `CHANGELOG.md` ne contient plus la chaîne `la reprise ne se rejoue pas après un import` (`grep -cF` → 0) et décrit le rejeu ;
  2. `docs/manual/fr/admin-manual.tex` décrit le rejeu en section restore, PDF régénéré et commité, macros de `kesh-style.sty` **non** touchées (gate 4-bis) ;
  3. `CLAUDE.md` porte un garde-fou **P7** sous « Migration breaking policy ».

### AC-C10 — Post-conditions testées de bout en bout

Le test de bout en bout est **indispensable et non substituable** : le défaut naît de l'**interaction** entre l'import et des migrations que la PR ne touche pas.

**Montage — la voie praticable est de muter la base AVANT l'export**, pas de forger le backup. Retirer une entrée de `columnNames` du manifeste est sûr (les octets NDJSON ne bougent pas, le SHA-256 et le `rowCount` restent valides — c'est ce que font déjà `full_import_refuses_schema_mismatch_400:474` et `full_import_refuses_missing_required_column_400:504`). **Modifier ou supprimer des lignes NDJSON ne l'est pas** : `parse_and_verify` vérifie le SHA-256 (`import.rs:152-157`) **et** le `rowCount` (`:163-166`), et `rezip` ne les recalcule pas. Pour C1-bis, faire donc `UPDATE invoice_lines SET revenue_account_id = NULL` sur la base source **puis** `export_backup`.

| Cas | Attendu | Ce qu'il discrimine |
|---|---|---|
| **C1** — backup **sans** `invoice_lines.revenue_account_id`, facture validée canonique | les lignes portent le compte crédité par leur écriture | cas nominal de l'issue #281 |
| **C1-bis** — base source mutée à `revenue_account_id = NULL` **puis** exportée, colonne donc **présente et vide** | les lignes sont **quand même** backfillées | **classe A / rejeu inconditionnel** — tombe si l'entrée est traitée en classe B |
| **C2** — backup sans `accounts.role` **ni** `postable` | rôles réattribués, `postable` recalculé | classe B, déclenchement |
| **C2-bis** — backup portant `accounts.role` mais **pas** `postable` | le rejeu se déclenche **quand même** | **sémantique OU des sentinelles** — tombe si le dev implémente un ET. **Discriminant synthétique assumé** : les deux colonnes sont ajoutées par le même `ALTER`, donc co-présentes en réalité ; le cas se construit par `strip_column` et ne teste aucun état atteignable — il verrouille la règle pour les entrées futures |
| **C3** — backup complet, `role` / `postable` posés **à la main** sur des valeurs non standard | la donnée du backup est **intacte** | **classe B / conditionnement** |
| **C4** — échec injecté **pendant le rejeu** | transaction annulée, destination **inchangée** | transactionnalité |
| **C5** — backup sans `accounts.role`, **sans `accounts.postable`** et sans `revenue_account_id` ; le candidat unique du backfill est le compte de résultat n° `2979` | la ligne reste `NULL` | **ordre croissant** — tombe si le registre est parcouru à l'envers |

**Notes de montage obligatoires** — chaque test porte une section « ce que ce test discrimine ». Trois montages ont un piège nommé :

- **C4** : le patron existant `full_import_rolls_back_on_insert_failure` (`:543`) injecte l'échec dans l'`INSERT` du restore, donc **avant** le point d'insertion du rejeu — le réutiliser tel quel produit un test vert qui n'atteint jamais le rejeu. Prévoir un point d'injection propre (entrée de registre supplémentaire compilée en `#[cfg(test)]` portant un SQL fautif), **plus une assertion de montage** prouvant que le rejeu a démarré.
- **C5** : retirer `postable` est **indispensable**. Présent dans le backup, il vaut déjà `FALSE` sur `2979` et le test passe dans les deux ordres. Le candidat doit de plus être obtenu en **rééditant l'écriture** de la facture validée via `PUT /api/v1/journal-entries/{id}` pour porter le crédit sur `2979` en conservant `credit = invoices.total_amount` — `POST /invoices` refuserait `2979` directement, la validation d'écriture tournant avec `enforce_postable = false`.
- **C1** : sans facture validée dans la source, l'assertion porte sur un ensemble vide et le **cas nominal de l'issue passe à vide**. D'où la mutation dédiée en T6.

---

## Tasks / Subtasks

- [ ] **T1 — Registre et mécanique de rejeu** (AC-C1, AC-C2, AC-C3)
  - [ ] Créer `crates/kesh-db/src/post_restore.rs` ; le déclarer dans `lib.rs` (les `pub mod` y sont en ordre alphabétique — `post_restore` s'insère entre `pool` et `repositories`).
  - [ ] Doc-comment de module : la **fenêtre d'importabilité** et pourquoi elle borne le registre, les **deux classes** de D-C1 et pourquoi aucune ne suffit seule, l'invariant d'ordre D-C2, et **ce que le rejeu n'est pas** (D-C8).
  - [ ] Types : `BackfillTrigger { Unconditional, Sentinels(&[(&str, &str)]) }`, `PostRestoreBackfill { version, label, trigger, sql }`, `ReplayedBackfill { version, label, trigger, rows_affected }`.
  - [ ] Registre `POST_RESTORE_BACKFILLS: &[PostRestoreBackfill]`, **2** entrées, triées par version croissante, chacune commentée avec la justification écrite de sa classe (celle de D-C1, pas une paraphrase).
  - [ ] `replay_post_restore_backfills(tx, tables)` : évaluation du déclencheur, exécution `sqlx::raw_sql`, collecte du rapport.
- [ ] **T2 — Extrait SQL** (AC-C5)
  - [ ] `crates/kesh-db/src/post_restore/20260722000001_accounts_role_postable.sql` — les **12** `UPDATE` (10 rôles + 2 `postable`), verbatim, **dans l'ordre du fichier source**.
  - [ ] En-tête de l'extrait : de quelle migration il provient, pourquoi un extrait plutôt que le fichier entier (DDL), sa classe, et l'interdiction de le reformater ou d'en juger l'utilité clause par clause.
  - [ ] `20260729000001` : `include_str!` de la migration, **aucun extrait**.
- [ ] **T3 — Câblage dans le restore** (AC-C4, AC-C7)
  - [ ] Appeler `replay_post_restore_backfills` dans `run_backup_and_restore`, **après** la garde de comptage et **avant** l'audit ; mapper l'erreur en `AppError::AdminFullImportFailed`.
  - [ ] `tracing::info!` / `debug!` par entrée ; clé `backfills_replayed` dans le détail de l'audit.
  - [ ] Vérifier que le corps de réponse HTTP est **inchangé** (AC-C8).
- [ ] **T4 — Garde-fous** (AC-C3, AC-C5, AC-C6)
  - [ ] Retrait des commentaires SQL + découpage **multi-ligne** en statements, unitairement testés sur les pièges réels : `ON UPDATE CURRENT_TIMESTAMP`, le mot `UPDATE` en prose, un `INSERT … SELECT` à cheval sur deux lignes, un `INSERT IGNORE INTO`.
  - [ ] Liste d'exemption, **7** entrées avec justification écrite (§ Contexte).
  - [ ] Les 6 tests d'AC-C6.
- [ ] **T5 — Tests de bout en bout** (AC-C10)
  - [ ] **Fixture métier** — `admin_full_import_e2e.rs` ne sait aujourd'hui créer qu'une société et un utilisateur (`seed_role:139`) : aucun plan comptable, aucune facture, aucune écriture. Écrire le montage : semer le plan via `bulk_create_from_chart`, créer contact et produit, créer puis **valider** une facture pour produire son écriture. Patron disponible dans `crates/kesh-db/tests/invoice_lines_revenue_account_backfill.rs`.
  - [ ] Helper `strip_column(manifest, data, table, column)` sur le couple `(Value, BTreeMap<String, Vec<u8>>)` rendu par `unzip` — geste partagé par C1, C2, C2-bis, C5.
  - [ ] Les 7 cas C1, C1-bis, C2, C2-bis, C3, C4, C5, en réutilisant `spawn_app` / `export_backup` / `unzip` / `rezip` / `post_import`.
  - [ ] Note de montage « ce que ce test discrimine » sur **tous** les cas ; les pièges nommés de C1, C4 et C5.
  - [ ] Assertions `backfills_replayed` (audit) et `trigger` / `rows_affected` (valeur de retour) — AC-C7.
- [ ] **T6 — Preuve par mutation** (AC-C10)
  - [ ] Classe A rendue conditionnelle (sentinelle `(invoice_lines, revenue_account_id)`) → **C1-bis** doit rougir, **C1** rester vert.
  - [ ] Sentinelles en ET au lieu de OU → **C2-bis** doit rougir.
  - [ ] Classe B rendue inconditionnelle → **C3** doit rougir.
  - [ ] Registre parcouru en ordre décroissant → **C5** doit rougir.
  - [ ] Registre vidé de l'entrée `20260729000001` → **C1** doit rougir *(prouve la non-vacuité du cas nominal, qui sans cela pourrait passer sur un ensemble vide)*.
  - [ ] Consigner les cinq résultats dans le Dev Agent Record. **Un test attendu qui ne rougit pas invalide le montage** : le corriger avant d'aller plus loin.
- [ ] **T7 — Documentation** (AC-C9, AC-C11)
  - [ ] **CHANGELOG** : la phrase actuelle *« si vous avez restauré une sauvegarde antérieure à cette version, le chiffre remonté n'a pas la cause annoncée ici : la reprise ne se rejoue pas après un import »* devient **fausse** avec cette story, et elle partirait telle quelle en v0.9.0 puisque les deux stories sont dans la même PR. La remplacer : la reprise **se rejoue** après un import, et un parc restauré avant cette version se répare en relançant le même import.
  - [ ] `docs/manual/fr/admin-manual.tex` — section restore : le rejeu, son ordre, et le fait qu'il ne touche pas une donnée que le backup portait. Régénérer le PDF (`make fr` dans `docs/manual/`) et le commiter. **Ne PAS** toucher `kesh-style.sty` (gate 4-bis, réservé au tag de release).
  - [ ] `CLAUDE.md` — garde-fou **P7** : toute PR ajoutant une migration écrivant des données doit la trier (registre ou exemption justifiée) ; manquement = finding **MEDIUM** en `bmad-code-review`. Mentionner que la détection couvre `UPDATE` / `INSERT` toutes formes / `REPLACE` / `DELETE`, en **multi-ligne**, et renvoyer aux tests qui l'outillent.
  - [ ] **Ne PAS** ajouter de ligne ni toucher aux compteurs de `docs/migrations-idempotence-audit.md`.
- [ ] **T8 — Gate** : `scripts/test-fast.sh` complet (fmt + clippy `-D warnings` + nextest workspace) sur l'**état final**, exit 0 exigé, non présumé d'un run antérieur. `npm run check` inutile — aucun fichier frontend touché.

---

## Dev Notes

### Fichiers à lire AVANT d'écrire quoi que ce soit

| Fichier | Ce qu'il faut en retenir |
|---|---|
| `crates/kesh-api/src/admin_backup/import.rs:117-136` | **Le contrôle de couverture des tables** — c'est lui qui borne toute la story. Égalité exacte dans les deux sens avec `TABLES_TO_TRUNCATE`. |
| `crates/kesh-api/src/admin_backup/import.rs:152-166` | SHA-256 **et** `rowCount` vérifiés par table ⇒ on ne forge pas les octets NDJSON, on mute la base avant l'export. |
| `crates/kesh-db/src/backup.rs:375-486` | `restore_tables_in_tx` : `FOREIGN_KEY_CHECKS` posé à 0 puis **rétabli à 1 systématiquement**, y compris sur erreur. |
| `crates/kesh-api/src/routes/admin.rs:221-328` | `run_backup_and_restore` : verrou → backup pré-import → restore → **garde de comptage** → audit → onboarding → commit. Point d'insertion : entre la garde et l'audit. |
| `crates/kesh-db/tests/invoice_lines_revenue_account_backfill.rs:1128-1140` | Le patron `sqlx::raw_sql(&sql)` sur le SQL réellement embarqué, et le montage d'une facture validée avec son écriture. |
| `crates/kesh-api/tests/admin_full_import_e2e.rs:135-276` | Le harnais réel : `seed_role` (société + utilisateur, **rien d'autre**), `export_backup`, `unzip`, `rezip`, `post_import`. |
| `crates/kesh-db/tests/common/mod.rs` | Résolution **par version** des fenêtres de migration (garde-fou P6). À ne pas confondre avec le registre, qui n'applique aucune migration. |

### ⚠️ `build_test_backup` n'est PAS le harnais à utiliser

Cette fonction existe, mais dans `crates/kesh-api/src/admin_backup/import.rs:246`, à l'intérieur d'un `#[cfg(test)] mod tests` **privé de la lib**. Un fichier `tests/*.rs` est une **crate d'intégration séparée** : ce code n'y est même pas compilé, l'appeler ne compile pas. Utiliser `unzip` / `rezip`.

### Décomptes — à RECOMPTER, jamais à relire

Recomptés le 2026-08-01, à revérifier à l'implémentation :

```sh
ls crates/kesh-db/migrations/*.sql | wc -l                       # 57
grep -c '^| `20' docs/migrations-idempotence-audit.md            # 57
grep -n "^\s*UPDATE\b" crates/kesh-db/migrations/*.sql           # 6 fichiers, 18 statements
grep -n "^\s*INSERT"   crates/kesh-db/migrations/*.sql           # 3 fichiers, 7 statements
grep -c "^\s*UPDATE\b" crates/kesh-db/migrations/20260722000001_accounts_role_postable.sql   # 12
```

⇒ **9** migrations écrivant des données, **2** au registre, **7** exemptes.

⚠️ **Deux nombres valent 18 et ne désignent pas la même chose.** Les `UPDATE` de **toutes** les migrations sont au nombre de 18 ; les statements des **2 migrations du registre** sont au nombre de **14** (12 + 2). Ne pas croiser les deux : ils divergeront dès la prochaine migration porteuse d'un `UPDATE`. Sur les 14, **2** sont non gardés (les deux `postable`).

### La garde de comptage du restore ne couvre PAS le rejeu

`run_backup_and_restore` vérifie `rows_restored == expected_rows` (`admin.rs:272-282`). Cette garde porte sur les `INSERT` du restore et **n'a aucun rapport** avec les statements du rejeu. Ne pas chercher à y intégrer `rows_affected` — cf. D-C5, zéro ligne touchée est normal en production.

### Pièges de recopie de l'extrait

- **12** statements, pas 10. Les deux derniers portent sur `postable` et sont ceux qui placent l'entrée en classe B ; les omettre viderait C2 de sa moitié la plus consultée (`postable` gouverne l'imputabilité).
- Backfill `postable` #1 : l'`EXISTS` corrélé porte sur la table cible `accounts`. MariaDB l'autorise dans un `UPDATE` (vérifié 10.11, commentaire du fichier source) — ne pas « corriger » en table dérivée en croyant contourner l'ER 1093.
- Ne pas réordonner : le backfill `postable` #2 lit le rôle `CurrentYearResult` que l'`UPDATE` de rôle n° 9 vient de poser.

### Pourquoi l'état de la base destination n'entre pas dans le raisonnement

Le restore **remplace intégralement** les tables de `TABLES_TO_TRUNCATE` (`DELETE` puis `INSERT`, `backup.rs:424-484`) **avant** le rejeu. L'état antérieur de la destination — fraîche, peuplée, multi-société, comptes archivés, exercice clos — est donc sans effet : seule compte la **source**. C'est ce qui rend le déclencheur analysable à partir du seul manifeste.

### Ce qui rend ce défaut invisible en revue de diff

Le mode d'échec ne naît ni du code écrit ni de la spec, mais de l'**interaction** entre une migration ajoutée par une story et un chemin de restore que cette story ne touche pas. C'est le profil du garde-fou **P6** (couplage positionnel des migrations), codifié en 16-1a après que trois tests ont changé de sens sans qu'aucune ligne de leur fichier ne bouge — dont un **passé à vide**. D'où T6 : prouver par **mutation** que les cas discriminants discriminent.

### Project Structure Notes

- Modules touchés : `kesh-db` (nouveau module + tests), `kesh-api` (câblage + tests E2E), `docs/` + `CHANGELOG.md` + `CLAUDE.md`. **3 modules** — sous le seuil de 5 de la règle de splitting préventif.
- Le sous-répertoire `crates/kesh-db/src/post_restore/` ne contient qu'un `.sql` embarqué par `include_str!`. Il ne doit **pas** être confondu avec `crates/kesh-db/migrations/` : rien de ce qu'il contient n'est jamais vu par `sqlx::migrate!("./migrations")`. La coexistence de `src/post_restore.rs` et d'un répertoire `src/post_restore/` ne contenant aucun `.rs` est légale.
- Aucun fichier `frontend/`. Aucune migration.

### References

- Issue **#281** — [Restaurer un backup antérieur au backfill 16-1a-bis rouvre le bug définitivement](https://github.com/guycorbaz/kesh/issues/281), et son commentaire d'arbitrage du 2026-08-01.
- Story **16-1a-bis** — `_bmad-output/implementation-artifacts/16-1a-bis-backfill-parc-existant.md`, décisions **D-B1** (source de vérité = l'écriture), **D-B2** (critère d'unicité), **D-B3** (`<=>` NULL-safe), **D-B5** (portée), **D-B6** (idempotence intrinsèque), **D-B7** (le backfill enregistre, il ne répare pas).
- Story **17-3c** — import transactionnel : `crates/kesh-api/src/admin_backup/import.rs`, `crates/kesh-api/src/routes/admin.rs`.
- `CLAUDE.md` § « Migration breaking policy » (P1-P6), § « Tech debt management », § « Review Iteration Rule ».
- `docs/migrations-idempotence-audit.md` — compteurs **57 / 5 / 52 / 0** au 2026-08-01, à recompter.

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

### Passe 1 de `bmad-create-story validate`

**2026-08-01 — Sonnet, 3 lentilles, contexte frais. 8 findings : 2 CRITICAL, 2 HIGH, 3 MEDIUM, 1 LOW**, tous vérifiés en ground-truth, 0 faux positif.

Les deux CRITICAL portaient sur la même décision. D-C1 conditionnait tout rejeu à l'absence d'une colonne sentinelle ; le critère est faux dans les deux directions. **BH-1** : `20260614000001` insère des **lignes** (comptes de TVA), qu'aucune sentinelle-colonne ne peut voir — manquée par un grep **mono-ligne** sur `INSERT INTO.*SELECT`. **ECH-1** : la sentinelle suppose « colonne présente ⇒ backfill appliqué », vrai seulement si DDL et backfill sont dans le même fichier — or `20260727000001` crée `revenue_account_id` et `20260729000001` la remplit. D'où la refonte en **deux classes**.

Un défaut qu'aucune lentille n'a vu, trouvé au recompte : le backfill `postable` #1 contient un `NOT EXISTS` mais n'est **pas** gardé — prédicat structurel, pas garde d'idempotence. Un garde-fou classant par détection textuelle l'aurait rangé en classe A. D'où un garde-fou **à l'exécution**, et une classe **déclarée**.

**Triple convergence** (HIGH) : `build_test_backup` est en `#[cfg(test)]` privé de la lib, inaccessible depuis une crate de test d'intégration. **HIGH compteurs** : `57/5/52/0` et non `57/4/53/0` — valeur relue et jamais recomptée. Le résidu correspondant dans 16-1a-bis a été **daté** plutôt que réécrit. MEDIUM : phrase « 3 `UPDATE` sur 16 gardés » inversée ; sémantique OU des sentinelles non discriminée ; AC-C7 sans test. LOW : point d'insertion ambigu.

### Passe 2 de `bmad-create-story validate`

**2026-08-01 — Opus, 3 lentilles, contexte frais. 19 findings : 1 CRITICAL, 4 HIGH, 6 MEDIUM, 8 LOW.**

**Le CRITICAL réduit la story au lieu de l'agrandir.** Il existe une **quatrième protection** que ni l'issue ni les deux rédactions précédentes n'avaient vue : `parse_and_verify` (`import.rs:117-136`) exige l'égalité **exacte, dans les deux sens**, entre les tables du manifeste et `TABLES_TO_TRUNCATE`. Comme aucune migration ne supprime de table, un backup n'est importable que s'il provient d'un binaire **≥ à la dernière migration créatrice de table** (`20260715000001`). Sur les 5 entrées du registre, **3 étaient donc injoignables** — le cas de `20260628000001` étant autoréfutant, puisqu'elle crée elle-même les tables dont l'absence accompagnerait celle de sa colonne.

Ce n'était pas du code mort inoffensif : les 3 entrées étaient en **classe A**, donc exécutées **inconditionnellement à chaque import**. Deux lentilles ont montré indépendamment (**ECH2-1**, **BH2-6**) que l'une d'elles aurait réattribué le compte créancier n° 2000 à une société dont l'administrateur avait délibérément vidé le champ — le `PUT` des réglages le lie en full-replace, et le lazy-create laisse ce champ `NULL` **de façon nominale** sur toute société créée après 2026-06-28. La justification « un `NULL` n'est l'expression d'aucun choix » était donc fausse ; elle est conservée dans la spec comme **mise en garde explicite**, parce que le critère resservira.

**Le garde-fou de triage aurait été muet à la livraison** (**BH2-2**, HIGH) : il ne détectait que `INSERT INTO … SELECT`, ce qui rate les 4 `INSERT **IGNORE** INTO … SELECT` de `20260428000001`. C'est le même angle mort qu'en passe 1 sur un autre axe — la remédiation n'avait traité que le multi-ligne, pas les variantes de clause. **ECH2-4** l'a élargi encore : `INSERT … VALUES`, `REPLACE`, `DELETE`, `ON DUPLICATE KEY` échappaient tous. La détection couvre désormais **toutes** les formes d'écriture, au prix d'une exemption d'une ligne par faux positif.

**L'invariant d'ordre était vrai, sa démonstration fausse, et son test muet** (**BH2-3** / **AA2-1** / **BH2-4**, HIGH). D-C2 justifiait l'ordre par « écrirait des comptes de regroupement » : impossible, la garde `NOT EXISTS (journal_entry_lines)` du backfill `postable` #1 exclut structurellement tout candidat de 16-1a-bis, qui porte toujours une ligne d'écriture. Le seul chemin réel passe par le rôle `CurrentYearResult` (compte n° 2979). Et C5, monté sans retirer `postable`, aurait été vert dans les deux ordres — `postable` étant déjà `FALSE` dans le backup.

**ECH2-2 (MEDIUM) est le finding qui aurait attrapé le précédent.** Le garde-fou de classe A spécifié mesurait l'**idempotence** (« rejouer deux fois ne change rien au second passage »), pas la propriété dont la classe A a besoin (« rejouer sur une base à jour ne change rien »). `20260628000001` satisfaisait la première et violait la seconde. Le garde-fou est remplacé par un **no-op sur base nominale** + une assertion de **non-vacuité**.

Trois findings de montage, convergents : le fichier E2E n'a **aucune fixture métier** (**ECH2-3** / **AA2-2**) — sans facture validée, le cas nominal de l'issue passe à vide ; **C4 n'avait aucun mécanisme d'injection** et le patron le plus proche injecte *avant* le rejeu (**AA2-3**) ; et le montage prescrit était **irréalisable** pour les cas mutant des lignes, le SHA-256 et le `rowCount` du manifeste étant vérifiés (**ECH2-5**) — d'où la voie « muter la base avant l'export ».

**AA2-4 (MEDIUM)** : aucun AC ne couvrait les livrables documentaires, dont la phrase du CHANGELOG que cette story rend fausse et qui partirait telle quelle en v0.9.0. D'où **AC-C11**, à critères grepables.

LOW appliqués : décompte de sévérité de la passe 1 corrigé (2/2/**3**/**1**) ; les deux nombres « 18 » dissociés (18 `UPDATE` toutes migrations ≠ 14 statements du registre) ; branche « table absente » qualifiée de défensive sans test ; justification du `ELSE category` retirée avec l'entrée.

**Deux lentilles ont ratifié à tort.** L'AcceptanceAuditor déclare le triage « exhaustif, vérifié par les deux greps » — son grep multi-ligne utilisait `INSERT\s+INTO` et ratait `INSERT IGNORE`. Aucune des trois n'a vu la protection de couverture avant que BlindHunter ne la trouve. Le rappel du projet vaut : une section « vérifié et jugé sain » est opposable sur ce qu'elle **énumère**, pas sur ce qu'elle conclut.

**Effet sur le périmètre** : registre 5 → **2** entrées, extraits 4 → **1**, exemptions 2 → **7**, cas de test 8 → **7**, mutations 4 → **5**, garde-fous 5 → **6**, AC 10 → **11**.

**Sur la règle de splitting** : le second critère (sévérité non décroissante) est formellement coché, `CRITICAL → CRITICAL`. Il n'est **pas** retenu comme déclencheur de split, pour deux raisons — le changement de modèle (Sonnet → Opus) rend les passes non comparables, précédent explicitement constaté en 16-1a-bis ; et surtout la story **rétrécit** à chaque passe (5 entrées → 2, 4 extraits → 1), ce qui est l'inverse du symptôme que la règle vise. À rouvrir si la passe 3 ne redescend pas nettement.

**Prochaine** : passe 3, LLM ≠ Opus, contexte frais.
