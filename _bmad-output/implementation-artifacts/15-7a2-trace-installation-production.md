# Story 15.7a2 : La piste de contrôle de l'installation de production

Status: done

<!-- Née le 2026-10-08 du découpage de la 15-7a (choix C-15-7-19), à la passe de validation P2 ; la
     15-7a était elle-même née du découpage de la 15-7 à la P1 (C-15-7-8). Elle garde les routes,
     l'audit, les registres, les libellés et le manuel ; les variantes `_in_tx` et les primitives
     `kesh-db` sont à la 15-7a1.
     Choix applicables de `epic-15-choix-autonomes.md` : C-15-7-2 (`installation.step_completed`),
     C-15-7-3 (plan comptable agrégé), C-15-7-6 (une transaction par route), C-15-7-7 (finalisation),
     C-15-7-12 (ordre des verrous, revérification sous verrou), C-15-7-13 (atomicité prouvée par un
     déclencheur de test), C-15-7-15 (helpers société et helper d'étape), C-15-7-18 (ordre des
     entrées), C-15-7-19 (découpage), C-15-7-20 (emplacement des helpers), C-15-7-21 (`is_stub`),
     C-15-7-25 (même release que la 15-7b — désormais 15-7b1 et 15-7b2, C-15-7-31), C-15-7-30 (`user.created` en tête des séquences montées par le bootstrap). Versions antérieures : `cabda169` (15-7), `3846b206` (15-7a).
     Validation : P1 (sur la 15-7), P2 (sur la 15-7a) et P3 appliquées — CONVERGÉE à la P3 (0 au-dessus de LOW). -->

**Issue** : `refs #434` — **partout, PR comprise**. C'est la **15-7b2**, dernière des sous-stories, qui porte
`closes #434` : cette fiche laisse `seed-demo` et `reset` non tracées.

**Dépendances** : **la 15-7a1, mergée** — elle pose les variantes `_in_tx` (`accounts`,
`bank_accounts`, `company_invoice_settings`, `vat_rates`), `companies::clear_stub_in_tx` et
`onboarding::lock_state_in_tx`. **La 15-7b1, puis la 15-7b2, dépendent de celle-ci** (helper d'étape
`record_step_completed_in_tx`, ordre des verrous des routes de production) et, par elle, de la 15-7a1
(`lock_state_in_tx`) ; elle n'emploie pas `lock_state_at_step`, qui vit dans `kesh-api` (R3-2). ⚠️ **Même release que la 15-7b1 et la 15-7b2** (C-15-7-25, C-15-7-31, finding F-11) : entre les deux, une
remise à zéro d'une installation de production à l'étape ≤ 2 effacerait les entrées neuves de cette
fiche **sans trace** — la 15-7b2 est celle qui inscrit l'effacement. ⚠️ **Conflits attendus au merge**
avec les autres branches de l'Epic 15 : les assertions de décompte de
`crates/kesh-api/tests/audit_route_registry.rs` (`the_registry_partition_is_what_the_story_declares`),
`ACTIONS` de `crates/kesh-api/src/audit_labels.rs`, les quatre `messages.ftl`, l'en-tête `## [0.13.0]`
du `CHANGELOG.md`. Tous se **recomptent depuis la source** au merge — jamais en additionnant des deltas.

## Story

En tant que **fiduciaire ou réviseur qui lit le journal d'audit d'une installation de production**,
je veux que **la séquence d'installation y figure** — qui a choisi la langue et le mode, lancé la
configuration de production, choisi le type d'organisation, chargé le plan comptable, saisi les
coordonnées et le compte bancaire, et finalisé —,
afin que **la piste de contrôle commence au premier geste de configuration et non au premier
exercice**, et qu'une étape ne laisse plus jamais la société modifiée à moitié.

## Décompte de modules — signal D5 déclaré

À la granularité retenue par C-15-7-8 : `kesh-api/routes/onboarding`, `kesh-api/audit_labels`,
`kesh-i18n` (quatre catalogues), `kesh-db/repositories/onboarding` (`record_step_completed_in_tx`) —
quatre modules de code — et des retouches de **commentaires seuls**, sans ligne exécutable, dans
`kesh-db/repositories/{accounts, vat_rates, fiscal_years}` et `kesh-api/routes/{profile, companies}`
(AC 12) — cinq modules. **Neuf modules touchés, dont quatre de code et cinq de commentaires seuls** :
le seuil de cinq est franchi au sens de la règle, qui compte les modules touchés (même méthode que la
15-7a1, F3-5 de la P3). **Le signal est déclaré au Project Lead** ; motif de ne pas découper : les
cinq modules de commentaires n'ont aucune ligne exécutable, la conception est concentrée dans
`routes/onboarding`. S'y ajoutent les deux registres de tests et la documentation.

## Ce que l'inventaire a établi, et qui n'est pas dans l'issue

1. **Les neuf routes de cette fiche sont atteignables par un jeton d'API.** Elles vivent dans
   `authenticated_routes` (`crates/kesh-api/src/lib.rs:839-880`), tout rôle, **sans**
   `require_not_pat`. ⇒ constructeur `NewAuditLogEntry::from_current_user` (ou le helper d'étape,
   qui reçoit l'acteur, AC 6) — **jamais `::user`** (AC 7 de la 25-1b, dette #431). *(`reset`, seule
   des onze dans `admin_routes`, `lib.rs:312`, derrière `require_not_pat`, est à la 15-7b2.)* Seul
   `finalize` extrait aujourd'hui `CurrentUser` (`routes/onboarding.rs:604`) : il faut l'ajouter aux
   **huit** autres handlers.
2. **Cinq des neuf routes ne sont pas atomiques** — `language`, `org-type`, `accounting-language`,
   `coordinates`, `bank-account` enchaînent deux transactions ou plus : la mutation de domaine
   (helpers `update_company_*`, `routes/onboarding.rs:923-1034`, `bulk_create_from_chart`,
   `upsert_primary`) commite seule, puis `onboarding::update_step` commite l'étape. Un conflit de
   version sur l'étape laisse aujourd'hui la société modifiée et l'étape non franchie. `mode`,
   `start-production` et `skip-bank` n'ont qu'une transaction (`update_step` seul) et `finalize` en tient
   une unique — mais aucune des neuf ne revérifie l'étape sous verrou ni n'écrit de trace (R3-1, F3-1).
3. **L'ordre des verrous serait inversé par une fusion naïve.** Réunir « mutation de société puis
   `update_step_in_tx` » verrouille `companies` **avant** `onboarding_state` — l'inverse de
   `finalize` et du Pattern 5 (`docs/MULTI-TENANT-SCOPING-PATTERNS.md:298-302`). Et l'étape est lue
   **hors verrou** (`get_or_init_state`) : deux requêtes concurrentes passent la garde. D'où C-15-7-12.
4. **`set_accounting_language` compte les comptes hors transaction** (`count_by_company(&state.pool,
   …)`, `routes/onboarding.rs:377`) : deux requêtes concurrentes peuvent toutes deux charger le plan.
5. **`company.updated` court-circuite déjà le no-op ailleurs** : `companies::update_in_tx`
   (`repositories/companies.rs:175-203`, KF-004) rend le snapshot `before` sans incrémenter `version`
   quand rien ne change, et `routes/companies.rs:155` n'écrit alors rien. ⚠️ Il **n'écrit pas**
   `is_stub` et ne le compare pas : d'où `clear_stub_in_tx` (15-7a1) et la règle composée de l'AC 3.
6. **Les libellés ne sont pas l'affaire du frontend.** L'écran du journal affiche ce que rend la
   route de vocabulaire (`crates/kesh-api/src/audit_labels.rs`, 25-1c-a) : une action neuve se
   déclare dans `ACTIONS` et se libelle dans les **quatre** `.ftl`, clé
   `audit-log-action-<action, « . » et « _ » → « - »>` (`audit_labels.rs:182`). La garde
   `crates/kesh-api/tests/audit_label_registry.rs` l'impose dans les deux sens : une action déclarée
   sans site d'écriture rougit, un site d'écriture sans déclaration aussi — d'où la répartition des
   actions entre 15-7a2 et 15-7b, et `record_step_completed_in_tx` ici plutôt qu'à la 15-7a1.
7. **Aucun changement de schéma.** `action VARCHAR(64)`, `entity_type VARCHAR(32)`, sans `CHECK` de
   valeur (`20260413000001_audit_log.sql:16-23`) ; aucun déclencheur sur `audit_log` : ni migration,
   ni P1–P8.
