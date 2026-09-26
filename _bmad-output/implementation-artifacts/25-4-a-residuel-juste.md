# Story 25.4-a : Le résiduel juste — l'avoir compté TTC, et refusé sur une facture réglée

Status: ready-for-dev

**Issues : [#455], [#456]**, qu'elle **ferme** : `closes #455, closes #456` dans le **titre ET le
corps** de la PR (squash ; un `refs` partout laisserait les issues ouvertes sans signal).

**Mère : `25-4-propager-le-residuel.md`** (`split`) — l'inventaire vérifié et la raison du
découpage y sont. **Sœurs à venir** : 25-4-b (#416, agrégats et relances), 25-4-c (#420,
rapprochement), qui **réemploient la formule** que cette story corrige : ⛔ elle passe en premier.

## Story

En tant que comptable,
je veux que le montant « reste dû » d'une facture soit calculé dans la même unité que ce que le
grand livre a passé, et qu'un avoir ne puisse pas viser une facture déjà réglée en partie,
afin que le reste dû ne mente jamais — ni en gardant la TVA d'une facture créditée, ni en laissant
une créance devenir créditrice sans que rien ne le signale.

## Le défaut, vérifié dans le code

### #455 — l'avoir est soustrait HORS TAXES d'un total TTC

`amount_due` (`crates/kesh-db/src/repositories/invoice_settlements.rs:131-145`) calcule
`TTC − avoir émis − Σ règlements` :

- le TTC vient de `INVOICE_TTC_SUBQUERY_SQL` (`invoices.rs:161-162`) :
  `Σ (line_total + ROUND(line_total × vat_rate / 100, 2))` sur `invoice_lines` ;
- l'avoir vient de `INVOICE_CREDITED_SUBQUERY_SQL` (`invoice_settlements.rs:51-52`) :
  `SUM(cn.total_amount)` — et `credit_notes.total_amount` est **HT**
  (`20260627000001_credit_notes.sql:19` : « `total_amount` stocké = HT (Σ line_total) » ;
  `credit_notes.rs:514-529`, `.bind(total_ht)`).

Or l'écriture de l'avoir crédite la créance du **TTC** (`generate_credit_note_journal_lines`,
`credit_notes.rs:217-223` : `credit: total_ht + total_vat`, la TVA arrondie **ligne par ligne** par
`kesh_core::accounting::vat::line_vat_amount`, `:194` et `:209`). Le grand livre et le résiduel ne parlent
donc pas la même unité : une facture de 100.— HT à 8,1 % entièrement créditée garde **8.10** de
reste dû. La forme jointe `INVOICE_CREDITED_DERIVED_JOIN_SQL` (`:56-58`) porte le même défaut.

⚠️ **Pourquoi aucun test ne l'a vu** : les factures des tests de règlement sont à **0 % de TVA**
(`crates/kesh-db/tests/invoice_settlement.rs:32`), où HT = TTC ; et aucun test n'associe un avoir
et `amount_due`.

⚠️ **Latent sur les données neuves, pas sur les anciennes.** L'avoir est **total** et fait passer
la facture à `cancelled` (`credit_notes.rs:560-575`) ; le terme « avoir » n'est donc non nul que sur
une facture annulée, où le reste dû ne s'affiche que si `amountSettled > 0`
(`invoices/[id]/+page.svelte:1022-1041`) — c'est-à-dire **exactement l'état que #456 permet de
produire**. Et un `.keshbackup` de la v0.12.0 **peut contenir** cet état : l'import le restaure.

### #456 — un avoir peut viser une facture partiellement réglée

`create_credit_note` (`credit_notes.rs:292-308`) refuse une facture `paid_at IS NOT NULL` (garde
AC2bis) — **et rien d'autre**. Depuis la 24-2, `paid_at` n'est posé qu'**au solde** : une facture
réglée en partie a une ligne `invoice_settlements`, un reste dû positif, `paid_at` à `NULL` — elle
passe.

Conséquence comptable : la créance reçoit **D TTC** (vente), **C réglé** (règlement), **C TTC**
(avoir) — elle finit **créditrice du montant encaissé**, sans flux de remboursement. C'est le solde
contre nature que la garde AC2bis voulait empêcher, atteint par un chemin qu'elle ne voit pas. La
facture passe ensuite à `cancelled`, et la ligne de règlement **survit** sur une facture annulée —
où `settlement_cancel_blocker` la déclare inannulable (`SettlementCancelBlocker::InvoiceCredited`,
`invoice_settlements_write.rs:327-333`).

⛔ **Le patron de la bonne garde existe déjà** : la dévalidation lit « **l'existence d'une ligne**
`invoice_settlements` **OU** `paid_at IS NOT NULL` — les deux, jamais l'une seule »
(`UnvalidationBlocker::Settled`, `errors.rs:134-160` ; `invoices.rs:1430-1448`).

⚠️ **Un test encode le défaut comme attendu** : la fixture `monter`
(`crates/kesh-db/tests/invoice_settlement.rs:937-990`) règle 40 sur 100 puis appelle
`create_credit_note` avec `.expect("avoir après règlement partiel — le vrai chemin l'accepte")`.

⚠️ **Le frontend affiche le bouton sur une facture partiellement réglée** :
`invoices/[id]/+page.svelte:836` n'exige que `!invoice.paidAt`. Le bouton voisin « Dévalider » a
retiré cette condition pour cette raison même (commentaire `:865`) ; « Créer un avoir » n'a pas été
aligné.

## Acceptance Criteria

### Volet 1 — l'avoir compté TTC (#455)

**AC 1** — `INVOICE_CREDITED_SUBQUERY_SQL` rend le **TTC** de l'avoir émis :
`Σ (cl.line_total + ROUND(cl.line_total × cl.vat_rate / 100, 2))` sur `credit_note_lines cl` jointe à
`credit_notes cn` (`cn.id = cl.credit_note_id`), filtrée `cn.invoice_id = i.id AND
cn.status = 'issued'`, `COALESCE(…, 0)`. **Même formule d'arrondi, ligne par ligne**, que
`INVOICE_TTC_SUBQUERY_SQL` et que `line_vat_amount` : c'est ce qui la fait concorder avec
l'écriture.

**AC 2** — `INVOICE_CREDITED_DERIVED_JOIN_SQL` rend la même grandeur sous forme de table dérivée
(alias `cnt`, colonne `credited`), agrégée par `invoice_id`.

**AC 3** — Les doc-comments des deux constantes disent **l'unité** (TTC, arrondi ligne par ligne) et
**pourquoi pas `credit_notes.total_amount`** (HT, miroir de `invoices.total_amount`) ; celui
d'`amount_due` dit que ses trois termes sont TTC.

**AC 4** — ⛔ **Aucune migration, aucune colonne.** Le résiduel se **calcule** (24-2, D3) ; stocker
un TTC d'avoir recréerait un chiffre qui peut diverger. `credit_notes.total_amount` reste HT et
n'est pas renommé.

**AC 5** — **DRY** : la formule « TTC d'une ligne » (`line_total + ROUND(line_total * vat_rate / 100,
2)`) n'est pas recopiée une troisième et une quatrième fois à la main. Elle est factorisée en **une
seule source** dont dérivent les quatre constantes (TTC facture scalaire et jointe, avoir scalaire
et joint) — par exemple une macro `macro_rules!` rendant un littéral via `concat!`, les constantes
restant des `&'static str`. Si la factorisation s'avère impossible sans perdre le caractère
`const`, le test de parité de l'AC 6 en tient lieu **et le doc-comment le dit**.

