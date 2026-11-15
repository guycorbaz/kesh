# Story 38-9b : écrans — règles étendues, propositions multi-lignes, rattachement en lot, file « à examiner » ; manuel utilisateur ; E2E

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 (Opus), branche `epic/38-paypal`. Epic 38 (`planning-artifacts/epic-38-paypal.md`),
E14, E15, E16, E21, E22 ; contrats C-38-6, C-38-8, C-38-9, C-38-14. Décisions B3, B4, B5, B7, B17, B19, B21, B24.
**Dépend de** 38-4 (règles : types, filtres, `RULE_CRITERION_UNKNOWN`), 38-6a (candidat `provider`, `providerLines`,
`blockedBy`, variante `provider` du lot, `confirmAlreadyBooked`), 38-6b (conversions, retraits, remboursements, file
`GET /reconciliation/review`, `…/review-done`, annulation d'une paire), 38-6c (`link-suggestions`, `link-entries`,
rattachement en devise), 38-8 (genres de mouvement) et 38-9a (vocabulaire et `provider-labels.ts`). **Aucune
migration, aucune route.** Dernière story de l'epic avant 38-10 côté écrans.*

## Story

En tant que comptable d'une société qui encaisse par PayPal, je veux écrire des règles qui reconnaissent une vente
par sa boutique (préfixe du numéro de facture), un courriel, un article ou un code d'événement, et les restreindre par
sens, devise et compte ; voir pour chaque transaction téléchargée l'écriture complète que Kesh propose — brut, TVA,
frais, cours — et l'accepter en lot ; rattacher en lot à leurs écritures existantes les transactions de 2024 déjà
comptabilisées ; et traiter à part celles qui attendent un humain — afin de tenir mon compte PayPal sans saisie, sans
doublon et sans qu'aucune écriture ne passe sans que je l'aie vue.

## Exigences

- `FR-38-9b-1` — Formulaire des règles : types neufs (dont boutique) et filtres sens, devise, compte → AC-1, AC-2.
- `FR-38-9b-2` — Propositions multi-lignes lisibles (lignes, TVA, frais, cours, « déjà comptabilisée »), acceptées en
  lot → AC-3, AC-4, AC-5.
- `FR-38-9b-3` — Rattachement en lot : univoques à cocher, ambiguës une à une → AC-6.
- `FR-38-9b-4` — File « à examiner » et action « examiné » → AC-7.
- `FR-38-9b-5` *(ajoutée)* — Annuler le rapprochement d'une moitié de conversion dit, avant confirmation, que l'autre
  moitié est annulée aussi (38-6b AC-8) → AC-8.
- `NFR-38-9b-1` — E2E contre le transport factice (téléchargement → règle → proposition → acceptation, et rattachement
  en lot) → AC-11.
- `NFR-38-9b-2` — Textes « proposer », jamais « comptabiliser automatiquement » ; quatre locales ; manuel utilisateur
  et PDF à jour → AC-9, AC-10.
- `NFR-38-9b-3` — Captures d'écran de chaque état touché → AC-12.
- `NFR-38-9b-4` *(ajoutée)* — Le code neuf vit dans des composants à part : `ReconciliationProposals.svelte`
  (836 lignes) et `RuleFormModal.svelte` (291 lignes) ne reçoivent que des appels → AC-3, AC-1.

## Critères d'acceptation

