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
//! `kesh-api`) applique [`crate::backup::check_backup_inventory`] — toute table
//! du manifeste appartient à [`crate::backup::TABLES_TO_TRUNCATE`], et toute
//! table créée au plus tard par le **plancher d'import**
//! [`crate::backup::IMPORT_FLOOR_MIGRATION`] y figure ; une table créée après le
//! plancher peut manquer, pourvu qu'elle soit postérieure à toutes celles que le
//! fichier porte (Story 32-12a, #722). **Un backup n'est donc importable que
//! s'il provient d'un binaire postérieur ou égal au plancher.**
//!
//! C'est cette fenêtre qui fixe le contenu du registre — et non le nombre de
//! migrations portant un backfill. La plupart des migrations qui écrivent des
//! données sont hors d'atteinte de ce mécanisme et sont **exemptées**
//! (cf. [`EXEMPT_MIGRATIONS`], qui porte la liste et la justification de
//! chacune — ⚠️ ne pas récrire ici un décompte, il se périmerait en silence à la
//! prochaine migration ; la source se lit, elle ne se résume pas).
//! La fenêtre est le plancher, une constante écrite : une table neuve ne la
//! déplace plus, et seul le **relèvement** du plancher — décision du Project
//! Lead, jamais un geste de maintenance — périmerait les entrées antérieures.
//! Le test `registry_entries_are_within_import_window` compare chaque entrée au
//! plancher et échoue bruyamment plutôt que de laisser du code mort qui *paraît*
//! fonctionner.
//!
//! ⚠️ **Une table créée après le plancher arrive VIDE d'une sauvegarde qui la
//! précède** (`restore_body` vide toute table de l'inventaire, n'insère que
//! celles que porte le fichier). Si sa migration — ou une migration ultérieure —
//! y écrit des lignes, l'argument « couverture » d'une exemption est faux pour
//! elle : la migration va au registre (test `post_floor_table_seeds_are_replayed`).
//!
//! # Deux classes d'entrées, et le déclencheur n'est pas le même
//!
//! **Classe A — auto-gardée, rejeu inconditionnel.** Tous ses statements sont
//! gardés contre l'écrasement d'une valeur posée par l'utilisateur ; sur une
//! base à jour, le rejeu ne délettre ni ne réécrit rien — il n'y a donc rien à
//! conditionner. ⚠️ Il **peut poser** ce que la base aurait posé aujourd'hui
//! (une pièce rendue lettrable depuis, un exercice rouvert depuis : Story
//! 15-1a2-ii, M1) : « no-op » ne vaut que pour une base **nominale**, sans ces
//! états — l'entrée qui le peut le dit dans sa justification.
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
//! Des entrées peuvent ne PAS être indépendantes — l'exemple canonique vit
//! aujourd'hui dans [`RETIRED_BACKFILLS`] : `20260722000001` attribue le
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
    /// **Classe A** : tous les statements sont gardés contre l'écrasement d'une
    /// valeur posée par l'utilisateur ; sur une base à jour, le rejeu ne
    /// délettre ni ne réécrit rien, mais **peut poser** ce que la base aurait
    /// posé aujourd'hui (pièce rendue lettrable depuis — Story 15-1a2-ii).
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
/// N'y figurent que les migrations de backfill **dans** la fenêtre
/// d'importabilité ; les autres sont hors fenêtre ou portent sur une table
/// système, et sont énumérées avec leur justification dans
/// [`EXEMPT_MIGRATIONS`].
pub const POST_RESTORE_BACKFILLS: &[PostRestoreBackfill] = &[
    // ⚠️ **VIDE de la Story 25-2-c (#381) à la Story 15-1a2-ii (#518)** — la
    // création de `journal_entry_number_sequences` (20260917000001) avait refermé
    // la fenêtre d'importabilité au-delà des deux entrées qu'il portait
    // (`20260828000001`, `20260910000001`, désormais dans [`RETIRED_BACKFILLS`]).
    // Il n'était pas mort, il attendait : M1 ci-dessous le remplit de nouveau.
    //
    // ⚠️ **Ce que M1 rend aux tests unitaires qui ne parcourent QUE ce registre**
    // (décompte de la 15-1a2-ii) : TROIS reprennent matière — couverture des
    // statements d'écriture, absence de DDL, littéraux dangereux ; DEUX restent
    // sans prise sur M1 — `class_b_sentinel_column_is_added_by_its_own_migration`
    // saute toute entrée qui n'est pas `Sentinels`, et l'extraction verbatim
    // compare, pour un `include_str!` de la migration entière, la source à
    // elle-même. Les tests de la machinerie de rejeu gardent leur matière par
    // `post_restore_class_a.rs` (`.chain(RETIRED_BACKFILLS.iter())`).
    PostRestoreBackfill {
        version: 20261010000001,
        label: "20261010000001_lettering_documents_backfill.sql",
        // CLASSE A. Les trois `UPDATE` (groupes `document` des factures clientes,
        // des factures fournisseurs payées, paires `reversal` des achats annulés)
        // sont gardés `lettering_key IS NULL` sur TOUTES les lignes du groupe
        // candidat : le rejeu ne délettre jamais et ne réécrit jamais une marque
        // posée. Aucune des lignes visées ne peut porter un `NULL` CHOISI : un
        // groupe `document` ne se dissout pas à la main (`LETTERING_IS_DOCUMENT`)
        // et se recalcule depuis les pièces — un groupe dissous à bon droit
        // (règlement annulé) ne se reforme pas, la pièce ayant changé ; une paire
        // `reversal` dont une ligne est celle d'une pièce ne se dissout pas à la
        // main non plus (C106).
        //
        // ⚠️ **Ce que la classe A ne garantit PAS ici** (C-15-1a2-16) : ce n'est
        // pas un « no-op strict ». Le rejeu PEUT LETTRER ce que la base à jour
        // avait laissé ouvert à bon droit — (1) une pièce historique entièrement
        // close à la mise à jour, dont un administrateur a depuis rouvert
        // l'exercice ou reculé la borne (rien ne la resynchronise en vivant) ;
        // (2) une pièce dont le compte n'était pas lettrable au geste et l'est
        // devenu. Le groupe posé est celui que la synchronisation poserait
        // aujourd'hui, et aucun geste de l'utilisateur ne l'avait défait (une
        // ligne de pièce n'est jamais délettrable à la main) — ce n'est pas
        // écraser un choix. Le manuel d'administration le dit.
        //
        // Hors fenêtre ? Non : elle est POSTÉRIEURE au plancher d'import
        // `IMPORT_FLOOR_MIGRATION` (`registry_entries_are_within_import_window`),
        // si bien qu'une sauvegarde qui la précède reste importable — et a besoin
        // de ce rejeu. Une table créée après le plancher ne change rien à cela.
        trigger: BackfillTrigger::Unconditional,
        sql: include_str!("../migrations/20261010000001_lettering_documents_backfill.sql"),
    },
];

