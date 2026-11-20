# Story 38-11d : écrans PayPal, manuel utilisateur, API externe, E2E

Status: ready-for-dev

*Fiche rédigée le 2026-11-20, branche `epic/38-paypal`, après la fusion de `origin/main` (commit `43ceb541`). Epic
38, **phase 1 KISS** (décision de Beat du 2026-11-20). **Dépend de** : 38-11b, 38-11c1 (rattachement en devise, filtres), 38-11c2 (routes de revue et d'acceptation). Reprend, réduits,
l'ancienne 38-9a (réglages d'un compte prestataire) et 38-9b (règles étendues, propositions, « à examiner »).*

## Story

En tant que **Comptable**, je veux régler mes comptes PayPal depuis l'écran des comptes bancaires, importer le CSV
d'un clic, voir les propositions, les écarts et les lignes à examiner sur une page, et accepter la sélection ; en
tant qu'**auteur d'un script**, je veux la même chose par l'API, documentée.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-11d-1` | Paramètres PayPal au formulaire du compte bancaire (compte PayPal, compte de frais), IBAN facultatif, compte du plan en devise admis ; un compte PayPal ne paraît pas là où l'on importe un relevé bancaire ou règle une facture. | AC-1 |
| `FR-38-11d-2` | Deux filtres facultatifs « Type PayPal » et « E-mail exact » et le sens, au formulaire et à la liste des règles (aucun type de critère neuf). | AC-2 |
| `FR-38-11d-3` | Page PayPal : compte PayPal, exercice, bouton d'import et compte rendu, propositions à cocher, écarts, à examiner. | AC-3 à AC-5 |
| `FR-38-11d-4` | Manuel utilisateur (`.tex`) et `docs/api-external.md` décrivent ce que fait le code, limites comprises, et corrigent ce que la phase 1 rend faux. | AC-6, AC-7 |
| `NFR-38-11d-1` | Un E2E traverse l'import et l'acceptation par l'interface ; captures d'écran de chaque état (recette visuelle). | AC-8 |

## Critères d'acceptation

**Décision d'epic reflétée ici** (Beat, 2026-11-20) : **aucun type de critère neuf** dans l'`ENUM` des règles ;
deux **filtres facultatifs**, `provider_type` et `counterparty_email`, s'ajoutent à toute règle, avec le sens — noms
de champs de 38-11c1 AC-3 (`providerType`, `counterpartyEmail`, `direction`, nullables ; `matchValue` obligatoire).

