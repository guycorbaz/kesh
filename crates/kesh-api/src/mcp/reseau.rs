//! Le filtre d'adresse, les mandataires de confiance et l'`Origin` — des
//! fonctions pures (AC-a7 à AC-a10, C-32-6-4 à C-32-6-7).
//!
//! - **Le client se déduit du pair TCP.** `X-Forwarded-For` n'est lu que si le
//!   pair est un mandataire **déclaré** de la société ; toutes ses lignes,
//!   coupées aux virgules, sont lues **de droite à gauche**, les mandataires
//!   sautés ; la première entrée qui n'en est pas un est le client. `Forwarded`
//!   et `X-Real-IP` ne sont **jamais** lus.
//! - **Toute adresse est normalisée** (`IpAddr::to_canonical` : une IPv4
//!   mappée `::ffff:a.b.c.d` devient IPv4), et les réseaux stockés aussi
//!   (`::ffff:172.16.0.0/108` devient `172.16.0.0/12`) — la 32-5c stocke une
//!   adresse mappée telle quelle, en `/128`.
//! - **Une entrée stockée invalide** de `allowed_networks` est ignorée (jamais
//!   élargissante) ; une entrée invalide de `trusted_proxies` **écarte la
//!   société**, quel que soit l'état du filtre (fail closed : l'ignorer ferait
//!   prendre le mandataire pour le client).
//! - **L'`Origin`** se normalise par la même règle que l'écriture des
//!   origines admises (`util::origine_canonique`), des deux côtés.
//!
//! ⚠️ Ce que le filtre ne garantit pas : il voit le pair que voit Kesh, non le
//! client réel — derrière un mandataire non déclaré ou la passerelle de
//! Docker, tous les clients paraissent venir d'une adresse privée, admise par
//! défaut.

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use axum::http::HeaderMap;
use ipnet::{IpNet, Ipv4Net};
use kesh_db::entities::McpServerSettings;

/// Normalise un réseau stocké : un réseau IPv6 inclus dans `::ffff:0:0/96`
/// devient le réseau IPv4 de préfixe `p − 96`.
pub fn reseau_canonique(reseau: IpNet) -> IpNet {
    if let IpNet::V6(v6) = reseau
        && v6.prefix_len() >= 96
        && let Some(v4) = v6.network().to_ipv4_mapped()
        && let Ok(net) = Ipv4Net::new(v4, v6.prefix_len() - 96)
    {
        return IpNet::V4(net);
    }
    reseau
}

/// Analyse une entrée stockée : une adresse nue (`/32` ou `/128`) ou un
/// réseau, normalisés ; `None` si l'entrée ne s'analyse pas.
pub fn analyser_reseau(entree: &str) -> Option<IpNet> {
    let entree = entree.trim();
    let reseau = match entree.parse::<IpAddr>() {
        Ok(adresse) => IpNet::from(adresse),
        Err(_) => entree.parse::<IpNet>().ok()?,
    };
    Some(reseau_canonique(reseau))
}

fn contient(reseaux: &[IpNet], adresse: IpAddr) -> bool {
    reseaux.iter().any(|r| r.contains(&adresse))
}

/// Le client d'une requête, vu d'un réglage dont les mandataires de confiance
/// sont `mandataires` (AC-a7). `None` : **aucun client** (refus) — une valeur
/// d'en-tête non ASCII, ou une entrée qui n'est pas une adresse IP nue.
pub fn client(pair: IpAddr, en_tetes: &HeaderMap, mandataires: &[IpNet]) -> Option<IpAddr> {
    let pair = pair.to_canonical();
    if !contient(mandataires, pair) {
        return Some(pair);
    }
    let mut entrees: Vec<IpAddr> = Vec::new();
    for valeur in en_tetes.get_all("x-forwarded-for") {
        let texte = valeur.to_str().ok()?;
        for morceau in texte.split(',') {
            let adresse = morceau.trim().parse::<IpAddr>().ok()?;
            entrees.push(adresse.to_canonical());
        }
    }
    if entrees.is_empty() {
        return Some(pair);
    }
    Some(
        entrees
            .iter()
            .rev()
            .copied()
            .find(|a| !contient(mandataires, *a))
            .unwrap_or(entrees[0]),
    )
}

/// Ce que décide un réglage pour une requête.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// La société sert ce client.
    Admis,
    /// Le client n'est pas admis par le filtre.
    Refuse,
    /// Une entrée de `trusted_proxies` ne s'analyse pas : la société est
    /// écartée (fail closed).
    Ecartee,
}

