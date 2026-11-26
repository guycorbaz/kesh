# Story 39-11d : écrans, manuel et E2E de l'import Wise

Status: done

*Fiche rédigée le 2026-11-20, branche `epic/39-wise`, revérifiée à la validation P1. Epic 39, **phase 1 KISS**.
**Dépend de** : 39-11a à 39-11c, **38-11d** (écrans PayPal, validée P1 : case « Compte PayPal » du formulaire des
comptes bancaires, `paypal.ts` et `withoutPaypal`, filtres de règle, page `reconciliation/paypal`, libellés des refus,
E2E `paypal-import.spec.ts`), dont les composants sont **généralisés**, non recopiés (G10).*

## Amendement du 2026-11-22 — contre le code réel (étape 0 du développement)

Relu au code de la branche `story/39-11d`, partie d'`epic/39-wise` (`1f03f180`, 39-11e comprise) avec `epic/40-stripe`
fusionné (`5064da7c`, 40-7d comprise ; décision de l'orchestrateur du 2026-11-22 : la 39-11d **s'ajoute à la page commune**
`frontend/src/lib/features/provider/ProviderReviewPage.svelte` de la 40-7d, sans la regénéraliser ; ordre des PR : 38, 40, 39).
Ce qui change, et pourquoi :

- **A1 — formulaire (AC-1)** : la 40-7d a remplacé la case « Compte PayPal » par **une case par prestataire, chacune exclut
  les autres** (`setFormProvider`, `providerCheckbox`), non par un choix « Aucun / PayPal / Wise ». La 39-11d y ajoute la case
  « Compte Wise » (même forme que PayPal : libellé, compte de frais, sans compte de versement) ; `provider.ts` gagne `wise` dans
  `ConfigurableProvider`, `isWise`, `wiseAccountLabels`, `providerPayload` (`{ wise }`) et la ligne « Wise · <compte> · <devise> ».
  Côté serveur, la fusion réunit `wise` (39-11b) et `stripe` (40-7b) dans `routes/bank_accounts.rs` et l'entité
  (`BankAccountProvider`, `ProviderSettings::wise`) : deux prestataires posés ensemble → `400 VALIDATION_ERROR`
  (`error-provider-settings-both`, texte généralisé).
- **A2 — page Wise (AC-2)** : le compte rendu de Wise est **par fichier** (`files[]`, `archive`, soldes, `warnings` `gap`,
  `ignoredEntries`), non par devise : la page commune gagne `fileAccept`, `multiple`, `api.importFiles` et le fragment
  `importReportMain` ; `currencies` devient facultatif dans `ProviderImportResponseBase`. Le bouton accepte
  `.csv,text/csv,.zip,application/zip`, plusieurs fichiers ; la **consigne d'export** est à l'écran (option « Afficher les
  transactions dont les frais sont indiqués séparément » à cocher) ; refus traduits : `WISE_CSV_FEES_NOT_SEPARATED` (fichier,
  zip, consigne, lignes), `WISE_CSV_INVALID` par raison, `WISE_DUPLICATE_CURRENCY_FILE` ; `WISE_ZIP_INVALID` et les refus
  communs par le message du serveur.
