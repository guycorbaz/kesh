# Story 48-12a : Refus serveur des factures fournisseur en double (numéro comparé par exercice)

Status: drafted (validation à faire ; `ready-for-dev` se pose à la clôture de la validation, par l'orchestrateur)

Issue : **#975** (côté serveur) — `bug`, **P2**. Release 0.20.0, Epic 48. Voie : **base et Rust**. Pas de migration. **La 48-12b (écran, #975 côté écran et #985) dépend de cette fiche.** Fiche-mère : `48-12-reponses-perdues-fournisseurs.md` (inventaire, reports 0.21.0, historique).

## Story

En tant que comptable qui tient des factures fournisseur réelles (dettes, lots de paiement pain.001),
je veux que **le serveur** refuse d'enregistrer deux fois la même facture fournisseur (même numéro chez le même fournisseur dans le même exercice, ou même date et même montant sans numéro à comparer), quel que soit le chemin par lequel elle arrive (écran, complétion d'un document importé, API par clé),
afin que mes dettes restent justes et qu'aucune facture ne soit payée deux fois.

**Usage réel.** Guy saisit ses factures fournisseur et en tire des lots pain.001. Un doublon de brouillon, une fois validé, double la dette (compte 2000) et peut être payé deux fois. Une réponse perdue suivie d'un second clic (#975) n'est qu'une cause parmi d'autres (double saisie, document déjà saisi à la main, intégration par clé d'API) : la règle se pose donc **là où l'écriture se fait**. Cette fiche livre **le serveur** ; l'affichage du refus et la confirmation à l'écran sont la **48-12b**, qui en dépend.

## Ce qui se lit au code

Tout est lu à `af4e0e313` (tête de la branche d'épique, 48-13 et 48-15 fusionnées).

1. **Aucun rejeu automatique d'une écriture.** `frontend/src/lib/shared/utils/api-client.ts:321-322` : `isIdempotentMethod` ne vaut vrai que pour `undefined`, `GET`, `HEAD`. Un `POST` ou un `PUT` dont la réponse se perd n'est jamais renvoyé par le client ; c'est l'utilisateur qui le renvoie, par un second clic. Délai : `REQUEST_TIMEOUT_MS = 30_000` (`api-client.ts:42`).
2. **#975, création.** `new/+page.svelte:148-149` : `save(req)` appelle `createSupplierInvoice(req)` (`POST /supplier-invoices`,
   `supplier-invoices.api.ts:54-58`). Le corps (`CreateSupplierInvoiceRequest`, `supplier_invoices.rs:337-349`) ne porte ni
   version ni clé. Le serveur crée un brouillon à chaque appel (`create_supplier_invoice`, `supplier_invoices.rs:731-752`).
   `SupplierInvoiceForm.svelte:465-474` attrape l'erreur et l'affiche sans rien relire ; `saving` retombe à faux
   (`:475`), le bouton se réactive (`:977`). Un `TIMEOUT` ou un `NETWORK_ERROR` après le commit donne donc un second
   brouillon au second clic. Aucun garde-fou serveur : `supplier_invoices` n'a pas d'unicité sur (société, fournisseur, numéro)
   (`migrations/20260628000001_supplier_invoices.sql`).
3. **Le serveur n'a aucun contrôle de doublon.** `create_draft_in_tx` (`crates/kesh-db/src/repositories/supplier_invoices.rs:650`) enchaîne
   `check_draft_input_in_tx` (`:502` : lignes non vides, projet, fournisseur `is_supplier`, forme des lignes, comptes de charge), le calcul
   du total (`draft_total`), l'`INSERT` (`:657-679`), les lignes (`insert_lines`), la relecture et l'audit `supplier_invoice.created` (`:684-693`).
   `supplier_invoices` n'a **aucune unicité** sur (société, fournisseur, numéro) (`migrations/20260628000001_supplier_invoices.sql`). Les deux seules
   écritures d'un brouillon passent par `create_draft_in_tx` et `update_draft_in_tx` (`:756`) ; leurs appelants hors tests : `routes/supplier_invoices.rs:740` (création, enveloppée par
   `kesh_db::retry::retry_on_deadlock`), `:817-834` (modification : `lock_draft_in_tx` `FOR UPDATE` sur la facture puis `update_draft_in_tx`),
   `routes/imported_supplier_invoices.rs:203` (complétion : staging verrouillé, `new_from_document`, puis `create_draft_in_tx`) ; `kesh-seed/src/pieges.rs:613` appelle `create_validated`
   (données de démonstration). Le service MCP des factures fournisseur ne crée ni ne modifie (`services/factures_fournisseur.rs` : lecture).
