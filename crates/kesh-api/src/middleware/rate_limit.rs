//! Rate limiter en mémoire pour `/api/v1/auth/login`.
//!
//! Compteur par clé avec fenêtre glissante et blocage temporaire.
//! Historiquement keyé par IP (login, recovery) ; **générique depuis la
//! Story 20-3b1** (`RateLimiter<K = IpAddr>`) pour supporter un rate-limit
//! par `(company_id, user_id)` (envoi de factures par e-mail). Le paramètre
//! de type a un défaut → les call-sites existants (`RateLimiter` nu)
//! restent inchangés.
//! Utilise `std::sync::Mutex` (pas `tokio::sync::Mutex`) — le lock
//! est toujours relâché avant tout `.await`.

use std::collections::HashMap;
use std::hash::Hash;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::Instant;

use crate::config::Config;

/// Enregistrement des tentatives pour une clé donnée.
#[derive(Debug)]
struct AttemptRecord {
    /// Timestamps des tentatives échouées dans la fenêtre glissante.
    attempts: Vec<Instant>,
    /// Si présent, la clé est bloquée jusqu'à cet instant.
    blocked_until: Option<Instant>,
}

/// Erreur retournée quand une clé est bloquée.
#[derive(Debug)]
pub struct RateLimitReject {
    /// Nombre de secondes avant déblocage.
    pub retry_after_secs: u64,
}

/// Rate limiter par clé (`IpAddr` par défaut), protégé par un `std::sync::Mutex`.
///
/// Injecté dans `AppState` et partagé entre tous les handlers.
/// Le nettoyage des entrées expirées est paresseux (lazy) : exécuté
/// avant chaque `check_rate_limit` pour éviter les faux blocages.
pub struct RateLimiter<K = IpAddr> {
    inner: Mutex<HashMap<K, AttemptRecord>>,
    max_attempts: u32,
    window: std::time::Duration,
    block_duration: std::time::Duration,
}

impl<K: Eq + Hash + Copy> RateLimiter<K> {
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

