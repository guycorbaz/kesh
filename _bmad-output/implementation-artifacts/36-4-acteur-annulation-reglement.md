# Story 36-4 : l'annulation d'un règlement par une clé d'API est attribuée à la clé — par sa route ou par le dé-rapprochement (#1131)

Status: done

*Fiche rédigée le 2026-11-14 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-4`, branche
`story/36-4-acteur-annulation-reglement`, par le workflow `bmad-create-story`. Phase 1, release **0.17.8**. **Aucune
migration** (colonnes `actor_type`, `actor_api_key_id` de la migration `20260605000002`, Story 17-2a). **`kesh-db` touché :
gate complet même en cours de boucle.** Voie **1** (lignée B, C-e360-4). Code lu à la tête du worktree, **`9d5a142df`**
(kickoff de l'Epic 36, sur `main` `42d5f368`). Registres : `e36-choix-autonomes-e360.md` (étape 0 de 36-4, C-e360-3, -7,
-9, -11) ; `e36-choix-autonomes-36-4.md` (C-36-4-1 à -4). Validation terminée (P1 Sonnet, P2 Opus ; Change Log). Patron : stories 35-8a1 et 35-8a2
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
`reconciliation_cancel.rs:581` (`cancel_in_tx`) **et** `routes/reconciliation.rs:5982` (`cancel_reconciliation_once`)
(`kesh-gate-logs/e36-4/actor-api-key-id-option.txt`) ; le doc-comment de la garde dit aussi « 26 fonctions de `src` hors
`delete_in_tx` à la fiche, P3 » et « 23 lignes de paramètre » (`:69-70`) — 23 recompté à `9d5a142df` : les 30 lignes moins
6 champs `pub actor_api_key_id` et 1 doc-comment. *(Ajouté à la remédiation de la validation P1, V1-2.)*

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
- `docs/manual/fr/user-manual.tex:4016-4023` (tableau du journal d'audit) : même énoncé ; `:4017` « le solde de son reste
  ou leur annulation, une facture fournisseur » dans la liste de ce qui « restent au nom du créateur de la clé » ;
  `:4022-4023` « une ligne écrite avant la 0.17.7 ne porte pas la marque » (V1-1) ;
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
  `git grep -nE 'actor_api_key_id:\s*Option<i64>' -- crates/` hors `tests/` rend **28** lignes (deux de moins avec D3 :
  `reconciliation_cancel.rs:581` et `routes/reconciliation.rs:5982`), soit **21** lignes de paramètre (28 moins 6 champs et
  1 doc-comment) ; ces deux nombres du doc-comment de la garde (`:69-72`) se recomptent depuis la source au développement,
  commande et sortie au Change Log, de même que « 26 fonctions », qui date de la P3 de la 35-8a1 et se récrit ou se date.
  *Écartée* :
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
- **D8 — Manuels et PDF** (C-36-4-3) : la story récrit les `.tex` (admin `:3212`, utilisateur `:4016-4023`), régénère les
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
   **Cas du solde (V1-3)** : la même route, par clé, sur un **solde** (`POST /api/v1/invoices/{id}/write-off`, montage de
   `a_write_off_returns_what_was_written`, `invoice_settlement_response_e2e.rs:357`, puis annulation de la ligne
   `settlementType = write_off`) → `200`, mêmes assertions « toutes traces » (actions relevées sur le montage réel, écrites
   au Change Log). Le chemin des **traces** de `cancel_settlement_in_tx` ne branche pas sur le type
   (`invoice_settlements_write.rs:917-1121` ne lit pas `settlement_type`) ; seul son bloqueur le lit
   (`settlement_cancel_blocker`, `:880` : `if write_off_exists && settlement_type != "write_off"`), et un solde existant
   n'admet que sa propre annulation *(V2-5, étape 0)* : ce cas prouve la promesse de la note de version, non un autre
   chemin de traces.
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
   jour (D4 : `pieges.rs` seul ; 28 lignes au `git grep`, 21 lignes de paramètre, recomptées depuis la source). La garde
   est verte.
6. **AC6 — Documents.** `docs/api-external.md` : `:404` dit que la clé figure au lettrage et au délettrage d'une facture
   client pour le rapprochement, l'**annulation d'un règlement ou d'un solde** (`POST …/settlements/{settlementId}/cancel`)
   et le **dé-rapprochement**, *depuis la v0.17.8* — non pour un règlement saisi, un solde ou un avoir (#431) ; et de même
   pour une facture fournisseur (`…/settlement/cancel` et dé-rapprochement) ; `:739` : la parenthèse dit que l'annulation
   d'un règlement par le dé-rapprochement porte la clé *depuis la v0.17.8* ; `:565`, `:653` : une phrase dit que les traces
   du geste portent la clé *depuis la v0.17.8*. Manuels : `admin-manual.tex:3212` et `user-manual.tex:4016-4023` disent que
   l'annulation d'un règlement ou d'un solde (route ou dé-rapprochement, client ou fournisseur) est attribuée à la clé depuis
   la 0.17.8, contre-passation et délettrage compris, et que les entrées antérieures gardent la marque « utilisateur ».
   Dans la **même phrase** des deux `.tex` (V1-1) :
   - « ou leur annulation » **sort** de la liste de ce qui « reste(nt) au nom du créateur de la clé » (`admin-manual.tex:3212`
     « le solde de son reste ou leur annulation, une facture fournisseur … restent au nom du créateur de la clé » ;
     `user-manual.tex:4017`, même énoncé) : le règlement saisi et le solde y restent, leur **annulation** n'y est plus ;
   - « une facture fournisseur (validation, paiement …, annulation) » devient « (validation, paiement …, annulation **de la
     facture**) », pour que l'annulation de la facture fournisseur (qui reste au créateur, lot 36-11c) ne se confonde pas avec
     celle de son paiement ; de même dans la **première** liste du même `\item` d'`admin-manual.tex:3212` (« les autres
     gestes des factures fournisseur (création, modification, validation, paiement depuis la fiche ou par un lot de paiement,
     annulation) … inscrivent encore `actor_type = 'user'` ») *(V2-2, étape 0)* ;
   - « une ligne écrite avant la 0.17.7 ne porte pas la marque » (`user-manual.tex:4022-4023`) et « avant la v0.17.7 »
     (`admin-manual.tex:3212`) sont complétés : avant la 0.17.8 pour l'annulation d'un règlement ou d'un solde.

   PDF régénérés, contrôlés aplatis (E6), dans chacun des deux PDF : **absence** des anciennes formulations —
   « contre-passation comprise, au nom du créateur de la clé » → `0`, « ou leur annulation, une facture fournisseur » → `0`
   (`1` dans chacun à `9d5a142df`) — **et** présence de la phrase neuve → `1`. Symptôme greppé sur le dépôt
   (`créateur de la clé`, `restent au nom du créateur`, `reste au nom du créateur`, `leur annulation`, `dé-rapprochement
   compris`, `ne portent que l'utilisateur`, `avant la 0.17.7`, `avant la v0.17.7`) : chaque occurrence restante justifiée au
   Change Log (`admin-manual.tex:2549`, relevé daté de la 0.17.7, et `README.md:243`, historique, restent tels quels).

