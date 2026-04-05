//! Encode/decode JWT HS256 pour l'authentification.
//!
//! Claims conformes RFC 7519 : `sub` (user_id en String), `role`,
//! `iat`, `exp`. Validation avec `leeway = 60s` pour absorber le
//! clock drift NTP.

use std::collections::HashSet;
use std::sync::LazyLock;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use kesh_db::entities::Role;
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

/// Set des claims requis par la validation JWT.
///
/// `LazyLock` évite la **reconstruction** (hashing de 3 strings, insertion
/// dans un HashMap bucket) à chaque decode. Le `.clone()` au point d'usage
/// reste une allocation — `jsonwebtoken::Validation` possède son champ
/// `required_spec_claims` par valeur, donc le clone est inévitable sans
/// changer l'API du crate. Patch V5 : commentaire corrigé pour refléter
/// la vraie nature de l'optimisation (le patch #13 initial disait « évite
/// l'allocation » ce qui était factuellement faux).
static REQUIRED_SPEC_CLAIMS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    ["exp", "sub", "iat"]
        .iter()
        .map(|s| s.to_string())
        .collect()
});

/// Claims portés par le JWT d'accès.
///
/// `sub` est un String (RFC 7519 §4.1.2 impose `StringOrURI`).
/// Le user_id i64 est sérialisé via `.to_string()` au moment de l'encode
/// et parsé côté middleware avec `.parse::<i64>()`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — user_id sérialisé en String (conformité RFC 7519).
    pub sub: String,
    /// Rôle RBAC : `Admin`, `Comptable`, `Consultation`.
    pub role: String,
    /// Issued at — unix timestamp (seconds).
    pub iat: i64,
    /// Expires at — unix timestamp (seconds).
    pub exp: i64,
}

/// Encode un JWT HS256 pour un utilisateur donné.
///
/// Force `Algorithm::HS256` explicitement (pas `Header::default()`)
/// pour éviter toute attaque via `alg: none`.
pub fn encode(
    user_id: i64,
    role: Role,
    secret: &[u8],
    lifetime: TimeDelta,
) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.as_str().to_owned(),
        iat: now.timestamp(),
        exp: (now + lifetime).timestamp(),
    };

    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|e| AppError::Internal(format!("jwt encode: {e}")))
}

/// Décode et valide un JWT HS256.
///
/// - Algo HS256 forcé (protection `alg: none`).
/// - `leeway = 60s` pour absorber le clock drift NTP.
/// - Claims `exp`, `sub`, `iat` obligatoires.
pub fn decode(token: &str, secret: &[u8]) -> Result<Claims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 60; // Tolérance clock drift NTP
    validation.required_spec_claims = REQUIRED_SPEC_CLAIMS.clone();

    jsonwebtoken::decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
        .map(|data| data.claims)
        .map_err(|e| AppError::Unauthenticated(format!("jwt decode: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

    #[test]
    fn encode_decode_round_trip() {
        let token = encode(42, Role::Comptable, TEST_SECRET, TimeDelta::minutes(15))
            .expect("encode should succeed");

        let claims = decode(&token, TEST_SECRET).expect("decode should succeed");
        assert_eq!(claims.sub, "42");
        assert_eq!(claims.role, "Comptable");
        // `exp` doit être dans ~15 minutes (tolérance 60s)
        let now = Utc::now().timestamp();
        let expected_exp = now + 15 * 60;
        assert!((claims.exp - expected_exp).abs() < 5);
    }

    #[test]
    fn decode_fails_on_wrong_secret() {
        let token = encode(1, Role::Admin, TEST_SECRET, TimeDelta::minutes(15))
            .expect("encode should succeed");

        let wrong_secret = b"wrong-secret-32-bytes-minimum-padding-long-enough";
        let result = decode(&token, wrong_secret);
        assert!(matches!(result, Err(AppError::Unauthenticated(_))));
    }

    #[test]
    fn decode_fails_on_expired_token_beyond_leeway() {
        // lifetime négatif : token émis maintenant, expiré il y a 120s
        let token = encode(1, Role::Admin, TEST_SECRET, TimeDelta::seconds(-120))
            .expect("encode should succeed");

        let result = decode(&token, TEST_SECRET);
        assert!(
            matches!(result, Err(AppError::Unauthenticated(_))),
            "expired token beyond leeway should fail, got {:?}",
            result
        );
    }

    #[test]
    fn decode_succeeds_on_expired_token_within_leeway() {
        // Token expiré il y a 30s, dans le leeway 60s → doit passer
        let token = encode(1, Role::Admin, TEST_SECRET, TimeDelta::seconds(-30))
            .expect("encode should succeed");

        let result = decode(&token, TEST_SECRET);
        assert!(
            result.is_ok(),
            "expired token within leeway should succeed, got {:?}",
            result
        );
    }

    #[test]
    fn decode_fails_on_garbage_token() {
        let result = decode("not-a-jwt", TEST_SECRET);
        assert!(matches!(result, Err(AppError::Unauthenticated(_))));
    }

    #[test]
    fn decode_succeeds_but_sub_is_parseable_as_i64() {
        let token = encode(9999, Role::Consultation, TEST_SECRET, TimeDelta::minutes(5))
            .expect("encode should succeed");

        let claims = decode(&token, TEST_SECRET).expect("decode should succeed");
        let user_id: i64 = claims.sub.parse().expect("sub should parse as i64");
        assert_eq!(user_id, 9999);
    }
}
