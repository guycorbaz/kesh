# Story 25.4-a : Le résiduel juste — l'avoir compté TTC, et refusé sur une facture réglée

Status: done

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

⚠️ **Masqué à l'écran, exposé par l'API.** L'avoir est **total** et fait passer la facture à
`cancelled` (`credit_notes.rs:560-575`) ; le terme « avoir » n'est donc non nul que sur une facture
annulée. **L'écran** n'y affiche le reste dû que si `amountSettled > 0`
(`invoices/[id]/+page.svelte:1022-1041`) — l'état que #456 permet de produire. ⛔ **Mais l'API le
rend toujours** : `get_invoice` (`crates/kesh-api/src/routes/invoices.rs:621-642`) calcule
`amount_due` **sans condition de statut**, et `GET /invoices/{id}` est **ouverte aux clés API en
lecture** (`docs/api-external.md:211`). Un avoir émis sur une facture **jamais réglée** — geste
ordinaire, sans #456 — suffit donc, aujourd'hui, à faire rendre à une intégration tierce
`amountDue` = **la TVA** au lieu de `0`. *(Relevé en validation P1 ; la fiche mère disait « latent »
à tort.)* Et un `.keshbackup` de la v0.12.0 **peut contenir** l'état de #456 : l'import le
restaure.

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
(`crates/kesh-db/tests/invoice_settlement.rs:937-1019`) règle 40 sur 100 puis appelle
`create_credit_note` avec `.expect("avoir après règlement partiel — le vrai chemin l'accepte")`.

⚠️ **Le frontend masque le bouton sur une facture payée, pas sur une facture partiellement
réglée** : `invoices/[id]/+page.svelte:836` n'exige que `!invoice.paidAt`. Le bouton voisin
« Dévalider » a **retiré** cette condition pour cette raison même (commentaire `:864-871`) et laisse
le serveur refuser avec un motif nommé ; « Créer un avoir » n'a pas été aligné.

## ✅ Arbitrage du 2026-09-27 — le refus reste, sa levée a son issue