**AC 6** — **Le test de parité promis par `invoice_settlements.rs:27-29`, et qui n'existe pas**,
est écrit : pour un jeu de factures qui couvre **sans règlement / un règlement / deux règlements /
avoir émis / avoir brouillon / deux taux de TVA dont 8,1 %**, la forme scalaire et la forme jointe
rendent **la même valeur, facture par facture**, pour le réglé (`st.settled`) **et** pour l'avoir
(`cnt.credited`). ⚠️ Patron : `crates/kesh-db/tests/invoice_ttc_parity.rs`.

**AC 7** — ⛔ **Concordance avec le grand livre** : pour une facture multi-taux (dont 8,1 %)
créditée, le montant d'avoir que rend la sous-requête est **égal au crédit porté au compte de
créance par l'écriture de l'avoir** (`credit_notes.journal_entry_id` → ligne de crédit du compte
`default_receivable_account_id`). C'est l'assertion qui aurait attrapé #455 ; une comparaison au
seul HT ou à une valeur recalculée en Rust **ne suffit pas**.

**AC 8** — Une facture validée à 8,1 %, **non réglée, créditée** : `amount_due` = **0** (et non la
TVA). Une facture validée à 8,1 %, **réglée de 40 puis créditée** (état **monté en SQL**, cf.
AC 13) : `amount_due` = **−40**, le montant encaissé en trop sur une vente annulée — et non
`TVA − 40`. ⚠️ **Pas d'écrêtage à zéro** : le doc-comment d'`amount_due` l'interdit déjà.

