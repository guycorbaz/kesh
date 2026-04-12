//! Entité `OnboardingState` : état du flux d'onboarding (single-row).

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

/// Mode d'utilisation choisi à l'étape 2 de l'onboarding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiMode {
    Guided,
    Expert,
}

impl UiMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Guided => "guided",
            Self::Expert => "expert",
        }
    }
}

impl std::str::FromStr for UiMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "guided" => Ok(Self::Guided),
            "expert" => Ok(Self::Expert),
            other => Err(format!("UiMode inconnu : {other}")),
        }
    }
}

impl Type<MySql> for UiMode {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for UiMode {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for UiMode {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// État d'onboarding persisté en base (table single-row).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OnboardingState {
    pub id: i64,
    /// 0=pas commencé, 1=langue, 2=mode, 3=chemin choisi. 4-10 réservés Chemin B.
    pub step_completed: i32,
    pub is_demo: bool,
    /// NULL tant que l'étape 2 (mode) n'est pas complétée.
    pub ui_mode: Option<UiMode>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
