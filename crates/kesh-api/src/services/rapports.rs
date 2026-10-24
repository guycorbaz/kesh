//! Les services des rapports comptables — le pilote « difficile » du motif :
//! chaque rapport écrit une trace au journal d'audit.
//!
//! Un service de rapport vérifie l'accès, valide la demande **dans l'ordre
//! d'aujourd'hui**, génère le rapport par la [`Lecture`] et rend un
//! [`Rapport`], dont le contenu ne s'obtient que par
//! [`Rapport::afficher`] (trace de consultation) ou [`Rapport::exporter`]
//! (rendu, **puis** trace d'export) : un handler ne peut ni obtenir le contenu
//! sans écrire la trace, ni tracer un export dont le rendu a échoué.
//!
//! La trace s'écrit sur la [`Trace`], par l'une des sept fonctions d'audit de ce
//! module (best-effort : un échec est journalisé par `warn!`, jamais remonté).
//! L'aiguillage `tracer` choisit **quelle fonction** appeler, jamais **quelle
//! action** écrire : l'action reste un littéral à chaque `NewAuditLogEntry::user(`.

use chrono::{NaiveDate, Utc};
use kesh_db::entities::AUDIT_ENTITY_ID_NONE;
use kesh_db::entities::Role;
use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::journal_entry::Journal;
use kesh_report::project_report::{ProjectPeriodMode, resolve_scope};
use kesh_report::{
    AgedReceivables, BalanceSheet, GeneralLedger, IncomeStatement, JournalReport, LedgerOptions,
    LedgerPeriod, MAX_LEDGER_LIMIT, ProjectExpensesReport, ProjectReportScope, ProjectReturnReport,
    ReportPeriod, TrialBalance, VatReport, generate_aged_receivables, generate_balance_sheet,
    generate_general_ledger, generate_income_statement, generate_journal_report,
    generate_project_expenses, generate_project_return, generate_trial_balance,
    generate_vat_report,
};

use crate::errors::AppError;
use crate::services::acces::{ContexteAcces, Exigence, Portee};
use crate::services::poignees::{Lecture, Trace};

// ===========================================================================
// Usage : affichage ou export
// ===========================================================================

/// Format d'un export, validé par le handler (`validate_format`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatExport {
    /// Document PDF.
    Pdf,
    /// Fichier CSV (UTF-8).
    Csv,
}

impl FormatExport {
    /// Nom du format tel qu'écrit au journal d'audit et au `span` d'export.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Csv => "csv",
        }
    }

    /// En-tête `Content-Type` de la réponse (présentation, suit le type).
    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Csv => "text/csv; charset=utf-8",
        }
    }

    /// Extension du nom de fichier (présentation, suit le type).
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Csv => "csv",
        }
    }
}

/// Usage « consultation à l'écran » : trace `report.generated`.
#[derive(Debug, Clone, Copy)]
pub struct Affichage;

/// Usage « export » dans un format : trace `report.exported` (sauf le grand
/// livre, #729). Jamais servi au MCP.
#[derive(Debug, Clone, Copy)]
pub struct Export(pub FormatExport);

mod scelle {
    /// Scellé : seuls [`super::Affichage`] et [`super::Export`] sont des usages.
    pub trait Scelle {}
    impl Scelle for super::Affichage {}
    impl Scelle for super::Export {}
}

/// Un usage de rapport : [`Affichage`] ou [`Export`] (trait scellé).
pub trait Usage: scelle::Scelle {
    /// `None` en affichage, le format en export.
    fn format_export(&self) -> Option<FormatExport>;
}

impl Usage for Affichage {
    fn format_export(&self) -> Option<FormatExport> {
        None
    }
}

impl Usage for Export {
    fn format_export(&self) -> Option<FormatExport> {
        Some(self.0)
    }
}

// ===========================================================================
// Exigences (règle R3) — une par opération
// ===========================================================================

/// Les exigences d'un service de rapport, une par usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExigencesRapport {
    /// Exigence de l'affichage.
    pub affichage: Exigence,
    /// Exigence de l'export.
    pub export: Exigence,
}

impl ExigencesRapport {
    fn pour<U: Usage>(&self, usage: &U) -> Exigence {
        match usage.format_export() {
            None => self.affichage,
            Some(_) => self.export,
        }
    }
}

/// Affichage d'un rapport des livres : tous rôles (`authenticated_routes`) ;
/// portée MCP `BooksRead`.
const AFFICHAGE_LIVRES: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: Some(Portee::BooksRead),
};

/// Export d'un rapport : tous rôles (`authenticated_routes`) ; jamais servi au
/// MCP.
const EXPORT_SANS_MCP: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: None,
};

