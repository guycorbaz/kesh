# Epic 37 — couverture API et MCP (audit du 2026-11-14)

Audit en lecture seule (Opus) des six fiches 37-* contre le code : chaque fonction de l'epic a-t-elle une route HTTP,
est-elle faisable **par l'API seule** (le responsable de l'epic importera son exercice 2024 par l'API, sans écran), et les outils du serveur
MCP (lecture seule, Epic 32) touchés par l'epic sont-ils étendus ? Routes et outils cités par les fiches : tous vérifiés
au code (une trentaine de lignes) ; un écart mineur : 37-3a AC-8 cite `ecritures.rs:126` pour la struct `Ligne`, qui
commence à `:122`.

## Résultat

Toutes les fonctions ont une route. Les trous sont des **calculs laissés au client Svelte** et des **contrats non
écrits**. Aucun HIGH.

| # | Sév. | Où | Trou | Correctif retenu (CR-6) |
|---|---|---|---|---|
| T1 | MEDIUM | 37-3a AC-6 ; 37-3b AC-2 | « Cours ou CHF » (cours = CHF ÷ montant) et qualification `afc`/`manual` faits par l'écran seulement | V1 |
| T2 | LOW | 37-3a AC-3, AC-6 | Débit/crédit CHF d'une ligne en devise calculés par le client | V2 |
| T3 | MEDIUM | 37-3b AC-2 bis (conversion) | Ligne de différence calculée par le client, comptes de gains/pertes non contrôlés | V3 |
| T4 | LOW | 37-3b (ligne d'écart R8) | Comptes de la ligne d'écart non dits aux clients de l'API | V4 |
| T5 | LOW | 37-3b (assistant TVA) | Calcul TVA en devise à l'écran seulement (déjà le cas en CHF) | V5 |
| T6 | MEDIUM | 37-1 AC-3 ; epic | Activation écrite « à l'écran » ; elle se fait par HTTP en session Bearer, jamais par clé d'API | V6 |
| T7 | MEDIUM | epic ; 37-5 | Aucun test de bout en bout **par l'API** de la recette 2024 | V7 |
| T8 | MEDIUM | 37-3a AC-6 ; 37-5 | Les réévaluations des livres d'origine ne s'importent pas telles quelles (source `revaluation` refusée) ; rien ne le dit | V8 |
| T9 | LOW | 37-3b AC-1 | Cours d'ouverture proposé calculé en TypeScript, alors que le serveur l'expose (case `opening`) | V9 |
| T10 | LOW | 37-1 l.102 | `kesh_exercices` non étendu, justification circulaire | V10 |
| T11 | LOW | 37-1 AC-8 | Réglage de la société exposé par aucun outil MCP, non dit | V11 |
| T12 | LOW | 37-5 | Aucun mot sur le MCP | V12 |
| T13 | LOW | 37-4 AC-7 | `kesh_postes_ouverts` touché (compte en devise lettrable), absent de l'inventaire « CHF seulement » | V13 |

## Correctifs retenus (orchestrateur, 2026-11-14)

- **V1** — 37-3a : une ligne en devise accepte `foreign { amount, chf }` **sans** `rate` : le serveur dérive le cours
  par la **même** fonction publique de `kesh-core` que l'écran (fixture partagée). `rateSource` absent : le serveur
  qualifie `afc` si le cours égale le cours retenu `afc` de la société pour le mois, `manual` sinon (A30 compris).
  `rate` **et** `chf` fournis : contrôle actuel inchangé. 37-3b l'emploie ou garde son calcul, à condition d'utiliser la
  même fonction (DRY).
- **V2** — 37-3a : `debit`/`credit` facultatifs sur une ligne en devise ; absents, le serveur les pose depuis
  `foreign` et le sens (`side`) ; présents, contrôle actuel (`FX_AMOUNT_MISMATCH`).
- **V3** — 37-3b, route de conversion : si le corps ne porte pas de ligne de différence, le serveur la calcule et
  l'ajoute (`fx_accounts_for` puis `validate_fx_accounts_in_tx`) ; s'il en porte une, il vérifie qu'elle emploie la
  paire résolue, sinon 400.
- **V4** — 37-3a AC-6 : phrase pour les clients de l'API : la ligne d'écart emploie les comptes propres du compte en
  devise, sinon ceux de la société (lisibles sur `AccountResponse` et `CompanyJson`).
- **V5** — 37-3b : limite écrite : l'assistant TVA reste un calcul de l'écran ; règle d'arrondi donnée pour un client
  de l'API.
- **V6** — 37-1 AC-3 et epic : « s'active par une session (`POST /auth/login`, Bearer), jamais par une clé d'API » ;
  test d'API en Bearer.
- **V7** — 37-5 : test d'intégration HTTP de bout en bout (Bearer) : activation, comptes en devise, exercice,
  `POST /exchange-rates/fetch` (source de test), ouverture, écritures (dont V1/V2 et une conversion V3), réévaluation
  avec `rates`, rapports.
- **V8** — 37-5 et epic : à l'import de livres tenus ailleurs, la réévaluation se repasse par
  `POST /fiscal-years/{id}/fx-revaluation` avec `rates` (p. ex. USD 0.91193, EUR 0.94947 au 31.12.2024), non par des
  écritures `revaluation`.
- **V9** — 37-3b AC-1 : renvoi des clients de l'API à la case `opening` de `GET /fiscal-years/{id}/exchange-rates`.
- **V10** — 37-1 AC-8 : `kesh_exercices` gagne `multi_devises`, `genre_ouverture`, `genre_cloture` (test, mesure de
  taille).
- **V11** — 37-1 AC-8 : « aucun outil n'expose la société (existant) : hors périmètre ».
- **V12** — 37-5 : « MCP : aucun outil pour l'aperçu (calcul à la demande) ; les écritures passées se lisent par
  `kesh_ecritures` (source `revaluation`, 37-3a AC-8) ».
- **V13** — 37-4 AC-7 : `kesh_postes_ouverts` ajouté aux sites « CHF seulement », renvoi à 37-3a AC-9.
