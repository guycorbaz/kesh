# Story 16.1c : Rejeu des backfills de données après un restore d'installation

## Status

review

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
- **La couverture de bout en bout** — six cas E2E et leurs preuves par mutation → **16-1d**, qui doit partir dans la même PR.
- **Le durcissement de `check_schema_compat`.** Écarté : cela transformerait un cas rattrapable en refus d'import.
- **Les 7 migrations exemptes.** Elles ne sont pas « à faire plus tard » : leur backfill est **inatteignable** par le chemin d'import, définitivement tant que la fenêtre reste où elle est. L'exemption est un résultat, pas un report.

---

## Décisions de conception

### D-C1 — DEUX CLASSES d'entrées, et le déclencheur n'est pas le même

Sur les **14** statements de backfill des 2 migrations rattrapables, **2 seulement** sont dépourvus de garde contre l'écrasement d'une valeur posée par l'utilisateur — mais ce sont eux qui interdisent un rejeu inconditionnel généralisé :

| Statement | Garde contre l'intention utilisateur | Rejeu inconditionnel sur un backup **récent** |
|---|---|---|
| `20260729000001`, les 2 `UPDATE` | `revenue_account_id IS NULL` **sur des factures `validated`** | no-op strict — cf. justification ci-dessous |
| `20260722000001`, les 10 `UPDATE` de rôle | `role IS NULL AND active = TRUE` | no-op **sur un compte dont le rôle est renseigné** — mais **PAS une garde d'intention** : `role: null` est un acte de retrait délibéré (`routes/accounts.rs:70`), et un compte hors plan standard (`2850` / `2860`, absents des chartes livrées) porte `role = NULL` nominalement. C'est l'une des raisons du classement en B |
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
) -> Result<Vec<ReplayedBackfill>, DbError>          // délègue avec POST_RESTORE_BACKFILLS

pub async fn replay_with_registry(
    tx: &mut Transaction<'_, MySql>,
    tables: &BTreeMap<String, TableRestore>,
    registry: &[PostRestoreBackfill],
) -> Result<Vec<ReplayedBackfill>, DbError>
```

**Le registre est un paramètre de la fonction interne, pas seulement un `const` global.** C'est ce qui rend l'**échec du rejeu testable** : `#[cfg(test)]` ne traverse pas la frontière de crate, et par le chemin HTTP l'erreur se réduit à un `500` générique dont le détail est loggé et jamais exposé. Sans cette porte, AC-C4 n'aurait aucun test constructible — cf. la note de montage de C4.

appelée depuis `run_backup_and_restore` (`routes/admin.rs`) **après la garde de cohérence de comptage** `rows_restored == expected_rows` (`:269-282`) et **avant** l'insertion de l'audit (`:298`).

**Pourquoi après la garde de comptage** : si le restore est déjà incohérent, la transaction va être annulée — rejouer par-dessus serait du travail perdu et brouillerait le diagnostic.

**Pourquoi dans la même transaction** : un échec de rejeu doit annuler le restore entier. Un restore committé suivi d'un rejeu échoué laisserait précisément l'état que la story existe pour empêcher.

**Pourquoi une fonction séparée et non dans `restore_tables_in_tx`** : cette dernière rétablit `FOREIGN_KEY_CHECKS = 1` en sortie, systématiquement (`backup.rs:403-416`). Le rejeu doit tourner **FK actives**. L'inclure dans le corps du restore le ferait tourner sous `FK = 0`.

**Pourquoi `tables` et non `parsed`** : le module vit dans `kesh-db`, qui ne connaît pas `ParsedBackup` (type `kesh-api`). `TableRestore.column_names` porte toute l'information nécessaire et le crate reste sans dépendance montante.

### D-C5 — Le rapport de rejeu : logs + audit, rien d'autre

Chaque entrée du registre — **rejouée comme sautée** — produit un `ReplayedBackfill { version, label, outcome, rows_affected }`. Le champ `outcome` porte **trois** états : rejeu inconditionnel (classe A), rejeu sur sentinelles absentes (classe B, avec la liste des sentinelles manquantes), et **sauté** (classe B, sentinelles toutes présentes).

⚠️ **Ne pas réutiliser le type `BackfillTrigger` du registre pour ce champ.** Le `trigger` déclare *ce qui doit déclencher* l'entrée et n'a que deux variants ; l'`outcome` rapporte *ce qui s'est passé* et en a trois. Les confondre sous un même nom — ce que faisait une rédaction antérieure — rend le variant « sauté » inencodable et laisse le `tracing::debug!` du skip sans état correspondant.

- **`tracing::info!` par backfill rejoué**, `tracing::debug!` par backfill sauté.
- **`audit_log`** : le détail JSON de l'entrée `admin.full_import` existante (`routes/admin.rs:298-304`) reçoit une clé supplémentaire `backfills_replayed`. Pas de nouvelle action d'audit, pas de nouvelle table.

