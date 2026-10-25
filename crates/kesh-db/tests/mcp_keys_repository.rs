//! Story 32-5a (refs #723) — les tables `mcp_keys` et `mcp_server_settings` :
//! leurs contraintes (AC-a2, AC-a3) et leurs dépôts (AC-a5).
//!
//! Les tests de contrainte écrivent en SQL brut, **sans** passer par les types
//! Rust (qui refusent déjà un ensemble de portées vide) : c'est MariaDB qui doit
//! refuser, et chaque test l'attend de lui.

use chrono::NaiveDate;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{
    Language, McpScope, McpScopes, McpServerSettings, McpServerSettingsUpdate, NewCompany,
    NewMcpKey, NewUser, OrgType, Role, SendValidation, StoredMcpScopes,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::{companies, mcp_keys, mcp_server_settings, users};
use sqlx::MySqlPool;

// ---------------------------------------------------------------------------
// Montage
// ---------------------------------------------------------------------------

async fn company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("société")
    .id
}

async fn user(pool: &MySqlPool, company_id: i64, username: &str, role: Role) -> i64 {
    users::create(
        pool,
        NewUser {
            username: username.into(),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$dGVzdA$dGVzdA".into(),
            role,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("utilisateur")
    .id
}

fn hash(c: char) -> String {
    std::iter::repeat_n(c, 64).collect()
}

fn far_future() -> chrono::NaiveDateTime {
    NaiveDate::from_ymd_opt(2999, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
}

fn far_past() -> chrono::NaiveDateTime {
    NaiveDate::from_ymd_opt(2000, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
}

async fn new_key(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    key_hash: String,
    scopes: &[McpScope],
    expires_at: Option<chrono::NaiveDateTime>,
) -> kesh_db::entities::McpKey {
    let mut tx = pool.begin().await.unwrap();
    let key = mcp_keys::create_in_tx(
        &mut tx,
        NewMcpKey {
            company_id,
            created_by_user_id: user_id,
            name: "agent".into(),
            key_hash,
            scopes: McpScopes::new(scopes.iter().copied()).unwrap(),
            send_validation: SendValidation::Required,
            expires_at,
        },
    )
    .await
    .expect("create_in_tx");
    tx.commit().await.unwrap();
    key
}

/// INSERT brut dans `mcp_keys` (sans `send_validation` si `None` : le défaut
/// de la colonne s'applique) ; rend le message d'erreur de MariaDB, `None` si
/// l'insertion passe.
async fn raw_key_insert(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    name: &str,
    key_hash: &str,
    scopes: &str,
    send_validation: Option<&str>,
) -> Option<String> {
    let sql = format!(
        "INSERT INTO mcp_keys (company_id, created_by_user_id, name, key_hash, scopes{}) \
         VALUES (?, ?, ?, ?, ?{})",
        if send_validation.is_some() {
            ", send_validation"
        } else {
            ""
        },
        if send_validation.is_some() { ", ?" } else { "" },
    );
    let query = sqlx::query(&sql)
        .bind(company_id)
        .bind(user_id)
        .bind(name)
        .bind(key_hash)
        .bind(scopes);
    let query = match send_validation {
        Some(v) => query.bind(v),
        None => query,
    };
    match query.execute(pool).await {
        Ok(_) => None,
        Err(sqlx::Error::Database(e)) => Some(e.message().to_string()),
        Err(e) => panic!("erreur inattendue : {e}"),
    }
}

/// Comme [`raw_key_insert`], mais l'insertion DOIT être refusée par MariaDB ;
/// rend le message, où les tests cherchent le nom de la contrainte.
async fn raw_key_insert_err(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    name: &str,
    key_hash: &str,
    scopes: &str,
    send_validation: Option<&str>,
) -> String {
    raw_key_insert(
        pool,
        company_id,
        user_id,
        name,
        key_hash,
        scopes,
        send_validation,
    )
    .await
    .expect("l'insertion devait être refusée par MariaDB")
}

// ---------------------------------------------------------------------------
// AC-a2 — contraintes de `mcp_keys`
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn mcp_keys_accepts_a_valid_row_and_defaults_to_required(pool: MySqlPool) {
    let c = company(&pool, "A").await;
    let u = user(&pool, c, "admin-a", Role::Admin).await;
    assert_eq!(
        raw_key_insert(&pool, c, u, "k", &hash('a'), r#"["books:read"]"#, None).await,
        None
    );
    let (sv, version): (String, i32) =
        sqlx::query_as("SELECT send_validation, version FROM mcp_keys WHERE key_hash = ?")
            .bind(hash('a'))
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(sv, "required", "défaut de send_validation");
    assert_eq!(version, 1);
}

#[sqlx::test(migrations = "./test-schema")]
async fn mcp_keys_refuses_empty_or_blank_name(pool: MySqlPool) {
    let c = company(&pool, "A").await;
    let u = user(&pool, c, "admin-a", Role::Admin).await;
    // `TRIM` ne retire que les espaces : la tabulation et le saut de ligne
    // sont les cas qui ont imposé le `REGEXP` de la migration.
    for (i, name) in ["", "   ", "\t", "\t\n "].into_iter().enumerate() {
        let msg = raw_key_insert_err(
            &pool,
            c,
            u,
            name,
            &hash(char::from(b'a' + i as u8)),
            r#"["books:read"]"#,
            None,
        )
        .await;
        assert!(
            msg.contains("chk_mcp_keys_name_nonempty"),
            "nom {name:?} : {msg}"
        );
    }
    // Un nom qui porte un caractère visible passe, blancs autour compris.
    assert_eq!(
        raw_key_insert(
            &pool,
            c,
            u,
            "\tagent ",
            &hash('z'),
            r#"["books:read"]"#,
            None
        )
        .await,
        None
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn mcp_keys_refuses_a_duplicate_key_hash(pool: MySqlPool) {
    let c = company(&pool, "A").await;
    let u = user(&pool, c, "admin-a", Role::Admin).await;
    assert_eq!(
        raw_key_insert(&pool, c, u, "k1", &hash('a'), r#"["books:read"]"#, None).await,
        None
    );
    let msg = raw_key_insert_err(&pool, c, u, "k2", &hash('a'), r#"["books:read"]"#, None).await;
    assert!(msg.contains("uq_mcp_keys_key_hash"), "{msg}");
}

#[sqlx::test(migrations = "./test-schema")]
async fn mcp_keys_refuses_scopes_that_are_not_a_non_empty_json_array(pool: MySqlPool) {
    let c = company(&pool, "A").await;
    let u = user(&pool, c, "admin-a", Role::Admin).await;
    // JSON invalide : la contrainte implicite de la colonne JSON (`mcp_keys.scopes`).
    // Revue P1 (E5, B4) : le nom de la contrainte implicite, et non « scopes »
    // seul, que le message de `chk_mcp_keys_scopes` satisferait aussi.
    let msg = raw_key_insert_err(&pool, c, u, "k", &hash('a'), "pas du json", None).await;
    assert!(msg.contains("`mcp_keys.scopes`"), "non JSON : {msg}");
    // JSON valide mais pas un tableau, puis tableau vide : `chk_mcp_keys_scopes`.
    for (i, scopes) in [r#"{}"#, r#""books:read""#, "[]"].into_iter().enumerate() {
        let msg = raw_key_insert_err(
            &pool,
            c,
            u,
            "k",
            &hash(char::from(b'b' + i as u8)),
            scopes,
            None,
        )
        .await;
        assert!(msg.contains("chk_mcp_keys_scopes"), "{scopes} : {msg}");
    }
}

#[sqlx::test(migrations = "./test-schema")]
async fn mcp_keys_refuses_an_unknown_send_validation(pool: MySqlPool) {
    let c = company(&pool, "A").await;
    let u = user(&pool, c, "admin-a", Role::Admin).await;
    // La casse et l'espace finale comptent : sans la comparaison binaire sans
    // remplissage du CHECK, `Required`, `EXEMPT` et `required ` passeraient et
    // feraient échouer le décodage strict.
    for (i, v) in ["maybe", "Required", "EXEMPT", "required ", ""]
        .into_iter()
        .enumerate()
    {
        let msg = raw_key_insert_err(
            &pool,
            c,
            u,
            "k",
            &hash(char::from(b'a' + i as u8)),
            r#"["books:read"]"#,
            Some(v),
        )
        .await;
        assert!(
            msg.contains("chk_mcp_keys_send_validation"),
            "{v:?} : {msg}"
        );
    }
    assert_eq!(
        raw_key_insert(
            &pool,
            c,
            u,
            "k",
            &hash('z'),
            r#"["books:read"]"#,
            Some("exempt")
        )
        .await,
        None
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn mcp_keys_foreign_keys_are_enforced_and_restrict(pool: MySqlPool) {
    let c = company(&pool, "A").await;
    let u = user(&pool, c, "admin-a", Role::Admin).await;
    let msg = raw_key_insert_err(
        &pool,
        999_999,
        u,
        "k",
        &hash('a'),
        r#"["books:read"]"#,
        None,
    )
    .await;
    assert!(msg.contains("fk_mcp_keys_company"), "{msg}");
    let msg = raw_key_insert_err(
        &pool,
        c,
        999_999,
        "k",
        &hash('b'),
        r#"["books:read"]"#,
        None,
    )
    .await;
    assert!(msg.contains("fk_mcp_keys_created_by"), "{msg}");

    // RESTRICT : ni le créateur ni la société ne se suppriment tant qu'une clé
    // les référence.
    new_key(&pool, c, u, hash('c'), &[McpScope::BooksRead], None).await;
    let err = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(u)
        .execute(&pool)
        .await
        .expect_err("RESTRICT sur users");
    assert!(err.to_string().contains("fk_mcp_keys_created_by"), "{err}");

    // Revue P1 (B4) : la société aussi. Une société D sans utilisateur, dont la
    // seule référence est une clé (créée par l'administrateur de C) : seul
    // `fk_mcp_keys_company` peut refuser sa suppression.
    let d = company(&pool, "D").await;
    new_key(&pool, d, u, hash('d'), &[McpScope::BooksRead], None).await;
    let err = sqlx::query("DELETE FROM companies WHERE id = ?")
        .bind(d)
        .execute(&pool)
        .await
        .expect_err("RESTRICT sur companies");
    assert!(err.to_string().contains("fk_mcp_keys_company"), "{err}");
}

// ---------------------------------------------------------------------------
// AC-a3 — contraintes de `mcp_server_settings`
// ---------------------------------------------------------------------------

async fn raw_settings_insert(
    pool: &MySqlPool,
    company_id: i64,
    networks: &str,
    proxies: &str,
    origins: &str,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO mcp_server_settings (company_id, allowed_networks, trusted_proxies, \
         allowed_origins) VALUES (?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(networks)
    .bind(proxies)
    .bind(origins)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(|e| match e {
        sqlx::Error::Database(d) => d.message().to_string(),
        other => panic!("erreur inattendue : {other}"),
    })
}

#[sqlx::test(migrations = "./test-schema")]
async fn mcp_server_settings_defaults_and_one_row_per_company(pool: MySqlPool) {
    let c = company(&pool, "A").await;
    raw_settings_insert(&pool, c, "[]", "[]", "[]")
        .await
        .expect("ligne valide");
    let (enabled, filter, version): (bool, bool, i32) = sqlx::query_as(
        "SELECT enabled, address_filter_enabled, version FROM mcp_server_settings \
         WHERE company_id = ?",
    )
    .bind(c)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!enabled, "serveur désactivé par défaut");
    assert!(filter, "filtre d'adresse activé par défaut");
    assert_eq!(version, 1);
    let msg = raw_settings_insert(&pool, c, "[]", "[]", "[]")
        .await
        .expect_err("une seule ligne par société");
    assert!(msg.contains("PRIMARY"), "{msg}");
    let msg = raw_settings_insert(&pool, 999_999, "[]", "[]", "[]")
        .await
        .expect_err("FK companies");
    assert!(msg.contains("fk_mcp_server_settings_company"), "{msg}");
    // Revue P1 (B4) : RESTRICT à la suppression — la société A n'a que sa ligne
    // de réglages pour référence.
    let err = sqlx::query("DELETE FROM companies WHERE id = ?")
        .bind(c)
        .execute(&pool)
        .await
        .expect_err("RESTRICT sur companies");
    assert!(
        err.to_string().contains("fk_mcp_server_settings_company"),
        "{err}"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn mcp_server_settings_lists_must_be_json_arrays(pool: MySqlPool) {
    let c = company(&pool, "A").await;
    for (args, chk) in [
        (("{}", "[]", "[]"), "chk_mcp_settings_networks"),
        (("[]", r#""x""#, "[]"), "chk_mcp_settings_proxies"),
        (("[]", "[]", "1"), "chk_mcp_settings_origins"),
    ] {
        let msg = raw_settings_insert(&pool, c, args.0, args.1, args.2)
            .await
            .expect_err("liste non tableau");
        assert!(msg.contains(chk), "{args:?} : {msg}");
    }
    let msg = raw_settings_insert(&pool, c, "pas du json", "[]", "[]")
        .await
        .expect_err("liste non JSON");
    assert!(
        msg.contains("`mcp_server_settings.allowed_networks`"),
        "{msg}"
    );
}

// ---------------------------------------------------------------------------
// AC-a5 — dépôt `mcp_keys`
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn create_list_and_find_are_scoped_by_company(pool: MySqlPool) {
    let a = company(&pool, "A").await;
    let b = company(&pool, "B").await;
    let ua = user(&pool, a, "admin-a", Role::Admin).await;
    let ub = user(&pool, b, "admin-b", Role::Admin).await;

    let k1 = new_key(
        &pool,
        a,
        ua,
        hash('1'),
        &[McpScope::PromptsWrite, McpScope::BooksRead],
        Some(far_future()),
    )
    .await;
    assert_eq!(k1.company_id, a);
    assert_eq!(k1.created_by_user_id, ua);
    assert_eq!(k1.name, "agent");
    assert_eq!(k1.key_hash, hash('1'));
    assert_eq!(
        k1.scopes.known().iter().copied().collect::<Vec<_>>(),
        vec![McpScope::BooksRead, McpScope::PromptsWrite],
        "portées relues dans l'ordre canonique"
    );
    assert!(k1.scopes.unknown().is_empty());
    assert_eq!(k1.send_validation, SendValidation::Required);
    assert_eq!(k1.expires_at, Some(far_future()));
    assert_eq!(k1.version, 1);
    assert_eq!(k1.last_used_at, None);
    assert_eq!(k1.revoked_at, None);
    // La colonne stocke la forme canonique.
    let stored: String =
        sqlx::query_scalar("SELECT CAST(scopes AS CHAR) FROM mcp_keys WHERE id = ?")
            .bind(k1.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stored, r#"["books:read","prompts:write"]"#);

    let k2 = new_key(&pool, a, ua, hash('2'), &[McpScope::InvoicesRead], None).await;
    let kb = new_key(&pool, b, ub, hash('3'), &[McpScope::BooksRead], None).await;

    let mut tx = pool.begin().await.unwrap();
    mcp_keys::revoke_in_tx(&mut tx, a, k2.id, k2.version)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let actives: Vec<i64> = mcp_keys::list_by_company(&pool, a, false)
        .await
        .unwrap()
        .iter()
        .map(|k| k.id)
        .collect();
    assert_eq!(
        actives,
        vec![k1.id],
        "révoquée exclue, autre société exclue"
    );
    let toutes: Vec<i64> = mcp_keys::list_by_company(&pool, a, true)
        .await
        .unwrap()
        .iter()
        .map(|k| k.id)
        .collect();
    assert_eq!(toutes.len(), 2);
    assert!(toutes.contains(&k1.id) && toutes.contains(&k2.id));
    assert!(!toutes.contains(&kb.id));

    assert_eq!(
        mcp_keys::find_by_id_for_company(&pool, a, k1.id)
            .await
            .unwrap()
            .map(|k| k.id),
        Some(k1.id)
    );
    assert!(
        mcp_keys::find_by_id_for_company(&pool, b, k1.id)
            .await
            .unwrap()
            .is_none(),
        "aucune fuite d'existence vers une autre société (KF-002)"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn create_in_tx_maps_a_duplicate_hash_to_unique_violation(pool: MySqlPool) {
    let a = company(&pool, "A").await;
    let ua = user(&pool, a, "admin-a", Role::Admin).await;
    new_key(&pool, a, ua, hash('1'), &[McpScope::BooksRead], None).await;
    let mut tx = pool.begin().await.unwrap();
    let err = mcp_keys::create_in_tx(
        &mut tx,
        NewMcpKey {
            company_id: a,
            created_by_user_id: ua,
            name: "dup".into(),
            key_hash: hash('1'),
            scopes: McpScopes::new([McpScope::BooksRead]).unwrap(),
            send_validation: SendValidation::Exempt,
            expires_at: None,
        },
    )
    .await
    .expect_err("condensat en double");
    assert!(
        matches!(err, DbError::UniqueConstraintViolation(_)),
        "{err:?}"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn auth_lookup_finds_an_active_key_with_its_creator(pool: MySqlPool) {
    let a = company(&pool, "A").await;
    let ua = user(&pool, a, "admin-a", Role::Admin).await;
    let uc = user(&pool, a, "compta-a", Role::Comptable).await;
    let k = new_key(
        &pool,
        a,
        ua,
        hash('1'),
        &[McpScope::BooksRead, McpScope::ContactsRead],
        Some(far_future()),
    )
    .await;
    let row = mcp_keys::find_active_auth_by_key_hash(&pool, &hash('1'))
        .await
        .unwrap()
        .expect("clé active trouvée");
    assert_eq!(row.mcp_key_id, k.id);
    assert_eq!(row.company_id, a);
    assert_eq!(row.created_by_user_id, ua);
    assert_eq!(StoredMcpScopes::from(row.scopes.clone()), k.scopes);
    assert_eq!(row.send_validation, SendValidation::Required);
    assert_eq!(row.creator_role, Role::Admin);
    assert!(row.creator_active);

    // Le rôle et l'état sont RENDUS, non filtrés : c'est l'authentification
    // (32-5b) qui en décide.
    new_key(&pool, a, uc, hash('2'), &[McpScope::BooksRead], None).await;
    let row = mcp_keys::find_active_auth_by_key_hash(&pool, &hash('2'))
        .await
        .unwrap()
        .expect("clé d'un Comptable rendue");
    assert_eq!(row.creator_role, Role::Comptable);

    assert!(
        mcp_keys::find_active_auth_by_key_hash(&pool, &hash('9'))
            .await
            .unwrap()
            .is_none(),
        "condensat inconnu"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn auth_lookup_ignores_a_revoked_key(pool: MySqlPool) {
    let a = company(&pool, "A").await;
    let ua = user(&pool, a, "admin-a", Role::Admin).await;
    let k = new_key(&pool, a, ua, hash('1'), &[McpScope::BooksRead], None).await;
    let mut tx = pool.begin().await.unwrap();
    mcp_keys::revoke_in_tx(&mut tx, a, k.id, k.version)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert!(
        mcp_keys::find_active_auth_by_key_hash(&pool, &hash('1'))
            .await
            .unwrap()
            .is_none(),
        "clé révoquée : jamais trouvée (mutation Ma4, revoked_at)"
    );
}

/// Revue P1 (E5) — la frontière de l'expiration, au plus près que l'horloge de
/// la base le permet : une clé expirée depuis une seconde n'est pas rendue, une
/// clé qui expire dans une minute l'est. Tue une tolérance glissée dans le
/// filtre (`NOW(3) - INTERVAL …`). ⚠️ Ne distingue PAS `>` de `>=` : il
/// faudrait une clé qui expire exactement à `NOW(3)`, ce que l'horloge de
/// MariaDB ne permet pas de poser — limite déclarée au Change Log.
#[sqlx::test(migrations = "./test-schema")]
async fn auth_lookup_expiry_boundary(pool: MySqlPool) {
    let a = company(&pool, "A").await;
    let ua = user(&pool, a, "admin-a", Role::Admin).await;
    let passee = new_key(&pool, a, ua, hash('1'), &[McpScope::BooksRead], None).await;
    let proche = new_key(&pool, a, ua, hash('2'), &[McpScope::BooksRead], None).await;
    sqlx::query("UPDATE mcp_keys SET expires_at = NOW(3) - INTERVAL 1 SECOND WHERE id = ?")
        .bind(passee.id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE mcp_keys SET expires_at = NOW(3) + INTERVAL 1 MINUTE WHERE id = ?")
        .bind(proche.id)
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        mcp_keys::find_active_auth_by_key_hash(&pool, &hash('1'))
            .await
            .unwrap()
            .is_none(),
        "expirée depuis une seconde : jamais rendue"
    );
    assert!(
        mcp_keys::find_active_auth_by_key_hash(&pool, &hash('2'))
            .await
            .unwrap()
            .is_some(),
        "expire dans une minute : encore rendue"
    );
}

/// Revue P1 (E5) — l'ordre de `list_by_company` : `created_at DESC`, puis
/// `id DESC` à égalité. Horodatages posés à la main (deux égaux).
#[sqlx::test(migrations = "./test-schema")]
async fn list_by_company_orders_by_creation_then_id_descending(pool: MySqlPool) {
    let a = company(&pool, "A").await;
    let ua = user(&pool, a, "admin-a", Role::Admin).await;
    let mut ids = Vec::new();
    for c in ['1', '2', '3'] {
        ids.push(
            new_key(&pool, a, ua, hash(c), &[McpScope::BooksRead], None)
                .await
                .id,
        );
    }
    for (id, at) in [
        (ids[0], "2026-10-20 10:00:00.000"),
        (ids[1], "2026-10-20 09:00:00.000"),
        (ids[2], "2026-10-20 09:00:00.000"),
    ] {
        sqlx::query("UPDATE mcp_keys SET created_at = ? WHERE id = ?")
            .bind(at)
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }
    let ordre: Vec<i64> = mcp_keys::list_by_company(&pool, a, true)
        .await
        .unwrap()
        .iter()
        .map(|k| k.id)
        .collect();
    assert_eq!(
        ordre,
        vec![ids[0], ids[2], ids[1]],
        "la plus récente d'abord, puis l'id le plus grand à égalité"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn auth_lookup_ignores_an_expired_key(pool: MySqlPool) {
    let a = company(&pool, "A").await;
    let ua = user(&pool, a, "admin-a", Role::Admin).await;
    new_key(
        &pool,
        a,
        ua,
        hash('1'),
        &[McpScope::BooksRead],
        Some(far_past()),
    )
    .await;
    assert!(
        mcp_keys::find_active_auth_by_key_hash(&pool, &hash('1'))
            .await
            .unwrap()
            .is_none(),
        "clé expirée : jamais trouvée (mutation Ma4, expires_at)"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn auth_lookup_ignores_an_orphan_key(pool: MySqlPool) {
    // Une clé dont la société a disparu (montage sous FOREIGN_KEY_CHECKS = 0,
    // motif `mount_orphans`) : la jointure à `companies` l'écarte, avant toute
    // réparation (C-32-5-12).
    let a = company(&pool, "A").await;
    let ua = user(&pool, a, "admin-a", Role::Admin).await;
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut *conn)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO mcp_keys (company_id, created_by_user_id, name, key_hash, scopes) \
         VALUES (?, ?, 'orpheline', ?, '[\"books:read\"]')",
    )
    .bind(999_999_i64)
    .bind(ua)
    .bind(hash('1'))
    .execute(&mut *conn)
    .await
    .unwrap();
    sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(&mut *conn)
        .await
        .unwrap();
    drop(conn);
    assert!(
        mcp_keys::find_active_auth_by_key_hash(&pool, &hash('1'))
            .await
            .unwrap()
            .is_none(),
        "clé orpheline : jamais trouvée (mutation Ma4, JOIN companies)"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn revoke_uses_the_optimistic_lock_and_the_company_scope(pool: MySqlPool) {
    let a = company(&pool, "A").await;
    let b = company(&pool, "B").await;
    let ua = user(&pool, a, "admin-a", Role::Admin).await;
    let k = new_key(&pool, a, ua, hash('1'), &[McpScope::BooksRead], None).await;

    // Version périmée.
    let mut tx = pool.begin().await.unwrap();
    let err = mcp_keys::revoke_in_tx(&mut tx, a, k.id, k.version + 1)
        .await
        .expect_err("version périmée");
    assert!(matches!(err, DbError::OptimisticLockConflict));
    // Autre société.
    let err = mcp_keys::revoke_in_tx(&mut tx, b, k.id, k.version)
        .await
        .expect_err("autre société");
    assert!(matches!(err, DbError::OptimisticLockConflict));
    // Bonne version.
    mcp_keys::revoke_in_tx(&mut tx, a, k.id, k.version)
        .await
        .unwrap();
    // Déjà révoquée (avec la version nouvelle).
    let err = mcp_keys::revoke_in_tx(&mut tx, a, k.id, k.version + 1)
        .await
        .expect_err("déjà révoquée");
    assert!(matches!(err, DbError::OptimisticLockConflict));
    tx.commit().await.unwrap();

    let relue = mcp_keys::find_by_id_for_company(&pool, a, k.id)
        .await
        .unwrap()
        .unwrap();
    assert!(relue.revoked_at.is_some());
    assert_eq!(relue.version, k.version + 1);
}

#[sqlx::test(migrations = "./test-schema")]
async fn set_send_validation_uses_the_optimistic_lock(pool: MySqlPool) {
    let a = company(&pool, "A").await;
    let b = company(&pool, "B").await;
    let ua = user(&pool, a, "admin-a", Role::Admin).await;
    let k = new_key(&pool, a, ua, hash('1'), &[McpScope::BooksRead], None).await;

    let mut tx = pool.begin().await.unwrap();
    let v2 =
        mcp_keys::set_send_validation_in_tx(&mut tx, a, k.id, k.version, SendValidation::Exempt)
            .await
            .unwrap();
    assert_eq!(v2, k.version + 1);
    let err =
        mcp_keys::set_send_validation_in_tx(&mut tx, a, k.id, k.version, SendValidation::Required)
            .await
            .expect_err("version périmée");
    assert!(matches!(err, DbError::OptimisticLockConflict));
    let err = mcp_keys::set_send_validation_in_tx(&mut tx, b, k.id, v2, SendValidation::Required)
        .await
        .expect_err("autre société");
    assert!(matches!(err, DbError::OptimisticLockConflict));
    tx.commit().await.unwrap();

    let relue = mcp_keys::find_by_id_for_company(&pool, a, k.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(relue.send_validation, SendValidation::Exempt);
    assert_eq!(relue.version, v2);

    // Une clé révoquée ne change plus de politique.
    let mut tx = pool.begin().await.unwrap();
    mcp_keys::revoke_in_tx(&mut tx, a, k.id, v2).await.unwrap();
    let err =
        mcp_keys::set_send_validation_in_tx(&mut tx, a, k.id, v2 + 1, SendValidation::Required)
            .await
            .expect_err("clé révoquée");
    assert!(matches!(err, DbError::OptimisticLockConflict));
    tx.commit().await.unwrap();
}

#[sqlx::test(migrations = "./test-schema")]
async fn touch_last_used_sets_it_but_never_on_a_revoked_key(pool: MySqlPool) {
    let a = company(&pool, "A").await;
    let ua = user(&pool, a, "admin-a", Role::Admin).await;
    let k1 = new_key(&pool, a, ua, hash('1'), &[McpScope::BooksRead], None).await;
    let k2 = new_key(&pool, a, ua, hash('2'), &[McpScope::BooksRead], None).await;

    mcp_keys::touch_last_used(&pool, &hash('1')).await.unwrap();
    let relue = mcp_keys::find_by_id_for_company(&pool, a, k1.id)
        .await
        .unwrap()
        .unwrap();
    assert!(relue.last_used_at.is_some());

    let mut tx = pool.begin().await.unwrap();
    mcp_keys::revoke_in_tx(&mut tx, a, k2.id, k2.version)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    mcp_keys::touch_last_used(&pool, &hash('2')).await.unwrap();
    let relue = mcp_keys::find_by_id_for_company(&pool, a, k2.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        relue.last_used_at, None,
        "une clé révoquée n'est pas réveillée"
    );
}

/// C-32-5-37 (revue P1, E1) — une portée inconnue en base (une sauvegarde
/// d'une version future, une écriture à la main) est **ignorée** à la lecture :
/// la clé garde ses portées connues, se liste à côté des autres, et
/// s'authentifie avec elles. Tue la mutation « décodage redevenu strict ».
#[sqlx::test(migrations = "./test-schema")]
async fn a_stored_unknown_scope_is_ignored_and_the_known_ones_remain(pool: MySqlPool) {
    let a = company(&pool, "A").await;
    let ua = user(&pool, a, "admin-a", Role::Admin).await;
    let saine = new_key(&pool, a, ua, hash('0'), &[McpScope::InvoicesRead], None).await;
    assert_eq!(
        raw_key_insert(
            &pool,
            a,
            ua,
            "future",
            &hash('1'),
            r#"["books:read","books:write"]"#,
            None
        )
        .await,
        None
    );
    let liste = mcp_keys::list_by_company(&pool, a, true)
        .await
        .expect("une portée inconnue ne fait pas échouer la liste");
    assert_eq!(liste.len(), 2, "la clé saine reste listée à côté");
    let future = liste.iter().find(|k| k.name == "future").unwrap();
    assert_eq!(
        future.scopes.known().iter().copied().collect::<Vec<_>>(),
        vec![McpScope::BooksRead]
    );
    assert_eq!(future.scopes.unknown(), ["books:write"]);
    assert!(liste.iter().any(|k| k.id == saine.id));
    let row = mcp_keys::find_active_auth_by_key_hash(&pool, &hash('1'))
        .await
        .expect("jamais une erreur de base")
        .expect("authentifiée par ses portées connues");
    assert_eq!(
        row.scopes.iter().collect::<Vec<_>>(),
        vec![McpScope::BooksRead]
    );
}

/// C-32-5-37 — une clé sans **aucune** portée connue (inconnues seules, ou un
/// tableau qui ne porte pas de chaînes) se liste, se relit et se révoque, mais
/// n'authentifie jamais — sans erreur de base.
#[sqlx::test(migrations = "./test-schema")]
async fn a_key_without_any_known_scope_is_listed_and_revocable_but_never_authenticates(
    pool: MySqlPool,
) {
    let a = company(&pool, "A").await;
    let ua = user(&pool, a, "admin-a", Role::Admin).await;
    for (c, scopes) in [('1', r#"["books:write"]"#), ('2', "[1, null]")] {
        assert_eq!(
            raw_key_insert(&pool, a, ua, "inconnue", &hash(c), scopes, None).await,
            None
        );
        let k = mcp_keys::list_by_company(&pool, a, false)
            .await
            .expect("liste lisible")
            .into_iter()
            .find(|k| k.key_hash == hash(c))
            .expect("la clé est listée");
        assert!(k.scopes.known().is_empty(), "{scopes}");
        assert_eq!(k.scopes.usable(), None, "{scopes}");
        assert!(
            mcp_keys::find_active_auth_by_key_hash(&pool, &hash(c))
                .await
                .expect("jamais une erreur de base")
                .is_none(),
            "{scopes} : une clé sans portée connue n'authentifie pas"
        );
        let relue = mcp_keys::find_by_id_for_company(&pool, a, k.id)
            .await
            .unwrap()
            .expect("relue par son id");
        let mut tx = pool.begin().await.unwrap();
        mcp_keys::revoke_in_tx(&mut tx, a, relue.id, relue.version)
            .await
            .expect("révocable");
        tx.commit().await.unwrap();
        assert!(
            mcp_keys::find_by_id_for_company(&pool, a, k.id)
                .await
                .unwrap()
                .unwrap()
                .revoked_at
                .is_some()
        );
    }
}

// ---------------------------------------------------------------------------
// AC-a5 — dépôt `mcp_server_settings`
// ---------------------------------------------------------------------------

fn update(enabled: bool, networks: &[&str], origins: &[&str]) -> McpServerSettingsUpdate {
    McpServerSettingsUpdate {
        enabled,
        address_filter_enabled: !enabled,
        allowed_networks: networks.iter().map(|s| s.to_string()).collect(),
        trusted_proxies: vec!["192.168.1.1/32".into()],
        allowed_origins: origins.iter().map(|s| s.to_string()).collect(),
    }
}

#[sqlx::test(migrations = "./test-schema")]
async fn settings_get_writes_nothing_and_defaults_have_version_zero(pool: MySqlPool) {
    let a = company(&pool, "A").await;
    assert_eq!(mcp_server_settings::get(&pool, a).await.unwrap(), None);
    assert_eq!(
        mcp_server_settings::get_or_defaults(&pool, a)
            .await
            .unwrap(),
        McpServerSettings::defaults(a)
    );
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mcp_server_settings")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 0, "aucune ligne écrite par une lecture (C-32-5-8)");
}

#[sqlx::test(migrations = "./test-schema")]
async fn settings_save_inserts_at_version_zero_then_updates_under_lock(pool: MySqlPool) {
    let a = company(&pool, "A").await;
    let b = company(&pool, "B").await;

    let mut tx = pool.begin().await.unwrap();
    let s1 = mcp_server_settings::save_in_tx(
        &mut tx,
        a,
        0,
        &update(
            true,
            &["10.0.0.0/8", "fd00::/8"],
            &["http://nas.local:3000"],
        ),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(s1.version, 1);
    assert!(s1.enabled);
    assert!(!s1.address_filter_enabled);
    assert_eq!(s1.allowed_networks, vec!["10.0.0.0/8", "fd00::/8"]);
    assert_eq!(s1.trusted_proxies, vec!["192.168.1.1/32"]);
    assert_eq!(s1.allowed_origins, vec!["http://nas.local:3000"]);
    assert!(s1.updated_at.is_some());
    assert_eq!(
        mcp_server_settings::get(&pool, a).await.unwrap(),
        Some(s1.clone())
    );

    // version 0 alors qu'une ligne existe : conflit, jamais d'écrasement
    // (mutation Mc4, côté dépôt).
    let mut tx = pool.begin().await.unwrap();
    let err = mcp_server_settings::save_in_tx(&mut tx, a, 0, &update(false, &[], &[]))
        .await
        .expect_err("version 0 sur une ligne existante");
    assert!(matches!(err, DbError::OptimisticLockConflict), "{err:?}");
    drop(tx);
    assert_eq!(
        mcp_server_settings::get(&pool, a).await.unwrap(),
        Some(s1.clone())
    );

    // version ≥ 1 : UPDATE sous verrou.
    let mut tx = pool.begin().await.unwrap();
    let s2 = mcp_server_settings::save_in_tx(&mut tx, a, 1, &update(false, &[], &[]))
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(s2.version, 2);
    assert!(!s2.enabled);
    assert!(s2.allowed_networks.is_empty());

    // Version périmée, négative, ou ligne absente (société B, version 1).
    for (company, version) in [(a, 1), (a, -1), (b, 1)] {
        let mut tx = pool.begin().await.unwrap();
        let err =
            mcp_server_settings::save_in_tx(&mut tx, company, version, &update(true, &[], &[]))
                .await
                .expect_err("conflit attendu");
        assert!(
            matches!(err, DbError::OptimisticLockConflict),
            "société {company}, version {version} : {err:?}"
        );
    }
    assert_eq!(mcp_server_settings::get(&pool, b).await.unwrap(), None);
    assert_eq!(mcp_server_settings::get(&pool, a).await.unwrap(), Some(s2));
}

#[sqlx::test(migrations = "./test-schema")]
async fn settings_with_a_non_string_list_fail_to_decode(pool: MySqlPool) {
    let a = company(&pool, "A").await;
    raw_settings_insert(&pool, a, "[1, 2]", "[]", "[]")
        .await
        .expect("tableau JSON valide pour MariaDB");
    let err = mcp_server_settings::get(&pool, a)
        .await
        .expect_err("liste de nombres refusée au décodage");
    assert!(matches!(err, DbError::Invariant(_)), "{err:?}");
}