const LIVRES: ExigencesRapport = ExigencesRapport {
    affichage: AFFICHAGE_LIVRES,
    export: EXPORT_SANS_MCP,
};

/// Exigences du bilan.
pub const EXIGENCES_BILAN: ExigencesRapport = LIVRES;
/// Exigences du compte de résultat.
pub const EXIGENCES_COMPTE_DE_RESULTAT: ExigencesRapport = LIVRES;
/// Exigences de la balance des comptes.
pub const EXIGENCES_BALANCE: ExigencesRapport = LIVRES;
/// Exigences des journaux.
pub const EXIGENCES_JOURNAUX: ExigencesRapport = LIVRES;
/// Exigences du rapport de TVA.
pub const EXIGENCES_TVA: ExigencesRapport = LIVRES;
/// Exigences du grand livre.
pub const EXIGENCES_GRAND_LIVRE: ExigencesRapport = LIVRES;
/// Exigences des dépenses par projet.
pub const EXIGENCES_DEPENSES_PROJET: ExigencesRapport = LIVRES;
/// Exigences du rendement par projet.
pub const EXIGENCES_RENDEMENT_PROJET: ExigencesRapport = LIVRES;

/// Exigences de la balance âgée : affichage tous rôles, portée MCP
/// `InvoicesRead` ; export **Comptable** (`comptable_routes`, D24), jamais servi
/// au MCP.
pub const EXIGENCES_BALANCE_AGEE: ExigencesRapport = ExigencesRapport {
    affichage: Exigence {
        role_api: Role::Consultation,
        portee_mcp: Some(Portee::InvoicesRead),
    },
    export: Exigence {
        role_api: Role::Comptable,
        portee_mcp: None,
    },
};

/// Exigence de [`identite_societe`] : le plus bas des rôles des neuf routes
/// d'export qu'elle sert (règle R3) ; jamais servie au MCP — le contexte PDF ne
/// sert qu'aux exports.
pub const EXIGENCE_IDENTITE_SOCIETE: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: None,
};

// ===========================================================================
// Demandes
// ===========================================================================

/// Demande d'un rapport à période (bilan, compte de résultat, balance, TVA).
#[derive(Debug, Clone, Copy)]
pub struct DemandePeriode {
    /// Exercice (`fiscalYearId`), strictement positif.
    pub fiscal_year_id: i64,
    /// Début de période ; défaut : début de l'exercice.
    pub period_start: Option<NaiveDate>,
    /// Fin de période ; défaut : fin de l'exercice.
    pub period_end: Option<NaiveDate>,
}

/// Demande du rapport des journaux.
#[derive(Debug, Clone, Copy)]
pub struct DemandeJournaux {
    /// La période.
    pub periode: DemandePeriode,
    /// Filtre sur un journal ; `None` : les cinq.
    pub journal: Option<Journal>,
}

/// Demande du grand livre — sans exercice : il franchit la borne d'exercice.
#[derive(Debug, Clone)]
pub struct DemandeGrandLivre {
    /// Première date.
    pub from: NaiveDate,
    /// Dernière date.
    pub to: NaiveDate,
    /// Comptes demandés, séparés par des virgules ; absent : ceux qui ont un
    /// mouvement ou un solde d'ouverture.
    pub account_ids: Option<String>,
    /// Rendre aussi les comptes sans mouvement **et** sans solde.
    pub include_zero: bool,
    /// Décalage de pagination (affichage seulement).
    pub offset: Option<i64>,
    /// Lignes par compte (affichage seulement ; défaut et plafond
    /// `MAX_LEDGER_LIMIT`). L'export n'a jamais de borne.
    pub limit: Option<i64>,
}

/// Mode de période d'un rapport de projet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeProjet {
    /// Un exercice (`fiscal_year`) : `fiscal_year_id` requis.
    Exercice,
    /// Cumulé depuis l'origine (`cumulative`), borné par les dates du projet.
    Cumule,
}

/// Demande d'un rapport de projet.
#[derive(Debug, Clone, Copy)]
pub struct DemandeProjet {
    /// Projet ciblé, strictement positif.
    pub project_id: i64,
    /// Mode de période.
    pub mode: ModeProjet,
    /// Exercice — requis en mode [`ModeProjet::Exercice`].
    pub fiscal_year_id: Option<i64>,
    /// Début de période (mode exercice).
    pub period_start: Option<NaiveDate>,
    /// Fin de période (mode exercice).
    pub period_end: Option<NaiveDate>,
}

// ===========================================================================
// Le rapport, et sa trace
// ===========================================================================

