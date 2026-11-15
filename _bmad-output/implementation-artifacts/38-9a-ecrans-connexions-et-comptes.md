# Story 38-9a : écrans — connexions de prestataire, secrets, soldes proposés, comptes prestataires, contrôles de solde ; manuel d'administration

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 (Opus), branche `epic/38-paypal`. Epic 38 (`planning-artifacts/epic-38-paypal.md`),
E1, E2, E3, E10, E11, E12, E19, E20, E24 ; contrats C-38-1, C-38-2, C-38-3, C-38-5, C-38-12. Décisions B1, B2, B3,
B8, B9, B11, B12, B22 ; arbitrages O1, O10, O12. **Dépend de** 38-1 (routes des connexions et des secrets), 38-2
(comptes prestataires, bloc `provider` de `GET /bank-accounts` ; **porte la migration breaking**, O1), 38-3a (colonnes
de `provider_fetch_runs` dont `truncated_from` et `ignored_count`, soldes vus et `PATCH …/balances-seen/{currency}`,
en-tête `PROVIDER_API` et `providerConnection` des imports, gardes d'import), 38-3b (passage asynchrone
`POST …/fetch` → 202, `GET …/runs`, `GET …/balances` et contrôle des soldes, `GET /provider-fetch/connections`,
transport factice et route `_test`), 38-7 (« Tester la connexion »,
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
   - Entrée **« Prestataires de paiement »** dans le groupe Paramètres, liste `adminOnly` de
     `frontend/src/routes/(app)/+layout.svelte:147` (à côté de `/admin/backup`, `/admin/mcp`), lien `/admin/providers`.
   - Pages `frontend/src/routes/(app)/admin/providers/+page.svelte` (liste) et `…/admin/providers/[id]/+page.svelte`
     (fiche) ; code dans une fonctionnalité neuve **`frontend/src/lib/features/provider-connections/`** (`*.api.ts`,
     `*.types.ts` — miroir de `ProviderConnectionResponse` de 38-1 AC-3, composants, tests Vitest) ; la page des comptes
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
     de l'application REST ; aide : « non secret ; le secret se saisit à part ») ; `payerId` **en lecture seule**
     (posé par le serveur, 38-1 AC-4, 38-7 AC-8). **`accountEmail` n'est ni affiché ni saisi** : retiré du JSON par O10
     (aucune API de PayPal ne le rend) — si 38-1 le garde encore, le champ est ignoré à l'écran.
   - **Surcharges de codes** (38-8 AC-7), dans un bloc replié « Avancé » : paires code `T0000` (quatre chiffres, `^T\d{4}$`)
     → genre (liste fermée : vente, achat, frais, retrait, dépôt, litige, autre, ignorer ; ni conversion ni
     remboursement), ajout et retrait de lignes, **50 au plus** ; aide : « ne vaut que pour les codes que Kesh ne classe
     pas ; s'applique aux mouvements téléchargés ensuite, les mouvements déjà téléchargés se traitent à la main »
     (texte de 38-8 AC-7). Refus du serveur (`PROVIDER_SETTINGS_INVALID`, `details.field`, `details.code`,
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
   soldes lus (devise, total, au format suisse ; « lus à l'instant, non enregistrés »). Pendant l'appel : bouton
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
   - Résultat, lu de la ligne de passage (`GET …/runs` de 38-3b, colonnes de 38-3a) : nombres insérés
     (`inserted_count`), déjà présents (`known_count`), comptes non téléchargés faute de compte Kesh
     (`skipped_unlinked_count`, renvoi à AC-6), état et code d'erreur (libellés AC-10). S'y ajoutent, **si 38-3b les
     porte** (liste inter-fiches P1 ; forme exacte relue à la fiche 38-3b, *Questions ouvertes*, Q-38-9a-1) :
     - les **enregistrements écartés** par le classement de 38-8 (`ignored_count`, total) et, si la ligne les détaille,
       leur ventilation `skipped` — refusés (`denied`), en attente (`pending`, « relus au prochain téléchargement »),
       hors solde (`non_balance`) — en texte discret, jamais en erreur ; zéro → rien d'affiché ;
     - l'avis de **date de départ bornée** par l'historique de PayPal (38-7 AC-9, `truncated_from` non nul) : « PayPal
       ne remonte que trois ans : téléchargé depuis le <date suisse de `truncated_from`> ».
     Absents de la réponse : l'écran ne les affiche pas (aucune valeur inventée).
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
     {currency}` `{ ignored }` (38-3a, Administrateur, session ; **pas de `version`**). Noms arrêtés par 38-3a et
     38-3b (O12), plus à proposer.
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
7. **AC-7 — Comptes prestataires de la connexion** (FR-38-9a-4 ; `GET|PATCH|DELETE …/accounts` de 38-2 AC-5, AC-8).
   Tableau : devise, compte bancaire (nom, lien vers `/bank-accounts`), compte du plan (numéro, nom), **compte de
   frais de la devise** (numéro, nom ; « non désigné » en avertissement : les propositions de frais seront bloquées,
   `FEE_ACCOUNT_INVALID` de 38-6a), contrôles de solde (AC-8). Actions : « Changer le compte de frais » (`PATCH`
   `{ version, feeAccountId }` ; sélecteur : comptes de charge actifs imputables **de la même devise** ; aide : « le
   regroupement des frais est le compte parent de ce compte dans le plan », B3) ; « Retirer les réglages » (`DELETE`,
   confirmation destructive : « le compte bancaire reste, il n'est plus alimenté par PayPal ») ; refus
   `PROVIDER_ACCOUNT_IN_USE` → les raisons de `details.reasons`, une par ligne.
8. **AC-8 — Contrôles de solde** (FR-38-9a-5, E20).
   - **(a) Complétude** (contrôle des soldes de 38-3b, C-38-5) : par compte, lue de `GET /provider-connections/{id}/balances` →
     `accounts[]` `{ bankAccountId, currency, opening, reported, expected, gap, complete }` : « Téléchargement complet au
     <`reported.asOf`> » (`complete: true`) ou **« Téléchargement incomplet : écart de <`gap`, devise> au <as_of> »** en
     avertissement (`complete: false`), avec le renvoi « Relancez un téléchargement ; si l'écart demeure, voir le
     manuel » ; **`complete: null`** (compte sans solde d'ouverture ou sans solde rapporté, ou passage en
     `balances_unavailable`) → « Contrôle non disponible » en gris, jamais en vert ; donnée lue telle que 38-3b la
     calcule (jamais recalculée à l'écran).
   - **(b) Écart comptable** : *Écart à l'epic* — E20 (b) n'est porté par aucune route des fiches 38-1 à 38-3b ; cette
     story l'ajoute en **lecture seule** : `GET /api/v1/bank-accounts/{id}/provider-balance-check` (`authenticated_routes`,
     `crates/kesh-api/src/lib.rs:1017`, comme la lecture `GET /api/v1/bank-accounts` de `:1229` ; tout rôle, clés d'API
     admises en lecture), réponse
     `{ currency, asOf, reported, ledger, unreconciled, unexplained }` ou `404` si le compte n'est pas prestataire ou
     n'a aucun solde rapporté :
     - `reported` = dernier solde rapporté de la devise (`provider_balance_snapshots`, `kind = 'reported'`, table de 38-3a écrite par 38-3b), `asOf`
       son instant (UTC) ;
     - `ledger` = solde **en devise** du compte du plan lié, **par `kesh_db::repositories::account_balances::
       balances_as_of(conn, company_id, date de asOf à Europe/Zurich, Some(&[compte]))`** (37-4 AC-1 quater : borne
       inclusive, débit net brut CHF et en devise, signé comme un actif : Σ débit − crédit) — **aucune seconde somme
       `amount_currency`** (DRY avec 38-10, C-38-10-4) ; montant CHF pour un compte en CHF ;
     - `unreconciled` = Σ `amount` des transactions `pending` du compte (rejetées et « examinées » comprises : elles
       restent `pending`, 38-6b C-38-6b-6) dont **`booked_at ≤ asOf`** (l'instant, comme le contrôle de complétude de
       38-3b, non `booking_date`) ;
     - `unexplained` = `reported − ledger − unreconciled`. ⚠️ **Limite écrite** (doc-comment, manuel) : `ledger` est
       arrêté à la **journée** de Zurich de `asOf`, `reported` et `unreconciled` à l'**instant** ; un mouvement de la même
       journée postérieur à `asOf` et déjà rapproché fait apparaître un `unexplained` transitoire, résorbé au passage
       suivant — test à deux mouvements de part et d'autre de `asOf`.
     Écran : « Selon PayPal : X · Aux livres : Y · Non rapproché : Z · **Inexpliqué : W** » (W ≠ 0 en avertissement,
     W = 0 en vert) sur la ligne du compte (AC-7) et sur la page des comptes bancaires (AC-9). Code dans un module de
     lecture neuf (`kesh-db::repositories::provider_balance_check`, une fonction), route dans
     `routes/provider_connections.rs` ou un fichier voisin — **pas** dans `routes/bank_accounts.rs`. Registre (C-38-12) :
     route de lecture, hors `LIB_ROUTES` des mutantes ; décompte des routes en delta (+1 lecture) si le registre les
     compte. Tests : compte EUR à transactions rapprochées et non rapprochées, écart construit, compte non prestataire →
     404, autre société → 404, Consultation → 200.
   - Le **relevé joint** et son solde de clôture (E20, contrôle de clôture) sont à 38-10, pas ici.
9. **AC-9 — Page des comptes bancaires** (FR-38-9a-6 ; bloc `provider` de 38-2 AC-8).
   - Ligne d'un compte prestataire : badge « PayPal · <libellé de connexion> · <devise> » (composant
     `ProviderAccountBadge.svelte`), contrôle de solde (AC-8 b) en résumé, bouton « Télécharger maintenant » (AC-5) pour
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
10. **AC-10 — Textes et i18n** (NFR-38-9a-2, NFR-38-9a-4). Clés neuves au préfixe `provider-` (écran) et
    `nav-providers`, **écrites en toutes lettres** (pas de clé construite par gabarit, `i18n-keys.test.ts`), aux
    quatre catalogues `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`, consommées par `i18nMsg` avec
    repli français (`frontend/src/lib/shared/utils/i18n.svelte`) ; **pas de sélecteur Fluent** (`{ $n -> … }`) — le
    client reçoit les messages pré-résolus (raison écrite dans `frontend/src/lib/features/reconciliation/already-booked.ts`) :
    paires `-one` / `-other` au besoin. Libellés des **codes** (`unusableReason`, `check`, `lastRunStatus`, codes
    d'erreur de 38-1, 38-2, 38-3a, 38-3b, 38-7, motifs d'écart `skipped`, genres de mouvement des surcharges) dans un module unique
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
    - **Mise à jour** (E19, NFR-38-3-2 de l'ex-38-3, désormais en 38-3a) : sous-section `\subsubsection{Passer à la <X.Y.Z>}` dans
      `\section{Mise à jour de Kesh}` (patron de `\subsubsection{Passer à la 0.17.6}`, ligne 2538), **numéro de Q1**
      (Guy) : **sauvegarde manuelle complète obligatoire avant la mise à jour** (base, `.keshbackup`, dossier des
      documents) et **seul chemin de retour** ; la migration breaking, **celle de 38-2** (O1) : la contrainte
      « un compte bancaire a un IBAN » remplacée (un compte prestataire n'en a pas), `match_type` des règles élargi,
      `kesh_version_min_required` relevé ; **aucun DDL** pour `source_format = 'PROVIDER_API'` (valeur écrite dès la
      première téléchargement, que la version précédente ne sait plus lire) ; refus de la version précédente sur la base
      migrée et sur une sauvegarde de la nouvelle ; sauvegarde `backup/pre-upgrade/` écrite par Kesh (\S\ref{sec:sauvegarde-avant-maj}) ;
      variable neuve `KESH_SECRETS_KEY` (renvoi). Le texte **ne contient pas** de numéro de version tant que Q1 n'est pas
      tranchée : la sous-section s'écrit au développement si le numéro est connu, sinon la story attend (aucun
      gabarit `<…>` dans le `.tex`).
    - PDF régénéré (`make fr` dans `docs/manual/`) et commité ; contrôle du **PDF aplati** (`pdftotext admin-manual.pdf -
      | tr '\n' ' ' | tr -s ' '`) pour les phrases clés (« Transaction Search », « ressaisir », « seul chemin de
      retour »), résultat au Change Log. Manuels DE/IT/EN : « à traduire » noté si l'écart existe.
12. **AC-12 — Tests** (Vitest, `npm run test:unit` ; E2E Playwright).
    - Vitest : formulaire de connexion (validation, `earliest`, surcharges, `overlapDays` et son refus `out_of_range`,
      refus `details`), résultat d'un passage (écartés et `truncated_from` présents puis absents), composant du secret
      (vidé après succès et après échec ; jamais dans un `data-*` ni dans le DOM rendu : rendu du composant après saisie
      puis enregistrement, `container.innerHTML` ne contient pas la valeur marqueur), liste de contrôles du test
      (chaque `check`, `ok: null`), libellés (`provider-labels.ts` : chaque code connu a un libellé, code inconnu → brut),
      bloc des soldes sans compte (actions selon le rôle), badge et formulaire des comptes bancaires (champs masqués
      pour un compte prestataire, présents pour un compte ordinaire et une carte), contrôle (b) (couleurs, signes).
    - Rust : la route de l'AC-8 (cas écrits à l'AC-8, dont le mouvement de part et d'autre de `asOf`).
    - Vitest de l'AC-15 : libellé « Téléchargé de <libellé> » (`providerConnection` non nul), valeur inconnue de
      `source_format` → « Source inconnue (`x`) », bouton *Supprimer* absent pour un import de prestataire,
      `PROVIDER_IMPORT_NOT_DELETABLE` et `BANK_IMPORT_PROVIDER_ACCOUNT` libellés, sélecteurs sans compte prestataire.
    - Vitest du suivi de passage (AC-5) : 202 puis `running` puis `succeeded` ; `PROVIDER_FETCH_IN_PROGRESS` → suit
      `details.runId` ; abandon de la page → plus d'appel.
    - **E2E** `frontend/tests/e2e/provider-connections.spec.ts`, contre le **transport factice** de 38-3b et ses
      réponses PayPal par défaut (38-7 AC-10) : Administrateur crée une connexion, dépose un secret marqueur, teste
      (✓ partout), télécharge, voit les soldes CHF/EUR/USD sans compte, crée le compte EUR (compte du plan EUR du seed
      ou créé par l'API dans le test), voit le badge sur `/bank-accounts` et le contrôle (b) ; **le secret marqueur
      n'apparaît jamais** dans `page.content()` ni dans les réponses interceptées (`page.on('response')`) ; route
      `_test` de 38-3b servant un 403 → contrôle `permission_present` ✗ avec le remède ; Comptable : pas d'entrée de
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
- **Routes des soldes vus** (E12) : non nommées par l'epic ; arrêtées par 38-3a (`PATCH …/balances-seen`) et 38-3b
  (`GET …/balances`), O12.
- **E2E** : l'epic n'en demande qu'en 38-9b ; un parcours d'administration est ajouté ici (AC-12) — c'est le seul test
  qui voie le secret ne pas traverser la frontière HTTP vers l'écran.

## Questions ouvertes

- **Q-38-9a-1** *(réduite, P1 ; presque close par la liste inter-fiches)* : routes et droits **arrêtés** (O12) ; la
  liste inter-fiches P1 ajoute à `provider_fetch_runs` (38-3a) `truncated_from` et `ignored_count`, rendus par
  `GET …/runs` (38-3b), et le passage écrit `skipped { denied, pending, non_balance }` à son état. Reste à relire à la
  fiche 38-3b la forme exacte (noms camelCase, `skipped` ventilé ou seul `ignored_count`) ; l'AC-5 n'affiche que ce
  que la réponse porte.
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
  des comptes, bloc `provider`) ; 38-3a (colonnes du passage, soldes vus et leur décision, en-tête `PROVIDER_API`,
  gardes d'import) ; 38-3b (fetch, passages, soldes rapportés, complétude, transport factice, route `_test`) ; 38-7 (`earliestFetchFrom`, contrôles distants, codes) ; 38-8 (surcharges) ; 38-9b (même vocabulaire,
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
