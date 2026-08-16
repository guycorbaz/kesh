//! Story 22-4a (#167) — un jeton PAT n'atteint aucune route d'administration.
//!
//! Deux familles de tests, et elles ne prouvent pas la même chose :
//!
//! - **Les tests de SOURCE** (`#[test]`, sans base) lisent `src/lib.rs` par
//!   `include_str!` et tiennent la **complétude**. Ils rougissent quand une route
//!   admin est ajoutée sans son couple dans [`ADMIN_COUPLES`], ou quand le
//!   montage du routeur cesse de garantir que la couche s'applique. C'est le
//!   rappel automatique qui manquait — l'approche par handler avait été
//!   appliquée 3 fois et oubliée 16.
//! - **Les tests HTTP** (`#[sqlx::test]`) tiennent le **comportement** : chaque
//!   couple rend le bon code d'erreur, aux deux portées de clé.
//!
//! ⚠️ `axum 0.8` n'expose aucune énumération de routes (`Router` n'offre que
//! `has_routes() -> bool`), d'où le détour par la source : c'est la seule chose
//! qu'un ajout de route modifie forcément.

use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::TimeDelta;
use chrono::Utc;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{Language, NewCompany, NewUser, OrgType, Role};
use kesh_db::repositories::{companies, users};
use serde_json::{Value, json};
use sqlx::MySqlPool;

// ============================================================================
// Le contrat de complétude : la liste, et les bornes qui la contrôlent
// ============================================================================

/// Le texte de `src/lib.rs`, lu à la compilation.
const LIB_RS: &str = include_str!("../src/lib.rs");

const MARKER_BEGIN: &str = "KESH-ADMIN-ROUTES-BEGIN";
const MARKER_END: &str = "KESH-ADMIN-ROUTES-END";

/// Les **25 constructeurs de méthode** du bloc `admin_routes`, un par couple
/// (méthode, chemin). Les identifiants de chemin sont arbitraires : la couche
/// répond **avant** le handler, donc aucune donnée n'a besoin d'exister.
///
/// ⚠️ Cette liste est tenue à jour par le test [`block_declares_exactly_the_listed_couples`],
/// qui la confronte au compte dérivé de la source. Ajouter une route sans
/// l'ajouter ici fait rougir ce test.
const ADMIN_COUPLES: &[(&str, &str)] = &[
    ("GET", "/api/v1/users"),
    ("POST", "/api/v1/users"),
    ("GET", "/api/v1/users/1"),
    ("PUT", "/api/v1/users/1"),
    ("PUT", "/api/v1/users/1/disable"),
    ("PUT", "/api/v1/users/1/reset-password"),
    ("PUT", "/api/v1/company/invoice-settings"),
    ("POST", "/api/v1/vat-rates"),
    ("PUT", "/api/v1/vat-rates/1"),
    ("DELETE", "/api/v1/vat-rates/1"),
    ("PUT", "/api/v1/company/dunning-settings"),
    ("POST", "/api/v1/dunning-levels"),
    ("PUT", "/api/v1/dunning-levels/1"),
    ("DELETE", "/api/v1/dunning-levels/1"),
    ("DELETE", "/api/v1/invoices/1"),
    ("POST", "/api/v1/invoices/1/reminders/1/cancel"),
    ("GET", "/api/v1/admin/full-export"),
    ("POST", "/api/v1/admin/full-import"),
    ("GET", "/api/v1/admin/email-templates"),
    ("GET", "/api/v1/admin/email-templates/invoice/fr"),
    ("PUT", "/api/v1/admin/email-templates/invoice/fr"),
    ("DELETE", "/api/v1/admin/email-templates/invoice/fr"),
    ("PUT", "/api/v1/companies/current/email"),
    ("PUT", "/api/v1/companies/current/contact-details"),
    ("POST", "/api/v1/fiscal-years/1/reopen"),
];

/// Les chemins que `axum` sert **aussi** en `HEAD`, en rejouant le handler `get`
/// et en tronquant le corps — donc exactement les chemins `GET` du bloc.
///
/// ⚠️ **Dérivée, et non maintenue à la main.** Une liste écrite en dur était le
/// dernier morceau du dispositif sans rappel automatique : ajouter un `GET` admin
/// faisait rougir le compteur puis le tuple de répartition, on corrigeait les
/// deux, et le couple `HEAD` correspondant — réellement servi, réellement
/// protégé — n'était exercé par rien.
fn admin_head_paths() -> Vec<&'static str> {
    ADMIN_COUPLES
        .iter()
        .filter(|(method, _)| *method == "GET")
        .map(|(_, path)| *path)
        .collect()
}

