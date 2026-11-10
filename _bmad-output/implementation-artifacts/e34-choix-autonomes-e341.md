# Epic 34 — choix autonomes du groupe e341 (34-1, 34-2, 34-11a1, 34-11a2, 34-11b)

*Registre tenu en autonomie (Opus 5.5), fiches du 2026-11-10, commit de référence `a75835db`. Chaque choix est réversible à la revue de la
fiche ; aucun n'est encore écrit dans le code. Mise à jour par union.*

## 34-1 — l'assistant pose la description et le compte lié (#966)

**C-e341-1 — `upsert_primary_in_tx` reçoit un second paramètre, `NewBankAccount` ne change pas.**
Contexte : `NewBankAccount` est construit 68 fois dans 33 fichiers, surtout des tests. Retenu : `PrimaryDetails { description,
journal_account_id }`, `upsert_primary` passant le défaut. Autres : élargir `NewBankAccount` (68 sites) ; une fonction parallèle (doublon).
Réversible : oui, interne.

**C-e341-2 — les contrôles de la création sont réemployés, non recopiés.** `normalize_description` et `validate_journal_account_id` passent
`pub(crate)`. Autre : un module partagé neuf (plus de déplacement pour le même effet). Réversible : oui.

**C-e341-3 — à l'assistant, refus d'un compte de rôle `Receivable` ou `Payable`.** Contexte : les réglages de facturation naissent à la
finalisation, qui désigne ces comptes ; à l'étape 6 aucun compte de créance n'est désigné, et un lien vers le futur compte débiteurs passerait,
puis deviendrait l'état que la 15-6c interdit. Retenu : lire les rôles (explicites, non le numéro) et réemployer `refuse_if_ledger_is_claim_account`,
même code d'erreur. Autres : ne rien faire (trou ouvert par la story) ; contrôler à la finalisation (refus tardif, l'étape bancaire déjà passée).
Réversible : oui.

**C-e341-4 — la présélection 1020 vit dans l'écran seulement.** Le serveur ne connaît aucun numéro ; la présélection lit le numéro `1020` dans la
liste déjà filtrée (actif, imputable, actif ou passif, hors créance). Conforme à la décision de Guy et à la 14-3a. Réversible : oui.

**C-e341-5 — le filtre des comptes liables s'écrit une fois** (`linkableLedgerAccounts`, `format.ts`), pour les deux écrans. Réversible : oui.
*(Validation P1, axe 8 : il reçoit une option `liabilityOnly`, C-e341-33.)*

**C-e341-34 — à l'assistant, champ absent = inchangé, `null` = effacé** (validation P1, 1.1). Contexte : l'`UPDATE` d'`upsert_primary_in_tx` écrit
chaque colonne qu'il nomme ; un corps 0.17.5 rejoué sur un compte principal qui porte déjà description et lien les aurait effacés. Retenu : double
option (`Option<Option<_>>`), l'`UPDATE` ne nomme que les champs présents. Autre : « absent = effacé » (casse la promesse « corps 0.17.5 inchangé »).
Réversible : oui.

## 34-2 — le solde d'ouverture avec le compte bancaire (#533)

**C-e341-6 — une portée du complément qui lève `NoEntries` pour un compte bancaire.** Contexte : le complément (25-7) refuse une société sans
écriture ; c'est le cas nominal d'une installation neuve. Retenu : `ComplementScope::SingleBankLedger` lève ce seul refus quand un premier exercice
existe ; l'écran *Soldes de départ* garde `Screen`. Conséquence assumée : après une telle écriture, la génération générale est refusée et l'écran
passe en mode *Compléter*. *(Validation P1, 2.2 : gardé sur arbitrage de l'orchestrateur ; la phrase de la conséquence est exigée à l'écran et au
manuel, AC-7 et AC-9 de la 34-2.)* Autres : refuser et renvoyer à *Soldes de départ* ; appeler la génération générale (second mécanisme). Réversible : oui.
⚠️ Point que Guy peut vouloir trancher autrement : voir le rapport.

