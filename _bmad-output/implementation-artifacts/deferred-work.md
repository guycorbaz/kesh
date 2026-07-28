# Deferred Work — Code Review Findings

Cumulé des items reportés en v0.2+ depuis les passes de code-review BMAD.
Source de vérité unique pour la dette technique non-bloquante post-merge.

---

## Deferred from: code review of 9-5-3-process-codification-claude-md (2026-05-18)

Pass 1 + Pass 2 code-review (Sonnet 4.6 × 3 reviewers + Haiku 4.5 × 2 reviewers), 30 findings bruts cumulés → 16 patches appliqués + 2 deferred ci-dessous + 12 dismiss.

- **E14 — Story de remédiation Catégorie B bloquée ou annulée — mécanisme de réévaluation périodique manquant** (MEDIUM, §Tech debt management — Catégorie B) : si la story de remédiation d'une dette B est elle-même bloquée (dépendance amont reportée) ou fermée `wontfix`, la dette B reste tracée indéfiniment sans révision. Acceptable v0.1 (cohérent zero carry-forward, la rétrospective d'epic est le point de contrôle implicite). À traiter v0.2+ : ajouter un mécanisme operational (e.g. revue trimestrielle du backlog v0.2-milestone GitHub Milestone, ou règle « si la story B est `wontfix`, la dette revient en A et doit être triée »). Hors scope CLAUDE.md durable — relève d'un processus operational projet.
- **P2-F6 — Workflow Project Lead indisponible** (MEDIUM, §Tech debt — Triage hors fenêtre rétrospective) : la règle dit que l'arbitrage est fait par le Project Lead, mais aucun fallback si le Project Lead est absent/indisponible au moment de la découverte d'une dette A en cours d'Epic. Pour v0.2+ : ajouter un workflow d'escalade type « ouvrir une issue GitHub `[TRIAGE-NEEDED]` avec scénario d'impact, attendre triage humain avant action — ne pas auto-classer ». Hors scope codification CLAUDE.md durable — processus opérationnel rare.

---

## Deferred from: code review of 9-2b-export-global-zip (2026-05-17)

Pass 1 Sonnet 4.6 × 12 reviewers (4 chunks × 3 layers BH+ECH+AA), 108 findings bruts → 5 deferred ci-dessous + 15 patches appliqués + 31 dismiss.

- **D1 — `journal_entries.list_all_by_company` `ORDER BY entry_date, id` vs index `(company_id, entry_date DESC)` → filesort systématique** : perf concern non-bloquant v0.1 (PME ≤ 5k écritures). À combiner avec L4 streaming v0.2 (option : passer à `ORDER BY id` ou ajouter index `(company_id, entry_date ASC, id ASC)` dédié export).
- **D2 — `zopfli` transitive dep ~120 Ko + license restriction clause** (`Cargo.lock` zopfli 0.8.3 tiré par `zip 2.4 features = ["deflate"]` malgré `default-features = false`) : audit license v0.2 + vérifier si `zip 2.5+` corrige le gating. Alternative envisageable : passer à `async-zip` si streaming v0.2 (D1).
- **D3 — `hex_encode` perf — `format!("{b:02x}")` par byte = 32 allocs par hash** (`crates/kesh-api/src/util.rs::hex_encode`) : pour les 16 SHA-256 par export = 512 allocs inutiles. Fix v0.2 : `use std::fmt::Write; write!(&mut s, "{b:02x}").unwrap()` in-place.
- **D4 — `export_date` capturée en fin de pipeline (post-queries SQL + serialize)** : spec non explicite sur "start vs end of pipeline". Écart théorique < 10s sur dataset référence — acceptable v0.1. v0.2 : passer `export_date` en paramètre à `build_metadata_json` depuis `Instant::now()` du début handler.
- **D5 — `aria-busy` manquant sur bouton `disabled` pendant export** (`frontend/src/routes/(app)/export/+page.svelte:export-global-start`) : WCAG 2.1 SC 4.1.3 Status Messages. Dette a11y cohérente avec KF-027 #91 (`#bits-c1` DropdownMenu pré-existant). v0.2 : ajouter `aria-busy={exporting}` + `aria-label` conditionnel selon état.

---

## Deferred from: code review of 10-5-httponly-tokens-security (2026-05-26)

Pass 3 Opus 4.7 × 3 reviewers (BH + ECH + AA), 19 findings post-dédup → 3 deferred ci-dessous + 12 patches appliqués + 4 decisions résolues.