1. **AC-1 — Formulaire des règles : types neufs** (`frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte`,
   38-4 AC-4, AC-10).
   - Le sélecteur de type (rendu **en création seulement**, `{#if !isEdit}` aujourd'hui, `RuleFormModal.svelte:147`
     — le type ne se modifie pas, 38-4 AC-4) gagne les quatre types, aux libellés de 38-4 AC-10 :
     « Courriel exact » (`counterparty_email_exact`), « Boutique (préfixe du numéro de facture) » (`invoice_prefix`),
     « Article contient » (`item_name_contains`), « Code d'événement commence par » (`event_code_prefix`) ; la liste
     des types vient d'**une** constante partagée avec `RulesList.svelte` (`matchTypeLabel`, `RulesList.svelte:31`) —
     aujourd'hui les libellés sont écrits deux fois (`RuleFormModal.svelte:158-178`, `RulesList.svelte:31-52`) :
     extraits dans `rules-labels.ts` (DRY).
   - Aide sous la valeur, par type : boutique → « Début du numéro de facture que votre boutique envoie à PayPal, p. ex.
     `LG-` ; les majuscules et les espaces ne comptent pas » ; courriel → « Adresse exacte du payeur ou du marchand » ;
     article → « Une partie du nom d'un article » ; code → « Code PayPal, p. ex. `T0006` ou `T00` ». Les quatre types
     neufs ne valent que pour des transactions de prestataire (champs vides ailleurs) : l'aide le dit.
   - Une règle d'un type **inconnu** de ce binaire (38-4 AC-7) s'affiche « Critère d'une version ultérieure (`x`) » ;
     son bouton *Modifier* est **désactivé** avec l'infobulle « Cette règle a été créée par une version plus récente de
     Kesh ; elle n'est jamais appliquée ici. Vous pouvez la désactiver. » ; *Désactiver* reste. Un `409
     RULE_CRITERION_UNKNOWN` reçu quand même (liste périmée) affiche son message.
2. **AC-2 — Filtres** (38-4 AC-3, AC-4, AC-6).
   - Trois champs facultatifs, en création **et** en modification : **Sens** (« Entrées et sorties » = `null`,
     « Entrées » = `in`, « Sorties » = `out`), **Devise** (« Toutes » = `null`, puis les devises des comptes bancaires
     de la société, lues de `GET /api/v1/bank-accounts` — aucune liste ISO recopiée), **Compte bancaire** (« Tous » =
     `null`, puis les comptes bancaires actifs, libellé `bankAccountOptionLabel`, `frontend/src/lib/features/bank-accounts/format.ts`).
     Aide : « Le sens ne suffit pas seul : il départage une vente et un achat du même courriel » (E14).
   - **Modification** : le formulaire envoie **toujours** les trois filtres au `PATCH` (`null` = retiré, 38-4 AC-4 :
     `double_option`) — ce qui remplace le comportement de 38-4 AC-10 (filtres absents = inchangés) maintenant que le
     formulaire les montre ; test Vitest sur le corps envoyé (filtre retiré → `null` présent ; filtre gardé → valeur).
   - **Compte de contrepartie** : le sélecteur admet les comptes en devise (38-4 AC-6) ; si une devise est choisie au
     filtre, les comptes **d'une autre devise étrangère** sont retirés du sélecteur (CHF reste) — miroir de
     `RULE_ACCOUNT_CURRENCY_MISMATCH`, que le serveur rend de toute façon (message affiché).
   - Refus affichés sous le champ concerné : `RECONCILIATION_RULE_DUPLICATE` (message qui cite type, valeur **et
     filtres**, `details` de 38-4 AC-5), `CURRENCY_UNKNOWN`, `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`,
     `RULE_ACCOUNT_CURRENCY_MISMATCH`, `VALIDATION_ERROR` du courriel.
   - **Liste** (`RulesList.svelte`) : les filtres s'affichent déjà depuis 38-4 AC-10 ; rien de plus ici que les
     libellés partagés.
3. **AC-3 — Proposition de prestataire à l'écran** (38-6a C-38-6a-3, C-38-6a-4 ; 38-6b AC-1). Dans
   `ReconciliationProposals.svelte`, une candidate `candidateType === 'provider'` est rendue par un composant neuf
   `ProviderProposalPreview.svelte` (le fichier hôte ne reçoit que l'appel et le bras du `switch`) :
   - en-tête : genre (`movementKind`, libellé de `provider-labels.ts` de 38-9a : « Vente », « Achat », « Frais »,
     « Conversion », « Retrait », « Dépôt », « Remboursement » ; inconnu → « Mouvement (`x`) »), règle appliquée
     (libellé et critère), boutique (`invoiceRef`) si présente, code d'événement ;
   - **aperçu des lignes** (`providerLines`) en tableau : compte (numéro, nom), débit, crédit, et pour une ligne en
     devise le montant en devise et le cours ; la ligne de **TVA** (genre et taux : « TVA due 8.1 % », « Impôt préalable
     8.1 % », « Impôt sur les acquisitions 8.1 % — deux lignes égales ») ; la ligne de **frais** (compte de frais de la
     devise) ; la **ligne d'arrondi** si elle existe, nommée (« Arrondi de change », 38-6a C-38-6a-6 : jamais
     silencieuse) ; le **cours** (`rate`, `rateMonth`, source « AFC » ou « saisi ») ; totaux débit = crédit en CHF ;
   - **conversion** (38-6b AC-1) : les **deux** écritures (`providerLines` et `pairedLines`), la ligne de différence de
     change, et la mention « accepter rapproche aussi la transaction <date, montant, devise> du compte <nom> » ;
   - **retrait / dépôt** : le compte bancaire suggéré (`transferBankAccountId`, sélecteur des comptes bancaires de même
     devise quand il est `null` ou à changer) **ou** l'écriture existante à rattacher (`linkJournalEntryId`, avec lien
     vers l'écriture) ;
   - **remboursement** : l'origine (`originBankTransactionId` : date, montant, boutique), « inverse de l'écriture
     d'origine, aux montants rapportés par PayPal » ;
   - **bloquée** (`blockedBy`) : pas de lignes ; le message du code (`failed-proposal-label.ts`, libellés posés par 38-6a
     AC-12 et 38-6b AC-10, `details` lus) et **le remède** en lien : `RATE_UNAVAILABLE` → cours de la devise et du mois
     (page des cours de 37-2) ; `VAT_ACCOUNT_NOT_CONFIGURED` → `/settings/invoicing` (réglages TVA, désignation) ;
     `FEE_ACCOUNT_INVALID` → réglages de la connexion (38-9a, Administrateur) ; `MULTI_CURRENCY_DISABLED` → réglages de
     devise. La case de la ligne est **désactivée** (une proposition bloquée ne part pas dans un lot).
   - Les candidates existantes (facture, règle, facture fournisseur) gardent leur rendu ; sur un compte prestataire en
     CHF, une facture qui masque le candidat `provider` (38-6a C-38-6a-7) est rendue comme aujourd'hui.
4. **AC-4 — Acceptation en lot** (38-6a AC-7, C-38-6a-9 ; 38-6b AC-2 à AC-5). `reconciliation.api.ts` : le type
   `AcceptProposalInput` (`reconciliation.types.ts:109`) gagne la variante `{ type: 'provider', bankTransactionId,
   ruleId?, counterpartyAccountId?, confirmAlreadyBooked?, pairedBankTransactionId?, transferBankAccountId?,
   linkJournalEntryId?, originBankTransactionId? }` — **champs repris tels que la proposition les porte**, jamais de
   lignes (le serveur recalcule, C-38-6a-9). Une fonction pure `providerAcceptInput(proposal, choices)` les assemble
   (test Vitest par genre).
   - **« Déjà comptabilisée »** (E15, 38-6a AC-9) : une transaction de prestataire signalée (`isAlreadyBooked`,
     `already-booked.ts`) n'est **jamais cochée d'office** ; cochée par l'utilisateur, elle passe par le dialogue
     existant `AlreadyBookedConfirmDialog.svelte` (garde `guard`, `ReconciliationProposals.svelte:315`), et seules les
     lignes confirmées partent avec `confirmAlreadyBooked: true` ; l'avertissement de la ligne renvoie au
     **rattachement** (AC-6) comme voie normale. Un `failed[]` `TRANSACTION_ALREADY_BOOKED` (confirmation absente : lot
     d'un autre onglet) s'affiche avec son libellé (38-6a).
   - Bouton **« Cocher les propositions prêtes »** (au-dessus de la liste, comptes prestataires seulement) : coche les
     lignes à candidat `provider` **non bloquées, non signalées « déjà comptabilisée »** — c'est le « tout accepter » de
     E15, qui exclut les signalées ; rien n'est envoyé sans le clic sur *Accepter (N)*.
   - Bilan du lot : existant (compteur, *Échecs partiels* avec les transactions relevées, `failedTx`) ; les codes neufs
     ont leurs libellés par 38-6a/38-6b ; une conversion acceptée compte **une** proposition et cite la moitié rapprochée
     avec elle (`pairedBankTransactionId` de l'`AcceptedProposal`).
5. **AC-5 — Comptes et vocabulaire de la page de réconciliation.**
   - Le sélecteur de compte (`routes/(app)/reconciliation/+page.svelte`) marque les comptes prestataires (« PayPal ·
     EUR », bloc `provider` de 38-2 AC-8, `GET /bank-accounts` — la page lit aujourd'hui `companies/current`,
     `+page.svelte:23` : lire les comptes par `GET /api/v1/bank-accounts` si `companies/current` ne porte pas le
     bloc `provider`).
   - Le sous-titre « Propositions automatiques de matching transaction ↔ facture » (`+page.svelte:41-44`) devient
     « Kesh propose ; rien n'est comptabilisé sans votre accord » (clé existante `reconciliation-page-subtitle`
     réécrite aux quatre locales) — NFR-38-9b-2.
   - Pour un compte prestataire, trois onglets : **Propositions** (existant), **À examiner (N)** (AC-7), **Rattacher en
     lot** (AC-6) ; pour tout autre compte, la page est inchangée.
6. **AC-6 — Rattachement en lot** (FR-38-9b-3, E21, C-38-14 ; routes de 38-6c).
   - Onglet « Rattacher en lot » : période (du, au ; défaut : l'exercice de la transaction en attente la plus ancienne),
     bouton « Chercher » → `GET /api/v1/reconciliation/link-suggestions?bankAccountId&from&to`.
   - **Univoques** : tableau transaction (date, montant en devise, contrepartie, boutique) ↔ écriture (date, numéro,
     libellé, montant sur le compte), une case par ligne, **non cochées** ; bouton « Tout cocher » (geste explicite) ;
     « Rattacher (N) » → `POST /api/v1/reconciliation/link-entries` avec `[{ bankTransactionId, journalEntryId, version
     }]` tels que la suggestion les rend ; si 38-6c borne la taille d'un lot, l'écran envoie par paquets de cette borne,
     séquentiellement, et agrège le bilan ; un paquet en échec HTTP (4xx/5xx global) arrête les suivants et le dit.
   - **Ambiguës** : liste à part, une ligne par transaction avec le nombre de candidates ; « Choisir » ouvre la modale
     existante `LinkSettlementModal.svelte` (rattachement un par un, `…/entry-candidates` et `…/link-entry`, en devise
     depuis 38-6c), qui rafraîchit la liste à la fermeture.
   - **Bilan** : rattachées, et `failed[]` avec libellés (`ENTRY_NOT_LINKABLE`, `ENTRY_ALREADY_LINKED`,
     `AMOUNT_MISMATCH`, `VERSION_CONFLICT` : codes de C-38-14, libellés posés par 38-6c ; sinon ajoutés ici dans
     `failed-proposal-label.ts`, quatre locales, décompte du doc-comment recompté). Une conversion (deux transactions,
     une écriture) apparaît avec ses deux transactions (34-11b) ; le texte le dit.
   - Aucun rattachement sans clic ; le vocabulaire est « rattacher », jamais « comptabiliser ».
7. **AC-7 — File « à examiner »** (FR-38-9b-4, C-38-9 ; 38-6b AC-6, AC-7).
   - Onglet « À examiner (N) » (N = longueur rendue, « 500+ » si `hasMore`) : `GET /api/v1/reconciliation/review?
     bankAccountId&limit` ; tableau date, montant, devise, genre, code d'événement, contrepartie, boutique, **motif**
     (libellé par valeur de `reason` : litige, origine du remboursement absente, origine non rapprochée, origine non
     inversible, moitié de conversion absente, paire de conversion ambiguë, code PayPal inconnu, autre ; valeur inconnue
     → « Motif inconnu (`x`) »), lien vers la transaction liée (`relatedBankTransactionId`).
   - Actions par ligne : **« Écriture manuelle »** (modale existante `ManualMatchModal.svelte`, 34-3) ; **« Rattacher à
     une écriture existante »** (`LinkSettlementModal.svelte`) ; **« Marquer examiné »** → confirmation (« la
     transaction sort de la file sans écriture ; *Annuler le rejet* depuis le détail de l'import la fait revenir ») puis
     `POST /api/v1/reconciliation/transactions/{id}/review-done` (204) ; refus (`PROVIDER_ACCOUNT_REQUIRED`,
     `BANK_ACCOUNT_CLOSED`, `RECONCILIATION_TRANSACTION_NOT_PENDING`, `RECONCILIATION_ALREADY_REJECTED`) avec leur
     message. Rôles : boutons d'écriture pour Comptable et Administrateur ; Consultation lit la file sans action.
8. **AC-8 — Annuler une paire** (FR-38-9b-5 ; 38-6b AC-8). `CancelReconciliationDialog.svelte` : quand la vue de la
   transaction porte `pairedBankTransactionId`, le dialogue l'annonce avant confirmation (« L'autre moitié de cette
   conversion — <date, montant, devise, compte> — sera annulée aussi ; deux écritures seront contre-passées ») ; après
   succès, le message cite les deux contre-passations (`pairedReversalJournalEntryId`). Test Vitest.
9. **AC-9 — Textes et i18n** (NFR-38-9b-2). Clés neuves écrites en toutes lettres (`reconciliation-provider-*`,
   `reconciliation-review-*`, `reconciliation-link-batch-*`, `reconciliation-rules-filter-*` complétées,
   `reconciliation-rules-help-*`), aux quatre catalogues `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`,
   par `i18nMsg` avec repli français ; pas de sélecteur Fluent (paires `-one`/`-other`, raison écrite dans
   `already-booked.ts`) ; `npm run lint-i18n-ownership` et `i18n-keys.test.ts` verts. Interdits, contrôlés par un test
   Vitest qui lit les quatre catalogues pour les clés neuves : « automatiquement comptabilis… », « écrit
   automatiquement », et leurs équivalents DE/IT/EN (« automatisch verbucht », « contabilizzat… automaticamente »,
   « automatically booked/posted ») — liste écrite dans le test.
10. **AC-10 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`).
    - `\subsection{Règles d'affectation automatique}` (ligne 3188) : la phrase « Une règle porte **une seule**
      condition » devient « un critère, et au plus trois filtres » ; « Une règle ne s'applique qu'aux transactions en
      CHF » devient faux pour les comptes prestataires (38-6a) : réécrit ; les quatre critères neufs, la boutique =
      préfixe du numéro de facture (B24), les filtres (sens, devise, compte), « une règle par boutique et par devise »
      (B19), la priorité entre préfixes emboîtés (`LG` et `LGT` : la plus petite priorité l'emporte — fait transmis par
      38-4 AC-12), le compte de règle en devise, la règle d'une version ultérieure ; reprise des faits listés au Change
      Log de 38-4.
    - Sous `\section{Réconciliation bancaire}` (ligne 3004), sous-section neuve **« Comptes de prestataires de paiement
      (PayPal) »** : ce que Kesh télécharge et quand (renvoi au manuel d'administration pour la connexion) ; lire une
      proposition (lignes, TVA, frais, cours, arrondi de change) ; accepter en lot et « Cocher les propositions
      prêtes » ; « déjà comptabilisée » et pourquoi il faut alors rattacher ; conversions (deux écritures, une
      acceptation) ; retraits et dépôts (virement entre vos comptes, 34-11b) ; remboursements (inverse, montants
      rapportés, B20) ; litiges et la file « à examiner » ; **rattachement en lot de l'exercice déjà saisi** (B17) ;
      annuler une conversion ; le cours au moment de l'acceptation (C-38-6a-9).
    - **Voie des scripts externes** (E22, B21) : un paragraphe qui dit qu'un script (WHMCS, CB Paid Subscriptions) peut
      chercher une transaction de prestataire, créer l'écriture puis la rattacher par une clé d'API `read-write`, avec
      renvoi à `docs/api-external.md` (écrit par 38-6c) — sans recopier l'API.
    - FAQ `Une transaction bancaire ne propose aucun match automatique` (ligne 4125) : complétée pour un compte
      prestataire (file « à examiner », proposition bloquée).
    - Captures référencées (`\keshscreenshot`, `docs/manual/fr/screenshots/`) pour la proposition multi-lignes, la file
      et le rattachement en lot ; PDF régénéré (`make fr`), contrôle du **PDF aplati** (`pdftotext user-manual.pdf - | tr
      '\n' ' ' | tr -s ' '`) : « une seule condition » et « qu'aux transactions en CHF » absents ou requalifiés,
      « préfixe du numéro de facture » présent — résultat au Change Log. Manuels DE/IT/EN : « à traduire » noté.
11. **AC-11 — Tests** (NFR-38-9b-1).
    - **Vitest** : formulaire des règles (types, aides, filtres envoyés au `POST` et au `PATCH`, règle inconnue non
      modifiable, refus `details`), `rules-labels.ts` partagé, `providerAcceptInput` par genre, `ProviderProposalPreview`
      (vente avec TVA et frais, achat avec acquisitions, frais seul, conversion à deux écritures, retrait suggéré et sans
      suggestion, remboursement, bloquée avec remède, arrondi visible), « Cocher les propositions prêtes » (exclut
      bloquées et signalées), lot avec `confirmAlreadyBooked` sur les seules lignes confirmées, rattachement en lot
      (paquets, arrêt sur erreur globale, bilan), file (actions selon rôle, motif inconnu), dialogue d'annulation d'une
      paire, catalogues sans les formules interdites.
    - **E2E** `frontend/tests/e2e/provider-reconciliation.spec.ts`, contre le **transport factice** de 38-3 et ses
      réponses PayPal par défaut (38-7 AC-10 : familles de 38-8 couvertes), connexion et comptes posés par l'API dans le
      `beforeAll` (le parcours d'administration est l'E2E de 38-9a) : (1) téléchargement à la demande ; (2) création
      d'une règle `invoice_prefix` (`LG-`, filtre EUR, entrées) vers un compte de ventes EUR à TVA par défaut (38-5) ;
      (3) la proposition de la vente montre brut, TVA, frais, cours ; (4) « Cocher les propositions prêtes » puis
      *Accepter* → écriture créée (vérifiée par `GET /journal-entries` : lignes, TVA, frais), transaction rapprochée ;
      (5) une conversion acceptée rapproche ses deux moitiés ; (6) un litige est dans « À examiner », « Marquer examiné »
      l'en sort ; (7) **rattachement en lot** : une écriture posée par `POST /journal-entries` (en devise) pour une
      transaction téléchargée, la suggestion univoque cochée et rattachée, la transaction sort des propositions ;
      (8) « déjà comptabilisée » : la ligne n'est pas cochée par « Cocher les propositions prêtes ». Chaque assertion
      d'écriture passe par l'API (la frontière HTTP, `CLAUDE.md` § E2E).
    - Gates : `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` ; **E2E complet au dernier commit de code**
      (`CLAUDE.md` : « rejouée au push » interdit) ; les échecs jugés fichier par fichier contre `docs/testing.md`
      § *Les échecs attendus*.
12. **AC-12 — Recette visuelle** (NFR-38-9b-3). Captures, thème clair, base de démonstration, dans `kesh-gate-logs/`,
    jointes au Dev Agent Record et au prompt de revue : formulaire de règle (chaque type neuf, filtres remplis), liste
    avec règle inconnue (bouton désactivé) ; proposition de vente (TVA, frais, cours), d'achat à acquisitions, de frais
    seul, de conversion (deux écritures), de retrait (suggéré, sans suggestion), de remboursement, bloquée (chaque
    remède), avec arrondi ; ligne « déjà comptabilisée » et dialogue de confirmation ; bilan d'un lot avec échecs ;
    onglet « Rattacher en lot » (univoques, ambiguës, bilan) ; file « à examiner » (chaque motif, confirmation) ;
    dialogue d'annulation d'une paire ; sous-titre de la page.
13. **AC-13 — Ce que la story ne touche pas.** Aucune migration ni route ; aucun audit neuf (les routes appelées
    auditent) ; **aucun outil MCP** (E23 : propositions, file et suggestions ne sont pas exposées ; les écritures
    passées se lisent par `kesh_ecritures`) ; aucune donnée réelle de Beat dans les tests ni les captures.

## Écart à l'epic

- **38-4 AC-10 dit le formulaire inchangé et un `PATCH` sans filtres** ; cette story montre les filtres et les envoie
  toujours (AC-2) : le contrat de 38-4 (absent = inchangé, `null` = retiré) le permet, et le test de 38-4 sur le corps
  du formulaire est remplacé par celui de l'AC-2.
- **Manuel** : deux phrases du manuel utilisateur deviennent fausses avec l'epic (« une seule condition », « qu'aux
  transactions en CHF », `user-manual.tex`, § *Règles d'affectation*, *Application des règles*) ; aucune fiche ne les
  réécrivait avant celle-ci (38-4 AC-12 les renvoie ici).
- **« Tout accepter »** (E15) n'existe pas à l'écran (la sélection est par case de transaction,
  `ReconciliationProposals.svelte:98-100`) : il devient « Cocher les propositions prêtes » (AC-4).

## Questions ouvertes

- **Q-38-9b-1** (validation de 38-6c) : borne de taille d'un lot `link-entries` et forme exacte de la réponse de
  `link-suggestions` (champs de l'écriture, `version`).
- **Q-38-9b-2** (Beat) : défaut de la période du rattachement en lot — l'exercice de la plus ancienne transaction en
  attente (proposé) ou l'exercice 2024 désigné ?

## Tâches

- [ ] `rules-labels.ts` partagé ; formulaire des règles : types, aides, filtres, règle inconnue (AC-1, AC-2).
- [ ] Types et API : variante `provider` d'`AcceptProposalInput`, `providerAcceptInput`, types de 38-6b/38-6c
      (AC-4, AC-6, AC-7).
- [ ] `ProviderProposalPreview.svelte` et branchement dans `ReconciliationProposals.svelte` (AC-3).
- [ ] « Cocher les propositions prêtes », confirmation « déjà comptabilisée », bilan (AC-4).
- [ ] Page de réconciliation : comptes marqués, sous-titre, onglets (AC-5).
- [ ] Onglet « Rattacher en lot » (AC-6) ; onglet « À examiner » (AC-7) ; dialogue d'annulation d'une paire (AC-8).
- [ ] i18n quatre locales et test des formules interdites (AC-9).
- [ ] Manuel utilisateur, captures, PDF, contrôle du PDF aplati (AC-10).
- [ ] Vitest, E2E, captures (AC-11, AC-12) ; gates complets au dernier commit de code.

## Dev Notes

- **Fichiers hôtes à ne pas grossir** : `ReconciliationProposals.svelte` (836 lignes), `RuleFormModal.svelte` (291),
  `routes/(app)/reconciliation/+page.svelte` (77) ; composants neufs dans `lib/features/reconciliation/` (et
  `…/rules/`).
- **Patrons** : confirmation « déjà comptabilisée » — `AlreadyBookedConfirmDialog.svelte`, `already-booked.ts`
  (`isAlreadyBooked`, `alreadyBookedWarning`) ; libellés de refus — `failed-proposal-label.ts` (clés en toutes
  lettres, repli sur le code brut) ; rattachement un par un — `LinkSettlementModal.svelte` ; écriture manuelle —
  `ManualMatchModal.svelte` ; annulation — `CancelReconciliationDialog.svelte`.
- **Le serveur recalcule** l'écriture à l'acceptation (C-38-6a-9) : l'aperçu est une **lecture** ; un cours changé entre
  l'affichage et l'acceptation donne l'écriture au cours du moment — dit au manuel (AC-10).
- **Points de contact** : 38-4 (types, filtres, codes, libellés), 38-6a (candidat, codes, corps), 38-6b (champs de
  conversion, retrait, remboursement, file, annulation), 38-6c (suggestions, lot, codes, API des scripts), 38-9a
  (`provider-labels.ts`, vocabulaire), 38-5 (genres de TVA affichés).
- **Splitting** : `frontend`, i18n, manuel — dans la règle.

### References

- `planning-artifacts/epic-38-paypal.md` § E14–E16, E21, E22, C-38-6, C-38-8, C-38-9, C-38-14, 38-9b.
- `implementation-artifacts/38-4-criteres-des-regles.md` AC-4 à AC-7, AC-10, AC-12 ;
  `38-6a-propositions-ventes-achats-frais.md` C-38-6a-3 à C-38-6a-9, AC-7, AC-9, AC-12 ;
  `38-6b-conversions-remboursements-litiges.md` AC-1 à AC-8, AC-10 ; `38-9a-ecrans-connexions-et-comptes.md` AC-10.
- `CLAUDE.md` § *E2E (Playwright)*, *Recette visuelle*, *Un appariement automatique propose, il ne crée jamais*,
  *Le prompt d'une passe doit NOMMER le manuel*.

## Change Log

- 2026-11-15 — rédaction (Opus).
