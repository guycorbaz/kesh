# Story 40-7d : écrans, manuel et E2E du compte Stripe (phase 1)

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, chemins relevés sur `origin/main` au commit `fd04099f`. Epic 40,
**phase 1 KISS** (décision de Beat du 2026-11-20). **Dépend de** : 40-7b, 40-7c ; les écrans PayPal de la phase 1 de
l'Epic 38 (38-11d : page, feature, `withoutPaypal`, modale de rattachement, manuel, `api-external.md`, E2E), dont elle
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
   `isPaypal` / `withoutPaypal` de 38-11d (`frontend/src/lib/features/bank-accounts/paypal.ts`, patron de
   `credit-card.ts:39`) deviennent **`isProvider` / `withoutProviders`** (`provider !== null`) et s'appliquent aux mêmes
   quatre sélecteurs (`BankImportUpload.svelte`, `ManualStatementForm.svelte`, `SettleInvoiceDialog.svelte`,
   `reconciliation/+page.svelte`) — sans quoi un compte Stripe s'offre à l'import d'un relevé bancaire ou au règlement
   d'une facture, que le serveur refuse (`BANK_ACCOUNT_IS_PROVIDER`) ; Vitest : chaque site, avec un compte Stripe, PayPal
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
3. **AC-3 — Propositions, écarts, à examiner.** Genres de proposition : « Rattachement à la vente n° … » (`link_sale`),
   « Frais » (`fee`), « Versement vers <compte de versement> » (`payout`), à cocher, « Tout cocher », **« Accepter la
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
   passer à la main ; rattachement en devise seul, les autres gestes de la réconciliation restant en CHF ; aucun code TVA ;
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

- [ ] T1 — Bloc de paramètres prestataire généralisé, champs Stripe, `withoutProviders` aux quatre sélecteurs (AC-1).
- [ ] T2 — Page Stripe sur le composant généralisé : import, compte rendu, revue (AC-2).
- [ ] T3 — Propositions, acceptation, écarts, à examiner, libellés (AC-3).
- [ ] T4 — Manuel et `api-external.md` (AC-4).
- [ ] T5 — E2E, Vitest, captures (AC-5).

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