/// Retire les commentaires de ligne en **tronquant** chaque ligne à son premier
/// `//`, plutôt qu'en écartant les lignes entières.
///
/// La nuance compte : un commentaire de **fin de ligne** (`… get(h)) // put( plus tard`)
/// ferait compter un constructeur fantôme à un filtre par ligne entière.
///
/// ⚠️ Cette troncature naïve couperait aussi un `//` à l'intérieur d'une chaîne
/// (une URL, par exemple). Le test [`lib_rs_has_no_double_slash_inside_literals`]
/// vérifie que `lib.rs` n'en contient aucune, ce qui rend la troncature sûre ici.
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Le texte du bloc `admin_routes`, bornes exclues.
///
/// Le bornage porte sur la source **brute** : les marqueurs sont eux-mêmes des
/// commentaires, un décommentage préalable les effacerait.
fn admin_block() -> String {
    let after = LIB_RS
        .split_once(MARKER_BEGIN)
        .expect("marqueur d'ouverture absent de lib.rs")
        .1;
    let block = after
        .split_once(MARKER_END)
        .expect("marqueur de fermeture absent de lib.rs")
        .0;
    strip_line_comments(block)
}

/// Vrai si `name(` apparaît dans `src` en **début de mot** — c'est-à-dire comme
/// un appel et non comme la queue d'un identifiant.
///
/// Partagé par le compteur et par la liste des constructeurs interdits : sans
/// cette frontière, `contains("any(")` est vrai pour tout `…company(`, et
/// `contains("on(")` pour tout `…reconciliation(`.
fn contains_word_call(src: &str, name: &str) -> bool {
    count_word_calls(src, name) > 0
}

/// Nombre d'occurrences de `name(` en début de mot.
fn count_word_calls(src: &str, name: &str) -> usize {
    let needle = format!("{name}(");
    let mut n = 0usize;
    let mut from = 0usize;
    while let Some(pos) = src[from..].find(&needle) {
        let abs = from + pos;
        let boundary = abs == 0 || {
            let prev = src[..abs].chars().next_back().expect("abs > 0");
            !prev.is_alphanumeric() && prev != '_'
        };
        if boundary {
            n += 1;
        }
        from = abs + needle.len();
    }
    n
}

/// Compte les constructeurs de méthode par nom.
fn count_method_constructors(src: &str) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for name in ["get", "post", "put", "delete", "patch", "head", "options"] {
        let n = count_word_calls(src, name);
        if n > 0 {
            counts.insert(name, n);
        }
    }
    counts
}

// ============================================================================
// Tests de SOURCE — la complétude
// ============================================================================

#[test]
fn lib_rs_has_no_double_slash_inside_literals() {
    // Prémisse de `strip_line_comments` : la troncature au premier `//` n'est
    // sûre que si ce `//` OUVRE réellement un commentaire. Deux cas le
    // démentent, et c'est EUX qu'il faut chercher : un `://` d'URL dans du code
    // (le premier `//` de la ligne est alors précédé de `:`), et un `//` à
    // l'intérieur d'un littéral de chaîne (nombre impair de `"` avant lui). En
    // commentaire — `/// cf. https://docs.rs/axum/…`, geste courant, présent
    // dans huit autres fichiers du crate — il reste inoffensif : le `//` du
    // marqueur vient avant, et c'est lui qui tronque.
    //
    // ⚠️ La première rédaction assertait `!code.contains("://")` APRÈS la
    // troncature — TAUTOLOGIE : `code` s'arrête au premier `//`, il ne peut par
    // construction pas en contenir, et le test ne pouvait jamais rougir.
    // (Réfutée en passe 3 : le garde-fou écrit pour empêcher la dérive du
    // compteur était lui-même muet.)
    for (i, line) in LIB_RS.lines().enumerate() {
        if let Some(pos) = line.find("//") {
            let before = &line[..pos];
            let inside_literal = before.matches('"').count() % 2 == 1;
            let part_of_url = before.ends_with(':');
            assert!(
                !(inside_literal || part_of_url),
                "lib.rs:{} : le premier `//` de la ligne n'ouvre pas un \
                 commentaire — la troncature de `strip_line_comments` couperait \
                 du CODE et fausserait les comptages : {}",
                i + 1,
                line.trim()
            );
        }
    }
}

