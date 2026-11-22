# Story 40-7d : écrans, manuel et E2E du compte Stripe (phase 1)

Status: done

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, chemins relevés sur `origin/main` au commit `fd04099f`. Epic 40,
**phase 1 KISS** (décision du responsable de l'epic du 2026-11-20). **Dépend de** : 40-7b, 40-7c ; les écrans PayPal de la phase 1 de
l'Epic 38 (38-11d : page, feature, `provider.ts` / `withoutProviders`, modale de rattachement, manuel, `api-external.md`, E2E), dont elle
reprend les composants sans les recopier ; la phase 1 de l'Epic 39 pour la seule E2E (réception côté Wise). Dernière story de la phase 1.*

## Story

En tant que **Comptable**, je veux régler mon compte Stripe, importer son rapport d'un bouton, voir ce que Kesh propose
(rattachements aux ventes, frais, versements) et l'accepter en lot, et parcourir les écarts et les lignes à examiner, et je veux que le
manuel me dise comment exporter le bon rapport de Stripe et ce que Kesh en fait.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-40-7d-1` | Le formulaire d'un compte bancaire offre les réglages Stripe (compte Stripe, compte de frais, compte de versement), à côté de ceux de PayPal, par les mêmes composants. | AC-1 |
| `FR-40-7d-2` | Sur la page Stripe — le composant de la page PayPal généralisé (38-11d, 39-11d) —, un bouton *Importer un rapport Stripe* envoie le fichier, montre le compte rendu et recharge la revue. | AC-2 |
| `FR-40-7d-3` | Propositions à cocher et « Accepter la sélection », écarts et « à examiner » par compte Stripe et exercice, avec leurs libellés. | AC-3 |
| `FR-40-7d-4` | Manuel utilisateur et `docs/api-external.md` : exporter le rapport (colonne `source_id`), régler, importer, accepter les propositions, lire les écarts, limites. | AC-4 |
| `NFR-40-7d-1` | E2E du parcours sur la fixture fictive ; captures de chaque état touché (recette visuelle). | AC-5 |

## Critères d'acceptation

1. **AC-1 — Paramètres Stripe.** Page *Comptes bancaires* (`frontend/src/routes/(app)/bank-accounts/+page.svelte`,
   formulaires de création en ligne `submitCreate` et de modification `submitUpdateFull`, **non**
   `BankAccountJournalLinkForm.svelte`, le lien par `PATCH` ; client
   `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts`) : le choix du prestataire (aucun, PayPal, Stripe) est
   le bloc que 38-11d a posé (généralisé par 39-11d si Wise l'a fait) ; pour Stripe : libellé du compte, compte de frais
   (comptes de charge, filtré CHF ou devise du compte lié), **compte bancaire de versement** (sélecteur des comptes
   bancaires de même devise que le compte lié, sans prestataire ou Wise, jamais Stripe ni PayPal ; il se vide quand le
   compte lié change). Les refus de 40-7b AC-2 s'affichent sur leur champ (codes traduits aux quatre locales). Aucun
   composant recopié de PayPal : le bloc commun est paramétré (DRY ; s'il est propre à PayPal, il est généralisé ici,
   tests Vitest de 38-11d **inchangés en assertions**). ⚠️ **Là où un compte Stripe ne doit pas paraître** :
   `isProvider` / `withoutProviders` sont **déjà livrés par 38-11d** (`frontend/src/lib/features/bank-accounts/provider.ts`,
   `provider !== null`, patron de `credit-card.ts:39`) et appliqués à **cinq sélecteurs** (`BankImportUpload.svelte`,
   `ManualStatementForm.svelte`, `SettleInvoiceDialog.svelte`, `reconciliation/+page.svelte`, paiement d'une facture
   fournisseur `supplier-invoices/[id]/+page.svelte`) : **réutilisés, non regénéralisés** ; cette story ne fait que les tester avec un compte Stripe (sans eux, un compte Stripe s'offrirait à l'import d'un relevé bancaire ou au règlement
   d'une facture, que le serveur refuse (`BANK_ACCOUNT_IS_PROVIDER`)) ; Vitest : chaque site, avec un compte Stripe, PayPal
   et Wise. `BankAccountSummary` porte le prestataire et `payoutBankAccountId` sous la forme de la réponse de 40-7b.
2. **AC-2 — Page Stripe et import.** Page `frontend/src/routes/(app)/reconciliation/stripe/+page.svelte`, sur le
   **composant de la page PayPal généralisé** (38-11d AC-3 à AC-5, paramétré par le prestataire par 39-11d AC-2 :
   sélecteurs compte et exercice, bouton d'import, revue rechargée après l'import et après l'acceptation, garde de
   génération, propositions par paquets de `MAX_IN_CLAUSE_IDS`, écarts, « à examiner ») — **aucun second composant** ;
   ce qui est propre à Stripe vit dans `frontend/src/lib/features/stripe/` (routes, libellés, forme du compte rendu).
   Entrée « Stripe » du menu `Mensuel` à côté de « PayPal » et « Wise », **dans `comptableOnly`** (le menu ne charge pas
   les comptes ; la page dit « aucun compte Stripe » avec le lien vers *Comptes bancaires* quand il n'y en a pas).
   Sélecteur du compte Stripe (libellés distincts des comptes bancaires `provider = 'stripe'`), bouton **« Importer un
   rapport Stripe »** : choix du fichier (`accept=".csv,text/csv"`), envoi par `apiClient.postFormData` (patron de
   `createBankImport`, `bank-import.api.ts:70-79`) à `POST /api/v1/stripe-imports` (`file`, `account`), puis la revue
   (`GET /api/v1/stripe/review`) se recharge. Compte rendu de l'import affiché jusqu'au prochain : par devise, insérées
   et doublons ; versements lus. Refus traduits : `STRIPE_CSV_INVALID` rend, par ses codes de 40-7a (fichier et lignes,
   liste fermée), une phrase par cause — colonnes manquantes **nommées**, et pour `source_id|charge_id` le message qui
   dit où ajouter la colonne dans Stripe ; `automatic_payout_id` vide : prendre le rapport des versements automatiques ;
   la devise sans compte nomme les devises. Rappel affiché : « les ventes viennent des boutiques ; importez-les avant le
   rapport Stripe ».
3. **AC-3 — Propositions, écarts, à examiner.** Genres de proposition : « Rattachement à la vente n° … » (`link_entry`),
   « Frais » (`fee`), « Versement vers <compte de versement> » (`payout`, compte et libellé portés par la proposition de la revue, 40-7c AC-4), à cocher, « Tout cocher », **« Accepter la
   sélection »** → `POST /api/v1/stripe/accept` (par paquets, compte rendu cumulé : acceptées, et chaque refus avec son
   message, par le module de libellés de 38-11d généralisé par 39-11d, délégation à `failedProposalLabel`). Écarts —
   codes de 38-11c libellés pour Stripe : `receipt_without_sale` « Charge sans vente », `sale_without_receipt` « Vente
   sans charge », `amount_or_currency_mismatch` « Montant ou devise différents » —, et propres à Stripe :
   `payout_total_mismatch`, `payout_without_receipt` (« réception à rattacher sur la page Wise, ou à la réconciliation
   de la banque », avec le lien vers la page Wise du compte de versement quand c'en est un) ; motifs « à examiner » :
   `ambiguous_sale`, `fee_account_missing`, `payout_account_missing`, `payout_reversal`, `payout_not_negative`,
   `refund`, `other_category` — **un libellé par code**, en toutes lettres (`stripe-…` pour ce qui est propre à Stripe,
   `provider-…` pour le commun, `docs/i18n-key-ownership-pattern.md`) ; chaque ligne avec date, montant et devise,
   identifiant, lien vers la transaction (`/bank-import/[id]`) ou l'écriture. `failed-proposal-label.ts` **n'est pas
   étendu** (son test compare ses codes aux littéraux de `reconciliation.rs`). Un test écrit en dur les codes d'écart et
   de motif contre leur source Rust (patron du test de `failed-proposal-label`). États vides : « Aucune proposition »,
   « Aucun écart ». Textes « proposer », jamais « comptabiliser automatiquement ».
4. **AC-4 — Manuel et `docs/api-external.md`.** *Manuel* — `docs/manual/fr/user-manual.tex`, section « Compte Stripe »
   après « Comptes PayPal » (38-11d), `.tex` seulement (PDF à la release) : exporter le rapport *Rapprochement des
   versements → Détaillé* **avec la colonne `source_id`**, en points décimaux, et le fuseau voulu (la date des lignes est
   celle du fuseau d'export) ; **ne pas filtrer le rapport par catégorie** (le versement serait faux, et sa réception
   introuvable) ; le schéma comptable (vente de la boutique au compte de transit avec l'identifiant au libellé ; frais ;
   versement) ; ce que Kesh propose et ce qu'écrit l'acceptation ; chaque écart et quoi faire ; la réception sur Wise
   (proposée par la page Wise) ou à la banque (« déjà comptabilisée » à la réconciliation) à rattacher à l'écriture du
   versement, jamais comptée deux fois ; limites (remboursements, retours de versement et versements qui en portent à
   passer à la main ; réception sur un autre exercice que le versement : l'écart `payout_without_receipt` demeure
   jusqu'à son rattachement ; annuler le côté Stripe d'un versement dont la réception est rattachée ne retire que le
   lien : rattacher de nouveau, ne pas accepter une seconde fois la proposition ; rattachement en devise seul, les autres gestes de la réconciliation restant en CHF ; aucun code TVA ;
   aucune conversion entre devises, un compte de versement de même devise ; monnaies à trois décimales non servies ;
   petit écart de change possible au compte de transit, traité par la réévaluation de fin d'exercice). Gardes de
   `textes_coherents.rs` **à connaître avant d'écrire** (relevées à 38-11d) : G2 interdit le mot capitalisé « Réglages »
   (écrire « paramètres Stripe ») ; G4-bis/G4-ter n'admettent comme numéro de compte cité que ceux d'un plan livré (dire
   « le compte de transit », « le compte de frais », jamais « 1120 » ni « 4170 ») ; G25 interdit « projet » pour le compte
   analytique. Manuel d'administration : rien de plus que la phrase du registre de 40-7b/40-7c. Autres langues : « à
   traduire » noté. Axe de revue : le manuel contre le code et le PDF régénéré localement (`pdftotext … | tr '\n' ' ' | tr
   -s ' '`), sans commit du PDF. *`docs/api-external.md`* — section neuve à côté de celle de PayPal (38-11d AC-7) :
   `POST /api/v1/stripe-imports`, `GET /api/v1/stripe/review`, `POST /api/v1/stripe/accept` (formes des propositions,
   *Pattern batch*), objet des paramètres du prestataire avec
   `payoutBankAccountId`, exemple `curl` avec une clé `read-write` (identifiants d'exemple **non à quatre chiffres**,
   G4-ter lit ce fichier), le script des ventes des boutiques (identifiant `ch_…` au libellé, ligne du transit en devise) ;
   table des erreurs : codes de 40-7b et 40-7c.
5. **AC-5 — E2E et captures.** `frontend/tests/e2e/stripe-import.spec.ts` (patron de `paypal-import.spec.ts` de 38-11d et
   de `fx-entries.spec.ts` ; aides `frontend/tests/e2e/helpers/bank-import-fixtures.ts`) : société `with-company`,
   **exercice 2020 ouvert au 01.01.2020**, multi-devises activé par l'API (activation irréversible : la spec **reseede
   `with-company` en fin de parcours**), comptes du plan EUR et USD, comptes Stripe CHF/EUR/USD, compte de frais, compte
   Wise EUR comme compte de versement (**dépend de 39-11b** : sa réception est importée par la route Wise avec **une ligne
   de CSV écrite dans la spec**, dont le montant est le net du versement EUR de la fixture de 40-7a — la fixture Wise de
   39-11a n'est pas requise), cours mensuels de 2020 posés par `PUT /exchange-rates/rates`, écritures de vente fictives
   par l'API. Import de la fixture de 40-7a, lue par chemin depuis `crates/kesh-import/tests/fixtures/` (ni copie ni
   seconde fixture), **par l'interface** ; compte rendu attendu ; propositions, un écart de chaque genre de 38-11c
   (`receipt_without_sale`, `sale_without_receipt`) ; « Tout cocher », « Accepter la sélection » par l'interface,
   compte rendu ; écart `payout_without_receipt` affiché, réception Wise rattachée par `POST
   …/link-entry` dans la spec (le geste à l'écran est celui de la page Wise, couvert par l'E2E de 39-11d), revue
   rechargée, écart disparu ; refus du
   fichier sans `source_id` avec sa marche à suivre. Captures (thème clair) de chaque état touché dans
   `KESH_CAPTURES_DIR` (mécanisme de `fx-entries.spec.ts`), hors dépôt (`kesh-gate-logs/`), jointes au Dev Agent Record et
   au prompt de revue. Vitest des composants neufs ; `npm run check`, `lint-i18n-ownership` (préfixes `stripe-` pour
   `features/stripe/`, `bank-accounts-` et `reconciliation-` ailleurs, `nav-*` pour le menu), `sitesTotal` de
   `frontend/src/lib/shared/i18n-keys.test.ts` **recompté** depuis la source après la fusion. Suite E2E complète au dernier
   commit de code (7 à 9 échecs attendus, jugés fichier par fichier, `docs/testing.md`).

## Tâches

- [x] T1 — Bloc de paramètres prestataire généralisé, champs Stripe, `withoutProviders` (déjà livré par 38-11d) testé aux cinq sélecteurs (AC-1).
- [x] T2 — Page Stripe sur le composant généralisé : import, compte rendu, revue (AC-2).
- [x] T3 — Propositions, acceptation, écarts, à examiner, libellés (AC-3).
- [x] T4 — Manuel et `api-external.md` (AC-4).
- [x] T5 — E2E, Vitest, captures (AC-5).

## Stratégie de tests

Vitest (formulaire, compte rendu, libellés des propositions, écarts et motifs, états vides) ; E2E complet au dernier commit de code ; tests
`textes_coherents.rs` du manuel s'il y a lieu.

## Modules touchés (3)

`frontend` (bank-accounts, reconciliation/stripe, E2E), `kesh-i18n` (s'il manque un message serveur), `docs/manual/fr`
et `docs/api-external.md`.

## Note de version

*Ajouté* — Écrans du compte Stripe : paramètres, import du rapport, propositions à accepter en lot, écarts et lignes à
examiner ; section « Compte
Stripe » du manuel utilisateur et de `docs/api-external.md`.

## Notes de développement

- Remplace l'ancienne 40-6 (`superseded`) pour la phase 1.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
- 2026-11-20 — validation P1 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 2 HIGH, 5 MEDIUM, 2 LOW, remédiés
  (rapport `kesh-gate-logs/epic-39-40-kiss/validation/40-7d-ecrans-manuel-stripe-P1.md`). HIGH : aucun compte Stripe
  n'était écarté des sélecteurs d'import de relevé, de règlement de facture et de réconciliation (`withoutPaypal` de
  38-11d ne vise que PayPal) → `withoutProviders` ; AC-2 se contredisait (« envoie le fichier puis le rapprochement » et
  `match=true` sur la route d'import, retiré en 40-7b). MEDIUM : page et menu alignés sur 38-11d (`reconciliation/stripe/`,
  `comptableOnly`, bouton d'import sur la page du prestataire et non sur l'import bancaire) ; modale de rattachement de
  38-11d citée, aucun second composant ; `docs/api-external.md` absent ; E2E sans exercice 2020 ni reseed ni cours
  mensuels, réception Wise sans moyen d'être créée (import de relevé en devise refusé hors prestataire) → ligne de CSV
  Wise dans la spec ; gardes G2, G4-bis/ter, G25 non nommées. LOW : libellés de tous les codes d'écart nommés ; avertissement
  contre le filtrage du rapport.
- 2026-11-20 — validation P2 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 0 LOW, remédié (rapport
  `kesh-gate-logs/epic-39-40-kiss/validation/40-7d-ecrans-manuel-stripe-P2.md`). MEDIUM (propagé de la 40-7c P2, décision
  de l'orchestrateur : une charge est une proposition) : la page enchaînait l'import et un geste de rapprochement, et
  confirmait la réception par la modale de rattachement → page sur le composant de PayPal généralisé par 39-11d
  (revue, propositions à cocher, acceptation en lot), codes d'écart de 38-11c libellés pour Stripe, réception rattachée
  sur la page Wise ou à la réconciliation de la banque (aucune modale propre), E2E par l'acceptation à l'écran.
  Remédiation P1 vérifiée (`withoutProviders`, chemins frontend, gardes du manuel).
- 2026-11-20 — validation P3 ciblée (Claude Sonnet 5.5, contexte frais ; propagation de la 40-7c) : 0 CRITICAL, 0 HIGH,
  0 MEDIUM, 2 LOW, remédiés (rapport `kesh-gate-logs/epic-39-40-kiss/validation/40-7-P3-ciblee.md`). LOW : le libellé
  « Versement vers <compte> » exige que la proposition de la revue porte le compte de versement (40-7c AC-4 d) ; deux
  limites ajoutées au manuel (réception sur un autre exercice, annulation du côté Stripe).
- 2026-11-20 — alignement sur la phase 1 de l'Epic 38 (Claude Sonnet 5.5, contexte frais ; fiches 38-11c1, 38-11c2, 38-11d et rapport de validation P3 de `28623871` lus) : `link_sale` → `link_entry` ; `provider.ts` / `withoutProviders` déjà livrés par 38-11d sur cinq sites : réutilisés, non regénéralisés (plus de `withoutPaypal`, plus de « quatre sélecteurs »). Rapport `kesh-gate-logs/epic-39-40-kiss/validation/alignement-38-P3.md`.
- 2026-11-22 — développée (Claude Opus 5.5) : écrans, manuel, `api-external.md`, E2E ; détail au Dev Agent Record ;
  revue P1 à lancer (prompt `kesh-gate-logs/s40-7d/revue-p1-prompt.md`).
- 2026-11-22 — revue de code P1 (rapport `kesh-gate-logs/s40-7d/revue-p1-rapport.md`) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 9 LOW ;
  remédiée (voir « Remédiation de la revue P1 » au Dev Agent Record) ; passe P2 à lancer (prompt
  `kesh-gate-logs/s40-7d/revue-p2-prompt.md`).
- 2026-11-22 — revue de code P2 ciblée (rapport `kesh-gate-logs/s40-7d/revue-p2-rapport.md`) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 4 LOW ;
  remédiée (§ « Remédiation de la revue P2 ») ; passe P3 ciblée à lancer (prompt `kesh-gate-logs/s40-7d/revue-p3-prompt.md`).
- 2026-11-22 — revue de code P3 ciblée (rapport `kesh-gate-logs/s40-7d/revue-p3-rapport.md`) : 0 CRITICAL, 1 HIGH, 1 MEDIUM, 4 LOW ;
  remédiée (§ « Remédiation de la revue P3 ») ; passe P4 ciblée à lancer (prompt `kesh-gate-logs/s40-7d/revue-p4-prompt.md`).
- 2026-11-22 — revue de code P4 ciblée sur `7bec552a` (rapport `kesh-gate-logs/s40-7d/revue-p4-rapport.md`) : 0 CRITICAL,
  0 HIGH, 0 MEDIUM, 3 LOW ; **boucle close** (aucun finding au-dessus de LOW). Les trois LOW touchent la production
  (`bank-accounts/+page.svelte`) et sortent du périmètre : versés à la liste d'issues à ouvrir de l'epic (destination
  introuvable affichée en blanc, `bug` P4 ; test du repli de `payoutPool`, `technical-debt` P4 ; destination archivée
  non rendue au retour au compte du plan d'origine, `enhancement` P4). **Synthèse de la boucle de revue de code** :
  P1 (complète) 0/0/3 MEDIUM/9 LOW → P2 (ciblée) 0/0/3 MEDIUM/4 LOW → P3 (ciblée) 0/1 HIGH/1 MEDIUM/4 LOW → P4 (ciblée)
  0/0/0/3 LOW. La P3 a trouvé un HIGH **introduit par la remédiation de la P2** (destination vidée à l'ouverture), motif
  « la sévérité se déplace vers ce qu'on vient d'écrire ». Passes en contexte frais, une lentille chacune, lancées par
  l'orchestrateur ; le modèle de chaque passe n'est pas consigné à son rapport. Story `done`.

## Dev Agent Record

### Base et fusion préalable

- Branche `story/40-7d`, partie de `epic/40-stripe` à `e1dd8def` (fusion de la 40-7c, qui porte la 40-7b).
- **Fusion de `story/38-11d` (`f988056c`, parent de la fusion `c94d1ef7` ; `6f8bba02` en est un ancêtre ;
  écrans PayPal, encore en revue dans l'Epic 38) dans la 40-7d**,
  commit `c94d1ef7` : la fiche demande de réutiliser les écrans PayPal sans les recopier, et ils
  n'étaient ni dans `epic/40-stripe` ni dans `epic/38-paypal`. Trois conflits, les deux côtés gardés :
  `sprint-status.yaml` (ligne 38-11d de la 38-11d), `audit_route_registry.rs` (166, celui de
  l'Epic 40 ; garde verte), `docs/api-external.md` (paragraphe PayPal de l'Epic 40 + la phrase de
  `GET /companies/current` de la 38-11d ; table des erreurs : lignes généralisées de l'Epic 40 + les
  lignes `COUNTERPARTY_CURRENCY_MISMATCH` et `PROVIDER_PROPOSAL_STALE` de la 38-11d,
  `PROVIDER_ACCOUNT_NOT_FOUND` avec les routes de revue et d'acceptation des deux prestataires).
  ⚠️ Les remédiations à venir de la 38-11d (revue P3) arriveront par `epic/38-paypal` ; les fichiers
  PayPal que cette story généralise (`reconciliation/paypal/+page.svelte`, `paypal.types.ts`,
  `paypal.api.ts`, `paypal-labels.ts`, `paypal-review.ts`, `bank-accounts/+page.svelte`,
  `provider.ts`) se recroiseront à cette fusion.
- La 39-11d (page généralisée par Wise) et la 39-11b (route Wise) ne sont pas dans `epic/40-stripe` :
  la page commune est généralisée ici.

### Ce qui est livré

- **Page commune des prestataires** `frontend/src/lib/features/provider/ProviderReviewPage.svelte`
  (générique en `P`, `B`, `R`), tirée de la page PayPal de la 38-11d : sélecteurs, import, compte rendu
  par devise, revue avec garde de génération, propositions par paquets de 500, écarts, « à examiner »
  avec la modale de rattachement et l'écriture manuelle ; ce qui est propre au prestataire passe par
  propriétés et fragments ; identifiants de test préfixés (`paypal-…`, `stripe-…`, inchangés pour
  PayPal). Types, client et règles communs : `provider-review.types.ts`, `provider-review.api.ts`,
  `provider-review.ts`, `provider-labels.ts` ; les modules PayPal en sont des alias ou des
  enveloppes. La page PayPal n'est plus que sa configuration.
- **Clés i18n** (recomptées sur le catalogue fr, `git diff c94d1ef7..HEAD`, 31 lignes retirées, 80
  ajoutées) : 30 clés `paypal-*` de la page devenues `provider-*` (même texte, quatre catalogues),
  `bank-accounts-help-paypal-primary` devenue `bank-accounts-help-provider-primary` (texte « PayPal ni
  Stripe »), `provider-col-name` copiée de `paypal-col-name` (toujours servie) ; 48 clés neuves aux
  quatre catalogues (`stripe-*`, `bank-accounts-*` Stripe, `nav-stripe`,
  `fiscal-years-checks-k2-link-stripe`). Les clés `stripe-review-*`,
  `stripe-gap-payout-*`, `provider-review-*`, `provider-failed-proposal-stale` posées par la
  40-7c et la 38-11c2 sont servies à l'écran.
- **AC-1** : cases « Compte PayPal » et « Compte Stripe » exclusives (`formProvider`), libellé,
  compte de frais, **compte bancaire de versement** (`payoutAccountCandidates`, miroir de
  `STRIPE_PAYOUT_ACCOUNT_INVALID`), charges `{ paypal }` / `{ stripe }` (`providerPayload`),
  décochée → `paypal: null` ou `stripe: null`, ligne « Stripe · <compte> · <devise> », pas de
  « Lier » ; `withoutProviders` (38-11d) non regénéralisé, testé aux cinq sites et aux lots de
  paiement avec Stripe et Wise. Lien « Ouvrir la page Stripe » aux contrôles de clôture.
- **AC-2 / AC-3** : `routes/(app)/reconciliation/stripe/+page.svelte`, `features/stripe/`
  (`stripe.types.ts`, `stripe.api.ts`, `stripe-review.ts`, `stripe-labels.ts`) ; menu *Mensuel* →
  *Stripe* (`comptableOnly`) ; rappel des ventes des boutiques ; refus de l'import par cause
  (colonne `source_id|charge_id` → marche à suivre, versement sans identifiant → rapport des
  versements automatiques) ; versements lus au compte rendu ; genres « Rattachement à la vente
  n° … », « Frais », « Versement vers <compte> » ; écarts « Charge sans vente », « Vente sans
  charge », « Montant ou devise différents », versement incomplet, versement sans réception (liens
  vers l'écriture du versement et vers la réconciliation) ; motifs ; test des codes d'écart et de
  motif contre les constantes `GAP_*` / `REVIEW_*` des sources Rust.
- **AC-4** : manuel utilisateur, section « Compte Stripe » (`sec:compte-stripe`) après « Comptes
  PayPal », renvois aux contrôles de clôture et à la saisie d'un relevé ; `.tex` seulement, PDF
  régénéré dans une copie (`kesh-gate-logs/s40-7d/user-manual.txt`, aplati : section présente,
  aucun renvoi `??`). Autres langues : à traduire. `docs/api-external.md` : écran et script des
  boutiques Stripe, exemple `curl` (identifiants à deux ou trois chiffres), objet `stripe`,
  `PROVIDER_PROPOSAL_STALE` étendu à `POST /stripe/accept` (les autres codes de 40-7b et 40-7c y
  étaient déjà).

### Écarts à la fiche, déclarés

1. **E2E, réception du versement** : la fiche veut une réception Wise EUR importée par la route Wise
   (39-11b), absente de la branche de l'Epic 40 ; un relevé en devise ne se saisit pas hors
   prestataire. La spec rattache donc la réception du **versement CHF** (`po_TEST0001`, 125.15),
   saisie par un relevé à la main sur une banque CHF, par `POST …/link-entry` ; même geste, même écart
   levé. Le versement EUR reste « à examiner » (compte EUR sans compte de versement).
2. **Lien vers la page Wise** d'un versement sans réception : non posé (pas de page Wise dans la
   branche). Revue P1 (M3) : le renvoi vers `/reconciliation` n'est plus posé pour un compte de destination
   Wise (que la réconciliation n'offre pas) : un texte dit d'importer le relevé du compte de destination ;
   le lien Wise viendra à l'intégration avec l'Epic 39 (note au fichier d'epic).
3. **Compte de versement vidé** quand la devise du compte du plan change (il n'est alors plus admis),
   non à tout changement du compte lié : un compte lié changé pour un autre de même devise garde un
   compte de versement encore valide.
4. Lignes « à examiner » : bouton *Rattacher à une écriture* (modale de la 38-11d) et lien
   *Écriture manuelle*, comme PayPal, plutôt qu'un lien vers `/bank-import/[id]` (la revue ne sert
   pas l'import d'une transaction).

### Tests (recomptés : `git diff c94d1ef7..HEAD -- frontend/src | grep -cE '^\+\s*it(\.each)?\('`)

47 sites `it` neufs (cinq `it.each` à deux cas) : `stripe-labels.test.ts` 16, `stripe-review.test.ts` 3,
`provider-stripe.test.ts` 6, `bank-accounts-page-stripe.test.ts` 8, `stripe-page.test.ts` 7, et 7 dans
des fichiers existants (Stripe et Wise aux sites de `withoutProviders`, lots de paiement, contrôles de
clôture). Gardes recomptées par elles-mêmes : `sitesTotal` 3632 → 3689, `i18n-libelle-en-dur` 138 → 145
(conformes 117 → 124). Tests Vitest de la 38-11d : assertions inchangées (seul `stripe: null` ajouté
aux fixtures de `BankAccountSummary`).

**Mutations** (copie hors dépôt, `node_modules` réel, copie non mutée verte d'abord ; « tuée » sur
une assertion d'un test nommé) : **10/10 tuées** — compte de versement Stripe/PayPal admis, versement
sans `payoutBankAccountId`, marche à suivre `source_id` retirée, décochée sans `stripe: null`,
compte de versement jamais vidé, « Tout cocher » sans `selectable` (page PayPal), liens de réception
sur tout écart, libellé du compte de versement ignoré, Stripe offert à l'import bancaire, lien Stripe
des contrôles de clôture retiré (`kesh-gate-logs/s40-7d/mutations.log`).

### Gates (dernier commit de code `7da520c4`)

- Fusion de la 40-7c dans `epic/40-stripe` (`e1dd8def`) : fmt, clippy workspace verts ; `cargo nextest`
  de `stripe_*`, `paypal_*`, `reconciliation_e2e`, `textes_coherents`, `configuration_transmise`,
  `audit_route_registry`, `test_schema_guard` : 333 passés, 1 ignoré ; `npm run check` vert, Vitest
  `i18n-keys` 11/11 (`fusion-*.log`).
- Backend : `scripts/test-fast.sh --ci` (fmt, clippy workspace `-D warnings`, nextest complet, base remise à zéro)
  à `2088d02d` : **6500 passés, 15 ignorés** (`gate-backend.log`) ; le seul commit de code suivant, `7da520c4`, n'ajoute
  qu'un commentaire d'exemption dans `reconciliation/stripe/+page.svelte` (aucun fichier Rust ni catalogue).
- Frontend à `7da520c4` : `npm run check`, `lint-i18n-ownership`, `test:unit` (**4998 passés, 292 fichiers**), `build`,
  tous verts (`gate-front.rc`, `front-*.log`). Le premier passage, à `2088d02d`, rendait 1 échec
  (`fx-account-filter-inventory.test.ts` : la page Stripe importe `fetchAccounts` sans marque ni exemption), corrigé
  par `7da520c4` (`front-test-unit-1.log`).
- **E2E partiel (1 spec)** : `frontend/tests/e2e/stripe-import.spec.ts`, la seule spec ajoutée par la
  story (règle de la nuit du 22.11) : 4/4 (`e2e-1.log`) ; E2E complet et `gate-ci.sh` à la branche
  d'intégration de fin.
- Captures (thème clair) : `kesh-gate-logs/s40-7d/captures/` 00 à 10 (aucun compte, formulaire,
  liste, refus `source_id`, compte rendu, propositions, écarts, à examiner, acceptation, versement
  sans réception, après réception).

### Remédiation de la revue P1 (2026-11-22)

- **M1** : le compte de versement ne se vide plus qu'au **changement de devise** du compte du plan
  (`$effect` de `bank-accounts/+page.svelte`, qui relit `payoutAccountCandidates` avec la destination
  déjà choisie) ; `payoutAccountCandidates` prend un `keepId` facultatif et garde cette destination dans
  la liste, archivée ou close, **marquée** (badge « Archivé » / « Clos le … », clés existantes), toujours
  soumise aux critères de devise, de type et de prestataire. Un renommage renvoie donc
  `payoutBankAccountId` inchangé. Tests : `bank-accounts-page-stripe.test.ts` (archivée, close : valeur
  gardée, marquée, renvoyée ; changement de devise : effacée même archivée), `provider-stripe.test.ts`.
- **M2** : `provider-labels.ts` traduit `FX_ACCOUNT_INVALID`, `FX_RATE_INVALID`, `NOT_FOUND`,
  `OPTIMISTIC_LOCK_CONFLICT`, `RECONCILIATION_ENTRY_NOT_LINKABLE` (clés `provider-failed-*`, quatre
  locales, textes repris de PayPal) ; PayPal délègue (les cinq clés `paypal-failed-*` correspondantes sont
  renommées, `PAYPAL_OWN_FAILED_CODES` ramenée à deux codes). Nouveau `provider-labels.test.ts` : la liste
  écrite en dur des 15 codes est comparée aux littéraux de `crates/kesh-api/src/routes/provider_accept.rs`
  (le fichier cité par la revue, `kesh-reconciliation/src/provider_accept.rs`, n'existe pas ; il est
  dans `kesh-api`), et chaque code a un libellé.
- **M3** : `payoutGoesToProvider` (page Stripe, comptes lus par la page commune via `bind:bankAccounts`) :
  pour un versement sans réception dont `details.payoutBankAccountId` est un compte Wise, le lien vers
  `/reconciliation` est remplacé par le texte `stripe-gap-receipt-import` (importer le relevé du compte de
  destination) ; banque ordinaire ou compte introuvable : lien inchangé. Manuel (`user-manual.tex`,
  « Versement sans réception ») corrigé ; le lien Wise final est noté au fichier d'epic. Test :
  `stripe-page.test.ts`.
- **Mutants survivants** : deux tests de `paypal-page.test.ts` (relecture : compte du retrait remis à zéro ;
  changement de compte : comptes rendus d'import et d'acceptation effacés). Le changement d'exercice n'est pas
  assertionné (la page de test n'offre qu'un exercice).
- **LOW** : L4 (parent `f988056c`, ci-dessus), L5 (règle de casse et d'accents au manuel, ligne Stripe), L6 (banques
  réelles des fixtures remplacées par « Banque Alpha » / « Banque Zeta » dans le test de Stripe ; les mêmes
  fixtures de `bank-accounts-page-paypal.test.ts` sont de la 38-11d, laissées), L9 (`api-external.md` : « ligne en
  devise » dit `foreign { amount, … }`, pour Stripe et PayPal). L1, L3, L7, L8 : entrées de
  `kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md`.
- Compteur : `sitesTotal` 3689 → 3692 (deux badges, un texte), relevé par la garde.
- **Mutations** (copie hors dépôt, base verte d'abord, `kesh-gate-logs/s40-7d/mutations-p1.log`) : 7/7 tuées sur une
  assertion nommée — garde `keepId` retirée, effet de vidage retiré, `FX_RATE_INVALID` retiré, `NOT_FOUND` retiré,
  lien Wise désactivé, `onReviewLoaded` retiré, compte rendu non effacé.
- **Gates** (dernier commit de code `da6d6427`, aucun fichier Rust touché, quatre catalogues `.ftl` modifiés) : `npm run check`
  (0 erreur), `lint-i18n-ownership`, Vitest **5008 passés, 293 fichiers**, `npm run build`, tous verts
  (`gate-front-p1.log`) ; E2E partiel de la seule spec de la story (`stripe-import.spec.ts`) : 4/4 (`e2e-2.log`). nextest non
  rejoué : aucun code Rust ne change (les locales sont lues par `kesh-i18n`, dont la parité est tenue par `i18n-keys.test.ts`).

### Remédiation de la revue P2 (2026-11-22)

Rapport `kesh-gate-logs/s40-7d/revue-p2-rapport.md` : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 4 LOW.

- **P2-1** : la destination archivée ou close n'est gardée que tant que le compte du plan est celui de
  l'ouverture (`payoutKeepId` de `bank-accounts/+page.svelte`, que l'effet suit aussi) ; à tout autre compte du
  plan, même de même devise, elle est retirée (le serveur la rejuge alors, `payout_is_unchanged`). Le manuel
  dit « un renommage ne l'efface pas » : toujours vrai. Test : destination archivée → vidée, active → gardée.
- **P2-2** : le texte `stripe-gap-receipt-import` (quatre locales) et le manuel ne disent plus que d'importer le
  relevé du compte de destination ; plus aucune promesse de page Wise. Le lien Wise reste noté à l'intégration
  avec l'Epic 39 (fichier d'epic).
- **P2-3** : la page Stripe relit les comptes bancaires **archivés et clos** (`listBankAccounts(true, true)`) pour
  résoudre la destination ; test : destination Wise archivée → texte, non lien. Si la lecture échoue, la liste est
  vide et le lien ordinaire s'affiche (limite acceptée).
- **P2-4** (note, non corrigé) : l'effet de la destination ne suit que la devise et le compte du plan, non l'arrivée
  tardive de `accounts` / `bankAccounts` ; une destination incompatible présente au chargement n'est vidée qu'au
  prochain changement : le serveur la refuserait explicitement, sans perte silencieuse.
- **P2-5** : la garde de `provider-labels.test.ts` lit aussi `closed_account_failed` et
  `counterparty_is_bank_ledger_failed_proposal` de `reconciliation.rs` (16 codes, `BANK_ACCOUNT_CLOSED` compris).
- **P2-6** : PayPal délègue aussi `PROVIDER_PROPOSAL_STALE` : un seul texte, `paypal-failed-proposal-stale`
  supprimée des quatre catalogues ; `PAYPAL_OWN_FAILED_CODES` supprimée.
- **P2-7** : `COUNTERPARTY_CURRENCY_MISMATCH` passe dans `providerFailedLabel` (clé `provider-failed-counterparty-currency-mismatch`,
  renommée depuis `paypal-…`, quatre locales).
- **Mutations** (`mutations-p2.log`, copie hors dépôt, base verte) : 4/4 tuées sur une assertion nommée — comparaison au
  compte du plan d'ouverture retirée, relecture des archivés et clos retirée, lecture de `reconciliation.rs` retirée,
  `COUNTERPARTY_CURRENCY_MISMATCH` retiré.
- **Gates** (dernier commit de code : celui de cette remédiation ; aucun Rust, quatre `.ftl` modifiés) : `npm run check` 0 erreur,
  `lint-i18n-ownership`, Vitest **5011 passés, 293 fichiers**, `npm run build` ; `cargo nextest -p kesh-i18n` 42/42 et
  `-p kesh-api --test textes_coherents` 30/30 (`gate-rust-p2.log`) ; E2E partiel de la spec de la story 4/4 (`e2e-3.log`).
  `sitesTotal` 3692 → 3691 (relevé par la garde).

### Remédiation de la revue P3 (2026-11-22)

Rapport `kesh-gate-logs/s40-7d/revue-p3-rapport.md` : 0 CRITICAL, 1 HIGH, 1 MEDIUM, 4 LOW. Commit de code `7bec552a`.

- **P3-1 (HIGH)** — prémisse vérifiée (sonde de la P3 rejouée : la page de `e50c11a9` vidait la destination archivée ou
  close à l'ouverture, en CHF comme en EUR, dans la vue par défaut). `bank-accounts/+page.svelte` :
  - `payoutPool` : `reload` lit aussi `listBankAccounts(true, true)` (la même lecture si l'écran montre déjà tout, `null`
    en échec ; lue **avant** la liste de l'écran, qui reste le dernier appel) ; `payoutSource` = cette liste, à défaut celle
    de l'écran ; les candidats de versement s'y résolvent : la destination archivée ou close s'affiche, marquée, au lieu de
    « Non configuré » ;
  - l'effet ne vide plus rien tant que le compte du plan est celui de l'ouverture (`ledgerUnchanged`), ni une destination
    absente de `payoutSource` ; il ne vide qu'une destination connue, après un changement du compte du plan, qui n'y est
    plus admise (autre devise, ou archivée ou close).
  - Tests (`bank-accounts-page-stripe.test.ts`) : mock `serve` qui respecte `includeArchived` / `includeClosed` (l'ancien
    rendait tout, d'où le défaut invisible) ; renommage d'un compte Stripe CHF et EUR, destination archivée ou close
    absente de la vue par défaut → gardée, montrée, renvoyée (4 cas) ; lecture des destinations en échec → non vidée, au
    renommage et au changement de compte du plan (2 cas). `bank-accounts-page.test.ts` (« un conflit de version recharge la
    liste ») : le décompte des lectures exclut désormais celle des destinations.
  - Sonde négative et mutations (`mutations-p3.log`, copie hors dépôt, vrai `node_modules`, témoin vert 19/19) : page de
    `e50c11a9` → 10 échecs ; garde « destination inconnue » retirée → 1 ; garde « compte du plan inchangé » retirée → 8 ;
    `payoutPool` retiré → 8 ; chacune sur une `AssertionError` nommée.
- **P3-2 (MEDIUM)** — `user-manual.tex` (§ *Compte Stripe*) et le commentaire de l'effet disent le comportement final : le
  choix ne se vide que si l'on change le compte du plan lié (autre devise, ou destination archivée ou close) ; sinon la
  destination archivée ou close reste choisie et affichée, même quand la liste masque ces comptes. **PDF non régénéré** :
  5 Go de mémoire disponible sur la station au moment de la remédiation.
- **P3-3** — commentaire de `reconciliation/stripe/+page.svelte` aligné sur P2-3 (seul un compte supprimé, ou une lecture
  en échec, donne le renvoi ordinaire).
- **P3-4** — la garde de `provider-labels.test.ts` relit la liste des fonctions importées de `reconciliation.rs` (sept, une
  huitième la fait rougir), leurs corps, celui de `fx_db_refusal` (`errors.rs`) et trois codes de `DbError::error_code()`
  (`ACCOUNT_NOT_POSTABLE`, `MULTI_CURRENCY_DISABLED`, `RATE_UNAVAILABLE`), cherchés dans `crates/kesh-db/src/errors.rs` :
  `CODES_MOTEUR` passe de 16 à 29 codes, tous traduits (aucun « Refus non reconnu »).
- **P3-5** — titre de l'`it.each` « même devise » : colonnes dans l'ordre du titre.
- **P3-6** — Notes de version : le texte PayPal du refus d'une proposition périmée change.
- Grep du symptôme (`change de devise`, `vidé quand`, `se vide quand`, `payoutBankAccountId`) : code, quatre catalogues
  (texte d'aide inchangé, juste), manuels, `docs/api-external.md` (§ Stripe juste : le serveur ne rejuge pas une
  destination inchangée), fiche (l'AC-1 historique n'est pas réécrite).
- **Gates** (au commit `7bec552a`, aucun Rust ni `.ftl` touché) : `npm run check` 0 erreur, `lint-i18n-ownership`, Vitest
  **5017 passés, 293 fichiers** (garde i18n comprise, `sitesTotal` 3691 inchangé), `npm run build` (`gate-front-p3.rc`) ;
  E2E partiel de la seule spec de la story (`stripe-import.spec.ts`) : 4/4 (`e2e-p3.log`, base `kesh_e2e_s407d` sur dev2).
- Non fait : captures d'écran du sélecteur montrant une destination archivée (la spec E2E de la story n'archive pas de
  destination ; état couvert par Vitest seulement).

### File List

`frontend/src/lib/features/provider/` (neuf : `ProviderReviewPage.svelte`, `provider-review.types.ts`,
`provider-review.api.ts`, `provider-review.ts`, `provider-labels.ts`), `frontend/src/lib/features/stripe/`
(neuf : `stripe.types.ts`, `stripe.api.ts`, `stripe-review.ts`, `stripe-labels.ts` et leurs tests),
`frontend/src/routes/(app)/reconciliation/stripe/` (neuf), `frontend/src/routes/(app)/reconciliation/paypal/+page.svelte`,
`frontend/src/routes/(app)/bank-accounts/+page.svelte`, `frontend/src/routes/(app)/+layout.svelte`,
`frontend/src/lib/features/bank-accounts/{bank-accounts.api.ts,provider.ts}`,
`frontend/src/lib/features/paypal/{paypal.types.ts,paypal.api.ts,paypal-review.ts,paypal-labels.ts}`,
`frontend/src/lib/features/fiscal-years/FiscalYearChecks.svelte`, tests (fixtures `stripe: null`,
cas Stripe et Wise), `frontend/tests/e2e/stripe-import.spec.ts`, quatre catalogues
`crates/kesh-i18n/locales/*/messages.ftl`, `docs/manual/fr/user-manual.tex`, `docs/api-external.md`.

## Notes de version

*Ajouté* — Écrans du compte Stripe : case « Compte Stripe » et compte bancaire de versement dans les
comptes bancaires, page *Mensuel* → *Stripe* (import du rapport, propositions à accepter en lot,
écarts, lignes à examiner) ; section « Compte Stripe » du manuel utilisateur et de
`docs/api-external.md`. *Modifié* — la page PayPal repose sur la page commune des prestataires ;
seul change le texte du refus d'une proposition périmée, désormais commun aux prestataires
(« Cette proposition n'est plus valable… Rechargez la revue. »).

