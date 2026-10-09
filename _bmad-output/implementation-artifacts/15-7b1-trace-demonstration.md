# Story 15.7b1 : Le chargement de la démonstration laisse sa trace

Status: done

<!-- Née le 2026-10-08 du découpage de la 15-7b (choix C-15-7-31), à la passe de validation P4 :
     coupe Volet A / Volet B que la section « Dérogation règle de splitting » de la 15-7b prévoyait
     « sans nouvel arbitrage » si un défaut né d'une remédiation revenait à sévérité égale (R4-2, F-1 de
     la P4). Cette fiche porte le Volet A (la démonstration) ; la 15-7b2 porte le Volet B (la remise à
     zéro, #528, #279).
     Choix applicables de `epic-15-choix-autonomes.md` : C-15-7-4 (démonstration en synthèse),
     C-15-7-13 (atomicité prouvée par un déclencheur de test), C-15-7-14 (dernière transaction de
     `seed_demo`, boucle de retry), C-15-7-25 (même release que la 15-7a2 — étendu à la 15-7b2 par
     C-15-7-31), C-15-7-31 (découpage). Version d'avant découpage : `9bbc52de` (fiche 15-7b, P3 appliquée).
     Validation : P1 (sur la 15-7), P2 à P4 (sur la 15-7b) appliquées ; P1 de cette fiche appliquée
     (4 MEDIUM, choix C-15-7-35) ; P2 appliquée (2 MEDIUM, choix C-15-7-37) ; P3 ciblée (Haiku) :
     0 — **validation CLOSE** (C-15-7-43). -->

**Issues** : `refs #434` — **partout, PR comprise**. C'est la **15-7b2**, dernière des sous-stories, qui
porte `closes #434` : cette fiche laisse `reset` non tracée. **`closes #544`** sur la PR (F2-2 de la P2 :
le manuel décrit une démonstration que `seed_demo` ne crée pas ; AC 11), `refs #544` sur les commits
intermédiaires.

**Dépendances** : **la 15-7a2, mergée** (et donc la 15-7a1) — elles posent
`onboarding::record_step_completed_in_tx`, `onboarding::lock_state_in_tx`,
`vat_rates::seed_default_swiss_rates_in_tx -> Vec<VatRate>`,
`company_invoice_settings::insert_with_defaults_in_tx -> (settings, bool)`,
`companies::clear_stub_in_tx`, le helper de test `audit_sequence` et le fichier
`crates/kesh-api/tests/onboarding_audit_e2e.rs`. **La 15-7b2 dépend de celle-ci.** ⚠️ **Même release
que la 15-7a2 et la 15-7b2** (C-15-7-25, C-15-7-31).
⚠️ **Renvois de ligne** : ils valent sur la base `b74c3dac` (avant la 15-7a1 et la 15-7a2, qui
réécrivent `routes/onboarding.rs`) ; à relocaliser **par symbole** au moment du développement.
⚠️ **Conflits attendus au merge** : registre des routes, `ACTIONS`, quatre `.ftl`, `## [0.13.0]` du
CHANGELOG — recompter depuis la source.
⚠️ **Numérotation** : les AC et les tests gardent les numéros de la 15-7b (traçabilité des passes P1 à
P4) ; les numéros absents sont à la 15-7b2.

## Story

En tant que **fiduciaire ou réviseur qui lit le journal d'audit d'une installation**,
je veux que **le chargement des données de démonstration y figure, en une entrée de synthèse**,
afin de savoir **qui a peuplé l'installation de données fictives, et quand**.

## Décompte de modules — signal D5 déclaré

À la granularité retenue par C-15-7-8 : `kesh-seed`, `kesh-api/routes/onboarding` (handler
`seed_demo`), `kesh-api/audit_labels`, `kesh-i18n` — **quatre modules de code** ; plus trois modules
de `kesh-db` touchés **en commentaires seulement** (`repositories/{vat_rates, company_invoice_settings,
fiscal_years}`, AC 10). Sept modules touchés : le seuil est franchi **au sens de la règle**, comme pour
la 15-7a2 (F3-5) ; **signal déclaré au Project Lead**, non découpée — aucune ligne exécutable hors des
quatre modules de code, qui sont sous le seuil.

## Ce que l'inventaire a établi, et qui n'est pas dans l'issue

1. **`seed-demo` est atteignable par un jeton d'API** (`authenticated_routes`, `lib.rs:851`). ⇒
   l'acteur `(user_id, api_key_id)` est **threadé** depuis le handler et écrit par
   `NewAuditLogEntry::for_actor` dans `kesh-seed`.
2. **`seed_demo` enchaîne sept validations** (`kesh-seed/src/lib.rs:72-226`) : le verrou de comptage,
   `companies::update`, `bulk_create_from_chart`, `create_for_seed`, `seed_default_swiss_rates`,
   `insert_with_defaults`, puis `update_step` sur le pool ; et le handler fait encore un `UPDATE
   companies SET is_stub = FALSE` **après** (`routes/onboarding.rs:211-214`), hors de toute
   transaction d'étape. Rendre atomiques les quatre premières (verrou, société, plan, exercice) est
   **hors périmètre** et **suivi par #538** — l'orchestrateur y a ajouté ce point (signalé par R4-3 de
   la P4 de la 15-7b). Conséquence que cette story **écrit et teste** sans la corriger (F-3 de la P1) :
   après un échec de la dernière transaction, les quatre premières sont commitées (société renommée,
   plan, exercice) et **un rejeu de `seed-demo` est impossible** — `bulk_create_from_chart` fait des
   `INSERT` secs (`accounts.rs:935-1000`) et `create_for_seed` rend `Invariant(FY_OVERLAP_KEY)`
   (`fiscal_years.rs:205-218`) : 500 jusqu'à une remise à zéro. La suite est à la portée de cette story.
3. **Aucun changement de schéma** : ni migration, ni P1–P8.

## Acceptance Criteria

