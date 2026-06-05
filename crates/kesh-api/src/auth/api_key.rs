//! Génération et validation des clés d'accès API (PAT) — Story 17-2a (#100).
//!
//! Format de clé : `kesh_pat_<base62>` où `<base62>` encode ≥ 160 bits
//! d'entropie cryptographique (DC1/AC2). Le secret en clair n'existe qu'en
//! mémoire au moment de la création ; seul `SHA-256(token)` hex est persisté
//! (`key_hash`), avec un index UNIQUE permettant un lookup O(1) à
//! l'authentification (pattern GitHub PAT).
//!
//! **DC1 — pas d'Argon2id par requête** : un PAT est un secret aléatoire à
//! haute entropie, pas un mot de passe faible. Le hashing lent (anti-bruteforce)
//! n'apporte rien et coûterait ~50 ms × N req/s. On utilise SHA-256 (rapide,
//! déterministe, indexable).
//!
//! **RNG** : `OsRng` est réutilisé depuis le chemin déjà présent
//! `argon2::password_hash::rand_core` (le même CSPRNG que le salt Argon2 de
//! `auth/password.rs`) — DRY, aucune nouvelle dépendance Cargo.

use argon2::password_hash::rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use sqlx::mysql::MySqlPool;

use kesh_db::entities::ApiKeyScope;
use kesh_db::repositories::api_keys;

use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

/// Préfixe discriminant des clés PAT. Testé **case-sensitive exact** (octets)
/// par le middleware pour router vers `validate_pat` plutôt que `jwt::decode`.
pub const PAT_PREFIX: &str = "kesh_pat_";

/// Nombre d'octets aléatoires du secret (160 bits — AC2).
const PAT_ENTROPY_BYTES: usize = 20;

/// Alphabet base62 (`0-9A-Za-z`) — encodage inline (pas de crate dédiée, DC).
const BASE62_ALPHABET: &[u8; 62] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Encode un tableau d'octets (big-endian) en base62.
///
/// Traite `bytes` comme un grand entier et le divise répétitivement par 62.
/// Implémentation inline ~15 lignes — pas de dépendance externe.
fn base62_encode(bytes: &[u8]) -> String {
    let mut num = bytes.to_vec();
    let mut out: Vec<u8> = Vec::new();
    while num.iter().any(|&b| b != 0) {
        let mut rem: u32 = 0;
        for byte in num.iter_mut() {
            let acc = (rem << 8) | (*byte as u32);
            *byte = (acc / 62) as u8;
            rem = acc % 62;
        }
        out.push(BASE62_ALPHABET[rem as usize]);
    }
    if out.is_empty() {
        out.push(BASE62_ALPHABET[0]);
    }
    out.reverse();
    // Tous les octets proviennent de BASE62_ALPHABET (ASCII) → UTF-8 valide.
    String::from_utf8(out).expect("base62 alphabet is ASCII")
}

/// Calcule `SHA-256(token)` encodé en hex (64 chars minuscules).
pub fn sha256_hex(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        write!(hex, "{byte:02x}").expect("write to String never fails");
    }
    hex
}

/// Génère une nouvelle clé PAT. Retourne `(token_clair, key_hash)` :
/// - `token_clair` = `kesh_pat_<base62>` — affiché **une seule fois** au client,
///   jamais persisté.
/// - `key_hash` = `SHA-256(token_clair)` hex — stocké en DB.
pub fn generate_pat() -> (String, String) {
    let mut buf = [0u8; PAT_ENTROPY_BYTES];
    OsRng.fill_bytes(&mut buf);
    let token = format!("{PAT_PREFIX}{}", base62_encode(&buf));
    let key_hash = sha256_hex(&token);
    (token, key_hash)
}

