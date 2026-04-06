//! Rate limiter en mémoire pour `/api/v1/auth/login`.
//!
//! Compteur par IP avec fenêtre glissante et blocage temporaire.
//! Utilise `std::sync::Mutex` (pas `tokio::sync::Mutex`) — le lock
//! est toujours relâché avant tout `.await`.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::Instant;

use crate::config::Config;

/// Enregistrement des tentatives pour une IP donnée.
#[derive(Debug)]
struct AttemptRecord {
    /// Timestamps des tentatives échouées dans la fenêtre glissante.
    attempts: Vec<Instant>,
    /// Si présent, l'IP est bloquée jusqu'à cet instant.
    blocked_until: Option<Instant>,
}

/// Erreur retournée quand une IP est bloquée.
#[derive(Debug)]
pub struct RateLimitReject {
    /// Nombre de secondes avant déblocage.
    pub retry_after_secs: u64,
}

/// Rate limiter par IP, protégé par un `std::sync::Mutex`.
///
/// Injecté dans `AppState` et partagé entre tous les handlers.
/// Le nettoyage des entrées expirées est paresseux (lazy) : exécuté
/// avant chaque `check_rate_limit` pour éviter les faux blocages.
pub struct RateLimiter {
    inner: Mutex<HashMap<IpAddr, AttemptRecord>>,
    max_attempts: u32,
    window: std::time::Duration,
    block_duration: std::time::Duration,
}

impl RateLimiter {
    /// Crée un `RateLimiter` à partir de la configuration.
    pub fn new(config: &Config) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            max_attempts: config.rate_limit_max_attempts,
            window: config
                .rate_limit_window
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(900)),
            block_duration: config
                .rate_limit_block_duration
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(1800)),
        }
    }

    /// Vérifie si l'IP est autorisée à tenter un login.
    ///
    /// Effectue d'abord un nettoyage lazy des entrées expirées,
    /// puis vérifie le blocage et le seuil de tentatives.
    ///
    /// Retourne `Ok(())` si autorisé, `Err(RateLimitReject)` si bloqué.
    pub fn check_rate_limit(&self, ip: IpAddr) -> Result<(), RateLimitReject> {
        let mut map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let now = Instant::now();

        // Nettoyage lazy : supprimer les entrées dont le blocage ET la
        // fenêtre sont tous deux expirés.
        let cutoff = self.block_duration + self.window;
        map.retain(|_, record| {
            if let Some(blocked_until) = record.blocked_until {
                if now >= blocked_until {
                    // Blocage expiré → vérifier si les tentatives sont aussi expirées
                    record.blocked_until = None;
                    record.attempts.clear();
                    return false; // entry fully expired, remove
                }
                return true; // still blocked
            }
            // Pas bloqué : garder si la dernière tentative est dans la fenêtre
            record
                .attempts
                .last()
                .map(|t| now.duration_since(*t) < cutoff)
                .unwrap_or(false)
        });

        let record = match map.get(&ip) {
            Some(r) => r,
            None => return Ok(()),
        };

        // Vérifier le blocage actif
        if let Some(blocked_until) = record.blocked_until {
            if now < blocked_until {
                let remaining = blocked_until.duration_since(now);
                return Err(RateLimitReject {
                    retry_after_secs: remaining.as_secs().max(1),
                });
            }
        }

        // Compter les tentatives dans la fenêtre
        let window_start = now - self.window;
        let recent_count = record
            .attempts
            .iter()
            .filter(|t| **t >= window_start)
            .count() as u32;

        if recent_count >= self.max_attempts {
            // Ne devrait pas arriver ici (le blocage est posé dans record_failed_attempt),
            // mais défense en profondeur.
            return Err(RateLimitReject {
                retry_after_secs: self.block_duration.as_secs().max(1),
            });
        }

        Ok(())
    }

    /// Enregistre une tentative de login échouée pour une IP.
    ///
    /// Si le seuil est atteint, l'IP est bloquée pour `block_duration`.
    pub fn record_failed_attempt(&self, ip: IpAddr) {
        let mut map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let now = Instant::now();

        let record = map.entry(ip).or_insert_with(|| AttemptRecord {
            attempts: Vec::new(),
            blocked_until: None,
        });

        // Purger les tentatives hors fenêtre
        let window_start = now - self.window;
        record.attempts.retain(|t| *t >= window_start);

        record.attempts.push(now);

        if record.attempts.len() as u32 >= self.max_attempts {
            record.blocked_until = Some(now + self.block_duration);
        }
    }

    /// Réinitialise le compteur pour une IP après un login réussi.
    pub fn reset(&self, ip: IpAddr) {
        let mut map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        map.remove(&ip);
    }
}