#[test]
fn admin_routes_block_is_bounded_exactly_once() {
    assert_eq!(
        LIB_RS.matches(MARKER_BEGIN).count(),
        1,
        "le marqueur d'ouverture doit apparaître exactement une fois"
    );
    assert_eq!(
        LIB_RS.matches(MARKER_END).count(),
        1,
        "le marqueur de fermeture doit apparaître exactement une fois"
    );
    assert!(
        !admin_block().trim().is_empty(),
        "bloc admin_routes vide — le comptage porterait sur rien et le test \
         passerait À VIDE (mode d'échec du test muet)"
    );
}

#[test]
fn block_declares_exactly_the_listed_couples() {
    let counts = count_method_constructors(&admin_block());
    let total: usize = counts.values().sum();

    assert_eq!(
        total,
        ADMIN_COUPLES.len(),
        "le bloc admin_routes déclare {total} constructeurs de méthode pour \
         {} couples listés dans ADMIN_COUPLES.\n\
         Ventilation lue dans la source : {counts:?}\n\
         → Une route d'administration a été ajoutée ou retirée sans mettre à \
         jour ADMIN_COUPLES. Ajoutez-y le couple (méthode, chemin) : c'est ce \
         qui garantit qu'il est effectivement TESTÉ contre un PAT.",
        ADMIN_COUPLES.len()
    );

    // Le total seul ne suffirait pas : un `get` retiré et un `post` ajouté le
    // laisseraient inchangé. La ventilation, elle, bouge.
    let mut expected: BTreeMap<&'static str, usize> = BTreeMap::new();
    for (method, _) in ADMIN_COUPLES {
        let key = match *method {
            "GET" => "get",
            "POST" => "post",
            "PUT" => "put",
            "DELETE" => "delete",
            "PATCH" => "patch",
            // `head` et `options` sont comptés par la source : les rendre
            // exprimables ici, sinon leur ajout mène à un cul-de-sac — le
            // compteur rougit, et la liste refuse la méthode qui le corrigerait.
            "HEAD" => "head",
            "OPTIONS" => "options",
            other => panic!("méthode inattendue dans ADMIN_COUPLES : {other}"),
        };
        *expected.entry(key).or_default() += 1;
    }
    assert_eq!(
        counts, expected,
        "la ventilation par méthode de la source ne correspond pas à celle \
         d'ADMIN_COUPLES"
    );

    // ⚠️ Total et ventilation ne suffisent pas : un couple RECOPIÉ d'une entrée
    // existante les satisfait tous les deux. Démontré — route neuve dans le bloc
    // + doublon dans la liste : total 26 des deux côtés, `put` +1 des deux côtés,
    // tout concorde, et le test HTTP exerce deux fois le même chemin en n'en
    // exerçant jamais le nouveau. Une faute de recopie dans une liste de 26
    // lignes est exactement ce qu'un tel contrat doit rattraper.
    let unique: BTreeSet<&(&str, &str)> = ADMIN_COUPLES.iter().collect();
    assert_eq!(
        unique.len(),
        ADMIN_COUPLES.len(),
        "ADMIN_COUPLES contient un couple en double — le total et la ventilation \
         restent justes, mais une route réelle n'est alors couverte par aucun test"
    );

    // ⚠️ Et le nombre d'ENREGISTREMENTS doit égaler le nombre de CHEMINS
    // distincts. Sans cela, un `MethodRouter` rendu par un helper local —
    // `.route("/x", admin_only_put(h))` — reste invisible au compteur : la
    // frontière de mot qui empêche de compter `delete_invoice(` empêche aussi
    // de compter `admin_only_put(`. Démontré : les tests restaient verts.
    let distinct_paths: BTreeSet<&str> = ADMIN_COUPLES.iter().map(|(_, p)| *p).collect();
    assert_eq!(
        admin_block().matches(".route(").count(),
        distinct_paths.len(),
        "le bloc déclare {} enregistrements `.route(` pour {} chemins distincts \
         dans ADMIN_COUPLES. Un écart signale une route dont le constructeur de \
         méthode échappe au comptage — typiquement un `MethodRouter` rendu par \
         un helper, dont le nom se termine par le nom d'une méthode.",
        admin_block().matches(".route(").count(),
        distinct_paths.len()
    );
}

