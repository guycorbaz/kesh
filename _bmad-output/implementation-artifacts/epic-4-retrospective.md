# Rétrospective Epic 4 — Carnet d'adresses & Catalogue

**Date** : 2026-04-13
**Facilitateur** : Bob (Scrum Master)
**Participants** : Guy (Project Lead), Alice (Product Owner), Murat (Architect), Bob (Scrum Master)
**Epic** : Epic 4 — Carnet d'adresses & Catalogue produits/services
**Status** : ✅ DONE (2 stories livrées, rétro optionnelle)

## Périmètre livré

| Story | Commit | Description |
|-------|--------|-------------|
| 4-1 Carnet adresses CRUD contacts | `e6290c6` | Contacts (clients/fournisseurs), IDE/n° TVA suisse validés, soft-delete, optimistic locking |
| 4-2 Conditions paiement & catalogue produits | `750881e` | Champ `default_payment_terms` sur contacts + catalogue produits complet (TVA whitelist, audit log) |

**Volumétrie** : 89 fichiers touchés, +8 746 / -669 lignes.

## 🟢 What went well

**Bob** : Ce qui a bien marché, à garder et à reproduire pour les prochains epics ?

- **Règle multi-passes adversariales orthogonales** (CLAUDE.md) appliquée systématiquement. Sur 4-2 : 4 passes de validation spec (Sonnet+Haiku → Haiku → Opus → Haiku) + 3 passes code-review (Opus → Sonnet → Haiku). **Verdict final CLEAN avec 0 régression** malgré 22 patches appliqués. Le changement de LLM entre passes attrape régulièrement des défauts que l'auteur est incapable de voir sur son propre code.
- **DRY réel entre stories** : `formatSwissAmount` créé en Story 3.2 réutilisé tel quel dans `product-helpers.ts` 4-2 (apostrophe typographique U+2019 Swiss SN01). Pas de duplication de la logique comptable critique.
- **Pattern P1 « onMount pour lecture URL »** (Story 4.1 post-remédiation) propagé proprement à 4-2 — a évité de re-découvrir le bug des deux fetches au montage.
- **Defense-in-depth comptable** : validations côté API (rust_decimal scale/upper bound/whitelist) ET côté DB (CHECK constraints). Test d'insertion directe qui bypasse le handler pour valider le CHECK → filet de sûreté indépendant du code applicatif.
- **Audit log atomique** du pattern Story 3.5 réutilisé cleanly (snapshot direct pour create/archive, wrapper `{before, after}` pour update, rollback sur échec de sérialisation JSON).
- **Couverture i18n** : ~85 clés Fluent x 4 locales (de/en/fr/it) ajoutées sur l'epic, 0 clé manquante en finale (après P11 du code-review qui a rétabli les clés dédiées `product-form-*` au lieu du couplage avec `contact-*`).

**Murat** : Le switch d'LLM entre passes est devenu un outil technique à part entière, pas juste une « bonne pratique ». Haiku passe 3 sur 4-2 a confirmé **CLEAN** en contexte frais — validation indépendante de la remédiation Opus+Sonnet.

## 🟡 What could improve

**Alice** : Ce qu'on garde en mémoire pour ne pas répéter ?

- **Anti-patterns hérités qui accumulent de la dette** : le helper `get_company()` qui fait `LIMIT 1` sans `ORDER BY` et ignore `CurrentUser.company_id` a été transmis de `contacts.rs` (4-1) à `products.rs` (4-2). Les 3 reviewers de la passe 1 l'ont flaggé en HIGH/MED, mais on l'a classé en `defer` car c'est pré-existant. **À chaque story, la dette multi-tenant se propage** et devient plus coûteuse à démêler.
- **Contradictions internes dans les specs** : le piège #7 des Dev Notes de 4-2 interdit explicitement le refactor sidebar en i18n (« scope creep »), tandis que T6.1 liste `nav-products` comme clé à créer. L'auteur du spec (Opus) s'est contredit. 4 passes de validation n'ont pas détecté la contradiction. **À surveiller** : chaque mention de clé i18n dans T6 devrait être cross-checked avec l'usage réel dans le code à produire.
- **Code-review passe 1 sur même LLM que l'implémenteur** (Opus backend sur 4-2 implémenté en Opus) → biais d'auteur sur les patches. Seule la passe 2 (Sonnet) a vraiment changé de perspective. **Règle à durcir dans CLAUDE.md** : la passe 1 de code-review doit elle aussi être orthogonale à l'implémenteur, pas seulement les passes de remédiation.
- **Tests d'intégration non bloquants** : les tests DB (12 tests `products::tests`) et Playwright (6 spec) ne sont exécutés qu'à la main, car ils nécessitent MariaDB up + seed + backend running. Chaque merge sans CI gate laisse passer des régressions potentielles. Story 8-4 (GitHub Actions CI) toujours en backlog.
- **Couplage inter-feature silencieux** détecté par le code-review : le dialog produits 4-2 utilisait les clés i18n `contact-form-cancel` / `contact-archive-confirm`. Svelte-check ne lève aucune alerte sur ce genre de reuse. **Pas de lint i18n** qui vérifierait que chaque `i18nMsg('X')` appelé dans une feature utilise uniquement des clés de son propre domaine.
- **Spec 4-2 longue et dense** (390 lignes) — bien que multi-passée, elle contient encore des anti-exemples explicites (2 occurrences de `DECIMAL(15,4)` dans sections historiques). Risque qu'un futur lecteur les confonde avec des normes.

## 🔴 What was hard / pénible