### Volet 2 — l'avoir refusé sur une facture réglée (#456)

**AC 9** — `create_credit_note` refuse une facture qui porte **au moins une ligne
`invoice_settlements` OU** `paid_at IS NOT NULL` — **les deux, jamais l'une seule**, comme
`UnvalidationBlocker::Settled`. La lecture des règlements se fait **après** le verrou de la facture
(`FOR UPDATE`, première instruction de la transaction, `credit_notes.rs:280-289`) et **dans** la
transaction ; elle est **verrouillante** (`… FOR UPDATE` ou `LOCK IN SHARE MODE`). ⛔ Leçon de la
25-3-b : sous REPEATABLE READ, une lecture non verrouillante fige l'instantané — ne pas l'ajouter
**avant** le verrou.

**AC 10** — Le refus est une **variante dédiée** de `DbError`, pas `IllegalStateTransition` — même
motif que `InvoiceNotUnvalidatable` (`errors.rs:119-130`) : le générique ne dit ni ce qui bloque ni
quoi faire. Elle porte l'identifiant du premier règlement trouvé (`Option<i64>`, `None` si seul
`paid_at` est posé) et le numéro de la facture. Code machine **`CREDIT_NOTE_INVOICE_SETTLED`**,
HTTP **409**, ajouté à `DbError::code()` (`errors.rs:640-670`) et à toute garde qui énumère les
codes.

**AC 11** — **Le cas « payée » passe par la même variante** : AC2bis cesse de rendre le générique.
Une seule garde, un seul message, qu'elle soit déclenchée par `paid_at` ou par une ligne de
règlement.

**AC 12** — Message, **quatre locales**, clé **`error-credit-note-blocked-settled`**, mapping dans
`kesh-api/src/errors.rs` sur le patron de `error-invoice-unvalidate-blocked-settled`
(`:2528-2530`) : fr « Cette facture porte un règlement, même partiel : annulez-le d'abord pour
pouvoir émettre un avoir. » Registre du glossaire (`docs/i18n-glossaire.md` § Registre : de =
*Sie*, it = 2ᵉ personne du singulier, en neutre). `details` : `{ "invoiceId", "settlementId" }`.

**AC 13** — La fixture `monter` (`invoice_settlement.rs:937-990`) **cesse de passer par
`create_credit_note`** pour le motif `InvoiceCredited` : elle monte l'état **en SQL** (avoir
`issued` inséré, facture `cancelled`), avec un commentaire qui dit que l'état n'est **plus
atteignable par l'application** depuis 25-4-a, mais **reste possible** par l'import d'une
sauvegarde antérieure — ce qui justifie de garder `SettlementCancelBlocker::InvoiceCredited`.
⛔ Ne **pas** supprimer ce motif ni ses tests.

**AC 14** — Frontend : le bouton « Créer un avoir » (`invoices/[id]/+page.svelte:836`) est masqué
quand `invoice.paidAt` est posé **ou** `Number(invoice.amountSettled) > 0` — même condition que le
serveur. Si la course fait arriver le refus malgré tout, le dialogue affiche le message du serveur
(`err.message`, `:98`), déjà traduit : rien à ajouter côté client.

**AC 15** — Rien n'est **réparé** dans les données existantes : pas de migration, pas de rejeu
(`CLAUDE.md` : aucune donnée de production à protéger ; précédent 24-2 D7). Les états hérités
restent **visibles** — reste dû négatif sur la fiche, motif `InvoiceCredited` à l'annulation du
règlement — et c'est ce que les textes disent (AC 17).

### Volet 3 — tests, textes

**AC 16** — Tests, au minimum (noms indicatifs) :

| Test | Prouve |
|---|---|
| `credited_amount_is_ttc_and_matches_the_ledger` | AC 1, AC 7 — facture multi-taux, avoir émis : sous-requête = crédit créance de l'écriture d'avoir |
| `amount_due_of_a_credited_invoice_is_zero` | AC 8 — 8,1 %, non réglée : `0`, pas `8.10` |
| `amount_due_after_settlement_then_legacy_credit_is_minus_settled` | AC 8 — état hérité monté en SQL : `−40` |
| `settled_and_credited_forms_are_at_parity` | AC 6 |
| `credit_note_refused_on_partially_settled_invoice` | AC 9, AC 10 — **rien écrit** : aucun avoir, facture `validated`, `version` inchangée, solde de créance inchangé, séquence d'avoir non consommée |
| `credit_note_refused_on_paid_invoice` (**existant, réécrit**) | AC 11 — nouvelle variante, plus `IllegalStateTransition` |
| `credit_note_accepted_after_the_settlement_is_cancelled` | la promesse du message : annuler le règlement rouvre l'avoir |
| `credit_note_waits_for_a_concurrent_settlement` | AC 9 — **entrelacement** : une transaction verrouille la facture et insère un règlement ; `create_credit_note` lancé en parallèle attend, puis **refuse** après le commit. Patron : la sonde d'entrelacement de la 25-3-b |
| e2e API `credit_note_on_partially_settled_invoice_is_409` | la frontière HTTP : 409, `CREDIT_NOTE_INVOICE_SETTLED`, message fr |
| Vitest fiche facture | AC 14 — bouton absent si `amountSettled > 0`, présent sinon |