- **BH3-L1∪ECH3-L2 — `STORAGE_KEY_*` dead exports + redéclaration drift risk dans `test-state.ts`** (`frontend/src/lib/app/stores/auth.svelte.ts:38-40` + `frontend/tests/e2e/helpers/test-state.ts:35-38`) : les 3 constantes `STORAGE_KEY_ACCESS_TOKEN` / `STORAGE_KEY_REFRESH_TOKEN` / `STORAGE_KEY_EXPIRES_IN` sont exportées depuis `auth.svelte.ts` mais le store n'écrit plus jamais en localStorage post-Story-10-5. Le test helper `test-state.ts` redéclare localement ces constantes avec un commentaire "must match auth.svelte.ts — if keys change there, update here too" → drift risk reconnu mais perpétué. v0.2 cleanup : (a) retirer les 3 `export const` du store (garder `const` privé pour `localStorage.removeItem` defensive seulement), OU (b) importer depuis le store dans test-state.ts pour éliminer la redéclaration.
- **BH3-L2 — `AUTH_EXCLUDED_URLS` dead code post-buildHeaders refactor** (`frontend/src/lib/shared/utils/api-client.ts:buildHeaders`) : la constante n'est plus référencée depuis le retrait de l'injection `Authorization: Bearer` header dans `buildHeaders` (Story 10-5 T7). Commentaire `// La constante AUTH_EXCLUDED_URLS est conservée pour traçabilité mais n'a plus de rôle actif` admet le dead code. v0.2 scope cleanup : retirer la déclaration + l'import si externe.
- **ECH3-L1 — Regex JWT trop large dans `xss-token-protection.spec.ts` Scénario (a)** (`frontend/tests/e2e/security/xss-token-protection.spec.ts:3307`) : `expect(cookieString).not.toMatch(/[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+/)` matche n'importe quelle chaîne 3-segments dot-separated alphanum. Si une future feature ajoute un cookie non-HttpOnly visible JS avec valeur `kesh.session.tracking` (3 segments), le test échouerait à tort. Faux-positif futur seulement, pas un bug actuel. v0.2 : restreindre le regex à pattern plus discriminant (e.g. `[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{10,}` qui exige base64 long) OU asserter explicitement `not.toContain("kesh_access_token=")` sans regex JWT générique.

## Deferred from: code review of v011-5-onboarding-self-service (2026-05-31)

Pass 1 Sonnet 4.6 × 3 reviewers (BH + ECH + AA), 16 findings raw → 11 patches + 3 deferred ci-dessous + 1 dismiss + 1 merge.

- **BH1-5 MEDIUM — `+layout.ts::load` redirect `_setupRequired` peut être dead code au premier paint** (`frontend/src/routes/+layout.ts:14-17`) : SvelteKit `+layout.ts::load` s'exécute AVANT que `hydrate()` (via `hooks.client.ts::init` async ClientInit) ait peuplé `_setupRequired`. Le redirect ne fire pas au premier paint — c'est le `window.location.replace('/setup')` de `api-client.ts` qui gère le path dynamique. Mais sur les **navigations client-side subséquentes** (après que `hydrate()` ait set le flag), le load `+layout.ts` fire avec `isSetupRequired=true` et le redirect SvelteKit prend effet. Defense-in-depth réelle pour le cas (a) cookie révoqué côté serveur entre 2 hydrates intra-SPA + (b) navigation programmatique post-truncate DB. v0.2 : ajouter un test E2E Playwright dédié vérifiant que la redirection vient bien de `+layout.ts` (et non de l'interceptor) sur une navigation client-side post-hydrate.
- **ECH1-4 MEDIUM — `KESH_PASSWORD_MIN_LENGTH` non-propagé au frontend SetupForm** (`frontend/src/lib/features/setup/SetupForm.svelte:22` + `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl::setup-password-min`) : frontend hardcoded `MIN_PASSWORD = 12` et i18n key `setup-password-min = "Au moins 12 caractères"`. Si opérateur set `KESH_PASSWORD_MIN_LENGTH=16` (range valide [8,128]), backend enforce 16 mais frontend permet submit pour passwords 12-15 chars → 400 backend + UX divergence. v0.2 : exposer `password_min_length` via endpoint public `GET /api/v1/config/public` (ou inclure dans la réponse boot), driver `MIN_PASSWORD` depuis la valeur server-returned, passer comme variable Fluent à `setup-password-min`. Hors scope v0.1.2 (introduit endpoint public + frontend i18n templating).
- **AUD1-4 LOW — `website/index.html` + `website/roadmap.html` non mis à jour pour v0.1.2** : la spec §T6 AUD1-5 demandait de vérifier la roadmap pour l'ajout des features onboarding self-service + recovery break-glass. `git diff main...HEAD -- website/` retourne 0 lignes. Non-bloquant pour la PR (le site rebuild au merge sur main n'aurait rien de plus à dire), mais **à faire avant tag release v0.1.2** (cohérent CLAUDE.md §"Synchroniser TOUTES les docs avant tout push/release").

