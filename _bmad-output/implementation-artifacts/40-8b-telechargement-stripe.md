# Story 40-8b : téléchargement Stripe par période — séquentiel, reprise sur 429, parallélisme en option

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, chemins relevés sur `origin/main` au commit `fd04099f`. Epic 40,
**phase 2** (décision de Beat du 2026-11-20). **Dépend de** : 40-8a (client), 40-7b (chemin d'import commun), 40-7c
(rapprochement) ; la phase 2 de l'Epic 38 (téléchargement PayPal) si elle est faite avant, dont elle reprend
l'exécuteur de téléchargement (C-40-K6). L'automatisme (téléchargement périodique) vient plus tard encore, story à part.*

## Story

En tant que **Comptable**, je veux télécharger d'un bouton (ou par une route d'API) les versements Stripe d'une période
choisie, voir la progression, et retrouver ensuite exactement ce que l'import du rapport CSV aurait donné — rattachements,
frais, versements, écarts —, sans doublon si j'ai déjà importé le CSV.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-40-8b-1` | Route et bouton : compte Stripe, période ; un seul téléchargement à la fois par compte. | AC-1 |
| `FR-40-8b-2` | Séquentiel par défaut, avec une pause entre deux appels et une reprise sur 429 (attente croissante, au plus N reprises) ; progression « versement 5/12 ». | AC-2, AC-3 |
| `FR-40-8b-3` | Parallélisme en option : `KESH_STRIPE_PARALLEL`, défaut 1, maximum 6. | AC-2 |
| `FR-40-8b-4` | Les lignes passent par le chemin d'import commun puis par le rapprochement de 40-7c ; CSV et téléchargement se mélangent. | AC-4 |
| `FR-40-8b-5` | Manuel d'administration : créer la clé restreinte, ses droits, les variables ; manuel utilisateur : le bouton. | AC-5 |

## Critères d'acceptation

1. **AC-1 — Route.** `POST /api/v1/stripe-downloads` (`{ account, from, to }`, Comptable et plus, clés `read-write`,
   registre des routes auditées) : 404 `STRIPE_ACCOUNT_NOT_FOUND` ; 409 `STRIPE_KEY_NOT_CONFIGURED` si aucune paire
   d'environnement ne porte ce libellé (40-8a AC-1) ; 400 `VALIDATION_ERROR` si `from > to` ou période de plus de
   366 jours ; 409 `STRIPE_DOWNLOAD_RUNNING` si un téléchargement du même compte tourne. Réponse `202 { downloadId }` ;
   `GET /api/v1/stripe-downloads/{id}` rend `{ state: running | done | failed, payoutsDone, payoutsTotal, requests,
   retries, result? }`, `result` = compte rendu de 40-7b AC-6 suivi de celui de 40-7c AC-8. État en mémoire du
   processus (perdu au redémarrage : limite écrite au manuel), comme l'exécuteur de la phase 2 de PayPal s'il existe —
   **un seul** exécuteur de téléchargement pour les prestataires (C-40-K6), paramétré par la source.
2. **AC-2 — Rythme.** Liste des versements de la période (séquentielle, pages de 100), puis transactions de chaque
   versement : par défaut une à la fois, pause de 100 ms entre deux appels (constante nommée ; la doc de Stripe annonce
   25 requêtes par seconde par point d'accès et 100 au total en mode réel, 25 en bac à sable) ; avec
   `KESH_STRIPE_PARALLEL = k` (1 ≤ k ≤ 6, autre valeur → refus de démarrer), au plus `k` versements en cours. La
   pagination d'un même versement reste séquentielle (curseur).
3. **AC-3 — Reprise.** Sur 429 (avec `Stripe-Rate-Limited-Reason`, ou `lock_timeout`), attente exponentielle avec
   gigue (base 1 s, plafond 30 s, au plus 6 reprises par appel), le parallélisme ramené à 1 jusqu'à la fin ; au-delà →
   `failed`, code `STRIPE_RATE_LIMITED`, **rien d'écrit** (l'import n'a lieu qu'une fois tout lu). 5xx : même reprise,
   trois fois. 401/403 : arrêt immédiat, code de 40-8a AC-4.
4. **AC-4 — Même résultat que le CSV.** Les lignes lues (40-8a AC-3) vont au chemin d'import commun (40-7b AC-7) en une
   transaction de base, `filename` = « API Stripe <from>–<to> », empreinte = SHA-256 de `compte ‖ from ‖ to ‖ devise ‖
   ids triés` (un second téléchargement identique est un doublon de fichier, refusé comme le CSV ; un téléchargement
   qui chevauche un CSV n'insère que les transactions neuves) ; puis le rapprochement de 40-7c. Test : la fixture CSV
   de 40-7a importée, puis le téléchargement simulé des mêmes versements → zéro insérée ; l'inverse aussi.
5. **AC-5 — Écran et manuels.** Page *Import bancaire* : bouton *Télécharger depuis Stripe* (visible si la clé du
   compte est configurée — l'API dit seulement « configurée / non configurée » et le mode `live`/`test`, jamais la
   clé), période, barre de progression (« versement 5/12 », reprises), compte rendu identique à l'import. Manuel
   d'administration (`docs/manual/fr/admin-manual.tex`) : créer une **clé restreinte** au tableau de bord Stripe, droits
   *Read* seuls sur les versements et le solde (noms exacts à relever, 40-8a), variables `KESH_STRIPE_<n>_ACCOUNT`,
   `KESH_STRIPE_<n>_KEY`, `KESH_STRIPE_PARALLEL`, la clé ni en base ni en sauvegarde (à reposer après une restauration
   sur un autre serveur), révocation. Manuel utilisateur : le bouton, la période, les délais. `.tex` seulement. E2E sur
   la source simulée (route `_test` ou `KESH_TEST_MODE`, sur le patron des cours de l'AFC simulés), captures.

## Tâches

- [ ] T1 — Exécuteur de téléchargement (repris de PayPal ou créé générique), route, état (AC-1).
- [ ] T2 — Rythme, parallélisme, reprise (AC-2, AC-3).
- [ ] T3 — Raccord au chemin d'import et au rapprochement, empreinte (AC-4).
- [ ] T4 — Écran, manuels, E2E, captures (AC-5).

## Stratégie de tests

Unitaires du rythme et de la reprise sur une horloge simulée (pas de sommeil réel) ; HTTP `#[sqlx::test]` avec
`MockStripeSource` : succès séquentiel, `k = 3`, 429 repris, 429 au-delà du plafond (rien d'écrit), 403, chevauchement
CSV ; E2E du bouton. Gate complet au push.

## Modules touchés (4)

`kesh-api` (route, exécuteur, raccord), `frontend` (bouton, progression), `kesh-i18n`, `docs/manual/fr`.

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