// Clone is needed because AppState derives Clone.
// The inner Mutex is wrapped in an Arc at the AppState level.
impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimiter")
            .field("max_attempts", &self.max_attempts)
            .field("window", &self.window)
            .field("block_duration", &self.block_duration)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_helpers::make_test_config;

    fn make_rate_limiter(max: u32, window_secs: u64, block_secs: u64) -> RateLimiter {
        let mut config = make_test_config("admin", "password");
        config.rate_limit_max_attempts = max;
        config.rate_limit_window =
            chrono::TimeDelta::seconds(window_secs as i64);
        config.rate_limit_block_duration =
            chrono::TimeDelta::seconds(block_secs as i64);
        RateLimiter::new(&config)
    }

    #[test]
    fn allows_under_threshold() {
        let rl = make_rate_limiter(5, 60, 60);
        let ip: IpAddr = "10.0.0.1".parse().unwrap();

        for _ in 0..4 {
            assert!(rl.check_rate_limit(ip).is_ok());
            rl.record_failed_attempt(ip);
        }
        // 4 attempts, threshold is 5 → still allowed
        assert!(rl.check_rate_limit(ip).is_ok());
    }

    #[test]
    fn blocks_at_threshold() {
        let rl = make_rate_limiter(3, 60, 60);
        let ip: IpAddr = "10.0.0.2".parse().unwrap();

        for _ in 0..3 {
            rl.record_failed_attempt(ip);
        }

        let result = rl.check_rate_limit(ip);
        assert!(result.is_err());
        let reject = result.unwrap_err();
        assert!(reject.retry_after_secs > 0);
    }

    #[test]
    fn reset_clears_counter() {
        let rl = make_rate_limiter(3, 60, 60);
        let ip: IpAddr = "10.0.0.3".parse().unwrap();

        for _ in 0..2 {
            rl.record_failed_attempt(ip);
        }

        rl.reset(ip);

        // Counter reset → can do 3 more attempts before block
        for _ in 0..2 {
            rl.record_failed_attempt(ip);
        }
        assert!(rl.check_rate_limit(ip).is_ok());
    }

    #[test]
    fn two_ips_independent() {
        let rl = make_rate_limiter(2, 60, 60);
        let ip1: IpAddr = "10.0.0.4".parse().unwrap();
        let ip2: IpAddr = "10.0.0.5".parse().unwrap();

        for _ in 0..2 {
            rl.record_failed_attempt(ip1);
        }

        // ip1 blocked, ip2 still free
        assert!(rl.check_rate_limit(ip1).is_err());
        assert!(rl.check_rate_limit(ip2).is_ok());
    }

    #[test]
    fn unknown_ip_always_allowed() {
        let rl = make_rate_limiter(5, 60, 60);
        let ip: IpAddr = "10.0.0.99".parse().unwrap();
        assert!(rl.check_rate_limit(ip).is_ok());
    }

    #[test]
    fn concurrent_same_ip_records_all_attempts() {
        use std::sync::Arc;
        use std::thread;

        let rl = Arc::new(make_rate_limiter(10, 60, 60));
        let ip: IpAddr = "10.0.0.50".parse().unwrap();

        // Spawn 10 threads, each recording one failed attempt
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let rl = Arc::clone(&rl);
                thread::spawn(move || {
                    rl.record_failed_attempt(ip);
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        // After 10 concurrent attempts with threshold=10, should be blocked
        assert!(
            rl.check_rate_limit(ip).is_err(),
            "10 concurrent attempts should trigger block at threshold=10"
        );
    }
}
