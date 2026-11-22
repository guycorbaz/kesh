# Story 40-8b : téléchargement Stripe par période — route, tâche, écran, manuels

Status: ready-for-dev

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, chemins relevés sur `origin/main` au commit `fd04099f`. Epic 40,
**phase 2** (décision de Beat du 2026-11-20). **Dépend de** : 40-8a (client), 40-7b (import CSV), du **cœur commun `ProviderLine`** (`provider_imports.rs`) extrait par 38-12a — ou par 40-7b s'il passe
avant (socle S6) ; 40-8b l'appelle sans l'extraire, 40-7c
(rapprochement) et du **socle S1 à S9** (fichier d'epic), écrit par 38-12a ou par la première fiche de phase 2 développée
(C-40-K6) ; le dialogue d'écran est celui de 38-12b (S7). L'automatisme (téléchargement périodique) vient plus tard encore,
story à part.*

## Amendement du 22.11 (avant le développement, contre le code de `epic/40-stripe` après la fusion de la 40-8a)

*Relevé le 2026-11-22 sur la branche `story/40-8b`, partie de `epic/40-stripe` au commit de fusion de la 40-8a (`5e7258d2`). Ce qui
suit l'emporte sur le texte de la fiche là où ils diffèrent.*

1. **Le dialogue générique de la 38-12b n'est pas encore sur `epic/40-stripe`** (S7, S8) :
   `frontend/src/lib/features/provider-download/` et `frontend/tests/e2e/support/provider-mock.ts` n'existent pas sur la
   branche (`git log epic/40-stripe --oneline | grep 38-12b` ne rend que des commits de fiche) ; la 38-12b est en revue
   sur sa propre branche, ni dans `epic/38-paypal` ni dans `epic/40-stripe`. Le champ générique `providerApi` de la réponse
   des comptes bancaires (38-12b AC-1) n'existe pas non plus (`grep -rn providerApi crates/kesh-api/src frontend/src` :
   aucun). **Le développement de l'AC-4 (écran, E2E) attend la fusion de la 38-12b dans `epic/38-paypal`, puis celle
   d'`epic/38-paypal` dans `epic/40-stripe`** ; le dialogue ne se réécrit pas ici. La 40-8b n'est pas commencée : la partie
   serveur (AC-1 à AC-3) est développable sans la 38-12b, mais la story se développe d'un tenant, après la fusion.
2. **Signature du socle modifiée par la 38-12b** (revue P1 de la 38-12b, M1) : `Downloads::start` prend la période lue
   `(from, to)` et rend au refus `RunningJob { id, from, to }` ; le 409 `PROVIDER_DOWNLOAD_RUNNING` porte alors
   `details: { provider, jobId, from, to }`. La route Stripe suit la forme de la 38-12b telle qu'elle sera fusionnée, non
   celle de `provider_download.rs` d'aujourd'hui (`start(provider, company_id, account, steps_total) -> Result<JobHandle,
   u64>`).
3. **Comptes du registre** (AC-1) : les valeurs écrites (157, 149, 161) sont périmées. Sur la base de cette story :
   `LIB_ROUTES.len()` **162**, `traced` **154**, `LIB_ROUTES.len() + TEST_ENDPOINT_ROUTES.len()` **166**
   (`crates/kesh-api/tests/audit_route_registry.rs:1142`, `:1188`, `:1239`) ; la 40-8b les relève de un (163, 155, 167),
   **recomptés après la fusion de la 38-12b**, qui peut en ajouter.
4. **Longueur des identifiants (revue P1 de la 40-8a, L5)** : le client (`stripe::rows`) contrôle la forme des
   identifiants (`valid_id`) mais pas leur longueur ; l'import du CSV applique `check_id_lengths`
   (`routes/stripe_imports.rs:129`, borne `STRIPE_ID_MAX_CHARS`). La tâche de la 40-8b **appelle `check_id_lengths`** sur
   les lignes de `fetch_rows` avant le chemin d'import commun (un refus rend `PROVIDER_RESPONSE_INVALID`, rien d'écrit),
   avec un test au serveur simulé (identifiant de 300 caractères).
