# Story 40-6 : écrans et manuels Stripe — connexion et clé restreinte, boutique, versements, écart ventilé ; E2E

Status: ready-for-dev

- **Epic** : 40 (`_bmad-output/planning-artifacts/epic-40-stripe.md`, § *40-6*, G4, G8, G10, G12, G13, G14, G17, G19,
  C-40-1 à C-40-6) ; décisions de Beat B3, B6, B10, B13, B17, B20, B21 (`epics-38-40-cadrage.md`), qui priment ;
  décision de l'orchestrateur **P40-1** (clés `rk_live_` et `rk_test_` admises, `sk_` refusée, mode montré).
- **Dépend de** : **40-1** (réglages `StripeSettings`, secret `restrictedKey`, refus `PROVIDER_SECRET_NOT_RESTRICTED`
  et ses motifs, mode et `PROVIDER_MODE_CHANGED`, contrôles de « Tester » `mode_unchanged`, `permission_present`,
  `provider_reachable`, réponse `accountId`/`livemode`/`apiVersion`, jeu factice `stripeResponses: "default"`), **40-2**
  (codes courts `stripe::event_codes`, surcharges et leurs refus), **40-3** (versement : `value_date`, écriture datée de
  l'arrivée, ligne UBS rattachée, échecs à la file), **40-4** (`inTransitPayouts`, `otherDateShifts`, `available` et
  `pending` à « Tester », détail de K9, tuile de l'accueil), **40-5** (rapports CSV aux relevés joints) ; **38-9a** et
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
| `NFR-40-6-2` | Quatre locales ; manuels d'administration et utilisateur à jour, PDF régénérés et contrôlés aplatis. | AC-7, AC-8 |
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
  contrôles de clôture `frontend/src/lib/features/fiscal-years/FiscalYearChecks.svelte` (existant ; K9 par 38-10).
- Manuels : `docs/manual/fr/admin-manual.tex` (`\section{Le serveur MCP}` ligne 3299, après laquelle 38-9a AC-11 pose
  « Prestataires de paiement (PayPal) ») ; `docs/manual/fr/user-manual.tex` (`\section{Réconciliation bancaire}` ligne
  3004, sous laquelle 38-9b AC-10 pose « Comptes de prestataires de paiement (PayPal) » ; tuile des comptes bancaires
  `:318-356`). PDF par `make fr` dans `docs/manual/`.
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
3. **Clé de test** : G4 la réservait au mode test ; P40-1 l'admet (40-1 *Écart* 1). L'écran dit le mode au lieu de
   refuser la clé.
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
     - **posés par le serveur, en lecture seule** : « Compte Stripe » (`accountId`, « non lu — la clé n'a pas ce droit »
       si `null` après un test réussi), « Version de l'API » (`apiVersion`), **mode** (`livemode` : badge « Mode test »
       ou « Mode réel » ; `null` → « Mode : à établir — lancez « Tester la connexion » »).
   - `providerSettingsBody(form)` (38-9a AC-2) gagne le bras Stripe : il envoie **toujours** `overlapDays`,
     `invoiceRefSource`, `eventKindOverrides`, **jamais** `accountId`, `apiVersion`, `livemode` (40-1 AC-2 : refusés du
     client). Vitest : réglages lus avec les trois champs du serveur → corps sans eux ; champs non touchés → corps qui les
     porte inchangés ; bras PayPal inchangé (ses tests de 38-9a verts sans retouche).
