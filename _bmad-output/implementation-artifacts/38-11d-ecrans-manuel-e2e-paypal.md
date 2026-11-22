# Story 38-11d : écrans PayPal, manuel utilisateur, API externe, E2E

Status: review

*Fiche rédigée le 2026-11-20, branche `epic/38-paypal`, après la fusion de `origin/main` (commit `43ceb541`). Epic
38, **phase 1 KISS** (décision du responsable de l'epic du 2026-11-20). **Dépend de** : 38-11b, 38-11c1 (rattachement en devise, filtres), 38-11c2 (routes de revue et d'acceptation). Reprend, réduits,
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

**Décision d'epic reflétée ici** (responsable de l'epic, 2026-11-20) : **aucun type de critère neuf** dans l'`ENUM` des règles ;
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

- [x] T1 — Formulaires et liste des comptes bancaires, `provider.ts`, `withoutProviders` aux cinq sélecteurs (AC-1).
- [x] T2 — Règles : filtres et sens (AC-2).
- [x] T3 — Feature et page PayPal : import, propositions, écarts, à examiner, modale de rattachement ; i18n,
  `sitesTotal` (AC-3 à AC-5).
- [x] T4 — Manuel utilisateur, `docs/api-external.md`, passages rendus faux (AC-6, AC-7).
- [x] T5 — E2E, captures, suite complète (AC-8).

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
- 2026-11-20 — développée (`bmad-dev-story`) : commits `31aca7f0` (écrans), `62b9d281` (manuel, `api-external.md`),
  `40516c88` (page alignée sur les formes de la revue), fusion de `story/38-11c2` (`ecd54a38`, sans conflit), `5d04b925`
  (libellés des refus de la route, spec E2E). Statut `review`. Détail au Dev Agent Record.

- 2026-11-21 — fusion du correctif de la 38-11c2 (`59acd9fd`, fusion `1943e067`, sans conflit) ; **revue de code P1**
  (Sonnet 5.5, `kesh-gate-logs/s38-11d/revue-p1-rapport.md`) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 9 LOW, remédiés aux commits
  `81916faf` et `10277d0e` (clippy). M1 : K2 (préparation de clôture) dit le prestataire de chaque compte en attente
  (`PendingByAccount.provider`, `kesh-db`) et lie la page PayPal pour un compte PayPal, le lien générique seulement s'il
  reste un compte ordinaire ; phrase au manuel. M2 : un test par site sans test (règlement client, paiement
  fournisseur, réconciliation — page neuve testée avec un remplaçant de `ReconciliationProposals`), mutations rouges ;
  le serveur **ne refuse pas** la réconciliation générique d'un compte PayPal (angle mort de 38-11b AC-3 b, écrit au
  manuel et à `api-external.md`), versé à `kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md` (P2). M3 : l'écart d'une
  vente déjà rattachée a son libellé (« Vente déjà rattachée à une autre transaction ») ; « aucun débit trouvé » au lieu
  de « trouvé » suivi de rien ; manuel. M4 : `api-external.md` cite les codes réels (`RECONCILIATION_FISCAL_YEAR_CLOSED`
  `{ entryDate }`, `LATER_FISCAL_YEAR_CLOSED` `{ fiscalYearId, fiscalYearName }`, et la liste des codes de `failed[]`
  relevée au code). M5 (décision de l'orchestrateur) : `TransactionSummary.counterparty_email` (module de la 38-11c2)
  servi par la revue, assertion HTTP dans `paypal_reconciliation_e2e`, colonne « E-mail » de « À examiner » remplie,
  documentée. LOW : sélecteurs et import désactivés pendant l'acceptation, compte figé avant la boucle (L1) ; tests de
  robustesse de la page (L2) ; codes convertis des refus d'écriture listés en dur (L3) ; textes du manuel et de
  `api-external.md` (L4 à L6 : retrait en devise, angle mort complet, place du choix de l'exercice, compte rendu effacé
  au changement de compte, retrait prérempli) ; compte de frais vidé **seulement** quand la devise du compte du plan ne
  l'admet plus (L7 — écart assumé à la lettre de l'AC-1 « se vide quand ce compte change », qui effaçait en silence un
  choix fait avant le compte du plan) ; captures 07 et 08 par élément et états ajoutés (L8) ; libellés de toutes les
  règles, page par page (L9). Mutations survivantes de la P1 tuées (`mutations-front-2.log`, 12 rouges sur 13 ; la
  treizième — compte relu à chaque paquet — équivalente depuis que le sélecteur est désactivé pendant l'acceptation).

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`bmad-dev-story`), worktree `kesh-wt38d`, branche `story/38-11d`.

