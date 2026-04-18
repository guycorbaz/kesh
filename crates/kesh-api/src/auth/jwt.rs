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
/// `LazyLock` évite la **reconstruction** (hashing de strings, insertion
/// dans un HashMap bucket) à chaque decode. Le `.clone()` au point d'usage
/// reste une allocation — `jsonwebtoken::Validation` possède son champ
/// `required_spec_claims` par valeur, donc le clone est inévitable sans
/// changer l'API du crate.
///
/// Story 6.2: `company_id` ajouté aux claims requis (multi-tenant scoping).
static REQUIRED_SPEC_CLAIMS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    ["exp", "sub", "iat", "company_id"]
        .iter()
        .map(|s| s.to_string())
        .collect()
});

/// Claims portés par le JWT d'accès.
///
/// `sub` est un String (RFC 7519 §4.1.2 impose `StringOrURI`).
/// Le user_id i64 est sérialisé via `.to_string()` au moment de l'encode
/// et parsé côté middleware avec `.parse::<i64>()`.
///
/// Story 6.2: `company_id` ajouté pour multi-tenant scoping.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — user_id sérialisé en String (conformité RFC 7519).
    pub sub: String,
    /// Rôle RBAC : `Admin`, `Comptable`, `Consultation`.
    pub role: String,
    /// Company ID — lié au user au moment du login (Story 6.2).
    pub company_id: i64,
    /// Issued at — unix timestamp (seconds).
    pub iat: i64,
    /// Expires at — unix timestamp (seconds).
    pub exp: i64,
}

/// Encode un JWT HS256 pour un utilisateur donné.
///
/// Force `Algorithm::HS256` explicitement (pas `Header::default()`)
/// pour éviter toute attaque via `alg: none`.
///
/// Story 6.2: `company_id` paramètre ajouté (multi-tenant scoping).
pub fn encode(
    user_id: i64,
    role: Role,
    company_id: i64,
    secret: &[u8],
    lifetime: TimeDelta,
) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.as_str().to_owned(),
        company_id,
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
        let token = encode(42, Role::Comptable, 7, TEST_SECRET, TimeDelta::minutes(15))
            .expect("encode should succeed");

        let claims = decode(&token, TEST_SECRET).expect("decode should succeed");
        assert_eq!(claims.sub, "42");
        assert_eq!(claims.role, "Comptable");
        assert_eq!(claims.company_id, 7);
        // `exp` doit être dans ~15 minutes (tolérance 60s)
        let now = Utc::now().timestamp();
        let expected_exp = now + 15 * 60;
        assert!((claims.exp - expected_exp).abs() < 5);
    }

    #[test]
    fn decode_fails_on_wrong_secret() {
        let token = encode(1, Role::Admin, 5, TEST_SECRET, TimeDelta::minutes(15))
            .expect("encode should succeed");

        let wrong_secret = b"wrong-secret-32-bytes-minimum-padding-long-enough";
        let result = decode(&token, wrong_secret);
        assert!(matches!(result, Err(AppError::Unauthenticated(_))));
    }

    #[test]
    fn decode_fails_on_expired_token_beyond_leeway() {
        // lifetime négatif : token émis maintenant, expiré il y a 120s
        let token = encode(1, Role::Admin, 5, TEST_SECRET, TimeDelta::seconds(-120))
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
        let token = encode(1, Role::Admin, 5, TEST_SECRET, TimeDelta::seconds(-30))
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
        let token = encode(9999, Role::Consultation, 5, TEST_SECRET, TimeDelta::minutes(5))
            .expect("encode should succeed");

        let claims = decode(&token, TEST_SECRET).expect("decode should succeed");
        let user_id: i64 = claims.sub.parse().expect("sub should parse as i64");
        assert_eq!(user_id, 9999);
    }

    #[test]
    fn encode_includes_company_id_in_claims() {
        let company_id = 123i64;
        let token = encode(1, Role::Admin, company_id, TEST_SECRET, TimeDelta::minutes(15))
            .expect("encode should succeed");

        let claims = decode(&token, TEST_SECRET).expect("decode should succeed");
        assert_eq!(claims.company_id, company_id);
    }

    #[test]
    fn decode_fails_on_missing_company_id_claim() {
        // Manually craft a token without company_id claim (legacy token)
        let now = Utc::now();
        let legacy_claims = serde_json::json!({
            "sub": "1",
            "role": "Admin",
            "iat": now.timestamp(),
            "exp": (now + chrono::Duration::minutes(15)).timestamp(),
        });

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
            &legacy_claims,
            &jsonwebtoken::EncodingKey::from_secret(TEST_SECRET),
        )
        .expect("crafted token should encode");

        let result = decode(&token, TEST_SECRET);
        assert!(
            matches!(result, Err(AppError::Unauthenticated(_))),
            "token without company_id should fail, got {:?}",
            result
        );
    }
}