**`rows_affected` est informatif et ne fonde AUCUNE assertion de succès.** Zéro ligne touchée est un résultat parfaitement normal : le backfill 16-1a-bis est délibérément incomplet (son AC-B2). Ne **pas** écrire de garde « le rejeu doit avoir touché au moins une ligne » — ce serait la post-condition fausse contre laquelle 16-1a-bis met explicitement en garde. *(La non-vacuité est prouvée là où elle doit l'être : sur les **fixtures de test**, cf. AC-C6.4, et **16-1d T-D3** pour les mutations, jamais sur la donnée de production.)*

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
  4. toute entrée de **classe A** est **non vacue** : sur une base montée en amont de sa migration, le premier passage touche **> 0** ligne. ⚠️ **Le code applicatif postérieur à la migration cible peut déjà écrire la donnée que le backfill rattrape** — pour `20260729000001`, la matérialisation de 16-1a à la validation (`invoices.rs:1700-1705`) : la fixture doit reconstituer l'état pré-migration explicitement (`UPDATE invoice_lines SET revenue_account_id = NULL` après validation, patron `invoice_lines_revenue_account_backfill.rs:1180`), sinon le test mesure 0 ;
  5. le registre est **strictement croissant** en version ;
  6. chaque entrée du registre est **postérieure à la dernière migration créatrice de table applicative**, fenêtre recalculée depuis le `MIGRATOR` **sur le SQL décommenté** (même prétraitement qu'en 6.1 — `20260702000001:8` et `20260714000001:4` contiennent `CREATE TABLE` en prose de commentaire). **« Applicative » = figurant dans `TABLES_TO_TRUNCATE`** : une future migration créant une table **système** (comme `20260522000001` l'a fait pour `_kesh_version`) ne déplace pas la fenêtre, puisque `parse_and_verify` ne compare que l'inventaire applicatif. Ne pas se contenter de chercher `CREATE TABLE`.
- **AC-C7 — Restitution** : chaque rejeu émet un `tracing::info!` portant la version, l'issue et le nombre de lignes touchées ; chaque skip émet un `tracing::debug!`. L'`outcome` et le `rows_affected` de chaque `ReplayedBackfill` sont **assertés sur la valeur de retour** de `replay_post_restore_backfills` dans au moins un cas de **16-1d, AC-D1** — dont **un cas où l'`outcome` vaut `Skipped`** (C3), sans quoi le troisième état ne serait vérifié nulle part — capter `tracing` en test est fragile, verrouiller le contenu du rapport ne l'est pas. Le détail JSON de l'audit `admin.full_import` porte une clé `backfills_replayed`, **assertée** en relisant `audit_log` après import.
- **AC-C8 — Contrat HTTP inchangé** : le corps de réponse de `POST /api/v1/admin/full-import` conserve exactement ses cinq clés. Aucun fichier de `frontend/` n'est modifié — **contrôle de revue** (`git diff --stat -- frontend/` vide), pas un test.
- **AC-C9 — Aucune migration nouvelle, aucune migration modifiée.** Donc aucun bump de `kesh_version_min_required` ni de version Cargo (P1/P2/P2-bis), et **aucune ligne ajoutée** à `docs/migrations-idempotence-audit.md`. Ses compteurs, **recomptés le 2026-08-01**, valent **57 / 5 / 52 / 0** (total / `yes` / `tracked-by-sqlx` / `no`) et ne doivent pas bouger. *(Le garde-fou P5 impose une ligne d'audit par migration **ajoutée** ; cette story n'en ajoute aucune. Ne pas « mettre à jour les compteurs par précaution » : ce serait les casser. **Recompter à l'implémentation** — la première rédaction de cette spec avait écrit 57/4/53/0, valeur *relue* dans un état de session périmé.)*
- **AC-C11 — Documentation, critères opposables** *(énoncé ici ; la section détaillée d'AC-C10 suit, sa longueur justifiant une sous-section propre)* :
  1. `CHANGELOG.md` ne contient plus la chaîne `la reprise ne se rejoue pas après un import` (`grep -cF` → 0) et décrit le rejeu ;
  2. `docs/manual/fr/admin-manual.tex` décrit le rejeu en section restore, PDF régénéré et commité, macros de `kesh-style.sty` **non** touchées (gate 4-bis) ;
  3. `CLAUDE.md` porte un garde-fou **P7** sous « Migration breaking policy ».

### AC-C10 — Transactionnalité prouvée, au seul niveau où l'échec est observable

**La couverture de bout en bout est en 16-1d** (`16-1d-couverture-e2e-rejeu-backfills.md`) : six cas E2E et leurs preuves par mutation. Elle **doit partir dans la même PR** — un mécanisme sans preuve d'interaction ne vaut pas.

Reste ici le seul cas qui ne relève pas de l'E2E :

| Cas | Attendu | Ce qu'il discrimine |
|---|---|---|
| **C4** — test d'intégration `kesh-db` : échec injecté **pendant le rejeu** via `replay_with_registry` et un registre fautif | l'appel rend `Err`, et après rollback la destination est **inchangée** | transactionnalité (AC-C4) |

**Note de montage obligatoire — « ce que ce test discrimine ».**

- **C4** : le patron existant `full_import_rolls_back_on_insert_failure` (`:543`) injecte l'échec dans l'`INSERT` du restore, donc **avant** le point d'insertion du rejeu — le réutiliser tel quel produit un test vert qui n'atteint jamais le rejeu. Prévoir un point d'injection propre, **plus une assertion de montage** prouvant que le rejeu a démarré.

  ⚠️ **`#[cfg(test)]` NE TRAVERSE PAS la frontière de crate — ne pas chercher à injecter par là.** `kesh-db` est une dépendance **ordinaire** de `kesh-api` (`crates/kesh-api/Cargo.toml:9`) : depuis un test d'intégration de `kesh-api`, `cfg(test)` de `kesh-db` vaut **faux**. Une entrée fautive `#[cfg(test)]` ne serait donc vue par **aucun** des six cas de 16-1d, ni par C4 ici — et non par tous, comme l'affirmait une rédaction antérieure de cette note. C'est le même piège que celui documenté trois sections plus bas pour `build_test_backup`.

  Et la déclarer inconditionnellement est **exclu** : le registre de production compterait 3 entrées au lieu de 2 (AC-C1), dont une dont le seul rôle est de faire échouer tout restore dès que sa sentinelle manque — la mine même que cette story désamorce.

  **Décision** : rendre le registre **injectable**. `replay_post_restore_backfills(tx, tables)` délègue à une fonction `pub` `replay_with_registry(tx, tables, registry)`, appelée avec `POST_RESTORE_BACKFILLS`. **C4 devient alors un test d'intégration de `kesh-db`**, non un cas HTTP : ouvrir une transaction, restaurer, appeler `replay_with_registry` avec un registre fautif, vérifier l'`Err` puis, après rollback, que la destination est intacte. C'est le seul niveau où l'échec est **observable** — par le chemin HTTP, `AppError::AdminFullImportFailed` rend un `500` générique dont le détail est **loggé et jamais exposé** (`errors.rs`), donc indiscernable d'un échec d'`INSERT` du restore.

  **Conséquence sur le décompte** : **AC-C10 ne porte plus que C4** ; les six cas E2E sont partis en **16-1d, AC-D1**. Aucune exemption n'est alors nécessaire dans les garde-fous d'AC-C6, puisqu'aucune entrée fautive n'entre dans le `const`.
*(La note de montage de **C5** est partie en **16-1d, AC-D1** avec le cas qu'elle décrit. Elle n'est pas dupliquée ici : une correction appliquée à une seule des deux copies les ferait diverger en silence, et ce montage a déjà été réécrit trois fois.)*
---

## Tasks / Subtasks

- [x] **T1 — Registre et mécanique de rejeu** (AC-C1, AC-C2, AC-C3)
  - [x] Créer `crates/kesh-db/src/post_restore.rs` ; le déclarer dans `lib.rs` (les `pub mod` y sont en ordre alphabétique — `post_restore` s'insère entre `pool` et `repositories`).
  - [x] Doc-comment de module : la **fenêtre d'importabilité** et pourquoi elle borne le registre, les **deux classes** de D-C1 et pourquoi aucune ne suffit seule, l'invariant d'ordre D-C2, et **ce que le rejeu n'est pas** (D-C8).
  - [x] Types — **la déclaration et l'issue sont deux choses distinctes, ne pas les confondre sous un même nom** : `BackfillTrigger { Unconditional, Sentinels(&[(&str, &str)]) }` déclare **ce qui doit déclencher** l'entrée (2 variants, propriété du registre) ; `ReplayOutcome { ReplayedUnconditional, ReplayedSentinelsAbsent(Vec<String>), Skipped }` rapporte **ce qui s'est passé** (3 variants, propriété de l'exécution). D'où `PostRestoreBackfill { version, label, trigger, sql }` et `ReplayedBackfill { version, label, outcome, rows_affected }`.
  - [x] Registre `POST_RESTORE_BACKFILLS: &[PostRestoreBackfill]`, **2** entrées, triées par version croissante, chacune commentée avec la justification écrite de sa classe (celle de D-C1, pas une paraphrase).
  - [x] `replay_with_registry(tx, tables, registry)` **`pub`** : évaluation du déclencheur, exécution `sqlx::raw_sql`, collecte du rapport. `replay_post_restore_backfills(tx, tables)` n'en est que l'appel avec `POST_RESTORE_BACKFILLS` — c'est la porte d'injection qui rend AC-C4 testable (D-C4).
- [x] **T2 — Extrait SQL** (AC-C5)
  - [x] `crates/kesh-db/src/post_restore/20260722000001_accounts_role_postable.sql` — les **12** `UPDATE` (10 rôles + 2 `postable`), verbatim, **dans l'ordre du fichier source**.
  - [x] En-tête de l'extrait : de quelle migration il provient, pourquoi un extrait plutôt que le fichier entier (DDL), sa classe, et l'interdiction de le reformater ou d'en juger l'utilité clause par clause.
  - [x] `20260729000001` : `include_str!` de la migration, **aucun extrait**.
- [x] **T3 — Câblage dans le restore** (AC-C4, AC-C7, AC-C8)
  - [x] Appeler `replay_post_restore_backfills` dans `run_backup_and_restore`, **après** la garde de comptage et **avant** l'audit ; mapper l'erreur en `AppError::AdminFullImportFailed`.
  - [x] `tracing::info!` / `debug!` par entrée ; clé `backfills_replayed` dans le détail de l'audit.
  - [x] Vérifier que le corps de réponse HTTP est **inchangé** (AC-C8).
- [x] **T4 — Garde-fous** (AC-C3, AC-C5, AC-C6)
  - [x] Retrait des commentaires SQL + découpage **multi-ligne** en statements, couverts par **un seul test paramétré** sur les quatre pièges réels : `ON UPDATE CURRENT_TIMESTAMP`, le mot `UPDATE` en prose de commentaire, un `INSERT … SELECT` à cheval sur deux lignes, un `INSERT IGNORE INTO`. *(Un test paramétré, pas quatre tests — il compte pour **un** dans le total de 8 ci-dessous.)*
  - [x] Liste d'exemption, **7** entrées avec justification écrite (§ Contexte).
  - [x] Les **6** tests d'AC-C6 (triage, DDL même fichier, no-op nominal, non-vacuité, ordre croissant, fenêtre).
  - [x] **Plus le test d'AC-C5**, qui n'est pas dans AC-C6 et n'appartient à aucune autre tâche : `extract_statements_are_verbatim_substrings_of_source_migration` — chaque statement de l'extrait `20260722000001` est un sous-texte du SQL de la migration tel qu'embarqué dans le `MIGRATOR`. **8 tests au total pour cette tâche** : 1 (parsing paramétré) + 6 (AC-C6) + 1 (AC-C5).
- [x] **T5 — Test de transactionnalité** (AC-C4, AC-C10)
  - [x] Cas **C4** en test d'intégration `kesh-db` (`crates/kesh-db/tests/`) : restaurer, appeler `replay_with_registry` avec un registre fautif, vérifier l'`Err`, puis après rollback que la destination est intacte.
  - [x] Note de montage « ce que ce test discrimine », reprenant le piège nommé en AC-C10.
  - [x] **La couverture E2E (six cas + six mutations) est en 16-1d** — ne pas la dupliquer ici. *(Cinq mutations à l'implémentation, une sixième ajoutée en passe 1 de sa revue de code.)*

*(**T6 — preuve par mutation** est partie en 16-1d avec les cas qu'elle vise. Numérotation conservée pour que les renvois des passes 1 à 5 restent lisibles.)*

- [x] **T7 — Documentation** (AC-C9, AC-C11)
  - [x] **CHANGELOG** : la phrase actuelle *« si vous avez restauré une sauvegarde antérieure à cette version, le chiffre remonté n'a pas la cause annoncée ici : la reprise ne se rejoue pas après un import »* devient **fausse** avec cette story, et elle partirait telle quelle en v0.9.0 puisque les deux stories sont dans la même PR. La remplacer : la reprise **se rejoue** après un import, et un parc restauré avant cette version se répare en réimportant la même sauvegarde — **en énonçant le coût**, l'import remplaçant l'intégralité de l'installation, donc en perdant tout ce qui a été saisi depuis le restore fautif. *(Ce coût était omis des trois sites qui portaient le remède ; corrigé en passe 2 de revue.)*
  - [x] `docs/manual/fr/admin-manual.tex` — section restore : le rejeu, son ordre, et le fait qu'il ne touche pas une donnée que le backup portait. Régénérer le PDF (`make fr` dans `docs/manual/`) et le commiter. **Ne PAS** toucher `kesh-style.sty` (gate 4-bis, réservé au tag de release).
  - [x] `CLAUDE.md` — garde-fou **P7** : toute PR ajoutant une migration écrivant des données doit la trier (registre ou exemption justifiée) ; manquement = finding **MEDIUM** en `bmad-code-review`. Mentionner que la détection couvre `UPDATE` / `INSERT` toutes formes / `REPLACE` / `DELETE`, en **multi-ligne**, et renvoyer aux tests qui l'outillent.
  - [x] **Ne PAS** ajouter de ligne ni toucher aux compteurs de `docs/migrations-idempotence-audit.md`.
- [x] **T8 — Gate** : `scripts/test-fast.sh` complet (fmt + clippy `-D warnings` + nextest workspace) sur l'**état final**, exit 0 exigé, non présumé d'un run antérieur. `npm run check` inutile — aucun fichier frontend touché.

### Review Findings

*Passe 1 de `bmad-code-review`, 2026-08-02, Opus 5, 3 lentilles (BlindHunter, EdgeCaseHunter, AcceptanceAuditor), diff aplati `f699eda5 → a06049d6`. 30 findings bruts → 13 après fusion et triage : 1 décision, 11 patches, 1 différé, 3 écartés.*

- [x] [Review][Decision] **(RÉSOLU — arbitrage de Guy : documenter la limite)** **Restore chaîné — la sentinelle de classe B ne rattrape pas un backup pris depuis un parc déjà cassé** — Un exploitant passé en v0.8.0 puis ayant importé un backup v0.7.0 (avant l'existence de ce mécanisme) a un parc sans rôles. Un backup pris **depuis cet état** porte les colonnes `role` / `postable` mais pas les données : `missing_sentinels` rend `[]`, l'entrée est `Skipped`, le parc reste cassé. Le mécanisme ne cause pas le dommage — il ne le répare pas. La § « Ce que ce mécanisme n'est PAS » ne liste pas ce cas. Deux options : **(a)** documenter la limite (module + § « Ce qui n'est PAS dans cette story ») ; **(b)** scinder l'entrée `20260722000001` en classe A pour les 10 `UPDATE` de rôle (gardés `role IS NULL`) et classe B pour les 2 `UPDATE` de `postable` (non gardés) — change la structure du registre. [`crates/kesh-db/src/post_restore.rs:279-287`, `:335-347`]

- [x] [Review][Patch] **CRITICAL — AC-C6.3 et AC-C6.4 ne sont pas implémentés, T4 est cochée à tort** : T4 exige « les **6** tests d'AC-C6 (triage, DDL même fichier, **no-op nominal**, **non-vacuité**, ordre croissant, fenêtre) » ; seuls 4 existent. La seule entrée de classe A (`20260729000001`), rejouée **inconditionnellement à chaque import**, n'a aucune preuve automatisée qu'elle ne réécrit pas une donnée présente — c'est le mode d'échec qui avait produit le CRITICAL de la passe 2 de spec. Le compte annoncé « 9 tests / 9 verts » masque la lacune : 3 des 9 tests sont hors AC (`registry_entries_match_a_real_migration`, `exemptions_are_real_and_disjoint_from_registry`, `missing_sentinels_uses_or_semantics`). [`crates/kesh-db/src/post_restore.rs:400-680`, story `:312`]
- [x] [Review][Patch] **HIGH — les deux tests de transactionnalité passent à vide** : `account_name(&pool, …)` lit depuis une **autre connexion** une transaction **jamais committée**. `assert_eq!(after, before)` et `assert_ne!(after, "TOUCHE PAR LE REJEU")` sont vrais par construction et ne peuvent pas échouer, y compris si le rejeu ne démarrait jamais ou continuait après l'erreur. Le doc-comment revendique pourtant une preuve (« ce qui prouve deux choses à la fois »). Mode d'échec « test muet » de P6. [`crates/kesh-db/tests/post_restore_transactionality.rs:92-142`]
- [x] [Review][Patch] **Le test de fidélité de l'extrait est unidirectionnel et tautologique pour la classe A** : aucune assertion de cardinalité. Supprimer un des 12 `UPDATE` de l'extrait laisse tous les tests verts (les 11 restants sont toujours des sous-textes), et vider l'extrait produit `rows_affected = 0` + un `info!` « rejoué » indiscernable d'un rejeu légitimement sans effet. Pour la classe A, `entry.sql` **est** la source : la comparaison est une tautologie. [`crates/kesh-db/src/post_restore.rs:563-580`]
- [x] [Review][Patch] **Asymétrie du garde-fou : les exemptions « hors fenêtre » ne sont vérifiées par rien** — `registry_entries_are_within_import_window` borne les entrées, mais `exemptions_are_real_and_disjoint_from_registry` ne contrôle que réalité, non-vacuité et disjonction. 5 des 7 justifications invoquent la fenêtre sans qu'elle soit recalculée. Le chemin le moins coûteux pour faire taire `every_data_backfill_migration_is_triaged` est de copier une justification voisine. [`crates/kesh-db/src/post_restore.rs:534-552`]
- [x] [Review][Patch] **Détection `CREATE TABLE` fragile — le garde-fou de fenêtre peut devenir muet** (3 lentilles convergentes) : `up.contains("CREATE TABLE {t}")` exige un espacement d'exactement un blanc et un nom non quoté, sans passer par `normalize()` qui existe pourtant dix lignes plus haut. Une future migration écrivant `` CREATE TABLE `quotes` `` ou avec le nom rejeté à la ligne suivante ne déplacerait pas la fenêtre. Vérifié : aucun backtick aujourd'hui, fenêtre correctement résolue à `20260715000001`. [`crates/kesh-db/src/post_restore.rs:586-606`]
- [x] [Review][Patch] **`split_statements` n'est pas conscient des littéraux, et `/* */` est une hypothèse sans garde-fou** : le dépôt contient **déjà** un `;` dans un littéral (`20260505000001_bank_profiles.sql:39`, `IN (',', ';', '\t')`) qui scinde effectivement son `CREATE TABLE` en deux fragments — inoffensif aujourd'hui, mais l'analyseur produit déjà du SQL incohérent sur un fichier réel. Un futur `/* … ; … */` au-dessus d'un `UPDATE` ferait rendre `false` à `writes_data` et échapper la migration au triage : troisième exemplaire de la classe de défaut que le module recense (multi-ligne, puis `INSERT IGNORE`). [`crates/kesh-db/src/post_restore.rs:474-489`]
- [x] [Review][Patch] **L'erreur de rejeu ne nomme ni l'entrée ni le statement, contredisant son propre commentaire** : le code annonce « le découpage a en outre l'avantage de faire remonter l'erreur avec le statement fautif » puis fait `map_err(map_db_error)?` sans ajouter `label`, `version` ni le statement. Le test l'accepte d'ailleurs en se contentant de `msg.contains("1054")` — un code numérique ne nomme aucun statement. [`crates/kesh-db/src/post_restore.rs:301-308`]
- [x] [Review][Patch] **AC-C6.1 — le message du garde-fou de triage ne nomme ni le fichier ni les issues** : `format!("{}_{}", m.version, m.description)` rend `20260722000001_accounts role postable`, qui n'est le nom d'aucun fichier (`m.description` porte des espaces — le module le sait, `registry_entries_match_a_real_migration` fait `.replace(' ', "_")`). Aucun numéro d'issue dans le message d'assert. [`crates/kesh-db/src/post_restore.rs:519-527`]
- [x] [Review][Patch] **La classe A n'a aucun verrou anti-DDL** : « backfill pur (aucun DDL) » est une affirmation de prose non testée, alors que le fichier d'extrait documente précisément que c'est ce qui casse un rejeu en bloc (erreur MariaDB 1060). La classe B a son verrou (`class_b_sentinel_column_is_added_by_its_own_migration`), la classe A n'en a pas. [`crates/kesh-db/src/post_restore.rs:283-311`]
- [x] [Review][Patch] **`class_b_sentinel_column_is_added_by_its_own_migration` vérifie moins que ce qu'il annonce** : le nom de **table** n'apparaît que dans le message d'erreur, et `contains("ADD COLUMN ROLE")` matche aussi `ADD COLUMN role_label`. Une sentinelle `("invoice_lines", "role")` passerait au seul motif que la migration fait `ALTER TABLE accounts ADD COLUMN role`. [`crates/kesh-db/src/post_restore.rs:592-618`]
- [x] [Review][Patch] **LOW groupés (5)** : (a) l'audit persiste `format!("{:?}", outcome)`, format d'archive non contractualisé, alors que 16-1d va asserter dessus [`crates/kesh-api/src/routes/admin.rs:326`] ; (b) `replay_with_registry` est `pub` en production sans `#[doc(hidden)]` — porte de contournement du registre canonique [`post_restore.rs:377`] ; (c) `LIMIT 1` sans `ORDER BY` face à `assert_eq!(rows_affected, 1)` sur un `UPDATE` non scopé par société [`post_restore_transactionality.rs:47-53`] ; (d) P7 de `CLAUDE.md` omet `LOW_PRIORITY` / `HIGH_PRIORITY` que D-C6 énumère (le code est plus large que la règle écrite, pas l'inverse) ; (e) les Completion Notes revendiquent « discrimination prouvée par mutation, non constatée » sans aucune trace au Debug Log — T6 étant partie en 16-1d, l'affirmation est elle-même constatée et non prouvée.

- [x] [Review][Defer] **`missing_sentinels` : `None => true` choisit le défaut « rejouer » sur table absente** [`crates/kesh-db/src/post_restore.rs:449-452`] — differé, comportement documenté comme défensif et inatteignable (`parse_and_verify` refuse en amont tout manifeste incomplet en tables) ; le basculer en `false` changerait une sémantique verrouillée par `missing_sentinels_uses_or_semantics`.

**Écartés (3)** — `TRUNCATE` non transactionnel qui invaliderait l'atomicité annoncée : **réfuté en ground-truth**, `restore_body` purge par `DELETE FROM \`{table}\`` (`crates/kesh-db/src/backup.rs:428`), le nom `TABLES_TO_TRUNCATE` est un abus de langage préexistant hors diff. — Sensibilité à la casse de `missing_sentinels` : le manifeste est produit par notre propre export depuis `information_schema`, aucun chemin réel. — Skip tracé en `debug!` seulement et branche `Skipped` non couverte : **conforme à AC-C7**, et la couverture de l'`outcome` `Skipped` est explicitement en **16-1d, AC-D1 cas C3**, qui part dans la même PR.

*Passe 2 de `bmad-code-review`, 2026-08-02, Sonnet, 3 lentilles, contexte frais, diff aplati `f699eda5 → 3f20f93d`. 5 findings → 3 retenus : 1 HIGH, 1 MEDIUM, 1 LOW ; 1 MEDIUM écarté avec justification, 1 MEDIUM réfuté.*

- [x] [Review][Patch] **HIGH — le remède documenté déclenche une restauration destructrice intégrale, et aucun des sites qui l'énoncent ne le dit** (EdgeCaseHunter) — *« il suffit de relancer le même import »* est écrit à **quatre** endroits. Le remède **fonctionne** (la sauvegarde ancienne n'a pas les colonnes, donc la sentinelle est absente et le rejeu a lieu), mais un import **remplace l'intégralité de l'installation** par le contenu du fichier : tout ce qui a été saisi depuis le restore fautif est perdu — factures, écritures, contacts. Le manuel porte l'avertissement de destructivité, mais ~30 lignes plus haut et dans un `keshwarning` distinct ; le CHANGELOG ne l'a nulle part à proximité. Et il n'existe **aucune** voie alternative : `replay_with_registry` est `#[doc(hidden)]` et aucun endpoint n'expose le rejeu seul. La formule « il suffit de » minimise activement un coût qui peut valoir des semaines de comptabilité. [`crates/kesh-db/src/post_restore.rs:87`, `CHANGELOG.md:44`, `docs/manual/fr/admin-manual.tex:1615`, story `:322`]
- [x] [Review][Patch] **MEDIUM — `adds_column` analysait le texte brut, faisant reposer le verrou de la classe B sur un accident d'ordonnancement** (BlindHunter) — la fonction appelait `normalize()` (qui ne fait que compacter les blancs) sur le SQL **non décommenté**, là où tous les autres analyseurs du module passent par `split_statements`, qui retire les `--`. Une vingtaine de migrations écrivent `ADD COLUMN` en prose de commentaire, et `20260722000001` — la migration de la **seule** entrée de classe B — le fait deux fois (`:61`, `:67`). Ces deux-là précèdent son premier `ALTER TABLE` littéral, donc le `.skip(1)` les écartait *par chance* ; un commentaire placé après aurait validé une sentinelle qu'aucune colonne ne porte. La portée d'un `ALTER TABLE` était en outre bornée au prochain `ALTER TABLE` **textuel**, donc à la fin du fichier en l'absence d'un second. **Troisième occurrence de la classe de défaut que le doc-module recense** (le grep mono-ligne, puis `INSERT IGNORE`) — et la première introduite par une **remédiation**. [`crates/kesh-db/src/post_restore.rs:1073-1096`]
- [x] [Review][Patch] **LOW — la ligne de suivi affirme « `bmad-code-review` pas encore lancée » dans le commit qui EST la passe 1** (AcceptanceAuditor) — `sprint-status.yaml:259` s'arrête à l'état post-`bmad-dev-story` alors que le diff qui la contient porte l'intégralité de la passe 1. Aucun AC ne couvre ce fichier ; c'est la dérive que la § « Synchroniser TOUTES les docs » de `CLAUDE.md` vise sur les artefacts de suivi. [`_bmad-output/implementation-artifacts/sprint-status.yaml:259`]

**Écarté avec justification (1)** — *aucun test n'exécute le registre réel combiné* (EdgeCaseHunter, MEDIUM). Le fait est **exact et vérifié** : `replay_post_restore_backfills` n'a aucun appelant de test (`grep -rn` → la déclaration et l'unique appel de production), `post_restore_class_a.rs:65` filtre sur `BackfillTrigger::Unconditional` donc exclut l'entrée de classe B, et `post_restore_transactionality.rs` n'utilise que des entrées fictives. Mais c'est le **périmètre du split arbitré en passe 5 de `validate`** : AC-C3 verrouille l'ordre en déclaration (AC-C6.5) **et en exécution par C5**, parti en **16-1d, AC-D1**, avec sa mutation dédiée « registre parcouru en ordre décroissant → C5 doit rougir ». Les deux stories partent dans la **même PR** — le trou se ferme avant son ouverture. *(Il resterait réel si 16-1d ne partait pas avec : c'est une raison de plus de ne pas les séparer.)*

*Passe 3 de `bmad-code-review`, 2026-08-02, Haiku 4.5, 3 lentilles, contexte frais, diff aplati `f699eda5 → a24cd383`. **0 finding au-dessus de LOW — critère d'arrêt de la § Review Iteration Rule atteint.** 6 findings bruts, tous écartés : 5 réfutés en ground-truth, 1 LOW écarté avec justification.*

- [x] [Review][Dismiss] **LOW — le `tracing::info!` du rejeu sérialise l'`outcome` en `Debug`** là où l'audit utilise le code d'archive stable (BlindHunter). **Écarté** : le contrat d'archive posé en passe 1 vise `audit_log`, relu des années après un renommage de variant ; un événement `tracing` est un journal **opérationnel**, où `?` est l'idiome Rust. Surtout, le `Debug` de `ReplayedSentinelsAbsent(Vec<String>)` **porte la liste des sentinelles manquantes**, que `code()` seul perdrait — le « correctif » appauvrirait le log. AC-C7 n'exige que version, issue et lignes touchées. [`crates/kesh-db/src/post_restore.rs:394-400`]

**Réfutés en ground-truth (5) — profil d'hallucination Haiku documenté.** ⚠️ *Les trois findings `CRITICAL`/`HIGH` de cette passe étaient faux, ce qui est exactement le cas de figure que la § « Haiku-specific guardrails » de `CLAUDE.md` prescrit de vérifier avant de patcher.*

1. **CRITICAL — « l'erreur du rejeu ne porte pas le statement fautif ».** Réfuté : le message porte le label, la version, la position **et** le statement, assertés en propre par `replay_stops_at_the_first_failing_entry` (`post_restore_transactionality.rs:224-235` : `fixture:fautif`, `statement #1`, `colonne_inexistante`). Le grief se réduit à ce qu'`elide` borne le statement à 200 caractères — une troncature délibérée, pas une omission. Le rapport se contredit d'ailleurs lui-même : « le label y est, le statement position aussi ».
2. **HIGH — « les deux tests de transactionnalité sont muets, ils lisent depuis le pool ».** Réfuté : `account_name_in_tx` existe (`:80`) et est **utilisé** aux lignes `165` et `212`, dans les deux tests, avec des messages d'assertion qui énoncent pourquoi. Haiku cite « lines 92-142 » et « 134-180 » pour un fichier de **236** lignes : il décrit l'état **d'avant la passe 1**, c'est-à-dire précisément le patch qui a corrigé ce défaut.
3. **HIGH — « les tests de classe A ne détecteraient pas un compte arbitraire ».** Réfuté : `class_a_entries_are_not_vacuous_on_a_pre_migration_base` n'asserte pas que la non-vacuité, il compare la **valeur** posée (`post_restore_class_a.rs:281-285`, `vec![Some(revenue), Some(revenue)]`). La mutation proposée par le rapport (`SELECT id FROM accounts LIMIT 1`) rendrait le compte `1000` et ferait rougir cette assertion.
4. **MEDIUM — « une sentinelle `postab` validerait `postable` ».** Réfuté par lecture : la comparaison est une **égalité** (`== Some(column_up.as_str())`), pas un test de préfixe — c'est le patch de la passe 1, qui a justement remplacé un `contains`.
5. **MEDIUM — « le compteur `checked == 4` est un crime, le remplacer par `> 0` ».** Écarté, et le remède serait une **régression** : c'est ce compteur qui force la relecture, une 5ᵉ exemption « Hors fenêtre » faisant passer `checked` à 5 et rougir le test. Avec `> 0`, plus rien ne signalerait le changement. Le trou résiduel décrit (une exemption qui *devrait* invoquer la fenêtre mais omet le marqueur) est la contrepartie **explicitement documentée** au garde-fou P7 de `CLAUDE.md`.

**Réfuté (1)** — *le remède documenté ne répare pas un parc cassé* (BlindHunter, MEDIUM). L'affirmation centrale est fausse : la phrase s'adresse à qui « a restauré une sauvegarde **avant** cette version », donc « le même import » désigne la sauvegarde **ancienne**, dont les colonnes manquent — sentinelle absente, rejeu effectué. Le doc-module dit d'ailleurs la même chose (`:87`). Le scénario du rapport (backup pris *depuis* le parc cassé) n'est pas celui que le texte vise. Restait un résidu réel — la formule « un parc restauré se retrouve dans le même état qu'un parc mis à jour » est absolue et admet le contre-exemple du restore chaîné — **repris dans le patch du HIGH**, qui touche les mêmes phrases.

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

Le mode d'échec ne naît ni du code écrit ni de la spec, mais de l'**interaction** entre une migration ajoutée par une story et un chemin de restore que cette story ne touche pas. C'est le profil du garde-fou **P6** (couplage positionnel des migrations), codifié en 16-1a après que trois tests ont changé de sens sans qu'aucune ligne de leur fichier ne bouge — dont un **passé à vide**. D'où **16-1d T-D3** : prouver par **mutation** que les cas discriminants discriminent.

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

**Claude Opus 5 (1M context)** — `bmad-dev-story`, 2026-08-01.

Deux commits, un par lot de tâches :

| Commit | Date | Tâches |
|---|---|---|
| `586bcd46` | 2026-08-01 15:53 | T1 — T4 (registre, extrait SQL, câblage, garde-fous) |
| `0387b9a0` | 2026-08-01 16:18 | T5 + T7 (transactionnalité, documentation) |

*(T6 — preuve par mutation — est partie en **16-1d** avec les cas qu'elle vise, au split arbitré en passe 5. T8 ci-dessous.)*

**La session de dev a été interrompue par un crash après `0387b9a0`.** Le Dev Agent Record, les cases de tâches, le `Status` et `sprint-status.yaml` ont été rattrapés le 2026-08-02 après vérification d'intégrité : `git fsck` 0 erreur, arbre propre, `fmt` / `build` / `clippy -D warnings` verts, 9/9 garde-fous `post_restore` verts. **Aucun travail n'a été perdu** — les deux commits contiennent l'intégralité du code livré.

⚠️ **Mais « 9/9 verts » ne disait rien des tests qui n'existaient pas.** La passe 1 de `bmad-code-review` a établi que T4, cochée, exigeait **8** tests précis (1 parsing + 6 d'AC-C6 + 1 d'AC-C5) dont **2 manquaient** — le no-op nominal et la non-vacuité de la classe A. Les 9 tests livrés en comptaient **3 hors AC**, et c'est ce passage de 8 à 9 qui a rendu la lacune invisible. Un compte de tests n'est un indicateur de couverture que si sa **composition** est vérifiée. Corrigé en passe 1 de revue ; T4 porte désormais ses 8 tests requis.

### Debug Log References

**Gate T8 — `scripts/test-fast.sh` sur l'état final, 2026-08-02, DB `kesh_gate` : VERT.** `exit 0`, **2086/2086**, 4 skipped, 9 slow, 3484 s (58 min). `fmt` + `clippy -D warnings` verts dans le même run. Verdict pris sur l'**état final**, non présumé d'un run antérieur.

⚠️ **Un premier run a rendu `exit 100` et n'est PAS une régression** : 6 échecs, tous `PoolTimedOut` à 30,0 s sur *« failed to connect to setup test database »*, aucun sur une assertion. Le run tournait dans le bac à sable de l'outillage, qui bloque `127.0.0.1:3306` — vérifié directement (`/dev/tcp/127.0.0.1/3306` → connexion refusée). **Aucun test touchant la base n'avait démarré** ; les 14 tests verts qui précèdent sont des tests unitaires purs. Relancé hors bac à sable.

**Deux pièges d'exécution du gate, tous deux rencontrés ici :**

1. **L'exit code d'un `script > log ; echo "EXIT=$?"` est celui du `echo`.** La notification de fin annonçait `exit 0` alors que le gate valait **100**. Le verdict se lit dans le log, jamais dans le code de retour du bloc englobant. *(Même famille que [[feedback_cargo_test_pipe_masks_exit]] : ne pas laisser une commande de confort masquer le statut réel.)*
2. **`kesh_gate` contrôlée AVANT le run** (5 comptes `active=1, postable=1`) — un run nextest tué en vol y laisse le compte `1000` non imputable et produit 26 faux échecs dans `journal_entries::tests`. Le crash de la veille n'avait rien laissé.

### Completion Notes List

- **Le registre compte 2 entrées, et son contenu n'est pas fixé par le nombre de backfills.** Il l'est par la **fenêtre d'importabilité** : `parse_and_verify` exige l'égalité exacte, dans les deux sens, entre les tables du manifeste et `TABLES_TO_TRUNCATE`, et aucune migration ne supprimant jamais de table, un backup n'est importable que depuis un binaire ≥ à la dernière migration créatrice de table applicative. Sur les **9** migrations qui écrivent des données, **7 sont exemptées** avec justification écrite (`EXEMPT_MIGRATIONS`), dont une **autoréfutante** : `20260628000001` crée elle-même les tables dont l'absence accompagnerait celle de sa colonne.
- **Écart de conception assumé — exécution statement par statement plutôt que `sqlx::raw_sql`.** Le futur de `raw_sql` n'est pas `Send`, ce qui casse la contrainte `Handler` d'Axum sur le handler d'import. Le découpage a de surcroît l'avantage de faire remonter l'erreur **avec le statement fautif**, ce dont dépend l'assertion de `replay_stops_at_the_first_failing_entry`.
- **La classe est déclarée, jamais devinée** (D-C1). Une détection textuelle par présence de `IS NULL` / `NOT EXISTS` classerait à tort le premier `UPDATE` de `postable` de `20260722000001` : son `NOT EXISTS` est un prédicat **structurel de ciblage**, pas une garde d'idempotence.
- **`20260729000001` est en classe A et non B**, bien qu'elle porte une garde `IS NULL` : sa colonne est créée par une migration **distincte** (`20260727000001`, DDL pur), donc une sentinelle mentirait sur un backup pris entre les deux, qui porte la colonne entièrement `NULL`. L'invariant est verrouillé par `class_b_sentinel_column_is_added_by_its_own_migration`.
- **`replay_with_registry` est `pub` pour une raison testable, pas par confort.** `#[cfg(test)]` ne traverse pas la frontière de crate : une entrée fautive déclarée ainsi dans `kesh-db` ne serait vue par **aucun** test d'intégration de `kesh-api`, et la déclarer inconditionnellement livrerait en production une entrée dont le seul rôle est de faire échouer tout restore. C'est la porte d'injection qui rend AC-C4 testable.
- **Le corps de réponse HTTP est inchangé** (AC-C8, D-C7) : le rapport de rejeu vit dans la clé `backfills_replayed` du détail d'audit et dans le journal serveur.
- **`rows_affected` est informatif et ne fonde aucune assertion de succès** — zéro est un résultat nominal, le backfill de `20260729000001` étant délibérément incomplet (AC-B2). C'est documenté au champ pour qu'aucun appelant n'en fasse un indicateur de santé.
- ~~**Discrimination prouvée par mutation, non constatée**~~ — **affirmation retirée à la passe 1 de `bmad-code-review`.** Elle était elle-même *constatée et non prouvée* : la liste de mutations qu'elle énumérait n'a aucune trace au Debug Log, et T6 — la tâche qui porte la preuve par mutation — est partie en **16-1d**. Les mutations réellement exécutées sont celles de la passe de revue, consignées au Change Log ci-dessous avec leur sortie.
- **La pré-condition de C4 n'est pas décorative** — mais sa rédaction d'origine ne la rendait pas opérante. Elle vivait dans un **autre test**, donc sur une **autre base éphémère**, et ne disait rien de l'exécution en cours ; l'assertion qu'elle prétendait garder lisait par ailleurs depuis le pool une transaction jamais committée, donc ne pouvait pas échouer. Corrigé en passe 1 de revue : chaque test constate le travail déjà fait **dans sa propre transaction**, avant l'abandon.
- **AC-C9 vérifié négativement** : aucune migration n'étant ajoutée, `docs/migrations-idempotence-audit.md` n'est **pas** touché — 57 lignes, compteurs `57 / 5 / 52 / 0` **recomptés** depuis le disque et le tableau, zéro écart.
- **Macros de version des manuels NON touchées** (`kesh-style.sty`) : elles ne bougent qu'au tag de release, gate 4-bis.

### File List

**Créés**

- `crates/kesh-db/src/post_restore.rs` (1191 l.) — registre `POST_RESTORE_BACKFILLS` (2 entrées), `EXEMPT_MIGRATIONS` (7 entrées justifiées), types `BackfillTrigger` / `ReplayOutcome` (+ `code()` / `missing_sentinels()`, contrat d'archive de l'audit) / `PostRestoreBackfill` / `ReplayedBackfill`, `replay_post_restore_backfills`, `replay_with_registry`, `missing_sentinels`, `split_statements`, `elide`, `literal_hazards`, `writes_data`, et **17 tests** de garde-fou.
- `crates/kesh-db/src/post_restore/20260722000001_accounts_role_postable.sql` (48 l.) — extrait verbatim des 12 `UPDATE` (10 rôles + 2 `postable`), dans l'ordre du fichier source.
- `crates/kesh-db/tests/post_restore_transactionality.rs` (236 l.) — cas **C4** + sa pré-condition + la borne d'arrêt au premier statement fautif.
- `crates/kesh-db/tests/post_restore_class_a.rs` (367 l., **ajouté en passe 1 de `bmad-code-review`**) — **AC-C6.3** (no-op sur une base nominale à jour) et **AC-C6.4** (non-vacuité sur une base en amont), les deux tests qu'exigeait T4 et qui manquaient.

**Modifiés**

- `crates/kesh-db/src/lib.rs` — déclaration du module (`pub mod post_restore;`, inséré entre `pool` et `repositories`).
- `crates/kesh-api/src/routes/admin.rs` — étape **5-bis** de `run_backup_and_restore` (après la garde de comptage, avant l'audit) + clé `backfills_replayed` du détail d'audit.
- `CHANGELOG.md` — la phrase « la reprise ne se rejoue pas après un import » devenait fausse ; remplacée, et une puce dédiée décrit le correctif côté utilisateur.
- `CLAUDE.md` — garde-fou **P7** (§ Migration breaking policy).
- `docs/manual/fr/admin-manual.tex` + `.pdf` — paragraphe sur le rejeu dans la section import, PDF régénéré.

**Passe 2 de `bmad-code-review` — correctifs**

- `crates/kesh-db/src/post_restore.rs` — `adds_column` analyse désormais le SQL **décommenté et découpé** (`split_statements`) au lieu du texte brut, ce qui borne au passage la portée d'un `ALTER TABLE` au `;` qui le termine ; `adds_column_discriminates` reçoit le cas du `ADD COLUMN` **en prose de commentaire placé après le DDL** — la position que l'ordonnancement ne protégeait pas.
- `crates/kesh-db/src/post_restore.rs` (doc-module) + `CHANGELOG.md` + `docs/manual/fr/admin-manual.tex` (+ `.pdf` régénéré) + ce fichier (T7) — les **quatre** sites du remède portent maintenant son coût : l'import remplace l'intégralité de l'installation. Le manuel passe du `keshtip` au `keshwarning`.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — entrée datée de la boucle de revue, préfixée plutôt que substituée à l'ancienne.

## Change Log

### Passe 3 de `bmad-code-review` — **BOUCLE CONVERGÉE**

**2026-08-02 — Haiku 4.5, 3 lentilles, contexte frais, diff APLATI `f699eda5 → a24cd383`. 6 findings bruts, 0 retenu : 5 réfutés en ground-truth, 1 LOW écarté avec justification. Critère d'arrêt de la § *Review Iteration Rule* atteint** (plus rien au-dessus de LOW ; plafond de 8 passes jamais approché).

**Trend des trois passes** : `1 CRITICAL / 1 HIGH / 8 MEDIUM / 5 LOW` (Opus) → `0 / 1 HIGH / 1 MEDIUM / 1 LOW` (Sonnet) → `0 / 0 / 0 / 0` (Haiku). Décroissance monotone, rotation complète Opus → Sonnet → Haiku.

**Le fait marquant : les trois findings `CRITICAL`/`HIGH` de cette passe étaient FAUX, et deux d'entre eux visaient les patches de la passe 1.** C'est le cas de figure exact que la § « Haiku-specific guardrails » de `CLAUDE.md` prescrit de vérifier avant de patcher — appliquée ici, elle a évité trois remédiations sur du code sain, dont deux qui auraient **défait** des correctifs.

Le plus instructif est le HIGH sur la transactionnalité : Haiku affirme que les deux tests lisent depuis le pool une transaction jamais committée — c'était vrai **avant** la passe 1, et c'est précisément ce qu'elle a corrigé. Il cite « lines 92-142 » et « 134-180 » pour un fichier qui en compte **236**, et `account_name_in_tx` est utilisé aux lignes **165** et **212**. Le modèle a reconstruit un état antérieur du fichier et l'a décrit comme actuel. **Un diff aplati n'a donc pas suffi à supprimer le mode d'échec** — la mitigation réduit la confusion d'indexation entre commits, elle ne protège pas d'une reconstruction de mémoire. Seul le `grep -nF` l'attrape.

⚠️ **Le CRITICAL se contredisait dans son propre corps** — titre : « l'erreur ne porte ni l'entrée ni le statement » ; texte : « le label y est, le statement position aussi ». Un rapport dont le corps réfute le titre est un signal à part entière.

**Le seul MEDIUM dont le diagnostic tenait proposait un remède régressif.** Haiku qualifie l'`assert_eq!(checked, 4)` de « crime » et demande un `checked > 0`. Or ce compteur est ce qui **force la relecture** : une 5ᵉ exemption « Hors fenêtre » le fait passer à 5 et rougir le test. Le remède aurait rendu muet un garde-fou qui ne l'est pas. *(Retenir le diagnostic et écarter le remède est un geste déjà consigné en passe 4 de la revue de 16-1a.)*

**Deux lentilles sur trois rendent 0 finding, et leurs rapports sont opposables.** L'AcceptanceAuditor énumère les **onze** AC avec leur preuve et redonne la table des 8 tests exigés par T4 avec leur fichier — le contrôle de composition refait une troisième fois, par un troisième modèle. L'EdgeCaseHunter énumère **sept** axes de bord (déclencheur, découpage SQL, transactionnalité, registre, classe A, classe B, fidélité de l'extrait) avec les ancres de leurs tests. C'est l'inverse des rapports Haiku vides de la 16-1a : la contre-mesure « exiger une section énumérée » continue de produire son effet.

*(Inexactitude sans conséquence dans le rapport de l'AcceptanceAuditor : il compte le coût du remède énoncé à **trois** sites là où le patch en a traité **quatre** — il omet le doc-comment du module, hors de son mandat.)*

**Aucun patch appliqué, donc aucun gate à rejouer** : le dernier verdict vaut pour l'état livré (`a24cd383`, gate complet vert `2096/2096`, exit 0, 3372 s, DB `kesh_gate`, `fmt` + `clippy -D warnings` inclus). Cette passe est un commit **doc-only**, cas explicitement prévu par la § « Test Locally First ».

**Statut** : story maintenue à `review` — le passage à `done` suit le merge de la PR groupée, comme pour 16-1a, 16-1a-bis et 16-1b. **Reste `bmad-dev-story` 16-1d**, sans laquelle cette story ne doit pas partir : c'est elle qui porte le verrou d'exécution de l'ordre (C5) et la couverture de bout en bout.

### Passe 2 de `bmad-code-review`

**2026-08-02 — Sonnet, 3 lentilles (BlindHunter, EdgeCaseHunter, AcceptanceAuditor), contexte frais, diff APLATI `f699eda5 → 3f20f93d`. 5 findings → 3 retenus : 0 CRITICAL, 1 HIGH, 1 MEDIUM, 1 LOW ; 1 MEDIUM écarté avec justification, 1 MEDIUM réfuté en ground-truth.**

**Le HIGH est un défaut de documentation à conséquence destructrice, et c'est l'EdgeCaseHunter seul qui l'a vu.** Le remède du restore chaîné — *« il suffit de relancer le même import »* — est écrit à **quatre** endroits. Il **fonctionne**, mais au prix d'un import complet, donc de la perte de tout ce qui a été saisi depuis le restore fautif ; aucun des quatre sites ne l'énonce, et il n'existe aucune voie qui rejouerait **sans** restaurer (`replay_with_registry` est `#[doc(hidden)]`, aucun endpoint ne l'expose). Les quatre sites portent désormais le coût, et le manuel passe du `keshtip` au `keshwarning` — un conseil qui peut détruire des semaines de comptabilité n'est pas une astuce.

⚠️ **BlindHunter avait visé le même texte et manqué la cible.** Il soutenait que le remède **ne marche pas** — réfuté : la phrase s'adresse à qui a restauré une sauvegarde *avant* cette version, donc « le même import » désigne la sauvegarde **ancienne**, dont les colonnes manquent, donc la sentinelle est absente et le rejeu a lieu. Ce qui manquait n'était pas l'efficacité du remède mais **son prix**. Deux lentilles sur le même paragraphe, une seule formulation juste : c'est la valeur de la redondance, à condition de vérifier laquelle des deux tient.

**Le MEDIUM est le troisième exemplaire d'une classe de défaut que le module recense lui-même — et le premier qu'une remédiation ait introduit.** `adds_column`, réécrit en passe 1 précisément pour cesser d'être un `contains` déguisé, analysait le SQL **brut** quand tous les autres analyseurs du module passent par `split_statements`, qui retire les `--`. Une vingtaine de migrations écrivent `ADD COLUMN` en prose de commentaire ; `20260722000001`, migration de la **seule** entrée de classe B, le fait deux fois. Le verrou tenait par **accident d'ordonnancement** : ces deux commentaires précèdent son premier `ALTER TABLE` littéral. Un commentaire placé après aurait validé une sentinelle qu'aucune colonne ne porte — et la classe B aurait perdu sa condition de validité en silence.

**Discrimination prouvée par mutation, sortie à l'appui.** Rétablir l'analyse du texte brut fait rougir `adds_column_discriminates` — **et lui seul**. `class_b_sentinel_column_is_added_by_its_own_migration` **reste vert** sous la mutation, ce qui *est* le finding : le verrou réel passait pour la mauvaise raison. Le cas ajouté place délibérément le commentaire **après** le DDL, la position que l'ordonnancement ne protégeait pas.

**Écarté avec justification (1).** Aucun test n'exécute le registre réel **combiné** — vérifié (`replay_post_restore_backfills` sans appelant de test, `post_restore_class_a.rs` filtrant la classe A, transactionnalité sur entrées fictives). C'est le périmètre du **split arbitré** : AC-C3 verrouille l'exécution par **C5**, parti en 16-1d avec sa mutation « registre parcouru en ordre décroissant ». Même PR, donc trou fermé avant ouverture — et une raison de plus de ne pas séparer les deux stories.

**L'AcceptanceAuditor rend 1 LOW et son rapport est opposable.** Il a refait **sans les présumer** les deux contrôles que la passe 1 avait rendus suspects : la **composition** de T4 (les 8 tests exigés ont chacun leur implémentant — l'erreur de comptage du CRITICAL de la passe 1 n'est pas reproduite) et la correspondance entre le garde-fou **P7** de `CLAUDE.md` et ce que le code fait réellement. Il a recompté indépendamment les compteurs de l'audit d'idempotence (57/5/52/0) et la fenêtre d'importabilité (`20260715000001`).

**Propagation post-patch — le grep a rapporté un quatrième site que les trois lentilles avaient manqué.** Les rapports citaient trois porteurs du remède (module, CHANGELOG, manuel) ; `grep -rn "relancer le même import\|réimporter le backup"` sur tout le dépôt en a rendu un **quatrième**, la tâche **T7 du story file** (`:322`), qui prescrivait la formulation fautive à qui rédigerait la doc. Traité dans le même patch. Contrôlés sans propagation : les manuels DE/IT/EN (vides en v0.1), `website/`, les quatre locales `.ftl`, le manuel utilisateur (le rejeu est une opération d'administration, il n'y figure pas).

**Trend** : passe 1 `1 CRITICAL / 1 HIGH / 8 MEDIUM / 5 LOW` → passe 2 `0 / 1 HIGH / 1 MEDIUM / 1 LOW`. **Boucle NON convergée** — un HIGH et un MEDIUM subsistent, donc la § *Review Iteration Rule* impose une **passe 3**, LLM ≠ Sonnet, contexte frais.

**Sur la règle de splitting** : le second critère n'est **pas** coché — la sévérité décroît nettement (`CRITICAL → HIGH`, et le volume passe de 13 findings retenus à 3). Rien à arbitrer.

### Passe 1 de `bmad-code-review`

**2026-08-02 — Opus 5, 3 lentilles (BlindHunter, EdgeCaseHunter, AcceptanceAuditor), contexte frais, diff APLATI `f699eda5 → a06049d6`** (garde-fou § *Haiku-specific guardrails* — un diff unique plutôt que la séquence de commits). **30 findings bruts → 13 après fusion et vérification ground-truth : 1 CRITICAL, 1 HIGH, 8 MEDIUM, 5 LOW (groupés), 1 décision, 1 différé, 3 écartés.**

**CRITICAL — AC-C6.3 et AC-C6.4 n'étaient pas implémentés, T4 était cochée.** T4 énonce littéralement « les **6** tests d'AC-C6 (triage, DDL même fichier, **no-op nominal**, **non-vacuité**, ordre croissant, fenêtre) » ; le code en portait 4. La seule entrée de **classe A** du registre — rejouée **inconditionnellement à chaque import** — n'avait donc aucune preuve automatisée qu'elle ne réécrit pas une donnée établie. C'est exactement le mode d'échec qui avait produit le CRITICAL de la passe 2 de spec (`20260628000001`, qui satisfaisait « idempotent au second passage » et violait « no-op sur une base nominale »). Ajout de `crates/kesh-db/tests/post_restore_class_a.rs`.

⚠️ **Le montage d'AC-C6.3 a lui-même failli être muet.** Avec des lignes de facture portant *le même* compte que celui que le backfill écrirait, MariaDB ne compte dans `rows_affected` que les lignes **réellement modifiées** : un rejeu ayant perdu sa garde `IS NULL` aurait réécrit `3000` sur `3000` et rapporté `0`. La fixture porte donc un compte **différent** (`3200`) sur les lignes — état nominal et documenté — sans quoi le test aurait eu la forme d'une preuve sans en être une.

**HIGH — les deux tests de transactionnalité passaient à vide, et c'est démontré.** `account_name(&pool, …)` lit depuis une **autre connexion** une transaction **jamais committée** : les assertions « la destination est inchangée » et « l'entrée suivante n'a pas tourné » étaient vraies par construction. Prouvé empiriquement : en injectant le bug (itération du registre inversée, donc l'entrée placée après la fautive s'exécute quand même), **l'assertion dans sa forme d'origine passe — `exit 0` — et dans sa forme corrigée elle échoue**. Chaque test lit désormais **dans sa propre transaction** ce qui n'y est observable que là.

**MEDIUM appliqués (8).** Fidélité de l'extrait rendue **bidirectionnelle** (`extract_carries_every_write_statement_of_its_source_migration`) — retirer un `UPDATE` de l'extrait laissait l'ancien test verbatim vert, et vider l'extrait passait à vide ; **symétrique du garde-fou de fenêtre côté exemptions** (`exemptions_claiming_out_of_window_really_are_out_of_window`), l'exemption étant l'issue la moins coûteuse du triage donc celle qu'il faut contrôler, avec marqueur textuel `Hors fenêtre` normalisé sur les 4 justifications concernées ; détection `CREATE TABLE` **normalisée en blancs et déquotée**, et comparant le nom **entier** (`supplier_invoice_lines` ne compte plus comme `supplier_invoices`) ; les deux limites de `split_statements` **outillées** au lieu d'être déclarées (`migrations_contain_no_block_comment`, `registry_sql_has_no_literal_hazard`) ; l'erreur de rejeu **nomme** désormais l'entrée, la position du statement et le statement lui-même, ce que son propre commentaire promettait déjà ; le message du garde-fou de triage rend un **nom de fichier grepable** (`m.description` porte des espaces) et cite les issues ; **verrou anti-DDL de la classe A** (`registry_sql_contains_no_ddl`), symétrique de celui de la classe B ; `class_b_sentinel_column_is_added_by_its_own_migration` vérifie le couple **(table, colonne)** sur le nom entier, là où un `contains("ADD COLUMN ROLE")` validait `role_label` et ignorait la table.

**LOW appliqués (5).** `outcome` de l'audit passé d'un `format!("{:?}")` d'enum à un **code d'archive stable** (`ReplayOutcome::code()`, + `missing_sentinels()`) — l'audit est relu des années après, un renommage de variant y faisait dériver des enregistrements déjà écrits ; `replay_with_registry` marquée `#[doc(hidden)]` (porte de test, pas API) ; `LIMIT 1` retiré au profit d'une borne explicite `assert_single_account` ; `CLAUDE.md` P7 complété (`LOW_PRIORITY` / `HIGH_PRIORITY`, et mention du symétrique des exemptions) ; revendication « discrimination prouvée par mutation » **retirée** des Completion Notes — elle était constatée et non prouvée, T6 étant partie en 16-1d.

**Décision de Guy — restore chaîné : documenter la limite, ne pas restructurer le registre.** Un backup pris *depuis* un parc déjà cassé par un restore antérieur porte les colonnes `role` / `postable` sans leur contenu : la sentinelle le déclare à jour et l'entrée est `Skipped`. L'option écartée — scinder l'entrée en classe A pour les 10 `UPDATE` de rôle — aurait contredit **D-C1**, qui établit que `role IS NULL` n'est *pas* une garde d'intention. Limite inscrite à la § « Ce que ce mécanisme n'est PAS ».

**Différé (1).** `missing_sentinels` rend `true` sur une table absente, c'est-à-dire *rejouer* — défaut discutable pour une garde anti-écrasement, mais chemin inatteignable (`parse_and_verify` refuse en amont). Tracé dans `deferred-work.md`.

**Écartés (3), dont un HIGH réfuté en ground-truth.** « `TRUNCATE` est du DDL et provoque un commit implicite, donc l'atomicité annoncée au CHANGELOG et au manuel est fausse » — **réfuté** : `restore_body` purge par `DELETE FROM \`{table}\`` (`crates/kesh-db/src/backup.rs:428`), `TABLES_TO_TRUNCATE` est un abus de langage préexistant hors diff. Sensibilité à la casse de `missing_sentinels` : le manifeste vient de notre propre export depuis `information_schema`. Skip tracé en `debug!` seulement et branche `Skipped` non couverte : **conforme à AC-C7**, la couverture de cet `outcome` étant explicitement en **16-1d, AC-D1 cas C3**.

**Discrimination des nouveaux garde-fous — prouvée par mutation, sortie à l'appui.** Chaque mutation tue son test **et lui seul** :

| Mutation | Test qui rougit |
|---|---|
| retirer un `UPDATE` de l'extrait | `extract_carries_every_write_statement_…` (l'ancien test verbatim reste **vert** — c'est le finding) |
| vider l'extrait | `extract_carries_every_write_statement_…` |
| glisser un `ALTER TABLE` dans le SQL rejoué | `registry_sql_contains_no_ddl` |
| exemption « Hors fenêtre » sur une version postérieure à la fenêtre | `exemptions_claiming_out_of_window_…` |
| retirer le marqueur « Hors fenêtre » d'une justification | `exemptions_claiming_out_of_window_…` |
| retirer la garde `WHERE il.revenue_account_id IS NULL` du backfill | `class_a_entries_are_no_ops_on_a_nominal_up_to_date_base` |
| rendre le SQL de classe A sans cible | `class_a_entries_are_not_vacuous_on_a_pre_migration_base` |
| itérer le registre à l'envers | `replay_stops_at_the_first_failing_entry` |

Les deux tests de classe A sont **complémentaires et non redondants** : chaque mutation en tue exactement un, l'autre restant vert.

**Contrôles positifs conduits par l'orchestrateur.** Recompte indépendant des migrations écrivant des données : **9** (7 exemptées + 2 au registre), conforme au § Contexte. Justification d'exemption de `20260714000002` vérifiée malgré un nom de fichier trompeur (`email_templates_reminder`) : son unique statement d'écriture est bien le bump de `_kesh_version`. Fenêtre d'importabilité **recalculée après réécriture du détecteur** : inchangée à `20260715000001`, et les **37** tables de `TABLES_TO_TRUNCATE` sont toutes couvertes. `FOREIGN_KEY_CHECKS = 1` au moment du rejeu : confirmé, un échec de rétablissement est une erreur dure (`backup.rs:412-417`).

**Propagation post-patch — le grep du symptôme a rapporté deux sites que les lentilles n'avaient pas vus.** (1) **16-1d** allait asserter sur le format d'audit modifié par le patch LOW : sa tâche T-D2 porte désormais la distinction entre l'enum `ReplayOutcome` (valeur de retour) et le **code d'archive stable** (`audit_log`), avec l'interdiction explicite d'asserter un `format!("{:?}")`. (2) **`bank_imports_repository.rs:168-180` porte le même patron de test muet** — `drop(tx)` puis `SELECT COUNT(*)` depuis le pool asserté à `0` sous le message « rollback : aucune entête en DB », vrai par construction. Hors du diff de cette story → **différé** et tracé, avec `reconciliation_rules_repository.rs` vérifié sain au passage (ses assertions post-`drop(tx)` portent sur le `Result`). Les autres symptômes grepés sont sans propagation : plus aucun autre site ne reconstruit un nom de fichier de migration, ni ne détecte `CREATE TABLE` / `ADD COLUMN` par `contains`, et les `format!("{:?}")` restants de `kesh-api` sont dans des **corps de réponse**, pas des archives.

**Troisième piège d'exécution du gate, rencontré ici.** Un premier run a rendu **1 échec** — `journal_entries::tests::test_check_constraint_rejects_debit_and_credit_same_line`, `InactiveOrInvalidAccounts` — et **ce n'est pas une régression** : le run visait `DATABASE_URL=…/kesh`, la base de **dev**, où le compte `1000` porte `postable = 0`. C'est exactement la pollution décrite au piège n° 2 ci-dessus, dont le remède est d'utiliser la base de gate `kesh_gate`, vérifiée dans son état nominal (5 comptes `active=1, postable=1`) avant relance. **La leçon à retenir n'est pas « reseeder » mais « viser la bonne base »** : le piège n° 2 disait de contrôler `kesh_gate` avant le run, il ne disait pas que s'en écarter reproduit le symptôme à l'identique.

**Trend** : passe 1 → 1 CRITICAL, 1 HIGH, 8 MEDIUM, 5 LOW. **Boucle NON convergée** — une passe 2 est requise par la § *Review Iteration Rule*, avec un LLM différent et un contexte frais.


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

### Passe 3 de `bmad-create-story validate`

**2026-08-01 — Haiku, 3 lentilles, contexte frais. 3 findings retenus : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 1 LOW** — plus **1 HIGH réfuté en ground-truth**.

La sévérité redescend nettement (`CRIT → CRIT → MED`), et **les deux MEDIUM sont l'un et l'autre des résidus de ma refonte de passe 2** — la remédiation reste la première source de défauts, troisième passe consécutive sur cette story.

- **AA3-1 (MEDIUM)** — le test de sous-texte verbatim avait **perdu sa tâche**. T4 se terminait par « les 6 tests d'AC-C6 », or ce test relève d'**AC-C5** ; l'en-tête de T4 citait bien AC-C5, aucune de ses puces ne l'implémentait. La rédaction d'avant la refonte le nommait explicitement. AC-C5 était donc un critère sans exécutant.
- **AA3-2 (MEDIUM)** — `trigger` désignait **deux choses différentes** sous un même nom : la **déclaration** du registre (`BackfillTrigger`, 2 variants) et l'**issue** d'exécution rapportée par `ReplayedBackfill`, que D-C5 décrit avec **trois** états dont « sauté ». Le troisième état était inencodable, et le `tracing::debug!` du skip n'avait aucun état correspondant. Séparé en `BackfillTrigger` (déclaration) et `ReplayOutcome` (issue) ; assertion `Skipped` exigée sur C3, sans quoi cet état ne serait vérifié nulle part. **Propagation** : le renommage avait laissé un site en arrière dans T5, attrapé par le grep post-patch et non par les lentilles.
- **BH3-1 (LOW, reclassé depuis HIGH)** — annoncé comme une contradiction entre la ligne C5 du tableau (« backup **sans** `postable` ») et sa note de montage (« **Présent** dans le backup, il vaut déjà `FALSE` »). **Réfuté** : le tableau décrit le backup *tel qu'il est importé*, après `strip_column`, et la note énonce un **contrefactuel** expliquant pourquoi le retrait est obligatoire — elle commence d'ailleurs par « retirer `postable` est indispensable ». Aucun développeur lisant les deux phrases ensemble ne construirait le mauvais backup. Conservé en LOW parce que la tournure participiale française est authentiquement ambiguë : reformulée en « s'il était laissé dans le manifeste, il vaudrait déjà `FALSE` ».

**L'EdgeCaseHunter rend 0 finding, et son rapport est opposable** : sa section « vérifié et jugé sain » énumère **12** contrôles avec leur commande — fenêtre d'importabilité, absence de `DROP TABLE`, décomptes, absence de garde `IS NULL` sur les deux `postable`, `NOT EXISTS` structurel du premier, création de `revenue_account_id` dans une migration distincte, séquence rôle → `postable` interne à `20260722000001`, exemption `users.company_id` démontrée sans cas rattrapable. C'est l'inverse des rapports Haiku vides de la 16-1a : la contre-mesure « exiger une section énumérée » produit l'effet attendu.

**Prochaine** : passe 4, LLM ≠ Haiku, contexte frais.

### Passe 4 de `bmad-create-story validate`

**2026-08-01 — Sonnet, 2 lentilles (BlindHunter, AcceptanceAuditor), contexte frais. 6 findings : 0 CRITICAL, 1 HIGH, 2 MEDIUM, 3 LOW.**

**Deux lentilles et non trois, délibérément** : l'EdgeCaseHunter de la passe 3 a rendu 0 finding avec 12 contrôles énumérés, après deux passes dont les matrices d'états couvraient chacune ~25 lignes. Le gisement résiduel n'était plus l'exploration d'états mais les résidus de refonte. *(Si une passe ultérieure remonte un état non traité, remettre la troisième lentille.)*

**Le HIGH, convergé sur les deux lentilles, tue le montage du seul test qui verrouille l'invariant d'ordre.** La note de C5 prescrivait de rééditer l'écriture de la facture via `PUT /api/v1/journal-entries/{id}` pour porter le crédit sur `2979`, en invoquant `enforce_postable = false`. **Trois faits indépendants le réfutent** :

- `journal_entries::update` passe `enforce_postable = **true**` en dur (`:936`), et le commentaire du fichier l'énonce lui-même : « L'update est toujours un flux MANUEL » (`:913`) ;
- son *grandfather* D-A1 n'exempte que les comptes **déjà référencés par l'écriture** (`:921`) — `2979` n'y est jamais, l'écriture de validation créditant le compte de produit ;
- et `2979` **naît** non imputable : `effective_postable` (`accounts.rs:126-131`) force `postable = false` dès que `role = CurrentYearResult`, à la création comme à l'update, et `bulk_create_from_chart` (`:836-852`) l'applique au seed. **Aucun chemin API ne peut le rendre imputable.**

Le `enforce_postable = false` invoqué existe bien, mais sur le chemin **automatique** de validation de facture (`invoices.rs:1798`) — la note l'avait attribué au mauvais site. Montage corrigé : la ligne d'écriture est posée par `sqlx::query` **directement sur le pool de la base source**, cohérent avec le principe déjà établi « muter la base avant l'export ».

**AA4-2 (MEDIUM) — l'injection de faute de C4 aurait cassé les six autres cas.** `replay_post_restore_backfills` ne prend pas le registre en paramètre et lit le `const` global : une entrée fautive en `#[cfg(test)]` serait vue par **tous** les appels du binaire de test. Et C4 étant un test HTTP de bout en bout, on ne peut pas lui injecter un registre. L'isolation passe par le mécanisme déjà en place — entrée fautive en **classe B**, sentinelle sur une colonne que **seul C4** retire.

**BH4-2 (MEDIUM)** : l'en-tête de T3 omettait `AC-C8`, alors qu'une de ses puces le vérifie — un audit de couverture mené sur les seuls en-têtes aurait conclu à un AC orphelin.

LOW appliqués : le décompte de tests de T4 était ambigu (« unitairement testés » sur 4 pièges vs « 7 tests au total ») — tranché en **un test paramétré** ; AC-C6.6 ne définissait pas « table applicative », alors que le calcul de la fenêtre doit **exclure les tables système** ; la mutation 5 (registre vidé — devenue **16-1d T-D3**) fait rougir **C1 et C1-bis**, pas C1 seul.

**Sur la règle de splitting — le second critère est coché pour la seconde fois, et cette fois par une hausse (`MEDIUM → HIGH`).** Il n'est toujours pas retenu, et voici pourquoi, à charge pour la passe 5 de le confirmer :

- **À modèle comparable, la sévérité décroît.** P1 Sonnet : 2 CRITICAL / 2 HIGH. P4 Sonnet : 0 CRITICAL / 1 HIGH. Le creux de la P3 est un artefact de modèle — Haiku n'a pas vu ce que Sonnet trouve, ce qui est le comportement attendu et documenté sur ce projet, pas un signe de convergence acquise.
- **Le HIGH ne porte pas sur la conception**, stable depuis la passe 2, mais sur le **montage d'un test**. Aucune décision D-C n'a bougé en passe 4.
- **La story est petite** : 2 entrées de registre, 1 extrait, 3 modules. Le symptôme que la règle vise — une story trop large pour tenir dans un mental-model adversarial — n'est pas celui qu'on observe.

**Prochaine** : passe 5, LLM ≠ Sonnet, contexte frais.

### Passe 5 de `bmad-create-story validate`

**2026-08-01 — Opus, 2 lentilles (BlindHunter, EdgeCaseHunter remis en jeu), contexte frais. 11 findings : 0 CRITICAL, 3 HIGH, 4 MEDIUM, 4 LOW.**

**La conception n'a pas bougé.** Les deux lentilles la déclarent saine avec vérification énumérée : fenêtre d'importabilité, deux classes, ordre croissant, périmètre 9/2/7, compteurs `57/5/52/0`, les 7 exemptions une par une, l'impossibilité de faire échouer un statement du rejeu par de la donnée forgée. **La totalité des findings porte sur la section « Notes de montage » des tests** — comme en passe 4.

**Les deux HIGH convergés annulent le patch de C4 de la passe 4, dont la prémisse était inversée.** J'avais écrit qu'une entrée fautive `#[cfg(test)]` serait vue par *tous* les appels du binaire de test ; elle ne serait vue par **aucun**. `kesh-db` est une dépendance **ordinaire** de `kesh-api` (`Cargo.toml:9`), donc `cfg(test)` y vaut faux depuis un test d'intégration — le piège exact que la spec documente elle-même pour `build_test_backup`, trois sections plus bas. Et l'alternative (entrée inconditionnelle) aurait livré en production une entrée dont le seul rôle est de faire échouer tout restore. **Résolution** : rendre le registre **injectable** (`replay_with_registry`), et faire de C4 un test d'intégration `kesh-db` — seul niveau où l'échec est observable, le chemin HTTP ne rendant qu'un `500` générique dont le détail est loggé et jamais exposé.

**Le HIGH le plus grave INVERSAIT le test C5** — trouvé par l'EdgeCaseHunter seul. Ma note de passe 4 prescrivait de « poser une ligne d'écriture créditant `2979` ». Or `HAVING COUNT(*) = 1` compte les candidats de l'écriture **entière**, et une facture canonique en produit déjà exactement un. Avec une ligne de plus : en ordre **correct**, `2979` est écarté par `a.postable = TRUE`, il reste un candidat, la ligne reçoit `3000` et l'attendu échoue ; en ordre **inversé**, deux candidats, `HAVING` échoue, la ligne reste `NULL` et le test passe. **Rouge sur l'implémentation correcte, vert sur la fautive.** Le montage juste est de **repointer** la ligne existante par `UPDATE`, ce qui préserve au passage l'équilibre débit/crédit qu'aucun `CHECK` ne protège.

**Le troisième HIGH est un défaut de propagation caractérisé.** La passe 4 a établi que `postable = TRUE` sur `2979` est inatteignable par l'API, et a corrigé C5. Elle n'a pas cherché **où ailleurs** ce fait était supposé : C3 repose dessus, et son montage littéral (« poser `role`/`postable` à la main ») aurait produit un rejeu entièrement no-op — donc un test muet et une mutation 3 qui ne rougit pas. Montage corrigé sur un **rôle délibérément effacé** (`role: null`, documenté comme l'acte de retrait), qui lui est atteignable.

MEDIUM appliqués : « no-op strict » pour les 10 `UPDATE` de rôle était faux au sens de la garde d'intention — `role: null` est un effacement délibéré, et les numéros `2850`/`2860` sont absents des chartes livrées donc `role = NULL` y est nominal ; « modifier des lignes NDJSON n'est pas sûr » était trop absolu, le fichier de test le fait déjà en recalculant le SHA ; le décompte de tests de T4 était arithmétiquement faux (1 + 6 + 1 = **8**, pas 7) — un dev s'y fiant aurait abandonné le test de parsing, rendant le garde-fou muet sur ses quatre pièges ; l'assertion « le rejeu a démarré » de C4 était inconstructible, résolue par la relocalisation du cas.

LOW appliqués : trois ancres dérivées dans ma note de passe 4 (`:913` → `:909-910`, `:921` → `:926-928`, et le seed passe par `is_postable`, jumelle d'`effective_postable`, pas par elle) ; AC-C6.4 prescrivait une fixture que le moteur courant ne sait plus produire, la matérialisation de 16-1a écrivant déjà la donnée que le backfill rattrape ; AC-C6.6 calculait la fenêtre sur du SQL non décommenté, protection accidentelle par le filtre `TABLES_TO_TRUNCATE`.

**Sur la règle de splitting — troisième déclenchement consécutif (`HIGH → HIGH`), et cette fois le signal est net mais il ne désigne pas ce que la règle vise.** Sur les passes 4 et 5, **100 % des findings sont dans la section des montages de test**, et **aucune décision `D-C` n'a bougé depuis la passe 2**. Le symptôme n'est pas une story trop large pour un mental-model — c'est une **section** qui ne converge pas pendant que le reste est stable. Le découpage qui en découlerait est donc « mécanisme + garde-fous » d'un côté, « suite de tests de bout en bout » de l'autre. **Arbitrage remonté à Guy**, conformément au précédent des deux splits de l'Epic 16.

**Arbitrage de Guy, 2026-08-01 : SPLIT.** La couverture de bout en bout — six cas E2E, leurs notes de montage et les cinq mutations — part en **16-1d**, dans son état convergé de la passe 5. 16-1c conserve la conception, les six garde-fous et le cas de transactionnalité C4, qui vit au niveau `kesh-db` et non en E2E.

**16-1c est considérée convergée** : aucune décision `D-C` n'a bougé depuis la passe 2, et les deux lentilles Opus de la passe 5 ont déclaré la conception saine avec vérification énumérée. La totalité des findings des passes 4 et 5 est partie avec la section extraite. **16-1d n'ayant jamais été revue comme un tout autonome, elle reprend la boucle à sa passe 1.**

**Les deux stories doivent partir dans la même PR et la même v0.9.0**, avec 16-1a-bis.
