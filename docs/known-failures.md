# Known Failures (KF)

Registre des tests/comportements **constatés cassés mais hors scope** du travail en cours, à ne pas oublier de corriger.

## Convention

- **Une entrée par défaillance** identifiée par un ID `KF-NNN`.
- Format : symptôme reproductible, root cause hypothétique, story d'origine, story de remédiation prévue, status (`open` / `closed`).
- Une KF résolue passe à `status: closed` + lien vers le commit fix.
- Pattern process : tout commit qui constate un échec hors scope doit mentionner la KF dans son message.
- **Double tracking GitHub Issues** depuis 2026-04-16 — chaque KF a une issue sur [guycorbaz/kesh](https://github.com/guycorbaz/kesh/issues?q=label%3Aknown-failure) avec les labels `known-failure` + éventuellement `technical-debt` + `triage`. Ce fichier reste la source de vérité offline / dans Git ; GitHub est la source online pour triage.

---

## KF-001 — `invoice_pdf_e2e` : violation `chk_companies_ide_format` + silent fail `validate_invoice`

- **GitHub** : [#7](https://github.com/guycorbaz/kesh/issues/7) (closed)
- **Découvert** : 2026-04-16 (session code review story 5-4 Groupe B)
- **Symptôme** : 10/11 tests de `cargo test -p kesh-api --test invoice_pdf_e2e` paniquent avec :
  ```
  CheckConstraintViolation("CONSTRAINT `chk_companies_ide_format` failed for `_sqlx_test_*`.`companies`")
  ```
- **Root cause effective** (2 problèmes imbriqués) :
  1. La fixture `seed_base` utilisait `ide_number: Some("CHE-123.456.789".into())` — format d'affichage avec séparateurs — alors que la CHECK DB `chk_companies_ide_format` impose la forme canonique `^CHE[0-9]{9}$` (pas de séparateurs, normalisation côté route `contacts.rs`). De surcroît `123456789` ne satisfait pas le checksum mod-11 du `CheNumber`.
  2. Une fois le CHECK corrigé, 8 tests échouaient toujours avec `INVOICE_NOT_VALIDATED` — `seed_validated_invoice` appelait `invoices::validate_invoice` mais swallowait le `Result`. `validate_invoice` exige un `fiscal_year` ouvert + `company_invoice_settings` avec `default_receivable_account_id` et `default_revenue_account_id` non-NULL, aucun desquels n'était seedé.
- **Scope d'origine** : Story 5-3 (Génération PDF QR Bill). Pré-existant — pas une régression introduite par la review story 5-4.
- **Correctif** (commit à suivre) :
  1. `ide_number` → `CHE109322551` (forme canonique, mod-11 valide, aligné avec `kesh-seed`).
  2. `seed_validated_invoice` refondu pour utiliser le pattern SQL bypass (`fiscal_year` + `journal_entry` stub + `UPDATE status='validated'`) aligné avec `invoice_echeancier_e2e::create_validated_invoice_via_sql`. Évite la dépendance à `validate_invoice` qui exige une config comptable complète non pertinente pour tester la route PDF.
- **Validation** : `cargo test -p kesh-api --test invoice_pdf_e2e` → 11/11 ✅
- **Status** : closed 2026-04-16

---

## KF-002 — Multi-tenant : `get_company()` ignore `CurrentUser.company_id`

- **GitHub** : [#2](https://github.com/guycorbaz/kesh/issues/2)
- **Découvert** : 2026-04 (code review Stories 4-1, 4-2, 5-1..5-4 — flag récurrent HIGH/MED)
- **Symptôme** : le helper backend `get_company(&state)` fait `companies::list(pool, 1, 0)` (LIMIT 1 sans ORDER BY) et ignore le `company_id` de l'utilisateur courant. Tant qu'il n'y a qu'une seule company en DB, le bug est invisible. En multi-tenant (plusieurs companies), un utilisateur pourrait se voir attribuer les données d'une autre company de manière non-déterministe.
- **Root cause** : pattern hérité de l'onboarding mono-tenant. Non refactoré quand le modèle multi-tenant implicite est apparu (users ↔ company_id).
- **Scope d'origine** : Story 1-4 (schéma initial) ; propagé dans `contacts.rs`, `products.rs`, `invoices.rs`, `invoice_pdf.rs`, `bank_accounts.rs`, `company_invoice_settings.rs`, `journal_entries.rs`.
- **Blocage** : aucun en production tant que Kesh reste mono-tenant. Critique dès qu'un second user avec une autre company existe.
- **Reproduction** : `grep -rn "get_company" crates/kesh-api/src/routes/` — 7+ fichiers concernés.
- **Story de remédiation** : **Story 6-2** (Epic 6 Qualité & CI/CD) — refactor `get_company()` → `get_company_for(user)` + tests IDOR cross-company par entité.
- **Status** : open

---

## KF-003 — TVA : whitelist des taux hardcodée (pas DB-driven)

- **GitHub** : [#3](https://github.com/guycorbaz/kesh/issues/3)
- **Découvert** : 2026-04 (Story 4-2 code review, dette D2 Epic 4 retro)
- **Symptôme** : les taux TVA valides (7.70%, 8.10%, 3.70%, 2.60%, 0.00%) sont hardcodés en whitelist Rust (`kesh-core::vat`) et TS (frontend). Tout changement de taux (comme la TVA suisse 2026 ↔ 2024) exige une PR + release binaire.
- **Root cause** : choix pragmatique Story 4-2 — une table `vat_rates` aurait élargi le scope. Reporté à Epic 10 (TVA Suisse).
- **Scope d'origine** : Story 4-2 (Conditions de paiement & catalogue produits).
- **Blocage** : aucun court terme (taux suisses stables). Critique à chaque changement réglementaire ou quand on veut supporter un autre pays.
- **Reproduction** : `grep -rn "7.70\|8.10\|3.70\|2.60" crates/kesh-core/src crates/kesh-db/src frontend/src/lib/features`.
- **Story de remédiation** : **Epic 10** TVA Suisse — table `vat_rates(id, company_id, rate, valid_from, valid_to)`, migration des hardcodes, config admin.
- **Status** : open

---

## KF-004 — `update()` bump `version` même sur no-op

- **GitHub** : [#4](https://github.com/guycorbaz/kesh/issues/4)
- **Découvert** : 2026-04 (Stories 4-1, 4-2 code review, dette D3 Epic 4 retro)
- **Symptôme** : l'appel `contacts::update()` / `products::update()` / `invoices::update()` incrémente toujours `version` même si aucun champ n'a changé. Deux utilisateurs qui cliquent « Enregistrer » sans rien modifier obtiennent un conflit optimistic lock trompeur.
- **Root cause** : pattern `UPDATE ... SET ..., version = version + 1` sans comparaison préalable des champs. Choix de vitesse implémentation.
- **Scope d'origine** : Story 1-8 (RBAC & verrouillage optimiste), pattern propagé.
- **Blocage** : UX dégradée sur les formulaires édition (faux conflits). Aucun impact intégrité.
- **Reproduction** : éditer un contact, ne rien changer, cliquer « Enregistrer » deux fois dans deux onglets → le second reçoit `CONFLICT`.
- **Story de remédiation** : à créer (priorité basse, cosmétique). Candidat post-v0.1.
- **Status** : open

---

## KF-005 — FULLTEXT index manquant sur colonnes de recherche

- **GitHub** : [#5](https://github.com/guycorbaz/kesh/issues/5)
- **Découvert** : 2026-04 (Story 3-4 review, dette D4 Epic 4 retro)
- **Symptôme** : la recherche contacts/produits/factures utilise `LIKE '%query%'` (full table scan). Performance acceptable jusqu'à ~10k lignes, puis dégradation linéaire.
- **Root cause** : MariaDB 10.11 supporte les index FULLTEXT mais exige `MATCH () AGAINST ()` (syntaxe différente de `LIKE`). Pas implémenté car pas bloquant pour MVP.
- **Scope d'origine** : Story 3-4 (Recherche, pagination & tri).
- **Blocage** : aucun sur données MVP. Critique à partir de ~50k contacts/produits ou dès qu'on cherche dans les lignes de facture (`invoice_lines.description`).
- **Reproduction** : `EXPLAIN SELECT * FROM contacts WHERE name LIKE '%foo%'` → `type: ALL` (full scan).
- **Story de remédiation** : à créer (priorité post-v0.1, quand les métriques prod le justifient). Migration DB + refactor handlers search.
- **Status** : open

---

## KF-006 — Sidebar `Catalogue` / `Facturer` hardcodée (pas i18n)

- **GitHub** : [#6](https://github.com/guycorbaz/kesh/issues/6)
- **Découvert** : 2026-04 (Story 4-2 code review, dette D8 Epic 4 retro)
- **Symptôme** : les labels sidebar dans `frontend/src/routes/(app)/+layout.svelte` sont en français hardcodé (`'Accueil'`, `'Carnet d'adresses'`, `'Catalogue'`, `'Facturer'`, etc.) au lieu d'utiliser `i18nMsg('nav-X', 'fallback')`. Les 3 autres locales (de-CH, it-CH, en-CH) voient l'UI en français.
- **Root cause** : la spec Story 4-2 piège #7 interdisait explicitement le refactor sidebar (scope creep), mais T6.1 de la même spec listait `nav-products` comme clé à créer — contradiction interne non détectée par 4 passes de validation. Code livré en français hardcodé.
- **Scope d'origine** : Story 1-10 (layout & page de login) ; accumulation progressive Stories 2-4, 3-1, 4-1, 4-2, 5-1, 5-4.
- **Blocage** : aucun FR. Critique dès qu'un utilisateur DE/IT/EN ouvre l'app.
- **Reproduction** : changer `instance_language` de company en `de-CH`, relancer frontend, observer sidebar toujours en français.
- **Story de remédiation** : **Story 6-3** (Epic 6 Qualité & CI/CD) — `nav-*` FTL keys × 4 locales + refactor layout.svelte + lint i18n key-ownership.
- **Status** : open