/// Les détails d'une trace, selon la forme du rapport.
#[derive(Debug, Clone)]
enum DetailsTrace {
    /// Rapport à période (bilan, compte de résultat, balance, journaux, TVA).
    Periode {
        report_type: &'static str,
        fiscal_year_id: i64,
        period: ReportPeriod,
        journal_filter: Option<String>,
    },
    /// Grand livre.
    GrandLivre {
        period: LedgerPeriod,
        account_ids: Option<Vec<i64>>,
    },
    /// Balance âgée, arrêtée à `as_of`.
    BalanceAgee { as_of: NaiveDate },
    /// Rapport de projet.
    Projet {
        report_type: &'static str,
        project_id: i64,
        mode: ProjectPeriodMode,
    },
}

/// Ce que rend un service de rapport : le contenu `T`, et de quoi écrire sa
/// trace pour l'usage `U`.
///
/// Le contenu **ne s'obtient que** par [`Rapport::afficher`] ou
/// [`Rapport::exporter`], qui écrivent la trace (règle R5). Les accesseurs
/// exposent, sans le contenu, ce qui nomme le fichier d'un export : la période
/// résolue, les bornes du grand livre, la date de la balance âgée — calculée une
/// fois, dans le service —, le mode d'un rapport de projet. Aucune `Lecture`
/// n'y est portée.
pub struct Rapport<T, U> {
    contenu: T,
    details: DetailsTrace,
    user_id: i64,
    usage: U,
}

impl<T, U> Rapport<T, U> {
    /// Constructeur commun des neuf services.
    fn nouveau(contenu: T, details: DetailsTrace, user_id: i64, usage: U) -> Self {
        Self {
            contenu,
            details,
            user_id,
            usage,
        }
    }

    /// Période résolue (rapports à période ; projet en mode exercice : `None`,
    /// cf. [`Rapport::mode_projet`]).
    pub fn periode(&self) -> Option<&ReportPeriod> {
        match &self.details {
            DetailsTrace::Periode { period, .. } => Some(period),
            _ => None,
        }
    }

    /// Bornes du grand livre.
    pub fn bornes_grand_livre(&self) -> Option<&LedgerPeriod> {
        match &self.details {
            DetailsTrace::GrandLivre { period, .. } => Some(period),
            _ => None,
        }
    }

    /// Date d'arrêté de la balance âgée — celle du contenu et de la trace.
    pub fn as_of(&self) -> Option<NaiveDate> {
        match &self.details {
            DetailsTrace::BalanceAgee { as_of } => Some(*as_of),
            _ => None,
        }
    }

    /// Mode (et bornes) d'un rapport de projet.
    pub fn mode_projet(&self) -> Option<&ProjectPeriodMode> {
        match &self.details {
            DetailsTrace::Projet { mode, .. } => Some(mode),
            _ => None,
        }
    }
}

impl<T> Rapport<T, Affichage> {
    /// Écrit la trace de consultation sur la `Trace`, puis rend le contenu.
    pub async fn afficher(self, trace: &Trace) -> T {
        tracer(trace, self.user_id, &self.details, None).await;
        self.contenu
    }
}

impl<T> Rapport<T, Export> {
    /// Rend le contenu par `rendre`, **puis**, si le rendu a réussi, écrit la
    /// trace d'export sur la `Trace` : un rendu en échec ne laisse aucune trace.
    pub async fn exporter(
        self,
        trace: &Trace,
        rendre: impl FnOnce(&T) -> Result<Vec<u8>, AppError>,
    ) -> Result<Vec<u8>, AppError> {
        let corps = rendre(&self.contenu)?;
        tracer(trace, self.user_id, &self.details, Some(self.usage.0)).await;
        Ok(corps)
    }
}

/// L'aiguillage commun : selon la forme du rapport et l'usage, l'une des sept
/// fonctions d'audit — huit chemins, l'export du grand livre gardant
/// `emit_ledger_audit` (`report.generated`, sans format : #729, conservé).
async fn tracer(trace: &Trace, user_id: i64, details: &DetailsTrace, format: Option<FormatExport>) {
    match (details, format) {
        (
            DetailsTrace::Periode {
                report_type,
                fiscal_year_id,
                period,
                journal_filter,
            },
            None,
        ) => {
            emit_report_audit(
                trace,
                user_id,
                report_type,
                *fiscal_year_id,
                period.start_date,
                period.end_date,
                journal_filter.as_deref(),
            )
            .await
        }
        (
            DetailsTrace::Periode {
                report_type,
                fiscal_year_id,
                period,
                journal_filter,
            },
            Some(f),
        ) => {
            emit_report_export_audit(
                trace,
                user_id,
                report_type,
                f.as_str(),
                *fiscal_year_id,
                period.start_date,
                period.end_date,
                journal_filter.as_deref(),
            )
            .await
        }
        // #729 : l'export du grand livre est tracé comme une consultation.
        (
            DetailsTrace::GrandLivre {
                period,
                account_ids,
            },
            _,
        ) => {
            emit_ledger_audit(
                trace,
                user_id,
                period.from,
                period.to,
                account_ids.as_deref(),
            )
            .await
        }
        (DetailsTrace::BalanceAgee { as_of }, None) => {
            emit_aged_receivables_audit(trace, user_id, *as_of).await
        }
        (DetailsTrace::BalanceAgee { as_of }, Some(_)) => {
            emit_aged_receivables_export_audit(trace, user_id, *as_of).await
        }
        (
            DetailsTrace::Projet {
                report_type,
                project_id,
                mode,
            },
            None,
        ) => {
            emit_project_report_audit(trace, user_id, report_type, *project_id, mode.as_str()).await
        }
        (
            DetailsTrace::Projet {
                report_type,
                project_id,
                mode,
            },
            Some(f),
        ) => {
            emit_project_report_export_audit(
                trace,
                user_id,
                report_type,
                f.as_str(),
                *project_id,
                mode.as_str(),
            )
            .await
        }
    }
}

