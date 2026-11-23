# Story 40-8b : téléchargement Stripe par période — route, tâche, écran, manuels

Status: done

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, chemins relevés sur `origin/main` au commit `fd04099f`. Epic 40,
**phase 2** (décision du responsable de l'epic du 2026-11-20). **Dépend de** : 40-8a (client), 40-7b (import CSV), du **cœur commun `ProviderLine`** (`provider_imports.rs`) extrait par 38-12a — ou par 40-7b s'il passe
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
   (« versement manuel à examiner »), non à l'import. Le mode de la clé est `StripeClient::mode()` (`live`/`test`) *(annoté le 22.11 : l'écran le lit en `live`/`sandbox`, second amendement, point 7)*.

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
11. *(Ajouté le 22.11, revue d'intégration de l'Epic 40 : M1, L1, L2.)* **M1** — avant la 40-8b, `providerApi`
    d'un compte Stripe à série rendait `configured: true` sans route et `mode: live` pour une clé d'essai (mode lu
    à l'hôte). Désormais la route existe, un test (`chaque_prestataire_lu_a_sa_route_de_telechargement`) exige
    les deux routes de chaque prestataire de `ALL_PROVIDERS` dans `lib.rs`, la branche de repli est de nouveau
    exercée (`wise`), et le mode suit le point 7 ; la fiche 40-8a est annotée. **L1** — le commentaire du filtre
    de l'écran dit l'écart (sans accents côté écran, sans casse seulement côté serveur) *(annoté le 22.11, à la
    fusion d'`epic/38-paypal` avec la 38-13 : écart fermé, `series_for` replie aussi les accents
    (`account_label_key`) ; le commentaire de `providerApiStatus` le dit)*. **L2** — `paypalApiStatus`
    et `ownLedgerIds` de la page PayPal passent par `providerSettingsOf` (`providerApiStatus` générique).
12. **Écran** (AC-4) : le bouton va sur la **page Stripe** (`/reconciliation/stripe`, page commune
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
   **plus de plafond de 366 jours** : la pagination par curseur borne chaque appel, la période est celle que choisit le comptable) → 409
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
   **`providerApi.configured`** (champ générique du compte bancaire ; `providerApi.mode` : `live` ou `sandbox` — *corrigé le 22.11 (second amendement, point 7 ; revue d'intégration de l'Epic 40, M1) : la fiche disait `live` ou `test`* —, **montré** dans le
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

- [x] T1 — Route, état, tâche sur le socle S2/S3 (AC-1).
- [x] T2 — Boucle d'étapes par versement, parallélisme (AC-2).
- [x] T3 — Raccord au chemin d'import et au rapprochement, empreinte (AC-3).
- [x] T4 — Paramétrage du dialogue, manuels, E2E, captures (AC-4).

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

*Ajouté* — Téléchargement des versements Stripe d'une période par l'API : bouton *Télécharger depuis Stripe* de la page
Stripe (Comptable et Administrateur, sessions seules ; `POST /api/v1/stripe-downloads`, `GET
/api/v1/stripe-downloads/{jobId}`), une clé restreinte en lecture par compte Stripe (`KESH_STRIPE_<n>_ACCOUNT`,
`KESH_STRIPE_<n>_KEY`), progression par versement, reprise commune sur limite de débit, parallélisme réglable
(`KESH_STRIPE_PARALLEL`, 1 à 6) ; même résultat et mêmes doublons que l'import du rapport CSV, versements manuels signalés
« à examiner ». La liste des comptes bancaires dit le mode d'un compte Stripe par sa clé (`providerApi.mode` : `live`
ou `sandbox`). Le refus `400 PROVIDER_PERIOD_INVALID` (aussi celui du téléchargement PayPal) porte en plus
`details.historyYears` (3 pour PayPal, 30 pour Stripe) : ajout additif, aucun champ retiré.

## Notes de développement

- Limites lues le 2026-11-20 (docs.stripe.com/rate-limits) : 100 requêtes/s global en mode réel, 25 en bac à sable, 25
  par point d'accès sauf mention ; 429 avec `Stripe-Rate-Limited-Reason` (`global-rate`, `endpoint-rate`,
  `global-concurrency`, `endpoint-concurrency`, `resource-specific`) ; 50 requêtes simultanées au plus par défaut ;
  conseil : attente exponentielle avec gigue ; allocation de lecture : 500 lectures par transaction sur 30 jours,
  au moins 10 000 par mois — un exercice du responsable de l'epic (≈ 100 versements, ≈ 340 transactions de solde) en demande quelques
  centaines.