**C-e341-7 — un montant positif et un sens *Avoir* / *Dette*.** Présélection par le type du compte lié. Autre : un montant signé. Réversible : oui.

**C-e341-8 — compte déjà mouvementé : refus du serveur, création annulée avec lui.** L'écran ne devine pas. Réversible : oui.

**C-e341-9 — atomicité** : compte et écriture dans la même transaction. Autre : deux requêtes (un compte sans son solde après un refus).

**C-e341-10 — un compte existant : `POST /bank-accounts/{id}/opening-balance`.** Couvre la « liaison » de #533 sans greffer une écriture sur le
`PUT` (remplacement, rejouable) ni sur le `PATCH`. Réversible : oui.

**C-e341-11 — libellé « Solde d'ouverture — {compte} »**, clé neuve, langue comptable. Autre : le libellé du complément. Réversible : oui.

**C-e341-12 — pas de solde à l'assistant** : aucun exercice à l'étape 6. Réversible : oui, une story future.

## 34-11a1 et 34-11a2 — la carte de crédit comme compte (#972)

*L'ancienne 34-11a a été découpée à la validation P1 (arbitrage de l'orchestrateur) : 34-11a1 (schéma, tolérance) et 34-11a2 (création). Les
choix C-e341-13 à 21 restent numérotés tels quels ; chacun dit désormais quelle moitié il gouverne.*

**C-e341-13 (34-11a1) — colonne `account_kind` (`bank`, `credit_card`).** Le type se déclare, il ne se déduit pas d'un IBAN nul. Réversible : par migration.

**C-e341-14 (34-11a1) — migration breaking, `min_required` à 0.17.6.** `MODIFY COLUMN iban NULL` (P3) ; une 0.17.5 décoderait mal un `iban` nul et ignorerait
`account_kind` à l'import d'une sauvegarde. Autre : `breaking-skip-bump` (justification fausse : la 0.17.5 lit `iban: String`). Réversible : non
une fois publiée — c'est l'objet du bump.