8. **Le chemin « aucune société » de `ensure_company_with_language`** (`routes/onboarding.rs:864-883`)
   insère une société stub sans rattacher personne. Il est atteint aujourd'hui après une remise à zéro
   (qui efface `companies`) et sur une base sans société ; la 15-7b2 le rend rare et y **rattache**
   utilisateurs et clés (#528, C-15-7-23). Cette fiche se borne à le tracer.

## Acceptance Criteria

### Volet A — ce que chaque route écrit

**1. Les neuf routes écrivent les entrées de ce tableau, et seulement elles**, dans la transaction
de leur mutation (AC 8). « Étape » désigne l'entrée `installation.step_completed`
(`entity_type = "installation"`, `entity_id = AUDIT_ENTITY_ID_NONE`,
`details = {"from": n, "to": n+1, "step": "<nom>"}`), écrite par **toute** route qui réussit et
franchit une étape (C-15-7-2). La notation `company.updated {champ}` abrège
`{"before": {"champ": …}, "after": {"champ": …}}` ; `company.updated` porte `entity_type = "company"`
et `entity_id` = id de la société verrouillée, comme `routes/companies.rs` (F3-7) — jamais
`AUDIT_ENTITY_ID_NONE`.

| Route (`POST /api/v1/onboarding/…`) | `step` | Entrées de domaine (avant l'étape, AC 2) |
|---|---|---|
| `language` | `language` (0→1) | `company.created` si la société est créée (`entity_id` = id inséré, `details = {"instance_language", "is_stub": true}`) ; sinon `company.updated {instance_language}` **si** la langue change |
| `mode` | `mode` (1→2) | `installation.ui_mode_changed` `{before, after}` — même action et même forme que `routes/profile.rs:66-79` — **si** le mode change (⚠️ `profile.rs:51-53` trace, lui, même sans changement : divergence assumée, Dev Notes) |
| `start-production` | `start_production` (2→3) | aucune — l'étape **est** le fait : l'installation devient non réinitialisable, **même avec** `KESH_PRODUCTION_RESET` (`OnboardingResetForbidden`, `routes/onboarding.rs:268-272` ; le drapeau ne gouverne que la démonstration aux étapes 3 à 6) |
| `org-type` | `org_type` (3→4) | `company.updated {org_type}` **si** il change |
| `accounting-language` | `accounting_language` (4→5) | `company.updated {accounting_language}` **si** elle change ; puis `account.chart_loaded` (AC 4) **si** le plan est chargé (garde `existing == 0`, dans la transaction) |
| `coordinates` | `coordinates` (5→6) | `company.updated` portant **les seuls champs que la route écrit** — `name`, `first_name`, `last_name`, `address_street`, `address_building`, `address_postal_code`, `address_city`, `address_country`, `ide_number`, `is_stub` — plus `version`, en `before`/`after`, selon la règle composée de l'AC 3 (la colonne combinée `address`, dérivée des cinq, n'y figure pas) |
| `bank-account` | `bank_account` (6→7) | `bank_account.created` (forme de `routes/bank_accounts.rs:470-477`) **ou** `bank_account.updated` (forme de `:572-590`, avec `"trigger": "onboarding"` **et** `"iban_changed"`, `"qr_iban_changed"` — booléens, sans secret, F-10) **ou** rien si inchangé — selon `UpsertPrimaryOutcome` (15-7a1) |
| `skip-bank` | `skip_bank` (6→7) | aucune |
| `finalize` | `finalize` (7→8) | `company_invoice_settings.created` **si** la ligne est insérée (AC 5) ; `vat_rate.created` **par taux réellement inséré** (forme de `routes/vat.rs:344-350`) ; `fiscal_year.created` — **déjà** écrit par `fiscal_years::create_if_absent_in_tx`, inchangé (AC 7) |

⚠️ **Le retour idempotent de `finalize` (`step_completed == 8`, `:676-679`) n'écrit rien** : il ne
mute rien. Toute route **refusée** (`OnboardingStepAlreadyCompleted`, `Validation`,
`OptimisticLockConflict`, erreur interne) n'écrit rien et ne laisse aucune mutation (AC 8).
`seed-demo` et `reset` restent hors de cette fiche (15-7b1, 15-7b2).

**2. L'ordre des entrées est fixé** (C-15-7-18) : au sein d'une route, les entrées de domaine dans
l'**ordre d'exécution** des mutations, **`installation.step_completed` en dernier**. Pour `finalize` :
`company_invoice_settings.created`, puis `vat_rate.created` dans l'ordre des catégories du seed
(`normal`, `special`, `reduced`, `exempt` — l'ordre de rendu de la variante, 15-7a1 AC 5), puis
`fiscal_year.created`, puis l'étape. Les tests assertent cette séquence.

**3. Une modification sans changement n'écrit pas de trace de domaine — règle composée pour la
société** (C-15-7-15, C-15-7-21). Les helpers société passent par `companies::update_in_tx`
(court-circuit no-op KF-004), avec un `CompanyUpdate` reconstruit depuis la ligne **verrouillée** dont
seuls les champs visés changent, et la `version` **verrouillée**. Pour `coordinates`, puis
`companies::clear_stub_in_tx` (15-7a1 : `+1` sur `version` ssi le drapeau était levé). La société est
**relue dans la transaction après les deux écritures**, et c'est cette relecture qui fournit `after`.
`company.updated` s'écrit **si et seulement si** (`version` rendue par `update_in_tx` ≠ `version`
verrouillée) **ou** (`clear_stub_in_tx` a rendu `true`). Conséquences, à asserter : coordonnées
changées sur société stub ⇒ `version + 2`, une seule entrée ; coordonnées **identiques** sur société
**stub** ⇒ `version + 1`, une entrée dont `before`/`after` ne diffèrent que par `is_stub` et `version` ;
coordonnées identiques sur société non stub ⇒ `version` inchangée, aucune entrée de domaine. Même
règle de no-op pour `bank-account` (`Unchanged` ⇒ aucune entrée de domaine) et `mode`. L'**étape**,
elle, s'écrit toujours : elle a été franchie.

**4. Le plan comptable s'inscrit en une entrée agrégée** (C-15-7-3) : action `account.chart_loaded`,
`entity_type = "account"`, `entity_id = AUDIT_ENTITY_ID_NONE`, `details = {"company_id", "org_type",
"language", "count", "accounts": [{"id", "number"}, …]}` — `count` égal au nombre de comptes
**réellement insérés** (le `Vec` rendu par `accounts::bulk_create_from_chart_in_tx`, 15-7a1), à la
longueur de `accounts`, et à `SELECT COUNT(*) FROM accounts WHERE company_id = ?` au commit.

**5. Finalisation — ce qui a été inséré, pas ce qui a été tenté** (C-15-7-7, C-15-7-17). `finalize`
emploie le booléen de `insert_with_defaults_in_tx` et le `Vec<VatRate>` de
`seed_default_swiss_rates_in_tx` (15-7a1, AC 4 et 5). `company_invoice_settings.created` :
`entity_type = "company_invoice_settings"`, `entity_id = company_id` (la table est clée par société),
`details = {"company_id", "invoice_number_format", "default_receivable_account_id",
"default_revenue_account_id", "default_payable_account_id", "default_rounding_account_id",
"default_discount_account_id", "default_bank_fees_account_id", "default_bad_debt_account_id"}`.

### Volet B — les invariants

**6. Le constructeur dit la vérité sur l'acteur.** Les entrées de domaine des handlers se
construisent par `from_current_user`. L'entrée d'étape passe par **un helper unique** (C-15-7-15,
C-15-7-20), logé dans `kesh-db` pour servir aussi `kesh-seed` (15-7b1, 15-7b2) :
`kesh_db::repositories::onboarding::record_step_completed_in_tx(tx, user_id, api_key_id, from, to,
step: &'static str)`, qui écrit `NewAuditLogEntry::for_actor(user_id, api_key_id,
"installation.step_completed", "installation", AUDIT_ENTITY_ID_NONE, …)` — action et type **en
littéral** dans le helper (forme reconnue par `FORMES`, `audit_label_registry.rs:43-51` ; pas de
`SITES_INDIRECTS`). Il ne commite pas. Aucun `NewAuditLogEntry::user(` neuf dans
`routes/onboarding.rs` ni `repositories/onboarding.rs` (garde de source, test 12). Un test prouve
qu'une étape franchie **par jeton d'API** porte `actor_type = 'api_key'` et `actor_api_key_id`
(patron `tests/api_keys_e2e.rs:445-509`, `create_key_via_http`) — sur `org-type` (entrée de domaine
**et** étape) et sur `skip-bank` (étape seule).

**7. `fiscal_year.created` n'est pas dupliqué** : `finalize` n'écrit pas sa propre entrée d'exercice.
Son attribution par `::user` (`repositories/fiscal_years.rs:126-133`) reste la dette #431 — l'écrire
dans les Dev Notes du Dev Agent Record, ne pas la corriger ici. ⚠️ Le test par jeton d'API (AC 6)
ne passe **pas** par `finalize` pour cette raison : il y trouverait `actor_type = 'user'`.

**8. La trace et la mutation sont atomiques, et les verrous suivent le Pattern 5** (C-15-7-6,
C-15-7-12). Chaque route mène **une** transaction, dans cet ordre :
  1. hors transaction, inchangé : validation du corps **avant** toute lecture d'état (codes et ordre
     des erreurs préservés) ; `get_or_init_state` pour garantir l'existence de la ligne ;
  2. `BEGIN` ; `lock_state_at_step(&mut tx, expected, require_not_demo) -> Result<OnboardingState,
     AppError>` (`kesh-api`, neuf), bâti sur `onboarding::lock_state_in_tx` (15-7a1) : verrouille
     `onboarding_state` **en premier**, **revérifie sous verrou** l'étape attendue (et `!is_demo` là où
     la garde existe), rend l'état verrouillé, `AppError::OnboardingStepAlreadyCompleted` si l'étape
     diffère, `AppError::Internal` si la ligne manque. ⚠️ Le helper reçoit `tx` **par référence** : il
     **ne peut pas** annuler (`Transaction::rollback` consomme la transaction) — il rend l'erreur, et
     **l'appelant** annule par `best_effort_rollback(tx)` (`routes/onboarding.rs:30`) ou par le `Drop`
     (R2-7). La lecture non verrouillée de la garde disparaît : pour une requête **séquentielle** hors
d'étape, le code rendu est le même ; pour le **perdant d'une course** (deux requêtes concurrentes à la
même étape), il passe de **409** `OPTIMISTIC_LOCK_CONFLICT` (aujourd'hui : `update_step` à version lue
hors verrou, `repositories/onboarding.rs:117-119`) à **400** `ONBOARDING_STEP_ALREADY_COMPLETED` —
changement consigné (Points de vigilance, R3-8/F3-8). Les
     trois copies du `SELECT … FOR UPDATE` de `routes/onboarding.rs` (`:250` — que la 15-7b2 remplace —,
     `:653`, `:806`) passent par `lock_state_in_tx` pour les deux que touche cette fiche ;
  3. `companies` (`COMPANY_SELECT_FOR_UPDATE`) si la route touche la société — `language`,
     `org-type`, `accounting-language`, `coordinates`, `bank-account` (qui remplace `get_company`,
     non verrouillé) ;
  4. `accounts` (`accounting-language` : `accounts::count_by_company(&mut *tx, …)` — générique depuis
     la 15-7a1 — **dans** la transaction, puis `bulk_create_from_chart_in_tx` si `existing == 0`) ;
     `bank_accounts` (`bank-account` : `upsert_primary_in_tx`) ;
  5. `update_step_in_tx(tx, n+1, …, version_verrouillée)` ; entrées d'audit (AC 2) ; `COMMIT` ;
     `response_with_stub` **après** commit.
`finalize` garde sa transaction, son ordre `onboarding_state → companies → accounts → settings →
fiscal_years` et son `retry_with` ; ses entrées s'écrivent **dans la closure** `finalize_inner`
(rejouée par `retry_with` : l'audit d'un essai annulé disparaît avec lui). Les variantes `_in_tx`
employées n'annulent rien elles-mêmes (15-7a1) : sur erreur, c'est le handler qui annule.

**9. Aucun secret dans `details_json`, clés en snake_case** : l'IBAN et le QR-IBAN n'y figurent que
par `iban_present` / `qr_iban_present` / `iban_changed` / `qr_iban_changed` ; le numéro IDE, public au
registre du commerce, y figure.

### Volet C — que les registres restent vrais

