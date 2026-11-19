# Story 48-14 : Story de solde de l'Epic 48 — dette des revues du règlement groupé (#1263, #1262, #1261, #1255)

Status: ready-for-dev

*Fiche rédigée le 2026-11-19 (Opus 5.5), étape `bmad-create-story`, sans code. Worktree `/home/gcorbaz/devel/kesh-48-14`,
branche `story/48-14-solde-dette-epic`, partie de `epic/48-corrections-0-20-0` (`59bdd104f`) ; **toutes les lignes citées
sont lues à cette tête**. Release **0.20.0** (coupure le jeudi 2026-11-26 au soir). Story de solde de l'épique
(`CLAUDE.md`, § *Priorités des défauts*) ; **plafond fixé au kickoff : quatre issues**, pas une de plus
(`epic-48-corrections-0-20-0.md`, § *48-14*). À la coupure, si le temps manque, **la 48-14 sort la première** (décisions
de Guy des 2026-11-18 et 2026-11-19) : cela ne change pas son périmètre. Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e4814/`.
Bases propres : `kesh_e4814`, `kesh_ci_e4814`, `kesh_e2e_e4814`.*

> **Écritures réelles.** Le règlement groupé tient des écritures réelles en production depuis la 0.19.0. Cette story
> ne change **aucune** écriture passée ni aucun règlement existant : le dé-rapprochement (#1255) est réécrit à
> comportement égal, et un critère (AC9) le prouve par un test qui compare l'état complet avant et après.
>
> **`kesh-db` est touché** (`repositories/grouped_settlements.rs`, #1255) : **gate complet même en cours de boucle**
> (`CLAUDE.md`, § *Pendant une boucle de revue*, exception `kesh-db`).

## Story

En tant que **comptable qui règle plusieurs factures par une transaction du relevé**, je veux que la fenêtre du
règlement groupé ne me demande jamais de choisir un compte qu'elle n'affiche pas, et que la liste du rapprochement ne
garde pas « à rapprocher » une transaction que je viens de régler ; en tant que **mainteneur**, je veux un seul code
pour défaire le lien d'un règlement groupé, client ou fournisseur, et un test de la sauvegarde sur disque plein qui
dise toujours la même chose.

Quatre issues, quatre lots indépendants :

| Lot | Issue | Label | Objet | Où |
|---|---|---|---|---|
| A | #1263 | bug, P3 | « Choisissez le compte de l'écart » affiché sans sélecteur après une relecture bloquée | `GroupSettlementModal.svelte` |
| B | #1262 | bug, P4 | liste non relue quand le règlement aboutit alors que la fenêtre a été fermée puis rouverte | `GroupSettlementModal.svelte`, `ReconciliationProposals.svelte` |
| C | #1255 | technical-debt, P3 | dé-rapprochement commun aux bras client et fournisseur | `kesh-db/src/repositories/grouped_settlements.rs` |
| D | #1261 | known-failure, P3 | [KF-063] `disque_sature_pendant_l_ecriture` variable | `kesh-api/tests/admin_auto_backup_e2e.rs` |

## Ce qui se lit au code (`59bdd104f`)

### Lot A — #1263 : un refus du compte sans sélecteur

Le refus `difference_account_required` déclenche la relecture des restes dus, puis trois issues seulement sont prévues
(`frontend/src/lib/features/reconciliation/GroupSettlementModal.svelte`) :

- `:537` — `serverWantsAccount = notReread.length > 0;` (relus tous, la demande du serveur tombe) ;
- `:538` — `if (showAccount) {` → phrase sous le sélecteur (`:543`–`:552`) ou rien (`:539`, la phrase est déjà au pied) ;
- `:555` — `if (notReread.length === 0 && blocks.length === 0) {` → « Les restes dus ont été relus : vérifiez l’écart,
  puis enregistrez de nouveau. » (`:561`–`:562`).

Sinon le refus **tombe** dans le cas général : `:570` — `if (isDifferenceAccountRefusal(f.errorCode, f.details,
sentAccountId) && showAccount) {` est faux puisque `showAccount` l'est, et `:577` — `refusals = [...refusals, text];`
ajoute `text` (`:508` — `const text = refusalText(f, sentAccountId);`), c'est-à-dire le libellé
`failed-proposal-label.ts:556`–`:559` :

```ts
			if (reason === 'difference_account_required') {
				return i18nMsg(
					'reconciliation-failed-group-difference-account-required',
					'Choisissez le compte de l’écart.'
```

**Quand tombe-t-on ?** `showAccount` (`:223`–`:229`) est faux quand l'écart est nul sans demande du serveur, ou trop
grand (`!tooLarge`), ou illisible (`!blocks.includes('unreadable')`). Après la relecture, les états qui atteignent `:570`
sont donc exactement ceux où `showAccount` est faux **et** `blocks` non vide (l'écart nul sans blocage s'arrête à `:555`) :

1. relus tous, écart nul, mais un arrondi sans compte d'arrondi désigné (`group-settlement.ts:255` —
   `if (dec.rounding !== 0n && input.roundingDesignated === false) blocks.push('rounding-not-configured');`) — le cas
   de l'issue ;
2. relus tous (ou non), écart au-delà du plafond (`group-settlement.ts:262` — `} else if (abs(dec.gap) > max)
   blocks.push('gap-too-large');`) — l'autre cas de l'issue ; il vaut aussi quand une facture n'a pas pu être relue
   (`notReread.length > 0`) et qu'une autre, relue, a fait grandir l'écart ;
3. plafond illisible (`'unreadable'`, `group-settlement.ts:260`–`:261`).

Dans les trois, l'écran affiche déjà la cause : l'avis d'arrondi (`:918` — `{#if
blocks.includes('rounding-not-configured')}`) ou la phrase du pied (`:249`–`:257`, `gap-too-large` ;
`:243`–`:247`, `unreadable`). La phrase « Choisissez le compte de l’écart » s'y ajoute à tort : aucun sélecteur n'est
affiché, et choisir un compte ne débloquerait rien. Rien n'est écrit (le refus est un refus).

**Inventaire par le symptôme** (« la phrase du compte demandé, sans sélecteur ») :

```
$ git grep -n "difference_account_required\|Choisissez le compte de l" -- frontend/src crates/kesh-i18n/locales/fr-CH | grep -v "\.test\.ts"
crates/kesh-i18n/locales/fr-CH/messages.ftl:1816:reconciliation-failed-group-difference-account-required = Choisissez le compte de l’écart.
frontend/src/lib/features/reconciliation/GroupSettlementModal.svelte:160:	 * Revue P2 (L-1) : le serveur a demandé un compte de l'écart (`difference_account_required`) ;
frontend/src/lib/features/reconciliation/GroupSettlementModal.svelte:259:					// Le même texte que le refus du serveur (`difference_account_required`).
frontend/src/lib/features/reconciliation/GroupSettlementModal.svelte:262:							reason: 'difference_account_required',
frontend/src/lib/features/reconciliation/GroupSettlementModal.svelte:464:	 * `difference_account_required`, les seuls refus du groupe qui portent sur l'écart, donc sur
frontend/src/lib/features/reconciliation/GroupSettlementModal.svelte:523:			failureReason(f.details) === 'difference_account_required';
frontend/src/lib/features/reconciliation/failed-proposal-label.ts:23: * `too_few`, `too_many`, `duplicate`, `difference_account_required`) et
frontend/src/lib/features/reconciliation/failed-proposal-label.ts:204: * `difference_account_required`). Dans un groupe, `ACCOUNT_NOT_FOUND` et `ACCOUNT_NOT_POSTABLE`
frontend/src/lib/features/reconciliation/failed-proposal-label.ts:227:			return failureReason(details) === 'difference_account_required';
frontend/src/lib/features/reconciliation/failed-proposal-label.ts:556:			if (reason === 'difference_account_required') {
frontend/src/lib/features/reconciliation/failed-proposal-label.ts:559:					'Choisissez le compte de l’écart.'
frontend/src/lib/features/reconciliation/group-settlement.ts:234:	 * Le serveur a refusé l'envoi faute de compte de l'écart (`difference_account_required`) :
```

Les sites qui **affichent** la phrase sont trois ; chacun est classé :

| Site | Affiché quand | Sélecteur visible ? | Verdict |
|---|---|---|---|
| `blockTexts`, `:258`–`:264` (`'account-required'`) | `blocks` contient `account-required` | **oui, par construction** : `group-settlement.ts:263` ne le pousse que si l'écart est dans le plafond et lisible, `:264`–`:265` que si l'écart est nul et que le serveur le demande ; dans les deux cas `showAccount` est vrai (≥ 2 factures, ni `tooLarge` ni `unreadable`) | sain |
| `accountError`, `:549`–`:552` | sous `if (showAccount)` (`:538`) | oui | sain |
| `refusals`, `:577` | la chute décrite ci-dessus | **non** | **le défaut** — dans le périmètre |

Les autres refus du compte de l'écart (`ACCOUNT_NOT_FOUND`, `ACCOUNT_NOT_POSTABLE`, `ACCOUNT_IN_FOREIGN_CURRENCY`,
`VALIDATION_ERROR` / `counterparty_equals_bank_ledger`, `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`) qui tombent à `:577`
quand le sélecteur est caché nomment **le compte envoyé** et sa faute : c'est le comportement écrit au manuel (« ou
avec les autres refus quand le sélecteur n'est pas affiché ») ; ils ne demandent pas de choisir un compte. Hors du
symptôme, inchangés.

### Lot B — #1262 : la liste non relue

`GroupSettlementModal.svelte:650`–`:656` :

```ts
				if (!session.isCurrent(mySession)) {
					// Fermée pendant l'envoi : le règlement a eu lieu, et le toast le dit (revue P1,
					// E-L7) ; le parent relit sa liste si aucune modale n'est ouverte, une modale
					// rouverte n'est pas touchée.
					toast.success(successText);
					if (!open) onSuccess();
					return;
```

Rouverte sur une autre transaction, `open` est vrai : `onSuccess` n'est pas appelé, et rien d'autre ne relit la liste.

**Pourquoi la modale n'appelle pas simplement `onSuccess`** — à lire avant toute remédiation :

- `ReconciliationProposals.svelte:322`–`:328` : `onGroupSuccess` **ferme** la modale (`groupOpen = false;
  groupProposal = null;`) puis `await load();` — l'appeler fermerait la fenêtre rouverte ;
- **toutes les modales vivent dans la branche `{:else}` de `{#if loading}`** (`ReconciliationProposals.svelte:555` —
  `{#if loading}` ; `:568` — `{:else}` ; les modales de `:872` à `:928`, dont la confirmation « déjà comptabilisée »,
  `:922` — `<AlreadyBookedConfirmDialog`, ouverte tant que `pendingGesture !== null` ; `:930` — `{/if}`), et `load()`
  pose `pendingGesture = null;` (`:182`) puis `loading = true;` (`:184`) : **toute relecture démonte la modale ouverte**, qui se remonte à neuf (son `$effect`
  d'ouverture, `GroupSettlementModal.svelte:276`–`:305`, remet tout à zéro) ou ne se remonte pas si la liste relue est
  vide (`:561` — `{:else if proposals.length === 0}`). Relire **pendant** qu'une modale est ouverte détruirait le
  travail en cours de l'utilisateur.

La relecture doit donc être **différée** jusqu'à la fermeture de la modale ouverte (D3).

Le même défaut touche une **autre** fenêtre que celle de l'issue : la garde `if (!open)` ne regarde que la modale du
règlement groupé ; si l'utilisateur a ouvert entre-temps *Écriture manuelle*, *Éclater* ou *Rattacher*, `onSuccess` est
appelé, `load()` démonte la modale ouverte. Le correctif D3 couvre ce cas pour la fin d'un envoi du règlement groupé.

**Inventaire par le symptôme** (« la fin d'un envoi parti d'une modale fermée ne relit pas la liste, ou relit en
détruisant la modale ouverte ») :

```
$ git grep -n "if (!open)" -- 'frontend/src/*.svelte'
frontend/src/lib/components/invoices/ContactPicker.svelte:269:			if (!open) return;
frontend/src/lib/components/invoices/InvoiceForm.svelte:1128:		if (!open) conflictError = '';
frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte:132:		if (!open) return;
frontend/src/lib/features/reconciliation/GroupSettlementModal.svelte:655:					if (!open) onSuccess();
frontend/src/lib/features/reconciliation/LinkSettlementModal.svelte:176:				if (!open) onSuccess();

$ for f in ManualMatchModal.svelte TransactionSplitModal.svelte; do grep -n "onSuccess\|onOpenChange(false)\|submitting = true" frontend/src/lib/features/reconciliation/$f; done
ManualMatchModal.svelte : 134: submitting = true; 146: onSuccess(); 147: onOpenChange(false);
TransactionSplitModal.svelte : 318: submitting = true; 338: onSuccess(); 339: onOpenChange(false);
```

| Site | Verdict |
|---|---|
| `GroupSettlementModal.svelte:655` | **le défaut de #1262** — dans le périmètre |
| `LinkSettlementModal.svelte:176` (même garde, `// rouverte entre-temps n'est pas touchée (C-29-8-37)`) | **même défaut, autre modale** — hors périmètre (plafond de quatre issues) : **issue à ouvrir** (Lot B, D3 lui donne le patron) |
| `ManualMatchModal.svelte:146`–`:147`, `TransactionSplitModal.svelte:338`–`:339` : aucune garde de session ; la fin d'un envoi appelle `onSuccess` puis `onOpenChange(false)` quelle que soit la modale ouverte entre-temps | motif voisin de #1007 (la fin d'un envoi agit sur une fenêtre rouverte), **prémisse non jouée** (que la fenêtre se ferme pendant l'envoi, par Échap) — hors périmètre : **issue à ouvrir** si la prémisse se vérifie |
| `ContactPicker.svelte:269`, `InvoiceForm.svelte:1128`, `SettleInvoiceDialog.svelte:132` | autre usage de `open` (garde d'entrée, remise à zéro) ; `SettleInvoiceDialog` est le site de #1007, déjà tracé — hors symptôme |

### Lot C — #1255 : le dé-rapprochement en double

`crates/kesh-db/src/repositories/grouped_settlements.rs` : `cancel_in_tx` (`:1750`, bras client) et
`supplier_cancel_in_tx` (`:1982`, bras fournisseur) répètent mot pour mot :

| Bloc | Client | Fournisseur |
|---|---|---|
| verrou de l'écriture et de son exercice — `let fiscal_year_id: i64 = sqlx::query_scalar(` … `FOR UPDATE` | `:1791`–`:1801` | `:2000`–`:2010` |
| le lien défait — `"UPDATE bank_transactions \` … `AND status = 'reconciled' AND version = ?"`, puis `OptimisticLockConflict` si `rows_affected() != 1` | `:1826`–`:1840` | `:2034`–`:2048` |
| l'audit `"reconciliation.cancelled",` (même charge, sauf la liste : client `"invoices": [{invoice_id, settlement_id, number}]`, fournisseur `"supplier_invoices": [{supplier_invoice_id, number}]` — la clé **et** les éléments diffèrent) | `:1925`–`:1953` (`:1929`, `:1940`) | `:2109`–`:2136` (`:2113`, `:2124`) |

Ce qui **diffère** et doit rester propre à chaque bras (l'ordre des verrous est celui de l'unitaire de chaque famille,
commentaires `:1759`–`:1764` et `:1961`–`:1964`) :

- client : factures verrouillées (`:1781`–`:1789`) **avant** l'écriture ; suppression des règlements
  (`:1855` — `let retires = super::invoice_settlements_write::delete_grouped_settlements_in_tx(`) **avant** la
  contre-passation (`:1868`), puis les factures marquées ;
- fournisseur : factures rouvertes (`:2066` — `"UPDATE supplier_invoices SET status = 'open', {CLEAR_SETTLEMENT_SQL}, \`)
  **avant** la contre-passation (`:2084`) ;
- chacun appelle `ensure_open_in_tx` (`:1804`, `:2011`) après le verrou de l'exercice et
  `recheck_open_locked_in_tx` (`:1904`, `:2088`) en dernière acquisition.

**Les gardes statiques qui lisent ces fonctions** (une extraction les fait rougir si elles ne suivent pas) :

```
$ grep -rn "grouped_settlements" crates/kesh-api/tests/*.rs crates/kesh-db/tests/*.rs | grep -v "^crates/kesh-db/tests/grouped_settlements"   (extrait)
crates/kesh-db/tests/letterings_lexical.rs:653:            "kesh-db/src/repositories/grouped_settlements.rs::supplier_cancel_in_tx",
crates/kesh-api/tests/bank_account_closed_inventory.rs:244 / :260 / :959 / :963 / :1067 / :1079 (entrées cancel_in_tx, supplier_cancel_in_tx)
```

- `bank_account_closed_inventory.rs` — **premier inventaire** (`REGISTRE`) : rattache tout littéral `UPDATE`
  visant `bank_transactions` ou `supplier_invoices` à sa fonction englobante, diff **dans les deux sens** (en-tête
  `:9`–`:16`). Entrées actuelles : `(grouped_settlements.rs, "cancel_in_tx")` `:242`–`:248` et
  `(grouped_settlements.rs, "supplier_cancel_in_tx")` `:258`–`:264`. **Troisième inventaire** (`FENETRE`,
  `ANNULATIONS`) : les fonctions qui portent `ensure_open_in_tx` et leur revérification « en dernier »
  (`:952`–`:963`, `:1063`–`:1081`). La position « en dernier » se juge **dans le corps de la fonction**
  (`fautes_en_dernier`, `:1342`) : `recheck_open_locked_in_tx` doit suivre le dernier geste verrouillant — jetons des
  littéraux SQL (`GESTES_VERROUILLANTS_SQL`, `:908`–`:914`, dont `"UPDATE "` et `"FOR UPDATE"`) et appels de
  `GESTES_VERROUILLANTS_CODE` (`:924`–`:930` : `reverse_in_tx(`, `reverse_owned_in_tx(`, `cancel_settlement_in_tx(`,
  `dissolve_invoice_document_group_in_tx(`, `dissolve_supplier_invoice_document_group_in_tx(`) — et précéder le premier
  `audit_log::insert_in_tx(` du corps (`:1381`). Le doc-comment de la liste (`:916`–`:923`) l'écrit : « un geste
  verrouillant appelé sous un nom absent de cette liste, ou un SQL assemblé hors littéral, n'est pas suivi ». **Une
  extraction en aides sort donc les gestes de la vue de la garde**, si la liste ne suit pas (D4). C'est la seule preuve
  mécanique de l'ordre des verrous : AC9 compare un état final, non un ordre d'acquisition.
- `audit_acteur_garde.rs` : `git grep -n grouped_settlements -- crates/kesh-api/tests/audit_acteur_garde.rs` ne rend
  rien ; l'aide d'audit de D4 emploie `NewAuditLogEntry::par(acteur, …)`, qui n'est pas l'un de ses motifs : cette
  garde ne réagit pas à l'extraction.
- `letterings_lexical.rs:649`–`:660` : `supplier_cancel_in_tx` est un des cinq écrivains de
  `settlement_journal_entry_id`, avec `dissolve_supplier_invoice_document_group_in_tx(` **avant**
  `reverse_owned_in_tx(` / la contre-passation.

Un troisième site porte le même `UPDATE` et un audit `reconciliation.cancelled` voisin : l'unitaire
`reconciliation_cancel.rs:829`–`:843` et `:948`–`:967` :

```
$ git grep -n "matched_entry_id = NULL" -- crates/*/src/*
crates/kesh-db/src/repositories/grouped_settlements.rs:1828:         SET matched_entry_id = NULL, status = 'pending', auto_match_rejected_at = NULL, \
crates/kesh-db/src/repositories/grouped_settlements.rs:2036:         SET matched_entry_id = NULL, status = 'pending', auto_match_rejected_at = NULL, \
crates/kesh-db/src/repositories/reconciliation_cancel.rs:830:         SET matched_entry_id = NULL, status = 'pending', auto_match_rejected_at = NULL, \
```

L'issue nomme **les deux bras du groupe** ; l'unitaire est hors périmètre (D5) : **issue à ouvrir**.

Tests existants du dé-rapprochement groupé : `crates/kesh-db/tests/grouped_settlements.rs:1067`
(`unreconciling_a_group_undoes_it_entirely`) et `crates/kesh-db/tests/grouped_settlements_supplier.rs:1063`
(`unreconciling_a_supplier_group_undoes_it_entirely`) ; ils n'assertent de l'audit que `kind` et la longueur de la liste
(`grouped_settlements.rs:1169`–`:1170`, `grouped_settlements_supplier.rs:1167`–`:1171`), pas la charge entière ni la
ligne de la transaction bancaire au-delà de `status` et `matched_entry_id`.

### Lot D — #1261 : la cause de la variabilité, au code

La chaîne, lien par lien (zip **2.6.1**, `Cargo.lock:5308`–`:5309` ; source
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zip-2.6.1/src/write.rs`) :

1. **L'écrivain du test compte les réécritures comme de l'espace consommé.**
   `admin_auto_backup_e2e.rs:98`–`:104` :
   ```rust
            Ecrivain::PleinApres(n) => {
                if self.ecrit + buf.len() as u64 > n {
                    return Err(io::Error::from(io::ErrorKind::StorageFull));
                }
                let w = self.fichier.write(buf)?;
                self.ecrit += w as u64;
   ```
   `ecrit` cumule **toutes** les écritures, y compris celles qui réécrivent des octets déjà écrits, et l'écriture qui
   déborde échoue **seule** (une plus petite, ensuite, passerait). Un disque réel ne refuse pas la réécriture d'un
   octet déjà alloué (sauf système de fichiers à copie sur écriture) : le modèle est faux sur ce point.
2. **zip réécrit l'en-tête de chaque entrée** en clôturant l'entrée : `finish_file` (`write.rs:1055`) appelle
   `update_local_file_header` (`:1912`–`:1918`) qui **recule** (`writer.seek(SeekFrom::Start(file.header_start +
   CRC32_OFFSET))?;`) puis écrit trois fois **4 octets** (`writer.write_u32_le(file.crc32)?;` `:1918`, puis `:1935`,
   `:1937`) avant de revenir à la fin (`:1101` — `writer.seek(SeekFrom::Start(file_end))?;`).
3. **Si la limite tombe sur l'une de ces écritures de 4 octets**, l'erreur `StorageFull` sort de `finish_file` par
   `?` **après le recul et avant le retour à la fin**, `writing_to_file` restant vrai (`:1108` —
   `self.writing_to_file = false;` n'est pas atteint). La position du fichier est **dans l'en-tête**. L'erreur remonte
   à `write_keshbackup` (`export.rs:269`, `zip.start_file(...)` → `start_entry` → `finish_file`, `write.rs:904`), qui
   rend `Err` — et `zip`, variable locale, est **détruit**.
4. **Le `Drop` du `ZipWriter` refait la clôture** (`write.rs:1625`–`:1628` : `if !self.inner.is_closed() { if let
   Err(e) = self.finalize() {`) → `finalize` → `finish_file` (`:1504`) : `let file_end = writer.stream_position()?;`
   (`:1077`) rend la position **dans l'en-tête**, inférieure à `self.stats.start` → `:1078` —
   `debug_assert!(file_end >= self.stats.start);` **panique** : c'est la panique « zip-2.6.1 write.rs:1078 » de
   l'issue. Les tests sont compilés avec `debug-assertions` (profil `dev` hérité, `Cargo.toml:37`–`:47`, dépendances
   comprises).
5. **La panique devient `Failed`.** Elle traverse la fermeture du `spawn_blocking` : `export.rs:169`–`:171` —
   `tokio::task::spawn_blocking(move || remplir(export, documents_dir)).await.map_err(|e|
   BackupError::Task(e.to_string()))?` ; `auto_backup.rs:717`–`:723` classe toute erreur de `build_keshbackup` en
   `Ecriture::Refusee(FailureStatus::Failed)`. Le test reçoit `Done(Failure(Failed))`.
6. **Si la limite tombe ailleurs** (en-tête d'une entrée, données compressées, répertoire central), aucune
   réécriture n'est interrompue après un recul ; le `Drop` échoue proprement (`write.rs:1629`, `stderr` seulement),
   l'erreur d'origine `StorageFull` remonte, `classify_write_error` (`auto_backup.rs:258`–`:263`) rend `DiskFull` :
   le test passe.

**D'où vient la variabilité.** La limite est fixe (`PleinApres(2048)`, `admin_auto_backup_e2e.rs:653`) ; ce qui
varie d'une exécution à l'autre, c'est **la suite des longueurs écrites** avant 2048 octets : l'archive compresse une
entrée NDJSON par table, dans l'ordre de `TABLES_TO_TRUNCATE` (`kesh-db/src/backup.rs:46` et suivantes :
`credit_note_lines`, `supplier_invoice_lines`, `payment_batch_items`, `invoice_lines`, …), et ces tables portent des
horodatages à la milliseconde posés au montage (`created_at datetime(3) NOT NULL DEFAULT current_timestamp(3)`,
`crates/kesh-db/test-schema/0001_schema_squash.sql:759`, `:1330`). La taille compressée de chaque entrée bouge donc de quelques
octets d'une exécution à l'autre, et avec elle **l'écriture sur laquelle tombe l'octet 2048** : une réécriture de 4
octets (rouge) ou une autre (vert). Chaque petite entrée compte environ trois réécritures de 4 octets pour quelques
dizaines d'octets écrits : une fraction des positions possibles donne le rouge (non chiffrée ici ; la preuve est la
reproduction déterministe d'AC13, non ce raisonnement). **`-j1` n'y est pour rien** ; la charge de la station peut
jouer **indirectement**, par l'étalement des horodatages du montage (un montage lent les disperse, un montage rapide en
donne d'identiques, ce qui change la compression) — compatible avec deux rouges sous charge et 20/20 vert seul, sans
être établi. Le titre de l'issue le disait à tort ; il a été corrigé le 2026-11-19 avec l'accord de Guy.

**Et la production ?** Elle est compilée en `--release` (`Dockerfile:6` — `RUN cargo build --release -p kesh-api \`) :
`debug_assert!` y est absent ; `file.compressed_size = file_end - self.stats.start;` (`write.rs:1079`) déborde sans
panique, puis `update_local_file_header` refuse (`:1929`–`:1933`, « Large file option has not been set ») et le `Drop`
l'avale (`:1629`) ; l'erreur rendue reste l'erreur d'origine, `StorageFull`, classée `DiskFull`. **La sauvegarde
automatique classe donc correctement un disque plein, y compris à la fermeture de l'archive** (la question de l'issue)
— ce que le code dit, **à la lecture**. AC12 ne l'exerce que pour un disque qui accepte les réécritures (l'écrivain
de D6). **Angle mort assumé** : sur un système de fichiers à copie sur écriture — **Btrfs**, que le manuel
d'administration recommande pour le NAS (`admin-manual.tex:2129`–`:2130`), et où `routes/admin.rs:762` note déjà que
« `ENOSPC` et `EDQUOT` surgissent souvent » —, une réécriture d'en-tête peut échouer ; c'est le cas que l'ancien
écrivain modélisait. En `--release`, la lecture ci-dessus le classe `DiskFull` ; en build de test, ce cas panique
toujours (`debug_assert!` de `write.rs:1078`) et **ne se teste donc pas** : il n'est établi que par la lecture du
code compilé en `--release`. Aucune ligne de production ne change pour #1261.

Fait connexe, sans portée en production : en build de débogage, la panique laisse le `.partial`
(`routes/admin.rs:783`–`:806`, `write_backup_file`, sans garde de destruction) ; le tour suivant le supprime
(`supprimer_les_partiels`, `auto_backup.rs:622`).

**Inventaire par le symptôme** (« un écrivain de test qui modélise le disque plein ») :

```
$ git grep -n "PleinApres\|TronqueApres\|StorageFull" -- crates   (hors classement de production)
crates/kesh-api/src/admin_backup/export.rs:680 / :682          (conversion d'erreur, sans écrivain)
crates/kesh-api/src/pre_upgrade_backup/tests.rs:99 / :104      (« chaque écriture rend StorageFull » : échec dès la première écriture, aucun recul → déterministe)
crates/kesh-api/src/routes/admin.rs:1034 / :1038               (conversion d'erreur, sans écrivain)
crates/kesh-api/tests/admin_auto_backup_e2e.rs:49–:53, :98–:118, :653, :683
```

`TronqueApres(512)` (`:683`) n'échoue jamais (il prétend écrire) : l'archive se ferme, la relecture échoue, `Failed`
attendu — déterministe, hors symptôme. Seul `PleinApres` est en cause.

### Manuels

```
$ git grep -n -i "compte de l.écart\|Régler plusieurs factures\|relus" -- docs/manual/fr/*.tex   (extrait)
docs/manual/fr/user-manual.tex:3555:\subsection{Régler plusieurs factures par une transaction}\label{sec:reglement-groupe}
docs/manual/fr/user-manual.tex:3579:\textbf{Un refus.} … Un refus du compte de l'écart s'affiche sous le sélecteur, ou avec les autres refus quand le sélecteur n'est pas affiché~; si Kesh demande un compte alors que l'écart affiché est nul, le sélecteur s'affiche quand même. …
```

- `user-manual.tex:3579` dit « si Kesh demande un compte alors que l'écart affiché est nul, le sélecteur s'affiche quand
  même ». **Déjà faux à cette tête** : depuis la revue P3 ciblée de la 43-3 (L-A, `GroupSettlementModal.svelte:537`), le
  sélecteur ne s'affiche que si une facture n'a pas pu être relue ; relus tous, la fenêtre dit « Les restes dus ont été
  relus… » (test `GroupSettlementModal.test.ts:761`). Le PDF dit de même (`pdftotext docs/manual/fr/user-manual.pdf - |
  tr '\n' ' ' | tr -s ' '` → « …si Kesh demande un compte alors que l’écart aﬀiché est nul, le sélecteur s’aﬀiche quand
  même. »). La phrase décrit la branche que le Lot A modifie : elle se réécrit dans cette story (AC4).
- Lot B : le manuel ne décrit pas la fermeture pendant l'envoi. `user-manual.tex:3490` dit de la confirmation « déjà
  comptabilisée » : « \emph{Annuler}, la touche Échap ou un clic hors de la confirmation ne font rien » et « une
  relecture de la liste \textbf{referme} une confirmation ouverte ». Après D3, *Annuler* peut lancer la relecture
  différée d'un règlement abouti en arrière-plan. **Verdict : inchangé** — l'annulation elle-même ne fait toujours
  rien ; la relecture est celle du règlement, que la page relit « après chaque geste » (même paragraphe) ; et D3
  **renforce** la phrase « une relecture referme une confirmation ouverte » en la rendant impossible pendant qu'elle est
  ouverte, sans la contredire (la relecture attend sa fermeture).
- Lots C et D : `git grep -n -i "disk.full\|disque plein\|disque saturé" -- docs/manual/fr/*.tex` ne rend que des
  passages de la restauration et des migrations (`admin-manual.tex:2183`, `:2330`, `:2335`, `:2564`) ; aucun
  comportement visible ne change. Rien.
- Les manuels DE, IT, EN n'existent qu'en `README.md` (`ls docs/manual/{de,en,it}/`) : seul le manuel français change.

### Modules (règle de splitting)

**Compte à la crate** (convention retenue par l'orchestrateur à la remédiation de la validation P1) : `kesh-db` (le
repository et ses tests) ; `kesh-api` (**tests seulement** : `admin_auto_backup_e2e.rs`,
`bank_account_closed_inventory.rs`) ; `kesh-i18n` (une clé, quatre locales) ; `frontend` (`features/reconciliation`,
`shared/i18n-keys.test.ts`, la spec E2E) ; `docs` (`manual/fr/user-manual.tex`). **Cinq modules** : la règle (« plus
de cinq ») ne découpe pas. Au grain plus fin du tableau de l'épique (qui sépare `features/…`, `shared/`, `tests/e2e`),
le compte serait de sept ; il ne découpe pas davantage, les quatre lots étant indépendants et revus chacun à part.

## Décisions

- **D1 (#1263) — après une relecture, une demande de compte sans sélecteur ne dit jamais « Choisissez le compte ».**
  Dans la branche `accountRequired` de `handleRefusal`, après `:555` : si le sélecteur n'est pas affiché
  (`!showAccount`), la fenêtre ajoute **une** phrase neuve aux refus et rend la main ; elle ne tombe plus à `:570`.
  Phrase (clé `reconciliation-group-reread-blocked`) : FR « Rien n’a été enregistré : voyez ce qui retient encore
  l’enregistrement. » ; DE « Es wurde nichts gespeichert: Beachten Sie, was das Speichern noch verhindert. » ;
  IT « Non è stato registrato nulla: vedete che cosa impedisce ancora la registrazione. » ; EN « Nothing was saved:
  see what still prevents saving. ». La cause elle-même reste dite par l'avis existant (arrondi, `:918`) ou la phrase
  du pied (`:249`, `:243`), et, s'il y a lieu, par la liste « non relue ».
  *Écartées* : (a) ne rien ajouter — un refus muet, contraire à la règle de la fenêtre (« jamais un refus muet »,
  `:568`–`:569`) ; (b) réutiliser « Les restes dus ont été relus : vérifiez l’écart, puis enregistrez de nouveau. » —
  faux : *Enregistrer* est grisé, et la relecture a pu échouer ; (c) afficher le sélecteur quand même — il ne
  débloquerait rien (écart trop grand, arrondi sans compte).
- **D2 (#1263) — le manuel suit le code, phrase comprise que la story n'a pas cassée.** `user-manual.tex:3579` se
  réécrit pour dire : le sélecteur s'affiche quand Kesh demande un compte et qu'une facture n'a pas pu être relue ;
  relus tous, la fenêtre le dit et invite à vérifier l'écart ; si l'enregistrement reste retenu (écart trop grand,
  arrondi sans compte), elle dit que rien n'a été enregistré. *Écartée* : laisser la phrase (P3 « manuel qui décrit
  autre chose que le code » sur la branche même qu'on touche).
- **D3 (#1262) — la relecture est différée jusqu'à ce qu'aucune modale ne soit ouverte.** La modale ne choisit plus :
  quand l'envoi aboutit alors que sa session a changé (fermée pendant l'envoi), elle dit le succès (toast, inchangé)
  et appelle une prop neuve **`onSettledInBackground`** (au lieu de `if (!open) onSuccess();`). Le parent :
  - « une modale est ouverte » veut dire : l'une des **cinq** — `groupOpen`, `manualOpen`, `splitOpen`, `linkOpen`, ou la
    confirmation « déjà comptabilisée » (`pendingGesture !== null`, `:922`) ;
  - aucune modale ouverte → ce que fait aujourd'hui `onGroupSuccess` (bilan effacé, « 1 opération(s) réussie(s). »,
    relecture) ;
  - une modale ouverte → un drapeau `reloadPending = true` ; la relecture part **à la fermeture** de cette modale par un
    chemin qui ne relit pas lui-même (Annuler ou Échap d'une des quatre modales ; *Annuler* ou Échap de la
    confirmation, tous deux par `cancelGesture`, `:454`, via `AlreadyBookedConfirmDialog.svelte:43` —
    `onOpenChange={(v) => !v && onCancel()}`), et elle fait alors ce que fait le cas immédiat (bilan effacé,
    « 1 opération(s) réussie(s). », relecture). *Continuer quand même* de la confirmation (`confirmGesture`, `:438`)
    ouvre une modale — le drapeau attend sa fermeture — ou lance *Accepter* / *Rejeter*, qui relisent eux-mêmes ;
  - **où la relecture différée se déclenche** : dans le `onOpenChange` des quatre modales (`:874`, `:887`, `:900`,
    `:910`, quand `v` est faux) et dans `cancelGesture`, le parent **ne relit pas sur place** : il appelle une fonction
    `flushPendingReload()` qui programme, par `queueMicrotask`, la vérification « `reloadPending` vrai **et** aucune des
    cinq modales ouverte » — et ne relit qu'alors. Ce report hors de la pile d'appel est nécessaire parce que les
    modales enchaînent deux appels dans un ordre qui varie : succès puis fermeture (`GroupSettlementModal.svelte:660`–`:661`,
    `onSuccess(); onOpenChange(false);`, de même les trois autres modales), mais **fermeture puis `onStale`** dans
    `closeStale` (`GroupSettlementModal.svelte:366`–`:373` : `onOpenChange(false); onStale();`). Dans les deux ordres, le
    gestionnaire qui relit (`onGroupSuccess`, `onGroupStale`, `:322`–`:335`) appelle `load()` **dans la même pile
    synchrone**, et `load()` éteint le drapeau avant son premier `await` (ci-dessous) : quand la microtâche s'exécute,
    le drapeau est éteint et elle ne relit pas. **Une seule relecture** dans tous les cas ;
  - **le drapeau s'éteint dans `load()`** (`reloadPending = false;`, écrit sans lecture préalable, à côté de
    `pendingGesture = null;` `:182`, pour ne pas devenir une dépendance de l'`$effect` `:202`–`:204`) : toute relecture,
    d'où qu'elle vienne (`onGroupSuccess`, `onGroupStale`, `onManualSuccess`, *Accepter*, *Rejeter*, `$effect` de
    `limit`), le consomme ; aucune relecture double ; le bilan affiché est alors celui du chemin qui a relu. Le
    drapeau doit s'éteindre **avant le premier `await` de `load()`** (sinon la microtâche le verrait encore armé) ;
  - **un échec d'*Accepter* / *Rejeter*** (lancés depuis la confirmation, `:518`–`:520`, `:537`–`:540` : `await load()`
    dans le `try`, le `catch` ne relit pas) laisse le drapeau armé : il se consomme à la prochaine relecture ou à la
    prochaine fermeture d'une modale. La relecture différée n'est donc pas perdue, mais elle peut attendre le geste
    suivant ; cas rare, sans effet sur les livres, assumé.
  *Écartées* : (a) appeler `onSuccess` sans condition — ferme la fenêtre rouverte (`:325`–`:326`) ; (b) relire
  aussitôt — `load()` démonte la modale ouverte (`:555`, `:184`), referme la confirmation (`:182`) et perd la saisie ; (c) retirer la ligne localement de
  `proposals` sans relire — la liste vidée bascule sur `:561` et démonte aussi la modale ; (e) relire
  synchroniquement dans `onOpenChange` — relit deux fois quand `closeStale` enchaîne `onOpenChange(false)` puis
  `onStale()` ; (f) inverser l'ordre de `closeStale` — touche la modale pour un défaut du parent, et laisse le parent
  dépendant d'un ordre d'appel ; (d) une relecture silencieuse
  sans `loading` — change la sémantique de `load()` pour tous ses appelants (35-4, 29-9).
- **D4 (#1255) — trois aides privées, l'ordre de chaque bras intact.** Dans `grouped_settlements.rs` :
  `lock_entry_fiscal_year_in_tx(tx, company_id, entry_id) -> Result<i64, DbError>` (le verrou de l'écriture et de son
  exercice) ; `unlink_group_transaction_in_tx(tx, company_id, bt) -> Result<(), DbError>` (l'`UPDATE` et
  `OptimisticLockConflict`) ; `audit_group_cancelled_in_tx(tx, acteur, bt, entry_id, reversal_id, liste)` où `liste`
  porte la clé (`"invoices"` ou `"supplier_invoices"`) et ses éléments, propres à chaque famille. **Restent dans chaque bras** :
  `ensure_open_in_tx` et `recheck_open_locked_in_tx` (le troisième inventaire du compte clos les lit par fonction), le
  verrou des factures, la suppression ou la réouverture, la contre-passation, l'audit par facture, et **l'ordre** des
  appels. Les gardes statiques suivent dans le même commit : au `REGISTRE`, l'entrée `(grouped_settlements.rs,
  "cancel_in_tx")` perd son seul littéral visé (l'`UPDATE bank_transactions`) et sort ; une entrée neuve
  `(grouped_settlements.rs, "unlink_group_transaction_in_tx")` entre en `Statut::Gardee("appelé par cancel_in_tx et
  supplier_cancel_in_tx seuls, après leur bank_accounts::ensure_open_in_tx ; revérifié en dernier par
  recheck_open_locked_in_tx")` ; `supplier_cancel_in_tx` garde la sienne (l'`UPDATE supplier_invoices`). **Le
  troisième inventaire suit les aides**, selon son mécanisme exact (`bank_account_closed_inventory.rs`, relu) :
  `fautes_en_dernier` exige que la revérification suive **toute** entrée de `GESTES_VERROUILLANTS_CODE`, sans
  exception (`:1362`–`:1365` : `for g in GESTES_VERROUILLANTS_CODE { for (p, _) in corps.match_indices(g) { garder(a.corps.0
  + p, g); } }`, puis `:1371` : `Some((p, g)) if a.position < p =>` faute), et précède le premier
  `audit_log::insert_in_tx(` (`:1381` : `if let Some(q) = corps.find("audit_log::insert_in_tx(")`). Donc :
  - **seules** `lock_entry_fiscal_year_in_tx(` et `unlink_group_transaction_in_tx(` entrent dans
    `GESTES_VERROUILLANTS_CODE` ; le test qui verrouille la liste jeton par jeton,
    `the_last_acquisition_rule_sees_each_locking_token` (`:1781`), reçoit leurs deux cas dans sa table
    `let appels: [&str; 5] = [` (`:1800`, qui passe à 7) ; son décompte `assert_eq!(joues, 10, "décompte anti-muet des
    cas joués");` (`:1837`) passe à **12**, et son doc-comment « chacune des dix entrées » (`:1772`) à « douze » ; son
    contrôle de couverture (`:1854`, « un appel verrouillant sans cas construit (ou un cas sans appel) ») rougit
    sinon ;
  - `audit_group_cancelled_in_tx(` **n'entre pas** dans `GESTES_VERROUILLANTS_CODE` (la revérification devrait à la fois
    la suivre et la précéder : la garde rougirait sur tout code) ; il entre **seulement** dans la borne « avant
    l'audit » : `:1381` devient le minimum des positions de `audit_log::insert_in_tx(` et de
    `audit_group_cancelled_in_tx(`, et la faute nomme le jeton trouvé ; un **cas construit** « suit l'aide d'audit »
    rejoint la table `cas` de `the_last_acquisition_rule_reports_each_fault_on_built_cases` (`:1666`, tableau
    `[…; 7]` qui passe à 8), à côté du cas « suit l'audit ». `FENETRE`,
  `ANNULATIONS`, `letterings_lexical.rs` et `audit_acteur_garde.rs` (aucun site de `grouped_settlements.rs`, motif
  `NewAuditLogEntry::par` hors de ses cibles) ne bougent pas. L'ordre des verrous se prouve par cette garde et par la
  comparaison bloc par bloc du diff, non par AC9. *Écartées* : (a) une seule fonction paramétrée par la
  famille — l'ordre des verrous diffère (factures avant écriture, suppression avant ou après la contre-passation) et
  se lirait mal ; (b) déplacer `ensure_open_in_tx` dans une aide — ferait bouger le troisième inventaire, dont la
  position « en dernier » est vérifiée par fonction.
- **D5 (#1255) — l'unitaire `reconciliation_cancel::cancel_in_tx` reste tel quel.** Son audit porte `kind`,
  `unlinkOnly`, `invoiceId`, `settlementId` variables, sans liste ; son `UPDATE` est identique. L'y brancher touche un
  autre repository et un chemin de production plus emprunté, hors de l'issue : **issue `technical-debt` à ouvrir**.
- **D6 (#1261) — l'écrivain du test modélise un disque plein comme le système.** `PleinApres(n)` devient : une écriture
  qui **prolongerait** le fichier au-delà de `n` octets écrit ce qui tient jusqu'à `n` (`Ok(partiel)`), puis toute
  écriture qui prolonge rend `StorageFull` ; une écriture **à l'intérieur** de ce qui est déjà écrit (une réécriture
  après recul) réussit. La limite se juge sur la **position** (`stream_position` + longueur), non sur un cumul. À la
  position exacte `n`, une écriture qui prolonge rend `Err(StorageFull)`, **jamais `Ok(0)`** (que `write_all`
  convertirait en `WriteZero`, classé `Failed` et non `DiskFull`).
  *Écartées* : (a) changer la limite (2048 → autre) — déplace le hasard, ne le retire pas ; (b) rattraper la panique
  en production (`catch_unwind`, `JoinError::is_panic` → `DiskFull`) — la production ne panique pas (`--release`) et
  une panique n'est pas un disque plein ; (c) `mem::forget` du `ZipWriter` en cas d'erreur — fuite de descripteur pour
  un défaut du seul test.
- **D7 (#1261) — le test balaie des limites, il ne parie plus sur une seule.** `disque_sature_pendant_l_ecriture` joue
  le tour pour **au moins 128 limites consécutives** à partir de 2048 et exige, à chacune, `DiskFull`, ni fichier neuf
  ni `.partial`, `last_status = "disk_full"`. Un second test, `disque_sature_a_la_fermeture`, fait de même pour au moins
  64 limites prises dans le répertoire central de l'archive (fenêtre `[S − 160, S − 32)` ; le répertoire central d'une
  trentaine d'entrées dépasse largement 160 octets, la dérive de `S` d'un tour à l'autre est de quelques octets). `S`
  se mesure **avant** de déposer les huit sauvegardes : un tour réussi avec une rétention haute (`retention(60)`),
  `S` lu sur le fichier écrit, ce fichier retiré, puis `deposer(8)` et `retention(3)` comme aujourd'hui — sans quoi la
  rétention du tour de mesure supprimerait des déposées et fausserait « les 8 déposées restent 8 »
  (`write_keshbackup` n'est pas joignable d'un test d'intégration : `admin_backup` est `pub(crate)`, `lib.rs:14`). *Écartée* : rejouer N fois la même limite —
  la variabilité ne vient pas du hasard du tour mais de la suite des longueurs.
- **D8 — aucune migration, aucune écriture passée touchée.** Le Lot C réécrit le chemin du dé-rapprochement à
  comportement égal ; aucun autre lot n'écrit en base.

## Critères d'acceptation

**Lot A — #1263**

1. Après un refus `difference_account_required`, relecture réussie, écart nul et arrondi sans compte d'arrondi
   désigné : la liste des refus contient « Rien n’a été enregistré : voyez ce qui retient encore l’enregistrement. »,
   **ne contient pas** « Choisissez le compte de l’écart. », aucun sélecteur n'est affiché, l'avis
   `group-rounding-not-configured` est visible, *Enregistrer* est grisé.
2. Même refus, relecture réussie, écart au-delà du plafond : même phrase neuve, pas « Choisissez… », pas de sélecteur,
   la phrase du pied « L’écart dépasse le plafond admis… » visible.
3. Même refus, une facture relue (son reste a grandi, écart au-delà du plafond) et une autre non relue : même phrase
   neuve, pas « Choisissez… », pas de sélecteur, la liste « non relue » nomme la facture non relue.
4. `user-manual.tex` (`\label{sec:reglement-groupe}`, paragraphe *Un refus.*) dit le comportement de D2 ; plus aucune
   phrase n'y affirme que « le sélecteur s'affiche quand même » à écart nul sans condition. Le `.tex` seul change (pas
   de PDF commité).
5. La clé `reconciliation-group-reread-blocked` existe dans les quatre catalogues (`fr-CH`, `de-CH`, `it-CH`,
   `en-CH`) avec les textes de D1 ; `i18n-keys.test.ts` (`sitesTotal`) est recompté depuis la source.

**Lot B — #1262**

6. La fenêtre fermée pendant l'envoi puis **rouverte sur une autre transaction** : quand l'envoi aboutit, le toast dit
   le succès, `onSettledInBackground` est appelé une fois, `onSuccess` ne l'est pas, et la fenêtre rouverte garde son
   état (une facture qu'on y a imputée y reste, la recherche tapée aussi).
7. Au parent, un règlement abouti en arrière-plan alors qu'une modale est ouverte **ne relit pas** la liste
   (`getProposals` non appelé) et ne démonte pas la modale ; **à sa fermeture** (Annuler ou Échap), la liste est relue
   une fois, et la transaction réglée n'y figure plus. Sans modale ouverte, la relecture est immédiate et
   « 1 opération(s) réussie(s). » s'affiche (comportement d'aujourd'hui, `onGroupSuccess`) ; différée puis lancée par
   Annuler ou Échap, la relecture affiche le même bilan. Un chemin de fermeture qui relit déjà (succès, refus de la
   transaction) ne relit pas une seconde fois. **La confirmation « déjà comptabilisée » ouverte compte comme une
   modale** : un règlement abouti en arrière-plan ne la referme pas et ne vide pas la sélection ; *Annuler* la ferme et
   lance la relecture une fois. L'invariant « jamais de relecture sous une modale ouverte » se constate pour **chacune
   des cinq** (règlement groupé, écriture manuelle, éclatement, rattachement, confirmation). Drapeau armé, la fenêtre
   rouverte refusée sur sa transaction (`closeStale` : `onOpenChange(false)` puis `onStale()`) : `getProposals` est
   appelé **une** fois ; de même dans l'ordre du succès (`onSuccess()` puis `onOpenChange(false)`).
8. Dans le navigateur (E2E) : deux transactions ; règlement groupé lancé sur la première, fenêtre fermée (Échap)
   pendant l'envoi retenu, fenêtre rouverte sur la seconde, envoi relâché ; la fenêtre rouverte reste ouverte et
   intacte ; fermée, la liste ne montre plus la première transaction.

**Lot C — #1255**

9. **Règlements et écritures existants identiques.** Pour un groupe client et un groupe fournisseur dé-rapprochés, l'état
   qui suit le geste est **le même avant et après la réécriture**, comparé champ par champ par un test écrit **avant**
   elle : la ligne `bank_transactions` (`status`, `matched_entry_id`, `auto_match_rejected_at`, `version`), l'écriture
   inverse — son en-tête (numéro, date, exercice, libellé, `reverses_entry_id`) — et ses lignes (comptes, débit,
   crédit, montants), chaque facture (client : `paid_at`,
   `version`, reste dû ; fournisseur : `status`, colonnes de règlement, `version`), les lettrages, et **chaque ligne
   d'audit écrite par le geste, entière hors horodatage** — `action`, `entity_type`, `entity_id`, `actor_user_id`,
   `actor_api_key_id`, `details_json` (`serde_json::Value`, égalité complète, éléments de la liste compris) —, **dans
   l'ordre des identifiants** : **toutes** les lignes d'identifiant supérieur au dernier avant le geste, sans filtre
   d'action — le délettrage (`lettering.removed`, `letterings.rs:1428`), la contre-passation (`journal_entry.created`
   de l'inverse, `journal_entry.reversed`, `journal_entries.rs:3341`, et un `lettering.created` par ligne libre d'un
   compte lettrable s'il y en a), les audits par facture, puis `reconciliation.cancelled` ; la suite exacte se relève
   sur l'ancien code et s'écrit en dur. Le geste
   est joué **avec une clé d'API**, pour que `actor_api_key_id` soit non nul. Avant le dé-rapprochement, le test pose
   par un `UPDATE` direct un `auto_match_rejected_at` **non nul** sur la transaction réconciliée d'un groupe (l'autre
   garde `NULL`) : après le geste, la colonne vaut `NULL`, et `wasPreviouslyRejected` vaut `true` pour l'un, `false`
   pour l'autre. **Les valeurs attendues sont écrites en dur dans le test** (identifiants pris dans ce que rend le
   règlement), puisque « avant » et « après » sont deux versions du code, non deux exécutions comparées. Un **second
   groupe**, non touché, garde une empreinte identique (écriture, lignes, règlements, transaction, factures) avant et
   après le dé-rapprochement du premier.
10. `cancel_in_tx` et `supplier_cancel_in_tx` ne contiennent plus ni le `SELECT … FOR UPDATE` de l'écriture et de son
    exercice, ni l'`UPDATE bank_transactions`, ni l'audit `reconciliation.cancelled` : chacun est écrit une fois
    (`grep -c` cité au Dev Agent Record : 1 occurrence de `SET matched_entry_id = NULL` dans `grouped_settlements.rs`,
    1 de `"reconciliation.cancelled"`). `bank_account_closed_inventory.rs` et `letterings_lexical.rs` sont verts, avec
    le registre, `GESTES_VERROUILLANTS_CODE` (deux aides) et la borne « avant l'audit » (l'aide d'audit) ajustés selon
    D4 ; `the_last_acquisition_rule_sees_each_locking_token` joue **12** cas, dont un par aide de verrou ou de lien
    placée après `recheck_open_locked_in_tx` ; `the_last_acquisition_rule_reports_each_fault_on_built_cases` joue le
    cas « suit l'aide d'audit ».

**Lot D — #1261**

11. L'écrivain de test refuse une écriture qui prolonge le fichier au-delà de la limite et accepte une réécriture à
    l'intérieur : un test sans base le prouve (écrire `n` octets, reculer à 14, écrire 4 octets → `Ok` ; écrire 1 octet
    à la position `n` → `Err` de genre `StorageFull`, jamais `Ok(0)` ; écrire 10 octets à `n − 4` → `Ok(4)`, puis
    `StorageFull`).
12. `disque_sature_pendant_l_ecriture` rend `DiskFull` (avec ni fichier neuf, ni `.partial`, et `last_status =
    "disk_full"`) pour chacune d'au moins 128 limites consécutives à partir de 2048 ; `disque_sature_a_la_fermeture`
    de même pour au moins 64 limites de la fenêtre du répertoire central (D7).
13. Le Dev Agent Record établit la cause par une **reproduction déterministe** de la panique avec l'ancien écrivain :
    une sonde relève la suite des écritures d'un tour — position, longueur, réécriture ou non, **et cumul des octets
    écrits avant chacune** (l'unité de l'ancien écrivain, `admin_auto_backup_e2e.rs:99`, qui juge sur un cumul, non
    sur une position) ; une réécriture de 4 octets est choisie, de cumul `c` ; l'ancien écrivain, à une limite
    `n ∈ [c, c + 3]`, rend `Failed` avec la panique `write.rs:1078` dans le journal
    (`/home/gcorbaz/devel/kesh-gate-logs/e4814/kf063-repro.log`) ; puis l'écrivain de D6, à la **position** de cette
    réécriture, rend `DiskFull`. La sonde vaut parce que, dans un même test, l'archive est identique d'un tour à
    l'autre : `_kesh_backup_schedule` n'est pas exportée (`kesh-db/src/backup.rs:34`–`:36`) et la sauvegarde n'écrit
    aucun audit (`admin_backup/export.rs:3`) ; la sonde le vérifie (deux tours, même suite) avant de s'en servir — ce
    qui fonde aussi la mesure de `S` de D7. Le code de production de la sauvegarde ne change pas (`git diff --stat` cité : aucun fichier de
    `crates/kesh-api/src/`).

**Communs**

14. Aucune migration ; aucun fichier de `crates/kesh-db/migrations/` ni de `test-schema/` touché.
15. Gates selon le `CLAUDE.md` : gate complet au dernier commit de code (et, `kesh-db` étant touché, à chaque passe de
    la boucle), `gate-ci` vert, E2E complet vert (seulement les échecs attendus), recette visuelle des états des Lots
    A et B.

## Tests à écrire, et la preuve du rouge

Ordre : **le test d'abord, rouge**, puis le correctif, vert. Mutations par l'outil **Edit** (jamais `git stash`) ; après
restauration, `touch` du fichier. Journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e4814/`.

| Test | Où | Rougit sans… | Preuve |
|---|---|---|---|
| T1 (AC1) arrondi sans compte, relus | `GroupSettlementModal.test.ts`, bloc « refus (D5, AC4) » | D1 | rouge naturel avant le correctif (`group-refusal` = « Choisissez le compte de l’écart. ») : `rouge-t1-t3.log` ; puis **M1** : la branche neuve retirée → rouge, `mut-m1.log` |
| T2 (AC2) écart trop grand, relus | idem | D1 | dans `rouge-t1-t3.log` ; M1 |
| T3 (AC3) une relue, une non relue, trop grand | idem | D1 | dans `rouge-t1-t3.log` ; M1 |
| T4 (AC6) rouverte, puis succès | `GroupSettlementModal.test.ts`, bloc « gardes de concurrence » | D3 (modale) | rouge naturel (`onSettledInBackground` jamais appelé) : `rouge-t4.log` ; **M2** : `if (!open) onSuccess();` remis → rouge, `mut-m2.log`. Le test existant `:1024` (« fermée pendant l’envoi, le règlement passé : le parent relit sa liste ») attend désormais `onSettledInBackground` : sa mise à jour se déclare au Change Log |
| T5 (AC7) parent : différé puis relu à la fermeture (bilan compris) ; immédiat sans modale ; pas de double relecture ; une fois par modale ouverte parmi les cinq, confirmation « déjà comptabilisée » comprise (ouverte : pas de relecture, sélection gardée ; *Annuler* : une relecture) | `ReconciliationProposals.test.ts`, avec les doublures décrites sous ce tableau | D3 (parent) | rouge naturel : `rouge-t5.log` ; **M3** : relecture immédiate au lieu du drapeau → rouge (`getProposals` appelé au clic « arrière-plan » alors que la doublure manuelle est ouverte) `mut-m3.log` ; **M4** : drapeau jamais consommé à la fermeture → rouge (aucun appel de `getProposals` après « fermer » de chacune des quatre doublures) `mut-m4.log` ; **M9** : `pendingGesture !== null` retiré de la condition « modale ouverte » → rouge (la confirmation, réelle, se referme au clic « arrière-plan ») `mut-m9.log` ; **M11** : relecture synchrone dans `onOpenChange` au lieu de `queueMicrotask` → rouge (deux appels de `getProposals` au bouton « fermer puis `onStale` ») `mut-m11.log` |
| T6 (AC8) E2E deux transactions | `frontend/tests/e2e/reconciliation-group.spec.ts` (`page.route` sur `**/api/v1/reconciliation/accept`, retenue puis `route.continue()`, comme `:727`) | D3 | rouge naturel (la première transaction reste listée après la fermeture) : `e2e-rouge-t6.log` ; M2 rejouée en E2E : `e2e-m2.log` |
| T7 (AC1–AC2) E2E refus après relecture | même spec (`page.route` sur `accept` et sur `group-candidates` pour la relecture, comme `:722`–`:745`) | D1 | rouge naturel : `e2e-rouge-t7.log` ; M1 rejouée : `e2e-m1.log` |
| T8 (AC9) caractérisation client | `crates/kesh-db/tests/grouped_settlements.rs` | — (test de non-régression) | **écrit et vert sur l'ancien code**, commité avant la réécriture : `carac-avant.log` ; vert après : `carac-apres.log` ; **M5** : `auto_match_rejected_at = NULL` retiré de l'aide → rouge `mut-m5.log` (non vide : le test a posé une valeur non nulle avant le geste, AC9) ; **M6** : clé de liste de l'audit changée (`"invoices"` → `"members"`) → rouge `mut-m6.log` ; **M8** : `entity_id` de l'audit de l'aide pris sur `entry_id` au lieu de `bt.id` → rouge `mut-m8.log` |
| T9 (AC9) caractérisation fournisseur | `crates/kesh-db/tests/grouped_settlements_supplier.rs` | idem | dans `carac-avant.log` / `carac-apres.log` ; M5, M6 (clé `"supplier_invoices"`), M8 |
| T10 (AC9) second groupe intact | les deux fichiers | idem | dans `carac-*.log` ; **M7** : l'`UPDATE` de l'aide sans `AND id = ?` (toutes les transactions réconciliées de la société) → rouge `mut-m7.log` |
| T11 (AC10) gardes statiques | `bank_account_closed_inventory.rs`, `letterings_lexical.rs` (existants) | D4 | **rouges à la réécriture tant que le registre n'est pas ajusté** : `inventaire-rouge.log` (attendu, preuve qu'ils voient l'extraction), puis verts |
| T11 bis (AC10) cas construits des aides | `bank_account_closed_inventory.rs` : deux cas dans `appels` de `the_last_acquisition_rule_sees_each_locking_token` (`:1800`, `joues` à 12) ; un cas « suit l'aide d'audit » dans `cas` de `the_last_acquisition_rule_reports_each_fault_on_built_cases` (`:1666`) | D4 (listes étendues) | rouges tant que les aides de verrou et de lien manquent à `GESTES_VERROUILLANTS_CODE` (« aucun geste verrouillant » au lieu de « précède ») et que l'aide d'audit manque à la borne de `:1381` : `rouge-t11bis.log` |
| T12 (AC11) écrivain, sans base | `admin_auto_backup_e2e.rs` (`#[test]`) | D6 | rouge avec l'ancien `PleinApres` (la réécriture refusée) : `rouge-t12.log` ; l'assertion « jamais `Ok(0)` à la position `n` » rougit sur un écrivain qui rend `Ok(0)` (**M10**, `mut-m10.log`) |
| T13 (AC12) balayage pendant l'écriture | `admin_auto_backup_e2e.rs::disque_sature_pendant_l_ecriture` | D6 | avec l'ancien écrivain, au moins une limite de la fenêtre rend `Failed` **si** une réécriture y tombe — non garanti (une grosse entrée peut couvrir la fenêtre) : **filet déclaré**, journal `t13-ancien-ecrivain.log` qui dit ce qui s'est produit ; la preuve déterministe est T12 et AC13 |
| T14 (AC12) balayage à la fermeture | `admin_auto_backup_e2e.rs::disque_sature_a_la_fermeture` | — | filet déclaré : il ne prouve la fermeture **qu'avec l'écrivain de D6**, dont la limite est une position d'archive. Avec l'ancien écrivain, qui juge sur un cumul comptant les réécritures (une douzaine d'octets par entrée), ces limites tombent quelques centaines d'octets **avant** le répertoire central, dans les dernières entrées : son comportement n'y est **pas prédit** (journal `t14-ancien-ecrivain.log`, sans conclusion). Les preuves du rouge restent T12 et AC13 |

**Les doublures de T5** (`ReconciliationProposals.test.ts:52`–`:64` remplace les quatre modales ; aujourd'hui
`ModalSuccessStub.test.svelte` n'accepte que `open`, `onSuccess`, `bankLedgerAccountId`, `modal`, ne rend son bouton que
sous `{#if open}`, et ses enveloppes `ModalSuccessStub{Manual,Split,Link,Group}.test.svelte` ne passent pas
`onOpenChange`) :

- `ModalSuccessStub.test.svelte` reçoit un `onOpenChange` facultatif et, sous `{#if open}`, un bouton « fermer »
  (`data-testid="stub-modal-close"`, `data-modal`) qui appelle `onOpenChange(false)` — l'Annuler/Échap d'une vraie
  modale ; les quatre enveloppes le transmettent (le parent le passe déjà, `ReconciliationProposals.svelte:874`, `:887`,
  `:900`, `:910`) ;
- `ModalSuccessStubGroup.test.svelte` reçoit en plus `onSettledInBackground` et `onStale` ; son bouton « arrière-plan »
  (`stub-group-background`) est rendu **hors** de `{#if open}` — le règlement en arrière-plan part d'une fenêtre du
  groupe fermée, pendant qu'une autre modale est ouverte ; sous `{#if open}`, deux boutons rejouent l'ordre réel des
  modales : `stub-group-success-then-close` (`onSuccess(); onOpenChange(false);`) et `stub-group-close-then-stale`
  (`onOpenChange(false); onStale();`) ;
- le test joue une ligne par modale ouverte : les quatre doublures (ouverte, « arrière-plan » → `getProposals` non
  appelé et la doublure toujours là ; « fermer » → un appel) et la confirmation, réelle (ouverte, « arrière-plan » →
  toujours ouverte, sélection gardée ; *Annuler* → un appel) ; puis les deux ordres de fermeture avec relecture, drapeau
  armé → un seul appel chacun. Les attentes portent sur le **nombre d'appels** de `getProposals` (la doublure remonte
  après une relecture, `manualOpen` restant vrai : sa présence seule ne prouve rien).

Numérotation : **M1 à M11, uniques, un journal chacune** ; le rejeu d'une mutation en E2E a son journal `e2e-m<N>.log`
sans numéro neuf.

## Tâches

**Étape 0** (toutes) : relire à la tête de la branche chaque ligne citée ici ; en copier trois au Change Log ; lire
`bank_account_closed_inventory.rs` en entier (registres, extracteur, tests synthétiques) avant le Lot C ; lire
`docs/testing.md` (§ *Prérequis Playwright local*, § *Les échecs attendus*) avant l'E2E ; vérifier à l'écran que la
fenêtre du règlement groupé se ferme par Échap pendant l'envoi (prémisse de T6 ; si elle ne se ferme pas, #1262 n'est
pas atteignable dans le navigateur : le dire au rapport, garder T4–T5).

**Lot D — #1261** (tests seuls, `kesh-api`) :
1. Sonde et reproduction déterministe (AC13) : journal `kf063-repro.log`. La sonde est un outil de diagnostic, non
   commité (ou commité derrière `#[ignore]` avec sa raison, au choix du développeur, dit au Change Log).
2. T12 rouge ; écrivain de D6 ; T12 vert ; M10.
3. T13, T14 ; balayages verts ; durée des deux tests relevée au Dev Agent Record (la CI les joue en `-j1`).

**Lot C — #1255** (`kesh-db`, puis `kesh-api/tests`) :
4. T8, T9, T10 écrits et **verts sur l'ancien code** ; commit à part (`carac-avant.log`).
5. Aides de D4 ; les deux bras les appellent, l'ordre de chaque bras inchangé (comparer le diff bloc par bloc).
6. Gardes statiques : `inventaire-rouge.log`, registre ajusté (D4), verts.
7. T8–T10 verts (`carac-apres.log`) ; M5, M6, M7, M8 ; T11 bis.

**Lot A — #1263** (frontend, `kesh-i18n`, manuel) :
8. T1–T3 rouges ; D1 ; clé ×4 ; `sitesTotal` recompté ; T1–T3 verts ; M1.
9. `user-manual.tex` (D2) ; `make` local du PDF pour vérifier qu'il compile (non commité) ; relire la phrase dans le PDF
   aplati.

**Lot B — #1262** (frontend) :
10. T4, T5 rouges ; D3 (modale, parent, doublures de T5) ; verts ; M2, M3, M4, M9, M11 ; test `:1024` mis à jour.

**E2E et recette** :
11. T6, T7 ; suite E2E complète sur `kesh_e2e_e4814` ; captures (Lot A : refus après relecture — arrondi (AC1),
    plafond (AC2), et une facture relue avec une non relue, où la phrase neuve et la liste « non relue » cohabitent
    (AC3) ; Lot B : la fenêtre rouverte intacte après le toast, la liste **encore périmée** derrière elle (relecture
    différée), puis la liste relue après la fermeture ; la confirmation « déjà comptabilisée » restée ouverte après le
    toast, sélection gardée) dans `kesh-gate-logs/e4814/captures/`,
    jointes au Dev Agent Record et au prompt de la revue de code.

**Gates** : gate ciblé entre deux passes pour les lots frontend et Lot D ; **gate complet à chaque passe dès que le
Lot C est commencé** (exception `kesh-db`) ; `gate-ci` et E2E complet au dernier commit de code. Base remise à zéro
avant tout gate complet (`CLAUDE.md`, § *Base de gate* ; en travail parallèle, sur les bases `kesh_*_e4814`, sans
redémarrer le conteneur).

## Note de version

- Règlement de plusieurs factures : quand Kesh relit les restes dus après un refus et que l'enregistrement reste
  retenu (écart trop grand, arrondi sans compte d'arrondi désigné), la fenêtre dit que rien n'a été enregistré au lieu
  de demander un compte qu'elle n'affiche pas (refs #1263).
- Règlement de plusieurs factures : une transaction réglée pendant que la fenêtre était fermée puis rouverte sur une
  autre ligne disparaît de la liste du rapprochement dès que la fenêtre se ferme (refs #1262).
- Interne : le dé-rapprochement d'un règlement groupé, client ou fournisseur, ne s'écrit plus qu'une fois, à
  comportement égal (refs #1255) ; le test de la sauvegarde automatique sur disque plein n'est plus variable
  (refs #1261).

## Dev Notes

**Hors périmètre** (plafond de quatre issues). Les trois issues ci-dessous sont **ouvertes par l'orchestrateur avant la
clôture de la story** (règle « on trace toujours »), après recherche de doublon : #1235 (« LinkSettlementModal : ni
anti-rebond ni abandon… ») ne couvre pas la relecture de liste ; #1007 porte l'échéancier, non le rapprochement.

- `LinkSettlementModal.svelte:176` — même défaut que #1262 dans *Rattacher à une écriture existante* ; D3 en donne le
  patron (le parent est prêt : `onSettledInBackground` y sera un second appelant).
- `ManualMatchModal.svelte:146`–`:147`, `TransactionSplitModal.svelte:338`–`:339` — la fin d'un envoi agit sur la
  modale rouverte entre-temps (motif de #1007), si la fenêtre peut se fermer pendant l'envoi (prémisse à jouer).
- `reconciliation_cancel.rs:829`–`:843`, `:948`–`:967` — le même `UPDATE` et un audit voisin dans le dé-rapprochement
  unitaire (D5).
- Les autres issues de la dette du règlement groupé (#1256 à #1260, #1264, #1265, P4) restent ouvertes, hors release.

**Fichiers communs avec les autres stories de la 0.20.0** :

| Fichier | Qui d'autre | Règle |
|---|---|---|
| catalogues `crates/kesh-i18n/locales/*/messages.ftl` | toutes | une clé, dans le bloc `reconciliation-group-*` ; conflit résolu en gardant les deux côtés |
| `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`) | 48-1, 48-2, 48-10, 48-12, 48-13, 48-15, Epic 49 | recompté après fusion |
| `docs/manual/fr/user-manual.tex` | toutes ; v0200-menu-sous-groupes (chemins de menu) | paragraphe *Un refus.* de `sec:reglement-groupe` seulement ; PDF jamais commité |
| `ReconciliationProposals.svelte` | aucune story de la release déclarée au kickoff | **écart au tableau des fichiers du kickoff** (qui ne nomme que `GroupSettlementModal.svelte`) : imposé par D3, même module ; dit au rapport |
| `crates/kesh-api/tests/bank_account_closed_inventory.rs` | aucune déclarée | registre seulement (D4) |

Aucun fichier de Beat (PR #1327 : `kesh-report/src/pdf.rs`, `kesh-qrbill`) n'est touché.

**Ce qui doit rester vrai** :

- l'ordre des verrous de chaque bras du dé-rapprochement (factures, écriture et exercice, `ensure_open_in_tx`,
  lectures ordinaires, refus, écritures, `recheck_open_locked_in_tx` en dernier) ;
- la charge de chaque entrée d'audit, à l'octet près de la valeur JSON ;
- la fenêtre du règlement groupé ne dit jamais un refus muet (`:568`–`:569`) ;
- une relecture de la liste du rapprochement ne se fait jamais sous une modale ouverte — les cinq, confirmation
  « déjà comptabilisée » comprise ;
- le classement `disk_full` / `failed` de la production, inchangé (`classify_write_error`).

**Références** : issues #1263, #1262, #1261 (et le commentaire de Guy du 2026-11-19), #1255, #1007 ; fiche 43-3 (écran
du règlement groupé, revues P1 à P4) ; fiches 43-1a et 43-1b (dé-rapprochement) ; `docs/testing.md` ; `CLAUDE.md`,
§ *Review Iteration Rule*, § *Base de gate*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Version | Description | Auteur |
|---|---|---|---|
| 2026-11-19 | 0.1 | Création de la fiche (`bmad-create-story`), lue à `59bdd104f` : quatre lots (#1263, #1262, #1255, #1261) ; cause de KF-063 établie au code (réécriture d'en-tête de zip interrompue après recul → `debug_assert` du `Drop`, `write.rs:1078` → panique → `Failed` ; variabilité par les horodatages du montage, non par `-j1`) ; relecture de la liste différée (toute relecture démonte la modale ouverte) ; cinq modules ; trois issues demandées (Lien, Écriture manuelle et Éclater, dé-rapprochement unitaire) | Opus 5.5 |
| 2026-11-19 | 0.2 | Validation P1 (Sonnet 5.5, passe complète) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 7 LOW ; tous appliqués. V1-1 : les trois aides de D4 entrent dans `GESTES_VERROUILLANTS_CODE`, l'aide d'audit dans la borne « avant l'audit » de `fautes_en_dernier`, sonde synthétique T11 bis ; V1-2 : la confirmation « déjà comptabilisée » (`pendingGesture`) compte comme modale ouverte (D3, AC7, M9), invariant testé sur les cinq modales ; V1-3 : AC9 compare la ligne d'audit entière hors horodatage, dans l'ordre des identifiants, geste joué avec une clé d'API (M8) ; V1-4 : `auto_match_rejected_at` non nul posé avant le geste, valeurs attendues en dur ; V1-5 : le drapeau s'éteint dans `load()`, bilan de la relecture différée dit ; V1-6 : raison de T14 corrigée (comportement de l'ancien écrivain non prédit) ; V1-7 : `S` mesuré avant les dépôts, jamais `Ok(0)` à la limite (M10) ; V1-8 : `ReconciliationProposals.svelte:555`, `export.rs:169`–`:171` ; V1-9 : éléments des listes d'audit, `audit_acteur_garde.rs` écrit non concernée ; V1-10 : captures AC3 et Lot B avant fermeture ; V1-11 : modules comptés à la crate (cinq), trois issues ouvertes par l'orchestrateur avant la clôture. Mutations M1 à M10. Rapport : `kesh-gate-logs/e4814/validation-p1.md` | Opus 5.5 |
| 2026-11-19 | 0.3 | Validation P2 (Opus 5.5, passe complète) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 6 LOW ; tous appliqués. **Signal de recyclage** (règle de splitting préventif) : deux des trois MEDIUM naissent de la remédiation de la P1 — V2-1 de V1-1 (l'aide d'audit mise dans `GESTES_VERROUILLANTS_CODE`, dont la règle `:1362`–`:1371` n'admet pas d'exception, et le test de couverture `:1781`/`:1837` ignoré), V2-3 de V1-2 (doublures de T5 incapables de jouer AC7 sur les cinq modales) ; cause : corrections écrites sans relire le mécanisme exact de la garde et des doublures. Déclaré à Guy par l'orchestrateur ; la P3 sera complète, non ciblée. Remédiations, mécanisme relu avant chaque correctif : V2-1 — seules les aides de verrou et de lien entrent dans `GESTES_VERROUILLANTS_CODE`, avec leurs cas dans `appels` (`:1800`) et `joues` à 12 (`:1837`) ; l'aide d'audit n'entre que dans la borne de `:1381`, avec un cas construit dans `:1666` (T11 bis) ; V2-2 — la relecture différée part par `queueMicrotask` depuis `onOpenChange` et `cancelGesture`, `load()` éteignant le drapeau avant son premier `await` : une seule relecture dans l'ordre de `closeStale` (`onOpenChange(false); onStale()`) comme dans celui du succès ; cas ajouté à AC7 et T5, mutation M11 ; V2-3 — doublures décrites (base avec `onOpenChange` et « fermer » ; groupe avec « arrière-plan » hors de `{#if open}` et les deux ordres réels) ; M3, M4, M9 jugés sur le nombre d'appels de `getProposals` ; V2-4 — « aucune relecture perdue » retiré, l'échec d'*Accepter*/*Rejeter* écrit comme cas assumé ; V2-5 — `user-manual.tex:3490` jugé (inchangé) et capture de la confirmation restée ouverte ; V2-6 — angle mort assumé : la copie sur écriture (Btrfs, recommandé par le manuel d'administration) n'est établie qu'à la lecture du code en `--release`, non par AC12 ; V2-7 — « ni la charge de la station » retiré (effet indirect possible, non établi) ; V2-8 — la sonde d'AC13 relève le cumul, unité de l'ancien écrivain ; identité de l'archive d'un tour à l'autre fondée (`backup.rs:34`–`:36`, `export.rs:3`) ; V2-9 — AC9 prend toutes les lignes d'audit du geste sans filtre (délettrage, contre-passation, factures, transaction) et l'en-tête de l'écriture inverse. Mutations M1 à M11. Rapport : `kesh-gate-logs/e4814/validation-p2.md` | Opus 5.5 |