// ===========================================================================
// Les services
// ===========================================================================

/// Validation `fiscalYearId > 0` (Pass 1 ECH-06 + Pass 3 BH3-01).
///
/// Le parsing serde i64 accepte 0 et négatifs sans erreur. Ce check explicite
/// retourne 400 `VALIDATION_ERROR` en JSON (via `build_response` standard).
fn validate_fiscal_year_id(fiscal_year_id: i64) -> Result<(), AppError> {
    if fiscal_year_id <= 0 {
        return Err(AppError::Validation(format!(
            "fiscalYearId must be > 0 (got {fiscal_year_id})"
        )));
    }
    Ok(())
}

/// Découpe `accountIds`. Une entrée non numérique est un 400 : mieux vaut
/// refuser que rendre silencieusement le grand livre d'un autre périmètre.
fn parse_ledger_account_ids(raw: Option<&str>) -> Result<Option<Vec<i64>>, AppError> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let mut ids = Vec::new();
    for part in trimmed.split(',') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        let id: i64 = p.parse().map_err(|_| {
            AppError::Validation(format!("accountIds : « {p} » n'est pas un identifiant"))
        })?;
        if id <= 0 {
            return Err(AppError::Validation(format!(
                "accountIds : « {p} » n'est pas un identifiant"
            )));
        }
        ids.push(id);
    }
    Ok(if ids.is_empty() { None } else { Some(ids) })
}

/// Résout la période d'un rapport à période (après validation).
async fn resoudre_periode(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: &DemandePeriode,
) -> Result<ReportPeriod, AppError> {
    Ok(ReportPeriod::resolve(
        lecture.pool(),
        acces.company_id(),
        demande.fiscal_year_id,
        demande.period_start,
        demande.period_end,
    )
    .await?)
}

/// Détails de trace d'un rapport à période.
fn details_periode(
    report_type: &'static str,
    demande: &DemandePeriode,
    period: &ReportPeriod,
    journal_filter: Option<String>,
) -> DetailsTrace {
    DetailsTrace::Periode {
        report_type,
        fiscal_year_id: demande.fiscal_year_id,
        period: period.clone(),
        journal_filter,
    }
}

/// Le bilan.
pub async fn bilan<U: Usage>(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: DemandePeriode,
    usage: U,
) -> Result<Rapport<BalanceSheet, U>, AppError> {
    acces.exiger(EXIGENCES_BILAN.pour(&usage))?;
    validate_fiscal_year_id(demande.fiscal_year_id)?;
    let period = resoudre_periode(acces, lecture, &demande).await?;
    let contenu = generate_balance_sheet(lecture.pool(), acces.company_id(), &period).await?;
    let details = details_periode("balance-sheet", &demande, &period, None);
    Ok(Rapport::nouveau(contenu, details, acces.user_id(), usage))
}

/// Le compte de résultat.
pub async fn compte_de_resultat<U: Usage>(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: DemandePeriode,
    usage: U,
) -> Result<Rapport<IncomeStatement, U>, AppError> {
    acces.exiger(EXIGENCES_COMPTE_DE_RESULTAT.pour(&usage))?;
    validate_fiscal_year_id(demande.fiscal_year_id)?;
    let period = resoudre_periode(acces, lecture, &demande).await?;
    let contenu = generate_income_statement(lecture.pool(), acces.company_id(), &period).await?;
    let details = details_periode("income-statement", &demande, &period, None);
    Ok(Rapport::nouveau(contenu, details, acces.user_id(), usage))
}