4. **Un précédent exact : le refus 409 qu'on lève par un champ de confirmation.** `accounts::archive` reçoit `confirm_designated: bool`
   (`crates/kesh-db/src/repositories/accounts.rs:1249`) et rend `DbError::AccountDesignatedInSettings { … }` (`:1290-1292`, déclarée `kesh-db/src/errors.rs:1584`, code `:2488`) ;
   la route lit `confirm_designated` du corps (`routes/accounts.rs:225`, `:540`) ; `kesh-api/src/errors.rs:4428` la range dans un sous-match **exhaustif** sur `DbError`
   (« pas de `_ =>` », `:4228-4230`) ; l'API le documente (`docs/api-external.md`, `confirmDesignated`). La règle neuve suit ce patron.
5. **Verrous.** Aucun code ne prend `FOR UPDATE` sur `contacts` (`git grep -n "FOR UPDATE" -- crates/kesh-db/src/repositories | grep -i contacts` ne rend qu'un commentaire, `contacts.rs:562`) ;
   `check_draft_input_in_tx` est écrit **sans verrou** (doc-comment `:493`, C-v017-4-28) et la création ne prend que les verrous **partagés** des clés étrangères, d'où le rejeu sur interblocage
   (`retry_on_deadlock`, `retry_app_on_deadlock` pour le `PUT` et la complétion, C-v017-4-37). La règle devra donc **prendre un verrou exclusif** (D3) et le dire.
6. **L'exercice d'une date de facture.** `fiscal_years` (colonnes `start_date`, `end_date`, `status`) : `find_covering_date_in_tx` (`crates/kesh-db/src/repositories/fiscal_years.rs:964`) rend l'exercice qui couvre une date, **quel que soit son statut**, par une lecture **non verrouillante** (doc-comment `:944-962`) ; la validation d'une facture fournisseur exige un exercice **ouvert** (`find_open_covering_date`, `repositories/supplier_invoices.rs:1760`, sinon `DbError::FiscalYearInvalid`) ; **un brouillon hors de tout exercice s'enregistre** (doc-comment de `check_draft_input_in_tx`, `:499` : « un brouillon hors de tout exercice s'enregistre »). Kesh crée par défaut un premier exercice sur l'**année civile** (manuel, `user-manual.tex:224`).
7. **Contrat de l'API.** `docs/api-external.md:659` (`POST /api/v1/supplier-invoices`, « inchangé »), `:1015` (complétion : corps de `POST /supplier-invoices`), `:1021` (validation des coordonnées de paiement) ;
   `docs/manual/fr/admin-manual.tex` : un paragraphe « Pour les intégrations » par version (`:2588`, `:2618`, `:2665`). Les actions d'audit se déclarent dans `kesh-api/src/audit_labels.rs:209-214`
   et se traduisent (`messages.ftl:3541-3548`, quatre locales) ; `audit_acteur_garde.rs:364`, `:370` suit `create_draft_in_tx` et `update_draft_in_tx`.

## Décisions

