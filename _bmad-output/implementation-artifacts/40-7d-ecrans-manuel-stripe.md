# Story 40-7d : écrans, manuel et E2E du compte Stripe (phase 1)

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, chemins relevés sur `origin/main` au commit `fd04099f`. Epic 40,
**phase 1 KISS** (décision de Beat du 2026-11-20). **Dépend de** : 40-7b, 40-7c ; les écrans PayPal de la phase 1 de
l'Epic 38 (38-11d : page, feature, `withoutPaypal`, modale de rattachement, manuel, `api-external.md`, E2E), dont elle
reprend les composants sans les recopier ; la phase 1 de l'Epic 39 pour la seule E2E (réception côté Wise). Dernière story de la phase 1.*

## Story

En tant que **Comptable**, je veux régler mon compte Stripe, importer son rapport d'un bouton, voir ce qui a été
rattaché et écrit, et parcourir la liste des écarts avec un lien vers chaque transaction ou écriture, et je veux que le
manuel me dise comment exporter le bon rapport de Stripe et ce que Kesh en fait.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-40-7d-1` | Le formulaire d'un compte bancaire offre les réglages Stripe (compte Stripe, compte de frais, compte de versement), à côté de ceux de PayPal, par les mêmes composants. | AC-1 |
| `FR-40-7d-2` | Sur la page Stripe, un bouton *Importer un rapport Stripe* envoie le fichier, puis lance le rapprochement (deux routes, l'une après l'autre), et montre le compte rendu. | AC-2 |
| `FR-40-7d-3` | Une liste des écarts, par compte Stripe et période, avec lien vers la transaction ou l'écriture, et le geste existant de rattachement pour une réception proposée. | AC-3 |
| `FR-40-7d-4` | Manuel utilisateur et `docs/api-external.md` : exporter le rapport (colonne `source_id`), régler, importer, rapprocher, lire les écarts, limites. | AC-4 |
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
   `isPaypal` / `withoutPaypal` de 38-11d (`frontend/src/lib/features/bank-accounts/paypal.ts`, patron de
   `credit-card.ts:39`) deviennent **`isProvider` / `withoutProviders`** (`provider !== null`) et s'appliquent aux mêmes
   quatre sélecteurs (`BankImportUpload.svelte`, `ManualStatementForm.svelte`, `SettleInvoiceDialog.svelte`,
   `reconciliation/+page.svelte`) — sans quoi un compte Stripe s'offre à l'import d'un relevé bancaire ou au règlement
   d'une facture, que le serveur refuse (`BANK_ACCOUNT_IS_PROVIDER`) ; Vitest : chaque site, avec un compte Stripe, PayPal
   et Wise. `BankAccountSummary` porte le prestataire et `payoutBankAccountId` sous la forme de la réponse de 40-7b.
2. **AC-2 — Page Stripe et import.** Page `frontend/src/routes/(app)/reconciliation/stripe/+page.svelte` (feature neuve
   `frontend/src/lib/features/stripe/`, **à côté de la page PayPal** de 38-11d, `reconciliation/paypal/`), entrée
   « Stripe » du menu `Mensuel` **dans `comptableOnly`** (comme PayPal : le menu ne charge pas les comptes ; la page dit
   « aucun compte Stripe » avec le lien vers *Comptes bancaires* quand il n'y en a pas). Sélecteur du compte Stripe
   (libellés distincts des comptes bancaires `provider = 'stripe'`), bouton **« Importer un rapport Stripe »** : choix du
   fichier (`accept=".csv,text/csv"`), envoi par `apiClient.postFormData` (patron de `createBankImport`,
   `bank-import.api.ts:70-79`) à `POST /api/v1/stripe-imports` (`file`, `account`), **puis**, s'il a réussi, `POST
   /api/v1/stripe-matching` (`account`) : deux appels, aucun paramètre `match`. Compte rendu affiché jusqu'au prochain
   import : par devise, insérées et doublons ; versements lus ; puis ventes rattachées, frais et versements écrits,
   réceptions proposées, nombre d'écarts. Refus traduits : `STRIPE_CSV_INVALID` rend, par ses codes de 40-7a, une phrase
   par cause — colonnes manquantes **nommées**, et pour `source_id|charge_id` le message qui dit où ajouter la colonne dans
   Stripe ; `automatic_payout_id` vide : prendre le rapport des versements automatiques ; `PROVIDER_CURRENCY_ACCOUNT_MISSING`
   nomme les devises. Si l'import réussit et que le rapprochement échoue (réseau, 5xx), le compte rendu de l'import reste
   affiché et le bouton *Relancer le rapprochement* (AC-3) le reprend : le geste est rejouable. Rappel affiché : « les
   ventes viennent des boutiques ; importez-les avant le rapport Stripe ».
3. **AC-3 — Écarts.** Sur la même page : période, bouton *Relancer le rapprochement* (`POST /api/v1/stripe-matching`),
   tableau `GET /api/v1/stripe-matching?account=&from=&to=` : date, genre d'écart (libellé traduit **pour chaque code**
   de 40-7c : `CHARGE_WITHOUT_SALE`, `SALE_WITHOUT_CHARGE`, `AMOUNT_MISMATCH`, `CURRENCY_MISMATCH`, `AMBIGUOUS_SALE`,
   `PAYOUT_TOTAL_MISMATCH`, `PAYOUT_WITHOUT_RECEIPT`, `PAYOUT_ACCOUNT_MISSING`, `RECEIPT_TO_CONFIRM`, `FEE_ACCOUNT_MISSING`,
   `RATE_UNAVAILABLE`, `TO_REVIEW`, et les codes d'exercice clos ou de période verrouillée), montant et devise,
   identifiant, lien vers la transaction (`/bank-import/[id]`) ou l'écriture ; pour `RECEIPT_TO_CONFIRM`, **la modale
   de rattachement à une écriture de 38-11d** (`LinkSettlementModal.svelte` en mode écriture, qui prend une transaction
   seule et affiche les montants dans la devise de la transaction : aucun second composant) ouverte sur la transaction
   proposée. Libellés d'écart : `stripe-labels.ts`, clés `stripe-…` en toutes lettres ; `failed-proposal-label.ts` **n'est
   pas étendu** (son test compare ses codes aux littéraux de `reconciliation.rs`). État vide : « Aucun écart ». Un test
   écrit en dur les codes d'écart contre `routes/stripe_matching.rs` (patron du test de `failed-proposal-label`).
4. **AC-4 — Manuel et `docs/api-external.md`.** *Manuel* — `docs/manual/fr/user-manual.tex`, section « Compte Stripe »
   après « Comptes PayPal » (38-11d), `.tex` seulement (PDF à la release) : exporter le rapport *Rapprochement des
   versements → Détaillé* **avec la colonne `source_id`**, en points décimaux, et le fuseau voulu (la date des lignes est
   celle du fuseau d'export) ; **ne pas filtrer le rapport par catégorie** (le versement serait faux, et sa réception
   introuvable) ; le schéma comptable (vente de la boutique au compte de transit avec l'identifiant au libellé ; frais ;
   versement) ; ce que fait le rapprochement ; chaque écart et quoi faire ; la réception sur Wise ou à la banque à
   confirmer, jamais comptée deux fois ; limites (remboursements, retours de versement et versements qui en portent à
   passer à la main ; rattachement en devise seul, les autres gestes de la réconciliation restant en CHF ; aucun code TVA ;
   aucune conversion entre devises, un compte de versement de même devise ; monnaies à trois décimales non servies ;
   petit écart de change possible au compte de transit, traité par la réévaluation de fin d'exercice). Gardes de
   `textes_coherents.rs` **à connaître avant d'écrire** (relevées à 38-11d) : G2 interdit le mot capitalisé « Réglages »
   (écrire « paramètres Stripe ») ; G4-bis/G4-ter n'admettent comme numéro de compte cité que ceux d'un plan livré (dire
   « le compte de transit », « le compte de frais », jamais « 1120 » ni « 4170 ») ; G25 interdit « projet » pour le compte
   analytique. Manuel d'administration : rien de plus que la phrase du registre de 40-7b/40-7c. Autres langues : « à
   traduire » noté. Axe de revue : le manuel contre le code et le PDF régénéré localement (`pdftotext … | tr '\n' ' ' | tr
   -s ' '`), sans commit du PDF. *`docs/api-external.md`* — section neuve à côté de celle de PayPal (38-11d AC-7) :
   `POST /api/v1/stripe-imports`, `POST` et `GET /api/v1/stripe-matching`, objet des paramètres du prestataire avec
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
   seconde fixture), **par l'interface** ; compte rendu attendu ; liste des écarts (au moins `CHARGE_WITHOUT_SALE`,
   `SALE_WITHOUT_CHARGE`, `RECEIPT_TO_CONFIRM`), confirmation de la réception par la modale, écart disparu ; refus du
   fichier sans `source_id` avec sa marche à suivre. Captures (thème clair) de chaque état touché dans
   `KESH_CAPTURES_DIR` (mécanisme de `fx-entries.spec.ts`), hors dépôt (`kesh-gate-logs/`), jointes au Dev Agent Record et
   au prompt de revue. Vitest des composants neufs ; `npm run check`, `lint-i18n-ownership` (préfixes `stripe-` pour
   `features/stripe/`, `bank-accounts-` et `reconciliation-` ailleurs, `nav-*` pour le menu), `sitesTotal` de
   `frontend/src/lib/shared/i18n-keys.test.ts` **recompté** depuis la source après la fusion. Suite E2E complète au dernier
   commit de code (7 à 9 échecs attendus, jugés fichier par fichier, `docs/testing.md`).

## Tâches

- [ ] T1 — Bloc de paramètres prestataire généralisé, champs Stripe, `withoutProviders` aux quatre sélecteurs (AC-1).
- [ ] T2 — Page Stripe : import puis rapprochement, compte rendu (AC-2).
- [ ] T3 — Écarts et modale de rattachement (AC-3).
- [ ] T4 — Manuel et `api-external.md` (AC-4).
- [ ] T5 — E2E, Vitest, captures (AC-5).

## Stratégie de tests

Vitest (formulaire, compte rendu, libellés d'écarts, état vide) ; E2E complet au dernier commit de code ; tests
`textes_coherents.rs` du manuel s'il y a lieu.

## Modules touchés (3)

`frontend` (bank-accounts, reconciliation/stripe, E2E), `kesh-i18n` (s'il manque un message serveur), `docs/manual/fr`
et `docs/api-external.md`.

## Note de version

*Ajouté* — Écrans du compte Stripe : paramètres, import du rapport suivi du rapprochement, liste des écarts ; section « Compte
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
