//! Entité `User` : compte utilisateur de l'application.
//!
//! **Sécurité** : `User` ne dérive PAS `Serialize` ni `Deserialize` pour
//! empêcher la fuite du `password_hash` via JSON (logs tracing, tests,
//! futures réponses API). Si `kesh-api` a besoin de sérialiser un User
//! (story 1.7), il créera un `UserDto` séparé qui exclut le hash.
//! `Debug` est également implémenté manuellement pour masquer le hash.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

/// Rôle d'un utilisateur dans le RBAC hiérarchique.
///
/// Hiérarchie : `Consultation < Comptable < Admin` (chaque rôle hérite
/// des permissions des rôles inférieurs). L'enforcement RBAC est dans
/// `kesh-api` (story 1.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Role {
    /// Administrateur : gestion des utilisateurs + accès comptable complet
    Admin,
    /// Comptable : CRUD sur toutes les données comptables
    Comptable,
    /// Consultation : lecture seule
    Consultation,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "Admin",
            Self::Comptable => "Comptable",
            Self::Consultation => "Consultation",
        }
    }

    /// Niveau hiérarchique du rôle (0 = le plus bas).
    ///
    /// Utilisé par `Ord` pour implémenter la hiérarchie RBAC :
    /// `Consultation(0) < Comptable(1) < Admin(2)`.
    pub fn level(&self) -> u8 {
        match self {
            Self::Consultation => 0,
            Self::Comptable => 1,
            Self::Admin => 2,
        }
    }
}

impl PartialOrd for Role {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Role {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.level().cmp(&other.level())
    }
}

impl std::str::FromStr for Role {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Admin" => Ok(Self::Admin),
            "Comptable" => Ok(Self::Comptable),
            "Consultation" => Ok(Self::Consultation),
            other => Err(format!("Role inconnu : {other}")),
        }
    }
}

impl Type<MySql> for Role {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for Role {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for Role {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// Utilisateur persisté en base.
///
/// Le `password_hash` contient le hash Argon2id au format PHC string.
/// Le hachage lui-même est fait dans `kesh-api` (story 1.5) — ce crate
/// stocke la chaîne telle que fournie.
///
/// **Multi-tenant (Story 6.2)** : `company_id` lie l'utilisateur à sa company.
/// Les requêtes authentifiées scopent par ce champ via `CurrentUser.company_id`.
#[derive(Clone, sqlx::FromRow)]
// NOTE: PAS de derive Debug/Serialize/Deserialize — Debug manuel ci-dessous,
// et Serialize/Deserialize interdits pour éviter la fuite du password_hash.
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: Role,
    pub active: bool,
    pub company_id: i64,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("password_hash", &"***")
            .field("role", &self.role)
            .field("active", &self.active)
            .field("company_id", &self.company_id)
            .field("version", &self.version)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

/// Données de création d'un utilisateur.
///
/// Le `password_hash` doit être fourni **déjà haché** par l'appelant
/// (typiquement `kesh-api` avec Argon2id). Ce crate ne fait jamais
/// de hachage lui-même.
///
/// **Multi-tenant (Story 6.2)** : `company_id` est obligatoire — un user
/// doit appartenir à exactement une company.
#[derive(Clone)]
pub struct NewUser {
    pub username: String,
    pub password_hash: String,
    pub role: Role,
    pub active: bool,
    pub company_id: i64,
}

impl std::fmt::Debug for NewUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NewUser")
            .field("username", &self.username)
            .field("password_hash", &"***")
            .field("role", &self.role)
            .field("active", &self.active)
            .field("company_id", &self.company_id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_hierarchy_consultation_less_than_comptable() {
        assert!(Role::Consultation < Role::Comptable);
    }

    #[test]
    fn role_hierarchy_comptable_less_than_admin() {
        assert!(Role::Comptable < Role::Admin);
    }

    #[test]
    fn role_hierarchy_consultation_less_than_admin() {
        assert!(Role::Consultation < Role::Admin);
    }

    #[test]
    fn role_hierarchy_admin_ge_comptable() {
        assert!(Role::Admin >= Role::Comptable);
    }

    #[test]
    fn role_hierarchy_comptable_ge_consultation() {
        assert!(Role::Comptable >= Role::Consultation);
    }

    #[test]
    fn role_hierarchy_self_equality() {
        assert!(Role::Consultation <= Role::Consultation);
        assert!(Role::Comptable >= Role::Comptable);
        assert!(Role::Admin == Role::Admin);
    }

    #[test]
    fn role_ord_consistent_with_partial_eq() {
        // Vérifie que Ord et PartialEq sont cohérents :
        // a == b ssi a.cmp(b) == Equal (contrat Rust std)
        let roles = [Role::Consultation, Role::Comptable, Role::Admin];
        for a in &roles {
            for b in &roles {
                assert_eq!(
                    a == b,
                    a.cmp(b) == std::cmp::Ordering::Equal,
                    "Ord/PartialEq inconsistency for {:?} vs {:?}",
                    a,
                    b
                );
            }
        }
    }

    #[test]
    fn role_levels() {
        assert_eq!(Role::Consultation.level(), 0);
        assert_eq!(Role::Comptable.level(), 1);
        assert_eq!(Role::Admin.level(), 2);
    }
}

/// Données de mise à jour d'un utilisateur : rôle et activation.
///
/// Le `password_hash` et le `username` ne sont PAS modifiables via cet update.
/// Story 1.7 introduira des flux dédiés (`change_password`, `rename_user`).
#[derive(Debug, Clone)]
pub struct UserUpdate {
    pub role: Role,
    pub active: bool,
}
