# Story 24.4a : La contre-passation d'une écriture — corriger sans réécrire

## Status

ready-for-dev

## Story

**As a** personne qui tient les livres,
**I want** corriger une écriture par une contre-passation,
**so that** la correction se voie dans les livres au lieu d'effacer ce qu'elle corrige.

## Le défaut

Une écriture enregistrée est aujourd'hui **réécrivable et destructible** :

- le `PUT` réécrit date, journal, libellé **et la totalité des lignes** — `DELETE FROM journal_entry_lines` puis réinsertion (`journal_entries.rs:1019`, dans `update`). L'état antérieur **disparaît des tables comptables** ;
- le `DELETE` est une **suppression physique** (`:1226`) ; aucun `deleted_at` n'existe au schéma ;
- l'unique verrou est la **clôture annuelle** — `update` (`:843`) et `delete_in_tx` (`:1162`) ne gardent que `fy_status == "Closed"`.

⚠️ **Art. 958f CO et Olico art. 3 : l'exigence n'est pas qu'on ne se trompe jamais, c'est que la correction soit APPARENTE.** Ici elle ne l'est pas.

⛔ **Et depuis la vague 1, ce trou défait ce qu'elle vient de construire.** Le résiduel d'une facture se calcule depuis `invoice_settlements.amount`, **jamais depuis l'écriture** (`amount_due`, `invoice_settlements.rs:129-141` : `TTC − avoirs − SUM(settlements.amount)`). Réécrire l'écriture d'un règlement fait donc diverger le grand livre et le résiduel **en silence** — le mode d'échec exact du défaut fondateur que la 24-2 a corrigé.

## Ce que cette story fait, et ce qu'elle ne fait PAS

⛔ **Elle n'enlève rien.** Le `PUT` et le `DELETE` restent en place, tels quels. Cette story **ouvre la porte de sortie** ; la **24-4b** ferme la porte destructive (statut `comptabilisé`, `PUT`/`DELETE` refusés), la **24-4c** pose le verrou de période.

⚠️ **L'ordre est contraint et il n'est pas négociable** : geler avant que la contre-passation existe rendrait Kesh **incorrigible**, la réécriture destructive étant aujourd'hui la seule voie de correction d'une OD.

⛔ **Risque accepté, entre la 24-4a et la 24-4b — et il est réel.** `update` ne garde que l'exercice clos : entre les deux stories, rien n'empêche de **réécrire par `PUT` une écriture déjà contre-passée**, ni de réécrire ou supprimer **la contre-passation elle-même** — qu'aucune des six FK de D3 ne possède. Une origine réécrite après coup rend sa contre-passation **fausse**, sans que rien ne le détecte : la divergence muette que l'epic ferme, réintroduite par le mécanisme même qu'on livre.

**Ce risque est assumé pour une fenêtre courte, et il est TRACÉ comme la première exigence de la 24-4b** : le gel doit refuser le `PUT`/`DELETE` d'une écriture **portant** `reverses_entry_id` comme d'une écriture **référencée** par le `reverses_entry_id` d'une autre. ⚠️ Si la 24-4b devait être différée, cette garde-là remonte dans la 24-4a — elle ne se perd pas.

## Le gabarit existe déjà — `supplier_invoices::cancel`

Ne rien réinventer. `supplier_invoices::cancel` (`supplier_invoices.rs`, à partir de `pub async fn cancel`) contre-passe déjà une écriture d'achat, et la story reprend sa séquence **à l'identique** :

1. verrou `FOR UPDATE` sur la cible + garde d'état ;
2. relecture des lignes de l'écriture d'origine, **inversion `D ↔ C`** (`debit: *credit, credit: *debit`) ;
   ⛔ **La requête du gabarit est INSUFFISANTE ici** : `cancel` fait `SELECT account_id, debit, credit … ORDER BY jel.id` et pose `project_id: None` en dur. Écrire :
   ```sql
   SELECT account_id, debit, credit, project_id FROM journal_entry_lines
   WHERE entry_id = ? ORDER BY line_order
   ```
   `line_order` porte contractuellement la position (`uq_jel_entry_order UNIQUE (entry_id, line_order)`), et `project_id` est indispensable à l'AC 8.
3. `fiscal_years::find_open_covering_date(&mut tx, company_id, today)` → `DbError::FiscalYearInvalid` si aucun ;
4. `journal_entries::create_in_tx(..., journal: Journal::OD, ..., false)` ;
5. `audit_log::insert_in_tx` portant `reversalJournalEntryId`.

Le seul écart : côté fournisseur, le lien et l'idempotence sont portés par `supplier_invoices.status = 'cancelled'`. Une écriture manuelle n'a **aucun statut** — d'où D2.

## D1 — La contre-passation est une écriture NEUVE, jamais une modification

L'écriture d'origine n'est **pas touchée** : ni ses lignes, ni sa date, ni son `entry_number`. La contre-passation est une écriture supplémentaire, au journal **OD**, portant les mêmes comptes avec débit et crédit échangés.

⛔ **Le projet se reprend LIGNE PAR LIGNE, positionnellement** — ligne *i* de la contre-passation ← `project_id` de la ligne *i* de l'origine — et `NewJournalEntry.project_id` reste **`None`**.

