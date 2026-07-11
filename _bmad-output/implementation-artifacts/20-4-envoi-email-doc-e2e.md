# Story 20.4: Envoi de factures par e-mail — documentation & E2E round-trip

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

En tant qu'administrateur (et utilisateur) de Kesh,
je veux des manuels à jour (configuration SMTP découplée du recovery, délivrabilité, modèles d'e-mail, mode d'emploi du bouton d'envoi) et une preuve E2E round-trip du flux d'envoi,
afin que la fonctionnalité livrée par 20-1→20-3b2 soit exploitable, supportable et non-régressable.

**Dernière story de l'Epic 20.** Consomme tout ce qui précède (20-1 templates, 20-2 UI admin, 20-3a PDF, 20-3b1 backend, 20-3b2 frontend). Après elle : rétrospective epic + push/PR groupée.

## Acceptance Criteria

**Backend test-mode — capture d'e-mails (pré-requis du round-trip Playwright)**

1. `main.rs` (:236-294) : en `config.test_mode` **et** `config.smtp_configured()`, construire un **`MockMailer`** (au lieu du `SmtpMailer` réel — un host factice passerait le build mais échouerait au `send()`, vérifié smtp.rs:73-81 : lettre ne connecte qu'à l'envoi) ; `smtp_ready = true` comme aujourd'hui. Sans vars SMTP : comportement inchangé (`NoopMailer`, `smtp_ready=false` → le fallback zéro-config reste testable). Hors test-mode : strictement inchangé. `MockMailer`/`CapturedEmail` sont déjà compilés dans le binaire (mail/mod.rs:132-233, aucun `#[cfg(test)]` — vérifié).
2. `AppState` : nouveau champ `test_mock_mailer: Option<MockMailer>` (les clones partagent le buffer `Arc<Mutex<...>>`) — `Some(mock)` uniquement dans la branche test-mode de l'AC#1, `None` partout ailleurs ; défaut `None` dans le **corps** de `new_for_tests` (signature figée) ; littéraux `AppState` des tests Rust étendus (mécanique, ~10 fichiers — même famille que `smtp_ready` en 20-3b1).
3. `GET /api/v1/_test/sent-emails` (`test_endpoints.rs`, monté sous le gate `KESH_TEST_MODE` existant lib.rs:796-800) : si `state.test_mock_mailer` est `Some` → 200 `{ "emails": [{ to, subject, body, fromDisplayName, replyTo, attachmentFilename, attachmentContentType, attachmentSize }] }` (camelCase, mapping complet depuis `CapturedEmail` — mod.rs:133-142, y compris `attachment_content_type`) ; sinon → 409 avec message explicite « test-mode sans SMTP factice — capture indisponible ». Précédent de style : `password_reset_token_handler` (:273-298).
4. `POST /_test/seed` purge aussi : le **buffer du MockMailer** (si présent) et le **rate-limiter send-email** (`rate_limiter_send_email`), comme il purge déjà login+recovery (cohérence inter-specs, budget 20/15 min).

**E2E Playwright — round-trip**

5. Helpers API partagés : extraire de `invoices.spec.ts` vers `tests/e2e/helpers/api-fixtures.ts` les helpers `createContactWithAddressViaApi` (:256-287, étendu de **deux** paramètres optionnels `email?: string` et `salutation?: 'Monsieur' | 'Madame' | 'Neutre'` — le payload actuel ne pose ni l'un ni l'autre, or l'AC#6 exige un contact `Madame` pour l'assertion de salutation genrée ; `POST /contacts` accepte `salutation` au même niveau que `email`, contacts.rs:93-96), `ensurePrimaryBankAccountViaApi` (:292-307) et `createAndValidateInvoiceViaApi` (:309-340) ; `invoices.spec.ts` les importe (DRY — ne pas dupliquer une 3e copie). Les sélecteurs/flows existants d'`invoices.spec.ts` ne changent pas.
6. Nouveau `tests/e2e/invoice-send-email.spec.ts` (modèle **`invoices.spec.ts:5-12`** pour le seed `with-company` en **`beforeAll`** — PAS `email-templates.spec.ts` qui utilise `beforeEach` [state singleton, cf. règle docs/testing.md:85-90] ; nos mutations sont scopées à des rows créées par le test → beforeAll ; login local + afterEach clearAuthStorage comme les deux) — cas minimum, contre un backend démarré **avec SMTP factice** (recette AC#9) :
   - **Round-trip complet** : contact Personne avec `email` + adresse structurée + civilité `Madame`, banque primary, facture validée (via helpers API) → fiche facture → bouton `send-email-button` visible et actif → clic → modale : `send-email-to` affiche l'e-mail (**et il n'existe AUCUN `<input>` contenant cette adresse** — destinataire verrouillé, décision #13), objet pré-rempli non vide, corps contient la salutation genrée → confirm → toast succès → métadonnée `invoice-emailed-at`/`invoice-emailed-to` affichées → `GET /_test/sent-emails` (via `authedApiContext`... non : contexte non-authentifié suffit, endpoint _test) : dernier e-mail = bon `to`, bon `subject`, `attachmentFilename` matche `facture-.*\.pdf`, `attachmentSize > 1000`.
   - **Renvoi** : le bouton affiche « Renvoyer par e-mail », 2e envoi → 2 e-mails capturés.
   - **Contact sans e-mail** : facture d'un contact sans email → modale avec `send-email-to-missing` + bouton confirm `disabled`.
   - **Gate rôle** : user `Consultation` (créé via API users) → le bouton `send-email-button` est **absent** de la fiche facture (gate `canManage`).
7. **Fallback zéro-config** : nouveau `tests/e2e/invoice-send-email-nosmtp.spec.ts`, **gaté par env** (`test.skip(process.env.KESH_E2E_NO_SMTP !== '1', ...)`) car il exige un backend démarré **sans** vars SMTP : `/health` → `smtpConfigured:false` ; fiche facture validée → bouton présent mais `disabled` (wrapper `send-email-disabled-wrapper`) ; hover sur le wrapper → tooltip avec le texte KESH_SMTP_*. (Deux configurations backend = deux runs Playwright séquentiels — assumé, documenté AC#9.)
8. Non-régression : `invoices.spec.ts` (helpers extraits !), `contacts.spec.ts`, `email-templates.spec.ts` restent verts. **Exit codes vérifiés sans pipe** (leçon 20-3b2 : `> log 2>&1; echo EXIT=$?`).
9. `docs/testing.md` : nouvelle sous-section « E2E envoi de factures par e-mail » — recette des **deux runs** : (1) backend test-mode avec SMTP factice (`KESH_SMTP_HOST=smtp.invalid KESH_SMTP_USER=e2e KESH_SMTP_PASSWORD=e2e KESH_SMTP_FROM=kesh@example.invalid` → MockMailer + capture via `/_test/sent-emails`) pour le spec principal ; (2) backend sans vars SMTP + `KESH_E2E_NO_SMTP=1` pour le spec fallback. Rappeler `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE` et DB `kesh_e2e`.

**Manuel administrateur (`docs/manual/fr/admin-manual.tex`, décision #17 epic)**

10. **Découplage SMTP/recovery** aux deux points d'ancrage (cartographie vérifiée) : paragraphe env-vars :749-765 et section `sec:recovery-smtp` :989-1010 — reformuler : la config SMTP sert **deux fonctions** (envoi de factures dès les 4 vars posées, sans fail-fast ; recovery = `KESH_FEATURE_FORGOT_PASSWORD` + `KESH_PUBLIC_BASE_URL` en plus, fail-fast conservé). Aligné sur le bloc `.env.example` déjà réécrit (commit `ef474b4b`).
11. Nouvelle `\subsection{Envoi de factures par e-mail}` (après :1080, avant break-glass :1082 — structure calquée sur `sec:recovery-smtp` : Variables/Fonctionnement/Test/Limitations) : activation par les 4 vars, `/health.smtpConfigured`, dégradation gracieuse (bouton grisé — jamais de fail-fast), From = `KESH_SMTP_FROM` avec display-name société + Reply-To = e-mail société (Réglages), rate-limit 20 envois/15 min par utilisateur, **`keshwarning` délivrabilité SPF/DKIM** (texte décision #17 : « utilisez le SMTP de votre fournisseur de messagerie, dont le domaine est déjà authentifié SPF/DKIM ; n'usurpez pas un domaine tiers »), `keshnote` L20-3 (« envoyée » = remise au serveur SMTP, pas accusé de réception ; bounces non remontés).
12. Nouvelle `\subsection{Modèles d'e-mail}` (à la suite) : section *Paramètres → Modèles d'e-mail* (Admin), objet+corps par langue FR/DE/IT/EN, variables `{salutation}`/`{contactName}`/`{invoiceNumber}`/`{amount}`/`{dueDate}`/`{companyName}`, défauts fournis zéro-config, « Restaurer le défaut », validation des variables inconnues au save.

**Manuel utilisateur (`docs/manual/fr/user-manual.tex`)**

13. **Corriger les deux contradictions** : réécrire :586 (« Kesh ne dispose pas d'envoi de facture intégré… ») et **retirer** la ligne :1307 (« Pas d'envoi de facture par email intégré ») de la liste des fonctionnalités non disponibles (section Nouveautés/limites).
14. Nouvelle `\subsection{Envoyer une facture par e-mail}` dans `\section{Facturation QR Bill}` (après :590, avant Échéancier :592 — patron :606-646 Avoirs) : bouton sur facture validée, modale pré-remplie dans la **langue du contact** avec civilité, objet/corps modifiables, **destinataire verrouillé** (pour changer l'adresse → fiche contact), l'indication « Envoyée le [date] » suivie de l'**adresse du destinataire** (le « à … » est l'adresse e-mail, PAS une heure — l'UI réelle affiche `emailedAt.slice(0,10)` + le destinataire sur une 2e ligne, +page.svelte:582-585 ; ne pas documenter une heure inexistante), renvoi possible, `keshnote` envoyée ≠ reçue, bouton grisé si SMTP non configuré (renvoi vers manuel admin), `% TODO capture:` placeholders (convention texte-d'abord, macro `\keshscreenshot`).
15. Documenter les à-côtés livrés par l'epic (jamais documentés — la 20-2 n'a pas de section user, vérifié) : `\subsection` « Modèles d'e-mail » (Paramètres → Modèles d'e-mail, Admin : onglets langue, variables, restaurer défaut — patron admin mais vue utilisateur) ; fiche contact : « Langue de correspondance » (héritée/FR/DE/IT/EN) + « Civilité » (Personne) ; Réglages → Organisation : « E-mail (adresse de réponse) ».
16. **PDF régénérés et commités** : `make fr` depuis `docs/manual/` (xelatex 2 passes, PDFs git-trackés — convention PR #102). **PAS de bump `\keshVersion`** (reste `0.5.2` — le bump est le gate release 4-bis au tag, pas ici).

**Gates**

17. Backend (changements main.rs/lib.rs/test_endpoints.rs/littéraux) : `cargo fmt --check` + `build --workspace --all-targets` + `clippy -D warnings` + `cargo test --workspace` série — les suites Rust existantes (dont `invoice_send_email_e2e` 19 tests) restent vertes.
18. Frontend/E2E : les 2 runs Playwright de l'AC#9 verts (**exit codes capturés sans pipe**) + non-régression AC#8 + `npm run check`/`test:unit`/`build` inchangés verts.

## Tasks / Subtasks

- [x] **T1 — Backend test-mode : capture d'e-mails** (AC: #1-#4)
  - [x] T1.1 `main.rs` branche test-mode → MockMailer ; `AppState.test_mock_mailer` + défaut + littéraux tests étendus
  - [x] T1.2 `GET /_test/sent-emails` + purge buffer & rate-limiter send-email au seed
  - [x] T1.3 Tests Rust du endpoint (dans la suite test_endpoints existante si elle existe, sinon e2e dédié léger)
- [x] **T2 — Helpers E2E partagés** (AC: #5) : extraction `api-fixtures.ts` + re-import dans `invoices.spec.ts`
- [x] **T3 — Specs Playwright** (AC: #6, #7)
  - [x] T3.1 `invoice-send-email.spec.ts` (round-trip, renvoi, sans-email, gate rôle)
  - [x] T3.2 `invoice-send-email-nosmtp.spec.ts` (gaté env) + tooltip hover
- [x] **T4 — `docs/testing.md`** (AC: #9)
- [x] **T5 — Manuel admin** (AC: #10-#12)
- [x] **T6 — Manuel user** (AC: #13-#15)
- [x] **T7 — `make fr` + PDFs commités** (AC: #16)
- [x] **T8 — Gates & commit** (AC: #17, #18)
  - [x] T8.1 Gate backend série + gate frontend + 2 runs Playwright (exit codes sans pipe)
  - [x] T8.2 Commit sur `story/20-1-envoi-factures-email`

## Dev Notes

### Ground-truth (cartographie 2 agents Explore, 2026-07-11)

**Test-mode backend** : endpoints `_test` dans `crates/kesh-api/src/routes/test_endpoints.rs` (router :47-54 — seed :164, reset :302, password-reset-token :273), montés par `lib.rs:796-800` ssi `config.test_mode` (parsing strict `KESH_TEST_MODE` config.rs:836-846 ; bind non-loopback → refus boot config.rs:850-852, `ConfigError::TestModeWithPublicBind`). Le précédent 17-4e (`password_reset_token_handler`) injecte/lit la DB et **retourne l'artefact au test** — même philosophie pour `sent-emails`. **Aucun MockMailer n'est instancié hors tests Rust aujourd'hui** (main.rs:236-294 : SmtpMailer si config, sinon NoopMailer). `MockMailer`/`CapturedEmail`/`CapturedMail` sont `pub` SANS `#[cfg(test)]` (mail/mod.rs:118-233) → utilisables dans main.rs sans changement de gates de compilation.

**Pourquoi MockMailer et pas un SMTP factice** : `smtp_configured()` (config.rs:388-396) ne vérifie que la présence des vars + `from` plausible ; `SmtpMailer::from_config` (smtp.rs:46-89) ne fait **aucune connexion au build** (lettre connecte à l'envoi) → un host factice donne `smtpConfigured:true` puis **`SmtpSendFailed` 500 au send réel**. Le round-trip exige donc la substitution du mailer en test-mode.

**Seed `with-company`** (`test_fixtures.rs:80-201` via test_endpoints :181-184) : company + 2 admins (`admin/admin123`) + fiscal year + 5 comptes + invoice_settings + 4 taux TVA. **NI contact, NI banque primary, NI facture** — le spec crée tout via API. ⚠️ Les helpers d'`invoices.spec.ts` créent des contacts **sans email** → paramètre `email` à ajouter à l'extraction (AC#5), sinon 400 `CONTACT_EMAIL_MISSING`.

**Helpers à extraire** (`invoices.spec.ts`) : `createContactWithAddressViaApi` :256-287 (réparé en 20-3b2 : firstName/lastName + addressStructured), `ensurePrimaryBankAccountViaApi` :292-307 (IBAN `CH93...`, tolère 200/201/409 — le seed ne pose pas de banque depuis v014-1), `createAndValidateInvoiceViaApi` :309-340 (POST invoice 1 ligne + validate, retourne id). `authedApiContext` (helpers/test-state.ts:169-186) clone les cookies HttpOnly du browser context. `global-setup.ts` fail-fast la connectivité backend/test-mode.

**Sélecteurs 20-3b2 disponibles** : `send-email-button`, `send-email-disabled-wrapper`, `send-email-to`, `send-email-to-missing`, `send-email-confirm`, `invoice-emailed-at`, `invoice-emailed-to` (data-testid, posés par la 20-3b2).

**Manuels** — `docs/manual/fr/`, classe `article`, build `make fr` (xelatex 2 passes, Makefile :44), PDFs **git-trackés**. Style : `shared/kesh-style.sty` — boîtes `keshnote`/`keshtip`/`keshwarning`/`keshdanger` (:249-327), `\keshcommand`/`\keshpath` (:365-396), lstlisting style sombre (:221-243), tableaux tabularx+booktabs avec `\keshtableheader` (patron admin :752-765). Macros version `\keshVersion=0.5.2` (:64) — NE PAS toucher (gate 4-bis release). `\keshscreenshot` définie **uniquement** dans user-manual.tex :15-23 (placeholder + `% TODO capture:`).

**Points d'insertion admin-manual** (2036 l.) : bloc SMTP env-vars **:749-765** (paragraphe « Recovery de mot de passe par email (SMTP) » — à renommer/reformuler) ; section canonique **`sec:recovery-smtp` :989-1080** (structure Variables/Providers/Fonctionnement/Test/Limitations = patron à copier ; fail-fast :1010 ; seule mention SPF/DKIM :1067) ; insérer les 2 nouvelles subsections après :1080, avant break-glass :1082.

**Points d'insertion user-manual** (1334 l.) : `\section{Facturation QR Bill}` :520 ; **:586 = phrase contradictoire à réécrire** (« le service de messagerie interne (SMTP) n'est utilisé que pour la récupération de mot de passe ») ; insérer la subsection envoi après :590 (avant Échéancier :592) ; patron récent :606-646 (Avoirs, v0.5.2) ; **:1307 = ligne à retirer** (« Pas d'envoi de facture par email intégré ») ; conventions :55 (italique/→/boîtes).

### Décisions de conception

- **MockMailer en test-mode = le mécanisme voulu par l'epic** (« E2E round-trip (MockMailer…) », découpage story 20-4) — le gate loopback-only de KESH_TEST_MODE (refus de boot sinon) borne le risque du endpoint de capture. Le 409 explicite (test-mode sans SMTP factice) évite un 200-vide ambigu.
- **Deux runs Playwright séquentiels** (avec/sans SMTP) plutôt qu'un toggle runtime de `smtp_ready` : `smtp_ready` est un bool immuable d'AppState — le rendre mutable pour un test serait de l'over-engineering ; la CI principale n'exécute pas les E2E (CLAUDE.md), la recette locale à deux runs est documentée dans testing.md.
- **Extraction des helpers plutôt que 3e duplication** (DRY CLAUDE.md) — `invoices.spec.ts` est re-pointé sur `api-fixtures.ts` sans changer ses flows.
- **Pas de bump `\keshVersion`** : story de doc, pas de release — le gate 4-bis s'applique au tag.

### Frontières de scope

- **AUCUN changement de comportement produit** : le backend hors test-mode et tout le frontend restent byte-identiques (seuls `main.rs` branche test-mode, `AppState` +1 champ, `test_endpoints.rs`, et les littéraux de tests bougent côté Rust).
- Manuels DE/IT/EN : stubs inchangés (v0.2+, note « à traduire » déjà en place).
- Website : release-time (checklist pré-release), hors story.
- KF-038 (#228) : flake pré-existant sans rapport, ne pas y toucher.

### Testing standards summary

- Gate backend 4 checks (CLAUDE.md §Test Locally First), kesh-db/kesh-api **en série**.
- E2E : DB `kesh_e2e`, `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`, **jamais de pipe sur le runner** (leçon 20-3b2 : `> log 2>&1; echo EXIT=$?`).
- `make fr` doit sortir sans erreur xelatex (2 passes) ; PDFs commités dans la même PR.

### References

- [Source: `_bmad-output/planning-artifacts/epic-20-envoi-factures-email.md`] — décision #17 (doc admin/user + délivrabilité + « envoyée ≠ reçue »), découpage 20-4, L20-1/2/3.
- [Source: `_bmad-output/implementation-artifacts/20-3b1-envoi-facture-backend.md` + `20-3b2-envoi-facture-frontend.md`] — contrats + sélecteurs + leçons process (pipe/tail, write-par-patch).
- [Source: cartographie 2 agents Explore 2026-07-11 — manuels LaTeX, test-mode/E2E] (références précises ci-dessus).
- [Source: CLAUDE.md §Test Locally First, §Règle de commit (PDF versionnés), gate 4-bis (\keshVersion au tag seulement)]

## Senior Developer Review (AI)

### Pass 1 — 2026-07-11, Sonnet 5 × 2 (Blind Hunter diff-only + AA/ECH combiné), contexte frais, diff `b4280ae5..fa7d0f44`

**10 findings bruts (1 CRITICAL + 2 MEDIUM + 7 LOW) → 1 MEDIUM patché + 5 LOW patchés/factorisés + 1 CRITICAL reclassé LOW/INFO ground-truth + 1 MEDIUM réfuté + 2 LOW dismissés. AA : 18/18 ACs conformes, déviation 400-vs-409 validée.**

- **[CRITICAL → reclassé LOW/INFO, ground-truth double] BH — « fuite du mode capture en prod avale les factures »** : le scénario exige `KESH_TEST_MODE=true` en prod avec de vraies creds SMTP. Réfutation structurelle : (1) le boot **refuse** test_mode avec bind non-loopback (`ConfigError::TestModeWithPublicBind`, config.rs:850-852, `0.0.0.0` explicitement rejeté) et le docker-compose standard met `KESH_HOST=0.0.0.0` → la fuite du flag casse le démarrage bruyamment ; (2) le vecteur résiduel (bare-metal loopback + reverse proxy local) relève du modèle de menace pré-existant de KESH_TEST_MODE, où `/seed` non authentifié TRUNCATE déjà toute la base — la capture n'ajoute pas de classe de risque. L'AA/ECH arrive indépendamment à la même conclusion (LOW/INFO « aucune action requise »). Action prise quand même : le commentaire de main.rs cite désormais précisément le garde-fou (au lieu d'une affirmation vague que le BH ne pouvait pas vérifier).
- **[MEDIUM → patché] BH — rustdoc disant encore « 409 »** : les 2 doc-comments (lib.rs champ AppState, test_endpoints.rs handler) contredisaient le code réel (400 VALIDATION_ERROR, déviation documentée) — corrigés + test renommé `_400_without_capture` (AA l'avait aussi flaggé).
- **[MEDIUM → réfuté ground-truth] BH — flakiness buffer avec workers parallèles** : `playwright.config.ts` impose `workers: 1` (sérialisation documentée pour le seed partagé) — pas de course possible ; le `clear()` non-atomique est sans conséquence sous exécution sérielle.
- **[LOW → patchés] AA** : test rôle anti-vacuous (assertion `toHaveURL` avant le `toHaveCount(0)`) ; **BH** : purge seed/reset factorisée (`clear_transient_test_state`, la duplication venait d'un replace-all qui avait — correctement — étendu les 2 handlers).
- **[LOW → dismissés documentés] BH** : bootstrap dupliqué des 2 tests Rust (ils exigent une `Config::with_test_mode` différente — étendre la signature de `spawn_app` toucherait 19 call-sites pour 2 usages) ; exposition PII du endpoint (subsumée par le triage du CRITICAL — même garde, même périmètre que `/seed`).

Gate post-patchs : fmt/clippy 0 warning, suite Rust 21/21, Playwright send-email 4/4 EXIT=0 (assertion anti-vacuous incluse).

## Change Log

- 2026-07-11 — `bmad-code-review` Pass 1 (Sonnet 5 × 2) : 10 bruts → 1 MEDIUM doc patché, CRITICAL reclassé LOW/INFO (double ground-truth : boot gate + périmètre test-mode pré-existant), 1 MEDIUM réfuté (workers:1), LOW patchés (anti-vacuous, purge factorisée, rename test). Pass 2 (Haiku) à suivre.

- 2026-07-11 — `validate-create-story` **CONVERGÉ 2 passes** (Sonnet 5 → Haiku 4.5, contextes frais, trend 5 [1 HIGH] → 0). Pass 2 Haiku : **0 finding, 0 faux-positif** (les garde-fous anti-catégorie et anti-hallucination du prompt ont tenu — toutes les références re-vérifiées contre le code : data-testid ×7, helpers, gates config, points d'insertion LaTeX, rate-limiter 20/15min). Statut ready-for-dev confirmé.
- 2026-07-11 — `validate-create-story` Pass 1 (Sonnet 5, contexte frais) : 5 findings (1 HIGH + 3 MEDIUM + 1 LOW), tous patchés. HIGH : le helper étendu doit aussi porter `salutation?` (l'AC#6 exige un contact Madame ; le payload actuel ne pose ni email ni salutation). MEDIUM : modèle `beforeAll` réattribué à `invoices.spec.ts` (email-templates utilise `beforeEach` — règle testing.md:85-90) ; refs `config.rs` corrigées (:836-846/:850-852, pas :1026/:94) ; « Envoyée le … à … » clarifié (le « à » = adresse destinataire, pas une heure — ne pas documenter un élément UI inexistant). LOW : `attachmentContentType` ajouté au contrat `/_test/sent-emails`. Le validateur a confirmé exactes toutes les autres références (data-testid ×7, ancres manuels, _test hors require_auth, 19 tests Rust, from `kesh@example.invalid` valide). Pass 2 (Haiku) à suivre.

## Dev Agent Record

### Agent Model Used

Claude Fable 5 (claude-fable-5) — run unique 2026-07-11.

### Debug Log References

- Regex d'extension des littéraux AppState : a inséré `test_mock_mailer: None` dans la SIGNATURE de `spawn_app` (invoice_send_email_e2e.rs) en plus du littéral — paramètre invalide retiré, leçon « un write par patch + grep post-write » appliquée ensuite systématiquement.
- Premier run E2E : `Address already in use` — un backend zombie d'un run antérieur tenait le port 8181 (le log du nouveau backend montrait pourtant « MockMailer actif ») ; kill par PID via `ss -tlnp`, relance propre.
- `make fr` : `\og`/`\fg` non définis (pas de babel) — mes ajouts utilisaient les guillemets babel ; remplacés par « » littéraux (XeLaTeX Unicode natif). Erreur attrapée au premier build.

### Completion Notes List

- **T1-T8 conformes, 1 déviation documentée** : l'AC#3 prévoyait 409 quand la capture est indisponible — implémenté en **400 `VALIDATION_ERROR`** (réutilise `AppError::Validation` avec le message diagnostique explicite ; un nouveau variant + FTL ×4 pour un endpoint de test n'aurait servi que le code HTTP). Le test Rust documente le 400.
- Backend : branche test-mode dans `main.rs` (MockMailer + `smtp_ready=true`, log WARN explicite), `AppState.test_mock_mailer` (défaut None, 4 littéraux étendus), `GET /_test/sent-emails` (mapping camelCase complet incl. `attachmentContentType`), purge seed (buffer + rate-limiter send-email), `MockMailer::clear()`. 2 tests Rust (capture+purge, 400 sans poignée) — 21 tests dans `invoice_send_email_e2e` désormais.
- E2E : helpers extraits vers `helpers/api-fixtures.ts` (+ params `email?`/`salutation?`), `invoices.spec.ts` re-pointé sans changement de flow. **Run principal : 25 passed / 2 skipped / 0 failed EXIT=0** (4 nouveaux tests round-trip [e-mail capturé avec PDF joint vérifié côté backend, salutation genrée « Chère Madame » assertée dans le corps pré-rempli, renvoi, sans-email, Consultation sans bouton] + non-régression invoices/contacts/email-templates). **Run no-smtp : 1 passed EXIT=0** (smtpConfigured=false vérifié, bouton grisé, tooltip au hover du wrapper).
- Manuels : admin — SMTP découplé du recovery aux 2 ancres + `\subsection` Envoi de factures (identité From/Reply-To, keshwarning SPF/DKIM, garanties/limites L20-3) + `\subsection` Modèles d'e-mail ; user — 2 contradictions corrigées (:586 réécrite, ligne « pas d'envoi intégré » retirée des limites) + `\subsection` Envoyer une facture par e-mail (+ sous-sections modèles [20-2 enfin documentée], langue/civilité contact, adresse de réponse) avec placeholders `% TODO capture:`. `make fr` EXIT=0, 3 PDFs régénérés et commités, `\keshVersion` inchangé (0.5.2 — gate 4-bis au tag).
- `docs/testing.md` : recette des 2 runs + phrase recovery amendée (MockMailer capture désormais en test-mode) + purges seed complétées + rappel anti-pipe.

### File List

**Nouveaux**

- `frontend/tests/e2e/helpers/api-fixtures.ts`
- `frontend/tests/e2e/invoice-send-email.spec.ts`
- `frontend/tests/e2e/invoice-send-email-nosmtp.spec.ts`

**Modifiés — backend (test-mode uniquement, aucun changement de comportement produit)**

- `crates/kesh-api/src/main.rs` (branche test-mode → MockMailer)
- `crates/kesh-api/src/lib.rs` (`AppState.test_mock_mailer` + défaut new_for_tests)
- `crates/kesh-api/src/mail/mod.rs` (`MockMailer::clear()`)
- `crates/kesh-api/src/routes/test_endpoints.rs` (GET sent-emails + purges seed)
- `crates/kesh-api/src/middleware/auth.rs`, `crates/kesh-api/tests/{invoice_send_email,password_recovery,setup_admin}_e2e.rs` (littéraux étendus ; +2 tests endpoint dans invoice_send_email_e2e)

**Modifiés — E2E & doc**

- `frontend/tests/e2e/invoices.spec.ts` (helpers re-pointés sur api-fixtures)
- `docs/testing.md` (recette 2 runs, purges, amendement recovery)
- `docs/manual/fr/admin-manual.tex` + `docs/manual/fr/admin-manual.pdf`
- `docs/manual/fr/user-manual.tex` + `docs/manual/fr/user-manual.pdf`
- `docs/manual/fr/marketing-brochure.pdf` (rebuild make fr, contenu inchangé)

## Change Log

- 2026-07-11 — `bmad-dev-story` COMPLETED run unique (Fable 5) : T1-T8, 1 déviation documentée (400 au lieu de 409 sur sent-emails sans capture). Gates : fmt/clippy 0 warning, frontend check 0 err + unit 377/377, E2E run principal 25/25 EXIT=0 + run no-smtp 1/1 EXIT=0, make fr EXIT=0 (3 PDFs). Workspace série : **92 suites, 1784 tests, 0 échec, EXIT=0**.