1. **AC-1 — Comptes bancaires.** Fichiers : `frontend/src/routes/(app)/bank-accounts/+page.svelte` (le formulaire de
   création en ligne, `submitCreate`, et celui de modification, `submitUpdateFull` : ce sont eux qui portent le
   choix du compte du plan, `form-journal-account`, et l'IBAN — **non** `BankAccountJournalLinkForm.svelte`, qui n'est
   que le formulaire de lien après coup, par `PATCH`, `updateBankAccountJournalLink`), `format.ts`
   (`linkableLedgerAccounts`, `:181`), `bank-accounts.api.ts` (résumé et charges), un module neuf
   `frontend/src/lib/features/bank-accounts/provider.ts` (à côté de `credit-card.ts`).
   - Case « Compte PayPal » **sous le type *Compte bancaire*** (jamais pour une *Carte de crédit*), à la création et à
     la modification. Cochée : champ « Compte PayPal » (libellé de 1 à 120 caractères, `<datalist>` des libellés déjà
     portés par les comptes chargés, p. ex. « PayPal www »), sélecteur « Compte de frais » (comptes actifs,
     imputables, de type charge, CHF ou de la devise du compte du plan choisi — le sélecteur se recalcule et se vide
     quand ce compte change), **IBAN facultatif** (le `required` tombe ; champ vide → `iban: null`), QR-IBAN masqué
     (`qrIban: null`), « compte principal » masqué (`isPrimary: false` ; sur un compte déjà principal la case est
     **désactivée** avec son motif : le serveur refuse, CHECK de 38-11b), compte du plan **obligatoire** (texte
     d'aide ; 400 `PROVIDER_LEDGER_REQUIRED` sinon).
   - **Comptes en devise offerts** : `linkableLedgerAccounts` gagne l'option `allowForeignCurrency` (défaut faux : son
     filtre `!isForeignCurrencyAccount(a)` de 37-3b est la seule règle des autres écrans) ; le formulaire la passe
     vraie **seulement case cochée**. Le formulaire de lien par `PATCH` garde son filtre : il n'est **pas offert** sur
     un compte PayPal (changer le compte du plan d'un compte PayPal se fait par le formulaire de modification) : le
     serveur refuse **tout** `PATCH` d'un compte PayPal, 400 `BANK_ACCOUNT_FIELD_NOT_ALLOWED` (`field:
     journalAccountId`, 38-11b AC-2).
   - Charges : `paypal: { account, feeAccountId }` à la création ; au `PUT`, le même objet (case cochée), ou
     `paypal: null` (décochée sur un compte qui en portait), ou **absent** (compte non PayPal, inchangé). Refus de
     38-11b affichés par leur message traduit (`PROVIDER_CURRENCY_DUPLICATE`, `PROVIDER_ACCOUNT_IN_USE`,
     `PROVIDER_FEE_ACCOUNT_INVALID`, `PROVIDER_LEDGER_REQUIRED`) : `errorMessageOf` les rend déjà par leur code, les
     quatre messages viennent de 38-11b (`kesh-i18n`), non d'une clé d'écran.
   - `BankAccountSummary` gagne `provider: 'paypal' | 'wise' | 'stripe' | null` et `paypal: { account: string;
     feeAccountId: number | null } | null` (38-11b AC-2 ; le mappage de `bank-accounts.api.ts:109-130` leur donne `null`
     par défaut : une réponse sans les champs est un compte sans prestataire).
     La liste des comptes montre sous le nom « PayPal · <compte> · <devise du compte du plan> » (la devise se lit au
     plan chargé par la page).
   - **Là où un compte de prestataire ne doit pas paraître** : `provider.ts` expose `isPaypal(a)` (`provider ===
     'paypal'`, pour la page PayPal), `isProvider(a)` (`provider !== null`) et `withoutProviders(accounts)` (patron de
     `withoutCreditCards`, `credit-card.ts:39`) — génériques : les Epics 39 et 40 n'y ajoutent rien —, appliqués aux
     sélecteurs de compte de
     `BankImportUpload.svelte` et `ManualStatementForm.svelte` (relevé bancaire : le CSV bancaire refuse une
     devise étrangère, et le CAMT n'a pas de compte PayPal), de `SettleInvoiceDialog.svelte`
     (`nonCardBankAccounts`, `:118` : une facture client ne se règle pas sur un solde PayPal par cet écran) et de la
     page `reconciliation/+page.svelte` (`get_proposals` saute le hors-CHF, et un compte PayPal CHF y recevrait des
     propositions que le serveur ne refuse pas, angle mort de 38-11b AC-3 (b) : la page PayPal a la sienne), et du
     **paiement d'une facture fournisseur** (`frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte:236`,
     `bankAccountsNotLinkedTo` : le serveur refuse ce paiement, 409 `BANK_ACCOUNT_IS_PROVIDER`, 38-11b AC-3 ; la
     liste ne l'offre pas). Autres
     écrans qui listent les comptes bancaires (lots de paiement, QR-facture) : relevés par `grep` de
     `bankAccounts`/`iban` au développement et notés au Change Log ; ceux qui exigent un IBAN le filtrent déjà par
     `iban !== null` (refus serveur existant).
   Vitest : charges (cochée, décochée, inchangée, carte), champs masqués, IBAN vide → `null`, comptes en devise offerts
   seulement case cochée, case désactivée sur le compte principal, `withoutProviders` et ses cinq sites,
   `bank-accounts.api.test.ts` (mappage `paypal`). Gardes à tenir verts : `bank-name-inventory.test.ts`.
2. **AC-2 — Règles** (`frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte`, `RulesList.svelte`,
   `rules.types.ts`, `rules.api.ts`). `MatchType` **ne change pas** (quatre valeurs). `ReconciliationRule`,
   `CreateRuleRequest` et `UpdateRuleRequest` gagnent `providerType`, `counterpartyEmail` et `direction`
   (`string | null` ; `direction` : `'credit' | 'debit' | null`) — au `PATCH`, **absent = inchangé, `null` =
   effacé**, comme `defaultProjectId` (`rules.types.ts:37-40`). Le formulaire gagne, sous le critère existant, trois
   champs facultatifs : « Type PayPal (exact) » (texte), « E-mail exact » (texte, minuscules, un `@` — même
   contrôle que 38-11c1 AC-2, message du serveur sinon) et « Sens » (« Entrées et sorties », « Entrées », « Sorties »
   → `null`, `credit`, `debit`) ; la liste montre les filtres posés à côté du libellé du critère. ⚠️ `matchValue`
   reste **obligatoire** : une règle ne se compose pas des seuls filtres (38-11c1 AC-3) ;
   le manuel dit « critère existant **et** filtres ». Un type de critère inconnu s'affiche brut (comportement actuel).
   Vitest : champs, corps envoyé (posés, vidés, jamais touchés), liste.
3. **AC-3 — Page PayPal** `frontend/src/routes/(app)/reconciliation/paypal/+page.svelte` (feature neuve
   `frontend/src/lib/features/paypal/` : `paypal.api.ts`, `paypal.types.ts`, `paypal-labels.ts`), entrée
   « PayPal » du menu `Mensuel` **dans `comptableOnly`** (`+layout.svelte:138` en est l'exemple ; la route est
   Comptable et plus, 38-11c2 AC-6/7), **toujours visible** pour ces rôles : le menu ne charge pas les comptes
   bancaires, et la page dit « aucun compte PayPal » avec le lien vers *Comptes bancaires* quand il n'y en a pas.
   Dépend de `v0200-menu-sous-groupes` : si ce chantier a réorganisé le menu à l'intégration, l'entrée suit
   le groupe du rapprochement. En tête : sélecteurs « Compte PayPal » (libellés `paypal.account` **distincts** des
   comptes bancaires chargés, `isPaypal`) et « Exercice » (exercices de la société, l'exercice courant par défaut).
   Bouton **« Importer le CSV PayPal »** : choix du fichier (`accept=".csv,text/csv"`), envoi par
   `apiClient.postFormData` (patron de `createBankImport`, `bank-import.api.ts:70-79`) à `POST
   /api/v1/paypal-imports` (`file`, `account`), puis **compte rendu** affiché jusqu'au prochain import (par devise :
   insérées, doublons ; écartées : non terminées avec leur liste, `Memo`, paires neutres) ; refus traduits
   (`PROVIDER_CURRENCY_ACCOUNT_MISSING` nomme les devises et renvoie aux comptes bancaires ; `PAYPAL_CSV_INVALID`
   rend, par ses codes de 38-11a, une phrase par cause — colonnes manquantes **nommées**, lignes en erreur avec numéro
   et champ, ordre des dates ambigu avec son remède — réutilisant `csv-line-error-label.ts` là où le code est celui
   du CSV bancaire). La revue (`GET /api/v1/paypal/review`) se recharge après l'import et après l'acceptation ;
   ses requêtes périmées sont écartées (garde de génération, patron de `LinkSettlementModal`).
4. **AC-4 — Propositions.** Tableau : date, genre (« Vente rattachée », « Frais », « Règle <libellé> »,
   « Conversion », « Retrait »), montant et devise, nom, TXN, ce qui sera fait (écriture rattachée n° … ; compte de
   frais ; contrepartie ; une écriture quand un côté est en CHF, sinon deux écritures au compte de conversion ;
   compte de destination). Le genre « Vente rattachée » est la proposition générique `link_entry` (38-11c2 AC-5). Une case par ligne,
   « Tout cocher » ; le retrait sans compte de destination porte un sélecteur (comptes d'actif CHF imputables) et
   reste décoché tant qu'il est vide. **« Accepter la sélection »** → `POST /api/v1/paypal/accept`, par paquets de
   500 propositions au plus (`MAX_IN_CLAUSE_IDS`, `reconciliation.rs:1121`, 38-11c2 AC-7 ; l'écran dit « lot 2 sur 3 »),
   un paquet à la fois dans l'ordre des dates, compte rendu cumulé : acceptées, et chaque refus avec son message.
   **Libellés des refus** : `failed-proposal-label.ts` **n'est pas étendu** (son test compare ses codes aux
   littéraux de `reconciliation.rs`, 48 codes) ; `paypal-labels.ts` rend `PROVIDER_PROPOSAL_STALE` et les codes
   propres au rapprochement des prestataires par des clés `paypal-failed-*`, et **délègue** à `failedProposalLabel` pour les autres
   (`RATE_UNAVAILABLE` et `FX_ACCOUNT_INVALID` y sont déjà : codes de `fx_failed_proposal`, 37-3b ; relevé de ce qui
   manque par `grep` des codes de `routes/paypal.rs` et `routes/provider_accept.rs` au développement). Vitest : un test
   écrit en dur des codes de la route PayPal contre ces deux fichiers (patron du test de `failed-proposal-label`).
5. **AC-5 — Écarts et à examiner.** Deux tableaux en lecture : **Écarts** — « Encaissement sans vente »,
   « Vente sans encaissement » (lien vers l'écriture), « Montant ou devise différents » (attendu et trouvé) ;
   **À examiner** — motif traduit (« Plusieurs ventes portent cette TXN », « Compte de frais non réglé », « Conversion
   sans sa paire », « Retrait d'un solde en devise : écriture à la main », « Aucune règle »), type PayPal, nom,
   e-mail. Action **« Rattacher à une écriture »** : ouvre
   `LinkSettlementModal.svelte` en mode écriture, mais ⚠️ **ses entrées ne sont pas celles de la page** : elle prend
   un `ReconciliationProposal` (la proposition du rapprochement CHF, `proposal.transaction.amount`) et lit les
   règlements (`getLinkCandidates`, route de la 29-8, **restée CHF**, 38-11c1 AC-1) pour tout **crédit**. Deux
   changements, testés : (a) la modale accepte une transaction seule (`{ transaction }`, que la page PayPal
   construit depuis l'élément « à examiner » et le rapprochement CHF depuis sa proposition — type `Pick` de
   `ReconciliationProposal`, ni copie ni second composant) ; (b) elle **ne charge pas** les règlements pour une
   transaction qui n'est pas en CHF (`transaction.currency`), et affiche montants et écarts dans **la devise de la
   transaction** (`entry-candidates` rend le mouvement en devise, 38-11c1 AC-1). Lien « Écriture manuelle » vers la
   saisie (`/journal-entries`). **Pas d'export CSV** des tableaux (retiré à la validation : hors périmètre KISS, aucune
   fonction de dépôt à réutiliser). Libellés aux quatre locales (`crates/kesh-i18n/locales/*`, propriété des clés :
   préfixe `paypal-` pour `features/paypal/`, `bank-accounts-` et `reconciliation-` ailleurs,
   `docs/i18n-key-ownership-pattern.md` ; le libellé du menu est un `nav-*`) ; `sitesTotal` de
   `frontend/src/lib/shared/i18n-keys.test.ts` (`:1153`, 3495 à la lecture de `origin/main`) **recompté** depuis la
   source après la fusion, jamais incrémenté de confiance. Chaque clé en toutes lettres (jamais construite par
   gabarit : les gardes ne les voient pas).
6. **AC-6 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, **le `.tex` seul** : les PDF se régénèrent à la
   release). Sous-section « Comptes PayPal » dans la section *Import bancaire* (`\section{Import bancaire}`, `:2995`)
   ou après *Comptabilité multi-devises* (`:1063`), avec renvois (`\S\ref{sec:multi-devises}`) : un compte bancaire
   par compte PayPal et par devise, lié au compte du plan de la devise (les devises étrangères exigent le
   **multi-devises** activé), sans IBAN ; compte de frais (CHF ou devise) ; exporter de PayPal le rapport « Balance
   affecting transactions » en CSV, colonnes en anglais, **dates sans ambiguïté** (un jour > 12 dans le fichier, sinon
   refus) ; ce que fait l'import (filtres, frais en ligne à part, y compris les frais positifs d'un remboursement,
   doublons, paires neutres) ; les ventes viennent des boutiques avec « TXN <id> » dans le libellé ; le rapprochement
   et les trois écarts ; les filtres « Type PayPal » et « E-mail exact », le sens et la **limite** d'une règle par
   critère et valeur ; la conversion — **une** écriture au CHF réel quand un côté est en francs, deux au même CHF par
   le compte de conversion entre deux devises étrangères —, et comment l'annuler (une écriture : annuler un côté ne
   retire que son lien ; deux écritures : annuler chacune de ses deux transactions, le compte de conversion n'étant
   pas soldé entre les deux) ; le retrait, depuis un solde en francs seulement (sinon « à examiner ») ; les
   **limites** : date de l'export sans conversion de fuseau (fin d'année), remboursements et litiges à la main,
   aucun code TVA, pas de téléchargement automatique (phase 2) ; **une facture fournisseur ne se paie pas depuis un
   compte PayPal** (38-11b AC-3 (a)) — remède : la régler par « compte interne », ou passer l'écriture à la main (en
   devise pour un solde en devise), puis rattacher la transaction PayPal à cette écriture ; **angle mort** (38-11b
   AC-3 (b)) : par l'API, la réconciliation générique ne refuse pas une proposition « facture » ou « règle », ni la
   saisie manuelle ou l'éclatement d'une transaction, sur un compte PayPal en francs — la traiter par la page PayPal
   (la proposition « facture fournisseur » et les lots pain.001 sont refusés, revue P1 de 38-11b). **Ce que la phase 1 rend faux, à corriger dans le
   même commit** (symptôme grepé : `tenu en devise`, `en devise` près de `relevé`, `restent en CHF`) : `:1137` (« un
   compte en devise ne peut pas être lié à un compte bancaire » : sauf un compte PayPal), `:3328-3333` (limites du
   relevé saisi : « un compte bancaire est toujours lié à un compte en francs suisses », « les relevés en devise sont
   prévus pour une version ultérieure (\#233) »), `:4881` (annexe des fonctionnalités non disponibles : « relevés
   bancaires en devise … restent en CHF » : le relevé PayPal, lui, est en devise). Gardes de `textes_coherents.rs`
   **à connaître avant d'écrire** : G2 interdit le mot capitalisé « Réglages » partout dans les manuels (écrire
   « paramètres PayPal », « Paramètres » pour le menu) ; G4-bis/G4-ter n'admettent comme numéro de compte cité que
   ceux d'un plan livré (pas de « 1018 », « 4170 », « 9901 » : dire « le compte de frais », « le compte de
   conversion ») ; G25 interdit « projet » pour le compte analytique ; G18 vise « toutes les sociétés ». Les autres
   langues : « à traduire » noté (manuels DE/IT/EN, règle du dépôt). Le PDF est contrôlé par le prompt de revue
   (`pdftotext … | tr '\n' ' ' | tr -s ' '`) **après** régénération locale, sans le commiter.
7. **AC-7 — `docs/api-external.md`.** Section neuve après *Accepter des propositions de rapprochement* (`:711`) :
   `POST /api/v1/paypal-imports` (multipart, réponse, refus), `GET /api/v1/paypal/review`, `POST
   /api/v1/paypal/accept` (formes des propositions, *Pattern batch*), champ `paypal` des comptes bancaires, filtres et
   `direction` des règles, rattachement en devise ; un exemple `curl` d'import et d'acceptation avec une clé
   `read-write` (identifiants d'exemple **non à quatre chiffres** ni numéros de compte : G4-ter lit ce fichier) ; le
   script des ventes des boutiques (R1) écrit ses écritures par `POST /api/v1/journal-entries` avec « TXN <id> » dans
   le libellé et la ligne PayPal en devise. **À corriger dans le même commit** : `:1073` et `:1309` (« un compte en
   devise ne sert pas … comme compte lié d'un compte bancaire » : sauf avec `paypal`), `:944` et `:1293`
   (`RECONCILIATION_ENTRY_NOT_LINKABLE`, « transaction hors CHF » : seulement hors de la devise du compte, 38-11c1
   AC-1 ; seule cette story corrige ce fichier), la table des erreurs du § 10 (codes `PROVIDER_*`, `PAYPAL_CSV_INVALID`
   et `BANK_ACCOUNT_IS_PROVIDER` de 38-11b et 38-11c2). Mêmes gardes que
   AC-6 (G4-ter, G25, G6).
8. **AC-8 — E2E et recette visuelle.** `frontend/tests/e2e/paypal-import.spec.ts`, patron de `fx-entries.spec.ts`
   (société `with-company`, **exercice 2020 ouvert au 01.01.2020**, multi-devises activé par l'API ; l'activation
   est irréversible : la spec **reseede `with-company` en fin de parcours**) : comptes du plan EUR et USD (le CHF
   existe), comptes bancaires PayPal EUR, USD et CHF d'un même compte PayPal, compte de frais, compte de conversion,
   cours mensuels de 2020 posés par `PUT /exchange-rates/rates`, une règle par e-mail, deux écritures de vente
   « TXN … » dont une de montant faux. Import de la fixture fictive de 38-11a (dates de 2020), lue par chemin depuis
   `crates/kesh-import/tests/fixtures/paypal-balance-affecting-fictif.csv` (ni copie, ni seconde fixture), **par
   l'interface** ; compte rendu attendu ; propositions, un écart de chaque genre ; « Tout cocher », « Accepter la
   sélection » → aucun refus, revue vide de propositions ; rattachement d'une ligne « à examiner » par la modale.
   Captures d'écran de chaque état (formulaire du compte PayPal, règle avec filtres, compte rendu d'import,
   propositions, écarts, à examiner, compte rendu d'acceptation), thème clair, dans `KESH_CAPTURES_DIR` (mécanisme de
   `fx-entries.spec.ts`), hors dépôt (`kesh-gate-logs/`), jointes au Dev Agent Record et au prompt de revue. Suite E2E
   complète au dernier commit de code (7 à 9 échecs attendus, jugés fichier par fichier, `docs/testing.md`).

## Tâches

- [ ] T1 — Formulaires et liste des comptes bancaires, `provider.ts`, `withoutProviders` aux cinq sélecteurs (AC-1).
- [ ] T2 — Règles : filtres et sens (AC-2).
- [ ] T3 — Feature et page PayPal : import, propositions, écarts, à examiner, modale de rattachement ; i18n,
  `sitesTotal` (AC-3 à AC-5).
- [ ] T4 — Manuel utilisateur, `docs/api-external.md`, passages rendus faux (AC-6, AC-7).
- [ ] T5 — E2E, captures, suite complète (AC-8).

## Stratégie de tests

Vitest pour chaque composant touché (corps envoyés, états, refus traduits) ; `npm run check`,
`npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build` ; E2E de l'AC-8 puis la suite complète au dernier
commit de code ; garde `textes_coherents.rs` verte (G2, G4-bis, G4-ter, G25 sur le manuel et `api-external.md`).

## Modules touchés (5)

`frontend` (bank-accounts, bank-import, invoices, supplier-invoices, reconciliation et rules, paypal neuve), `kesh-i18n` (libellés),
manuel utilisateur FR, `docs/api-external.md`, E2E.

## Note de version

*Ajouté* — Écran PayPal : import du CSV d'un clic, propositions à accepter en lot, écarts entre boutiques et PayPal,
lignes à examiner ; paramètres PayPal au formulaire du compte bancaire ; filtres de règle « Type PayPal » et
« E-mail exact », avec un sens facultatif.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
- 2026-11-20 — validation P1 (Sonnet 5.5) : 0 CRITICAL, 3 HIGH, 6 MEDIUM, 3 LOW, tous remédiés. HIGH : AC-1 visait
  `BankAccountJournalLinkForm.svelte` (formulaire de lien par `PATCH`) au lieu des formulaires de la page, et le
  filtre de devise de `linkableLedgerAccounts` n'avait pas d'option ; AC-2 ajoutait deux types de critère contre la
  décision d'epic (filtres) ; AC-5 : `LinkSettlementModal` prend un `ReconciliationProposal` et charge les règlements
  CHF pour tout crédit (une ligne EUR). MEDIUM : comptes PayPal offerts aux sélecteurs d'import, de règlement et de
  réconciliation ; `failed-proposal-label.ts` étendu malgré son test de 48 codes ; passages du manuel (`:1137`,
  `:3328`, `:4881`) et de `api-external.md` (`:1073`, `:944`, `:1293`, `:1309`) rendus faux ; gardes G2 (« Réglages »),
  G4-bis/ter (numéros de compte), G25 non nommées ; E2E : exercice et dates de la fixture, reseed final ; menu
  « visible dès qu'un compte… » impossible sans charger les comptes. LOW : export CSV retiré (hors KISS) ; phrase
  sur la clé d'API de la sonde négative, supprimée ; paquets de 500 précisés. Rapport :
  `kesh-gate-logs/epic-38-kiss/validation/38-11d-ecrans-manuel-e2e-paypal-P1.md`.
- 2026-11-20 — validation P3 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW, remédiés (rapport
  `kesh-gate-logs/epic-38-kiss/validation/38-11-P3.md`). Limite du paiement fournisseur et son remède, et angle mort
  de la réconciliation générique, au manuel ; écran de paiement d'une facture fournisseur (`supplier-invoices/[id]`)
  sans compte de prestataire ; `isProvider` / `withoutProviders` (`provider !== null`, module `provider.ts`) au lieu de
  `withoutPaypal`, cinq sites ; texte du `PATCH` aligné (tout `PATCH` d'un compte PayPal refusé, 38-11b AC-2) ;
  conversion à une écriture quand un côté est en CHF et son annulation, retrait depuis un solde CHF seulement ;
  proposition `link_entry` ; codes `PROVIDER_*` ; renvois à 38-11c1 / 38-11c2 ; paragraphe de décision et « à
  confirmer » périmés retirés.
