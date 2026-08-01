//! Story 16-1c (#281) — rejeu des **backfills de données** après un restore
//! d'installation.
//!
//! # Le défaut que ce module ferme
//!
//! Restaurer un `.keshbackup` produit avant une migration de backfill rouvrait
//! **définitivement** le bug que cette migration fermait. Trois protections
//! auraient pu l'arrêter, aucune ne le fait :
//!
//! - `check_import_version_compat` ne refuse qu'un backup exigeant un binaire
//!   **plus récent** — restaurer une sauvegarde ancienne est le cas nominal ;
//! - `check_schema_compat` n'exige une colonne que si
//!   [`crate::backup::ColumnConstraint::is_required`] est vrai, c'est-à-dire ni
//!   nullable, **ni pourvue d'un `DEFAULT`** ;
//! - `_sqlx_migrations` n'est pas restaurée (elle n'est pas dans
//!   [`crate::backup::TABLES_TO_TRUNCATE`]), donc la migration reste marquée
//!   appliquée et ne repassera jamais.
//!
//! # La fenêtre d'importabilité, qui borne tout ce module
//!
//! Une **quatrième** protection, elle, mord : `parse_and_verify` (côté
//! `kesh-api`) exige l'égalité **exacte, dans les deux sens**, entre les tables
//! du manifeste et [`crate::backup::TABLES_TO_TRUNCATE`]. Comme aucune migration
//! du dépôt ne supprime de table, l'inventaire ne fait que croître : **un backup
//! n'est importable que s'il provient d'un binaire postérieur ou égal à la
//! dernière migration créatrice de table applicative.**
//!
//! C'est cette fenêtre qui fixe le contenu du registre — et non le nombre de
//! migrations portant un backfill. Sur les neuf migrations qui écrivent des
//! données, sept sont hors d'atteinte de ce mécanisme et sont **exemptées**
//! (cf. [`EXEMPT_MIGRATIONS`]). Toute future migration créant une table
//! applicative **referme la fenêtre** et périme les entrées antérieures : le
//! test `registry_entries_are_within_import_window` le fait échouer bruyamment
//! plutôt que de laisser du code mort qui *paraît* fonctionner.
//!
//! # Deux classes d'entrées, et le déclencheur n'est pas le même
//!
//! **Classe A — auto-gardée, rejeu inconditionnel.** Tous ses statements sont
//! gardés contre l'écrasement d'une valeur posée par l'utilisateur ; le rejeu
//! est un no-op strict sur une base à jour, il n'y a donc rien à conditionner.
//!
//! **Classe B — sentinelle, rejeu conditionné.** L'entrée contient au moins un
//! statement non gardé. Elle n'est rejouée que si l'une de ses **colonnes
//! sentinelles** manque aux `column_names` du manifeste source : le backup
//! précède alors la migration, et il n'existe aucune intention utilisateur à
//! écraser.
//!
//! ⚠️ La classe B n'est valide **que si le DDL et le backfill sont dans le même
//! fichier de migration** — sans quoi « colonne présente » n'implique pas
//! « backfill appliqué ». C'est précisément ce qui interdit de traiter
//! `20260729000001` en classe B : sa colonne est créée par `20260727000001`,
//! une migration DDL distincte. Le test
//! `class_b_sentinel_column_is_added_by_its_own_migration` verrouille l'invariant.
//!
//! ⚠️ **La classe est DÉCLARÉE, jamais devinée.** Une détection textuelle par
//! présence de `IS NULL` / `NOT EXISTS` se tromperait : le premier `UPDATE` de
//! `postable` de `20260722000001` porte un `NOT EXISTS`, qui est un prédicat
//! **structurel** de ciblage et non une garde d'idempotence.
//!
//! # L'ordre de rejeu est un invariant, pas un détail de style
//!
//! Le registre est trié par **version croissante**, et l'itération suit cet
//! ordre, afin de reproduire exactement ce qu'aurait fait une montée de version.
//! Les deux entrées ne sont pas indépendantes : `20260722000001` attribue le
//! rôle `CurrentYearResult` au compte de résultat puis le rend non imputable, et
//! la condition d'imputabilité de `20260729000001` s'appuie sur ce `postable`.
//! Rejoué à l'envers, le second verrait `postable = TRUE` partout (valeur
//! `DEFAULT` posée par le restore) et retiendrait un compte que le premier
//! s'apprête à écarter.
//!
//! # Ce que ce mécanisme n'est PAS
//!
//! Il ne rattrape qu'un cas : *le backup est dans la fenêtre d'importabilité et
//! précède une migration qui a rempli une colonne*. Il ne détecte ni ne corrige
//! une colonne remplie par du **code applicatif**, une donnée dont la
//! **sémantique** a changé sans changement de colonne, ni un backup hors fenêtre
//! (refusé en amont). Un mainteneur qui le croirait plus général y verserait des
//! rattrapages qui n'y ont pas leur place.
//!
//! # Note d'organisation
//!
//! Le sous-répertoire `src/post_restore/` ne contient que des `.sql` embarqués
//! par `include_str!`. Il ne doit **pas** être confondu avec
//! `crates/kesh-db/migrations/` : rien de ce qu'il contient n'est jamais vu par
//! `sqlx::migrate!`.