**1. `seed-demo` écrit `installation.demo_seeded` puis l'étape, dans sa dernière transaction**
(C-15-7-4, C-15-7-14). `kesh_seed::seed_demo(pool, locale, actor)` — `actor =
(user_id, Option<api_key_id>)` ; les paramètres `onboarding_version` **et `ui_mode`** disparaissent :
l'un et l'autre sont **relus sous verrou** sur l'état que rend `lock_state_in_tx` (`ui_mode` :
`state.ui_mode.unwrap_or(UiMode::Guided)`, la règle par défaut du handler actuel, `routes/onboarding.rs:187`).
R-5 de la P1 : la version était relue sous verrou mais `ui_mode` venait de la lecture non verrouillée du
handler — deux sources pour un même état. Le handler cesse de lire `ui_mode`.
Le handler **garde** sa pré-vérification **non verrouillée** `step_completed != 2`
(`routes/onboarding.rs:182-185`) : sans elle, les validations de société, de plan et d'exercice
s'exécuteraient avant que la garde finale ne refuse — à l'inverse des neuf routes de la 15-7a2, où la
lecture non verrouillée disparaît. Les quatre premières validations (verrou de comptage, société,
plan comptable, exercice) sont **inchangées** (inventaire § 2) ; la **dernière transaction** devient,
dans cet ordre (verrou d'état en tête, puis celui de `finalize`) :
  1. `onboarding::lock_state_in_tx` (15-7a1) ; `None` (ligne absente, alors que le handler l'a
     garantie par `get_or_init_state` : corruption) ⇒ `SeedError::Db(DbError::Invariant(…))`, 500, rien
     d'écrit ; revérification `step_completed == 2` sous verrou (sinon
     `SeedError::StepAlreadyCompleted`, rendu `400 ONBOARDING_STEP_ALREADY_COMPLETED`) ;
  2. `companies::clear_stub_in_tx` sur la société (l'`UPDATE companies SET is_stub = FALSE` du
     handler, `:211-214`, **disparaît** du handler). **Changement de sémantique assumé** (F-3 de la P3
     de la 15-7a1) : l'ancien `UPDATE` n'était borné à aucune société et ne bumpait pas `version` ;
     `clear_stub_in_tx` est borné à l'id et bumpe `version` si le drapeau était levé — la version de la
     société bouge désormais sur la voie démo ;
  3. `company_invoice_settings::insert_with_defaults_in_tx` (lectures verrouillantes des comptes de
     rôle) **puis** `vat_rates::seed_default_swiss_rates_in_tx` — l'ordre de `finalize`
     (`routes/onboarding.rs:721` puis `:745`, F2-6 de la P2 : la rédaction précédente inversait les deux
     et les plaçait sous l'étiquette « Pattern 5 », qui ne nomme pas `vat_rates`). Séquence de verrous
     de la transaction, écrite telle quelle à la ligne `seed_demo` du Pattern 5 (AC 10) :
     `onboarding_state → companies → accounts → company_invoice_settings → vat_rates → audit_log` ;
  4. `onboarding::update_step_in_tx(tx, 3, true, Some(ui_mode_verrouillé), version_verrouillée)` ;
  5. `installation.demo_seeded` puis `record_step_completed_in_tx(…, 2, 3, "seed_demo")` ; `COMMIT`.
**Rejeu sur interblocage** (F2-3 de la P2, C-15-7-37 — décision C54 de l'epic : tout flux d'écriture
rejoue sur 1213) : la dernière transaction est enveloppée dans `kesh_db::retry::retry_with`
(`DEFAULT_MAX_DEADLOCK_ATTEMPTS`, patron `finalize`, `routes/onboarding.rs:606-614`) ; chaque essai
`BEGIN` → `COMMIT`, rollback explicite sur erreur dont l'erreur éventuelle est journalisée et **jamais**
substituée à l'erreur d'origine (même règle que l'AC 6 de la 15-7b2). ⚠️ **Le prédicat doit voir les
erreurs de l'essai** (C-15-7-27) : `is_deadlock_error` ne reconnaît que `DbError::Sqlx`, alors qu'un
`?` brut sur `sqlx::Error` produirait `SeedError::Sqlx`. D'où : le corps d'un essai est une fonction
privée qui rend `Result<_, SeedAttemptError>`, **`#[doc(hidden)] pub enum SeedAttemptError {
Db(DbError), StepAlreadyCompleted }`**, avec `From<DbError>` et **sans** `From<sqlx::Error>` — `begin`
et `commit` passent par `map_db_error`, sinon le code ne compile pas —, et le prédicat est
**`#[doc(hidden)] pub fn is_seed_retryable(e: &SeedAttemptError) -> bool { matches!(e,
SeedAttemptError::Db(d) if is_deadlock_error(d)) }`** ; après `retry_with`, `SeedAttemptError` se
convertit en `SeedError` (`Db`, `StepAlreadyCompleted`). La 15-7b2 **étend** ce type (variante
`ResetForbidden`) et ce prédicat pour la remise à zéro, et son test 13 l'exerce sur un **vrai** 1213 :
le prédicat de cette fiche est prouvé là (même release, C-15-7-31) ; le type est la garde de la
conversion. L'interblocage de `seed_demo` lui-même n'est pas provoqué par un test (non déterministe) —
angle mort écrit au Dev Agent Record. *(Levé à la revue de code P1, B-3 : un déclencheur lève une vraie
1213 à la première écriture de la synthèse ; tests `is_seed_retryable_accepts_1213_and_only_it` et
`seed_demo_last_transaction_is_replayed_on_deadlock`, C-15-7b1-2.)*
La **boucle de retry** existante (`lib.rs:184-219` : `max_retries = 3`, soit **trois rejeux, quatre
essais**, 50 ms, sur `InactiveOrInvalidAccounts` — R2-5 de la P2) est **conservée telle quelle**
(C-15-7-14, arbitrage réservé à Guy) et enveloppe le `retry_with` : un essai annulé n'écrit ni taux,
ni réglages, ni trace. **Non exercée par les tests, et c'est assumé** (F-4 / R-7 de la P1) : dans la
dernière transaction, `InactiveOrInvalidAccounts` est permanent (lectures verrouillantes), son rejeu
est « sans effet attendu ». La P2 (F2-3) n'en change rien : le rejeu sur 1213 s'**ajoute** à elle, il
ne la remplace pas.
**Correspondance d'erreurs du handler** (R2-3 de la P2), écrite à T2 : `StepAlreadyCompleted` ⇒
`400 ONBOARDING_STEP_ALREADY_COMPLETED` ; **toute autre erreur** garde le repli actuel
`AppError::Internal(format!("Seed demo failed: {other}"))` (`routes/onboarding.rs:203`), **500** — un
1213 épuisé compris. La correspondance `Db(d) ⇒ AppError::Database(d)` de la 15-7b2 n'est **pas**
reprise ici : elle rendrait le rejeu du test 2 en **409** (`UniqueConstraintViolation` sur
`uq_accounts_company_number`) et le perdant concurrent en 409 (`OptimisticLockConflict`).
`installation.demo_seeded` : `entity_type = "installation"`, `entity_id = AUDIT_ENTITY_ID_NONE`,
`details = {"company_id", "org_type", "accounts_created", "fiscal_year_id", "vat_rates_created",
"invoice_settings_created"}` — `accounts_created` = longueur du `Vec<Account>` rendu par
`bulk_create_from_chart`, `fiscal_year_id` = id rendu par `create_for_seed`, `vat_rates_created` =
longueur du `Vec<VatRate>` rendu, `invoice_settings_created` = le booléen rendu. Les données de
démonstration n'écrivent **pas** d'entrée par fait de domaine (C-15-7-4).

**7. Le constructeur dit la vérité sur l'acteur** : `for_actor(user_id, api_key_id, …)` dans
`kesh-seed`, acteur threadé depuis `Extension(current_user)` du handler `seed_demo`. Aucun
`NewAuditLogEntry::user(` dans `kesh-seed/src/lib.rs` (garde de source, test 11). La 15-7b2 étend
le même acteur à `reset_demo`.

**8. Le registre des routes passe `seed_demo` à `Traced`** (`audit_route_registry.rs:164`) ;
partition **recomptée depuis la source** : sur la base de la 15-7a2 mergée (`200f5e79`), `traced`
104 → **105**, `exempt` 6 → **5** (`reset` et les quatre de #435), `no_matter` **2**, total **112**
(T0 du développement : la fiche écrivait 103 → 104 et 3 « sans matière » ; la 15-7a2 a livré 104 / 6 / 2,
recompté depuis `LIB_ROUTES`). Le message
« 2 routes d'onboarding (#434, 15-7b1, 15-7b2) » (chaîne exacte écrite par la 15-7a2, AC 10 ; `:478`
après elle) devient « 1 route d'onboarding (#434, 15-7b2) ». Le message de l'assertion **`traced`**
(`:469-475`, qui énumère les contributions au total depuis la 15-7a2) reçoit « plus le peuplement de
démonstration (15-7b1, #434) » (R2-6 de la P2). **Le motif `Exempt` de `reset`** (`:76`),
réécrit par la 15-7a2 en « issue #434 — peuplement de démonstration (15-7b1) et remise à zéro
(15-7b2) », est **rectifié par cette fiche** en « issue #434 — remise à zéro (15-7b2) » (R-3 de la P1).
⚠️ *Recompter au merge.*

**9. Une action neuve, déclarée et libellée dans les quatre catalogues** (`ACTIONS` triée, quatre
`.ftl` près de `audit-log-action-installation-ui-mode-changed`, fr `:2295`) :

| Action | Clé | fr-CH | de-CH | it-CH | en-CH |
|---|---|---|---|---|---|
| `installation.demo_seeded` | `audit-log-action-installation-demo-seeded` | Données de démonstration chargées | Demodaten geladen | Dati dimostrativi caricati | Demo data loaded |

**10. Les doc-comments que la story rend faux disent le nouvel état.** Grep exécuté, sortie collée au
Dev Agent Record :
`grep -rnE "#434|seed_demo|KF-002-H-002|lock-and-release|contexte système|sans audit log|onboarding_version|is_stub|Deny list" crates docs/MULTI-TENANT-SCOPING-PATTERNS.md`
(`is_stub` rend de nombreuses occurrences légitimes, à trier à la main ; un site du grep absent de la
liste se traite quand même, ou se renvoie **nommément** à la 15-7b2). Sites **attribués à cette fiche**
(attribution complète dans l'index) :
- `kesh-seed/src/lib.rs` — doc de `seed_demo` (`:70`, `onboarding_version`), commentaire `:80-93` sur
  la fenêtre résiduelle, `:155-156`, `:167` `create_for_seed`. La doc de `seed_demo` **dit** (F-3 de la
  P1) : « après un échec de la dernière transaction, la société renommée, le plan et l'exercice restent
  commités ; relancer `seed-demo` échoue (500) — seule la remise à zéro rend l'installation à l'étape
  de départ, **et seulement par l'API** : l'interface ne la propose que dans la bannière de
  démonstration, affichée si `isDemo` (`frontend/src/routes/(app)/+layout.svelte:337-338`), alors que
  l'échec laisse `is_demo = false` (F2-4 de la P2) ; atomicité des quatre premières validations : #538 » ;
- `kesh-api/src/routes/onboarding.rs:204-216` (handler `seed_demo` : `UPDATE is_stub` retiré) ;
- `kesh-db/src/repositories/vat_rates.rs:323-324` et `company_invoice_settings.rs:516` (« Utilisée par
  `seed_demo` (Path A) ») — `seed_demo` n'appelle plus les enveloppes pool ; `company_invoice_settings.rs:588-598`
  — la boucle de retry de `seed_demo` reconnaît désormais la variante rendue par `_in_tx` ; et sa
  **raison d'être change** : le rejeu se justifiait par la visibilité entre transactions
  (`bulk_create_from_chart` commité hors verrou) ; dans la dernière transaction,
  `insert_with_defaults_in_tx` lit les comptes par lectures verrouillantes (dernier état commité), si
  bien que `InactiveOrInvalidAccounts` y est permanent. Le commentaire dit : « rejeu conservé par
  prudence, sans effet attendu » — ne pas réécrire l'ancienne justification ;
- `kesh-db/src/repositories/fiscal_years.rs:199-204` (doc-comment ; à relocaliser par symbole) — `create_for_seed` : la démonstration est tracée
  par sa synthèse (`installation.demo_seeded`), pas par fait de domaine ;
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md:314` (`:313` est la ligne `reset`, à la 15-7b2 — R3-5 de la P3 de la 15-7b2) — la ligne `seed_demo` de la table, qui écrit la
  séquence de verrous exacte de la dernière transaction (AC 1, étape 3) et son rejeu sur 1213 ; `:320-330`
  (« Known Risk — KF-002-H-002 ») : `seed_demo` reste en lock-and-release **pour ses quatre premières
  validations** — renvoyer à **#538**, jamais à #43 (close le 2026-05-02, sans rapport : elle portait le
  rejeu sur interblocage de `finalize`, R4-3 de la P4 de la 15-7b).
Sites légitimes à laisser : `kesh-db/src/test_fixtures.rs:176`, `journal_entries.rs:1189-1190`,
`retry.rs:7`, `reconciliation.rs:3831`.

**11. Les manuels disent ce que l'inventaire établit**, PDF régénérés (`make fr`) et contrôlés
aplatis :

| Site | Après la 15-7a2 | Après la 15-7b1 |
|---|---|---|
| `admin-manual.tex:1948` (T0 : était `:1821`) | « 104 des 112 routes » ; exceptions \#434 (2) et \#435 (4), plus deux « sans matière » (104 + 6 + 2) | « **105** des 112 routes » ; exceptions : la remise à zéro des données de démonstration (\#434) et les quatre gestes de session (\#435), plus les deux routes « sans matière » (105 + 5 + 2 = 112) |
| `admin-manual.tex:2004` | « le peuplement de démonstration et la remise à zéro (\#434), et les gestes de session (\#435) » | « la remise à zéro des données de démonstration (\#434) et les gestes de session (\#435) » |
| `user-manual.tex:2019-2023` | « Deux familles … le peuplement de démonstration et sa réinitialisation, et les gestes de session » | « Deux familles … la **réinitialisation** des données de démonstration, et les gestes de session » |
| `user-manual.tex:2216` (glossaire) | « les deux familles d'opérations » | inchangé (toujours deux familles) — **relire** |
| `admin-manual.tex:2244` (glossaire) | « voir les réserves de la section Conformité » | relire : ne doit pas contredire `:1821` et `:2004` |
| `user-manual.tex`, § *Consulter le journal d'audit* | paragraphe des entrées de configuration (15-7a2) | le compléter : le chargement de la démonstration y figure en une entrée **lorsqu'il aboutit** ; un chargement interrompu peut laisser une partie des données de démonstration (société renommée, plan comptable, exercice) **sans entrée** — limite connue, suivie par \#538 (F2-1 de la P2) |
| `user-manual.tex:179-189` (§ *Chemin A — Mode exploration*, F2-2 de la P2, **#544**) | « une company fictive avec un plan comptable Sterchi PME », « quelques contacts d'exemple », « quelques produits et conditions de paiement », « un exercice fiscal », « des écritures d'exemple » ; « créez une nouvelle company en mode B » | la liste dit ce que crée `seed_demo` : une société fictive (« Démo SA » ou sa traduction), le plan comptable PME, l'exercice de l'année, les quatre taux de TVA et les réglages de facturation — **ni** contacts, **ni** produits, **ni** écritures (`journal-entries.spec.ts:83` le dit aussi) ; la phrase « créez une nouvelle company » est remplacée par le renvoi à la **réinitialisation** (« Réinitialiser pour la production », bannière de démonstration), **avec sa condition** : elle n'est possible que si l'administrateur de l'installation l'a autorisée (`KESH_PRODUCTION_RESET`, manuel d'administration) — sinon elle est refusée, même à un administrateur, la démonstration étant à l'étape 3 (`routes/onboarding.rs:276`) ; report de F4-2/R4-2 de la P4 de la 15-7b2, C-15-7-48 — l'installation est mono-société (`kesh-seed/src/lib.rs:105-111`) |

⚠️ **Greper la valeur, sans casse** (`grep -niE`) : `\b434\b`, `\b103\b`, `\b104\b`, `\b112\b`,
« deux familles » (minuscule à `user-manual.tex:2216`, majuscule à `:2019`), `peuplement`,
`aboutit|interrompu` (F2-1), `contacts d.exemple|écritures d.exemple|nouvelle company` (F2-2 : zéro
occurrence attendue après la story),
`séquence d.installation` (apostrophe typographique dans le PDF), sur `docs/manual/fr/*.tex`, le PDF
aplati, `README.md`, `website/`. La ligne v0.12.1 du README est historique.

**12. CHANGELOG** : sous `## [0.13.0] — Non publié`, `### Corrigé` : le chargement de la démonstration
s'inscrit au journal d'audit, en une entrée de synthèse (#434).

## Tasks / Subtasks

- [x] **T1 — `kesh-seed::seed_demo`** (AC 1, 7) — `SeedError::StepAlreadyCompleted`, `SeedAttemptError` et `is_seed_retryable`, signature `actor`, dernière transaction (réglages puis taux), `retry_with` sur 1213 autour d'elle, boucle `InactiveOrInvalidAccounts` conservée autour, `serde_json` ajouté à `crates/kesh-seed/Cargo.toml`.
- [x] **T2 — Handler** (AC 1, 7) — `seed_demo` : `Extension(current_user)`, `StepAlreadyCompleted` ⇒ 400, **toute autre erreur ⇒ `AppError::Internal`, 500** (repli actuel conservé, R2-3 de la P2), plus d'`UPDATE is_stub` ni de lecture de `ui_mode` (pré-vérification conservée).
- [x] **T3 — Libellés et registre** (AC 8, 9).
- [x] **T4 — Doc-comments** (AC 10).
- [x] **T5 — Tests** (Dev Notes § Tests) — helper de création de jeton remonté dans `tests/common/mod.rs`, `api_keys_e2e.rs` adapté (dix appels), création de jeton du test 8 de la 15-7a2 (`onboarding_audit_e2e.rs`) remplacée par le helper.
- [x] **T6 — Manuels, PDF, CHANGELOG** (AC 11, 12).
- [x] **T7 — Gates** : `kesh-db` non touché en code (commentaires seuls), mais `kesh-seed` et les tests DB le sont ⇒ gate complet avant push ; base remise à zéro avant ; E2E complet au dernier commit de code (règle D7), jugé contre les échecs attendus de `docs/testing.md`.

## Dev Notes

### Points de vigilance

- **Concurrence avec `start-production`** (les deux partent de l'étape 2) : les premières validations
  de `seed_demo` commitent, la dernière trouve l'étape 3 et rend `StepAlreadyCompleted` ; une
  installation **de production** à l'étape 3 garde le nom « Démo SA », le plan PME et l'exercice.
  Fenêtre étroite, **suivie par #538** ; non corrigée ici. *(Revue de code P1, B-1 = E-3 : ce résidu
  est désormais écrit au doc-comment de `seed_demo`, au CHANGELOG et aux deux manuels ; la course est
  rendue déterministe par le test `seed_demo_race_with_start_production_is_a_400`.)*
- **Deux `seed-demo` concurrents** (F-3 de la P1) : tous deux passent la pré-vérification non
  verrouillée ; le perdant échoue en **500** — à `companies::update` (conflit de version,
  `OptimisticLockConflict`, s'il a lu la société avant l'`update` du gagnant : le verrou de comptage
  est relâché avant lui, `kesh-seed/src/lib.rs:116-125`) ou à `bulk_create_from_chart` (doublon de
  comptes, `uq_accounts_company_number`), selon l'entrelacement (R2-4/F2-5 de la P2) ; jamais en
  `StepAlreadyCompleted` — la revérification sous verrou n'est atteinte que contre `start-production`.
  Constaté, non corrigé ici (même cause que le résidu de l'inventaire § 2, #538).
- **Dépendances vérifiées par défaut seulement** (F-6 de la P1) : `## [0.13.0]` (AC 12) et la cellule
  « 103 des 112 » de l'`admin-manual.tex:1821` (AC 11) n'existent qu'après la 15-7a2 ; la base actuelle
  porte encore « 87 des 105 ». **À re-contrôler au développement**, sur la base de la 15-7a2 mergée.
  *(Re-contrôlé au T0 du développement, sur `200f5e79` : `## [0.13.0] — Non publié` existe ; le manuel
  porte « 104 des 112 », `:1948` — cf. AC 8 et 11, corrigés.)*
- **Tests existants qui changent de sens** : `onboarding_e2e.rs` (chemin `seed-demo`), tout test qui
  passe `onboarding_version` à `seed_demo` (`grep -rn "seed_demo(" crates`), `kesh-seed` lui-même ;
  `crates/kesh-api/tests/fiscal_years_e2e.rs` si un `COUNT(*)` global d'`audit_log` suit un
  `seed-demo`.
- **Statut de la fiche** : `ready-for-dev` pendant les passes de validation est la convention de
  l'epic ; la clôture de la validation se lit au Change Log, pas au statut.

### Tests

Fichier `crates/kesh-api/tests/onboarding_audit_e2e.rs` (créé par la 15-7a2), même patron : lecture
d'`audit_log` par les helpers de `crates/kesh-api/tests/common/mod.rs`, dont `audit_sequence` ajouté
par la 15-7a2 (séquence exacte, contenu des détails), DDL de test par **`sqlx::raw_sql`**.

**Montage des tests 1, 2 et 3, fixé** (R-2 de la P1) : **`ensure_admin_user`** sur base vide, avec la
configuration d'administrateur, **sans** `create_test_company` — le bootstrap insère alors la société
**provisoire** (`is_stub = TRUE`) et l'administrateur, et écrit `user.created` (C-15-7-30) ; patron
`onboarding_e2e.rs:581` (`fresh_install_stub_cleared_by_seed_demo`). Chaque test **asserte
`is_stub = TRUE` avant** `seed-demo` — sans quoi les assertions sur `is_stub` seraient creuses
(`create_test_company` pose une société non provisoire).

| # | Test | AC |
|---|---|---|
| 1 | Démonstration `language` → `mode` → `seed-demo` : `is_stub = TRUE` avant ; séquence exacte (`user.created`, entrées de la 15-7a2, puis `installation.demo_seeded`, puis l'étape 2→3), détails cohérents avec la base (`accounts_created` = `COUNT(*)` des comptes, `fiscal_year_id` existant, `vat_rates_created` = 4, `invoice_settings_created` = true), `is_stub = FALSE` après. **Variante** : montage à deux taux préexistants (8.10 et 2.60 au 2024-01-01 avant `seed-demo`) ⇒ `vat_rates_created = 2` | 1 |
| 2 | Atomicité de la dernière transaction de `seed_demo` — déclencheur de test **sélectif** (F-1 de la P1), posé **après** la montée à l'étape 2 (sinon les `installation.step_completed` de `language` et `mode` échouent d'abord, R-4) : `BEFORE INSERT ON audit_log FOR EACH ROW … IF NEW.action = 'installation.step_completed' THEN SIGNAL …`, soit la **seconde** écriture de la transaction (patron du test 9 de la 15-7a2, DDL par `sqlx::raw_sql`). Assertions : 500 ; **aucune entrée neuve** (le compte d'`audit_log` relevé juste avant l'appel est celui d'après), en particulier zéro `installation.demo_seeded` — la première écriture a été annulée avec la transaction ; étape toujours 2 ; `is_stub = TRUE` (avant et après) ; `vat_rates` et `company_invoice_settings` vides. **Résidu asserté** (F-3 de la P1) : comptes > 0, un exercice, société renommée — les quatre premières validations sont commitées. **Rejeu impossible asserté** : `DROP TRIGGER`, nouveau `POST seed-demo` ⇒ 500 (renvoi à #538 en commentaire du test ; C-15-7-13 ne prévoyait un rejeu réussi qu'avant que l'inventaire n'établisse ce résidu). **Variante** : déclencheur sur `NEW.action = 'installation.demo_seeded'` (la première écriture), mêmes assertions | 1 |
| 3 | `seed-demo` par jeton d'API : jeton **`read-write`** (un jeton en lecture seule prend 403), créé par `POST /api/v1/settings/api-keys` **sous le JWT de l'administrateur** (la route est interdite aux jetons). Le helper `create_key_via_http` (aujourd'hui privé à `api_keys_e2e.rs:179`, sur le `TestApp` local) **remonte dans `tests/common/mod.rs`**, paramétré par client et URL de base ; `api_keys_e2e.rs` l'emploie (**dix** appels, `:205` à `:521` ; la ligne `:179` est la définition — R2-1/F2-7 de la P2), **et le test 8 de la 15-7a2**, dans `onboarding_audit_e2e.rs`, aussi : la création de jeton qu'il porte (copie ou écriture en ligne, le helper étant privé à sa naissance) est **remplacée** par l'appel au helper remonté. Les huit autres fichiers qui créent leur jeton en ligne (`admin_full_export_e2e.rs:406`, `admin_full_import_e2e.rs:414`, `admin_pat_denied_e2e.rs:731`, `audit_log_e2e.rs:254`, `fiscal_years_e2e.rs:1611`, `invoice_unvalidate_e2e.rs:517`, `invoice_frozen_pdf_e2e.rs:1004`, `reconciliation_e2e.rs:4209`) restent **hors périmètre** — la fiche ne prétend pas les couvrir. Séquence attendue : `user.created`, `api_key.created`, entrées de la 15-7a2, `installation.demo_seeded`, `installation.step_completed` ; `actor_type = 'api_key'` sur les **deux dernières** ; journal lu **par le pool** (il n'est pas lisible par jeton, `user-manual.tex:2044-2045`) | 7 |
| 11 | Gardes de source : (a) `NewAuditLogEntry::user(` absent de `kesh-seed/src/lib.rs` — **vert dès avant la story**, garde de régression, **pas** preuve de l'AC 7, que porte le test 3 ; (b) `UPDATE companies SET is_stub` absent de `kesh-api/src/routes/onboarding.rs` (R-1/F-2 de la P1 : aucun test de comportement ne distingue un `UPDATE` résiduel redondant) — **rouge avant la story** (`:211`). *Revue P1 (B-4 = A-4) : les deux sources sont normalisées (blancs réduits, casse abaissée) avant la recherche ; angle mort assumé : une requête assemblée de constantes ou de fragments de chaîne* | 1, 7 |
| 12 | *(revue P1, E-1 = A-1)* `kesh_seed::seed_demo` appelé **directement** (sans la pré-vérification non verrouillée du handler) sur une installation passée à l'étape 3, puis 4 (`seed_demo_refuses_step_{3,4}_under_lock`) : `Err(StepAlreadyCompleted)` ; séquence d'audit, étape **et version** inchangées, `is_demo` non levé, `is_stub` intact, ni taux ni réglages ; plan commité (la garde est celle de la dernière transaction) | 1 |
| 13 | *(revue P1, E-1 = A-1)* La course `start-production` / `seed-demo` rendue déterministe : déclencheur `AFTER INSERT ON fiscal_years` qui pose l'étape 3 (`seed_demo_race_with_start_production_is_a_400`) : `400 ONBOARDING_STEP_ALREADY_COMPLETED`, la dernière transaction n'écrit rien | 1 |
| 14 | *(revue P1, B-3)* `is_seed_retryable` sur une **vraie** 1213 (`SIGNAL … MYSQL_ERRNO = 1213`, passée par `map_db_error`) : vrai ; 1205, `OptimisticLockConflict`, `StepAlreadyCompleted` : faux | 1 |
| 15 | *(revue P1, B-3)* Rejeu de bout en bout : déclencheur qui lève une 1213 à la **première** écriture de `installation.demo_seeded` seulement (compteur dans une table MyISAM, que l'annulation n'efface pas) ⇒ 200, une seule synthèse, étape 3 (`seed_demo_last_transaction_is_replayed_on_deadlock`) | 1 |

**AC prouvés par les gardes existantes** (R-7 de la P1) : AC 8 par
`the_registry_partition_is_what_the_story_declares` (`audit_route_registry.rs`) ; AC 9 par
`audit_label_registry.rs` (balaie `crates/*/src/**`, `kesh-seed` compris) ; AC 10 et 11 par les greps
collés au Dev Agent Record ; AC 12 (CHANGELOG) par **aucun test** — relu au Dev Agent Record.

⚠️ **Prouver que les tests mordent** — chaque mutation seule, puis le fichier **touché** (cargo garde
sinon le binaire muté) :

| Mutation | Attendu |
|---|---|
| écrire `installation.demo_seeded` **hors** de la dernière transaction, par le pool | test 2 (déclencheur sur `installation.step_completed`) rouge : l'entrée survit au 500 |
| sortir `clear_stub_in_tx` de la dernière transaction (appel par le pool **avant** elle) — R-1/F-2 de la P1 : la mutation « `UPDATE` laissé dans le handler après l'appel » ne mordait pas, le `?` rendant la main avant lui | test 2 rouge : `is_stub` levé malgré l'échec |
| remettre l'`UPDATE companies SET is_stub = FALSE` dans le handler | test 11 (b) rouge |
| écrire l'entrée par `NewAuditLogEntry::user(` | test 3 rouge (`actor_type = 'user'`), et test 11 (a) |
| `vat_rates_created` en dur à 4 | variante du test 1 rouge (`2` attendu) |
| prédicat `is_seed_retryable` toujours faux *(revue P1, B-3 ; l'angle mort d'origine, F2-3 de la P2, est levé)* | tests 14 et 15 rouges (la route rend 500) |
| revérification de l'étape sous verrou retirée de `final_body` *(revue P1, E-1 = A-1)* | tests 12 (étapes 3 et 4 : `Ok(())`) et 13 (200) rouges |
| bras `StepAlreadyCompleted ⇒ 400` retiré du handler *(revue P1, E-1 = A-1)* | test 13 rouge (500) |
| `UPDATE companies` / `set is_stub` remis dans le handler, sur deux lignes et en minuscules *(revue P1, B-4 = A-4)* | test 11 (b) rouge |

### Ce que la story ne fait pas

- **La remise à zéro** (`reset`, #528, #279) : la 15-7b2.
- **L'atomicité des quatre premières validations de `seed_demo`**, et donc le rejeu après échec :
  hors périmètre, **suivi par #538** (inventaire § 2) ; le résidu est écrit et testé ici (test 2).
- **Le RBAC de `seed-demo`** (tout rôle authentifié) : constaté, hors périmètre.
- **Dette — message du `422`** (revue de code P1, E-2, LOW ; C-15-7b1-3) : le bras
  `InactiveOrInvalidAccounts` du handler dit « Vérifiez que le plan comptable a bien été chargé avant
  de relancer la démo » ; l'erreur survient désormais dans la dernière transaction, **après** le commit
  du plan et de l'exercice, et relancer échoue (500, doublon de comptes). Le texte n'est pas changé ici
  (code de production exécutable, règle de la remédiation) ; à reprendre avec l'atomicité des quatre
  premières validations, **#538**, qui fait disparaître l'état non relançable. Atteinte pratiquement
  impossible avec le plan PME embarqué.
- **Boucle `InactiveOrInvalidAccounts`** (revue de code P1, B-2, LOW) : rejeu ×4 d'une erreur
  permanente, code non exercé — conservé, **arbitrage réservé à Guy** (C-15-7-14).
- **La remise à zéro dans l'interface après un échec de `seed-demo`** (F2-4 de la P2) : la bannière qui
  la propose n'est affichée qu'en démonstration ; le chemin par l'API est écrit (AC 10), l'interface
  n'est pas changée — signalé pour le complément de #538.
- **#435**, **#431** — inchangées. Aucun changement de code frontend, aucune migration.

### References

- Issues #434, #538, #544.
- Fiche d'origine `15-7b-trace-demo-et-remise-a-zero.md` (corps vidé ; version complète au commit
  `9bbc52de`) ; fiche sœur `15-7b2-remise-a-zero.md` ; fiches `15-7a1-socle-transactions-onboarding.md`,
  `15-7a2-trace-installation-production.md` ; fiche index `15-7-trace-onboarding.md`.
- Passes de la 15-7b : prompts versionnés `15-7b-validate-prompt-p{2,3,4}.md` ; rapports
  `target/gate-logs/15-7b-p{2,3,4}-{R,F}.md` (non versionnés). Passes de cette fiche : prompts versionnés
  `15-7b1-validate-prompt-p{1,2}.md` ; rapports `target/gate-logs/15-7b1-p{1,2}-{R,F}.md` (non versionnés).

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`), agent de développement en autonomie (consignes de l'Epic 15),
worktree `kesh-15-7b1`, cible `CARGO_TARGET_DIR` propre, bases `kesh_157b1` / `kesh_e2e_157b1`.

### Debug Log References

Journaux non versionnés : `target/gate-logs/15-7b1-gate.log` (backend), `15-7b1-front.log`,
`15-7b1-e2e.log`, `15-7b1-backend-e2e.log` ; mutations : `scratchpad/mut157b1/M{1..5}-*.log`.

### Completion Notes List

- **T0** : écarts consignés au Change Log (partition 104/6/2 → 105/5/2, colonne `Rejeu`, bras `422`,
  renvois relocalisés) ; choix **C-15-7b1-1**. Aucun écart ne changeait une règle ni un AC.
- **Tests d'abord** : les six tests neufs ont été écrits avant le code et exécutés sur `200f5e79` + tests :
  **5 rouges** (tests 1-variante, 2, 2-variante, 3, 11) — le test 1 principal n'a pas été atteint
  (fail-fast) —, la garde 11 rouge sur sa part (b) comme prévu.
- **Gates réels, au dernier commit de code (`638a80ef`, commentaires de `kesh-db` ; la suite ne porte que
  des `.tex`, PDF, CHANGELOG et la fiche)**, base remise à zéro avant (DROP/CREATE de mes deux bases,
  migrations, seed) :
  - backend `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest) : **2979/2979**, 4 ignorés ;
  - frontend : `npm run check` vert, `lint-i18n-ownership` PASS, Vitest **1095/1095** (112 fichiers),
    `npm run build` vert ;
  - E2E complet (port 3016, secrets aléatoires, `KESH_TEST_MODE=true` des deux côtés,
    `KESH_COOKIE_SECURE=false`, SMTP factices, `/health` → `smtpConfigured:true`, inbox et documents du
    scratchpad) : **245 passés, 9 échecs, 19 ignorés**, tous attendus selon `docs/testing.md` — les 7
    KF-029 (`mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`) et
    les 2 KF-045 (`invoices:415`, `:439`), run à 05:50–06:03 UTC, avant midi. Backend arrêté par son PID.
  - gate ciblé intermédiaire : 122/122 (`onboarding_audit_e2e`, `api_keys_e2e`, `audit_route_registry`,
    `audit_label_registry`, `onboarding_e2e`, `fiscal_years_e2e`, `kesh-seed`).
- **Mutations jouées, chacune seule, fichier restauré puis touché** — 5/5 rouges :
  M1 `demo_seeded` écrite par le pool hors de la dernière transaction → test 2 rouge (« aucune entrée
  neuve » : 5 ≠ 4) ; M2 `clear_stub_in_tx` sorti de la transaction → test 2 rouge (drapeau levé) ;
  M3 `UPDATE … is_stub` remis dans le handler → test 11 rouge ; M4 `NewAuditLogEntry::user(` → test 3
  rouge (`actor_type = 'user'`) et test 11 rouge ; M5 `vat_rates_created` en dur à 4 → variante du
  test 1 rouge (4 ≠ 2). Restauration vérifiée par grep (zéro résidu).
- **Angles morts déclarés** (fiche, mutations non distinguées), **après la revue de code P1** : boucle
  `InactiveOrInvalidAccounts` non exercée (C-15-7-14) ; garde de source 11 (b) aveugle à une requête
  assemblée de fragments. *Retirés à la P1* : `ui_mode` lu hors verrou (E-4 — la mutation n'existe
  plus, le paramètre ayant disparu) ; `retry_with` retiré ou prédicat faux (B-3 — tests 14 et 15).
- **Revue de code P1 — remédiation** (commit « fix(15-7b1): revue P1 — … ») : **5 tests neufs** dans
  `onboarding_audit_e2e.rs` (28 → **33**, recompté `grep -cE '#\[sqlx::test|#\[test|#\[tokio::test'`
  aux deux bornes `99335b53` / commit de remédiation) : `seed_demo_refuses_step_3_under_lock`,
  `seed_demo_refuses_step_4_under_lock`, `seed_demo_race_with_start_production_is_a_400`,
  `is_seed_retryable_accepts_1213_and_only_it`, `seed_demo_last_transaction_is_replayed_on_deadlock` ;
  garde 11 (b) normalisée. **Mutations jouées, chacune seule, fichier restauré puis touché — 4/4
  rouges** : M6 garde sous verrou retirée (`kesh-seed/src/lib.rs`, `if state.step_completed != 2`) →
  tests 12 (×2, `Ok(())`) et 13 (200) rouges ; M7 bras 400 du handler retiré → test 13 rouge (500) ;
  M8 prédicat toujours faux → tests 14 et 15 rouges (500) ; M9 `update companies\n set is_stub`
  (minuscules, deux lignes) dans le handler → test 11 rouge. Journaux : `scratchpad/mut157b1-p1/`.
  **Gate ciblé seulement**, base `kesh_157b1` remise à zéro avant : `cargo fmt --check` vert,
  `cargo clippy --workspace --all-targets -D warnings` vert, `onboarding_audit_e2e` **33/33**,
  `kesh-seed` **2/2**, `audit_route_registry` **11/11** (doc-comment touché)
  (`target/gate-logs/15-7b1-review-p1-gate.log`) ; gate complet et E2E au push.
  **Aucune ligne de code de production exécutable touchée** : doc-comments et commentaires de
  `kesh-seed/src/lib.rs`, `routes/profile.rs`, `repositories/fiscal_years.rs`, doc-comment de
  `tests/audit_route_registry.rs`, tests, CHANGELOG, manuels (`.tex` et PDF, contrôlés aplatis).
- **Décomptes, recomptés depuis la source** (de `200f5e79` à `HEAD`) : tests de
  `onboarding_audit_e2e.rs` 22 → **28** (6 neufs) ; appels du helper remonté : **10** dans
  `api_keys_e2e.rs` + 1 dans le test 8 de la 15-7a2 ; registre **105 / 5 / 2 = 112**, colonne `Rejeu`
  inchangée (22 / 4 / 89 sur 115) ; une action neuve, quatre libellés.
- **AC 10 — grep exécuté** (`grep -rnE "#434|seed_demo|KF-002-H-002|lock-and-release|contexte système|sans audit log|onboarding_version|is_stub|Deny list" crates docs/MULTI-TENANT-SCOPING-PATTERNS.md`) ; sites traités :
  doc et commentaires de `seed_demo` (`kesh-seed`), handler (`UPDATE` retiré, doc réécrite),
  `company_invoice_settings.rs` (variante pool non appelée ; rejeu « conservé par prudence, sans effet
  attendu »), `vat_rates.rs` (variante pool), `fiscal_years.rs::create_for_seed` (synthèse), `accounts.rs`
  (en-tête et `bulk_create_from_chart` : « contexte système » remplacé), `onboarding.rs::update_step_in_tx`
  (le seed trace désormais), `fiscal_years_e2e.rs:1299`, Pattern 5 (ligne `seed_demo` : séquence exacte
  et rejeu) et « Known Risk » (renvoi à **#538**, non à #43). Laissés : les sites légitimes nommés par la
  fiche, `company.rs:178` (juste) ; **renvoyés nommément à la 15-7b2** : `routes/onboarding.rs` doc de
  `reset` (« KF-002-H-002 (issue #43) », `:264-269`, `:314`) et le commentaire de `reset_demo`. Contrôle :
  `grep -rnF "UPDATE companies SET is_stub = FALSE" crates/` ne rend plus que `companies.rs` (reçu E-2).
- **AC 11 — PDF régénérés** (`make fr`, admin et utilisateur ; la brochure, inchangée, n'est pas
  committée) et **contrôlés aplatis** : « 105 des 112 », « 105 + 5 + 2 = 112 », « cinq routes
  exemptées » ; « Deux familles … la réinitialisation » ; « Données de démonstration chargées »,
  « aboutit », « interrompu », `KESH_PRODUCTION_RESET` présents ; **zéro** occurrence de `contacts
  d.exemple|écritures d.exemple|nouvelle company|peuplement` dans les deux `.tex`, les PDF, `README.md`
  et `website/` (la phrase d'accueil « nouvelle company » du § Onboarding a été réécrite en « nouvelle
  installation », et la négation « ni … écritures d'exemple » reformulée, pour que le grep de l'AC tienne).
  Glossaires relus : user « deux familles » toujours juste ; admin renvoie aux réserves, cohérent.
- **AC 12** : entrée `### Corrigé` sous `## [0.13.0]`, relue ; la phrase de l'entrée 15-7a2 « le
  peuplement de démonstration et la remise à zéro restent à tracer » corrigée en conséquence.

- **Clôture (2026-10-09)** — premier `git fetch` : branche déjà sur `200f5e79`, rebase sans objet ; gates
  complets passés sur cet état (backend 2984/2984, Vitest 1095/1095, E2E 244 / 10 attendus), branche
  poussée. **`origin/main` avait avancé pendant ces gates** (`f8b2accd`, 15-6b, #580) : **second rebase**,
  conflits résolus en union — registre des choix (C-15-6b-1 à 3 puis C-15-7b1-1, aucun doublon),
  `sprint-status.yaml` (ligne de la 15-7b1 renumérotée **(37)** au-dessus de la (36) de la 15-6b), PDF des
  deux manuels (binaires : **régénérés** depuis les `.tex` fusionnés par `make fr` à recompilation forcée
  — `latexmk` jugeait le PDF d'administration à jour —, commit `PDF des manuels régénérés`) ;
  CHANGELOG, `.tex` et `messages.ftl` fusionnés sans conflit (une rubrique de chaque sous `[0.13.0]` ;
  2134 clés par locale, aucun doublon). La 15-6b ne touche ni le registre ni `audit_labels.rs`.
  Partition **recomptée depuis `LIB_ROUTES`** sur l'état rebasé : 112 entrées, **105** `Traced`, **5**
  `Exempt`, **2** `NoMatter` ; 115 avec les 3 routes de test.
  **Gates complets sur l'état rebasé** (dernier commit de code : la remédiation P1 rebasée ; la suite ne
  porte que le prompt, les PDF et cette fiche), bases `kesh_157b1` / `kesh_e2e_157b1` remises à zéro avant
  (DROP/CREATE, migrations, seed), après `wait-kesh.sh` :
  - backend `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest) : **3017/3017**, 4 ignorés
    (`target/gate-logs/15-7b1-close2-gate.log`) ;
  - frontend : `npm run check` 0 erreur, `lint-i18n-ownership` PASS, Vitest **1118/1118** (112 fichiers),
    `npm run build` vert (`15-7b1-close2-front.log`) ;
  - E2E complet (port 3016, secrets aléatoires neufs, `KESH_TEST_MODE=true` des deux côtés,
    `KESH_COOKIE_SECURE=false`, SMTP factices, `/health` → `smtpConfigured:true`, inbox et documents
    neufs du scratchpad) : **244 passés, 10 échecs, 19 ignorés**, run achevé à 07:55 UTC
    (`15-7b1-close2-e2e.log`). Jugés fichier par fichier contre `docs/testing.md` : les 7 KF-029
    (`mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`), les 2
    KF-045 avant midi UTC (`invoices:415`, `:439`) et `sidebar-navigation:75`, **rouge rejoué seul** :
    c'est la KF-046, devenue KF-052 (#424), déterministe selon l'état de la base — au premier run, sur
    `200f5e79`, il passait seul (pollution). 7 + 2 + 1 = 10, dans la fourchette 8 à 12 de
    `docs/testing.md`. Backend arrêté par son PID.
  - **Axe manuel repris par l'orchestration de clôture** : la P2 ciblée ne le déclare ni exercé ni non
    exercé alors que `4303ac01` touche les deux `.tex` et leurs PDF ; PDF aplatis contrôlés — les deux
    phrases ajoutées (course avec la configuration de production, renvoi à #538) y figurent, « 105 + 5 +
    2 = 112 » et « 105 des 112 » aussi, et elles concordent avec les tests 12 et 13.

### File List

- `crates/kesh-seed/Cargo.toml`, `crates/kesh-seed/src/lib.rs`, `Cargo.lock`
- `crates/kesh-api/src/routes/onboarding.rs`, `crates/kesh-api/src/audit_labels.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `crates/kesh-api/tests/onboarding_audit_e2e.rs`, `tests/common/mod.rs`, `tests/api_keys_e2e.rs`,
  `tests/audit_route_registry.rs`, `tests/fiscal_years_e2e.rs`
- `crates/kesh-db/src/repositories/{accounts,company_invoice_settings,fiscal_years,onboarding,vat_rates}.rs`
  (commentaires seuls)
- revue P1 : `crates/kesh-api/src/routes/profile.rs` (commentaire seul)
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md`, `docs/manual/fr/{admin,user}-manual.{tex,pdf}`, `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/{15-7b1-trace-demonstration,15-7b2-remise-a-zero,15-7-trace-onboarding,epic-15-choix-autonomes}.md`,
  `sprint-status.yaml`

## Change Log

- 2026-10-08 — **Née du découpage de la 15-7b** à la passe de validation P4 (choix C-15-7-31). Reprend
  le Volet A de la 15-7b (AC 1, et la part « démonstration » des AC 7 à 12), ses tests 1, 2, 3 et 11,
  avec l'historique des passes P2 à P4 de la 15-7b (Change Log de la fiche d'origine, au commit
  `9bbc52de` et dans sa version vidée). **Signal D5 constaté à la P4 de la 15-7b** — R4-2 : le constat
  qui fondait la dérogation au découpage était faux (trois des quatre MEDIUM de la P3 venaient des
  correctifs de la P2) ; F-1/R4-1 : un MEDIUM né de la remédiation P3 — **et suivi** : la coupe Volet A /
  Volet B prévue par la fiche s'applique. Remédiations P4 qui concernent ce volet : R4-3 (#43, close et
  sans rapport, remplacée par #538 ; l'atomicité des quatre premières validations n'a pas d'issue,
  signalé à l'orchestrateur), R4-8 (choix applicables bornés). Mutations du test 1 complétées
  (`vat_rates_created` sur montage à deux taux). Registre : 103 → **104**. Recompte : **7 AC** (1, 7,
  8, 9, 10, 11, 12), **7 tâches**, **4 tests** (1, 2, 3, 11).
- 2026-10-08 — **Passe de validation P1** (prompt versionné `15-7b1-validate-prompt-p1.md` ; deux
  lentilles en contexte frais, R auditeur d'acceptation et F full-scope adversary ; rapports
  `target/gate-logs/15-7b1-p1-{R,F}.md`, non versionnés). Bruts, recomptés depuis les rapports : R
  **2 MEDIUM, 6 LOW** ; F **3 MEDIUM, 3 LOW**. Après fusion (R-1 = F-2 ; R-4 absorbé par F-3 ; R-6 =
  F-5 ; F-4 absorbé par R-7) : **0 CRITICAL, 0 HIGH, 4 MEDIUM, 6 LOW distincts**.

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | R-1 = F-2 | MEDIUM | R, F | mutation « `UPDATE is_stub` laissé dans le handler » tuée par aucun test (le `?` rend la main avant) | mutation remplacée : `clear_stub_in_tx` sorti de la dernière transaction (test 2 rouge) ; garde de source test 11 (b) contre le retour de l'`UPDATE` |
  | R-2 | MEDIUM | R | montage « société stub » conditionnel : assertions `is_stub` creuses | montage fixé par `ensure_admin_user` (stub), `is_stub = TRUE` asserté avant |
  | F-1 | MEDIUM | F | déclencheur global : la mutation « écriture par le pool » survit | déclencheur **sélectif** sur `installation.step_completed`, variante sur `installation.demo_seeded` |
  | F-3 (+ R-4) | MEDIUM | F, R | résidu commité et rejeu impossible après échec de la dernière transaction ; déclencheur à poser après l'étape 2 ; « aucune entrée » = aucune entrée neuve | résidu et rejeu (500) assertés au test 2, écrits dans la doc de `seed_demo` (AC 10), renvoyés à #538 ; inventaire § 2 et Dev Notes (deux `seed-demo` concurrents) |
  | R-3 | LOW | R | chaîne du registre et motif `Exempt` de `reset` désalignés sur la 15-7a2 | chaîne exacte ; motif de `reset` rectifié par cette fiche |
  | R-5 | LOW | R | `ui_mode` pris sur l'état non verrouillé | `ui_mode` relu sous verrou, paramètre retiré |
  | R-6 = F-5 | LOW | R, F | jeton du test 3 : portée, helper, entrée `api_key.created` | `read-write`, JWT d'administrateur, helper remonté dans `tests/common/mod.rs`, séquence complète, lecture par le pool |
  | R-7 (+ F-4) | LOW | R, F | AC 8, 9, 12 sans test nommé ; boucle de retry ni testée ni assumée | gardes existantes nommées, AC 12 relu ; boucle **non testée, angle mort déclaré** (C-15-7-14 réservé à Guy) |
  | R-8 | LOW | R | `fiscal_years.rs:198-203` | `:199-204`, propagé à l'index |
  | F-6 | LOW | F | dépendances (`## [0.13.0]`, « 103 des 112 ») vérifiées par défaut | Points de vigilance : à re-contrôler au développement |

  **Signal D5** : première passe de la fiche, aucun défaut né d'une remédiation ; la fiche reste à sept
  modules touchés dont quatre de code (signal déjà déclaré). **#538** porte désormais l'atomicité des
  quatre premières validations (ajout de l'orchestrateur) : inventaire § 2 et « ne fait pas » mis à
  jour. Changement de sémantique de `clear_stub_in_tx` (F-3 de la P3 de la 15-7a1) écrit à l'AC 1.
  Choix C-15-7-35. Propagation : grep de `onboarding_version`, `seed_demo(pool, locale, ui_mode`,
  `UPDATE is_stub`, `198-203`, `15-7b)`, `aucune issue` sur les quatre fiches et l'index. Recompte :
  **7 AC** (1, 7, 8, 9, 10, 11, 12), **7 tâches**, **4 tests** (1, 2, 3, 11 — le 11 en deux gardes),
  **6 mutations**.
- 2026-10-08 — **Passe de validation P2** (prompt versionné `15-7b1-validate-prompt-p2.md` ; deux
  lentilles en contexte frais, R regression hunter et F full-scope adversary ; rapports
  `target/gate-logs/15-7b1-p2-{R,F}.md`, non versionnés). Bruts, recomptés depuis les rapports : R
  **6 LOW** ; F **2 MEDIUM, 5 LOW**. Après fusion (R2-1 = F2-7, R2-4 = F2-5) : **0 CRITICAL, 0 HIGH,
  2 MEDIUM, 9 LOW distincts**.

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | F2-1 | MEDIUM | F | le paragraphe prescrit au manuel promettait une entrée que le test 2 prouve absente après un échec | AC 11 : « lorsqu'il aboutit ; un chargement interrompu peut laisser des données sans entrée (\#538) » ; motif `aboutit\|interrompu` aux greps |
  | F2-2 | MEDIUM | F | `user-manual.tex:179-189` promet contacts, produits, écritures que `seed_demo` ne crée pas, et « créez une nouvelle company » | ligne ajoutée à l'AC 11 ; **`closes #544`** (issue ouverte par l'orchestrateur, jalon E15) — C-15-7-37 |
  | F2-3 | LOW | F | dernière transaction sans rejeu sur 1213 | `retry_with` autour d'elle (décision C54 de l'epic), `SeedAttemptError` sans `From<sqlx::Error>`, `is_seed_retryable` partagé avec la 15-7b2 (test 13) ; boucle `InactiveOrInvalidAccounts` **non touchée** (C-15-7-14, arbitrage de Guy) — C-15-7-37 |
  | F2-4 | LOW | F | après un échec, la remise à zéro n'est atteignable que par l'API | AC 10 (doc de `seed_demo`) et « ne fait pas » ; signalé pour #538 |
  | F2-6 | LOW | F | « Pattern 5 » sur un ordre que le Pattern ne décrit pas, inverse de `finalize` | ordre de `finalize` (réglages puis taux), séquence de verrous écrite et reportée à la ligne `seed_demo` du Pattern 5 |
  | R2-1 = F2-7 | LOW | R, F | « onze appels » (dix) ; copie du helper dans la 15-7a2 | dix appels ; le test 8 de la 15-7a2 emploie le helper remonté ; huit autres fichiers hors périmètre, nommés |
  | R2-2 | LOW | R | en-tête de tableau orphelin | supprimé |
  | R2-3 | LOW | R | les 500 des tests reposent sur une correspondance d'erreurs non écrite | AC 1 et T2 : toute erreur autre que `StepAlreadyCompleted` ⇒ `AppError::Internal`, 500 ; correspondance de la 15-7b2 non reprise, motif écrit |
  | R2-4 = F2-5 | LOW | R, F | lieu d'échec du perdant de deux `seed-demo` | `companies::update` (version) ou `bulk_create_from_chart` (doublon), selon l'entrelacement |
  | R2-5 | LOW | R | « trois essais » : quatre | « trois rejeux, quatre essais », `lib.rs:184-219` |
  | R2-6 | LOW | R | message de l'assertion `traced` non mis à jour | AC 8 : « plus le peuplement de démonstration (15-7b1, #434) » ; propagé à la 15-7b2 |

  **Signal D5** : aucun défaut né de la remédiation P1 (les deux lentilles le déclarent, revérifié : F2-1
  porte sur une ligne de l'AC 11 écrite à la naissance de la fiche, F2-2 sur un texte préexistant du
  manuel) — pas de recyclage ; toujours sept modules, dont quatre de code. La remédiation touche la
  conception de la dernière transaction (rejeu, ordre) : **la validation n'est pas close**, P3 à lancer
  (une passe ciblée suffit si l'orchestrateur la juge adaptée : un seul module de code, `kesh-seed`,
  porte le changement). Propagation : grep de `onze`, `trois essais`, `188-218`, `Pattern 5`,
  `AlreadyFinalized`, `ResetAttemptError`, `is_reset_retryable`, `contacts d.exemple`, `\b544\b` sur
  les quatre fiches, l'index, le registre et `sprint-status.yaml`. Recompte : **7 AC** (1, 7, 8, 9, 10,
  11, 12), **7 tâches**, **4 tests** (1, 2, 3, 11), **7 mutations**.
- 2026-10-08 — **Passe de validation P3, ciblée** (prompt versionné
  `15-7b1-validate-prompt-p3-ciblee.md` ; une lentille Haiku en contexte frais, braquée sur la seule
  remédiation P2, `git diff 327ea9df ffcdcf8d` de la fiche ; rapport
  `target/gate-logs/15-7b1-p3-ciblee.md`, non versionné). **0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW.** Axes
  exercés : contradictions introduites par chaque hunk (rejeu `retry_with` et boucle
  `InactiveOrInvalidAccounts`, `SeedAttemptError`, `closes #544`, ordre réglages puis taux), faits
  revérifiés au code (`user-manual.tex:183`, `routes/onboarding.rs:721` et `:745`), mutations, recompte.
  **Seul axe non exercé, le PDF : sans objet**, `ffcdcf8d` ne touchant pas `docs/manual/` (vérifié par
  l'orchestrateur). La remédiation P2 relue ne touche aucune ligne de code de production (la fiche est une
  spec ; aucun code n'est écrit) : **validation CLOSE** (C-15-7-43). Un résidu de la P3 de la 15-7b2 y
  est reporté (R3-5 : ligne `seed_demo` du Pattern 5 en `:314`, `:313` étant celle de `reset`). Trend des
  passes de cette fiche : P1 **4 MEDIUM** (Sonnet) → P2 **2 MEDIUM** (Opus) → P3 ciblée **0** (Haiku).
  Recompte inchangé : **7 AC**, **7 tâches**, **4 tests**, **7 mutations**.
- 2026-10-08 — **Report d'un résidu de la P4 de la 15-7b2, sans rouvrir la validation** (C-15-7-48, sur
  le patron du report de R3-5, C-15-7-43) : la cellule `user-manual.tex:179-189` de l'AC sur le manuel
  renvoyait au bouton « Réinitialiser pour la production » sans dire qu'il est refusé, même à un
  administrateur, tant que l'exploitant n'a pas posé `KESH_PRODUCTION_RESET` (F4-2/R4-2 de la P4 de la
  15-7b2) ; la condition y est écrite. Renvoi de doc, pas de conception ; la 15-7b2, qui réécrit le même
  paragraphe après elle, le vérifie (T7). Recompte inchangé : **7 AC**, **7 tâches**, **4 tests**,
  **7 mutations**.
- 2026-10-09 — **Reçu de la revue de code de la 15-7a1** (P1, constat E-2, LOW) : `seed_demo` lève
  `is_stub` par `UPDATE companies SET is_stub = FALSE WHERE is_stub = TRUE` (`routes/onboarding.rs:211`),
  sur le pool, hors transaction, sans borner à `id` ni bumper `version` — contrairement à
  `companies::clear_stub_in_tx`. C'est déjà le périmètre de cette fiche (§ 2 : `clear_stub_in_tx`
  remplace l'`UPDATE` dans la transaction de `seed_demo`) ; le constat est noté pour que le grep de
  fin de développement `grep -rnF "UPDATE companies SET is_stub = FALSE" crates/` ne rende plus que
  `companies.rs`. Recompte inchangé.
- 2026-10-09 — **T0 du développement** : fiche relue contre le code de `200f5e79` (15-7a1 et 15-7a2
  mergées). Écarts, **aucun ne change une règle ni un AC sur le fond** :
  (a) **partition du registre recomptée depuis `LIB_ROUTES`** : la 15-7a2 a livré **104** `Traced` /
  **6** `Exempt` / **2** `NoMatter` (= 112), et non 103 / 6 / 3 ; après cette story, **105 / 5 / 2**.
  Corrigé à l'AC 8, à l'AC 11 (cellule du manuel d'administration : « 105 + 5 + 2 = 112 », « deux »
  routes sans matière), aux Points de vigilance, dans `sprint-status.yaml` (15-7b1 et 15-7b2), dans
  l'index `15-7-trace-onboarding.md` et dans la fiche sœur 15-7b2 (AC 8 : 105 → 106, `no_matter` 2 ;
  cellule du manuel : 106 + 4 + 2) — symptôme grepé (`\b10[3456]\b`, `\b112\b`) ;
  (b) **colonne `Rejeu` du registre, que la fiche ne nomme pas** : `seed_demo` reste
  `SansEcritureAuJournal`, comme les neuf routes d'onboarding tracées par la 15-7a2 (la colonne grave
  l'inventaire de l'AC1 de la 15-5e1, « journal » s'entendant du journal comptable — C-15-7a2-4) ; ses
  compteurs (22 / 4 / 89) sont inchangés. Le point (vi) du doc-comment du registre, qui énumère les routes
  `SansEcritureAuJournal` rejouées quand même, reçoit `seed_demo` (rejouée **dans `kesh-seed`**, hors de
  `src/routes/` : ni le volet (c) ni le (c bis) ne la voient) — C-15-7b1-1 ;
  (c) **le handler a un bras `422`** (`InactiveOrInvalidAccounts` ⇒ `AppError::Validation`) que l'AC 1
  et T2 ne nomment pas (« toute autre erreur ⇒ 500 ») : **conservé** — la fiche n'en demande pas le
  retrait, et le retirer changerait un code de réponse hors périmètre — C-15-7b1-1 ;
  (d) renvois de ligne relocalisés par le texte : `seed_demo` du handler `routes/onboarding.rs:203-245`
  (`UPDATE … is_stub` à `:236`) ; `finalize_inner` (réglages puis taux) ; `admin-manual.tex:1948`
  (« 104 des 112 »), `:2131` (réserve OLICo), `:2376` (glossaire) ; `user-manual.tex:177-189`
  (§ Chemin A), `:2131-2134` (« Deux familles »), `:2182-2196` (§ entrées de la configuration
  initiale), `:2352` (glossaire) ; `MULTI-TENANT-SCOPING-PATTERNS.md:325` (ligne `seed_demo`), `:340-348`
  (« Known Risk ») ; `vat_rates.rs:351-352` et `company_invoice_settings.rs` (`insert_with_defaults`,
  `:879`) ; `fiscal_years.rs` `create_for_seed` (`:356`) ;
  (e) **reçu E-2 de la revue de la 15-7a1** (Change Log précédent) : déjà au périmètre (AC 1 étape 2),
  rien à ajouter ;
  (f) primitives de la 15-7a2 vérifiées : `lock_state_at_step`, `complete_step`, `conclude_step` et
  `company_select!` sont **privés au handler** (`AppError`) — `kesh-seed` emploie les primitives de
  `kesh-db` que nomme la fiche (`onboarding::lock_state_in_tx`, `update_step_in_tx`,
  `record_step_completed_in_tx`), non les helpers du handler.
- 2026-10-09 — **Développée** (commits `518b5ae8` T0, `a04fde73` code et tests, `638a80ef`
  doc-comments, `97b367c0` manuels et CHANGELOG). Gates au dernier commit de code : backend 2979/2979,
  frontend vert (Vitest 1095/1095), E2E 245 / 9 échecs attendus (7 KF-029, 2 KF-045) ; 5 mutations
  rouges sur 5 jouées. Statut `review`. Choix C-15-7b1-1.
- 2026-10-09 — **Revue de code P1** (Sonnet ×3 : lentilles B, E, A ; rapports
  `target/gate-logs/15-7b1-review-p1-{B,E,A}.md`, non versionnés). **B : 4 LOW ; E : 1 MEDIUM, 3 LOW ;
  A : 1 MEDIUM, 3 LOW** — un seul MEDIUM distinct (E-1 = A-1), convergé par deux lentilles.
  Remédiation, **sans ligne de code de production exécutable** :
  **E-1 = A-1** (MEDIUM, la revérification de l'étape sous verrou et son 400 ne mordaient sur aucun
  test) → tests 12 (appel direct de `kesh_seed::seed_demo` aux étapes 3 et 4) et 13 (course
  `start-production` rendue déterministe par un déclencheur sur `fiscal_years`) ; mutations M6, M7
  rouges. **B-3** → tests 14 (prédicat sur une vraie 1213 `SIGNAL`ée) et 15 (rejeu de bout en bout,
  1213 levée une fois par déclencheur), mutation M8 rouge — l'angle mort F2-3 de la P2 est levé
  (C-15-7b1-2). **B-1 = E-3** → le résidu de la course (société, plan et exercice commités sur une
  installation de production) écrit au doc-comment de `seed_demo`, au CHANGELOG et au manuel
  utilisateur, renvoi à #538. **A-3** → la limite #538 écrite au manuel d'administration (`:1948`),
  PDF régénérés et contrôlés aplatis. **A-2** → commentaires `routes/profile.rs:47-51` (« dix
  appelants tracés », le seed compris) et `fiscal_years.rs::create_for_seed` (contradiction retirée).
  **B-4 = A-4** → garde 11 (b) normalisée (blancs, casse), mutation M9 rouge ; angle mort résiduel écrit.
  **E-4** → angle mort « `ui_mode` hors verrou » retiré (tableau des mutations, Dev Agent Record).
  **E-2** → écrit comme dette (le texte du `422` est du code exécutable ; C-15-7b1-3). **B-2** →
  inchangé, arbitrage de Guy (C-15-7-14). Propagation : grep de `contexte système`, `reste le seed`,
  `15-7b1\)`, `pas provoqué` — le doc-comment (vi) de `audit_route_registry.rs` renvoie désormais aux
  tests 14 et 15. Gate ciblé : fmt, clippy, `onboarding_audit_e2e` 33/33, `kesh-seed` 2/2, `audit_route_registry`
  11/11 ; gate complet et E2E au push. Choix C-15-7b1-2, C-15-7b1-3.
- 2026-10-09 — **Revue de code P2, ciblée** (prompt versionné `15-7b1-review-prompt-p2-ciblee.md` ;
  une lentille Haiku en contexte frais, braquée sur le seul commit de remédiation `4303ac01` ; rapport
  `target/gate-logs/15-7b1-review-p2-ciblee.md`, non versionné). **0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW.**
  Axes exercés : revérification de l'étape sous verrou (tests 12), course HTTP et son 400 (test 13),
  prédicat `is_seed_retryable` (test 14), rejeu de la dernière transaction (test 15), normalisation de la
  garde 11 (b), absence de ligne de production exécutable. Non exercés : exécution (interdite),
  interblocage physique réel (simulé par déclencheur), manuel — **repris à la clôture**, PDF aplatis
  conformes (Dev Agent Record). L'orchestrateur a vérifié que les tests à déclencheur sont des
  `#[sqlx::test]` (base éphémère). La remédiation relue ne touche aucune ligne de code de production :
  **revue CLOSE**. Trend : P1 **1 MEDIUM distinct** (Sonnet ×3 ; E-1 = A-1) → P2 ciblée **0** (Haiku).
- 2026-10-09 — **Clôture** : second rebase sur `f8b2accd` (15-6b, arrivée pendant les premiers gates),
  conflits en union (registre des choix, sprint-status), PDF régénérés et contrôlés aplatis ; partition
  105 / 5 / 2 = 112 recomptée depuis la source ; gates complets sur l'état rebasé : backend 3017/3017,
  Vitest 1118/1118, E2E 244 / 10 échecs tous attendus (7 KF-029, 2 KF-045, KF-052 #424). Statut `done`.