5. **Textes de configuration (revue P1 de la 40-8a, L6)** : `.env.example`, `docker-compose.yml` et
   `docker-compose.prod.yml` décrivent déjà au présent la lecture Stripe par l'API (« Lecture Stripe par l'API (Story
   40-8a) ») ; la 40-8b les relit quand la fonction existe (route, écran) et y renvoie au manuel d'administration.
6. **Client et forme des lignes** : `fetch_rows(client, window, parallel) -> StripeFetched { rows, manual_payouts }`
   (`crates/kesh-api/src/stripe/mod.rs`) lit une seule fenêtre (Stripe sans `max_days`) et ne rend pas de progression ;
   l'étape « versement k/n » de l'AC-1 demande à la 40-8b un crochet de progression autour de la lecture des transactions
   (par versement), sans changer le classement des erreurs. Les versements manuels (`manual_payouts`) vont au compte rendu
   (« versement manuel à examiner »), non à l'import. Le mode de la clé est `StripeClient::mode()` (`live`/`test`).

## Second amendement du 22.11 (étape 0 du développement, après la fusion d'`epic/40-stripe` à `15e9f8b13`)

*Relevé le 2026-11-22 sur `story/40-8b`, après la fusion non squash d'`epic/40-stripe` (commit `d9771239`), qui
porte la 40-7d, la 40-8a et l'Epic 38 avec la 38-12b. Ce qui suit l'emporte sur la fiche et sur le premier
amendement là où ils diffèrent ; chaque point a été relu au code.*

1. **Le point 1 du premier amendement est levé** : `frontend/src/lib/features/provider-download/`
   (`ProviderDownloadDialog.svelte`, `provider-download.ts`, `provider-download.api.ts`),
   `frontend/tests/e2e/support/provider-mock.ts` (avec `hold` et `release()`) et le champ `providerApi` de la liste
   des comptes bancaires (`routes/bank_accounts.rs`, `provider_api_status`) sont sur la branche. Ils se réutilisent
   **tels quels** ; le serveur simulé reçoit des routes Stripe (`stripeMockRoutes`), sans toucher au serveur.
2. **Signature de `Downloads::start`** (point 2) confirmée : `start(provider, company_id, account, (from, to),
   steps_total) -> Result<JobHandle, RunningJob>`, `RunningJob { id, from, to }` ; le `409` porte
   `details: { provider, jobId, from, to }`, comme PayPal.
3. **Comptes du registre** (point 3) recomptés après la fusion : la 38-12b n'ajoute aucune route — `LIB_ROUTES.len()`
   **162**, `traced` **154**, `LIB_ROUTES.len() + TEST_ENDPOINT_ROUTES.len()` **166** ; la 40-8b les porte à **163**,
   **155**, **167**, et la phrase du manuel à « 155 des 163 » et « 155 + 4 + 4 = 163 ».
4. **L5** (point 4) confirmé : `check_id_lengths` (`routes/stripe_imports.rs:129`) se joue sur les lignes de
   `fetch_rows` avant la ventilation ; un refus rend `PROVIDER_RESPONSE_INVALID` `details.lines`, rien d'écrit.
5. **L6** (point 5) : les trois fichiers disaient « Lecture Stripe par l'API (Story 40-8a) » ; ils disent désormais
   « Téléchargement Stripe par l'API (Stories 40-8a, 40-8b) » et renvoient au manuel d'administration.
6. **Crochet de progression** (point 6) : `fetch_rows` reste tel quel (ses tests de la 40-8a aussi) et délègue à
   `fetch_rows_tracked(client, w, parallel, job: Option<JobHandle>)`. Le socle reçoit `JobHandle::set_total(n)`
   (`add_steps` ne crée pas un total inconnu). **Étapes** : `total` nul pendant la lecture de la liste, puis le
   **nombre de versements automatiques**, une étape par versement lu — « versement 5/12 » pour douze versements
   (et non `1 + n`, qui ferait lire « versement 1/13 » avant le premier). Le classement des erreurs ne change pas.
7. **Mode** (AC-4) : `ProviderApiMode` n'a que `live` et `sandbox` (38-12b), et le mode se lisait à l'hôte. Pour
   Stripe, il se lit au **préfixe de la clé** (`stripe::series_mode`, appelé par `ProviderApiConfig::status_for`) :
   `rk_live_` → `live`, `rk_test_` → **`sandbox`** (l'environnement d'essai de Stripe), que le dialogue commun montre
   « bac à sable » sans changement. L'AC-4 écrivait `test` : remplacé par `sandbox`, qui évite un troisième mode au
   socle et un libellé propre au dialogue.
