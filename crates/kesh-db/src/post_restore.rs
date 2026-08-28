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
//! ⚠️ **Il ne rattrape pas non plus un backup pris DEPUIS un parc déjà cassé par
//! un restore antérieur** — la limite structurelle de la classe B. Un exploitant
//! passé en v0.8.0 puis ayant importé un backup v0.7.0 (avant l'existence de ce
//! mécanisme) a un parc sans rôles. Un backup pris *depuis cet état* **porte** les
//! colonnes `role` / `postable` — ce sont des colonnes du schéma, pas des
//! données — mais pas leur contenu : la sentinelle le déclare à jour et l'entrée
//! est `Skipped`. Le mécanisme ne **cause** pas ce dommage, il ne le **répare**
//! pas ; le remède reste de réimporter le backup **ancien**, qui déclenche bien
//! le rejeu — **au prix d'un import complet**, donc de la perte de tout ce qui a
//! été saisi depuis le restore fautif. Ce coût doit être énoncé partout où le
//! remède l'est (CHANGELOG, manuel d'administration) : le rejeu n'existe qu'en
//! tant qu'étape de l'import, il n'y a aucune voie qui rejouerait sans
//! restaurer. Fermer ce cas supposerait de décider sur la **donnée** plutôt que sur
//! le manifeste, ce que la classe B ne fait délibérément pas — un `role IS NULL`
//! n'est pas une garde d'intention (cf. D-C1 : `role: null` est un acte de retrait
//! délibéré, et les comptes hors plan standard portent `NULL` nominalement).
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

impl ReplayOutcome {
    /// Code d'archive **stable**, destiné au détail JSON de l'audit.
    ///
    /// Il existe parce que l'audit est une **archive**, pas un log : ses
    /// enregistrements sont relus des mois plus tard, et un `format!("{self:?}")`
    /// y aurait écrit une chaîne de debug Rust — donc un format non
    /// contractualisé, que le simple renommage d'un variant ferait dériver, avec
    /// des enregistrements anciens devenus illisibles par toute exploitation
    /// automatique. Ces codes-ci ne changent pas ; les noms de variants, si.
    pub fn code(&self) -> &'static str {
        match self {
            Self::ReplayedUnconditional => "REPLAYED_UNCONDITIONAL",
            Self::ReplayedSentinelsAbsent(_) => "REPLAYED_SENTINELS_ABSENT",
            Self::Skipped => "SKIPPED",
        }
    }

    /// Sentinelles manquantes ayant déclenché le rejeu (`"table.colonne"`).
    /// Vide pour les deux autres issues.
    pub fn missing_sentinels(&self) -> &[String] {
        match self {
            Self::ReplayedSentinelsAbsent(missing) => missing,
            Self::ReplayedUnconditional | Self::Skipped => &[],
        }
    }
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
pub const POST_RESTORE_BACKFILLS: &[PostRestoreBackfill] = &[];

// ⚠️ **Le registre est VIDE depuis la Story 24-2 (#371), et ce n'est pas un
// oubli — c'est le mécanisme qui fonctionne.**
//
// La création de `invoice_settlements` (`20260827000001`) a **refermé la fenêtre
// d'importabilité** au-delà des deux entrées qu'il portait (`20260722000001` et
// `20260729000001`) : un backup assez ancien pour les déclencher est désormais
// dépourvu de cette table, donc refusé en 400 au contrôle de couverture de
// `parse_and_verify`, bien avant tout rejeu. Les y laisser aurait produit du
// **code mort qui paraît fonctionner** — et, pour celle de classe A, exécuté à
// chaque import.
//
// Les deux sont passées en [`EXEMPT_MIGRATIONS`] avec le marqueur `Hors fenêtre`,
// que `exemptions_claiming_out_of_window_really_are_out_of_window` recalcule
// depuis le `MIGRATOR` — l'affirmation est donc vérifiée, pas crue sur parole.
//
// ⚠️ **Toute nouvelle table applicative aura le même effet** sur les entrées
// futures. Ce n'est pas une raison de renoncer au mécanisme : il reste la seule
// réponse au cas « le backup est dans la fenêtre et précède un backfill », et ce
// cas se représentera dès la prochaine migration qui remplit une colonne.