## Deferred from: code review of v011-5-onboarding-self-service Pass 2 (2026-05-31)

Pass 2 Haiku 4.5 × 3 reviewers, 7 findings → 6 dismiss + 1 defer ci-dessous. ECH+AA 0 finding actionnable, 1 HIGH BH faux-positif réfuté grep ground-truth.

- **BH2-4 LOW — `password 'changeme' is forbidden (placeholder)` message non-i18n** (`crates/kesh-api/src/routes/setup.rs:95`) : le validateur `AppError::Validation("password 'changeme' is forbidden (placeholder)".into())` renvoie un message en anglais quel que soit le locale browser. Cohérence frontend i18n divergente vs autres messages backend localisés. Marginal car le user ne tape pas "changeme" en pratique (placeholder `.env.example`). v0.2 cleanup : ajouter clés `error-password-changeme-forbidden` dans 4 locales + utiliser `t("error-password-changeme-forbidden", "...")`.

## Deferred from: code review of 17-4c-backend-endpoints (2026-06-11)

Pass 1 Fable 5 × 3 reviewers (BH + ECH + AA), 30 findings bruts → 10 patches + 4 deferred ci-dessous + 3 dismiss.

- **D1 MEDIUM — `ConnectInfo` derrière reverse proxy → DoS global du recovery** (`crates/kesh-api/src/routes/auth.rs` `enforce_recovery_rate_limit` + `middleware/rate_limit.rs`) : l'IP vient du socket TCP, jamais de `X-Forwarded-For`. Derrière Traefik/proxy, tous les clients partagent l'IP du proxy ; le record inconditionnel DC5 fait que 5 requêtes recovery quelconques / 15 min (mêmes légitimes, utilisateurs différents) bloquent forgot-password ET reset-password 30 min pour toute l'installation. Pattern pré-existant (limiter login idem), amplifié par DC5. Remédiation : support `X-Forwarded-For` opt-in (env var trusted proxy) — **issue #173** (`enhancement` + `v0.2-milestone`).
- **D2 LOW — Lockout utilisateur légitime à mi-flux** (`routes/auth.rs` + `lib.rs::build_recovery_rate_limiter`) : limiter partagé forgot+reset, chaque requête consomme un slot (y compris un reset rejeté par la politique mdp) ; blocage 30 min = TTL token → token expiré pendant le blocage. Lié L5 (seuils configurables v0.2+).
- **D3 LOW — `expires_at` horloge app vs `NOW(3)` horloge MariaDB** (`routes/auth.rs` + `password_reset_tokens.rs:67`) : skew NTP ou TZ MariaDB non-UTC fait dériver le TTL réel (30 min sensible à un offset d'1 h). Pattern pré-existant identique `refresh_tokens`. Documenter exigence NTP/UTC dans le manuel admin (17-4f).
- **D4 LOW — `tokio::spawn` fire-and-forget non drainé au shutdown** (`routes/auth.rs`) : redeploy Docker pendant l'envoi SMTP = email jamais envoyé, zéro trace log. Acceptable v0.1 ; piste `tokio_util::task::TaskTracker` si récurrent.

## Deferred from: code review of 17-4c-backend-endpoints Pass 2 (2026-06-11)

Pass 2 Sonnet 4.6 × 3 reviewers, 13 findings bruts → 4 réfutés grep ground-truth (dont 1 HIGH ConnectInfo + 1 MEDIUM timeout SMTP) + 5 dismiss + 3 patches (PP1 lookup détaché, PP2 Instant::checked_sub, PP3 story file) + 1 defer ci-dessous.

- **AA2-L2 LOW — garde `@` username non-i18n dans `setup.rs`** (`crates/kesh-api/src/routes/setup.rs:96-103`) : `AppError::Validation("username must not contain '@' (reserved for email recovery routing)")` anglais hardcodé, vs `users.rs` qui utilise la clé i18n `error-username-contains-at`. Cohérent avec le pattern pré-existant de setup.rs (`"username must be non-empty"` idem). À aligner lors du même cleanup v0.2 que BH2-4 v011-5 (`error-password-changeme-forbidden`) : ajouter les clés et basculer setup.rs sur `state.i18n.format`.

## Deferred from: code review of 18-1a-comptes-tva (2026-06-15)

Cycle 5 passes Sonnet→Haiku→Opus→Sonnet→Haiku (convergé Pass 5, 0 > LOW). Findings deferred ci-dessous (pré-existants ou stories suivantes).

- **D1 MEDIUM — Archive d'un compte TVA configuré → FK CIS obsolète bloque la sauvegarde des invoice-settings** (`crates/kesh-db/src/repositories/accounts.rs` archive + `routes/company_invoice_settings.rs::validate_account`) : `archive()` ne fait que `active=FALSE` (jamais `DELETE`), donc la FK `ON DELETE RESTRICT` ne se déclenche pas ; un compte TVA archivé reste référencé dans `company_invoice_settings`, et `validate_account` rejette ensuite tout `PUT /company/invoice-settings` (« compte archivé ») jusqu'à ce que l'utilisateur efface le sélecteur TVA obsolète. **Pré-existant** : `default_receivable_account_id`/`default_revenue_account_id` ont exactement le même gap ; 18-1a multiplie l'exposition de 2 à 5 colonnes FK mais n'introduit pas de défaut neuf. → **Issue GitHub à créer** (`bug` + `technical-debt`), pas un blocage 18-1a.
- **D2 LOW — `admin_backup_e2e` ne vérifie pas l'intégrité FK des 3 nouvelles colonnes VAT après restore** (`crates/kesh-api/tests/admin_backup_e2e.rs`) : la vérification post-restore ne couvre que `default_receivable_account_id`. Ne se déclenche qu'avec TVA configurée + round-trip backup/restore. → 18-1f (tests).
- **D3 LOW — Pas de contrainte `default_vat_payable ≠ default_vat_decompte`** (`routes/company_invoice_settings.rs`) : les deux acceptent le même compte Liability ; sémantiquement TVA due ≠ décompte net AFC. Concern design consommé en 18-1b. → 18-1b.

## Deferred from: code review of 20-1-templates-email-socle (2026-07-08)

Pass 1 Sonnet 5 × 3 reviewers (Blind Hunter + Edge Case Hunter + Acceptance Auditor, parallèle), 21 findings bruts → 9 patches + 1 deferred ci-dessous (4 findings mergées) + 6 dismiss (dont 2 réfutés ground-truth).

- **D1 LOW — `tx.rollback().await.map_err(map_db_error)?` masque l'erreur d'origine si le rollback lui-même échoue** (`crates/kesh-db/src/repositories/email_templates.rs`, 4 sites : conflit version, no-op, restore idempotent, échec audit) : si le `ROLLBACK` échoue après détection d'un conflit optimiste / court-circuit no-op / audit KO, le `?` propage l'erreur du rollback plutôt que l'erreur métier attendue (409 attendu devient 500 générique, causerait aussi la perte de la cause racine d'un échec audit). **Pré-existant, pattern identique dans TOUS les repositories du projet** (confirmé : `company_invoice_settings.rs` a exactement les mêmes 5 occurrences) — pas une régression introduite par Story 20-1. Remédiation potentielle générique (hors scope d'une story ponctuelle) : logger explicitement l'erreur de rollback avant de retourner l'erreur métier d'origine, sur tous les repositories concernés.

## Deferred from: code review of 20-2-admin-templates-ui (2026-07-08)

Pass 1 Sonnet 5 × 3 reviewers (Blind Hunter + Edge Case Hunter + Acceptance Auditor, parallèle), 24 findings bruts → 6 patches + 1 deferred ci-dessous + ~11 dismiss.

- **D1 LOW — `onMount` lance `listEmailTemplates()` (Admin-only) inconditionnellement → 403 inutile pour un non-Admin** (`frontend/src/routes/(app)/settings/email-templates/+page.svelte`) : le garde `{#if !isAdmin}` ne gate que le rendu, pas le fetch. Pour un utilisateur Comptable/Consultation, `onMount` déclenche un appel réseau qui reçoit 403 (harmless, `loadError` non affiché car la branche `!isAdmin` gagne). **Pré-existant, comportement identique au pattern établi `settings/invoicing/+page.svelte`** (fetch inconditionnel). Guarder le fetch sur `isAdmin` introduirait la régression de course d'hydratation auth (ECH#7 : `authState.currentUser` momentanément `null` au mount → `isAdmin=false` → admin légitime verrait une page vide sans re-fetch après hydratation). Remédiation propre = pattern cross-pages (attendre l'hydratation auth avant de décider fetch/guard) hors scope d'une story feature ponctuelle → à traiter dans un cleanup frontend dédié.

