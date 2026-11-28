//! Entité `Account` : compte du plan comptable d'une company.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

/// Type de compte comptable.
///
/// Stocké en DB en PascalCase : `"Asset"`, `"Liability"`, `"Revenue"`, `"Expense"`.
/// CHECK BINARY en DB pour éviter les problèmes de collation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    Asset,
    Liability,
    Revenue,
    Expense,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asset => "Asset",
            Self::Liability => "Liability",
            Self::Revenue => "Revenue",
            Self::Expense => "Expense",
        }
    }
}

impl std::str::FromStr for AccountType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Asset" => Ok(Self::Asset),
            "Liability" => Ok(Self::Liability),
            "Revenue" => Ok(Self::Revenue),
            "Expense" => Ok(Self::Expense),
            other => Err(format!("AccountType inconnu : {other}")),
        }
    }
}

impl Type<MySql> for AccountType {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for AccountType {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for AccountType {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// Rôle métier explicite d'un compte (Story 14-3a).
///
/// Stocké en DB en PascalCase dans `accounts.role` (VARCHAR(32), NULL = aucun
/// rôle), avec un CHECK BINARY à liste fermée — même convention que
/// [`AccountType`].
///
/// Le rôle dit **à quoi sert** un compte indépendamment de son numéro : le plan
/// comptable suisse est un usage, pas une obligation légale. Aucune logique
/// applicative ne doit déduire un rôle d'un numéro.
///
/// # Duplication assumée avec `kesh-core`
///
/// Cet enum existe **en double** : ici (avec les impls `sqlx`) et dans
/// `kesh_core::chart_of_accounts::AccountRole` (`Deserialize` seul, pour les
/// plans JSON). L'orphan rule Rust l'impose — `sqlx` n'est une dépendance que de
/// `kesh-db`, donc `impl Type<MySql> for kesh_core::…::AccountRole` est
/// `error[E0117]`, et déplacer l'enum ici créerait un cycle Cargo. `AccountType`
/// est dupliqué pour exactement la même raison. Le garde-fou est le test de
/// cohérence `singleton_list_matches_sql_generation_expression`
/// (`repositories::accounts`), qui compare les deux enums entre eux **et** au
/// schéma réellement en base — pas la fusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccountRole {
    /// Créances clients (débiteurs).
    Receivable,
    /// Produit par défaut de facturation.
    DefaultRevenue,
    /// Dettes fournisseurs (créanciers).
    Payable,
    /// Impôt préalable (TVA récupérable).
    VatRecoverable,
    /// TVA due.
    VatPayable,
    /// Décompte TVA.
    VatSettlement,
    /// Capital (social / de l'exploitant / de l'association).
    EquityCapital,
    /// Autres fonds propres : réserves, fonds affectés ou libres, prélèvements
    /// et apports privés. Intitulé volontairement neutre (cf. `kesh-core`).
    EquityOther,
    /// Bénéfice / perte reporté.
    RetainedEarnings,
    /// Résultat de l'exercice.
    CurrentYearResult,
}

impl AccountRole {
    /// Les 10 rôles, dans l'ordre de déclaration.
    pub const ALL: [AccountRole; 10] = [
        Self::Receivable,
        Self::DefaultRevenue,
        Self::Payable,
        Self::VatRecoverable,
        Self::VatPayable,
        Self::VatSettlement,
        Self::EquityCapital,
        Self::EquityOther,
        Self::RetainedEarnings,
        Self::CurrentYearResult,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Receivable => "Receivable",
            Self::DefaultRevenue => "DefaultRevenue",
            Self::Payable => "Payable",
            Self::VatRecoverable => "VatRecoverable",
            Self::VatPayable => "VatPayable",
            Self::VatSettlement => "VatSettlement",
            Self::EquityCapital => "EquityCapital",
            Self::EquityOther => "EquityOther",
            Self::RetainedEarnings => "RetainedEarnings",
            Self::CurrentYearResult => "CurrentYearResult",
        }
    }