    /// Story 17-4c (DC5) — Crée un `RateLimiter` avec des seuils explicites,
    /// indépendants de la `Config` login. Utilisé pour l'instance dédiée recovery
    /// (`rate_limiter_recovery`, 5 req / 15 min / blocage 30 min hardcodés) afin
    /// de ne pas muter un `Config` cloné juste pour surcharger 3 champs.
    pub fn with_thresholds(
        max_attempts: u32,
        window: std::time::Duration,
        block_duration: std::time::Duration,
    ) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            max_attempts,
            window,
            block_duration,
        }
    }

    /// Nettoyage lazy (sous lock) : supprime les entrées dont le blocage ET la
    /// fenêtre sont tous deux expirés.
    fn purge_expired(&self, map: &mut HashMap<K, AttemptRecord>, now: Instant) {
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
    }

    /// Vérification du blocage et du seuil (sous lock) — logique partagée entre
    /// [`Self::check_rate_limit`] et [`Self::check_and_record`].
    fn check_locked(
        &self,
        map: &HashMap<K, AttemptRecord>,
        key: K,
        now: Instant,
    ) -> Result<(), RateLimitReject> {
        let record = match map.get(&key) {
            Some(r) => r,
            None => return Ok(()),
        };

        // Vérifier le blocage actif
        if let Some(blocked_until) = record.blocked_until
            && now < blocked_until
        {
            let remaining = blocked_until.duration_since(now);
            return Err(RateLimitReject {
                retry_after_secs: remaining.as_secs().max(1),
            });
        }

        // Compter les tentatives dans la fenêtre. Pass 2 17-4c ECH2-M1 :
        // `Instant` est monotonic-depuis-boot — `now - window` paniquerait si la
        // machine a démarré il y a moins de `window` (réaliste : autostart
        // Docker post-reboot NAS). Fenêtre pas encore entamée → toutes les
        // tentatives comptent.
        let window_start = now.checked_sub(self.window);
        let recent_count = record
            .attempts
            .iter()
            .filter(|t| window_start.is_none_or(|ws| **t >= ws))
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

    /// Enregistrement d'une tentative (sous lock) — logique partagée entre
    /// [`Self::record_failed_attempt`] et [`Self::check_and_record`].
    fn record_locked(&self, map: &mut HashMap<K, AttemptRecord>, key: K, now: Instant) {
        let record = map.entry(key).or_insert_with(|| AttemptRecord {
            attempts: Vec::new(),
            blocked_until: None,
        });

        // Purger les tentatives hors fenêtre (cf. note `checked_sub` dans
        // `check_locked` — pas de soustraction directe d'`Instant`).
        let window_start = now.checked_sub(self.window);
        record
            .attempts
            .retain(|t| window_start.is_none_or(|ws| *t >= ws));

        record.attempts.push(now);

        if record.attempts.len() as u32 >= self.max_attempts {
            record.blocked_until = Some(now + self.block_duration);
        }
    }

    /// Vérifie si l'IP est autorisée à tenter un login.
    ///
    /// Effectue d'abord un nettoyage lazy des entrées expirées,
    /// puis vérifie le blocage et le seuil de tentatives.
    ///
    /// Retourne `Ok(())` si autorisé, `Err(RateLimitReject)` si bloqué.
    pub fn check_rate_limit(&self, key: K) -> Result<(), RateLimitReject> {
        let mut map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let now = Instant::now();
        self.purge_expired(&mut map, now);
        self.check_locked(&map, key, now)
    }

    /// Enregistre une tentative de login échouée pour une IP.
    ///
    /// Si le seuil est atteint, l'IP est bloquée pour `block_duration`.
    pub fn record_failed_attempt(&self, key: K) {
        let mut map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let now = Instant::now();
        self.record_locked(&mut map, key, now);
    }

    /// Story 17-4c Pass 1 (P8) — « check puis record » **atomique** sous un seul
    /// verrou, pour la sémantique always-200 du recovery (chaque requête
    /// consomme un slot, jamais `reset`). Ferme la fenêtre entre
    /// `check_rate_limit` et `record_failed_attempt` où un burst concurrent de
    /// la même IP pouvait dépasser le seuil. Une requête rejetée (IP bloquée)
    /// ne consomme PAS de slot — identique à la séquence deux-appels remplacée.
    pub fn check_and_record(&self, key: K) -> Result<(), RateLimitReject> {
        let mut map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let now = Instant::now();
        self.purge_expired(&mut map, now);
        self.check_locked(&map, key, now)?;
        self.record_locked(&mut map, key, now);
        Ok(())
    }

    /// Story 21-5b — nombre de slots restants dans la fenêtre pour une clé, **sans
    /// consommer** (pré-check de capacité avant un envoi par lot, pour éviter un blocage
    /// à mi-course). Retourne `0` si la clé est actuellement bloquée. Calcul
    /// `max_attempts - recent_count` (borné ≥ 0 via `saturating_sub`).
    pub fn remaining_slots(&self, key: K) -> u32 {
        let mut map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let now = Instant::now();
        self.purge_expired(&mut map, now);
        let record = match map.get(&key) {
            Some(r) => r,
            None => return self.max_attempts,
        };
        if let Some(blocked_until) = record.blocked_until
            && now < blocked_until
        {
            return 0;
        }
        let window_start = now.checked_sub(self.window);
        let recent_count = record
            .attempts
            .iter()
            .filter(|t| window_start.is_none_or(|ws| **t >= ws))
            .count() as u32;
        self.max_attempts.saturating_sub(recent_count)
    }

    /// Réinitialise le compteur pour une IP après un login réussi.
    pub fn reset(&self, key: K) {
        let mut map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        map.remove(&key);
    }

    /// Story 17-4e (DE-2) — vide TOUTES les entrées (toutes IPs, blocages compris).
    ///
    /// Support de test UNIQUEMENT : appelé par le `seed_handler` `/_test/seed`
    /// (lui-même monté seulement si `config.test_mode`) pour que chaque spec
    /// E2E reparte avec des limiters vierges — sans ça, le budget recovery
    /// (5 req / 15 min / IP, partagé forgot+reset) rend les re-runs locaux
    /// flaky. Aucun appelant en production.
    pub fn clear_all(&self) {
        let mut map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        map.clear();
    }
}

// Clone is needed because AppState derives Clone.
// The inner Mutex is wrapped in an Arc at the AppState level.
impl<K> std::fmt::Debug for RateLimiter<K> {
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
        config.rate_limit_window = chrono::TimeDelta::seconds(window_secs as i64);
        config.rate_limit_block_duration = chrono::TimeDelta::seconds(block_secs as i64);
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
    fn with_thresholds_builds_independent_limiter() {
        use std::time::Duration;
        // Story 17-4c — instance recovery construite sans Config (seuils explicites).
        let rl =
            RateLimiter::with_thresholds(5, Duration::from_secs(900), Duration::from_secs(1800));
        let ip: IpAddr = "10.0.1.1".parse().unwrap();

        // Sémantique always-200 du flux forgot-password : on `record` à chaque
        // requête (jamais `reset`). Au 5e enregistrement, l'IP est bloquée.
        for _ in 0..4 {
            assert!(rl.check_rate_limit(ip).is_ok());
            rl.record_failed_attempt(ip);
        }
        // 4 slots consommés sur 5 → toujours autorisé.
        assert!(rl.check_rate_limit(ip).is_ok());
        rl.record_failed_attempt(ip);
        // 5e slot consommé → bloqué.
        let reject = rl.check_rate_limit(ip).unwrap_err();
        assert!(reject.retry_after_secs > 0);
    }