/// La balance des comptes.
pub async fn balance<U: Usage>(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: DemandePeriode,
    usage: U,
) -> Result<Rapport<TrialBalance, U>, AppError> {
    acces.exiger(EXIGENCES_BALANCE.pour(&usage))?;
    validate_fiscal_year_id(demande.fiscal_year_id)?;
    let period = resoudre_periode(acces, lecture, &demande).await?;
    let contenu = generate_trial_balance(lecture.pool(), acces.company_id(), &period).await?;
    let details = details_periode("trial-balance", &demande, &period, None);
    Ok(Rapport::nouveau(contenu, details, acces.user_id(), usage))
}

/// Les journaux (cinq sections, ou un seul journal filtré).
pub async fn journaux<U: Usage>(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: DemandeJournaux,
    usage: U,
) -> Result<Rapport<JournalReport, U>, AppError> {
    acces.exiger(EXIGENCES_JOURNAUX.pour(&usage))?;
    validate_fiscal_year_id(demande.periode.fiscal_year_id)?;
    let period = resoudre_periode(acces, lecture, &demande.periode).await?;
    let contenu =
        generate_journal_report(lecture.pool(), acces.company_id(), &period, demande.journal)
            .await?;
    let journal_filter = demande.journal.map(|j| j.as_str().to_string());
    let details = details_periode("journals", &demande.periode, &period, journal_filter);
    Ok(Rapport::nouveau(contenu, details, acces.user_id(), usage))
}

/// Le rapport de TVA.
pub async fn tva<U: Usage>(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: DemandePeriode,
    usage: U,
) -> Result<Rapport<VatReport, U>, AppError> {
    acces.exiger(EXIGENCES_TVA.pour(&usage))?;
    validate_fiscal_year_id(demande.fiscal_year_id)?;
    let period = resoudre_periode(acces, lecture, &demande).await?;
    let contenu = generate_vat_report(lecture.pool(), acces.company_id(), &period).await?;
    let details = details_periode("vat", &demande, &period, None);
    Ok(Rapport::nouveau(contenu, details, acces.user_id(), usage))
}

/// Le grand livre — deux régimes : l'affichage lit `offset` et borne `limit`
/// (défaut et plafond `MAX_LEDGER_LIMIT`) ; l'export rend le livre **entier**
/// (`offset` 0, aucune borne) : un export tronqué n'aurait aucune valeur
/// probante.
pub async fn grand_livre<U: Usage>(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: DemandeGrandLivre,
    usage: U,
) -> Result<Rapport<GeneralLedger, U>, AppError> {
    acces.exiger(EXIGENCES_GRAND_LIVRE.pour(&usage))?;
    let period = LedgerPeriod::new(demande.from, demande.to)?;
    let account_ids = parse_ledger_account_ids(demande.account_ids.as_deref())?;
    let (offset, limit) = match usage.format_export() {
        // Écran : le défaut est le plafond.
        None => (
            demande.offset.unwrap_or(0),
            Some(demande.limit.unwrap_or(MAX_LEDGER_LIMIT)),
        ),
        // Export : `None` = aucune borne.
        Some(_) => (0, None),
    };
    let options = LedgerOptions {
        account_ids,
        include_zero: demande.include_zero,
        offset,
        limit,
    };
    let contenu =
        generate_general_ledger(lecture.pool(), acces.company_id(), &period, &options).await?;
    let details = DetailsTrace::GrandLivre {
        period,
        account_ids: options.account_ids,
    };
    Ok(Rapport::nouveau(contenu, details, acces.user_id(), usage))
}

/// La balance âgée des débiteurs, arrêtée à aujourd'hui (UTC) — date calculée
/// **une fois**, ici : le contenu, la trace et le nom de fichier portent la même.
pub async fn balance_agee<U: Usage>(
    acces: &ContexteAcces,
    lecture: &Lecture,
    usage: U,
) -> Result<Rapport<AgedReceivables, U>, AppError> {
    acces.exiger(EXIGENCES_BALANCE_AGEE.pour(&usage))?;
    let as_of = Utc::now().naive_utc().date();
    let contenu = generate_aged_receivables(lecture.pool(), acces.company_id(), as_of).await?;
    let details = DetailsTrace::BalanceAgee { as_of };
    Ok(Rapport::nouveau(contenu, details, acces.user_id(), usage))
}