#[test]
fn block_uses_no_unlisted_route_constructor() {
    // Le compteur ne reconnaît que sept constructeurs ; `axum` en exporte
    // vingt-deux. Plutôt que d'énumérer les quinze autres, on asserte le
    // COMPLÉMENT — sinon `any(handler)` enregistrerait NEUF méthodes en
    // laissant le compteur à 25 : la route serait protégée, mais absente de la
    // couverture, et rien ne le signalerait.
    let block = admin_block();
    for forbidden in ["any", "on", "trace", "connect"] {
        assert!(
            !contains_word_call(&block, forbidden),
            "constructeur `{forbidden}(` dans admin_routes : il enregistre des \
             méthodes que le compteur ne voit pas. Utilisez les constructeurs \
             nommés, ou étendez ADMIN_COUPLES *et* count_method_constructors."
        );
    }
    assert!(
        !block.contains("_service("),
        "constructeur `*_service(` dans admin_routes : invisible au compteur, \
         même raison que ci-dessus."
    );
}

#[test]
fn nothing_is_added_after_the_first_route_layer() {
    let block = admin_block();
    assert_eq!(
        block.matches(".route_layer(").count(),
        2,
        "admin_routes doit porter exactement deux couches — le RBAC et l'anti-PAT. \
         Sans cette assertion, un marqueur de fin posé trop tôt rendrait la \
         vérification suivante VIDE."
    );
    let after_first_layer = block
        .split_once(".route_layer(")
        .expect("aucune couche dans le bloc")
        .1;
    for forbidden in [
        ".route(",
        ".route_service(",
        ".nest(",
        ".nest_service(",
        ".merge(",
        ".fallback(",
        ".method_not_allowed_fallback(",
    ] {
        assert!(
            !after_first_layer.contains(forbidden),
            "`{forbidden}` après la première couche : `route_layer` n'enveloppe \
             que les routes DÉJÀ enregistrées à l'appel, donc cette route \
             échapperait AUX DEUX couches — ni RBAC, ni anti-PAT. Elle compile \
             et ne panique pas. Déplacez-la au-dessus des `.route_layer(...)`."
        );
    }
}

#[test]
fn no_route_enters_the_block_by_composition() {
    // ⚠️ Le trou que ferme ce test a été DÉMONTRÉ, pas supposé : un
    // `.merge(un_sous_routeur())` placé dans le bloc **avant** les couches est
    // correctement protégé à l'exécution — `route_layer` enveloppe le routeur
    // assemblé — mais ses constructeurs `get(`/`post(` vivent dans un AUTRE
    // fichier, invisibles à `count_method_constructors`, qui ne lit que `lib.rs`.
    //
    // Conséquence : le compteur reste à 25, `ADMIN_COUPLES` ne gagne pas le
    // couple, et **aucun test HTTP n'exerce jamais cette route**. La protection
    // tient ; c'est le RAPPEL qui tombe — or c'est le rappel qui est l'objet de
    // cette story, et ce qui avait laissé 16 routes sur 19 sans garde.
    //
    // `nothing_is_added_after_the_first_route_layer` ne couvre pas ce cas : il
    // ne regarde qu'APRÈS la première couche, là où le danger est de contourner
    // les couches. Ici le danger est de contourner le COMPTEUR, et il vit avant.
    let block = admin_block();
    for forbidden in [".merge(", ".nest(", ".nest_service(", ".route_service("] {
        assert!(
            !block.contains(forbidden),
            "`{forbidden}` dans le bloc admin_routes : la route serait protégée \
             mais INVISIBLE au compteur, ses constructeurs vivant dans un autre \
             fichier — donc jamais couverte par les tests HTTP d'ADMIN_COUPLES. \
             Déclarez les routes admin par `.route(...)` direct dans ce bloc ; si \
             une composition devient nécessaire, elle vient AVEC l'extension du \
             compteur et des couples."
        );
    }
}