**10. Le registre des routes passe les neuf routes de `Exempt("issue #434 …")` à `Traced`**
(`crates/kesh-api/tests/audit_route_registry.rs:162-171`, sauf `seed_demo` `:164` et `reset` `:76`,
qui restent `Exempt` jusqu'à la 15-7b1 et la 15-7b2, **avec un motif réécrit** : « issue #434 — peuplement de
démonstration (15-7b1) et remise à zéro (15-7b2) », F-9), et `the_registry_partition_is_what_the_story_declares`
est **recompté depuis la source** : sur la base de cette branche, `traced` 95 → **104**, `exempt`
15 → **6** (deux de #434, quatre de #435), `no_matter` **2**, total **112** inchangé. *(Revue P1,
A-3 : la fiche disait 94 → 103 et `no_matter` 3, valeurs de sa base de rédaction ; le T0 les a
recomptées — 95/15/2 → 104/6/2 —, et c'est la valeur livrée.)* Messages des
assertions mis à jour (« 11 routes d'onboarding (#434) » `:478` → « 2 routes d'onboarding (#434,
15-7b1, 15-7b2) ») ; le message de l'assertion `traced` (`:470`, « 73 tracées avant la 25-1b, plus ses 14, … »)
énumère les contributions au total et reçoit « plus les neuf routes de configuration de l'installation
(15-7a2, #434) » ; le doc-comment de tête (`:20`, `:24-25`, « 111 routes », « 114 », déjà périmés) est
**recompté** depuis l'inventaire (R3-6). ⚠️ *Ces nombres valent pour cette branche ; au merge, recompter.*

**11. Quatre actions neuves, déclarées et libellées dans les quatre catalogues** — `ACTIONS` de
`crates/kesh-api/src/audit_labels.rs` (**triée**, test `les_trois_listes_sont_triees_et_sans_doublon`)
et `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`, à côté des clés voisines (fr :
`:2204-2208`, `:2240-2241`, `:2292-2295`) :

| Action | Clé | fr-CH | de-CH | it-CH | en-CH |
|---|---|---|---|---|---|
| `account.chart_loaded` | `audit-log-action-account-chart-loaded` | Plan comptable chargé | Kontenplan geladen | Piano dei conti caricato | Chart of accounts loaded |
| `company.created` | `audit-log-action-company-created` | Société créée | Unternehmen erstellt | Società creata | Company created |
| `company_invoice_settings.created` | `audit-log-action-company-invoice-settings-created` | Réglages de facturation créés | Fakturierungseinstellungen erstellt | Impostazioni di fatturazione create | Invoicing settings created |
| `installation.step_completed` | `audit-log-action-installation-step-completed` | Étape d'installation franchie | Installationsschritt abgeschlossen | Passo d'installazione completato | Installation step completed |

Aucun type d'entité neuf (`account`, `company`, `company_invoice_settings`, `installation`,
`bank_account`, `vat_rate` existent). Les actions s'écrivent **en littéral** au site d'appel ; un
site dont l'action est une variable entre dans `SITES_INDIRECTS` et fait rougir
`l_inventaire_compte_ce_que_le_fichier_annonce` — l'éviter (le choix créé/modifié de `bank-account`
et de `language` s'écrit donc en deux appels littéraux, pas en une variable).

**12. Les doc-comments que la story rend faux disent le nouvel état.** Grep du symptôme, à exécuter
et non à supposer, sortie collée au Dev Agent Record :
`grep -rnE "#434|TODO\(L65|Pas d'audit log|ne génère PAS|PAS d'entrées d'audit|sans audit log|contexte système|dix appelants|Dix appelants|appelants la partagent|company only|is_stub = FALSE|inconditionnel" crates docs/MULTI-TENANT-SCOPING-PATTERNS.md`.
Le motif rend aussi des occurrences légitimes, à trier à la main ; la liste ci-dessous est celle de
cette passe, et un site du grep qu'elle omet se traite quand même (R2-5). Sites **attribués à cette
fiche** (l'attribution complète des trois fiches est dans l'index `15-7-trace-onboarding.md`) :
- `crates/kesh-api/src/routes/onboarding.rs:523-527` — le `TODO(L65 Story 8-5a-zero) : backfill
  audit_log bank_account.created`, que la story accomplit : **à supprimer** ;
- `crates/kesh-api/src/routes/onboarding.rs:999-1003` (« `is_stub = FALSE` inconditionnel ») — dire
  `clear_stub_in_tx` et la règle de l'AC 3 ;
- `crates/kesh-api/src/routes/companies.rs:233` (« `update_company_coordinates` (onboarding) pose
  `is_stub = FALSE` inconditionnellement ») — relire après la réécriture du helper ;
- `crates/kesh-db/src/repositories/accounts.rs:8` et `:932-934` — « ne génère PAS d'entrées d'audit
  log (contexte seed système) » : dire que la fonction n'écrit pas elle-même, et que l'appelant
  onboarding écrit `account.chart_loaded` ;
- `crates/kesh-db/src/repositories/vat_rates.rs:284` — « Pas d'audit log : seed = contexte système » :
  idem, l'appelant `finalize` écrit `vat_rate.created` ;
- `crates/kesh-db/src/repositories/fiscal_years.rs:200-203` — « cohérent avec la décision story 3.5
  sur `bulk_create_from_chart` : le contexte système (seed) ne génère pas d'entrée d'audit » : la
  référence à `bulk_create_from_chart` devient fausse quand l'onboarding trace le plan ; la règle reste
  vraie pour `create_for_seed` (R2-11) ;
- `crates/kesh-db/src/repositories/onboarding.rs:76-83` et `crates/kesh-api/src/routes/profile.rs:47-49`
  (« dix appelants », coupé sur deux lignes dans `profile.rs`) : « neuf appelants tracés (huit routes
  et `/profile/mode`) ; reste le seed de démonstration (15-7b1) » (R2-4) ;
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md:312` — la ligne « `/coordinates`, `/org-type`,
  `/accounting-language` | company only » devient fausse : une ligne par famille de route avec sa
  séquence `onboarding_state → companies → …` ; ajouter `bank_accounts` à la liste ordonnée
  (`:298-302`) **après** `accounts` et **avant** `company_invoice_settings` (donnée de la société avant
  ses réglages, selon la logique écrite sous la liste ; aucune route ne prend les deux), après avoir vérifié par `grep -rn "FOR UPDATE" crates/kesh-db/src/repositories/bank_accounts.rs`
  qu'aucun chemin ne prend `bank_accounts` avant `companies`.
Sites légitimes à laisser : `auth/bootstrap.rs:128` (historique : le bootstrap n'est pas une route,
il échappait à #434 et le reste) ; `routes/onboarding.rs:597`, `:608` (`retry_with` de `finalize`).

### Volet D — ce que lisent l'utilisateur et le réviseur

**13. Les manuels disent ce que l'inventaire établit**, PDF régénérés (`make fr` dans
`docs/manual/`) et contrôlés aplatis (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) :

| Site | Aujourd'hui | Après la 15-7a2 |
|---|---|---|
| `admin-manual.tex:1821` | « 87 des 105 routes … la séquence d'installation (issue \#434) et les gestes de session (\#435) » | décompte **recompté depuis le registre** (AC 10) : « 104 des 112 routes » ; exceptions, **comptées** : six routes exemptées — deux de \#434 (le peuplement de démonstration et la remise à zéro) et **quatre** de \#435 (connexion, déconnexion, **renouvellement de session**, changement de son propre mot de passe : le texte actuel n'en nomme que trois, F3-6) —, et les deux routes « sans matière » — de sorte que 104 + 6 + 2 = 112 se lise (R12 ; valeurs recomptées au T0, revue P1 A-3) |
| `admin-manual.tex:2004` | « Ce qui manque est la séquence d'installation (\#434) et les gestes de session (\#435) » | « le peuplement de démonstration et la remise à zéro (\#434), et les gestes de session (\#435) » |
| `admin-manual.tex:2244` (glossaire) | renvoie aux réserves | relire : ne doit pas contredire les deux lignes ci-dessus |
| `user-manual.tex:2013-2016` | énumération « …exports, gestion des utilisateurs et modification de la société » | ajouter « et la configuration initiale de l'installation » |
| `user-manual.tex:2019-2023` | « Deux familles … la séquence d'installation — création du plan comptable, du compte bancaire, peuplement de démonstration — et les gestes de session » | « Deux familles … le **peuplement de démonstration** et sa réinitialisation, et les gestes de session » |
| `user-manual.tex`, § *Consulter le journal d'audit* | — | un paragraphe : les entrées de configuration, une par étape franchie — les deux premières (langue, mode) **communes aux deux chemins**, production et démonstration (F-12) ; le plan comptable chargé **en une seule entrée** — filtrer sur le type *Compte* et un numéro d'entité ne retrouve pas la création d'un compte du plan livré (C-15-7-3, F-8 : l'écran filtre par type et **numéro d'entité**, pas par numéro de compte) ; la création de la société de départ par le premier démarrage n'est pas inscrite, seule la configuration qui suit l'est |

⚠️ **Greper la valeur**, après modification : `\b434\b`, `\b87\b`, `\b105\b`, `\b112\b`,
`séquence d.installation` (le PDF porte l'apostrophe typographique : un grep à apostrophe droite rend
un faux négatif), « installation » près de « journal d'audit » sur `docs/manual/fr/*.tex`, le PDF
aplati, `README.md`, `website/`. La ligne v0.12.1 du README (« Restent ouverts… [#434] ») est
**historique** : ne pas la réécrire. `docs/api-external.md` n'a rien à changer (consultation du
journal fermée aux clés, `:81`).

**14. CHANGELOG** : sous `## [0.13.0] — Non publié` (à créer en tête, après le `---` d'introduction,
si absente — motif lu par `scripts/prepare-release.sh:189`), `### Corrigé` : la configuration d'une
installation de production s'inscrit au journal d'audit (refs #434) — les quatre actions, l'entrée
agrégée du plan comptable, la correction d'atomicité (une étape ne laisse plus la société modifiée à
moitié) et la fin de la double prise du plan comptable sous requêtes concurrentes.

## Tasks / Subtasks

- [x] **T1 — Helpers** (AC 6, 8) — `onboarding::record_step_completed_in_tx` (`kesh-db`) ; `lock_state_at_step` (`kesh-api`) sur `lock_state_in_tx`.
- [x] **T2 — Handlers de progression** (AC 1, 2, 3, 6, 8, 9) — `Extension(current_user)` sur les huit qui ne l'ont pas ; une transaction chacun ; helpers société en `_in_tx` sur `companies::update_in_tx` et `clear_stub_in_tx`, relecture après écriture ; `ensure_company_with_language` rend créé{id} / modifié{before} / inchangé.
- [x] **T3 — `finalize`** (AC 1, 2, 5, 7) — entrées dans `finalize_inner`.
- [x] **T4 — Libellés et registres** (AC 10, 11) — `ACTIONS`, quatre `.ftl`, registre, motifs des deux exemptions restantes, partition recomptée.
- [x] **T5 — Doc-comments et Pattern 5** (AC 12) — grep exécuté, sortie collée au Dev Agent Record.
- [x] **T6 — Tests** (Dev Notes § Tests).
- [x] **T7 — Manuels, PDF, CHANGELOG** (AC 13, 14).
- [x] **T8 — Gates** : `kesh-db` touché ⇒ **gate complet même en cours de boucle** ; base de gate remise à zéro avant ; E2E complet au dernier commit de code (comparer aux échecs attendus de `docs/testing.md`).

## Dev Notes

### Le patron d'appel — relevé au sol

- **Seule porte d'écriture** : `kesh_db::repositories::audit_log::insert_in_tx`
  (`repositories/audit_log.rs:62`), qui ne commite jamais ; `actor_label` et `company_id` sont posés
  par sous-SELECT sur `users` (`:91-92`) — **rien à fournir**.
- **Constructeurs** : `from_current_user` (`crates/kesh-api/src/audit.rs`, trait `:24`, impl `:34`),
  `for_actor` (`entities/audit_log.rs:194`). Le critère est « un jeton peut-il atteindre ce chemin » —
  ici, oui pour les neuf routes.
- **Précédent le plus proche** : `routes/profile.rs:27-85` — `update_step_in_tx` + audit
  `installation.*` dans le handler, `entity_id = AUDIT_ENTITY_ID_NONE`. ⚠️ Son commentaire dit
  pourquoi l'audit **ne va pas** dans `update_step_in_tx` : la trace y mentirait. Même règle ici : le
  helper d'étape est **distinct** d'`update_step_in_tx`, appelé explicitement.
- **`entity_type = "installation"`** : `onboarding_state` est mono-ligne, globale, sans
  `company_id` ; l'étape porte sur l'installation, pas sur la société (25-1b AC 5/9).

### État des fichiers modifiés

- `crates/kesh-api/src/routes/onboarding.rs` (1034 l.) — neuf handlers ; helpers société `:923-1034`
  (une transaction chacun, `UPDATE` inconditionnel) ; `ensure_company_with_language` `:846-907` ;
  `get_company` `:916` (non verrouillé — ne sert plus à `bank-account`). **À préserver** : codes
  d'erreur et ordre des contrôles, `retry_with` de `finalize`, `response_with_stub` **après** commit.
- `crates/kesh-db/src/repositories/onboarding.rs` — `record_step_completed_in_tx`.
- `crates/kesh-api/src/audit_labels.rs`, `crates/kesh-api/tests/audit_route_registry.rs`,
  `crates/kesh-i18n/locales/*/messages.ftl` — cf. AC 10, 11.
- Commentaires seuls : cf. AC 12. `docs/MULTI-TENANT-SCOPING-PATTERNS.md` — AC 12.

### Points de vigilance

- **Le court-circuit no-op change le comportement de `version`** sur `companies` quand rien ne
  change, et `coordinates` sur société stub l'incrémente **deux fois** quand les coordonnées changent
  (AC 3) : sans conséquence (la version n'est lue que par le verrou optimiste du même flux et par le
  frontend après relecture), mais à dire dans le Change Log.
- **`mode` : deux écrivains d'une même action, deux sémantiques du no-op** (R2-10). L'onboarding
  n'écrit `installation.ui_mode_changed` que si le mode change ; `routes/profile.rs:51-53` l'écrit
  même sans changement (« Assumé »). Le dire au Dev Agent Record ; aligner `profile.rs` est hors
  périmètre — à signaler à l'orchestrateur, pas à corriger ici.
- **`companies::update_in_tx` préserve `address`** quand `combined()` est vide
  (`repositories/companies.rs:~205-230`) ; `coordinates` exige une adresse
  (`validate_required`), donc sans effet ici — à ne pas « corriger ».
- **Tests existants qui comptent des entrées d'audit après un onboarding HTTP** :
  `crates/kesh-api/tests/fiscal_years_e2e.rs` (seul fichier qui combine `onboarding/` et
  `audit_log` ; commentaire `:1299` « seed contexte système » à relire) — les filtres par
  `entity_type = 'fiscal_year'` restent vrais, un `COUNT(*)` global non. `onboarding_e2e.rs`,
  `onboarding_path_b_e2e.rs` : relire les assertions d'état et de `version`. Le test 8 de la 15-7a1
  a ajouté à `onboarding_path_b_e2e.rs::full_path_b_flow` un appel à `finalize` et l'assertion
  « `audit_log` = `["user.created", "fiscal_year.created"]` exactement » (C-15-7-30 : la première entrée
  est écrite par le bootstrap, `ensure_admin_user`, `auth/bootstrap.rs:148`) : elle **doit** changer
  ici — la remplacer par `user.created` **suivi de** la séquence de l'AC 2 pour ce parcours (les
  assertions d'état — quatre taux, une ligne de réglages — restent). ⚠️ Ce parcours part de
  `create_test_company` (société **non** stub) : la `company.updated` des coordonnées n'y porte pas la
  levée du stub. Côté Playwright, `frontend/tests/e2e/audit-log.spec.ts` (filtre par
  `entity_type`/`entityId`) et les specs d'onboarding : l'E2E complet tranche.
- **Le perdant d'une course change de code** (R3-8, F3-8) : 409 `OPTIMISTIC_LOCK_CONFLICT` → 400
  `ONBOARDING_STEP_ALREADY_COMPLETED` (AC 8.2). Sans effet visible : le frontend ne lit ce code que pour
  `finalize` (`routes/onboarding/+page.svelte:133`) et affiche un message générique ailleurs. À écrire au
  Change Log de l'implémentation, comme la variation de `version`.
- **Deux casses dans un même type d'entité** (R3-9) : les `details` de `company_invoice_settings.created`
  (AC 5) sont en snake_case, convention des entrées neuves ; ceux de `company_invoice_settings.updated`,
  écrits par `settings_snapshot_json` (`repositories/company_invoice_settings.rs:37-57`), sont en
  camelCase — héritage. Ne « corriger » ni l'un vers l'autre ; le dire au Dev Agent Record.
- **Frontend** : aucun consommateur des actions ; `ONBOARDING_STEP_ALREADY_COMPLETED`, seul code lu
  (`routes/onboarding/+page.svelte:133`), est inchangé.

### Tests

Nouveau fichier `crates/kesh-api/tests/onboarding_audit_e2e.rs`,
`#[sqlx::test(migrations = "../kesh-db/test-schema")]`. **Lecture d'`audit_log` par les helpers
existants** de `crates/kesh-api/tests/common/mod.rs` (`audit_actions` `:56`, `audit_actor` `:72`,
`audit_details` `:94`, `audit_count` `:115` — ils portent déjà le piège `details_json` en
`Option<Vec<u8>>`), **plus un helper neuf `audit_sequence(pool) -> Vec<String>`** (`SELECT action FROM
audit_log ORDER BY id`) ajouté à `common/mod.rs` et **partagé avec la 15-7b1 et la 15-7b2** (F3-4, DRY) — pas de copie
du patron de `reports_e2e.rs`.
⛔ Asserter la **séquence exacte** des `action` (`ORDER BY id`) et le **contenu** des détails, pas un
nombre — un compte resterait vert sur une action remplacée par une autre. Le DDL de test (déclencheur)
passe par **`sqlx::raw_sql`** (protocole texte, précédent `kesh-db/tests/audit_log_company_id_backfill.rs:197`) :
`sqlx::query("CREATE TRIGGER …")` prend le protocole préparé, que rien n'a vérifié pour `CREATE
TRIGGER … SIGNAL` (R-11 de la P2 de la 15-7b).

| # | Test | AC |
|---|---|---|
| 1 | Parcours production complet — **valeurs fixées** (R3-7) : société stub du bootstrap — montage par `ensure_admin_user` sur base **vide** (cas `(0, true)` : stub, admin, et entrée `user.created`, `auth/bootstrap.rs:148`), **sans** `create_test_company` — (`org_type = Independant`, `accounting_language = Fr`, `instance_language = Fr`, `auth/bootstrap.rs:347-360`) ; requêtes `language` = allemand, `mode` = `guided`, `org-type` = `Association`, `accounting-language` = allemand — chaque route change donc sa valeur et écrit son `company.updated` — ; `language` → `mode` → `start-production` → `org-type` → `accounting-language` → `coordinates` → `bank-account` → `finalize` : séquence exacte = `user.created` (bootstrap, avant toute route) puis celle de l'AC 2 (correction de fait, C-15-7-30), `step` de chaque étape, `installation.ui_mode_changed`, `company.updated` des coordonnées avec ses seuls champs (société stub ⇒ `version + 2`), `account.chart_loaded.count` = `COUNT(*)` des comptes, `bank_account.created` avec `iban_present` et **sans** IBAN en clair (recherche de la chaîne IBAN dans tout `details_json` : 0), `vat_rate.created` ×4 dans l'ordre, `company_invoice_settings.created`, un seul `fiscal_year.created` | 1, 2, 3, 4, 5, 9 |
| 2 | Même parcours avec `skip-bank` : étape seule, aucune entrée `bank_account.*` | 1 |
| 3 | `finalize` rejoué à l'étape 8 : **aucune** entrée nouvelle | 1 |
| 4 | `finalize` sur société **déjà dotée** de réglages et de **deux taux qui heurtent la clé unique `(company_id, rate, valid_from)` des défauts** (8.10 et 2.60 au 2024-01-01, insérés au montage — sinon `INSERT IGNORE` insérerait les quatre, R2-8) : pas de `company_invoice_settings.created`, `vat_rate.created` ×2 seulement (`special`, `exempt`) | 5 |
| 5 | No-op, une route par helper : `language` avec la langue courante ; `mode` avec le mode déjà posé au montage (`UPDATE onboarding_state SET ui_mode = 'guided'` à l'étape 1) ; `org-type` égal ; `accounting-language` égale **avec le plan préchargé au montage** (`accounts::bulk_create_from_chart`, sinon `existing == 0` charge le plan et écrit `account.chart_loaded`, F-4) ; `coordinates` identiques à une société **non stub** ⇒ étape écrite, **aucune** entrée de domaine, `companies.version` inchangée. Puis `coordinates` identiques à une société **stub** (montage `is_stub = TRUE`) ⇒ **une** `company.updated` dont `before`/`after` ne diffèrent que par `is_stub` (`true → false`) et `version`, `companies.version + 1` (AC 3) | 3 |
| 6 | `bank-account` : création, puis — étape remise à 6 au montage — modification de l'IBAN (`bank_account.updated`, `"trigger": "onboarding"`, `iban_changed = true`, `qr_iban_changed = false`, `before` complet), puis données identiques (aucune entrée de domaine) | 1, 3, 8, 9 |
| 7 | `language` sans société en base (montage : aucune société, comme le preset `fresh` ; si `users.company_id` l'exige, utilisateur créé sous `FOREIGN_KEY_CHECKS=0` sur une connexion dédiée) : `company.created` avec `entity_id` = id inséré ; **`audit_log.company_id` asserté explicitement** — il vaut l'id que désigne `users.company_id` (sous-SELECT, `repositories/audit_log.rs:92`), donc l'id **mort** du montage et non `entity_id` : c'est #528, que la 15-7b2 ferme en rattachant les utilisateurs (son test 6c remplace cette assertion) (F-7) | 1 |
| 8 | Jeton d'API sur `org-type` (montage : type d'organisation **`Association`** demandé, la société stub étant `Independant` — sinon aucune entrée de domaine, R2-8) et sur `skip-bank` : `actor_type = 'api_key'`, `actor_api_key_id` renseigné, sur **chaque** entrée | 6 |
| 9 | **Atomicité** (C-15-7-13) — **un montage par route**, l'échec prouvé venir du déclencheur. Déclencheur posé par `sqlx::raw_sql` : `CREATE TRIGGER t_15_7a_fail BEFORE INSERT ON audit_log FOR EACH ROW SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = '15-7a atomicity'`. **(a)** étape posée à 4 par SQL, aucun compte : `accounting-language` avec une langue **différente** de celle de la société (allemand ; la souche est `Fr` — sinon la moitié « société » de l'assertion serait vide, F3-2) ⇒ **500**, `accounts` vide, société (`accounting_language`, `version`), `onboarding_state` (`step_completed`, `version`) et `audit_log` inchangés. **(b)** étape posée à 5 par SQL : `coordinates` (nom changé) ⇒ **500**, `name`, `version`, `is_stub`, `onboarding_state` et `audit_log` inchangés. Preuve d'origine : avant chaque appel, le déclencheur figure dans `information_schema.TRIGGERS` ; puis `DROP TRIGGER`, **le même montage** est rejoué et **la même requête** rend 200 — seul le déclencheur diffère entre l'échec et le succès (R2-1, F-3). ⚠️ Si l'utilisateur de test n'a pas le privilège `TRIGGER` (erreur 1142), le test **échoue** — il ne se saute pas (dev : `ALL PRIVILEGES ON \_sqlx\_test%.*`, `scripts/mariadb-init/01-dev-grants.sql:38` ; CI : `root`) | 8 |
| 10 | Étape refusée sous verrou : `org-type` alors que l'étape vaut 4 ⇒ 400 `ONBOARDING_STEP_ALREADY_COMPLETED`, aucune mutation ni entrée. Vert aussi sur le code d'avant (non-régression du code rendu) ; il **mord** depuis que la lecture non verrouillée disparaît : sans la comparaison de `lock_state_at_step`, `update_step_in_tx` à la version verrouillée réussit et la route rend 200 (F3-3) | 1, 8 |
| 11 | `accounting-language` appelée deux fois en parallèle (deux `tokio::spawn`) à l'étape 4, version `v` relevée avant : statuts **exactement** `{200, 400}`, le 400 portant `ONBOARDING_STEP_ALREADY_COMPLETED` (la garde d'étape sous verrou passe avant `existing == 0`) ; **un seul** `account.chart_loaded` ; **exactement une** `installation.step_completed` de `step = "accounting_language"` ; `onboarding_state.version = v + 1` ; `COUNT(*)` des comptes égal au plan. L'issue ne dépend pas de l'entrelacement : la seconde requête trouve toujours l'étape 5 (F-2) | 4, 8 |
| 12 | Garde de source : `NewAuditLogEntry::user(` absent de `routes/onboarding.rs` et `repositories/onboarding.rs` (lecture `include_str!`, patron `admin_pat_denied_e2e`) | 6 |
| 13 | **Pool d'une connexion** (F3-9) : `MySqlPoolOptions::new().max_connections(1).acquire_timeout(2 s)` sur les options du pool `#[sqlx::test]`, application montée sur ce pool ; `org-type`, `accounting-language`, `coordinates`, `bank-account` déroulés depuis les étapes voulues ⇒ 200 chacune. Un prélèvement du pool laissé **dans** la transaction (`get_company(&state)`, `count_by_company(&state.pool, …)`) attend une connexion que tient la transaction et expire | 8 |

⚠️ **Prouver que les tests mordent** — chaque mutation appliquée seule, puis le fichier **touché**
(cargo garde sinon le binaire muté) : retirer l'appel au helper d'étape ⇒ test 1 rouge ; remplacer le
`commit` unique de `coordinates` par deux commits ⇒ test 9 (b) rouge ; **commiter la société avant le
chargement du plan** dans `accounting-language` ⇒ test 9 (a) rouge (F3-2) ; **retirer la comparaison
d'étape de `lock_state_at_step`** ⇒ tests 10 et 11 rouges (F-2, F3-3) ; retirer la condition « stub
levé » de l'AC 3 ⇒ test 5 rouge ; remettre `get_company(&state)` dans la transaction de `bank-account`
⇒ test 13 rouge (F3-9). Consigner au Dev Agent Record.

### Ce que la story ne fait pas

- **`seed-demo`** — 15-7b1 ; **`reset`** — 15-7b2 (avec #528 et #279).
- **Les variantes `_in_tx` et les primitives `kesh-db`** — 15-7a1.
- **#435** (`login`, `logout`, `refresh`, `change_password`) — reste `Exempt`.
- **#431** — les `::user` existants, dont `fiscal_years::create_if_absent_in_tx`.
- **La création de la société stub par le bootstrap** (`auth/bootstrap.rs:347`, `insert_stub_company`)
  reste sans `company.created` : le premier démarrage n'a pas d'acteur authentifié. Dit au manuel
  (AC 13).
- **Le RBAC des routes d'onboarding** (tout rôle authentifié) : constaté, hors périmètre.
- Aucun changement frontend, aucune migration.

### References

- Issue #434 ; Story 25-1b (`25-1b-trous-alimentation.md`, AC 7–11 et Dev Notes « Le patron
  d'appel ») ; Story 25-1c-a (libellés, `audit_labels.rs`).
- Fiche index `15-7-trace-onboarding.md` ; fiches sœurs `15-7a1-socle-transactions-onboarding.md`,
  `15-7b-trace-demo-et-remise-a-zero.md`.
- Rapports : P1 `target/gate-logs/15-7-p1-{R,F}.md`, P2 `target/gate-logs/15-7a-p2-{R,F}.md` (non
  versionnés) ; prompts versionnés `15-7-validate-prompt-p1.md`, `15-7a-validate-prompt-p2.md`.
- `CLAUDE.md` § Review Iteration Rule (propagation, recompte, manuel), § Test Locally First.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`), agent de développement en autonomie (consignes de l'Epic 15).

### Debug Log References

- Gate backend complet, au dernier commit de code (`7fb1a552`), base `kesh_157a2` remise à zéro
  (DROP/CREATE, migrations, seed) : `scripts/test-fast.sh` — fmt, clippy `-D warnings`,
  nextest **2908 exécutés, 2908 passés, 4 ignorés** (`target/gate-15-7a2-backend.log`).
- Gate frontend complet : `npm run check` 0 erreur (27 avertissements préexistants),
  `lint-i18n-ownership` PASS, `test:unit` **112 fichiers, 1091 tests passés**, `build` vert.
- E2E complet au même commit (backend `kesh-api` sur `:3013`, base `kesh_e2e_157a2` reconstruite,
  secrets `openssl rand`, `KESH_COOKIE_SECURE=false`, SMTP factices, répertoires inbox/documents du
  worktree ; `/health` → `smtpConfigured: true`) : **245 passés, 9 échoués, 19 sautés** (01:27 UTC).
  Les neuf échecs sont ceux de `docs/testing.md` § « Les échecs attendus », fichier par fichier :
  sept KF-029 (#97 — `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
  `onboarding.spec.ts:57`, `:77`, `:150` ; les deux Path B attendent `#coord-address`, absent de
  l'écran) et deux KF-045 (#421 — `invoices.spec.ts:415`, `:439`, avant 12:00 UTC) ; pas de huitième
  variable sur ce run. Le log backend ne porte aucune erreur interne (que des `unauth` attendus).
  Backend arrêté par son PID.
- **Clôture — gates complets sur l'état rebasé** (`origin/main` = `de285ea8`, 15-11b ; tête de code
  `004341d0`, dernier commit de code de la story), base `kesh_157a2` remise à zéro (DROP/CREATE,
  migrations, seed) : `scripts/test-fast.sh` — fmt, clippy `-D warnings`, nextest **2934 exécutés,
  2934 passés, 4 ignorés** (`target/gate-15-7a2-backend-cloture.log`). Frontend : `npm run check`
  0 erreur (27 avertissements préexistants), `lint-i18n-ownership` PASS, `test:unit` **112 fichiers,
  1091 tests**, `build` vert. E2E complet (backend `:3013`, base `kesh_e2e_157a2` reconstruite,
  secrets `openssl rand`, `KESH_COOKIE_SECURE=false`, SMTP factices, inbox/documents du worktree,
  `/health` → `smtpConfigured: true`) : **247 passés, 9 échoués, 17 sautés** (02:37–02:47 UTC). Les
  neuf échecs, fichier par fichier contre `docs/testing.md` : sept KF-029 (`mode-expert.spec.ts:26`,
  `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) et deux
  KF-045 (#421, `invoices.spec.ts:415`, `:439`, avant 12:00 UTC). Log backend sans erreur interne
  ni `Permission denied`. Backend arrêté par son PID.
- Gates ciblés pendant le développement : `binary(onboarding_audit_e2e)` 19/19 ;
  `onboarding_e2e`, `onboarding_path_b_e2e`, `audit_route_registry`, `audit_label_registry`,
  `fiscal_years_e2e`, `profile_e2e` 88/88.

### Completion Notes List

- **T0** : écarts de fait seulement (Change Log du 2026-10-09). Registre recompté depuis la source :
  95/15/2 → **104/6/2**, total 112 (la fiche disait 94→103 et 3 « sans matière »). Le doc-comment de
  tête du registre disait déjà 112 et 115 (exacts) : rien à recompter.
- **Mutations jouées une à une, constatées rouges, puis fichier restauré et touché** (choix
  C-15-7a2-3 pour la forme des deux premières) :
  1. appel à `record_step_completed_in_tx` retiré de `complete_step` ⇒ test 1 rouge ;
  2. `COMMIT` SQL après `update_in_tx` dans `update_company_coordinates_in_tx` ⇒ test 9 (b) rouge
     (société `Nouveau Nom SA`, `version` 3, `is_stub` levé, malgré le 500) ;
  3. `COMMIT` SQL après `update_in_tx` dans `update_company_in_tx` (société commitée avant le plan)
     ⇒ test 9 (a) rouge (`accounting_language` = DE, `version` 2) ;
  4. comparaison d'étape retirée de `lock_state_at_step` ⇒ tests 10 (200 au lieu de 400) et 11
     (`[200, 200]`) rouges ;
  5. condition « stub levé » retirée de la règle de l'AC 3 ⇒ test 5 (f) rouge ;
  6. `companies::list(&state.pool, …)` au lieu de `lock_company` dans la transaction de
     `bank-account` ⇒ test 13 rouge (503 : le pool d'une connexion expire).
- **Test 5** est écrit en six fonctions (a à f), une par montage ; le fichier compte donc
  **19** fonctions de test pour les **13** tests de la fiche (recompté : `grep -c` des attributs).
- **E-1** (revue de la 15-7a1) : `grep "FROM onboarding_state WHERE singleton = TRUE FOR UPDATE"`
  ne rend plus que `reset` (`routes/onboarding.rs:274`, 15-7b2). **E-3** : la garde « aucun compte »
  est lue dans la transaction après le verrou d'état (`count_by_company(&mut **tx, …)`). **Booléen
  « inséré »** : lu dans `finalize_inner`, donc dans la tentative rejouée. **B-2** : `lock_state_in_tx`,
  `clear_stub_in_tx`, `UpsertPrimaryOutcome::{Updated, Unchanged}` et le booléen `inserted` ont
  désormais un appelant de production.
- **Changements de comportement à dire** (Points de vigilance) : (i) le perdant d'une course à la
  même étape reçoit 400 `ONBOARDING_STEP_ALREADY_COMPLETED` au lieu de 409 ; (ii) `companies.version`
  ne bouge plus quand rien ne change (`language`, `org-type`, `accounting-language`, `coordinates`
  sur société non provisoire), et `coordinates` sur société provisoire aux coordonnées changées la
  porte à `+2` ; (iii) `mode` n'écrit `installation.ui_mode_changed` que si le mode change, alors
  que `PUT /profile/mode` l'écrit toujours — divergence assumée, **à signaler** (alignement hors
  périmètre) ; (iv) `company_invoice_settings.created` est en snake_case, `.updated` en camelCase
  (héritage `settings_snapshot_json`) — non corrigé.
- **Dette #431** : `fiscal_years::create_if_absent_in_tx` attribue toujours `fiscal_year.created`
  par `::user` ; `finalize` n'écrit pas sa propre entrée d'exercice (AC 7). Non corrigé ici.
- **AC 12 — grep du symptôme**, exécuté après correction ; restent, triés à la main comme
  légitimes : `kesh-seed/src/lib.rs:167` et `fiscal_years_e2e.rs:1299` (seed de démonstration,
  15-7b1) ; `routes/onboarding.rs:237` (`seed_demo`, E-2 → 15-7b1) ; `auth/bootstrap.rs:34`
  (toujours vrai, nom du helper mis à jour), `:129` (historique), `:626`, `:821` (tests du
  break-glass) ; `routes/profile.rs:53` (la route de profil bumpe toujours) ;
  `audit_route_registry.rs:190`, `:284`, `:603` (les deux exemptions restantes, motif réécrit) ;
  `companies.rs:147`, `:159` (le SQL de `clear_stub_in_tx`) ; `bank_accounts.rs:633` (sans
  rapport) ; `fiscal_years.rs:205`, `accounts.rs:9`, `:991` (réécrits par cette story). Corrigés :
  `TODO(L65 …)` et commentaire « `is_stub = FALSE` inconditionnel » d'`onboarding.rs` (code
  réécrit), `companies.rs` (route) `:233`, `repositories/companies.rs:151`, `accounts.rs:8`, `:985`,
  `:1023`, `vat_rates.rs:300`, `fiscal_years.rs:203-206`, `repositories/onboarding.rs` (« dix
  appelants »), `profile.rs:47-49`, `bootstrap.rs:32-35` et `kesh-seed/src/lib.rs:66`, `:156`
  (nom du helper renommé `ensure_company_with_language_in_tx`), Pattern 5 (`bank_accounts` en 5ᵉ
  position, `grep FOR UPDATE` de `bank_accounts.rs` : seule la sentinelle `companies` le précède ;
  une ligne par famille de route).
- **Manuels** : `admin-manual.tex` (décompte 104/112, six exemptions nommées dont le
  renouvellement de session, deux « sans matière », 104 + 6 + 2 = 112 ; réserve OLICo), glossaire
  relu (renvoie aux réserves, ne contredit rien) ; `user-manual.tex` (énumération, deux familles,
  paragraphe « Les entrées de la configuration initiale »). PDF régénérés (`make fr`) et contrôlés
  aplatis ; le PDF de la brochure, régénéré sans changement de source, a été rétabli. Valeurs
  grepées (`434`, `87`, `105`, `séquence d.installation`) : restent la ligne v0.12.1 du README
  (historique) et la mention légitime de #434 dans le décompte.
- Choix consignés : **C-15-7a2-1**, **C-15-7a2-2**, **C-15-7a2-3**.
- **Remédiation de la revue de code P1** (2026-10-09) : gate **ciblé** seulement — base `kesh_157a2`
  remise à zéro (DROP/CREATE, migrations, seed), `cargo fmt --check` vert, `cargo clippy --workspace
  --all-targets -D warnings` vert, `binary(onboarding_audit_e2e)` **22/22**, `binary(audit_route_registry)`
  11/11. **Gate complet et E2E complet non rejoués : ils viendront à la clôture.** Le fichier compte
  désormais **22** fonctions de test (21 `#[sqlx::test(` + 1 `#[test]`, recompté), contre 19 au commit
  `a298d19f` : trois neuves (9 (c), 9 (d), 14), deux étendues (1, 13). Huit mutations jouées une à une
  sur `routes/onboarding.rs`, constatées rouges, fichier restauré (`cmp` contre la copie) et touché :
  7–12. `require_not_demo` à `false` sur `start-production`, `org-type`, `accounting-language`,
  `coordinates`, `bank-account`, `skip-bank` ⇒ test 14 rouge à chaque fois, sur la route mutée
  (200 au lieu de 400) ;
  13. `COMMIT` SQL entre `bulk_create_from_chart_in_tx` et `account.chart_loaded` ⇒ 9 (c) rouge
  (86 comptes restés au lieu de 0) ;
  14. `COMMIT` SQL entre `update_step_in_tx` et `record_step_completed_in_tx` ⇒ 9 (d) rouge (état
  `(7, 2)` au lieu de `(6, 1)`).
  Aucune ligne de code de production exécutable touchée. Choix consigné : **C-15-7a2-4**.

### File List

- `crates/kesh-api/src/routes/onboarding.rs` — neuf handlers en une transaction, helpers
  (`lock_state_at_step`, `complete_step`, `conclude_step`, `lock_company`, `update_company_in_tx`,
  `ensure_company_with_language_in_tx`, `update_company_coordinates_in_tx`,
  `audit_bank_account_upsert`, `audit_finalize_seed`, macro `company_select!`).
- `crates/kesh-db/src/repositories/onboarding.rs` — `record_step_completed_in_tx`, doc-comment.
- `crates/kesh-api/src/audit_labels.rs`, `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`.
- `crates/kesh-api/tests/onboarding_audit_e2e.rs` (neuf), `tests/common/mod.rs`
  (`audit_sequence`), `tests/onboarding_path_b_e2e.rs`, `tests/audit_route_registry.rs`.
- Commentaires seuls : `crates/kesh-db/src/repositories/{accounts,vat_rates,fiscal_years,companies}.rs`,
  `crates/kesh-api/src/routes/{profile,companies}.rs`, `crates/kesh-api/src/auth/bootstrap.rs`,
  `crates/kesh-seed/src/lib.rs`.
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md`, `docs/manual/fr/{admin,user}-manual.{tex,pdf}`,
  `CHANGELOG.md`.
- `_bmad-output/implementation-artifacts/{15-7a2-trace-installation-production.md,sprint-status.yaml,epic-15-choix-autonomes.md}`.

## Change Log

- 2026-10-08 — Née du découpage de la 15-7 à la passe de validation P1 (choix C-15-7-8), sous le nom
  **15-7a** (`15-7a-trace-installation-production.md`, commit `3846b206`). Reprend les AC de la 15-7
  portant sur les neuf routes de production, remédiés selon les findings P1 (détail au Change Log de la
  fiche index). Recompte d'alors : 14 AC, 8 tâches, 12 tests.
- 2026-10-08 — **Passe de validation P2** (prompt versionné `15-7a-validate-prompt-p2.md` ; deux
  lentilles en contexte frais, R regression hunter et F full-scope adversary ; rapports
  `target/gate-logs/15-7a-p2-{R,F}.md`). Bruts, recomptés depuis les rapports : R **3 MEDIUM, 8 LOW** ;
  F **5 MEDIUM, 7 LOW**. Après fusion des doublons : **0 CRITICAL, 0 HIGH, 5 MEDIUM, 15 LOW distincts**.
  Trois des cinq MEDIUM (F-2, F-3/R2-1, F-4) portaient sur des tests ajoutés par la remédiation P1.

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | R2-3 = F-1 | MEDIUM | R, F | la 15-7a franchit elle-même le seuil de cinq modules (neuf) | **découpage** en 15-7a1 (socle `kesh-db`) + 15-7a2 (cette fiche) — C-15-7-19 ; décomptes déclarés (sept / quatre) |
  | R2-1 = F-3 | MEDIUM | R, F | test 9 inexécutable : `coordinates` exige l'étape 5, `accounting-language` l'étape 4 | test 9 : un montage par route, origine prouvée par différence, `raw_sql` |
  | R2-2 = F-5 | MEDIUM | R, F | AC 3 : « ssi la version change » contre « `is_stub` seul suffit » ; effet de `clear_stub_in_tx` non dit | AC 3 réécrit (règle composée, relecture), SQL fixé à la 15-7a1 AC 6 — C-15-7-21 ; test 5 étendu |
  | F-2 | MEDIUM | F | test 11 ne mord pas sur la revérification sous verrou | test 11 : statuts, une seule étape, version, mutation dédiée |
  | F-4 | MEDIUM | F | test 5 contredit l'AC 1 sur `accounting-language` (plan non préchargé) | test 5 : plan préchargé |
  | R2-4 | LOW | R | « deux encore suivis par #434 » faux | AC 12 |
  | R2-5 | LOW | R | « le grep l'est » faux (site coupé sur deux lignes) | AC 12 : affirmation retirée, `appelants la partagent` au motif |
  | R2-6 | LOW | R | `count_by_company` sans autre appelant ; appelants de `upsert_primary` | 15-7a1 AC 2, 3 |
  | R2-7 | LOW | R | `lock_state_at_step` ne peut pas annuler une transaction prise par référence | AC 8.2 ; primitive à la 15-7a1 AC 7 |
  | R2-8 | LOW | R | montages des tests 4, 8, 11 sous-spécifiés | tests 4, 8, 11 |
  | R2-9 | LOW | R | lignes approximatives (`company_invoice_settings.rs`, `audit.rs`) | 15-7a1 AC 4 ; Dev Notes |
  | R2-10 | LOW | R | deux sémantiques du no-op pour `installation.ui_mode_changed` | AC 1, Points de vigilance (signalé) |
  | R2-11 | LOW | R | `fiscal_years.rs:202` rendu partiellement faux par cette fiche | AC 12 (attribué ici) |
  | F-6 | LOW | F | appels directs de `insert_with_defaults_in_tx`, tests tautologiques, MIRROR hors grep | 15-7a1 AC 1, 4, 8, Points de vigilance |
  | F-7 | LOW | F | `company_id` de `company.created` = société morte | test 7 ; fermé par la 15-7b (C-15-7-23) |
  | F-8 | LOW | F | le manuel nomme un filtre par numéro de compte inexistant | AC 13 |
  | F-9 | LOW | F | motif des deux exemptions restantes devenu faux | AC 10 |
  | F-10 | LOW | F | changement d'IBAN invisible dans `bank_account.updated` | AC 1, 9 ; test 6 |
  | F-11 | LOW | F | une remise à zéro à l'étape ≤ 2 efface les entrées neuves sans trace jusqu'à la 15-7b | même release que la 15-7b — C-15-7-25 |
  | F-12 | LOW | F | `language` et `mode` communes aux deux chemins | AC 13 |

  **Trouvé pendant la remédiation** : `record_step_completed_in_tx` ne peut pas vivre à la 15-7a1 —
  la garde bilatérale des libellés exige l'action déclarée et libellée dès qu'un site de production
  l'écrit ; et un helper privé `kesh-api` sans appelant ferait échouer `clippy -D warnings`
  (`dead_code`) — d'où la primitive `lock_state_in_tx` dans `kesh-db` et `lock_state_at_step` ici
  (C-15-7-20). Propagation : grep du symptôme sur les trois fiches, l'index, le registre et le
  sprint-status. Recompte : **14 AC, 8 tâches, 12 tests**.
- 2026-10-08 — **Passe de validation P3 — CONVERGÉE** (prompt versionné `15-7a2-validate-prompt-p3.md` ;
  deux lentilles Sonnet en contexte frais, R regression hunter et F full-scope adversary ; rapports
  `target/gate-logs/15-7a2-p3-{R,F}.md`, non versionnés). Bruts, recomptés depuis les rapports : R
  **9 LOW** ; F **10 LOW** ; aucun CRITICAL, HIGH ni MEDIUM. Après fusion (R3-1 = F3-1, R3-8 = F3-8) :
  **17 LOW distincts**, tous appliqués.

  | finding | lentilles | objet | sort |
  |---|---|---|---|
  | R3-1 = F3-1 | R, F | « huit » routes à plusieurs transactions : cinq | inventaire § 2 |
  | R3-2 | R | dépendance de la 15-7b mal attribuée | Dépendances |
  | R3-3 | R | « trois copies » remplacées (15-7a1 AC 7) : deux ici, la troisième à la 15-7b | 15-7a1 AC 7 |
  | R3-4 | R | `start-production` : non réinitialisable **même avec** le drapeau | AC 1 |
  | R3-5 | R | lignes `audit_log.rs:91-92`, `:999-1003` | corrigées |
  | R3-6 | R | message de l'assertion `traced` et doc-comment de tête du registre | AC 10 |
  | R3-7 | R | valeurs du test 1 non fixées | test 1 |
  | R3-8 = F3-8 | R, F | perdant d'une course : 409 → 400 non consigné | AC 8.2, Points de vigilance |
  | R3-9 | R | snake_case contre camelCase pour `company_invoice_settings.*` | Points de vigilance |
  | F3-2 | F | test 9 (a) vide si la langue ne change pas | langue différente, mutation « commit de la société avant le plan » |
  | F3-3 | F | test 10 ne mord pas | conservé : mord sans la comparaison de `lock_state_at_step` (mutation partagée avec le 11) — C-15-7-29 |
  | F3-4 | F | helpers de `tests/common/mod.rs` ignorés | `audit_actions`/`actor`/`details`/`count` + `audit_sequence` neuf, partagé avec la 15-7b |
  | F3-5 | F | « sous le seuil » inexact | neuf modules touchés (quatre de code, cinq de commentaires), signal déclaré |
  | F3-6 | F | manuel : #435 compte quatre routes, le texte en nomme trois | AC 13 |
  | F3-7 | F | `entity_type`/`entity_id` de `company.updated` | AC 1 |
  | F3-9 | F | aucun test à pool d'une connexion | test 13 neuf + mutation — C-15-7-29 |
  | F3-10 | F | lignes du Pattern 5 ; position de `bank_accounts` | `:298-302` ; après `accounts`, avant `company_invoice_settings` |

  **Trend de la validation** (fiche 15-7 → 15-7a → 15-7a2) : P1 (sur la 15-7) **1 HIGH, 8 MEDIUM, 10
  LOW** distincts → P2 (sur la 15-7a) **5 MEDIUM, 15 LOW** → P3 (sur la 15-7a2) **0 au-dessus de LOW,
  17 LOW**. Modèles : lentilles Sonnet aux trois passes (rotation D6 : Sonnet ↔ Opus pour les passes
  complètes ; l'orchestrateur, Opus, applique et reprend les axes déclarés non exercés). Reclassements :
  aucun. Deux découpages en cours de route (C-15-7-8, C-15-7-19). **Validation close** au critère
  « uniquement des LOW » (§ Review Iteration Rule). Recompte : **14 AC, 8 tâches, 13 tests**.
- 2026-10-08 — **Correction de fait, hors passe de validation** (la validation reste close) : la P2 de la
  15-7a1 (R-2/F-2) a établi que tout parcours monté par `ensure_admin_user` sur base vide écrit
  `user.created` avant toute route (`auth/bootstrap.rs:148`). Le test 1 et la consigne héritée du test 8
  de la 15-7a1 (`Points de vigilance`) asserteront donc `user.created` en tête de séquence ; le montage
  du test 1 est écrit (bootstrap, sans `create_test_company`). Choix C-15-7-30. Et la 15-7b est
  découpée (C-15-7-31) : les renvois à la « 15-7b » du corps désignent désormais la 15-7b1
  (`seed-demo`) ou la 15-7b2 (`reset`, #528, #279) ; le motif réécrit du registre devient « peuplement
  de démonstration (15-7b1) et remise à zéro (15-7b2) ». Recompte inchangé :
  **14 AC, 8 tâches, 13 tests**.
- 2026-10-09 — **Reçu de la revue de code de la 15-7a1** (passe P1, trois lentilles Sonnet, 0 au-dessus
  de LOW ; rapports `target/gate-logs/15-7a1-review-p1-{B,E,A}.md`, non versionnés). Quatre constats
  portent sur des sites que **cette** fiche câble ; aucun ne change ses AC, ils sont écrits ici pour que
  le développement ne les manque pas :
  - **E-1 — trois copies en ligne de `LOCK_SQL`** dans `routes/onboarding.rs` (`:250` `reset`, `:649`
    `finalize_inner`, `:802` ; relevé à la revue, les `:653`/`:806` de l'AC 8.2 ont dérivé de quatre
    lignes). Elles sélectionnent en plus `singleton`, que `OnboardingState` ne porte pas : résultat
    identique, que rien ne garantit. L'AC 8.2 prévoit déjà de passer `:649` et `:802` par
    `onboarding::lock_state_in_tx` (donc par `onboarding::LOCK_SQL`) ; `:250` reste à la 15-7b2. Le
    gate du développement greppe `FROM onboarding_state WHERE singleton = TRUE FOR UPDATE` et ne doit
    plus rendre que `:250`.
  - **E-3 — la garde « aucun compte » est lue sur le pool, hors verrou** (`routes/onboarding.rs:376-387` :
    `count_by_company(&state.pool, …)` puis `bulk_create_from_chart` sur le pool). Deux appels
    concurrents à l'étape 4→5 peuvent lire 0 tous deux ; le second échoue en 1062 et rend un 5xx.
    L'AC 8.2 (point 4) prescrit déjà la correction — `count_by_company(&mut *tx, …)` **dans** la
    transaction, **après** `lock_state_at_step`, puis `bulk_create_from_chart_in_tx` ; le constat
    confirme que c'est un défaut de concurrence réel et non une simple mise en ordre, et le test 13
    (pool d'une connexion) en est le témoin.
  - **Booléen « inséré » relu à chaque tentative** : `finalize` est rejouée par
    `retry_app_on_deadlock` (`routes/onboarding.rs:615`). Le booléen de `insert_with_defaults_in_tx`
    et le `Vec<VatRate>` de `seed_default_swiss_rates_in_tx` (AC 5) doivent être lus **dans** la
    fermeture rejouée, de la tentative qui commite — jamais capturés d'une tentative annulée : une
    première tentative qui insère puis bute sur un interblocage est annulée, et la suivante insère à
    nouveau ; c'est elle seule qui dit vrai. Les entrées d'audit s'écrivent dans la même fermeture.
  - **B-2 — code sans appelant de production** jusqu'ici : `companies::clear_stub_in_tx`,
    `onboarding::lock_state_in_tx`, `UpsertPrimaryOutcome::{Updated, Unchanged}` et le booléen
    `inserted` (ignoré par `routes/onboarding.rs:717`). Cette fiche (et la 15-7b1 pour
    `clear_stub_in_tx` côté `seed_demo`) les consomme ; ce qui resterait sans appelant à la fin de
    l'Epic 15 se retire.
  Le constat **E-2** (`seed_demo` lève `is_stub` sans borner à `id` ni bumper `version`,
  `routes/onboarding.rs:211`) relève de la 15-7b1, qui le prévoit déjà (son § 2 : `clear_stub_in_tx`
  remplace l'`UPDATE`). Aucun AC, tâche ni test ne change : **14 AC, 8 tâches, 13 tests**.
- 2026-10-09 — **T0 du développement — relecture de la fiche contre `ec745d0c`** (`origin/main`, 15-7a1,
  15-5d et 15-11a mergées). Aucun écart ne change une règle ni un AC sur le fond ; écarts de **fait**,
  relocalisés par le texte :
  - **Registre des routes, recompté depuis la source** (`the_registry_partition_is_what_the_story_declares`
    et `LIB_ROUTES`) : la base porte `traced` **95** (et non 94 : la 15-8a a ajouté le `PUT` des
    écritures), `exempt` 15, `no_matter` **2** (et non 3), total 112. Après la story :
    `traced` **104**, `exempt` **6**, `no_matter` **2**, total 112. Le manuel (AC 13) dira donc
    « 104 des 112 routes » et « deux routes sans matière » — 104 + 6 + 2 = 112.
  - **Lignes dérivées** : `routes/onboarding.rs` compte 1030 lignes ; les trois copies du verrou d'état
    sont à `:249-250` (`reset`), `:648-649` (`finalize_inner`), `:801-802` (relecture) ; le retour
    idempotent de `finalize` à `:672-675` ; le `TODO(L65 …)` à `:523-527` ; le commentaire
    « `is_stub = FALSE` inconditionnel » à `:995-998`. `admin-manual.tex` : `:1945` (décompte) et
    `:2128` (réserve), au lieu de `:1821` et `:2004`. Pattern 5 : liste `:298-302` inchangée, table
    `:318` (« company only »).
  - **Variantes de la 15-7a1 vérifiées présentes** : `lock_state_in_tx` et `LOCK_SQL` publique,
    `clear_stub_in_tx` (rend `bool`), `upsert_primary_in_tx` (`UpsertPrimaryOutcome`),
    `bulk_create_from_chart_in_tx` (`Vec<Account>`), `count_by_company` générique sur l'exécuteur,
    `insert_with_defaults_in_tx` (`(réglages, inséré)`), `seed_default_swiss_rates_in_tx`
    (`Vec<VatRate>`, ordre du seed).
  - **« Journal » de la colonne `Rejeu` du registre** = journal **comptable** : les neuf routes restent
    `SansEcritureAuJournal` (elles n'écrivent aucune écriture comptable) — la story ne touche que la
    colonne d'audit.
- 2026-10-09 — **Développement** (commits `c64ad977` code, `e38a3b7a` tests, `7fb1a552` documentation
  et commentaires). Gate backend complet 2908/2908 (4 ignorés), frontend complet vert, E2E complet
  245 passés / 9 échecs attendus (7 KF-029, 2 KF-045) au dernier commit de code ; six mutations
  rouges. Le perdant d'une course passe de 409 à 400 ; `companies.version` ne bouge plus sur un
  no-op et prend +2 sur `coordinates` d'une société provisoire. Statut `review`.
- 2026-10-09 — **Revue de code P1** (Sonnet ×3, contexte frais, prompt versionné ; rapports
  `target/gate-logs/15-7a2-review-p1-{B,E,A}.md`). Bruts, recomptés depuis les rapports : B **4 LOW** ;
  E **1 MEDIUM, 5 LOW** ; A **1 MEDIUM, 5 LOW**. Après fusion des doublons (A-4 = B-1 ; A-5 = B-2 = E-2 ;
  B-4 = E-4) : **0 CRITICAL, 0 HIGH, 2 MEDIUM, 10 LOW distincts**. Remédiation sans aucune ligne de
  code de production exécutable (tests, doc-comment de test, fiche, CHANGELOG, registres).

  | finding | sévérité | objet | sort |
  |---|---|---|---|
  | E-1 | MEDIUM | la garde `require_not_demo` n'est mordue par aucun test | **test 14** `demo_installation_is_refused_by_every_production_step` : les six routes gardées (`start-production`, `org-type`, `accounting-language`, `coordinates`, `bank-account`, `skip-bank` — revérifiées au code ; E en annonçait « sept » pour six citées), `is_demo = TRUE` à l'étape exacte ⇒ 400 `ONBOARDING_STEP_ALREADY_COMPLETED`, société, état, comptes, comptes bancaires et audit inchangés ; contrôle par différence (`is_demo` levé ⇒ 200). Six mutations rouges |
  | A-1 | MEDIUM | le test 9 échouait sur `company.updated`, avant le plan et avant l'étape : ses moitiés « plan » et « étape » étaient vraies par construction | **9 (c)** `chart_loading_is_atomic_with_its_trace` (langue égale, plan non chargé : la première écriture d'audit est `account.chart_loaded`) et **9 (d)** `step_is_atomic_with_its_own_entry` (`skip-bank` : la première écriture est l'entrée d'étape) ; les deux mutations `COMMIT` rouges |
  | A-2 | LOW | `details` de `company_invoice_settings.created` et `vat_rate.created` peu assertés | test 1 : objets **entiers** comparés aux lignes en base (neuf clés ; cinq clés par taux) |
  | A-3 | LOW | AC 10 / AC 13 et `sprint-status.yaml` portaient 94→103 / 3 | corrigés en 95→104 / 6 / 2 = 112, annotés. ⚠️ Résidu **hors périmètre** signalé : la fiche 15-7b1 (`:199`, `:243`, `:351`) part encore de « 103 des 112 » et de trois « sans matière » — à recompter à son développement (104 → 105, 105 + 5 + 2 = 112) |
  | A-4 = B-1 | LOW | variation de `companies.version` non dite au CHANGELOG | une phrase au CHANGELOG (inchangée sur no-op, +2 sur les coordonnées changées d'une société provisoire) |
  | A-5 = B-2 = E-2 | LOW | test 13 limité à quatre routes | étendu aux **neuf** routes (parcours complet sur le pool d'une connexion, puis `skip-bank` à l'étape 6). Reste non prouvé par un test : le rejeu de `finalize` sur un 1213 forcé (A-5, second volet) — angle mort laissé, le banc `capture_rejeu` n'a pas été branché ici |
  | B-4 = E-4 | LOW | huit routes d'étape sans rejeu sur interblocage | **choix assumé** C-15-7a2-4, écrit au point (iv) du doc-comment de `audit_route_registry.rs`, renvoi au point (vi) |
  | B-3 | LOW | duplication du `SELECT … FOR UPDATE` de la société dans `ensure_company_with_language_in_tx` ; verrou de `coordinates` pris deux niveaux plus bas | non corrigé (code de production) ; dette cosmétique |
  | E-3 | LOW | `get_or_init_state` hors transaction : course d'initialisation possible sur une base sans ligne d'état | angle mort **préexistant**, non reproduit ; le bootstrap crée la ligne. Non corrigé (code de production) |
  | E-5 | LOW | échec de `response_with_stub` après `COMMIT` ⇒ 500 alors que l'étape est franchie | forme inchangée depuis avant la story ; angle mort de réponse, non de données |
  | E-6 | LOW | `fiscal_year.created` attribué par `::user` dans `finalize` (garde de source aveugle à `fiscal_years.rs`) | dette **#431** déjà assumée à l'AC 7 ; le manuel ne promet pas l'attribution à la clé (relu par E) |
  | A-6 | LOW | E2E : les specs Path B échouent (KF-029) avant `coordinates` : le navigateur n'exerce pas `coordinates`, `bank-account`, `finalize` | limite de preuve, non défaut ; ces routes sont tenues par `onboarding_audit_e2e` et `onboarding_path_b_e2e` |

  Gate **ciblé** (cf. Dev Agent Record) ; gate complet et E2E à la clôture.

- 2026-10-09 — **Revue de code P2 ciblée — boucle CLOSE** (Haiku, une lentille braquée sur le seul commit
  de remédiation `23b3e46e` — `004341d0` après rebase ; prompt versionné `15-7a2-review-prompt-p2-ciblee.md` ;
  rapport `target/gate-logs/15-7a2-review-p2-ciblee.md`, non versionné). **0 CRITICAL, 0 HIGH, 0 MEDIUM,
  0 LOW** ; axes exercés, déclarés par la passe : les cinq demandés (test 14 aux étapes exactes avec
  contrôle par différence, 9 (c) et 9 (d), test 13 sur neuf routes, `details` complets du test 1,
  aucune ligne de production touchée) ; axe non exercé : aucun déclaré. Le cinquième est vérifié par
  l'orchestrateur de clôture (`git show --stat` : tests, fiche, registres, CHANGELOG seulement) : la
  remédiation ne touche **aucune ligne de code de production**, ce qui permet de clore après une passe
  ciblée (§ « La passe ciblée »). **Trend de la revue de code** : P1 (Sonnet ×3) **2 MEDIUM, 10 LOW**
  distincts → P2 ciblée (Haiku) **0**. Modèles : Sonnet pour la passe complète, Haiku pour la passe
  ciblée (D6). Reclassements : aucun ; B-4 = E-4 assumé (C-15-7a2-4).
- 2026-10-09 — **Clôture** : rebasée sur `origin/main` (`de285ea8`, 15-11b). Conflits : `admin-manual.pdf`
  (régénéré sur le `.tex` fusionné, contrôlé aplati) et le registre des choix (union). Aucun conflit de
  code ; `routes/onboarding.rs` lit déjà l'environnement par `config::env_nonempty`. Partition du
  registre **recomptée depuis `LIB_ROUTES`** : 104 + 6 + 2 = 112 (rejeu : 22 + 4 + 89 = 115), inchangée,
  manuel juste. Gates complets sur l'état rebasé : backend 2934/2934, frontend vert (1091 tests), E2E
  247 passés / 9 échecs attendus (7 KF-029, 2 KF-045). Statut **done**. Choix C-15-7a2-5. `refs #434` :
  c'est la 15-7b2 qui la fermera — **même release que la 15-7b1 et la 15-7b2**.