⚠️ **C'est le seul point où le gabarit `cancel` NE S'APPLIQUE PAS, et le suivre à la lettre corromprait les données.** `project_id` n'existe **pas** sur `journal_entries` : la colonne vit sur `journal_entry_lines` (`20260702000001_projects_analytics.sql:38`). Le champ `NewJournalEntry.project_id` est un **estampillage document-level à l'écriture** — `line.project_id.or(new.project_id)` — que `cancel` peut employer parce qu'une facture fournisseur n'a qu'**un** projet. Une écriture **manuelle**, elle, porte un tag **par ligne** (Story 19-2) : reprendre « le projet de l'origine » comme valeur unique écraserait silencieusement les tags distincts d'une écriture multi-projets — et casserait précisément l'invariant que la clause sert, le retour à zéro du net par projet.

⛔ **Un projet archivé NE DOIT PAS bloquer, et le code actuel bloquerait.** `create_in_tx` appelle `validate_taggable_in_tx` sur les tags par-ligne (`journal_entries.rs:204`), et celle-ci **rejette** dès qu'un projet est archivé (`projects.rs:115-118`, `IllegalStateTransition("le projet analytique est archivé")`). Comme la contre-passation fournit des tags **par ligne**, elle tomberait sur ce refus.

**Il faut donc une exemption**, symétrique de `enforce_postable` : un paramètre `enforce_project_taggable: bool` (ou un chemin d'insertion dédié), à `false` pour la contre-passation. Le dépôt a déjà tranché dans ce sens — `pay` et `cancel` « estampillent volontairement un projet potentiellement archivé après coup », et un test le verrouille (`pay_succeeds_when_project_archived_after_tagging`).

⚠️ **L'asymétrie avec le compte archivé (D5-bis) est VOULUE, et il faut la comprendre pour ne pas la « corriger ».**

| | compte archivé | projet archivé |
|---|---|---|
| décision | **refuser**, en nommant le compte | **tolérer** |
| pourquoi | poster sur un compte archivé le **ressusciterait** ; la garde protège tous les flux, et l'avoir refuse déjà de la même façon | le tag est **copié, pas choisi** ; les flux automatiques du dépôt estampillent déjà des projets archivés |
| voie de sortie | `PUT /api/v1/accounts/{id}/reactivate` | aucune n'est nécessaire |

⛔ Ne pas aligner les deux cas : ce n'est pas une incohérence, c'est la différence entre écrire **dans** un compte et **étiqueter** une ligne.

Libellé, **au format exact** — le numéro, jamais l'`id` de base de données :

| cas | libellé |
|---|---|
| `origine.fiscal_year_id == contre-passation.fiscal_year_id` | `Contre-passation écriture n° {entry_number}` |
| exercices différents | `Contre-passation écriture n° {entry_number} ({nom de l'exercice DE L'ORIGINE})` |

⚠️ Le nom est celui de l'exercice **de l'origine** — c'est lui qui lève l'ambiguïté du numéro ; celui de la contre-passation n'apprendrait rien. Et puisque D4 impose la date du jour, la branche courte ne s'atteint que si l'origine est elle aussi dans l'exercice courant.

⚠️ Ce libellé est un littéral **français stocké en base**, comme `format!("Annulation facture fournisseur {…}")` (`supplier_invoices.rs:793`). Il n'est pas traduit — l'AC 19 ne porte que sur les libellés **d'écran**.

⚠️ **`entry_number` REPART À 1 à chaque exercice** (leçon de la 24-1) : sans le suffixe, deux « écriture n° 12 » sont indiscernables. Le format est figé ici parce que deux implémenteurs en produiraient sinon deux, et qu'aucun test ne les départagerait.

## D2 — Le lien vit en base : `journal_entries.reverses_entry_id`

Une colonne `reverses_entry_id BIGINT NULL`, FK vers `journal_entries(id)` `ON DELETE RESTRICT`, **`UNIQUE`**.

⛔ **`RESTRICT` a deux conséquences que la story DOIT porter — elles ne sont pas des effets de bord, elles changent du code existant.**

**(a) La suppression en masse casse, de façon INTERMITTENTE.** `delete_all_by_company` (`journal_entries.rs:1290`) fait `DELETE FROM journal_entries WHERE company_id = ?` **en un seul statement**, et InnoDB vérifie les clés étrangères **ligne par ligne, sans différer** : si l'origine est traitée avant sa contre-passation, la contrainte déclenche une erreur 1451. Le résultat dépend de l'ordre de parcours — donc il est **aléatoire**. Ce chemin sert `reset_demo` (production) **et le teardown de dix tests unitaires** de `journal_entries.rs`. ⇒ `delete_all_by_company` doit remettre `reverses_entry_id` à `NULL` pour la société **avant** le `DELETE`.

**(b) Le `DELETE` unitaire d'une écriture contre-passée cesse de réussir.** C'est **souhaitable** — supprimer une écriture qu'on a corrigée effacerait la correction — mais ce n'est plus le comportement d'aujourd'hui, et il ne faut pas le livrer sous la forme d'une 1451 au message générique. ⇒ discriminer le nom de la contrainte et rendre **409 `ENTRY_IS_REVERSED`**, comme le dépôt le fait déjà pour `fk_invoices_journal_entry`.

⚠️ **Le paragraphe « Risque accepté » ci-dessus dit que le `PUT` et le `DELETE` restent « tels quels » : c'est vrai du `PUT`, faux du `DELETE` d'une origine contre-passée.** C'est le seul endroit où cette story referme déjà un pan de la 24-4b.

Elle porte **trois** propriétés d'un seul geste, et c'est ce qui la justifie :

| propriété | comment |
|---|---|
| la correction est **apparente** | le lien est dans les livres, pas seulement au journal d'audit |
| **idempotence** | l'`UNIQUE` interdit structurellement de contre-passer deux fois la même écriture |
| **lisibilité** | l'écran et le grand livre peuvent afficher « contre-passée » / « contre-passe l'écriture n° X » |

⛔ **Ne PAS poser un booléen `is_reversed` sur l'origine** : deux colonnes à tenir cohérentes là où une seule suffit, et le lien inverse se perd.

⚠️ Une écriture qui **est** une contre-passation (`reverses_entry_id IS NOT NULL`) n'est **pas** contre-passable à son tour — cf. D3. Se contre-passer une contre-passation revient à réécrire l'original, en trois écritures au lieu d'une.

## D3 — Ce qui est contre-passable, et ce qui ne l'est pas

⛔ **Refuser toute écriture qu'une PIÈCE possède.** Sept clés étrangères pointent vers `journal_entries` ; la septième, `fk_jel_entry` (`journal_entry_lines.entry_id`, `ON DELETE CASCADE`), est la table de détail de l'écriture **elle-même**, pas une pièce externe. Restent **six pièces** — cinq `RESTRICT`, une `SET NULL` :

| table · colonne | ON DELETE | pourquoi le refus |
|---|---|---|
| `invoices.journal_entry_id` | RESTRICT | la facture resterait `validated` en pointant une écriture contre-passée → le chemin est l'**avoir** |
| `credit_notes.journal_entry_id` | RESTRICT | l'avoir EST déjà la contre-passation |
| `supplier_invoices.purchase_journal_entry_id` | RESTRICT | le chemin est `supplier_invoices::cancel` |
| `supplier_invoices.settlement_journal_entry_id` | RESTRICT | annuler un règlement fournisseur → **#414** |
| `invoice_settlements.journal_entry_id` | RESTRICT | ⛔ **le cas le plus grave** : le résiduel se calcule depuis `settlements.amount`, que la contre-passation ne toucherait pas → grand livre et résiduel divergent **en silence**. Le chemin est **#414** |
| `bank_transactions.matched_entry_id` | **SET NULL** | la transaction resterait « rapprochée » contre une écriture contre-passée |

⚠️ **La dernière ligne est la décision la moins évidente, et elle est assumée.** Il n'existe **aucune** route de dé-rapprochement (`accept`, `reject`, `manual`, `split` — rien d'autre, vérifié au montage `lib.rs:558-576`) : refuser laisse donc une écriture de rapprochement manuel **sans voie de correction**. On refuse quand même, parce que l'alternative — contre-passer en laissant `matched_entry_id` pointer l'origine — recrée exactement la désynchronisation muette que cette vague supprime. **Le manque est réel, et il est tracé : #418.** ⛔ Après la 24-4b, l'écriture de rapprochement manuel devient **définitivement** incorrigible — d'où l'urgence relative de #418.