Guy : *« si on peut créer un avoir pour une facture payée, il faudrait aussi pouvoir le faire pour
une facture partiellement payée »* — les deux cas sont traités **de la même façon**, et, pour
l'instant, **dans le sens du refus** : un avoir sur facture encaissée laisse au client un crédit que
Kesh ne sait ni montrer, ni rembourser, ni imputer. La levée du refus est tracée par **[#471]**. Le
message de refus (AC 12) n'a donc pas à promettre une fonction à venir ; le manuel (AC 17) renvoie à
la limite existante « avoir sur facture déjà encaissée ».

⚠️ **Ne pas contester l'arbitrage en revue** : en contester la mise en œuvre.

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
TVA) — **au repository ET à la frontière HTTP** : `GET /api/v1/invoices/{id}` rend `amountDue` **présent et non `null`**
(`null` veut dire « non calculé », `routes/invoices.rs:242-248` ; ⛔ aucun `unwrap_or`, aucune
valeur par défaut dans l'assertion), sérialisé en
**chaîne** (sérialisation par défaut de `rust_decimal`, `kesh-api/Cargo.toml:39`), dont la valeur **parsée en
`Decimal`** vaut zéro — ⛔ ni comparaison de chaîne (l'échelle suit le calcul SQL : `"0.0000"` et
`"0.00"` sont également justes), ni `f64` (patron à ne pas suivre :
`invoice_echeancier_e2e.rs:636-641`). Même règle pour `amountSettled`. Une facture validée à 8,1 %, **réglée de 40 puis créditée** (état **monté en SQL**, cf.
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
motif que `InvoiceNotUnvalidatable` (`errors.rs:453`, raison écrite `:119-130`) : le générique ne dit ni ce qui bloque ni
quoi faire. Elle porte l'identifiant du premier règlement trouvé (`Option<i64>`, `None` si seul
`paid_at` est posé) et le numéro de la facture. Code machine **`CREDIT_NOTE_INVOICE_SETTLED`**,
HTTP **409**, ajouté à `DbError::error_code()` (`errors.rs:641`) et à toute garde qui énumère les
codes.

**AC 11** — **Le cas « payée » passe par la même variante** : AC2bis cesse de rendre le générique.
Une seule garde, un seul message, qu'elle soit déclenchée par `paid_at` ou par une ligne de
règlement.

**AC 12** — Message, **quatre locales**, clé **`error-credit-note-blocked-settled`**, mapping dans
`kesh-api/src/errors.rs` sur le patron de `error-invoice-unvalidate-blocked-settled`
(`:2528-2530`) : fr « Cette facture porte un règlement, même partiel : annulez-le d'abord pour
pouvoir émettre un avoir. » Registre du glossaire (`docs/i18n-glossaire.md` § Registre : de =
*Sie*, it = 2ᵉ personne du singulier, en neutre). `details` : `{ "invoiceId", "settlementId" }`.

**AC 13** — La fixture `monter` (`invoice_settlement.rs:937-1019`) cesse d'obtenir l'état
`InvoiceCredited` **par le chemin que 25-4-a ferme**, avec un commentaire qui dit que l'état n'est
**plus atteignable par l'application** depuis 25-4-a, mais **reste possible** par l'import d'une
sauvegarde antérieure — ce qui justifie de garder `SettlementCancelBlocker::InvoiceCredited`.
⛔ Ne **pas** supprimer ce motif ni ses tests. Ses doc-comments, qui le présentent comme un chemin
normal (« créditée par un avoir après ce règlement … pas une anomalie », `errors.rs:214-217` ;
« Hors `validated`, c'est donc un avoir », `invoice_settlements_write.rs:327-330`), disent
désormais que l'état n'est plus atteignable **que par l'import d'une sauvegarde antérieure**.

⚠️ **`monter` sert quinze cas, pas un** : `la_precedence_de_l_annulation_lecture_et_ecriture`
(`:1056-1064`) l'appelle avec les cinq motifs seuls **et les dix paires** (`RANGS`,
`assert_eq!(cas.len(), 15)`), dont **quatre combinent `InvoiceCredited`** avec un motif monté
**après** lui dans le même corps (clôture d'exercice, rapprochement bancaire, compte archivé,
aucun exercice ouvert). Le montage doit rester compatible avec ces quatre suites, et **les quinze
cas restent verts**.

⚠️ **Un avoir `issued` ne s'insère pas en SQL nu** : `chk_credit_notes_issued_has_je` exige
`credit_note_number` **et** `journal_entry_id` non nuls (`20260627000001_credit_notes.sql:53-54`),
donc une écriture de contre-passation valide. **Gabarit recommandé**, qui garde les **deux
écritures produites par les vrais chemins** et ne touche en SQL qu'un rattachement : régler (vrai
chemin) ; **détacher** la ligne de règlement vers une facture auxiliaire validée de la même société
(`UPDATE invoice_settlements SET invoice_id = ?`) ; créer l'avoir (vrai chemin, désormais accepté) ;
**rattacher** la ligne à la facture d'origine. La facture auxiliaire est créée **dans `monter`**
par le helper existant `validated_invoice` (même société, montant quelconque), n'est jamais réglée
ni créditée, et ne sert qu'à porter la ligne le temps de l'avoir ; aucun motif ne la lit. Seule
contrainte d'unicité d'`invoice_settlements` : `uq_invoice_settlements_entry (journal_entry_id)`
(`20260827000001_invoice_settlements.sql:65`) — le détachement ne la touche pas. ⚠️ Pendant le
détachement, l'écriture de règlement reste celle de la facture d'origine : aucun motif de
`settlement_cancel_blocker` ne doit être évalué entre les deux `UPDATE`. Si le gabarit se révèle
impraticable, **arrêter et demander** : le repli naturel — construire l'écriture d'avoir avec les
lignes de `generate_credit_note_journal_lines` — est **impossible depuis un test**, la fonction
étant privée (`credit_notes.rs:187`), et la rendre publique pour un test n'est pas un choix du
développeur. La facture auxiliaire est datée **dans l'exercice seedé** (`d − 20` suffit) : c'est ce
qui garde valide le cas « aucun exercice ouvert ».

