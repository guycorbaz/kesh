# Story 38-9a : écrans — connexions de prestataire, secrets, soldes proposés, comptes prestataires, contrôles de solde ; manuel d'administration

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 (Opus), branche `epic/38-paypal`. Epic 38 (`planning-artifacts/epic-38-paypal.md`),
E1, E2, E3, E10, E11, E12, E19, E20, E24 ; contrats C-38-1, C-38-2, C-38-3, C-38-5, C-38-12. Décisions B1, B2, B3,
B8, B9, B11, B12, B22 ; arbitrages O1, O10, O12. **Dépend de** 38-1 (routes des connexions et des secrets), 38-2
(comptes prestataires, bloc `provider` de `GET /bank-accounts` ; **porte la migration breaking**, O1), 38-3a (colonnes
de `provider_fetch_runs` dont `truncated_from` et `ignored_count`, table des soldes vus, en-tête `PROVIDER_API` et
`providerConnection` des imports, gardes d'import), 38-3b (passage asynchrone `POST …/fetch` → 202, `GET …/runs`,
`GET …/balances` et contrôle des soldes, **`PATCH …/balances-seen/{currency}`** (38-3b AC-7), `GET
/provider-fetch/connections`, transport factice et route `_test`), 38-7 (« Tester la connexion »,
`earliestFetchFrom`, codes d'erreur PayPal), 38-8 (surcharges `eventKindOverrides`), 37-4 (`account_balances::
balances_as_of`). **Suivie de** 38-9b (règles, propositions, rattachement, file, E2E du parcours comptable).
**Aucune migration** ; une route de lecture (AC-8, *Écart à l'epic*).*

## Story

En tant qu'Administrateur, je veux créer et régler mes connexions PayPal à l'écran, y déposer le secret sans jamais
le revoir, tester l'accès, lancer un téléchargement, voir les soldes de devise que PayPal annonce sans compte dans
Kesh pour créer ou lier ce compte, régler le compte de frais de chaque devise, et voir d'un coup d'œil si le
téléchargement est complet et si mes livres concordent avec le solde PayPal — et, comme Comptable, voir sur la page
des comptes bancaires quels comptes sont alimentés par un prestataire et les télécharger à la demande — afin de tenir
un compte PayPal sans saisie ni outil externe, en sachant toujours ce que Kesh a reçu et ce qu'il n'a pas encore
rapproché.

## Exigences

- `FR-38-9a-1` — Écrans de liste et de fiche des connexions, réglages compris ; secrets en saisie seule → AC-1, AC-2,
  AC-3.
- `FR-38-9a-2` — Boutons « Tester la connexion » et « Télécharger maintenant », état du dernier passage → AC-4, AC-5.
- `FR-38-9a-3` — Soldes sans compte listés, avec « créer » ou « lier » un compte (B2) → AC-6.
- `FR-38-9a-4` — Réglages d'un compte prestataire et compte de frais de la devise → AC-7.
- `FR-38-9a-5` — Contrôles de solde affichés : téléchargement incomplet, écart comptable → AC-8.
- `FR-38-9a-6` *(ajoutée)* — Sur la page des comptes bancaires, un compte prestataire se reconnaît (prestataire,
  connexion, devise) et ses champs interdits (QR-IBAN, principal, genre, carte, solde d'ouverture) ne sont pas
  proposés ; son IBAN, facultatif, reste conservé → AC-9.
- `FR-38-9a-7` *(ajoutée, P1)* — Les écrans existants que les gardes de 38-2 et 38-3a touchent disent la vérité : un
  import « téléchargé de … » se lit comme tel et ne se supprime pas, un compte prestataire n'est offert ni à l'import
  de fichier ni au lot de paiement → AC-15.
- `NFR-38-9a-1` — Aucun secret dans le DOM ni les réponses lues par l'écran → AC-3, AC-12.
- `NFR-38-9a-2` — Textes aux quatre locales ; manuel d'administration et PDF à jour (sauvegarde d'avant la mise à
  jour, `KESH_SECRETS_KEY`) → AC-10, AC-11.
- `NFR-38-9a-3` — Captures d'écran de chaque état touché → AC-13.
- `NFR-38-9a-4` *(ajoutée)* — Textes qui disent « télécharger », « proposer », jamais « comptabiliser
  automatiquement » ; un appariement propose, il ne crée jamais (`CLAUDE.md`) → AC-6, AC-10.

## Critères d'acceptation

1. **AC-1 — Navigation et pages** (Administrateur seul).
   - Entrée **« Prestataires de paiement »** (clé `nav-admin-providers`, AC-10) dans le groupe Administration
     (`nav-administration`), liste `adminOnly` de
     `frontend/src/routes/(app)/+layout.svelte:147` (à côté de `/admin/backup`, `/admin/mcp`), lien `/admin/providers`.
   - Pages `frontend/src/routes/(app)/admin/providers/+page.svelte` (liste) et `…/admin/providers/[id]/+page.svelte`
     (fiche) ; code dans une fonctionnalité neuve **`frontend/src/lib/features/provider-connections/`** (`*.api.ts`,
     `*.types.ts` — miroir de `ProviderConnectionResponse` de 38-1 AC-3, composants, tests Vitest ; clés i18n au
     préfixe `provider-connections-`, AC-10) ; la page des comptes
     bancaires (`routes/(app)/bank-accounts/+page.svelte`, **1 295 lignes**) ne reçoit que des appels de composants neufs
     (AC-9), pas de logique inline de plus.
   - Un Comptable ou une Consultation qui ouvre `/admin/providers` voit le refus habituel des pages
     d'administration (patron de `/admin/backup`), jamais une page vide ; les routes de 38-1 rendent de toute façon 403.
   - **Liste** : libellé, prestataire (« PayPal »), « télécharger depuis » (date suisse, `formatSwissDate`), réglage
     (« Chaque jour » / « À la demande »), **état** (`usable` / `unusableReason` → texte : « Prête », « Secret à saisir »,
     « Clé de chiffrement absente du serveur », « Secret illisible — à ressaisir », « Réglages illisibles »,
     « Prestataire inconnu de cette version »), dernier passage (date, état, AC-5), nombre de comptes (**compté sur les
     blocs `provider` de `GET /api/v1/bank-accounts`**, groupés par `connectionId` : `ProviderConnectionResponse` de
     38-1 AC-3 n'a pas ce champ) ; bouton « Nouvelle connexion » ; liste vide → phrase d'aide qui renvoie au manuel.
     Données : `GET /api/v1/provider-connections` (38-1, Administrateur seul).
2. **AC-2 — Créer et modifier une connexion** (FR-38-9a-1 ; `POST`, `PATCH` de 38-1 AC-4).
   - Champs : prestataire (liste des prestataires connus ; PayPal seul ; non modifiable ensuite), libellé (≤ 100),
     « Télécharger depuis » (date ; **minimum** = `earliestFetchFrom` de la connexion, 38-7 AC-9 — en création, le
     refus `PROVIDER_FETCH_FROM_TOO_OLD` du serveur affiche `details.earliest` sous le champ ; aucune constante de trois
     ans dans le TypeScript ; aide : « la date ne remonte pas dans le temps après un premier téléchargement : chaque
     compte repart de son dernier téléchargement moins le recouvrement (trois jours par défaut) » — passage de 38-3b,
     début = `max(fetch_from, fetched_through − overlap())`), réglage (`daily` | `manual`, avec la phrase « Chaque jour, Kesh télécharge les nouveaux
     mouvements ; vous pouvez toujours télécharger à la demande », B9), **réglages PayPal** : `clientId` (identifiant
     de l'application REST ; 1 à 128 caractères `[A-Za-z0-9_-]` (38-1 AC-4 : sinon `PROVIDER_SETTINGS_INVALID`,
     `details.field = "clientId"`, `details.reason` ∈ `invalid_type`, `invalid_format` de 38-1 AC-4, libellés dans
     `provider-labels.ts` avec ceux de 38-8 et `out_of_range`, sous le champ) ; aide : « non secret ; le secret se
     saisit à part ») ; `payerId` **en lecture seule**
     (posé par le serveur, 38-1 AC-4, 38-7 AC-8) ; **un `clientId` changé remet `payerId` à `null`** dans la même
     écriture (38-1 AC-4) : après l'enregistrement, la fiche relit la connexion, affiche « Identifiant du compte : à
     reconnaître — lancez « Tester la connexion » » et « Tester » le repose (AC-4) ; test Vitest. **`accountEmail` n'est ni affiché ni saisi** : retiré du JSON par O10
     (aucune API de PayPal ne le rend ; 38-1 AC-4 le refuse comme clé inconnue).
   - **Corps de `settings` envoyé** (38-1 AC-4 : `settings` présent **remplace** les champs modifiables par le client et
     **refuse** un champ posé par le serveur) : une fonction pure `providerSettingsBody(form)` construit l'objet à partir
     des **seuls** champs du client — `clientId`, `eventKindOverrides`, `overlapDays` —, **jamais** `payerId` (sinon `400
     PROVIDER_SETTINGS_INVALID`, `details.field = "payerId"`) et **toujours les trois** (un champ omis serait remis à son
     défaut : surcharges effacées, recouvrement ramené à 3) ; l'objet `settings` lu au `GET` n'est jamais renvoyé tel
     quel. Test Vitest : réglages lus avec `payerId` → corps sans `payerId` ; surcharges et recouvrement non touchés → corps
     qui les porte inchangés. « Télécharger depuis » : maximum = lendemain du jour (38-1 AC-4, `VALIDATION_ERROR` sinon).
   - **Surcharges de codes** (38-8 AC-7), dans un bloc replié « Avancé » : paires code `T0000` (quatre chiffres, `^T\d{4}$`)
     → genre (liste fermée : vente, achat, frais, retrait, dépôt, litige, autre, ignorer ; ni conversion ni
     remboursement), ajout et retrait de lignes, **50 au plus** ; aide : « ne vaut que pour les codes que Kesh ne classe
     pas ; s'applique aux mouvements téléchargés ensuite, les mouvements déjà téléchargés se traitent à la main »
     (texte de 38-8 AC-7). **Genre « ignorer »** : avertissement sous la ligne choisie (38-8 AC-7, « c'est dit sous le
     champ ») — « ignorer un code qui peut changer le solde fera apparaître un écart durable au contrôle de
     complétude » (le contrôle (a) de l'AC-8 ; aucun renvoi interne dans le texte affiché ; dit aussi au manuel,
     AC-11) ; test Vitest : genre `ignore` choisi → avertissement affiché, autre genre → absent. Refus du serveur (`PROVIDER_SETTINGS_INVALID`, `details.field`, `details.code`,
     `details.reason` ∈ `kind_not_allowed`, `code_not_overridable`, `too_many`, `malformed` : un libellé par motif)
     affiché à la ligne fautive.
   - **Recouvrement** (`overlapDays`, 38-8 AC-7, O11), dans le même bloc « Avancé » : nombre de jours (1 à 30 ; vide =
     défaut 3, affiché en indication), modifiable par le client au `PATCH` ; aide : « nombre de jours relus à chaque
     téléchargement, pour reprendre les paiements en attente devenus réussis ; à n'augmenter que si le contrôle de
     complétude signale un écart » ; refus `PROVIDER_SETTINGS_INVALID` `details.reason = "out_of_range"` sous le champ.
   - `PATCH` avec `version` ; `409 OPTIMISTIC_LOCK_CONFLICT` → message existant et relecture proposée ; libellé pris →
     `PROVIDER_CONNECTION_LABEL_DUPLICATE` sous le champ.
   - **Supprimer** (`DELETE` avec `version`, 38-1 AC-5) : boîte de confirmation au jeton des actions destructives
     (`destructive-buttons.spec.ts` pour le patron) ; `PROVIDER_CONNECTION_IN_USE` → message qui liste les comptes
     (`details.bankAccountIds`, noms lus de `GET /bank-accounts`) et dit de retirer d'abord leurs réglages (AC-7).
3. **AC-3 — Secret en saisie seule** (FR-38-9a-1, NFR-38-9a-1 ; `PUT …/secrets` de 38-1 AC-6).
   - Bloc « Secret de l'application » : champ `type="password"`, `autocomplete="new-password"`, `spellcheck="false"`,
     **jamais prérempli** ; à côté, l'état seul : « Secret enregistré le <date> » (`secretSet`, `secretUpdatedAt` de 38-1
     AC-3) ou « Aucun secret ». Bouton « Enregistrer le secret » (désactivé champ vide) ; succès → le champ est **vidé**,
     la variable d'état qui tenait la valeur est remise à `''`, l'état se relit. Bouton « Remplacer » sur un secret
     existant (même formulaire).
   - La valeur ne passe par **aucun** store partagé, journal de console, `toast`, message d'erreur ni attribut
     `data-*` ; un échec (`PROVIDER_SECRETS_INVALID`, `PROVIDER_SECRETS_KEY_MISSING`) affiche le message du code,
     jamais la valeur.
   - Après une restauration (secrets effacés, E2), la fiche dit « Secret à saisir : les secrets ne sont pas emportés
     par les sauvegardes » (texte unique, `unusableReason = "secret_missing"`).
4. **AC-4 — Tester la connexion** (FR-38-9a-2 ; 38-1 AC-7, 38-7 AC-8). Bouton « Tester la connexion » ; la réponse
   (toujours 200) s'affiche en **liste de contrôles** : un libellé par `check` (`provider_known`, `settings_readable`,
   `client_id_present`, `secrets_key_present`, `secret_present`, `secret_decryptable`, `token_obtained`,
   `permission_present`, `provider_reachable`), ✓ / ✗ / « non tenté » (`ok: null`), et, pour un ✗, le message de son
   `errorCode` (AC-10) avec le remède : `PROVIDER_PERMISSION_MISSING` → « Dans PayPal Developer, cochez *Transaction
   Search* pour cette application, puis réessayez » ; `PROVIDER_AUTH_FAILED` → « Identifiant ou secret refusé par
   PayPal : vérifiez l'identifiant de l'application et remplacez le secret ». Succès : `payerId` et la liste des
   soldes lus (devise, total, au format suisse ; « lus à l'instant, non enregistrés »). **La réponse porte `version`**
   (38-7 AC-8 : un `payerId` neuf s'écrit avec `version + 1`) : la fiche adopte cette `version` et relit `payerId`
   (`GET /provider-connections/{id}`) avant tout `PATCH` — sans quoi le formulaire ouvert avant le test échouerait en
   `409 OPTIMISTIC_LOCK_CONFLICT` ; test Vitest : test réussi puis `PATCH` → corps à la version rendue. Pendant l'appel : bouton
   désactivé, indicateur ; `timeoutMs` du client à **250 s** (`CallOptions`, `frontend/src/lib/shared/utils/api-client.ts:61`)
   — le défaut de 30 s ne couvre pas un jeton puis des soldes à 60 s de délai chacun, avec un second essai après 2 s sur
   5xx (38-7 AC-1, AC-7 : au pire 2 × (60 + 2 + 60) s) ; la valeur se relit de 38-7 si ses bornes changent. `retry:
   false` (un `POST` n'est pas rejoué).
5. **AC-5 — Télécharger maintenant et état des passages** (FR-38-9a-2 ; `POST /provider-connections/{id}/fetch` de
   38-3b, E10).
   - Bouton « Télécharger maintenant » sur la fiche (Administrateur) **et** par compte prestataire sur la page des
     comptes bancaires (Comptable et Administrateur, AC-9), selon les droits que fixe 38-3b (`comptable_routes`,
     session seule). **Le passage est asynchrone** : `POST /api/v1/provider-connections/{id}/fetch` rend **202**
     `{ runId }` aussitôt (38-3b) ; l'écran **suit** le passage par `GET /api/v1/provider-connections/{id}/runs?
     limit=` (Comptable et Administrateur ; toutes les 3 s, arrêt dès que le passage `runId` n'est plus `running`, ou à
     l'abandon de la page) et affiche « Téléchargement en cours… ». Refus synchrones : `409
     PROVIDER_CONNECTION_UNUSABLE` (`details.reason` = `unusableReason`, message du code), `409
     PROVIDER_FETCH_IN_PROGRESS` (`details.runId` : l'écran se met à suivre ce passage), `404` ; bouton réactivé.
   - Résultat, lu de la ligne de passage (`GET …/runs` de 38-3b AC-6, champs camelCase) : nombres insérés
     (`insertedCount`), déjà présents (`knownCount`), comptes non téléchargés faute de compte Kesh
     (`skippedUnlinkedCount`, renvoi à AC-6), état (`status`) et code d'erreur (`errorCode`) (libellés AC-10). S'y ajoutent (38-3b AC-6 :
     colonnes de la table, `truncatedFrom` et `ignoredCount` compris ; Q-38-9a-1 close) :
     - les **enregistrements écartés** (`ignoredCount`, **total seul** : 38-3b AC-4 somme `skipped.denied + pending +
       non_balance` dans `ignored_count` et n'en garde pas la ventilation) — « N enregistrements PayPal écartés (refusés,
       en attente — relus au prochain téléchargement —, ou sans effet sur le solde) », en texte discret, jamais en erreur ;
       zéro → rien d'affiché ;
     - l'avis de **date de départ bornée** par l'historique de PayPal (38-7 AC-9, `truncatedFrom` non nul) : « PayPal
       ne remonte que trois ans : téléchargé depuis le <date suisse de `truncatedFrom`> » ;
     - les **motifs** d'un passage `succeeded` : `errorDetail` en porte **un ou plusieurs, joints par `; `** (38-3b
       AC-7 : `balances_unavailable`, puis `opening_unavailable`, un par devise au plus, chacun constant) ; une fonction
       pure de `provider-labels.ts` **découpe** `errorDetail` sur `; ` et lit **chaque motif séparément** par son
       préfixe (jamais `errorDetail` entier par « commence par » : le second motif d'un passage n'est pas en tête) ;
       un avis par **libellé** distinct (deux devises au même motif, ou `opening_unavailable: auth` et
       `opening_unavailable: transport`, qui portent le même libellé → un seul avis), en avertissement discret :
       - `balances_unavailable: <code()>` (soldes non lus, mouvements téléchargés) → « Soldes non lus à ce passage :
         contrôle de complétude non mis à jour » ;
       - `opening_unavailable: <code()>` (l'`opening` rendu antérieur au début effectif n'a pu être complété —
         lecture de `[A, D)` en échec — et aucun solde d'ouverture n'a été écrit) → « Solde d'ouverture non établi à
         ce passage : contrôle de complétude non disponible, réessayé au passage suivant » ;
       - `opening_unavailable: span` (même cas, lecture **non tentée** : `[A, D)` dépasse trois `max_window()`) →
         « PayPal annonce un solde d'ouverture trop ancien pour être complété : contrôle de complétude non disponible
         tant que PayPal annonce la même date (voir le manuel) » — jamais « réessayé » seul : 38-3b AC-7 le dit
         **durablement** non disponible tant que PayPal sert le même `A` ;
       - préfixe inconnu (lecture tolérante, E4) → « Avertissement de passage non reconnu (`<préfixe>`) », le seul
         préfixe avant `:` (motif sans `:` : le motif entier, tronqué à 40 caractères) ;
       l'état du contrôle (a) de l'AC-8 reste celui que rend 38-3b (`complete: null` → « Contrôle non disponible »),
       jamais déduit d'un motif ; le reste d'un motif (le `code()` ou `span` après `: `) n'est **jamais**
       affiché brut, ni `errorDetail` entier ; libellés aux quatre locales (AC-10).
     **Deux vocabulaires de codes** (38-3b AC-4, 38-7 AC-7) : `errorCode` d'un **passage** est `code()` en minuscules
     (`auth`, `permission`, `rate_limited`, `unavailable`, `invalid_response`, `transport`, `window_too_large`,
     `rejected`, `invalid_window`) ou un code propre du passage (`connector_unavailable`, `interrupted`,
     `account_unavailable`, `database`) ; `lastError` de la **connexion** et `errorCode` de « Tester » portent le
     **code d'API** (`PROVIDER_AUTH_FAILED`…, `PROVIDER_FETCH_INTERRUPTED`…). `provider-labels.ts` a une table par
     vocabulaire (AC-10), test : chaque valeur des deux listes a un libellé, une valeur inconnue → brute.
   - **État du dernier passage** (liste et fiche) : `lastRunAt`, `lastRunStatus` (texte par valeur ; une valeur
     inconnue → « État inconnu (`<valeur>`) », lecture tolérante E4), `lastError` → message du code (AC-10), jamais un
     texte de PayPal. **Historique** des derniers passages (déclencheur, utilisateur, fenêtre, durée, état, nombres) par
     `GET …/runs` (38-3b, 20 par défaut, 100 au plus). Pour le Comptable (page des comptes bancaires), l'état se lit
     de `GET /api/v1/provider-fetch/connections` (38-3b : sans `settings`, avec `runningRunId`).
   - Ni bouton ni texte ne disent « comptabiliser » : un téléchargement ajoute des transactions **à rapprocher**
     (lien vers `/reconciliation` du compte).
6. **AC-6 — Soldes sans compte** (FR-38-9a-3, B2, E12 ; soldes vus et décision de 38-3a, lecture de 38-3b).
   - Bloc « Soldes PayPal sans compte dans Kesh » : par devise, premier vu, dernier solde rapporté, état (nouveau,
     ignoré). Données : `GET /api/v1/provider-connections/{id}/balances` (38-3b ; Comptable et Administrateur) →
     `unlinked: [{ currency, firstSeenAt, lastSeenAt, lastAmount, ignored }]` (devises déjà tenues par un compte
     exclues) et `accounts: […]` (AC-8 a). **Ignorer** : `PATCH /api/v1/provider-connections/{id}/balances-seen/
     {currency}` `{ ignored }` (**38-3b AC-7**, Administrateur, session ; **pas de `version`** ; table de 38-3a). Noms
     arrêtés par 38-3b (O12), plus à proposer.
   - Actions, Administrateur : **« Créer le compte »** → dialogue `POST …/accounts` `mode: "create"` (38-2 AC-3) :
     devise **fixée** à celle du solde, nom proposé « <libellé de la connexion> <devise> » (modifiable), compte du plan
     **en cette devise** (sélecteur filtré : de bilan, actif, imputable, non lié à un autre compte prestataire —
     comptes en devise de 37-1), compte de frais facultatif (AC-7) ; **« Lier un compte bancaire existant »** →
     `mode: "attach"` : sélecteur des comptes bancaires `bank` actifs non prestataires, compte du plan à garder ou
     remplacer ; **« Ignorer »** / **« Ne plus ignorer »**. Jamais de création sans clic : le bloc **propose**.
   - Chaque refus de 38-2 (`PROVIDER_ACCOUNT_CURRENCY_MISMATCH`, `PROVIDER_ACCOUNT_DUPLICATE_CURRENCY`,
     `PROVIDER_ACCOUNT_ALREADY_ATTACHED`, `PROVIDER_ACCOUNT_LEDGER_REQUIRED`, `PROVIDER_ACCOUNT_LEDGER_SHARED`,
     `PROVIDER_ACCOUNT_KIND_INVALID`, `PROVIDER_ACCOUNT_CANNOT_BE_PRIMARY`, `FEE_ACCOUNT_INVALID`, `BANK_ACCOUNT_CLOSED`)
     s'affiche dans le dialogue avec son message ; `MULTI_CURRENCY_DISABLED` (37-1) → renvoi aux réglages de devise.
     **`409 PROVIDER_ACCOUNT_HAS_TRANSACTIONS`** (« Lier », 38-2 AC-3 : réglages refusés sur un compte qui porte des
     transactions) → message « Ce compte bancaire porte déjà <`details.transactionCount`> transactions (relevés
     importés ou saisis) : PayPal les téléchargerait une seconde fois. Créez un compte neuf pour cette devise, ou
     liez un compte sans transaction » ; libellé dans `provider-labels.ts`, quatre locales ; test Vitest.
7. **AC-7 — Comptes prestataires de la connexion** (FR-38-9a-4 ; `GET|PATCH|DELETE …/accounts` de 38-2 AC-5, AC-8).
   Tableau : devise, compte bancaire (nom, lien vers `/bank-accounts`), compte du plan (numéro, nom), **compte de
   frais de la devise** (numéro, nom ; « non désigné » en avertissement : les propositions de frais seront bloquées,
   `FEE_ACCOUNT_INVALID` de 38-6a), contrôles de solde (AC-8). Actions : « Changer le compte de frais » (`PATCH`
   `{ version, feeAccountId }` ; sélecteur : comptes de charge actifs imputables **de la même devise** ; aide : « le
   regroupement des frais est le compte parent de ce compte dans le plan », B3) ; « Retirer les réglages » (`DELETE`,
   confirmation destructive : « le compte bancaire reste, il n'est plus alimenté par PayPal ») ; refus
   `PROVIDER_ACCOUNT_IN_USE` → les raisons de `details.reasons`, une par ligne, chacune libellée (`foreign_currency_link`,
   `transactions`, `no_iban`, `balance_seen` — 38-2 AC-5, 38-3a AC-4 ; table de `provider-labels.ts`) ; de même les
   raisons de `FEE_ACCOUNT_INVALID` (`not_found`, `archived`, `not_postable`, `not_expense`, `currency_mismatch`,
   `missing`). Le sélecteur du compte de frais **suggère** un sous-compte par prestataire et par devise (B3, 38-2 AC-5),
   sans l'imposer.
8. **AC-8 — Contrôles de solde** (FR-38-9a-5, E20).
   - **(a) Complétude** (contrôle des soldes de 38-3b, C-38-5) : par compte, lue de `GET /provider-connections/{id}/balances` →
     `accounts[]` `{ bankAccountId, currency, opening, reported, expected, gap, complete }` : « Téléchargement complet au
     <`reported.asOf`> » (`complete: true`) ou **« Téléchargement incomplet : écart de <`gap`, devise> au
     <`reported.asOf`> »** — l'instant affiché est **celui du solde affiché** : `reported.asOf`, l'`as_of` que le
     connecteur rend pour ce solde (38-3b AC-7 : PayPal peut servir un solde antérieur à la fin de la fenêtre), en date
     et heure suisses de Zurich, jamais l'heure du passage ni la fin de la fenêtre demandée ; test Vitest : `asOf`
     antérieur au `finishedAt` du passage → c'est `asOf` qui s'affiche — en
     avertissement (`complete: false`), avec le renvoi « Relancez un téléchargement ; si l'écart demeure, voir le
     manuel » ; **`complete: null`** (38-3b AC-7 : compte sans solde d'ouverture ou sans solde rapporté, ou
     `fetched_through` du compte antérieur à l'`asOf` du dernier solde rapporté — un passage en `balances_unavailable`
     ne rend pas `null`, il laisse le dernier solde rapporté, que l'avis de l'AC-5 signale ; un passage en
     `opening_unavailable` laisse le compte sans solde d'ouverture, d'où `null`, et l'avis de l'AC-5 en dit le motif) →
     « Contrôle non disponible » en gris, jamais en vert ; donnée lue telle que 38-3b la calcule (jamais recalculée à l'écran).
   - **(b) Écart comptable** : *Écart à l'epic* — E20 (b) n'est porté par aucune route des fiches 38-1 à 38-3b ; cette
     story l'ajoute en **lecture seule** : `GET /api/v1/bank-accounts/{id}/provider-balance-check` (`comptable_routes`,
     `crates/kesh-api/src/lib.rs:524`, **mêmes droits que `GET /provider-connections/{id}/balances` de 38-3b AC-7**, qui
     rend le même solde rapporté : Comptable et Administrateur, par **session** — **clés d'API refusées dans le
     handler** (`ApiKeyOperationForbidden`) : les données du prestataire ne s'ouvrent ni aux clés ni à la Consultation
     par une seconde porte (E23 ; tests : clé `read` → 403, Consultation → 403 ; l'écran n'appelle pas la route pour
     la Consultation et n'affiche pas le résumé — le badge de l'AC-9, lu de `GET /bank-accounts`, reste visible à tous),
     réponse
     `{ currency, asOf, reported, ledger, unreconciled, unexplained }` ou `404` si le compte n'est pas prestataire ou
     n'a aucun solde rapporté :
     - `reported` = dernier solde rapporté de la devise (`provider_balance_snapshots`, `kind = 'reported'`, table de 38-3a écrite par 38-3b), `asOf`
       son instant (UTC) — l'`as_of` de l'instantané, **rendu par le connecteur** (38-3b AC-7), non l'instant demandé ;
       l'écran l'affiche avec le solde (« Selon PayPal, au <`asOf`> : X »), comme au (a) ;
     - `ledger` = solde **en devise** du compte du plan lié, **par `kesh_db::repositories::account_balances::
       balances_as_of(conn, company_id, date de asOf à Europe/Zurich, Some(&[compte]))`** (37-4 AC-1 quater : borne
       inclusive, débit net brut CHF et en devise, signé comme un actif : Σ débit − crédit) — **aucune seconde somme
       `amount_currency`** (DRY avec 38-10, C-38-10-4) ; montant CHF pour un compte en CHF ;
     - `unreconciled` = Σ `amount` des transactions `pending` du compte (rejetées et « examinées » comprises : elles
       restent `pending`, 38-6b C-38-6b-6) dont **`booked_at < asOf`** — borne **exclue**, comme le contrôle de complétude de
       38-3b AC-7 (fenêtre demi-ouverte : un mouvement à l'instant exact du `reported` n'y est pas compris) ; l'instant,
       non `booking_date` ; test : un mouvement `pending` à `booked_at = asOf` n'est pas compté ; une transaction
       **sans** `booked_at` (import de fichier d'un compte posé en `attach` avant ses réglages, 38-2 AC-3) compte si sa `booking_date ≤` la date de Zurich de `asOf` (borne à la journée, comme
       `ledger`) — sans quoi elle manquerait en silence (test) ;
     - `unexplained` = `reported − ledger − unreconciled`. ⚠️ **Sens** : relevé moins livres, **l'inverse** de
       `statementGap` de la même page (`statement_gap`, `repositories/bank_accounts.rs:2019-2024` : livres − relevé) ;
       l'écran ne montre donc jamais les deux nombres sous le même mot — « Inexpliqué » ici, « Écart » là — et le manuel
       dit le sens de chacun. ⚠️ **Limite écrite** (doc-comment, manuel) : `ledger` est
       arrêté à la **journée** de Zurich de `asOf`, `reported` et `unreconciled` à l'**instant** ; un mouvement de la même
       journée postérieur à `asOf` et déjà rapproché fait apparaître un `unexplained` transitoire, résorbé au passage
       suivant — test à deux mouvements de part et d'autre de `asOf`.
     Écran : « Selon PayPal, au <`asOf`> : X · Aux livres : Y · Non rapproché : Z · **Inexpliqué : W** » (W ≠ 0 en avertissement,
     W = 0 en vert) sur la ligne du compte (AC-7) et sur la page des comptes bancaires (AC-9). Code dans un module de
     lecture neuf (`kesh-db::repositories::provider_balance_check`, une fonction), route dans
     `routes/provider_connections.rs` ou un fichier voisin — **pas** dans `routes/bank_accounts.rs`. Registre (C-38-12) :
     route de lecture, hors `LIB_ROUTES` — le registre ne balaie que `post`, `put`, `delete`, `patch`
     (`crates/kesh-api/tests/audit_route_registry.rs:151`) : **delta 0**. Tests : compte EUR à transactions rapprochées
     et non rapprochées, écart construit, compte non prestataire → 404, autre société → 404, Consultation → 403, Comptable →
     200, clé d'API → 403.
   - Le **relevé joint** et son solde de clôture (E20, contrôle de clôture) sont à 38-10, pas ici.
9. **AC-9 — Page des comptes bancaires** (FR-38-9a-6 ; bloc `provider` de 38-2 AC-8).
   - Ligne d'un compte prestataire : badge « PayPal · <libellé de connexion> · <devise> » (composant
     `ProviderAccountBadge.svelte`), contrôle de solde (AC-8 b) en résumé (Comptable et Administrateur ; absent pour la
     Consultation, qui garde le badge), bouton « Télécharger maintenant » (AC-5) pour
     Comptable et Administrateur, lien « Réglages de la connexion » pour l'Administrateur.
   - Formulaire de modification d'un compte prestataire : QR-IBAN, « compte principal », genre (bancaire / carte),
     quatre chiffres de carte **non affichés** (38-2 AC-4 : `PROVIDER_ACCOUNT_FIELD_NOT_ALLOWED`,
     `PROVIDER_ACCOUNT_CANNOT_BE_PRIMARY`). ⚠️ **L'IBAN reste affiché, facultatif** : `PUT /bank-accounts/{id}` est un
     remplacement complet (38-2 AC-4 : un compte posé en `attach` avec IBAN « le garde tant que le `PUT` le renvoie ») ;
     masquer le champ effacerait silencieusement l'IBAN. Le formulaire le pré-remplit, accepte de le vider
     (`iban` absent ou `null`), et renvoie `accountKind: 'bank'` ; test Vitest : `PUT` d'un compte prestataire avec IBAN →
     le corps porte l'IBAN. Le compte du plan reste modifiable dans les limites de 38-2
     (`PROVIDER_ACCOUNT_CURRENCY_MISMATCH`, `PROVIDER_ACCOUNT_LEDGER_REQUIRED` affichés). Formulaire de **création** :
     inchangé (un compte prestataire se crée depuis la connexion, AC-6). Les `data-testid` existants
     (`edit-iban`, `edit-is-primary`, `edit-kind`, `edit-card-last4`) restent pour les comptes ordinaires.
   - Un compte prestataire archivé ou clos (28-4a) garde son badge ; « Télécharger maintenant » y est absent.
10. **AC-10 — Textes et i18n** (NFR-38-9a-2, NFR-38-9a-4). Préfixes imposés par `lint-i18n-ownership`
    (`frontend/scripts/lint-i18n-ownership.js:177-186`, `keyBelongsToFeature` : sous `features/<f>/`, un nom à tiret
    exige `key.startsWith('<f>-')`, sinon le premier segment de la clé doit valoir `<f>` ; seuls `error-`, `tooltip-`,
    `common-`, `mode-`, `shortcut-`, `demo-` y échappent, `:16`) : clés neuves de l'écran au préfixe
    **`provider-connections-`** (tout fichier de `features/provider-connections/`, `provider-labels.ts` et
    `ProviderAccountBadge.svelte` compris — le badge vit dans cette fonctionnalité, la page des comptes bancaires
    l'importe) ; entrée de menu **`nav-admin-providers`** (patron `nav-admin-backup`, `nav-admin-mcp` ; posée dans
    `routes/(app)/+layout.svelte`) ; un texte neuf écrit **dans** une autre fonctionnalité porte le préfixe de celle-ci
    (`bank-import-` pour `bank-import-display.ts`, AC-15 : « Téléchargé de … », « Source inconnue (`x`) » ;
    `payment-batches-`, `bank-accounts-`, `accounts-` de même), sinon le libellé est lu par un appel à
    `provider-labels.ts`, jamais par une clé `provider-connections-*` écrite hors de sa fonctionnalité. Le linter ne
    balaie que `src/lib/features/` : les pages de `routes/(app)/` (comptes bancaires, plan comptable, import) ne
    sont pas contrôlées, et le préfixe y suit l'usage de la page (`bank-accounts-`, `accounts-`) ; la règle ci-dessus
    ne lie que les fichiers posés sous `features/`. Clés **écrites en toutes lettres** (pas de clé construite par
    gabarit, `i18n-keys.test.ts`), aux
    quatre catalogues `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`, consommées par `i18nMsg` avec
    repli français (`frontend/src/lib/shared/utils/i18n.svelte`) ; **pas de sélecteur Fluent** (`{ $n -> … }`) — le
    client reçoit les messages pré-résolus (raison écrite dans `frontend/src/lib/features/reconciliation/already-booked.ts`) :
    paires `-one` / `-other` au besoin. Libellés des **codes** (`unusableReason`, `check`, `lastRunStatus`, codes
    d'erreur d'API de 38-1, 38-2, 38-3a, 38-3b, 38-7, **codes de passage** `errorCode` en minuscules — table à part,
    AC-5 —, raisons de `PROVIDER_ACCOUNT_IN_USE` et de `FEE_ACCOUNT_INVALID`, motifs de `PROVIDER_SETTINGS_INVALID`,
    **motifs d'`errorDetail` d'un passage** — `balances_unavailable`, `opening_unavailable`, `opening_unavailable: span`
    et le préfixe inconnu, AC-5 —, genres de mouvement des surcharges) dans un module unique
    `provider-labels.ts` (patron `failed-proposal-label.ts` : table écrite à la main, repli qui montre le code brut).
    Les messages d'erreur rendus par le serveur (`error.message`, traduits par `t(…)`) s'affichent tels quels quand le
    module n'a pas de libellé propre. `npm run lint-i18n-ownership` vert. Aucun texte ne promet une écriture
    automatique : vocabulaire « télécharger », « proposer », « rapprocher ».
11. **AC-11 — Manuel d'administration** (`docs/manual/fr/admin-manual.tex`, NFR-38-9a-2).
    - Section neuve **« Prestataires de paiement (PayPal) »** (après *Le serveur MCP*, `\section{Le serveur MCP}` à la
      ligne 3299), complétant ce que 38-1 AC-10 y a posé sur `KESH_SECRETS_KEY` (ne pas le recopier : renvoi
      `\ref`) : créer l'application REST *live* chez PayPal en lecture seule, cocher **Transaction Search**, relever
      l'identifiant ; créer la connexion ; déposer le secret ; tester (sens de chaque contrôle et remède) ; date de
      départ et limite de trois ans ; réglage quotidien ou à la demande ; soldes proposés, créer ou lier un compte en
      devise (comptes en devise de l'Epic 37, lien qui garde numéro, historique et soldes — D8) ; compte de frais par
      devise et regroupement par le compte parent (B3) ; surcharges de codes ; contrôles de solde (a) et (b), et que
      faire d'un écart ; **les secrets ne voyagent pas avec les sauvegardes et se ressaisissent après toute
      restauration** (E2) ; rotation de la clé hors périmètre (C-38-13) ; la recette sur une instance jetable (B12).
    - **Limites et faits que les fiches amont renvoient à ce manuel** (inventaire relevé par `grep -n "38-9a"` sur les
      fiches 38-1 à 38-8, à recompter au développement, chacun écrit ou dit sans objet au Change Log) :
      - 38-2 AC-4 et Notes : un compte prestataire **archivé** garde sa ligne de réglages et **tient sa devise** pour la
        connexion (`uq_ppa_connection_currency`) — recréer un compte de même devise exige le retrait des réglages,
        refusé s'il a des transactions ;
      - 38-3a AC-4 : retirer les réglages est refusé dès qu'un passage a lu le solde de la devise (`balance_seen`) — la
        connexion ne se supprime donc plus ; on la passe « à la demande » pour arrêter les téléchargements ;
      - **retrait d'un compte créé par erreur**, dans l'ordre : délier le compte du plan, archiver le compte bancaire,
        retirer ses réglages de prestataire (AC-7), supprimer la connexion (AC-2) ; chaque refus
        (`PROVIDER_ACCOUNT_IN_USE` et ses raisons, `PROVIDER_CONNECTION_IN_USE`) dit ce qui reste à défaire, et un
        compte qui porte des transactions ou dont le solde a été lu garde ses réglages (point précédent) ;
      - 38-2 AC-3 : « lier » un compte bancaire qui porte déjà des transactions est refusé
        (`PROVIDER_ACCOUNT_HAS_TRANSACTIONS`, doublons évités) — créer un compte neuf ;
      - 38-3b AC-8 : le passage quotidien est vérifié **toutes les heures** et lancé **24 h après le début** du
        dernier passage (un échec attend donc le lendemain, sauf « Télécharger maintenant »), jamais en mode test ; le
        passage est **asynchrone** (bouton, puis suivi) ; un passage interrompu (redémarrage, panique) passe
        « interrompu » ; un passage en cours venu d'une **sauvegarde importée** bloque la connexion **6 h** au plus ;
        recouvrement de 3 jours (`overlapDays`) ; date de départ bornée par l'historique (`truncated_from`) ; la devise
        d'un compte archivé ou clos n'est ni téléchargée ni proposée ;
      - 38-8 AC-7, AC-10 : comment PayPal est classé (familles de codes), ce qui va à la file « à examiner » (code
        inconnu, litige), que les surcharges ne valent que pour les codes classés « autre » et pour les mouvements
        téléchargés **après** leur écriture, et qu'**ignorer** un code qui peut changer le solde fait apparaître un
        écart durable au contrôle de complétude (38-8 AC-7 : « c'est dit sous le champ et au manuel » ; même phrase
        qu'à l'AC-2) ;
      - l'instant des contrôles (a) et (b) est celui du **solde rapporté** que PayPal rend (`asOf`), qui peut précéder
        la fin de la fenêtre téléchargée (38-3b AC-7) ;
      - 38-3b AC-7 : un solde d'ouverture que PayPal date avant le début du téléchargement se complète par une relecture ;
        si elle échoue, le contrôle (a) reste « non disponible » et se retente au passage suivant ; si l'écart de dates
        dépasse ce que Kesh relit (avis « trop ancien pour être complété », AC-5), il le reste **tant que PayPal annonce
        la même date** ;
      - 38-7 AC-8, AC-9 : la permission *Transaction Search* et la création de l'application REST ; la borne de trois
        ans ;
      - le sens des nombres du contrôle (b) (« Inexpliqué » = selon PayPal − aux livres − non rapproché), distinct de
        l'écart de relevé de la liste (AC-8).
    - **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, liste que 38-3a AC-7 et 38-2 transmettent « au manuel
      utilisateur et d'administration », non seulement à l'administration) : dans les sections des imports bancaires et
      des comptes bancaires, un compte alimenté par un prestataire ne reçoit ni import de fichier ni relevé saisi
      (`BANK_IMPORT_PROVIDER_ACCOUNT`), n'est pas offert au lot de paiement (`PAYMENT_BATCH_ACCOUNT_IS_PROVIDER`) ;
      un import « téléchargé de … » ne se supprime pas (`PROVIDER_IMPORT_NOT_DELETABLE`) ; les champs interdits du
      formulaire du compte (AC-9). Les sections de réconciliation des comptes prestataires sont à 38-9b (AC-10), qui
      renvoie à celle-ci ; PDF du manuel utilisateur régénéré et contrôlé aplati avec celui de l'administration.
    - **Mise à jour** (E19, NFR-38-2-3 et 38-2 AC-1 — la migration breaking, O1) : sous-section `\subsubsection{Passer à la <X.Y.Z>}` dans
      `\section{Mise à jour de Kesh}` (patron de `\subsubsection{Passer à la 0.17.6}`, ligne 2538), **numéro de Q1**
      (Guy) : **sauvegarde manuelle complète obligatoire avant la mise à jour** (base, `.keshbackup`, dossier des
      documents) et **seul chemin de retour** ; la migration breaking, **celle de 38-2** (O1) : la contrainte
      « un compte bancaire a un IBAN » remplacée (un compte prestataire n'en a pas), `match_type` des règles élargi,
      `kesh_version_min_required` relevé ; **aucun DDL** pour `source_format = 'PROVIDER_API'` (valeur écrite dès le
      premier téléchargement, que la version précédente ne sait plus lire) ; refus de la version précédente sur la base
      migrée et sur une sauvegarde de la nouvelle ; sauvegarde `backup/pre-upgrade/` écrite par Kesh (\S\ref{sec:sauvegarde-avant-maj}) ;
      variable neuve `KESH_SECRETS_KEY` (renvoi). Le texte **ne contient pas** de numéro de version tant que Q1 n'est pas
      tranchée : la sous-section s'écrit au développement si le numéro est connu, sinon la story attend (aucun
      gabarit `<…>` dans le `.tex`).
    - PDF régénéré (`make fr` dans `docs/manual/`) et commité ; contrôle du **PDF aplati** (`pdftotext admin-manual.pdf -
      | tr '\n' ' ' | tr -s ' '`) pour les phrases clés (« Transaction Search », « ressaisir », « seul chemin de
      retour »), résultat au Change Log. Manuels DE/IT/EN : « à traduire » noté si l'écart existe.
12. **AC-12 — Tests** (Vitest, `npm run test:unit` ; E2E Playwright).
    - Vitest : formulaire de connexion (validation, `earliest`, surcharges, `overlapDays` et son refus `out_of_range`,
      refus `details`, `providerSettingsBody` sans `payerId` et avec les trois champs, version adoptée après le test),
      résultat d'un passage (`ignoredCount` nul puis non nul, `truncatedFrom` présent puis nul, `balances_unavailable`, `opening_unavailable` (avis distinct, contrôle (a) « non disponible »),
      `opening_unavailable: span` (son propre libellé, sans « réessayé » seul), les deux motifs joints
      `balances_unavailable: auth; opening_unavailable: span` (deux avis, le second reconnu hors de tête), deux
      `opening_unavailable` identiques ou de codes différents (un avis), préfixe inconnu (libellé générique, préfixe
      seul ; sans `:`, motif tronqué) ; le texte rendu
      (`textContent` des avis, non `innerHTML`, qui porte des balises `<span>`) ne contient ni `auth` ni `span`,
      code de passage et code d'API libellés chacun par sa table), composant du secret
      (vidé après succès et après échec ; jamais dans un `data-*` ni dans le DOM rendu : rendu du composant après saisie
      puis enregistrement, `container.innerHTML` ne contient pas la valeur marqueur), liste de contrôles du test
      (chaque `check`, `ok: null`), libellés (`provider-labels.ts` : chaque code connu a un libellé, code inconnu → brut),
      bloc des soldes sans compte (actions selon le rôle), badge et formulaire des comptes bancaires (champs masqués
      pour un compte prestataire, présents pour un compte ordinaire et une carte), contrôle (b) (couleurs, signes ;
      rôle Consultation → badge rendu, route `provider-balance-check` non appelée et résumé absent, AC-8, AC-9).
    - Rust : la route de l'AC-8 (cas écrits à l'AC-8, dont le mouvement de part et d'autre de `asOf` et le mouvement
      `pending` à `booked_at = asOf`, exclu de `unreconciled`).
    - Vitest de l'AC-15 : libellé « Téléchargé de <libellé> » (`providerConnection` non nul), valeur inconnue de
      `source_format` → « Source inconnue (`x`) », bouton *Supprimer* absent pour un import de prestataire,
      `PROVIDER_IMPORT_NOT_DELETABLE` et `BANK_IMPORT_PROVIDER_ACCOUNT` libellés, sélecteurs sans compte prestataire.
    - Vitest du suivi de passage (AC-5) : 202 puis `running` puis `succeeded` ; `PROVIDER_FETCH_IN_PROGRESS` → suit
      `details.runId` ; abandon de la page → plus d'appel.
    - **E2E** `frontend/tests/e2e/provider-connections.spec.ts`, contre le **transport factice** de 38-3b et ses
      réponses PayPal par défaut (38-7 AC-10), **demandées explicitement** : sitôt la connexion créée à l'écran (son id
      lu de l'URL de la fiche), le test appelle `POST /api/v1/_test/provider-connector` avec, pour ce `connectionId`,
      **`paypalResponses: "default"`** (38-3b AC-3 : sans la variante, la connexion garde le `FakeConnector`, qui ne
      rend ni `payerId` ni les contrôles distants de PayPal) — préalables posés par l'API dans le `beforeAll` : société et exercice
      **multi-devises** (37-1), sans quoi le compte EUR est refusé `MULTI_CURRENCY_DISABLED` — : Administrateur crée une connexion, dépose un secret marqueur, teste
      (✓ partout), télécharge, voit les soldes CHF/EUR/USD sans compte, crée le compte EUR (compte du plan EUR du seed
      ou créé par l'API dans le test), voit le badge sur `/bank-accounts` et le contrôle (b) ; **le secret marqueur
      n'apparaît jamais** dans `page.content()` ni dans les réponses interceptées (`page.on('response')`) ; route
      `_test` de 38-3b (variante `paypalResponses` : une seule réponse, le jeton `POST /v1/oauth2/token` en 200 dont le
      `scope` ne contient pas `https://uri.paypal.com/services/reporting/search/read`) → contrôle `permission_present` ✗
      avec le remède, `provider_reachable` « non tenté » (38-7 AC-3, AC-8 : `permission_present` est la **portée du
      jeton** ; un 403 servi à la lecture des soldes, lui, ferait échouer `provider_reachable`, non `permission_present`)
      — sur une **seconde connexion** (avec son secret), ou après avoir remplacé le secret de la première : le cache de
      jeton, par connexion, n'est vidé que par le remplacement du secret, la suppression ou `/_test/reset` (38-7 AC-3),
      et un jeton déjà obtenu sur les réponses par défaut masquerait la portée manquante ; Comptable : pas d'entrée de
      menu, `/admin/providers` refusé, bouton « Télécharger maintenant » présent sur `/bank-accounts`. La recette E2E
      de `CLAUDE.md` gagne `KESH_SECRETS_KEY` (posée par 38-1 AC-10 ; vérifier qu'elle y est).
    - Gates : `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` ; E2E complet au dernier commit de code ;
      gate backend complet (une route et un dépôt neufs).
13. **AC-13 — Recette visuelle** (NFR-38-9a-3, `CLAUDE.md` § *Recette visuelle*). Captures, thème clair, base de
    démonstration, dans `kesh-gate-logs/` (hors dépôt), jointes au Dev Agent Record et au prompt de revue : liste vide ;
    liste avec une connexion de chaque état (`usable`, `secret_missing`, `secrets_key_missing`) ; formulaire de création
    (avec refus de date) ; bloc « Avancé » des surcharges ; secret absent, enregistré, après restauration ; résultat du
    test tout vert et avec ✗ `permission_present` ; téléchargement en cours et terminé (avec date bornée) ; soldes sans
    compte et dialogues « Créer » et « Lier » ; tableau des comptes prestataires (frais désigné et non désigné) ;
    contrôles (a) complet et incomplet, (b) concordant et en écart ; page des comptes bancaires avec badge et
    formulaire de modification d'un compte prestataire.
14. **AC-14 — Ce que la story ne touche pas.** Aucune migration ; aucune action d'audit neuve (les écritures passent
    par les routes de 38-1, 38-2, 38-3a, 38-3b, 38-7 qui auditent ; la route de l'AC-8 lit) ; **aucun outil MCP** (E23 :
    connexions, passages et contrôles ne sont pas exposés ; `kesh_ecritures` suffit pour les écritures) ; aucune
    donnée de la production de Beat dans les captures ni les tests.
15. **AC-15 — Les écrans existants que les gardes de 38-2 et 38-3a touchent** (FR-38-9a-7 ; dévolus à cette story par
    38-3a et 38-2 AC-7).
    - **Imports** (`frontend/src/lib/features/bank-import/bank-import-display.ts:14-36`, aujourd'hui code brut pour toute
      valeur hors `MANUAL`) : un import portant `providerConnection` (liste et détail de `GET /api/v1/bank-imports…`,
      38-3a) s'affiche « Téléchargé de <libellé de la connexion> » ; une valeur inconnue de `source_format` →
      « Source inconnue (`x`) » (lecture tolérante, E4). Le bouton *Supprimer l'import* est **absent** pour un tel import
      (le serveur refuse : `409 PROVIDER_IMPORT_NOT_DELETABLE`, libellé du code si le bouton est atteint par une liste
      périmée).
    - **Import de fichier et relevé saisi** : le sélecteur de compte des écrans d'import CAMT/CSV et du relevé manuel
      n'offre pas un compte prestataire ; `409 BANK_IMPORT_PROVIDER_ACCOUNT` (38-3a) libellé.
    - **Lot de paiement** : le sélecteur du compte débité (`payment-batches`) n'offre pas un compte prestataire ; `409
      PAYMENT_BATCH_ACCOUNT_IS_PROVIDER` (38-2 AC-7) libellé.
    - **Plan comptable** : `409 ACCOUNT_USED_BY_PROVIDER_ACCOUNT` (38-2 AC-5, archivage d'un compte de frais ou lié, sans
      confirmation possible) affiche son message serveur tel quel (AC-10).
    - Fichiers touchés : composants et sélecteurs existants, par un appel au module `provider-labels.ts` / un filtre sur
      `provider`, sans logique inline de plus ; captures de chaque état (AC-13).
## Écart à l'epic

- **E20 (b) sans route** : l'epic veut l'écart comptable « affiché à l'écran du compte » mais aucune story n'en porte la
  lecture (38-3b : (a) ; 38-10 : relevé joint) ; 38-9a l'ajoute (AC-8), ce qui lui fait toucher `kesh-db` et `kesh-api`
  en plus de `frontend` (quatre modules avec i18n et manuels : dans la règle de splitting).
- **Routes des soldes vus** (E12) : non nommées par l'epic ; arrêtées par 38-3b (`PATCH …/balances-seen`, `GET
  …/balances`, AC-7), sur la table de 38-3a, O12.
- **E2E** : l'epic n'en demande qu'en 38-9b ; un parcours d'administration est ajouté ici (AC-12) — c'est le seul test
  qui voie le secret ne pas traverser la frontière HTTP vers l'écran.

## Questions ouvertes

- ~~**Q-38-9a-1**~~ *(close, P2)* : relu à 38-3b AC-4 et AC-6 — le passage somme `skipped` dans `ignored_count` et ne
  garde pas la ventilation ; `GET …/runs` rend les colonnes de la table, `truncatedFrom` et `ignoredCount` compris.
  L'AC-5 affiche le total seul.
- **Q-38-9a-2** (Guy, Q1 de l'epic) : numéro de la release, pour la sous-section *Passer à la …* (AC-11).
- ~~**Q-38-9a-3**~~ *(close, P1)* : `accountEmail` retiré par O10.

## Tâches

- [ ] Fonctionnalité `provider-connections` : types, API, `provider-labels.ts` (AC-1, AC-10).
- [ ] Pages liste et fiche, entrée de menu `adminOnly` (AC-1).
- [ ] Formulaire de connexion, surcharges, suppression (AC-2).
- [ ] Composant du secret (AC-3).
- [ ] Test de connexion, téléchargement, état des passages (AC-4, AC-5).
- [ ] Soldes sans compte, dialogues « Créer » / « Lier » / « Ignorer » (AC-6).
- [ ] Comptes prestataires de la connexion, compte de frais, retrait (AC-7).
- [ ] Route `GET /bank-accounts/{id}/provider-balance-check`, dépôt, tests ; affichage (a) et (b) (AC-8).
- [ ] Page des comptes bancaires : badge, contrôles, bouton, formulaire de modification (AC-9).
- [ ] Écrans existants : imports « Téléchargé de », suppression masquée, sélecteurs d'import et de lot de paiement
      (AC-15).
- [ ] i18n quatre locales (AC-10).
- [ ] Manuel d'administration, PDF, contrôle du PDF aplati (AC-11).
- [ ] Vitest, test Rust, E2E (AC-12) ; captures (AC-13).

## Dev Notes

- **Patrons à suivre** : pages d'administration `routes/(app)/admin/backup/+page.svelte`, `…/admin/mcp/+page.svelte`
  (refus de rôle, formulaires) ; avertissements et textes à paires `-one`/`-other` :
  `lib/features/reconciliation/already-booked.ts` ; table de libellés de codes : `failed-proposal-label.ts` ; dates
  suisses : `formatSwissDate` (`lib/features/reports/reports.api`) ; option de délai : `CallOptions.timeoutMs`
  (`api-client.ts:61`).
- **Pourquoi pas de constante de trois ans au frontend** : la limite est propre à PayPal et vit dans le connecteur
  (38-7 AC-9) ; Wise et Stripe auront la leur. L'écran lit `earliestFetchFrom`.
- **Pourquoi la route (b) lit à la date de Zurich** : `booking_date` des transactions et `entry_date` des écritures sont
  des dates locales (38-8 AC-4) ; comparer un instant UTC à des dates ferait basculer les mouvements de fin de soirée.
- **Secrets** : le composant ne tient la valeur que dans un `$state` local au composant, remis à `''` dans un
  `finally` ; pas de `bind:value` vers un store.
- **Points de contact** : 38-1 (forme de réponse, `unusableReason`, contrôles du test, codes) ; 38-2 (routes et codes
  des comptes, bloc `provider`) ; 38-3a (colonnes du passage, table des soldes vus, en-tête `PROVIDER_API`,
  gardes d'import) ; 38-3b (fetch, passages, soldes rapportés et décision « ignorer » d'un solde vu, complétude,
  transport factice, route `_test`) ; 38-7 (`earliestFetchFrom`, contrôles distants, codes) ; 38-8 (surcharges) ; 38-9b (même vocabulaire,
  libellés des genres de mouvement partagés par `provider-labels.ts`).

### References

- `planning-artifacts/epic-38-paypal.md` § E1–E3, E10–E12, E19, E20, E24, 38-9a.
- `implementation-artifacts/38-1-secrets-et-connexions.md` AC-3 à AC-7, AC-10 ;
  `38-2-compte-prestataire.md` AC-3 à AC-5, AC-8 ; `38-7-client-paypal.md` AC-7 à AC-10 ;
  `38-8-codes-evenement-paypal.md` AC-7.
- `CLAUDE.md` § *Recette visuelle*, *E2E (Playwright)*, *Synchroniser TOUTES les docs*, *Migration breaking policy*.

## Change Log

- 2026-11-15 — rédaction (Opus).
- 2026-11-15 — validation P1 (Sonnet) : 15 findings (0 CRITICAL, 3 HIGH, 7 MEDIUM, 5 LOW) ; remédiés dans la fiche sauf
  les trois inter-fiches (38-9a-P1-13 : compteurs écartés et `truncated_from` absents de la ligne de passage de 38-3 ;
  -14 : codes d'erreur de passage de 38-3 contre 38-7 et 202 contre « erreur de `POST /fetch` » ; -15 : délai total de
  30 s en 38-3 contre 60 s en 38-7). Trend : P1 = 15. Axes non exercés : code de 38-1/38-2/38-3 (non écrit) ; PDF
  du manuel (non régénéré, hors fiche).
- 2026-11-15 — alignement inter-fiches P1 (Opus) : AC-5 — écartés (`ignored_count`, ventilation `skipped` si portée)
  et avis `truncated_from` affichés si 38-3b les rend ; AC-2 — `overlapDays` aux réglages avancés (1–30, défaut 3),
  aide de « télécharger depuis » sur le recouvrement réglable ; AC-10 — libellés des motifs `skipped` ; Q-38-9a-1
  réduite à la forme exacte à relire en 38-3b ; les inter-fiches P1-13 à P1-15 sont portées par la liste (P1-15 :
  bornes par requête, 60 s de 38-7, déjà prises par l'AC-4) ; découpage D-P1-1 : renvois à 38-3 portés sur 38-3a
  (colonnes, soldes vus et `PATCH …/balances-seen`, `PROVIDER_API`, gardes d'import) ou 38-3b (fetch, runs, `GET
  …/balances`, complétude, transport factice, `_test`).
- 2026-11-15 — validation P2 (Opus) : 12 findings (0 CRITICAL, 0 HIGH, 7 MEDIUM, 5 LOW) ; remédiés dans la fiche, aucun
  inter-fiches. MEDIUM : P2-1 corps de `settings` au `PATCH` (sans `payerId`, refusé par 38-1 AC-4 ; les trois champs du
  client toujours, `settings` remplaçant) ; P2-2 `version` rendue par « Tester » (38-7 AC-8) adoptée, sinon 409 au
  `PATCH` suivant ; P2-3 `PATCH …/balances-seen` posé par 38-3b AC-7, non 38-3a (en-tête, AC-6, Écart, Notes) ; P2-4
  prémisse de Q-38-9a-1 fausse — 38-3b ne garde que `ignored_count` (somme) : ventilation retirée, Q close ; P2-5 deux
  vocabulaires de codes (passage : `code()` en minuscules et codes propres ; connexion et test : codes d'API) et
  `errorDetail` `balances_unavailable:` ; P2-6 la route (b) ouvrait aux clés d'API le solde rapporté que 38-3b AC-7
  leur ferme — clés refusées ; P2-7 manuel : limites renvoyées ici par 38-2 (devise tenue par un compte archivé), 38-3a
  (`balance_seen` rend la connexion insupprimable), 38-8 (classement, file, surcharges) absentes de l'AC-11. LOW : P2-8
  cas de `complete: null` alignés sur 38-3b AC-7 ; P2-9 le registre ne balaie pas les `GET` (`audit_route_registry.rs:151`)
  : delta 0 ; P2-10 sens d'« Inexpliqué » (relevé − livres) inverse de `statementGap` (`bank_accounts.rs:2019-2024`) ;
  P2-11 transaction de fichier sans `booked_at` comptée par `booking_date` ; P2-12 rédaction (renvoi NFR-38-2-3,
  « le premier téléchargement », reste d'`accountEmail`, préalable multi-devises de l'E2E, raisons de refus libellées,
  suggestion d'un sous-compte de frais, date maximale). Relu au code : `+layout.svelte:147`, `api-client.ts:59-64`,
  `lib.rs:1017`, `:1229`, `admin-manual.tex:2538`, `:3299`, `:2425`, `bank-import-display.ts:14-36`,
  `audit_route_registry.rs:151`, `bank_accounts.rs:2019-2024`. Trend : P1 = 15 (3 HIGH, 7 MEDIUM) → P2 = 12 (0 HIGH, 7 MEDIUM). Axes non exercés : code
  des stories amont (non écrit) ; PDF du manuel (non régénéré).
- 2026-11-15 — alignement inter-fiches P2 (Opus) : Q-38-9a-1 (seul `ignoredCount`) et AC-6 (`PATCH …/balances-seen`
  de 38-3b AC-7) constatées déjà alignées par la P2 ; AC-2 — `payerId` remis à `null` quand `clientId` change (38-1
  AC-4), « Tester » le repose ; AC-6 — message de `PROVIDER_ACCOUNT_HAS_TRANSACTIONS` (« Lier », 38-2 AC-3) ; AC-11 —
  faits de 38-3b AC-8 (contrôle chaque heure, passage 24 h après le début du dernier, asynchrone, « interrompu »,
  blocage 6 h au plus après une sauvegarde importée, `overlapDays`, `truncated_from`, devise d'un compte archivé ou
  clos), retrait d'un compte créé par erreur (délier, archiver, retirer les réglages, supprimer la connexion), refus
  de « lier » un compte à transactions.
- 2026-11-15 — validation P3 (Sonnet) : 7 findings (0 CRITICAL, 0 HIGH, 2 MEDIUM, 5 LOW) ; remédiés dans la fiche sauf
  P3-7 (proposition, ci-dessous), aucun inter-fiches. MEDIUM : P3-1 la route (b) s'ouvrait à la Consultation alors que
  la même donnée, le solde rapporté, est réservée au Comptable et à l'Administrateur par 38-3b AC-7 (`GET …/balances`) —
  montée en `comptable_routes` (`lib.rs:524`), Consultation → 403, résumé masqué à l'écran (badge conservé) ; P3-2 le
  manuel utilisateur manquait : 38-3a AC-7 et 38-2 lui transmettent les refus visibles (import et relevé saisi refusés
  sur un compte prestataire, en-tête téléchargé non supprimable, lot de paiement) pour « le manuel utilisateur et
  d'administration », l'AC-11 ne visait que l'administration — bullet ajouté. LOW : P3-3 avertissement « ignorer » sous
  le champ des surcharges (38-8 AC-7 le renvoie à cet écran) ; P3-4 champs du passage écrits en snake_case, la route
  rend du camelCase (38-3b AC-6) ; P3-5 `clientId` (1 à 128, `[A-Za-z0-9_-]`) et motifs `invalid_type`/`invalid_format`
  de 38-1 AC-4 absents des refus affichés ; P3-6 E2E : le 403 passe par la variante `paypalResponses` de 38-3b ; P3-7
  taille : quinze AC sur cinq modules (`frontend`, `kesh-db`, `kesh-api`, `kesh-i18n`, manuels) — **coupure proposée,
  non appliquée** : 38-9a-1 (AC-1 à AC-5, AC-10 à AC-12 pour ces écrans) et 38-9a-2 (AC-6 à AC-9, AC-15 : comptes,
  contrôles, route (b)) ; décision de l'orchestrateur. Relu au code : `lib.rs:524`, `:1017`, `+layout.svelte:147`,
  `api-client.ts:59-64`, `bank-import-display.ts:14-36`, `admin-manual.tex:2538`, `:3299`, `bank_accounts.rs:2019-2024` ;
  aux fiches : 38-1 AC-4, AC-7 ; 38-2 AC-5, AC-8 ; 38-3a AC-7 ; 38-3b AC-5 à AC-8 ; 38-7 AC-8, AC-9 ; 38-8 AC-7 (tous
  concordent). Trend : P1 = 15 → P2 = 12 → P3 = 7 (0 HIGH, 2 MEDIUM). Axes non exercés : code des stories amont (non
  écrit) ; PDF du manuel (non régénéré) ; site web de PayPal (aucun WebFetch).
- 2026-11-15 — validation P4 ciblée (Opus) : 5 findings (0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW), tous remédiés dans la
  fiche ; périmètre : `git diff f005ba2f a5682b2e` et les trois points inter-fiches visant 38-9a. MEDIUM : P4-1 E2E
  (remédiation P3-6) — « un 403 servi à la lecture des soldes → `permission_present` ✗ » est faux : `permission_present`
  est la **portée du jeton** (38-7 AC-3, AC-8 ; table de l'AC-7 : « portée absente du jeton »), un 403 aux soldes ferait
  échouer `provider_reachable` ; remplacé par un jeton 200 dont le `scope` n'a pas `…/reporting/search/read`, sur une
  seconde connexion ou après remplacement du secret (le cache de jeton par connexion masquerait sinon la portée) ;
  P4-2 E2E — les réponses PayPal par défaut ne sont servies que sur `paypalResponses: "default"` (38-3b AC-3, 38-7
  AC-10 : sans elle, `FakeConnector`) : l'appel `POST /_test/provider-connector` est écrit, après la création de la
  connexion (état par `connectionId`). LOW : P4-3 instant affiché du contrôle (a) — `<as_of>` indéterminé → `reported.asOf`,
  l'`as_of` rendu par le connecteur (38-3b AC-7), en date et heure de Zurich, test ; même instant au (b) (« Selon PayPal,
  au <`asOf`> ») ; P4-4 avertissement « ignorer » : renvoi « (AC-8 a) » sorti du texte affiché, et la phrase ajoutée au
  manuel (AC-11, 38-8 AC-7 : « sous le champ et au manuel »), avec l'instant des contrôles ; P4-5 Vitest du rôle
  Consultation (route (b) non appelée, résumé absent, badge présent) qui manquait à la remédiation P3-1. Relu : 38-3b
  AC-3 (`paypalResponses`, `"default"`), AC-6, AC-7 ; 38-7 AC-3 (portée, cache), AC-7 (table), AC-8, AC-10 ; 38-8 AC-7 ;
  38-1 AC-4 (`clientId`, motifs) ; 38-3a AC-6, AC-7 (manuel utilisateur) ; `lib.rs:522-524`, `:1015-1017`. Remédiations
  P3 confirmées : P3-1, P3-2, P3-4, P3-5. Axes non exercés : code des stories amont (non écrit), PDF, documentation
  PayPal. Trend : P3 = 7 (2 MEDIUM) → P4 = 5 (2 MEDIUM) : une passe de plus est due (MEDIUM rendus, CLAUDE.md).
- 2026-11-15 — validation P5 ciblée (Sonnet) : 1 finding (0 CRITICAL, 0 HIGH, 1 MEDIUM, 0 LOW), remédié dans la fiche ;
  aucun autre inter-fiche. MEDIUM : P5-1 inter-fiche de la passe — le motif `opening_unavailable` d'`errorDetail`
  (38-3b AC-7 : `opening` antérieur au début effectif, lecture de `[A, D)` en échec, aucun solde d'ouverture écrit, passage
  `succeeded`) n'avait ni libellé ni état à l'écran : avis ajouté à l'AC-5 (à côté de `balances_unavailable:`, reste de
  `errorDetail` jamais affiché brut), `complete: null` de l'AC-8 (a) en dit la cause, cas ajouté aux Vitest de l'AC-12
  (quatre locales par l'AC-10). Remédiation P4 relue : 38-3b AC-3 (`paypalResponses`, `"default"`, état par
  `connectionId`), AC-7 ; 38-7 AC-3 (portée absente → `Permission` dès le jeton ; cache vidé par secret, suppression,
  `clear_transient_test_state`), AC-8 (`permission_present` = portée, contrôles suivants `ok: null`), AC-10 ;
  `lib.rs:522-524` ; la connexion multiple est permise (`uq_ppc_company_label`, 38-1) : seconde connexion réalisable —
  tous justes. Axes non exercés : code des stories amont, PDF, documentation PayPal. Trend : P4 = 5 (2 MEDIUM) → P5 =
  1 (1 MEDIUM) : une passe de plus est due (MEDIUM rendu, CLAUDE.md).
- 2026-11-15 — validation P6 ciblée (Opus) : 2 findings (0 CRITICAL, 0 HIGH, 1 MEDIUM, 1 LOW), remédiés dans la fiche ;
  aucun inter-fiche. Périmètre : `git diff 6ec31855 d6018f7d` de la fiche (remédiation P5) et le point inter-fiche du
  prompt. MEDIUM : P6-1 AC-5 (remédiation P5-1, inter-fiche 38-3b AC-7 de la P5) — l'avis se déclenchait sur
  « `errorDetail` commence par `opening_unavailable:` » ; or 38-3b AC-7 joint plusieurs motifs par `; `, dans l'ordre
  `balances_unavailable` puis `opening_unavailable` : dans un passage aux deux motifs, `opening_unavailable` n'est pas en
  tête et n'était jamais lu ; `opening_unavailable: span` (lecture non tentée, durable tant que PayPal sert le même `A`)
  recevait le texte « réessayé au passage suivant », qui promet ce qui n'arrivera pas, et « le `code()` qui suit » ne
  valait pas pour `span`. AC-5 réécrit : découpage sur `; `, un motif lu par préfixe, un avis par motif distinct, libellé
  propre à `span`, préfixe inconnu → libellé générique au seul préfixe, état du (a) lu de 38-3b et jamais déduit du
  motif ; AC-10 : motifs d'`errorDetail` dans `provider-labels.ts` ; AC-12 : Vitest (jointure, motif hors de tête,
  `span`, doublon, préfixe inconnu, `textContent` sans `auth` ni `span` — non `innerHTML`, qui porte des `<span>`).
  LOW : P6-2 AC-11 — la limite durable de `span` (38-3b AC-7) n'était pas au manuel : ajoutée à l'inventaire. Relu :
  38-3b AC-7 (`opening` rendu antérieur, borne de trois `max_window()`, `opening_unavailable: <code()>` / `: span`,
  jointure `; ` « dans cet ordre », « un par devise au plus », « 38-9a lit chaque motif séparément ») et son Change Log
  P5 (LOW 38-3b-P5-1, P5-2) ; AC-8 (a) de la fiche (`complete: null` sans `opening` : juste, l'`opening` n'étant pas
  écrit). Axes non exercés : code des stories amont (non écrit), PDF, documentation PayPal, reste de la fiche. Trend :
  P5 = 1 (1 MEDIUM) → P6 = 2 (1 MEDIUM) : une passe de plus est due (MEDIUM rendu, CLAUDE.md).
- 2026-11-15 — validation P7 ciblée (Sonnet) : 2 findings (0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW), remédiés dans la fiche ;
  aucun inter-fiche. Périmètre : `git diff d0b7c24f e7814a45` de la fiche (remédiation P6) ; point inter-fiche (motifs
  `opening_unavailable` / `: span`, jointure `; `) vérifié contre 38-3b AC-7 (lignes 337-347 : ordre, un par devise,
  constants, « 38-9a lit chaque motif séparément ») : appliqué et cohérent. LOW : P7-1 AC-5 — « motif distinct »
  ambigu (`opening_unavailable: auth` et `: transport` partagent un libellé) → un avis par libellé distinct, test
  complété ; P7-2 AC-5 — préfixe inconnu sans `:` : repli non défini → motif entier tronqué à 40 caractères, test
  complété. Relu sans défaut : renvois AC-5/8/10/11/12, état du (a) (`complete: null`), libellé `span` (durable),
  Vitest `textContent`. Axes non exercés : code amont, PDF, documentation PayPal, reste de la fiche. Trend : P6 = 2
  (1 MEDIUM) → P7 = 2 (0 MEDIUM) : aucun finding au-dessus de LOW, boucle clôturable pour cette fiche.
- 2026-11-15 — constats de la validation de l'Epic 40 (Opus) : AC-8 (b), `unreconciled` borné par
  `booked_at < asOf` (borne exclue) au lieu de `≤`, aligné sur la fenêtre demi-ouverte du contrôle de complétude
  (38-3b AC-7, relu : « `< as_of` du dernier `reported` … à l'instant exact du `reported` … jamais ») ; repli sans
  `booked_at` inchangé (journée de Zurich, inclusive, comme `ledger`) ; test Rust du mouvement à `booked_at = asOf`
  ajouté (AC-8, tâches de test). Garde G4-bis (`textes_coherents.rs`, lue : manuel utilisateur, guide de démarrage,
  quatre catalogues) : la fiche ne cite aucun numéro de compte du plan de Beat — rien à changer. À relire par une passe
  ciblée.
- 2026-11-15 — validation P8 ciblée (Sonnet) : 1 finding LOW, 0 au-dessus. Relu : `git diff be2ce1a9 d3abf458` de la
  fiche ; AC-8 (b) `booked_at < asOf` concorde avec 38-3b AC-7 (« `< as_of` du dernier `reported` … à l'instant exact
  du `reported` … jamais ») ; repli sans `booked_at` à la journée de Zurich cohérent avec `ledger` et avec la limite
  écrite (`ledger` à la journée, `reported`/`unreconciled` à l'instant) ; test Rust présent aux tâches. LOW 38-9a-P8-1 :
  ligne de la phrase de test trop longue, recoupée (rédaction). **Inter-fiche signalé, non appliqué** : 40-4 écrit encore
  `booked_at ≤ asOf` (lignes 58, 65, 118) et son test (a) de la ligne 146-148 attend la borne incluse, Q-40-4-2 ; à
  aligner sur `<` (son texte proposé, ligne ~300, est déjà celui de 38-9a). Axes non exercés : code amont, reste de la
  fiche. Trend : P7 = 2 (0 MEDIUM) → P8 = 1 LOW : boucle clôturable pour cette fiche.
- 2026-11-15 — préfixes i18n (constat de la validation de l'Epic 40, Opus) : le préfixe `provider-` sous
  `features/provider-connections/` aurait fait rougir `lint-i18n-ownership` (`lint-i18n-ownership.js:177-186`, nom à
  tiret → `startsWith('provider-connections-')`). AC-10 : clés de l'écran en `provider-connections-`, badge et
  `provider-labels.ts` dans cette fonctionnalité ; `nav-providers` → `nav-admin-providers` (patron `nav-admin-*`,
  `+layout.svelte`, hors du linter) ; textes neufs des autres fonctionnalités (AC-15 : `bank-import-display.ts` ;
  `payment-batches`, `bank-accounts`, `accounts`) au préfixe de la leur, ou lus par appel à `provider-labels.ts`.
  AC-1 renvoie à AC-10. Correction de rédaction, sans passe de plus (aucune règle métier touchée).
- 2026-11-15 — validation ciblée des préfixes i18n (Sonnet) : 2 findings (0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW), remédiés.
  Relu au code : `lint-i18n-ownership.js:16`, `:177-186` justes ; le linter ne balaie que `src/lib/features/`.
  38-9a-I18N-1 AC-10 : précisé que les pages de `routes/(app)/` ne sont pas contrôlées (préfixe d'usage), la règle ne
  liant que les fichiers sous `features/` ; 38-9a-I18N-2 AC-1 disait « groupe Paramètres » pour le groupe
  `nav-administration` (liste `adminOnly`, `+layout.svelte:147`) — corrigé. `nav-admin-providers` suit `nav-admin-backup`,
  `nav-admin-mcp` (liste `adminOnly`). Inter-fiches : 40-6 (lecture seule) cite encore `nav-providers` (Questions
  ouvertes, « Inter-fiches 38-9a (P2-3) », et `:185-187`) ; 38-9b, 38-6a, 38-6b : aucun renvoi faux.