use std::collections::BTreeMap;

use sqlx::{MySql, Transaction};

use crate::backup::TableRestore;
use crate::errors::{DbError, map_db_error};

/// Ce qui déclenche le rejeu d'une entrée du registre — **propriété de la
/// déclaration**, à ne pas confondre avec [`ReplayOutcome`], qui rapporte ce qui
/// s'est effectivement passé.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackfillTrigger {
    /// **Classe A** : tous les statements sont gardés contre l'intention
    /// utilisateur, le rejeu est un no-op strict sur une base à jour.
    Unconditional,
    /// **Classe B** : rejeu si **au moins une** des colonnes `(table, colonne)`
    /// manque aux `column_names` du manifeste source.
    Sentinels(&'static [(&'static str, &'static str)]),
}

/// Une entrée du registre : le SQL de backfill d'une migration, et la règle qui
/// décide de le rejouer après un restore.
#[derive(Debug, Clone, Copy)]
pub struct PostRestoreBackfill {
    /// Version de la migration d'origine (celle du `MIGRATOR`).
    pub version: i64,
    /// Nom du fichier de migration, pour les logs et les messages d'erreur.
    pub label: &'static str,
    /// Classe A ou B — **déclarée**, jamais devinée.
    pub trigger: BackfillTrigger,
    /// Le SQL rejoué : la migration entière si elle est du backfill pur, un
    /// extrait sinon.
    pub sql: &'static str,
}

/// Ce qui s'est effectivement passé pour une entrée — **propriété de
/// l'exécution**, trois états là où [`BackfillTrigger`] n'en déclare que deux.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayOutcome {
    /// Classe A : rejoué sans condition.
    ReplayedUnconditional,
    /// Classe B : rejoué, avec la liste des sentinelles manquantes qui l'ont
    /// déclenché (`"table.colonne"`).
    ReplayedSentinelsAbsent(Vec<String>),
    /// Classe B : sauté, toutes les sentinelles étant présentes au manifeste.
    Skipped,
}

/// Rapport de rejeu d'une entrée — produit pour **toutes** les entrées, rejouées
/// comme sautées.
#[derive(Debug, Clone)]
pub struct ReplayedBackfill {
    pub version: i64,
    pub label: &'static str,
    pub outcome: ReplayOutcome,
    /// Somme des lignes touchées par les statements de l'entrée. **Informatif** :
    /// zéro est un résultat parfaitement normal et ne doit fonder aucune
    /// assertion de succès — le backfill de `20260729000001` est délibérément
    /// incomplet (cf. son AC-B2).
    pub rows_affected: u64,
}