/// La décision d'un réglage `s` pour le pair `pair` (AC-a8). Journalise,
/// sans état partagé et sans la valeur, l'entrée ignorée de `allowed_networks`
/// (`debug!`) et l'exclusion pour `trusted_proxies` (`error!`).
pub fn decider(s: &McpServerSettings, pair: IpAddr, en_tetes: &HeaderMap) -> Decision {
    let mut mandataires = Vec::with_capacity(s.trusted_proxies.len());
    for entree in &s.trusted_proxies {
        match analyser_reseau(entree) {
            Some(r) => mandataires.push(r),
            None => {
                tracing::error!(
                    company_id = s.company_id,
                    "serveur MCP : une entrée de trusted_proxies ne s'analyse pas, la société \
                     n'est pas servie"
                );
                return Decision::Ecartee;
            }
        }
    }
    if !s.address_filter_enabled {
        return Decision::Admis;
    }
    let Some(client) = client(pair, en_tetes, &mandataires) else {
        return Decision::Refuse;
    };
    let mut admis = false;
    for entree in &s.allowed_networks {
        match analyser_reseau(entree) {
            Some(r) => admis |= r.contains(&client),
            None => tracing::debug!(
                company_id = s.company_id,
                "serveur MCP : une entrée de allowed_networks ne s'analyse pas, ignorée"
            ),
        }
    }
    if admis {
        Decision::Admis
    } else {
        Decision::Refuse
    }
}

/// Les origines propres de Kesh (AC-a10), normalisées : `public_base_url`
/// réduite à son origine, et `http://127.0.0.1:<port>`, `http://localhost:<port>`,
/// `http://[::1]:<port>`. Un chemin de `public_base_url` (installation sous un
/// sous-chemin, `https://h/app`) est retiré avant la normalisation (revue P1,
/// A2) ; un `public_base_url` qui n'est pas une origine `http(s)` (sans schéma,
/// identifiants, schéma autre) n'en donne **aucune**, avec un avertissement.
pub fn origines_propres(port: u16, public_base_url: Option<&str>) -> Arc<[String]> {
    let mut v: Vec<String> = Vec::with_capacity(4);
    if let Some(base) = public_base_url {
        let base = base.trim();
        let base = match base.split_once("://") {
            Some((schema, reste)) => {
                let autorite = reste.split(['/', '?', '#']).next().unwrap_or("");
                &base[..schema.len() + 3 + autorite.len()]
            }
            None => base,
        };
        match crate::util::origine_canonique(base) {
            Some(o) => v.push(o),
            None => tracing::warn!(
                "serveur MCP : KESH_PUBLIC_BASE_URL n'est pas une origine http(s)://hôte[:port] ; \
                 aucune origine propre n'en vient"
            ),
        }
    }
    for hote in [
        IpAddr::V4(Ipv4Addr::LOCALHOST).to_string(),
        "localhost".to_string(),
        "[::1]".to_string(),
    ] {
        if let Some(o) = crate::util::origine_canonique(&format!("http://{hote}:{port}")) {
            v.push(o);
        }
    }
    v.into()
}

