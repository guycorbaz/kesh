//! Hash et vérification de mot de passe avec Argon2id.
//!
//! Paramètres : `Argon2::default()` — m=19456 KiB, t=2, p=1, variante
//! `Argon2id`, coût ~50 ms. PHC string standard stocké en base.
//!
//! **Dette Argon2 sync / tokio** : les appels `hash_password` et
//! `verify_password` sont synchrones et bloquent le worker tokio pendant
//! ~50 ms. Pour MVP 2-5 users, acceptable. Avant mise en production,
//! livrer la story 1.6 (rate limiting) **et** wrapper les appels dans
//! `tokio::task::spawn_blocking`. Documenté dans le story file Dev Notes
//! section *Performance/DoS debt*.

use std::sync::LazyLock;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::errors::AppError;

/// Hash un mot de passe en clair en PHC string Argon2id.
pub fn hash_password(plain: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(plain.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(format!("argon2 hash: {e}")))
}

/// Vérifie un mot de passe en clair contre un PHC string Argon2id.
///
/// Retourne `Ok(true)` en cas de match, `Ok(false)` en cas de mismatch,
/// `Err(AppError::Internal)` si le PHC string est mal formé (bug serveur).
pub fn verify_password(plain: &str, phc: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(phc)
        .map_err(|e| AppError::Internal(format!("argon2 phc parse: {e}")))?;

    match Argon2::default().verify_password(plain.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(AppError::Internal(format!("argon2 verify: {e}"))),
    }
}

/// Wrapper async pour `hash_password`, utilisant `spawn_blocking`
/// pour ne pas bloquer le runtime tokio (~50ms par appel Argon2).
pub async fn hash_password_async(plain: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || hash_password(&plain))
        .await
        .map_err(|_| AppError::Internal("argon2 hash thread panic".into()))?
}

/// Wrapper async pour `verify_password`, utilisant `spawn_blocking`.
pub async fn verify_password_async(plain: String, phc: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || verify_password(&plain, &phc))
        .await
        .map_err(|_| AppError::Internal("argon2 verify thread panic".into()))?
}

/// PHC string statique consommé par `dummy_verify` pour normaliser la
/// durée de login quand l'utilisateur n'existe pas ou est inactif.
///
/// Généré une fois via `LazyLock`. **Important** : `main.rs` appelle
/// `warm_up_dummy_hash()` au démarrage pour que l'éventuelle panique
/// (`OsRng` indisponible dans un conteneur hardened, etc.) se produise
/// à l'initialisation du process, pas dans le handler du premier login.
static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    hash_password("dummy-password-never-matches-anything")
        .expect("dummy hash generation must succeed at startup")
});

/// Force l'initialisation eager de `DUMMY_HASH`.
///
/// À appeler dans `main.rs` après le bootstrap admin pour que toute
/// panique d'init (ex. `OsRng` indisponible) tombe immédiatement, pas
/// au premier login. Sans ce pré-chauffage, une défaillance d'Argon2
/// au premier appel ferait paniquer le handler async et laisserait le
/// process dans un état bancal (tous les logins suivants feraient
/// re-paniquer la LazyLock).
pub fn warm_up_dummy_hash() {
    // Force l'évaluation du LazyLock. Si `hash_password` panique,
    // ça panique ici, à un moment où le process n'a pas encore
    // commencé à servir des requêtes.
    let _ = &*DUMMY_HASH;
}

/// Exécute un verify Argon2id sur un hash factice pour brûler les
/// cycles CPU et normaliser la durée de la réponse login.
///
/// Appelée quand `find_by_username` retourne `None` ou quand l'utilisateur
/// est inactif, afin d'empêcher l'énumération par timing attack (un
/// attaquant ne peut pas distinguer « user inconnu » de « user actif +
/// mauvais mot de passe » via la durée de réponse).
///
/// Le résultat est passé à `std::hint::black_box` pour empêcher LLVM
/// d'élider l'appel en cas de refactor futur de `verify_password` qui
/// le rendrait `#[inline]` et visiblement pure (patch #14). Aujourd'hui
/// Argon2 a des effets de bord opaques qui empêchent déjà l'élision,
/// mais cette défense en profondeur protège contre des changements
/// silencieux dans les versions futures du crate.
pub fn dummy_verify() {
    let result = verify_password("wrong-dummy-password", &DUMMY_HASH);
    let _ = std::hint::black_box(result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_round_trip() {
        let plain = "hunter2";
        let phc = hash_password(plain).expect("hash should succeed");
        assert!(phc.starts_with("$argon2id$"));
        assert!(
            verify_password(plain, &phc).expect("verify should succeed"),
            "hash/verify round trip should return true"
        );
    }

    #[test]
    fn verify_returns_false_on_wrong_password() {
        let phc = hash_password("correct-password").expect("hash should succeed");
        let is_valid = verify_password("wrong-password", &phc).expect("verify should not error");
        assert!(!is_valid);
    }

    #[test]
    fn verify_returns_err_on_malformed_phc() {
        let result = verify_password("anything", "not-a-phc-string");
        assert!(
            matches!(result, Err(AppError::Internal(_))),
            "malformed PHC should return AppError::Internal, got {:?}",
            result
        );
    }

    #[test]
    fn dummy_verify_does_not_panic() {
        // Le but de dummy_verify est de consommer du CPU sans retourner
        // de valeur — on vérifie juste qu'il n'explose pas.
        dummy_verify();
        dummy_verify();
    }
}