// ⚠️ **Le registre a été VIDE entre les Stories 24-2 et 24-3, et ce n'était pas
// un oubli — c'était le mécanisme qui fonctionnait.**
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
// que `exemptions_claiming_out_of_window_really_are_out_of_window` compare au
// plancher d'import `IMPORT_FLOOR_MIGRATION` — l'affirmation est donc vérifiée,
// pas crue sur parole.
//
// ⚠️ **Depuis la Story 32-12a (#722), une nouvelle table applicative n'a plus
// cet effet** : la fenêtre est le plancher, gelé à la v0.15.0, et seul son
// relèvement — décision du Project Lead — périmerait des entrées. Le mécanisme
// reste la seule réponse au cas « le backup est dans la fenêtre et précède un
// backfill ».
//
// ⛔ **Et ce cas s'est représenté à la story SUIVANTE** : `20260828000001`
// (Story 24-3) est postérieure à la table qui avait refermé la fenêtre, donc un
// backup pris entre les deux est importable ET dépourvu de la colonne. Le
// registre n'était pas mort, il attendait.

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
    PostRestoreBackfill {
        version: 20260828000001,
        label: "20260828000001_invoice_settlements_type.sql",
        // CLASSE B — cf. le doc-comment du fichier extrait. La sentinelle est
        // valide : `settlement_bank_account_id` est ajoutée par le MÊME
        // `ALTER TABLE` que l'`UPDATE` de backfill.
        //
        // ⚠️ **Première entrée depuis que la Story 24-2 a vidé ce registre**, et
        // c'est le mécanisme qui reprend exactement son office : la 24-2 avait
        // refermé la fenêtre d'importabilité en créant `invoice_settlements`
        // (20260827000001) ; cette migration lui est POSTÉRIEURE, donc un backup
        // situé entre les deux est parfaitement importable — et il porte la table
        // sans la colonne. C'est le cas que ce dispositif existe pour rattraper.
        trigger: BackfillTrigger::Sentinels(&[(
            "invoice_settlements",
            "settlement_bank_account_id",
        )]),
        sql: include_str!("post_restore/20260828000001_invoice_settlements_type.sql"),
    },
    PostRestoreBackfill {
        version: 20260910000001,
        label: "20260910000001_audit_log_actor_label.sql",
        // CLASSE B — la sentinelle est valide : `actor_label` est ajoutée par le
        // MÊME `ALTER TABLE` que l'`UPDATE` de backfill, donc son absence prouve que
        // la migration n'a pas tourné sur les données restaurées.
        //
        // ⚠️ **Le cas de déclenchement est ici PARTICULIER, et il vaut d'être écrit** :
        // à partir de la Story 25-1a, `audit_log` n'est plus effacée ni restaurée par
        // l'import — elle rejoint `onboarding_state` dans l'exclusion, et les entrées
        // de l'archive sont FUSIONNÉES. Le rejeu ne porte donc pas sur des lignes
        // « restaurées » au sens habituel, mais sur celles **issues du backup** :
        // écrites par une instance antérieure à cette migration, elles arrivent avec
        // `actor_label` vide et le rejeu leur donne un nom.
        //
        // ⛔ Sans ce rejeu, la fusion produirait exactement le trou qu'elle prétend
        // fermer : des entrées conservées dont l'acteur ne serait nommé nulle part —
        // ni par la FK, qui n'existe plus, ni par le libellé, qui serait vide.
        trigger: BackfillTrigger::Sentinels(&[("audit_log", "actor_label")]),
        sql: include_str!("post_restore/20260910000001_audit_log_actor_label.sql"),
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
/// compare la version au plancher d'import `IMPORT_FLOOR_MIGRATION` et vérifie
/// l'affirmation. Une
/// justification « hors fenêtre » rédigée autrement échappe au contrôle et
/// redevient une affirmation crue sur parole — laquelle, si elle est fausse,
/// désactive **définitivement et en silence** le rejeu du backfill concerné.
/// **Sur quoi repose une exemption — et si ce fondement peut CESSER d'être vrai.**
///
/// Le champ existe parce qu'un marqueur textuel dans la justification ne suffit
/// pas. Écrit en passe 2 de revue de code de la Story 24-5, il a été pris en
/// défaut **quatre fois en deux passes** : le `grep` du script de release
/// comptait aussi les citations du marqueur par le test qui le gardait (1 → 5) ;
/// l'épreuve par mutation a montré que ce test ne couvrait pas l'ajout d'une
/// entrée sans marqueur ; `rustfmt` collapse une entrée courte sur une ligne, ce
/// qui faisait réattribuer par l'`awk` la version de l'entrée précédente ; et une
/// continuation `\` coupant la chaîne entre « SE » et « PÉRIME » est invisible
/// au `grep` alors que `contains()` la voit.
///
/// Aucun de ces quatre modes d'échec n'était une faute d'écriture : ils tiennent
/// tous à ce qu'un shell lisait une chaîne accentuée dans du source Rust. Le
/// fondement se DÉCLARE donc, et se lit comme une donnée.
///
/// *(Refondu en passe 3, findings P3-2 et P3-3.)*
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExemptionBasis {
    /// Le fondement ne se périme pas : fenêtre d'importabilité, table système,
    /// propriété du schéma. Rien de ce qu'on publiera ne peut le rendre faux.
    Durable,
    /// Le fondement est un **fait daté** : « aucune version publiée ne se situe
    /// dans `[borne .. la migration exemptée)` ». Publier une version dans cet
    /// intervalle le rend faux — et le rejeu du backfill est alors désactivé
    /// définitivement, en silence.
    ///
    /// La valeur portée est la **borne basse** de l'intervalle déclaré vide.
    PerishableSince(i64),
}

pub const EXEMPT_MIGRATIONS: &[(i64, ExemptionBasis, &str)] = &[
    (
        20260828000001,
        ExemptionBasis::Durable,
        "Hors fenêtre depuis 20260917000001 (journal_entry_number_sequences, Story 25-2-c) : \
         un backup assez ancien pour porter invoice_settlements sans \
         settlement_bank_account_id est dépourvu de la table des compteurs d'écriture, donc \
         refusé au contrôle de couverture. Figurait au registre de rejeu jusque-là ; l'y \
         laisser aurait produit du code mort qui paraît fonctionner.",
    ),
    (
        20260910000001,
        ExemptionBasis::Durable,
        "Hors fenêtre depuis 20260917000001 (journal_entry_number_sequences, Story 25-2-c) : \
         même raison que 20260828000001. Elle était de CLASSE B, conditionnée à l'absence de \
         audit_log.actor_label — un cas de déclenchement qui ne peut plus se présenter, le \
         backup correspondant étant refusé en amont.",
    ),
    (
        20260917000001,
        ExemptionBasis::Durable,
        "La migration crée elle-même journal_entry_number_sequences : un backup dépourvu de ses \
         lignes d'amorçage est un backup dépourvu de la TABLE, donc refusé au contrôle de \
         couverture de parse_and_verify ; un backup postérieur les porte, et l'import les \
         restaure puisque la table est dans TABLES_TO_TRUNCATE. Le cas de déclenchement ne peut \
         se présenter d'aucun côté. ⚠️ NE PAS écrire « Hors fenêtre » ici, quoique le fondement \
         ressemble à celui de 20260428000001 (vat_rates) : ce marqueur est réservé à un fait de \
         CHRONOLOGIE, et cette migration EST la dernière créatrice de table applicative — donc la \
         fenêtre elle-même. `exemptions_claiming_out_of_window_really_are_out_of_window` exige \
         version < window et l'a refusé, à juste titre. Le fondement est ici la COUVERTURE, pas \
         la fenêtre.",
    ),
    (
        20260909000001,
        ExemptionBasis::PerishableSince(20260827000001),
        "Parc vide, et NON « hors fenêtre » — cette migration EST dans la fenêtre d'importabilité, \
         et invoquer la fenêtre serait faux. L'argument porte sur le PARC : l'intervalle où un \
         backup porterait `postable = TRUE` sur les comptes de clôture est \
         [20260827000001 .. celle-ci), et AUCUNE version publiée ne s'y trouve — la dernière est \
         v0.11.1 (2026-08-24), antérieure à la borne basse ; l'instance en service exécute cette \
         version (arbitrage du 2026-09-09). L'intervalle ne contient que des builds de \
         développement non distribués. \
         POURQUOI PAS LE REGISTRE : l'entrée serait de classe A, or son UPDATE n'est PAS gardé \
         contre l'écrasement d'une valeur posée par l'utilisateur — `postable` est réouvrable par \
         `PUT /api/v1/accounts/{id}` (sémantique full-replace, cf. `effective_postable`), et rien \
         ne distingue « jamais touché » de « rouvert exprès ». La classe A démentirait donc la \
         promesse publiée du manuel d'administration (« vos données ne sont jamais écrasées »), \
         et la classe B est indisponible faute de DDL offrant une sentinelle. \
         ⚠️ CE FONDEMENT SE PÉRIME, d'où `ExemptionBasis::PerishableSince(20260827000001)` \
         ci-dessus : il cesserait d'être vrai si une version était publiée depuis `main` dans \
         l'intervalle. Ce n'est plus une affirmation sur parole — `scripts/prepare-release.sh` \
         la CONTRÔLE avant chaque tag, en cherchant un tag publié dont l'arbre porte la borne \
         basse sans porter cette migration, et refuse la release s'il en trouve un \
         — Story 24-5, #375.",
    ),
    (
        20260915000001,
        ExemptionBasis::PerishableSince(20260827000001),
        "Parc vide, et NON « hors fenêtre » — cette migration ne crée aucune table : elle EST dans \
         la fenêtre d'importabilité, et invoquer la fenêtre serait faux. L'argument porte sur le \
         PARC : un backup importable porte `invoice_settlements` (20260827000001), et il ne manque \
         `audit_log.company_id` que s'il précède celle-ci. L'intervalle \
         [20260827000001 .. celle-ci) ne contient AUCUNE version publiée — la dernière est v0.11.1 \
         (2026-08-24), antérieure à la borne basse ; l'instance en service exécute cette version. \
         POURQUOI PAS LE REGISTRE : la classe B serait techniquement disponible — le DDL et \
         l'UPDATE sont dans le même fichier —, mais elle est écartée par l'arbitrage du 2026-09-11, \
         « ni COALESCE ni rejeu post-restore ». Un rejeu gardé `company_id IS NULL` ne \
         distinguerait pas une entrée d'archive d'une entrée LOCALE restée NULL, dont le `user_id` \
         désigne, après le remplacement de `users`, le porteur de cet identifiant dans l'instance \
         SOURCE : il l'attribuerait à la mauvaise société. NULL signifie « société \
         indéterminable », un état légitime et permanent. CE QUE CELA COÛTE : un backup pris dans \
         l'intervalle — build de développement non distribué — fusionne ses entrées avec \
         `company_id = NULL`. \
         ⚠️ CE FONDEMENT SE PÉRIME, d'où `ExemptionBasis::PerishableSince(20260827000001)` \
         ci-dessus : il cesserait d'être vrai si une version était taguée depuis `main` avant le \
         merge de cette migration — d'où l'arbitrage du 2026-09-15, aucune release avant ce merge. \
         `scripts/prepare-release.sh` le CONTRÔLE avant chaque tag — Story 25-1c-zero, refs #378.",
    ),
    (
        20260722000001,
        ExemptionBasis::Durable,
        "Hors fenêtre depuis 20260827000001 (invoice_settlements, Story 24-2) : un backup assez \
         ancien pour porter role/postable vides est dépourvu de cette table, donc refusé au \
         contrôle de couverture. Figurait au registre de rejeu jusque-là ; l'y laisser aurait \
         produit du code mort qui paraît fonctionner.",
    ),
    (
        20260729000001,
        ExemptionBasis::Durable,
        "Hors fenêtre depuis 20260827000001 (invoice_settlements, Story 24-2) : même raison que \
         20260722000001. Elle était de CLASSE A, donc rejouée inconditionnellement à chaque \
         import — la laisser au registre aurait fait exécuter à chaque restore un backfill dont \
         le cas de déclenchement ne peut plus se présenter.",
    ),
    (
        20260419000002,
        ExemptionBasis::Durable,
        "users.company_id finit NOT NULL sans défaut => is_required() vrai => un backup qui ne la \
         porte pas est refusé par check_schema_compat (400). Le cas ne peut pas se produire.",
    ),
    (
        20260428000001,
        ExemptionBasis::Durable,
        "Hors fenêtre : la migration crée elle-même vat_rates, donc un backup dépourvu de ses 4 \
         lignes de taux est un backup dépourvu de la table => refusé au contrôle de couverture.",
    ),
    (
        20260522000001,
        ExemptionBasis::Durable,
        "INSERT sur _kesh_version, table système hors TABLES_TO_TRUNCATE : jamais exportée ni \
         restaurée. Ce n'est pas un backfill de données applicatives.",
    ),
    (
        20260613000001,
        ExemptionBasis::Durable,
        "Hors fenêtre : un backup dépourvu de vat_rates.category précède 20260628000001, donc \
         n'a pas les tables supplier_invoices => refusé au contrôle de couverture.",
    ),
    (
        20260614000001,
        ExemptionBasis::Durable,
        "Hors fenêtre : un backup dépourvu des comptes 1171/2206 précède 20260628000001, donc \
         n'a pas les tables supplier_invoices => refusé au contrôle de couverture. (Version de \
         référence citée EXPLICITEMENT — un « antérieur aux tables créées depuis » ne se vérifie \
         pas.)",
    ),
    (
        20260628000001,
        ExemptionBasis::Durable,
        "Hors fenêtre, et autoréfutante : la migration crée elle-même supplier_invoices et \
         supplier_invoice_lines, donc un backup dépourvu de default_payable_account_id est \
         dépourvu de ces tables.",
    ),
    (
        20260714000002,
        ExemptionBasis::Durable,
        "UPDATE sur _kesh_version (bump kesh_version_min_required), table système jamais \
         restaurée. Ce n'est pas un backfill applicatif.",
    ),
    (
        20260814000001,
        ExemptionBasis::Durable,
        "UPDATE sur _kesh_version (bump kesh_version_min_required), table système jamais \
         restaurée. Le reste de la migration est du DDL pur ; le remplissage du parc vit HORS \
         migration (backfill_client_number_canonical, appelée au boot et en fin d'import — D6 \
         Story 22-1), ce qui tient l'esprit de P7 sans entrée au registre.",
    ),
    (
        20261010000002,
        ExemptionBasis::Durable,
        "Rejeu exclu à dessein : un groupe reversal sans ligne de pièce se délettre à la main \
         (DELETE /letterings) ; un NULL peut y être un choix de l'utilisateur, qu'un rejeu à \
         chaque import réécrirait en silence. Coût assumé : une sauvegarde d'avant la 15-1a2-ii \
         importée laisse ces paires ouvertes, lettrables à la main. Les paires des écritures \
         d'achat, elles, sont portées par 20261010000001, au registre (Story 15-1a2-ii, #518).",
    ),
    (
        20261009000001,
        ExemptionBasis::Durable,
        "UPDATE sur _kesh_version (bump kesh_version_min_required), table système jamais \
         restaurée. Le reste de la migration est du DDL pur (colonnes de lettrage, Story \
         15-1a-i) : une sauvegarde antérieure les restaure à NULL, aucun groupe n'existait.",
    ),
    (
        20261011000001,
        ExemptionBasis::Durable,
        "UPDATE sur _kesh_version (bump kesh_version_min_required), table système jamais \
         restaurée. Le reste de la migration est du DDL pur (colonne debit_sign, plafond de \
         header_row_count, Story 26-1) : une sauvegarde antérieure restaure la colonne à son \
         DEFAULT 'positive', qui est la sémantique des profils d'avant.",
    ),
    (
        20261015000001,
        ExemptionBasis::Durable,
        "UPDATE sur _kesh_version (bump kesh_version_min_required), table système jamais \
         restaurée. Le reste de la migration est du DDL pur (sept colonnes QR rendues \
         facultatives, contrainte « tout ou rien », unicité de supplier_invoice_id, Story \
         30-2a) : les lignes d'une sauvegarde antérieure portent toutes un QR et satisfont la \
         contrainte telles quelles ; aucune valeur n'est à recalculer.",
    ),
    (
        20261024000001,
        ExemptionBasis::Durable,
        "INSERT sur _kesh_backup_schedule, table système hors TABLES_TO_TRUNCATE : jamais \
         exportée ni restaurée (réglage et état de la sauvegarde automatique, Story 31-1c). \
         Ce n'est pas un backfill de données applicatives — patron de 20260522000001.",
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
/// Une table entièrement absente compte comme sentinelle absente. L'état est
/// **atteignable** depuis la Story 32-12a (#722) : `parse_and_verify` tolère
/// qu'une table créée après le plancher d'import `IMPORT_FLOOR_MIGRATION`
/// manque à une sauvegarde qui la précède — la sauvegarde précède alors aussi
/// toute colonne de cette table, et le rejeu conditionné doit avoir lieu.
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
///
/// `pub(crate)` depuis la Story 32-12a (#722) : [`crate::backup`] l'emploie pour
/// dater les tables de l'inventaire ([`created_application_table`]).
pub(crate) fn split_statements(sql: &str) -> Vec<String> {
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

/// Normalise les blancs (toute suite de blancs, sauts de ligne compris, devient
/// une espace) — pour comparer un statement à son fichier source sans dépendre
/// de l'indentation résiduelle du découpage, et pour lire l'en-tête d'un
/// statement dont le nom est rejeté à la ligne suivante.
pub(crate) fn normalize(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// L'entrée de [`crate::backup::TABLES_TO_TRUNCATE`] que crée ce statement, ou
/// `None` s'il n'en crée aucune — Story 32-12a (#722), AC1.
///
/// C'est la brique de la carte **table → migration créatrice**
/// ([`crate::backup::table_creators`]), qui date chaque table de l'inventaire
/// depuis le `MIGRATOR` au lieu d'une liste écrite à la main. Jusqu'à la 32-12a,
/// cette reconnaissance vivait dans les tests (elle y calculait une fenêtre
/// d'importabilité qui est désormais le plancher écrit
/// [`crate::backup::IMPORT_FLOOR_MIGRATION`]) ; elle passe en production, en un
/// seul exemplaire.
///
/// **Pourquoi [`split_statements`] suffit ici**, malgré les dangers d'un
/// analyseur naïf que liste son doc-comment (`/* */`, `--` ou `;` dans un
/// littéral) : seul l'**en-tête** du statement est lu (`CREATE TABLE <nom>`).
/// Les blocs `/* */` sont tenus absents de **toutes** les migrations par
/// `migrations_contain_no_block_comment` ; un `;` dans un littéral découpe un
/// statement en fragments dont aucun ne commence par `CREATE TABLE` sans en
/// être le vrai en-tête. Une graphie ratée rend **zéro** migration créatrice
/// pour une table : le test `every_inventory_table_has_exactly_one_creating_migration`
/// (`backup.rs`) échoue alors en la nommant, et la production, faute de date,
/// tient la table pour **exigée** — refus plutôt que tolérance à tort.
///
/// - **Filtre sur l'inventaire applicatif** : une table **système**
///   (`_kesh_version`, `_kesh_backup_schedule`) n'est pas dans l'inventaire et
///   n'a donc pas de migration créatrice dans la carte. Le SQL est décommenté
///   avant analyse ([`split_statements`]) — plusieurs migrations portent
///   `CREATE TABLE` en prose.
/// - **Reconnaissance large** : blancs normalisés (`CREATE  TABLE`, nom rejeté à
///   la ligne suivante), accents graves retirés (`` CREATE TABLE `x` ``, graphie
///   absente du dépôt aujourd'hui mais que rien n'interdit), `IF NOT EXISTS`
///   admis, nom comparé sans casse et **isolé** — un `CREATE TABLE
///   supplier_invoice_lines` ne crée pas `supplier_invoices` —, la parenthèse
///   ouvrante pouvant être collée au nom (`CREATE TABLE accounts(`).
pub(crate) fn created_application_table(statement: &str) -> Option<&'static str> {
    let up = normalize(statement).to_ascii_uppercase().replace('`', "");
    let rest = up.strip_prefix("CREATE TABLE ")?;
    let rest = rest.strip_prefix("IF NOT EXISTS ").unwrap_or(rest);
    // Le nom s'arrête au premier blanc ou à la parenthèse ouvrante.
    let name = rest
        .split([' ', '('])
        .next()
        .unwrap_or_default()
        .trim_end_matches(';');
    crate::backup::TABLES_TO_TRUNCATE
        .iter()
        .copied()
        .find(|t| t.eq_ignore_ascii_case(name))
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
    /// Un backup n'est importable que s'il porte toutes les tables créées au plus
    /// tard par le plancher d'import [`crate::backup::IMPORT_FLOOR_MIGRATION`]
    /// (Story 32-12a, #722 — `check_backup_inventory`). Une entrée de registre
    /// antérieure ou égale au plancher est du **code mort qui paraît
    /// fonctionner** : un backup assez ancien pour la déclencher est dépourvu
    /// d'une table exigée, donc refusé en 400 bien avant le rejeu.
    ///
    /// La fenêtre est le **plancher**, constante écrite et non plus calculée :
    /// une table neuve ne la déplace plus. Le plancher lui-même est tenu pour une
    /// migration créatrice réelle du `MIGRATOR` par
    /// `import_floor_is_a_real_table_creating_migration` (`backup.rs`).
    #[test]
    fn registry_entries_are_within_import_window() {
        let window = crate::backup::IMPORT_FLOOR_MIGRATION;
        for entry in POST_RESTORE_BACKFILLS {
            assert!(
                entry.version > window,
                "l'entrée {} est ANTÉRIEURE ou égale au plancher d'import \
                 IMPORT_FLOOR_MIGRATION ({window}) : un backup assez ancien pour la déclencher est \
                 dépourvu d'une table exigée, donc refusé par `parse_and_verify`. Elle doit passer \
                 en exemption.",
                entry.version
            );
        }
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
                || EXEMPT_MIGRATIONS.iter().any(|(v, _, _)| *v == m.version);
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
        for (version, _, justification) in EXEMPT_MIGRATIONS {
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
    ///
    /// La fenêtre est le plancher d'import [`crate::backup::IMPORT_FLOOR_MIGRATION`]
    /// (Story 32-12a, #722) : une migration située entre le plancher et une table
    /// créée après lui est déclenchable par une sauvegarde importable, et ne peut
    /// donc pas se dire « Hors fenêtre ».
    #[test]
    fn exemptions_claiming_out_of_window_really_are_out_of_window() {
        let window = crate::backup::IMPORT_FLOOR_MIGRATION;
        let mut checked = 0usize;

        for (version, _, justification) in EXEMPT_MIGRATIONS {
            if !justification.starts_with("Hors fenêtre") {
                continue;
            }
            checked += 1;
            assert!(
                *version < window,
                "exemption {version} : la justification invoque « Hors fenêtre », mais la version \
                 est POSTÉRIEURE ou égale au plancher d'import IMPORT_FLOOR_MIGRATION ({window}). \
                 Un backup assez ancien pour la déclencher est donc IMPORTABLE, et son backfill ne \
                 sera jamais rejoué. → l'inscrire à POST_RESTORE_BACKFILLS, ou motiver \
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
        // ⚠️ **6 → 8 (Story 25-2-c, #381)**, et les deux entrantes viennent encore
        // du REGISTRE : `20260828000001` et `20260910000001` y figuraient jusqu'à
        // ce que la création de `journal_entry_number_sequences` referme la
        // fenêtre au-delà d'elles. C'est le même mouvement qu'à la 24-2, à une
        // table près — le module l'annonçait : *« toute nouvelle table
        // applicative aura le même effet »*.
        //
        // ⛔ **La troisième exemption de cette story n'entre PAS dans ce
        // décompte**, et c'est le point à retenir : elle EST la migration qui
        // referme la fenêtre, donc elle ne peut pas l'invoquer. Sa justification
        // a d'abord porté le marqueur par analogie avec `vat_rates` ; ce test
        // l'a refusée, à juste titre, et le fondement a été réécrit en termes de
        // COUVERTURE. *Le marqueur dit un fait de chronologie, pas une famille
        // d'arguments qui se ressemblent.*
        assert_eq!(
            checked,
            8,
            "attendu 8 exemptions marquées « Hors fenêtre » sur {}, trouvé {checked} — soit une \
             justification a été ajoutée sans le marqueur (et échappe alors au contrôle), soit le \
             marqueur a dérivé et ce test est devenu MUET.",
            EXEMPT_MIGRATIONS.len()
        );
    }

    /// **Les fondements PÉRISSABLES ne sont pas laissés sans filet.**
    ///
    /// Une exemption peut reposer non sur la fenêtre d'importabilité ni sur une
    /// propriété du schéma, mais sur un FAIT DATÉ — « aucune version publiée ne
    /// se situe dans tel intervalle ». Ce fait-là cesse d'être vrai dès qu'on
    /// publie, et le test ci-dessus ne le voit pas : il ne contrôle que le
    /// fondement « Hors fenêtre ».
    ///
    /// D'où le champ [`ExemptionBasis`], que `scripts/prepare-release.sh` lit
    /// via `examples/perishable_exemptions.rs` pour vérifier, avant chaque tag,
    /// qu'aucune version publiée ne se situe dans l'intervalle déclaré vide.
    ///
    /// **Le fondement déclaré d'une exemption est-il cohérent, et exact ?**
    ///
    /// Remplace le contrôle par marqueur textuel écrit en passe 2 — voir
    /// [`ExemptionBasis`] pour les quatre modes d'échec qui l'ont condamné. Ici
    /// plus rien n'est grepé : le fondement est une donnée, ce test la lit, et
    /// `examples/perishable_exemptions.rs` la donne au script de release.
    ///
    /// Trois contrôles :
    /// 1. une justification « Hors fenêtre » ne peut pas être déclarée
    ///    périssable, ni l'inverse — les deux fondements sont exclusifs ;
    /// 2. la borne basse d'un fondement périssable désigne une migration RÉELLE
    ///    et STRICTEMENT antérieure à l'exemption : sans quoi l'intervalle
    ///    déclaré vide est vide de sens ;
    /// 3. garde de non-vacuité sur la taille du registre — toute entrée ajoutée
    ///    force à déclarer son fondement plutôt qu'à l'omettre.
    ///
    /// *(Refondu en passe 3 de revue de code de la Story 24-5, #375, findings
    /// P3-2 et P3-3.)*
    #[test]
    fn every_exemption_declares_a_coherent_basis() {
        let versions: Vec<i64> = crate::MIGRATOR
            .migrations
            .iter()
            .map(|m| m.version)
            .collect();

        for (version, basis, justification) in EXEMPT_MIGRATIONS {
            let claims_window = justification.starts_with("Hors fenêtre");
            match basis {
                ExemptionBasis::Durable => assert!(
                    !justification.contains("aucune version publiée"),
                    "exemption {version} : déclarée Durable, mais sa justification argumente sur \
                     le PARC (« aucune version publiée »), ce qui est un fait DATÉ. La déclarer \
                     PerishableSince(<borne>), faute de quoi aucune release ne la relira."
                ),
                ExemptionBasis::PerishableSince(borne) => {
                    assert!(
                        !claims_window,
                        "exemption {version} : déclarée périssable ET « Hors fenêtre ». Les deux \
                         fondements sont exclusifs — la fenêtre ne se périme pas."
                    );
                    assert!(
                        versions.contains(borne),
                        "exemption {version} : la borne {borne} ne désigne aucune migration du \
                         MIGRATOR. L'intervalle déclaré vide est donc invérifiable."
                    );
                    assert!(
                        borne < version,
                        "exemption {version} : la borne {borne} ne lui est pas antérieure — \
                         l'intervalle [{borne} .. {version}) est vide de sens."
                    );
                }
            }
        }

        // Garde de non-vacuité : une entrée ajoutée fait rougir ce test, ce qui
        // force à déclarer son fondement au lieu de le laisser par défaut.
        // Le nombre est codé en dur À DESSEIN, comme celui de « Hors fenêtre ».
        // ⚠️ **12 → 15 (Story 25-2-c, #381)**, et les trois se ventilent : deux
        // viennent du REGISTRE — `20260828000001` et `20260910000001`, que la
        // création de `journal_entry_number_sequences` a mises hors fenêtre — et
        // la troisième est cette migration même. ⛔ La sienne ne porte PAS le
        // marqueur « Hors fenêtre » : elle EST la fenêtre, et son fondement est
        // la couverture, pas la chronologie.
        // ⚠️ **15 → 16 (Story 15-1a-i, #518)** : `20261009000001`, le bump
        // `kesh_version_min_required` à 0.13.0 — `Durable`, patron des deux bumps
        // précédents (table système jamais restaurée, DDL pur sinon).
        // ⚠️ **16 → 17 (Story 15-1a2-ii, #518)** : `20261010000002`, les paires
        // de contre-passation libres — `Durable`, rejeu exclu parce qu'un `NULL`
        // y peut être un choix de l'utilisateur (`DELETE /letterings`).
        // ⚠️ **17 → 18 (Story 26-1, #620)** : `20261011000001`, le bump
        // `kesh_version_min_required` à 0.13.1 — `Durable`, patron de
        // `20261009000001` (table système jamais restaurée, DDL pur sinon).
        // ⚠️ **18 → 19 (Story 30-2a, #685)** : `20261015000001`, le bump
        // `kesh_version_min_required` à 0.14.0 — `Durable`, même patron.
        // ⚠️ **19 → 20 (Story 31-1c, #503)** : `20261024000001`, l'INSERT du
        // singleton `_kesh_backup_schedule` — `Durable`, patron de
        // `20260522000001` (table système jamais exportée ni restaurée).
        assert_eq!(
            EXEMPT_MIGRATIONS.len(),
            20,
            "le registre d'exemptions a changé de taille : déclarer le fondement de l'entrée \
             neuve (Durable, ou PerishableSince(<borne>) si elle argumente sur un fait daté), \
             puis bumper ce nombre."
        );
    }

    /// **Le script de release voit-il exactement les exemptions périssables ?**
    ///
    /// L'exemple `perishable_exemptions` est ce que `scripts/prepare-release.sh`
    /// exécute ; si son inventaire diverge du registre, le rappel de release
    /// devient muet — le mode d'échec que toute cette mécanique combat.
    #[test]
    fn the_release_script_sees_every_perishable_exemption() {
        let perishable: Vec<(i64, i64)> = EXEMPT_MIGRATIONS
            .iter()
            .filter_map(|(version, basis, _)| match basis {
                ExemptionBasis::PerishableSince(borne) => Some((*version, *borne)),
                ExemptionBasis::Durable => None,
            })
            .collect();

        assert_eq!(
            perishable,
            vec![
                (20260909000001, 20260827000001),
                (20260915000001, 20260827000001)
            ],
            "l'inventaire des exemptions périssables a changé. `scripts/prepare-release.sh` le \
             lit via `cargo run -p kesh-db --example perishable_exemptions` : vérifier que le \
             rappel de release dit encore vrai, puis mettre ce test à jour."
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

        // Table entièrement absente — atteignable depuis la Story 32-12a (table
        // créée après le plancher, absente d'une sauvegarde qui la précède) :
        // compte comme sentinelle absente.
        let empty = BTreeMap::new();
        assert_eq!(missing_sentinels(&empty, sentinels).len(), 2);
    }

    // =======================================================================
    // Story 32-12a (#722), AC11 — une table postérieure au plancher, amorcée
    // ou corrigée par une migration, va au registre.
    // =======================================================================

    /// Les tables de `inventaire` créées **après** `plancher` (selon `creation`)
    /// dans lesquelles `sql` écrit — premier mot-clé `INSERT`/`REPLACE`/
    /// `UPDATE`/`DELETE` ([`writes_data`]), cible lue après `INTO`, `UPDATE` ou
    /// `FROM`.
    ///
    /// La date de la cible vient de `creation`, non de la migration examinée :
    /// une migration qui écrit dans une table créée par une migration
    /// **antérieure** (la 32-6 amorçant une table de la 32-5) est visée au même
    /// titre que celle qui crée sa table et l'amorce.
    fn post_floor_write_targets(
        sql: &str,
        inventaire: &[&'static str],
        creation: &dyn Fn(&str) -> Option<i64>,
        plancher: i64,
    ) -> Vec<&'static str> {
        let mut cibles = Vec::new();
        for statement in split_statements(sql) {
            if !writes_data(&statement) {
                continue;
            }
            let up = normalize(&statement).to_ascii_uppercase().replace('`', "");
            let mots: Vec<&str> = up.split([' ', '(']).filter(|m| !m.is_empty()).collect();
            let apres = |cle: &str| -> Option<&str> {
                let i = mots.iter().position(|m| *m == cle)?;
                mots.get(i + 1).copied()
            };
            let cible = match mots.first().copied() {
                Some("INSERT") | Some("REPLACE") => apres("INTO"),
                Some("DELETE") => apres("FROM"),
                // `UPDATE [LOW_PRIORITY] [IGNORE] <table>` : le premier mot qui
                // n'est pas un modificateur.
                Some("UPDATE") => mots
                    .iter()
                    .skip(1)
                    .copied()
                    .find(|m| *m != "LOW_PRIORITY" && *m != "IGNORE"),
                _ => None,
            };
            let Some(cible) = cible else { continue };
            let Some(table) = inventaire
                .iter()
                .copied()
                .find(|t| t.eq_ignore_ascii_case(cible))
            else {
                // Table système (`_kesh_version`…) ou hors inventaire : non visée.
                continue;
            };
            if creation(table).is_some_and(|v| v > plancher) && !cibles.contains(&table) {
                cibles.push(table);
            }
        }
        cibles
    }

    /// La détection de l'AC11 nourrie de migrations **fictives** (mutation M7 de
    /// la fiche 32-12) — sans quoi le test réel, vacant tant qu'aucune table
    /// n'est postérieure au plancher, ne prouverait rien.
    #[test]
    fn post_floor_write_targets_on_fictitious_migrations() {
        const PLANCHER: i64 = crate::backup::IMPORT_FLOOR_MIGRATION;
        // Une migration fictive, postérieure au plancher.
        const T_CREEE: i64 = 99_991_231_000_001;
        static INVENTAIRE: &[&str] = &["t", "ancienne"];
        let creation = |t: &str| match t {
            "t" => Some(T_CREEE),
            "ancienne" => Some(20260415000001),
            _ => None,
        };

        // (1) La migration crée sa table et l'amorce.
        let cree_et_amorce = "-- Amorçage par société.\n\
            CREATE TABLE t (id BIGINT PRIMARY KEY, company_id BIGINT NOT NULL);\n\
            INSERT INTO t (company_id)\n  SELECT id FROM companies\n  WHERE NOT EXISTS (SELECT 1 FROM t);\n";
        assert_eq!(
            post_floor_write_targets(cree_et_amorce, INVENTAIRE, &creation, PLANCHER),
            vec!["t"],
            "une migration qui crée une table postérieure au plancher et l'amorce est à inscrire"
        );

        // (2) Une migration ULTÉRIEURE écrit dans la table créée avant elle.
        let ecrit_plus_tard = "UPDATE `t` SET company_id = company_id WHERE id = 1;\n\
            INSERT IGNORE INTO t (company_id) VALUES (1);";
        assert_eq!(
            post_floor_write_targets(ecrit_plus_tard, INVENTAIRE, &creation, PLANCHER),
            vec!["t"],
            "une migration qui écrit dans une table postérieure créée par une migration \
             antérieure est à inscrire"
        );
        assert_eq!(
            post_floor_write_targets(
                "DELETE FROM t WHERE id = 0;",
                INVENTAIRE,
                &creation,
                PLANCHER
            ),
            vec!["t"]
        );

        // (3) Ne déclenchent rien : la table système, une table au plus tard au
        // plancher (exigée, donc présente dans toute sauvegarde importable), le
        // DDL `ON UPDATE CURRENT_TIMESTAMP`.
        let rien = "UPDATE _kesh_version SET kesh_version_min_required = '0.16.0' WHERE id = 1;\n\
            UPDATE ancienne SET a = 1;\n\
            ALTER TABLE t ADD COLUMN u DATETIME ON UPDATE CURRENT_TIMESTAMP;";
        assert!(
            post_floor_write_targets(rien, INVENTAIRE, &creation, PLANCHER).is_empty(),
            "ni une table système, ni une table exigée, ni du DDL ne sont visés"
        );
    }

    /// **AC11 — une table postérieure au plancher amorcée va au registre.**
    ///
    /// Une table créée après [`crate::backup::IMPORT_FLOOR_MIGRATION`] arrive
    /// **vide** d'une sauvegarde qui la précède (`restore_body` la vide, et le
    /// fichier ne la porte pas). Si une migration y écrit des lignes — en la
    /// créant, ou plus tard —, ces lignes manqueraient après l'import : l'argument
    /// « couverture » d'une exemption serait faux, et
    /// `every_data_backfill_migration_is_triaged`, qui accepte toute
    /// justification écrite, ne le verrait pas. La migration doit donc figurer à
    /// [`POST_RESTORE_BACKFILLS`] — par un **extrait** de ses seuls statements
    /// d'écriture si elle crée aussi la table (`registry_sql_contains_no_ddl`).
    ///
    /// Vacant à la Story 32-12a : aucune table de l'inventaire n'est postérieure
    /// au plancher. Sa matière vient de
    /// `post_floor_write_targets_on_fictitious_migrations`, puis, à la 32-5, du
    /// réel.
    #[test]
    fn post_floor_table_seeds_are_replayed() {
        let floor = crate::backup::IMPORT_FLOOR_MIGRATION;
        let creation = |t: &str| crate::backup::table_creating_migrations().get(t).copied();
        let mut fautives = Vec::new();
        for m in crate::MIGRATOR.migrations.iter() {
            if m.version <= floor {
                continue;
            }
            let cibles = post_floor_write_targets(
                &m.sql,
                crate::backup::TABLES_TO_TRUNCATE,
                &creation,
                floor,
            );
            if cibles.is_empty() {
                continue;
            }
            let au_registre = POST_RESTORE_BACKFILLS
                .iter()
                .any(|e| e.version == m.version);
            let exemptee = EXEMPT_MIGRATIONS.iter().any(|(v, _, _)| *v == m.version);
            if !au_registre || exemptee {
                fautives.push(format!("{} → {cibles:?}", migration_file_name(m)));
            }
        }
        assert!(
            fautives.is_empty(),
            "migration(s) écrivant dans une table créée APRÈS le plancher d'import ({floor}) sans \
             figurer à POST_RESTORE_BACKFILLS (ou exemptée) : {fautives:?}\n\
             Une sauvegarde antérieure à la table arrive sans elle ; la table est vidée à \
             l'import, et ces lignes manqueraient. → l'inscrire au registre, en classe A gardée, \
             par un EXTRAIT de ses seuls statements d'écriture si elle crée aussi la table \
             (`registry_sql_contains_no_ddl`). Cf. fiche 32-12, *Contrat*, point 2."
        );
    }
}