/// L'`Origin` reçue est-elle admise par `propres` ∪ `admises` ? Absente →
/// admise ; plusieurs lignes, valeur non ASCII ou qui n'est pas une origine
/// (`null` compris) → refusée. Jamais déduite de `Host`.
pub fn origine_admise<'a>(
    en_tetes: &HeaderMap,
    propres: &[String],
    admises: impl IntoIterator<Item = &'a String>,
) -> bool {
    let mut valeurs = en_tetes.get_all("origin").iter();
    let Some(valeur) = valeurs.next() else {
        return true;
    };
    if valeurs.next().is_some() {
        return false;
    }
    let Some(recue) = valeur
        .to_str()
        .ok()
        .and_then(crate::util::origine_canonique)
    else {
        return false;
    };
    propres.contains(&recue)
        || admises
            .into_iter()
            .any(|o| crate::util::origine_canonique(o).is_some_and(|o| o == recue))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    fn nets(v: &[&str]) -> Vec<IpNet> {
        v.iter().map(|s| analyser_reseau(s).unwrap()).collect()
    }

    fn xff(lignes: &[&str]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for l in lignes {
            h.append("x-forwarded-for", HeaderValue::from_str(l).unwrap());
        }
        h
    }

    #[test]
    fn reseaux_stockes_normalises() {
        assert_eq!(
            analyser_reseau("::ffff:172.18.0.2/128")
                .unwrap()
                .to_string(),
            "172.18.0.2/32"
        );
        assert_eq!(
            analyser_reseau("::ffff:172.16.0.0/108")
                .unwrap()
                .to_string(),
            "172.16.0.0/12"
        );
        assert_eq!(
            analyser_reseau("::ffff:172.18.0.2").unwrap().to_string(),
            "172.18.0.2/32"
        );
        assert_eq!(
            analyser_reseau("192.168.1.10").unwrap().to_string(),
            "192.168.1.10/32"
        );
        assert_eq!(analyser_reseau("fc00::/7").unwrap().to_string(), "fc00::/7");
        assert!(analyser_reseau("pas-un-réseau").is_none());
        assert!(analyser_reseau("").is_none());
    }

    #[test]
    fn client_sans_mandataire_est_le_pair_et_xff_ignore() {
        let h = xff(&["192.168.1.10"]);
        assert_eq!(client(ip("8.8.8.8"), &h, &[]), Some(ip("8.8.8.8")));
        assert_eq!(client(ip("::ffff:8.8.8.8"), &h, &[]), Some(ip("8.8.8.8")));
    }

    #[test]
    fn client_derriere_mandataire_droite_a_gauche() {
        let m = nets(&["172.18.0.2", "172.18.0.3"]);
        let pair = ip("172.18.0.2");
        assert_eq!(
            client(pair, &xff(&["192.168.1.20"]), &m),
            Some(ip("192.168.1.20"))
        );
        assert_eq!(
            client(pair, &xff(&["192.168.1.20, 8.8.8.8"]), &m),
            Some(ip("8.8.8.8"))
        );
        assert_eq!(
            client(pair, &xff(&["192.168.1.20, 172.18.0.3"]), &m),
            Some(ip("192.168.1.20"))
        );
        assert_eq!(
            client(pair, &HeaderMap::new(), &m),
            Some(pair),
            "en-tête absent"
        );
        assert_eq!(
            client(pair, &xff(&["172.18.0.3, 172.18.0.2"]), &m),
            Some(ip("172.18.0.3")),
            "toutes de confiance : la plus à gauche"
        );
        assert_eq!(
            client(pair, &xff(&["192.168.1.20", "8.8.8.8"]), &m),
            Some(ip("8.8.8.8"))
        );
        assert_eq!(
            client(pair, &xff(&["8.8.8.8", "172.18.0.2"]), &m),
            Some(ip("8.8.8.8"))
        );
        assert_eq!(
            client(pair, &xff(&["::ffff:8.8.8.8"]), &m),
            Some(ip("8.8.8.8")),
            "entrée mappée normalisée"
        );
        for invalide in ["unknown", "8.8.8.8:443", "[::1]", "", "192.168.1.20,"] {
            assert_eq!(client(pair, &xff(&[invalide]), &m), None, "{invalide:?}");
        }
        let mut h = HeaderMap::new();
        h.append(
            "x-forwarded-for",
            HeaderValue::from_bytes("192.168.1.2\u{e9}".as_bytes()).unwrap(),
        );
        assert_eq!(client(pair, &h, &m), None, "non ASCII");
    }

    #[test]
    fn origines_normalisees() {
        let p = origines_propres(80, Some("https://kesh.example:443"));
        assert_eq!(
            p.to_vec(),
            vec![
                "https://kesh.example".to_string(),
                "http://127.0.0.1".to_string(),
                "http://localhost".to_string(),
                "http://[::1]".to_string(),
            ]
        );
        assert_eq!(origines_propres(3000, Some("kesh.example")).len(), 3);
        // Un sous-chemin est retiré : l'origine propre reste (revue P1, A2).
        for base in [
            "https://kesh.example/app",
            "https://kesh.example:443/app/?x#y",
        ] {
            assert_eq!(
                origines_propres(80, Some(base))[0],
                "https://kesh.example",
                "{base}"
            );
        }
        assert_eq!(
            origines_propres(80, Some("https://moi@kesh.example/app")).len(),
            3,
            "identifiants : aucune origine propre"
        );
        // Lignes multiples : refusées (revue P1, E4).
        let mut deux = HeaderMap::new();
        deux.append("origin", HeaderValue::from_static("http://localhost"));
        deux.append("origin", HeaderValue::from_static("http://localhost"));
        assert!(
            !origine_admise(&deux, &p, &Vec::new()),
            "deux lignes Origin"
        );
        let mut h = HeaderMap::new();
        assert!(origine_admise(&h, &p, &Vec::new()), "absente");
        for (o, admise) in [
            ("http://localhost", true),
            ("HTTP://LOCALHOST:80", true),
            ("http://localhost:3000", false),
            ("https://evil.example", false),
            ("null", false),
            ("http://kesh.example:443", false),
            ("https://kesh.example", true),
        ] {
            h.insert("origin", HeaderValue::from_str(o).unwrap());
            assert_eq!(origine_admise(&h, &p, &Vec::new()), admise, "{o}");
        }
        h.insert("origin", HeaderValue::from_static("https://autre.example"));
        let stockees = vec!["https://autre.example:443".to_string()];
        assert!(origine_admise(&h, &p, &stockees));
    }
}