**D1. La règle anti-doublon (décisions de Guy des 2026-11-19).** Pour toute **création** et toute **modification** d'une facture fournisseur, sous verrou, sans migration :
1. **Même société, même fournisseur (`contactId`), même numéro de facture du fournisseur** (non vide, comparé après normalisation, D2), **dans le même exercice comptable** → **refus absolu** `409 SUPPLIER_INVOICE_DUPLICATE`, sans exception ni sortie. L'exercice est celui de **la date de facture** (« la période », D2bis) : deux factures de même numéro dans deux exercices différents ne sont **jamais** un doublon au critère 1 (la numérotation d'un fournisseur qui repart à un chaque année est ainsi respectée) ;
2. **Même société, même fournisseur, même date de facture, même total TTC** → **refus par défaut** `409 SUPPLIER_INVOICE_PROBABLE_DUPLICATE`, **avec une sortie explicite** : un champ de requête `confirmNotDuplicate: true` que l'écran n'envoie qu'après le clic sur « Ce n'est pas un doublon, enregistrer » ; la confirmation est **inscrite au journal d'audit** (D5). Même date implique même exercice : l'exercice n'ajoute rien à ce critère ;
3. **si les deux factures portent un numéro du fournisseur, seul le numéro décide** : deux numéros différents ne sont **jamais** un doublon, même date et même montant ; le critère 2 ne s'applique que si **l'une au moins** des deux n'a pas de numéro ;
4. les factures **annulées** (`status = 'cancelled'`) sont **exclues** de la comparaison ; brouillons, validées et payées comptent ;
5. le **numéro reste facultatif** ; les données existantes ne bougent pas : le contrôle ne vaut que pour une nouvelle saisie ou une modification (D6).
Les deux refus portent `details : { supplierInvoiceId, internalNumber, status }` (identifiant, numéro interne — `null` pour un brouillon jamais validé — et statut de **la facture existante** ; s'il y en a plusieurs, celle d'identifiant le plus petit).
*Écartée — relecture et signalement côté écran* (conception précédente) : elle ne protège ni la double saisie, ni la complétion, ni l'API, et laisse une fenêtre quand la requête est encore en vol. *Écartée — clé d'idempotence durable* : migration, exclue de la release. *Écartée — contrainte `UNIQUE` en base* : migration, et elle ne porterait pas la règle 2. *Écartée (décision de Guy) — comparer le numéro sur toute l'histoire du fournisseur* : refuserait à tort une numérotation qui repart à un chaque année.

