# Story v0174-session-1 : la 0.17.4, côté serveur — le jeton de renouvellement n'est plus stocké qu'en empreinte (#890), le paramètre ignoré des tests retiré (#896), les LOW serveur de la 0.17.3 (#898, #899)

Status: ready-for-dev

*Sous-fiche de `v0174-session.md` (contexte, cause lue au code par issue, triage des LOW, issues demandées), qui fait foi pour
ce qu'elle porte. Branche `story/v0174-session`, worktree `/home/gcorbaz/devel/kesh-0174s`, port **3177**, bases
`kesh_v0174s` / `kesh_e2e_v0174s`. Registre `v0174-choix-autonomes-v0174s.md` : **C-v0174s-2 à 9** s'appliquent ici.
Numéros de ligne de `7bd4d5c0`. Se développe **avant** la `v0174-session-2` ; aucune dépendance de code entre elles.*

## Story

En tant qu'Administrateur, je veux qu'une lecture de la base, d'un dump SQL ou d'une sauvegarde `.keshbackup` ne donne
aucun jeton de session utilisable ; en tant que développeur, je veux que la configuration de test ne porte plus de
paramètre ignoré.

## Contrainte

Production réelle (0.17.3) : **aucune migration** (89, inchangé) ; la 0.17.3 redémarre sur une base passée par la 0.17.4
(C-v0174s-7) ; une sauvegarde 0.17.3 s'importe. Un dépôt est modifié : **gate complet en cours de boucle** (`CLAUDE.md`,
exception `kesh-db`). Code édité par l'outil Edit seulement.

## Conception

### 1. L'empreinte (C-v0174s-2, C-v0174s-5)

Dans `crates/kesh-db/src/repositories/refresh_tokens.rs` :

```rust
/// Empreinte d'un jeton de renouvellement, seule valeur stockée (#890).
///
/// Les 16 premiers octets de SHA-256(jeton), en hexadécimal minuscule, dont le
/// 13ᵉ chiffre — celui de la version d'un UUID — est forcé à `0`, mis en forme
/// `8-4-4-4-12` : la valeur satisfait `chk_refresh_tokens_token_format` (aucune
/// migration), et ne ressemble jamais à un UUID v4 (`4` à cette place) — ce qui
/// distingue une ligne d'avant la 0.17.4 (`LEGACY_CLEAR_TOKEN_PREDICATE`).
/// 124 bits d'empreinte pour 122 bits d'aléa : rien à gagner d'un hachage lent
/// (C-v0174s-5).
pub fn token_digest(clear: &str) -> String
```

- `sha2 = "0.10"` ajouté à `[dependencies]` de `crates/kesh-db/Cargo.toml`, avec un commentaire (story, motif) sur le
  modèle des voisins ; même version que `kesh-api` : **`Cargo.lock` ne gagne aucune crate** (à vérifier au diff : seule
  la liste de dépendances de `kesh-db` change).
- Constante `LEGACY_CLEAR_TOKEN_PREDICATE: &str = "SUBSTRING(token, 15, 1) <> '0'"` (une ligne qui ne porte pas une
  empreinte), employée par les deux effacements (§ 3), jamais réécrite ailleurs (DRY).

### 2. Le dépôt hache, les appelants ne changent pas (C-v0174s-3, C-v0174s-4)

Chaque fonction qui reçoit la valeur d'un jeton lie `token_digest(valeur)` au lieu de la valeur : `create` (`:30-33`,
`.bind(token_digest(&new.token))`), `find_active_by_token` (`:83-87`), `find_by_token_include_revoked` (`:98-102`),
`revoke_by_token` (`:112-122`), `set_idle_deadline` (`:168-179`), `was_rotated_within` (`:191-200`). Signatures
**inchangées** ; les **neuf** appels de `routes/auth.rs` (huit : `:309`, `:361`, `:432`, `:449`, `:500`, `:509`, `:618`, `:745`) et
`routes/setup.rs` (un : `:293`) **inchangés** (recompté à la validation P1, R7). Recherche par égalité sur
`uq_refresh_tokens_token` (C-v0174s-4). En-tête du module : un paragraphe « Stockage » (seule l'empreinte est en base ;
la colonne s'appelle encore `token` et son commentaire dit encore `'UUID v4 opaque'`, corrigés à la prochaine migration
de la table — issue demandée, fiche parente).