/// Résout `(scope, ProjectPeriodMode)` d'un rapport de projet. Valide
/// `project_id > 0` et, en mode exercice, la présence de `fiscalYearId` — dans
/// l'ordre d'aujourd'hui (le projet est lu avant la validation de l'exercice).
async fn resoudre_projet(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: &DemandeProjet,
) -> Result<(ProjectReportScope, ProjectPeriodMode), AppError> {
    let company_id = acces.company_id();
    if demande.project_id <= 0 {
        return Err(AppError::Validation(
            "projectId doit être strictement positif".into(),
        ));
    }
    let scope = resolve_scope(lecture.pool(), company_id, demande.project_id).await?;

    let mode = match demande.mode {
        ModeProjet::Exercice => {
            let fy_id = demande.fiscal_year_id.ok_or_else(|| {
                AppError::Validation("fiscalYearId requis en mode fiscal_year".into())
            })?;
            validate_fiscal_year_id(fy_id)?;
            let period = ReportPeriod::resolve(
                lecture.pool(),
                company_id,
                fy_id,
                demande.period_start,
                demande.period_end,
            )
            .await?;
            ProjectPeriodMode::FiscalYear { period }
        }
        ModeProjet::Cumule => {
            // Borne haute = date de fin du projet si définie, sinon aujourd'hui
            // (résolu ici — pas de Date::now() en lib).
            //
            // Note (Pass 3 Opus, DC4) : les bornes proviennent du projet **ciblé**
            // (racine du scope). Pour un rapport racine agrégeant des sous-projets,
            // les écritures d'un sous-projet hors de la fenêtre de dates de la racine
            // sont donc exclues — comportement voulu (« borné par les dates du
            // projet »). Une racine sans start_date (cas courant) ⇒ pas de borne
            // basse (`entry_date <= end`), donc tous les enfants sont inclus.
            let today = chrono::Utc::now().date_naive();
            let end = scope.root.end_date.unwrap_or(today);
            ProjectPeriodMode::Cumulative {
                start: scope.root.start_date,
                end,
            }
        }
    };
    Ok((scope, mode))
}

/// Les dépenses par projet.
pub async fn depenses_projet<U: Usage>(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: DemandeProjet,
    usage: U,
) -> Result<Rapport<ProjectExpensesReport, U>, AppError> {
    acces.exiger(EXIGENCES_DEPENSES_PROJET.pour(&usage))?;
    let (scope, mode) = resoudre_projet(acces, lecture, &demande).await?;
    let contenu =
        generate_project_expenses(lecture.pool(), acces.company_id(), &scope, &mode).await?;
    let details = DetailsTrace::Projet {
        report_type: "project-expenses",
        project_id: demande.project_id,
        mode,
    };
    Ok(Rapport::nouveau(contenu, details, acces.user_id(), usage))
}

/// Le rendement par projet.
pub async fn rendement_projet<U: Usage>(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: DemandeProjet,
    usage: U,
) -> Result<Rapport<ProjectReturnReport, U>, AppError> {
    acces.exiger(EXIGENCES_RENDEMENT_PROJET.pour(&usage))?;
    let (scope, mode) = resoudre_projet(acces, lecture, &demande).await?;
    let contenu =
        generate_project_return(lecture.pool(), acces.company_id(), &scope, &mode).await?;
    let details = DetailsTrace::Projet {
        report_type: "project-return",
        project_id: demande.project_id,
        mode,
    };
    Ok(Rapport::nouveau(contenu, details, acces.user_id(), usage))
}

/// Nom et `accounting_language` de la société du contexte — ce que le handler
/// met en forme en contexte PDF et en nom de fichier d'un export.
///
/// **Scoping multi-tenant** (Pass 1 code-review H5) : `company_id` est celui du
/// contexte, établi par le middleware d'authentification depuis le claim JWT
/// signé ; il n'y a donc pas d'IDOR ici tant que le JWT guard est sain
/// (Limitations L13).
pub async fn identite_societe(
    acces: &ContexteAcces,
    lecture: &Lecture,
) -> Result<(String, String), AppError> {
    acces.exiger(EXIGENCE_IDENTITE_SOCIETE)?;
    let row: (String, String) =
        sqlx::query_as("SELECT name, accounting_language FROM companies WHERE id = ?")
            .bind(acces.company_id())
            .fetch_one(lecture.pool())
            .await
            .map_err(kesh_db::errors::map_db_error)?;
    Ok(row)
}

// ===========================================================================
// La trace — sept fonctions d'audit, sur la `Trace`
// ===========================================================================

/// Story 21-7 — audit best-effort génération balance âgée (as_of, pas de période).
/// Bespoke (ne réutilise pas `emit_report_audit`, signature période).
async fn emit_aged_receivables_audit(trace: &Trace, user_id: i64, as_of: NaiveDate) {
    let result = async {
        let mut tx = trace
            .pool()
            .begin()
            .await
            .map_err(kesh_db::errors::map_db_error)?;
        kesh_db::repositories::audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(
                user_id,
                "report.generated",
                "report",
                AUDIT_ENTITY_ID_NONE,
                Some(serde_json::json!({
                    "report_type": "aged-receivables",
                    "as_of": as_of.format("%Y-%m-%d").to_string(),
                })),
            ),
        )
        .await?;
        tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
        Ok::<(), kesh_db::errors::DbError>(())
    }
    .await;
    if let Err(e) = result {
        tracing::warn!(error = ?e, user_id, "audit insert failed (report.generated aged) — non-blocking");
    }
}