## Deferred from: code review of 14-3a-socle-roles-comptes (2026-07-22)

Pass 1 Opus 4.8 × 3 reviewers (Blind Hunter + Edge Case Hunter + Acceptance Auditor, parallèle), 34 findings bruts → 4 decision-needed + 17 patches + 2 deferred ci-dessous + 1 dismiss (faux positif d'aveuglement i18n, réfuté par ground-truth `fr-CH/messages.ftl:146-153`).

- **D1 LOW — `accounts-count` sans forme plurielle** (`crates/kesh-i18n/locales/*/messages.ftl`, consommée par `frontend/src/routes/(app)/accounts/+page.svelte`) : une société à un seul compte affiche « 1 comptes » / « 1 Konten » / « 1 conti » / « 1 accounts ». Le correctif propre est le sélecteur Fluent `{ $count -> [one] … *[other] … }`, mais `i18nMsg` ne sait pas l'interpréter — il fait un simple `String.replace` sur `{ $var }` et rendrait la syntaxe de sélection verbatim. Étendre le moteur i18n aux sélecteurs Fluent concerne **toutes** les pages et dépasse le périmètre d'une story socle → à traiter dans un lot i18n dédié (candidat : même lot que la pluralisation des autres compteurs de l'app).
- **D2 LOW — Colonne Rôle : cellule vide au lieu du tiret prévu par AC-H** (`frontend/src/routes/(app)/accounts/+page.svelte:369`, `{#if account.role}`) : divergence avec la lettre de l'AC (« ou un tiret si aucun »), assumée — la page présente les rôles sous forme de badges, et un badge « — » ajouterait du bruit visuel sur la majorité des lignes (la plupart des comptes n'ont pas de rôle). `roleLabel(null)` renvoie déjà « Aucun » si le besoin d'un libellé explicite réapparaît (p. ex. en 14-3b si la colonne devient filtrable).

## Deferred from: code review of 14-4-bilan-ouverture-soldes-depart (2026-07-25)

- **ECH-1 MED → defer, issue [#274](https://github.com/guycorbaz/kesh/issues/274)** — TOCTOU sur la garde « comptes de bilan » du `POST /opening-balances` : le type-check s'exécute hors du lock final, un `PUT /accounts/{id}` retypant Asset→Revenue dans la fenêtre passe. Cause racine **pré-existante** (hors story) : `accounts::update` autorise le retypage d'`account_type` d'un compte **avec écritures** à tout moment (verrou optimiste seul) — l'état final de la course est donc atteignable légalement en séquentiel (générer puis retyper), et déplacer le check sous le lock ne fermerait pas le retypage post-commit. Remédiation = immutabilité (ou reclassement audité) du type d'un compte à historique — tracée #274 (`known-failure` + `technical-debt`).

## Deferred from: code review of 16-1a-compte-produit-par-ligne-backend (2026-07-28)

- **AC5-ter — le décompte « 8 + 1 » de la spec est faux.** Trois sites réels ne sont pas comptés par le critère numérique de la spec : l'`INSERT` de `insert_lines` (`invoices.rs:404-408`, liste de colonnes distincte de `LINE_COLUMNS`), `is_no_op_change` (`invoices.rs:963`) et `InvoiceLineResponse` (`routes/invoices.rs:159`). La spec affirmait « si le décompte ne tombe pas sur 8 + 1, il manque quelque chose » — assertion démentie à l'implémentation, après 7 passes de `validate`. Le livrable, lui, couvre bien les sites réels. Dette de qualité de spec, pas de code.
- **Chemin avoir : un compte introuvable est étiqueté `Inactive`, avec un numéro vide dans le message.** `credit_notes.rs:432-444` mappe tout id absent du `SELECT ... active = TRUE` sur `RevenueAccountRejection::Inactive`, sans distinguer « archivé » de « inexistant » — contrairement au chemin facture qui a un variant `UnknownOrCrossCompany`. Si l'id est introuvable, `account_number` vaut `None` et `unwrap_or_default()` produit « le compte  est archivé » (numéro vide), avec l'instruction inapplicable « réactivez le compte ». Chemin jugé inatteignable en fonctionnement normal : la FK `ON DELETE RESTRICT` empêche la suppression et les comptes ne changent pas de société. Robustesse, pas bug démontré.
- **Duplication de la collecte des « sites » à contrôler.** `credit_notes.rs:387-394` réimplémente le concept « ligne N ou compte par défaut » avec une sentinelle `0: i32`, là où `invoices.rs:1511-1520` utilise `explicit_line_account_sites` et `Option<i32>`. Deux conventions, deux tris/dédups séparés, aucune vérification de type pour signaler une divergence future. Exactement la classe de risque que le diff corrige par ailleurs sciemment sur `LINE_COLUMNS`, mais non appliquée ici.