- **A3 — revue et acceptation (AC-3) reportées** : la revue Wise (`GET /api/v1/wise/review`, `POST /api/v1/wise/accept`) est
  la Story 39-11c, **en revue sur sa propre branche** et absente d'`epic/39-wise`. Sans elle, la page Wise n'a que l'import
  (la page commune, sans `api.getReview`, n'affiche ni exercice ni propositions). AC-3 passe à une story **39-11d2** (écran de
  la revue Wise sur la page commune, libellés des genres, écarts et motifs, E2E de l'acceptation), à faire après la fusion de
  la 39-11c dans `epic/39-wise`. Le filtre de type générique (AC-1) et le lien de la page Stripe vers la page Wise (versement
  sans réception) restent ici.
- **A4 — manuel et API (AC-4)** : sous-section « Comptes Wise » (`sec:comptes-wise`) avant `sec:wise-export` de la 39-11e,
  qui la cite ; le rapprochement y est dit « pas encore dans cette version ». `docs/api-external.md` : paragraphe « L'écran
  Wise » ; table des erreurs réunie pour PayPal, Wise et Stripe à la fusion.
- **A5 — E2E (AC-5)** : `wise-import.spec.ts` sans acceptation (A3) : compte Wise par l'écran, refus d'un relevé exporté sans
  les frais séparés, import du zip (fait par la spec à partir des trois fixtures, plus une entrée PDF ignorée), réimport refusé.
  Règle des gates de la nuit du 22.11 : E2E de la seule spec ajoutée, pas d'E2E complet ni de `gate-ci.sh`.

## Story

En tant que **Comptable**, je veux régler mes comptes Wise, importer mes relevés et traiter les propositions et les
écarts depuis l'écran, et trouver au manuel comment exporter le relevé de Wise et ce que Kesh en fait.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-11d-1` | Formulaire d'un compte bancaire : prestataire « Wise » à côté de « PayPal » (un seul choix), libellé, compte de frais. | AC-1 |
| `FR-39-11d-2` | Page Wise : import d'un ou plusieurs relevés (un par devise), compte rendu par fichier (insérées, doublons, soldes, trou). | AC-2 |
| `FR-39-11d-3` | Propositions Wise (frais, conversion, rattachement à l'autre côté, règle), écarts et « à examiner » lisibles, quatre locales. | AC-3 |
| `FR-39-11d-4` | Filtre de règle « type » devenu générique : aide qui liste les types PayPal et les préfixes Wise. | AC-1 |
| `NFR-39-11d-1` | Manuel utilisateur et `docs/api-external.md` à jour (`.tex` seulement, PDF à la release) ; *Note de version*. | AC-4 |
| `NFR-39-11d-2` | E2E sur les fixtures fictives ; captures d'écran de chaque état touché (*Recette visuelle*). | AC-5 |

## Critères d'acceptation

1. **AC-1 — Comptes bancaires et règles.** Le formulaire de 38-11d AC-1 (`frontend/src/routes/(app)/bank-accounts/
   +page.svelte`, `submitCreate`, `submitUpdateFull`) remplace la case « Compte PayPal » par un **choix de
   prestataire** (« Aucun », « PayPal », « Wise »), mêmes champs et mêmes règles d'affichage (IBAN facultatif,
   QR-IBAN et « principal » masqués, compte du plan en devise offert, compte de frais filtré comme au serveur) ; charge
   `paypal: {…}` ou `wise: {…}` (39-11b AC-2), `null` pour retirer. `frontend/src/lib/features/bank-accounts/provider.ts` est **déjà livré par 38-11d AC-1** (`isPaypal`, `isProvider`,
   `withoutProviders`, génériques : `provider !== null`), appliqué à **cinq sites** (`BankImportUpload.svelte`,
   `ManualStatementForm.svelte`, `SettleInvoiceDialog.svelte`, page `reconciliation/+page.svelte`, paiement d'une facture
   fournisseur `supplier-invoices/[id]/+page.svelte`) : **réutilisé, non regénéralisé** — cette story n'ajoute qu'un `isWise(a)` si l'écran en a besoin ; `BankAccountSummary` (qui porte déjà `provider`, 38-11d) gagne `wise` ; la liste dit « Wise · <libellé> · <devise> ».
   Le filtre « Type PayPal (exact) » de `RuleFormModal.svelte` devient « Type (prestataire) », aide : types PayPal
   (`Website Payment`…) et préfixes Wise (`TRANSFER`, `CARD`, `BALANCE`, `CONVERSION_ORDER`, `FEE-TRANSFER`…). Refus du
   serveur affichés par leur code (`errorMessageOf`). Vitest : choix du prestataire (charges, champs masqués), les
   cinq sites (déjà couverts par 38-11d, un compte Wise y est ajouté aux fixtures), mappage `wise`.
2. **AC-2 — Page Wise** `frontend/src/routes/(app)/reconciliation/wise/+page.svelte`, sur le **composant de la page
   PayPal généralisé** (38-11d AC-3 à AC-5 : sélecteurs compte et exercice, bouton d'import, revue rechargée après import
   et acceptation, garde de génération, propositions par paquets de `MAX_IN_CLAUSE_IDS`, écarts, « à examiner »,
   modale « Rattacher à une écriture » en devise) paramétré par le prestataire (routes, libellés, forme du compte rendu),
   entrée « Wise » du menu à côté de « PayPal », dans `comptableOnly`. Bouton **« Importer des relevés Wise »** :
   sélection multiple (`accept=".csv,text/csv"`, `multiple`), envoi à `POST /api/v1/wise-imports` ; compte rendu par
   fichier ; *(amendé le 2026-11-22, story 39-11e : `accept=".csv,text/csv,.zip,application/zip"`, le zip de Wise
   envoyé tel quel ; compte rendu avec `archive` et `ignoredEntries` ; refus `WISE_ZIP_INVALID` et
   `WISE_CSV_FEES_NOT_SEPARATED` traduits par leur code)* ; refus traduits par les codes fermés de 39-11a (colonnes manquantes nommées, ligne et champ, rupture du
   solde courant avec la ligne) et de 39-11b (devise sans compte, fichier de même devise) ; avertissement `gap`
   expliqué (« le relevé ne commence pas où finissait le précédent »).
3. **AC-3 — Propositions, écarts, à examiner.** Genres : « Frais », « Conversion au cours de Wise : 100.00 EUR → 95.00
   CHF » (exemple fictif ; « deux écritures par le compte de conversion » entre deux devises étrangères), « Rattachement
   à l'écriture n° … » (`link_entry`), « Règle <libellé> » ; écarts (`conversion_half_missing`, `fee_without_origin`,
   `balance_gap`) ; motifs « à examiner » (`fee_account_missing`, `ambiguous_link`, `stripe_payout_unmatched`,
   `no_rule`). Textes « proposer », jamais « comptabiliser automatiquement ». Libellés des refus d'acceptation par le
   module de 38-11d (`paypal-labels.ts` généralisé, délégation à `failedProposalLabel`), test des codes de la route Wise
   contre leur source. Clés aux quatre locales, préfixe `wise-` pour ce qui est propre à Wise, `provider-` pour le
   commun (`docs/i18n-key-ownership-pattern.md`), chaque clé en toutes lettres ; `sitesTotal` de
   `frontend/src/lib/shared/i18n-keys.test.ts` **recompté** depuis la source.