#[test]
fn the_admin_guards_exist_once_each_and_live_inside_the_block() {
    // ⚠️ Le trou que ferme ce test a été DÉMONTRÉ : un SECOND routeur admin,
    // déclaré hors du bloc et mergé dans `protected` avec `require_admin_role`
    // mais SANS `require_not_pat`, échappe à la totalité du dispositif. Le
    // compteur reste à 25, `admin_routes` apparaît toujours 2 fois, il y a
    // toujours 2 `.route_layer(` dans le bloc — et une route d'administration
    // redevient atteignable par un PAT. C'est **littéralement le mode d'échec
    // de #167**, rouvert par une autre porte.
    //
    // Le reste du fichier ancre la complétude au TEXTE DU BLOC. Ce test ancre
    // l'autre moitié de la propriété : « toute route gardée par
    // `require_admin_role` porte aussi `require_not_pat` », en interdisant qu'il
    // existe un second site d'application.
    let clean = strip_line_comments(LIB_RS);
    let block = admin_block();
    for guard in ["require_admin_role", "require_not_pat"] {
        assert_eq!(
            clean.matches(guard).count(),
            1,
            "`{guard}` doit être appliqué à UN SEUL endroit du routeur. Une \
             seconde occurrence signale un autre sous-routeur d'administration \
             — qui échapperait au compteur, à la liste ADMIN_COUPLES et donc à \
             tout test anti-PAT. Montez les routes admin dans le bloc borné."
        );
        assert!(
            block.contains(guard),
            "`{guard}` n'est plus appliqué DANS le bloc borné. L'unicité ne \
             suffit pas : il faut que l'unique application soit celle que le \
             compteur surveille."
        );
    }
}

#[test]
fn the_admin_guards_have_no_alias_and_no_second_consumer_crate_wide() {
    // Le test précédent compte les occurrences DANS `lib.rs` — un ALIAS déclaré
    // ailleurs (`pub use require_admin_role as check_admin;` dans `rbac.rs`) le
    // laisse vert : `lib.rs` consommerait `check_admin`, les compteurs restent
    // à 1, et un second sous-routeur d'administration échapperait de nouveau à
    // `require_not_pat` — la classe du second routeur, rouverte par un autre
    // biais. Même chose pour un wrapper ou un second consommateur dans un autre
    // module. (Relevé en passe 3.)
    //
    // Fermeture : sur le crate ENTIER décommenté, chaque identifiant de garde
    // apparaît exactement DEUX fois — sa définition dans `middleware/rbac.rs`,
    // sa consommation dans le bloc de `lib.rs`. Un alias, un ré-export, un
    // wrapper ou un second consommateur en crée forcément une troisième, où
    // qu'il vive : l'alias se déclare quelque part, et cette déclaration porte
    // le nom réel.
    fn rs_files(dir: &std::path::Path, acc: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("lecture d'un répertoire de src/") {
            let path = entry.expect("entrée de répertoire lisible").path();
            if path.is_dir() {
                rs_files(&path, acc);
            } else if path.extension().is_some_and(|e| e == "rs") {
                acc.push(path);
            }
        }
    }
    fn count_word(hay: &str, word: &str) -> usize {
        let bytes = hay.as_bytes();
        let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
        hay.match_indices(word)
            .filter(|(i, _)| {
                let before_ok = *i == 0 || !is_ident(bytes[i - 1]);
                let after = i + word.len();
                let after_ok = after >= bytes.len() || !is_ident(bytes[after]);
                before_ok && after_ok
            })
            .count()
    }

    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rs_files(&src, &mut files);
    assert!(
        files.len() > 10,
        "balayage suspect : {} fichiers .rs seulement sous src/ — le garde ne \
         couvre plus le crate",
        files.len()
    );

    for guard in ["require_admin_role", "require_not_pat"] {
        let mut sites: Vec<(String, usize)> = Vec::new();
        for path in &files {
            let content = std::fs::read_to_string(path).expect("lecture d'un .rs de src/");
            let n = count_word(&strip_line_comments(&content), guard);
            if n > 0 {
                sites.push((path.display().to_string(), n));
            }
        }
        sites.sort();
        let total: usize = sites.iter().map(|(_, n)| n).sum();
        assert!(
            total == 2
                && sites.len() == 2
                && sites
                    .iter()
                    .any(|(p, n)| p.ends_with("middleware/rbac.rs") && *n == 1)
                && sites
                    .iter()
                    .any(|(p, n)| p.ends_with("src/lib.rs") && *n == 1),
            "`{guard}` doit apparaître (hors commentaires) exactement DEUX fois \
             dans le crate : sa définition dans middleware/rbac.rs et sa \
             consommation dans le bloc admin_routes de lib.rs. Trouvé : {sites:?}. \
             Une occurrence de plus signale un alias, un ré-export, un wrapper ou \
             un second consommateur — autant de portes vers un sous-routeur \
             d'administration sans `require_not_pat`."
        );
    }
}

