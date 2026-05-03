---
title: "Spike — `kesh-import` crate design"
date: 2026-05-03
status: completed
verdict: feasible
relatedEpic: 8
relatedStory: 8-1
relatedDecisions:
  - "Architecture decision #6 (crates autonomes)"
  - "Architecture decision #7 (types autonomes + From/Into)"
relatedIssues:
  - "#62 CR-010 (statement balance check)"
sourceContext: epic-7-retro-2026-05-01.md (prep sprint scope, item #4 critical path)
---

# Spike — `kesh-import` crate design

## Contexte

Le retro Epic 7 (2026-05-01) a identifié comme **critical path bloquant Story 8-1** la validation du choix architectural pour `kesh-import` :

- **Crate publiable indépendante** sur crates.io
- **Zéro dépendance interne** sur les autres crates Kesh (`kesh-core`, `kesh-db`, `kesh-api`, etc.)
- **Types autonomes** dans `kesh-import`, conversion vers les types domaine `kesh-core` via `From`/`Into` côté `kesh-core`
- Direction de dépendance Cargo : `kesh-core → kesh-import` (jamais l'inverse)

Le risque adressé par ce spike : découvrir mid-Story 8-1 qu'un type `kesh-core` traîne une trait dérivée incompatible (ex. `sqlx::FromRow`, validateur stateful) qui force `kesh-import` à dépendre d'une dépendance transversale, invalidant le choix architectural.

## Périmètre du spike

Conformément à l'option « Hybride » validée 2026-05-03 :

1. **Inventaire** des types `kesh-core` que Story 8-1 va convertir
2. **Définition** des types autonomes `kesh-import` correspondants
3. **POC code minimal** : 1 paire de types + impl `From`
4. **Vérification** : `cargo build`, `cargo test`, `cargo publish --dry-run`
5. **Decision doc** (ce fichier)

Hors périmètre :

- Implémentation des parseurs CAMT.053 et CSV (Stories 8-1 et 8-2)
- Modèle DB `bank_imports` / `bank_transactions` (Story 8-1)
- Intégration HTTP / route `kesh-api/routes/bank_imports.rs` (Story 8-1)

## Inventaire des types

### Côté `kesh-core` (cible des conversions)

`kesh-core` contient aujourd'hui (2026-05-03) :

| Module | Types pertinents pour Story 8-1 |
|---|---|
| `types::Money` | wrapper `Decimal` infaillible, idéal pour les montants importés |
| `types::Iban` | validation MOD-97 + longueurs ISO 13616 — **fallible** (`Result<Iban, CoreError>`) |
| `types::QrIban` | validation IID 30000-31999 — fallible |
| `types::CheNumber` | numéro IDE suisse — non utilisé en import bancaire |
| `accounting::JournalEntryDraft` | écriture comptable validée — utilisée par Story 8-4 (réconciliation), pas Story 8-1 |
| `errors::CoreError` | enum d'erreurs métier |

**Aucun type `BankTransaction` ou `BankImport` n'existe encore** — Story 8-1 les introduira dans un nouveau module `kesh-core::bank_imports`. Ce spike a posé une **scaffold minimale** (`BankTransactionDraft`) que Story 8-1 étendra.

### Côté `kesh-import` (types définis par ce spike)

Trois types autonomes dans `kesh-import::types` :

| Type | Rôle | Champs notables |
|---|---|---|
| `ImportedStatement` | Un relevé complet (= un `<Stmt>` CAMT.053 ou un fichier CSV) | `account_iban`, `opening_balance`, `closing_balance`, `period_from`, `period_to`, `transactions` |
| `ImportedTransaction` | Une transaction unitaire | `booking_date`, `amount` (signé), `currency`, `reference`, `details`, `end_to_end_id`, `transaction_id`, `counterparty_iban`, `counterparty_name` |
| `SourceFormat` | Provenance audit | enum `Camt053 { version }` ou `Csv { encoding, profile_name }` |

**Décisions de design :**

- `account_iban` et `counterparty_iban` sont des **`String` brutes** (non validées) : la validation MOD-97 vit dans `kesh-core::types::Iban`. Cela permet un import « tolérant » (transaction conservée même si l'IBAN est mal formé, statut d'avertissement) plutôt qu'un rejet strict du fichier entier — décision à finaliser Story 8-1.
- `amount` est signé selon la convention CAMT.053 (positif = crédit du compte titulaire). Les profils CSV (Story 8-2) doivent normaliser dans cette convention.
- `transaction_id` (= `<AcctSvcrRef>` CAMT.053) capturé pour permettre la détection de doublons stricte (Story 8-3).
- `opening_balance` et `closing_balance` sont `Option<Decimal>` : ils ne sont pas tous présents dans tous les CSV. Le check de cohérence FR42-bis (CR-010 #62) ne s'appliquera qu'aux relevés où `closing_balance.is_some()`.
- Méthode utilitaire `ImportedStatement::sum_transactions()` fournie pour le check CR-010 (somme algébrique vs `closing_balance - opening_balance`).

### Dépendances externes choisies

`kesh-import/Cargo.toml` :

| Dépendance | Justification |
|---|---|
| `chrono = "0.4"` (feature `serde`) | `NaiveDate` pour `booking_date`, `value_date`, période — même version que `kesh-core` |
| `rust_decimal = "1.41"` (feature `serde-str`) | Précision décimale exacte sur les montants — même version que `kesh-core` |
| `serde = "1"` (feature `derive`) | Round-trip JSON pour API REST + tests |
| `thiserror = "2"` | Erreurs typées pour les futurs parseurs CAMT/CSV (Stories 8-1/8-2) |

**Dépendances ajoutées Stories 8-1/8-2 (hors spike) :** `quick-xml` (CAMT.053), `csv` + `encoding_rs` (CSV multi-encodage).

**Aucune dépendance workspace interne.** Vérifié : `kesh-import/Cargo.toml` ne contient aucune entrée `path = "../kesh-*"` ni `kesh-*.workspace = true`.

## Mécanique `From` / `Into` (POC validé)

Direction : **`kesh-core` dépend de `kesh-import`** (Cargo path dep ajoutée à `kesh-core/Cargo.toml`).

Les impls vivent dans `kesh-core::bank_imports` :

```rust
impl From<ImportedTransaction> for BankTransactionDraft { ... }
impl From<&ImportedTransaction> for BankTransactionDraft { ... }
```

La variante `&ImportedTransaction → BankTransactionDraft` (par référence) est fournie pour permettre la réutilisation d'un même `ImportedStatement` sans clone explicite côté appelant — utile si Story 8-1 veut produire à la fois le draft pour persistance et un payload de prévisualisation pour le frontend à partir du même parsed result.

**Asymétrie conservée :** `kesh-import` ne référence aucun type `kesh-core`. La conversion inverse (`BankTransactionDraft → ImportedTransaction`) n'est pas implémentée car non utile (l'import est unidirectionnel : fichier → DB).

## Vérification

```sh
$ cargo build -p kesh-import
    Finished dev profile

$ cargo test -p kesh-import
test result: ok. 7 passed; 0 failed

$ cargo test -p kesh-core bank_imports
test result: ok. 4 passed; 0 failed; 134 filtered out

$ cargo build --workspace
    Finished dev profile

$ cargo publish --dry-run --allow-dirty -p kesh-import
   Packaging kesh-import v0.1.0 (...)
   Verifying kesh-import v0.1.0 (...)
    Finished dev profile
   Uploading kesh-import v0.1.0 (...)
warning: aborting upload due to dry run
```

Tous les checks passent. Aucune régression sur les 138 tests `kesh-core` ni sur le build du workspace complet (`kesh-api`, `kesh-db`, `kesh-seed` inclus).

## Verdict

✅ **Feasible — décision architecture #7 confirmée.**

`kesh-import` est publiable indépendamment, ses types sont autonomes, et la mécanique `From`/`Into` côté `kesh-core` fonctionne sans accroc. Aucune dépendance transitive non désirée n'est apparue à la frontière.

## Implications Story 8-1

Le scaffold `kesh-core::bank_imports::BankTransactionDraft` introduit par ce spike est **volontairement minimal** :

- Contient uniquement les champs portés par `ImportedTransaction` (pas de FK `bank_account_id`, `import_id`, `company_id`).
- Pas de validation IBAN via `kesh-core::types::Iban` (champ `String` brut).
- Pas de gestion des références ESR/QR-Référence (champ `Option<String>` brut).
- Pas de mapping de la devise (champ `String` brut, devrait probablement devenir un type `Currency` validé Story 8-1).

Story 8-1 doit :

1. **Étendre `BankTransactionDraft`** pour inclure les FK (`bank_account_id`, `import_id`, `company_id`) — ou introduire un type intermédiaire `PersistableBankTransaction` au-dessus du draft.
2. **Décider la stratégie IBAN** : tolérant (conserver `String` brut, statut avertissement) vs strict (passer par `Iban::new`, rejet de la transaction si invalide). Recommandation : tolérant + statut, pour ne pas perdre une transaction à cause d'une coquille bancaire.
3. **Décider la stratégie devise** : pour v0.1, accepter uniquement `"CHF"` (PRD §FR42 implicite via cible PME suisse) ; rejeter les transactions multi-devises avec un message explicite.
4. **Implémenter le check CR-010 (#62)** : `ImportedStatement::sum_transactions() == closing_balance - opening_balance ± 0.01` quand les deux soldes sont présents. Erreur métier `BankImportError::BalanceMismatch { expected, actual, diff }` à ajouter à un nouveau module `kesh-import::error` (ou `kesh-core::errors::CoreError`).
5. **Choisir l'emplacement des From/Into** : au fur et à mesure que Story 8-1 ajoute `BankImportDraft` (méta-fichier) et `BankTransactionPersistable`, conserver l'invariant « impls dans `kesh-core`, jamais dans `kesh-import` ».

## Risques résiduels

| # | Risque | Probabilité | Mitigation |
|---|---|---|---|
| RS-1 | Le crate `kesh-import` finit par dépendre d'un crate workspace via une des futures features (`quick-xml` schémas Kesh, etc.) | Faible | Convention CI à instaurer Story 8-1 : test de non-régression `cargo metadata -p kesh-import --format-version 1 \| jq '.packages[0].dependencies[] \| select(.path != null)'` — doit retourner vide |
| RS-2 | La signature unique `amount: Decimal` ne capture pas les montants à devise multiple (CAMT.053 multi-currency `<Amt Ccy>`) si Kesh étend v0.2 hors CHF | Moyenne | Ajouter un field `currency: String` au niveau transaction (déjà fait). Pour v0.2, introduire un type `MoneyWithCurrency` côté `kesh-core` |
| RS-3 | Les futurs parseurs (Story 8-1) pourraient avoir besoin d'un type `Position` (offset + ligne du fichier source) pour les messages d'erreur, nécessitant un refactor des structs | Faible | Story 8-1 décide ; ajout backward-compat possible (`pub source_position: Option<Position>` avec default serde) |
| RS-4 | La direction `kesh-core → kesh-import` rend `kesh-core` un peu moins « pur » (jusqu'ici aucune dep workspace) | Acceptée | Tradeoff délibéré ; `kesh-core` reste sans I/O (`kesh-import` n'a pas non plus d'I/O dans cette première version) |

## Suivi

- ✅ Item #4 du critical path Epic 8 (retro 2026-05-01) clos par ce spike.
- ✅ Crate `kesh-import` opérationnelle avec API publique (`ImportedStatement`, `ImportedTransaction`, `SourceFormat`).
- ✅ Scaffold `kesh-core::bank_imports::BankTransactionDraft` posé pour Story 8-1.
- 🟡 CR-010 #62 (statement balance check) : structure prête (`ImportedStatement::sum_transactions()`), implémentation effective déléguée à Story 8-1.
- 🟡 CR-009 #61 (renumérotage `epics.md`) : non bloquant pour Story 8-1 spec, à traiter avant `epic-9.md`.

**Prochaine étape critical path Epic 8 :** la totalité du critical path est désormais close (5/5). Le sprint prep peut basculer en mode cleanup parallèle (#54, #55, #57) ou enchaîner directement avec `/bmad-create-story 8-1`.

## Références

- `crates/kesh-import/src/types.rs` — types autonomes
- `crates/kesh-core/src/bank_imports.rs` — scaffold conversion
- [Architecture §11.5 et §17](../planning-artifacts/architecture.md)
- [`epic-8.md`](../planning-artifacts/epic-8.md) — Story 8-1 ACs (notamment AC `kesh-import publiable indépendante`)
- [`epic-7-retro-2026-05-01.md`](epic-7-retro-2026-05-01.md) — prep sprint critical path item #4
- [Issue #62 CR-010](https://github.com/guycorbaz/kesh/issues/62) — statement balance check à intégrer Story 8-1