/// Valide un PAT et construit le `CurrentUser` correspondant (DC2/AC5).
///
/// 1. `SHA-256(token)` → lookup `find_active_auth_by_key_hash` (clé active :
///    non révoquée, non expirée), jointe au créateur (rôle/état COURANT).
/// 2. Créateur inactif → `401` (une désactivation invalide le PAT immédiatement).
/// 3. Construit `CurrentUser { user_id = créateur, role = rôle courant,
///    company_id = company de la clé, api_key_id = Some(id) }`.
/// 4. `touch_last_used` best-effort (un échec est loggé, n'échoue PAS la requête).
///
/// Retourne aussi le `scope` (consommé par le gate DC3 du middleware).
/// Token inconnu/révoqué/expiré → `AppError::Unauthenticated` (401).
pub async fn validate_pat(
    token: &str,
    pool: &MySqlPool,
) -> Result<(CurrentUser, ApiKeyScope), AppError> {
    let key_hash = sha256_hex(token);

    let row = api_keys::find_active_auth_by_key_hash(pool, &key_hash)
        .await?
        .ok_or_else(|| AppError::Unauthenticated("pat: unknown, revoked, or expired key".into()))?;

    if !row.creator_active {
        return Err(AppError::Unauthenticated(
            "pat: creator user is inactive".into(),
        ));
    }

    // S4-2 — `exp` PAT : timestamp d'expiration de la clé, ou i64::MAX si
    // permanente (ne jamais laisser 0 qui donnerait un expires_in incohérent).
    let exp = row
        .expires_at
        .map(|dt| dt.and_utc().timestamp())
        .unwrap_or(i64::MAX);

    let current_user = CurrentUser {
        user_id: row.created_by_user_id,
        role: row.creator_role,
        company_id: row.company_id,
        exp,
        api_key_id: Some(row.api_key_id),
    };

    // Best-effort (AC5) — eventual consistency, ne fait jamais échouer la requête.
    if let Err(e) = api_keys::touch_last_used(pool, &key_hash).await {
        tracing::warn!("pat touch_last_used failed (non-fatal): {e}");
    }

    Ok((current_user, row.scope))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_pat_format_and_entropy() {
        let (token, hash) = generate_pat();
        assert!(token.starts_with("kesh_pat_"), "préfixe attendu");
        // Partie base62 non vide et suffisamment longue pour 160 bits (~27 chars).
        let body = token.strip_prefix("kesh_pat_").unwrap();
        assert!(
            body.len() >= 20,
            "base62 de 160 bits doit faire ~27 chars, got {}",
            body.len()
        );
        assert!(
            body.bytes().all(|b| b.is_ascii_alphanumeric()),
            "base62 = alphanumérique uniquement"
        );
        // Le hash est un SHA-256 hex (64 chars).
        assert_eq!(hash.len(), 64);
        assert!(hash.bytes().all(|b| b.is_ascii_hexdigit()));
    }

    #[test]
    fn generate_pat_is_unique() {
        let (t1, h1) = generate_pat();
        let (t2, h2) = generate_pat();
        assert_ne!(t1, t2, "deux clés générées doivent différer");
        assert_ne!(h1, h2);
    }

    #[test]
    fn sha256_hex_is_deterministic_and_known() {
        // Vecteur de test SHA-256 connu : "" → e3b0c442...
        assert_eq!(
            sha256_hex(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(sha256_hex("kesh"), sha256_hex("kesh"));
        assert_ne!(sha256_hex("a"), sha256_hex("b"));
    }

    #[test]
    fn base62_encode_alphabet_and_determinism() {
        let bytes = [0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02, 0x03, 0x04];
        let a = base62_encode(&bytes);
        let b = base62_encode(&bytes);
        assert_eq!(a, b, "déterministe");
        assert!(a.bytes().all(|c| c.is_ascii_alphanumeric()));
        assert_eq!(base62_encode(&[0, 0, 0]), "0", "all-zero → '0'");
        assert_ne!(base62_encode(&[1]), base62_encode(&[2]));
    }
}