/// Story 21-7 — audit best-effort export balance âgée (CSV).
async fn emit_aged_receivables_export_audit(trace: &Trace, user_id: i64, as_of: NaiveDate) {
    let result = async {
        let mut tx = trace
            .pool()
            .begin()
            .await
            .map_err(kesh_db::errors::map_db_error)?;
        kesh_db::repositories::audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(
                user_id,
                "report.exported",
                "report",
                AUDIT_ENTITY_ID_NONE,
                Some(serde_json::json!({
                    "report_type": "aged-receivables",
                    "format": "csv",
                    "as_of": as_of.format("%Y-%m-%d").to_string(),
                })),
            ),
        )
        .await?;
        tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
        Ok::<(), kesh_db::errors::DbError>(())
    }
    .await;
    if let Err(e) = result {
        tracing::warn!(error = ?e, user_id, "audit insert failed (report.exported aged) — non-blocking");
    }
}

/// Trace la consultation du grand livre.
///
/// Audit distinct de [`emit_report_audit`] : celui-ci exige un `fiscal_year_id`,
/// que le grand livre n'a pas — il franchit les exercices.
async fn emit_ledger_audit(
    trace: &Trace,
    user_id: i64,
    from: NaiveDate,
    to: NaiveDate,
    account_ids: Option<&[i64]>,
) {
    let result = async {
        let mut tx = trace
            .pool()
            .begin()
            .await
            .map_err(kesh_db::errors::map_db_error)?;
        kesh_db::repositories::audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(
                user_id,
                "report.generated",
                "report",
                AUDIT_ENTITY_ID_NONE,
                Some(serde_json::json!({
                    "report_type": "general-ledger",
                    "period_start": from.format("%Y-%m-%d").to_string(),
                    "period_end": to.format("%Y-%m-%d").to_string(),
                    "account_ids": account_ids,
                })),
            ),
        )
        .await?;
        tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
        Ok::<(), kesh_db::errors::DbError>(())
    }
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "audit report.generated (general-ledger) non enregistré");
    }
}

/// Audit log `report.generated` — best-effort (Pass 1 ECH-15).
///
/// Pattern strict : log `warn!` sur erreur, ne JAMAIS faire échouer la réponse rapport.
///
/// ## Convention projet — `audit_log.details_json` JSON keys
///
/// **Snake_case obligatoire** pour les clés JSON de `details_json` (cohérent SQL
/// JSON path `details_json->>'$.field_name'` future-proof, AC #23 Story 9-2b
/// explicite). Les autres surfaces API restent **camelCase** par convention
/// REST/JS :
///
/// | Surface | Convention | Exemple |
/// |---|---|---|
/// | Query params HTTP URL | camelCase | `?fiscalYearId=...&periodStart=...` |
/// | Request/Response body HTTP | camelCase (via `#[serde(rename_all = "camelCase")]`) | `{"reportType": "..."}` |
/// | `AppError.details` HTTP error body | camelCase | `{"details": {"fiscalYearId": ...}}` (cf. `errors.rs`) |
/// | Frontend `metadata.json` | camelCase | `{"keshVersion", "exportDate"}` |
/// | **`audit_log.details_json` (serveur)** | **snake_case** | `{"report_type", "fiscal_year_id"}` |
///
/// **Référence canonique** : `emit_global_export_audit` (Story 9-2b
/// `crates/kesh-api/src/routes/exports.rs`) — premier `emit_*_audit` snake_case.
/// Migration camelCase → snake_case appliquée 2026-05-18 Story 9-5-2 pour
/// `emit_report_audit` (Story 9-1) + `emit_report_export_audit` (Story 9-2a).
/// Référence pour futures fonctions audit Epic 10+ (`vat.calculated`,
/// `payment.created`, etc.).
async fn emit_report_audit(
    trace: &Trace,
    user_id: i64,
    report_type: &str,
    fiscal_year_id: i64,
    period_start: NaiveDate,
    period_end: NaiveDate,
    journal_filter: Option<&str>,
) {
    let result = async {
        let mut tx = trace
            .pool()
            .begin()
            .await
            .map_err(kesh_db::errors::map_db_error)?;
        kesh_db::repositories::audit_log::insert_in_tx(
            &mut tx,
            // Story 9-5-2 (Epic 9.5) — clés snake_case pour cohérence SQL JSON
            // path `details_json->>'$.field_name'` future-proof.
            // Story 17-2a (DC5) — `::user` : audit READ-path (report.generated).
            // Limitation L2 documentée : non threadé actor_api_key_id (4 callers,
            // chemin lecture non-mutation hors scope strict AC8(b)) → `actor_type='user'`
            // même via PAT. Couverture exhaustive = v0.3.
            NewAuditLogEntry::user(
                user_id,
                "report.generated",
                "report",
                AUDIT_ENTITY_ID_NONE,
                Some(serde_json::json!({
                    "report_type": report_type,
                    "fiscal_year_id": fiscal_year_id,
                    "period_start": period_start.format("%Y-%m-%d").to_string(),
                    "period_end": period_end.format("%Y-%m-%d").to_string(),
                    "journal_filter": journal_filter,
                })),
            ),
        )
        .await?;
        tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
        Ok::<(), kesh_db::errors::DbError>(())
    }
    .await;

    if let Err(e) = result {
        tracing::warn!(
            error = ?e,
            user_id,
            report_type,
            fiscal_year_id,
            "audit insert failed (report.generated) — non-blocking"
        );
    }
}