**L'entité** (`entities/refresh_token.rs`) : le champ `token` de `RefreshToken` devient `token_digest`
(`#[sqlx(rename = "token")]`), doc « empreinte, jamais le jeton » ; son `Debug` masque toujours la valeur ;
`NewRefreshToken.token` reste le **jeton clair** (doc : « le dépôt n'en stocke que l'empreinte »). Le paragraphe
`:9-12` (« la story 1.6 ajoutera une colonne `token_hash` ») est remplacé par l'état de la 0.17.4. Sonde au Dev Agent
Record : `git grep -nE "\.token\b" -- crates/kesh-api/src crates/kesh-db/src` ne rend, à `7bd4d5c0`, que deux lignes sans
lecteur du champ renommé — `routes/auth.rs:1141` (`req.token.trim()`, jeton de réinitialisation) et
`refresh_tokens.rs:32` (`new.token`, la liaison remplacée) ; après le correctif, la première seule.

### 3. Les jetons d'avant : invalidés, puis effacés (C-v0174s-6)

- **Invalidés par construction** : un cookie 0.17.3 porte un UUID dont l'empreinte ne correspond à aucune ligne ;
  `refresh` rend `401 INVALID_REFRESH_TOKEN` (« unknown token », `routes/auth.rs:434-439`), le battement 401, la
  déconnexion n'a rien à révoquer (et pose quand même les cookies expirés). **Une reconnexion**, une fois.
- **Effacés au démarrage** : `delete_expired_and_revoked` (`:228-245`, appelée par `main.rs:232`) efface aussi les lignes
  `LEGACY_CLEAR_TOKEN_PREDICATE` — `WHERE (revoked_at IS NOT NULL AND revoked_at < ?) OR (expires_at < ?) OR (<prédicat>)`.
  Sa doc et le message de `main.rs:237` (« expired/revoked tokens removed ») disent « expirés, révoqués ou d'avant la
  0.17.4 ». Aucun autre changement de `main.rs`.
- **Effacés à la restauration complète** : `pub async fn delete_clear_tokens_in_tx(tx) -> Result<u64, DbError>`
  (`DELETE FROM refresh_tokens WHERE <prédicat>`), appelée dans `routes/admin.rs` au point 5-quinquies (`:487-497`),
  **juste avant** `revoke_all_in_tx` — donc après la garde de comptage (`:443-447`) et avant le refus « aucun compte
  Admin » (`:505-516`) : un refus ultérieur annule aussi l'effacement. Le commentaire du point 5-quinquies et le
  doc-comment de `full_import` (`:180-189`, fonction `:190`) le disent. Une sauvegarde 0.17.3 importée n'y replante ainsi aucun jeton clair ;
  une sauvegarde 0.17.4 ne porte que des empreintes, toutes révoquées par la ligne suivante, comme en 0.17.3.

### 4. Ce qui ne change pas

Le cookie (UUID v4, `routes/auth.rs:307`, `:506`, `:615` ; `setup.rs:292`), le corps de `RefreshResponse` (le jeton
neuf, clair, au client), la rotation, la grâce, la détection de vol, l'export de souveraineté (table déjà exclue,
`exports/global.rs:214-217`), le navigateur. `bootstrap.rs` (`revoke_all_for_user`, par utilisateur) et ses tests
(`:798-860`, UUID insérés à la main puis révoqués par utilisateur) restent verts sans modification.

### 5. #896 — retrait du paramètre ignoré (C-v0174s-8)

`Config::from_fields_for_test` (`config.rs:448-…`) perd `_refresh_inactivity_ignored: TimeDelta` (`:455`) et son doc
(`:441-447`) ; les **85** appels (80 fichiers : `crates/kesh-api/src/config.rs:2947`, `:2966`,
`src/middleware/auth.rs:260`, et les fichiers de `crates/kesh-api/tests/`) perdent leur septième argument, par
l'outil Edit. **Contrôle obligatoire** (lecture seule, script de lecture permis) : pour chaque appel, la ligne retirée
est le septième argument de la version `7bd4d5c0`, et la suite des autres arguments est identique ; résultat (85/85)
au Dev Agent Record. Aucun test ne change de sens.

### 6. LOW serveur de #898 et #899 (fiche parente, § *Triage*)

1. **S-B2/S-E2** — test `session_idle_e2e.rs` : jeton `encode_session` dont `idle_exp = Utc::now().timestamp()` ⇒ `GET
   /api/v1/auth/me` **401**. **S-A8** — commentaire à `routes/auth.rs:48` et `:743` : `.trunc_subsecs(3)` aligne la valeur
   écrite sur la précision de `DATETIME(3)` quel que soit le mode d'arrondi du serveur (`TIME_ROUND_FRACTIONAL`) ; sa
   mutation est équivalente sous le réglage par défaut — déclaré.
2. **S-E3** — `auth_e2e.rs` : deux cas sur le modèle de `refresh_replay_after_rotation_revokes_all` (`:1068`) :
   `revoked_at = NOW(3) - INTERVAL 29 SECOND` ⇒ grâce (401, **aucune** autre ligne révoquée) ; `31 SECOND` ⇒ vol (toutes
   révoquées, motif `theft_detected`).
