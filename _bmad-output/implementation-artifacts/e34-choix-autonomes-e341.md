# Epic 34 — choix autonomes du groupe e341 (34-1, 34-2, 34-11a, 34-11b)

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

## 34-2 — le solde d'ouverture avec le compte bancaire (#533)

**C-e341-6 — une portée du complément qui lève `NoEntries` pour un compte bancaire.** Contexte : le complément (25-7) refuse une société sans
écriture ; c'est le cas nominal d'une installation neuve. Retenu : `ComplementScope::SingleBankLedger` lève ce seul refus quand un premier exercice
existe ; l'écran *Soldes de départ* garde `Screen`. Conséquence assumée : après une telle écriture, la génération générale est refusée et l'écran
passe en mode *Compléter*. Autres : refuser et renvoyer à *Soldes de départ* ; appeler la génération générale (second mécanisme). Réversible : oui.
⚠️ Point que Guy peut vouloir trancher autrement : voir le rapport.

**C-e341-7 — un montant positif et un sens *Avoir* / *Dette*.** Présélection par le type du compte lié. Autre : un montant signé. Réversible : oui.

**C-e341-8 — compte déjà mouvementé : refus du serveur, création annulée avec lui.** L'écran ne devine pas. Réversible : oui.

**C-e341-9 — atomicité** : compte et écriture dans la même transaction. Autre : deux requêtes (un compte sans son solde après un refus).

**C-e341-10 — un compte existant : `POST /bank-accounts/{id}/opening-balance`.** Couvre la « liaison » de #533 sans greffer une écriture sur le
`PUT` (remplacement, rejouable) ni sur le `PATCH`. Réversible : oui.

**C-e341-11 — libellé « Solde d'ouverture — {compte} »**, clé neuve, langue comptable. Autre : le libellé du complément. Réversible : oui.

**C-e341-12 — pas de solde à l'assistant** : aucun exercice à l'étape 6. Réversible : oui, une story future.

## 34-11a — la carte de crédit comme compte (#972)

**C-e341-13 — colonne `account_kind` (`bank`, `credit_card`).** Le type se déclare, il ne se déduit pas d'un IBAN nul. Réversible : par migration.

**C-e341-14 — migration breaking, `min_required` à 0.17.6.** `MODIFY COLUMN iban NULL` (P3) ; une 0.17.5 décoderait mal un `iban` nul et ignorerait
`account_kind` à l'import d'une sauvegarde. Autre : `breaking-skip-bump` (justification fausse : la 0.17.5 lit `iban: String`). Réversible : non
une fois publiée — c'est l'objet du bump.

**C-e341-15 — quatre chiffres exactement, et garde de Luhn** sur nom, description et chiffres, pour les deux types. Refus, jamais de troncature.
Autres : tronquer aux quatre derniers (le numéro complet transiterait et pourrait se loguer) ; pas de garde (#972 : « aucun numéro complet n'est
accepté »). Réversible : oui.

**C-e341-16 — une carte est toujours liée à un passif** (création, `PUT`, `PATCH`). Autre : lien facultatif (`BANK_ACCOUNT_NOT_CONFIGURED` à la
première réconciliation, le défaut même de #966). Réversible : oui.

**C-e341-17 — invariants au schéma (`CHECK`) et type immuable.** Réversible : par migration.

**C-e341-18 — refus nommés : `PAYMENT_BATCH_ACCOUNT_HAS_NO_IBAN` (lot), `BANK_IMPORT_CAMT_NEEDS_IBAN` (CAMT).** Le lot est refusé à sa création, et
la génération refuse au lieu d'un 500. Réversible : oui.

**C-e341-19 — l'assistant reste bancaire** (IBAN obligatoire, compte principal). Réversible : oui.

**C-e341-20 — dérogation à la règle de splitting** : plus de cinq modules, mais un découpage livrerait un `iban` nul sans ses gardes (500 du lot,
accueil qui plante). Ordre de commits imposé : tolérance d'abord, création ensuite. Autre : 34-11a-i (tolérance) et 34-11a-ii (création), que
l'orchestrateur peut préférer — les tâches T3 et T4 en sont déjà la frontière. Réversible : oui.

**C-e341-21 — affichage « •••• 1234 »**, une fonction de `format.ts`. Réversible : oui.

## 34-11b — relevé, réconciliation, prélèvement (#972)

**C-e341-22 — condition 4 du rattachement : au plus une transaction par compte du grand livre.** Contexte, établi au code : la condition « l'écriture
n'est rapprochée d'aucune transaction » rend le virement banque → carte irréconciliable côté carte. Autres : statut « virement interne » sans écriture
(carte non réconciliée) ; deux écritures et un compte de transit (double saisie). Réversible : oui, le code ; les liens posés resteraient.

**C-e341-23 — l'annulation d'un côté ne défait que ce côté** (`unlink_only`), la contre-passation au dernier. Autre : refus (impasse : chaque côté
bloquerait l'autre). Réversible : oui.

**C-e341-24 — aucun appariement d'office**, la liste des candidats en tient lieu ; la proposition automatique est tracée en issue. Réversible : oui.

**C-e341-25 — CSV : profils existants tels quels** ; l'inversion d'une colonne unique demande une colonne et un bump : issue. Réversible : oui.

**C-e341-26 — PDF : rien n'est joint** (aucun moyen n'existe) ; deux issues (joindre, lire). Réversible : oui.

**C-e341-27 — pas de règle de rapprochement par compte.** Réversible : oui.

**C-e341-28 — la condition neuve vaut pour tout virement entre deux comptes de Kesh**, pas seulement la carte. Réversible : oui.