**C-e341-15 (34-11a2) — quatre chiffres exactement, et garde de Luhn** sur nom et description, pour les deux types. Refus, jamais de troncature.
*(Validation P1, 3.8 : pas sur les quatre chiffres, sans objet ; appelée par `validate_bank_input` et par l'assistant ; forme reconnue = 13 à 19
chiffres contigus ou groupes de quatre séparés par espace ou tiret, puis Luhn — une date ou une référence n'en est pas.)*
Autres : tronquer aux quatre derniers (le numéro complet transiterait et pourrait se loguer) ; pas de garde (#972 : « aucun numéro complet n'est
accepté »). Réversible : oui.

**C-e341-16 (34-11a2) — une carte est toujours liée à un passif** (création, `PUT`, `PATCH`). Autre : lien facultatif (`BANK_ACCOUNT_NOT_CONFIGURED` à la
première réconciliation, le défaut même de #966). Réversible : oui.

**C-e341-17 (34-11a1 pour le schéma, 34-11a2 pour l'immuabilité) — invariants au schéma (`CHECK`) et type immuable.** Réversible : par migration.

**C-e341-18 (34-11a1) — refus nommés, fondés sur `iban IS NULL` : `PAYMENT_BATCH_ACCOUNT_HAS_NO_IBAN` (lot), `BANK_IMPORT_CAMT_NEEDS_IBAN` (CAMT).** Le lot est refusé à sa création, et
la génération refuse au lieu d'un 500. Réversible : oui.

**C-e341-19 (34-11a1, 34-11a2) — l'assistant reste bancaire** (IBAN obligatoire, compte principal). Réversible : oui.

**C-e341-20 — ~~dérogation à la règle de splitting~~ — REMPLACÉ** (validation P1, 3.1 : l'alternative proposée elle-même prouvait un état
intermédiaire sans `iban` nul atteignable ; l'orchestrateur a découpé en 34-11a1 et 34-11a2). Texte d'origine : : plus de cinq modules, mais un découpage livrerait un `iban` nul sans ses gardes (500 du lot,
accueil qui plante). Ordre de commits imposé : tolérance d'abord, création ensuite. Autre : 34-11a-i (tolérance) et 34-11a-ii (création), que
l'orchestrateur peut préférer — les tâches T3 et T4 en sont déjà la frontière. Réversible : oui.

**C-e341-21 (34-11a1 pour la fonction, 34-11a2 pour l'usage) — affichage « •••• 1234 »**, une fonction de `format.ts`. Réversible : oui.

## 34-11b — relevé, réconciliation, prélèvement (#972)

**C-e341-22 — condition 4 du rattachement : au plus une transaction par compte du grand livre.** *(Validation P1, 4.4 : précisée pour ne pas se
fier au seul lien actuel, modifiable — refusée si l'autre transaction est du même compte bancaire, d'un compte lié à `NULL` ou au même grand
livre, ou d'un compte dont le grand livre ne porte aucune ligne de l'écriture ; au plus deux transactions par écriture.)* Contexte, établi au code : la condition « l'écriture
n'est rapprochée d'aucune transaction » rend le virement banque → carte irréconciliable côté carte. Autres : statut « virement interne » sans écriture
(carte non réconciliée) ; deux écritures et un compte de transit (double saisie). Réversible : oui, le code ; les liens posés resteraient.

**C-e341-23 — l'annulation d'un côté ne défait que ce côté** (`unlinkOnly`), la contre-passation au dernier. *(Validation P1 : 4.6, le retrait seul
n'est refusé ni par un exercice clos ni par un lettrage clos, le compte clos refusant toujours ; 4.7, `unlinkOnly` en camelCase ; 4.2,
`reversalJournalEntryId` nullable dans la réponse et la trace.)* Autre : refus (impasse : chaque côté
bloquerait l'autre). Réversible : oui.

**C-e341-24 — aucun appariement d'office**, la liste des candidats en tient lieu ; la proposition automatique est tracée en issue. Réversible : oui.

**C-e341-25 — CSV : profils existants tels quels** ; l'inversion d'une colonne unique demande une colonne et un bump : issue. Réversible : oui.

**C-e341-26 — PDF : rien n'est joint** (aucun moyen n'existe) ; deux issues (joindre, lire). Réversible : oui.

**C-e341-27 — pas de règle de rapprochement par compte.** Réversible : oui.

**C-e341-28 — la condition neuve vaut pour tout virement entre deux comptes de Kesh**, pas seulement la carte. Réversible : oui.

## Ajouts de la validation P1 (2026-11-10)

**C-e341-29 (34-11a1) — une seule migration pour les deux moitiés de la carte.** La 34-11a2 n'en porte pas ; deux migrations breaking
successives feraient deux montées. Seule migration de l'epic (la 34-5 n'en porte pas, C-e343-1). Réversible : oui, avant publication.

**C-e341-30 (34-11a2) — `PUT` d'une carte : `accountKind` absent = type courant**, présent et différent = `BANK_ACCOUNT_KIND_IMMUTABLE` ; une carte
refuse IBAN, QR-IBAN, `isPrimary`. Autre : absent = `bank` (un client 0.17.5 transformerait une carte, ou se verrait réclamer un IBAN). Réversible : oui.

**C-e341-31 (34-11a2) — retypage refusé d'un compte lié à un compte bancaire non archivé** vers un type que le lien n'admet plus (carte : hors passif ;
compte bancaire : hors actif ou passif), `409 ACCOUNT_LINKED_TO_BANK_ACCOUNT`. Limite voisine corrigée (vraie aussi des comptes bancaires
aujourd'hui). Autre : avertir seulement ; une issue. Réversible : oui.

**C-e341-32 (34-11a2) — une carte est proposée au paiement d'une facture fournisseur, pas au règlement d'une facture client.** Réversible : oui.

**C-e341-33 (34-1, 34-11a2) — `linkableLedgerAccounts` reçoit `liabilityOnly`.** Réversible : oui.

**C-e341-35 (34-11b) — une seule définition de l'empêchement par « l'autre côté »**, appelée par la garde d'annulation, la vue d'annulation, les
candidates et la garde de règlement (validation P1, 4.1, HIGH). Autre : corriger le seul `cancel_in_tx` (l'écran aurait annoncé un faux obstacle et
refusé l'annulation). Réversible : oui.

**C-e341-36 (34-11b) — la 34-11b corrige les textes devenus faux** (manuel `:2915-2921`, `:2991`, AC-8 de la 34-3, cas (b) de la 34-4), arrivant
après elles (ordre 34-3 → 34-4 → 34-11b), sans modifier les fiches d'un autre groupe. Arbitrage de l'orchestrateur. Réversible : oui.

**C-e341-37 (34-11b) — #823 en partie** : la 34-11b résout le virement entre comptes propres, non le lot `pain.001` de plusieurs factures ni le
paiement groupé ; `refs #823` à la PR, commentaire à demander. Réversible : oui.

## Ajouts et précisions de la validation P2 (2026-11-10)

**C-e341-15 (34-11a2), précisé** — la garde de Luhn est **gardée et restreinte** (validation P2, 4.1 et 4.4 ; arbitrage de l'orchestrateur : elle ne
refuse jamais un IBAN, une référence QR ni un numéro de téléphone) : suite entière, bornée par ce qui n'est ni chiffre, ni lettre, ni séparateur ;
groupements des réseaux seuls (contigu, 4-4-4-4, 4-4-4-4-3, 4-6-5 Amex, 4-6-4 Diners) ; 13 à 19 chiffres ; préfixe IIN cohérent avec la longueur ;
puis Luhn ; tests négatifs construits pour passer Luhn. **Repli** : si une borne ne tient pas simplement, la garde est retirée du nom et de la
description (la décision de Guy porte sur le champ du numéro), écrit au Change Log. Réversible : oui.

**C-e341-35 (34-11b), récrit** — la décision « autre côté légitime » se prend **en tête**, avant les rangs 1 à 5 ; elle reçoit la transaction concernée,
liée ou prospective ; `None` garde son sens strict (écrans de facture, dévalidation fournisseur). Validation P2, 5.1 et 5.2. Réversible : oui.

**C-e341-38 (34-2) — le complément avant la démotion et l'`INSERT`** (validation P2, 2.1) : la démotion relit l'ancien principal sans verrou et
ouvrirait l'instantané trop tôt ; le complément n'a pas besoin de la ligne `bank_accounts`. Autre : verrouiller la relecture de la démotion (fonction
partagée avec le `PUT`). Réversible : oui.

**C-e341-39 (34-11a1, 34-11a2) — `NewBankAccount.iban` reste `String`** ; la carte lie son IBAN (`NULL`) à part, comme la description (validation P2,
3.2). Autre : `Option<String>` (68 sites). Réversible : oui.

**C-e341-40 (34-11a2) — le verrou du type** : retypage `FOR UPDATE` en tête de `accounts::update`, relecture du type `LOCK IN SHARE MODE` dans la
transaction du lien et de la création, route du plan rejouée (validation P2, 4.2). Autre : contrôle sur le pool seulement (course ouverte).
Réversible : oui.

**C-e341-41 (34-11a2) — règlement client sur une carte refusé au serveur** (`SETTLEMENT_ACCOUNT_IS_CREDIT_CARD`) ; l'acceptation d'une proposition
« facture » par la réconciliation d'une transaction de carte reste possible, angle mort écrit (validation P2, 4.3). Réversible : oui.
