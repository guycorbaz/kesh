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
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
/// Le numéro n'est PAS modifiable après création.
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
}