    /// `true` si au plus **un compte actif** par société peut porter ce rôle.
    ///
    /// # ⚠️ Liste synchronisée à TROIS endroits
    ///
    /// 1. le `CASE WHEN active AND role IN (…)` de la colonne générée
    ///    `accounts.singleton_role` (migration `20260722000001_accounts_role_postable.sql`) ;
    /// 2. **ici** ;
    /// 3. `kesh_core::chart_of_accounts::AccountRole::is_singleton()` (nécessaire
    ///    car `validate_chart` est privé à `kesh-core`).
    ///
    /// Le test `singleton_list_matches_sql_generation_expression` compare cette
    /// liste à l'expression SQL réellement en base — c'est le seul garde-fou qui
    /// ferme les trois sources.
    pub fn is_singleton(&self) -> bool {
        match self {
            Self::Receivable
            | Self::DefaultRevenue
            | Self::Payable
            | Self::VatRecoverable
            | Self::VatPayable
            | Self::VatSettlement
            | Self::RetainedEarnings
            | Self::CurrentYearResult => true,
            Self::EquityCapital | Self::EquityOther => false,
        }
    }

    /// Les rôles singleton, triés — utilisé par le test de cohérence SQL.
    pub fn singletons() -> Vec<&'static str> {
        let mut v: Vec<&'static str> = Self::ALL
            .iter()
            .filter(|r| r.is_singleton())
            .map(|r| r.as_str())
            .collect();
        v.sort_unstable();
        v
    }
}

impl std::str::FromStr for AccountRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .find(|r| r.as_str() == s)
            .copied()
            .ok_or_else(|| format!("AccountRole inconnu : {s}"))
    }
}

impl Type<MySql> for AccountRole {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for AccountRole {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for AccountRole {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// Compte comptable persisté en base.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: i64,
    pub company_id: i64,
    pub number: String,
    pub name: String,
    pub account_type: AccountType,
    pub parent_id: Option<i64>,
    pub active: bool,
    /// Rôle métier explicite, `None` si le compte n'en porte aucun (Story 14-3a).
    pub role: Option<AccountRole>,
    /// `false` pour un compte titre / de regroupement ou le compte de résultat.
    ///
    /// **Story 14-3a : cet attribut n'est lu par aucun code métier.** La garde à
    /// la saisie d'écriture est posée par la Story 14-3b.
    ///
    /// L'invariant « compte avec sous-comptes actifs ⇒ non postable » est
    /// normalisé à l'écriture par le repository (`create` et `update`), pas
    /// seulement au seed — cf. `effective_postable`.
    pub postable: bool,
    /// Devise du compte (Story 37-1), code ISO 4217 ; `None` = devise
    /// principale (CHF). Décodée en `String` **sans validation** : la liste
    /// ISO 4217 ne contrôle qu'à l'écriture, de sorte qu'un code retiré plus
    /// tard de la liste ne fasse jamais échouer la lecture d'un compte.
    pub currency: Option<String>,
    /// Compte de gains de change **propre** à ce compte en devise (Story 37-1,
    /// AC-4 bis) ; `None` = suivre le réglage de la société. Pointeur logique
    /// sans clé étrangère. Posé avec [`Self::fx_loss_account_id`], ou aucun.
    pub fx_gain_account_id: Option<i64>,
    /// Compte de pertes de change propre (Story 37-1), même régime que
    /// [`Self::fx_gain_account_id`].
    pub fx_loss_account_id: Option<i64>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Paire de comptes de différence de change **propres** à un compte en devise
/// (Story 37-1, AC-4 bis) : les deux ensemble, jamais un seul. Ils peuvent
/// désigner le même compte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnFxAccounts {
    /// Compte des gains de change.
    pub gain: i64,
    /// Compte des pertes de change.
    pub loss: i64,
}

/// Changement de la devise d'un compte à la mise à jour (Story 37-1, AC-4) —
/// trois états, résolus **dans le dépôt**, sous le `FOR UPDATE` du compte.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CurrencyChange {
    /// Champ absent de la requête : la devise ne change pas.
    #[default]
    Unchanged,
    /// La devise principale (CHF) : `accounts.currency` passe à `NULL`.
    Base,
    /// Une devise étrangère, code ISO 4217 déjà normalisé (jamais `CHF`).
    Foreign(String),
}

impl CurrencyChange {
    /// La devise du compte après le changement, depuis la devise actuelle.
    pub fn resolve(&self, current: Option<&str>) -> Option<String> {
        match self {
            Self::Unchanged => current.map(str::to_string),
            Self::Base => None,
            Self::Foreign(code) => Some(code.clone()),
        }
    }
}

/// Changement des comptes de différence de change propres d'un compte à la
/// mise à jour (Story 37-1, AC-4 bis) — trois états, résolus dans le dépôt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OwnFxAccountsChange {
    /// Champs absents de la requête : inchangés.
    #[default]
    Unchanged,
    /// Par défaut : suivre le réglage de la société (les deux colonnes à `NULL`).
    Company,
    /// Comptes propres.
    Own(OwnFxAccounts),
}