⛔ **Chaque test porte au moins une facture à TVA non nulle.** À 0 %, HT = TTC et #455 est
invisible — c'est ainsi qu'il est passé.

⛔ **Mutations à exécuter et à consigner** (résultat **observé**, pas supposé) : (m1) revenir à
`SUM(cn.total_amount)` dans la forme scalaire ; (m2) idem dans la forme jointe seule ; (m3) retirer
la lecture des règlements de la garde (ne garder que `paid_at`) ; (m4) retirer `paid_at` (ne garder
que la ligne) ; (m5) lire les règlements **avant** le verrou, non verrouillant ; (m6) remettre
`!invoice.paidAt` seul sur le bouton. Chacune doit faire rougir au moins un test.

**AC 17** — Manuels et notes (`docs/manual/fr/user-manual.tex`, PDF régénéré et **contrôlé
aplati**) :
- `:1040` et `:2073` : « un avoir est refusé sur une facture **marquée payée** » devient « … sur une
  facture **réglée, même en partie** » — annuler d'abord le règlement ;
- § *Avoirs* (`\label{sec:avoirs}`, `:1007`) : le refus et son motif sont nommés à côté du refus
  « compte de produit archivé » ;
- `:963` et `:1455` (« la facture a été créditée par un avoir… ») : précisés comme **cas hérité**
  — une facture créditée avant la v0.12.1, ou restaurée d'une sauvegarde antérieure ;
- `CHANGELOG.md` `[0.12.1]` *Fixed* : les deux défauts, en langage d'utilisateur.

⚠️ **Greper le symptôme, pas la phrase** : `grep -rniE "payée|encaissée|réglée" docs/manual/fr/*.tex`
restreint aux lignes qui parlent d'avoir, plus le PDF aplati. Les manuels DE/IT/EN sont des stubs.

## Tasks / Subtasks

- [ ] **T1 — formule de l'avoir TTC** (AC 1-5) : factoriser la formule de ligne, réécrire les deux
      constantes d'avoir, doc-comments.
- [ ] **T2 — parité et concordance** (AC 6-8) : nouveau fichier de test sur le patron
      `invoice_ttc_parity.rs` ; montage de l'état hérité en SQL.
- [ ] **T3 — garde de l'avoir** (AC 9-11) : lecture verrouillante des règlements après le verrou,
      variante `DbError`, `code()`.
- [ ] **T4 — API et i18n** (AC 12) : mapping, clé ×4 ; e2e 409.
- [ ] **T5 — fixture `monter`** (AC 13) : état monté en SQL, commentaire ; les tests d'annulation de
      règlement restent verts.
- [ ] **T6 — frontend** (AC 14) : condition du bouton, Vitest.
- [ ] **T7 — tests et mutations** (AC 16) : m1 à m6 exécutées, résultats consignés.
- [ ] **T8 — textes** (AC 17) : manuel, PDF aplati, CHANGELOG.
- [ ] **T9 — gates** : backend complet (base remise à zéro), frontend complet, E2E complet.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **Ne pas « corriger » `credit_notes.total_amount` en TTC.** Il est HT par construction, miroir
  de `invoices.total_amount`, et d'autres lecteurs le consomment comme tel (export, PDF, liste des
  avoirs). Le défaut est dans **le lecteur**, pas dans la colonne.
- ⛔ **Ne pas calculer le TTC de l'avoir en Rust** puis le soustraire : `amount_due` doit rester
  **une requête**, et la forme jointe est destinée aux agrégats de 25-4-b (pas de N+1).
- ⛔ **Ne pas supprimer `SettlementCancelBlocker::InvoiceCredited`** : l'import d'un `.keshbackup`
  v0.12.0 peut ramener l'état qu'il décrit.