## Tests à écrire, et la preuve du rouge

Écrits **d'abord**, sur la base de la branche, commités avant le correctif ; rouges par `actor_type = 'user'` (journal
`/home/gcorbaz/devel/kesh-gate-logs/e36-4/rouges.log`, avec le commit). AC4 vert au départ (témoin).

| Test (nom indicatif) | Fichier hôte | AC | Rouge attendu sur la base |
|---|---|---|---|
| `an_api_key_cancels_an_invoice_settlement_and_is_named_everywhere` | `crates/kesh-api/tests/invoice_settlement_response_e2e.rs` (montage `setup`, `settle`) | AC1 | les traces du geste à `("user", None, créateur)` — l'aide les liste **toutes**, la première par `id` étant `lettering.removed` *(V2-4, étape 0)* ; dont `invoice.settlement_cancelled` |
| `an_api_key_cancels_a_write_off_and_is_named_everywhere` | `crates/kesh-api/tests/invoice_settlement_response_e2e.rs` (montage de `a_write_off_returns_what_was_written`) | AC1, cas du solde | idem |
| `an_api_key_cancels_a_supplier_settlement_and_is_named_everywhere` | `crates/kesh-api/tests/supplier_settlement_cancel_e2e.rs` (montage de `pay_then_cancel_the_settlement`) | AC2 | idem, `supplier_invoice.settlement_cancelled` |
| `an_api_key_unreconciles_an_invoice_settlement_and_is_named_everywhere` | `crates/kesh-api/tests/reconciliation_e2e.rs` | AC3 (a) | `invoice.settlement_cancelled` à `'user'` (`reconciliation.cancelled` déjà à la clé) |
| `an_api_key_unreconciles_a_supplier_settlement_and_is_named_everywhere` | `crates/kesh-api/tests/reconciliation_e2e.rs` | AC3 (b) | `supplier_invoice.settlement_cancelled` à `'user'` |
| assertions de session ajoutées | les quatre tests existants nommés en AC4 (le quatrième, `cancelling_a_supplier_reconciliation_by_the_route`, à la remédiation de la revue P1, E3 = A1) | AC4 | vertes |
| `an_api_key_unlinks_one_side_and_is_named` | `crates/kesh-api/tests/reconciliation_link_entry_e2e.rs` (montage de `s34_11b_ac4_annuler_un_cote_puis_l_autre`) — remédiation de la revue P1, B2 = E1 | AC3, simple détachement | ajouté après le correctif : non rouge |
| `an_api_key_cancels_a_partial_settlement_and_is_named_everywhere` | `invoice_settlement_response_e2e.rs` — remédiation de la revue P1, E2 | AC1, règlement partiel | ajouté après le correctif : non rouge |

Le développeur choisit les hôtes au mieux (un test par clé hors de ces fichiers est permis s'il y trouve un montage plus
court) ; la clé se crée par HTTP (`common::create_key_via_http` ou la requête en ligne de
`reconciliation_e2e.rs:5372-5385`).