3. **S-E4 (c)** — texte seulement (§ *Manuel*).
4. **S-E5** — `refresh` : `installation_settings::get` (`routes/auth.rs:527-529`) déplacé **avant** l'étape 8
   (`:500`, rotation) ; `change_password` (validation P1, R6) : `new_session_deadline` (`:616`) déplacé **avant**
   l'étape 3 (`users::update_password`, `:602`) et donc avant la révocation (`:606`) ; `setup` : `new_session_deadline` (`setup.rs:283`) déplacé **avant** la section critique
   (`:152`, étape 4) — la valeur calculée est alors l'échéance de la session neuve, à quelques millisecondes près
   (l'Administrateur est créé juste après), ce qui reste dans la tolérance de ± 2 s des tests de l'AC-3 (h) de la 0.17.3 ;
   le dire au commentaire. Pas de test neuf (une panne de lecture ne s'injecte pas) : vérifié par lecture, ligne citée au
   Dev Agent Record.
5. **S-E6** — `routes/auth.rs:481` : `token.expires_at < now` → `<=` (aligné sur le middleware, `now >= idle_exp`).
   Le second volet de l'issue (battement à `idleMs ≥ N × 60 000`, réponse `idleExpiresIn: 0`) n'est **pas** nommé dans
   l'écart (d) du manuel : décision écrite (fiche parente, triage) — le navigateur ne l'envoie jamais.
6. **S-B3/S-E4 (a)** — doc-comment de `set_idle_deadline` : « pas de garde de monotonie, délibérément : un abaissement
   du réglage raccourcit l'échéance à la prochaine activité (C-v0172s-38) ; deux battements réordonnés n'en déplacent
   l'échéance que de l'écart (a) ».
7. **S2-B1** — commentaires `middleware/auth.rs:134`, `:137` et `auth/api_key.rs:31` : `jwt::decode_session`.
8. **S2-B2** — `routes/session_settings.rs:95` : nom d'opération `"session_settings::update_session_settings"` (patron
   de `kesh-db/src/retry.rs` : le nom de la route) ; le témoin du test de course qui cite l'ancien nom est aligné
   (`git grep -n "installation_settings::update_session_idle_minutes" -- crates`, chaque site trié).
9. **S2-B3** — `session_idle_e2e.rs:465` `heartbeat_refuses_api_keys` : assertion du code `API_KEY_MANAGEMENT_FORBIDDEN`
   dans le corps.
10. **S2-E2** — doc-comment d'`activity` (`routes/auth.rs:697-702`) : « corps illisible ⇒ 400 » est jugé **avant** le
    cookie (session + corps illisible + pas de cookie ⇒ 400) ; l'ordre écrit est corrigé.
11. **S2-A1** — `tests/audit_route_registry.rs:159` (et tout autre site du même décompte : `git grep -nE
    "vingt-deux|\b22\b" -- crates/kesh-api/tests/audit_route_registry.rs`, trié) : recompter les routes
    `SansEcritureAuJournal` rejouées depuis la table du registre et écrire le nombre juste, `update_session_settings`
    compris.
12. **S2-A3 et N2-A2** — `_bmad-output/implementation-artifacts/v0172-session.md` : deux annotations **datées**
    (2026-11-0x, « revue de la 0.17.4, #899 ») — l'une après l'annotation S-A1 de § *Conception* 3 (« huit heures » était
    juste ; c'est « ne déconnecte personne » qui était faux), l'autre au Dev Agent Record (« dont deux dans des boucles » :
    quatre, dans trois boucles ; `scripts/recette-e-v0172s.mjs` vit sous `kesh-gate-logs/v0172s/scripts/`). Aucune entrée
    passée n'est réécrite.

## Acceptance Criteria

**AC-1 — L'empreinte, au vecteur.** Test unitaire (`refresh_tokens.rs`, `#[cfg(test)] mod tests`) :
`token_digest("00000000-0000-4000-8000-000000000000") == "db8055e0-e030-0d5a-016b-ec4dc338d698"` (vecteur **recalculé**
au développement, indépendamment du code, par `printf '%s' … | sha256sum` ; la commande et sa sortie au Dev Agent
Record) ; pour 1 000 UUID v4 tirés au hasard : l'empreinte satisfait la regex de `chk_refresh_tokens_token_format`, porte
`0` à la position 15, et diffère de l'entrée. **Rouge avant correctif** (fonction absente).

**Assistant d'empreinte indépendant, unique** (validation P1, R2 c) : `pub fn independent_refresh_digest(clear: &str) ->
String` dans `crates/kesh-api/tests/common/mod.rs` (`#![allow(dead_code)]` déjà posé, `:9`), qui recalcule l'empreinte
**par `sha2` directement**, sans appeler `token_digest` ; AC-2, AC-5 et AC-6 l'emploient (aucune troisième copie de
l'algorithme). Son propre test : il rend le vecteur de l'AC-1. Les fichiers qui n'incluent pas encore `mod common;`
l'ajoutent.

**AC-2 — Seule l'empreinte est en base.** `crates/kesh-api/tests/auth_e2e.rs` (cas neuf
`login_stores_only_the_token_digest`) : après `POST /api/v1/auth/login`, avec `c` le cookie `kesh_refresh_token` reçu :
`SELECT COUNT(*) FROM refresh_tokens WHERE token = c` = **0** ; la ligne de l'utilisateur porte une valeur égale à
l'empreinte de `c` **calculée par `independent_refresh_digest`** (sans appeler `token_digest` — sinon la mutation
« empreinte = identité » passerait) ; même contrôle après `POST /auth/refresh` (ligne neuve), après `PUT
/auth/password` et après `POST /api/v1/setup/admin` (`setup_admin_e2e.rs`). `sha2` est une dépendance normale de `kesh-api`
(`Cargo.toml:55`), donc visible de ses tests d'intégration. **Rouge avant correctif.**