/// Story 9-2a — audit log `report.exported` (best-effort, distinct de
/// `report.generated` pour faciliter les requêtes audit — Pass 1 BH-M2).
///
/// Fonction **séparée** de `emit_report_audit` — la modifier briserait les
/// 4 callers Story 9-1.
#[allow(clippy::too_many_arguments)]
async fn emit_report_export_audit(
    trace: &Trace,
    user_id: i64,
    report_type: &str,
    format: &str,
    fiscal_year_id: i64,
    period_start: NaiveDate,
    period_end: NaiveDate,
    journal_filter: Option<&str>,
) {
    let result = async {
        let mut tx = trace
            .pool()
            .begin()
            .await
            .map_err(kesh_db::errors::map_db_error)?;
        kesh_db::repositories::audit_log::insert_in_tx(
            &mut tx,
            // Story 9-5-2 (Epic 9.5) — clés snake_case.
            // Story 17-2a (DC5) — `::user` : audit READ-path (report.exported),
            // limitation L2 (cf. emit_report_audit) → `actor_type='user'` même via PAT.
            NewAuditLogEntry::user(
                user_id,
                "report.exported",
                "report",
                AUDIT_ENTITY_ID_NONE,
                Some(serde_json::json!({
                    "report_type": report_type,
                    "format": format,
                    "fiscal_year_id": fiscal_year_id,
                    "period_start": period_start.format("%Y-%m-%d").to_string(),
                    "period_end": period_end.format("%Y-%m-%d").to_string(),
                    "journal_filter": journal_filter,
                })),
            ),
        )
        .await?;
        tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
        Ok::<(), kesh_db::errors::DbError>(())
    }
    .await;

    if let Err(e) = result {
        tracing::warn!(
            error = ?e,
            user_id,
            report_type,
            format,
            fiscal_year_id,
            "audit insert failed (report.exported) — non-blocking"
        );
    }
}

/// Story 19-6a — audit best-effort génération d'un rapport projet.
async fn emit_project_report_audit(
    trace: &Trace,
    user_id: i64,
    report_type: &str,
    project_id: i64,
    mode: &str,
) {
    let result = async {
        let mut tx = trace
            .pool()
            .begin()
            .await
            .map_err(kesh_db::errors::map_db_error)?;
        kesh_db::repositories::audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(
                user_id,
                "report.generated",
                "report",
                AUDIT_ENTITY_ID_NONE,
                Some(serde_json::json!({
                    "report_type": report_type,
                    "project_id": project_id,
                    "mode": mode,
                })),
            ),
        )
        .await?;
        tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
        Ok::<(), kesh_db::errors::DbError>(())
    }
    .await;
    if let Err(e) = result {
        tracing::warn!(error = ?e, user_id, report_type, project_id, "audit insert failed (report.generated project) — non-blocking");
    }
}

/// Story 19-6a — audit best-effort export d'un rapport projet.
async fn emit_project_report_export_audit(
    trace: &Trace,
    user_id: i64,
    report_type: &str,
    format: &str,
    project_id: i64,
    mode: &str,
) {
    let result = async {
        let mut tx = trace
            .pool()
            .begin()
            .await
            .map_err(kesh_db::errors::map_db_error)?;
        kesh_db::repositories::audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(
                user_id,
                "report.exported",
                "report",
                AUDIT_ENTITY_ID_NONE,
                Some(serde_json::json!({
                    "report_type": report_type,
                    "format": format,
                    "project_id": project_id,
                    "mode": mode,
                })),
            ),
        )
        .await?;
        tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
        Ok::<(), kesh_db::errors::DbError>(())
    }
    .await;
    if let Err(e) = result {
        tracing::warn!(error = ?e, user_id, report_type, format, project_id, "audit insert failed (report.exported project) — non-blocking");
    }
}