4. **AC-4 — Manuel et API** (`docs/manual/fr/user-manual.tex`, `.tex` seul ; DE, IT, EN « à traduire » selon la règle
   du dépôt) : sous-section « Comptes Wise » à côté de « Comptes PayPal » (38-11d AC-6) — un compte bancaire par devise,
   compte de frais, compte de conversion requis entre deux devises étrangères ; exporter de Wise le relevé CSV de chaque
   solde (période, une devise par fichier ; le zip se donne tel quel) *(amendé le 2026-11-22 : l'export est décrit par
   la sous-section `sec:wise-export` de la story 39-11e — CSV, option des frais séparés cochée, zip tel quel —, à
   laquelle cette sous-section renvoie)* ; l'importer ; ce que Kesh propose ligne par ligne ;
   les conversions au cours de Wise ; le rattachement à l'autre côté (virement depuis votre banque, versement Stripe de
   l'Epic 40 : jamais compté deux fois, jamais affecté par une règle) ; un virement vers sa banque par une règle quand
   l'autre côté n'est pas encore passé ; les écarts ; les limites (date de l'export sans conversion de fuseau, aucun
   code TVA, pas de téléchargement automatique ; **celles de 38-11b AC-3 reprises pour Wise** : une facture fournisseur ne se paie
   pas depuis un compte Wise — régler par un compte interne ou une écriture à la main, puis rattacher la transaction —, et la
   réconciliation générale ne refuse pas un compte Wise **en francs** : ses lignes se traitent par la seule page Wise, sans quoi une
   conversion ou des frais pourraient s'écrire deux fois). Gardes de `textes_coherents.rs` à connaître avant d'écrire (38-11d
   AC-6 : G2 « Réglages », G4-bis/G4-ter numéros de compte — ni « 1020 », « 4830 » ni « 9900 » —, G25, G18). La phrase
   du compte des routes du manuel d'administration est tenue par 39-11b et 39-11c. `docs/api-external.md` (section de
   38-11d AC-7) : `POST /api/v1/wise-imports`, `GET /api/v1/wise/review`, `POST /api/v1/wise/accept`, champ `wise`
   des comptes bancaires, codes `WISE_*` à la table des erreurs. Contrôle local du PDF permis (aplati, `pdftotext … |
   tr '\n' ' ' | tr -s ' '`), PDF non commité.