/// Les entrées **retirées** du registre parce que la fenêtre d'importabilité
/// s'est refermée sur elles (Story 24-2, #371).
///
/// ⚠️ **Ce n'est PAS un registre de production** — `replay_post_restore_backfills`
/// ne le consulte jamais. Il existe pour que les tests de la **machinerie**
/// (ordre de rejeu, sémantique OU des sentinelles, classes A/B, codes de sortie,
/// transactionnalité) gardent un registre réaliste à exercer. Sans lui, ces
/// tests tourneraient à vide — ou disparaîtraient, ce qui reviendrait à perdre
/// la seule couverture d'un mécanisme qui redeviendra actif à la prochaine
/// migration de backfill.
///
/// Les justifications de leur exemption sont dans [`EXEMPT_MIGRATIONS`].
#[doc(hidden)]
pub const RETIRED_BACKFILLS: &[PostRestoreBackfill] = &[
    PostRestoreBackfill {
        version: 20260722000001,
        label: "20260722000001_accounts_role_postable.sql",
        // CLASSE B. Sur ses 12 statements, les 10 `UPDATE` de rôle sont gardés
        // `role IS NULL`, mais les 2 `UPDATE` de `postable` ne portent AUCUNE
        // garde — rejoués sur une base à jour, ils écraseraient un `postable`
        // posé à la main (`PUT /api/v1/accounts/{id}`, sémantique full-replace).
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
        // qui porte la sûreté : une facture validée n'est plus modifiable, donc
        // un `NULL` qui y subsiste ne peut PAS être un choix utilisateur.
        //
        // ⚠️ Ne PAS généraliser « un NULL n'est l'expression d'aucun choix » :
        // le critère est FAUX en général (cf. `default_payable_account_id`, que
        // le `PUT` des réglages efface délibérément en full-replace).
        //
        // Classe A et non B parce que sa colonne est créée par une migration
        // DISTINCTE (`20260727000001`, DDL pur) : une sentinelle mentirait sur
        // un backup pris entre les deux.
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
///
/// ⚠️ **Une justification qui invoque la fenêtre d'importabilité DOIT commencer
/// par la chaîne `Hors fenêtre`.** Ce n'est pas une convention de style : c'est
/// le marqueur sur lequel `exemptions_claiming_out_of_window_really_are_out_of_window`
/// recalcule la fenêtre depuis le `MIGRATOR` et vérifie l'affirmation. Une
/// justification « hors fenêtre » rédigée autrement échappe au contrôle et
/// redevient une affirmation crue sur parole — laquelle, si elle est fausse,
/// désactive **définitivement et en silence** le rejeu du backfill concerné.
pub const EXEMPT_MIGRATIONS: &[(i64, &str)] = &[
    (
        20260722000001,
        "Hors fenêtre depuis 20260827000001 (invoice_settlements, Story 24-2) : un backup assez \
         ancien pour porter role/postable vides est dépourvu de cette table, donc refusé au \
         contrôle de couverture. Figurait au registre de rejeu jusque-là ; l'y laisser aurait \
         produit du code mort qui paraît fonctionner.",
    ),
    (
        20260729000001,
        "Hors fenêtre depuis 20260827000001 (invoice_settlements, Story 24-2) : même raison que \
         20260722000001. Elle était de CLASSE A, donc rejouée inconditionnellement à chaque \
         import — la laisser au registre aurait fait exécuter à chaque restore un backfill dont \
         le cas de déclenchement ne peut plus se présenter.",
    ),
    (
        20260419000002,
        "users.company_id finit NOT NULL sans défaut => is_required() vrai => un backup qui ne la \
         porte pas est refusé par check_schema_compat (400). Le cas ne peut pas se produire.",
    ),
    (
        20260428000001,
        "Hors fenêtre : la migration crée elle-même vat_rates, donc un backup dépourvu de ses 4 \
         lignes de taux est un backup dépourvu de la table => refusé au contrôle de couverture.",
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
        "Hors fenêtre : un backup dépourvu des comptes 1171/2206 précède 20260628000001, donc \
         n'a pas les tables supplier_invoices => refusé au contrôle de couverture. (Version de \
         référence citée EXPLICITEMENT — un « antérieur aux tables créées depuis » ne se vérifie \
         pas.)",
    ),
    (
        20260628000001,
        "Hors fenêtre, et autoréfutante : la migration crée elle-même supplier_invoices et \
         supplier_invoice_lines, donc un backup dépourvu de default_payable_account_id est \
         dépourvu de ces tables.",
    ),
    (
        20260714000002,
        "UPDATE sur _kesh_version (bump kesh_version_min_required), table système jamais \
         restaurée. Ce n'est pas un backfill applicatif.",
    ),
    (
        20260814000001,
        "UPDATE sur _kesh_version (bump kesh_version_min_required), table système jamais \
         restaurée. Le reste de la migration est du DDL pur ; le remplissage du parc vit HORS \
         migration (backfill_client_number_canonical, appelée au boot et en fin d'import — D6 \
         Story 22-1), ce qui tient l'esprit de P7 sans entrée au registre.",
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
///
/// ⚠️ **Porte de test, pas d'API de production.** `#[doc(hidden)]` la retire de la
/// documentation publique : le code applicatif doit passer par
/// [`replay_post_restore_backfills`], seule entrée qui garantit le registre
/// canonique. L'appeler directement depuis un handler contournerait tout le
/// dispositif de triage inscrit au garde-fou P7 de `CLAUDE.md`.
#[doc(hidden)]
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
            // `Handler` d'Axum sur le handler d'import.
            //
            // Le découpage n'a d'intérêt diagnostique que si l'erreur PORTE le
            // statement fautif. Remontée nue, elle arrive à l'exploitant sous la
            // forme d'un `AppError::AdminFullImportFailed("rejeu des backfills :
            // <erreur DB>")` qui ne dit ni quelle reprise ni quel statement a
            // échoué — un code MariaDB nu ne nomme rien.
            let mut rows = 0u64;
            for (index, statement) in split_statements(entry.sql).into_iter().enumerate() {
                let position = index + 1;
                rows += sqlx::query(&statement)
                    .execute(&mut **tx)
                    .await
                    .map_err(|e| {
                        let cause = map_db_error(e);
                        tracing::error!(
                            version = entry.version,
                            label = entry.label,
                            statement_position = position,
                            error = %cause,
                            "échec d'un statement de backfill post-restore — le restore entier \
                             est annulé"
                        );
                        DbError::Invariant(format!(
                            "backfill post-restore {} (version {}), statement #{position} : \
                             {cause} — statement : {}",
                            entry.label,
                            entry.version,
                            elide(&statement),
                        ))
                    })?
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
/// # Deux limites, et elles sont OUTILLÉES et non simplement déclarées
///
/// Cet analyseur est **volontairement naïf** : il ne connaît ni les commentaires
/// de bloc `/* */`, ni les littéraux entre apostrophes. Les deux limites étaient
/// jusqu'ici de la prose ; elles sont désormais tenues par un test chacune,
/// parce qu'une hypothèse sur le contenu du dépôt qui n'échoue pas bruyamment
/// quand elle cesse d'être vraie est exactement le « garde-fou muet » que ce
/// module existe pour empêcher.
///
/// 1. **`/* */` — risque de FAUX NÉGATIF au triage.** Un `UPDATE` enfermé dans un
///    commentaire de bloc, ou un `;` à l'intérieur d'un tel bloc, décale le
///    découpage et peut faire rendre `false` à [`writes_data`] : la migration
///    échapperait alors au triage **en silence**. Tenu par
///    `migrations_contain_no_block_comment`.
/// 2. **`--` ou `;` dans un littéral — risque de SQL CORROMPU à l'exécution.** La
///    troncature au premier `--` et la découpe au premier `;` ignorent les
///    apostrophes ; un `SET label = 'poste -- 4000'` partirait tronqué à MariaDB.
///    Le dépôt contient **déjà** un `;` littéral (`20260505000001_bank_profiles`,
///    `IN (',', ';', '\t')`), inoffensif parce que ce fichier n'est pas rejoué —
///    ce qui montre que le risque n'est pas théorique. Tenu, **sur le SQL
///    réellement exécuté** et lui seul, par `registry_sql_has_no_literal_hazard`.
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

/// Abrège un statement pour un message d'erreur : blancs normalisés, tronqué à
/// 200 caractères sur une **frontière de caractère** (`char_indices`, jamais un
/// slice d'octets — les migrations portent des accents).
fn elide(statement: &str) -> String {
    const MAX: usize = 200;
    let flat = statement.split_whitespace().collect::<Vec<_>>().join(" ");
    match flat.char_indices().nth(MAX) {
        Some((byte_idx, _)) => format!("{}…", &flat[..byte_idx]),
        None => flat,
    }
}

/// Positions (1-based) des `--` et `;` situés **à l'intérieur d'un littéral**
/// entre apostrophes, que [`split_statements`] traiterait à tort comme un
/// commentaire ou une fin de statement.
///
/// Scanner minimal : bascule sur `'`, gère l'échappement doublé `''` et
/// l'échappement antislash `\'` de MariaDB. Il n'a pas à couvrir les chaînes
/// entre guillemets doubles ni les identifiants entre accents graves — aucun
/// n'apparaît dans le SQL rejoué, et le test qui l'emploie ne porte que sur
/// celui-là.
#[cfg(test)]
fn literal_hazards(sql: &str) -> Vec<String> {
    let chars: Vec<char> = sql.chars().collect();
    let mut hazards = Vec::new();
    let mut in_literal = false;
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];
        if in_literal {
            if c == '\\' {
                i += 2; // séquence échappée : `\'` ne ferme pas le littéral
                continue;
            }
            if c == '\'' {
                // `''` est une apostrophe littérale, pas une fermeture.
                if chars.get(i + 1) == Some(&'\'') {
                    i += 2;
                    continue;
                }
                in_literal = false;
            } else if c == ';' {
                hazards.push(format!("`;` littéral au caractère {}", i + 1));
            } else if c == '-' && chars.get(i + 1) == Some(&'-') {
                hazards.push(format!("`--` littéral au caractère {}", i + 1));
                i += 1;
            }
        } else if c == '\'' {
            in_literal = true;
        } else if c == '-' && chars.get(i + 1) == Some(&'-') {
            // Commentaire de fin de ligne : sauter jusqu'au saut de ligne, sinon
            // une apostrophe en prose de commentaire ouvrirait un faux littéral
            // (« l'exercice », « n'est ») et décalerait tout le scan.
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        i += 1;
    }

    hazards
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
            let expected = migration_file_name(m);
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
    ///
    /// ⚠️ **La reconnaissance doit être LARGE, pour la raison inverse de
    /// [`writes_data`].** Ici, une graphie ratée ne produit pas un faux positif
    /// mais une fenêtre qui **recule** : `registry_entries_are_within_import_window`
    /// resterait vert sur des entrées devenues injoignables, c'est-à-dire du code
    /// mort qui *paraît* fonctionner — le mode d'échec que l'AC-C6.6 existe pour
    /// interdire. D'où la normalisation des blancs (`CREATE  TABLE`, nom rejeté à
    /// la ligne suivante) et le retrait des accents graves (`` CREATE TABLE `x` ``,
    /// graphie absente du dépôt aujourd'hui mais que rien n'interdit).
    fn last_table_creating_migration() -> i64 {
        crate::MIGRATOR
            .migrations
            .iter()
            .filter(|m| {
                split_statements(&m.sql)
                    .iter()
                    .any(|st| creates_application_table(st))
            })
            .map(|m| m.version)
            .max()
            .expect("aucune migration créatrice de table applicative — schéma vide ?")
    }

    /// Vrai si le statement crée une table figurant dans
    /// [`crate::backup::TABLES_TO_TRUNCATE`].
    ///
    /// La comparaison porte sur le nom **isolé** (et non sur un `contains`) : un
    /// `CREATE TABLE supplier_invoice_lines` ne doit pas être compté comme créant
    /// `supplier_invoices`, ni l'inverse.
    fn creates_application_table(statement: &str) -> bool {
        let up = normalize(statement).to_ascii_uppercase().replace('`', "");
        let Some(rest) = up.strip_prefix("CREATE TABLE ") else {
            return false;
        };
        let rest = rest.strip_prefix("IF NOT EXISTS ").unwrap_or(rest);
        // Le nom s'arrête au premier blanc ou à la parenthèse ouvrante, qui peut
        // être collée au nom (`CREATE TABLE accounts(`).
        let name = rest
            .split([' ', '('])
            .next()
            .unwrap_or_default()
            .trim_end_matches(';');
        crate::backup::TABLES_TO_TRUNCATE
            .iter()
            .any(|t| t.to_ascii_uppercase() == name)
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
                // `m.description` porte des ESPACES là où le fichier porte des
                // `_` (sqlx dérive la description du nom de fichier). Rendre la
                // description telle quelle donnerait « 20260722000001_accounts
                // role postable », qui n'est le nom d'aucun fichier et ne se
                // grep pas. AC-C6.1 exige de nommer LE FICHIER.
                untriaged.push(migration_file_name(m));
            }
        }
        assert!(
            untriaged.is_empty(),
            "migration(s) écrivant des données et NON TRIÉE(S) : {untriaged:?}\n\
             → soit l'ajouter à POST_RESTORE_BACKFILLS (son backfill doit être rejoué après un \
             restore), soit à EXEMPT_MIGRATIONS avec une justification écrite (elle est hors de la \
             fenêtre d'importabilité, ou porte sur une table système).\n\
             Contexte : issue #281 (pourquoi ce rejeu existe) et issue #152 (Epic 16, qui l'a \
             introduit).\n\
             Cf. `crates/kesh-db/src/post_restore.rs` et le garde-fou P7 de CLAUDE.md."
        );
    }

    /// Nom de fichier réel d'une migration, reconstruit depuis le `MIGRATOR`.
    ///
    /// Point unique de vérité de cette reconstruction : elle était auparavant
    /// écrite deux fois, et l'un des deux sites l'avait oubliée.
    fn migration_file_name(m: &sqlx::migrate::Migration) -> String {
        format!("{}_{}.sql", m.version, m.description.replace(' ', "_"))
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

    /// **Le symétrique de `registry_entries_are_within_import_window`, et il
    /// manquait.**
    ///
    /// Le garde-fou de triage n'offre que deux issues, et la moins coûteuse est
    /// l'exemption : `every_data_backfill_migration_is_triaged` échoue en
    /// proposant lui-même « soit l'ajouter à EXEMPT_MIGRATIONS avec une
    /// justification écrite ». Rien ne vérifiait cette justification. Un
    /// développeur pressé par un test rouge y recopie la voisine — « hors
    /// fenêtre » — et le backfill n'est **jamais** rejoué après un restore, sans
    /// qu'aucun test ne le dise. C'est la même classe de défaut que l'entrée de
    /// registre hors fenêtre, prise par l'autre bout.
    ///
    /// Le contrôle ne porte QUE sur les exemptions invoquant la fenêtre : les
    /// autres (table système, colonne `is_required()`) reposent sur des
    /// raisonnements que la version seule ne décide pas. Le marqueur est la
    /// chaîne `Hors fenêtre` en tête de justification — d'où l'exigence, dans le
    /// message d'échec, de l'écrire ainsi.
    #[test]
    fn exemptions_claiming_out_of_window_really_are_out_of_window() {
        let window = last_table_creating_migration();
        let mut checked = 0usize;

        for (version, justification) in EXEMPT_MIGRATIONS {
            if !justification.starts_with("Hors fenêtre") {
                continue;
            }
            checked += 1;
            assert!(
                *version < window,
                "exemption {version} : la justification invoque « Hors fenêtre », mais la version \
                 est POSTÉRIEURE à la dernière migration créatrice de table applicative \
                 ({window}). Un backup assez ancien pour la déclencher est donc IMPORTABLE, et son \
                 backfill ne sera jamais rejoué. → l'inscrire à POST_RESTORE_BACKFILLS, ou motiver \
                 l'exemption autrement."
            );
        }

        // Garde de non-vacuité : sans elle, une dérive du marqueur textuel rendrait
        // ce test vert en ne vérifiant plus rien. Le nombre est codé en dur À
        // DESSEIN — l'ajouter à ce compteur est le geste qui force à relire la
        // justification qu'on vient d'écrire.
        //
        // ⚠️ **4 → 6 (Story 24-2, #371), et les deux entrantes viennent du
        // REGISTRE, pas de nulle part** : `20260722000001` et `20260729000001`
        // y figuraient jusqu'à ce que la création de `invoice_settlements`
        // referme la fenêtre au-delà d'elles. Leurs justifications ont été
        // relues à ce titre — c'est exactement l'office de ce compteur.
        assert_eq!(
            checked,
            6,
            "attendu 6 exemptions marquées « Hors fenêtre » sur {}, trouvé {checked} — soit une \
             justification a été ajoutée sans le marqueur (et échappe alors au contrôle), soit le \
             marqueur a dérivé et ce test est devenu MUET.",
            EXEMPT_MIGRATIONS.len()
        );
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

    /// **Le sens INVERSE, et c'est lui qui manquait.**
    ///
    /// « Chaque statement de l'extrait est dans la source » est vrai d'un extrait
    /// auquel on a **retiré** un statement — et vrai *a fortiori* d'un extrait
    /// **vide**, qui ne fait alors tourner aucune assertion. Or un extrait amputé
    /// est exactement ce que produit un rebase malheureux, et l'en-tête du
    /// fichier de classe B annonce lui-même une chaîne « ORDRE À PRÉSERVER » où
    /// le dernier statement lit ce que le neuvième vient de poser : en retirer un
    /// casse la chaîne **en silence**, `rows_affected` restant un compteur
    /// informatif dont zéro est légitime.
    ///
    /// Ce test compte donc les statements d'écriture des deux côtés et exige
    /// l'égalité. Il ferme du même coup la tautologie de la classe A : quand
    /// `entry.sql` **est** la migration source (`include_str!` du fichier
    /// lui-même), le test précédent compare la source à elle-même et ne peut pas
    /// échouer ; celui-ci reste, lui, une contrainte réelle sur la cardinalité.
    #[test]
    fn extract_carries_every_write_statement_of_its_source_migration() {
        for entry in POST_RESTORE_BACKFILLS {
            let source = &crate::MIGRATOR
                .migrations
                .iter()
                .find(|m| m.version == entry.version)
                .expect("migration du registre présente dans le MIGRATOR")
                .sql;

            let in_source = split_statements(source)
                .iter()
                .filter(|s| writes_data(s))
                .count();
            let in_extract = split_statements(entry.sql)
                .iter()
                .filter(|s| writes_data(s))
                .count();

            assert!(
                in_extract > 0,
                "l'extrait de {} ne porte AUCUN statement d'écriture. Un extrait vide se rejoue \
                 sans erreur, rapporte `rows_affected = 0` — valeur légitime par ailleurs — et \
                 rend un `info!` « backfill post-restore rejoué » indiscernable du succès. \
                 `include_str!` repointé, ou fichier vidé ?",
                entry.label
            );
            assert_eq!(
                in_extract, in_source,
                "l'extrait de {} porte {in_extract} statement(s) d'écriture pour {in_source} dans \
                 la migration source. Un extrait AMPUTÉ passe le test de sous-texte verbatim (les \
                 statements restants sont toujours des sous-textes) : seule cette cardinalité le \
                 rattrape.",
                entry.label
            );
        }
    }

    /// **Le verrou anti-DDL de la classe A, symétrique de celui de la classe B.**
    ///
    /// « Backfill pur, aucun DDL » était une affirmation de prose, alors que
    /// l'en-tête du fichier de classe B documente précisément que c'est ce qui
    /// casse un rejeu : une migration mixte rejouée en bloc échoue dès son premier
    /// `ALTER TABLE` (MariaDB 1060, colonne déjà présente) — et comme le rejeu vit
    /// dans la transaction de restore, elle ferait échouer **tous** les imports.
    ///
    /// Le contrôle vaut pour les deux classes : un extrait de classe B n'a pas
    /// davantage à porter de DDL.
    #[test]
    fn registry_sql_contains_no_ddl() {
        const DDL: &[&str] = &[
            "ALTER TABLE",
            "CREATE TABLE",
            "DROP TABLE",
            "CREATE INDEX",
            "DROP INDEX",
            "RENAME TABLE",
            "TRUNCATE",
        ];
        for entry in POST_RESTORE_BACKFILLS {
            for statement in split_statements(entry.sql) {
                let up = normalize(&statement).to_ascii_uppercase();
                for keyword in DDL {
                    assert!(
                        !up.starts_with(keyword),
                        "l'entrée {} porte du DDL (`{keyword}`) dans son SQL rejoué. Rejoué sur \
                         une base à jour, il échouerait et ferait échouer TOUT import. Une \
                         migration mixte DDL + données exige un EXTRAIT de ses seuls statements \
                         de données. Statement :\n{statement}",
                        entry.label
                    );
                }
            }
        }
    }

    /// **Les deux limites déclarées de [`split_statements`], tenues.**
    ///
    /// Voir son doc-comment : la prose annonçait « aucune migration du dépôt
    /// n'utilise `/* */` », sans rien pour le vérifier le jour où ce serait faux.
    /// Un `UPDATE` enfermé dans un commentaire de bloc échappe à
    /// [`writes_data`] — donc au triage — **en silence**.
    #[test]
    fn migrations_contain_no_block_comment() {
        let offenders: Vec<String> = crate::MIGRATOR
            .migrations
            .iter()
            .filter(|m| m.sql.contains("/*"))
            .map(migration_file_name)
            .collect();
        assert!(
            offenders.is_empty(),
            "migration(s) contenant un commentaire de bloc `/* */` : {offenders:?}\n\
             `split_statements` ne les connaît pas : un statement d'écriture enfermé dans un tel \
             bloc, ou un `;` à l'intérieur, échappe au triage SANS que rien ne le signale.\n\
             → soit réécrire le commentaire en `--`, soit enseigner les blocs à \
             `split_statements` (et étendre ce test)."
        );
    }

    /// Second volet : le SQL **réellement exécuté** ne doit pas contenir de `--`
    /// ni de `;` à l'intérieur d'un littéral, que le découpage tronquerait.
    ///
    /// Le contrôle est délibérément borné au registre, et non étendu à toutes les
    /// migrations : le dépôt contient **déjà** un `;` littéral
    /// (`20260505000001_bank_profiles`, `IN (',', ';', '\t')`), parfaitement
    /// inoffensif puisque ce fichier n'est jamais rejoué. Élargir le test le
    /// rendrait rouge sans qu'aucun défaut n'existe — et le vrai risque n'est pas
    /// l'analyse, c'est l'**exécution**.
    #[test]
    fn registry_sql_has_no_literal_hazard() {
        for entry in POST_RESTORE_BACKFILLS {
            let hazards = literal_hazards(entry.sql);
            assert!(
                hazards.is_empty(),
                "l'entrée {} porte un `--` ou un `;` DANS un littéral : {hazards:?}. \
                 `split_statements` tronque au premier `--` et découpe au premier `;` sans \
                 connaître les apostrophes — ce SQL partirait TRONQUÉ à MariaDB.",
                entry.label
            );
        }
    }

    /// **Le contrat d'archive de l'audit.**
    ///
    /// Ces trois chaînes partent dans le détail JSON d'`admin.full_import` et y
    /// restent des années. Les figer dans un test, c'est faire de leur
    /// modification un geste **délibéré** : sans lui, renommer un variant de
    /// [`ReplayOutcome`] suffirait à changer le contenu d'enregistrements
    /// d'audit déjà écrits, en silence.
    #[test]
    fn replay_outcome_codes_are_stable() {
        assert_eq!(
            ReplayOutcome::ReplayedUnconditional.code(),
            "REPLAYED_UNCONDITIONAL"
        );
        assert_eq!(ReplayOutcome::Skipped.code(), "SKIPPED");
        let sentinels = ReplayOutcome::ReplayedSentinelsAbsent(vec!["accounts.role".into()]);
        assert_eq!(sentinels.code(), "REPLAYED_SENTINELS_ABSENT");

        assert_eq!(sentinels.missing_sentinels(), ["accounts.role"]);
        assert!(ReplayOutcome::Skipped.missing_sentinels().is_empty());
        assert!(
            ReplayOutcome::ReplayedUnconditional
                .missing_sentinels()
                .is_empty()
        );
    }

    /// Le détecteur de littéraux dangereux doit lui-même discriminer : sans ce
    /// test, `literal_hazards` pourrait rendre `vec![]` sur tout et
    /// `registry_sql_has_no_literal_hazard` serait vert à vide.
    #[test]
    fn literal_hazards_discriminates() {
        assert!(literal_hazards("UPDATE t SET a = 'x' WHERE b = 1;").is_empty());
        assert!(
            literal_hazards("-- commentaire avec l'apostrophe et un ; dedans\nUPDATE t SET a = 1;")
                .is_empty(),
            "un `;` en prose de commentaire n'est pas un littéral"
        );
        assert_eq!(
            literal_hazards("UPDATE t SET a = 'x;y';").len(),
            1,
            "un `;` dans un littéral doit être signalé"
        );
        assert_eq!(
            literal_hazards("UPDATE t SET a = 'poste -- 4000';").len(),
            1,
            "un `--` dans un littéral doit être signalé"
        );
        assert!(
            literal_hazards("UPDATE t SET a = 'l''apostrophe doublée' WHERE b = 1;").is_empty(),
            "`''` est une apostrophe littérale, pas une fermeture de littéral"
        );
    }

    /// Normalise les blancs pour comparer un statement à son fichier source sans
    /// dépendre de l'indentation résiduelle du découpage.
    fn normalize(s: &str) -> String {
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// **Condition de validité de la classe B** : la colonne sentinelle doit être
    /// créée par la migration elle-même, sans quoi « colonne présente » n'implique
    /// pas « backfill appliqué » et la sentinelle ment.
    ///
    /// Le contrôle porte sur le couple **(table, colonne)** et sur le nom
    /// **entier**. Un simple `contains("ADD COLUMN ROLE")` vérifiait moins que ce
    /// qu'il annonçait : il ignorait le nom de table — une sentinelle
    /// `("invoice_lines", "role")` serait passée au seul motif que la migration
    /// fait `ALTER TABLE accounts ADD COLUMN role` — et matchait par préfixe, donc
    /// `ADD COLUMN role_label` validait une sentinelle `role`.
    #[test]
    fn class_b_sentinel_column_is_added_by_its_own_migration() {
        for entry in POST_RESTORE_BACKFILLS {
            let BackfillTrigger::Sentinels(sentinels) = entry.trigger else {
                continue;
            };
            let source = &crate::MIGRATOR
                .migrations
                .iter()
                .find(|m| m.version == entry.version)
                .expect("migration du registre présente dans le MIGRATOR")
                .sql;
            for (table, column) in sentinels {
                assert!(
                    adds_column(source, table, column),
                    "sentinelle {table}.{column} de l'entrée {} : la colonne n'est PAS créée par \
                     cette migration SUR CETTE TABLE. « Colonne présente » n'implique alors pas \
                     « backfill appliqué » — l'entrée doit passer en classe A.",
                    entry.version
                );
            }
        }
    }

    /// Vrai si le SQL contient un `ALTER TABLE <table> … ADD COLUMN <column>` où
    /// `column` est le nom **entier** (pas un préfixe).
    ///
    /// L'analyse porte sur le SQL **décommenté et découpé en statements**
    /// (`split_statements`), jamais sur le texte brut. Les migrations du dépôt
    /// citent abondamment `ALTER TABLE` et `ADD COLUMN` **en prose de
    /// commentaire** — `20260722000001`, précisément la migration de la seule
    /// entrée de classe B, le fait elle-même deux fois (`:61`, `:67`). Analyser
    /// le texte brut faisait reposer ce verrou sur un **accident
    /// d'ordonnancement** : ces deux commentaires-là précèdent le premier
    /// `ALTER TABLE` littéral, donc le `.skip(1)` d'une rédaction antérieure les
    /// écartait par chance. Un commentaire placé **après** aurait validé une
    /// sentinelle qu'aucune colonne ne porte. C'est la troisième occurrence de
    /// la classe de défaut que le doc-module recense (le mono-ligne, puis
    /// `INSERT IGNORE`), et la première introduite par une remédiation.
    ///
    /// Le découpage borne en outre la portée d'un `ALTER TABLE` au `;` qui le
    /// termine — la version antérieure la bornait au prochain `ALTER TABLE`
    /// **textuel**, donc à la fin du fichier lorsqu'il n'y en avait pas d'autre.
    fn adds_column(sql: &str, table: &str, column: &str) -> bool {
        let table_up = table.to_ascii_uppercase();
        let column_up = column.to_ascii_uppercase();

        split_statements(sql).iter().any(|statement| {
            let up = normalize(statement).to_ascii_uppercase().replace('`', "");
            let Some(scope) = up.strip_prefix("ALTER TABLE ") else {
                return false;
            };
            if scope.split([' ', '(']).next() != Some(table_up.as_str()) {
                return false;
            }
            scope.match_indices("ADD COLUMN ").any(|(idx, marker)| {
                scope[idx + marker.len()..]
                    .split([' ', '(', ',', ';'])
                    .next()
                    == Some(column_up.as_str())
            })
        })
    }

    /// `adds_column` doit discriminer, sans quoi le verrou de la classe B
    /// redeviendrait un `contains` déguisé.
    #[test]
    fn adds_column_discriminates() {
        let sql = "ALTER TABLE accounts\n  ADD COLUMN role VARCHAR(32) NULL,\n  \
                   ADD COLUMN postable BOOLEAN NOT NULL DEFAULT TRUE;\n\
                   ALTER TABLE invoice_lines ADD COLUMN role_label VARCHAR(8) NULL;";

        assert!(adds_column(sql, "accounts", "role"));
        assert!(adds_column(sql, "accounts", "postable"));
        assert!(
            !adds_column(sql, "invoice_lines", "role"),
            "`role_label` ne doit PAS valider une sentinelle `role` (match par préfixe)"
        );
        assert!(
            !adds_column(sql, "invoice_lines", "postable"),
            "la colonne doit être ajoutée SUR LA TABLE de la sentinelle"
        );
        assert!(
            !adds_column(sql, "accounts", "role_label"),
            "`role_label` est ajoutée sur invoice_lines, pas sur accounts"
        );

        // Un `ADD COLUMN` en PROSE DE COMMENTAIRE ne crée aucune colonne.
        // Le commentaire est placé APRÈS le DDL, délibérément : c'est la
        // position que l'analyse du texte brut ne protégeait pas. Les
        // commentaires de ce genre sont nombreux dans le dépôt (une vingtaine
        // de migrations écrivent « ADD COLUMN nullable » en prose), et
        // `20260722000001` en porte deux — mais avant son `ALTER TABLE`, ce qui
        // masquait le défaut.
        let prose = "ALTER TABLE accounts ADD COLUMN role VARCHAR(32) NULL;\n\
                     -- Non-breaking : ALTER TABLE invoice_lines ADD COLUMN project_id BIGINT.\n";
        assert!(
            adds_column(prose, "accounts", "role"),
            "le DDL réel doit rester reconnu une fois les commentaires retirés"
        );
        assert!(
            !adds_column(prose, "invoice_lines", "project_id"),
            "un `ADD COLUMN` en PROSE DE COMMENTAIRE ne crée aucune colonne : la \
             sentinelle mentirait et la classe B perdrait sa condition de validité"
        );
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