**AC 14** — Frontend : le bouton « Créer un avoir » (`invoices/[id]/+page.svelte:836`) **suit le
précédent du bouton « Dévalider »** (`:864-871`) : la condition `!invoice.paidAt` est **retirée**, le
bouton s'affiche pour `canManage`, et **le serveur refuse avec le motif nommé**, que le dialogue
affiche déjà (`creditNoteError = err.message`, `:101`). ⛔ L'écran **ne rejoue pas** la règle métier :
une condition client qui en couvre une moitié laisse l'utilisateur sans rien à lire, et une qui la
couvre entière se désynchronise à la première évolution du serveur. La garde étant désormais
**unique** côté serveur (AC 11), le motif « payée » et le motif « réglée en partie » s'affichent de
la même façon. Le commentaire de `:864-871` est étendu aux deux boutons, ou dupliqué au-dessus du
second. ⚠️ Le commentaire du bloc Suspendre/Reprendre (`:842-844`) affirme que ces boutons sont
« masqués, comme les boutons voisins « Créer un avoir » / « Supprimer » » : il devient faux et se
corrige dans le même patch.

⚠️ **Changement visible sur une facture entièrement payée** : le bouton, masqué jusqu'ici, apparaît ;
l'utilisateur ouvre le dialogue, confirme, et lit le refus nommé (AC 12) dans le dialogue même
(`creditNoteError`, `:77`, `:101`). C'est le comportement de « Dévalider » depuis le 2026-09-19, et
le CHANGELOG le dit.

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
| e2e API `get_credited_invoice_reports_zero_amount_due` | AC 8 — la surface exposée aux clés API |
| `amount_due_after_settlement_then_legacy_credit_is_minus_settled` | AC 8 — état hérité monté en SQL : `−40` |
| `settled_and_credited_forms_are_at_parity` | AC 6 |
| `credit_note_refused_on_partially_settled_invoice` | AC 9, AC 10 — **rien écrit** : aucun avoir, facture `validated`, `version` inchangée, solde de créance inchangé, séquence d'avoir non consommée |
| `credit_note_refused_on_paid_invoice` (**existant, réécrit**) | AC 11 — nouvelle variante, plus `IllegalStateTransition` |
| `credit_note_accepted_after_the_settlement_is_cancelled` | la promesse du message : annuler le règlement rouvre l'avoir |
| `credit_note_waits_for_a_concurrent_settlement` | AC 9 — **entrelacement** : une transaction verrouille la facture et insère un règlement ; `create_credit_note` lancé en parallèle attend, puis **refuse** après le commit. Patron : la sonde d'entrelacement de la 25-3-b |
| e2e API `credit_note_on_partially_settled_invoice_is_409` | la frontière HTTP : 409, `CREDIT_NOTE_INVOICE_SETTLED`, message fr |
| Vitest fiche facture | AC 14 — bouton **présent** sur une facture réglée (en partie **et** en totalité) ; au refus, le dialogue affiche le message du serveur |

⛔ **Chaque test porte au moins une facture à TVA non nulle.** À 0 %, HT = TTC et #455 est
invisible — c'est ainsi qu'il est passé.

⛔ **Mutations à exécuter et à consigner** (résultat **observé**, pas supposé) : (m1) revenir à
`SUM(cn.total_amount)` dans la forme scalaire ; (m2) idem dans la forme jointe seule ; (m3) retirer
la lecture des règlements de la garde (ne garder que `paid_at`) ; (m4) retirer `paid_at` (ne garder
que la ligne) ; (m5) lire les règlements **avant** le verrou, non verrouillant ; (m6) remettre
`!invoice.paidAt` sur le bouton ; (m7) mapper la variante sur le générique `ILLEGAL_STATE_TRANSITION`. Chacune doit faire rougir au moins un test.

**AC 17** — Manuels et notes (`docs/manual/fr/user-manual.tex`, PDF régénéré et **contrôlé
aplati**) :
- `:1040` et `:2073` : « un avoir est refusé sur une facture **marquée payée** » devient « … sur une
  facture **réglée, même en partie** » — annuler d'abord le règlement ;