    #[test]
    fn check_and_record_atomic_blocks_at_threshold() {
        use std::time::Duration;
        // Story 17-4c Pass 1 (P8) — variante atomique : mêmes seuils que la
        // séquence check→record qu'elle remplace, requêtes 1..=5 passent,
        // la 6e est rejetée (et ne consomme pas de slot supplémentaire).
        let rl =
            RateLimiter::with_thresholds(5, Duration::from_secs(900), Duration::from_secs(1800));
        let ip: IpAddr = "10.0.2.1".parse().unwrap();

        for i in 1..=5 {
            assert!(
                rl.check_and_record(ip).is_ok(),
                "requête {i}/5 doit passer (slot consommé)"
            );
        }
        let reject = rl.check_and_record(ip).unwrap_err();
        assert!(reject.retry_after_secs > 0, "6e requête doit être bloquée");
        // Une IP distincte n'est pas affectée.
        let other: IpAddr = "10.0.2.2".parse().unwrap();
        assert!(rl.check_and_record(other).is_ok());
    }

    #[test]
    fn remaining_slots_decrements_and_zeroes_when_blocked() {
        use std::time::Duration;
        // Story 21-5b — pré-check de capacité pour le lot.
        let rl =
            RateLimiter::with_thresholds(3, Duration::from_secs(900), Duration::from_secs(1800));
        let ip: IpAddr = "10.0.5.1".parse().unwrap();
        assert_eq!(rl.remaining_slots(ip), 3, "clé inconnue → max_attempts");
        assert!(rl.check_and_record(ip).is_ok());
        assert_eq!(rl.remaining_slots(ip), 2, "1 consommé");
        assert!(rl.check_and_record(ip).is_ok());
        assert!(rl.check_and_record(ip).is_ok());
        assert_eq!(rl.remaining_slots(ip), 0, "seuil atteint → 0");
        assert!(rl.check_and_record(ip).is_err(), "bloqué");
        assert_eq!(rl.remaining_slots(ip), 0, "bloqué → 0 slot");
    }

    #[test]
    fn clear_all_unblocks_a_blocked_ip() {
        use std::time::Duration;
        // Story 17-4e (DE-2) — une IP bloquée redevient autorisée après clear_all.
        let rl =
            RateLimiter::with_thresholds(2, Duration::from_secs(900), Duration::from_secs(1800));
        let ip: IpAddr = "10.0.4.1".parse().unwrap();
        assert!(rl.check_and_record(ip).is_ok());
        assert!(rl.check_and_record(ip).is_ok());
        assert!(
            rl.check_and_record(ip).is_err(),
            "3e requête bloquée (seuil 2)"
        );

        rl.clear_all();
        assert!(
            rl.check_and_record(ip).is_ok(),
            "après clear_all, l'IP repart avec un compteur vierge"
        );
    }

    #[test]
    fn check_and_record_window_larger_than_uptime_does_not_panic() {
        use std::time::Duration;
        // Pass 3 17-4c — fenêtre immense ⇒ `now.checked_sub(window)` = `None`,
        // équivalent d'une machine bootée depuis moins de `window` (ECH2-M1).
        // Ne doit ni paniquer ni rejeter à tort : toutes les tentatives comptent
        // (branche `is_none_or` exercée dans check_locked ET record_locked).
        let rl = RateLimiter::with_thresholds(
            2,
            Duration::from_secs(u64::MAX / 4),
            Duration::from_secs(60),
        );
        let ip: IpAddr = "10.0.3.1".parse().unwrap();

        assert!(rl.check_and_record(ip).is_ok());
        assert!(rl.check_and_record(ip).is_ok());
        // Seuil 2 atteint → 3e requête bloquée (sans panique d'`Instant`).
        let reject = rl.check_and_record(ip).unwrap_err();
        assert!(reject.retry_after_secs > 0);
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
