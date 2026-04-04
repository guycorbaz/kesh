# Story 1.3 : Types core (newtypes & validation)

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a développeur,
I want des types métier forts avec validation intégrée,
so that l'intégrité des données soit garantie dès la couche logique.

## Acceptance Criteria

1. **Given** un montant, **When** création de `Money(Decimal)`, **Then** le type utilise `rust_decimal` exclusivement, jamais de `f64`
2. **Given** un IBAN, **When** création de `Iban(String)`, **Then** le format et le checksum sont validés (retour d'erreur si invalide)
3. **Given** un numéro IDE, **When** création de `CheNumber(String)`, **Then** le format CHE-xxx.xxx.xxx et le checksum sont validés
4. **Given** un QR-IBAN, **When** création de `QrIban(String)`, **Then** le format QR-IBAN est validé (plage QR-IID 30000-31999)
5. **And** chaque type a des tests unitaires couvrant les cas valides et invalides
6. **And** documentation `///` sur chaque type public

## Tasks / Subtasks

- [x] Task 1 : Configurer `kesh-core/Cargo.toml` avec les dépendances (AC: 1-4)
  - [x] 1.1 Ajouter `rust_decimal` 1.41 avec features `serde-str`, `maths`
  - [x] 1.2 Ajouter `rust_decimal_macros` 1.40
  - [x] 1.3 Ajouter `serde` 1 avec feature `derive`
  - [x] 1.4 Ajouter `thiserror` 2 pour les erreurs typées
  - [x] 1.5 Ajouter `[dev-dependencies]` : `serde_json = "1"` (requis pour les tests de sérialisation round-trip)
  - [x] 1.6 Vérifier la compilation du workspace : `cargo build --workspace`
- [x] Task 2 : Créer le module `types/` et `errors.rs` (AC: 1-4)
  - [x] 2.1 Créer `crates/kesh-core/src/types/mod.rs` avec réexports publics
  - [x] 2.2 Créer `crates/kesh-core/src/errors.rs` avec `CoreError` enum
  - [x] 2.3 Mettre à jour `crates/kesh-core/src/lib.rs` pour exposer `pub mod types` et `pub mod errors`
- [x] Task 3 : Implémenter `Money` (AC: 1, 5, 6)
  - [x] 3.1 Créer `crates/kesh-core/src/types/money.rs`
  - [x] 3.2 Newtype `Money(Decimal)` avec `new()`, `amount()`, `zero()`, `is_negative()`
  - [x] 3.3 Implémenter `Add`, `Sub`, `Neg`, `Mul<Decimal>` (qty*price, TVA), `Display`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Copy`. Retour de Add/Sub/Mul = `Money` (pas `Result`, overflow impossible en comptabilité)
  - [x] 3.4 Implémenter `Sum` trait (`impl Sum for Money`) pour `iter().sum::<Money>()`
  - [x] 3.5 Méthode `round_chf()` : arrondi commercial au centime (2 décimales, `MidpointAwayFromZero`)
  - [x] 3.6 Serde : `#[serde(transparent)]` pour sérialisation directe en string décimal via `serde-str` (`"1234.56"`, jamais de float JSON)
  - [x] 3.7 `///` doc comments sur chaque élément public
  - [x] 3.8 Tests unitaires : création, arithmétique (add/sub/mul/sum/neg), arrondi, sérialisation round-trip, montants négatifs (avoirs, contre-passations), `Money::zero()`, cas limites
- [x] Task 4 : Implémenter `Iban` (AC: 2, 5, 6)
  - [x] 4.1 Créer `crates/kesh-core/src/types/iban.rs`
  - [x] 4.2 Newtype `Iban(String)` avec constructeur validant (retourne `Result<Iban, CoreError>`)
  - [x] 4.3 Validation : format + checksum MOD-97 (ISO 13616) + longueur par pays via table de lookup (CH/LI=21, DE=22, FR=27, etc.). Supporter les IBAN internationaux car CAMT.053 (story 6.1) et pain.001 (story 10.2) contiennent des IBAN étrangers
  - [x] 4.4 Méthodes : `country_code()`, `bank_clearing_number() -> Option<&str>` (retourne `Some` uniquement pour CH/LI, `None` pour les autres pays), `is_swiss() -> bool`, `as_str()`, `formatted()` (avec espaces par groupes de 4)
  - [x] 4.5 Implémenter `Display`, `FromStr`, `TryFrom<String>` (délègue à `FromStr`), `AsRef<str>`, `From<Iban> for String` (requis par serde `into`), `Serialize` (via `#[serde(into = "String")]`), `Deserialize` (via `#[serde(try_from = "String")]` — la désérialisation DOIT valider le checksum)
  - [x] 4.6 `///` doc comments
  - [x] 4.7 Tests : IBAN suisses valides/invalides, IBAN internationaux (DE, FR, AT), checksum erroné, longueur incorrecte par pays, format avec/sans espaces, sérialisation/désérialisation round-trip avec validation
- [x] Task 5 : Implémenter `QrIban` (AC: 4, 5, 6)
  - [x] 5.1 Créer `crates/kesh-core/src/types/qr_iban.rs`
  - [x] 5.2 Newtype `QrIban(Iban)` — wraps un `Iban` validé dont le QR-IID est dans 30000-31999
  - [x] 5.3 Constructeur : valide d'abord comme IBAN, puis vérifie la plage QR-IID (positions 5-9)
  - [x] 5.4 Méthodes : `qr_iid()`, `as_iban()`, `as_str()`
  - [x] 5.5 Implémenter `Display`, `FromStr`, `TryFrom<String>` (délègue à `FromStr`), `From<QrIban> for String` (extrait la string de l'Iban interne, requis par serde `into`), `Serialize` (via `#[serde(into = "String")]`), `Deserialize` (via `#[serde(try_from = "String")]` — valide IBAN + plage QR-IID)
  - [x] 5.6 `///` doc comments
  - [x] 5.7 Tests : QR-IBAN valides (IID 30000, 31999), IBAN régulier rejeté, bornes limites
- [x] Task 6 : Implémenter `CheNumber` (AC: 3, 5, 6)
  - [x] 6.1 Créer `crates/kesh-core/src/types/che_number.rs`
  - [x] 6.2 Newtype `CheNumber(String)` — stocke la forme normalisée (sans séparateurs)
  - [x] 6.3 Constructeur : normaliser en majuscules, retirer le suffixe TVA/MWST/IVA s'il est présent, parse CHE-xxx.xxx.xxx ou CHExxxxxxxxx, valide checksum modulo 11
  - [x] 6.4 Méthode `formatted()` : retourne `"CHE-xxx.xxx.xxx"`
  - [x] 6.5 Implémenter `Display` (format `CHE-xxx.xxx.xxx`), `FromStr`, `TryFrom<String>` (délègue à `FromStr`), `AsRef<str>`, `From<CheNumber> for String` (requis par serde `into`), `Serialize` (via `#[serde(into = "String")]`), `Deserialize` (via `#[serde(try_from = "String")]` — valide checksum)
  - [x] 6.6 `///` doc comments
  - [x] 6.7 Tests : CHE valides (dont CHE-109.322.551 de la spec officielle), checksum invalide, check digit=10 (numéro impossible), formats variés, minuscules (`che-109.322.551`), avec suffixe (`CHE-109.322.551 MWST`, `TVA`, `IVA`), sérialisation/désérialisation round-trip
- [x] Task 7 : Validation finale (AC: 1-6)
  - [x] 7.1 `cargo build --workspace` sans erreur
  - [x] 7.2 `cargo test -p kesh-core` — tous les tests passent (78 unit + 4 doc-tests après fixes)
  - [x] 7.3 `cargo doc -p kesh-core --no-deps` — documentation générée sans warning
  - [x] 7.4 `cargo clippy -p kesh-core` — aucun warning

### Review Follow-ups (AI)

Revue de code adversariale (3 reviewers en parallèle) — 2 HIGH + 6 MED + 1 LOW à corriger.

- [x] [AI-Review HIGH] Fix #1 — Iban::new panique sur input UTF-8 multi-octet (`iban.rs:70`). Ajout `is_ascii()` guard + 2 tests.
- [x] [AI-Review HIGH] Fix #2 — Money arithmétique panique sur overflow Decimal. Documenté dans la doc `Money`.
- [x] [AI-Review MED] Fix #3 — QrIban::qr_iid() unwrap en production. Stocké comme champ `iid: u32` à la construction.
- [x] [AI-Review MED] Fix #4 — Iban::formatted() unwrap. Remplacé par `.expect("invariant...")`.
- [x] [AI-Review MED] Fix #5 — `round_chf` renommé en `round_to_centimes` + clarification doc vs rappen rounding.
- [x] [AI-Review MED] Fix #6 — QrIban::new propage les erreurs IBAN sous-jacentes sans re-wrapping.
- [x] [AI-Review MED] Fix #7 — CheNumber normalise les whitespace Unicode (tab, NBSP, double espace) avant de stripper le suffixe MWST/TVA/IVA. +3 tests.
- [x] [AI-Review MED] Fix #8 — Retrait de `FK` (Falkland) de IBAN_LENGTHS (non dans le registre SWIFT officiel).
- [x] [AI-Review LOW] Fix #9 — `is_swiss()` documenté pour clarifier l'inclusion de LI via SIX Swiss Payment Standards.

## Dev Notes

### Périmètre strict de cette story

**UNIQUEMENT ces 4 newtypes :** `Money`, `Iban`, `QrIban`, `CheNumber`. Ne PAS créer `UserId`, `CompanyId`, `Role`, `Version`, `AccountId`, `FiscalYearId` ou tout autre type domaine — ils seront définis dans les stories 1.4 (UserId, CompanyId), 1.7 (Role), 1.8 (Version) respectivement.

### Architecture kesh-core — Contrainte fondamentale

`kesh-core` est un crate de **logique métier pure, zéro I/O**. Aucune dépendance sur la base de données, le réseau ou le filesystem. C'est le fondement de tous les autres crates : `kesh-db`, `kesh-api`, `kesh-reconciliation`, `kesh-report` en dépendent.

### Structure de fichiers à créer

L'architecture prévoit deux répertoires séparés (`types/` pour les newtypes, `validation/` pour la logique de validation). Pour cette story, la validation est co-localisée dans les fichiers types (constructeur validant dans chaque newtype). Le répertoire `validation/` sera créé dans une story future si des validations transversales (ex: validation d'écriture équilibrée) nécessitent un module dédié.

Les modules `accounting/` et `chart_of_accounts/` prévus par l'architecture seront ajoutés dans les stories 3.x (plan comptable et écritures). Ne PAS les créer maintenant.

```
crates/kesh-core/
├── Cargo.toml                          # Dépendances (rust_decimal, serde, thiserror)
└── src/
    ├── lib.rs                          # pub mod types; pub mod errors;
    ├── errors.rs                       # CoreError enum
    └── types/
        ├── mod.rs                      # pub use money::Money; pub use iban::Iban; etc.
        ├── money.rs                    # Money(Decimal) + validation co-localisée
        ├── iban.rs                     # Iban(String) + validation ISO 13616 co-localisée
        ├── qr_iban.rs                  # QrIban(Iban) + validation QR-IID co-localisée
        └── che_number.rs              # CheNumber(String) + validation modulo 11 co-localisée
```

### Conventions de code (établies stories 1.1-1.2)

| Élément | Convention |
|---------|-----------|
| Structs/Enums | PascalCase : `Money`, `Iban`, `CoreError` |
| Fonctions/méthodes | snake_case : `validate_iban`, `round_chf` |
| Modules | snake_case : `che_number`, `qr_iban` |
| Fichiers | snake_case : `che_number.rs`, `qr_iban.rs` |
| Tests | Co-localisés : `#[cfg(test)] mod tests` dans chaque fichier |
| Documentation | `///` sur chaque type, méthode et constante publique |
| Erreurs | Types d'erreur par crate via `thiserror`, `CoreError` dans `errors.rs` |

### Pattern de newtype

Chaque type suit le même pattern :
1. **Struct tuple** : `pub struct TypeName(inner_type);`
2. **Constructeur validant** : `pub fn new(input) -> Result<Self, CoreError>`  — jamais de construction directe sans validation. **Exception : `Money::new(Decimal)` est infaillible** (tout `Decimal` est un montant valide, pas de variant `CoreError` pour Money)
3. **Accesseur** : `pub fn as_str(&self)` ou `pub fn amount(&self)` pour accéder à la valeur interne
4. **Traits standards** : `Display`, `FromStr`, `Clone`, `Copy` (si le type interne est Copy), `Debug`, `PartialEq`, `Eq`, `Hash`
5. **Conversions** : `TryFrom<String>` pour chaque type validé (délègue à `FromStr` via `s.parse()`), `AsRef<str>` pour les types wrappant un `String`. Requises par l'architecture pour l'interop entre crates.
6. **Serde — CRITIQUE** :
   - **Money** : `#[serde(transparent)]` — délègue à `Decimal` qui sérialise en string via feature `serde-str`
   - **Iban, QrIban, CheNumber** : `#[serde(try_from = "String", into = "String")]` — la désérialisation DOIT passer par le constructeur validant. Ne JAMAIS utiliser `#[derive(Deserialize)]` avec `#[serde(transparent)]` sur ces types car cela bypasse la validation. **Attention :** `into = "String"` exige `impl From<TypeName> for String` — ne pas l'oublier sinon erreur de compilation
7. **Tests** : cas valides, cas invalides, cas limites, sérialisation/désérialisation round-trip (vérifier que la désérialisation rejette les valeurs invalides)

### CoreError — Design

```rust
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum CoreError {
    #[error("IBAN invalide : {0}")]
    InvalidIban(String),
    #[error("QR-IBAN invalide : {0}")]
    InvalidQrIban(String),
    #[error("Numéro IDE invalide : {0}")]
    InvalidCheNumber(String),
    // Seules les erreurs Money, Iban, QrIban, CheNumber dans cette story
}
```

**Important :** Les messages `#[error("...")]` sont pour le **logging serveur uniquement**. `kesh-api` mappe chaque variante vers un code structuré (`INVALID_IBAN`, `INVALID_QR_IBAN`, `INVALID_CHE_NUMBER`) et un message traduit via `kesh-i18n`. Ne jamais exposer le `Display` de `CoreError` au frontend.

Ajouter une méthode utilitaire pour faciliter le mapping dans kesh-api :

```rust
impl CoreError {
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidIban(_) => "INVALID_IBAN",
            Self::InvalidQrIban(_) => "INVALID_QR_IBAN",
            Self::InvalidCheNumber(_) => "INVALID_CHE_NUMBER",
        }
    }
}
```

### Algorithmes de validation — Spécifications exactes

#### IBAN (ISO 13616) — Algorithme MOD-97 (international)

L'IBAN doit supporter tous les pays (pas seulement CH/LI) car CAMT.053 (story 6.1) et pain.001 (story 10.2) contiennent des IBAN étrangers (fournisseurs DE, FR, AT, etc.).

1. Retirer les espaces, mettre en majuscules
2. Extraire le code pays (2 premières lettres), vérifier la longueur via une table de lookup par pays (constante `IBAN_LENGTHS: &[(&str, usize)]` — environ 80 entrées). **Source officielle : SWIFT IBAN Registry (swift.com/standards/data-standards/iban).** Le dev agent DOIT vérifier les longueurs contre ce registre et NE PAS inventer de valeurs. Exemples vérifiés : CH=21, LI=21, DE=22, FR=27, AT=20, IT=27, ES=24, GB=22, BE=16, NL=18, LU=20
3. Vérifier format : 2 lettres + 2 chiffres + alphanumériques
4. Déplacer les 4 premiers caractères à la fin
5. Convertir les lettres en nombres (A=10, B=11, ..., Z=35)
6. Calculer modulo 97 de manière itérative (chunks de 7 chiffres pour éviter l'overflow u64)
7. Résultat doit être exactement 1

**Méthode `is_swiss(&self) -> bool`** : retourne true si le code pays est CH ou LI (utile pour QrIban).
**Méthode `bank_clearing_number(&self) -> Option<&str>`** : retourne `Some(&self.0[4..9])` uniquement pour CH/LI, `None` pour les autres pays (la structure du BBAN varie par pays : DE=8 chiffres Bankleitzahl, FR=5+5 code banque+guichet, etc.).

**Cas de test IBAN suisse :** `CH93 0076 2011 6238 5295 7` (valide, 21 chars)
**Cas de test IBAN allemand :** `DE89 3704 0044 0532 0130 00` (valide, 22 chars)
**Cas de test IBAN français :** `FR76 3000 6000 0112 3456 7890 189` (valide, 27 chars)

#### QR-IBAN — Validation SIX

1. Valider d'abord comme IBAN standard (MOD-97)
2. Vérifier que le pays est CH ou LI
3. Extraire le QR-IID : positions 5-9 (0-indexed: 4..9)
4. Vérifier que QR-IID est dans la plage 30000-31999 (inclus)

**Cas de test QR-IBAN :** `CH44 3199 9123 0008 8901 2` (QR-IID=31999, limite haute)

#### CHE/IDE — Algorithme modulo 11 (eCH-0097 v2.0)

Poids : `[5, 4, 3, 2, 7, 6, 5, 4]` appliqués aux 8 premiers chiffres.

1. Normaliser : majuscules, retirer les espaces, retirer le suffixe `MWST`/`TVA`/`IVA` s'il est présent (fréquent dans les documents commerciaux : `CHE-109.322.551 MWST`)
2. Extraire les 9 chiffres après le préfixe CHE (retirer séparateurs `-` et `.`)
3. Multiplier chaque des 8 premiers chiffres par son poids
4. Sommer les produits
5. `remainder = sum % 11`
6. Si `remainder == 0` → check digit = 0
7. Si `remainder == 1` → numéro **invalide** (check digit serait 10, impossible)
8. Sinon → check digit = `11 - remainder`
9. Comparer avec le 9ème chiffre

**Cas de test officiel :** `CHE-109.322.551` → somme=109, 109%11=10, check=11-10=1 ✓

### Dépendances kesh-core/Cargo.toml

`thiserror` est un choix d'implémentation (non imposé par l'architecture, qui spécifie le pattern `CoreError` + `From<T>` sans imposer de bibliothèque). Ce choix simplifie le boilerplate par rapport à une implémentation manuelle de `Display + Error`.

```toml
[package]
name = "kesh-core"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
rust_decimal = { version = "1.41", features = ["serde-str", "maths"] }
rust_decimal_macros = "1.40"
serde = { version = "1", features = ["derive"] }
thiserror = "2"

[dev-dependencies]
serde_json = "1"
```

**Ne PAS ajouter** : `sqlx` (c'est dans `kesh-db`), `axum` (c'est dans `kesh-api`), `tokio` (zéro I/O).

### Sérialisation JSON — Conventions projet

- Montants : string décimal `"1234.56"` — jamais de float JSON (garanti par `#[serde(transparent)]` + feature `serde-str` sur `Decimal`)
- IBAN/QR-IBAN/CHE : string normalisée sans espaces — via `#[serde(try_from = "String", into = "String")]`
- Champs JSON : camelCase via `#[serde(rename_all = "camelCase")]` au niveau des structs qui les contiennent (pas sur les newtypes eux-mêmes)
- **Désérialisation validante** : un JSON `{"iban": "INVALIDE"}` doit échouer avec une erreur serde contenant le message de `CoreError`. C'est garanti par `try_from = "String"` qui appelle le constructeur validant

### Money — Traits arithmétiques

`Money` est `Copy` (car `Decimal` est `Copy`, 128 bits). Les opérations arithmétiques retournent `Money` directement (pas `Result`) car `Decimal` a un mantisse 96 bits qui ne peut pas overflow pour des montants comptables réalistes.

- `Add<Money>`, `Sub<Money>`, `Neg` → `Money`
- `Mul<Decimal>` → `Money` (pour quantité * prix unitaire, montant * taux TVA)
- `Sum` → `Money` (pour `iter().sum::<Money>()` sur les lignes de facture/écriture)
- Les montants négatifs sont valides (avoirs, contre-passations, soldes créditeurs)

### Formats d'affichage suisses (pour `Display`)

- `Money` : affiche le montant brut (`1234.56`). Le formatage suisse avec apostrophe (`1'234.56`) sera dans `kesh-i18n` (story future), pas dans le type lui-même.
- `Iban` : affiche sans espaces (`CH9300762011623852957`). Méthode `formatted()` pour le format avec espaces (`CH93 0076 2011 6238 5295 7`).
- `CheNumber` : affiche avec séparateurs (`CHE-109.322.551`) via `Display`.

### Règles obligatoires (architecture doc)

1. **Jamais de f64 pour les montants** — `rust_decimal::Decimal` exclusivement
2. **Tout code public documenté** — `///` Rust
3. **Tests unitaires pour toute logique métier** — pas de code métier sans test
4. **Erreurs structurées avec code métier** — jamais de string d'erreur en dur

### Project Structure Notes

- `crates/kesh-core/src/lib.rs` existe (placeholder `//! Crate placeholder`), à remplacer
- `crates/kesh-core/Cargo.toml` existe (vide de dépendances), à compléter
- Les modules `types/` et `errors.rs` sont nouveaux
- Aucun conflit avec l'existant — `kesh-core` est entièrement vierge

### Learnings des stories précédentes (1.1, 1.2)

- **tower-http** : version 0.6.x (pas 0.5.x) pour compatibilité Axum 0.8 — ne concerne pas cette story mais illustre l'importance de vérifier les versions
- **Pattern d'erreur établi** : `ConfigError` dans story 1.2 utilise un enum custom avec `Display + Error` — cette story utilise `thiserror` pour simplifier
- **Tests co-localisés** : confirmé fonctionnel dans `config.rs` (story 1.2)
- **Commit pattern** : `feat:` pour nouvelles fonctionnalités, mentionner le numéro de story

### Anti-patterns à éviter

- **NE PAS** utiliser un crate externe pour la validation IBAN/CHE — les algorithmes sont simples (~50 lignes chacun) et critiques pour le métier, garder le contrôle total
- **NE PAS** ajouter de dépendance I/O dans kesh-core (pas de sqlx, tokio, reqwest, etc.)
- **NE PAS** implémenter le formatage suisse (apostrophe milliers) dans `Money` — c'est la responsabilité de `kesh-i18n`
- **NE PAS** utiliser `unwrap()` dans le code de production — uniquement dans les tests
- **NE PAS** exposer les champs internes des newtypes — toujours passer par les constructeurs validants
- **NE PAS** créer `UserId`, `CompanyId`, `Role`, `Version`, `AccountId` ou tout autre type domaine — ces types sont dans les stories 1.4, 1.7, 1.8
- **NE PAS** créer les modules `accounting/`, `chart_of_accounts/`, `validation/` — ils viendront dans les stories 3.x et au-delà

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Structure Complète du Répertoire] — Structure crates/kesh-core
- [Source: _bmad-output/planning-artifacts/architecture.md#Décisions Architecturales] — Séparation core/db, types forts
- [Source: _bmad-output/planning-artifacts/architecture.md#Naming Patterns] — Conventions de nommage
- [Source: _bmad-output/planning-artifacts/architecture.md#Versions Vérifiées] — rust_decimal 1.39+ (mis à jour 1.41)
- [Source: _bmad-output/planning-artifacts/architecture.md#Gestion des erreurs Rust] — CoreError pattern
- [Source: _bmad-output/planning-artifacts/prd.md#Conformité & réglementaire] — Standards SIX, IDE CHE, arithmétique exacte
- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.3] — Acceptance criteria
- [Source: eCH-0097 v2.0] — Algorithme checksum IDE/CHE (modulo 11, poids 5,4,3,2,7,6,5,4)
- [Source: ISO 13616] — Algorithme validation IBAN (MOD-97)
- [Source: SIX Technical Information QR-IID] — Plage QR-IID 30000-31999
- [Source: SWIFT IBAN Registry] — Table officielle des longueurs IBAN par pays (swift.com/standards/data-standards/iban)

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Debug Log References

- QR-IBAN test borne basse (`CH78...`) avait un checksum MOD-97 invalide → recalculé en `CH57 3000 0123 4567 8901 2`
- rust_decimal 1.41.0 et rust_decimal_macros 1.40.0 confirmés disponibles sur crates.io
- Table IBAN_LENGTHS : 87 entrées basées sur le registre SWIFT IBAN

### Completion Notes List

- 4 newtypes implémentés : Money, Iban, QrIban, CheNumber
- 73 tests unitaires + 4 doc-tests, tous passent
- Clippy 0 warnings, docs générées sans warnings
- Workspace complet compile sans erreur
- Serde : Money utilise `#[serde(transparent)]`, les 3 autres `#[serde(try_from, into)]` avec validation
- IBAN supporte 87 pays via table de lookup
- CheNumber accepte minuscules et suffixes MWST/TVA/IVA
- CoreError avec `error_code()` pour le mapping API structuré

### Change Log

- 2026-04-04 : Implémentation complète story 1.3 — 4 newtypes avec validation, 77 tests
- 2026-04-04 : Revue de code adversariale — 9 findings corrigés (2 HIGH, 6 MED, 1 LOW). 82 tests au total.

### File List

- crates/kesh-core/Cargo.toml (modifié)
- crates/kesh-core/src/lib.rs (modifié)
- crates/kesh-core/src/errors.rs (nouveau)
- crates/kesh-core/src/types/mod.rs (nouveau)
- crates/kesh-core/src/types/money.rs (nouveau)
- crates/kesh-core/src/types/iban.rs (nouveau)
- crates/kesh-core/src/types/qr_iban.rs (nouveau)
- crates/kesh-core/src/types/che_number.rs (nouveau)