2. **AC-2 — Clé restreinte en saisie seule (FR-40-6-1, NFR-40-6-1).** Bloc du secret de 38-9a AC-3, pour Stripe :
   libellé « Clé restreinte Stripe », corps `{ restrictedKey }` (40-1 AC-4), mêmes garanties (jamais pré-rempli, vidé
   après succès **et** échec, aucun store, journal, `toast` ni `data-*`).
   - **Contrôle avant envoi** (fonction pure `stripeKeyRefusal(value) -> reason | null`, `provider-labels.ts`) : une
     valeur qui commence par `sk_` est refusée **sans être envoyée** (une clé qui peut tout faire ne quitte pas le
     navigateur), motif `secret_key` ; `pk_` → `publishable_key` ; tout préfixe autre que `rk_live_` / `rk_test_` →
     `unknown_prefix` — motifs de 40-1 AC-4, le serveur restant l'autorité. Le champ est alors vidé.
   - Messages, par motif (local ou `400 PROVIDER_SECRET_NOT_RESTRICTED`, `details.reason`) : `secret_key` → « C'est une
     clé secrète (sk_…) : elle donne tous les droits sur votre compte Stripe. Kesh ne l'accepte pas. Créez une clé
     restreinte en lecture seule (voir le manuel d'administration) et, par prudence, faites tourner cette clé secrète
     dans Stripe. » ; `publishable_key` → « C'est une clé publiable (pk_…) : elle ne lit pas vos transactions. » ;
     `unknown_prefix` → « Une clé restreinte Stripe commence par rk_live_ ou rk_test_. » ; motif inconnu → message du code.
   - Après dépôt d'une clé `rk_test_` : avis « Clé de test : Kesh lira votre compte Stripe en mode test (aucun argent
     réel) », tiré du **seul préfixe** lu avant que le champ soit vidé (un booléen d'état du composant, jamais la valeur ni
     un fragment) ; il disparaît au rechargement. Le **badge** de mode, lui, est celui de la connexion (AC-1), lu du
     serveur : « à établir » tant que « Tester » n'a pas réussi (le dépôt ne pose pas `livemode`, 40-1 AC-9).
   - Vitest : `sk_live_…` → aucun appel réseau (espion sur `fetch`), champ vidé, message ; `rk_test_…` envoyé ; la valeur
     marqueur absente de `container.innerHTML` après succès et après refus.
3. **AC-3 — « Tester la connexion » Stripe (FR-40-6-1, FR-40-6-4).** Liste de contrôles de 38-9a AC-4 : libellés de
   `mode_unchanged` (« Même mode que les téléchargements passés ») et, pour Stripe, de `permission_present` (« Droit de
   lire les transactions de solde et leurs paiements ») ; remèdes : `PROVIDER_AUTH_FAILED` → « Clé refusée par Stripe :
   elle a pu être révoquée ; créez-en une et remplacez-la » ; `PROVIDER_PERMISSION_MISSING` → « Dans Stripe, donnez à la
   clé le droit *Lecture* sur les ressources listées au manuel » ; `PROVIDER_MODE_CHANGED` → « Cette clé n'est pas du même
   mode que les mouvements déjà téléchargés : créez une connexion neuve pour changer de mode ». Succès : compte Stripe,
   mode, version, et par devise : **total**, **disponible** (`available`) et **en attente** (`pending`) quand présents
   (40-4 AC-5), au format suisse, avec « lus à l'instant, non enregistrés » ; une devise sans compte Kesh renvoie au bloc
   des soldes sans compte (38-9a AC-6). `provider-labels.ts` : chaque contrôle, chaque code et chaque **motif
   d'`errorDetail`** neufs (40-1 AC-5, AC-6, AC-7, AC-10 : `livemode`, `pagination`, `value_date`, `currency`,
   `balance_parts`, … ; liste relevée par `grep -n 'reason: "' ` des fiches 40-1 à 40-4 au développement) ont leur libellé,
   et `provider_reachable` le sien (test de table de 38-9a étendu ; code ou motif inconnu → rendu brut, jamais vide). Vitest : réponse Stripe complète ; `accountId: null` ; ✗
   `mode_unchanged`.