**AC-3 — Les recherches passent par l'empreinte.** Les tests existants du cycle restent verts, adaptés là où ils lisent
ou écrivent la colonne à la main (§ *Tests existants qui changent*) : connexion, renouvellement et rotation, grâce,
vol, déconnexion, battement, changement de mot de passe, désactivation. Un jeton **forgé à la main** dont la ligne porte
la valeur claire (forme 0.17.3) n'ouvre rien : `refresh` ⇒ 401 `INVALID_REFRESH_TOKEN` (code identique aux autres refus,
`all_refresh_error_codes_are_identical` reste vert), battement ⇒ 401, déconnexion ⇒ **204** (`routes/auth.rs:371`, `StatusCode::NO_CONTENT` ; « 200 » écrit à la fiche était faux,
R8) sans révoquer cette ligne.
**Rouge avant correctif** : le cas « forme 0.17.3 » (aujourd'hui le jeton clair est trouvé, 200).

**AC-4 — Effacement au démarrage.** `crates/kesh-db/tests/refresh_tokens_repository.rs` : une ligne forme 0.17.3
(UUID v4 inséré à la main, active, échéance future) et une ligne créée par `create` (active, échéance future) ;
`delete_expired_and_revoked(pool, now - 7 j)` rend 1, efface la première, garde la seconde. **Et la rétention reste
prouvée** (validation P1, R1) : `cleanup_removes_old_tokens` (`auth_e2e.rs:1524`) pose ses deux lignes anciennes **en
empreintes** (aujourd'hui des UUID clairs, `:1530`, `:1537`, que le prédicat neuf effacerait quel que soit leur âge — le
test deviendrait muet sur les clauses d'âge), et gagne deux lignes d'empreinte qui **doivent survivre** — une récente non
révoquée, une révoquée depuis moins de 7 jours — : `removed == 2`, et deux lignes restantes. **Rouge avant correctif** : le
cas du dépôt (la ligne forme 0.17.3 survit aujourd'hui) ; `cleanup_removes_old_tokens` adapté est vert avant et après,
c'est M13 qui le prouve.

**AC-5 — Effacement à l'import, sauvegarde 0.17.3 importable.** `admin_full_import_e2e.rs` : (a) une archive dont la
table `refresh_tokens` porte des UUID clairs (forme 0.17.3 : lignes posées à la main avant l'export, actives et
révoquées) s'importe (200) ; après l'import, `SELECT COUNT(*) FROM refresh_tokens WHERE SUBSTRING(token, 15, 1) <> '0'`
= 0 ; les lignes à empreinte de l'archive sont présentes et **toutes** révoquées ; (b) le cas existant
`v0172s_full_import_revokes_every_session_and_keeps_idle_setting` (`:5183`) reste vrai, son assistant `v0172s_refresh_row`
(`:5149`) posant désormais l'**empreinte** et rendant le jeton clair (les recherches `WHERE token = ?` liant
l'empreinte) ; (c) `v0172s_failed_import_after_revocation_point_keeps_sessions` (`:5268`) : une ligne forme 0.17.3 de la
destination **survit** à l'import refusé (la transaction annule aussi l'effacement). (d) La fixture
`tests/fixtures/keshbackup-0.14.0-format1.keshbackup` s'importe toujours (les tests qui l'emploient restent verts).
**Rouge avant correctif** : (a) seulement. (c) est **vert avant correctif** (le rollback existe en 0.17.3, `:5287`) — il
n'est pas déclaré rouge (validation P1, R3) ; il tue la mutation M14 (effacement exécuté sur le `pool` au lieu de la
transaction).