impl OwnFxAccountsChange {
    /// Les comptes propres après le changement, depuis l'état actuel.
    pub fn resolve(&self, current: Option<OwnFxAccounts>) -> Option<OwnFxAccounts> {
        match self {
            Self::Unchanged => current,
            Self::Company => None,
            Self::Own(own) => Some(*own),
        }
    }
}

/// Changement du compte parent d'un compte à la mise à jour (correctif
/// `fix-api-parent-compte`) — trois états, comme [`CurrencyChange`], résolus
/// **dans le dépôt**, sous le `FOR UPDATE` du compte.
///
/// Côté API, `parentId` **absent** = inchangé (un client qui ne connaît pas le
/// champ ne détache rien), `null` = racine, un identifiant = le nouveau parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ParentChange {
    /// Champ absent de la requête : le parent ne change pas.
    #[default]
    Unchanged,
    /// Le compte devient racine (`parent_id` à `NULL`).
    Root,
    /// Le compte se range sous ce compte.
    Under(i64),
}

impl ParentChange {
    /// Le parent du compte après le changement, depuis le parent actuel.
    pub fn resolve(&self, current: Option<i64>) -> Option<i64> {
        match self {
            Self::Unchanged => current,
            Self::Root => None,
            Self::Under(id) => Some(*id),
        }
    }
}

/// D'où viennent les comptes de différence de change résolus pour un compte en
/// devise (Story 37-1, AC-4 bis).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FxAccountsOrigin {
    /// Les comptes propres du compte (`accounts.fx_gain_account_id` et
    /// `fx_loss_account_id`).
    Own,
    /// Le réglage de la société (`companies.fx_gain_account_id` et
    /// `fx_loss_account_id`), défaut des comptes sans comptes propres.
    Company,
}

/// Comptes de différence de change **résolus** pour un compte en devise
/// (Story 37-1, AC-4 bis) — rendus par
/// [`crate::repositories::accounts::fx_accounts_for`], seule fonction qui les
/// choisit côté serveur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FxAccounts {
    /// Compte des gains de change.
    pub gain: i64,
    /// Compte des pertes de change.
    pub loss: i64,
    /// Comptes propres du compte, ou réglage de la société.
    pub origin: FxAccountsOrigin,
    /// Le compte en devise pour lequel la paire a été résolue — rendu par
    /// `FX_ACCOUNT_INVALID` en `holderAccountId` quand `origin = Own`.
    pub holder_account_id: i64,
}

impl Account {
    /// Les comptes de différence propres du compte, s'il en porte (les deux
    /// colonnes posées ; une seule ne se rencontre pas, `chk_accounts_fx_own_pair`).
    pub fn own_fx_accounts(&self) -> Option<OwnFxAccounts> {
        match (self.fx_gain_account_id, self.fx_loss_account_id) {
            (Some(gain), Some(loss)) => Some(OwnFxAccounts { gain, loss }),
            _ => None,
        }
    }
}

/// Données de création d'un compte.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAccount {
    pub company_id: i64,
    pub number: String,
    pub name: String,
    pub account_type: AccountType,
    pub parent_id: Option<i64>,
    /// Rôle métier explicite (Story 14-3a), `None` par défaut.
    pub role: Option<AccountRole>,
    /// Postabilité (Story 14-3a), `true` par défaut.
    pub postable: bool,
    /// Devise du compte (Story 37-1), code ISO 4217 déjà normalisé (jamais
    /// `CHF`) ; `None` = devise principale.
    pub currency: Option<String>,
    /// Comptes de différence de change propres (Story 37-1, AC-4 bis) ;
    /// `None` = suivre le réglage de la société.
    pub own_fx_accounts: Option<OwnFxAccounts>,
}

