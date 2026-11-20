# Story 40-8b : téléchargement Stripe par période — route, tâche, écran, manuels

Status: ready-for-dev

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, chemins relevés sur `origin/main` au commit `fd04099f`. Epic 40,
**phase 2** (décision de Beat du 2026-11-20). **Dépend de** : 40-8a (client), 40-7b (import CSV), du **cœur commun `ProviderLine`** (`provider_imports.rs`) extrait par 38-12a — ou par 40-7b s'il passe
avant (socle S6) ; 40-8b l'appelle sans l'extraire, 40-7c
(rapprochement) et du **socle S1 à S9** (fichier d'epic), écrit par 38-12a ou par la première fiche de phase 2 développée
(C-40-K6) ; le dialogue d'écran est celui de 38-12b (S7). L'automatisme (téléchargement périodique) vient plus tard encore,
story à part.*

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
