# Change Requests — Kesh

> **📌 Migration 2026-04-16** : ce fichier est **archivé**. Les CR sont désormais suivis sur [GitHub Issues](https://github.com/guycorbaz/kesh/issues?q=label%3Aenhancement) conformément à la règle `CLAUDE.md` « Issue Tracking Rule ». Tout nouveau CR doit être créé via le template `feature_request.yml`.
>
> Les 8 CR historiques ci-dessous ont été migrés sur GitHub le 2026-04-16 (tous non-implémentés à cette date). Ce fichier reste la trace offline de l'origine des CR.

## Statut global

| CR | Issue GitHub | Statut implémentation | Commentaire |
|----|--------------|------------------------|-------------|
| CR-001 | [#8](https://github.com/guycorbaz/kesh/issues/8) | 📅 planifié | Absorbé par Epic 7 (Import Bancaire & Réconciliation) |
| CR-002 | [#9](https://github.com/guycorbaz/kesh/issues/9) | ❌ non planifié | Post-MVP |
| CR-003 | [#10](https://github.com/guycorbaz/kesh/issues/10) | ❌ non planifié | Post-MVP |
| CR-004 | [#11](https://github.com/guycorbaz/kesh/issues/11) | ❌ non planifié (explicite) | Post-MVP |
| CR-005 | [#12](https://github.com/guycorbaz/kesh/issues/12) | 🔴 open | Story 5-3 dette spec (ciseaux ✂) |
| CR-006 | [#13](https://github.com/guycorbaz/kesh/issues/13) | 🔴 open | Story 5-3 dette spec (namespace FTL) |
| CR-007 | [#14](https://github.com/guycorbaz/kesh/issues/14) | 🔴 open | Story 5-3 dette spec (nomenclature FTL) |
| CR-008 | [#15](https://github.com/guycorbaz/kesh/issues/15) | 🔴 open | Story 5-3 dette test (golden PDF Plan C acté) |

**Bilan 2026-04-16** : 0 / 8 CR implémentés. Tous migrés sur GitHub.

---

## Contenu historique (pour référence)

### CR-001 : Réconciliation bancaire

Pouvoir réconcilier les écritures comptables avec les importations de fichiers bancaires (CAMT, MT940, CSV). Rapprochement automatique et/ou manuel entre les transactions importées et les écritures saisies.

### CR-002 : Dashboard configurable

À la connexion, afficher un tableau de bord configurable présentant les valeurs clefs de la comptabilité (soldes des comptes principaux, trésorerie, résultat courant, factures en attente, etc.). L'utilisateur peut choisir quels indicateurs afficher et leur disposition.

### CR-003 : Gestion de stocks simplifiée

Gestion de stocks basique intégrée : suivi des articles, entrées/sorties, valorisation du stock. Adaptée aux besoins d'un indépendant ou d'une petite structure, sans la complexité d'un ERP.

### CR-004 : Calcul d'amortissements (post-MVP)

Gestion des amortissements annuels par investissement. L'utilisateur enregistre un actif (ex : véhicule d'entreprise acheté le 03.03.2026), choisit un type d'amortissement pré-configuré, et Kesh calcule automatiquement les amortissements à déduire et la valeur résiduelle pour chaque exercice.

**Fonctionnalités :**
- Types d'amortissement pré-configurés selon les taux AFC (véhicules, mobilier, informatique, machines, immobilier, etc.)
- Méthode linéaire (montant constant sur la durée) et dégressive (pourcentage sur la valeur résiduelle)
- Calcul au prorata temporis (achat en cours d'année → amortissement proportionnel)
- Tableau d'amortissement par actif : valeur d'acquisition, amortissements cumulés, valeur résiduelle par exercice
- Génération automatique des écritures d'amortissement en fin d'exercice
- Possibilité de créer des types d'amortissement personnalisés (taux et durée libres)

### CR-005 : Symbole ciseaux ✂ sur la ligne de séparation QR Bill (SIX 2.2 §5.3)

**Origine** : Story 5.3 code review 2026-04-15 (BS3 — bad_spec).

**Contexte** : SIX QR Bill 2.2 §5.3 impose l'affichage d'un symbole ciseaux sur la ligne pointillée séparant la section facture de la section paiement. L'implémentation v0.1 utilise Helvetica (built-in printpdf, encodage WinAnsi), qui n'encode pas le glyph ciseaux (U+2702). Le symbole est donc absent du PDF généré (cf. `crates/kesh-qrbill/src/pdf.rs::draw_separator`, commentaire explicite).

**Scope** :
- Remplacer la police Helvetica par Liberation Sans (ou équivalente libre) avec embedding UTF-8 dans le PDF.
- Ajouter le glyph ✂ sur la ligne de séparation.
- Bénéfice collatéral : support correct de `é è ü €` et autres caractères Latin étendu pour les noms / adresses des entreprises.
- Régénérer le golden test (la taille PDF changera du fait de l'embedding, ~+100 KB attendus).

**Référence SIX** : `docs/six-references/ig-qr-bill-v2.4-en.pdf` §5.3.

### CR-006 : Namespace Fluent dédié `invoice-pdf.ftl`

**Origine** : Story 5.3 code review 2026-04-15 (BS2 — bad_spec).

**Contexte** : La spec de la Story 5.3 prévoyait un fichier `crates/kesh-i18n/locales/{fr,de,it,en}/invoice-pdf.ftl` dédié aux traductions du PDF QR Bill. Le loader actuel (`crates/kesh-i18n/src/loader.rs`) ne charge qu'un unique `messages.ftl` par locale ; les ~31 clés `invoice-pdf-*` ont donc été fusionnées dans `messages.ftl`.

**Scope** :
- Étendre `I18nBundle::load` pour charger plusieurs fichiers `.ftl` par locale (glob `*.ftl`).
- Déplacer toutes les clés `invoice-pdf-*` dans `locales/{fr,de,it,en}/invoice-pdf.ftl`.
- Garder le pattern réutilisable pour futures features (`bank.ftl`, `reports.ftl`…).
- Pas de breaking change côté API consommatrice (mêmes clés, même résolution).

### CR-007 : Aligner la nomenclature des clés FTL sur la spec Story 5.3 §11

**Origine** : Story 5.3 code review 2026-04-15 (BS1 — bad_spec).

**Contexte** : La spec §11 référence les clés `invoice-pdf-error-not-validated`, `invoice-pdf-error-not-pdf-ready`, `invoice-pdf-error-invalid-iban`, `invoice-pdf-error-too-many-lines`. L'implémentation utilise un préfixe plus long (`invoice-pdf-error-invoice-*`) et la clé `invoice-pdf-error-invalid-iban` est absente (erreurs IBAN reclassées vers `INVOICE_NOT_PDF_READY`).

**Scope** :
- Renommer les 4 clés concernées dans les 4 fichiers FTL (fr/de/it/en).
- Ajouter la clé `invoice-pdf-error-invalid-iban` si on veut distinguer ce cas côté UX.
- Adapter la construction dynamique côté frontend `+page.svelte` (`err.code.toLowerCase().replace(/_/g, '-')`) si la convention de mapping change.
- **Risque** : breaking change pour clients qui consomment ces clés — coordonner avec refactor frontend dans la même PR.

**Priorité** : basse (cosmétique) — ne bloque pas le merge Story 5.3.

### CR-008 : Golden test PDF — hash SHA-256 binaire vs round-trip payload

**Origine** : Story 5.3 code review 2026-04-15 (passe 3 — Acceptance Auditor).

**Contexte** : La spec Story 5.3 §AC15 + T4.7 exige un **golden test PDF** basé sur un hash SHA-256 binaire stocké dans `crates/kesh-qrbill/tests/fixtures/golden/invoice.pdf.sha256`. L'implémentation v0.1 adopte le **Plan C** (validé par Guy, cf. `crates/kesh-qrbill/tests/golden_test.rs` commentaire d'en-tête) :

1. `payload_is_deterministic_across_10_runs` — vérifie byte-equal du payload QR Bill string sur 10 runs.
2. `pdf_size_stable_with_fixed_date` — vérifie taille PDF stable avec date figée (modulo random `/ID` trailer de printpdf).

Pas de comparaison binaire du PDF (printpdf injecte un random dans le trailer `/ID`).

**Scope** :
- **Plan B** (idéal spec) : extraire le payload QR via `rxing` depuis les bytes PDF (décoder l'image QR intégrée) et comparer à une string attendue figée.
- Permet de détecter une régression sur la partie QR (le plus critique pour SIX conformité).
- Couvrir aussi fixtures officielles SIX (`docs/six-references/samples-payment-part-en/`) comme round-trip tests.

**Statut** : dette technique documentée. Remédiation possible en story future dédiée aux tests PDF.

**Référence code** : `crates/kesh-qrbill/tests/golden_test.rs` (commentaire « Plan C »), `crates/kesh-qrbill/src/generator.rs::qr_roundtrip_via_rxing` (déjà implémente round-trip sur QR matrix seule, pas sur bytes PDF).