- § *Avoirs* (`\label{sec:avoirs}`, `:1007`) : le refus et son motif sont nommés à côté du refus
  « compte de produit archivé » ;
- `:963` et `:1455` (« la facture a été créditée par un avoir… ») : précisés comme **cas hérité**
  — une facture créditée avant la v0.12.1, ou restaurée d'une sauvegarde antérieure ;
- `CHANGELOG.md` `[0.12.1]` *Fixed* : les deux défauts, en langage d'utilisateur — et, pour #455,
  **nommer l'API** : `GET /invoices/{id}` rendait la TVA en `amountDue` sur une facture créditée ;
  une intégration tierce qui le lisait doit le savoir.

⚠️ **Greper le symptôme, pas la phrase** : `grep -rniE "payée|encaissée|réglée" docs/manual/fr/*.tex`
restreint aux lignes qui parlent d'avoir, plus le PDF aplati. Les manuels DE/IT/EN sont des stubs.

## Tasks / Subtasks

- [x] **T1 — formule de l'avoir TTC** (AC 1-5) : factoriser la formule de ligne, réécrire les deux
      constantes d'avoir, doc-comments.
- [x] **T2 — parité et concordance** (AC 6-8) : nouveau fichier de test sur le patron
      `invoice_ttc_parity.rs`. ⚠️ Le helper `validated_invoice` (`invoice_settlement.rs:32`) est
      **à 0 % en dur** et passe une écriture de vente **au HT** : les factures à 8,1 % se montent
      par le **vrai chemin de validation**, sur le patron de `create_and_validate`
      (`credit_notes_repository.rs`) ; l'état hérité, par le gabarit de l'AC 13.
- [x] **T3 — garde de l'avoir** (AC 9-11) : lecture verrouillante des règlements après le verrou,
      variante `DbError`, `error_code()`.
- [x] **T4 — API et i18n** (AC 12) : mapping, clé ×4 ; e2e 409.
- [x] **T5 — fixture `monter`** (AC 13) : gabarit « détacher, créditer, rattacher », commentaire ;
      **les quinze cas** de la précédence restent verts, dont les quatre paires avec `InvoiceCredited`.
- [x] **T6 — frontend** (AC 14) : condition du bouton retirée, commentaire, Vitest.
- [x] **T7 — tests et mutations** (AC 16) : m1 à **m7** exécutées, résultats consignés.
- [x] **T8 — textes** (AC 17) : manuel, PDF aplati, CHANGELOG.
- [x] **T9 — gates** : backend complet (base remise à zéro), frontend complet, E2E complet.

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
| `crates/kesh-db/src/errors.rs:119-175, 205-260, 300-470, 641-670` | `UnvalidationBlocker`, `SettlementCancelBlocker` (doc d'`InvoiceCredited` `:214-217`), `DbError`, `error_code()` |
| `crates/kesh-db/src/repositories/invoice_settlements_write.rs:60-72, 320-333` | verrou de `settle_invoice` ; rang 1 de `settlement_cancel_blocker` (commentaire `:327-330`, AC 13) |
| `crates/kesh-db/src/repositories/invoices.rs:1430-1448` | la garde de dévalidation, patron de l'AC 9 |
| `crates/kesh-api/src/errors.rs:2515-2560, 2741-2748` | mapping `InvoiceNotUnvalidatable`, générique |
| `crates/kesh-api/src/routes/credit_notes.rs:183-205` | handler de création |
| `crates/kesh-db/tests/invoice_ttc_parity.rs` | patron du test de parité |
| `crates/kesh-db/tests/credit_notes_repository.rs:267-299` | `credit_note_refused_on_paid_invoice` |
| `crates/kesh-db/tests/invoice_settlement.rs:32, 116-125, 937-1019` | helper à 0 %, `solde()`, `monter` |
| `frontend/src/routes/(app)/invoices/[id]/+page.svelte:90-104, 836, 864-871, 1022-1041` | dialogue d'avoir (`err.message` `:101`), bouton, précédent « Dévalider », reste dû |
| `crates/kesh-api/src/routes/invoices.rs:621-642` | `get_invoice` : `amountDue` rendu sans condition de statut |

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

Claude Opus 5.5.

### Debug Log References

- `settled_and_credited_forms_are_at_parity` attendait **huit** factures ; il y en a **sept** (six du jeu +
  la facture auxiliaire du gabarit). Erreur de décompte dans le test, pas dans le code — corrigée,
  et l'assertion anti-vacuité garde des valeurs **non nulles** des deux côtés.
- `SUM(next_number)` rend un `DECIMAL` sous MariaDB : `CAST(… AS SIGNED)` dans l'empreinte du test de
  refus.
- Un doc-comment égaré (« Paramètres de recherche, tri et pagination. ») précédait
  `INVOICE_TTC_SUBQUERY_SQL` : remis au-dessus d'`InvoiceListQuery`, qu'il décrit.
- ⚠️ **Relevé hors périmètre, non corrigé** : la clé voisine `error-invoice-unvalidate-blocked-settled`
  (it-CH) dit « annullatelo » — forme *voi*, contraire au registre du glossaire (2ᵉ personne du
  singulier). La nouvelle clé suit le glossaire (« annullalo »).

### Completion Notes List

- **Formule du TTC d'une ligne, source unique** (AC 5) : macro `line_ttc_sql!` (`invoices.rs`),
  rendant un littéral via `concat!` ; les **quatre** constantes en dérivent (TTC facture scalaire et
  jointe, avoir scalaire et joint), qui restent des `&'static str`.
- **Avoir compté TTC** (AC 1-4) : les deux constantes d'avoir somment `credit_note_lines` (jointes à
  `credit_notes`, `status = 'issued'`), arrondi ligne par ligne ; doc-comments sur l'unité et sur
  `total_amount` HT. Aucune migration.