5. **AC-5 — E2E** `frontend/tests/e2e/wise-import.spec.ts`, patron de `paypal-import.spec.ts` (38-11d AC-8 : société
   `with-company`, exercice 2020, multi-devises activé par l'API, reseed en fin de parcours) : comptes du plan EUR et
   USD, trois comptes bancaires Wise d'un libellé, compte de frais, compte de conversion, cours mensuels de 2020 ;
   import par l'interface des trois fixtures de 39-11a, lues par chemin (`crates/kesh-import/tests/fixtures/`, ni
   copie ni seconde fixture) ; compte rendu ; acceptation d'un frais et d'une conversion CHF↔EUR (une écriture) ; ligne de versement Stripe « à examiner » (`stripe_payout_unmatched`) affichée ; réimport → refus doublon. Captures (thème clair) de chaque état touché (formulaire du compte Wise,
   compte rendu d'import, propositions, écarts, à examiner, acceptation), dans `KESH_CAPTURES_DIR`, hors dépôt
   (`kesh-gate-logs/`), jointes au Dev Agent Record et au prompt de revue. Suite E2E complète au dernier commit de code.

## Tâches

- [x] T1 — Choix du prestataire, `provider.ts`, filtre de type générique (AC-1).
- [x] T2 — Composant de la page PayPal généralisé, page Wise, menu (AC-2, AC-3).
- [x] T3 — Manuel utilisateur, `docs/api-external.md` (AC-4).
- [x] T4 — E2E, captures (AC-5).

## Dev Agent Record

*Développée le 2026-11-22, branche `story/39-11d` (partie d'`epic/39-wise`, `epic/40-stripe` fusionné en `3a721279`, page commune des prestataires). Dernier commit de code `d1ded344` ; `5bfe2478` ne touche que deux tests Vitest (gardes i18n recomptées).*

**Livré** (amendement A1 à A5) : case « Compte Wise » du formulaire des comptes bancaires (exclusive des autres, compte de frais, aucun compte de versement) et ligne « Wise » de la liste ; page `reconciliation/wise` sur la page commune (`ProviderReviewPage` : `fileAccept`, `multiple`, `importFiles`, fragment `importReportMain`, revue facultative), consigne d'export sous le bouton, refus traduits (`WISE_CSV_FEES_NOT_SEPARATED`, `WISE_CSV_INVALID` par raison, `WISE_DUPLICATE_CURRENCY_FILE`, `WISE_ZIP_INVALID` par le message du serveur), compte rendu par fichier ; filtre de règle « Type chez le prestataire » générique ; menu et liens (Stripe vers Wise, contrôles de clôture) ; 48 clés i18n aux quatre locales *(annoté le 2026-11-22, revue P1, L6 : **49** au diff `e194fb84..440ae42c`, `reconciliation-rules-help-provider-type` comprise, 3 clés modifiées ; 50 après la remédiation P1)* ; manuel (`sec:comptes-wise`, `.tex` seul) et `docs/api-external.md` (« L'écran Wise »). **Reporté (A3, validé)** : revue et acceptation Wise, à la 39-11d2 (la 39-11c n'est pas dans `epic/39-wise`).

**Gates** (règle de la nuit du 22.11 : E2E de la seule spec ajoutée, pas de `gate-ci.sh`, déclaré tel quel) :
- backend : `nextest` complet au commit `d1ded344` (journal `kesh-gate-logs/s39-11d/nextest-1.log`) : 6614 exécutés, 6614 passés, 17 ignorés, rc 0. `5bfe2478` ne touche aucun fichier Rust.
- frontend au commit `5bfe2478` (`s39-11d/front-2.rc`) : `npm run check` rc 0, `lint-i18n-ownership` rc 0, Vitest 297 fichiers, 5063 tests passés, `npm run build` rc 0. Tests neufs de la story : 29 `it`/`test` Vitest ajoutés au diff `e194fb84..HEAD` (+ fichiers neufs `provider-wise`, `bank-accounts-page-wise`, `wise-labels`, `wise-page`).
- E2E partiel, 1 spec : `wise-import.spec.ts` (3 tests, nouvelle spec de la story ; aucune autre spec jouée), au commit `5bfe2478` (`e2e.head`), 3 passés (`s39-11d/e2e.log`).
- Non joués : `cargo fmt --check` et clippy workspace n'ont pas été relancés par cette reprise (ceux de l'agent précédent, `lint.sh`/`lint-fusion.log`, sont au journal) ; `gate-ci.sh` et E2E complet à l'intégration finale.

**Captures** (thème clair) : `kesh-gate-logs/s39-11d/captures/` (hors dépôt) — 00 aucun compte, 01 formulaire, 02 liste, 03 page Wise avec la consigne de cocher « Afficher les transactions dont les frais sont indiqués séparément » (vérifié à l'image), 04 refus des frais non séparés (consigne et lignes 6, 7), 05 compte rendu du zip par fichier (CHF, EUR, USD ; entrée PDF non lue), 06 page après import, 07 zip déjà importé.

## Modules touchés (4)

`frontend`, `kesh-i18n` (libellés), `docs/manual` (`.tex`) et `docs/api-external.md`, E2E.

## Note de version

*Ajouté* — Écrans de l'import Wise : réglage des comptes Wise (case « Compte Wise » des comptes bancaires), import des
relevés par la page *Mensuel* → *Wise* (le zip de Wise tel quel ou plusieurs CSV, un par devise), compte rendu par
fichier et refus qui nomment le fichier en cause ; section Wise au manuel utilisateur. La revue et l'acceptation des
transactions Wise viennent avec la 39-11d2 : d'ici là, les transactions importées restent en attente.

*Modifié (API, additif)* — les `details` du refus `422 WISE_CSV_INVALID` portent une clé `archive` : le nom du zip
d'où vient le relevé refusé, `null` pour un fichier envoyé à part (comme `WISE_CSV_FEES_NOT_SEPARATED`). Aucune clé
retirée ni renommée ; `docs/api-external.md` le dit.

## Limites du rapprochement à reprendre au manuel (revue de code P3 de la 39-11c)

*Annoté à l'intégration de la 39-11d (2026-11-22) : à reprendre par la 39-11d2 (revue Wise à l'écran et son manuel).*

À écrire dans la sous-section « Comptes Wise » (AC-4), en termes d'utilisateur, à côté des motifs « à examiner » du
paiement par carte converti (`docs/api-external.md`, section du rapprochement Wise, fait foi) :

- **Continuité des relevés** : Kesh tient pour complet un intervalle entre deux relevés d'un solde quand l'ouverture du
  second égale la clôture du premier. Des lignes non importées qui s'annulent (une recharge et une dépense du même
  montant, ou un solde resté à zéro des deux côtés) n'y paraissent pas : le paiement se propose sans elles. Conseil :
  exporter chaque solde sur des périodes jointives, sans trou.