### Implementation Plan / décisions

- **AC-1** — `provider.ts` (`isPaypal`, `isProvider`, `withoutProviders`, `paypalAccountLabels`,
  `feeAccountCandidates`, `paypalSummaryLine`, `paypalPayload`) ; `linkableLedgerAccounts` gagne
  `allowForeignCurrency` ; formulaires de création et de modification (snippet `paypalFields`) : case sous *Compte
  bancaire*, désactivée sur le compte principal avec son motif, `<datalist>` des libellés, compte de frais recalculé et
  vidé au changement de compte du plan, IBAN facultatif (`null` si vide), QR-IBAN et principal masqués, `paypal: null`
  au `PUT` d'un compte décoché qui en portait, absent sinon. **Décision** : le solde d'ouverture est masqué quand le
  compte du plan est en devise (formulaire et bouton de ligne) — la route du solde d'ouverture n'écrit qu'en CHF ; le
  manuel renvoie à une écriture en devise. « Lier » (`PATCH`) n'est pas offert sur un compte PayPal.
- **`withoutProviders`, six sites** (cinq de la fiche + l'écran des lots pain.001, items 9 et 10 de
  `a-faire-p3-38.md`) : import CSV/CAMT, saisie manuelle, règlement d'une facture client, réconciliation, paiement d'une
  facture fournisseur, lots de paiement. **Écart à la fiche (module `kesh-api` touché)** : l'import, la saisie et la
  réconciliation lisent les comptes de `GET /companies/current`, qui ne portait pas le prestataire ; `BankAccountJson`
  gagne `provider` (changement additif, documenté à `api-external.md`, test
  `companies_current_porte_le_prestataire_des_comptes`). Relevé des autres écrans (grep `listBankAccounts`,
  `bankAccounts`) : accueil (affichage seul), *Paramètres → Facturation* (exclusion des comptes liés, juste avec un
  compte PayPal), QR-facture (compte principal, jamais PayPal) — aucun autre site d'écriture.
- **AC-2** — `rule-filters.ts` (normalisation miroir de `normalize_email`, `changedFilters` : au `PATCH` seuls les
  filtres changés partent), trois champs au formulaire, filtres affichés à côté du critère dans la liste ; `parseRule`
  sur toutes les lectures (champs absents → `null`).
