//! Démarrage face à MariaDB — Story 15-13a (#551).
//!
//! Lance le binaire réel : c'est le seul moyen d'établir (1) que
//! l'avertissement sur un mot de passe publié est bien émis par le chemin de
//! démarrage (`Config::from_env` appelé par `main`), et (2) que l'extraction
//! du numéro d'erreur MariaDB (`numero_erreur_mariadb`) fonctionne sur une
//! vraie `sqlx::Error` — `MySqlDatabaseError` n'a pas de constructeur public.

#[path = "common/binaire.rs"]
mod binaire;

use kesh_api::config::{AVERTISSEMENT_MOT_DE_PASSE_PUBLIE, INDICE_ACCES_REFUSE};

/// Secret JWT valide (≥ 32 octets, ni `change-me` ni gabarit) : sans lui,
/// `Config::from_env` refuse AVANT toute connexion.
const SECRET: &str = "0123456789abcdef0123456789abcdef-15-13a";

/// Test 8 de la fiche — AC 5 (câblage). Hôte `.invalid` : échec de résolution
/// immédiat (mesuré au T0 : 16 ms), non rejoué par `sqlx`.
#[test]
fn mot_de_passe_publie_avertit_au_demarrage() {
    let sortie = binaire::lancer_binaire(&[
        (
            "DATABASE_URL",
            "mysql://kesh:kesh_dev@kesh-15-13.invalid/kesh",
        ),
        ("KESH_JWT_SECRET", SECRET),
        ("KESH_HOST", "127.0.0.1"),
    ]);
    let texte = binaire::texte(&sortie);
    assert!(
        !sortie.status.success(),
        "échec de connexion attendu : {texte}"
    );
    // Assertion de montage : le binaire est allé jusqu'à la connexion.
    assert!(
        texte.contains("Base de données indisponible"),
        "la connexion doit avoir été tentée : {texte}"
    );
    assert!(
        texte.contains(AVERTISSEMENT_MOT_DE_PASSE_PUBLIE),
        "avertissement « mot de passe publié » attendu : {texte}"
    );
}

/// Test 10 de la fiche — AC 6 a-b. La `DATABASE_URL` des tests (obligatoire)
/// avec un mauvais mot de passe : MariaDB répond 1045, le message porte
/// l'erreur d'origine et l'indice, sans citer le mot de passe.
#[test]
fn mauvais_mot_de_passe_donne_l_indice() {
    let base = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL obligatoire pour ce test (MariaDB de test) — pas de saut silencieux");
    let mut url = url::Url::parse(&base).expect("DATABASE_URL des tests lisible");
    let mauvais = "mauvais-15-13";
    url.set_password(Some(mauvais))
        .expect("mot de passe posable");
    let sortie = binaire::lancer_binaire(&[
        ("DATABASE_URL", url.as_str()),
        ("KESH_JWT_SECRET", SECRET),
        ("KESH_HOST", "127.0.0.1"),
    ]);
    let texte = binaire::texte(&sortie);
    assert!(
        !sortie.status.success(),
        "échec de connexion attendu : {texte}"
    );
    // Assertion de montage : la configuration a été acceptée.
    assert!(
        !texte.contains("Erreur de configuration"),
        "le binaire doit avoir tenté la connexion : {texte}"
    );
    assert!(
        texte.contains("1045"),
        "erreur d'origine (1045) attendue : {texte}"
    );
    assert!(
        texte.contains(INDICE_ACCES_REFUSE),
        "indice du refus 1045 attendu : {texte}"
    );
    assert!(
        !texte.contains(mauvais),
        "la sortie ne doit pas citer le mot de passe : {texte}"
    );
}