- **Solde en devise peu utilisé** : si ses relevés encadrent la date d'un paiement sans se suivre (exports épars), **tout**
  paiement par carte daté entre eux passe « à examiner », même sans rapport avec ce solde. Remède : importer le relevé qui
  comble l'intervalle, ou traiter la ligne à la main.
- **Part arrivée après coup** : un paiement par carte groupé dont une ligne de carte est déjà rapprochée passe « à
  examiner » (« une part du paiement est déjà rapprochée ») ; la ligne en francs d'une partielle importée après le groupe
  de sa part en devise se propose seule par la règle (deux écritures, aucun montant compté deux fois) ; un frais de carte
  passé seul ne bloque rien : il va au compte de frais au lieu de la différence de change, et la conversion en deux
  morceaux peut ajouter un centime d'arrondi (aucune écriture n'est fausse). La somme des deux écritures est celle de
  l'écriture unique dans le cas de la ligne en francs seule.

## Change Log

- 2026-11-22 — amendée par la fiche 39-11e (zip de Wise et format des frais) : AC-2 (bouton, compte rendu, refus
  neufs) et AC-4 (renvoi à `sec:wise-export`) ; amendement relu par la validation P1 de 39-11e (0 au-dessus de LOW ;
  son L8 a retiré la phrase « le zip n'est pas lu », qui contredisait l'amendement). Les `reason` de `WISE_ZIP_INVALID`
  se traduisent à l'écran par le message de l'erreur (rendu par le serveur), sans clé par raison.

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
- 2026-11-20 — validation P1 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 2 LOW, remédiés dans la
  fiche (rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11d-ecrans-manuel-e2e-wise-P1.md`). MEDIUM : composants
  de 38-11d nommés sans dire lesquels ni comment les généraliser (case PayPal, `withoutPaypal` et ses quatre sites,
  page PayPal, libellés des refus) ; E2E sans exercice d'accueil ni multi-devises (patron de `paypal-import.spec.ts`) ;
  `docs/api-external.md` absent. LOW : gardes du manuel (numéros de compte), clés i18n et `sitesTotal`.
- 2026-11-20 — validation P2 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 1 LOW, remédiés (rapport
  `kesh-gate-logs/epic-39-40-kiss/validation/39-11d-ecrans-manuel-e2e-wise-P2.md`). MEDIUM : le manuel ne reprenait pas les limites
  de la phase 1 de 38-11b (facture fournisseur, angle mort de la réconciliation générale, plus lourd pour un solde Wise en francs
  qui reçoit conversions et frais). LOW : « écart Stripe » est un motif « à examiner », non un écart.
- 2026-11-20 — alignement sur la phase 1 de l'Epic 38 (Claude Sonnet 5.5, contexte frais ; fiches 38-11c1, 38-11c2, 38-11d et rapport de validation P3 de `28623871` lus) : `provider.ts` / `withoutProviders` déjà livrés par 38-11d sur cinq sites (paiement de facture fournisseur compris) : réutilisés, plus de `paypal.ts` à généraliser ni de « quatre sites » ; `providerOf` retiré. Rapport `kesh-gate-logs/epic-39-40-kiss/validation/alignement-38-P3.md`.
- 2026-11-22 — limites du rapprochement Wise à reprendre au manuel, notées par la remédiation de la revue de code P3 de la
  39-11c (continuité tenue pour preuve, solde en devise épars, part arrivée après coup).

- 2026-11-22 — développée, en `review` (gates ci-dessus) ; revue P1 à lancer (`kesh-gate-logs/s39-11d/revue-p1-prompt.md`).
- 2026-11-22 — revue de code P1 (Claude Opus 5.5, contexte frais, complète) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 7 LOW
  (rapport `kesh-gate-logs/s39-11d/revue-p1-rapport.md`). Remédiation :
  - **M1** : le refus `BANK_IMPORT_DUPLICATE_FILE` se dit par sa phrase, qui nomme le relevé sous le nom où il a déjà été
    importé (`details.existingFilename`, clé `wise-import-error-duplicate-file` aux quatre locales ; repli : message du
    serveur sans nom). Le zip d'un import passé n'est pas nommé : `bank_imports` ne le garde pas (pas de migration dans
    cette story). Tests Vitest (`wise-labels`, `wise-page`), E2E étape 3 (texte exact).
  - **M2** (choix de l'orchestrateur) : `csv_error` ajoute `archive` (nul pour un fichier à part) aux `details` de
    `WISE_CSV_INVALID`, comme `WISE_CSV_FEES_NOT_SEPARATED` ; tests d'intégration (`archive` nul à part ; entrée
    illisible du second de deux zips : entrée et zip nommés). Le cas `wise-labels.test.ts` « WISE_CSV_LINES … archive »
    décrit désormais une forme rendue. `errors.rs`, `docs/api-external.md` (ordre des refus, table) ; manuel : la phrase
    « dit le fichier en cause » ventilée par refus (relevé refusé à la lecture : avec son zip ; deux fichiers d'une
    devise : leurs noms ; déjà importé : le nom sous lequel il l'a été ; zip refusé : son nom).
  - **M3** : note de version ramenée à ce qui est livré ; E2E étape 3 affirme le texte du refus de doublon, étape 2 relit
    au serveur qu'aucun import `chf.csv` n'existe.
  - **L1** : quatre assertions dans `wise_import_e2e.rs` (`wise` + `stripe` refusés, `wise.payoutBankAccountId` refusé,
    Wise → Stripe par `PUT` sans transaction, `wise: null` sans effet sur un compte Stripe).
  - **L2** (le plus simple : garder le lien, la clé existait aux quatre locales) : manuel des contrôles de clôture,
    « Ouvrir la page Wise », avec la limite (la page n'y fait qu'importer) ; test `FiscalYearChecks` d'un compte Wise.
  - **L3** : `docs/api-external.md`, paragraphes Wise et Stripe généralisés (deux prestataires ensemble,
    `payoutBankAccountId` refusé dans `wise`, `wise: null` sans effet sur Stripe).
  - **L4** : `wise-csv-line-date` dit les deux formes admises (JJ-MM-AAAA ou AAAA-MM-JJ), quatre locales.
  - **L6** : décompte annoté au Dev Agent Record (49, puis 50).
  - **L7** : `ProviderReviewPage` ne lit les exercices que si la page a une revue ; test (lecture en échec, import
    visible, `listFiscalYears` non appelé).
  - **L5** noté, non traité : aucune garde des codes Wise contre leur source (patron des 38-11d/40-7d) ; à reprendre avec
    la 39-11d2.
  - `sitesTotal` 3739 → **3740**, relevé par la garde.
  - Gates de la remédiation (journaux `kesh-gate-logs/s39-11d/`) :
    - `cargo fmt --check` et clippy workspace `-D warnings` : rc 0 au commit `a60904e7` (`lint-p1r.log`), et de nouveau au
      commit `bcb66301` (`cible-p1r.log`).
    - nextest complet au commit `a60904e7` : 6614 exécutés, 6614 passés, 17 ignorés, rc 0 (`nextest-p1r.log`) ; après lui,
      `bcb66301` ne change qu'une assertion de `wise_import_e2e.rs` : binaire `wise_import_e2e` rejoué, 13 passés, 1 ignoré
      (`cible-p1r.log`).
    - frontend au commit `a60904e7` : `npm run check` rc 0, `lint-i18n-ownership` rc 0, Vitest 297 fichiers, 5067 tests
      passés (+4 : `FiscalYearChecks` compte Wise, `wise-labels` doublon, `wise-page` doublon et exercices non lus),
      `npm run build` rc 0 (`front-p1r.rc`). `2f2e259a` ne touche que la spec E2E (non rejoué au front).
    - E2E partiel, `wise-import.spec.ts` seule (règle de la nuit) : au commit `a60904e7`, 2 passés, 1 échec (l'étape 3
      affirmait le texte sur le bloc entier, qui porte aussi le lien « Comptes bancaires » : `e2e-p1r.log`) ; corrigé
      (`2f2e259a`, la ligne du refus) : 3 passés (`e2e-p1r2.log`). Ni E2E complet ni `gate-ci.sh`.
    - Mutations hors dépôt, une à la fois, depuis un témoin vert (Vitest 51 passés ; nextest 2 passés) : M1 (cas
      `BANK_IMPORT_DUPLICATE_FILE` neutralisé) tuée par `wise-labels` « relevé déjà importé … » et `wise-page` « refus :
      relevé déjà importé … » (AssertionError) ; L7 (garde `hasReview` retirée) tuée par `wise-page` « les exercices ne
      sont pas lus … » ; L2 (lien Wise neutralisé) tuée par `FiscalYearChecks` « K2 : un compte Wise lie la page Wise » ;
      M2 (`archive` retiré de `csv_error`) tuée par `import_zip_refus_et_cles` (`panicked at wise_import_e2e.rs:1612`,
      `left: Null, right: "b.zip"`). L'assertion « à part » de `import_refus_dans_l_ordre_roles_et_cles` survivait (clé
      absente lue `Null`) : exigée présente en `bcb66301`. Journaux `kesh-gate-logs/s39-11d/mut-p1r/`.
    - Manuel : PDF régénéré dans une copie (`make user`, rc 0), aplati et grepé : « Ouvrir la page Wise pour un compte Wise
      (§14.4) — dans cette version, la page Wise ne fait qu'importer … », la phrase des refus ventilée ; aucun `??`.
  - Passe suivante : P2 complète, Sonnet (`kesh-gate-logs/s39-11d/revue-p2-prompt.md`).
- 2026-11-22 — revue de code P2 (Claude Sonnet 5.5, contexte frais, complète) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 4 LOW
  (rapport `kesh-gate-logs/s39-11d/revue-p2-rapport.md`) ; aucune remédiation P1 n'introduit de défaut. Clôture :
  - **L3** : test Vitest `wise-labels.test.ts`, `archive` sur les sept autres raisons de `WISE_CSV_INVALID` (7 cas
    d'un `it.each`, test seul) ; 34 passés (`cloture/vitest-l3.log`) ; mutation (archive ignorée dans `fileOf`) tuée
    par les sept cas neufs, AssertionError (`cloture/vitest-l3-mut.log`).
  - **L4** : note de version, `archive` ajouté aux `details` de `WISE_CSV_INVALID` (changement additif de l'API).
  - **L1** (`WISE_DUPLICATE_CURRENCY_FILE` ne nomme pas le zip de chaque fichier) et **L2** (le refus de doublon ne
    nomme que le premier relevé déjà importé, sans le dire) : non traités, versés à la liste des issues à ouvrir.
  - Trend de la boucle de revue de code : P1 (Opus) 0/0/3/7 → P2 (Sonnet) 0/0/0/4. Close sur la P2, rien au-dessus de
    LOW. Story `done`. Intégration dans `epic/39-wise` (avec `epic/40-stripe` à jour) : gates au commit de fusion.
- 2026-11-22 — intégrée dans `epic/39-wise` (fusion non squash, puis `epic/40-stripe` à jour). La section « Limites du
  rapprochement à reprendre au manuel », venue de la 39-11c, n'est pas écrite au manuel par cette story : la page Wise
  n'y fait qu'importer, la revue Wise à l'écran et son manuel sont à la 39-11d2, qui reprend ces limites.