- **AC-3 à AC-5** — feature `features/paypal/` (`paypal.types.ts` aligné sur `provider_review.rs` /
  `provider_accept.rs` de 38-11c2 : `transaction`, `otherTransaction`, `entry`, constats `code` / `entries` /
  `details`), `paypal.api.ts` (import multipart, revue, acceptation, paquets de 500), `paypal-labels.ts` (une phrase
  par cause de refus de l'import ; genres, écarts, motifs ; refus de l'acceptation : six codes propres
  `paypal-failed-*`, les autres délégués à `failedProposalLabel`, non étendu), `paypal-review.ts` (exercice par défaut,
  corps dans l'ordre des dates, retrait incomplet écarté). Page `reconciliation/paypal` : garde de génération, revue
  relue après import et acceptation, retrait non cochable tant que sans compte (la société `with-company` n'a pas de
  compte principal : sélecteur), libellés des règles lus par `listRules`. Menu `Mensuel`, `comptableOnly`.
  `LinkSettlementModal` prend un `LinkTarget` (`Pick` de `ReconciliationProposal`) et ne charge les règlements que pour
  un crédit **en CHF** ; montants des écritures suivis de la devise de la transaction. L'e-mail de la contrepartie
  n'est pas servi par `TransactionSummary` (38-11c2) : la colonne reste vide (champ facultatif du type).
- **AC-6, AC-7** — § `sec:comptes-paypal` du manuel (fin de *Import bancaire*), passages corrigés (`:1137`, limites du
  relevé saisi, annexe, rattachement, règles : filtres et limite du doublon, règles en devise sur la page PayPal) ;
  limite fournisseur et remède, angle mort complet (`get_proposals`, `split`) — items 4 et 10. `api-external.md` :
  § « Rapprocher un compte PayPal » (revue, acceptation, filtres, script des boutiques, exemple `curl`), ligne du
  tableau des ressources, `provider` de `GET /companies/current`, rattachement en devise (règle, tableau des refus,
  § 10), `ACCOUNT_IN_FOREIGN_CURRENCY` sauf compte PayPal, angle mort de `:1001` complété, `PROVIDER_PROPOSAL_STALE`.
  Le texte du `PATCH` disait déjà « refusé » sans réserve (item 4) : inchangé. Manuels DE/IT/EN : à traduire.
- **i18n** — 107 clés neuves aux quatre locales (11 `bank-accounts-`, 11 `reconciliation-rules-`, 1 `nav-paypal`,
  84 `paypal-`, recomptées sur les trois commits de la story) ; `sitesTotal` 3495 → **3615** et `CANDIDATES_ATTENDUES` 130 → **137** (conformes 109 → 116), relevés
  par les gardes, non additionnés ; `bank-name-inventory` 5 → 7 pour la page des comptes (deux payloads PayPal).

### Debug Log / gates (cible propre `kesh-targets/kesh-wt38d`, `CARGO_INCREMENTAL=0`, dev2 port 3307, `heavy.sh`)

Aucun gate cargo n'a été joué sur l'ancienne cible partagée : rien à rejouer. Journaux : `kesh-gate-logs/s38-11d/`.
Tous au commit `5d04b925` (dernier commit de code), `gates-all.sh` :

- **Frontend** : `npm run check` 0 erreur (32 avertissements, tous antérieurs) ; `lint-i18n-ownership` vert ;
  `test:unit` **4691/4691** (`front-test-unit.log`) ; `build` vert.
- **E2E complet** : **482 passés, 10 échecs, 18 ignorés** (`e2e.log`) — les dix attendus, jugés un à un : les sept de la
  KF-029 (`mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`),
  `backup-restore-new-install:262`, `:381`, `opening-balances:156`. `paypal-import.spec.ts` 5/5 (aussi seul,
  `e2e-cible.log`).
- **Backend** `scripts/test-fast.sh --ci` : **rouge, rc 100** — 6265 exécutés, **6258 passés, 7 échecs**, 14 ignorés
  (`gate-backend.log`). Les sept sont **tous dans le code fusionné de la 38-11c2** (aucun fichier de la 38-11d) :
  `audit_label_registry` ×2, `audit_route_registry::every_replayed_route_calls_an_envelope` (31 routes rejouées pour
  30 déclarées), `paypal_reconciliation_e2e` ×3 (colonne `api_key_id` inconnue, fixture à dates ambiguës, interblocage),
  `kesh-reconciliation provider_review::tests::la_cle_de_rattachement_est_au_parametrage`. La 38-11c2 est encore
  `in-progress`, ses fichiers en cours de modification dans `kesh-wt38c2` (`provider_accept.rs`,
  `audit_route_registry.rs`, `paypal_reconciliation_e2e.rs`, `provider_review.rs`). Les tests de la 38-11d passent
  (`companies_current_porte_le_prestataire_des_comptes`, `textes_coherents` entier).
- **gate-ci** : **rouge, rc 101**, arrêté au binaire `audit_label_registry` (mêmes deux échecs de la 38-11c2 ; les
  binaires précédents verts). Cible de la copie supprimée.
- **À rejouer** après la fusion du correctif de la 38-11c2 : gate backend et gate-ci (la story ne touche que
  `routes/companies.rs` et un test côté Rust).
- **Mutations front** (`mutations-front-1.log`) : 9/9 rouges (`withoutProviders` de l'import, de la saisie, des lots ;
  `allowForeignCurrency` ; `changedFilters` ; CHF de la modale ; `paypal: null` décochée ; case du principal ;
  corps incomplets).
- **Manuel** : compilé dans une copie (`kesh-gate-logs/s38-11d/manual/`, `latexmk -xelatex`, aucune référence
  indéfinie au passage final), PDF non commité ; § lu aplati par `pdftotext`.
- **Captures** (thème clair) : `kesh-gate-logs/s38-11d/captures/01` à `11` (formulaire du compte PayPal, liste,
  règle avec filtres, liste des règles, compte rendu d'import, propositions, écarts, à examiner, compte rendu
  d'acceptation, modale, après rattachement).

### Gates après la revue P1 (dernier commit de code `10277d0e`, même cible et base)

- **Gate backend** `scripts/test-fast.sh --ci` : **vert**, 6266 exécutés, **6266 passés**, 14 ignorés
  (`gate-backend-3.log`) — les sept échecs de la 38-11c2 relevés plus haut sont levés par son correctif.
- **Frontend** : `check` 0 erreur ; `lint-i18n-ownership` vert ; `test:unit` **4719/4719** ; `build` vert
  (`front-3-*.log`).
- **E2E complet** : **484 passés, 10 échecs, 18 ignorés** (`e2e-3.log`) — les dix attendus, jugés un à un (sept
  KF-029, `backup-restore-new-install:262`, `:381`, `opening-balances:156`) ; `paypal-import.spec.ts` 7/7.
- **gate-ci** (`gate-ci-3.log`) : rc 101, un seul échec, `admin_auto_backup_e2e::disque_sature_pendant_l_ecriture`
  (panique du crate `zip`), test d'un module que la story ne touche pas, vert aux deux gate-ci précédents et sous nextest
  au même commit : intermittent, versé à `issues-a-ouvrir.md` (KF). Rejoué : voir la ligne suivante.
- **gate-ci rejoué** (`gate-ci-4.log`, même commit `10277d0e`) : **vert**, rc 0, 6285 passés, 0 échec, 18 ignorés (260 résultats de binaire) ; `disque_sature_pendant_l_ecriture` vert.
- **Gate ciblé** de la remédiation (`cible-p1-nextest.log`) : 156/156 (K2, revue PayPal avec e-mail, comptes,
  `textes_coherents`, registres d'audit, `kesh-reconciliation`), clippy vert.
- Entre les deux : le gate backend au commit de fusion `1943e067` n'a pas tourné (`test-fast.sh` exige un dépôt git, la
  copie `git archive` n'en est pas un) ; le gate-ci de `1943e067` (`gate-ci-2.log`) a rougi sur deux tests
  d'`auth_e2e` par une base de test disparue en cours de route (`Unknown database _sqlx_test_…`), sans rapport.
- **Captures** refaites par la suite complète : `kesh-gate-logs/s38-11d/captures/00` à `13` (dont `05a` refus de
  l'import, `06a` retrait à choisir, `12` acceptation avec refus, `13` case désactivée sur le principal, `07` et `08`
  par élément) ; celles des autres specs rangées à part (`captures/autres-specs/`).
- Tests ajoutés par la remédiation (de `1943e067` à `10277d0e`) : 2 tests d'intégration Rust
  (`k2_dit_le_prestataire_du_compte_en_attente`, assertion e-mail dans `revue_puis_acceptation_du_tout`), Vitest : 3 sites
  (`SettleInvoiceDialog`, `supplier-settlement-page`, `reconciliation-page.test.ts` 2), K2 1, écarts 5, page 6,
  compte de frais 1, sens 1, codes convertis 1 `it.each` de 10 ; E2E : étapes 6 et 7.
- Compteurs : `sitesTotal` **3618**, `CANDIDATES_ATTENDUES` **138** (conformes 117), relevés par les gardes.
- Prompt de la revue P2 (Opus, passe complète) : `kesh-gate-logs/s38-11d/revue-p2-prompt.md`.

### Reprise après la revue P2 et la fusion d'`epic/38-paypal` (`1e91465b`) — dernier commit de code `681281f7`

- **Fusion** : conflits résolus en gardant les deux intentions dans `docs/api-external.md` (compte PayPal admis comme compte lié ; règle de rapprochement en devise refusée par proposition), `docs/manual/fr/user-manual.tex` (mêmes deux phrases) et `i18n-keys.test.ts` (compteur relevé par la garde, non additionné). Relais de la 38-11c2 intégré : `COUNTERPARTY_CURRENCY_MISMATCH` traduit par `paypalFailedLabel` (clé `paypal-failed-counterparty-currency-mismatch`, quatre locales : le préfixe `provider-` de la 38-11c2 est refusé par `lint-i18n-ownership` hors de sa feature), documenté à `api-external.md` (route d'acceptation, règle `rule`, tableau des erreurs) et au manuel (règles) ; l'écran de règle propose déjà les comptes en devise (38-11c2).
- **Revue P2** : L1 (bouton grisé pendant la relecture), L2 (test de la remise à zéro de la sélection), L3 (compte de frais vidé seulement sur la devise), L5 (`provider` de K2 écrit à `api-external.md`) remédiés ; L4 (Wise et Stripe sans lien K2) reporté aux Epics 39 et 40 ; L6 : les trois échecs E2E `backup-restore-new-install:262`, `:381` et `opening-balances:156` se produisent aussi sur d'autres branches de l'epic, à ajouter à la liste de `docs/testing.md` par la responsabilité de l'epic.
- **Gates** (journal `kesh-gate-logs/s38-11d/`) : ciblé vert (clippy ; nextest 176/176, `cible-p3-nextest.log`) ; **backend complet** `test-fast.sh --ci` : 6419 passés, 15 ignorés (`gate-backend-5.log`) ; **frontend** : `check`, `lint-i18n-ownership`, `build` verts, Vitest **4941/4941** (`front-5-*.log`), `sitesTotal` **3632** relevé par la garde.
- **E2E complet** (`e2e-5.log`, au dernier commit de code) : **497 passés, 11 échecs, 18 ignorés** : les sept de la KF-029, `backup-restore-new-install:262` et `:381`, `opening-balances:156` (attendus), et `vat-purchase-assistant.spec.ts:115` (connexion refusée : `/login`), **non attendu**, rejoué : vert (`e2e-partiel.log`, 8 passés avec la spec de la page PayPal) ; `paypal-import.spec.ts:241` rouge dans cette reprise partielle pour la même cause (connexion), vert à la reprise seule : `paypal-import.spec.ts` **7/7** (`e2e-partiel2.log`). Cause de la connexion refusée non établie (instabilité de la suite groupée, pas de la story).
- **Par décision du responsable de l'epic** : pas de `gate-ci.sh` nouveau (le dernier, vert, reste celui de `10277d0e`), et l'E2E complet ci-dessus (`e2e-5.log`) est le seul rejoué à cette reprise ; E2E partiel : `vat-purchase-assistant`, `paypal-import`, `smoke`.
- **Mutations** : retrait de `selected = new Set()` et de `reviewLoading` du bouton lancées, non menées à leur terme (file de la station) ; les tests neufs (L1, L2, L3) passent, leur sonde négative reste à faire en revue.
- Prompt de la revue P3 (passe complète, règle neuve) : `kesh-gate-logs/s38-11d/revue-p3-prompt.md`.

### Remédiation de la revue P3 (0 au-dessus de LOW) — dernier commit de code `e45f3a34`

- **Remédiés** : L1 (`withdrawalAccountCandidates(accounts, excludedIds)` : la page écarte le compte du plan du compte PayPal choisi, que le serveur refuse, `check_counterparty` ; le compte de conversion n'est pas refusé par le serveur, il reste offert), L2 (« Accepter la sélection » grisé pendant l'import ; « Tout cocher » et « Tout décocher » grisés pendant l'acceptation ; `reviewLoading` remis à faux à la sortie anticipée de `loadReview`, cas latent sans test), L4 (`paypalAccountLabels` sans casse, premier libellé gardé), L5 (commentaire de `LinkSettlementModal.svelte`), L6 (la phrase de la reprise P2 sur l'E2E).
- **Reportés ou sans suite** : L3 (manuel « pas de téléchargement automatique » contre les routes de la 38-12a) écrit en tête de la fiche 38-12b, section « À reprendre » ; L7 et L8 : rien.
- **Tests neufs** : `paypal-review.test.ts` (exclusion), `provider.test.ts` (casse), `paypal-page.test.ts` (destinations sans le compte du plan ; boutons pendant l'import et l'acceptation). **Sondes par mutation** : les six « tuées » du compte rendu P3 (`revue-p3-fix-mut/`) étaient des échecs de chargement (« Cannot find module », « no tests »), sans valeur de preuve (revue P4, P4-1) ; elles sont **rejouées** à la remédiation P4 (voir ci-dessous). La remise à faux de `reviewLoading` n'a pas de sonde (cas inatteignable à l'écran).
- **Gates** (`kesh-gate-logs/s38-11d/front-6-*.log`) : `check`, `lint-i18n-ownership`, `build` verts ; Vitest **4945/4945** (287 fichiers). Aucun Rust modifié : ni fmt, ni clippy, ni nextest rejoués. Aucune spec E2E ajoutée : aucun E2E rejoué.
- Prompt de la passe ciblée P4 : `kesh-gate-logs/s38-11d/revue-p4-prompt.md`.

### Remédiation de la revue P4 (HIGH P4-1, LOW P4-2, LOW P4-3) — dernier commit de code `6f8bba02`

- **P4-1** : mutations rejouées dans une copie de `frontend/` du commit de code (`git archive`, `node_modules` copié, non lié), copie non mutée verte (3 fichiers, 40 tests) ; « tuée » = un test nommé en échec sur une assertion (`kesh-gate-logs/s38-11d/revue-p4-mut/`, `run.sh`, `mut.log`, `out-*.log`). Huit mutations, huit tuées :
  - L1-filtre (`paypal-review.ts`, filtre d'exclusion → `true`) : « comptes de retrait : les comptes exclus … ne sont pas offerts », `expected [ 1, 2 ] to deeply equal [ 1 ]` ;
  - L1-page (page sans `ownLedgerIds`) : « retrait : le compte du plan du compte PayPal n'est pas offert », `expected [ '', '20', '31' ] to not include '31'` ;
  - L2-import (sans `importing` sur « Accepter ») ; L2-tout et L2-aucun (sans `disabled={accepting}` sur « Tout cocher » / « Tout décocher ») : « pendant l'import : … grisé ; pendant l'acceptation : … », `expected false to be true` ;
  - L4-casse (clé sans normalisation) : « paypalAccountLabels : sans tenir compte de la casse », `expected [ 'paypal WWW', 'PayPal www' ] to deeply equal [ 'PayPal www' ]` ;
  - P4-2-accents (retrait des diacritiques neutralisé) : « … sans tenir compte des accents (Zürich = Zurich) », `expected [ 'paypal zurich', 'PayPal Zürich' ] to deeply equal [ 'PayPal Zürich' ]` ;
  - P4-3-cases (cases sans `accepting`) : « pendant l'acceptation : sélecteurs et import désactivés … », `expected false to be true`.
- **P4-2** : `accountLabelKey` (`normalize('NFD')`, retrait des marques, minuscules) dans `provider.ts`, utilisée par `paypalAccountLabels` et par `ownLedgerIds` de la page ; commentaire corrigé ; test « Zürich » / « Zurich ».
- **P4-3** : cases par ligne grisées pendant `accepting`, assertion ajoutée au test existant de l'acceptation.
- **Gates** (`kesh-gate-logs/s38-11d/front-7-*.log`) : `check`, `lint-i18n-ownership`, `build` verts ; Vitest **4946/4946** (287 fichiers). Aucun Rust modifié, aucune spec E2E ajoutée.
- Prompt de la passe ciblée P5 : `kesh-gate-logs/s38-11d/revue-p5-prompt.md`.

### Remédiation de la revue P5 (M1, L1, L3 ; L2 sans suite) — dernier commit de code `aa643813`

- **M1** : test de page « retrait : un libellé de compte qui ne diffère que par la casse et les accents est le même compte » (compte 1 « PayPal Zürich » au compte du plan 31, compte 2 « paypal zurich » au compte du plan 20 : ni 31 ni 20 offerts en destination) ; mutation `P5-M1-normalisation-page` (`ownLedgerIds` revenu à `toLowerCase()`) tuée par ce test : `expected [ '', '20' ] to not include '20'`.
- **L1** : `run.sh` lance et consigne le témoin non muté en tête de `mut.log` : 3 fichiers, **41 tests verts** (40 + le test neuf). Neuf mutations, neuf tuées sur une assertion nommée (`kesh-gate-logs/s38-11d/revue-p5-mut/`).
- **L3** : phrase au manuel utilisateur (`user-manual.tex`, « Comptes PayPal ») : la casse et les accents ne distinguent pas deux libellés. PDF compilé dans une copie hors dépôt, phrase retrouvée (`pdftotext`) ; aucun PDF commité.
- **L2** (limite acceptée, déjà écrite) : la clé fond casse et diacritiques, non les équivalences de `utf8mb4_unicode_ci` (`ß` = `ss`, `æ` = `ae`, ligatures) ; effet : deux options au sélecteur, refus propre du serveur, aucune écriture fausse.
- **Gates** (`kesh-gate-logs/s38-11d/front-8-*.log`) : `check`, `lint-i18n-ownership`, `build` verts ; Vitest **4947/4947** (287 fichiers). Aucun Rust, aucun E2E.
- Prompt de la passe ciblée P6 : `kesh-gate-logs/s38-11d/revue-p6-prompt.md`.


### Completion Notes

- Tests ajoutés (de `a02f3372` à `5d04b925`, déclarations `it`/`test` recomptées par fichier) : **60 Vitest**
  (`provider.test.ts` 7 dont un `it.each` de six sites, `rule-filters.test.ts` 11, `paypal-labels.test.ts` 9 dont un
  `it.each` de 15 codes, `paypal-review.test.ts` 7, `bank-accounts-page-paypal.test.ts` 12, `paypal-page.test.ts` 8,
  `LinkSettlementModal.test.ts` 2, `bank-accounts.api`, `BankImportUpload`, `ManualStatementForm`,
  `payment-batches-page` 1 chacun), **5 E2E**, **1 test d'intégration Rust**.
- Prompt de la revue P1 : `kesh-gate-logs/s38-11d/revue-p1-prompt.md`.

### File List

- `crates/kesh-api/src/routes/companies.rs`
- `crates/kesh-api/tests/companies_e2e.rs`
- `crates/kesh-i18n/locales/de-CH/messages.ftl`
- `crates/kesh-i18n/locales/en-CH/messages.ftl`
- `crates/kesh-i18n/locales/fr-CH/messages.ftl`
- `crates/kesh-i18n/locales/it-CH/messages.ftl`
- `docs/api-external.md`
- `docs/manual/fr/user-manual.tex`
- `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm-credit-card.test.ts`
- `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.test.ts`
- `frontend/src/lib/features/bank-accounts/bank-accounts.api.test.ts`
- `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts`
- `frontend/src/lib/features/bank-accounts/bank-name-inventory.test.ts`
- `frontend/src/lib/features/bank-accounts/format.ts`
- `frontend/src/lib/features/bank-accounts/provider.test.ts`
- `frontend/src/lib/features/bank-accounts/provider.ts`
- `frontend/src/lib/features/bank-import/BankImportUpload.svelte`
- `frontend/src/lib/features/bank-import/BankImportUpload.test.ts`
- `frontend/src/lib/features/bank-import/ManualStatementForm.svelte`
- `frontend/src/lib/features/bank-import/ManualStatementForm.test.ts`
- `frontend/src/lib/features/homepage/HomepageCards.test.ts`
- `frontend/src/lib/features/homepage/homepage.test.ts`
- `frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte`
- `frontend/src/lib/features/paypal/paypal-labels.test.ts`
- `frontend/src/lib/features/paypal/paypal-labels.ts`
- `frontend/src/lib/features/paypal/paypal-review.test.ts`
- `frontend/src/lib/features/paypal/paypal-review.ts`
- `frontend/src/lib/features/paypal/paypal.api.ts`
- `frontend/src/lib/features/paypal/paypal.types.ts`
- `frontend/src/lib/features/reconciliation/LinkSettlementModal.svelte`
- `frontend/src/lib/features/reconciliation/LinkSettlementModal.test.ts`
- `frontend/src/lib/features/reconciliation/reconciliation.types.ts`
- `frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte`
- `frontend/src/lib/features/reconciliation/rules/RuleFormModal.test.ts`
- `frontend/src/lib/features/reconciliation/rules/RulesList.svelte`
- `frontend/src/lib/features/reconciliation/rules/RulesList.test.ts`
- `frontend/src/lib/features/reconciliation/rules/rule-filters.test.ts`
- `frontend/src/lib/features/reconciliation/rules/rule-filters.ts`
- `frontend/src/lib/features/reconciliation/rules/rules.api.ts`
- `frontend/src/lib/features/reconciliation/rules/rules.types.ts`
- `frontend/src/lib/features/settings/settings.types.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts`
- `frontend/src/routes/(app)/+layout.svelte`
- `frontend/src/routes/(app)/bank-accounts/+page.svelte`
- `frontend/src/routes/(app)/bank-accounts/bank-accounts-page-credit-card.test.ts`
- `frontend/src/routes/(app)/bank-accounts/bank-accounts-page-paypal.test.ts`
- `frontend/src/routes/(app)/bank-accounts/bank-accounts-page.test.ts`
- `frontend/src/routes/(app)/payment-batches/+page.svelte`
- `frontend/src/routes/(app)/payment-batches/payment-batches-page.test.ts`
- `frontend/src/routes/(app)/reconciliation/+page.svelte`
- `frontend/src/routes/(app)/reconciliation/paypal/+page.svelte`
- `frontend/src/routes/(app)/reconciliation/paypal/paypal-page.test.ts`
- `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte`
- `frontend/tests/e2e/paypal-import.spec.ts`
- `_bmad-output/implementation-artifacts/38-11d-ecrans-manuel-e2e-paypal.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `frontend/src/routes/(app)/reconciliation/ReconciliationProposalsStub.test.svelte`
- `frontend/src/routes/(app)/reconciliation/reconciliation-page.test.ts`
- `crates/kesh-api/tests/closing_checks_e2e.rs`
- `crates/kesh-api/tests/paypal_reconciliation_e2e.rs`
- `crates/kesh-db/src/repositories/closing_checks.rs`
- `crates/kesh-reconciliation/src/provider_review.rs`
- `frontend/src/lib/features/fiscal-years/FiscalYearChecks.svelte`
- `frontend/src/lib/features/fiscal-years/FiscalYearChecks.test.ts`
- `frontend/src/lib/features/fiscal-years/fiscal-years.types.ts`
- `frontend/src/lib/features/invoices/SettleInvoiceDialog.test.ts`
- `frontend/src/routes/(app)/supplier-invoices/[id]/supplier-settlement-page.test.ts`
