# Epic 34 — choix autonomes de la story 34-11a2 (clé e34a2, carte de crédit, #972)

*Registre du développement (Opus 5.5, 2026-11-12). Les choix de la fiche sont au registre du groupe,
`e34-choix-autonomes-e341.md` (C-e341-15, 16, 19, 21, 30 à 33, 39 à 43) ; ceux-ci les complètent sans les contredire.*

**C-e34a2-1 — la base de la branche ne porte ni la 34-1 ni la 34-2.** Contexte : la branche part de `27ca0bd9` (34-11a1), elle-même
partie de `07f714a2` ; `linkableLedgerAccounts` (34-1, C-e341-5, C-e341-33) et le solde d'ouverture avec le compte bancaire (34-2)
n'y existent pas. Retenu : (a) le filtre « passif seulement » est écrit une fois, `admitsLedgerType(kind, accountType)`
(`frontend/src/lib/features/bank-accounts/credit-card.ts`), appelé par la page et par `BankAccountJournalLinkForm` ; **à l'intégration**,
`linkableLedgerAccounts(accounts, excluded, { liabilityOnly })` de la 34-1 doit le recevoir (ou l'appeler) et les deux listes de la page
passer par lui ; (b) le sens *Dette* présélectionné du solde d'ouverture d'une carte (AC-7) **n'est pas implémenté** ici : à faire à
l'intégration de la 34-2 (le manuel l'annonce déjà, § *Une carte de crédit*, *Solde d'ouverture*) ; (c) le « complément de la 34-2 »
(T4 : relecture du type avant lui) n'existe pas ici : la relecture du type se place avant la démotion et l'`INSERT`, et la 34-2 devra placer
son complément **après** `recheck_ledger_type_in_tx` ; (d) l'assistant de la 34-1 reçoit une description : la garde de Luhn ne couvre ici que
le nom (`onboarding::set_bank_account`) ; l'intégration ajoute `refuse_card_number("description", …)` (commentaire au site). Autres :
fusionner la 34-1 et la 34-2 dans la branche (hors du mandat : seule la 34-11a1 se fusionne, sur demande). Réversible : oui.

**C-e34a2-2 — où se relit le type du compte lié.** Création et `PUT` : `recheck_ledger_type_in_tx` juste après les réglages
(`claims_for_target` ou la lecture des créances), avant la démotion du principal et la ligne du compte bancaire — le type du compte
bancaire est connu (corps au `POST`, lecture sur le pool au `PUT`, C-e341-43). `PATCH` : le type du compte bancaire n'est connu que sous son
`FOR UPDATE` ; le type du compte lié est lu sous verrou partagé **avant** cette ligne (`locked_ledger_type`), puis contrôlé contre
`before.account_kind` **après** `set_journal_account_id_for_company`, l'erreur annulant la transaction (`conclure`). Les deux ordres de verrous
restent `réglages → accounts → bank_accounts`. Autres : lire le compte bancaire sur le pool avant le `PATCH` (aurait déplacé l'ordre des
erreurs du `PATCH`, `404` du compte bancaire avant `404` du compte lié). Réversible : oui.

**C-e34a2-3 — le retypage : garde avant celle des écritures, comptes clos compris.** `first_linked_bank_account_refusing` lit les comptes
bancaires liés `archived = FALSE` `LOCK IN SHARE MODE`, clos compris (un compte clos se rouvre), après le court-circuit no-op et la
garde du rôle, avant `AccountHasEntries` : une confirmation de retypage ne lève pas ce refus, autant le dire d'abord. Réversible : oui.

**C-e34a2-4 — codes.** Une carte **sans** compte lié rend le même code qu'une carte liée à un non-passif,
`CREDIT_CARD_LEDGER_MUST_BE_LIABILITY`, `details.accountId = null` : un seul motif pour l'écran. Des quatre chiffres mal formés rendent
`400 VALIDATION_ERROR` (l'AC-2 ne nomme pas de code). Un compte bancaire lié à un type non admis garde `INVALID_ACCOUNT_TYPE`, y compris
à la relecture sous verrou. Réversible : oui.

**C-e34a2-5 — l'écran.** Choix du type par deux boutons radio en tête du formulaire de création (`form-kind-*`) ; changer de type
retire un compte lié que le nouveau type n'admet pas et décoche *principal*. Une carte : libellé *Émetteur de la carte*, champs IBAN,
QR-IBAN et *principal* absents du DOM, compte lié libellé *Compte de dette lié*, aide propre. Le payload d'un **compte bancaire est celui
de la 0.17.5** (ni `accountKind` ni `cardLast4`) ; celui d'une carte n'a ni IBAN ni `isPrimary`, et le `PUT` d'une carte n'envoie ni IBAN
ni type (points hérités E5/B7). Le formulaire de lien (`PATCH`) ne propose que des passifs pour une carte et n'offre pas *Délier*.
Contrôles d'écran : quatre chiffres ou rien, compte lié obligatoire ; le serveur reste l'autorité. Réversible : oui.

**C-e34a2-6 — règlement client : les cartes écartées avant le filtre du compte débiteurs** (`SettleInvoiceDialog.svelte`), pour que le
message « le seul compte lié est le compte débiteurs » ne compte pas les cartes. Réversible : oui.

**C-e34a2-7 — tests dans des fichiers neufs** (`crates/kesh-api/tests/credit_card_e2e.rs`, `crates/kesh-db/tests/credit_card_repository.rs`,
`bank-accounts-page-credit-card.test.ts`, `credit-card.test.ts`, `bank-accounts-credit-card.spec.ts`) : l'AC-4 veut les tests de
`bank_accounts_e2e.rs` verts **sans modification**, et des fichiers neufs n'entrent pas en conflit avec la 34-1 et la 34-2, qui touchent les
fichiers de test existants. Les huit littéraux de `bank_accounts_repository.rs` sont modifiés, comme la fiche l'exige (T4). Réversible : oui.

**C-e34a2-8 — points hérités.** E5/B7 : traité (payload d'une carte sans IBAN, Vitest). B5 : pas de migration (C-e341-29) ; la promesse
est bornée au doc-comment de `BankAccount::iban` (l'invariant tient par le type ; un futur type sans IBAN devra poser le `CHECK`) — issue
demandée pour la dette. E6 : test Vitest qui montre le refus `BANK_IMPORT_CAMT_NEEDS_IBAN` affiché après un dépôt `.xml`. E3 : apostrophe
droite dans la clé fr-CH et ses deux replis Svelte, assertion Vitest alignée. B3 : `CARD_LAST4_SEPARATOR = ' '`. Réversible : oui.

**C-e34a2-9 — écart à la consigne Edit (B3).** L'outil Edit n'a pas su apparier le littéral `' '` (U+00A0 entre apostrophes) : deux essais
rendus « String to replace not found ». Le remplacement s'est fait par un script Python d'une ligne, contrôlant que la chaîne exacte
n'apparaissait qu'une fois (`assert s.count(old) == 1`). C'est ce que Guy veut savoir : l'outil ne voit pas une espace insécable saisie.