- **Parité et concordance** (AC 6-8) : `tests/invoice_amount_due_parity.rs`, 4 tests, **tous à TVA
  non nulle** ; factures par le vrai chemin de validation ; l'état hérité par le gabarit « détacher,
  créditer, rattacher ». Le crédit de l'avoir égale le crédit de la créance **dans l'écriture**
  (1207.71 sur trois lignes à 8,1 % / 2,6 % / sous-centime), et le test s'assure que `total_amount`
  (HT) en diffère — sans quoi il ne distinguerait rien.
- **Garde de l'avoir** (AC 9-11) : lecture **verrouillante** des règlements, **après** le verrou de la
  facture ; une ligne **ou** `paid_at` ; variante `DbError::CreditNoteBlockedBySettlement`
  (`CREDIT_NOTE_INVOICE_SETTLED`), qui remplace aussi le générique du cas « payée ».
- **API et i18n** (AC 12) : 409, message suffixé du numéro de facture, `details { invoiceId,
  settlementId }` ; clé `error-credit-note-blocked-settled` ×4.
- **Fixture `monter`** (AC 13) : gabarit « détacher, créditer, rattacher » ; les **quinze cas** de la
  précédence restent verts. Doc-comments de `InvoiceCredited` et du rang 1 : état hérité.
- **Frontend** (AC 14) : condition `!invoice.paidAt` retirée du bouton, commentaires Suspendre/Reprendre
  corrigé ; `data-testid` sur le bouton, la confirmation et l'erreur du dialogue.