8. **Période** (AC-1) : la règle **commune** (`check_period`, `effective_to`) s'applique telle quelle, comme PayPal
   depuis la 38-12a — une fin future est **ramenée au jour de Zurich**, non refusée (l'AC-1 la refusait) ; `future`
   ne vise que le début. Stripe ne borne pas l'historique : `too_old` vaut au-delà de **30 ans**
   (`stripe::HISTORY_YEARS`), borne de garde contre une date absurde que la plage de la base admet (an 1000).
   **Une seule fenêtre**, toute la période (`period_windows` sans plafond de jours).
9. **Nom et empreinte** (AC-3) : le cœur commun prend **un** nom par import ; il vaut « Stripe API <du>…<au> »
   (période lue), comme « PayPal API <du>…<au> » — la devise est celle du compte de chaque entrée `bank_imports`.
   L'empreinte est celle du cœur pour un téléchargement (`ImportSource::Download` : SHA-256 des identifiants triés de
   la devise ‖ `:` ‖ devise), sans préfixe propre à Stripe ; aucun refus `BANK_IMPORT_DUPLICATE_FILE`.
10. **Compte rendu et rapprochement** (AC-1, AC-3) : `result` = la réponse `201` de `POST /stripe-imports`
    (`account`, `currencies`, `payouts`) plus `manualPayouts` (versements manuels non lus, « à examiner »). Le
    rapprochement de la 40-7c est la **revue** de la page Stripe, que l'import du CSV ne lance pas non plus : la
    page la relit à la fin du téléchargement. Trace `stripe_download.created` (libellés aux quatre locales).
11. **Écran** (AC-4) : le bouton va sur la **page Stripe** (`/reconciliation/stripe`, page commune
    `ProviderReviewPage`, fragments `importActions` et `importNotes`), comme PayPal, et non sur *Import bancaire* ; le
    compte rendu de l'import est extrait en `StripeImportReport.svelte` et sert à la page et au dialogue.

## Story

En tant que **Comptable**, je veux télécharger d'un bouton (ou par une route d'API) les versements Stripe d'une période
choisie, voir la progression, et retrouver ensuite exactement ce que l'import du rapport CSV aurait donné — rattachements,
frais, versements, écarts —, sans doublon si j'ai déjà importé le CSV.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-40-8b-1` | Route et bouton : compte Stripe, période ; un seul téléchargement à la fois par compte. | AC-1, AC-4 |
| `FR-40-8b-2` | Séquentiel par défaut, reprise sur 429 par la politique commune du socle ; progression « versement 5/12 ». | AC-2 |
| `FR-40-8b-3` | Parallélisme en option : `KESH_STRIPE_PARALLEL`, défaut 1, maximum 6. | AC-2 |
| `FR-40-8b-4` | Les lignes passent par le chemin d'import commun puis par le rapprochement de 40-7c ; CSV et téléchargement se mélangent. | AC-3 |
| `FR-40-8b-5` | Manuel d'administration : créer la clé restreinte, ses droits, les variables ; manuel utilisateur : le bouton. | AC-4 |

## Critères d'acceptation

