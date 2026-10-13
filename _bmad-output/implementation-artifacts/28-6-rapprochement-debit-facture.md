# Story 28.6 : Le rapprochement refuse de régler une facture client par un débit

## Status

**ready-for-dev** — fiche créée le 2026-10-13 en autonomie (`bmad-create-story`, Opus 5.5), sur `5aad8b35`
(kickoff de l'Epic 28), branche `story/28-6`, worktree `/home/gcorbaz/devel/kesh-28-6` (port **3039**, bases
`kesh_286` / `kesh_e2e_286`). Choix consignés au registre `epic-28-choix-autonomes.md`, **C-28-6-1 à C-28-6-14**.
9 AC (AC1 à AC9). Validation P1 faite (lentilles R et F, Sonnet : 1 MEDIUM, 9 LOW dont 8 distincts, remédiés) ; passe ciblée P2 à
lancer : `28-6-validate-prompt-p2-ciblee.md` (Haiku).
PR : **`closes #482`**.

⚠️ **Développement en série** avec la 28-5 (#526) et la 28-7 (#548), qui touchent aussi
`crates/kesh-api/src/routes/reconciliation.rs` : la 28-6 se développe **après le merge de la 28-5** et se
rebase sur elle ; la 28-7 se rebase sur la 28-6. Régions touchées, § *Fichiers communs* des Dev Notes.

## Story

**As a** Comptable — ou une intégration par clé d'API — qui accepte des propositions de rapprochement,
**I want** qu'une transaction bancaire de **débit**, ou de montant **nul**, ne puisse jamais régler une facture
client, et que ce refus soit dit pour ce qu'il est,
**so that** une sortie d'argent ne soit jamais liée à une facture de vente, et qu'une intégration ne reçoive pas
une « erreur de la base de données » qui l'invite à réessayer indéfiniment.

Issue : [#482](https://github.com/guycorbaz/kesh/issues/482) (bug, **P2**), relevée en revue de code P1 de la
25-4-c (#420), lentille Edge Case Hunter. Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md`,
§ *28-6*. Cible : **v0.13.3**.

### Le défaut, au sol

`get_proposals` écarte les débits (`crates/kesh-api/src/routes/reconciliation.rs:648-652`,
`if tx.amount <= Decimal::ZERO`) : l'écran ne propose jamais de facture pour un débit, et
`ReconciliationProposals.svelte:217-221` ne soumet que la candidate affichée. Mais `POST
/api/v1/reconciliation/accept` est appelable directement — par une clé d'API `read-write` comme par un appel
forgé — avec un couple `(bankTransactionId, invoiceId)` arbitraire. Ni le pré-vol (`classify_batch_transaction`,
`:876`), ni la recharge sous le verrou du compte (`find_pending_by_id_for_account`, `:1567` — une lecture sans
`FOR UPDATE` ; le verrou est celui du compte bancaire, pris par `post_accept`), ni `accept_one_invoice`
(`:1553-2228`) ne regardent le **signe** de la transaction.

Une transaction de débit est aujourd'hui arrêtée par la garde de score (`:1729`, `score.total <= 0.0` →
`RECONCILIATION_SCORE_TOO_LOW`) **seulement si elle ne ressemble à rien** : le score de montant vaut 0 (le reste
dû est positif), mais le score de référence (`kesh-reconciliation/src/matching.rs:121-145`) et le score de contact
(`:148`) ne dépendent pas du signe. Un débit dont la référence contient le numéro de la facture (score 0,40) ou
dont la contrepartie porte le nom du client (0,10) **passe la garde de score**, puis celle du trop-perçu
(`classify_payment`, `kesh-db/src/repositories/invoice_settlements.rs:175-184` : `paid > raw_due` est faux pour un
montant négatif → `Ordinary`).

⚠️ **Prémisse de l'issue corrigée (C-28-6-1) — à constater au rouge, non à croire.** L'issue écrit que le
rapprochement « écrirait un règlement de montant négatif ». À la lecture, l'écriture est construite par
`settlement_journal_lines` (`invoice_settlements.rs:198-240`) avec `debit: paid` = le montant **négatif** de la
transaction ; son insertion viole `chk_jel_debit_credit_exclusive` / `chk_jel_debit_nonneg`
(`kesh-db/migrations/20260412000001_journal_entries.sql:46-47`), MariaDB rend 4025, `map_db_error` le traduit en
`DbError::CheckConstraintViolation` (`kesh-db/src/errors.rs:1469-1471`), et `project_error_to_failed_proposal`
(`reconciliation.rs:345`) le range dans son bras `_` : **`DATABASE_ERROR`**, avec le message brut de la contrainte
dans `details.message`, et le point de sauvegarde de la proposition est annulé. Rien ne serait donc écrit — mais
la seule garde est **une contrainte de la base**, et le refus se présente comme une panne : à l'écran,
« Erreur de la base de données : réessayez ; si le problème persiste, contactez le support »
(`reconciliation-failed-database-error`) ; pour une intégration, un code qui appelle le rejeu. Un montant **nul**
suit le même chemin (ligne `debit = 0, credit = 0`, refusée par `chk_jel_debit_credit_exclusive`).

**Cette lecture est une hypothèse** : la tâche T1 écrit d'abord les tests de l'AC2, les fait tourner **avant**
le correctif, et consigne au Dev Agent Record le code **observé**. Si l'observation contredit la lecture —
notamment si quoi que ce soit est écrit —, le défaut est un **P1** (erreur comptable) : le dire au rapport, sans
changer le correctif prescrit.

## Critères d'acceptation

**AC1 — Une garde de sens à l'entrée d'`accept_one_invoice`, en `FailedProposal` (C-28-6-2, C-28-6-3, C-28-6-4).**

- Une fonction **pure** (sans I/O), à côté d'`accept_one_invoice`, classe le montant de la transaction :

  | `bank_transaction.amount` | Résultat | `errorCode` | `details` |
  |---|---|---|---|
  | `< 0` (débit) | refus | **`RECONCILIATION_TRANSACTION_NOT_CREDIT`** (code neuf) | `{ "reason": "debit_transaction", "transactionAmount": <montant> }` |
  | `= 0` (dont `-0.00`, cas de la seule fonction pure — cf. *Pièges*) | refus | `RECONCILIATION_TRANSACTION_NOT_CREDIT` | `{ "reason": "zero_amount_transaction", "transactionAmount": <montant> }` |
  | `> 0` | accepté (suite inchangée) | — | — |

- `transactionAmount` a la **même forme** que le champ homonyme de `RECONCILIATION_OVERPAYMENT` (`:1859`,
  `serde_json::json!` sur le `Decimal`), pour qu'un client lise les deux de la même façon.
- Le code est écrit **au site de construction**, sous la forme `error_code: "RECONCILIATION_TRANSACTION_NOT_CREDIT".to_string()`
  — la forme que compte `grep -ohE 'error_code: "[A-Z_]+"'` (27 codes distincts aujourd'hui, **28** après) et que
  relève le commentaire de tête de `failed-proposal-label.test.ts`.
- **Place** : dans `accept_one_invoice`, **après** l'étape 4 (statut `pending`, `:1592-1598`) et **avant**
  l'étape 5 (lecture de la facture, `:1601`). Ordre des refus qui en résulte, pré-vol compris :
  1. **au pré-vol** de `post_accept` (`classify_batch_transaction`, `:876`), sur l'instantané, avant le verrou :
     `BANK_TRANSACTION_NOT_FOUND`, `RECONCILIATION_ALREADY_RECONCILED`, `RECONCILIATION_ALREADY_REJECTED` —
     un débit **déjà rejeté** rend donc `RECONCILIATION_ALREADY_REJECTED`, jamais le code neuf ;
  2. **dans `accept_one_invoice`**, sous le verrou du compte : `BANK_TRANSACTION_NOT_FOUND` (recharge) →
     `RECONCILIATION_ALREADY_RECONCILED` (statut) → **`RECONCILIATION_TRANSACTION_NOT_CREDIT`** →
     `INVOICE_NOT_FOUND` → `RECONCILIATION_INVOICE_NOT_ELIGIBLE` → `RECONCILIATION_SCORE_TOO_LOW` → …

  La garde s'applique à la transaction **rechargée sous le verrou du compte**, jamais à l'instantané du pré-vol.
- La garde s'écrit `amount <= Decimal::ZERO`, la forme de `get_proposals` (`:649`) : **une seule** règle « un
  règlement de facture exige un encaissement » au code, écrite deux fois (proposition et acceptation), et le
  doc-comment de la fonction le dit, en nommant `get_proposals`.
- Le doc-comment d'`accept_one_invoice` (`:1523-1552`) gagne une ligne : la garde, sa place, et pourquoi elle
  précède la facture (le refus ne dépend que de la transaction).

**AC2 — Les cas qui la prouvent, de bout en bout (tests d'intégration, `crates/kesh-api/tests/reconciliation_e2e.rs`).**

Chacun via `POST /api/v1/reconciliation/accept`, `200`, une proposition `type: "invoice"` dans `failed[]`, et
**rien d'écrit** — assertions explicites : nombre de lignes de `journal_entries`, de `journal_entry_lines` et
d'`invoice_settlements` de la société inchangé, compteur de numéros de l'exercice
(`journal_entry_number_sequences.next_number`) inchangé (cf. T1), transaction toujours `pending` avec `matched_entry_id` nul et `version` inchangée, facture
`paid_at` nul et `version` inchangée, aucune trace `reconciliation.accepted` ni `invoice.paid` /
`invoice.partially_settled`.

1. **Débit à référence reconnue** (le cas de l'issue) : facture `INV-DEBIT` validée de 100.00
   (`seed_validated_invoice`, qui pose une écriture de vente au débit du 1100), transaction de **−100.00** de
   référence `"INV-DEBIT"` et de contrepartie le nom du contact (`format!("{label} Client")`, `setup_company`).
   Attendu : `RECONCILIATION_TRANSACTION_NOT_CREDIT`, `details.reason == "debit_transaction"`,
   `details.transactionAmount` présent. **Avant le correctif**, le test est rouge ; le code observé est
   consigné (hypothèse : `DATABASE_ERROR`, cf. *Prémisse*).
2. **Montant nul à référence reconnue** : même montage, transaction de **0.00**. Attendu : même code,
   `reason == "zero_amount_transaction"`.
3. **`post_accept_filters_signed_amount` adapté** (`:2129`) : le débit de −100.00 à référence `"ANY-REF"` rend
   désormais **exactement** `RECONCILIATION_TRANSACTION_NOT_CREDIT` (la garde de sens précède celle du score).
   L'assertion actuelle, qui admet deux codes (`"RECONCILIATION_SCORE_TOO_LOW" | "BANK_TRANSACTION_NOT_FOUND"`),
   est remplacée par une égalité ; son doc-comment, qui décrit une « déviation spec » et le passage par le score,
   est réécrit.
4. **Un refus n'empêche pas les autres** : un lot de deux propositions — le débit du cas 1 et un encaissement
   valable sur une **autre** facture — rend l'encaissement dans `accepted[]` et le débit seul dans `failed[]`
   (§ *Pattern batch* du `CLAUDE.md`).
5. **Précédence sur la facture** : un débit proposé contre un `invoiceId` **inexistant** rend
   `RECONCILIATION_TRANSACTION_NOT_CREDIT`, non `INVOICE_NOT_FOUND`.

Les codes sont écrits **en dur** dans les tests, jamais relus d'une constante de production.

**AC3 — Tests unitaires de la fonction pure** (module `#[cfg(test)]` propre, en fin de `reconciliation.rs`) :
−100.00 → débit ; −0.01 → débit ; 0.00 → nul ; `dec!(-0.00)` (zéro négatif) → nul ; 0.01 → aucun refus ; `bank_transaction_id` reporté tel quel ; `details` exacts.
Le cas `-0.00` **asserte d'abord sa précondition** (`assert!(dec!(-0.00).is_sign_negative())`) : si la valeur
construite n'était pas un zéro négatif, le cas serait un doublon de 0.00 et passerait à vide. Ce cas n'existe
qu'ici : `bank_transactions.amount` est `DECIMAL(18,2)`, et la base ne rend jamais un zéro négatif — aucun test
d'intégration ne peut l'atteindre (C-28-6-12).

**AC4 — Ce qui ne change pas, et qui est vérifié.**

- `accept_one_split`, `accept_one_rule`, `POST /reconciliation/manual`, `POST /reconciliation/split` : **aucune
  garde de sens ajoutée**. Ils n'écrivent aucun règlement de facture (contrepartie choisie, écriture
  `banque ↔ compte`), et un débit y est légitime (une dépense, un remboursement) — `kesh-reconciliation/src/rules.rs:8`
  le dit déjà (« PAS hériter du sign filter 8-4 invoice »), `validate_split_balance` compare à `tx_amount.abs()`
  (`split.rs:171`). Leur refus du montant nul existe déjà et n'est pas touché (C-28-6-5) : pour `split` et `rule`
  dans `accept`, par proposition, en `VALIDATION_ERROR` + `details.reason = "zero_amount_transaction"` (`:2455`,
  `:2878`) ; pour `POST /reconciliation/manual` et `/split`, en `400 VALIDATION_ERROR` « zero_amount_transaction »
  (`:3664`, `:4114`).
- **Ce qui le tient** (C-28-6-14) — des tests existants, qui acceptent un **débit** et rougiraient si une garde de
  sens était ajoutée à la variante :
  - `accept_one_split` : `reconciliation_e2e.rs::accept_with_explicit_split_type_runs_split_flow` (−100.00,
    `accepted.len() == 1`) ;
  - `accept_one_rule` : `reconciliation_rules_e2e.rs::accept_with_rule_creates_journal_entry_and_increments_count`
    (−150.00, `accepted` à un élément, `failed` vide) ;
  - `post_manual` : `reconciliation_manual_e2e.rs::post_manual_creates_journal_entry_and_marks_transaction_reconciled`
    (−150.00, `200`) ;
  - `post_split` : `reconciliation_split_e2e.rs::split_creates_journal_entry_with_n_plus_1_lines` (montage
    `setup_split_ctx`, −10700.00, `200`).

  Ces quatre tests restent verts **sans modification** ; la revue de code vérifie qu'ils n'ont pas été touchés.
- Les tests existants d'acceptation d'un **encaissement** (montant positif) restent verts sans modification.
- `get_proposals` : inchangé.

**AC5 — L'écran sait nommer le refus (C-28-6-6).**

- `frontend/src/lib/features/reconciliation/failed-proposal-label.ts` : un `case
  'RECONCILIATION_TRANSACTION_NOT_CREDIT'` → clé `reconciliation-failed-transaction-not-credit`, repli français
  identique au catalogue. Un seul libellé pour les deux `reason`.
- `failed-proposal-label.test.ts` : la table `EXPECTED_KEYS` gagne l'entrée ; « couvre les 29 codes relevés » →
  **30** (`:68-69`), et le commentaire de tête (« les 27 littéraux … 29 codes », `:5-9`) passe à **28 littéraux,
  30 codes**, sa généalogie (« Story 15-12b : … ; Story 27-4 : … aussi ») gagnant « Story 28-6 :
  `RECONCILIATION_TRANSACTION_NOT_CREDIT` aussi ».
- Le **doc-comment de `failed-proposal-label.ts`** porte le même décompte, et passe aussi : « **Les 29 codes** … les
  27 littéraux » (`:9-10`) → **30 codes**, **28 littéraux** ; la phrase de la 27-4 (`:25-28`, « le littéral ajouté
  en dernier (ce qui porte le compte à 27) est `RECONCILIATION_ALREADY_REJECTED` ») n'est plus vraie telle quelle —
  elle devient « le littéral ajouté par la Story 27-4 (ce qui portait le compte à 27) », suivie d'une phrase pour la
  Story 28-6 (#482) : le littéral ajouté en dernier, qui porte le compte à 28, est
  `RECONCILIATION_TRANSACTION_NOT_CREDIT`, un débit ou un nul proposé contre une facture. Recompter les deux
  fichiers par `grep -nE "\b(27|28|29|30)\b"` après la modification.
- `frontend/src/lib/shared/i18n-keys.test.ts:601` : `sitesTotal` **+1** (un `i18nMsg(` de plus dans
  `failed-proposal-label.ts`, 34 → 35), **recompté** par la collecte du test, avec une ligne de commentaire datée
  dans la généalogie (patron de la 27-4, `:595-600`). `sitesNonResolus`, `relais`, `sitesGabarit` inchangés —
  à constater, non à supposer. ⚠️ La 28-2 (#520) modifie aussi ce compteur : à la rebase, **recompter**, ne pas
  additionner.
- Aucune capture d'écran : le refus n'est pas atteignable depuis l'écran (C-28-6-7).

**AC6 — Catalogue, quatre locales** (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`), clé
`reconciliation-failed-transaction-not-credit`, insérée dans le bloc `reconciliation-failed-*` (qui n'est **pas**
alphabétique : `already-rejected` suit `bank-transaction-not-found`), après `reconciliation-failed-score-too-low`
et avant `reconciliation-failed-transaction-not-pending` :

| Locale | Texte |
|---|---|
| fr-CH | `Cette transaction n’est pas une entrée d’argent (débit ou montant nul) : elle ne peut pas régler une facture client.` |
| de-CH | `Diese Transaktion ist kein Zahlungseingang (Belastung oder Betrag null): Sie kann keine Kundenrechnung begleichen.` |
| it-CH | `Questa transazione non è un’entrata di denaro (addebito o importo nullo): non può saldare una fattura cliente.` |
| en-CH | `This transaction is not an incoming payment (debit or zero amount): it cannot settle a customer invoice.` |

Apostrophe typographique `’` comme les clés voisines (`reconciliation-failed-already-rejected`). Le repli
français de `failed-proposal-label.ts` est **identique** à la ligne fr-CH.

**AC7 — Contrat d'API** (`docs/api-external.md`, § `POST /api/v1/reconciliation/accept`, `:538`) : après « Pour une
facture, le score est recalculé par le serveur », une phrase : une transaction de **débit** ou de montant **nul**
ne règle jamais une facture — refusée par proposition en `RECONCILIATION_TRANSACTION_NOT_CREDIT`,
`details.reason` = `debit_transaction` ou `zero_amount_transaction`, `details.transactionAmount`, **avant** la
lecture de la facture — mais **après** les refus du pré-vol (une transaction introuvable, rapprochée ou déjà
rejetée garde son code) *(depuis la v0.13.3)* ; ⚠️ **changement** : pour une facture à reste dû positif, elle
rendait auparavant `RECONCILIATION_SCORE_TOO_LOW` (rien de reconnu) ou **le code observé en T1** (référence ou
contact reconnus). Les propositions `split` et `rule` admettent un débit (inchangé). Une demi-phrase dit la
divergence assumée du **montant nul** (C-28-6-13) : pour `split` et `rule`, il reste refusé en `VALIDATION_ERROR`
avec `details.reason = "zero_amount_transaction"` (en `400` pour le rapprochement manuel et ventilé) — un client
qui ne teste que `details.reason` sous `VALIDATION_ERROR` ne verra pas le cas de la facture.

**AC8 — CHANGELOG** : section `## [0.13.3] — Non publié` en tête (après le `---` de l'en-tête, avant
`## [0.13.2]`), créée si aucune story de l'Epic 28 ne l'a encore ouverte — sinon **réunie** à celle de `main`,
jamais écrasée. Sous `### Corrigé` : une puce *Un débit ne règle plus une facture client
([#482](https://github.com/guycorbaz/kesh/issues/482))* — l'écran ne le proposait pas, mais l'API l'acceptait
jusqu'à la base, qui le refusait sous une « erreur de la base de données » ; désormais refusé par proposition, en
`RECONCILIATION_TRANSACTION_NOT_CREDIT`, libellé à l'écran ; intégrations par clé d'API : nouveau code, renvoi à
`docs/api-external.md`.

**AC9 — Manuels : le manuel utilisateur vérifié et non modifié ; le manuel d'administration reçoit une ligne
pour les intégrations (C-28-6-8, C-28-6-11).**

- **Manuel utilisateur** : il dit que les factures ne sont cherchées que pour une transaction « créditrice, en
  CHF » (`docs/manual/fr/user-manual.tex:1919`) — vrai avant comme après ; il ne promet rien de l'acceptation d'un
  débit, que l'écran ne peut pas soumettre. Non modifié.
- **Manuel d'administration** : il **nomme** des codes de `failed[]`. Le paragraphe « Intégrations par l'API : les
  lots de réconciliation » de la 0.13.2 (`admin-manual.tex:1992-1994`) annonce un **changement de contrat** de
  `POST /api/v1/reconciliation/accept` pour les clés d'API, avec trois codes (`BANK_TRANSACTION_NOT_FOUND`,
  `RECONCILIATION_ALREADY_RECONCILED`, `RECONCILIATION_ALREADY_REJECTED`), puis renvoie à `docs/api-external.md`.
  Ce paragraphe reste vrai après la story (il ne parle que des transactions absentes, rapprochées ou rejetées, jamais d'un débit). Mais la 28-6 change elle
  aussi le contrat d'`accept` (AC7), et le précédent de la 0.13.2 veut qu'un tel changement soit dit aux
  intégrations dans la section de version. Donc, à T5 :
  - si, au rebase, la sous-section **« Passer à la 0.13.3 »** existe (ouverte par une story antérieure de
    l'Epic 28), y écrire une ligne — au paragraphe d'intégrations par l'API, créé s'il manque, sur le modèle de
    celui de la 0.13.2 — qui nomme le code neuf **`RECONCILIATION_TRANSACTION_NOT_CREDIT`** : une proposition
    `invoice` sur une transaction de débit ou de montant nul est refusée par proposition sous ce code (elle
    rendait le code observé en T1, ou `RECONCILIATION_SCORE_TOO_LOW`) ; une intégration qui rejouait ce refus
    doit cesser ; détail dans `docs/api-external.md`. Puis régénérer le PDF (`make fr` dans `docs/manual/`, sous
    `flock`) et le commiter ;
  - sinon, **ne pas créer** la sous-section : la préparation de la release v0.13.3 l'ouvre et y écrit cette ligne,
    ce que C-28-6-11 lui prescrit au registre ; le dire au Dev Agent Record. La puce du CHANGELOG (AC8) en est
    l'entrée.

La lentille de validation et la revue de code contrôlent ces constats **sur le PDF aplati**
(`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) ; le `make` n'est dû que dans le premier cas.

## Tasks / Subtasks

- [ ] **T0 — Préparer** : rebase sur `main` après le merge de la 28-5 ; bases `kesh_286` / `kesh_e2e_286`
  (migrations du worktree, seed) ; `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-28-6/target` ; `node_modules` par
  `cp -al`. Relever les numéros de ligne cités ici contre le `HEAD` rebasé (ils bougent avec la 28-5).
- [ ] **T1 — Rouge d'abord (AC2, AC3)** : écrire les cas 1, 2, 4, 5 de l'AC2 et l'adaptation du cas 3 ; les faire
  tourner **avant** le correctif (gate ciblé `binary(reconciliation_e2e)`) ; consigner au Dev Agent Record le code
  **observé** pour le cas 1 et le cas 2, **sur le montage nommé** — facture validée de 100.00 sans règlement, donc
  **reste dû > 0** (un reste dû négatif, s'il est atteignable, rendrait `RECONCILIATION_OVERPAYMENT` avant le
  journal : non établi, non exercé, et `docs/api-external.md` ne doit pas présenter le code observé comme le seul
  possible) —, et le constat « rien d'écrit » (cf. *Prémisse*). Ce constat nomme **ce qui s'écrit avant la ligne
  refusée** et que seul le `ROLLBACK TO SAVEPOINT` annule : la valeur du compteur de numéros de l'exercice
  (`journal_entry_number_sequences.next_number`, consommé par `next_number_for`, étape 3 de
  `create_in_tx_inner`) et l'en-tête d'écriture (`journal_entries`, étape 4), relevés avant et après — avec
  `journal_entry_lines`. Les assertions de l'AC2 portent aussi le compteur.
- [ ] **T2 — La garde (AC1)** : fonction pure + appel entre l'étape 4 et l'étape 5 ; doc-comments ; tests unitaires
  (AC3). Vert ciblé.
- [ ] **T3 — Écran (AC5)** : `failed-proposal-label.ts`, son test, `i18n-keys.test.ts` (`sitesTotal` recompté).
- [ ] **T4 — Catalogue (AC6)** : quatre locales.
- [ ] **T5 — Documentation (AC7, AC8, AC9)** : `docs/api-external.md` (avec le code observé en T1), CHANGELOG ;
  vérification du manuel utilisateur (PDF aplati), consignée ; ligne du manuel d'administration si la sous-section
  de la 0.13.3 existe au rebase (puis `make fr`, PDF aplati contrôlé), sinon le dire au Dev Agent Record (C-28-6-11).
- [ ] **T6 — Mutations** (§ *Mutations*) : chacune rougit au moins un test nommé ; restauration
  `git checkout -- <fichier>` **puis `touch`**.
- [ ] **T7 — Gates au dernier commit de code**, sous `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock` : base
  remise à zéro, `scripts/test-fast.sh` (fmt + clippy + nextest complet), `npm run check`,
  `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`, **E2E Playwright complet** (recette
  `docs/testing.md`, port 3039) jugé fichier par fichier contre la liste des échecs attendus.

## Mutations

| # | Mutation | Test qui doit rougir |
|---|---|---|
| M1 | supprimer l'appel de la garde dans `accept_one_invoice` | AC2-1, AC2-2, AC2-3, AC2-5 |
| M2 | `<= Decimal::ZERO` → `< Decimal::ZERO` | AC2-2, AC3 (0.00, -0.00) |
| M3 | garde déplacée **après** l'étape 5 (lecture de la facture) | AC2-5 |
| M4 | garde déplacée **après** l'étape 7bis (score) | AC2-3 (`ANY-REF` → `SCORE_TOO_LOW`) |
| M5 | `reason` des deux branches échangés | AC2-1 ou AC2-2, AC3 |
| M6 | comparaison inversée (`>= ZERO` refusé) | tout test d'acceptation d'un encaissement existant (ex. le cas « encaissement » de AC2-4) |
| M7 | retrait du `case` frontend | `failed-proposal-label.test.ts` (clé attendue) |
| M8 | classification par **test de signe** : `amount.is_sign_negative()` → débit, sinon `amount.is_zero()` → nul (le piège nommé aux *Pièges*) | AC3 (`dec!(-0.00)` → nul, rendu `debit_transaction`) |

## Dev Notes

### Inventaire clos — tout chemin qui lie une transaction bancaire à une facture ou à une créance

Méthode (à refaire, et à **élargir** si une graphie manque) :

```sh
grep -rn "matched_entry_id\s*=" crates/*/src --include=*.rs          # qui rapproche une transaction
grep -rn "invoice_settlements::create_in_tx\|INSERT INTO invoice_settlements" crates/*/src --include=*.rs
grep -rln "bank_transaction" crates/kesh-api/src/routes/ crates/kesh-db/src/repositories/
grep -n "supplier" crates/kesh-api/src/routes/reconciliation.rs crates/kesh-db/src/repositories/reconciliation.rs
```

Relevé au `5aad8b35`, numéros de ligne recalés au `eab77cf1` (cf. § *Fichiers communs*) (hors modules `#[cfg(test)]`, hors tests d'intégration) :

| # | Chemin | Lie à une facture ? | Sens contrôlé aujourd'hui | Après la story |
|---|---|---|---|---|
| 1 | `get_proposals` (`reconciliation.rs:587`, filtre `:648-652`) | propose seulement | `<= 0` écarté | inchangé (résolu) |
| 2 | `accept_one_invoice` (`:1553`) — règlement `invoice_settlements::create_in_tx` `:1990`, rapprochement `UPDATE … matched_entry_id` `:2044` | **oui** (facture client) | **non** — seule la contrainte de base arrête | **résolu** (AC1) |
| 3 | `accept_one_split` (`:2244`, `UPDATE` `:2543`) | non — contreparties choisies | deux sens légitimes ; nul refusé `:2455` | inchangé (C-28-6-5) |
| 4 | `accept_one_rule` (`:2657`, `UPDATE` `:2995`) | non | deux sens légitimes (`rules.rs:8`) ; nul refusé `:2878` | inchangé |
| 5 | `post_manual` (`:3527`, `UPDATE` `:3806`) | non | deux sens ; nul refusé `:3664` | inchangé |
| 6 | `post_split` (`:3939`, `UPDATE` `:4263`) | non | deux sens ; nul refusé `:4114` | inchangé |
| 7 | Règlement depuis la facture, `invoice_settlements_write::settle_invoice` (`:67`, appel `routes/invoices.rs:1283`) | oui | montant saisi `<= 0` refusé (`:76`) ; **aucune transaction bancaire** n'y est liée | hors classe (résolu) |
| 8 | Solde d'une facture, `write_off_invoice` (`:429`) | oui | `<= 0` refusé (`:495`) ; aucune transaction | hors classe |
| 9 | `post_cancel_reconciliation` / `reconciliation_cancel.rs:340` | défait un lien | — | hors classe |
| 10 | Factures fournisseurs (`supplier_invoices::pay_in_tx`, lots `payment_batches`) | facture fournisseur | aucune transaction bancaire liée (`grep -n bank_transaction` vide dans ces fichiers) | hors classe |
| 11 | Restauration d'une sauvegarde (`backup.rs`) | recharge un état | — | hors classe (rejoue un état déjà validé) |

**Le symétrique — un crédit lié à une facture fournisseur — n'existe pas (C-28-6-9).** Aucun chemin ne rapproche une
transaction bancaire d'une facture fournisseur : la réconciliation ne cherche que des `invoices` (factures
clientes ; `grep -n supplier` dans les deux fichiers de réconciliation ne rend que le commentaire `:1756`, qui
cite `pay_in_tx` comme gabarit), le paiement fournisseur et les lots de paiement n'écrivent aucun lien vers
`bank_transactions`. Rien à refuser ; l'angle mort est écrit pour que l'absence ne passe pas pour un oubli. Le jour
où un rapprochement de facture fournisseur existera, la même règle — un règlement fournisseur exige une
**sortie** — s'y écrira.

**Avoirs** : aucun chemin ne lie une transaction à un avoir (`credit_notes`). Un remboursement de client
(débit) se rapproche aujourd'hui par affectation manuelle sur le compte débiteurs ; il ne touche ni le reste dû ni
le lettrage d'une facture. Hors périmètre (pas une régression, pas un défaut de sens : aucune facture n'y est
liée).

### Inventaire des sites NON RÉSOLUS après la story (ensemble clos)

Classe : *un site qui écrit le règlement d'une facture à partir d'une transaction bancaire sans contrôler que la
transaction est un encaissement*. Après la story, l'unique site de la classe (ligne 2 ci-dessus) est résolu. Ce qui
reste, écrit comme angle mort **assumé** :

| Site | Ce qui reste | Pourquoi assumé |
|---|---|---|
| Rapprochement manuel, ventilé ou par règle **vers le compte débiteurs** (lignes 3 à 6) | un débit ou un crédit peut mouvementer le 1100 sans régler ni facture ni reste dû | catégorie C : contrepartie choisie par l'utilisateur, deux sens légitimes (remboursement, acompte non affecté) ; le lettrage manuel le rattache (`docs/api-external.md:339`, « Une écriture rapprochée … se lettre ») ; aucune facture n'est liée, donc pas le défaut de #482 |
| Le pré-vol de `post_accept` (`:1048-1087`) ne connaît pas le sens | un lot fait de seuls débits prend le verrou du compte avant d'être refusé | sans effet comptable ; la garde vit là où la transaction est rechargée sous le verrou du compte, une seule fois ; l'ajouter au pré-vol la dupliquerait (C-28-6-3) |
| Défense en profondeur de la base (`chk_jel_*`) | reste la dernière barrière, inchangée | voulue |

### État actuel des fichiers modifiés — ce qui doit être préservé

- **`accept_one_invoice`** : recharge sous le verrou du compte (C2 Pass 1, TOCTOU) ; ordre des refus existant ; garde de score
  (P3-H4) ; trop-perçu à double borne (#476) ; créance lue sur l'écriture de vente (15-6a) ; refus « compte =
  créance » (15-6b) ; verrou optimiste `version = ?` contre le double règlement (25-4-c2) ; lettrage après l'`UPDATE`
  (15-1a2-i) ; double trace d'audit. **La story n'ajoute qu'un refus en tête** ; rien d'autre ne bouge.
- **`post_accept` / `accept_batch`** : un point de sauvegarde par proposition, rejeu `retry_with` (1213 + 1305) —
  inchangés ; la nouvelle garde rend un `Err(FailedProposal)` comme les autres, donc le point de sauvegarde est
  relâché par le chemin ordinaire.
- **`failed-proposal-label.ts`** : `switch` sur le code, `default` → « Refus non reconnu ({ $code }) ».

### Fichiers communs avec la 28-5 et la 28-7 (régions touchées)

| Fichier | 28-6 touche | 28-5 (#526) attendue | 28-7 (#548) attendue |
|---|---|---|---|
| `routes/reconciliation.rs` | fonction pure **juste au-dessus** d'`accept_one_invoice` (vers `:1522`) ; un appel entre l'étape 4 et l'étape 5 (`:1598-1601`) ; module de tests **en fin de fichier** | rejet, liste des rejetées, « annuler le rejet » (`post_reject`, `reject_batch`, `:3128-3463`), peut-être `classify_batch_transaction` (`:876`) | bornes de date d'`accept_one_invoice`, étape 6 (`:1638-1688`) — voisine, hunk distinct |
| `tests/reconciliation_e2e.rs` | `post_accept_filters_signed_amount` (`:2129`) + nouveaux tests en fin de fichier | tests du rejet | tests aux bornes de date |
| `failed-proposal-label.ts` / `.test.ts` | un `case`, une entrée, 29 → 30 | possible (si un code neuf) | non attendu |
| `i18n-keys.test.ts` (`sitesTotal`) | +1 | possible | non attendu |
| `messages.ftl` ×4 | une clé | clés du rejet | non attendu |
| `docs/api-external.md` | `:538` | `:540`, `:552` | `:538` possible (date) |
| `CHANGELOG.md` | `[0.13.3]` | idem | idem |
| `admin-manual.tex` / `.pdf` | une ligne, sous-section 0.13.3, **seulement si elle existe** (AC9, C-28-6-11) | possible (contrat du rejet) | non attendu |

Conflits attendus : textuels, à réunir (union) ; **recompter** `sitesTotal`, les 29/30 codes et les littéraux après
chaque rebase.

**Relu au rebase sur `eab77cf1`** (28-2, 28-3 et 28-5a fusionnées ; étape 0 du développement). Tous les numéros de
ligne de `routes/reconciliation.rs` cités dans cette fiche ont été recalés sur ce `HEAD`, par lecture de chaque site
(la 28-5a décale de +4 tout ce qui précède `:4312` ; quelques numéros de la fiche visaient déjà, au `5aad8b35`, la
ligne voisine du site — le commentaire au lieu du `return` — et sont recalés sur le site lui-même ; `accept_one_invoice`
court de `:1553` à `:2228`). Ce que la 28-5a a réellement fait du fichier : une section
« annuler le rejet » **en fin de fichier**, avant les modules de tests (`bank_account_of_transaction` `:4595`,
`classify_unreject` `:4646`, `post_unreject_transaction` `:4682`, `unreject_once` `:4701`), un module de tests
`unreject_classification_tests` (`:5106`, dernier du fichier), et `conclude_locked_attempt` rendue générique
(`:4332`, `<T>`). Elle n'a touché ni `classify_batch_transaction` (`:876`), ni `accept_one_invoice`. La 28-6 reste
hors de ces régions : sa fonction pure va au-dessus d'`accept_one_invoice`, son module de tests **après**
`unreject_classification_tests`.

### Pièges connus du dépôt, applicables ici

- **`-0.00`** : défense en profondeur de la fonction pure, **pas un cas atteignable par la base**. Un CSV peut
  porter un zéro négatif (`user-manual.tex:1788` l'accepte en import), mais `bank_transactions.amount` est
  `DECIMAL(18,2)` : la valeur s'écrit et se relit `0.00`, et la garde d'`accept_one_invoice` ne verra jamais
  `-0.00`. `Decimal` compare `-0.00 <= 0` vrai, mais `is_zero()` et `is_sign_negative()` divergent : la garde
  **compare**, elle ne teste pas le signe — sinon `-0.00` rendrait `debit_transaction`. AC3 le fixe (précondition
  assertée) et M8 le prouve (C-28-6-12).
- **Test qui accepte deux codes** (`post_accept_filters_signed_amount`) : la forme `matches!(code, A | B)` rend un
  test vert pour deux comportements ; c'est ce qui a laissé #482 ouverte. Une égalité stricte, partout.
- **Mutation restaurée, binaire périmé** : après `git checkout -- <fichier>`, `touch <fichier>` (cargo compare des
  dates).
- **Base de gate** : remise à zéro avant tout gate complet (KF-039).

### Découpage préventif — non requis

Modules touchés (règle de splitting, > 5 modules) : **4** — `kesh-api` (route + tests), `frontend/src/lib/features/reconciliation`,
`frontend/src/lib/shared` (un compteur de test), `kesh-i18n` (catalogues) ; plus deux documents
(`docs/api-external.md`, `CHANGELOG.md`), et une ligne du manuel d'administration dans un seul des deux cas de
l'AC9. Une seule règle métier ajoutée, à un seul site.

### Exclusions écrites (périmètre minimal, C-28-6-10)

- Aucune garde de sens sur `split`, `rule`, `manual`, `split` standalone (AC4).
- Pas de garde au pré-vol de `post_accept`.
- Pas de changement de `get_proposals`, du score, de `classify_payment`, de `settlement_journal_lines`.
- Pas d'harmonisation du `VALIDATION_ERROR` + `zero_amount_transaction` des autres variantes avec le nouveau code.
- Pas de rapprochement de facture fournisseur ni d'avoir.
- Pas de modification du manuel utilisateur ; au manuel d'administration, la seule ligne d'intégrations de
  l'AC9, et seulement si la sous-section de la 0.13.3 existe au rebase (C-28-6-11). Pas de capture d'écran.
- Le bras `_` → `DATABASE_ERROR` de `project_error_to_failed_proposal` reste tel quel (une violation de contrainte
  y redevient une panne réelle, ce qu'elle doit être une fois la garde métier posée).

### Project Structure Notes

Conforme : refus par proposition dans la route (patron 27-4, `classify_batch_transaction`) ; libellé frontend dans
`failed-proposal-label.ts` (15-5c) ; clés dans `kesh-i18n` ; contrat dans `docs/api-external.md`.

### References

- Issue #482 ; epic `planning-artifacts/epic-28-recette-v0-13-2.md` § 28-6.
- `CLAUDE.md` § *Pattern batch — FailedProposal per-proposal* ; § *Review Iteration Rule* (sites non résolus,
  propagation post-patch, recompte) ; `docs/agents-orchestration.md`.
- Story 27-4 (`27-4-lot-reconciliation-transaction-disparue.md`) : classification par proposition, codes en dur.
- Story 15-5c : `failed-proposal-label.ts`. Story 24-2 : écriture d'encaissement. Story 25-4-c (#420) : reste dû.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-13 — Fiche créée (`bmad-create-story`, Opus 5.5, autonomie). Prémisse de l'issue corrigée (C-28-6-1) :
  à la lecture, la base refuse l'écriture négative ; le défaut est le refus présenté comme une panne, à constater
  au rouge (T1). Choix C-28-6-1 à C-28-6-10.
- 2026-10-13 — **Validation P1** (lentilles R et F, Sonnet 5.5 ; rapports `kesh-gate-logs/28-6-validate-p1-{R,F}.md`) :
  0 CRITICAL, 0 HIGH, **1 MEDIUM**, 9 LOW, 8 distincts (R : 1 MEDIUM, 5 LOW ; F : 4 LOW ; F3 = R5). Prémisse C-28-6-1
  confirmée par les deux lentilles, maillon par maillon (lecture, non exécution). Remédiés (Opus 5.5) :
  **R1** (MEDIUM) — l'AC9 et C-28-6-8 disaient à tort que le manuel d'administration ne nomme aucun code de
  `failed[]` (`admin-manual.tex:1994` en nomme trois) : justification réécrite, une ligne prescrite au paragraphe
  d'intégrations de la 0.13.3 si la sous-section existe au rebase, sinon à la préparation de release (C-28-6-11) ;
  **F1** « sous le verrou du compte » ; **F2** montage de T1 nommé (reste dû > 0) et AC7 borné à ce cas ;
  **F3/R5** `-0.00` cas de la seule fonction pure, précondition assertée en AC3, mutation **M8** (test de signe)
  ajoutée (C-28-6-12) ; **F4** demi-phrase d'AC7 sur la divergence du montant nul (C-28-6-13) ; **R2** doc-comment
  de `failed-proposal-label.ts` (29/27 → 30/28, généalogie 27-4 complétée) à l'AC5 ; **R3** quatre tests existants
  nommés à l'AC4 (C-28-6-14) ; **R4** compteur de numéros et en-tête d'écriture nommés au constat « rien d'écrit »
  de T1 ; **R6** ordre des refus écrit pré-vol compris (`RECONCILIATION_ALREADY_REJECTED` d'abord), « alphabétique »
  retiré de l'AC6. Choix C-28-6-11 à C-28-6-14 ajoutés, C-28-6-8 amendé. Mutations : 7 → 8. Suite : passe ciblée
  P2 (Haiku), `28-6-validate-prompt-p2-ciblee.md`.