**AC-6 — La sauvegarde n'emporte aucun jeton utilisable.** `admin_backup_e2e.rs` (ou `admin_full_import_e2e.rs`) :
après une connexion, l'export `.keshbackup` ; la table `refresh_tokens` de l'archive (NDJSON) ne contient **pas** la
valeur du cookie reçu, et contient son empreinte (entrée `data/refresh_tokens.ndjson`, `admin_backup/export.rs:26`,
`:258`). **Rouge avant correctif.**

**AC-7 — #896.** `from_fields_for_test` n'a plus de paramètre d'inactivité ; contrôle 85/85 (§ *Conception* 5) au Dev
Agent Record ; sonde `git grep -n "_refresh_inactivity_ignored\|refresh_inactivity" -- crates` vide ; build et suite
verts. Pas de rouge avant correctif (dette sans comportement).

**AC-8 — Contrats tenus.** `crates/kesh-api/tests/configuration_transmise.rs`, `audit_route_registry.rs`,
`admin_pat_denied_e2e.rs`, `textes_coherents.rs`, `test_schema_guard.rs`, `migrations_upgrade_path.rs`,
`migrations_fresh_install.rs` verts ; `ls crates/kesh-db/migrations/*.sql | wc -l` = 89 ; `migrations.sha384`,
`docs/migrations-idempotence-audit.md` et le squash **inchangés** (`git diff --stat 7bd4d5c0 -- crates/kesh-db/migrations
crates/kesh-db/test-schema docs/migrations-idempotence-audit.md` vide).

**AC-9 — LOW serveur.** Les points 1 à 12 du § *Conception* 6, chacun constaté au Dev Agent Record (ligne lue citée,
test nommé) ; S-B2 et S-E3 : leurs mutations jouées (M10, M11).

**AC-10 — Les textes disent vrai.** § *Manuel*, § *CHANGELOG*, § *README et spécifications* ; PDF contrôlé aplati.

## Tests existants qui changent (inventaire à l'étape 0, chaque site trié au Dev Agent Record)

Sites qui lisent ou écrivent `refresh_tokens.token` à la main avec la valeur d'un cookie, ou insèrent une ligne puis
présentent sa valeur : ils lient désormais l'**empreinte** (`kesh_db::repositories::refresh_tokens::token_digest`) —
sauf ceux qui veulent justement une ligne forme 0.17.3 (AC-3, AC-4, AC-5).

- `crates/kesh-api/tests/auth_e2e.rs` : `logout_revokes_refresh_token` (`:636`), `refresh_rotates_token` (`:1058`),
  `refresh_replay_after_rotation_revokes_all` (`:1089`), `refresh_with_expired_token_returns_401` (`:1178`),
  `cleanup_removes_old_tokens` (`:1529`, `:1542` — ses **deux** lignes, toutes deux anciennes, passent en empreintes, et
  deux lignes à garder s'y ajoutent : AC-4 ; « ses lignes à garder » écrit à la fiche était faux, il n'y en avait aucune,
  R1), `all_refresh_error_codes_are_identical` (`:1676`, `:1714`),
  `refresh_replay_within_grace_does_not_revoke_all` (`:1771`) ;
- `crates/kesh-api/tests/auth_cookies_e2e.rs` : `logout_invalidates_cookie` (`:384`, `:470`) ;
- `crates/kesh-api/tests/session_idle_e2e.rs` : `row_expires_at` (`:203`), `refresh_copies_the_deadline` (`:309`),
  `heartbeat_refuses_foreign_revoked_or_expired_rows` (`:434`, `:444`), `nothing_is_accepted_past_the_deadline` (`:519`) ;
- `crates/kesh-api/tests/admin_full_import_e2e.rs` : `:5149-5165`, `:5208`, `:5225`, `:5295` (AC-5) ;
- `crates/kesh-db/tests/refresh_tokens_repository.rs` : `create_and_find_active_by_token` (`:74`, `created.token ==
  token_value` → `created.token_digest == token_digest(&token_value)`), et
  **`create_with_invalid_token_format_fails_check_constraint` (`:233`)** — `create` hache toute entrée, si bien que la
  valeur `"not-a-uuid"` n'atteint plus la contrainte : le test passe à un `INSERT` brut (la contrainte reste la garde de
  la colonne), et un cas neuf établit que `create("not-a-uuid")` **réussit** et stocke une empreinte valide.
- Restent **verts sans modification** (à confirmer au gate) : `auth/bootstrap.rs` (tests `:798-860`),
  `migrations_upgrade_path.rs:1734-1769`, `password_recovery_e2e.rs:362`, `onboarding_audit_e2e.rs:2118`, `:2208`.

**Garde contre l'adaptation oubliée** (validation P1, R2) : un `UPDATE … WHERE token = ?` lié à la valeur claire touche
0 ligne **en silence**, et plusieurs tests resteraient verts pour la mauvaise raison (`refresh_with_expired_token_returns_401`
et `all_refresh_error_codes_are_identical` rendent le même 401 pour un jeton « inconnu » que pour un jeton expiré ou
révoqué). Donc : **chaque `UPDATE`/`DELETE` de test adapté asserte `rows_affected() == 1`** (ou le nombre attendu), et
les mutations M15 et M16 prouvent que les fixtures adaptées atteignent le chemin visé.