4. **AC-4 — Versement : sortie, arrivée, proposition (FR-40-6-3, NFR-40-6-5).** Dans `ProviderProposalPreview.svelte`
   (38-9b AC-3), bras **retrait** : quand la transaction porte `valueDate` (champ que 40-3 ajoute au DTO de transaction
   des propositions ; nom relevé à 40-3), l'en-tête dit « Sorti de Stripe le <date de la transaction> · arrivée en banque
   prévue le <`valueDate`> » et l'aperçu « Écriture proposée au <date de l'écriture proposée, 40-3> » ; le compte bancaire
   suggéré (`transferBankAccountId`, cherché à partir de l'arrivée, C-40-5) et l'écriture existante à rattacher (voie a)
   gardent le rendu de 38-9b. Sans `valueDate` (PayPal) : rendu de 38-9b inchangé (Vitest). File « à examiner » : les
   codes `PAYOUT_FAIL`, `PAYOUT_CANCEL` portent l'aide « Le versement est revenu sur le solde Stripe ; s'il avait été
   rapproché, annulez ce rapprochement » et le lien vers le versement d'origine (`relatedBankTransactionId` de l'élément de la file, 40-3 AC-7 ; `null` → « versement
   d'origine hors de Kesh », sans lien) ; un litige (`DISPUTE`, `DISPUTE_REV`) « à examiner, sans proposition » (B7). Libellés de genre et de code inconnus :
   lecture tolérante de 38-9b. Vitest : retrait avec et sans `valueDate`, échec de versement.
5. **AC-5 — Écart ventilé et K9 (FR-40-6-4).** Résumé du contrôle (b) (38-9a AC-8 b, ligne du compte et page des
   comptes bancaires) : « Selon Stripe, au <`asOf`> : X · Aux livres : Y · Non rapproché : Z · **Versements en route :
   T** · Autres décalages de date : O · Inexpliqué : W », les deux termes neufs **seulement quand présents** (40-4 AC-2 :
   absents = 0) ; l'écran ne recalcule rien ; une infobulle sur « Versements en route » : « Partis de Stripe, pas encore
   arrivés en banque selon leur écriture : c'est encore une créance sur Stripe ». Les termes se montrent **dans le signe où le serveur
   les rend** (un versement en route : `−300.00`, 40-4 Dev Notes) ; le manuel dit le sens. « Selon Stripe » au lieu de « Selon
   PayPal » par le libellé du prestataire (`provider-labels.ts`, jamais un nom écrit en dur ; PayPal inchangé, Vitest).
   Panneau de clôture (`FiscalYearChecks.svelte`) : pour un compte K9 en écart portant `inTransitPayouts` (40-4 AC-4),
   « dont versements en route : T » sous l'écart (et « autres décalages de date : O ») ; absent sinon. Vitest des deux.
6. **AC-6 — Codes courts Stripe dans l'aide et garde de concordance (FR-40-6-5).** `provider-labels.ts` porte
   `STRIPE_EVENT_CODES` (tous les codes de la table de 40-2 AC-2, ordre de la table, chacun avec son libellé :
   « `CHARGE` — paiement par carte », « `PAYOUT` — versement vers la banque », …) et `STRIPE_OVERRIDABLE_CODES` (40-2
   AC-6). Formulaire des règles (38-9b AC-1, `rules-labels.ts`) : l'aide du critère `event_code_prefix` **de 38-9b (« Code PayPal, p. ex. `T0006` ou `T00` ») reste telle quelle** (G18,
   AC-11 : ses Vitest passent sans retouche) et gagne une **ligne en dessous**, clé neuve `rules-help-event-code-stripe` :
   « Stripe : `CHARGE`, `PAY_`, `PAYOUT` — liste complète au manuel », avec un lien « Codes Stripe » qui ouvre la liste
   (composant à part) ; l'aide du critère `invoice_prefix` ajoute « Stripe : lu à l'endroit
   réglé dans la connexion (métadonnée ou description) ». **Garde** (`crates/kesh-api/tests/textes_coherents.rs`, garde
   neuve au tableau d'en-tête du fichier) : l'ensemble des codes de `kesh_providers::stripe::event_codes` (40-2 ; la table doit être **`pub`** et lisible d'un test de
   `kesh-api`, `kesh-providers` étant sa dépendance) égale celui de `STRIPE_EVENT_CODES` (lu dans le `.ts`) **et** celui du
   tableau du manuel utilisateur (AC-8 : une ligne par code, le code en première colonne en `\texttt{…}`, sous un
   `\label` que la garde cherche — présence positive du tableau exigée avant toute égalité) ; les surchargeables égalent
   ceux de 40-2 (liste **publique** `OVERRIDABLE_CODES` à demander à 40-2, sinon la garde ne peut la lire — inter-fiches) ; rouge en nommant le code en trop ou manquant. Mutation déclarée au Change Log
   (un code retiré du `.ts` → rouge).
7. **AC-7 — Textes et i18n (NFR-40-6-2, NFR-40-6-5).** Clés neuves au préfixe `provider-stripe-` (et `rules-help-*` pour
   l'aide), écrites en toutes lettres (`i18n-keys.test.ts`), aux **quatre** locales
   (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`) ; `npm run lint-i18n-ownership` vert. Aucun texte ne dit
   « comptabiliser automatiquement » ni « importer » pour un téléchargement (grep des clés neuves au Change Log). **G4-bis** (`textes_coherents.rs`) lit le manuel utilisateur et les catalogues : tout
   nombre de quatre chiffres y doit être un compte des plans livrés — les clés neuves et la section Stripe du **manuel
   utilisateur** nomment « Débiteurs Stripe » **sans numéro** (1101–1103, 6912–6914 sont des comptes de la société, non des
   plans livrés) ; les numéros ne se citent qu'au manuel d'administration, que G4-bis ne lit pas. `cargo test -p kesh-api
   --test textes_coherents` vert, cité au Change Log.
8. **AC-8 — Manuels (NFR-40-6-2, FR-40-6-5).**
   - **Administration** (`admin-manual.tex`, section des prestataires de 38-9a AC-11, renommée « Prestataires de paiement
     (PayPal, Stripe) » ou sous-section *Stripe* — renvois `\ref` mis à jour) : créer la clé restreinte (*Développeurs →
     Clés API → Créer une clé restreinte* ; *Lecture* sur les seules ressources relevées à la recette 40-0 — Q-40-8 —,
     *Aucun* sur toutes les autres ; jamais une clé `sk_`) ; la valeur ne se relit pas chez Stripe et ne se réaffiche
     pas dans Kesh ; mode test et réel, connexion neuve pour changer de mode ; « Tester » et chaque contrôle avec son
     remède ; compte « Débiteurs Stripe » par devise (1101–1103, numéros de la société de Beat : à écrire comme exemple) lié au compte
     bancaire Kesh de la devise, frais au sous-compte de la devise (6912–6914, B3) ; boutique (`invoiceRefSource`) ; surcharges de codes ; **la clé est effacée
     à toute restauration et se ressaisit** (E2) ; révocation dans Stripe ; recette sur une instance jetable (B12).
   - **Utilisateur** (`user-manual.tex`, sous la sous-section des prestataires de 38-9b AC-10, sous-section « Stripe ») :
     le compte Stripe est une **créance** (B13) ; la vente est passée **à la date du paiement** au compte « Débiteurs
     Stripe », produit hors TVA et TVA séparée, frais au compte de frais (G2, B6) ; la TVA est tirée du montant dans la
     devise de règlement, l'arrondi avec la TVA calculée par la boutique dans la devise présentée est assumé (G19, B20) ;
     le **versement** sort de Stripe un jour et arrive en banque plus tard : son écriture est datée de l'**arrivée** et
     la ligne du relevé UBS se rattache à la même écriture (40-3, 34-11b) ; versement échoué ou annulé : à examiner ;
     litige : à examiner, sans proposition (B7) ; **l'écart ventilé** (non rapproché, versements en route, autres
     décalages, inexpliqué) et son sens ; à la clôture, le solde de 1101–1103 **est** la créance, aucune écriture à
     passer ; joindre le rapport de solde Stripe en CSV au compte et à l'exercice, saisir son solde de clôture, K9 et
     « dont versements en route » (40-5, 40-4) ; le rapport CSV doit avoir un en-tête à au moins deux colonnes (limite de
     40-5, `no_header`) ; la tuile de l'accueil montre la créance hors total (40-4) — et le
     paragraphe de la tuile (`:318-330`) le dit ; **liste des codes courts** (tableau `CHARGE` … `OTHER`, AC-6) en aide
     du critère `event_code_prefix` ; rattachement en lot des transactions de 2024 aux écritures existantes (B17, 38-6c1).
   - PDF régénérés (`make fr` dans `docs/manual/`) et commités ; contrôle **aplati** (`pdftotext user-manual.pdf - | tr
     '\n' ' ' | tr -s ' '`, et de même `admin-manual.pdf` : « clé restreinte », « Lecture », « Tester ») : chaque code de la liste, « clé restreinte », « versements en route », « créance » ; résultat
     au Change Log. Manuels DE/IT/EN : « à traduire » noté si l'écart existe. Brochure (`marketing-brochure.tex`) et
     `README.md` § *Fonctionnalités* : Stripe ajouté là où PayPal l'est (relu au développement), `website/` relu
     (aucune promesse au-delà de `main`).
9. **AC-9 — E2E du parcours Stripe (NFR-40-6-3, NFR-40-6-1).** `frontend/tests/e2e/provider-stripe.spec.ts`, contre le
   transport factice : préalables par l'API au `beforeAll` (société et exercice **multi-devises**, 37-1 ; compte du plan
   USD « Débiteurs Stripe USD », compte de frais USD, règle `event_code_prefix` `CHARGE` → compte de produits USD,
   compte bancaire UBS USD) ; Administrateur : crée la connexion Stripe à l'écran, puis le test appelle `POST
   /api/v1/_test/provider-connector` avec **`stripeResponses: "default"`** pour ce `connectionId` (40-1 AC-11) ; colle
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
    « Tester » tout vert avec `available`/`pending`, et avec ✗ `permission_present` ; proposition d'un versement (sortie,
    arrivée) ; échec de versement à la file ; résumé du contrôle (b) avec versements en route ; K9 avec « dont versements
    en route » ; aide des codes Stripe au formulaire des règles.
11. **AC-11 — Ce que la story ne touche pas.** Aucune migration ; aucune route ni action d'audit (C-40-6) ; **aucun outil
    MCP** (G17, E23 : connexions, contrôles et transactions de prestataire ne sont pas exposés ; `kesh_ecritures` lit les
    écritures acceptées) ; écrans et textes PayPal inchangés (leurs Vitest et E2E de 38-9a/38-9b verts sans retouche,
    G18) ; aucune donnée réelle de Beat dans les captures ni les tests.

## Tâches

- [ ] T1 — `StripeSettingsFields.svelte`, bras Stripe de `providerSettingsBody`, mode (AC-1).
- [ ] T2 — Bloc de la clé : `stripeKeyRefusal`, messages, avis du mode test (AC-2).
- [ ] T3 — « Tester » Stripe : libellés, remèdes, `available`/`pending` (AC-3).
- [ ] T4 — Proposition de versement, file des échecs (AC-4) ; résumé (b) et K9 (AC-5).
- [ ] T5 — Codes courts : constantes, aides des règles, garde `textes_coherents.rs` (AC-6) ; i18n (AC-7).
- [ ] T6 — Manuels admin et utilisateur, PDF aplatis, brochure, README, site (AC-8).
- [ ] T7 — E2E `provider-stripe.spec.ts`, captures (AC-9, AC-10) ; gates complets.

## Dev Notes

- **Ne rien recopier de 38-9a/38-9b** : un bras de prestataire par composant ou par table de `provider-labels.ts` ;
  `+page.svelte` des comptes bancaires et `ReconciliationProposals.svelte` ne reçoivent rien de plus que ce que 38-9a
  et 38-9b y ont mis.
- **Clé** : le contrôle local `sk_` est une protection de plus, non une validation ; aucune expression régulière de
  format complet (40-1 AC-4) recopiée au TypeScript.
- **Montants** : en devise de la transaction ou du compte, jamais « CHF » supposé (38-9b AC-3) ; `big.js`, jamais
  `Number`.
- **Libellés de 40-1 transmis** : contrôles `mode_unchanged`, `permission_present` ; codes `PROVIDER_SECRET_NOT_RESTRICTED`
  (motifs `secret_key`, `publishable_key`, `unknown_prefix`), `PROVIDER_MODE_CHANGED` (40-1 AC-14).
- **Références** : epic G4, G8, G10, G12, G13, G14, G19, C-40-1 à C-40-6, Q-40-2, Q-40-8, Q-40-10 ; 40-1 AC-2, AC-4,
  AC-5, AC-9, AC-11, AC-14 ; 40-2 AC-2, AC-6 ; 40-4 AC-2, AC-4, AC-5, AC-7 ; 40-5 AC-6 ; 38-9a AC-2 à AC-13 ; 38-9b
  AC-1, AC-3, AC-10, AC-11.

## Questions ouvertes

- **Q-40-8 (recette 40-0)** — noms exacts des ressources à cocher en *Lecture* : sans eux, le manuel ne peut lister les
  droits (AC-8) ; la story ne se développe pas avant que 40-0 les ait relevés.
- **Inter-fiches 40-3** — la fiche 40-3 (relue le 2026-11-15) n'ajoute **aucun** `valueDate` au DTO de transaction des
  propositions (38-6b/38-9b ne portent que `bookingDate`) ; la date de l'écriture proposée (`entry_date`, 38-6a C-38-6a-10)
  n'est pas non plus nommée au DTO. Texte proposé à 40-3 (AC-3 ou AC-1) : « le DTO de transaction de `GET
  /api/v1/reconciliation/proposals` gagne `valueDate` (`AAAA-MM-JJ` ou `null`, additif) et la proposition `provider` de
  type retrait gagne `entryDate` (la date de l'écriture proposée), PayPal : `valueDate: null`, `entryDate` = `bookingDate` ;
  test de clés exactes PayPal inchangées ». 40-6 AC-4 lit ces deux noms.
- **Inter-fiches 40-2** — liste publique `OVERRIDABLE_CODES` (AC-6) : texte proposé à 40-2 AC-6 : « les codes surchargeables
  sont une constante `pub` du module `stripe::event_codes`, lue par la validation et par la garde de 40-6 ».

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

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