- **Non confirmé** : délai entre un versement payé et sa présence dans la liste (PayPal a ~3 h ; Stripe : inconnu,
  écrit « peut tarder » au manuel).

## Dev Agent Record

*Rédigé le 2026-11-22 au dernier commit de code `ca8590a50` (branche `story/40-8b`). Périmètre des décomptes :
`d97712397..ca8590a50` (après la fusion d'`epic/40-stripe`), recomptés à la source.*

### Étape 0

Fusion **non squash** d'`epic/40-stripe` (`15e9f8b13`) dans `story/40-8b` : `d97712397`, sans conflit. Recontrôle de la
fiche contre le code : second amendement du 22.11 (`d2769651a`), points 1 à 11 ; le point 11 ajouté en cours de
développement sur demande de l'orchestrateur (revue d'intégration de l'Epic 40, M1, L1, L2).

### Ce qui est fait

- **Serveur** : `routes/stripe_downloads.rs` (neuf : `create`, `show`, tâche `Run::execute`) sur le socle commun ;
  `stripe::fetch_rows_tracked` (crochet de progression, `fetch_rows` inchangé), `stripe::HISTORY_YEARS` (30),
  `stripe::series_mode` ; `JobHandle::set_total` au socle ; `ProviderApiConfig::status_for` lit le mode de Stripe à la
  clé ; `stripe_imports::groups_by_currency` (ventilation partagée par le CSV et le téléchargement) ; routes dans
  `lib.rs` ; action `stripe_download.created` (cœur commun, `audit_labels`, inventaire des sites indirects, quatre
  locales).
- **Écran** : bouton, mention « bac à sable », ligne d'aide de l'Administrateur et dialogue générique de la 38-12b sur la
  page Stripe ; compte rendu extrait en `StripeImportReport.svelte` (page et dialogue, versements manuels) ;
  `providerApiStatus` générique sur `providerSettingsOf` (`paypalApiStatus`, `stripeApiStatus`), `ownLedgerIds` de la page
  PayPal de même (L2) ; commentaire de l'écart des libellés corrigé (L1) ; sept clés `stripe-download-*` aux quatre
  locales.
- **Documentation** : manuel d'administration (§ `sec:stripe-api` neuf, tableau du § `sec:env-vars`, accès sortant,
  compte du registre 155/163), manuel utilisateur (§ `sec:stripe-telecharger`, limites), `docs/api-external.md` (section,
  §4, tableaux §10), `docs/testing.md` (montage Stripe du serveur simulé), `.env.example` et les deux compose (L6).
  `.tex` seulement ; PDF compilés dans une copie et contrôlés (aplatis).

### Tests ajoutés (recomptés sur `d97712397..ca8590a50`)

- `crates/kesh-api/tests/stripe_download_e2e.rs` : **11** `#[sqlx::test]` contre un serveur Stripe simulé (fixtures de
  la 40-8a, filtre `arrival_date`, pages de deux) : ventilation et relance sans doublon, CSV puis téléchargement et
  l'inverse, `k = 3` égal à `k = 1`, progression, 429 repris puis épuisé (rien d'écrit), 401/403, total de versement faux,
  identifiant de 300 caractères (L5), refus dans l'ordre et 409, liste des comptes (mode `sandbox`).
- `provider_env.rs` : **2** `#[test]` neufs (`mode_d_un_compte_stripe_par_sa_cle`,
  `chaque_prestataire_lu_a_sa_route_de_telechargement`) et la branche de repli de `status_for` de nouveau exercée
  (`wise`, `inconnu`) ; `provider_download.rs` : `set_total` ajouté au test existant.