/// Registre des backfills à rejouer après un restore, **trié par version
/// croissante** (invariant verrouillé par `registry_versions_are_strictly_increasing`).
///
/// Deux entrées seulement : les sept autres migrations du dépôt qui écrivent des
/// données sont hors de la fenêtre d'importabilité ou portent sur une table
/// système (cf. [`EXEMPT_MIGRATIONS`]).
pub const POST_RESTORE_BACKFILLS: &[PostRestoreBackfill] = &[
    PostRestoreBackfill {
        version: 20260722000001,
        label: "20260722000001_accounts_role_postable.sql",
        // CLASSE B. Sur ses 12 statements, les 10 `UPDATE` de rôle sont gardés
        // `role IS NULL`, mais les 2 `UPDATE` de `postable` ne portent AUCUNE
        // garde — rejoués sur une base à jour, ils écraseraient un `postable`
        // posé à la main (`PUT /api/v1/accounts/{id}`, sémantique full-replace).
        //
        // Et la garde `role IS NULL` n'en est pas une contre l'intention non
        // plus : `role: null` est documenté comme l'acte de RETRAIT délibéré
        // (`routes/accounts.rs`), et un compte hors plan standard (2850, 2860)
        // porte `role = NULL` nominalement.
        //
        // Sentinelle valide : `role` et `postable` sont ajoutées par le MÊME
        // `ALTER TABLE` que les 12 `UPDATE` — donc « colonne présente » implique
        // bien « backfill appliqué ».
        trigger: BackfillTrigger::Sentinels(&[("accounts", "role"), ("accounts", "postable")]),
        sql: include_str!("post_restore/20260722000001_accounts_role_postable.sql"),
    },
    PostRestoreBackfill {
        version: 20260729000001,
        label: "20260729000001_invoice_lines_revenue_account_backfill.sql",
        // CLASSE A. Les deux `UPDATE` sont gardés `revenue_account_id IS NULL`
        // ET restreints aux pièces `validated` / `issued`. C'est la CONJONCTION
        // qui porte la sûreté : une facture validée n'est plus modifiable
        // (`update` rejette tout statut != `draft`), donc un `NULL` qui y
        // subsiste ne peut PAS être un choix utilisateur — contrairement à un
        // `NULL` sur un brouillon, que le `PUT` produit dès qu'un client omet
        // `revenueAccountId` (CR #278).
        //
        // ⚠️ Ne PAS généraliser « un NULL n'est l'expression d'aucun choix » :
        // le critère est FAUX en général (cf. `default_payable_account_id`, que
        // le `PUT` des réglages efface délibérément en full-replace). Il doit
        // être vérifié route par route pour toute entrée de classe A future.
        //
        // Classe A et non B parce que sa colonne est créée par une migration
        // DISTINCTE (`20260727000001`, DDL pur) : une sentinelle mentirait sur
        // un backup pris entre les deux, qui porte la colonne entièrement `NULL`.
        //
        // Backfill pur (aucun DDL) : la migration est rejouable EN ENTIER, d'où
        // `include_str!` du fichier lui-même — zéro duplication, et un renommage
        // casse la compilation plutôt que de dégrader en échec runtime.
        trigger: BackfillTrigger::Unconditional,
        sql: include_str!(
            "../migrations/20260729000001_invoice_lines_revenue_account_backfill.sql"
        ),
    },
];

/// Migrations qui écrivent des données mais que ce mécanisme n'a **pas** à
/// rattraper, chacune avec sa justification. Le test
/// `every_data_backfill_migration_is_triaged` exige que toute migration porteuse
/// d'un statement d'écriture figure ici ou au registre.
pub const EXEMPT_MIGRATIONS: &[(i64, &str)] = &[
    (
        20260419000002,
        "users.company_id finit NOT NULL sans défaut => is_required() vrai => un backup qui ne la \
         porte pas est refusé par check_schema_compat (400). Le cas ne peut pas se produire.",
    ),
    (
        20260428000001,
        "Crée la table vat_rates. Un backup dépourvu de ses 4 lignes de taux est un backup \
         dépourvu de la table => refusé au contrôle de couverture. Hors fenêtre.",
    ),
    (
        20260522000001,
        "INSERT sur _kesh_version, table système hors TABLES_TO_TRUNCATE : jamais exportée ni \
         restaurée. Ce n'est pas un backfill de données applicatives.",
    ),
    (
        20260613000001,
        "Hors fenêtre : un backup dépourvu de vat_rates.category précède 20260628000001, donc \
         n'a pas les tables supplier_invoices => refusé au contrôle de couverture.",
    ),
    (
        20260614000001,
        "Hors fenêtre, même raisonnement : un backup dépourvu des comptes 1171/2206 est \
         antérieur aux tables créées depuis.",
    ),
    (
        20260628000001,
        "Autoréfutante : la migration crée elle-même supplier_invoices et supplier_invoice_lines, \
         donc un backup dépourvu de default_payable_account_id est dépourvu de ces tables.",
    ),
    (
        20260714000002,
        "UPDATE sur _kesh_version (bump kesh_version_min_required), table système jamais \
         restaurée. Ce n'est pas un backfill applicatif.",
    ),
];