- **Textes** (AC 17) : manuel (refus nommé en § Avoirs, deux limites corrigées pour couvrir le règlement
  partiel — l'une « réglée, même en partie », l'autre « déjà encaissée, même en partie », deux motifs d'annulation de règlement précisés comme **cas hérité**), PDF régénéré et
  contrôlé aplati (0 `??`, six phrases de contrôle présentes, « marquée payée » absente) ; CHANGELOG
  *Fixed* ×2 (dont **l'API** pour #455) et *Changed* (le bouton). Grep large du symptôme « avoir sur
  facture payée » : aucun autre site.

**Tests ajoutés** (recomptés depuis la source, `main` → arbre de travail) : `invoice_amount_due_parity`
0 → 4, `credit_notes_repository` 6 → 9 (dont `credit_note_refused_on_paid_invoice` réécrit),
`invoice_echeancier_e2e` 11 → 13, Vitest `invoice-settlements-page.test.ts` 2 → 5.

**Mutations** (résultat **observé**, chaque fichier restauré après) :

| Mutation | Tests rouges |
|---|---|
| m1 — avoir scalaire en `SUM(cn.total_amount)` | 4 / 4 de `invoice_amount_due_parity` |
| m2 — avoir joint seul en `SUM(total_amount)` | `settled_and_credited_forms_are_at_parity` |
| m3 — garde sur `paid_at` seul | `…refused_on_partially_settled_invoice`, `…accepted_after_the_settlement_is_cancelled`, `…waits_for_a_concurrent_settlement` |
| m4 — garde sur la ligne seule | `credit_note_refused_on_paid_invoice` |
| m5 — lecture avant le verrou, non verrouillante | `credit_note_waits_for_a_concurrent_settlement` |
| m6 — `!invoice.paidAt` remis sur le bouton | 2 Vitest (bouton sur facture payée ; refus dans le dialogue) |
| m7 — code mappé sur `ILLEGAL_STATE_TRANSITION` | `credit_note_on_partially_settled_invoice_is_409` |

### Gates

- Backend : base remise à zéro (tmpfs + migrations + seed), `scripts/test-fast.sh` (fmt + clippy +
  nextest) **2484 / 2484** (2475 + 9).
- Frontend : `check` 0 erreur (27 avertissements), `lint-i18n-ownership` PASS, `test:unit`
  **826 / 826** (823 + 3), build OK.
- E2E : `kesh_e2e` reconstruite, montage complet (`smtpConfigured:true`, répertoires inbox et documents),
  frontend buildé après le dernier patch, run à **22:52 UTC** : **222 passés / 10 échoués / 19
  ignorés**. Les 7 de KF-029 ; les 3 autres — `product-revenue-account:133`, `sidebar-navigation:75`
  (pollution documentée) et ⚠️ `accounts.spec.ts:185` (**non documenté**) — passent **rejoués seuls,
  18 / 18**, et ne touchent aucun écran de la story. Trois pollutions dans un même run, là où la
  baseline en compte une : à surveiller, sans régression de la story.

### File List

| Fichier | Nature |
|---|---|
| `crates/kesh-db/src/repositories/invoices.rs` | macro `line_ttc_sql!`, constantes TTC recomposées, doc-comment replacé |
| `crates/kesh-db/src/repositories/invoice_settlements.rs` | avoir TTC (scalaire, joint), doc d'`amount_due` |
| `crates/kesh-db/src/repositories/credit_notes.rs` | garde : lecture verrouillante des règlements |
| `crates/kesh-db/src/errors.rs` | `CreditNoteBlockedBySettlement`, `error_code()`, doc d'`InvoiceCredited` |
| `crates/kesh-db/src/repositories/invoice_settlements_write.rs` | commentaire du rang 1 |
| `crates/kesh-api/src/errors.rs` | mapping 409 |
| `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` | `error-credit-note-blocked-settled` |
| `crates/kesh-db/tests/invoice_amount_due_parity.rs` | **nouveau** — parité, concordance, zéro, hérité |
| `crates/kesh-db/tests/credit_notes_repository.rs` | refus payée réécrit ; partiel, réouverture, entrelacement |
| `crates/kesh-db/tests/invoice_settlement.rs` | `monter` : gabarit « détacher, créditer, rattacher » |
| `crates/kesh-api/tests/invoice_echeancier_e2e.rs` | e2e : `amountDue` nul, 409 |
| `frontend/src/routes/(app)/invoices/[id]/+page.svelte` | bouton, commentaires, `data-testid` |
| `frontend/src/routes/(app)/invoices/[id]/invoice-settlements-page.test.ts` | 3 Vitest |
| `docs/manual/fr/user-manual.tex` / `.pdf` | avoirs, limites, cas hérité |
| `CHANGELOG.md` | *Fixed* ×2, *Changed* |

## Change Log

- **2026-09-27** — ⚠️ **Rectification après livraison** (relevée en validation de 25-4-b1, P3-P4) :
  l'AC 13 dit l'état « avoir après règlement partiel » atteignable **« que par l'import d'une
  sauvegarde antérieure »**. C'est **faux par omission** : la **0.12.0 publiée** accepte cet avoir
  (`git show v0.12.0:crates/kesh-db/src/repositories/credit_notes.rs:302`), si bien qu'une
  installation **mise à jour sur place** peut le porter. Lire : « des données antérieures à la
  0.12.1, restaurées ou mises à jour ». Les quatre commentaires de code qui reprenaient la
  formulation sont rectifiés par 25-4-b1 (T0) ; le manuel disait déjà juste.
- **2026-09-27** — **revue de code P1** (Sonnet, diff `ab9b9448..9bf2bb2b`, prompt
  `25-4-a-review-prompt-p1.md`) — **0 au-dessus de LOW, 1 LOW** : le compte rendu attribuait la
  formule « réglée, même en partie » aux deux limites du manuel, la seconde dit « déjà encaissée,
  même en partie » — compte rendu corrigé, le manuel était juste. Les huit axes exercés avec preuve :
  les constantes TTC facture **octet pour octet identiques** à leur forme d'avant, l'entrelacement ne
  peut pas passer à vide (le motif ne correspond qu'à la requête verrouillante de l'avoir), tous les
  lecteurs d'`amount_due` inchangés et resserrés, PDF aplati, i18n, Vitest, décomptes recomptés.
  Non exercé : la ré-exécution des gates et des mutations (interdite à la lentille). **Revue close en
  une passe ; story `done`.**
- **2026-09-27** — **dev** (Opus 5.5) : formule du TTC de ligne factorisée (`line_ttc_sql!`), avoir
  compté TTC, garde verrouillante de l'avoir sur facture réglée, variante `CREDIT_NOTE_INVOICE_SETTLED`
  ×4 locales, fixture `monter` par le gabarit, bouton aligné sur « Dévalider », manuel et CHANGELOG.
  +9 tests Rust, +3 Vitest ; **7 / 7 mutations tuées**. Gates : backend 2484/2484, frontend 826/826,
  E2E 222/10/19 sans régression (7 KF-029 + 3 pollutions, rejouées seules vertes).
- **2026-09-27** — **validation P4 ciblée** (Sonnet, diff `1344b3a2..b18f7cd2`, prompt
  `25-4-a-validate-prompt-p4.md`) — rapporte **1 MEDIUM, 3 LOW**, tous confirmés et corrigés.
  **MEDIUM reclassé LOW** par l'orchestrateur : `invoice_settlements_write.rs` manquait au tableau
  « Où regarder », mais l'AC 13 qui prescrit son édition le **nomme avec sa ligne** — une omission de
  navigation, sans risque qu'une prescription soit perdue. LOW : citation `:326-329` → `:327-330` ;
  plage d'`errors.rs` étendue à `SettlementCancelBlocker` ; commentaire de la ligne mère de
  `sprint-status` resté à trois stories. Le gabarit de l'AC 13, l'AC 8 et l'AC 14 revérifiés sans
  défaut ; la sérialisation en chaîne confirmée dans la source de `rust_decimal` 1.42.1.
  ⛔ **Boucle de validation CLOSE en 4 passes** : `1H/3M/1L → 1M → 2M/5L → 0 > LOW` (après
  reclassement), rotation Sonnet → Haiku → Opus → Sonnet, deux passes ciblées ; toutes les
  corrections ont porté sur la spec, aucune sur du code. **Prête pour le dev.**
- **2026-09-27** — Arbitrage de Guy : le refus de l'avoir sur facture réglée (en tout ou en partie) reste ; sa levée est tracée par #471.
- **2026-09-27** — **validation P3 ciblée** (Opus, diff aplati des remédiations P1-P2, prompt
  `25-4-a-validate-prompt-p3.md`) — **2 MEDIUM, 5 LOW**, tous de **propagation** des corrections P1,
  tous confirmés par grep. MEDIUM : T7 disait encore « m1 à m6 » (m7 ajoutée) ; l'AC 14 rendait faux
  le commentaire Suspendre/Reprendre (`:842-844`) et l'AC 13 laissait deux doc-comments décrire
  l'avoir après règlement comme un chemin normal. LOW : « non nul » pour « non `null` » (et
  `unwrap_or` interdit) ; repli de l'AC 13 impossible (fonction privée) — remplacé par « arrêter et
  demander », date de la facture auxiliaire précisée ; cause de la sérialisation en chaîne ;
  fiche mère et `sprint-status` en retard d'une passe ; T2 ne disait pas comment monter une facture
  à 8,1 %. Le gabarit « détacher, créditer, rattacher » a été **exécuté mentalement** contre
  `create_credit_note`, le rang 1 et les quatre paires : il tient. Référence de l'AC 10 corrigée
  (`errors.rs:453`). Aucun code de production touché par ces corrections.
- **2026-09-27** — **validation P2** (Haiku, diff aplati, prompt `25-4-a-validate-prompt-p2.md`) —
  rapporte 3 MEDIUM et 1 LOW ; **un seul MEDIUM retenu** : l'AC 13 ne disait pas d'où vient la facture
  auxiliaire du gabarit — précisé (helper `validated_invoice`, contrainte d'unicité vérifiée, aucun
  motif évalué pendant le détachement). **Écartés** : deux « MEDIUM » reprochent au **code** de ne pas
  encore faire ce que la fiche prescrit (bouton `:836`, test e2e absent) — ce n'est pas un défaut de
  spec ; le LOW (deux formulations du manuel) est déjà couvert par l'AC 17. ⚠️ **Repris par
  l'orchestrateur**, la priorité 3 que la passe n'a pas tranchée : `amountDue` est une **chaîne** à
  l'échelle du calcul SQL — l'AC 8 impose désormais `null` interdit et une comparaison **en
  `Decimal`**, ni chaîne ni `f64` ; et la priorité 2 : ce que voit l'utilisateur sur une facture
  entièrement payée (bouton désormais visible, refus nommé dans le dialogue) est écrit à l'AC 14.
  La passe affirmait aussi que `accept_one_invoice` ne crée pas de règlement — faux
  (`invoice_settlements::create_in_tx`, `reconciliation.rs:1416-1428`) ; sans conséquence sur la fiche.
- **2026-09-26** — **validation P1** (Sonnet, prompt `25-4-a-validate-prompt-p1.md`) — **1 HIGH,
  3 MEDIUM, 1 LOW**, tous confirmés dans le code avant correction. **HIGH** : le récit « #455
  latent » était faux — `get_invoice` rend `amountDue` sans condition de statut, et
  `GET /invoices/{id}` est ouverte aux clés API : une facture créditée à TVA non nulle y rend la
  TVA ; récit corrigé ici et dans la fiche mère, AC 8 étendu à la frontière HTTP, e2e ajouté,
  CHANGELOG à nommer l'API. **MEDIUM** : `DbError::code()` n'existe pas — c'est `error_code()` ;
  AC 14 contredisait le précédent « Dévalider » qu'il citait — il le suit désormais (bouton
  visible, le serveur refuse avec le motif nommé, m7 ajoutée) ; AC 13 ignorait que `monter` sert
  quinze cas dont quatre paires avec `InvoiceCredited`, et qu'un avoir `issued` exige une écriture
  (`chk_credit_notes_issued_has_je`) — gabarit « détacher, créditer, rattacher » prescrit. **LOW** :
  `err.message` est à `:101`, non `:98`. Axes tous exercés ; non exercé : la sémantique REPEATABLE
  READ n'a pas été mesurée (raisonnée).
- **2026-09-26** — Créée (Opus 5.5) après découpage de la 25-4. Inventaire vérifié par `grep -nF`
  sur chaque site cité. Deux faits hors des issues : #455 est **latent** sur les données neuves et
  ne se voit que dans l'état que #456 produit — d'où leur réunion ; et la fixture `monter` encode
  #456 comme attendu.

[#471]: https://github.com/guycorbaz/kesh/issues/471
[#455]: https://github.com/guycorbaz/kesh/issues/455
[#456]: https://github.com/guycorbaz/kesh/issues/456
