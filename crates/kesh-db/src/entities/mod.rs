//! Entités de données persistées.
//!
//! Chaque entité correspond à une table MariaDB. Les structs de création
//! (`New*`) et de mise à jour (`*Update`) ne contiennent que les champs
//! modifiables par le client — l'id, la version et les timestamps sont
//! gérés par la base.

pub mod company;
pub mod fiscal_year;
pub mod onboarding;
pub mod refresh_token;
pub mod user;

pub use company::{Company, CompanyUpdate, Language, NewCompany, OrgType};
pub use fiscal_year::{FiscalYear, FiscalYearStatus, NewFiscalYear};
pub use onboarding::{OnboardingState, UiMode};
pub use refresh_token::{NewRefreshToken, RefreshToken};
pub use user::{NewUser, Role, User, UserUpdate};