/// Rejoue les backfills du registre canonique dans la transaction de restore.
///
/// Appelée par le handler d'import **après** la garde de cohérence de comptage et
/// **avant** l'insertion de l'audit, donc avec `FOREIGN_KEY_CHECKS = 1` :
/// `restore_tables_in_tx` rétablit systématiquement le flag avant de rendre la
/// main. Toute erreur remonte et annule le restore entier.
pub async fn replay_post_restore_backfills(
    tx: &mut Transaction<'_, MySql>,
    tables: &BTreeMap<String, TableRestore>,
) -> Result<Vec<ReplayedBackfill>, DbError> {
    replay_with_registry(tx, tables, POST_RESTORE_BACKFILLS).await
}

/// Cœur du rejeu, **paramétré par le registre**.
///
/// Cette porte existe pour une raison précise : sans elle, l'échec du rejeu
/// serait **intestable**. `#[cfg(test)]` ne traverse pas la frontière de crate,
/// donc une entrée fautive déclarée ainsi dans `kesh-db` serait invisible depuis
/// un test d'intégration de `kesh-api` ; et par le chemin HTTP l'erreur se
/// réduit à un `500` générique dont le détail est loggé et jamais exposé.
/// Injecter un registre fautif ici est le seul niveau où le rollback est
/// observable.
pub async fn replay_with_registry(
    tx: &mut Transaction<'_, MySql>,
    tables: &BTreeMap<String, TableRestore>,
    registry: &[PostRestoreBackfill],
) -> Result<Vec<ReplayedBackfill>, DbError> {
    let mut report = Vec::with_capacity(registry.len());

    for entry in registry {
        let outcome = match entry.trigger {
            BackfillTrigger::Unconditional => ReplayOutcome::ReplayedUnconditional,
            BackfillTrigger::Sentinels(sentinels) => {
                let missing = missing_sentinels(tables, sentinels);
                if missing.is_empty() {
                    ReplayOutcome::Skipped
                } else {
                    ReplayOutcome::ReplayedSentinelsAbsent(missing)
                }
            }
        };

        let rows_affected = if matches!(outcome, ReplayOutcome::Skipped) {
            tracing::debug!(
                version = entry.version,
                label = entry.label,
                "rejeu post-restore sauté : toutes les sentinelles sont présentes au manifeste"
            );
            0
        } else {
            // Exécution **statement par statement**, et non par `sqlx::raw_sql` :
            // le futur de ce dernier n'est pas `Send`, ce qui casse la contrainte
            // `Handler` d'Axum sur le handler d'import. Le découpage a en outre
            // l'avantage de faire remonter l'erreur avec le statement fautif.
            let mut rows = 0u64;
            for statement in split_statements(entry.sql) {
                rows += sqlx::query(&statement)
                    .execute(&mut **tx)
                    .await
                    .map_err(map_db_error)?
                    .rows_affected();
            }
            tracing::info!(
                version = entry.version,
                label = entry.label,
                outcome = ?outcome,
                rows_affected = rows,
                "backfill post-restore rejoué"
            );
            rows
        };

        report.push(ReplayedBackfill {
            version: entry.version,
            label: entry.label,
            outcome,
            rows_affected,
        });
    }

    Ok(report)
}

/// Sentinelles absentes du manifeste, au format `"table.colonne"`.
///
/// Une table entièrement absente compte comme sentinelle absente. C'est
/// **défensif seulement** : `parse_and_verify` refuse en amont tout manifeste
/// incomplet en tables, donc l'état est inatteignable par le flux réel.
fn missing_sentinels(
    tables: &BTreeMap<String, TableRestore>,
    sentinels: &[(&'static str, &'static str)],
) -> Vec<String> {
    sentinels
        .iter()
        .filter(|(table, column)| match tables.get(*table) {
            Some(data) => !data.column_names.iter().any(|c| c == column),
            None => true,
        })
        .map(|(table, column)| format!("{table}.{column}"))
        .collect()
}

// ===========================================================================
// Analyse du SQL des migrations — support des garde-fous fail-loud.
// ===========================================================================