1. **AC-1 — Route et état** (socle S3). `POST /api/v1/stripe-downloads` `{ account, from, to }` : **Comptable et plus,
   sessions seules — clés d'API non admises** (la route fait sortir un appel authentifié par la clé de l'Administrateur ; la
   version initiale admettait les clés `read-write`) : une clé d'API est refusée sur le `POST` **et** le `GET`, dans le
   gestionnaire, avant tout, en 403 `API_KEY_OPERATION_FORBIDDEN` (patron de `bank_imports::delete`,
   `routes/bank_imports.rs:1688`, comme 38-12a AC-8). Le `POST` est inscrit à `LIB_ROUTES`
   (`crates/kesh-api/tests/audit_route_registry.rs:440`) en `Traced, SansEcritureAuJournal` ; dans cette story, le compte de
   `the_registry_partition_is_what_the_story_declares` (`:1082`) relevé de un avec sa raison et la phrase « N des M routes à
   verbe mutant » du manuel d'administration (lue par `le_manuel_dit_le_compte_du_registre`, `:1194`) mise à jour, **ainsi que la phrase « N + E + R = M »** du même paragraphe (même test) et les assertions à valeur littérale qui bougent (`LIB_ROUTES.len()` 157, `traced` 149, `LIB_ROUTES.len() + TEST_ENDPOINT_ROUTES.len()` 161 : relevées de un ; `exempt` et `no_matter` inchangés), comptes
   **recomptés** après les stories fusionnées avant (validation P2). Refus dans cet ordre, rien lancé : 403
   `API_KEY_OPERATION_FORBIDDEN` → 400 `VALIDATION_ERROR` → 404 `PROVIDER_ACCOUNT_NOT_FOUND` (40-7b ;
   plus de `STRIPE_ACCOUNT_NOT_FOUND`) → 422 `PROVIDER_API_NOT_CONFIGURED` (aucune paire d'environnement ne porte ce libellé,
   40-8a AC-1 ; plus de `STRIPE_KEY_NOT_CONFIGURED`) → 400 `PROVIDER_PERIOD_INVALID` (`from` > `to`, `to` dans le futur à Zurich ;
   **plus de plafond de 366 jours** : la pagination par curseur borne chaque appel, la période est celle que Beat choisit) → 409
   `PROVIDER_DOWNLOAD_RUNNING` (une tâche du même compte tourne ; `details: { provider: "stripe", jobId }`, pour que l'écran
   reprenne le suivi). Réponse `202 { jobId }` ; `GET /api/v1/stripe-downloads/{jobId}`
   (société de l'appelant seulement, sinon 404) → `{ state, steps: { done, total }, pausedUntilSeconds, result, error: { code, message, details? } }` — la
   forme commune (un seul nom, `details`), **sans `requests` ni `retries`**. **La panique de la tâche** → `INTERNAL_ERROR` (socle S4). Étapes : la lecture de la liste des versements (`total` nul jusqu'à sa fin),
   puis **une par versement** (« versement 5/12 »). `result` = compte rendu de 40-7b AC-6 suivi de celui de 40-7c AC-8. État en
   mémoire du processus, perdu au redémarrage (limite écrite au manuel) : **celui du socle S2**, non un exécuteur propre à Stripe.
2. **AC-2 — Rythme et reprise : ceux du socle S2, sans variante propre à Stripe.** Liste des versements (pages de 100), puis
   transactions de chaque versement ; par défaut une requête à la fois, **sans pause fixe** (la doc de Stripe annonce 25
   requêtes par seconde par point d'accès et 100 au total en mode réel, 25 en bac à sable : une requête séquentielle dure bien
   plus de 40 ms, la reprise sur 429 suffit) ; avec `KESH_STRIPE_PARALLEL = k` (1 ≤ k ≤ 6 ; autre valeur → refus de démarrer,
   lecture du socle S1), au plus `k` versements en cours ; la pagination d'un même versement reste séquentielle (curseur).
   **Reprise** : 429 (avec `Stripe-Rate-Limited-Reason`, ou `lock_timeout`) et 5xx → la politique **commune** (`Retry-After` entier
   borné à 120 s s'il est donné, sinon 5, 10, 20, 40, 60 s ; six essais par requête ; pause commune à tous les fils, **sans
   ramener le parallélisme à 1** ; la version initiale avait une attente exponentielle à gigue propre, abandonnée : un seul
   client, pas de troupeau de clients à désynchroniser) ; au-delà → tâche `failed`, `PROVIDER_RATE_LIMITED`, **rien d'écrit**
   (l'import n'a lieu qu'une fois tout lu). 401/403 : arrêt immédiat, code de 40-8a AC-4.
3. **AC-3 — Même résultat que le CSV.** Les lignes lues (40-8a AC-3) vont au chemin d'import commun (40-7b AC-7, socle S6) en
   **une** transaction de base, une entrée `bank_imports` par devise, `filename` = « API Stripe <devise> <from>–<to> » (comme
   Wise ; l'empreinte est par devise), puis au rapprochement de 40-7c. **Empreinte** = SHA-256
   de `« stripe-api:<devise> » ‖ identifiants triés` ; **le refus `BANK_IMPORT_DUPLICATE_FILE` ne s'applique pas** à un
   téléchargement (la version initiale refusait un second téléchargement identique « comme le CSV », ce qui contredit PayPal et
   Wise et fait échouer, après lecture complète, une relance sans risque) : relancer la même période rend « 0 insérée » ; un
   téléchargement qui chevauche un CSV n'insère que les transactions neuves. Test : la fixture CSV de 40-7a importée, puis le
   téléchargement simulé des mêmes versements → zéro insérée ; l'inverse aussi ; deux téléchargements de suite → zéro insérée.
4. **AC-4 — Écran et manuels.** *Écran* : le **dialogue générique de 38-12b** (S7), sur la page *Import bancaire*, paramétré
   par Stripe (route `stripe-downloads`, libellés « versement ») ; le bouton *Télécharger depuis Stripe* se montre quand
   **`providerApi.configured`** (champ générique du compte bancaire ; `providerApi.mode` : `live` ou `test`, **montré** dans le
   dialogue ; jamais la clé). *Manuel d'administration* (`docs/manual/fr/admin-manual.tex`) : créer une **clé restreinte** au
   tableau de bord Stripe, droits *Read* seuls sur les versements et le solde (noms exacts **à relever**, 40-8a), variables
   `KESH_STRIPE_<n>_ACCOUNT` (libellé **propre à l'installation** : deux sociétés au même libellé atteignent le même compte
   Stripe), `KESH_STRIPE_<n>_KEY`, `KESH_STRIPE_BASE_URL`, `KESH_STRIPE_PARALLEL` dans **un tableau neuf du § `sec:env-vars`** (modèle `:840-862`) **et** le tableau de la section Stripe, jamais le tableau SMTP (`:1155`),, comment aller **au-delà de trois
   comptes** (lignes à ajouter aux compose), la clé ni en base ni en sauvegarde (à
   reposer après une restauration sur un autre serveur), révocation ; que faire d'un `PROVIDER_API_ERROR` (vérifier d'abord les
   droits de la clé). *Manuel utilisateur* : le bouton, la période, les délais, les dates de Zurich. `.tex` seulement.
   `docs/api-external.md` : la route (sessions seules, 403 `API_KEY_OPERATION_FORBIDDEN`) et ses codes. **E2E** `frontend/tests/e2e/stripe-download.spec.ts`
   contre le **serveur de prestataire simulé de 38-12b** (S8, `KESH_E2E_PROVIDER_MOCK_PORT`, backend d'E2E démarré avec
   `KESH_STRIPE_1_*` et `KESH_STRIPE_BASE_URL` vers lui ; plus de route `_test` ni de `KESH_TEST_MODE` propres à Stripe) : sans
   variables, bouton absent ; période → progression → compte rendu ; relance → « 0 insérée » ; sans la variable du port,
   ignoré avec son motif écrit. Captures hors du dépôt.

## Tâches

- [ ] T1 — Route, état, tâche sur le socle S2/S3 (AC-1).
- [ ] T2 — Boucle d'étapes par versement, parallélisme (AC-2).
- [ ] T3 — Raccord au chemin d'import et au rapprochement, empreinte (AC-3).
- [ ] T4 — Paramétrage du dialogue, manuels, E2E, captures (AC-4).

## Stratégie de tests

Unitaires de la boucle sur horloge et attente injectables (pas de sommeil réel) ; HTTP `#[sqlx::test]` contre le serveur
Stripe simulé de 40-8a : succès séquentiel, `k = 3` égal à `k = 1`, 429 repris, 429 au-delà du plafond (rien d'écrit), 403,
chevauchement CSV, double téléchargement, clé d'API → 403 `API_KEY_OPERATION_FORBIDDEN` sur le `POST` et le `GET`, deux
tâches du même compte → 409 avec `details.jobId`, versement dont Σ net ≠ montant → `failed`, rien d'écrit ; registre et
phrase du manuel au compte (`binary(audit_route_registry)`) ; E2E du bouton. Gate complet au
push.

## Modules touchés (4)

`kesh-api` (route, boucle, raccord), `frontend` (paramétrage du dialogue), `kesh-i18n`, `docs/manual/fr`.

## Fichiers communs

`crates/kesh-api/src/lib.rs`, `errors.rs`, catalogues i18n, `frontend/src/routes/(app)/bank-accounts/` ou la page d'import,
manuels `.tex`, `docs/api-external.md`, `docs/testing.md` : à relire à la reprise de `main`.

## Note de version

*Ajouté* — Téléchargement des versements Stripe d'une période par l'API (bouton ou route), séquentiel avec reprise sur
limite de débit, parallélisme réglable (`KESH_STRIPE_PARALLEL`, 1 à 6) ; même résultat que l'import du rapport CSV.

## Notes de développement

- Limites lues le 2026-11-20 (docs.stripe.com/rate-limits) : 100 requêtes/s global en mode réel, 25 en bac à sable, 25
  par point d'accès sauf mention ; 429 avec `Stripe-Rate-Limited-Reason` (`global-rate`, `endpoint-rate`,
  `global-concurrency`, `endpoint-concurrency`, `resource-specific`) ; 50 requêtes simultanées au plus par défaut ;
  conseil : attente exponentielle avec gigue ; allocation de lecture : 500 lectures par transaction sur 30 jours,
  au moins 10 000 par mois — un exercice de Beat (≈ 100 versements, ≈ 340 transactions de solde) en demande quelques
  centaines.
- **Non confirmé** : délai entre un versement payé et sa présence dans la liste (PayPal a ~3 h ; Stripe : inconnu,
  écrit « peut tarder » au manuel).

## Change Log

- 2026-11-20 — fiche rédigée (phase 2), non validée.
- 2026-11-20 — validation P1 (Claude Sonnet 5.5, contexte frais) : voir 40-8a ; pour 40-8b, MEDIUM — routes, codes (`STRIPE_*`)
  et forme d'état (`requests`, `retries`) différents de ceux de PayPal et Wise, clés d'API admises, exécuteur « propre ou
  repris » non tranché, politique de reprise exponentielle à gigue et parallélisme ramené à 1 propres à Stripe, refus de
  l'empreinte sur un second téléchargement identique, E2E par route `_test` propre à Stripe ; LOW — pause fixe de 100 ms,
  plafond de 366 jours, mode `live`/`test` non rattaché à `providerApi`.
- 2026-11-20 — validation P2 (Claude Opus 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/40-8-P2.md`) :
  ici, compte du registre et phrase du manuel relevés dans cette story, clé d'API refusée en 403
  `API_KEY_OPERATION_FORBIDDEN` sur les deux routes (comme 38-12a), 409 avec `details: { provider, jobId }`, `details` seul
  nom, une entrée `bank_imports` et un `filename` par devise.
- 2026-11-20 — validation P3 ciblée (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-40-phase2-P3-ciblee.md`) : registre (deux phrases du manuel, six `assert_eq!`), tableaux de variables (§ `sec:env-vars` neuf, jamais SMTP), panique de tâche → `INTERNAL_ERROR`.
- 2026-11-20 — validation P4 ciblée (Claude Haiku 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-40-phase2-P4-ciblee.md`) : registre : six `assert_eq!` dont cinq littéraux, les deux phrases du manuel ; garde : noms sans numéro `KESH_STRIPE_BASE_URL` et `KESH_STRIPE_PARALLEL` inscrits ; `payout_total_mismatch` (code exact de 40-7c).
- 2026-11-20 — validation P5 ciblée (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-40-phase2-P5-ciblee.md`) : 0 C, 0 H, 0 M, 1 L, remédié : entrée P4 limitée à ce qui concerne Stripe (renvoi à la Q-39-16 de Wise retiré), casse des noms.
- 2026-11-20 — validation P2, complément (Claude Opus 5.5)
 : cœur `ProviderLine` extrait par 38-12a (ou 40-7b), appelé ici ;
  manuel : deux tableaux de variables, au-delà de trois comptes.
- 2026-11-22 — amendement du 22.11 (avant le développement) contre le code d'`epic/40-stripe` après la fusion de la
  40-8a : dialogue générique et serveur simulé de la 38-12b absents de la branche (développement suspendu jusqu'à sa
  fusion), signature de `Downloads::start` changée par la 38-12b, comptes du registre relevés (162, 154, 166), LOW L5 et
  L6 de la revue P1 de la 40-8a versés ici, forme de `fetch_rows`. Statut inchangé (`ready-for-dev`).