**Guy** (à faire émerger par l'utilisateur) :
- _Zone laissée volontairement minimale — à compléter par Guy s'il a des frustrations spécifiques._

**Dette technique connue reportée** (depuis 4-2 code-review) :
- D1 — multi-tenant scoping (`get_company` → `CurrentUser.company_id`)
- D2 — whitelist TVA DB-driven (table `vat_rates` vs hardcode)
- D3 — `update` bump version sur no-op (pattern 4-1 & 4-2)
- D4 — FULLTEXT index sur search (perf Epic 5+)
- D5 — audit log sur lecture (`list`/`get` sans `CurrentUser`)
- D6 — i18n des messages d'erreur backend
- D7 — refus archive si produit référencé (bloqué jusqu'à Epic 5)
- D8 — sidebar `Catalogue` hardcodée (contradiction spec piège #7 vs T6.1)

## ✅ Action items

Chaque item doit être actionable et indexable. **Pas d'estimation temporelle**.

### Pour maintenant (avant Epic 5)

1. **Durcir CLAUDE.md règle multi-passes**  — Préciser que la passe 1 de `bmad-code-review` doit utiliser un LLM différent de l'implémenteur (pas seulement les passes de remédiation). Propriétaire : Guy. Critère fait : paragraphe mis à jour dans `CLAUDE.md`.

2. **Créer story transversale « Sidebar i18n + refactor anti-patterns hérités »** (post-Epic 4, pré-Epic 5). Résout D1 (multi-tenant `get_company`), D8 (sidebar `Catalogue` hardcodée), et harmonise le pattern `update` pour ne pas bumper `version` si no-op (D3). Propriétaire : PM agent (`bmad-correct-course` ou `bmad-create-story` standalone). Critère fait : story créée dans `sprint-status.yaml`, validée 4 passes.

3. **Push commit `750881e` vers GitHub** (repo `kesh` créé par Guy). Propriétaire : Guy. Critère fait : `origin/main` aligné sur HEAD.

### Pour Epic 5 preparation

4. **Epic 5 dépend de 4-2** : vérifier que la ligne facture pourra consommer `products.unit_price DECIMAL(19,4)` × `quantity` sans overflow. Cap 1 milliard CHF posé en 4-2 offre une marge confortable pour des factures jusqu'à 10⁶ unités. Propriétaire : Architect lors du `bmad-create-story 5-1`.

5. **Étudier l'écosystème QR Bill Rust** avant de créer 5-3 : crates existantes (`iso11649`, générateurs PDF), spécification QR Bill 2.2 officielle, contraintes de conformité Six Payment Services. Propriétaire : Guy + Architect. Critère fait : choix crate documenté dans `_bmad-output/planning-artifacts/architecture.md` ou Dev Notes 5-3.

### Pour plus tard (dette, non bloquant)

6. **CI pipeline GitHub Actions** (Story 8-4 du backlog) : lancer `cargo test` + `npm run test:unit` + `npm run test:e2e` + `cargo test --test '*integration*'` à chaque push. Sans ça, chaque nouvelle feature laisse potentiellement passer des régressions DB + Playwright.

7. **Lint i18n key-ownership** : règle qui interdit à une feature d'utiliser des clés d'une autre feature (namespace strict). Propriétaire : à scoper dans une story outillage.

8. **Table paramétrable `vat_rates`** (D2) — reportée à v0.2 Epic 9 (TVA Suisse).

## Préparation Epic 5 — Facturation QR Bill

**Bob** : Epic 5 a 3 stories backlog :

| Story | Titre | Dépendances bloquantes |
|-------|-------|------------------------|
| 5-1 | Création factures brouillon | products (4-2) ✅, contacts (4-1) ✅, payment terms (4-1/4-2) ✅ |
| 5-2 | Validation & numérotation factures | 5-1 |
| 5-3 | Génération PDF QR Bill | 5-1, 5-2, étude crate QR Bill (action #5) |

**Murat** : Rien ne bloque 5-1. Le catalogue produits 4-2 fournit exactement ce qui manque — `unit_price DECIMAL(19,4)` + `vat_rate` whitelistés.

**Alice** : Critère d'acceptance business à clarifier lors du spec de 5-1 : comportement des lignes facture sur produit archivé après création du brouillon (cf. D7).

**Points chauds identifiés pour Epic 5** :
- **QR Bill 2.2 conformité** : spec dense, erreurs de format → banque refuse le paiement. Tests d'intégration avec références IBAN suisses réelles indispensables.
- **Multi-lignes + TVA mixte** : une facture peut avoir lignes à 8,10% et 2,60% (hébergement). La whitelist TVA de 4-2 est prête.
- **Précision** : `unit_price × quantity + vat_rate` calculé en `rust_decimal` (pas `f64`), pattern `big.js` côté frontend.
- **PDF generation** : pas de solution Rust mature pour QR Bill intégré natif — probable dépendance externe (library C ou binaire).

## Bilan Epic 4

**Guy — à confirmer** : on valide Epic 4 comme livré et on rend la rétro-Epic 4 `done` dans `sprint-status.yaml` ?

**Bob** : Si oui, on peut marquer `epic-4-retrospective: done` et passer aux 3 action items (durcir CLAUDE.md, story transversale, push) avant d'attaquer Epic 5 avec `bmad-create-story 5-1`.

---

*Rétro générée 2026-04-13 via `/bmad-retrospective`. Epic 1-3 avaient également leurs rétros documentées (epic-1-retrospective `done`, epic-2-retrospective `done`, epic-3-retrospective `optional`).*