/// Retire les commentaires `--` (en ligne entière comme en fin de ligne) et
/// découpe le SQL en statements sur les `;`.
///
/// **Le découpage doit être multi-ligne** : plusieurs backfills du dépôt portent
/// leur `SELECT` sur la ligne suivant l'`INSERT`, et un détecteur mono-ligne les
/// rate — c'est ainsi qu'une migration exposée a échappé au triage manuel.
///
/// **Le retrait des commentaires n'est pas cosmétique** : les migrations sont
/// très commentées et plusieurs commentaires contiennent le mot `UPDATE` en
/// prose. Attention en particulier à `ON UPDATE CURRENT_TIMESTAMP`, présent dans
/// une vingtaine de migrations — c'est du DDL, pas un statement `UPDATE`.
///
/// Aucune migration du dépôt n'utilise `/* */` ; ce cas n'est donc pas traité.
fn split_statements(sql: &str) -> Vec<String> {
    let stripped: String = sql
        .lines()
        .map(|line| match line.find("--") {
            Some(idx) => &line[..idx],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n");

    stripped
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Vrai si le statement **écrit des données** : premier mot-clé `UPDATE`,
/// `INSERT` (toutes formes), `REPLACE` ou `DELETE`.
///
/// Volontairement **large**. Le coût d'une forme en trop est une exemption d'une
/// ligne ; le coût d'une forme manquante est un garde-fou **muet** — ce qui est
/// arrivé deux fois pendant la spécification de cette story, d'abord sur le
/// multi-ligne puis sur `INSERT IGNORE`.
#[cfg(test)]
fn writes_data(statement: &str) -> bool {
    let first = statement
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    matches!(first.as_str(), "UPDATE" | "INSERT" | "REPLACE" | "DELETE")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le registre est parcouru dans l'ordre de sa déclaration : il doit être
    /// strictement croissant en version, sans quoi l'invariant d'ordre de D-C2
    /// est rompu en silence.
    #[test]
    fn registry_versions_are_strictly_increasing() {
        for pair in POST_RESTORE_BACKFILLS.windows(2) {
            assert!(
                pair[0].version < pair[1].version,
                "registre non trié : {} précède {} — l'ordre de rejeu doit reproduire une montée \
                 de version (D-C2)",
                pair[0].version,
                pair[1].version
            );
        }
    }

    /// Chaque entrée du registre correspond à une migration réelle du `MIGRATOR`,
    /// et son `label` est bien le nom de fichier de cette migration.
    #[test]
    fn registry_entries_match_a_real_migration() {
        for entry in POST_RESTORE_BACKFILLS {
            let m = crate::MIGRATOR
                .migrations
                .iter()
                .find(|m| m.version == entry.version)
                .unwrap_or_else(|| {
                    panic!(
                        "entrée de registre {} introuvable dans le MIGRATOR — migration renommée \
                         ou supprimée ?",
                        entry.version
                    )
                });
            let expected = format!("{}_{}.sql", m.version, m.description.replace(' ', "_"));
            assert_eq!(
                entry.label, expected,
                "label de l'entrée {} désynchronisé du nom de fichier réel",
                entry.version
            );
        }
    }

    /// **Garde-fou de la fenêtre d'importabilité.**
    ///
    /// Un backup n'est importable que si son inventaire de tables est identique
    /// au nôtre ; les tables ne faisant que s'ajouter, cela borne les backups
    /// acceptés aux binaires postérieurs à la dernière migration créatrice de
    /// table **applicative**. Une entrée de registre antérieure à cette borne est
    /// du **code mort qui paraît fonctionner** : son cas de déclenchement est
    /// refusé en 400 bien avant le rejeu.
    ///
    /// La fenêtre est **recalculée depuis le `MIGRATOR`**, jamais codée en dur :
    /// toute future migration créant une table applicative la referme et fait
    /// échouer ce test, forçant le triage.
    #[test]
    fn registry_entries_are_within_import_window() {
        let window = last_table_creating_migration();
        for entry in POST_RESTORE_BACKFILLS {
            assert!(
                entry.version > window,
                "l'entrée {} est ANTÉRIEURE à la dernière migration créatrice de table applicative \
                 ({window}) : un backup assez ancien pour la déclencher est refusé au contrôle de \
                 couverture de `parse_and_verify`. Elle doit passer en exemption.",
                entry.version
            );
        }
    }

    /// Version de la dernière migration créant une table **applicative**,
    /// c'est-à-dire figurant dans [`crate::backup::TABLES_TO_TRUNCATE`].
    ///
    /// Le filtre sur l'inventaire applicatif est indispensable : `parse_and_verify`
    /// ne compare que ces tables, donc une migration créant une table **système**
    /// (comme `_kesh_version`) ne déplace pas la fenêtre. Le SQL est décommenté
    /// avant analyse — plusieurs migrations contiennent `CREATE TABLE` en prose.
    fn last_table_creating_migration() -> i64 {
        crate::MIGRATOR
            .migrations
            .iter()
            .filter(|m| {
                split_statements(&m.sql).iter().any(|st| {
                    let up = st.to_ascii_uppercase();
                    up.starts_with("CREATE TABLE")
                        && crate::backup::TABLES_TO_TRUNCATE.iter().any(|t| {
                            up.contains(&format!("CREATE TABLE {}", t.to_ascii_uppercase()))
                                || up.contains(&format!(
                                    "CREATE TABLE IF NOT EXISTS {}",
                                    t.to_ascii_uppercase()
                                ))
                        })
                })
            })
            .map(|m| m.version)
            .max()
            .expect("aucune migration créatrice de table applicative — schéma vide ?")
    }

    /// **Garde-fou fail-loud du triage.**
    ///
    /// Toute migration portant un statement d'écriture de données doit figurer au
    /// registre ou à la liste d'exemption. Sans ce test, le registre redérive au
    /// fil des Epics — et deux migrations exposées ont déjà échappé au triage
    /// manuel pendant la seule spécification de cette story.
    #[test]
    fn every_data_backfill_migration_is_triaged() {
        let mut untriaged = Vec::new();
        for m in crate::MIGRATOR.migrations.iter() {
            if !split_statements(&m.sql).iter().any(|s| writes_data(s)) {
                continue;
            }
            let known = POST_RESTORE_BACKFILLS
                .iter()
                .any(|e| e.version == m.version)
                || EXEMPT_MIGRATIONS.iter().any(|(v, _)| *v == m.version);
            if !known {
                untriaged.push(format!("{}_{}", m.version, m.description));
            }
        }
        assert!(
            untriaged.is_empty(),
            "migration(s) écrivant des données et NON TRIÉE(S) : {untriaged:?}\n\
             → soit l'ajouter à POST_RESTORE_BACKFILLS (son backfill doit être rejoué après un \
             restore), soit à EXEMPT_MIGRATIONS avec une justification écrite (elle est hors de la \
             fenêtre d'importabilité, ou porte sur une table système).\n\
             Cf. `crates/kesh-db/src/post_restore.rs` et le garde-fou P7 de CLAUDE.md."
        );
    }

    /// Les exemptions désignent des migrations réelles, et ne font pas double
    /// emploi avec le registre.
    #[test]
    fn exemptions_are_real_and_disjoint_from_registry() {
        for (version, justification) in EXEMPT_MIGRATIONS {
            assert!(
                crate::MIGRATOR
                    .migrations
                    .iter()
                    .any(|m| m.version == *version),
                "exemption {version} : migration introuvable dans le MIGRATOR"
            );
            assert!(
                !justification.trim().is_empty(),
                "exemption {version} : justification vide"
            );
            assert!(
                !POST_RESTORE_BACKFILLS.iter().any(|e| e.version == *version),
                "migration {version} présente À LA FOIS au registre et aux exemptions"
            );
        }
    }

    /// **Fidélité de l'extrait.** Chaque statement de l'extrait de classe B doit
    /// être un sous-texte du SQL de la migration source tel qu'embarqué dans le
    /// `MIGRATOR`.
    ///
    /// Les migrations étant immuables (checksums sqlx), ce test ne protège pas
    /// d'une dérive future mais d'une **erreur de copie à l'écriture** — c'est là
    /// qu'est le risque réel. Un `<>` glissé à la place d'un `<=>` en recopiant
    /// serait indiscernable du succès.
    #[test]
    fn extract_statements_are_verbatim_substrings_of_source_migration() {
        for entry in POST_RESTORE_BACKFILLS {
            let source = &crate::MIGRATOR
                .migrations
                .iter()
                .find(|m| m.version == entry.version)
                .expect("migration du registre présente dans le MIGRATOR")
                .sql;
            for statement in split_statements(entry.sql) {
                assert!(
                    normalize(source).contains(&normalize(&statement)),
                    "statement de l'extrait {} absent VERBATIM de sa migration source :\n{}",
                    entry.label,
                    statement
                );
            }
        }
    }

    /// Normalise les blancs pour comparer un statement à son fichier source sans
    /// dépendre de l'indentation résiduelle du découpage.
    fn normalize(s: &str) -> String {
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// **Condition de validité de la classe B** : la colonne sentinelle doit être
    /// créée par la migration elle-même, sans quoi « colonne présente » n'implique
    /// pas « backfill appliqué » et la sentinelle ment.
    #[test]
    fn class_b_sentinel_column_is_added_by_its_own_migration() {
        for entry in POST_RESTORE_BACKFILLS {
            let BackfillTrigger::Sentinels(sentinels) = entry.trigger else {
                continue;
            };
            let source = crate::MIGRATOR
                .migrations
                .iter()
                .find(|m| m.version == entry.version)
                .expect("migration du registre présente dans le MIGRATOR")
                .sql
                .to_ascii_uppercase();
            for (table, column) in sentinels {
                assert!(
                    source.contains(&format!("ADD COLUMN {}", column.to_ascii_uppercase())),
                    "sentinelle {table}.{column} de l'entrée {} : la colonne n'est PAS créée par \
                     cette migration. « Colonne présente » n'implique alors pas « backfill \
                     appliqué » — l'entrée doit passer en classe A.",
                    entry.version
                );
            }
        }
    }

    /// Le découpage doit survivre aux pièges réels du dépôt : `UPDATE` en prose de
    /// commentaire, `ON UPDATE CURRENT_TIMESTAMP` (du DDL), `INSERT … SELECT` à
    /// cheval sur deux lignes, et `INSERT IGNORE INTO`.
    #[test]
    fn statement_splitting_survives_the_real_traps() {
        let sql = "\
-- Les douze UPDATE de backfill sont idempotents.
CREATE TABLE t (
  id BIGINT,
  updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3)
);
INSERT IGNORE INTO vat_rates (company_id, label)
SELECT id, 'x' FROM companies;
UPDATE accounts SET role = 'X' WHERE number = '1'; -- commentaire de fin de ligne
";
        let statements = split_statements(sql);
        let writing: Vec<&String> = statements.iter().filter(|s| writes_data(s)).collect();

        assert_eq!(
            writing.len(),
            2,
            "attendu 2 statements d'écriture (l'INSERT IGNORE multi-ligne et l'UPDATE), obtenu \
             {writing:?}"
        );
        assert!(
            writing[0].to_ascii_uppercase().starts_with("INSERT IGNORE"),
            "l'INSERT IGNORE à cheval sur deux lignes doit être détecté : {:?}",
            writing[0]
        );
        assert!(
            !statements.iter().any(|s| s
                .to_ascii_uppercase()
                .starts_with("UPDATE ACCOUNTS SET ROLE = 'X' WHERE NUMBER = '1' --")),
            "le commentaire de fin de ligne doit être retiré"
        );
        assert!(
            statements
                .iter()
                .filter(|s| writes_data(s))
                .all(|s| !s.to_ascii_uppercase().contains("CREATE TABLE")),
            "ON UPDATE CURRENT_TIMESTAMP est du DDL et ne doit pas compter comme écriture"
        );
    }

    /// Une sentinelle absente déclenche, une sentinelle présente non, et la
    /// sémantique est bien un **OU** : une seule manquante suffit.
    #[test]
    fn missing_sentinels_uses_or_semantics() {
        let mut tables = BTreeMap::new();
        tables.insert(
            "accounts".to_string(),
            TableRestore {
                column_names: vec!["id".into(), "role".into()],
                rows: Vec::new(),
            },
        );
        let sentinels: &[(&str, &str)] = &[("accounts", "role"), ("accounts", "postable")];

        let missing = missing_sentinels(&tables, sentinels);
        assert_eq!(
            missing,
            vec!["accounts.postable".to_string()],
            "une seule sentinelle manquante doit suffire à déclencher (OU, pas ET)"
        );

        // Table entièrement absente : défensif, compte comme sentinelle absente.
        let empty = BTreeMap::new();
        assert_eq!(missing_sentinels(&empty, sentinels).len(), 2);
    }
}
