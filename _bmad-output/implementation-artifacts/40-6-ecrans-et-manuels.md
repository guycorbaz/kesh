# Story 40-6 : écrans et manuels Stripe — connexion et clé restreinte, boutique, versements, écart ventilé ; E2E

Status: superseded — remplacée le 2026-11-20 par la refonte KISS de l'Epic 40 (voir le registre du fichier d'epic) ; fiche gardée comme historique

- **Epic** : 40 (`_bmad-output/planning-artifacts/epic-40-stripe.md`, § *40-6*, G4, G8, G10, G12, G13, G14, G17, G19,
  C-40-1 à C-40-6) ; décisions de Beat B3, B6, B10, B13, B17, B20, B21 (`epics-38-40-cadrage.md`), qui priment ;
  décision de l'orchestrateur **P40-1** (clés `rk_live_` et `rk_test_` admises, `sk_` refusée, mode montré).
- **Dépend de** : **40-1** (réglages `StripeSettings`, secret `restrictedKey`, refus `PROVIDER_SECRET_NOT_RESTRICTED`
  et ses motifs, mode et `PROVIDER_MODE_CHANGED`, compte gardé et `PROVIDER_ACCOUNT_CHANGED`,
  `PROVIDER_ACCOUNT_UNREADABLE`, contrôles de « Tester » `mode_unchanged`, `permission_present`, `account_readable`,
  `account_unchanged`, `provider_reachable`, réponse `accountId`/`livemode`/`apiVersion`, jeu factice `stripeResponses: "default"`), **40-2**
  (codes courts `stripe::event_codes`, surcharges et leurs refus), **40-3** (versement : `value_date`, écriture datée de
  l'arrivée, ligne UBS rattachée, échecs à la file), **40-4** (`inTransitPayouts`, `otherDateShifts`, `available` et
  `pending` à « Tester », détail de K10, tuile de l'accueil), **40-5** (rapports CSV aux relevés joints) ; **38-9a** et
  **38-9b** (écrans génériques des prestataires, qu'elle complète sans les recopier).
- **Migration** : **aucune** (C-40-0).
- **Modules** : `frontend`, `kesh-i18n`, `docs/manual`, E2E (`frontend/tests/e2e`) ; un test de concordance dans
  `crates/kesh-api/tests/textes_coherents.rs` (AC-6).

## Story

En tant qu'**Administrateur**, je veux créer et régler une connexion Stripe à l'écran, y déposer une **clé restreinte**
sans jamais la revoir — et être arrêté net si je colle une clé secrète —, voir en quel mode elle travaille, choisir où
Kesh lit le numéro de facture de mes boutiques, et lire au manuel quels droits donner à la clé ; et, en tant que
**Comptable**, voir pour un versement Stripe sa sortie et son arrivée en banque, l'écart de mon compte « Débiteurs
Stripe » ventilé (non rapproché, versements en route, inexpliqué) et les codes Stripe à employer dans mes règles —
afin de tenir les comptes Stripe sans saisie, en sachant ce qui est une créance et ce qui est en route.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-40-6-1` | Créer et régler une connexion Stripe, clé en saisie seule, refus d'une clé non restreinte lisible. | AC-1, AC-2, AC-3 |
| `FR-40-6-2` | Choisir la source de la boutique (`invoiceRefSource`). | AC-1 |
| `FR-40-6-3` | Voir, pour un versement, sa sortie et son arrivée, et sa proposition datée de l'arrivée. | AC-4 |
| `FR-40-6-4` | Voir l'écart ventilé et `available` / `pending` d'un compte Stripe. | AC-3, AC-5 |
| `FR-40-6-5` | Joindre un rapport CSV ; codes courts Stripe en aide du critère « code d'événement ». | AC-6, AC-8 (CSV : écran livré par 40-5) |
| `NFR-40-6-1` | Aucun secret au DOM ni dans les réponses lues. | AC-2, AC-9 |
| `NFR-40-6-2` | Quatre locales ; manuels d'administration et utilisateur à jour (`.tex` seulement ; PDF régénérés à la release, rendu contrôlé aplati en local, non commité). | AC-7, AC-8 |
| `NFR-40-6-3` | E2E contre le transport factice (`stripeResponses: "default"`) : connexion → test → téléchargement → vente proposée et acceptée → versement rattaché à la ligne UBS. | AC-9 |
| `NFR-40-6-4` | Captures d'écran de chaque état touché (*Recette visuelle*). | AC-10 |
| `NFR-40-6-5` *(ajoutée)* | Textes « télécharger », « proposer », jamais « comptabiliser automatiquement » ; écrans PayPal inchangés (G18). | AC-1, AC-4, AC-7 |

## Ce que le code dit (prémisses vérifiées le 2026-11-15 sur `a2b1da9c`)

- Les écrans de prestataire n'existent pas encore : ils viennent de **38-9a** (fonctionnalité
  `frontend/src/lib/features/provider-connections/`, pages `/admin/providers`, `provider-labels.ts`, fonction pure
  `providerSettingsBody(form)`, bloc du secret, « Tester », contrôles (a) et (b), `ProviderAccountBadge.svelte`) et de
  **38-9b** (`ProviderProposalPreview.svelte`, `rules-labels.ts`, aide du critère `event_code_prefix` « Code PayPal, p. ex.
  `T0006` », AC-1). Cette story s'y branche ; ses noms sont ceux de ces fiches.
- Relevés joints : `BankAccountDocumentsDialog.svelte` (38-10 AC-9), étendu aux CSV par **40-5** (AC-6) ; panneau des
  contrôles de clôture `frontend/src/lib/features/fiscal-years/FiscalYearChecks.svelte` (existant ; K10 par 38-10).
- Manuels : `docs/manual/fr/admin-manual.tex` (`\section{Le serveur MCP}` ligne 3299, après laquelle 38-9a AC-11 pose
  « Prestataires de paiement (PayPal) ») ; `docs/manual/fr/user-manual.tex` (`\section{Réconciliation bancaire}` ligne
  3004, sous laquelle 38-9b AC-10 pose « Comptes de prestataires de paiement (PayPal) » ; tuile des comptes bancaires
  `:318-356`). Rendu local par `make fr` dans `docs/manual/` : PDF non commité, régénéré à la release.
- Garde des textes : `crates/kesh-api/tests/textes_coherents.rs` lit déjà manuels et catalogues (`manuels_fr`, `:54`,
  `catalogue`, `:109`) — lieu d'une garde de concordance de plus.
- Restriction des clés Stripe (<https://docs.stripe.com/keys/restricted-api-keys>, relu le 2026-11-15) : préfixe
  `rk_live_` / `rk_test_` ; droits *Aucun* / *Lecture* / *Écriture* par ressource, *Aucun* par défaut, `GET` = lecture ;
  création par *Développeurs → Clés API → Créer une clé restreinte*, valeur non relisible ensuite ; les noms exacts des
  ressources à cocher ne sont pas dans cette page (Q-40-8, recette 40-0).

## Écarts à l'epic

1. **CSV aux *Relevés joints*** (FR-40-6-5) : l'écran est livré par **40-5** (AC-6, avec ses Vitest, E2E et captures) ;
   40-6 n'y ajoute que le manuel (AC-8). Aucune double écriture.
2. **Tuile de l'accueil** (créance hors total) : livrée par **40-4** (AC-7, AC-9) ; 40-6 l'écrit au manuel.
3. ~~**Clé de test**~~ — plus un écart : l'epic est aligné sur P40-1 (G4, C-40-1, C-40-3, FR-40-1-2, § *Validation
   P1*) ; `rk_test_` est admise, l'écran dit le mode (AC-1, AC-2) au lieu de refuser la clé.
4. **Un test Rust** (AC-6) dans une story d'écrans : la liste des codes courts écrite au TypeScript et au manuel doit
   égaler la table de 40-2 ; seul un test qui lit les deux côtés le garantit.

## Critères d'acceptation

1. **AC-1 — Connexion Stripe : création et réglages (FR-40-6-1, FR-40-6-2, NFR-40-6-5).** Fiche de connexion de 38-9a
   AC-2 :
   - « Stripe » dans la liste des prestataires à la création (non modifiable ensuite) ; « Télécharger depuis » sans minimum
     (`earliestFetchFrom` `null` pour Stripe, 40-1 AC-2) ;
   - **réglages Stripe**, par un composant à part `StripeSettingsFields.svelte` (le formulaire hôte ne reçoit que l'appel et
     le bras du prestataire) :
     - **Numéro de facture de la boutique** (`invoiceRefSource`) : choix « Métadonnée du paiement » (champ « clé », défaut
       `invoice`) ou « Description du paiement » ; aide : « Où votre boutique (WHMCS, CB Paid Subscriptions…) inscrit le
       numéro de facture dans Stripe. Si la clé manque sur un paiement, aucune règle de boutique ne s'y applique : Kesh ne
       devine pas. » ; le corps envoie `metadata:<clé>` ou `description` ; aucune expression régulière recopiée de 40-1 :
       le refus `PROVIDER_SETTINGS_INVALID` (`details.field = "invoiceRefSource"`, `reason` : `invalid_format`, 40-1 AC-2)
       s'affiche sous le champ, un libellé par motif (motif inconnu : message du code) ;
     - **Recouvrement** (`overlapDays`) et **surcharges de codes** (`eventKindOverrides`) dans le bloc « Avancé » de
       38-9a, mêmes règles d'écran, à deux différences : la clé est un **code court** choisi dans la liste des codes
       surchargeables (`STRIPE_OVERRIDABLE_CODES` de `provider-labels.ts`, AC-6 ; 40-2 AC-6), non un `T0000` ; un refus
       `code_not_overridable` ou `malformed`, `kind_not_allowed` ou `too_many` (40-2 AC-6) s'affiche à la ligne ou au bloc, un libellé par motif ; la valeur se
       choisit parmi les genres admis par 40-2 AC-6 (`conversion` et `refund` absents) ;
     - **posés par le serveur, en lecture seule** : « Compte Stripe » (`accountId` ; « non lu » tant que `null` — ni
       « Tester » réussi, ni passage qui ait écrit une page (40-1 AC-9 : `Pose` au test ou avec la première page d'un
       passage) ; depuis D40-P3-1 de 40-1, un test ou un passage refuse une clé sans droit de lire le compte
       (`account_readable`, `PROVIDER_ACCOUNT_UNREADABLE`), si bien que `null` ne vaut plus « test réussi, droit
       absent »), « Version de l'API » (`apiVersion`), **mode** (`livemode` : badge « Mode test »
       ou « Mode réel » ; `null` → « Mode : à établir — lancez « Tester la connexion » »).
   - `providerSettingsBody(form)` (38-9a AC-2) gagne le bras Stripe : il envoie **toujours** `overlapDays`,
     `invoiceRefSource`, `eventKindOverrides`, **jamais** `accountId`, `apiVersion`, `livemode` (40-1 AC-2 : refusés du
     client). Vitest : réglages lus avec les trois champs du serveur → corps sans eux ; champs non touchés → corps qui les
     porte inchangés ; bras PayPal inchangé (ses tests de 38-9a verts sans retouche).
2. **AC-2 — Clé restreinte en saisie seule (FR-40-6-1, NFR-40-6-1).** Bloc du secret de 38-9a AC-3, pour Stripe :
   libellé « Clé restreinte Stripe », corps `{ restrictedKey }` (40-1 AC-4), mêmes garanties (jamais pré-rempli, vidé
   après succès **et** échec, aucun store, journal, `toast` ni `data-*`).
   - **Contrôle avant envoi** (fonction pure `stripeKeyRefusal(value) -> reason | null`, `provider-labels.ts`), sur la
     valeur **sans espaces ni fin de ligne en tête et en queue** (un collage en porte souvent ; la valeur rognée est
     celle qui est envoyée — le serveur, lui, ne rogne pas : 40-1 AC-4 n'admet que `[A-Za-z0-9]` après le préfixe) : une
     valeur qui commence par `sk_` est refusée **sans être envoyée** (une clé qui peut tout faire ne quitte pas le
     navigateur), motif `secret_key` — comme au serveur, qui motive `secret_key` pour **tout** préfixe `sk_`
     (`sk_live_`, `sk_test_` ou autre, 40-1 AC-4) ;
     `pk_` → `publishable_key` ; tout préfixe autre que `rk_live_` / `rk_test_` →
     `unknown_prefix` — motifs de 40-1 AC-4, le serveur restant l'autorité. Le champ est alors vidé.
   - Messages, par motif (local ou `400 PROVIDER_SECRET_NOT_RESTRICTED`, `details.reason`) : `secret_key` → « C'est une
     clé secrète (sk_…) : elle donne tous les droits sur votre compte Stripe. Kesh ne l'accepte pas. Créez une clé
     restreinte en lecture seule (voir le manuel d'administration) et, par prudence, faites tourner cette clé secrète
     dans Stripe. » ; `publishable_key` → « C'est une clé publiable (pk_…) : elle ne lit pas vos transactions. » ;
     `unknown_prefix` → « Une clé restreinte Stripe commence par rk_live_ ou rk_test_. » ; motif inconnu → message du code.
   - Après dépôt d'une clé `rk_test_` : avis « Clé de test : Kesh lira votre compte Stripe en mode test (aucun argent
     réel) », tiré du **seul préfixe** lu avant que le champ soit vidé (un booléen d'état du composant, jamais la valeur ni
     un fragment) ; il disparaît au rechargement. Le **badge** de mode, lui, est celui de la connexion (AC-1), lu du
     serveur : « à établir » tant que ni « Tester » ni un passage ne l'a posé (le dépôt ne pose pas `livemode`, 40-1 AC-5,
     AC-9).
   - Vitest : `sk_live_…` → aucun appel réseau (espion sur `fetch`), champ vidé, message ; ` sk_live_…\n` (espaces et
     fin de ligne) → de même ; `rk_test_…\n` → envoyé sans la fin de ligne ; `rk_test_…` envoyé ; la valeur
     marqueur absente de `container.innerHTML` après succès et après refus.
3. **AC-3 — « Tester la connexion » Stripe (FR-40-6-1, FR-40-6-4).** Liste de contrôles de 38-9a AC-4 : libellés de
   `mode_unchanged` (« Même mode que les téléchargements passés »), de `account_unchanged` (« Même compte Stripe que les
   téléchargements passés », 40-1 AC-9) et, pour Stripe, de `permission_present` (« Droit de
   lire les transactions de solde et leurs paiements ») ; remèdes : `PROVIDER_AUTH_FAILED` → « Clé refusée par Stripe :
   elle a pu être révoquée ; créez-en une et remplacez-la » ; `PROVIDER_PERMISSION_MISSING` → « Dans Stripe, donnez à la
   clé le droit *Lecture* sur les ressources listées au manuel » ; `PROVIDER_MODE_CHANGED` → « Cette clé n'est pas du même
   mode que les mouvements déjà téléchargés : créez une connexion neuve pour changer de mode » ;
   `PROVIDER_ACCOUNT_CHANGED` → « Cette clé est celle d'un autre compte Stripe que les mouvements déjà téléchargés :
   déposez la clé du bon compte, ou créez une connexion neuve » ; **clé sans droit de lire le compte** (D40-P3-1, 40-1
   AC-9) : contrôle **`account_readable`** (libellé « Droit de lire le compte Stripe », listé après `permission_present`
   et avant `account_unchanged`, dans l'ordre de 40-1 AC-9) et code `PROVIDER_ACCOUNT_UNREADABLE` → « Cette clé ne peut
   pas lire le compte Stripe : ajoutez le droit de lecture du compte » (texte de `provider-error-provider-account-unreadable`,
   40-1 AC-14). Les trois codes `PROVIDER_MODE_CHANGED`, `PROVIDER_ACCOUNT_CHANGED` et `PROVIDER_ACCOUNT_UNREADABLE`
   se lisent aussi dans **`lastError`** de la connexion (table des codes d'API de 38-9a AC-5) : un passage refusé pour
   le compte (40-1 AC-9 : `errorCode` `permission` ou `invalid_response`, `errorDetail` `account`) laisse
   `lastError = PROVIDER_ACCOUNT_UNREADABLE` ou `PROVIDER_ACCOUNT_CHANGED`, rendu par le même libellé qu'à « Tester ».
   Succès : compte Stripe,
   mode, version, et par devise : **total**, **disponible** (`available`) et **en attente** (`pending`) quand présents
   (40-4 AC-5), au format suisse, avec « lus à l'instant, non enregistrés » ; une devise sans compte Kesh renvoie au bloc
   des soldes sans compte (38-9a AC-6). `provider-labels.ts` : chaque contrôle, chaque code et chaque **motif
   d'`errorDetail`** neufs (40-1 AC-5, AC-6, AC-7, AC-9, AC-10 : `livemode`, `pagination`, `value_date`, `currency`,
   `account`, … ; liste relevée par `grep -n 'reason: "' ` des fiches 40-1 à 40-4 au développement) ont leur libellé,
   et `provider_reachable` le sien (test de table de 38-9a étendu ; code ou motif inconnu → rendu brut, jamais vide). Vitest : réponse Stripe complète ; `accountId: null` (connexion jamais testée : « non lu ») ; ✗
   `mode_unchanged` ; ✗ `account_readable` avec `PROVIDER_ACCOUNT_UNREADABLE` (libellé, jamais vide) ; connexion à
   `lastError: "PROVIDER_ACCOUNT_UNREADABLE"` → même libellé ; passage à `errorDetail: "account"` → libellé du motif.
4. **AC-4 — Versement : sortie, arrivée, proposition (FR-40-6-3, NFR-40-6-5).** Dans `ProviderProposalPreview.svelte`
   (38-9b AC-3), bras **retrait** : quand la transaction porte `valueDate` (**déjà rendu** au DTO de `GET …/proposals` :
   `transaction.valueDate`, `TransactionSummary.value_date`, `crates/kesh-api/src/routes/reconciliation.rs:172-176` ;
   40-3 AC-3 n'y ajoute que `entryDate`), l'en-tête dit « Sorti de Stripe le <`bookingDate`> · arrivée en banque
   prévue le <`valueDate`> » et l'aperçu « Écriture proposée au <`entryDate`> » (`entryDate` de la proposition
   `provider` de retrait, 40-3 AC-3) ; le compte bancaire
   suggéré (`transferBankAccountId`, cherché à partir de l'arrivée, C-40-5) et l'écriture existante à rattacher (voie a)
   gardent le rendu de 38-9b. Sans `valueDate` (PayPal) : rendu de 38-9b inchangé (Vitest). File « à examiner » : les
   codes `PAYOUT_FAIL`, `PAYOUT_CANCEL` portent l'aide « Le versement est revenu sur le solde Stripe ; s'il avait été
   rapproché, annulez ce rapprochement, ou rapprochez l'échec par une écriture manuelle ; après une annulation, marquez
   versement et échec examinés » et le lien vers le versement d'origine (`relatedBankTransactionId` de l'élément de la file, 40-3 AC-7 ; `null` → « versement
   d'origine hors de Kesh », sans lien) ; un litige (`DISPUTE`, `DISPUTE_REV`) « à examiner, sans proposition » (B7). Libellés de genre et de code inconnus :
   lecture tolérante de 38-9b. Vitest : retrait avec et sans `valueDate`, échec de versement.
5. **AC-5 — Écart ventilé et K10 (FR-40-6-4).** Résumé du contrôle (b) (38-9a AC-8 b, ligne du compte et page des
   comptes bancaires) : « Selon Stripe, au <`asOf`> : X · Aux livres : Y · Non rapproché : Z · **Versements en route :
   T** · Autres décalages de date : O · Inexpliqué : W », les deux termes neufs **seulement quand présents** (40-4 AC-2 :
   absents = 0) ; l'écran ne recalcule rien ; une infobulle sur « Versements en route » : « Partis de Stripe, pas encore
   arrivés en banque selon leur écriture : c'est encore une créance sur Stripe ». Les termes se montrent **dans le signe où le serveur
   les rend** (un versement en route : `−300.00`, 40-4 Dev Notes) ; le manuel dit le sens. « Selon Stripe » au lieu de « Selon
   PayPal » par le libellé du prestataire (`provider-labels.ts`, jamais un nom écrit en dur ; PayPal inchangé, Vitest).
   **Inventaire des textes de 38-9a/38-9b qui nomment PayPal** (relevé par `grep -no "«[^»]*PayPal[^»]*»"` des deux
   fiches, refait au développement sur le code livré) — chacun passe par le libellé du prestataire, rendu PayPal
   identique (Vitest de 38-9a/38-9b sans retouche) : 38-9a AC-5 « N enregistrements PayPal écartés (refusés, en attente
   …, ou sans effet sur le solde) » — pour Stripe, **texte propre** : « N transactions Stripe écartées (hors du solde des
   paiements : réserves, Issuing…) », seul `skipped.non_balance` y entrant (G9) — et « PayPal annonce un solde
   d'ouverture trop ancien… » (`opening_unavailable: span`) ; 38-9a AC-6 « Soldes PayPal sans compte dans Kesh » ;
   AC-7 « … n'est plus alimenté par PayPal » ; AC-9 badge « PayPal · <libellé> · <devise> » ; 38-9b AC-3 « Frais PayPal
   (<devise>) » (la ligne de frais d'une vente Stripe) et « PayPal · … » (`:350`). L'avis « PayPal ne remonte que trois
   ans » (`truncatedFrom`) ne concerne pas Stripe (`history_start` `None`, G3). Vitest : chaque site rendu « Stripe » pour
   une connexion `stripe`.
   Panneau de clôture (`FiscalYearChecks.svelte`) : pour un compte K10 en écart portant `inTransitPayouts` (40-4 AC-4),
   « dont versements en route : T » sous l'écart (et « autres décalages de date : O ») ; absent sinon. Vitest des deux.
6. **AC-6 — Codes courts Stripe dans l'aide et garde de concordance (FR-40-6-5).** `provider-labels.ts` porte
   `STRIPE_EVENT_CODES` (tous les codes de la table de 40-2 AC-2, ordre de la table, chacun avec son libellé :
   « `CHARGE` — paiement par carte », « `PAYOUT` — versement vers la banque », …) et `STRIPE_OVERRIDABLE_CODES` (40-2
   AC-6). Formulaire des règles (38-9b AC-1, `rules-labels.ts`) : l'aide du critère `event_code_prefix` **de 38-9b (« Code PayPal, p. ex. `T0006` ou `T00` ») reste telle quelle** (G18,
   AC-11 : ses Vitest passent sans retouche) et gagne une **ligne en dessous**, clé neuve
   `reconciliation-rules-help-event-code-stripe` (préfixe des aides de 38-9b AC-9 ; `rules-labels.ts` vit sous
   `features/reconciliation/`, où `lint-i18n-ownership` n'admet que l'espace `reconciliation-`) :
   « Stripe : `CHARGE`, `PAY_`, `PAYOUT` — liste complète au manuel », avec un lien « Codes Stripe » qui ouvre la liste
   (composant à part) ; l'aide du critère `invoice_prefix` ajoute « Stripe : lu à l'endroit
   réglé dans la connexion (métadonnée ou description) ». **Garde** (`crates/kesh-api/tests/textes_coherents.rs`, garde
   neuve au tableau d'en-tête du fichier) : l'ensemble des codes de `kesh_providers::stripe::event_codes::EVENT_CODES` (constante **`pub`** de 40-2 AC-2,
   `OTHER` compris, définie dans `kesh_core::providers::settings::stripe_codes` et réexportée ; lisible d'un test de
   `kesh-api`) égale celui de `STRIPE_EVENT_CODES` (lu dans le `.ts`) **et** celui du
   tableau du manuel utilisateur (AC-8 : une ligne par code, le code en première colonne en `\texttt{…}`, sous un
   `\label` que la garde cherche — présence positive du tableau exigée avant toute égalité) ; les surchargeables égalent
   `kesh_providers::stripe::event_codes::OVERRIDABLE_CODES` (constante `pub` de 40-2 AC-6) ; rouge en nommant le code en trop ou manquant. Mutation déclarée au Change Log
   (un code retiré du `.ts` → rouge).
7. **AC-7 — Textes et i18n (NFR-40-6-2, NFR-40-6-5).** Règle de `lint-i18n-ownership`, relue au code
   (`frontend/scripts/lint-i18n-ownership.js` : `extractFeatureFromPath` `:144`, `getNamespace` `:163`,
   `keyBelongsToFeature` `:177-186`, `GLOBAL_NAMESPACES` `:16`) : une clé appelée par `i18nMsg('…', …)` dans un fichier
   sous `src/lib/features/<dossier>/` doit, si `<dossier>` porte un tiret, **valoir** `<dossier>` ou **commencer par**
   `<dossier>-` ; sinon avoir `<dossier>` pour premier segment ; les espaces globaux (`error`, `tooltip`, `common`,
   `mode`, `shortcut`, `demo`) sont admis partout. Donc : clés neuves des écrans de connexion **`provider-connections-stripe-…`**
   (`features/provider-connections/` ; préfixe **retenu par 38-9a AC-10, corrigée et close** : `provider-connections-`,
   `provider-labels.ts` et `ProviderAccountBadge.svelte` compris ; l'entrée de menu est `nav-admin-providers`, déjà posée
   par 38-9a, que Stripe ne change pas) ;
   aide des règles `reconciliation-rules-help-*` (AC-6, `features/reconciliation/`) ; résumé (b) et K10 au préfixe de
   leur dossier hôte (`bank-accounts-…`, `fiscal-years-…`) ; les messages `provider-error-*` du serveur (40-1 AC-14)
   ne sont pas appelés par `i18nMsg` sous un dossier de fonctionnalité (sinon : clé du dossier qui les relaie). Écrites
   en toutes lettres (`i18n-keys.test.ts`), aux **quatre** locales (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`) ;
   `npm run lint-i18n-ownership` vert, jamais une entrée à `KNOWN_VIOLATIONS`. Aucun texte ne dit
   « comptabiliser automatiquement » ni « importer » pour un téléchargement (grep des clés neuves au Change Log). **G4-bis** (`textes_coherents.rs`) lit le manuel utilisateur et les catalogues : tout
   nombre de quatre chiffres y doit être un compte des plans livrés — les clés neuves et la section Stripe du **manuel
   utilisateur** nomment « Débiteurs Stripe » **sans numéro** (1101–1103, 6912–6914 sont des comptes de la société, non des
   plans livrés) ; les numéros ne se citent qu'au manuel d'administration, que G4-bis ne lit pas — mais que **G4-ter**
   lit (`la_forme_libre_nnnn_nom_est_juste_partout`, `textes_coherents.rs:565-600` : `admin-manual.tex`, brochure,
   README, `api-external.md`, `frontend/src`) : un nombre suivi d'une espace et d'un mot qui ouvre un nom de compte d'un
   plan livré y est contrôlé, et « 1101 Débiteurs Stripe » **rougit** (1101 absent des trois plans, « Débiteurs » ouvre
   le nom de 1100). Au manuel d'administration, un numéro de la société de Beat ne précède donc jamais un nom : « le
   compte « Débiteurs Stripe CHF » (p. ex. 1101) », « 1101–1103 » ; jamais `NNNN Débiteurs…`. `cargo test -p kesh-api
   --test textes_coherents` vert (G4-bis et G4-ter), cité au Change Log.
8. **AC-8 — Manuels (NFR-40-6-2, FR-40-6-5).**
   - **Administration** (`admin-manual.tex`, section des prestataires de 38-9a AC-11, renommée « Prestataires de paiement
     (PayPal, Stripe) » ou sous-section *Stripe* — renvois `\ref` mis à jour) : créer la clé restreinte (*Développeurs →
     Clés API → Créer une clé restreinte* ; *Lecture* sur les seules ressources relevées à la recette 40-0 — Q-40-8 —,
     *Aucun* sur toutes les autres ; jamais une clé `sk_`) ; la valeur ne se relit pas chez Stripe et ne se réaffiche
     pas dans Kesh ; mode test et réel, connexion neuve pour changer de mode ; « Tester » et chaque contrôle avec son
     remède, dont `account_readable` : le droit de **lire le compte** est exigé — sans lui, « Tester » **et chaque
     passage** refusent la clé (`PROVIDER_ACCOUNT_UNREADABLE`, D40-P3-1, 40-1 AC-9) ; compte « Débiteurs Stripe » par devise (1101–1103, numéros de la société de Beat : à écrire comme exemple) lié au compte
     bancaire Kesh de la devise, frais au sous-compte de la devise (6912–6914, B3) ; boutique (`invoiceRefSource`) ; surcharges de codes ; **la clé est effacée
     à toute restauration et se ressaisit** (E2) ; révocation dans Stripe ; recette sur une instance jetable (B12).
   - **Utilisateur** (`user-manual.tex`, sous la sous-section des prestataires de 38-9b AC-10, sous-section « Stripe ») :
     le compte Stripe est une **créance** (B13) ; la vente est passée **à la date du paiement** au compte « Débiteurs
     Stripe », produit hors TVA et TVA séparée, frais au compte de frais (G2, B6) ; la TVA est tirée du montant dans la
     devise de règlement, l'arrondi avec la TVA calculée par la boutique dans la devise présentée est assumé (G19, B20) ;
     le **versement** sort de Stripe un jour et arrive en banque plus tard : son écriture est datée de l'**arrivée** et
     la ligne du relevé UBS se rattache à la même écriture (40-3, 34-11b) ; versement échoué ou annulé : à examiner ;
     litige : à examiner, sans proposition (B7) ; **l'écart ventilé** (non rapproché, versements en route, autres
     décalages, inexpliqué) et son sens ; à la clôture, le solde des comptes « Débiteurs Stripe » **est** la créance
     (aucun numéro au manuel utilisateur, AC-7), aucune écriture à passer ; joindre le rapport de solde Stripe en CSV au compte et à l'exercice, saisir son solde de clôture, K10 et
     « dont versements en route » (40-5, 40-4) ; le rapport CSV doit avoir un en-tête à au moins deux colonnes (limite de
     40-5, `no_header`) ; le solde saisi est celui de fin de période du **rapport de solde** — le rapprochement des
     versements se joint sans solde (40-5 AC-6, texte de l'aide repris) ;
   - **Utilisateur, section des relevés joints de 38-10 AC-10** (*Joindre un relevé à un compte bancaire*), **hors** de la
     sous-section Stripe — 40-5 AC-8 la confie ici : une pièce est un **PDF ou un CSV**, sur **tout** compte bancaire
     (40-5 Écart 3) ; la liste dit le type (« PDF », « Rapport CSV ») ; le CSV est gardé tel quel, jamais lu (son solde
     se saisit) ; refus d'un CSV et leurs trois motifs (`not_utf8` : enregistrer le fichier en UTF-8 ; `binary` ;
     `no_header`) ; le nouveau message « ni un PDF ni un CSV » ; les phrases de 38-10 qui disent « un PDF » seul relues
     et ajustées (grep `PDF` dans la section, résultat au Change Log) ;
   - **Administration, surcharges de codes** : une surcharge vaut pour les mouvements téléchargés **après** elle (aucun
     reclassement, 40-2 AC-6, E6) ; `ignore` laisse un écart durable au contrôle de complétude (avertissement sous le
     champ hérité de 38-9a AC-2, dit aussi au manuel) ; la tuile de l'accueil montre la créance hors total (40-4) — et le
     paragraphe de la tuile (`:318-330`) le dit ; **liste des codes courts** (tableau `CHARGE` … `OTHER`, AC-6) en aide
     du critère `event_code_prefix` ; rattachement en lot des transactions de 2024 aux écritures existantes (B17, 38-6c1) ;
     changer de compte Stripe après un téléchargement demande une connexion neuve (`PROVIDER_ACCOUNT_CHANGED`, 40-1 AC-9) —
     le passage lit le compte à chaque fois et refuse de même une clé d'un autre compte, sans attendre « Tester ».
   - **Limite de 38-9a retirée** : la phrase que 38-9a (AC-8 b, « limite écrite ») pose au manuel — un mouvement de la
     même journée postérieur à `asOf` et déjà rapproché fait un « inexpliqué » transitoire — est **retirée** et remplacée
     par le terme « autres décalages de date » de l'écart ventilé (40-4 Écart 6, `otherDateShifts`) ; `grep -n
     transitoire` des deux manuels, résultat au Change Log.
   - Modifier les `.tex` seulement : le PDF se régénère à la release (aucun PDF commité dans la branche) ; rendu contrôlé en local par `make fr`, **aplati** (`pdftotext user-manual.pdf - | tr
     '\n' ' ' | tr -s ' '`, et de même `admin-manual.pdf` : « clé restreinte », « Lecture », « Tester ») : chaque code de la liste, « clé restreinte », « versements en route », « créance » ; résultat
     au Change Log. Manuels DE/IT/EN : « à traduire » noté si l'écart existe. Brochure (`marketing-brochure.tex`) et
     `README.md` § *Fonctionnalités* : Stripe ajouté là où PayPal l'est (relu au développement), `website/` relu
     (aucune promesse au-delà de `main`).
9. **AC-9 — E2E du parcours Stripe (NFR-40-6-3, NFR-40-6-1).** `frontend/tests/e2e/provider-stripe.spec.ts`, contre le
   transport factice : préalables par l'API au `beforeAll` (société et exercice **multi-devises**, 37-1 — l'exercice et
   la date « Télécharger depuis » se fixent sur les dates du jeu `"default"` de 40-1 AC-11, jamais sur la date du jour :
   un versement arrivé après la date du run apparaît « en route », et l'assertion ne porte que sur « Inexpliqué » ; compte du plan
   USD « Débiteurs Stripe USD », compte de frais USD, règle `event_code_prefix` `CHARGE` → compte de produits USD,
   compte bancaire UBS USD) ; Administrateur : crée la connexion Stripe à l'écran, puis le test appelle `POST
   /api/v1/_test/provider-connector` avec **`stripeResponses: "default"`** pour ce `connectionId` (40-1 AC-11 : servi
   **par chemin**, `/v1/account` compris, si bien que la lecture du compte au début du passage, D40-P3-1, n'exige rien de
   plus ; un programme **ordonné** — captures de l'AC-10 (✗ `permission_present`, ✗ `account_readable`) — sert la réponse de `/v1/account` **en premier** pour un
   passage, et dans l'ordre de 40-1 AC-9 pour « Tester » : `/v1/balance`, `/v1/balance_transactions`, `/v1/account`) ; colle
   d'abord une clé `sk_test_…` → refus local, **aucune requête `PUT …/secrets`** (interception) ; dépose une clé
   `rk_test_` marqueur ; teste (✓, mode test, `available`/`pending` affichés) ; crée le compte USD depuis le bloc des
   soldes sans compte (38-9a AC-6) ; télécharge ; Comptable : la vente USD est **proposée** et acceptée ; importe par
   l'API un relevé CAMT synthétique du compte UBS portant le crédit du versement à la date d'arrivée du jeu par défaut ;
   le versement Stripe est proposé daté de l'arrivée (AC-4), accepté (voie b) ; la ligne UBS se **rattache** à la même
   écriture par le geste de l'utilisateur (34-11b) ; le résumé du contrôle (b) affiche « Inexpliqué : 0.00 ». La clé
   marqueur n'apparaît jamais dans `page.content()` ni dans les réponses interceptées (`page.on('response')`). Données
   synthétiques seulement. Gates : `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` ; **E2E complet au
   dernier commit de code** ; gate backend complet (AC-6 ajoute un test Rust).
10. **AC-10 — Recette visuelle (NFR-40-6-4).** Captures (thème clair, base de démonstration, `kesh-gate-logs/`, hors
    dépôt), jointes au Dev Agent Record et au prompt de revue : formulaire Stripe (boutique « métadonnée » et
    « description », bloc « Avancé ») ; refus d'une clé `sk_` ; clé de test déposée (badge « Mode test ») ; résultat de
    « Tester » tout vert avec `available`/`pending`, avec ✗ `permission_present` et avec ✗ `account_readable` ; proposition d'un versement (sortie,
    arrivée) ; échec de versement à la file ; résumé du contrôle (b) avec versements en route ; K10 avec « dont versements
    en route » ; aide des codes Stripe au formulaire des règles.
11. **AC-11 — Ce que la story ne touche pas.** Aucune migration ; aucune route ni action d'audit (C-40-6) ; **aucun outil
    MCP** (G17, E23 : connexions, contrôles et transactions de prestataire ne sont pas exposés ; `kesh_ecritures` lit les
    écritures acceptées) ; écrans et textes PayPal inchangés (leurs Vitest et E2E de 38-9a/38-9b verts sans retouche,
    G18) ; aucune donnée réelle de Beat dans les captures ni les tests.

## Tâches

- [ ] T1 — `StripeSettingsFields.svelte`, bras Stripe de `providerSettingsBody`, mode (AC-1).
- [ ] T2 — Bloc de la clé : `stripeKeyRefusal`, messages, avis du mode test (AC-2).
- [ ] T3 — « Tester » Stripe : libellés, remèdes, `available`/`pending` (AC-3).
- [ ] T4 — Proposition de versement, file des échecs (AC-4) ; résumé (b) et K10 (AC-5).
- [ ] T5 — Codes courts : constantes, aides des règles, garde `textes_coherents.rs` (AC-6) ; i18n (AC-7).
- [ ] T6 — Manuels admin et utilisateur, rendu aplati contrôlé (PDF non commité), brochure, README, site (AC-8).
- [ ] T7 — E2E `provider-stripe.spec.ts`, captures (AC-9, AC-10) ; gates complets ; compteurs (`Cargo.lock`, registres)
      recomptés depuis la source après chaque reprise de `main`.

## Note de version

*Texte prêt à assembler au `CHANGELOG.md` à la release ; aucune ligne n'y est ajoutée dans la branche.*

- **Ajouté** : écrans du compte Stripe — formulaire de connexion et clé restreinte (mode test signalé), résultat de
  « Tester » avec les droits manquants, proposition et file des versements, écart ventilé avec versements en route, aide
  des codes courts ; manuels d'administration et utilisateur mis à jour (`.tex`).

## Dev Notes

- **Ne rien recopier de 38-9a/38-9b** : un bras de prestataire par composant ou par table de `provider-labels.ts` ;
  `+page.svelte` des comptes bancaires et `ReconciliationProposals.svelte` ne reçoivent rien de plus que ce que 38-9a
  et 38-9b y ont mis.
- **Clé** : le contrôle local `sk_` est une protection de plus, non une validation ; aucune expression régulière de
  format complet (40-1 AC-4) recopiée au TypeScript.
- **Montants** : en devise de la transaction ou du compte, jamais « CHF » supposé (38-9b AC-3) ; `big.js`, jamais
  `Number`.
- **Libellés de 40-1 transmis** : contrôles `mode_unchanged`, `account_unchanged`, `account_readable`,
  `permission_present` ; codes `PROVIDER_SECRET_NOT_RESTRICTED` (motifs `secret_key`, `publishable_key`,
  `unknown_prefix`), `PROVIDER_MODE_CHANGED`, `PROVIDER_ACCOUNT_CHANGED`, `PROVIDER_ACCOUNT_UNREADABLE` ; motif
  d'`errorDetail` `account` (40-1 AC-9, AC-14).
- **Références** : epic G4, G8, G10, G12, G13, G14, G19, C-40-1 à C-40-6, Q-40-2, Q-40-8, Q-40-10 ; 40-1 AC-2, AC-4,
  AC-5, AC-9, AC-11, AC-14 ; 40-2 AC-2, AC-6 ; 40-4 AC-2, AC-4, AC-5, AC-7 ; 40-5 AC-6 ; 38-9a AC-2 à AC-13 ; 38-9b
  AC-1, AC-3, AC-10, AC-11.

## Questions ouvertes

- **Q-40-8 (recette 40-0)** — noms exacts des ressources à cocher en *Lecture* : sans eux, le manuel ne peut lister les
  droits (AC-8) ; la story ne se développe pas avant que 40-0 les ait relevés.
- ~~Inter-fiches 40-3 (`valueDate`, `entryDate`)~~ et ~~inter-fiches 40-2 (`OVERRIDABLE_CODES`)~~ — **réglées** le
  2026-11-15 (alignement inter-fiches P1) : 40-3 AC-3 et 40-2 AC-6 portent les noms que lisent AC-4 et AC-6.
- ~~Inter-fiches 38-9a (P2-3)~~ — **réglée** le 2026-11-15 : 38-9a AC-10 (corrigée et close) retient
  `provider-connections-` pour les clés de l'écran et `nav-admin-providers` pour le menu ; 40-6 AC-7 écrit la règle de
  `keyBelongsToFeature` et le préfixe `provider-connections-stripe-`.
- ~~Inter-fiches 40-2 (P2-8)~~ — **réglée** le 2026-11-15 : 40-2 AC-2 rend `EVENT_CODES` `pub` (`OTHER` compris), lue
  par l'AC-6.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-16 — alignement sur les règles de travail à plusieurs (Sonnet) : section Note de version ajoutée ; AC-8, T6,
  NFR-40-6-2 : `.tex` seulement, PDF non commité, régénéré à la release (rendu aplati contrôlé en local) ; compteurs
  recomptés après chaque reprise de `main` (T7) ; K9 → K10 partout hors entrées datées de ce journal (l'Epic 37 prend K9).

- 2026-11-15 — validation P5 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche. Périmètre :
  `git diff 98bca548 b5d23154` de la fiche (remédiation P4). Confirmés : AC-2, tout `sk_` → `secret_key` (40-1 AC-4,
  valeur rognée, serveur sans rognage) ; AC-3, contrôle `account_readable` après `permission_present` et avant
  `account_unchanged` (ordre de 40-1 AC-9), codes lus dans `lastError` (38-9a AC-5, deux vocabulaires ; passage
  `permission` / `invalid_response`, `errorDetail` `account`, 40-1 AC-9), texte du message identique à 40-1 AC-14 (après
  P5-1 de 40-1) ; AC-4, `valueDate` déjà rendu (`routes/reconciliation.rs:172-176`), `entryDate` seule à 40-3 AC-3 ;
  AC-9, ordre des programmes (passage : `/v1/account` d'abord ; « Tester » : balance, balance_transactions, account) =
  40-1 AC-9 et AC-11 ; AC-10, capture ✗ `account_readable`. LOW : P5-1 AC-9, « capture de l'AC-10 » au singulier pour
  deux états (✗ `permission_present`, ✗ `account_readable`) → les deux nommés. Inter-fiches : 40-1 AC-14 aligné sur le
  texte de l'écran (fait, fiche 40-1). Axes exercés : chaque phrase changée en P4 contre 40-1, 38-9a, 40-3 et le code.
  Non exercés : code des écrans (non écrit), PDF des manuels, noms de ressources Stripe (Q-40-8).
- 2026-11-15 — rédaction (Opus) : 11 AC ; 5 FR de l'epic, 5 NFR (4 + 1) ; aucune migration, aucune route ; un test
  Rust de concordance (codes courts : table de 40-2, TypeScript, manuel) ; écarts : écran CSV livré par 40-5, tuile de
  l'accueil par 40-4, clé de test admise (P40-1). Relu : fiches 38-9a (AC-1 à AC-13), 38-9b (AC-1, AC-3, AC-10), 40-1
  (AC-2, AC-4, AC-5, AC-9, AC-11, AC-14), 40-2 (AC-2, AC-6) ; code : `admin-manual.tex:3299`, `user-manual.tex:318-356`,
  `:3004`, `textes_coherents.rs:1-120`.
- 2026-11-15 — validation P1 (Sonnet) : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 3 LOW ; remédiés dans la fiche sauf P1-1 et P1-3
  (inter-fiches 40-3 et 40-2, texte proposé aux Questions ouvertes). HIGH : P1-1 AC-4 lit `valueDate`/date de l'écriture
  que 40-3 ne rend nulle part (relu : `40-3` AC-1 à AC-8, 38-6b/38-9b `bookingDate` seul). MEDIUM : P1-2 AC-6 changeait l'aide
  « Code PayPal » de 38-9b, contre AC-11 et G18 (désormais ligne ajoutée, clé neuve) ; P1-3 garde de concordance sans liste
  publique des surchargeables ni format du tableau du manuel ; P1-4 G4-bis (`textes_coherents.rs:372-420`) rougirait sur
  1101–1103 / 6912–6914 cités au manuel utilisateur (règle posée à l'AC-7). LOW : P1-5 lien d'un échec de versement par
  `related_provider_tx_id` au lieu de `relatedBankTransactionId` (40-3 AC-7) ; P1-6 avis `rk_test_` « lu du serveur » alors
  que `livemode` n'est posé qu'à « Tester » ; P1-7 motifs d'`errorDetail` neufs, signe des versements en route, PDF du
  manuel d'administration non aplati, refus `kind_not_allowed`/`too_many`/`invalid_format` sans libellé. Prémisses relues et
  justes : `admin-manual.tex:3299`, `user-manual.tex:3004`, `textes_coherents.rs:54,109`, codes courts G8 (22) = table 40-2.
- 2026-11-15 — alignement inter-fiches P1 (Opus) : P1-1 et P1-3 réglés aux fiches sources — AC-4 lit `valueDate`,
  `bookingDate` et `entryDate` de 40-3 AC-3 ; AC-6 lit `stripe::event_codes::OVERRIDABLE_CODES` de 40-2 AC-6 ; Questions
  ouvertes inter-fiches closes.
- 2026-11-15 — validation P2 (Opus) : 9 findings (0 CRITICAL, 0 HIGH, 5 MEDIUM, 4 LOW) ; remédiés dans la fiche sauf
  P2-3 (part 38-9a) et P2-8 (40-2), inter-fiches, texte proposé aux Questions ouvertes. MEDIUM : P2-1 AC-8 écrivait
  « le solde de 1101–1103 » au manuel **utilisateur**, contre la règle de l'AC-7 (résidu de P1-4 : 1101 absent des trois
  plans, `pme.json`, G4-bis rouge) ; P2-2 AC-7 disait le manuel d'administration hors des gardes, or **G4-ter**
  (`textes_coherents.rs:565-600`) le lit en forme libre : « 1101 Débiteurs Stripe » rougirait (« Débiteurs » ouvre 1100)
  — forme admise écrite ; P2-3 clé `rules-help-event-code-stripe` appelée sous `features/reconciliation/` : refusée par
  `lint-i18n-ownership` (`keyBelongsToFeature`) et contraire au préfixe `reconciliation-rules-help-*` de 38-9b AC-9 —
  renommée ; même règle pour `provider-stripe-` sous `features/provider-connections/` (préfixe de 38-9a AC-10 : inter-
  fiches) ; P2-4 seul « Selon PayPal » était adapté : sept autres textes de 38-9a/38-9b nomment PayPal (écartés, solde
  d'ouverture, soldes sans compte, retrait des réglages, badge, « Frais PayPal (<devise>) », résumé) — inventaire à
  l'AC-5, texte propre des écartés Stripe (`non_balance` seul) ; P2-5 40-5 AC-8 confie à 40-6 le manuel des relevés
  joints, que l'AC-8 ne traitait que dans la sous-section Stripe : la section générique de 38-10 aurait dit « PDF » seul
  pour tout compte — puce ajoutée (PDF ou CSV, motifs, solde du rapport de solde seulement). LOW : P2-6 Écart 3 périmé
  (l'epic est aligné sur P40-1) ; P2-7 E2E : exercice et « Télécharger depuis » sur les dates du jeu `"default"` (versement
  arrivé le lendemain de sa sortie : « en route » selon la date du run) ; P2-8 la table `stripe::event_codes` doit être
  `pub` pour la garde, 40-2 ne le dit que d'`OVERRIDABLE_CODES` ; P2-9 manuel d'administration : surcharge non
  rétroactive et `ignore` (40-2 AC-6 : « dit par 40-6 au manuel »). Relu : `admin-manual.tex:3299`,
  `user-manual.tex:3004`, `:316-358`, `textes_coherents.rs:50-56`, `:105-112`, `:228-267`, `:372-445`, `:510-600`,
  `crates/kesh-core/assets/charts/*.json` (ni 1101–1103 ni 6910–6914), `frontend/scripts/lint-i18n-ownership.js`
  (`GLOBAL_NAMESPACES`, `extractFeatureFromPath`, `keyBelongsToFeature`) ; fiches 38-9a (AC-2, AC-5 à AC-10), 38-9b
  (AC-1, AC-3, AC-9), 40-1 (AC-2, AC-4, AC-5, AC-11, AC-14, Écart 1), 40-2 (AC-2, AC-6, Dev Notes), 40-3 (AC-3, AC-7),
  40-4 (AC-2, AC-4, AC-5, AC-7, Dev Notes), 40-5 (AC-6, AC-8, Écart 3) — noms concordants. Trend : P1 = 7 (1 HIGH,
  3 MEDIUM) → P2 = 9 (5 MEDIUM ; P2-1 est un résidu de remédiation P1, les autres d'origine) ; une passe de plus est due.
  Axes non exercés : code des écrans de 38-9a/38-9b (non écrit) ; PDF des manuels (non régénérés) ; `api-external.md`
  pour la connexion Stripe (réglages, corps `restrictedKey`, codes neufs) — aucune fiche 40-1/40-2 ne le nomme,
  signalé à l'orchestrateur, hors de cette fiche.
- 2026-11-15 — alignement inter-fiches P2 (Opus) : AC-2, badge « à établir » tant que ni « Tester » ni un passage n'a
  posé le mode (40-1 AC-5, AC-9) ; AC-3, motif `balance_parts` retiré (supprimé en P1 de 40-4), contrôle
  `account_unchanged` et remède de `PROVIDER_ACCOUNT_CHANGED` (D40-P2-1, 40-1 AC-9) ; AC-4, aide `PAYOUT_FAIL`/`PAYOUT_CANCEL`
  complétée (écriture manuelle ; versement et échec marqués examinés après une annulation) ; AC-6, garde lue sur
  `EVENT_CODES` `pub` de 40-2 AC-2 ; AC-7, règle de `lint-i18n-ownership` relue au code (`:16`, `:144`, `:163`,
  `:177-186`), préfixe `provider-connections-stripe-` ; AC-8, limite « inexpliqué transitoire » de 38-9a retirée du
  manuel au profit des « autres décalages de date » (40-4 Écart 6), changement de compte Stripe au manuel
  d'administration ; Dev Notes et Questions ouvertes mises à jour. Le point `api-external.md` signalé en P2 est porté
  par 40-1 AC-15.
- 2026-11-15 — validation P3 (Sonnet) : 4 findings (0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW) ; remédiés dans la fiche sauf
  P3-1 en part 40-1 (inter-fiches, ci-dessous). MEDIUM : P3-1 AC-1/AC-3 tenaient `accountId: null` pour « test réussi, la
  clé n'a pas ce droit » (40-1 AC-9 actuel : 403/404 sur `/v1/account` n'échouent pas), alors que D40-P3-1 de 40-1
  (orchestrateur) fait lire `GET /v1/account` à chaque passage et refuser une clé sans droit de lire le compte — « non
  lu » ne vaut plus que pour une connexion jamais testée, un libellé et un Vitest du refus sont ajoutés ; le code ou le
  motif exact est à relever dans 40-1 au développement (non fixé à la date). LOW : P3-2 `stripeKeyRefusal` refuse tout
  `sk_` alors que le serveur ne motive `secret_key` que pour `sk_live_`/`sk_test_` — écart dit (local plus strict) ; P3-3
  collage avec fin de ligne ou espaces → faux `unknown_prefix` — valeur rognée avant contrôle et envoi, deux Vitest ;
  P3-4 rédaction : Questions ouvertes et AC-7 citaient encore la correction de 38-9a AC-10 comme à venir — 38-9a (close)
  retient `provider-connections-` et `nav-admin-providers` (Stripe n'ajoute aucune entrée de menu). Relu au code :
  `admin-manual.tex:3299`, `user-manual.tex:3004`, `:300-334`, `textes_coherents.rs:54`, `:109`, `frontend/scripts/
  lint-i18n-ownership.js:16`, `:177-186` ; fiches 38-9a AC-10, 40-1 (AC-4, AC-9, AC-11), 40-2 (AC-2, AC-6), 40-3 (AC-3,
  AC-7), 40-4 (AC-2, AC-4), 40-5 (AC-6, AC-8) — noms concordants. Axes non exercés : code des écrans (non écrit) ; PDF des
  manuels ; noms de ressources Stripe (Q-40-8). Trend : P2 = 9 (5 MEDIUM) → P3 = 4 (1 MEDIUM, d'origine inter-fiches) ;
  une passe de plus est due.
- 2026-11-15 — validation P4 ciblée (Opus) : 6 findings (0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW), braquée sur la remédiation
  P3 (`git diff b4dc35dc 88c6b022`) et les points inter-fiches de D40-P3-1 ; remédiés dans la fiche. MEDIUM : P4-1 AC-2
  (écrit en P3-2) disait que le serveur ne motive `secret_key` que pour `sk_live_`/`sk_test_` et rend `unknown_prefix`
  pour un autre `sk_` — faux depuis la P3 de 40-1, dont l'AC-4 motive `secret_key` pour **tout** `sk_` : local et serveur
  concordent, phrase réécrite ; P4-2 AC-3 (écrit en P3-1) laissait le refus d'une clé sans droit de lire le compte sans
  contrôle ni code (« relevé au développement »), avec un libellé d'écran portant une référence interne
  (« recette 40-0, Q-40-8 »), ni lecture dans `lastError`, ni motif `account`, ni capture, ni phrase au manuel — point
  inter-fiches appliqué : contrôle `account_readable` (« Droit de lire le compte Stripe », ordre de 40-1 AC-9), code
  `PROVIDER_ACCOUNT_UNREADABLE` → « Cette clé ne peut pas lire le compte Stripe : ajoutez le droit de lecture du compte »
  (texte de 40-1 AC-14), les trois codes `PROVIDER_*_CHANGED`/`UNREADABLE` lus dans `lastError` (38-9a AC-5), motif
  `account` du passage, trois Vitest, capture ✗ `account_readable` (AC-10), AC-8 : droit de lire le compte exigé, refus
  au passage comme à « Tester », compte d'un autre Stripe refusé au passage. LOW : P4-3 AC-1 « non lu » ⇔ « jamais
  testée » oubliait le `Pose` du passage (40-1 AC-9) ; P4-4 AC-2 « comme le serveur la lit » laissait croire que le
  serveur rogne (40-1 AC-4 : `[A-Za-z0-9]` après le préfixe) ; P4-5 AC-4 attribuait `valueDate` à 40-3 — déjà rendu
  (`routes/reconciliation.rs:172-176`, posé à `:1478` ; 40-3 AC-3 n'ajoute que `entryDate`, point inter-fiches) ; P4-6
  AC-9 : jeu `"default"` servi par chemin (lecture du compte au passage sans rien à programmer), programme ordonné :
  `/v1/account` en premier pour un passage, ordre de 40-1 AC-9 pour « Tester » ; « Dépend de » et Dev Notes gagnent
  `account_readable`, `account_unchanged`, `PROVIDER_ACCOUNT_UNREADABLE`. Aucune mention « passage sans « Tester » non
  gardé » dans la fiche (grep `non gardé`, `sans « Tester »` : aucune occurrence, rien à retirer). Relu :
  40-1 AC-4, AC-9 (« Tester », compte gardé, passage), AC-10, AC-11 (jeu `"default"` par chemin), AC-14 ; 40-3 AC-3 et
  Écart 3 ; 38-9a AC-5 (`:177-186`, deux vocabulaires), AC-10 (`:296-298`) ; 38-3b `:313` (soldes avant les mouvements) ;
  `routes/reconciliation.rs:172-176`, `:1478`. Axes non exercés : code des écrans (non écrit) ; PDF des manuels ; noms
  de ressources Stripe (Q-40-8). Trend : P3 = 4 (1 MEDIUM) → P4 = 6 (2 MEDIUM, nés de la remédiation P3 et d'une
  remédiation concurrente de 40-1 : recyclage d'inter-fiches, non défaut d'origine — signal déclaré) ; une passe de plus
  est due.