**Une aide commune du relevé « toutes traces »** (V1-5, DRY) : un fichier neuf `crates/kesh-api/tests/common/audit_traces.rs`
(`#![allow(dead_code)]`, comme `autour_appel.rs`), avec deux fonctions — `max_audit_id(pool) -> i64` (0 sur une table
vide) et `audit_depuis(pool, id_min) -> Vec<(String /* action */, String /* actor_type */, Option<i64>, i64 /* user_id */)>`,
ordonné par `id` — et une assertion `assert_toutes_a_la_cle(traces, cle, createur, actions_attendues)` qui échoue sur une
liste vide, une action manquante ou en trop, ou une trace sans la clé, en nommant **toutes** les traces fautives (V2-4).
`actions_attendues` se compare **avec sa multiplicité** (liste triée contre liste triée) : un doublon d'une même action,
dont `*.settlement_cancelled`, ne passe pas (V2-3). `create_key_via_http` est **déplacée** de `common/mod.rs` dans ce
fichier et réexportée par `common/mod.rs` (`pub use audit_traces::create_key_via_http;`), si bien que les hôtes au module
`common` en ligne l'ont aussi sans la recopier (V2-3, C-36-4-5). Il est déclaré
`pub mod audit_traces;` dans `common/mod.rs` (vu par `reconciliation_e2e.rs`, `mod common;` `:52`), et ajouté au module
`common` en ligne de `supplier_settlement_cancel_e2e.rs` (`mod common { pub mod jour; }`, `:24-26`) et, sous la même forme,
de `invoice_settlement_response_e2e.rs` (qui n'inclut aujourd'hui que `#[path = "common/autour_appel.rs"]`, `:26-27`). Les
cinq tests par clé l'emploient ; aucun ne recopie la requête. Chaque test relève d'abord que le montage a bien posé ce que le geste défait (lettrage
de la pièce existant, sans quoi le délettrage ne serait pas exercé — patron d'AC2 de la 35-8a2).

**Mutations** (sur l'arbre commité ; script versionné sous `/home/gcorbaz/devel/kesh-gate-logs/e36-4/mut/`, restauration
par `git restore --worktree -- <fichier>` puis `touch <fichier>`, E5 ; journaux `mut/*.log`) :

| Mutation | Doit être tuée par |
|---|---|
| M1 — client `cancel_settlement_in_tx` : le geste par `NewAuditLogEntry::user(acteur.user_id, …)` | AC1, AC3 (a), la garde (site `User` neuf) |
| M1' — fournisseur : le geste par `NewAuditLogEntry::user(acteur.user_id, …)` | AC2, AC3 (b), la garde (`User`) |
| M2 — client : le délettrage reçoit `Actor { user_id: acteur.user_id, api_key_id: None }` | AC1, AC3 (a), la garde (`ApiKeyNone`) |
| M2' — fournisseur : le délettrage reçoit `letterings::Actor { user_id: acteur.user_id, api_key_id: None }` | AC2, AC3 (b), la garde (`ApiKeyNone`) |
| M3 — fournisseur : la contre-passation reçoit `Acteur::utilisateur(acteur.user_id)` | AC2, AC3 (b), la garde (`Utilisateur`) |
| M3' — client : la contre-passation reçoit `Acteur::utilisateur(acteur.user_id)` | AC1 (règlement et solde), AC3 (a), la garde (`Utilisateur`) |
| M4 — route client : `Acteur { api_key_id: current_user.api_key_id.filter(\|_\| false), ..Acteur::from(&current_user) }` (forme que la garde ne voit pas) | AC1 seul |
| M5 — route fournisseur : même forme | AC2 seul |
| M6 — `reconciliation_cancel.rs`, appel client : l'acteur passé avec la clé jetée par `.filter(\|_\| false)` | AC3 (a) seul |
| M7 — `reconciliation_cancel.rs`, appel fournisseur : même forme | AC3 (b) seul |
| M8 — route du dé-rapprochement : l'acteur tiré avec la clé jetée | AC3 (a) et (b), et `an_api_key_cancels_and_is_named_in_the_audit` |

Toute mutation survivante s'écrit au Change Log avec sa raison. Des mutations jouées ensemble se disent, chaque test tué
nommé.

## Tâches

- [x] **Étape 0** : relire les lignes citées à la base réelle (la branche part de `9d5a142df`, ou de `main` après la fusion
      d'une autre story) ; relancer `inventaire-appels.py` et la commande `git grep`, sorties au Change Log ; recompter la
      table de la garde ; relire les LOW du dernier rapport de validation.
- [x] Aide commune `common/audit_traces.rs` (V1-5) ; rouges : les cinq tests par clé (dont le solde), commités ;
      `rouges.log`.
- [x] `invoice_settlements_write.rs` : `cancel_settlement_in_tx` et `cancel_settlement` à l'`Acteur` (D1, D2).
- [x] `supplier_invoices.rs` : idem.
- [x] `reconciliation_cancel.rs` : `cancel_in_tx` et `cancel` à l'`Acteur` (D3, D4) ; doc-comments.
- [x] Routes : `routes/invoices.rs`, `routes/supplier_invoices.rs`, `routes/reconciliation.rs` (D2, D3).
- [x] Tests existants : 77 appels dans 17 fichiers, mécaniques (`Acteur::utilisateur(u)`), fichier par fichier ; le
      compilateur ferme la liste ; assertions de session d'AC4 ; doc-comment de `reconciliation_e2e.rs:5349-5355`.
- [x] Garde : six entrées retirées, table recomptée, doc-comments (AC5).
- [x] Documents : `docs/api-external.md` (`:404`, `:565`, `:653`, `:739`), `admin-manual.tex:3212`,
      `user-manual.tex:4016-4023` (dont « ou leur annulation », « annulation de la facture », « avant la 0.17.7 », V1-1),
      PDF régénérés (`make fr` dans `docs/manual/`, sous `.gate-front.lock` ou la voie de la
      consigne en vigueur), contrôlés aplatis (AC6) ; symptôme greppé.
- [x] Mutations M1 à M8, et M1', M2', M3' (V1-4).
- [x] **Gates** du passage en `review` (`test-fast --ci`, `gate-ci` ; E2E à la clôture) (voie 1 ; `kesh-db` touché) : en cours de boucle, `scripts/test-fast.sh --ci` (complet, non ciblé) sur
      `kesh_e364` remise à neuf ; au **passage en `review`**, `test-fast --ci` puis `scripts/gate-ci.sh kesh_ci_e364`, une
      commande à la fois sous `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock
      scripts/mem-guard.sh …` ; frontend non touché (le dire) ; **pas d'E2E** ; aux remédiations, `test-fast --ci` seul ; à la
      **clôture**, `gate-ci` **et** E2E complet sur `kesh_e2e_e364` recréée, au dernier commit de code, échecs nommés contre
      `docs/testing.md` (sept de la KF-029). Chaque chiffre cite son journal sous `kesh-gate-logs/e36-4/` et son commit.
- [x] Prompt de la revue P1 à côté de la fiche (`36-4-acteur-annulation-reglement-review-prompt-p1.md`) : trois lentilles,
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
  mécanique et fermé par le compilateur ; il se revoit fichier par fichier. Les signatures, les trois routes et les cinq
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

Opus 5.5 (agent de développement de l'Epic 36), 2026-11-14, worktree `/home/gcorbaz/devel/kesh-36-4`, voie 1.

### Debug Log References

Sous `/home/gcorbaz/devel/kesh-gate-logs/e36-4/` : `rouges.log`, `rouges-run1.log`, `rouges-run2.log` ;
`inventaire-appels-846d64e98.txt`, `git-grep-appels-846d64e98.txt` ; `recompte-garde.py`, `recompte-garde-apres.txt` ;
`actor-api-key-id-option-apres.txt` ; `clippy-1.log` ; `test-fast-ci-c5d67ce42.log` ; `mut/*.log`, `mut/muter.py`,
`mut/jouer.sh` ; `manuels-make-c5d67ce42.log` ; `gate-ci-e958fb169.log` (+ `.console.log`) ; `bases-a-neuf.sh`.

### Completion Notes List

- **Commits** : `80bc35072` (rouges, aide `audit_traces.rs`, étape 0 de la fiche), `c5d67ce42` (correctif, garde, conversion
  des tests — **dernier commit de code**), `e958fb169` (documents et PDF), puis la fiche, le registre, `sprint-status.yaml` et
  le prompt de revue.
- **Gates** :
  - `cargo clippy --workspace --all-targets -- -D warnings` vert (`clippy-1.log`) et `cargo fmt --check` vert (`▶ cargo fmt
    --check`, `test-fast-ci-c5d67ce42.log:4` — *corrigé à la remédiation de la revue P1, A2 : ce point citait `clippy-1.log`,
    qui ne porte aucune trace de `fmt`*), arbre de `c5d67ce42` avant son
    commit (`clippy-1.log`).
  - `scripts/test-fast.sh --ci` sur `kesh_e364` remise à neuf (`bases-a-neuf.sh`), au commit `c5d67ce42` :
    **5450 exécutés, 5450 passés, 13 ignorés** (`test-fast-ci-c5d67ce42.log`, `Summary [ 338.596s]`). ⚠️ Il précède le commit
    des documents `e958fb169` ; les tests qui lisent les manuels ou `api-external.md` y sont couverts par `gate-ci`.
  - `scripts/gate-ci.sh kesh_ci_e364` au commit `e958fb169` : **premier essai rouge** — 2120 passés, 1 échec, 7 ignorés,
    arrêté au premier binaire en échec (`gate-ci-e958fb169.log`) : `mcp_transport_e2e::abandon_au_delai_le_permis_reste_pris`,
    au témoin (`mcp_transport_e2e.rs:1876`, « témoin : sonde servie »), avant tout geste — **KF-061, #1126** (ouverte), test
    de délais du serveur MCP, sans rapport avec la story (aucun fichier MCP touché ; le même test est passé dans
    `test-fast-ci-c5d67ce42.log`). **Second essai vert** : **5457 passés, 0 échec, 17 ignorés**
    (`gate-ci-e958fb169-r2.log`, code 0).
  - Frontend : **non touché**, aucun gate frontend lancé.
  - **E2E (Playwright) : non lancé** — à la clôture, avec le second `gate-ci` (consignes de l'Epic 35).
- **Mutations** : onze jouées, onze tuées (Change Log).
- **Choix** : C-36-4-5 (aide `audit_traces.rs`, `create_key_via_http` déplacée, multiplicité).
- **Écart** : les listes d'actions relevées diffèrent de la lecture (deux lettrages de contre-passation en caisse) ; écrit au
  Change Log. Aucune migration.

### File List

- `crates/kesh-db/src/repositories/invoice_settlements_write.rs`, `supplier_invoices.rs`, `reconciliation_cancel.rs`
- `crates/kesh-api/src/routes/invoices.rs`, `routes/supplier_invoices.rs`, `routes/reconciliation.rs`
- `crates/kesh-api/tests/common/audit_traces.rs` (neuf), `common/mod.rs`, `audit_acteur_garde.rs`,
  `invoice_settlement_response_e2e.rs`, `supplier_settlement_cancel_e2e.rs`, `reconciliation_e2e.rs`,
  `bank_accounts_closed_e2e.rs`, `letterings_e2e.rs`
- `crates/kesh-db/tests/` : `credit_notes_repository.rs`, `invoice_amount_due_parity.rs`, `invoice_settlement.rs`,
  `invoice_write_off.rs`, `lettering_documents.rs`, `lettering_documents_backfill.rs`, `letterings.rs`, `open_items.rs`,
  `payment_batches_repository.rs`, `reconciliation_cancel.rs`, `supplier_invoices_draft.rs`, `supplier_invoices_repository.rs`,
  `support/lettering_documents.rs`
- `crates/kesh-report/tests/aged_receivables.rs`, `vat_report_write_off.rs`
- `docs/api-external.md`, `docs/manual/fr/admin-manual.tex`, `.pdf`, `docs/manual/fr/user-manual.tex`, `.pdf`
- `_bmad-output/implementation-artifacts/36-4-acteur-annulation-reglement.md`, `…-review-prompt-p1.md`,
  `e36-choix-autonomes-36-4.md`, `sprint-status.yaml`

## Change Log

- 2026-11-14 — **Fiche rédigée** (Opus 5.5, agent de rédaction des fiches de l'Epic 36, worktree `kesh-36-4`), code lu à
  `9d5a142df`. Étape 0 et C-e360-7 repris tels quels ; prémisse de #1131 relue au code (la route du dé-rapprochement porte
  la clé, `routes/reconciliation.rs:5955` ; la perte est dans les deux `cancel_settlement_in_tx`,
  `invoice_settlements_write.rs:917`, `supplier_invoices.rs:2815`). Inventaire par le symptôme : 8 appels de production,
  77 appels de tests dans 17 fichiers ; garde recomptée (93/98 → 87/92 attendus). Choix C-36-4-1 à -4. Aucune migration.
  Statut : `ready-for-dev`. Validation P1 à lancer.
- 2026-11-14 — **Validation P1 (Sonnet) : 0 CRITICAL / 0 HIGH / 2 MEDIUM / 3 LOW → remédiés**
  (`kesh-gate-logs/e36-4/validation-p1.md`). Remédiation par l'agent de rédaction (Opus 5.5), outil Edit seul ; prémisses
  relues au code à `9d5a142df` :
  - **V1-1 (MEDIUM), confirmé** : `user-manual.tex:4017` « saisi sur la facture, le solde de son reste ou leur annulation,
    une facture fournisseur » et `:4022-4023` « une ligne écrite avant la 0.17.7 ne porte pas la marque » lus ; contrôle
    aplati `pdftotext … | /usr/bin/grep -c 'ou leur annulation, une facture fournisseur'` → `1` dans `admin-manual.pdf` et
    dans `user-manual.pdf`. AC6 étendu (« ou leur annulation » retiré de la liste, « annulation de la facture »
    fournisseur, « avant la 0.17.7 » complété, absence des anciennes formulations exigée dans les PDF, symptômes greppés
    élargis) ; *Ce qui se lit au code*, D8 et la tâche *Documents* suivent (`:4016-4023`).
  - **V1-2 (MEDIUM), confirmé** : `git grep -nE 'actor_api_key_id:\s*Option<i64>' -- crates/` hors `tests/` rend
    `crates/kesh-api/src/routes/reconciliation.rs:5982:    actor_api_key_id: Option<i64>,` et
    `crates/kesh-db/src/repositories/reconciliation_cancel.rs:581:    actor_api_key_id: Option<i64>,`, que D3 convertit
    toutes deux ; 30 lignes dont 6 champs `pub` et 1 doc-comment (23 lignes de paramètre, recompté). Valeur attendue :
    **28** (21 lignes de paramètre), non 29. Jeton `29` greppé dans la fiche et le registre (`\b(29|30|23|26)\b`) :
    corrigé dans D4, AC5 et C-36-4-2 (annoté) ; restent `27/29` (`AdminSansCle`, sans rapport) et `26` (appels de tests).
    *Ce qui se lit au code* nomme les deux lignes et les chiffres du doc-comment de la garde ; D4 en demande le recompte au
    développement, « 26 fonctions » compris.
  - **V1-3 (LOW), appliqué** : cas du solde dans AC1 (`write-off` puis annulation par clé), test
    `an_api_key_cancels_a_write_off_and_is_named_everywhere` ; le corps de `cancel_settlement_in_tx` ne lit pas
    `settlement_type` (`invoice_settlements_write.rs:917-1121`).
  - **V1-4 (LOW), appliqué** : mutations miroir M1' (geste fournisseur), M2' (délettrage fournisseur,
    `supplier_invoices.rs:2899-2902`), M3' (contre-passation client, `invoice_settlements_write.rs:1034`).
  - **V1-5 (LOW), confirmé et appliqué** : `invoice_settlement_response_e2e.rs:26` n'inclut que
    `common/autour_appel.rs`, `supplier_settlement_cancel_e2e.rs:24-26` `mod common { pub mod jour; }` ; aide commune
    `common/audit_traces.rs` prescrite (`max_audit_id`, `audit_depuis`, `assert_toutes_a_la_cle`), déclarée dans
    `common/mod.rs` et dans les modules `common` en ligne des deux hôtes.
  - Une passe 2 est requise (deux MEDIUM en P1).
- 2026-11-14 — **Validation P2 (Opus) : 0 CRITICAL / 0 HIGH / 0 MEDIUM / 5 LOW** (`kesh-gate-logs/e36-4/validation-p2.md`).
  **Validation terminée** : trend P1 Sonnet 0/0/2/3 → P2 Opus 0/0/0/5 ; la dernière passe ne rend rien au-dessus de LOW. Les
  cinq LOW traités à l'étape 0 du développement (entrée suivante).
- 2026-11-14 — **Développement** (Opus 5.5, agent de développement de l'Epic 36, worktree `kesh-36-4`).
  - **Étape 0.** Lignes relues à `846d64e98` : `invoice_settlements_write.rs:917-924` (`cancel_settlement_in_tx(…, user_id:
    i64, aujourd_hui)`), `:1021-1024` (`Actor { user_id, api_key_id: None }`), `:1034` (`Acteur::utilisateur(user_id)`),
    `:1100-1102` (`NewAuditLogEntry::user(user_id, "invoice.settlement_cancelled"`), `:1142-1149` ;
    `supplier_invoices.rs:2815-2821`, `:2899-2902`, `:2914`, `:2954-2956`, `:3003-3011` ; `reconciliation_cancel.rs:576-584`,
    `:713-720`, `:731-737`, `:744-747`, `:797-801`, `:831-847` ; routes `invoices.rs:1396-1403`, `supplier_invoices.rs:1022-1029`,
    `reconciliation.rs:5954-5955`, `:5977-6007` — toutes conformes à la fiche. `inventaire-appels.py` relancé
    (`kesh-gate-logs/e36-4/inventaire-appels-846d64e98.txt`) et `git grep` (`git-grep-appels-846d64e98.txt`) : **identiques** à
    ceux de `9d5a142df` (`diff` vide). Garde recomptée (`recompte-garde.py`, versionné sous `kesh-gate-logs/e36-4/`) : 93 entrées,
    98 sites, ventilation de la fiche.
  - **LOW de la validation P2** : **V2-1** — `:469` « les quatre tests neufs » → « cinq » ; en-tête « Validation P1 à lancer »
    → « Validation terminée » ; `:156` `:4016-4022` → `:4016-4023`. **V2-2** — `admin-manual.tex:3212` lu : la première liste
    « (création, modification, validation, paiement depuis la fiche ou par un lot de paiement, annulation) » ; AC6 étendu, et le
    `.tex` dit « annulation de la facture » dans les deux listes. **V2-3** — `invoice_settlement_response_e2e.rs:26-27`
    (`#[path = "common/autour_appel.rs"] mod autour_appel;`), `supplier_settlement_cancel_e2e.rs:24-26` (`mod common { pub mod
    jour; }`), `common/mod.rs:204` (`pub async fn create_key_via_http(`) : la fonction est **déplacée** dans
    `common/audit_traces.rs` et réexportée (C-36-4-5) ; l'assertion compare les actions avec leur multiplicité. **V2-4** —
    `invoice_settlements_write.rs:1016-1026` (délettrage avant la contre-passation et le geste) : l'aide nomme **toutes** les
    traces fautives ; la colonne « Rouge attendu » le dit. **V2-5** — `:880` `if write_off_exists && settlement_type !=
    "write_off" {` : AC1 précise que seul le bloqueur lit le type.
  - **Écart à la fiche (listes d'actions)** : les listes relevées sur les montages réels diffèrent de la lecture —
    **deux** `lettering.created` quand le règlement est en caisse (`1000`, compte lettrable : la contre-passation lettre une
    paire par ligne libre d'un compte lettrable, `journal_entries.rs` étape (5) « chaque ligne de l'origine sur un compte
    lettrable, non lettrée, forme avec son miroir … un groupe `reversal` »), **un seul** pour le solde (contrepartie `4000`) et
    pour les dé-rapprochements (compte bancaire). Relevé au premier run des rouges (`rouges-run1.log` : les deux témoins de
    session rougissaient sur la liste, non sur l'acteur), corrigé avant le second (`rouges-run2.log`).
  - **Écart à la consigne Edit** : aucun. Toutes les modifications de code, de tests, de documents et de la fiche par l'outil
    Edit (ou Write pour `audit_traces.rs` et le prompt de revue) ; `cargo fmt` seul a reformaté. Les remplacements multiples
    (`replace_all`) de l'outil Edit ont touché à tort cinq appels de `supplier_invoices::cancel` / `cancel_in_tx` dans
    `kesh-db/tests/supplier_invoices_repository.rs` (même suite d'arguments) : le compilateur les a refusés (`expected i64,
    found Acteur`, lignes 2134, 2254, 2313, 2961, 3279), rétablis un par un. L'outil n'a pas manqué ; le motif choisi était
    trop large.
  - **Rouges** (`80bc35072`, code de production de la base) : `kesh-gate-logs/e36-4/rouges.log`, `rouges-run2.log` — 37 tests,
    32 passés, 5 échecs, exactement les cinq tests par clé, chacun parce que les traces du geste sont à `("user", None, 1)`
    (`reconciliation.cancelled` déjà à la clé en AC3) ; témoins de session (AC4) verts.
  - **Listes relevées** (actions du geste, multiplicité comprise) : règlement client en caisse (route) — `lettering.removed`,
    `journal_entry.created`, `journal_entry.reversed`, `lettering.created` ×2, `invoice.settlement_cancelled` ; solde —
    même liste, `lettering.created` ×1 ; paiement fournisseur en caisse (route) — idem avec
    `supplier_invoice.settlement_cancelled`, `lettering.created` ×2 ; dé-rapprochement client — `lettering.removed`,
    `journal_entry.created`, `journal_entry.reversed`, `lettering.created`, `invoice.settlement_cancelled`,
    `reconciliation.cancelled` ; dé-rapprochement fournisseur — idem avec `supplier_invoice.settlement_cancelled`. Les mêmes
    listes, à l'utilisateur, aux témoins de session : le nombre et les actions des entrées ne changent pas.
  - **Correctif** `c5d67ce42` (D1 à D4) ; conversion de 77 appels de tests dans 17 fichiers (`git diff 80bc35072 c5d67ce42
    --stat -- 'crates/*/tests/*'` : 18 fichiers, la garde comprise).
  - **Garde recomptée** (`recompte-garde.py` → `kesh-gate-logs/e36-4/recompte-garde-apres.txt`) : **87 entrées, 92 sites** ;
    `AConvertir` 47/48, `AdminSansCle` 27/29, `PreAuth` 4/4, `Systeme` 2/3, `Definition` 3/3, `Restauration` 1/2, `Mcp` 2/2,
    `Fixture` 1/1 (somme 87/92) ; `User` 69/74, `ApiKeyNone` 8/8, `Utilisateur` 10/10 (somme 87/92). Angle mort 1 : `git grep
    -nE 'actor_api_key_id:\s*Option<i64>' -- crates/ ':!crates/*/tests/*'` → **28** lignes (`actor-api-key-id-option-apres.txt`),
    dont 6 champs `pub` et 1 doc-comment (`entities/audit_log.rs:236`), soit **21** lignes de paramètre ; aucune dans
    `reconciliation_cancel.rs`, et `routes/reconciliation.rs:5982` n'y est plus. Le doc-comment de la garde dit « 26 fonctions …
    à la fiche de la 35-8a1, P3 — relevé daté, non recompté » : je ne l'ai pas recompté.
  - **Mutations** (`kesh-gate-logs/e36-4/mut/`, `muter.py` + `jouer.sh`, sur l'arbre commité `c5d67ce42`, une par prise de
    verrou, restauration `git restore --worktree` + `touch`, chaque journal finit par « restauré ») — **onze tuées, aucune
    survivante** :
    - M1 (geste client par `user(acteur.user_id`) : garde (`chaque_acteur_sans_cle_…_site_par_site`), AC1 ×2, AC3 a ;
    - M1' (fournisseur) : garde, AC2, AC3 b ;
    - M2 (délettrage client sans clé) : garde, AC1 ×2, AC3 a ; M2' : garde, AC2, AC3 b ;
    - M3 (contre-passation fournisseur `Acteur::utilisateur`) : garde, AC2, AC3 b ; M3' (client) : garde, AC1 ×2, AC3 a ;
    - M4 (route client, clé jetée par `.filter`) : AC1 ×2 seuls ; M5 (route fournisseur) : AC2 seul ;
    - M6 (`reconciliation_cancel`, appel client) : AC3 a seul ; M7 (appel fournisseur) : AC3 b seul — premier essai
      « échec d'application » (motif d'avant `cargo fmt`, `M7-essai1-motif-absent.log`), motif corrigé et rejoué ;
    - M8 (route du dé-rapprochement) : AC3 a, AC3 b, `an_api_key_cancels_and_is_named_in_the_audit`.
  - **Documents** `e958fb169` : `api-external.md` (`:404`, `:565`, `:653`, `:739`), `admin-manual.tex:3212` (« annulation de la
    facture » dans les deux listes, « ou leur annulation » retiré, phrase « Depuis la v0.17.8 », dates), `user-manual.tex:4016-4024` ;
    PDF régénérés (`make -C docs/manual fr` sous `.gate-front.lock`, `kesh-gate-logs/e36-4/manuels-make-c5d67ce42.log` ; la
    brochure, régénérée sans changement de source, rétablie par `git restore`). Contrôle aplati (E6), `admin-manual.pdf` /
    `user-manual.pdf` : « contre-passation comprise, au nom du créateur de la clé » `0` / `0` ; « ou leur annulation, une facture
    fournisseur » `0` / `0` ; « est attribuée à la clé dans toutes ses traces » `1` (admin) ; « porte la marque, contrepassation et
    délettrage compris » `1` (user — `pdftotext` rend « contrepassation » : le trait d'union tombe à la coupure de ligne).
    Symptôme greppé (`restent au nom du créateur`, `reste au nom du créateur`, `leur annulation`, `dé-rapprochement compris`,
    `ne portent que l'utilisateur`, `avant la 0.17.7`, `avant la v0.17.7`, `créateur de la clé`) sur `docs website README.md
    crates frontend/src` : restent `admin-manual.tex:2549` (relevé daté de la 0.17.7, non touché), `user-manual.tex:4019` (la
    liste de ce qui reste au créateur, juste), `api-external.md:1103` (« leur annulation » des verrous, sans rapport), les
    commentaires de code sur le MCP et l'imputabilité (justes), `invoice_format.rs:196` et `invoice-number-format.ts:174` (« avant
    la 0.17.7 » du format de numérotation, sans rapport), `InvoiceSettlements.svelte:2`, `+page.svelte:158` (« leur annulation »,
    sans rapport). Les autres `AConvertir, Epic 36` du code (`credit_notes.rs:706`, `invoice_settlements_write.rs:341`, `:705`,
    `opening_complement.rs:719`, `supplier_invoices.rs:1768`, `:2625`) sont d'autres fonctions, hors périmètre (#431).
  - **Gates du passage en `review`** : `test-fast --ci` vert à `c5d67ce42` (5450/5450, 13 ignorés) ; `gate-ci` vert à
    `e958fb169` au second essai (5457 passés, 17 ignorés ; premier essai rouge sur KF-061 #1126, sans rapport) ; frontend non
    touché ; **E2E non lancé** (à la clôture, avec le second `gate-ci`). Détail au Dev Agent Record.
  - Statut : `review`. Prompt de la revue P1 : `36-4-acteur-annulation-reglement-review-prompt-p1.md`.
- 2026-11-14 — **Revue P1 (trois lentilles) : B 0/0/0/2, E 0/0/0/3, A 0/0/0/4** (`kesh-gate-logs/e36-4/review-p1-{B,E,A}.md`),
  rien au-dessus de LOW. **Remédiation de la revue P1** (`a8b1ebb08`), sur décision de l'orchestrateur (« Tarir la source » :
  les LOW se corrigent dans la story, puis une passe ciblée relit ce seul correctif). Chaque prémisse relue au code :
  - **B1**, confirmé : `reconciliation_cancel.rs:786-788` lu, « trois lignes au total (celle-ci, `invoice.settlement_cancelled`,
    `journal_entry.reversed`) », contre six relevées (`INVOICE_UNRECONCILE_ACTIONS`). Commentaire récrit (six traces, sans
    `lettering.removed` pour un partiel, la seule `reconciliation.cancelled` pour un simple détachement) ; aucune ligne de code.
  - **B2 = E1**, confirmé : aucun test par clé du simple détachement. Ajouté `an_api_key_unlinks_one_side_and_is_named`
    (`reconciliation_link_entry_e2e.rs`, montage de `s34_11b_ac4_annuler_un_cote_puis_l_autre`) : une seule trace,
    `reconciliation.cancelled`, à la clé.
  - **E2**, confirmé : ajouté `an_api_key_cancels_a_partial_settlement_and_is_named_everywhere` (`invoice_settlement_response_e2e.rs`,
    règlement de 40.00 sur 100.00) ; `cancel_by_key` prend `piece_lettree` et affirme la présence **ou l'absence** du lettrage de
    pièce. Liste relevée : `PARTIAL_SETTLEMENT_CANCEL_ACTIONS` (sans `lettering.removed`, deux `lettering.created`). **La
    contre-passation à l'exercice suivant n'est pas testée par clé, parce que ce chemin ne la produit pas** :
    `journal_entries::reverse_owned_in_tx`, qu'appellent les deux `cancel_settlement_in_tx`, passe
    `ReversalDating::Today(aujourd_hui)` (`journal_entries.rs`, `reverse_owned_in_tx` → `reverse_in_tx_inner(…,
    ReversalDating::Today(aujourd_hui))`) ; la datation `NextFiscalYearStart` n'est prise que par
    `reverse_at_next_fiscal_year_start_in_tx` (Story 29-1b), hors de l'annulation d'un règlement. Aucun montage à écrire.
  - **E3 = A1**, confirmé : `cancelling_a_supplier_reconciliation_by_the_route` n'avait pas de témoin « toutes traces ». Ajouté
    (`assert_toutes_a_l_utilisateur`, `seeded.admin_user_id`, `SUPPLIER_UNRECONCILE_ACTIONS`) ; la cellule `:388` de la fiche
    accordée, et les deux tests neufs ajoutés au tableau.
  - **A2**, confirmé : `/usr/bin/grep -niE 'fmt' clippy-1.log` ne rend que `Checking powerfmt`. Le Dev Agent Record cite
    `test-fast-ci-c5d67ce42.log:4` (`▶ cargo fmt --check`). Le journal du gate de la remédiation,
    `test-fast-ci-remediation-p1.log`, porte son commit en première ligne (`# commit a8b1ebb08…`, par `git rev-parse HEAD` dans
    la même commande). *(Corrigé à la passe ciblée P2, R1 : cette phrase disait « les journaux de la remédiation » ;
    `manuels-make-remediation-p1.log` et `remediation-p1-cibles.log` portent en tête `c52e10b50` **et l'arbre de travail**
    non commité, la remédiation n'étant pas encore commitée quand ils ont été écrits.)*
  - **A3**, confirmé : `api-external.md:565` dit « `lettering.removed` quand la pièce était lettrée — un règlement partiel n'en
    écrit pas ».
  - **A4**, confirmé : les deux `.tex` rattachent « un simple détachement ne contre-passe rien » à la phrase de l'écriture seule,
    hors de « depuis la 0.17.8 » ; `user-manual.tex` reprend la parenthèse de l'admin (« un règlement ou un solde de facture
    client, le paiement d'une facture fournisseur »). PDF régénérés (`manuels-make-remediation-p1.log`, en tête `c52e10b50` + l'arbre de travail, avant le commit `a8b1ebb08` qui
    porte ces PDF ; la
    brochure rétablie par `git restore`). Contrôle aplati (E6), admin / user : « délettrage ; un simple détachement » `0` / `0`,
    « compris, et un simple détachement » `0` / `0`, « client ou fournisseur, par sa route » `0` / `0`, « une écriture rapprochée
    seule, et un simple détachement ne contre-passe rien » `1` / `1`. Sonde positive sur les PDF de `c52e10b50` : « délettrage ; un
    simple détachement » `1` (admin), « compris, et un simple détachement » `1`, « client ou fournisseur, par sa route » `1` (user).
  - **Gate de remédiation** : `scripts/test-fast.sh --ci`, bases remises à neuf, au commit `a8b1ebb08` :
    **5452 exécutés, 5452 passés, 13 ignorés** (`test-fast-ci-remediation-p1.log`, première ligne `# commit
    a8b1ebb08000f4a1cf33903e71af30c9792c27dc`, `Summary [ 329.009s]`) — les trois tests neufs ou complétés y passent
    (`an_api_key_unlinks_one_side_and_is_named`, `an_api_key_cancels_a_partial_settlement_and_is_named_everywhere`,
    `cancelling_a_supplier_reconciliation_by_the_route`). ⚠️ Les deux tests neufs ont été écrits après le correctif : ils n'ont
    pas de rouge sur la base. Ni `gate-ci` ni E2E (à la clôture). Un premier run ciblé des trois tests neufs, mis en file derrière
    d'autres gates, a été arrêté par son PID avant de démarrer (`remediation-p1-cibles.log` ne porte que sa ligne d'en-tête) :
    `test-fast --ci` les couvre.
  - Prompt de la passe ciblée P2 : `36-4-acteur-annulation-reglement-review-prompt-p2-ciblee.md`.
- 2026-11-14 — **Passe ciblée P2 (Haiku) : 0 CRITICAL / 0 HIGH / 0 MEDIUM / 1 LOW** (`kesh-gate-logs/e36-4/review-p2-ciblee.md`) :
  la remédiation n'introduit aucun défaut ; **R1** (compte rendu) — l'entrée précédente disait que « les journaux de la
  remédiation » portent leur commit en tête, alors que seul `test-fast-ci-remediation-p1.log` porte `a8b1ebb08` ;
  `manuels-make-remediation-p1.log` et `remediation-p1-cibles.log` portent `c52e10b50` et l'arbre de travail. Corrigé dans la
  fiche seule (annotation datée sur l'entrée précédente). **Boucle de revue close** : P1 B 0/0/0/2, E 0/0/0/3, A 0/0/0/4 →
  remédiés `a8b1ebb08` → P2 ciblée Haiku 0/0/0/1 (compte rendu seul, aucune ligne de code).
- 2026-11-14 — **Clôture** (Opus 5.5), au dernier commit de code `a8b1ebb08` (HEAD `f8e313142` n'ajoute que la fiche et le prompt
  P2), voie 1, une commande lourde à la fois sous son verrou :
  - `scripts/gate-ci.sh kesh_ci_e364` : **5459 passés, 0 échec, 17 ignorés**, code 0 (`kesh-gate-logs/e36-4/gate-ci-cloture.log`,
    première ligne `# commit f8e313142…`, sortie brute `gate-ci-cloture.brut.log`). KF-061 (#1126) n'a pas rougi.
  - **E2E complet** sur `kesh_e2e_e364` recréée à neuf (vide, migrée au démarrage), port 3364, recette entière de `e34-commun.md`,
    dossiers sous `target/e2e-r1/`, frontend `npm run build` (`e2e-build-frontend.log`), backend `cargo build -p kesh-api`
    (`e2e-build-backend.log`) ; script `kesh-gate-logs/e36-4/e2e-cloture.sh`, journal `e2e-cloture.log` (commit en tête, après
    la ligne de `mem-guard` ; `/health` : `smtpConfigured: true`) : **433 tests, 408 passés, 7 échecs, 18 ignorés** (17,7 min).
    Les sept échecs sont exactement ceux de la KF-029 (#97) de `docs/testing.md` § « Les échecs attendus » :
    `mode-expert.spec.ts:26`, `mode-expert.spec.ts:41`, `onboarding-path-b.spec.ts:65`, `onboarding-path-b.spec.ts:92`,
    `onboarding.spec.ts:57`, `onboarding.spec.ts:77`, `onboarding.spec.ts:150`. Aucun échec de pollution. `ss -ltnp` avant et
    après : 3364 libre ; le backend arrêté par son PID.
  - Statut : `done`. Bases `kesh_e364`, `kesh_e2e_e364`, `kesh_ci_e364` et worktree gardés jusqu'à l'intégration.