Sonde de l'inventaire : `git grep -nE "refresh_tokens[^\"]*(WHERE|SET)[^\"]*token|INSERT INTO refresh_tokens|token = \?"
-- crates/kesh-api/tests crates/kesh-db/tests crates/kesh-api/src` (filtrer `password_reset`) ; un site absent de la
liste ci-dessus est trié au Dev Agent Record.

## Tests et mutations

| # | Mutation | Tuée par |
|---|---|---|
| M1 | `token_digest` rend `clear.to_string()` (**stockage en clair**) | AC-1 (vecteur), AC-2 (empreinte calculée par le test), AC-6 |
| M2 | le `0` forcé retiré | AC-1 (vecteur ; sur 1 000 tirages, ≈ 937 n'ont plus `0` à la position 15 — « ≈ 16 » écrit à la fiche était faux, R7) |
| M3 | `create` lie `&new.token` (clair) | AC-2 ; AC-3 (renouvellement : ligne introuvable) |
| M4 | `find_by_token_include_revoked` lie la valeur claire | AC-3 (`refresh_rotates_token` : 401) |
| M5 | `revoke_by_token` lie la valeur claire | `logout_revokes_refresh_token` |
| M6 | `set_idle_deadline` lie la valeur claire | tests du battement (`session_idle_e2e.rs`, 401) |
| M7 | `was_rotated_within` lie la valeur claire | `refresh_replay_within_grace_does_not_revoke_all` (vol au lieu de grâce) |
| M8 | prédicat inversé (`= '0'`) | AC-4, AC-5 (a) |
| M9 | `delete_clear_tokens_in_tx` retiré de l'import | AC-5 (a) |
| M10 | middleware `now >= deadline` → `>` | AC-9 (S-B2) — tuée sauf franchissement de seconde, déclaré |
| M11 | `ROTATION_GRACE_SECS` 30 → 5, puis 30 → 60 | AC-9 (S-E3, 29 s puis 31 s) |
| M12 | renouvellement `<=` → `<` | **équivalente en pratique** (égalité à la milliseconde), déclarée sans test |
| M13 | `delete_expired_and_revoked` : clause `expires_at < ?`, puis clause `revoked_at < ?`, retirée (deux mutations) | `cleanup_removes_old_tokens` adapté (AC-4) |
| M14 | `delete_clear_tokens_in_tx` exécutée sur le `pool` au lieu de la transaction | AC-5 (c) (la ligne de la destination disparaît malgré le refus) |
| M15 | contrôle d'expiration de `refresh` retiré (`routes/auth.rs:481`) | `refresh_with_expired_token_returns_401` adapté (200 au lieu de 401) |
| M16 | contrôle de révocation de `refresh` retiré (bloc `token.revoked_at.is_some()`, `:443-477`) | `all_refresh_error_codes_are_identical` adapté (200 au lieu de 401) |
| M17 | `find_active_by_token` lie la valeur claire (aucun appelant de production) | `create_and_find_active_by_token` (R11) |

Chaque mutation : motif vérifié unique, jouée sous les deux verrous (consignes communes), restaurée par
`git checkout -- <fichier>` **puis `touch`** ; résultat (rouge attendu, test qui rougit) au Dev Agent Record. Rouges
avant correctif constatés et déclarés : AC-1, AC-2, le cas « forme 0.17.3 » de l'AC-3, le cas du dépôt de l'AC-4,
AC-5 (a), AC-6.

## Tâches

1. **Étape 0** — relire les lignes citées (code `7bd4d5c0`) ; recalculer le vecteur ; inventaire des tests (§ *Tests
   existants*) ; recenser les appels de `from_fields_for_test`. Écart ⇒ Change Log.
2. Tests rouges (ceux que § *Tests et mutations* déclare rouges), constatés.
3. `kesh-db` : `sha2`, `token_digest`, prédicat, liaisons, effacements, entité, docs. Gate complet.
4. `kesh-api` : `admin.rs` (effacement à l'import), commentaires (S-A8, S2-B1, S2-E2), ordre de lecture (S-E5, trois
   sites), `<=`
   (S-E6), nom d'opération (S2-B2), `main.rs:237` (message). Tests adaptés et neufs.
5. #896 (§ *Conception* 5), contrôle 85/85.
6. LOW 1 à 12 ; annotations de `v0172-session.md`.
7. Textes (§ *Manuel*, *CHANGELOG*, *README*). **Le PDF du manuel d'administration est régénéré par cette sous-fiche**
   (`make fr` sous les deux verrous) et contrôlé aplati : son AC-10 se vérifie sans la 2, qui le régénérera encore
   (validation P1, R9).
8. Mutations M1 à M17.
9. Gate complet (`scripts/test-fast.sh --ci` sous les deux verrous), Vitest **non** requis (aucun fichier frontend),
   **E2E complet** au dernier commit de code (le format du stockage touche la connexion de toutes les specs), recette
   (§ *Recette*).

## Recette (base jetable `kesh_v0174s`, port 3177)

(a) **Montée depuis une 0.17.3** : base migrée par le worktree, une session ouverte sur la base par un binaire **0.17.3**
(`git worktree` du tag `v0.17.3`, ou ligne forme 0.17.3 posée à la main si le binaire n'est pas constructible sous le
budget mémoire — le dire) ; démarrage de la 0.17.4 : `SELECT COUNT(*) FROM refresh_tokens WHERE SUBSTRING(token, 15, 1)
<> '0'` = 0 ; l'onglet resté ouvert revient à la connexion **à son premier battement** après la mise à jour — au plus
15 s après sa première activité (`activity_with` : `set_idle_deadline` rend `false`, 401, `routes/auth.rs:745-756` ; le
renouvellement avec le même cookie échoue, `/login?reason=session_expired`) — et non « à l'échéance de son jeton
d'accès », comme l'écrivait la fiche (validation P1, R10 et F15) ; reconnexion, puis la ligne neuve porte une empreinte. (b) **Retour** : redémarrer la 0.17.3 sur cette base —
elle démarre (aucune migration inconnue), et demande une reconnexion. (c) Export `.keshbackup` après connexion :
`unzip -p <archive> data/refresh_tokens.ndjson` ne contient pas le cookie. `KESH_JWT_EXPIRY_MINUTES` ≥ la
durée réglée pour tout contrôle 401 au `curl` (leçon N-A5). Chiffres et heures au Dev Agent Record.

## Manuel (`docs/manual/fr/admin-manual.tex` et PDF)

- `:3027` → « Le refresh token n'est **jamais stocké en clair** : la table \texttt{refresh\_tokens} n'en garde qu'une
  empreinte (SHA-256), si bien qu'une lecture de la base ou d'une sauvegarde ne permet pas de reprendre une session ;
  30 jours est la durée du \emph{cookie} ; la ligne porte l'échéance d'inactivité de la session. »
- **Les trois sites « jetons de session »** (validation P1, F1 et R5 ; `grep -nE "jetons? de session"
  docs/manual/fr/admin-manual.tex`, à refaire à l'étape 0) :
  - `:1665` (répétition de restauration) → « (mots de passe hachés, **empreintes** des jetons de session --- depuis la
    0.17.4 ; une sauvegarde antérieure porte encore des jetons en clair : \S\ref{sec:backup-ui-keshbackup}) » ;
  - `:1991` (dumps du NAS) → « condensés des mots de passe, **empreintes** des jetons de session (depuis la 0.17.4 ; les
    dumps **antérieurs** à la mise à jour portent encore des jetons en clair) » ;
  - `:2263` → « … ainsi que les **empreintes** des jetons de session, inutilisables sans le jeton lui-même (depuis la
    0.17.4). Une sauvegarde, une base ou un dump **antérieurs** à la 0.17.4 portent encore les jetons **en clair**, et le
    manuel des versions jusqu'à la 0.17.2 annonçait à tort un hachage : traitez-les comme le secret qu'ils sont. »
- `:3039`, écart (a) : ajouter « … ; un battement qui croise un renouvellement de la même session peut être perdu de
  même, et rattrapé au battement suivant ».
- **Sous-section neuve « Passer à la 0.17.4 »** (`\subsubsection{Passer à la 0.17.4}\label{sec:maj-0-17-4}`, avant
  `sec:maj-0-17-3`, `:2513`, patron de la 0.17.2, `:2527`) — validation P1, F2 et R9 ; **la rédaction commune aux trois
  stories de la 0.17.4 revient à l'intégration**, cette sous-fiche y écrit sa part (fiche parente, § *Textes*, points 1 et
  2) : aucune migration, retour à la 0.17.3 **sans restauration** au prix d'une reconnexion, sauvegarde complète
  recommandée ; chaque utilisateur se reconnecte **une fois**, un onglet resté ouvert revenant à la connexion **à sa
  prochaine activité** (enregistrer avant de mettre à jour) ; jetons d'avant effacés au premier démarrage et à l'import
  d'une sauvegarde antérieure ; bases, dumps et sauvegardes antérieurs encore porteurs de jetons clairs.
- § *Durée d'inactivité*, paragraphe **Mise à jour** (`:3051`) : une phrase de renvoi — « Pour la 0.17.4 (jetons de
  session en empreinte, une reconnexion), voir \S\ref{sec:maj-0-17-4}. » ; le reste du paragraphe, qui vaut pour la
  0.17.3, est daté (« À la mise à jour vers la 0.17.3 : … »).
- § *Dépannage* (`:3765`, « Cause ») — validation P1, F11 (5) : ajouter « juste après une mise à jour vers la 0.17.4 :
  la reconnexion unique (\S\ref{sec:maj-0-17-4}) ».
- **Contrôle du PDF par motifs locaux** (validation P1, F14 : « empreinte » sort déjà ailleurs, une sonde globale réussirait
  avant correctif) : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` greppé sur « tel quel (non
  haché » (absent), « n'en garde qu'une empreinte » (présent), « empreintes des jetons de session » (**trois**
  occurrences), « jetons de session --- : d'où » et « mots de passe hachés, jetons de session : » (absents), « Passer à la
  0.17.4 » (présent) ; motifs et sorties au Dev Agent Record.

## README et spécifications

- `crates/kesh-api/README.md:162` : « UUID v4 opaque ; **seule son empreinte** (SHA-256 tronqué, mis en forme dans la
  colonne `token`) est persistée » ; `:180` (limitation 1) : retirée ou réécrite au passé (« corrigé en 0.17.4, #890 »),
  numérotation des limitations suivantes tenue.
- `docs/kesh-specifications.txt:2693` : « … stocké en base **sous forme d'empreinte** (depuis la 0.17.4) ».
- `admin_backup/mod.rs:3-4` : « (dont `users.password_hash` et les empreintes des `refresh_tokens`) ».

## CHANGELOG (`## [0.17.4] — Non publié`, en tête, au-dessus de `[0.17.3]`)

- **Sécurité** — « **Les jetons de session ne sont plus stockés en clair** ([#890](…/issues/890)) : la base, un dump SQL
  ou une sauvegarde `.keshbackup` n'en gardent qu'une empreinte, qui ne permet pas de reprendre une session. *À la mise à
  jour, chaque utilisateur se reconnecte une fois* ; les jetons d'avant sont effacés au premier démarrage et à l'import
  d'une sauvegarde antérieure. Aucune migration : revenir à la 0.17.3 reste possible sans la sauvegarde (avec, là aussi,
  une reconnexion) ; la sauvegarde complète avant la mise à jour reste recommandée. **Le manuel des versions jusqu'à la
  0.17.2 annonçait à tort un hachage** : une base, un dump ou une sauvegarde antérieurs à la 0.17.4 contiennent les jetons
  en clair — gardez-les comme le secret qu'ils sont. »
- **Corrigé** — « Une panne de **lecture du réglage d'inactivité** ne fait plus perdre la session au renouvellement (le
  réglage est lu avant que le jeton ne soit remplacé), ne laisse plus un mot de passe changé sans session neuve, ni
  l'assistant d'installation en erreur après avoir créé l'Administrateur (refs #898). » — formulation bornée à la
  lecture de N, seul cas fermé (validation P1, R6).
- Avertissement d'en-tête de section, sur le modèle de la 0.17.3 : « La 0.17.4 ne porte **aucune migration**. »
  (la `v0174-session-2` et les autres stories de la 0.17.4 complètent la section ; union à l'intégration).

## Dev Notes

- Le précédent de hachage est `password_reset_tokens` (`repositories/password_reset_tokens.rs:1-10`) et `api_keys`
  (`auth/api_key.rs:98-109`) ; ici le hachage est **dans** le dépôt (C-v0174s-3), à dessein.
- `uq_refresh_tokens_token` reste l'index de recherche ; aucune collision à craindre sur 124 bits.
- `RefreshToken`/`NewRefreshToken` ne dérivent ni `Debug` (impl manuelle masquante) ni `Serialize` : à garder.
- Ne pas toucher `migrations/`, `test-schema/`, `migrations.sha384` (AC-8) — P8.
- Données réelles : aucune ; les UUID de test sont tirés au hasard ou fixes et synthétiques.

## Change Log

- 2026-11-05 — Sous-fiche créée (Opus 5.5, autonomie), avec la parente. Prochaine : validation P1 (Sonnet, R et F).
- 2026-11-05 — Validation P1 remédiée (détail au Change Log de la parente) : R1, R2 (`rows_affected`, assistant
  `independent_refresh_digest`, M13 à M17), R3 (AC-5 (c) non rouge, M14), R6 (troisième site S-E5), R7, R8 (204), R9
  (PDF régénéré ici, sous-section « Passer à la 0.17.4 »), R10/F15 (reconnexion au premier battement, entrée
  `data/refresh_tokens.ndjson`), R11 (M17, décision S-E6), F1/R5 (trois sites), F11 (3, 5), F14 (motifs locaux).
  Prochaine : validation P2 (Opus, complète).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