#[test]
fn the_block_carries_no_block_comment() {
    // `strip_line_comments` ne connaît que `//`. Un commentaire `/* … */` dont la
    // prose mentionne `get(` serait compté comme un constructeur — démontré : le
    // compteur passe alors à 26 et le test rougit sur un texte qui n'ajoute
    // aucune route. Fail-loud ici, avec le motif, plutôt qu'un décompte faux à
    // diagnostiquer ailleurs.
    assert!(
        !admin_block().contains("/*"),
        "commentaire de bloc `/* … */` dans admin_routes : le décommentage ne le \
         retire pas, et sa prose peut fausser le décompte des constructeurs. \
         Utilisez des commentaires `//` dans ce bloc."
    );
}

#[test]
fn admin_routes_is_never_reassigned() {
    // Le trou de `route_layer` ne vit pas que dans le bloc : une réaffectation
    // `admin_routes = admin_routes.route(...)` écrite n'importe où plus loin
    // dans le fichier échapperait aux deux couches ET au compteur. Le fichier
    // utilise déjà trois fois cet idiome sur `main_router`.
    let clean = strip_line_comments(LIB_RS);
    assert_eq!(
        clean.matches("admin_routes").count(),
        2,
        "`admin_routes` doit apparaître exactement deux fois hors commentaires \
         — sa déclaration et son `.merge(`. Toute autre occurrence est \
         probablement une réaffectation qui contourne les couches."
    );
    // Le compte d'occurrences ci-dessus est le garde-fou principal — toute
    // réaffectation en ajoute une. Ce second contrôle vise la forme `mut`, qui
    // annonce l'intention de réaffecter avant même qu'elle s'écrive.
    //
    // ⚠️ Il ne cherche PAS les lignes « commençant par admin_routes et
    // contenant = » : cette formulation dépendait d'une précédence d'opérateurs
    // douteuse et ratait une réaffectation coupée sur deux lignes. Le compte
    // d'occurrences, lui, la voit quelle que soit sa mise en page.
    assert!(
        !clean.contains("mut admin_routes"),
        "`admin_routes` déclaré `mut` : la seule raison de le rendre mutable est \
         de le réaffecter plus loin, ce qui ferait échapper les routes ajoutées \
         aux deux couches ET au compteur."
    );
}

#[test]
fn the_three_intrinsic_guards_are_still_wired() {
    // Décision D3 : les trois `ensure_not_pat` d'`admin_routes` restent, parce
    // que l'interdiction est intrinsèque à ces opérations et doit survivre à un
    // déplacement de route. Mais depuis que la couche répond la première, leurs
    // tests prouvent la COUCHE : les retirer ne ferait plus rougir personne.
    // Cette assertion de source est ce qui rend la revendication de D3 vérifiable.
    const ADMIN_RS: &str = include_str!("../src/routes/admin.rs");
    const FISCAL_YEARS_RS: &str = include_str!("../src/routes/fiscal_years.rs");

    assert_eq!(
        strip_line_comments(ADMIN_RS)
            .matches("ensure_not_pat(&current_user)?")
            .count(),
        2,
        "les gardes intrinsèques de full-export et full-import ont disparu \
         d'admin.rs (décision D3)"
    );
    assert_eq!(
        strip_line_comments(FISCAL_YEARS_RS)
            .matches("ensure_not_pat(&current_user)?")
            .count(),
        1,
        "la garde intrinsèque de la réouverture d'exercice a disparu de \
         fiscal_years.rs (décision D3)"
    );
}

// ============================================================================
// Harnais HTTP
// ============================================================================

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

struct TestApp {
    base_url: String,
    client: reqwest::Client,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Envoie `method path` avec le jeton porté en `Authorization: Bearer`.
    async fn send(&self, method: &str, path: &str, bearer: &str) -> reqwest::Response {
        let m = reqwest::Method::from_bytes(method.as_bytes()).expect("méthode HTTP valide");
        self.client
            .request(m, self.url(path))
            .header("Authorization", format!("Bearer {bearer}"))
            // Corps JSON vide : les extracteurs vivent dans le handler, que la
            // couche empêche d'atteindre. S'il était atteint, on lirait un 4xx
            // de validation au lieu du 403 attendu — et le test le dirait.
            .json(&json!({}))
            .send()
            .await
            .expect("requête HTTP")
    }
}

fn test_config() -> Config {
    Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        TEST_ADMIN_PASSWORD.to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        TimeDelta::minutes(15),
        100,
        TimeDelta::minutes(30),
        12,
    )
}