⚠️ **Une écriture d'ouverture d'exercice EST contre-passable, et c'est délibéré.** Il n'existe ni table ni marqueur qui la distingue d'une OD ordinaire — `opening_balances` est une **route** (`routes/opening_balances.rs`), pas une entité. Aucune des six FK ne la possède : elle passe donc le filtre, et il n'y a aucun moyen de la refuser sans inventer un marqueur hors périmètre. Corriger un bilan d'ouverture faux est d'ailleurs un besoin légitime.

**Le refus est un code métier, pas un 500** : `DbError::IllegalStateTransition` → HTTP **409**, avec un message nommant la pièce propriétaire et le chemin de correction à emprunter. Un utilisateur qui lit « cette écriture appartient à la facture F-2026-014, corrigez-la par un avoir » sait quoi faire ; un « interdit » sec, non.

## D4 — La date : aujourd'hui, dans un exercice ouvert

La contre-passation porte la **date du jour**, et exige un exercice **ouvert** qui la couvre (`find_open_covering_date`, sinon `DbError::FiscalYearInvalid`).

⛔ **Ce refus rend `400 FISCAL_YEAR_INVALID`, PAS 409** — le mappage est partagé (`kesh-api/src/errors.rs:2482`). Ne pas le « corriger » vers 409 pour satisfaire un test : cela changerait le statut de tous les flux qui remontent cette erreur (`supplier_invoices.rs:342`, `:622`, `:784`), hors périmètre et non testés.

⛔ **Jamais la date de l'origine.** Deux raisons : une origine dans un exercice **clos** rendrait la correction impossible, alors que la contre-passer dans l'exercice courant est précisément la pratique comptable ; et dater la correction du jour de l'erreur la rend invisible dans la période où elle a été décidée.

## D5 — La postabilité n'est pas exigée

`enforce_postable = false`, comme tous les flux de contre-passation existants. La contre-passation **reprend les comptes de l'origine** : si l'un d'eux est devenu non-postable depuis (Story 14-3a), l'exiger rendrait l'écriture **incorrigible à cause d'un changement de configuration postérieur**. Le commentaire de `create_in_tx` (`journal_entries.rs:183-186`) énonce déjà cette règle pour les flux automatiques.