- ⚠️ **Ne pas lire `cn.total_amount` × taux** : un avoir multi-taux n'a pas un taux, et l'arrondi
  doit être **par ligne** pour concorder avec l'écriture (`line_vat_amount`).

### Où regarder

| Fichier | Rôle |
|---|---|
| `crates/kesh-db/src/repositories/invoice_settlements.rs:24-58, 122-145` | les quatre constantes, `amount_due` |
| `crates/kesh-db/src/repositories/invoices.rs:155-190` | `INVOICE_TTC_SUBQUERY_SQL` / `…_DERIVED_JOIN_SQL`, `total_ttc` |
| `crates/kesh-db/src/repositories/credit_notes.rs:187-257, 260-330, 510-575` | écriture de l'avoir, garde AC2bis, `total_amount`, bascule `cancelled` |
| `crates/kesh-db/src/errors.rs:119-175, 300-470, 640-670` | `UnvalidationBlocker`, `DbError`, `code()` |
| `crates/kesh-db/src/repositories/invoices.rs:1430-1448` | la garde de dévalidation, patron de l'AC 9 |
| `crates/kesh-api/src/errors.rs:2515-2560, 2741-2748` | mapping `InvoiceNotUnvalidatable`, générique |
| `crates/kesh-api/src/routes/credit_notes.rs:183-205` | handler de création |
| `crates/kesh-db/tests/invoice_ttc_parity.rs` | patron du test de parité |
| `crates/kesh-db/tests/credit_notes_repository.rs:267-299` | `credit_note_refused_on_paid_invoice` |
| `crates/kesh-db/tests/invoice_settlement.rs:32, 116-125, 937-990` | helper à 0 %, `solde()`, `monter` |
| `frontend/src/routes/(app)/invoices/[id]/+page.svelte:90-104, 836, 865, 1022-1041` | dialogue d'avoir, bouton, précédent « Dévalider », reste dû |

### Courses

Les trois chemins qui écrivent un règlement ou un avoir, et ce qui les sérialise :

| Chemin | Verrou sur la facture |
|---|---|
| `settle_invoice` (`invoice_settlements_write.rs:60-72`) | `FOR UPDATE`, première lecture |
| `create_credit_note` (`credit_notes.rs:280-289`) | `FOR UPDATE`, première lecture |
| `accept_one_invoice` (`kesh-api/src/routes/reconciliation.rs`) | **aucun** — lecture simple, puis `UPDATE invoices … AND version = ? AND status = 'validated'` (`:1490-1500`) |

Le troisième se sérialise par **verrou optimiste** : si l'avoir passe d'abord, la facture est
`cancelled` et l'`UPDATE` du rapprochement n'affecte aucune ligne. Si le rapprochement passe
d'abord, son `UPDATE` tient le verrou de ligne jusqu'au commit ; `create_credit_note` l'attend sur
son `FOR UPDATE`, puis **doit** voir le règlement — d'où l'exigence d'une lecture verrouillante
(AC 9). ⚠️ Vérifier en dev que l'échec de l'`UPDATE` du rapprochement sur 0 ligne rend bien un
`FailedProposal` et annule tout (test existant ou à ajouter).

### Gardes-fous du dépôt

- **Aucune migration** : P1-P8 ne s'arment pas. Si une migration devient nécessaire, arrêter et
  demander.
- **Gate complet** : la story touche des repositories — § *Exception `kesh-db`* : gate complet même
  en cours de boucle de revue.
- **i18n** : une clé ×4 côté serveur ; `sitesTotal` ne bouge que si le frontend ajoute un
  `i18nMsg(` — AC 14 n'en ajoute pas.

### References

- 24-2, D3 (le résiduel se calcule) et D4 (`paid_at` projection du solde) —
  `24-2-encaissement-client.md:120-160`.
- 25-3-b, leçon REPEATABLE READ — mémoire `etat-epic-25`.
- Glossaire et registre — `docs/i18n-glossaire.md`.

## Questions pour Guy

Aucune qui bloque cette story. La question de la QR-facture jointe aux relances (Q1 de la mère)
bloque **25-4-b**, pas celle-ci.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-09-26** — Créée (Opus 5.5) après découpage de la 25-4. Inventaire vérifié par `grep -nF`
  sur chaque site cité. Deux faits hors des issues : #455 est **latent** sur les données neuves et
  ne se voit que dans l'état que #456 produit — d'où leur réunion ; et la fixture `monter` encode
  #456 comme attendu.

[#455]: https://github.com/guycorbaz/kesh/issues/455
[#456]: https://github.com/guycorbaz/kesh/issues/456