async fn spawn_app(pool: MySqlPool) -> TestApp {
    let config = test_config();
    let rate_limiter = kesh_api::middleware::rate_limit::RateLimiter::new(&config);
    let i18n = Arc::new(
        kesh_i18n::I18nBundle::load(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
    let state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n);
    let app = build_router(state.clone(), "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            Err(e) => panic!("test server not ready in 2s: {e}"),
        }
    }
    TestApp {
        base_url: format!("http://{}", addr),
        client: reqwest::Client::new(),
    }
}

fn forge_jwt(user_id: i64, role: &str, company_id: i64) -> String {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        company_id,
        iat: now,
        exp: now + 3600,
    };
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET),
    )
    .unwrap()
}

/// Monte une société, un utilisateur du rôle voulu, et rend un PAT de la portée
/// demandée créé par cet utilisateur.
async fn pat_for(app: &TestApp, pool: &MySqlPool, label: &str, role: Role, scope: &str) -> String {
    let company_id = companies::create(
        pool,
        NewCompany {
            name: label.into(),
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
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap()
    .id;

    let user_id = users::create(
        pool,
        NewUser {
            username: format!("{label}_user"),
            password_hash: hash_password("password123").unwrap(),
            role,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id;

    let jwt = forge_jwt(user_id, role.as_str(), company_id);
    let resp = app
        .client
        .post(app.url("/api/v1/settings/api-keys"))
        .header("Authorization", format!("Bearer {jwt}"))
        .json(&json!({ "name": format!("{label} key"), "scope": scope }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "création de la clé de test");
    let body: Value = resp.json().await.unwrap();
    body["key"].as_str().unwrap().to_string()
}

/// Lit `error.code` du corps, en disant clairement ce qu'on a reçu à la place.
async fn error_code(resp: reqwest::Response) -> String {
    let status = resp.status();
    let body: Value = resp
        .json()
        .await
        .unwrap_or_else(|e| panic!("corps JSON attendu (statut {status}) : {e}"));
    body["error"]["code"]
        .as_str()
        .unwrap_or_else(|| panic!("champ error.code absent du corps : {body}"))
        .to_string()
}

// ============================================================================
// Tests HTTP — le comportement
// ============================================================================

/// AC1, jambe `read-write` : les 25 couples rendent `API_KEY_ADMIN_FORBIDDEN`.
///
/// ⚠️ On asserte le **code**, pas le statut. Trois gardes distinctes rendent
/// `403` sur ces routes — le RBAC, le gate de portée, et la couche : un test
/// qui se contenterait du statut passerait sans que la couche soit atteinte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn every_admin_couple_denies_a_read_write_pat(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let pat = pat_for(&app, &pool, "RW", Role::Admin, "read-write").await;

    for (method, path) in ADMIN_COUPLES {
        let resp = app.send(method, path, &pat).await;
        let status = resp.status();
        let code = error_code(resp).await;
        assert_eq!(
            (status.as_u16(), code.as_str()),
            (403, "API_KEY_ADMIN_FORBIDDEN"),
            "{method} {path} : un PAT read-write de créateur Admin doit être \
             refusé PAR LA COUCHE"
        );
    }
}

/// AC1, jambe `read-only` — et elle se lit par couple, parce que le gate de
/// portée répond **avant** la couche.
///
/// Les 5 `get` atteignent la couche ; les 20 méthodes mutantes sont arrêtées en
/// amont par `require_auth` avec `API_KEY_READ_ONLY`, **qui existait avant cette
/// story**. Prescrire `403` partout ferait un test muet sur ces vingt-là : la
/// mutation « retirer la couche doit faire rougir » y est insatisfaisable.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn read_only_pat_is_stopped_by_the_right_guard_on_each_couple(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let pat = pat_for(&app, &pool, "RO", Role::Admin, "read").await;

    let mut reached_layer = 0usize;
    let mut stopped_by_scope = 0usize;

    for (method, path) in ADMIN_COUPLES {
        let expected = if *method == "GET" {
            reached_layer += 1;
            "API_KEY_ADMIN_FORBIDDEN"
        } else {
            stopped_by_scope += 1;
            "API_KEY_READ_ONLY"
        };
        let resp = app.send(method, path, &pat).await;
        let status = resp.status();
        let code = error_code(resp).await;
        assert_eq!(
            (status.as_u16(), code.as_str()),
            (403, expected),
            "{method} {path} : mauvais garde-fou pour une clé read-only"
        );
    }

    assert_eq!(
        (reached_layer, stopped_by_scope),
        (5, 20),
        "la répartition attendue entre la couche et le gate de portée a changé"
    );
}

/// Les `HEAD` servis par les handlers `get` sont protégés eux aussi.
///
/// ⚠️ **Seul le statut est assertable ici** : une réponse `HEAD` n'a pas de
/// corps, donc `error.code` est illisible par construction. C'est une preuve plus
/// faible que celle des autres couples — assumée, parce que `HEAD` emprunte
/// exactement la pile de couches de son `GET`, déjà couvert au code.
///
/// ⚠️⚠️ **Et sur un chemin, la preuve est NULLE, non pas seulement faible.**
/// `/api/v1/admin/full-export` porte la garde intrinsèque `ensure_not_pat` de la
/// décision D3, qui rend elle aussi un `403` : couche retirée, ce couple resterait
/// **vert**. Les autres chemins n'ont pas de garde dans leur handler et sont donc
/// bien sensibles. Le fait est nommé ici plutôt que masqué — c'est le prix d'une
/// redondance assumée, et il faut savoir où elle rend un test aveugle.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn head_couples_are_denied_too(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let pat = pat_for(&app, &pool, "HEAD", Role::Admin, "read").await;

    // Les chemins dont le handler porte une garde intrinsèque : leur `403` peut
    // venir d'ailleurs que de la couche, donc ils ne discriminent pas.
    const GUARDED_BY_HANDLER: &[&str] = &["/api/v1/admin/full-export"];

    let mut discriminating = 0usize;
    for path in admin_head_paths() {
        let resp = app.send("HEAD", path, &pat).await;
        assert_eq!(
            resp.status(),
            403,
            "HEAD {path} est servi par le handler GET et doit être refusé"
        );
        if !GUARDED_BY_HANDLER.contains(&path) {
            discriminating += 1;
        }
    }
    assert!(
        discriminating >= 4,
        "moins de quatre couples HEAD discriminants : la preuve de ce test \
         reposerait presque entièrement sur des chemins dont le handler rend \
         déjà 403 par lui-même"
    );
}

/// D6 — quand les deux couches refusent, c'est l'anti-PAT qui répond.
///
/// Un PAT créé par un **Comptable** est refusé par le RBAC (rôle insuffisant)
/// *et* par la couche (c'est un PAT). Le code observable doit être unique,
/// quel que soit le rôle du créateur : sinon AC1 devrait se ramifier.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn comptable_created_pat_gets_the_same_code_as_an_admin_one(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let pat = pat_for(&app, &pool, "Comptable", Role::Comptable, "read-write").await;

    let resp = app.send("POST", "/api/v1/users", &pat).await;
    let status = resp.status();
    let code = error_code(resp).await;
    assert_eq!(
        (status.as_u16(), code.as_str()),
        (403, "API_KEY_ADMIN_FORBIDDEN"),
        "l'ordre des deux couches ne donne pas la précédence voulue par D6 — \
         déplacez la couche anti-PAT APRÈS le RBAC dans lib.rs"
    );
}

/// AC5 — le chemin d'attaque de #167, rejoué en entier.
///
/// C'est le test qui dit *pourquoi* la story existe : un jeton fuité créait un
/// administrateur, s'y connectait par l'interface, et se forgeait de nouvelles
/// clés — si bien que **révoquer le jeton n'arrêtait plus l'incident**.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_leaked_admin_pat_can_no_longer_create_an_administrator(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let pat = pat_for(&app, &pool, "Fuite", Role::Admin, "read-write").await;

    let resp = app
        .client
        .post(app.url("/api/v1/users"))
        .header("Authorization", format!("Bearer {pat}"))
        .json(&json!({
            "username": "attaquant",
            "password": "MotDePasseChoisiParLAttaquant1",
            "role": "Admin",
        }))
        .send()
        .await
        .unwrap();

    let status = resp.status();
    let code = error_code(resp).await;
    assert_eq!(
        (status.as_u16(), code.as_str()),
        (403, "API_KEY_ADMIN_FORBIDDEN"),
        "la première marche de l'escalade de #167 est rouverte"
    );

    // Et le compte n'existe pas : l'échec est réel, pas seulement rapporté.
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE username = ?")
        .bind("attaquant")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0, "aucun utilisateur ne doit avoir été créé");
}