- Vitest : **6** `it(` neufs (2 dans `provider-api.test.ts`, 4 dans `stripe-page.test.ts`).
- E2E : `frontend/tests/e2e/stripe-download.spec.ts`, **3** tests ; routes `stripeMockRoutes` au serveur simulé commun.
- Compteurs relevés : registre des routes 163 / 155 / 167, partition `SansEcritureAuJournal` 132, inventaire
  `tout.len()` 167 ; `COMPTE_CHAMPS` des dates reçues 74 ; `sitesTotal` 3740.

### Mutations (hors dépôt, une par invocation de `heavy.sh`, cible propre, « tuée » sur assertion nommée)

Rust (copie `git archive`, cible `kesh-targets/s40-8b-mut`, supprimée) : témoin non muté **vert** ; `check_id_lengths`
retiré → tuée par `identifiant_trop_long_refuse_avant_l_ecriture` ; `set_total` retiré → tuée par
`progression_par_versement` ; `rk_test_` → `Live` → tuée par `mode_d_un_compte_stripe_par_sa_cle`. Journaux
`kesh-gate-logs/s40-8b/mut-*.log`. Vitest (copie de `frontend/` avec son propre `node_modules`, `npm ci`, supprimée) : témoin **vert** ; bouton ouvert à tout rôle → tuée par « Consultation : jamais le bouton, même configuré » ; `stripeApiStatus` sur `paypal` → tuée par « lit les soldes Stripe du libellé… » ; versements manuels masqués → tuée par « compte configuré, Comptable… » (`mut-front-all.console`, `mut-f*.log`).

### Gates (règle de la nuit du 22.11 : pas de `gate-ci.sh` ni d'E2E complet par story)

- `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings` : verts à `ca8590a50`
  (`cible-5.console`).
- `scripts/test-fast.sh --ci` (nextest complet, base remise à zéro) à `ce915c575` : **6548 exécutés, 6547 passés,
  1 échec, 15 ignorés** — l'échec, `dates_recues_garde::l_inventaire_n_est_pas_vide` (72 → 74, le `DownloadRequest` de
  la story), corrigé au commit de test seul `ca8590a50`, puis `binary(dates_recues_garde)` rejoué vert (10/10)
  (`gate-backend-1.log`, `cible-5.log`).
- Frontend à `ce915c575` : `npm run check` 0 erreur, `lint-i18n-ownership` vert, Vitest **5072 / 5072** (296
  fichiers), `npm run build` vert (`front-3-*.log`).
- **E2E partiel** (la seule spec ajoutée par la story) : `stripe-download.spec.ts` **3 / 3** à `ce915c575`, backend
  avec `KESH_STRIPE_1_*` vers le serveur simulé (`e2e-2.log`). E2E complet et `gate-ci.sh` : à l'intégration de fin.
- Captures (thème clair) : `kesh-gate-logs/s40-8b/captures/` (10 à 16).

### Écarts et points relevés

