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

/// Largeur fixe de la partie base62 du token. 160 bits big-endian tiennent dans
/// 27 digits base62 (`62^27 > 2^160 > 62^26`) ; on left-pad au digit zéro pour
/// garantir une longueur de token constante (code-review 17-2a Pass 1).
const PAT_BASE62_LEN: usize = 27;

/// Alphabet base62 (`0-9A-Za-z`) — encodage inline (pas de crate dédiée, DC).
const BASE62_ALPHABET: &[u8; 62] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Encode un tableau d'octets (big-endian) en base62.
///
/// Traite `bytes` comme un grand entier et le divise répétitivement par 62.
/// Implémentation inline ~15 lignes — pas de dépendance externe.
///
/// `pub(crate)` (Story 17-4c) : partagé avec `generate_reset_token` pour éviter
/// la duplication de l'encodage base62 (DC3).
pub(crate) fn base62_encode(bytes: &[u8]) -> String {
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

/// Encode `bytes` en base62 à **largeur fixe** `PAT_BASE62_LEN`, left-padé au
/// digit zéro (`'0'`). Garantit une longueur de token constante quels que soient
/// les octets de tête (un buffer big-endian commençant par des `0x00` produit
/// moins de digits). Le padding est déterministe et injectif → l'entropie et
/// l'unicité du secret sont préservées (code-review 17-2a Pass 1).
///
/// `pub(crate)` (Story 17-4c) : réutilisé par `generate_reset_token` (token de
/// réinitialisation de mot de passe, même format base62 largeur fixe, DC3).
pub(crate) fn base62_fixed_width(bytes: &[u8]) -> String {
    // L'invariant de largeur fixe ne tient que pour un secret de `PAT_ENTROPY_BYTES`
    // octets (160 bits → ≤ 27 digits base62). Un buffer plus long encoderait sur
    // plus de 27 digits et le left-pad ne tronquerait pas → invariant cassé. Garde
    // défensive en debug pour tout futur appelant (code-review 17-2a Pass 2).
    debug_assert_eq!(
        bytes.len(),
        PAT_ENTROPY_BYTES,
        "base62_fixed_width attend exactement {PAT_ENTROPY_BYTES} octets"
    );
    let mut body = base62_encode(bytes);
    if body.len() < PAT_BASE62_LEN {
        body.insert_str(0, &"0".repeat(PAT_BASE62_LEN - body.len()));
    }
    body
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
    let token = format!("{PAT_PREFIX}{}", base62_fixed_width(&buf));
    let key_hash = sha256_hex(&token);
    (token, key_hash)
}

/// Story 17-4c (DC3) — Génère un token de réinitialisation de mot de passe.
///
/// Retourne `(token_clair, token_hash)` :
/// - `token_clair` = 27 chars base62 (`0-9A-Za-z`), **sans préfixe** : un token
///   de reset n'est PAS une clé PAT. Le préfixe `kesh_pat_` route vers
///   `validate_pat` côté middleware ; un token reset ne doit jamais matcher ce
///   chemin. base62 est URL-safe → aucun escaping dans le query param de l'email.
/// - `token_hash` = `SHA-256(token_clair)` hex — seul élément persisté
///   (`password_reset_tokens.token_hash`). Le brut ne vit que dans l'URL envoyée
///   par email.
///
/// Généralise le cœur entropique de [`generate_pat`] (mêmes 160 bits `OsRng` +
/// `base62_fixed_width`) sans dupliquer l'encodage base62 (DC3).
pub fn generate_reset_token() -> (String, String) {
    let mut buf = [0u8; PAT_ENTROPY_BYTES];
    OsRng.fill_bytes(&mut buf);
    let token = base62_fixed_width(&buf);
    let token_hash = sha256_hex(&token);
    (token, token_hash)
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
        // Partie base62 à largeur fixe : 160 bits → exactement 27 chars (left-padés
        // au digit zéro si octets de tête nuls — voir PAT_BASE62_LEN).
        let body = token.strip_prefix("kesh_pat_").unwrap();
        assert_eq!(
            body.len(),
            PAT_BASE62_LEN,
            "token base62 doit faire exactement {PAT_BASE62_LEN} chars (largeur fixe), got {}",
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
    fn generate_reset_token_format_alphabet_and_hash() {
        let (token, hash) = generate_reset_token();
        // 27 chars base62 largeur fixe (160 bits), SANS préfixe `kesh_pat_`.
        assert_eq!(
            token.len(),
            PAT_BASE62_LEN,
            "token reset doit faire exactement {PAT_BASE62_LEN} chars (largeur fixe), got {}",
            token.len()
        );
        assert!(
            !token.starts_with(PAT_PREFIX),
            "token reset NE DOIT PAS porter le préfixe PAT (routerait vers validate_pat)"
        );
        assert!(
            token.bytes().all(|b| b.is_ascii_alphanumeric()),
            "base62 = alphanumérique uniquement (URL-safe sans escaping)"
        );
        // Le hash stocké est bien SHA-256(token_clair) hex (64 chars).
        assert_eq!(hash.len(), 64);
        assert!(hash.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_eq!(hash, sha256_hex(&token), "hash == sha256_hex(token brut)");
    }

    #[test]
    fn generate_reset_token_is_unique() {
        let (t1, h1) = generate_reset_token();
        let (t2, h2) = generate_reset_token();
        assert_ne!(t1, t2, "deux tokens reset générés doivent différer");
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

    #[test]
    fn base62_fixed_width_pads_to_constant_length() {
        // Buffer 20 octets dont la valeur est petite (octets de tête nuls) :
        // l'encodage brut fait < 27 chars, le helper doit le padder à 27.
        let mut small = [0u8; PAT_ENTROPY_BYTES];
        small[PAT_ENTROPY_BYTES - 1] = 1; // valeur = 1
        let padded = base62_fixed_width(&small);
        assert_eq!(padded.len(), PAT_BASE62_LEN, "largeur fixe");
        assert!(padded.starts_with('0'), "left-padé au digit zéro");
        assert!(padded.ends_with('1'), "valeur significative préservée");

        // All-zero → 27 zéros (cas extrême, jamais produit par OsRng en pratique).
        assert_eq!(
            base62_fixed_width(&[0u8; PAT_ENTROPY_BYTES]),
            "0".repeat(PAT_BASE62_LEN)
        );

        // Injectivité : deux valeurs distinctes restent distinctes après padding.
        let mut two = [0u8; PAT_ENTROPY_BYTES];
        two[PAT_ENTROPY_BYTES - 1] = 2;
        assert_ne!(base62_fixed_width(&small), base62_fixed_width(&two));

        // Une valeur maximale (tous les bits à 1 = 160 bits) tient en 27 chars
        // sans dépassement (pas de padding, pas de troncature).
        let max = [0xFFu8; PAT_ENTROPY_BYTES];
        assert_eq!(base62_fixed_width(&max).len(), PAT_BASE62_LEN);
    }
}