**D2. Normalisation et comparaison.** Une fonction **pure** de `kesh-db` (testée seule) normalise le numéro : tous les blancs retirés (`char::is_whitespace`, plus U+200B et U+FEFF, l'ensemble de `is_payment_blank`, `routes/supplier_invoices.rs:378`), puis `to_lowercase()` ; vide après normalisation = **absent**. Le total comparé est le **total TTC calculé** (`draft_total`) du côté nouveau, et `total_amount` stocké du côté existant, en égalité de valeur décimale. La comparaison se fait **en Rust** sur les factures **du même fournisseur** (`SELECT id, internal_number, supplier_invoice_number, status, invoice_date, total_amount FROM supplier_invoices WHERE company_id = ? AND contact_id = ? AND status <> 'cancelled'` et, à la modification, `AND id <> ?`) : une normalisation SQL des blancs Unicode serait fragile, et le nombre de factures d'un fournisseur reste petit. Ordre de décision : **d'abord** le refus absolu (règle 1, **à période égale**) sur toutes les candidates, **puis** le refus probable (règle 2, avec la règle 3) ; la facture citée est la plus petite en identifiant parmi celles du même refus.

**D2bis. La période d'une date (règle 1, décision de Guy du 2026-11-19, option (a)).** Une fonction **pure** `periode_de(date, exercices)` rend la **clé de période** d'une date de facture, où `exercices` est la liste `(id, début, fin)` des exercices de la société, lue **une fois** par un `SELECT id, start_date, end_date FROM fiscal_years WHERE company_id = ?` **sans verrou** (tout statut, ouvert ou clos, comme `find_covering_date_in_tx`) :
- si un exercice couvre la date (`début ≤ date ≤ fin`) : `Exercice(id)` ;
- **si la date tombe hors de tout exercice** (Kesh accepte pourtant ce brouillon, point 6 : seule la validation l'exige ouvert) : `AnneeCivile(année de la date)`. *Motif de ce choix* : le refus ne doit **ni dépendre de l'existence d'un exercice** (le brouillon s'enregistre, la règle aussi), **ni supprimer la comparaison** (un doublon hors exercice passerait, puis serait validé plus tard dans l'exercice créé), et l'année civile est le découpage que Kesh propose par défaut (point 6) ;
- deux dates sont de **même période** si leurs clés sont **égales** : deux `Exercice` de même identifiant, ou deux `AnneeCivile` de même année ; une date dans un exercice et une date hors exercice ne sont **jamais** de même période.
*Écartée — ne pas appliquer le critère 1 hors exercice* : laisse passer des doublons de brouillons non encore rattachés à un exercice. *Écartée — un seul seau « hors exercice »* : mélangerait des années éloignées et refuserait à tort.

**D3. Verrou.** Avant la comparaison, la transaction prend `SELECT id FROM contacts WHERE id = ? AND company_id = ? FOR UPDATE` sur **la ligne du fournisseur** : deux envois simultanés pour le même fournisseur se **sérialisent**, le second voit la facture du premier à la comparaison et est refusé. La lecture des exercices (D2bis) reste **sans verrou** (patron de `find_covering_date_in_tx`, qui évite d'inverser l'ordre des verrous). Le rejeu sur interblocage des trois chemins existe déjà (point 5 du code). L'ordre des verrous est écrit au doc-comment : **création** : fournisseur puis `INSERT` ; **modification** : facture (`lock_draft_in_tx`) puis fournisseur ; **complétion** : staging puis fournisseur. Un test de concurrence (deux créations simultanées, un seul succès) et une mutation « sans verrou » le prouvent. *Écartée — verrou sur la société (`companies`)* : sérialise toutes les créations de la société. *Écartée — pas de verrou* : deux envois simultanés passeraient (« TOCTOU »).

**D4. Ordre des refus.** La comparaison se fait **après** `check_draft_input_in_tx` (lignes, projet, fournisseur, forme, comptes : les refus de saisie passent d'abord, et le total n'est calculable qu'après) et **avant** l'`INSERT`. Pour la complétion, `ImportNotPendingCompletion` (statut du staging), puis les règles « document » (`new_from_document`), puis cette règle : un document déjà complété rend toujours `409 IMPORT_NOT_PENDING_COMPLETION`, jamais un refus de doublon. Un refus **annule** la transaction : le document reste `to_complete`, rien n'est écrit.

**D5. Confirmation et audit.** `CreateSupplierInvoiceRequest` (donc le corps du `PUT`, qui l'aplatit, `routes/supplier_invoices.rs:654-658`, et celui de la complétion) reçoit `confirm_not_duplicate: bool` (`#[serde(default)]`, `confirmNotDuplicate`), transmis au repository sous la forme d'un paramètre explicite (patron `confirm_designated`, `DuplicatePolicy { Refuse, Confirmed }`) : `create_draft_in_tx` et `update_draft_in_tx` gardent leur nom, reçoivent la politique, et les appelants de tests et de démonstration passent `Refuse`. La confirmation **ne lève jamais le refus absolu** (règle 1). Quand la règle 2 **aurait refusé** (une facture existante répond au critère) et que `Confirmed` la laisse passer, le repository écrit, **dans la même transaction**, une entrée d'audit `supplier_invoice.duplicate_confirmed` (acteur, facture créée ou modifiée, `details : { supplierInvoiceId, internalNumber }` de la facture existante qui aurait refusé). Elle se déclare dans `audit_labels.rs` et se traduit dans les quatre catalogues (`audit-log-action-supplier-invoice-duplicate-confirmed`). Un `confirmNotDuplicate: true` sans doublon probable n'écrit **rien** de plus.

**D6. À la modification.** La règle (décision de Guy du 2026-11-19) ne se rejoue que si l'un des quatre éléments qui la fondent **change** par rapport à la facture avant modification : **fournisseur, numéro (normalisé), date de facture, total TTC** (un changement de date qui fait **changer de période** compte, D2bis). Sans changement de ces quatre, la modification passe (corriger une description, une échéance ou un IBAN d'une facture qui était déjà, avant 0.20.0, le double d'une autre ne doit pas être bloqué : « les données existantes ne bougent pas »). La facture modifiée est exclue de la comparaison (`id <> ?`). Même verrou (D3).

**D7. Contrat de l'API (changement à signaler).** Deux codes neufs, `409 SUPPLIER_INVOICE_DUPLICATE` et `409 SUPPLIER_INVOICE_PROBABLE_DUPLICATE`, avec `details` ; un champ de requête neuf, facultatif, `confirmNotDuplicate`. Une intégration qui créait deux fois la même facture **reçoit désormais un refus** ; le critère 1 ne compare que les factures **du même exercice** (période de la date de facture). Documenté dans `docs/api-external.md` (`POST` et `PUT /supplier-invoices`, complétion `POST /imported-supplier-invoices/{id}/complete`, au même endroit que « Les coordonnées de paiement se valident à un seul endroit »), dans le paragraphe « Pour les intégrations » du manuel d'administration, et à la **Note de version** (« changement de contrat »). Messages (clés `error-supplier-invoice-duplicate`, `error-supplier-invoice-probable-duplicate`, quatre locales) : une phrase pour l'utilisateur, dans la langue de l'installation, sans le numéro de la facture existante dans le texte (il est dans `details`).


## Critères d'acceptation

1. **AC-1 (règle 1).** Une création, une modification ou une complétion dont le fournisseur et le numéro normalisé (blancs retirés, casse ignorée) sont ceux d'une facture non annulée **du même exercice** rend `409 SUPPLIER_INVOICE_DUPLICATE`, `details.supplierInvoiceId` / `internalNumber` / `status` de l'existante ; **rien n'est écrit** ; `confirmNotDuplicate: true` n'y change rien.
2. **AC-2 (exercice, option (a)).** Deux factures de même fournisseur et de même numéro (par exemple « 15 ») dans **deux exercices différents** sont acceptées ; dans le **même** exercice, la seconde est refusée. Une date hors de tout exercice se compare **par année civile** : « 15 » en 2023 et « 15 » en 2023 hors exercice sont refusées ; « 15 » en 2023 hors exercice et « 15 » en 2024 hors exercice sont acceptées ; une date dans un exercice et une date hors exercice ne sont jamais de même période. Les exercices clos comptent comme les ouverts.
3. **AC-3 (règle 2).** Sans numéro à comparer (l'un au moins des deux n'en a pas), même fournisseur, même date, même total TTC : `409 SUPPLIER_INVOICE_PROBABLE_DUPLICATE` ; avec `confirmNotDuplicate: true` la facture s'enregistre et **une entrée d'audit `supplier_invoice.duplicate_confirmed`** est écrite dans la même transaction ; sans doublon probable, `confirmNotDuplicate: true` n'écrit rien de plus.
4. **AC-4 (règle 3).** Deux factures portant **chacune** un numéro différent ne sont jamais un doublon, même date et même montant.
5. **AC-5 (règle 4).** Une facture annulée ne compte pas : on recrée une facture au numéro d'une annulée, ou aux mêmes date et total.
6. **AC-6 (règle 5 et D6).** Le numéro reste facultatif ; une modification qui ne change ni le fournisseur, ni le numéro, ni la date, ni le total passe même si la facture était déjà le double d'une autre ; une modification exclut la facture elle-même de la comparaison ; un changement de date qui change de **période** rejoue la règle 1.
7. **AC-7 (concurrence, D3).** Deux créations simultanées de la même facture : **une seule** réussit, l'autre rend le refus attendu ; aucun interblocage ne remonte (rejeu existant).
8. **AC-8 (ordre, D4).** Un refus de saisie (ligne, compte, projet, fournisseur non fournisseur) passe avant le refus de doublon ; pour la complétion, un document déjà complété rend `IMPORT_NOT_PENDING_COMPLETION` ; un refus de doublon laisse le document `to_complete`.
9. **AC-9 (chemins).** La règle vaut par la route de l'écran, la complétion et une **clé d'API `read-write`** (`POST` et `PUT /supplier-invoices`, `POST …/complete`) ; les données de démonstration (`kesh-seed`) et les tests existants **ne dépendent pas d'un doublon** (fixtures corrigées, jamais la règle).
10. **AC-10 (contrat et documentation).** `docs/api-external.md` décrit les deux codes, `details`, `confirmNotDuplicate` et la comparaison **par exercice** ; le manuel d'administration, « Pour les intégrations », le signale ; le manuel utilisateur décrit la règle (numéro identique refusé **dans le même exercice**, même date et montant sans numéro, annulées exclues) ; la Note de version dit « changement de contrat ». Aucune migration ; aucune facture client, aucun avoir, aucun PDF touché.
11. **AC-11.** Messages d'erreur et libellé d'audit dans les quatre locales ; gardes i18n et d'audit à jour.

## Tests à écrire, et la preuve du rouge

Chaque test neuf rougit sans le correctif ; journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e4812a/`. **Rouge d'abord** : chaque test est écrit et joué rouge avant le code, ou rougi par mutation.

| Test | Fichier | Mutation qui doit le rougir |
|---|---|---|
| normalisation : `"FA 2026-1"`, `"fa2026-1"`, `"FA\u{A0}2026-1"`, `" FA2026-1\u{200B}"` égales ; vide ou blancs seuls = absent | unitaire `kesh-db` (fonction pure) | retirer `to_lowercase`, ne retirer que l'espace ASCII, ne pas retirer U+200B |
| période : `periode_de` dans un exercice (début et fin inclus), exercice non civil (01.07 au 30.06, 30.06 et 01.07 de période différente), hors exercice (année civile), exercice clos | unitaire `kesh-db` | `Exercice` remplacé par l'année civile partout, bornes exclues, un seul seau hors exercice |
| règle 1 : même fournisseur, même numéro, même exercice → refus ; `details` exacts ; `confirm` sans effet ; autre fournisseur ou autre société → accepté | `kesh-db/tests/supplier_invoice_duplicate.rs` (neuf) | omettre le filtre `contact_id`, omettre le filtre société, laisser `Confirmed` lever la règle 1 |
| **exercice : deux factures n° 15 d'exercices différents → acceptées ; même exercice → refus ; hors exercice : même année civile → refus, années différentes → accepté ; une dans un exercice, une hors exercice → accepté** | idem | « ne pas comparer la période » (règle 1 sur toute l'histoire), « année civile partout » |
| règle 2 : sans numéro, même date et même total → refus probable ; `Confirmed` passe et écrit `supplier_invoice.duplicate_confirmed` ; total ou date différents → accepté | idem | omettre le total, omettre la date, ne pas écrire l'audit |
| règle 3 : deux numéros différents, même date et même total → accepté ; un numéro et un absent, même date et total → refus probable | idem | appliquer la règle 2 aux deux numéros |
| règle 4 : annulée exclue (numéro et date/total) ; brouillon, validée, payée comptent | idem | retirer `status <> 'cancelled'` |
| modification : exclut la facture elle-même ; changement de fournisseur, de numéro, de date (dont changement de période), de total rejoue la règle ; aucun changement des quatre = passe même si doublon préexistant | idem | retirer `id <> ?`, rejouer toujours la règle |
| concurrence : deux `tokio::spawn` de `create_draft` identiques → un `Ok`, un refus | idem | retirer le `FOR UPDATE` du fournisseur |
| ordre : refus de saisie (compte non imputable) avant le refus de doublon ; refus de doublon ne laisse aucune ligne | idem | comparer avant `check_draft_input_in_tx` |
| API : `POST` deux fois → `201` puis `409` (codes, `details`) ; avec `confirmNotDuplicate` ; `PUT` ; clé `read-write` ; complétion : doublon → `409`, document `to_complete` ; second passage → `IMPORT_NOT_PENDING_COMPLETION` | `kesh-api/tests/supplier_invoice_duplicate_e2e.rs` (neuf) | ignorer le champ `confirmNotDuplicate` du corps |
| audit : `supplier_invoice.duplicate_confirmed` listée (`audit_labels.rs`), traduite quatre locales, garde `audit_acteur_garde.rs` à jour | gardes existantes | retirer la traduction d'une locale |

**Étape 0 obligatoire** : jouer `scripts/test-fast.sh --no-lint` **avant** le code pour relever les tests existants qui créent deux factures au même fournisseur avec le même numéro dans le même exercice, ou avec la même date et le même total (`kesh-db/tests/supplier_invoices_*`, `supplier_invoice*e2e`, `kesh-seed/src/pieges.rs:613`) ; leurs **fixtures** se corrigent (numéros distincts, exercices distincts, ou `Confirmed` pour la règle 2) ; la règle 1 n'a pas de sortie. Chaque mutation est suivie d'un `touch` du fichier restauré (cargo garde le binaire muté).

## Tâches

S1 : `DbError` (deux variantes, codes, `kesh-db/src/errors.rs`) et sous-match exhaustif de `kesh-api/src/errors.rs`. S2 : fonctions pures (normalisation, période, comparaison) ; `DuplicatePolicy` ; contrôle sous verrou dans `create_draft_in_tx` et `update_draft_in_tx` (D2 à D6) ; audit `supplier_invoice.duplicate_confirmed`. S3 : `routes/supplier_invoices.rs` (champ `confirm_not_duplicate`, transmission à la création et au `PUT`) et `routes/imported_supplier_invoices.rs` (complétion). S4 : `audit_labels.rs`, catalogues (quatre locales : messages d'erreur, libellé d'audit). S5 : `docs/api-external.md`, manuel d'administration, manuel utilisateur (règle). S6 : tests Rust et correction des fixtures. C0 : étape 0. C1 : gates. C2 : Dev Agent Record, Change Log, Note de version.

## Manuels et documentation

Seul `docs/manual/fr/user-manual.tex` existe pour l'utilisateur (`docs/manual/{de,it,en}/` ne portent qu'un `README.md`) : **aucun équivalent DE/IT/EN à écrire**. Les ajouts portent « depuis la version~0.20.0 » (`docs-numeros-de-release`).
- **Manuel utilisateur** (`user-manual.tex`, saisie `:2072-2090`, brouillon `:2195`) : la règle (numéro du fournisseur identique **dans le même exercice** = refusé sans exception ; même date et même montant sans numéro à comparer = refusé, levé à la saisie par la 48-12b ; annulées exclues ; la numérotation qui repart à un chaque année est respectée). Le bouton et l'avis de l'écran sont décrits par la 48-12b.
- **Manuel d'administration** (`admin-manual.tex`, un paragraphe « Pour les intégrations » par version, p. ex. `:2588`, `:2618`, `:2665`) : le changement de contrat (deux codes, `details`, `confirmNotDuplicate`, comparaison par exercice), à la section de la 0.20.0 (à créer si elle n'existe pas, avec l'orchestrateur).
- **`docs/api-external.md`** (`:659`, `:1015`, `:1021`) : voir D7. Les PDF ne se commitent pas.

## Gates

Selon `e0190-dev.md` et `e35-dev-commun.md` § *Quels gates, quand*. **Cette story modifie du Rust et des catalogues** : gate Rust complet.
- Entre les passes : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run -E 'binary(<le binaire touché>)'`.
- **Exception `kesh-db`** (`CLAUDE.md`) : dès qu'un patch touche un repository, **gate complet même en cours de boucle** ; base remise à zéro avant tout gate complet.
- **Au passage en `review`** : `scripts/test-fast.sh --ci`, puis `scripts/gate-ci.sh`, plus `npm run check` et `lint-i18n-ownership` (les catalogues sont partagés avec le frontend).
- **À la clôture** : `gate-ci` et **E2E complet** au dernier commit de code (montage `docs/testing.md`, `KESH_COOKIE_SECURE=false`, `KESH_TEST_MODE=true` des deux côtés) ; « vert » = les seuls échecs attendus. Aucun E2E neuf (aucun écran) : la 48-12b porte l'E2E du refus.
Sous `scripts/mem-guard.sh`, une commande lourde à la fois, `flock` du verrou de la voie (base et Rust).

## Note de version

**Changement de contrat de l'API.** Kesh refuse d'enregistrer deux fois la même facture fournisseur : même fournisseur et même numéro (comparé sans blancs ni casse) **dans le même exercice** = refus `409 SUPPLIER_INVOICE_DUPLICATE`, sans exception ; même fournisseur, même date et même total sans numéro à comparer = refus `409 SUPPLIER_INVOICE_PROBABLE_DUPLICATE`, qu'une intégration lève par `confirmNotDuplicate: true` (inscrit au journal d'audit) ; les factures annulées ne comptent pas ; deux factures de même numéro dans deux exercices différents sont acceptées. La règle vaut à la création, à la modification et à la complétion d'un document (refs #975).

## Dev Notes

- **Dépendance** : aucune ; la **48-12b** (écran, #975 côté écran et #985) dépend de cette fiche (codes, `details`, `confirmNotDuplicate`). Avant la 48-12b, l'écran affiche le refus par le message générique de l'API.
- **Modules** : quatre (`kesh-db`, `kesh-api`, `kesh-i18n`, documentation), au grain du fichier d'épique ; découpage décidé par Guy le 2026-11-19.
- **Fichiers communs** : `routes/supplier_invoices.rs` et `repositories/supplier_invoices.rs` avec la **48-1** (réimputation d'une facture fournisseur), le second arrivé résout ; catalogues i18n, `audit_labels.rs`, `audit_route_registry.rs`, manuels `.tex` avec les autres stories de l'Epic 48 et l'Epic 49 ; `InvoiceForm.svelte` (Epic 49) et `SupplierCreatePanel.svelte` (48-15) ne sont pas touchés.
- **Ce qui doit rester vrai** : aucune migration ; une facture client, un avoir et leur PDF inchangés ; **les données existantes ne bougent pas** (aucun backfill, aucun refus rétroactif) ; le refus absolu n'a pas de sortie ; la confirmation du refus probable est tracée à l'audit ; la lecture des exercices est sans verrou.
- **Hors périmètre** : l'écran (48-12b) ; #986 et le lot comptes bancaires, reportés à la 0.21.0 (fiche-mère `48-12-reponses-perdues-fournisseurs.md`) ; les autres écrans de création (inventaire de la fiche-mère).
- **Références** : `accounts::archive` et `confirm_designated` (#486, `accounts.rs:1249`, `:1290`) ; fiche 34-8.
- **Statut** : `drafted` ; `ready-for-dev` se pose à la clôture de la validation (orchestrateur, `sprint-status.yaml`).

## Dev Agent Record

(vide)

## Change Log

- 2026-11-19 — Création (Claude Sonnet 5.5, `bmad-create-story`) : fiche **découpée de la 48-12** (décision de Guy du 2026-11-19, découpage approuvé : 48-12a serveur, quatre modules, puis 48-12b écran). Reprend la règle anti-doublon de la refonte (`a5026a3f2`) **avec la décision de Guy du 2026-11-19 sur l'exercice** : le critère 1 compare les factures du **même exercice** (période de la date de facture) ; hors de tout exercice, l'**année civile** (D2bis, choix de la fiche, cohérent avec un brouillon hors exercice que Kesh accepte et un premier exercice qu'il propose sur l'année civile) ; la « conséquence » de la numérotation annuelle est retirée. D6 (rejeu à la modification seulement si fournisseur, numéro, date ou total change) approuvé par Guy. Historique des passes P1 à P6 : fiche-mère.