## D5-bis — Le compte ARCHIVÉ bloque, et il faut le dire proprement

⛔ **`enforce_postable = false` ne suffit pas.** La validation des comptes filtre `active = TRUE` **inconditionnellement** ; seule la clause `postable = TRUE` est conditionnée par le drapeau :

```sql
-- journal_entries.rs:97
SELECT id FROM accounts WHERE company_id = ? AND active = TRUE AND id IN (…)
```

Une écriture dont un compte a été **archivé** depuis sa création est donc **incontre-passable** — c'est-à-dire, après la 24-4b, **définitivement incorrigible**.

**Le dépôt a déjà tranché ce cas pour l'avoir, et on le copie JUSQU'AU STATUT** : `400 ACCOUNT_ARCHIVED`, en **nommant les comptes à réactiver** dans `details.rejected[]` (`accountId` / `accountNumber`). Le gabarit est `CreditNoteRevenueAccountsArchived` (`kesh-api/src/errors.rs:2444`), qui rend bien **400** (`:2472`) — pas 409. Ce qu'on refuse, c'est le `400 INACTIVE_OR_INVALID_ACCOUNTS` **générique** (`:2404`), qui ne dit pas quel compte.

⚠️ **Deux statuts cohabitent, et la distinction est réelle** : un refus de **propriété** (D3) est un conflit d'état → `IllegalStateTransition` → **409** (`kesh-api/src/errors.rs:2388`) ; un compte archivé ou l'absence d'exercice ouvert sont des **données d'entrée invalides** → **400**.

⚠️ **Le chemin de sortie existe et le message doit y renvoyer** : `PUT /api/v1/accounts/{id}/reactivate` (`lib.rs:325-328`). Un refus qui nomme le compte et l'action à faire est utilisable ; un refus opaque ne l'est pas.

⛔ **Ne PAS contourner en retirant `active = TRUE`** : poster sur un compte archivé le ressusciterait par la bande, et cette garde protège tous les autres flux.

## D6 — Route, RBAC, audit

- `POST /api/v1/journal-entries/{id}/reverse` — monté dans **`comptable_routes`** (Admin + Comptable), comme `create` / `update` / `delete` (`lib.rs:332-339`). ⛔ Pas dans `authenticated_routes` : Consultation ne contre-passe pas.
- Réponse **201** avec l'écriture créée (même forme que `create_journal_entry`).
- Audit : action `journal_entry.reversed`, cible `journal_entry` / id de l'origine, détails `{ "reversalJournalEntryId": <id> }` — miroir exact de `supplier_invoice.cancelled`.

⛔ **La lecture doit porter de quoi décider, sinon l'écran devine.** `GET /api/v1/journal-entries/{id}` et la liste exposent :

| champ | rôle |
|---|---|
| `reversesEntryId` | l'écriture que celle-ci contre-passe (`null` sinon) |
| `reversedByEntryId` | l'écriture qui la contre-passe (`null` sinon) — **dérivé**, pas une colonne |
| `reversable` | booléen |
| `reversalBlockedBy` | code canonique, `null` si `reversable` — `OWNED_BY_INVOICE`, `OWNED_BY_CREDIT_NOTE`, `OWNED_BY_SUPPLIER_INVOICE`, `OWNED_BY_SETTLEMENT`, `MATCHED_BANK_TRANSACTION`, `ALREADY_REVERSED`, `IS_A_REVERSAL`, `ACCOUNT_ARCHIVED` (D5-bis) |

⛔ **Ces quatre champs sont exposés sur le DÉTAIL seulement — `GET /api/v1/journal-entries/{id}`.** La liste paginée ne les porte pas : aucun écran ne les y consomme (le bouton vit sur la fiche, AC 18), et les calculer par ligne obligerait à joindre cinq tables plus `journal_entry_lines`→`accounts`, pour rien. `list_by_company_paginated` (`journal_entries.rs:711`) reste **inchangée**.

⚠️ **Les causes de blocage se CUMULENT** — une écriture peut être possédée par une facture, déjà contre-passée **et** porter un compte archivé. Le champ étant scalaire, la précédence est **figée** ici, sans quoi le test « une cause, un test » deviendrait non déterministe :

`IS_A_REVERSAL` → `ALREADY_REVERSED` → les cinq codes de propriété, **dans l'ordre du tableau D3** → `ACCOUNT_ARCHIVED`

Chaque test de l'AC 17 monte **une seule** cause.

⚠️ **Un code, jamais une phrase** (convention `FailedProposal` du `CLAUDE.md`) : la traduction se fait à l'écran, dans les quatre locales. Sans ces champs, l'écran ne peut ni masquer le bouton ni dire pourquoi, et se rabattrait sur un 409 découvert **après** le clic.

⚠️ **`reversedByEntryId` se dérive de l'`UNIQUE`** — un `SELECT ... WHERE reverses_entry_id = ?`, jamais une seconde colonne à tenir cohérente (cf. D2).

## Critères d'acceptation

