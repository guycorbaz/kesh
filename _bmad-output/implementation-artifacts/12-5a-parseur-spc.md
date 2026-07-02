# Story 12.5a: Parseur SPC (socle kesh-qrbill)

Status: done

<!-- Sous-story 1/4 de l'umbrella 12-5 (validate convergé 6 passes). Story-zéro : pose le socle parseur, aucune dép native ni DB. Réutilisable par 12-4 (scan manuel) différé. -->

## Story

As a développeur de l'import de factures (12-5),
I want une fonction `parse_spc_payload` qui décode un texte de payload Swiss QR (SPC) en structure typée,
so that les couches supérieures (décodage image/PDF, service d'import) disposent du socle de parsing, inverse exact du générateur, robuste aux QR émis par des logiciels tiers.

## Contexte & source

- Sous-story **1/4** de l'umbrella **12-5** (import répertoire factures, issue #194). Cf. `12-5-import-repertoire-factures.md` (AC1, DC figés, Dev Notes ground-truth).
- **Socle commun** avec 12-4 (scan QR manuel, différé) — le parseur livré ici sera réutilisé.
- Aucune dépendance native (pdfium/rxing) ni DB : pure logique de chaîne dans `crates/kesh-qrbill`. **À implémenter en premier** (série a→b→c→d).

## Acceptance Criteria

1. **`parse_spc_payload(text: &str) -> Result<ScannedQrBill, QrBillError>`** (nouveau module `crates/kesh-qrbill/src/parser.rs`, exporté dans `lib.rs`). Inverse exact de `generator.rs::build_payload` (mêmes index de lignes SIX 2.2 §3).

2. **Types exportés** (`lib.rs`) :
   ```rust
   pub struct ScannedQrBill {
       pub creditor_iban: String,        // normalisé (sans espaces, majuscules)
       pub is_qr_iban: bool,             // IID 30000–31999
       pub creditor: ScannedAddress,
       pub amount: Option<Decimal>,      // None si champ montant vide (open amount)
       pub currency: String,             // "CHF"|"EUR" (autres tolérées, validées en aval)
       pub reference: ScannedReference,
       pub unstructured_message: Option<String>,
       pub billing_information: Option<String>,
   }
   pub struct ScannedAddress {
       pub address_type: char,           // 'K' ou 'S'
       pub name: String,
       pub street_or_line1: String,
       pub building_or_line2: String,
       pub postal_code: Option<String>,  // type S
       pub town: Option<String>,         // type S
       pub country: String,
   }
   pub enum ScannedReference { Qrr(String), Scor(String), None }
   ```

3. **Validation & robustesse** (réutilise `validation.rs`, ne duplique pas) :
   - En-tête : ligne 0 == `SPC` ET ligne 1 == `0200`, sinon `QrBillError::InvalidPayload` (nouveau variant, mappé `INVALID_SPC_PAYLOAD` en aval). Trailer `EPD` attendu à l'index 30.
   - **IBAN** (ligne 3) : `validate_iban` → `creditor_iban` normalisé ; `is_qr_iban` = vrai si IID `[4..9] ∈ [30000,31999]` (cf. `validate_qr_iban`). IBAN invalide → `QrBillError::InvalidIban`/`InvalidQrIban`.
   - **Adresse créancier (type K ou S)** : `address_type` = 1ᵉʳ char de la ligne 4. K : `postal_code`/`town` = `None` (lignes 8/9 vides). S : `postal_code = Some(ligne 8)`, `town = Some(ligne 9)`. `street_or_line1` = ligne 6, `building_or_line2` = ligne 7, `country` = ligne 10. (Kesh n'émet que K ; les tiers émettent souvent S → les deux DOIVENT être gérés.)
   - **Montant** (ligne 18) : vide → `None` ; sinon `Decimal::from_str` → erreur `InvalidAmount` si non-parsable.
   - **Devise** (ligne 19) : conservée brute (validée CHF/EUR en aval, AC7 umbrella).
   - **Référence** : type ligne 27, valeur ligne 28. `QRR` → `validate_qrr(valeur)?` puis `ScannedReference::Qrr` ; `SCOR` → `ScannedReference::Scor` (toléré, pas émis par Kesh) ; `NON` → `ScannedReference::None` ; autre → `InvalidPayload`.
   - **unstructured_message** (ligne 29) : `None` si vide.
   - **Nombre de lignes variable (31/32/34)** : `billing_information` = ligne 31 si présente et non-vide, sinon `None`. **AltPmtInf** (lignes 32-33) lu si présent mais **ignoré** (pas stocké). Longueur minimale acceptée : 31 (indices 0..30). Robuste aux champs de fin absents (pas d'indexation en dur sur 32).
   - **Bloc débiteur final** (lignes 20-26) : non requis pour l'import → non exposé dans `ScannedQrBill` (le créancier + montant + référence suffisent).

4. **Tests** (`parser.rs` `#[cfg(test)]`) :
   - **Round-trip type K** : `build_payload(QrBillData)` → `parse_spc_payload` → vérifier IBAN/montant/devise/référence/créancier identiques (Kesh n'émet que K).
   - **Type S** : fixture string SPC type S construite à la main → vérifier `address_type='S'`, `postal_code`/`town` `Some`.
   - **Variantes de lignes** : 31 (billing absent), 32 (billing présent), 34 (AltPmtInf présent, ignoré).
   - **Montant vide** → `amount=None`.
   - **Références** : QRR valide (round-trip), QRR invalide → erreur, `NON` → `ScannedReference::None`.
   - **QR-IBAN** → `is_qr_iban=true` ; IBAN normal → `false`.
   - **Erreurs** : en-tête non-SPC → `InvalidPayload` ; IBAN invalide → `InvalidIban`.

5. **Quality gate** (Test Locally First, kesh-qrbill) **exit code vérifié** : `cargo fmt -p kesh-qrbill -- --check` + `cargo clippy -p kesh-qrbill --all-targets -- -D warnings` + `cargo test -p kesh-qrbill`. 0 régression sur les tests existants du crate.

## Tasks / Subtasks

- [x] **T1** — Ajouter variant `QrBillError::InvalidPayload(String)` (`types.rs`) + mapping doc INVALID_SPC_PAYLOAD.
- [x] **T2** — `src/parser.rs` : `ScannedQrBill`/`ScannedAddress`/`ScannedReference` + `parse_spc_payload` (indices SIX, type K/S, lignes variables, réutilise `validate_iban`/`validate_qr_iban`/`validate_qrr`).
- [x] **T3** — Exporter dans `lib.rs` (`pub mod parser; pub use parser::{...};`).
- [x] **T4** — Tests round-trip K + fixture S + variantes lignes + erreurs.

## Dev Notes

### Ground-truth (vérifié 2026-06-29)
- Ordre des lignes (0-indexé, `generator.rs:14-90`) : `[0]SPC [1]0200 [2]1 [3]IBAN [4]créancier AdrTp(K) [5]name [6]line1 [7]line2 [8]PstCd("") [9]TwnNm("") [10]country [11..17]ultimate creditor (7 vides) [18]amount [19]currency [20..26]ultimate debtor (7) [27]ref tp [28]ref value [29]unstructured_message [30]EPD [31]billing_information`. AltPmtInf (`[32][33]`) omis par Kesh.
- `validation.rs` : `validate_iban(&str)->Result<String,_>` (normalise+CH/LI+mod97), `validate_qr_iban(&str)->Result<String,_>` (IID `[4..9]∈[30000,31999]`), `validate_qrr(&str)->Result<(),_>` (27 chiffres + mod-10 récursif). `normalize_iban` dispo. **Réutiliser**, ne pas dupliquer.
- `QrBillError` (`types.rs:227`) : variants existants `InvalidIban`/`InvalidQrIban`/`InvalidQrr`/`InvalidAmount`/`InvalidCurrency`/... → **ajouter `InvalidPayload(String)`**.
- `Decimal` = `rust_decimal` (déjà dans le crate). `amount` parsé via `Decimal::from_str` (format payload = `123.45` sans séparateur de milliers).
- Détection K/S : Kesh n'émet que `AddressType::Combined` (K, `types.rs`). Le parseur doit gérer S pour les QR tiers (même bloc 7 lignes, interprétation des champs 8/9 différente).

### Conventions
- **Test Locally First** exit code vérifié (PAS `cargo test | grep`).
- **Branche** : `story/12-5-import-repertoire-factures` (umbrella).
- Pas de DB, pas de dép native dans cette sous-story.

### References
- [Source: 12-5-import-repertoire-factures.md AC1] — définition `ScannedQrBill`, robustesse type S / lignes variables.
- [Source: generator.rs:14-90] — ordre des lignes (miroir). [validation.rs] — IBAN/QR-IBAN/QRR.

## Dev Agent Record

### Agent Model Used

Opus 4.8 (claude-opus-4-8[1m]). Implémentation interrompue par un crash système puis reprise — intégrité git + working tree vérifiée (0 octet nul, UTF-8 valide, `git fsck` propre) avant complétion du quality gate.

### Debug Log References

- Quality gate kesh-qrbill (exit codes vérifiés) : `cargo fmt -p kesh-qrbill -- --check` = 0, `cargo clippy -p kesh-qrbill --all-targets -- -D warnings` = 0, `cargo test -p kesh-qrbill` = 0 (47 tests, dont 10 nouveaux `parser::tests::*`).
- Workspace : `cargo fmt --all --check` = 0, `cargo clippy --workspace --all-targets -D warnings` = 0, `cargo test --workspace -j1 -- --test-threads=1` (serial DB).

### Completion Notes List

- **T1** : variant `QrBillError::InvalidPayload(String)` ajouté (`types.rs`), doc « mappé `INVALID_SPC_PAYLOAD` par la couche d'import (Story 12-5) ». Bras `map_qrbill_error` ajouté dans `kesh-api/src/routes/invoice_pdf.rs` (le match exhaustif sur `QrBillError` aurait cassé la compilation sinon) → mappé `AppError::InvoiceNotPdfReady` (chemin import, n'arrive pas en génération PDF).
- **T2** : `parser.rs` — `parse_spc_payload` miroir exact de `generator::build_payload` (constantes d'index `L_*`). Type K/S géré (S → `postal_code`/`town` `Some`). Robuste CRLF (`trim_end_matches('\r')`), lignes variables (`MIN_LINES=31`, `billing_information` via `lines.get`), AltPmtInf lu mais ignoré. Réutilise `validate_iban`/`validate_qrr` (pas de duplication). `is_qr_iban` via IID `[4..9] ∈ [30000,31999]`.
- **T3** : `lib.rs` exporte `pub mod parser;` + `pub use parser::{ScannedAddress, ScannedQrBill, ScannedReference, parse_spc_payload};`.
- **T4** : 10 tests — round-trip K (sans réf + QRR), type S construit à la main, variantes 31/32/34 lignes, montant vide, en-tête non-SPC, IBAN invalide, QRR checksum corrompu, type de réf inconnu, trop peu de lignes.
- **Décision d'implémentation** : le parseur utilise `validate_iban` (pas `validate_qr_iban`) puis détecte le QR-IBAN via l'IID, afin d'accepter aussi bien les IBAN normaux que les QR-IBAN des émetteurs tiers (un `validate_qr_iban` strict rejetterait un IBAN normal valide). `is_qr_iban` reste exact par la plage IID.

### File List

- `crates/kesh-qrbill/src/parser.rs` (nouveau)
- `crates/kesh-qrbill/src/lib.rs` (exports parser)
- `crates/kesh-qrbill/src/types.rs` (variant `InvalidPayload`)
- `crates/kesh-api/src/routes/invoice_pdf.rs` (bras `map_qrbill_error`)

## Change Log

### Code review (2026-06-29) — CONVERGÉ Pass 1 (0 > LOW)

Cycle adversarial 3 couches parallèles **Opus 4.8** (Blind Hunter / Edge Case Hunter / Acceptance Auditor) sur le diff `97abfef..85a235b`. **Trend > LOW : Pass 1 = 0** → critère d'arrêt Review Iteration Rule atteint dès la 1ʳᵉ passe (pas de 2ᵉ passe nécessaire).

**Verdicts** :
- **Blind Hunter** (diff seul) : 0 > LOW. Confirme parser panic-safe, mapping `InvalidPayload → InvoiceNotPdfReady` exhaustif, aucun `?` avalé.
- **Edge Case Hunter** (diff + ground-truth) : 0 > LOW. **Vérifie le miroir d'index `L_*` contre `generator.rs` champ par champ — exact** (vecteur de corruption silencieuse le plus à risque : propre). Slice `creditor_iban[4..9]` prouvablement sûr (`validation.rs:47` garantit `len==21` + ASCII avant retour).
- **Acceptance Auditor** (diff + spec) : 0 > LOW. AC1-AC5 satisfaites. Confirme que `validate_iban` + détection IID **n'est pas une déviation** (le `(cf. validate_qr_iban)` de l'AC3 est une référence croisée, pas une instruction d'appel).

**Décisions de reclassement (toutes les remontées étaient LOW)** :
- *dismiss* — Blind F1 (slice IBAN) : réfuté par ground-truth Edge (`len==21` garanti). Blind F2 (fallback K silencieux) : SIX impose majuscule, lenience intentionnelle. Auditor LOW-1 (S postal/town vide→None) : code plus robuste que le texte de spec, champ `Option`, inerte. Auditor LOW-2 (`InvalidQrIban` inatteignable) : conséquence du choix `validate_iban` prescrit. Edge L4 (BOM) : rejet sûr sans corruption.
- *defer → 12-5b/c* — Edge L1 (montant signe/échelle), L2 (cross-check QR-IBAN↔référence SIX §3.3), L3 (currency/SCOR/ustrd verbatim) : validations métier **déléguées en aval par design** (AC umbrella « validées en aval »). À enforcer dans la couche import 12-5b/c — déjà tracé dans la spec umbrella 12-5.

Aucun patch de code appliqué (convergence propre). Statut → `done`.
