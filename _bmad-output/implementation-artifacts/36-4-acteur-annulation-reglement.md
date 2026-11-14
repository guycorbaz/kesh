# Story 36-4 : l'annulation d'un règlement par une clé d'API est attribuée à la clé — par sa route ou par le dé-rapprochement (#1131)

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-4`, branche
`story/36-4-acteur-annulation-reglement`, par le workflow `bmad-create-story`. Phase 1, release **0.17.8**. **Aucune
migration** (colonnes `actor_type`, `actor_api_key_id` de la migration `20260605000002`, Story 17-2a). **`kesh-db` touché :
gate complet même en cours de boucle.** Voie **1** (lignée B, C-e360-4). Code lu à la tête du worktree, **`9d5a142df`**
(kickoff de l'Epic 36, sur `main` `42d5f368`). Registres : `e36-choix-autonomes-e360.md` (étape 0 de 36-4, C-e360-3, -7,
-9, -11) ; `e36-choix-autonomes-36-4.md` (C-36-4-1 à -4). Validation P1 à lancer. Patron : stories 35-8a1 et 35-8a2
(`Acteur`, `NewAuditLogEntry::par`, garde `audit_acteur_garde.rs`).*

**Issues** : la PR porte `refs #1131` (le mot-clé de fermeture, s'il en faut un, est l'affaire de la PR de release) et
`refs #431` (#431 reste ouverte : lots 36-11a à 36-11d). **Aucun mot-clé de fermeture** dans un commit.

⚠️ **La prémisse de #1131 est en partie dépassée** (étape 0, C-e360-7). L'issue dit que `reconciliation_cancel::cancel`
passe `None` à `cancel_in_tx` et que l'annulation d'un rapprochement perd la clé. Or la route du dé-rapprochement porte la
clé depuis la 35-8a2 ; `cancel` est une enveloppe **sans appelant de production**. La perte réelle est plus bas, dans les
deux `cancel_settlement_in_tx`, et elle touche **aussi** les deux routes d'annulation d'un règlement.

## Story

En tant que titulaire dont un agent écrit dans Kesh par une **clé d'API REST**, je veux que la piste de contrôle dise que
c'est **la clé** qui a annulé un règlement — un règlement ou un solde de facture client, le paiement d'une facture
fournisseur —, qu'elle l'ait fait par la route d'annulation du règlement ou en annulant le rapprochement bancaire qui
l'avait passé. Aujourd'hui, chaque trace de ce geste est au nom du créateur de la clé, sauf la ligne
`reconciliation.cancelled`. Le journal d'audit doit répondre juste à « qui a fait quoi » (CO art. 957-964, OLICo art. 9).

## Ce qui se lit au code (`9d5a142df`)

**Les deux fonctions qui perdent la clé** — chacune reçoit un `user_id: i64` nu et écrit **trois** sites sans clé :

- `crates/kesh-db/src/repositories/invoice_settlements_write.rs:917-924` — `pub async fn cancel_settlement_in_tx(tx,
  company_id, invoice_id, settlement_id, user_id: i64, aujourd_hui)` :
  - `:1016-1026`, le délettrage de la pièce : `letterings::dissolve_invoice_document_group_in_tx(…, Actor { user_id,
    api_key_id: None })` (`:1021-1024`) → `lettering.removed` (`letterings.rs:1425-1428`, `for_actor(actor.user_id,
    actor.api_key_id, …)`) ;
  - `:1029-1038`, la contre-passation : `journal_entries::reverse_owned_in_tx(…, Acteur::utilisateur(user_id), …)` (`:1034`,
    commentaire `:1033` : « Story 35-8a2 (#431) : au nom de l'utilisateur (`AConvertir`, Epic 36) ») →
    `journal_entry.reversed`, `journal_entry.created` de l'inverse, et le lettrage `reversal` du règlement avec son miroir
    (`lettering.created`) ;
  - `:1098-1115`, le geste : `NewAuditLogEntry::user(user_id, "invoice.settlement_cancelled", "invoice", …)` (`:1100-1102`).
- `crates/kesh-db/src/repositories/supplier_invoices.rs:2815-2821` — `pub async fn cancel_settlement_in_tx(tx, company_id,
  id, user_id: i64, aujourd_hui)` :
  - `:2894-2904`, `letterings::dissolve_supplier_invoice_document_group_in_tx(…, letterings::Actor { user_id, api_key_id:
    None })` (`:2899-2902`) ;
  - `:2909-2920`, `journal_entries::reverse_owned_in_tx(…, Acteur::utilisateur(user_id), …)` (`:2914`, même commentaire
    `:2913`) ;
  - `:2952-2967`, `NewAuditLogEntry::user(user_id, "supplier_invoice.settlement_cancelled".to_string(), …)` (`:2954-2956`).

**Leurs enveloppes** (transaction comprise), qui reçoivent aussi un `user_id` nu :
- `invoice_settlements_write.rs:1142-1149` — `pub async fn cancel_settlement(pool, user_id: i64, company_id, invoice_id,
  settlement_id, aujourd_hui)`, qui appelle `cancel_settlement_in_tx(&mut tx, company_id, invoice_id, settlement_id,
  user_id, aujourd_hui)` (`:1152-1159`) ;
- `supplier_invoices.rs:3003-3009` — `pub async fn cancel_settlement(pool, company_id, id, user_id: i64, aujourd_hui)`,
  `:3011` : `let result = cancel_settlement_in_tx(&mut tx, company_id, id, user_id, aujourd_hui).await;`.

**Les trois routes** (toutes dans `comptable_routes`, atteintes par une clé `read-write`) :
- `POST /api/v1/invoices/{id}/settlements/{settlement_id}/cancel` (`lib.rs:774-775`) →
  `routes/invoices.rs:1387` `cancel_invoice_settlement_handler`, qui passe `current_user.user_id` (`:1399`) à
  `invoice_settlements_write::cancel_settlement` (`:1397`), sous `retry_on_deadlock("invoices::cancel_settlement", …)`
  (`:1396`). La même route annule un **solde** (`settlementType = write_off`, `docs/api-external.md:563`, `:565`).
- `POST /api/v1/supplier-invoices/{id}/settlement/cancel` (`lib.rs:682-683`) → `routes/supplier_invoices.rs:1013`
  `cancel_supplier_invoice_settlement`, qui passe `current_user.user_id` (`:1027`) à `supplier_invoices::cancel_settlement`
  (`:1023`), sous `retry_on_deadlock` (`:1022`).
- `POST /api/v1/reconciliation/transactions/{id}/cancel` (`lib.rs:915-916`) → `routes/reconciliation.rs:5947`
  `post_cancel_reconciliation` : `let user_id = current_user.user_id;` (`:5954`), `let actor_api_key_id =
  current_user.api_key_id;` (`:5955`) → `cancel_reconciliation_once(…, user_id: i64, actor_api_key_id: Option<i64>, …)`
  (`:5977-5985`) → `reconciliation_cancel::cancel_in_tx(…, user_id, actor_api_key_id, …)` (`:5999-6007`). **La clé arrive
  donc jusqu'au dépôt** (prémisse de l'issue réfutée).

**Le dé-rapprochement** — `crates/kesh-db/src/repositories/reconciliation_cancel.rs:576-584` : `pub async fn
cancel_in_tx(tx, company_id, bank_transaction_id, user_id: i64, actor_api_key_id: Option<i64>, expect_unlink_only,
aujourd_hui)`. Étape (6), `:707-753` : retrait seul (`unlink_only`) → rien ; règlement client →
`invoice_settlements_write::cancel_settlement_in_tx(tx, company_id, invoice_id, settlement_id, user_id, aujourd_hui)`
(`:713-720`) — **la clé est perdue ici** ; règlement fournisseur → `supplier_invoices::cancel_settlement_in_tx(tx,
company_id, id, user_id, aujourd_hui)` (`:731-737`) — **et ici** ; écriture seule → `Acteur { user_id, api_key_id:
actor_api_key_id }` (`:744-747`) passé à `reverse_in_tx` (`:749`, 35-8a2). Étape (7) : `NewAuditLogEntry::for_actor(user_id,
actor_api_key_id, "reconciliation.cancelled", …)` (`:797-801`) — porte la clé. `user_id` et `actor_api_key_id` n'ont pas
d'autre usage dans le corps (`awk` sur `:576-827` : `:580`, `:581`, `:718`, `:735`, `:745`, `:746`, `:800`, `:801`).

**L'enveloppe de test** — `reconciliation_cancel.rs:829-859` : `pub async fn cancel(pool, company_id, bank_transaction_id,
user_id: i64, aujourd_hui)` (« mince enveloppement de [`cancel_in_tx`], sans verrou de compte ni rejeu (réservés à la
route) », `:829-830`) appelle `cancel_in_tx(&mut tx, company_id, bank_transaction_id, user_id, None, None, aujourd_hui)`
(`:839-847`). **Aucun appelant de production** (inventaire ci-dessous) ; c'est le site de l'angle mort 1 de la garde
(`audit_acteur_garde.rs:68-75` : « `reconciliation_cancel.rs` (`cancel` → `cancel_in_tx`) et `kesh-seed/src/pieges.rs`
(`invoices::mark_emailed`) aujourd'hui »).

**Le type** — `crates/kesh-db/src/entities/audit_log.rs:172-176` : `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct
Acteur { pub user_id: i64, pub api_key_id: Option<i64> }` ; `Acteur::utilisateur` (`:182`) ; `NewAuditLogEntry::par(acteur,
…)` (`:263`, délègue à `for_actor`) ; `letterings::Actor` est un alias (`letterings.rs:207` : `pub type Actor =
crate::entities::audit_log::Acteur;`) ; `impl From<&CurrentUser> for Acteur` (`crates/kesh-api/src/audit.rs:18-25`).
`Acteur` est `Copy` : il se capture tel quel dans la fermeture de `retry_on_deadlock`.

**La garde** — `crates/kesh-api/tests/audit_acteur_garde.rs`, table `SITES` (`:187-377`). Les **six entrées `AConvertir`**
que la story retire :

| Ligne | Fichier | Fonction | Motif | Nombre |
|---|---|---|---|---|
| `:294` | `invoice_settlements_write.rs` | `cancel_settlement_in_tx` | `User` | 1 |
| `:296` | `invoice_settlements_write.rs` | `cancel_settlement_in_tx` | `ApiKeyNone` | 1 |
| `:298` | `invoice_settlements_write.rs` | `cancel_settlement_in_tx` | `Utilisateur` | 1 |
| `:356` | `supplier_invoices.rs` | `cancel_settlement_in_tx` | `User` | 1 |
| `:358` | `supplier_invoices.rs` | `cancel_settlement_in_tx` | `ApiKeyNone` | 1 |
| `:360` | `supplier_invoices.rs` | `cancel_settlement_in_tx` | `Utilisateur` | 1 |

Table recomptée depuis la source à `9d5a142df` (`awk` sur les lignes `    ("crates/` du bloc `:187-377`) : **93 entrées, 98
sites** — `AConvertir` 53/54, `AdminSansCle` 27/29, `PreAuth` 4/4, `Systeme` 2/3, `Definition` 3/3, `Restauration` 1/2,
`Mcp` 2/2, `Fixture` 1/1 ; par motif `User` 71/76, `ApiKeyNone` 10/10, `Utilisateur` 12/12 (identique au relevé de clôture
de la 35-8a2). **Attendu après la story : 87 entrées, 92 sites** — `AConvertir` 47/48 ; `User` 69/74, `ApiKeyNone` 8/8,
`Utilisateur` 10/10. Aucune entrée n'existe pour `reconciliation_cancel.rs` (`grep` sur la table : aucune ligne). `REFERENCES`
(`:387-468`) ne porte aucune des fonctions touchées (elles sont `AConvertir`, non finales). Angle mort 1 (`:68-75`) : `git
grep -nE 'actor_api_key_id:\s*Option<i64>' -- crates/` hors `tests/` rend **30** lignes à `9d5a142df`, dont
`reconciliation_cancel.rs:581` (`kesh-gate-logs/e36-4/actor-api-key-id-option.txt`).

**Les tests qui lisent ces fonctions par leur nom** (le nom ne change pas ; l'ordre des appels non plus) :
- `crates/kesh-db/tests/letterings_lexical.rs:437-447` : `appelle_avant(&source, "cancel_settlement_in_tx",
  "dissolve_invoice_document_group_in_tx(", "reverse_owned_in_tx(")` ; `:627-635` : les trois écrivains connus du règlement
  fournisseur, dont `supplier_invoices.rs::cancel_settlement_in_tx` ;
- `crates/kesh-api/tests/bank_account_closed_inventory.rs:869-875`, `:1708-1714` : `"cancel_settlement_in_tx("` parmi les
  gestes verrouillants (la revérification du compte clos reste la dernière acquisition de verrou, avant l'audit) ;
- `crates/kesh-api/tests/invoice_settlement_response_e2e.rs:647`, `:664` ; `supplier_invoice_response_e2e.rs:369`, `:391` :
  le handler appelle `cancel_settlement`, et la relecture précède le commit ;
- `crates/kesh-api/tests/rejeu_interblocage_e2e.rs:713`, `:2335` : `capture.exiger_un_rejeu("invoices::cancel_settlement")`
  — le libellé du rejeu ne change pas.

**Les tests existants qui touchent l'audit de ces gestes** :
- `crates/kesh-api/tests/reconciliation_e2e.rs:5349-5423`, `an_api_key_cancels_and_is_named_in_the_audit` : dé-rapprochement
  **par clé** d'une écriture seule ; son doc-comment dit (`:5353-5355`) « ⚠️ L'annulation d'un **règlement** de facture par
  un dé-rapprochement reste au nom du créateur de la clé (#431, déploiement) » — **devient faux**, à récrire ;
- `reconciliation_e2e.rs:4465` `cancelling_an_invoice_reconciliation_undoes_everything` (par session, trois lignes d'audit,
  `:4523-4539`) ; `:4727` `cancelling_a_supplier_reconciliation_by_the_route` (par session, `:4802-4810`) ; montages
  `reconciled_invoice` (`:4415`), `fournisseur_rapproche::reglement_rapproche` (`:4732`), `cancel_reco` (`:4313`) ;
- `crates/kesh-api/tests/invoice_settlement_response_e2e.rs:395` `a_cancellation_returns_what_was_written` (route client,
  par session) ; `crates/kesh-api/tests/supplier_settlement_cancel_e2e.rs:230` `pay_then_cancel_the_settlement` (route
  fournisseur, par session) ;
- aides : `crates/kesh-api/tests/common/mod.rs:127` `audit_actor_unique(pool, action, entity_type, entity_id) ->
  (actor_type, actor_api_key_id, user_id)` (échoue s'il n'y a pas exactement une entrée) ; `:204`
  `create_key_via_http(client, base_url, jwt, name, scope)`. `invoice_settlement_response_e2e.rs` n'inclut que
  `common/autour_appel.rs` (`:26`) ; `supplier_settlement_cancel_e2e.rs` a son propre `mod common { … }` (`:24`) ;
  `reconciliation_e2e.rs` a `mod common;` (`:52`).

**Les documents qui décrivent l'attribution** :
- `docs/api-external.md:404` (lettrage) : « la clé n'y figure que pour le **rapprochement** (`POST /reconciliation/accept`) :
  les autres gestes — dé-rapprochement compris — ne portent que l'utilisateur » ;
- `docs/api-external.md:739` (dé-rapprochement) : « l'annulation d'un règlement de facture, client ou fournisseur, reste au
  nom du créateur de la clé » ;
- `docs/api-external.md:565` (annulation d'un règlement client) et `:653` (fournisseur) : n'en disent rien ;
- `docs/manual/fr/admin-manual.tex:3212` (item *Journal d'audit* du § des clés d'API) : « celles que Kesh passe pour […] un
  règlement saisi sur la facture […], le solde de son reste ou leur annulation, une facture fournisseur (validation, paiement
  depuis la fiche […], annulation) […] restent au nom du créateur de la clé~; […] celui d'un règlement de facture (client ou
  fournisseur) l'annule, contre-passation comprise, au nom du créateur de la clé » ;
- `docs/manual/fr/user-manual.tex:4016-4022` (tableau du journal d'audit) : même énoncé ;
- `docs/manual/fr/admin-manual.tex:2549` : paragraphe de mise à jour **vers la 0.17.7**, relevé daté — **non touché**.
- Contrôle des PDF à `9d5a142df` (E6) : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' |
  /usr/bin/grep -c 'contre-passation comprise, au nom du créateur de la clé'` → `1` ; même commande sur `user-manual.pdf`
  → `1`. Manuels de/en/it : aucune occurrence de #431 (`git grep -nlE '431' -- docs/manual/de docs/manual/en
  docs/manual/it` vide). `website/`, `README.md` : rien (`git grep -nE "créateur de la clé|dé-rapprochement" -- website/
  README.md` vide).

## Inventaire des appelants, par le symptôme

**Commande** (un appel coupé sur deux lignes est vu : texte joint, commentaires de ligne retirés), script versionné
`/home/gcorbaz/devel/kesh-gate-logs/e36-4/inventaire-appels.py`, sortie complète
`kesh-gate-logs/e36-4/inventaire-appels-9d5a142df.txt` :

```sh
python3 -I /home/gcorbaz/devel/kesh-gate-logs/e36-4/inventaire-appels.py /home/gcorbaz/devel/kesh-36-4
```

**Recoupé** par `git grep` (compte par ligne, commentaires compris), sortie `kesh-gate-logs/e36-4/git-grep-appels-9d5a142df.txt` :

```sh
git grep -cE '(invoice_settlements_write|supplier_invoices)::cancel_settlement(_in_tx)?\(|reconciliation_cancel::cancel(_in_tx)?\(' -- crates/
```
```
crates/kesh-api/src/routes/invoices.rs:1
crates/kesh-api/src/routes/reconciliation.rs:1
crates/kesh-api/src/routes/supplier_invoices.rs:1
crates/kesh-api/tests/bank_account_closed_inventory.rs:1
crates/kesh-api/tests/bank_accounts_closed_e2e.rs:3
crates/kesh-api/tests/letterings_e2e.rs:1
crates/kesh-db/src/repositories/reconciliation_cancel.rs:2
crates/kesh-db/tests/credit_notes_repository.rs:1
crates/kesh-db/tests/invoice_amount_due_parity.rs:1
crates/kesh-db/tests/invoice_settlement.rs:15
crates/kesh-db/tests/invoice_write_off.rs:5
crates/kesh-db/tests/lettering_documents.rs:14
crates/kesh-db/tests/lettering_documents_backfill.rs:1
crates/kesh-db/tests/letterings.rs:3
crates/kesh-db/tests/open_items.rs:2
crates/kesh-db/tests/payment_batches_repository.rs:3
crates/kesh-db/tests/reconciliation_cancel.rs:6
crates/kesh-db/tests/supplier_invoices_draft.rs:3
crates/kesh-db/tests/supplier_invoices_repository.rs:16
crates/kesh-db/tests/support/lettering_documents.rs:1
crates/kesh-report/tests/aged_receivables.rs:1
crates/kesh-report/tests/vat_report_write_off.rs:1
```

Plus les appels **internes** non qualifiés : `git grep -nE '(^|[^:_a-z])cancel_settlement(_in_tx)?\(|(^|[^:_a-z])cancel(_in_tx)?\('
-- crates/kesh-db/src/repositories/{invoice_settlements_write,supplier_invoices,reconciliation_cancel}.rs` →
`invoice_settlements_write.rs:1152` (l'enveloppe), `supplier_invoices.rs:3011` (l'enveloppe), `reconciliation_cancel.rs:839`
(l'enveloppe de test) ; et les définitions (`:917`, `:1142` ; `:2815`, `:3003` ; `:576`, `:831`). `supplier_invoices.rs:2530`,
`:2703`, `:2711` sont `cancel_in_tx`/`cancel` de la **facture** fournisseur, autre fonction (hors périmètre, lot 36-11c).

**Production** (toutes dans le périmètre) :

| Appelant | Appelé | Ce qu'il passera |
|---|---|---|
| `routes/invoices.rs:1397` (`cancel_invoice_settlement_handler`) | `invoice_settlements_write::cancel_settlement` | `Acteur::from(&current_user)` |
| `routes/supplier_invoices.rs:1023` (`cancel_supplier_invoice_settlement`) | `supplier_invoices::cancel_settlement` | `Acteur::from(&current_user)` |
| `routes/reconciliation.rs:5999` (`cancel_reconciliation_once`) | `reconciliation_cancel::cancel_in_tx` | l'`Acteur` de la requête (D3) |
| `invoice_settlements_write.rs:1152` (enveloppe) | `cancel_settlement_in_tx` | son `acteur` |
| `supplier_invoices.rs:3011` (enveloppe) | `cancel_settlement_in_tx` | son `acteur` |
| `reconciliation_cancel.rs:713` (`cancel_in_tx`, règlement client) | `invoice_settlements_write::cancel_settlement_in_tx` | son `acteur` |
| `reconciliation_cancel.rs:731` (`cancel_in_tx`, règlement fournisseur) | `supplier_invoices::cancel_settlement_in_tx` | son `acteur` |
| `reconciliation_cancel.rs:839` (`cancel`, enveloppe de test) | `cancel_in_tx` | son `acteur` (D4) |

Aucun autre appelant de production : ni `kesh-seed` (`pieges.rs`), ni le serveur MCP, ni un lot de paiement.

**Tests** (conversion mécanique, **77 appels dans 17 fichiers** ; recompte : sortie du script, lignes `APPEL` sous
`/tests/`, moins les quatre chaînes de `bank_account_closed_inventory.rs` — `:872`, `:1648`, `:1651`, `:1711`, des
littéraux d'un détecteur lexical, non des appels) :

| Fonction | Appels | Fichiers |
|---|---|---|
| `invoice_settlements_write::cancel_settlement` | 35 | `kesh-db/tests/` : `invoice_settlement.rs` 13, `lettering_documents.rs` 8, `invoice_write_off.rs` 4, `letterings.rs` 2, `open_items.rs` 2, `credit_notes_repository.rs` 1, `invoice_amount_due_parity.rs` 1, `support/lettering_documents.rs` 1 ; `kesh-api/tests/letterings_e2e.rs` 1 ; `kesh-report/tests/` : `aged_receivables.rs` 1, `vat_report_write_off.rs` 1 |
| `invoice_settlements_write::cancel_settlement_in_tx` | 2 | `kesh-db/tests/invoice_settlement.rs:1085`, `kesh-api/tests/bank_accounts_closed_e2e.rs:1722` |
| `supplier_invoices::cancel_settlement` | 26 | `kesh-db/tests/` : `supplier_invoices_repository.rs` 15, `lettering_documents.rs` 5, `payment_batches_repository.rs` 3, `letterings.rs` 1, `supplier_invoices_draft.rs` 1, `lettering_documents_backfill.rs` 1 |
| `supplier_invoices::cancel_settlement_in_tx` | 2 | `kesh-db/tests/supplier_invoices_repository.rs:1472`, `kesh-api/tests/bank_accounts_closed_e2e.rs:1742` |
| `reconciliation_cancel::cancel` | 11 | `kesh-db/tests/` : `reconciliation_cancel.rs` 6, `supplier_invoices_draft.rs` 2, `invoice_settlement.rs` 1, `invoice_write_off.rs` 1, `lettering_documents.rs` 1 |
| `reconciliation_cancel::cancel_in_tx` | 1 | `kesh-api/tests/bank_accounts_closed_e2e.rs:1704` (`user_id, None, None`) |

**Sonde positive** du détecteur : il rend les deux sites connus de l'étape 0, `reconciliation_cancel.rs:713` et `:731`. Une
première version comptait à tort `supplier_invoices::cancel(` comme un appel nu de `cancel` dans
`kesh-db/tests/reconciliation_cancel.rs` (13 au lieu de 11) : relevée par le recoupement `git grep` (6 lignes contre 8),
corrigée dans le script versionné. **Angles morts du détecteur** : un appel par un alias de module (`use … as m;`) ou par
un `use …::cancel_settlement;` suivi d'un appel nu hors du fichier de définition (aucun aujourd'hui : le recoupement
`git grep` sur le nom nu rend les mêmes fichiers) ; le compilateur fermera la liste au développement (changement de
signature : tout appel oublié ne compile pas).

**Les sites qui présentent le même défaut, hors périmètre** — un geste atteint par une clé dont le dépôt reçoit un
`user_id` nu ; ils restent `AConvertir` dans la garde, au compte exact, et sont portés par les lots de #431 (C-e360-9) :
`invoice_settlements_write::settle_invoice` (`:300-305` de la garde) et `write_off_invoice` (`:306-311`) → **36-11b** ;
`supplier_invoices::cancel_in_tx` (l'annulation de la **facture**, `:350-355`), `pay_in_tx` (`:366`) → **36-11c**. Aucune
issue à ouvrir : #431 les porte.

## Décisions

- **D1 — Les deux `cancel_settlement_in_tx` prennent `acteur: Acteur`** à la place de `user_id: i64` (C-e360-7), à la même
  position. Dans chacune : le délettrage reçoit `acteur` (`dissolve_*_document_group_in_tx(…, acteur)`, l'alias
  `letterings::Actor` étant `Acteur`) ; la contre-passation reçoit `acteur` (`reverse_owned_in_tx(…, acteur, …)`) ; le geste
  s'écrit par `NewAuditLogEntry::par(acteur, …)`. Les commentaires `:1033` et `:2913` (« au nom de l'utilisateur
  (`AConvertir`, Epic 36) ») sont retirés ou récrits. *Écartée* : ajouter un paramètre `actor_api_key_id: Option<i64>` à
  côté de `user_id` — forme que la 35-8a2 a éliminée de `delete_in_tx` et qui rouvre l'angle mort 1 de la garde (un `None`
  positionnel invisible).
- **D2 — Les deux enveloppes `cancel_settlement` prennent `acteur: Acteur`** à la place de `user_id` (même position :
  2ᵉ paramètre côté client, 4ᵉ côté fournisseur) ; les routes passent `Acteur::from(&current_user)`, calculé hors de la
  fermeture de rejeu (patron de la 35-8a2, `routes/journal_entries.rs`). *Écartée* : laisser `user_id` aux enveloppes et
  construire l'acteur dedans — l'enveloppe n'a pas la clé, elle écrirait `Acteur::utilisateur` (un site `AConvertir` de
  plus).
- **D3 — `reconciliation_cancel::cancel_in_tx` prend `acteur: Acteur`** à la place du couple `user_id: i64,
  actor_api_key_id: Option<i64>` (C-36-4-1) ; il le passe aux deux `cancel_settlement_in_tx`, à `reverse_in_tx` (genre
  `Entry`, à la place du littéral `:744-747`) et écrit `reconciliation.cancelled` par `NewAuditLogEntry::par(acteur, …)`
  (à la place de `for_actor(user_id, actor_api_key_id, …)`). La route : `post_cancel_reconciliation` tire
  `Acteur::from(&current_user)` à la place de `user_id` et `actor_api_key_id` (`:5954-5955`), `cancel_reconciliation_once`
  prend `acteur: Acteur` (`:5981-5982`). *Écartée* : garder le couple et ne changer que les deux appels (`:718`, `:735`) en
  `Acteur { user_id, api_key_id: actor_api_key_id }` — plus court, mais laisse au dépôt une signature `Option<i64>` de plus
  dans l'angle mort 1 (`reconciliation_cancel.rs:581`).
- **D4 — L'enveloppe de test `reconciliation_cancel::cancel` prend `acteur: Acteur`** à la place de `user_id` et le passe
  à `cancel_in_tx` (C-e360-7 : « prend un `Acteur` ou est documentée comme telle » ; C-36-4-2). Ses onze appels de tests
  passent `Acteur::utilisateur(u)`. Son doc-comment (`:829-830`) dit qu'elle n'a pas d'appelant de production. Le site de
  l'angle mort 1 disparaît : le doc-comment de la garde (`:68-75`) ne nomme plus que `kesh-seed/src/pieges.rs`, et
  `git grep -nE 'actor_api_key_id:\s*Option<i64>' -- crates/` hors `tests/` rend **29** lignes (une de moins). *Écartée* :
  la garder telle quelle avec un doc-comment « tests seuls » — le `None` positionnel resterait, invisible de la garde, sur
  une fonction publique de `kesh-db` qu'un appelant futur pourrait prendre.
- **D5 — Les tests prouvent l'attribution par exécution, toutes traces comprises** (règle des inventaires,
  `e35-dev-commun.md`, point 1) : chaque test par clé relève l'identifiant maximal d'`audit_log` **avant** le geste, puis
  asserte que **toutes** les lignes écrites par le geste portent `('api_key', Some(clé), créateur)` **et** que leur liste
  d'actions est exactement celle attendue (non vide, nommée). Un site futur du chemin qui perdrait la clé rougit quelle que
  soit sa forme ; une liste vide ne passe pas. *Écartée* : asserter seulement les actions nommées aujourd'hui (une trace
  ajoutée plus tard échapperait).
- **D6 — Pas de spec Playwright neuve.** Le défaut se joue à la frontière HTTP par les tests d'API de `kesh-api` (serveur
  lancé, requêtes `reqwest`, clé créée par `POST /settings/api-keys`) ; l'écran du journal d'audit lit `actor_type` sans
  changement (35-8a1, 35-8a2 : aucune spec non plus), et aucun contrôle visible ne change. L'E2E complet tourne à la clôture
  comme non-régression. *Écartée* : une spec qui crée une clé et lit le journal d'audit à l'écran — elle ne prouverait rien
  de plus que les tests d'API sur ce chemin.
- **D7 — Ce qui ne change pas** : `user_id` d'une entrée par clé reste le créateur de la clé ; vocabulaire des actions,
  `details`, schéma, routes, réponses et refus inchangés ; aucune migration. **Le passé ne se corrige pas** : les entrées
  écrites avant la 0.17.8 par une clé gardent `actor_type = 'user'`. Le nom des fonctions et l'ordre des appels restent ceux
  que lisent les tests lexicaux (*Ce qui se lit au code*).
- **D8 — Manuels et PDF** (C-36-4-3) : la story récrit les `.tex` (admin `:3212`, utilisateur `:4016-4022`), régénère les
  deux PDF français et les contrôle aplatis ; `user-manual.tex` est commun avec 36-3 et 36-5 (tableau du fichier d'epic) :
  à l'intégration, les PDF se régénèrent après l'union des `.tex`. Le paragraphe de mise à jour vers la 0.17.7
  (`admin-manual.tex:2549`) n'est pas touché ; un paragraphe de mise à jour vers la 0.17.8, s'il en faut un, est l'affaire
  de l'intégration (comme le CHANGELOG, C-e360-11).

## Critères d'acceptation

1. **AC1 — Annulation d'un règlement client par sa route, par clé.** Une facture client validée, réglée **entièrement**
   (le lettrage de la pièce existe : sa dissolution est exercée), une clé `read-write` créée par HTTP ;
   `POST /api/v1/invoices/{id}/settlements/{settlementId}/cancel` avec la clé → `200`, et **toutes** les entrées d'audit
   écrites par le geste portent `actor_type = 'api_key'`, `actor_api_key_id` = la clé, `user_id` = son créateur ; leurs
   actions sont exactement `invoice.settlement_cancelled`, `journal_entry.reversed`, `journal_entry.created` (l'inverse),
   `lettering.removed` (la pièce) et `lettering.created` (la paire de contre-passation) — liste relevée au développement sur
   le montage réel, écrite au Change Log, chaque action présente au moins une fois. `invoice.settlement_cancelled` est
   **unique** (`common::audit_actor_unique` ou forme en ligne équivalente).
2. **AC2 — Annulation d'un règlement fournisseur par sa route, par clé.** Une facture fournisseur payée (lettrage de la
   pièce posé), une clé `read-write` ; `POST /api/v1/supplier-invoices/{id}/settlement/cancel` avec la clé → `200` ; même
   assertion qu'AC1 (`supplier_invoice.settlement_cancelled` unique, la contre-passation, le délettrage, la paire).
3. **AC3 — Dé-rapprochement d'un règlement, par clé, client et fournisseur.** Par clé :
   `POST /api/v1/reconciliation/transactions/{id}/cancel` (a) sur une transaction rapprochée d'une **facture client**
   (montage `reconciled_invoice`, `reconciliation_e2e.rs:4415`), (b) sur une transaction rapprochée d'un **paiement
   fournisseur** (montage `fournisseur_rapproche::reglement_rapproche`, `:4732`). Dans les deux cas, toutes les entrées
   écrites par le geste portent la clé, actions relevées comme en AC1, **dont** `reconciliation.cancelled` et
   `invoice.settlement_cancelled` (a) ou `supplier_invoice.settlement_cancelled` (b). Le cas de l'écriture seule
   (`an_api_key_cancels_and_is_named_in_the_audit`, `:5357`) reste vert, et son doc-comment (`:5349-5355`) est récrit.
4. **AC4 — Par session, rien ne change.** Les mêmes trois gestes par session (JWT) écrivent leurs entrées à
   `actor_type = 'user'`, `actor_api_key_id` nul : au moins une assertion par chemin, ajoutée aux tests existants
   (`a_cancellation_returns_what_was_written`, `pay_then_cancel_the_settlement`,
   `cancelling_an_invoice_reconciliation_undoes_everything` ou `cancelling_a_supplier_reconciliation_by_the_route`). Les tests
   de dépôt existants restent verts, `Acteur::utilisateur(u)` à la place de `u` : ils écrivent exactement les entrées
   d'avant.
5. **AC5 — La garde.** Les six entrées du tableau (*Ce qui se lit au code*) sont retirées ; aucune n'est ajoutée ; la table
   est recomptée depuis la source à l'étape finale (attendu : **87 entrées, 92 sites** ; `AConvertir` 47/48 ; `User` 69/74,
   `ApiKeyNone` 8/8, `Utilisateur` 10/10), ventilation et total cohérents écrits au Change Log ; aucune entrée pour
   `routes/invoices.rs`, `routes/supplier_invoices.rs` (fonctions touchées), `routes/reconciliation.rs`,
   `repositories/reconciliation_cancel.rs`. Le doc-comment de la table (`:175-185`) et l'angle mort 1 (`:68-75`) sont mis à
   jour (D4 : `pieges.rs` seul, 29 lignes). La garde est verte.
6. **AC6 — Documents.** `docs/api-external.md` : `:404` dit que la clé figure au lettrage et au délettrage d'une facture
   client pour le rapprochement, l'**annulation d'un règlement ou d'un solde** (`POST …/settlements/{settlementId}/cancel`)
   et le **dé-rapprochement**, *depuis la v0.17.8* — non pour un règlement saisi, un solde ou un avoir (#431) ; et de même
   pour une facture fournisseur (`…/settlement/cancel` et dé-rapprochement) ; `:739` : la parenthèse dit que l'annulation
   d'un règlement par le dé-rapprochement porte la clé *depuis la v0.17.8* ; `:565`, `:653` : une phrase dit que les traces
   du geste portent la clé *depuis la v0.17.8*. Manuels : `admin-manual.tex:3212` et `user-manual.tex:4016-4022` disent que
   l'annulation d'un règlement ou d'un solde (route ou dé-rapprochement, client ou fournisseur) est attribuée à la clé depuis
   la 0.17.8, contre-passation et délettrage compris, et que les entrées antérieures gardent la marque « utilisateur » ;
   PDF régénérés, contrôlés aplatis (E6) : la phrase « contre-passation comprise, au nom du créateur de la clé » rend `0`, la
   phrase neuve rend `1`, dans chacun des deux PDF. Symptôme greppé sur le dépôt (`créateur de la clé`, `dé-rapprochement
   compris`, `reste au nom du créateur`) : chaque occurrence restante justifiée au Change Log.

## Tests à écrire, et la preuve du rouge

Écrits **d'abord**, sur la base de la branche, commités avant le correctif ; rouges par `actor_type = 'user'` (journal
`/home/gcorbaz/devel/kesh-gate-logs/e36-4/rouges.log`, avec le commit). AC4 vert au départ (témoin).

| Test (nom indicatif) | Fichier hôte | AC | Rouge attendu sur la base |
|---|---|---|---|
| `an_api_key_cancels_an_invoice_settlement_and_is_named_everywhere` | `crates/kesh-api/tests/invoice_settlement_response_e2e.rs` (montage `setup`, `settle`) | AC1 | `invoice.settlement_cancelled` à `("user", None, créateur)` |
| `an_api_key_cancels_a_supplier_settlement_and_is_named_everywhere` | `crates/kesh-api/tests/supplier_settlement_cancel_e2e.rs` (montage de `pay_then_cancel_the_settlement`) | AC2 | idem, `supplier_invoice.settlement_cancelled` |
| `an_api_key_unreconciles_an_invoice_settlement_and_is_named_everywhere` | `crates/kesh-api/tests/reconciliation_e2e.rs` | AC3 (a) | `invoice.settlement_cancelled` à `'user'` (`reconciliation.cancelled` déjà à la clé) |
| `an_api_key_unreconciles_a_supplier_settlement_and_is_named_everywhere` | `crates/kesh-api/tests/reconciliation_e2e.rs` | AC3 (b) | `supplier_invoice.settlement_cancelled` à `'user'` |
| assertions de session ajoutées | les quatre tests existants nommés en AC4 | AC4 | vertes |

Le développeur choisit les hôtes au mieux (un test par clé hors de ces fichiers est permis s'il y trouve un montage plus
court) ; la clé se crée par HTTP (`common::create_key_via_http` ou la requête en ligne de
`reconciliation_e2e.rs:5372-5385`). Chaque test relève d'abord que le montage a bien posé ce que le geste défait (lettrage
de la pièce existant, sans quoi le délettrage ne serait pas exercé — patron d'AC2 de la 35-8a2).

**Mutations** (sur l'arbre commité ; script versionné sous `/home/gcorbaz/devel/kesh-gate-logs/e36-4/mut/`, restauration
par `git restore --worktree -- <fichier>` puis `touch <fichier>`, E5 ; journaux `mut/*.log`) :

| Mutation | Doit être tuée par |
|---|---|
| M1 — client `cancel_settlement_in_tx` : le geste par `NewAuditLogEntry::user(acteur.user_id, …)` | AC1, AC3 (a), la garde (site `User` neuf) |
| M2 — client : le délettrage reçoit `Actor { user_id: acteur.user_id, api_key_id: None }` | AC1, la garde (`ApiKeyNone`) |
| M3 — fournisseur : la contre-passation reçoit `Acteur::utilisateur(acteur.user_id)` | AC2, AC3 (b), la garde (`Utilisateur`) |
| M4 — route client : `Acteur { api_key_id: current_user.api_key_id.filter(\|_\| false), ..Acteur::from(&current_user) }` (forme que la garde ne voit pas) | AC1 seul |
| M5 — route fournisseur : même forme | AC2 seul |
| M6 — `reconciliation_cancel.rs`, appel client : l'acteur passé avec la clé jetée par `.filter(\|_\| false)` | AC3 (a) seul |
| M7 — `reconciliation_cancel.rs`, appel fournisseur : même forme | AC3 (b) seul |
| M8 — route du dé-rapprochement : l'acteur tiré avec la clé jetée | AC3 (a) et (b), et `an_api_key_cancels_and_is_named_in_the_audit` |

Toute mutation survivante s'écrit au Change Log avec sa raison. Des mutations jouées ensemble se disent, chaque test tué
nommé.

## Tâches

- [ ] **Étape 0** : relire les lignes citées à la base réelle (la branche part de `9d5a142df`, ou de `main` après la fusion
      d'une autre story) ; relancer `inventaire-appels.py` et la commande `git grep`, sorties au Change Log ; recompter la
      table de la garde ; relire les LOW du dernier rapport de validation.
- [ ] Rouges : les quatre tests par clé, commités ; `rouges.log`.
- [ ] `invoice_settlements_write.rs` : `cancel_settlement_in_tx` et `cancel_settlement` à l'`Acteur` (D1, D2).
- [ ] `supplier_invoices.rs` : idem.
- [ ] `reconciliation_cancel.rs` : `cancel_in_tx` et `cancel` à l'`Acteur` (D3, D4) ; doc-comments.
- [ ] Routes : `routes/invoices.rs`, `routes/supplier_invoices.rs`, `routes/reconciliation.rs` (D2, D3).
- [ ] Tests existants : 77 appels dans 17 fichiers, mécaniques (`Acteur::utilisateur(u)`), fichier par fichier ; le
      compilateur ferme la liste ; assertions de session d'AC4 ; doc-comment de `reconciliation_e2e.rs:5349-5355`.
- [ ] Garde : six entrées retirées, table recomptée, doc-comments (AC5).
- [ ] Documents : `docs/api-external.md` (`:404`, `:565`, `:653`, `:739`), `admin-manual.tex:3212`,
      `user-manual.tex:4016-4022`, PDF régénérés (`make fr` dans `docs/manual/`, sous `.gate-front.lock` ou la voie de la
      consigne en vigueur), contrôlés aplatis (AC6) ; symptôme greppé.
- [ ] Mutations M1 à M8.
- [ ] **Gates** (voie 1 ; `kesh-db` touché) : en cours de boucle, `scripts/test-fast.sh --ci` (complet, non ciblé) sur
      `kesh_e364` remise à neuf ; au **passage en `review`**, `test-fast --ci` puis `scripts/gate-ci.sh kesh_ci_e364`, une
      commande à la fois sous `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock
      scripts/mem-guard.sh …` ; frontend non touché (le dire) ; **pas d'E2E** ; aux remédiations, `test-fast --ci` seul ; à la
      **clôture**, `gate-ci` **et** E2E complet sur `kesh_e2e_e364` recréée, au dernier commit de code, échecs nommés contre
      `docs/testing.md` (sept de la KF-029). Chaque chiffre cite son journal sous `kesh-gate-logs/e36-4/` et son commit.
- [ ] Prompt de la revue P1 à côté de la fiche (`36-4-acteur-annulation-reglement-review-prompt-p1.md`) : trois lentilles,
      manuels et PDF nommés, garde et tests lexicaux nommés, axes exercés et non exercés exigés.

## Note de version

*Pour l'orchestrateur, qui l'assemble dans `## [0.17.8] — Non publié` (C-e360-11) ; cette branche n'écrit pas
`CHANGELOG.md`.*

**Corrigé**

- Le journal d'audit attribue à la **clé d'API** l'annulation d'un règlement qu'elle fait — un règlement ou un solde de
  facture client, le paiement d'une facture fournisseur —, que ce soit par l'annulation du règlement ou par l'annulation du
  rapprochement bancaire qui l'avait passé : le geste, la contre-passation et le délettrage portent désormais la marque
  « clé API » et le numéro de la clé, au lieu du seul nom de son créateur. Les entrées écrites avant la 0.17.8 gardent la
  marque « utilisateur ». Aucun changement des routes ni des réponses de l'API. (refs #1131, refs #431)

## Dev Notes

### Taille et découpage

- **Crates** : trois — `kesh-db` (trois dépôts : `invoice_settlements_write`, `supplier_invoices`, `reconciliation_cancel`),
  `kesh-api` (trois routes, la garde, les tests d'API), `kesh-report` (deux fichiers de tests, un appel chacun). **Modules
  métier** : règlement client, règlement fournisseur, réconciliation ; plus la garde et les documents. **Cinq au plus** au
  sens du `CLAUDE.md` (§ *Règle de splitting préventif*) : pas de découpage. Le gros du volume (77 appels de tests) est
  mécanique et fermé par le compilateur ; il se revoit fichier par fichier. Les signatures, les trois routes et les quatre
  tests neufs restent à la boucle complète.
- **Migration** : aucune (C-e360-3). Si le développement en trouvait une nécessaire, il s'arrête et le dit.

### Fichiers communs avec les autres stories de la phase 1 (tableau du fichier d'epic)

- `docs/manual/fr/user-manual.tex` (36-3, 36-5 possible) : paragraphe du journal d'audit seulement ; PDF régénéré après
  l'union. `docs/manual/fr/admin-manual.tex` : seule 36-4 dans la phase 1.
- `docs/api-external.md` : 36-4 seule.
- `crates/kesh-api/tests/audit_acteur_garde.rs` : 36-4, puis les lots 36-11 (phase 2), qui partent de 36-4 fusionnée.
- `CHANGELOG.md` : non touché (note de version ci-dessus).
- Aucun fichier de code commun avec 36-1, 36-2, 36-3 (frontend) ni 36-5 (frontend).

### Ce qui doit rester vrai

- Aucune entrée d'audit perdue ni doublée : le nombre et les actions des entrées de chaque geste sont ceux d'avant ; seuls
  `actor_type` et `actor_api_key_id` changent quand une clé agit.
- L'ordre des étapes de chaque `cancel_settlement_in_tx` (verrous, refus, délettrage avant contre-passation, revérification
  du compte clos en dernière acquisition de verrou, audit) ne bouge pas : les tests lexicaux (`letterings_lexical.rs`,
  `bank_account_closed_inventory.rs`, `invoice_settlement_response_e2e.rs`, `supplier_invoice_response_e2e.rs`) le lisent.
- Le libellé des rejeux (`"invoices::cancel_settlement"`, `"supplier_invoices::cancel_settlement"`,
  `"reconciliation::cancel"`) ne change pas (`rejeu_interblocage_e2e.rs`).
- Fixtures synthétiques seulement ; rien sous `~/kesh/`.

### Hors périmètre

- `settle_invoice`, `write_off_invoice` (36-11b), `supplier_invoices::cancel_in_tx`, `pay_in_tx` (36-11c) : restent
  `AConvertir`, au compte exact.
- Le serveur MCP (#1057) ; la restauration ; l'angle mort 1 pour `pieges.rs` (36-11d).
- Le paragraphe de mise à jour vers la 0.17.8 au manuel d'administration et le CHANGELOG : intégration.

### Références

- Fiches `35-8a1-audit-acteur-mecanisme.md` (type `Acteur`, garde), `35-8a2-audit-acteur-ecritures.md` (écritures,
  dé-rapprochement d'une écriture seule, tests par clé du rapprochement) ; `35-8b-audit-acteur-deploiement.md`
  (`superseded`) et `kesh-gate-logs/e35-8b-validation-p1.md` (V1-3 : `api-external.md:404`).
- Fichier d'epic `epic-36-premiere-facture-et-suite.md` (ordre 4, fichiers communs) ; registre `e36-choix-autonomes-e360.md`
  (étape 0 de 36-4, C-e360-7).
- Inventaire : `kesh-gate-logs/e36-4/inventaire-appels.py`, `inventaire-appels-9d5a142df.txt`,
  `inventaire-appels-tests-par-fichier-9d5a142df.txt`, `git-grep-appels-9d5a142df.txt`, `actor-api-key-id-option.txt`.
- Issues #1131 (P2), #431.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-14 — **Fiche rédigée** (Opus 5.5, agent de rédaction des fiches de l'Epic 36, worktree `kesh-36-4`), code lu à
  `9d5a142df`. Étape 0 et C-e360-7 repris tels quels ; prémisse de #1131 relue au code (la route du dé-rapprochement porte
  la clé, `routes/reconciliation.rs:5955` ; la perte est dans les deux `cancel_settlement_in_tx`,
  `invoice_settlements_write.rs:917`, `supplier_invoices.rs:2815`). Inventaire par le symptôme : 8 appels de production,
  77 appels de tests dans 17 fichiers ; garde recomptée (93/98 → 87/92 attendus). Choix C-36-4-1 à -4. Aucune migration.
  Statut : `ready-for-dev`. Validation P1 à lancer.