1. `POST /api/v1/journal-entries/{id}/reverse` crée une écriture au journal **OD**, à la date du jour, dont les lignes sont celles de l'origine avec **débit et crédit échangés**, dans l'ordre de `line_order`.
2. L'écriture d'origine est **inchangée** — lignes, date, journal, libellé, `entry_number`, `version`.
3. La nouvelle écriture porte `reverses_entry_id = {id de l'origine}` ; l'origine porte `reverses_entry_id = NULL`.
4. Un second appel sur la **même** origine rend **409 `ALREADY_REVERSED`** — obtenu en discriminant le **nom de la contrainte** dans la violation d'unicité, jamais le `RESOURCE_CONFLICT` générique, et jamais 500.
5. Contre-passer une écriture **qui est elle-même une contre-passation** rend **409 `IS_A_REVERSAL`**.
6. Contre-passer une écriture possédée par une pièce rend **409**, avec un message nommant la pièce **et** le chemin de correction à emprunter.
7. Sans exercice **ouvert** couvrant la date du jour : **400 `FISCAL_YEAR_INVALID`** (pas 409). L'exercice de l'**origine** n'entre pas dans la décision : contre-passer une écriture d'un exercice **clos** est autorisé, la contre-passation tombant dans l'exercice courant.
8. Le `project_id` est repris **ligne par ligne** : la ligne *i* de la contre-passation porte celui de la ligne *i* de l'origine. Un test exerce une écriture **multi-projets** (deux lignes, deux projets différents) — un test mono-projet ne verrait pas l'écrasement.
9. La contre-passation aboutit même si un projet de l'origine a été **archivé** depuis (exemption `enforce_project_taggable`, cf. D1).
10. La contre-passation aboutit même si un compte de l'origine est devenu **non-postable** (`enforce_postable = false`).
11. Un compte de l'origine **archivé** fait échouer la contre-passation en **400 `ACCOUNT_ARCHIVED`**, dont les `details.rejected[]` portent `accountId` et `accountNumber` — jamais le `INACTIVE_OR_INVALID_ACCOUNTS` générique. `reversable` vaut alors `false` **avant** le clic.
12. Le rôle **Consultation** reçoit 403 ; Admin et Comptable passent. Un `id` inexistant **ou appartenant à une autre société** rend **404** — jamais 403, qui révélerait l'existence de la ressource (convention IDOR du dépôt).
13. Une entrée d'audit `journal_entry.reversed` est écrite, portant `reversalJournalEntryId`.
14. L'opération est **atomique** : si quoi que ce soit échoue, aucune trace — ni écriture, ni ligne, ni audit.
15. `DELETE /api/v1/journal-entries/{id}` sur une origine **déjà contre-passée** rend **409 `ENTRY_IS_REVERSED`**, et non une violation de clé étrangère au message générique.
16. `delete_all_by_company` réussit sur une société contenant une écriture **et sa contre-passation** — un test le verrouille, `reset_demo` en dépend, et l'échec serait **intermittent** (cf. D2 (a)).
17. La lecture `GET /api/v1/journal-entries/{id}` expose `reversesEntryId`, `reversedByEntryId`, `reversable` et `reversalBlockedBy`. ⛔ **Ce sont les SIX CHEMINS FK qui sont testés, pas les codes** : `supplier_invoices.purchase_journal_entry_id` et `settlement_journal_entry_id` partagent `OWNED_BY_SUPPLIER_INVOICE` mais renvoient vers deux corrections différentes (`cancel` / #414) — un test par code laisserait le second **jamais exercé**. Soit 6 chemins FK + `ALREADY_REVERSED` + `IS_A_REVERSAL` + `ACCOUNT_ARCHIVED` = **9 tests**, chacun montant **une seule** cause.
18. **Écran** : la fiche offre « Contre-passer », sous confirmation ; l'écriture contre-passée affiche un renvoi vers sa contre-passation, et réciproquement. Le bouton est **absent** — pas seulement désactivé — quand `reversable` est faux, et le motif est affiché en clair, traduit depuis `reversalBlockedBy`. La **liste** n'affiche rien de tout cela.
19. Les libellés **d'écran** sont dans les **quatre** locales, sous le préfixe `journal-entries-`. Le libellé de l'écriture, lui, est un littéral français stocké en base (cf. D1).

## Invariants testables

- **I1 — Somme nulle** : pour toute écriture contre-passée, `SUM(debit) = SUM(credit)` sur l'origine **et** sur la contre-passation, et la somme des deux écritures est **nulle compte par compte**. C'est l'invariant qui prouve que l'inversion est exacte, là où un total global se laisserait tromper par une compensation entre comptes.
- **I2 — Le grand livre montre les deux** : après contre-passation, l'extrait du compte touché porte **deux** lignes, jamais zéro. (La correction est apparente — c'est l'exigence légale, écrite en test.)
- **I3 — Aucune pièce désynchronisée** : après la suite complète, aucune ligne de `invoices`, `credit_notes`, `supplier_invoices`, `invoice_settlements`, `bank_transactions` ne référence une écriture portant un `reverses_entry_id` entrant.

## Hors périmètre

- **Le gel** — statut `comptabilisé`, `PUT`/`DELETE` refusés : **24-4b**.
- **Le verrou de période** plus fin que l'année : **24-4c**.
- **Annuler un règlement** (client et fournisseur) : **#414**.
- **Annuler un rapprochement bancaire** : **#418**, ouverte à ce titre (cf. D3).
- **Les trous de numérotation** laissés par le `DELETE` physique : **#381**.
- Aucun statut brouillon — arbitrage du Project Lead : l'écriture reste définitive dès l'insertion.

## Dev Notes

### Garde-fous de migration — TOUS déclenchés, aucun optionnel

La migration `20260828000002_journal_entries_reversal.sql` ajoute une colonne nullable et un index : **non breaking** (P1), donc **aucun bump** de `kesh_version_min_required` ni de version Cargo. Mais quatre garde-fous parlent quand même :

| garde | ce qu'il exige |
|---|---|
| **P5** | une ligne au tableau de `docs/migrations-idempotence-audit.md` + les compteurs, **recomptés depuis la source** : `63 → 64` à l'en-tête `## Table d'audit (N migrations)` ET à la ligne `Total`, plus la partition (`yes` / `tracked-by-sqlx` / `no`) dont la **somme** doit valoir le total. ⚠️ Les compteurs de partition ne valent pas le total. |
| **P6** | `assert_eq!(total, 63)` de `migrations_upgrade_path.rs:95` **VA ROUGIR** → `64`. C'est un garde-fou volontaire, pas un défaut. `migrations_before_backfill()` résout **par version**, il ne bouge pas. |
| **P7** | la migration **n'écrit aucune donnée** (DDL pur) → ni registre `POST_RESTORE_BACKFILLS`, ni exemption. Ne pas confondre avec `ON UPDATE CURRENT_TIMESTAMP`, qui est du DDL. |
| **P8** | une ligne `<version> <sha384>` dans `crates/kesh-db/migrations.sha384` (78 lignes aujourd'hui), et **ne jamais retoucher** une migration déjà appliquée — pas même un commentaire. |

⛔ **`crates/kesh-db/test-schema/0001_schema_squash.sql` doit être mis à jour**, sans quoi `test_schema_guard.rs` rougit : 1102 des attributs `#[sqlx::test]` montent le squash, pas les migrations réelles.

⛔ **Gate complet obligatoire, ciblage interdit** (`CLAUDE.md`, exception `kesh-db` de la § *Review Iteration Rule*). La 24-2 a vu **sept** garde-fous se déclencher sur des fichiers qu'elle ne touchait pas ; seul le gate réellement exécuté les révèle. Remettre la base à zéro **avant** (KF-039), inconditionnellement.

### Fichiers à toucher

| fichier | nature |
|---|---|
| `crates/kesh-db/migrations/20260828000002_journal_entries_reversal.sql` | NEW |
| `crates/kesh-db/migrations.sha384` · `test-schema/0001_schema_squash.sql` | UPDATE |
| `crates/kesh-db/src/entities/journal_entry.rs` | UPDATE — `reverses_entry_id: Option<i64>` |
| `crates/kesh-db/src/repositories/journal_entries.rs` | UPDATE — `reverse_in_tx` + `reverse`, les `SELECT` qui listent les colonnes, **et `delete_all_by_company` (cf. D2 (a))** |
| `crates/kesh-db/src/repositories/projects.rs` *ou* le chemin d'insertion | UPDATE — l'exemption `enforce_project_taggable` (cf. D1) |
| `crates/kesh-api/src/routes/journal_entries.rs` | UPDATE — handler `reverse_journal_entry` |
| `crates/kesh-api/src/lib.rs` | UPDATE — route sous `comptable_routes` |
| `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl` | UPDATE — préfixe `journal-entries-` (64 clés `journal-` aujourd'hui en fr-CH) |
| `frontend/src/lib/features/journal-entries/journal-entries.{api,types}.ts` | UPDATE |
| `frontend/src/routes/(app)/journal-entries/[id]/+page.svelte` | UPDATE — bouton + confirmation + renvois |
| `crates/kesh-api/src/exports/csv_tables.rs` | UPDATE — ⛔ voir ci-dessous |
| `crates/kesh-api/tests/journal_entry_reversal_e2e.rs` | NEW |
| `frontend/tests/e2e/journal-entries.spec.ts` | UPDATE |

⛔ **L'export comptable complet perdrait le lien EN SILENCE.** `serialize_journal_entries_csv` (`exports/csv_tables.rs:240`) énumère ses colonnes **à la main** — `id`, `company_id`, `fiscal_year_id`, `entry_number`, `entry_date`, `journal`, `description`, `version`, `created_at`, `updated_at` — et **aucun test n'impose son exhaustivité vis-à-vis du schéma** : `backup_inventory_matches_schema` ne compare que la liste des **tables**, jamais les colonnes d'un CSV.

⚠️ **C'est l'artefact que l'expert-comptable ou le réviseur consulte.** Livrer la story sans y ajouter `reversesEntryId`, c'est produire un export où la correction n'est **pas apparente** — l'exigence même (art. 958f CO) que la story sert. Et rien ne rougirait : le mode d'échec muet de cette vague, reproduit dans un fichier que la story ne pensait pas toucher. Ajouter la colonne **et** le test qui l'exige. ⛔ En **`snake_case`** — `reverses_entry_id` : cet en-tête est un export **table**, pas une réponse JSON, et ses dix colonnes actuelles sont toutes en snake_case.

⚠️ **Il n'existe AUCUN fichier de test d'intégration dédié aux écritures** — `journal_entries.rs` n'a que des tests unitaires (`mod tests` à `:1299`, **zéro `#[sqlx::test]`**), et seul `idor_multi_tenant_e2e.rs` touche l'endpoint (4 occurrences). Le fichier E2E est donc à **créer**, et c'est la couverture de cette story qui portera aussi la 24-4b.

### Pièges vérifiés au sol

- ⚠️ **`chk_jel_debit_credit_exclusive`** interdit `debit = 0 AND credit = 0`. Une ligne d'origine à montant nul est donc **impossible** — inutile de la traiter, mais ne pas introduire de ligne nulle en inversant.
- ⚠️ **`uq_jel_entry_order UNIQUE (entry_id, line_order)`** : `line_order` porte contractuellement la position d'une ligne (1, 2, 3…). C'est la clé d'ordre de la reprise positionnelle des projets — **pas** `id`, que le gabarit `cancel` utilise faute d'avoir à conserver quoi que ce soit par position.
- ⚠️ **`uq_journal_entries_number (company_id, fiscal_year_id, entry_number)`** : la contre-passation consomme un numéro dans l'exercice **courant**, pas dans celui de l'origine.
- ⚠️ **`journal_entry_lines` n'a AUCUN libellé** (leçon 24-1) — il vit sur l'en-tête.
- ⚠️ `create_in_tx` valide les projets par-ligne **avant** le lock `fiscal_years` (ordre `companies → projects → fiscal_years`, Pattern 5) : ne pas prendre de verrou dans un autre ordre sous peine d'ABBA inter-flux.
- ⚠️ **Sélecteurs E2E** : la garde KF-043 (#326) refuse un sélecteur figé sur un libellé traduit **et** refuse les entrées mortes de `DETTE_CONNUE`. Le nouveau bouton se cible par `data-testid`. `journal-entries.spec.ts` a déjà trois entrées de dette (`Annuler`, `Supprimer`, `Valider`) : ne pas en ajouter.
- ✅ **Le `409` sur double contre-passation ne demande AUCUN code neuf** : `map_db_error` traduit déjà le code MariaDB **1062** en `DbError::UniqueConstraintViolation` (`errors.rs:300`), et le test `db_unique_constraint_maps_to_409` prouve le mappage HTTP. Il reste à **discriminer par le nom de la contrainte** — `uq_journal_entries_reverses` — pour rendre `ALREADY_REVERSED` plutôt que le `RESOURCE_CONFLICT` générique, exactement comme le fait déjà `uq_accounts_company_singleton_role` (`errors.rs:2265-2280`).
- ⚠️ **`i18n-keys.test.ts` compte les SITES d'usage** (`ATTENDU.sitesTotal`, 1613 aujourd'hui) et rougira. Sa ventilation se documente dans le doc-comment, avec le delta et son motif.

### Références

- `journal_entries.rs:843` (`update`, garde `Closed` seule) · `:981-1023` (réécriture destructive) · `:1139`/`:1162` (`delete_by_id` / `delete_in_tx`) · `:160-186` (contrat `create_in_tx`, `enforce_postable`)
- `supplier_invoices.rs` — `pub async fn cancel`, le gabarit d'inversion `D ↔ C`
- `invoice_settlements.rs:129-141` — `amount_due`, le résiduel calculé hors écriture
- `20260412000001_journal_entries.sql` — schéma, contraintes, index
- `lib.rs:332-339` (montage `comptable_routes`) · `:558-576` (routes de réconciliation, sans dé-rapprochement)
- Issue **#380** ; Epic `_bmad-output/planning-artifacts/epic-24-vague1-livres-justes.md`
- `CLAUDE.md` §§ *Migration breaking policy* (P1, P5-P8), *Test Locally First*, *Review Iteration Rule*

## Journal de revue

### Passe 1 — 2026-08-28 · Sonnet 4.6 + Haiku 4.5, contextes frais, orthogonales à l'auteur (Opus 5)

**2 CRITICAL, 1 HIGH, 6 MEDIUM retenus · 3 findings réfutés au sol.**

| # | sév. | lentille | ce qui manquait |
|---|---|---|---|
| S1-1 | CRITICAL | Sonnet | le projet se reprend **par ligne**, pas comme valeur unique — le gabarit `cancel` ne s'applique PAS ici |
| EC-1 | CRITICAL | Haiku | `active = TRUE` est **inconditionnel** : un compte archivé rendait l'écriture incontre-passable, sans message utile |
| S1-2 | HIGH | Sonnet | l'**export comptable complet** perdait le lien de contre-passation en silence |
| S1-3 | MEDIUM | Sonnet | le `PUT` reste ouvert entre 24-4a et 24-4b — risque désormais assumé **et tracé** vers la 24-4b |
| S1-4 | MEDIUM | Sonnet | tester les **six chemins FK**, pas les huit codes : deux FK partagent `OWNED_BY_SUPPLIER_INVOICE` |
| S1-5 | MEDIUM | Sonnet | la liste paginée doit calculer les champs dérivés en **une** requête |
| EC-3 | MEDIUM | Haiku | l'écriture d'ouverture : contre-passable, et c'est désormais écrit comme délibéré |
| EC-5 | MEDIUM | Haiku | le format **exact** du libellé, qui était laissé à l'interprétation |
| EC-4 | LOW→retenu | Haiku | le `404` sur `id` inconnu ou d'une autre société |

⚠️ **Le motif de la passe est net : les deux CRITICAL viennent de la même faute — avoir cru un gabarit transposable sans vérifier son modèle de données.** `cancel` porte un projet **document-level** parce qu'une facture n'en a qu'un ; une écriture manuelle porte des tags **par ligne**. Et `enforce_postable = false` a été pris pour « aucune garde de compte », alors qu'il ne gouverne que `postable`.

**Réfutés au sol, consignés pour ne pas être re-signalés :**

- **EC-2 (HIGH) — « exercices ouverts se chevauchant, choix non déterministe »** : `find_open_covering_date` n'a effectivement pas d'`ORDER BY` (`fiscal_years.rs:508-512`), mais **deux exercices ne peuvent pas se chevaucher** — un pré-check `find_overlapping FOR UPDATE` le refuse à la création (`fiscal_years.rs:141`). Le `LIMIT 1` porte sur au plus un élément.
- **EC-6 (MEDIUM) — « pas de mappage 1062 → 409 »** : il existe (`errors.rs:300` + test `db_unique_constraint_maps_to_409`). Reste la discrimination par nom de contrainte, ajoutée aux pièges.
- **`payment_batches` / `opening_balances` comme septième et huitième propriétaires** : aucune FK vers `journal_entries` ; le chemin des lots passe par `supplier_invoices.settlement_journal_entry_id`, déjà couvert.

### Passe 2 — 2026-08-28 · Opus 5, contexte frais, braquée sur la remédiation de la passe 1

**0 CRITICAL, 4 HIGH, 6 MEDIUM, 1 LOW · et 8 findings sur 11 naissent du patch de la passe 1.**

| # | sév. | origine | ce qui clochait |
|---|---|---|---|
| P2-1 | HIGH | **patch P1** | le patch a corrigé le compte archivé et laissé, **dans le même paragraphe**, le projet archivé à l'état de question ouverte — alors que `validate_taggable_in_tx` **rejette** (`projects.rs:115-118`) |
| P2-2 | HIGH | conception | `FiscalYearInvalid` rend **400**, pas 409 (`kesh-api/src/errors.rs:2482`) |
| P2-3 | HIGH | **patch P1** | le patch dit « on copie le gabarit de l'avoir » puis fixe **409** là où le gabarit rend **400** (`:2472`) — les deux moitiés du même correctif se contredisaient |
| P2-4 | HIGH | conception | la FK auto-référente `RESTRICT` casse `delete_all_by_company` de façon **intermittente** (un seul `DELETE`, FK vérifiées ligne à ligne) et change le `DELETE` unitaire sans qu'aucun AC ne le dise |
| P2-5 → P2-10 | MEDIUM | **patch P1** | dénombrement de jointures périmé par un patch voisin ; `reversalBlockedBy` scalaire sans précédence alors que les causes cumulent ; requête de reprise des lignes non écrite ; libellé encore ambigu sur deux points ; `reversesEntryId` en camelCase dans un CSV en snake_case ; champs dérivés exigés sur une liste qu'aucun écran ne consomme |
| P2-11 | LOW | conception | « six FK » (il y en a sept, dont une interne) ; `:1188` au lieu de `:1226` ; `errors.rs` désignant deux fichiers de crates différents |

⚠️ **Le motif codifié du `CLAUDE.md` s'est vérifié une fois de plus : la sévérité s'est déplacée vers ce qui venait d'être écrit.** Trois correctifs de la passe 1 se contredisaient entre eux ou contredisaient le code qu'ils invoquaient en modèle.

⛔ **Le plus instructif est P2-1** : le patch EC-1 a corrigé le compte archivé **et écrit noir sur blanc que le cas du projet archivé restait à vérifier** — c'est-à-dire qu'il a délégué au développeur une question dont la réponse tenait en deux `grep`. Une spec ne demande pas de découvrir un cas : elle le tranche.

**Arbitrages rendus dans ce patch :**

- le projet archivé est **toléré** (exemption), le compte archivé **refusé** — asymétrie voulue, désormais expliquée en D1 ;
- la FK garde `RESTRICT`, et la story assume ses **deux** conséquences : `delete_all_by_company` remet le lien à `NULL` avant la suppression en masse, et le `DELETE` unitaire d'une origine contre-passée devient un **409 explicite** ;
- les champs dérivés sont **retirés de la liste** paginée — ce qui supprime du même coup l'exigence de requête unique et son test anti-N+1 ;
- les statuts sont figés : **409** pour un refus de propriété, **400** pour un compte archivé ou l'absence d'exercice ouvert ;
- les 19 critères sont **renumérotés en continu** (les `9-bis` et `13-bis` de la passe 1 ont disparu).

**Prochaine passe** : la sévérité reste `> LOW` (4 HIGH), donc passe 3 obligatoire. La sévérité **maximale recule** (CRITICAL → HIGH), donc le critère de splitting préventif n'est pas atteint. Huit findings sur onze portant sur la remédiation, la passe 3 sera **ciblée** — une seule lentille sur le seul commit de remédiation de la passe 2, prompt versionné à côté de ce fichier.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