impl NewAccount {
    /// Compte sans rôle et postable — le cas courant.
    ///
    /// Évite d'avoir à répéter `role: None, postable: true` sur chaque littéral
    /// et absorbe le churn des prochains champs ajoutés à la struct.
    pub fn new(
        company_id: i64,
        number: impl Into<String>,
        name: impl Into<String>,
        account_type: AccountType,
        parent_id: Option<i64>,
    ) -> Self {
        Self {
            company_id,
            number: number.into(),
            name: name.into(),
            account_type,
            parent_id,
            role: None,
            postable: true,
            currency: None,
            own_fx_accounts: None,
        }
    }

    /// Variante avec rôle et postabilité explicites (seed depuis un plan comptable).
    pub fn with_role(mut self, role: Option<AccountRole>, postable: bool) -> Self {
        self.role = role;
        self.postable = postable;
        self
    }
}

/// Données de modification d'un compte.
/// Le numéro n'est PAS modifiable après création ; le parent l'est depuis le
/// correctif `fix-api-parent-compte` ([`ParentChange`]).
///
/// Sémantique **full-replace** : tous les champs sont obligatoires côté API, y
/// compris `role` et `postable`. Un `Option` laxiste aurait permis à un client
/// qui corrige un libellé d'effacer silencieusement le rôle du compte — une
/// donnée perdue en silence est pire qu'un 400 explicite.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpdate {
    pub name: String,
    pub account_type: AccountType,
    pub role: Option<AccountRole>,
    pub postable: bool,
    /// Devise (Story 37-1, AC-4) : **à la différence des champs ci-dessus**,
    /// à trois états — absente côté API = inchangée (un client qui ne connaît
    /// pas le champ n'efface rien).
    pub currency: CurrencyChange,
    /// Comptes de différence propres (Story 37-1, AC-4 bis), trois états.
    pub own_fx_accounts: OwnFxAccountsChange,
    /// Compte parent (correctif `fix-api-parent-compte`), trois états —
    /// absent côté API = inchangé. Mêmes contrôles qu'à la création, plus le
    /// refus d'un cycle (cf. `repositories::accounts::update`).
    pub parent: ParentChange,
}

/// Nombre maximal de propositions d'une demande de la route groupée du plan
/// (Story 46-5a) ; au-delà, `ACCOUNT_GROUP_INVALID` `tooMany`.
pub const MAX_GROUP_PROPOSALS: usize = 100;

/// Nombre maximal de membres, toutes propositions réunies, d'une demande de la
/// route groupée du plan (Story 46-5a) ; au-delà, `tooMany`.
pub const MAX_GROUP_MEMBERS: usize = 1000;

/// Ce que devient le compte de groupe d'une proposition (Story 46-5a).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupKind {
    /// Un groupe à créer : numéro et nom déjà contrôlés par l'appelant (non
    /// vides après `trim`, longueurs de `POST /accounts`), parent facultatif.
    /// Type déduit des membres ; devise principale, aucun rôle, non imputable.
    Create {
        number: String,
        name: String,
        parent_id: Option<i64>,
    },
    /// Un compte de groupe existant (actif, au moins un sous-compte actif).
    Attach { account_id: i64 },
    /// Un compte actif sans sous-compte actif, qui devient groupe.
    Reuse { account_id: i64 },
}

impl GroupKind {
    /// Le compte existant visé par un rattachement ou une réutilisation.
    pub fn target(&self) -> Option<i64> {
        match self {
            Self::Create { .. } => None,
            Self::Attach { account_id } | Self::Reuse { account_id } => Some(*account_id),
        }
    }
}

/// Un compte à ranger sous le groupe d'une proposition, avec la `version` lue
/// à l'aperçu (comparée sous verrou : `OPTIMISTIC_LOCK_CONFLICT` s'il a changé).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GroupMember {
    pub id: i64,
    pub version: i32,
}

/// Une proposition de la route groupée du plan : un groupe et ses membres.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupProposal {
    pub kind: GroupKind,
    pub members: Vec<GroupMember>,
}

/// Ce qu'a fait une demande groupée acceptée : les groupes créés, les comptes
/// réutilisés comme groupe (devenus non imputables) et les comptes réellement
/// déplacés (un membre déjà sous son groupe n'y figure pas), chacun par `id`
/// croissant.
#[derive(Debug, Clone, Default)]
pub struct GroupsOutcome {
    pub created: Vec<Account>,
    pub reused: Vec<Account>,
    pub moved: Vec<Account>,
}