- Écarts à la fiche : ceux du second amendement (période commune avec fin ramenée, `sandbox` au lieu de `test`, nom
  d'import unique, une fenêtre, `n` étapes, écran sur la page Stripe).
- Station : la place 5 de `heavy.sh` (serveur `kesh-mariadb-dev7`, arrêté) n'était plus tenue par son `flock` ; deux
  lancements y sont tombés et ont échoué sur un conteneur arrêté (`e2e-1.console`). Refermée comme le prévoit la
  consigne (`flock /tmp/station-heavy-b5.lock sleep 14400`), sans toucher à `heavy.sh` ni au conteneur.
- Non traité ici : L3 et L5 de la revue d'intégration de l'Epic 40 (compte rendu CSV resté affiché après un
  téléchargement ; deux « aujourd'hui » sur la page) et l'écart de pliage des accents entre l'écran et le serveur (L1,
  issue déjà notée).

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
- 2026-11-22 — second amendement (étape 0, après la fusion d'`epic/40-stripe`) ; développement (route, tâche, écran,
  manuels, tests), M1, L1, L2 de la revue d'intégration de l'Epic 40 intégrés ; gates de la nuit ; story en `review`.
- 2026-11-22 — revue de code P1 (Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/s40-8b/revue-p1-rapport.md`) : 0 C, 0 H,
  1 M, 3 L. Remédiés sans toucher la production, en un commit `test(40-8b)` : M1 (`KESH_STRIPE_PARALLEL` non tenu par un
  test : `versements_lus_en_meme_temps_egal_le_reglage`, serveur simulé qui compte les requêtes de transactions en vol,
  réglage 2 sur 3 versements), L1 (phrase de `docs/api-external.md` sur `too_old` de Stripe), L2 (le test de routes de
  `provider_env` ignore les lignes commentées de `lib.rs`), L3 (GET d'une tâche d'une autre société : 404, dans
  `refus_tache_unique_et_cles_d_api`). Un test neuf ; gates : fmt, clippy workspace, nextest ciblé 43/43 verts (journaux
  `p2-*.log`). Sondes : voir la P2.
- 2026-11-22 — remédiation du M1 de la revue P2 (`kesh-gate-logs/s40-8b/revue-p2-rapport.md` ; l'écran Stripe passe bien par
  `ProviderDownloadDialog` → `startRefusal` → `periodRefusalLabel`, vérifié) : les textes `too_old` disaient « trois ans » pour
  Stripe, dont la borne est de trente. Le refus `400 PROVIDER_PERIOD_INVALID` porte désormais `details.historyYears` (3
  PayPal, 30 Stripe) ; le texte `provider-download-period-too-old` (quatre locales) l'interpole (`{ $years }`), sans
  profondeur lisible l'écran retombe sur le texte général, qui ne cite plus aucun nombre (idem `provider-download-period-invalid`
  et `error-provider-period-invalid`, texte de l'API). N1 de la P2 : le test de parallélisme est renommé
  `requetes_http_en_vol_egal_le_reglage` (il prouve la borne HTTP, tenue par le `ProviderGate`). Tests : assertions
  `historyYears` (3 / 30) dans les deux E2E d'API, Vitest PayPal 3 ans / Stripe 30 ans / sans profondeur. Manuel
  utilisateur et `docs/api-external.md` à jour. Passe P3 ciblée : `kesh-gate-logs/s40-8b/revue-p3-prompt.md`.
  Gates : fmt, clippy workspace, nextest ciblé 38/38 verts (`binary(stripe_download_e2e) | binary(paypal_download_e2e) | test(provider_env)`, journal `cible-6.log`) ; frontend : check, lint-i18n-ownership, Vitest 5073 (296 fichiers), build verts, `sitesTotal` inchangé (aucun site neuf) ; E2E `stripe-download` et `paypal-download` verts (`e2e-6.log`).
- 2026-11-22 — **clôture de la boucle de revue.** Tendance : P1 (Claude Sonnet 5.5, complète) 0 C / 0 H / 1 M / 3 L → P2
  (Claude Haiku, ciblée sur la remédiation de la P1) 0 / 0 / 1 M / 2 L → P3 (Claude Opus 5.5, ciblée sur la remédiation du M1
  de la P2, `kesh-gate-logs/s40-8b/revue-p3-rapport.md`) 0 / 0 / 0 / 3 L : aucune remédiation de la P3 ne touche la production.
  L1 de la P3 : test Vitest de la garde de type de `periodRefusalLabel` (2.5, −1, 0, `'30'`, `null`, `NaN` → texte général,
  sans chiffre) ; la mutation F3 (garde remplacée par `true`), rejouée sur une copie hors dépôt avec un vrai `node_modules`,
  est tuée sur le test nommé « une profondeur illisible » (`mut-f3-garde-type.log`). L2 : la note de version mentionne
  `details.historyYears` (ajout additif). L3 : les gates précédaient le commit de la P3 ; rejoués au commit final de la story
  (journaux `final-*` de `kesh-gate-logs/s40-8b/`). Story `done`.
