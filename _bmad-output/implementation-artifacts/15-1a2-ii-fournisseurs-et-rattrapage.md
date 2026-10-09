# Story 15.1a2-ii : Le lettrage des pièces fournisseurs, et le rattrapage des données existantes

## Status

ready-for-dev *(découpée de la 15-1a2 le 2026-10-09 à la remédiation de sa validation P1 — C-15-1a2-1 ;
validation P2 remédiée le 2026-10-09 — refus au délettrage (C-15-1a2-10), découverte par statut,
classe A justifiée honnêtement ; validation P3 remédiée le 2026-10-09 — les refus fournisseurs du rang 2 bis
partis à la **15-1a2-0** (C-15-1a2-19), un test existant de plus à modifier, la définition de la classe A
réécrite dans le code ; **validation P4 (ciblée) CLOSE le 2026-10-09** — 0 au-dessus de LOW, six LOW appliqués)*

## Story

**As a** indépendant, PME ou fiduciaire qui paie ses fournisseurs dans Kesh, et qui met à jour une
installation où des factures sont déjà soldées,
**I want** que l'achat et le paiement d'une facture fournisseur payée soient lettrés entre eux, et que
les pièces soldées et les contre-passations **d'avant la mise à jour** le soient aussi,
**so that** le grand livre dise, dès la mise à jour, ce que disent les pièces — et que, après la
restauration d'une sauvegarde, les pièces soldées soient de nouveau lettrées (les contre-passations
libres d'une sauvegarde antérieure, elles, restent à lettrer à la main : M2 n'est pas rejouée, P6).

Troisième des trois sous-fiches de la **15-1a2** (index : `15-1a2-lettrage-des-pieces.md`). ⛔ **Suppose la
15-1a2-0 et la 15-1a2-i mergées**. De la **15-1a2-0** : la règle des périodes (`OpenPeriodRule`,
`lines_in_open_period`, sa D1) et le **refus du rang 2 bis** dans la file commune **et dans les deux gestes
fournisseurs** (`cancel_settlement_in_tx`, `cancel_in_tx` — sa D3), avec textes et écran. De la
**15-1a2-i** : sa synchronisation (P3 : découverte verrouillante, groupe existant, cible, abstention), son
extension d'audit (`DocumentRef`, défini en sa P3 ; `*_inner`) et sa fixture partagée
(`crates/kesh-db/tests/support/lettering_documents.rs`, incluse par `#[path]` — sa T5, C-15-1a2-28) ; et son test
d'accord (AC6) compare le rattrapage à **cette** synchronisation.
Ordre : 15-1a-i → 15-1a-ii → 15-1a2-0 → 15-1a2-i → **15-1a2-ii** → 15-1b-0 → 15-1b → 15-1c.

**Numérotation conservée** de la 15-1a2 (P2, P3, P4, P6 ; AC6, AC7, AC8–AC12) ; numéro neuf : **AC16**.
Les éléments partagés avec la 15-1a2-i (AC8, AC9, AC10, AC12, P3, P4) portent ici leur **part ii**.

## Le modèle réel — relevé sur `056997b0`

*(Lignes sur `056997b0` ; **citer et re-greper par le nom de fonction**.)*

**Compte fournisseurs `B`** = celui de **la première ligne au crédit de l'écriture d'achat** (lecture
`supplier_invoices::purchase_payable_line`, `supplier_invoices.rs:660`, déjà faite par `pay_in_tx`).
Achat : **un** crédit sur `B` = TTC ; règlement : **un** débit sur `B` = TTC, **toujours complet** (pas
de paiement partiel fournisseur, une seule colonne de règlement). Mode de règlement : virement
(`bank_transfer`) ou **compte interne** (`internal_account`, `pay_in_tx` ≈ `:776`).

**Écrivains de `supplier_invoices.settlement_journal_entry_id`** — trois `UPDATE` :

| fonction | ligne | effet |
|---|---|---|
| `pay_in_tx` (`:713`) — appelée par `pay` (`:680`) **et** par `payment_batches::confirm_batch` (`payment_batches.rs:400`, appel `:457`, dans la boucle de `:456`) | `UPDATE … status = 'paid' …` `:895` | pose le règlement |
| `cancel_in_tx` (`:1032`) | `UPDATE … status = 'cancelled' …` `:1090` | contre-passe l'achat ; une facture **payée** voit son règlement **détaché** (colonnes à `NULL`), **non** contre-passé |
| `cancel_settlement_in_tx` (`:1229`) | `UPDATE … status = 'open' …` `:1299` | contre-passe le règlement |

Un règlement **détaché** n'est plus possédé (`reversal_blockers` lit `settlement_journal_entry_id`,
désormais `NULL`) : il est **lettrable à la main** (15-1a R5). L'achat d'une facture **annulée**, lui,
reste possédé (`purchase_journal_entry_id` intact, motif `OwnedBySupplierInvoice`) : sa paire `reversal`
**ne se dissout pas à la main** (C106, test
`supplier_invoice_cancel_letters_a_pair_that_cannot_be_dissolved_by_hand`).

## Décisions

### P2 — Le groupe `document` d'une facture fournisseur

Pour une facture fournisseur **payée** (`status = 'paid'`, `settlement_journal_entry_id` non nul) :
l'**ancre** est la ligne d'achat sur `B` ; `C(S)` = l'ancre et la ligne **sur `B`** du règlement. Même
règle que P1 (15-1a2-i) : `|C| ≥ 2`, somme nulle, `B` lettrable, au moins une ligne en période ouverte.

**La découverte dépend du statut** (validation P2, R-3 = F2-3 ; C-15-1a2-15) — c'est la seule partie de
l'algorithme qui diffère du client, où l'ancre est toujours dans `C(I)` :

| statut | `C(S)` | pourquoi |
|---|---|---|
| `paid` | l'ancre et la ligne sur `B` du règlement | le cas nominal |
| `open` | l'ancre **seule** (`\|C\| = 1` : jamais de cible) | aucun règlement en vigueur ; une facture ouverte n'a jamais de groupe `document` — le refus du rang 2 bis (15-1a2-0, D2-D3) interdit l'annulation qui en laisserait un |
| `cancelled` | **vide** — `Unchanged`, sans lecture verrouillante des lignes | l'achat d'une facture annulée reste **possédé** (`reversal_blockers`, `journal_entries.rs:2127-2129`, sans filtre de statut) et R6 l'a lettré `reversal` avec son miroir ; le soumettre à l'étape 2 de la part i (« une ligne de `C` lettrée `manual` ou `reversal` → `Invariant` ») rendrait `Invariant` sur un état légitime |

### P3 (part ii) — La synchronisation d'une facture fournisseur

`sync_supplier_invoice_in_tx(tx, company_id, supplier_invoice_id, held_open_fiscal_year_id, actor)` et
`dissolve_supplier_invoice_document_group_in_tx(…)`, dans `letterings.rs`, **même algorithme** que leurs
jumelles client (15-1a2-i P3, étapes 1 à 6) — factorisé : la découverte change, le reste est commun
(⛔ pas de seconde copie de l'algorithme ; une fonction privée prend la découverte en paramètre).
Compte `B` non lettrable → `SyncOutcome::AccountNotLetterable`, **aucune erreur** : un paiement
n'échoue **jamais** à cause d'un **refus** de lettrage (C-15-1a2-3). ⚠️ **Ce que la phrase ne couvre pas**
(validation P3, F3-7) : une erreur **structurelle** de la synchronisation — `DbError::Invariant`, ou
`LetteringConcurrentChange`, inatteignable par construction (15-1a2-i P4, C-15-1a2-14) — se propage par `?`
depuis `pay_in_tx` : le paiement est annulé avec sa transaction, et dans `confirm_batch`
(`payment_batches.rs:457`, dans la boucle `for item` de `:456` ; N règlements en une transaction) **le lot
entier**. La route répond **selon l'erreur** (validation P4, F4-1 — la version P3 disait « `500` » pour les deux) :
`Invariant` → `500` (`kesh-api/src/errors.rs` ≈ `:3720-3726`), c'est l'exception « 500 » du `CLAUDE.md`
§ « Pattern batch » (un défaut structurel n'est pas une erreur de proposition) ; `LetteringConcurrentChange` →
`409 LETTERING_CONCURRENT_CHANGE` (`errors.rs` ≈ `:3061-3068`, « un refus MÉTIER, jamais un 500 »), que la route
rendrait telle quelle s'il sortait — inatteignable, il n'entre à aucun tableau de refus. Écrit, non masqué. Audit : `DocumentRef` de la 15-1a2-i (sa P3), `document_type =
"supplierInvoice"` — la valeur même de `document.type` de la vue des postes ouverts (15-1b), non plus
`"supplier_invoice"` (C-15-1a2-22) —, `number = supplier_invoice_number`, **`None` quand la facture n'en a
pas**, émis `documentNumber: null` (la clé est présente ; la colonne est optionnelle ; `documentId` porte l'identifiant — même règle que `reversal_blockers`, « le numéro accompagne
l'identifiant quand il existe » ; validation P2, R-8 = F2-6 ; C-15-1a2-17). ⚠️ Ce n'est **pas** le repli de
`pay_in_tx` (`inv.supplier_invoice_number.clone().unwrap_or_else(|| inv.id.to_string())`,
`supplier_invoices.rs:848-851`), qui fabrique un libellé d'écriture, pas un numéro de pièce.

### P4 (part ii) — Où la synchronisation est appelée

| geste (`056997b0`) | appel | place | exercice tenu (`FOR UPDATE`) | acteur |
|---|---|---|---|---|
| `pay_in_tx` (`:713`) — **couvre** `pay` et `confirm_batch` | `sync_supplier_invoice_in_tx` | après l'`UPDATE … status = 'paid'` (`:895`) | `fy` de `find_open_covering_date(payment_date)` (`:843`) — l'exercice du règlement | `Actor { user_id, api_key_id: None }` |
| `cancel_settlement_in_tx` (`:1229`) | `dissolve_supplier_invoice_document_group_in_tx` | après les refus de l'étape (2) — qui portent le **rang 2 bis** depuis la 15-1a2-0 —, **avant** `reverse_owned_in_tx` (étape (3), `:1285`) | l'exercice du règlement, verrouillé à l'étape (1-bis) (`:1255` — valeur gardée) | idem |
| `cancel_in_tx` (`:1032`), facture **payée** | `dissolve_supplier_invoice_document_group_in_tx` | après les refus de l'étape (2) — qui portent le **rang 2 bis** depuis la 15-1a2-0 —, **avant** `reverse_owned_in_tx` (étape (3), `:1076`) | l'exercice de l'achat, verrouillé à l'étape (1) (`:1050` — valeur gardée) | idem |

**Les deux annulations refusent le rang 2 bis** (C-15-1a2-10, C-15-1a2-12) — **livré par la 15-1a2-0**
(sa D3 : motif ajouté aux filtres de `cancel_settlement_in_tx` et de `cancel_in_tx`, clés
`settlement-cancel-blocked-lettering-closed` et `supplier-invoices-cancel-blocked-lettering-closed`,
prédicteurs par la file commune), éprouvé là sur des groupes posés à la main ; la présente story ne touche
plus aux refus (C-15-1a2-19) et en éprouve l'**intégration** sur des groupes posés par le paiement (AC7).
**Ce que cela ferme** (validation P2, F2-1 HIGH = R-1) : l'annulation d'une facture **payée** dont l'achat et
le paiement sont sous la borne laissait, sous l'abstention de la version P1, un groupe `document` {achat,
règlement détaché} sur une facture **annulée**, indissoluble pour toujours (refus 1 de
`dissolve_group_in_tx`, et plus aucun geste ne resynchronise une facture `cancelled`). Elle est refusée ;
**aucun groupe orphelin** ne survit à l'annulation d'une facture fournisseur.

Pour une facture **ouverte** annulée, la dissolution est un **no-op** : une facture ouverte n'a jamais de
groupe `document` (P2 ; validation P2, R-9 — la phrase était fausse sous l'abstention, qui laissait un groupe
gardé sur une facture redevenue ouverte). Après la dissolution, la contre-passation lettre ce qui est libre
(15-1a-ii R6) : règlement ↔ miroir (annulation du règlement), achat ↔ miroir (annulation de la facture).
Le **règlement détaché** reste **ouvert**, lettrable à la main — et **contre-passable** (sa fiche
d'écriture : `modification_guard` ne gèle que la modification, `journal_entries.rs:1031-1040`) ; sa
contre-passation le lettre `reversal` avec son miroir (R6). Un règlement par **compte interne** lettrable
(ex. `1000`, actif) voit aussi sa ligne de contrepartie appariée à son miroir par R6 : AC7 l'asserte
(finding F-10).

Périodes closes : la règle de la 15-1a2-i P7 s'applique telle quelle — **abstention** à la création
(pièce historique entièrement close), **refus** du geste qui exigerait de délettrer (rang 2 bis de la
15-1a2-0, ci-dessus).

### P6 — Rattrapage des données existantes : DEUX migrations, triées différemment (C-15-1a2-4)

Les pièces soldées et les contre-passations passées **avant** la 15-1a2 (et, pour les contre-passations,
avant la 15-1a-ii) doivent être lettrées. ⛔ Des migrations qui écrivent des données → triage **P7**.
⛔ **Une migration appliquée ne se modifie plus** (P8) : `20261009000001` (15-1a-i, mergée) n'est pas
touchée.

⚠️ **Pourquoi deux fichiers et non un** (heurt avec la consigne « une migration », signalé) : le
registre de rejeu exige qu'un extrait porte **tous** les statements d'écriture de sa migration
(`extract_carries_every_write_statement_of_its_source_migration`, `post_restore.rs`), et une migration
ne peut être **à la fois** au registre et exemptée (`exemptions_are_real_and_disjoint_from_registry`).
Rejouer les groupes `document` sans rejouer les paires `reversal` libres impose donc deux fichiers.

**M1 — `<AAAAMMJJ>NNNNNN_lettering_documents_backfill.sql`** (version **strictement supérieure** à
`20261009000001`, datée au développement — à partir de `20261009000002` si le développement a lieu le
2026-10-09 : `<AAAAMMJJ>000001` ce jour-là serait un doublon de version, refusé par sqlx ; validation P3,
R3-5) — **registre `POST_RESTORE_BACKFILLS`, classe A** :

1. les groupes `document` **client** (15-1a2-i P1) ;
2. les groupes `document` **fournisseur** (P2) ;
3. les paires `reversal` dont l'**origine est l'écriture d'achat** d'une facture fournisseur
   (`supplier_invoices.purchase_journal_entry_id`) — factures annulées avant la 15-1a-ii.

**Pourquoi la classe A tient pour M1** (`CLAUDE.md` P7 : « ne pas réutiliser le critère "un `NULL` n'est
l'expression d'aucun choix" ») : aucune des lignes visées ne peut porter un `NULL` **choisi**. Un groupe
`document` ne se dissout pas à la main (`LETTERING_IS_DOCUMENT`) et se **recalcule** depuis
`invoice_settlements`, `credit_notes` et `supplier_invoices` : un groupe dissous à bon droit (règlement
annulé) ne se reforme pas, la pièce ayant changé. Une paire `reversal` dont une ligne est celle d'une
pièce ne se dissout pas à la main non plus (C106). Chaque statement est en outre gardé par
`lettering_key IS NULL` sur **toutes** les lignes du groupe candidat : le rejeu **ne délettre jamais et
ne réécrit jamais une marque posée**. `sql` = la migration entière (backfill pur, aucun DDL, patron
`20260729000001`). `registry_entries_are_within_import_window` : aucune table applicative créée depuis
`20260917000001`.

⚠️ **Ce que la classe A ne garantit PAS — écrit honnêtement** (validation P2, F2-5 ; C-15-1a2-16). Le
module définit la classe A par « le rejeu est un no-op strict sur une base à jour »
(`post_restore.rs:41-43`). Pour M1, c'est **presque** vrai : le rejeu peut **lettrer** ce que la base à
jour avait laissé ouvert **à bon droit**, dans deux états —
1. une pièce historique entièrement close à la mise à jour (M1 s'est abstenue), dont un administrateur a
   **depuis rouvert** l'exercice ou **reculé** la borne (`fiscal_years::reopen`, `companies::unlock_books`) :
   rien ne la resynchronise en vivant (15-1a2-i P7 point 1), le rejeu la lettre ;
2. une pièce dont le compte `A` ou `B` n'était pas lettrable au geste (`AccountNotLetterable`) et l'est
   devenu (retypage d'un compte de charge en compte de bilan) : la lettrabilité n'est jugée qu'à la création (C104),
   le rejeu crée le groupe.
Ce n'est **pas** écraser un choix : dans les deux cas, le groupe que pose le rejeu est celui que la
synchronisation poserait aujourd'hui, et aucun geste de l'utilisateur ne l'avait défait (une ligne de
pièce n'est jamais délettrable à la main). Mais ce n'est pas un « no-op strict » : une restauration
d'une base sur elle-même peut ajouter des marques. La justification du registre le **dit**, au lieu
d'affirmer le no-op ; le manuel d'administration aussi (AC12) ; AC16 (c) est borné à une base sans ces
deux états, et le cas 1 a son test (AC16 (d)). ⛔ **Et la définition de la classe A elle-même** — qui dit,
dans le code, « no-op strict sur une base à jour » (`post_restore.rs:41-43` au module, `:121` sur
`BackfillTrigger::Unconditional`, et l'échec de `post_restore_class_a.rs:358`) — est **réécrite par cette
story** (T3 ; validation P3, F3-2 = R3-2) : « gardée contre l'écrasement d'une valeur posée par
l'utilisateur ; sur une base à jour, le rejeu ne délettre ni ne réécrit rien, mais **peut poser** ce que la
base aurait posé aujourd'hui (pièce rendue lettrable depuis) ». M1 sera la seule entrée vivante de classe
A : la définition qu'en lira le prochain rédacteur d'une entrée doit être vraie d'elle.

**M2 — la version suivant immédiatement M1, `…_lettering_reversal_pairs_backfill.sql`** — **EXEMPTÉE du rejeu**
(`EXEMPT_MIGRATIONS`, `ExemptionBasis::Durable`) : les autres paires `reversal` (contre-passations
d'écritures manuelles, de rapprochements hors facture, de règlements annulés). ⛔ M2 **exclut
explicitement** les paires dont l'origine est une écriture d'achat (`NOT EXISTS (SELECT 1 FROM
supplier_invoices s WHERE s.purchase_journal_entry_id = <origine>)`), au lieu de s'en remettre à la garde
`lettering_key IS NULL` après M1 (validation P2, F2-8) : aujourd'hui les deux reviennent au même, mais un
écart futur entre les règles de M1 et M2 ferait porter par M2 — non rejouée — une paire que M1 seule doit
porter. **Justification**, qui
ne commence **pas** par « Hors fenêtre » (la migration est dans la fenêtre) : « Rejeu exclu à dessein :
un groupe `reversal` sans ligne de pièce se délettre à la main (`DELETE /letterings`) ; un `NULL` peut y
être un choix de l'utilisateur, qu'un rejeu à chaque import réécrirait en silence. Coût assumé : une
sauvegarde d'avant la 15-1a2 importée laisse ces paires ouvertes, lettrables à la main. »
`EXEMPT_MIGRATIONS.len()` : **16 → 17** (`every_exemption_declares_a_coherent_basis`, commentaire du
compteur à compléter).

**Règles communes du SQL** (M1 et M2) — la **même règle** que la synchronisation, sans quoi AC6 rougit :

- **compte lettrable** : `accounts.account_type IN ('Asset', 'Liability')` **et**
  `NOT EXISTS (SELECT 1 FROM bank_accounts b WHERE b.journal_account_id = a.id)` — le prédicat de
  `letterings::letterable_account`, recopié (C-15-1a2-3) ;
- **au moins une ligne du groupe en période ouverte** : exercice `Open`, aucun exercice postérieur
  `Closed` de la société, `entry_date > COALESCE(companies.books_locked_through, '0001-01-01')` —
  abstention sinon (C-15-1a2-2 ; Reçu points 3 et 10, recommandation du socle suivie) ;
- `|groupe| ≥ 2`, somme nulle, **toutes** les lignes `lettering_key IS NULL` ;
- clé = `MIN(jel.id)` du groupe, origine `document` (M1 1-2) ou `reversal` (M1 3, M2) ;
- paires `reversal` appariées **par position** (Reçu point 7) : rang dans `ORDER BY line_order` de
  l'origine contre rang dans `ORDER BY line_order` du miroir (`journal_entries.reverses_entry_id`), même
  compte, montants croisés — la forme de 15-1a-ii R6 ;
- ⛔ **forme des statements** : `UPDATE journal_entry_lines jel JOIN (SELECT … ) g ON g.line_id = jel.id
  SET …` — table dérivée, **pas** de `WITH` en tête : le détecteur P7 (`writes_data`) classe un statement
  sur son **premier mot-clé**, et un `WITH … UPDATE` lui échapperait ; pas de commentaire `/* */`
  (`migrations_contain_no_block_comment`), pas de littéral piège (`registry_sql_has_no_literal_hazard`) ;
  la faisabilité MariaDB 10.11 d'un `UPDATE` joint à une dérivée de la même table est vérifiée en T0.
- **Pas de relèvement** de `kesh_version_min_required` : la 15-1a-i l'a porté à `0.13.0` (C101) et les
  crates sont en `0.13.0` ; motif exact (C115) : tout binaire publié depuis la v0.10.0 annulerait un
  règlement sans dissoudre son groupe. **Non-breaking** (P1 : `UPDATE` de colonnes nullables).
- **Pas d'audit** pour le rattrapage (une migration n'a pas d'acteur) ; le manuel le dit (AC12).

⛔ **Deux implémentations de la même règle** (SQL de M1/M2, Rust de P3) : la duplication que le
`CLAUDE.md` interdit, acceptée **parce que P8 fige la migration** — et **tenue par le test d'accord**
AC6, sans lequel elle ne serait pas acceptable.

**Ce que M1/M2 laissent, et pourquoi** : les pièces historiques entièrement en période close restent
ouvertes, non lettrables à la main (15-1a2-i P7 point 1) ; une facture hors exercice ouvert reste comme
elle était lue au grand livre avant la 15-1b. **Aucune** des deux ne dissout un groupe : elles ne
posent que des marques sur des lignes libres.

## Critères d'acceptation

**AC6** — **Accord rattrapage ↔ synchronisation**, sur la fixture partagée (15-1a2-i T5) **étendue**
(paiements fournisseurs directs et par lot, compte interne, annulations de paiement et de facture
payée, contre-passation d'écriture manuelle, pièces **lettrées vivantes puis passées sous la borne ou dans
un exercice clos**, compte non lettrable) : (a) on relève les marques posées par les gestes vivants ;
(b) on efface en SQL brut les marques d'origine **`document` et `reversal`** — **pas** les groupes `manual` :
R6 laisse volontairement une ligne d'origine dans son groupe `manual` et son miroir ouvert
(`journal_entries.rs:2575`, `if lettering_key.is_some() { continue; }`) ; effacer aussi ce groupe ferait
apparier cette ligne par M2 et rougir l'accord à tort (validation P3, F3-4 ii) ; (c) on exécute M1 puis M2 ;
(d) **deux régimes**, chaque pièce et chaque paire étant classée **par construction de la fixture** — la
liste des identifiants de chaque régime est **écrite dans le test** au montage, d'après les dates et les
verrous que le test a lui-même posés —, **jamais** en appelant `lines_in_open_period` / `open_period_rule`
(le prédicat même que la synchronisation emploie : la frontière sortirait du code comparé, assertion verte
par construction — D4-ter ; validation P3, F3-4 i ; C-15-1a2-16) :
- **(d-i) une ligne au moins en période ouverte** au moment de (c) → marques **identiques** à (a), sans
  exception ;
- **(d-ii) aucune ligne en période ouverte** au moment de (c) — pièce créée en période ouverte et
  lettrée par le geste, **puis** figée par un verrou ou une clôture (une écriture ne se crée ni sous la
  borne ni dans un exercice clos : c'est la seule façon de fabriquer cet état) ; ou paire `reversal` dont
  le miroir est depuis passé sous la borne → le rattrapage **s'abstient** : lignes **ouvertes** après (c),
  là où le vivant avait lettré. Différence **attendue, assertée comme telle** — pièce par pièce, non par
  un décompte.
Il n'y a pas d'autre différence : le « groupe gardé » de la version P1, seconde divergence, n'existe plus
(refus, 15-1a2-0 D2-D3) ; (e) l'appel de `sync_invoice_in_tx` / `sync_supplier_invoice_in_tx` sur
**chaque** pièce rend `Unchanged`, `AbstainedClosedPeriods` ou `AccountNotLetterable` — **jamais** une
écriture, jamais `Invariant` (une facture fournisseur **annulée** a une découverte vide, P2) —, et ne
produit **aucune** entrée d'audit. Cela vaut **aussi** pour la pièce dont un groupe `document` a survécu au
passage de son compte en non lettrable (C104) : l'étape 3 de la synchronisation est **terminale** — elle rend
`AccountNotLetterable` sans rien écrire, quel que soit le groupe existant (15-1a2-i P3, validation P4, M-2 =
F4-3 ; C-15-1a2-29) — et la fixture porte ce cas. **Exercice tenu** (validation P3, R3-7 a) : le test tient `FOR UPDATE`
l'exercice **ouvert courant** de la fixture et le passe à chaque appel ; `check_held_fiscal_year` n'est
évalué que par les primitives, c'est-à-dire **seulement quand la synchronisation écrit** — or (e) asserte
qu'elle n'écrit pas : un appel qui voudrait écrire échoue le test (issue `Created`/`Recreated`, ou
`Invariant` d'un exercice tenu qui ne couvre aucune ligne) — dans les deux cas un défaut, jamais un faux
vert.

**AC7** — Fournisseur : paiement (direct **et** par lot pain.001 confirmé, `confirm_batch`) → achat et
règlement lettrés `document` ; annulation du règlement → dissous, règlement ↔ miroir lettrés, achat
ouvert ; annulation d'une facture payée → dissous, achat ↔ miroir lettrés, **règlement détaché ouvert**
et lettrable à la main (`POST /letterings` l'accepte avec une ligne de même compte qui le solde) — puis,
dans un second volet, **contre-passé** par sa fiche d'écriture → lettré `reversal` avec son miroir ;
paiement par **compte interne** lettrable puis annulation → la ligne de contrepartie et son miroir sont
aussi lettrés `reversal` ; compte `B` **non lettrable** → paiement réussi, aucun groupe, aucune erreur.
**Périodes closes** (rang 2 bis de la 15-1a2-0, ici sur un groupe posé **par le paiement**) : achat et
paiement en période ouverte, groupe posé par le paiement, verrou posé ensuite → au **dépôt**, où vit le test
(validation P4, F4-6), l'annulation du **paiement** rend `SettlementNotCancellable { blocker:
DocumentLetteringInClosedPeriods }`, celle de la **facture** `SupplierInvoiceNotCancellable { blocker:
DocumentLetteringInClosedPeriods }` ; rien n'est écrit, le groupe est intact, la facture reste `paid` ; les deux
prédicteurs de dépôt (`supplier_settlement_cancel_blocker`, `supplier_invoice_cancel_blocker`) rendent ce
motif ; après déverrouillage (borne avant la date la plus récente du groupe), les deux passent — avec
dissolution et paires `reversal`, ce que la 15-1a2-0 ne pouvait pas éprouver. *(Le code HTTP `409
LETTERING_ALL_LINES_IN_CLOSED_PERIODS` et les deux clés de texte sont éprouvés par l'API à la 15-1a2-0, AC5 —
le mappage ne dépend pas de la façon dont le groupe a été posé.)*

**AC8 (part ii)** — Test lexical (même fichier et mêmes outils que la part i) : chacun des trois
`UPDATE supplier_invoices … settlement_journal_entry_id` de production est dans une fonction dont le
corps appelle `sync_supplier_invoice_in_tx(` (après l'`UPDATE`, pour `pay_in_tx`) ou
`dissolve_supplier_invoice_document_group_in_tx(` (avant `reverse_owned_in_tx(`, pour les deux
annulations). Un site neuf rougit **en se nommant**.

**AC9 (part ii)** — `lettering_invariants` étendu : une ligne `document` d'une facture fournisseur est
sur l'achat **ou** le règlement en vigueur d'une facture **`paid`** — le statut vaut pour les deux
colonnes (`purchase_journal_entry_id`, `settlement_journal_entry_id`), **jamais** sur une facture `open` ni
`cancelled` (validation P3, R3-7 b) ; ces lignes ne sont ni `manual` ni `reversal`, **sauf** l'achat d'une facture
annulée (`reversal`, avec son miroir). Les **anciens règlements** — détaché par l'annulation de la facture,
ou annulé par `cancel_settlement_in_tx` — ne sont **jamais** `document` : libres, `manual` ou `reversal`
(validation P2, R-4 = F2-4 : un détaché contre-passé est `reversal` ; un règlement annulé l'est avec son
miroir, ou libre si `B` n'est plus lettrable). ⛔ **Comment le test les reconnaît** — plus aucune colonne
ne les rattache à leur facture (`supplier_invoices.rs:1092`, `:1301`) : par la **trace d'audit**, patron
`modification_guard` (`journal_entries.rs:1083-1090`) — `JSON_VALUE(details_json,
'$.settlementJournalEntryId')` des entrées `supplier_invoice.cancelled` et
`supplier_invoice.settlement_cancelled` (`supplier_invoices.rs:1113`, `:1325`). La réserve de faux positif
après restauration (C-15-8-25 point 2) ne joue pas dans la fixture, qui ne restaure rien. Aucune
exception de période : sans groupe gardé, aucun état orphelin (refus, P4).

**AC10 (part ii)** — Audit : `documentType = "supplierInvoice"` (C-15-1a2-22), `documentId`, `documentNumber` (clé présente, `null` sans numéro) ;
acteur : l'auteur du geste, `api_key_id: None` (écart nommé, comme la part i).

**AC11** — Outillage des migrations (finding R10 = F-13) :
- **P7** : M1 au registre (classe A, `include_str!` de la migration entière, commentaire de classe sur
  le patron des entrées existantes ; sa justification de classe A dit **ce qu'elle ne garantit pas**,
  P6) ; M2 à `EXEMPT_MIGRATIONS` (`Durable`, justification ci-dessus), compteur `16 → 17` ; des cinq tests
  qui ne parcouraient que le registre, **trois reprennent matière** — couverture des statements
  d'écriture, absence de DDL, littéraux dangereux —, **deux restent sans prise** sur M1 (validation P2,
  R-7) : `class_b_sentinel_column_is_added_by_its_own_migration` saute toute entrée qui n'est pas
  `Sentinels` (`post_restore.rs:1420-1423`), et l'extraction verbatim compare, pour un `include_str!` de la
  migration entière, la source à elle-même (`post_restore.rs:1220-1222` le dit). Le commentaire « VIDE
  depuis la Story 25-2-c » de `POST_RESTORE_BACKFILLS` (`:205`) est réécrit **avec ce décompte-là** ; la
**définition** de la classe A est réécrite aux trois sites qui la donnent (`post_restore.rs:41-43`, `:121`,
`post_restore_class_a.rs:358` — P6, F3-2) ;
- **P5** : deux lignes au tableau de `docs/migrations-idempotence-audit.md`, **à leur place
  chronologique**, verdict `yes` (gardes `lettering_key IS NULL`) ; compteurs **recomptés depuis le
  tableau** : en-tête `## Table d'audit (78 migrations)` et ligne `Total` (`… + 2 Story 15-1a2-ii`),
  `yes` **10** — **et les deux noms ajoutés à la liste nominative des `yes`** entre parenthèses
  (`docs/migrations-idempotence-audit.md:103`, huit noms aujourd'hui ; validation P2, R-10) —,
  `tracked-by-sqlx` **68**, `no` 0 — et `ls crates/kesh-db/migrations/*.sql | wc -l` = 78 =
  `grep -c '^| \`20' docs/migrations-idempotence-audit.md` ;
- **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et inspection de chaque site —
  `migrations_upgrade_path.rs` porte `assert_eq!(total, 76)` et la frontière `total - 42` (figée à 34) :
  `total` passe à **78**, l'écart à **44** ; les montages par version (`common::migrations_before`) ne
  bougent pas ;
- **squash** `crates/kesh-db/test-schema/` régénéré **pour le suivi `_sqlx_migrations`** — aucune
  différence de structure attendue (`test_schema_guard.rs` compare structure, `_kesh_version` et suivi) ;
- `crates/kesh-db/migrations.sha384` : deux lignes ;
- le nouveau fichier de test à fenêtre (`lettering_documents_backfill.rs`, vrai `MIGRATOR`) est inscrit
  à `ALLOWED_REAL_MIGRATOR_FILES` (`test_schema_guard.rs`) ;
- **P8** : `20261009000001` inchangée (`git diff` nul sur le fichier).

**AC12 (part ii)** — Documentation, par la valeur :
- `CHANGELOG.md` : « une facture fournisseur payée est lettrée avec son achat ; à la mise à
  jour, les pièces déjà soldées et les contre-passations déjà passées sont lettrées (sauf en période
  close) — sans entrée au journal d'audit ; une sauvegarde **antérieure** restaurée retrouve le lettrage de
  ses pièces, **pas** celui de ses contre-passations libres » (validation P2, R-11 : le coût de M2 au
  CHANGELOG, pas seulement au manuel d'administration). *(Le refus de l'annulation sous une période close —
  paiement et facture fournisseurs compris — a **une** entrée, celle de la 15-1a2-i AC18 : la clause est retirée
  d'ici, validation P4, F4-2, pour qu'il n'entre pas deux fois aux notes de version.)*
- `docs/api-external.md` : `document` couvre aussi les factures fournisseurs ; le message
  `LETTERING_IS_DOCUMENT` (réécrit par la 15-1a2-0) reste juste ; § de l'annulation d'une facture payée : le
  règlement détaché, une fois contre-passé, est lettré avec son miroir. *(Le refus sous une période close
  dans les deux listes en prose des annulations fournisseurs — après `FISCAL_YEAR_CLOSED`, avant
  `ACCOUNT_ARCHIVED` — est l'AC18 de la 15-1a2-i, C-15-1a2-24 ; validation P3, R3-6 = F3-5.)*
- `docs/manual/fr/user-manual.tex` : § des factures fournisseurs (paiement, `sec:annuler-facture-fournisseur`,
  `sec:paiements-fournisseurs`) — lettrage, règlement détaché lettrable à la main (le refus de l'annulation
  sous une période close est dans les listes de motifs, 15-1a2-i AC18 : y renvoyer, sans le redire) ; glossaire
  *Lettrage* élargi aux factures fournisseurs ; une note sur le rattrapage (période close, pas d'audit).
- `docs/manual/fr/admin-manual.tex` § « Reprises de données rejouées à l'import » (`:1722-1723`) : le
  lettrage des pièces est rejoué ; les paires de contre-passation libres **ne le sont pas** (motif, coût) ;
  et le rejeu **peut lettrer** une pièce restée ouverte que la base n'aurait plus lettrée d'elle-même — pièce
  historique dont l'exercice a été rouvert, compte devenu lettrable (P6) : la promesse « Si la sauvegarde
  contenait déjà l'information, elle fait foi » est nuancée pour ce cas, sans être démentie (aucune marque
  n'est jamais effacée ni réécrite).
- `make fr`, trois PDF, contrôle aplati (`pdftotext … | tr '\n' ' ' | tr -s ' '`).
- `README.md` « Feuille de route » (ligne v0.13.0) : vérifier qu'elle ne contredit pas l'état livré.

**AC16** — **Rejeu après restauration** (finding R3 ; `admin_full_import_e2e.rs`) : (a) une sauvegarde
**sans** lettrage (marques à `NULL` dans l'archive) importée → les groupes `document` et les paires
`reversal` d'achats annulés sont posés, `backfills_replayed` du détail d'audit nomme M1
`REPLAYED_UNCONDITIONAL` ; les autres paires `reversal` restent **ouvertes** ; (b) une paire `reversal`
**libre** délettrée à la main (`DELETE /letterings`), sauvegardée puis réimportée → **reste délettrée** ;
(c) une base à jour, **portant des pièces lettrées par les gestes** (au moins une facture client soldée,
une facture fournisseur payée, une facture fournisseur annulée — soit les trois étapes de M1),
**sans** les deux états de P6 (pièce historique rouverte, compte devenu lettrable), réimportée → marques
inchangées **et** `rows_affected == 0` pour l'entrée M1 du rapport de rejeu, **asserté** (validation P2,
R-5) — et, second lieu de la même assertion, `full_import_report_mirrors_the_production_registry` (modifié,
ci-dessous). Ce n'est pas une assertion de succès — la réserve de `post_restore.rs:190-193` ne s'y oppose pas — :
c'est le seul discriminant **de ce test-là**. sqlx pose `CLIENT_FOUND_ROWS` (`letterings.rs:736`) : un M1 privé de sa garde
`lettering_key IS NULL` **trouverait** les lignes déjà lettrées et compterait > 0 en les réécrivant à
l'identique, là où « marques inchangées » resterait vert ; (d) une base où une facture historique
entièrement close est restée **non lettrée** (abstention), dont un administrateur **rouvre** ensuite
l'exercice — la pièce reste ouverte en vivant —, exportée puis réimportée → M1 **lettre** la pièce (P6,
état 1). **Recette** (validation P3, F3-3 : un geste vivant lettre toujours, l'état ne se fabrique pas
autrement) : facture validée et réglée en période ouverte (le règlement la lettre) ; marques de la pièce
**effacées en SQL brut** ; exercice **clôturé** (ou borne posée après le règlement) ; puis
`fiscal_years::reopen` (ou `unlock_books`) — la pièce reste ouverte en vivant, rien ne la resynchronise ;
export, import : comportement attendu, asserté et nommé, pour que la justification de classe A ne puisse plus
affirmer un no-op strict.

⚠️ **`post_restore_class_a.rs` ne doit pas laisser M1 tourner à vide** (validation P2, R-5) : sa fixture est
faite pour `20260729000001` (facture validée **sans** règlement, `:317-375`), et
`class_a_entries_are_not_vacuous_on_a_pre_migration_base` additionne les lignes de toutes les entrées
(`:225-246`, `.sum()`) — M1 y serait vide sans que rien ne rougisse. La fixture gagne une facture soldée
(et une facture fournisseur payée), et le test asserte un compte **par entrée** : M1 > 0 sur la base
d'avant la migration, M1 == 0 sur la base à jour (`class_a_entries_are_no_ops_on_a_nominal_up_to_date_base`,
`:318`). **Recette** (validation P4, F4-5 ; le fichier fabrique tout en SQL brut, `insert_canonical_entry`,
`insert_invoice`) : la base **« à jour »** reçoit ses marques par un `UPDATE journal_entry_lines` **brut et
explicite** — le groupe que la synchronisation aurait posé (vente + règlement, achat + paiement ; clé = plus petite
ligne, origine `document`), suivi d'une assertion de montage (lignes trouvées) — et **jamais** en exécutant M1
elle-même, ce qui rendrait le test tautologique sur l'accord rattrapage ↔ vivant (que tient AC6, D4-ter) ; la base
**« d'avant »** porte les **mêmes** pièces, marques à `NULL` (l'état pré-migration explicite qu'exige déjà
l'en-tête du fichier, `:35-39`).

## Tasks

- [ ] **T0** — Relevés : faisabilité et `EXPLAIN` sur MariaDB 10.11 de la forme `UPDATE … JOIN (dérivée)`
      de M1/M2 sur une base de gate peuplée ; re-greper les ancres de P4 ; routes `Rejouee` :
      `POST /supplier-invoices/{id}/pay`, `…/cancel`, `…/settlement/cancel`, `POST /payment-batches/{id}/confirm`.
- [ ] **T1** (P3 part ii) — `sync_supplier_invoice_in_tx`, `dissolve_supplier_invoice_document_group_in_tx`,
      sur l'algorithme factorisé de la part i.
- [ ] **T2** (P4 part ii) — Les trois appels, exercices tenus gardés ; découverte par statut (P2). *(Le rang
      2 bis aux refus des deux annulations est livré par la 15-1a2-0, D3.)*
- [ ] **T3** (P6, AC11) — M1, M2, registre, exemption, squash, sha384, audit d'idempotence, P6, compteurs.
      Les en-têtes qui parlent de « la migration de rattrapage de la 15-1a2 » au singulier —
      `letterings.rs:15` et `letterings_lexical.rs:15` — nomment les deux fichiers, **et** l'exception R3
      de `letterings.rs` gagne le **rejeu à l'import** (`replay_post_restore_backfills`, entrée M1 de
      `POST_RESTORE_BACKFILLS`) : un écrivain de production de la marque, hors des deux primitives et sans
      audit, à chaque import (validation P2, F2-7). `post_restore_class_a.rs` : fixture et assertions par
      entrée (AC16 ; recette de la base « à jour » et de la base « d'avant » écrite sous AC16, F4-5). **Définition de la classe A** réécrite (P6 ; F3-2 = R3-2) : `post_restore.rs:41-43`
      (doc du module), `:121` (doc de `BackfillTrigger::Unconditional`), `post_restore_class_a.rs:358`
      (message d'échec « no-op STRICT » — réécrit pour dire aussi « base **nominale** : sans pièce rendue
      lettrable depuis », validation P4, F4-5) ; le **paragraphe** « ⚠️ Le montage qui rend ce test
      DISCRIMINANT, et le piège qu'il évite » de `class_a_entries_are_no_ops_on_a_nominal_up_to_date_base`
      (`post_restore_class_a.rs:300-312`) est **réécrit en entier**, titre compris — non un mot remplacé
      (validation P4, F4-4) : toute sa prémisse est fausse (« le **seul** montage qui teste quelque chose », le
      test « serait **muet** », « MariaDB ne compte … que les lignes réellement **modifiées** » et
      « rapporterait quand même `0` »). sqlx pose `CLIENT_FOUND_ROWS` (`sqlx-mysql-0.8.6`,
      `connection/stream.rs:46` ; `letterings.rs:736` s'y appuie) : `rows_affected` compte les lignes
      **trouvées**, un rejeu privé de sa garde réécrivant `3000` sur `3000` compterait donc la ligne
      (F3-8 = R3-3). Le paragraphe neuf dit : le montage `3200` ≠ `3000` reste **valide** — il distingue un
      rejeu qui écraserait une valeur posée —, mais n'est plus le seul discriminant, puisque `CLIENT_FOUND_ROWS`
      fait aussi compter une réécriture à l'identique ; il ne prétend plus qu'un montage `3000` serait muet. Les
      docs `:13`, `:21`, `:297` (« no-op sur une base nominale ») restent vraies sous la réserve « nominale » et
      ne sont pas réécrites. Contrôle
      par la **valeur** : `git grep -niE "no-op strict|no-op STRICT|réellement \*\*modifiées" -- crates docs`,
      chaque site trié (hors registre : `invoices.rs:5501`, `accounts.rs:2685`,
      `invoice_lines_revenue_account_backfill.rs:1151` parlent d'autre chose). `admin_full_import_e2e.rs` :
      `full_import_report_mirrors_the_production_registry` modifié (ci-dessous).
- [ ] **T4** (AC8 part ii) — Test lexical fournisseur.
- [ ] **T5** — Tests (liste ci-dessous) ; extension de la fixture partagée — dans
      `crates/kesh-db/tests/support/lettering_documents.rs` (15-1a2-i T5, C-15-1a2-28 : hors de `src/`, états
      hérités en SQL brut), incluse par `#[path]` dans `lettering_documents_backfill.rs`.
- [ ] **T6** (AC12 part ii) — CHANGELOG, `api-external.md`, manuels FR (utilisateur, administrateur) +
      `make fr` + PDF aplati, README.

**Tests prévus** (18 neufs, 5 modifiés) :
- `crates/kesh-db/tests/lettering_documents.rs` (fichier de la part i) — 10 neufs :
  `supplier_payment_letters_purchase_and_payment` (AC7), `batch_confirm_letters` (AC7),
  `supplier_settlement_cancel_dissolves_and_pairs` (AC7), `cancel_paid_supplier_invoice_detaches_an_open_payment` (AC7),
  `detached_payment_reversed_is_lettered_reversal` (AC7 second volet, AC9),
  `internal_account_payment_pairs_its_counterpart_on_cancel` (AC7), `payable_not_letterable_is_skipped` (AC7),
  `locked_period_supplier_cancels_are_refused_until_unlocked` (AC7 périodes closes — paiement et facture, sur un
  groupe posé par le paiement ; puis déverrouillage, dissolution et paires),
  `cancelled_supplier_invoice_sync_is_unchanged` (P2 découverte par statut, AC6 e),
  `supplier_audit_details_carry_the_invoice` (AC10 — `documentType = "supplierInvoice"`, et une facture **sans**
  numéro : `documentNumber` présent et nul) ;
- `crates/kesh-db/tests/lettering_documents_backfill.rs` (neuf, vrai `MIGRATOR`, montage par version) — 3 :
  `backfill_matches_live_sync` (AC6 d-i, e), `backfill_abstains_on_closed_and_locked_history` (AC6 d-ii),
  `backfill_pairs_reversals_by_position` (AC6) ;
- `crates/kesh-db/tests/letterings.rs` — `lettering_invariants` **étendu** (AC9 part ii) ;
- `crates/kesh-db/tests/letterings_lexical.rs` — 1 neuf : `supplier_settlement_writers_sync_or_dissolve` (AC8 part ii) ;
- `crates/kesh-api/tests/admin_full_import_e2e.rs` — 4 neufs : `full_import_replays_document_lettering` (AC16 a),
  `full_import_keeps_a_dissolved_free_reversal_pair` (AC16 b), `full_import_of_an_up_to_date_base_is_a_noop`
  (AC16 c, `rows_affected == 0` de M1), `full_import_letters_a_reopened_historical_piece` (AC16 d, recette
  écrite) ;
- `crates/kesh-api/tests/admin_full_import_e2e.rs` — `full_import_report_mirrors_the_production_registry`
  **modifié** (validation P3, R3-1 = F3-1) : sa boucle exige aujourd'hui `SKIPPED` pour **toute** entrée du
  registre (`:1572-1577`), **vacuement verte** tant que le registre est vide (`post_restore.rs:205`) ; M1, de
  classe A, y rend `REPLAYED_UNCONDITIONAL` à chaque import et la ferait rougir. L'issue attendue se **déduit**
  de `entry.trigger` — `BackfillTrigger::Unconditional` → `"REPLAYED_UNCONDITIONAL"`, `Sentinels(_)` →
  `"SKIPPED"` —, jamais écrite en dur ; pour M1, `rows_affected == 0` (la base du test : une facture validée
  non soldée) ; son commentaire (« l'entrée de classe B doit être SAUTÉE ») est réécrit. C'est la première
  fois depuis la 25-2-c que ce test exerce une entrée ;
- `crates/kesh-db/tests/post_restore_class_a.rs` — `class_a_entries_are_not_vacuous_on_a_pre_migration_base`
  et `class_a_entries_are_no_ops_on_a_nominal_up_to_date_base` **modifiés** (fixture avec règlements,
  comptes par entrée, commentaire « trouvées » ; AC16) ;
- `crates/kesh-db/tests/migrations_upgrade_path.rs` — `upgrade_path_preserves_data` **modifié** (total 78,
  écart 44 ; AC11 P6).

*(Recompte depuis cette liste : 10 + 3 + 1 + 4 = **18 fonctions de test neuves**, **5 tests modifiés** —
`lettering_invariants`, `full_import_report_mirrors_the_production_registry`, les deux de
`post_restore_class_a.rs`, `upgrade_path_preserves_data` —, plus trois tests unitaires de `post_restore.rs` qui
reprennent matière sans être écrits, AC11. Parti à la 15-1a2-0 : `supplier_settlement_cancel_e2e.rs::cancel_blocked_by_closed_lettering`
— le refus par l'API, éprouvé là sur un groupe posé à la main.)*

## Dev Notes

- **Gate `kesh-db` complet** (migrations, `post_restore.rs`, repositories : ciblage interdit) ; base
  remise à zéro avant ; **démarrage réel** contre une base persistante (E2E) — seul révélateur d'un
  défaut de checksum (P8).
- **Verrous** : ceux de la part i ; l'ordre facture fournisseur `FOR UPDATE` → exercice → lignes est
  celui des trois gestes. `confirm_batch` règle N factures dans une transaction : N synchronisations,
  chacune après son `UPDATE` ; interblocage → rejeu de la route (`Rejouee`).
- **Tests existants touchés** (F-10, non exécutés) : `supplier_invoices_repository.rs:2511`
  (`supplier_invoice_cancel_letters_a_pair_that_cannot_be_dissolved_by_hand`, facture **non payée**) —
  inchangé ; `payment_batches_repository.rs` — les paiements de lot portent désormais des marques ;
  `admin_full_import_e2e.rs:1546` (`full_import_report_mirrors_the_production_registry`) — la liste des
  versions (`:1556`) suit le registre, mais l'assertion d'issue (`:1572-1577`, `SKIPPED` pour tout) rougirait :
  **modifié** (R3-1 = F3-1, ci-dessus) ;
  `post_restore_class_a.rs` (rejoue registre + retirés sur base à jour) — **modifié** : M1 ne doit pas y
  tourner à vide (AC16) ; `supplier_settlement_cancel_e2e.rs` et les tests des prédicteurs fournisseurs —
  relèvent de la 15-1a2-0 (le rang y est ajouté).
- **Ordre avec d'autres migrations** (validation P3, R3-5) : `registry_entries_are_within_import_window`
  exige qu'aucune table applicative ne soit créée **après** M1. Toute story de l'epic 15 qui crée une table
  avant le tag v0.13.0 (la 15-2, pièces justificatives, y est candidate — sa fiche n'existe pas encore ; la 15-4,
  citée en P3, est **sortie de l'epic**, `sprint-status.yaml` ; validation P4, F4-3) fera sortir M1 de la fenêtre
  d'importabilité : M1 devra alors être
  exemptée « Hors fenêtre », et AC16 (a), (d) perdront leur objet. Ce rouge est **attendu** — à traiter par la
  story qui crée la table, pas à découvrir.
- **Verrous du rang 2 bis** : aucun neuf — lecture sans verrou des lignes du groupe, des exercices et de
  la borne, après les verrous que les deux annulations prennent déjà (tolérance nommée en 15-1a2-i P7
  point 2).
- **Règle de découpage** — comptée aux **deux** grains (C-15-1a2-21) : au grain « crates Rust, packages npm » :
  `kesh-db` = **1** (les tests de `kesh-api` ne sont pas un module de production) ; au grain des modules métier
  de premier niveau : `kesh-db/repositories/letterings` (synchronisation fournisseur), `…/supplier_invoices`
  (trois appels ; `payment_batches.rs` **inchangé**, couvert par `pay_in_tx`), `kesh-db/migrations` (M1, M2,
  squash, sha384), `kesh-db/post_restore` (registre, exemption, définition de la classe A) = **4**, sous le
  seuil. La documentation (CHANGELOG, `api-external.md`, deux manuels, README, audit d'idempotence) est
  déclarée à part. L'**écran**, les **textes** et les **refus** du rang 2 bis — fournisseurs compris — sont à
  la 15-1a2-0 : aucun fichier `frontend/` ni `.ftl` n'est touché ici.
- **Dépendances** : la 15-1a2-0 et la 15-1a2-i **mergées** (dures).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

### Validation P4 ciblée — 2026-10-09 (une lentille, chasseur de régressions de `76e7893a` ; remédiation Opus 5.5, seul remédiateur des fiches de la suite du lettrage, en autonomie) — VALIDATION CLOSE

**Rapport** : `kesh-gate-logs/15-1a2-ii-validate-p4-ciblee.md` (**0 CRITICAL, 0 HIGH, 0 MEDIUM, 6 LOW**), prompt
versionné `15-1a2-ii-validate-prompt-p4-ciblee.md`, axes 1 à 5 déclarés exercés. **Trend** : P1 (fiche mère)
**3 HIGH / 7 MEDIUM** (R), **2 HIGH / 7 MEDIUM** (F) → P2 **1 HIGH / 6 MEDIUM distincts** → P3 **0 HIGH / 2 MEDIUM
distincts** → P4 ciblée **0 au-dessus de LOW** : la boucle de validation est **close** (CLAUDE.md, « uniquement
des findings LOW »). Les six LOW appliqués, chacun relu au code (`056997b0`) :

| finding | verdict | où |
|---|---|---|
| F4-1 — `LetteringConcurrentChange` ne répond pas `500` | **corrigé** : `Invariant` → `500`, `LetteringConcurrentChange` → `409 LETTERING_CONCURRENT_CHANGE` (vérifié `kesh-api/src/errors.rs:3062`, `:3720`) ; lot annulé dans les deux cas ; appel cité `payment_batches.rs:457` | P3 part ii |
| F4-2 — le refus entrerait deux fois au CHANGELOG | **corrigé** : clause retirée d'ici ; **une** entrée, celle de la 15-1a2-i AC18 (où la documentation du refus est désormais, C-15-1a2-24) | AC12 |
| F4-3 — l'exemple « la 15-4 » est hors de l'epic | **corrigé** (« toute story de l'epic 15 qui crée une table, la 15-2 y est candidate ») | Dev Notes |
| F4-4 — correction prescrite comme un mot remplacé | **corrigé** : le **paragraphe** `post_restore_class_a.rs:300-312` réécrit en entier, titre compris | T3 |
| F4-5 — l'état « à jour » de M1 sans recette | **corrigé** : marques par `UPDATE` brut explicite (jamais par M1 elle-même), base « d'avant » = mêmes pièces sans marques ; message `:358` borné à la base « nominale » | AC16, T3 |
| F4-6 — AC7 affirme un `409` et des clés que son test de dépôt n'éprouve pas | **corrigé** : AC7 énoncé au dépôt (variantes, prédicteurs) ; code et clés renvoyés à la 15-1a2-0 AC5 | AC7 |

**Reçu de la validation P4 de la 15-1a2-i** (même remédiation) : AC6 (e) aligné sur l'étape 3 **terminale**
(C-15-1a2-29 — un groupe survivant de C104 rend `AccountNotLetterable` sans écriture, la fixture porte le cas) ;
la fixture partagée sort de `kesh_db::test_fixtures` pour `tests/support/` (C-15-1a2-28) ; les renvois
« 15-1a2-0 AC8 » deviennent « 15-1a2-i AC18 » (C-15-1a2-24). Aucun de ces reçus ne change une règle de cette
fiche. **Recompte** (depuis ce fichier) : **8 critères**, **7 tâches** (T0–T6), **18 tests neufs + 5 modifiés** —
inchangés. Prochaine étape : développement, après la 15-1a2-0 et la 15-1a2-i.

### Validation P3 — 2026-10-09 (Sonnet 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, seul remédiateur des fiches de la suite du lettrage, en autonomie)

**Rapports** : `kesh-gate-logs/15-1a2-ii-validate-p3-R.md` (**0 CRITICAL, 0 HIGH, 1 MEDIUM, 6 LOW**) et
`…-F.md` (**0 CRITICAL, 0 HIGH, 2 MEDIUM, 6 LOW**). Recoupements : R3-1 = F3-1, R3-2 = F3-2 (LOW chez R, MEDIUM
chez F : compté MEDIUM), R3-3 = F3-8, R3-4 = F3-6, R3-6 = F3-5 → **2 MEDIUM distincts** et **8 LOW distincts**.
**Trend** : P2 **1 HIGH / 6 MEDIUM distincts** → P3 **0 HIGH / 2 MEDIUM distincts**. Signal D5 : **non levé**
(sévérité en baisse) ; R3-1 est d'**origine** (P1), F3-2 un **résidu de propagation** du patch P2 sur F2-5 — sur
la fiche, pas sur le code qu'elle prescrit de modifier. Chaque finding relu au code (`056997b0`).

| finding | sévérité | verdict | où |
|---|---|---|---|
| R3-1 = F3-1 — `full_import_report_mirrors_the_production_registry` exige `SKIPPED` pour toute entrée ; M1 (classe A) le fera rougir | MEDIUM | **corrigé** : test **modifié** (5 modifiés, non 4) — issue déduite de `entry.trigger`, `rows_affected == 0` pour M1, commentaire réécrit ; vérifié au code (`admin_full_import_e2e.rs:1546-1578`, registre vide `post_restore.rs:205`) | Tests, Dev Notes, AC16 (c) |
| F3-2 = R3-2 — la définition « no-op strict » de la classe A reste dans le code | MEDIUM | **corrigé** : la story **réécrit** `post_restore.rs:41-43`, `:121`, `post_restore_class_a.rs:358` ; grep de la valeur, sites hors registre triés | P6, AC11, T3 |
| R3-3 = F3-8 — commentaire faux « modifiées » (`post_restore_class_a.rs:309-312`) | LOW | **corrigé** (« trouvées », `CLIENT_FOUND_ROWS`) | T3 |
| R3-4 = F3-6 — `DocumentRef` non défini, `documentNumber` nul | LOW | **corrigé** : défini par la 15-1a2-i (C-15-1a2-22) ; `documentType = "supplierInvoice"` | P3, AC10, tests |
| R3-5 — version de M1 au jour de `20261009000001` ; dépendance avec la 15-4 | LOW | **corrigé** (strictement supérieure ; rouge attendu nommé) | P6, Dev Notes |
| R3-6 = F3-5 — les « tableaux » fournisseurs sont des listes en prose, sans `MATCHED_BANK_TRANSACTION` | LOW | **corrigé, porté à la 15-1a2-0** (AC8 (b) : après `FISCAL_YEAR_CLOSED`, avant `ACCOUNT_ARCHIVED` ; rang 3 absent = écart préexistant, signalé) | AC12, 15-1a2-0 |
| R3-7 — exercice tenu d'AC6 (e) ; statut `paid` pour les deux colonnes d'AC9 | LOW | **corrigé** | AC6, AC9 |
| F3-3 — AC16 (d) non constructible tel qu'écrit | LOW | **corrigé** (recette écrite : marques effacées, clôture, réouverture) | AC16 |
| F3-4 — AC6 classé par l'extracteur ; effacement trop large | LOW | **corrigé** (classement par construction de la fixture ; effacement borné à `document` et `reversal`) | AC6 |
| F3-7 — erreur structurelle de la synchronisation dans `pay_in_tx` / `confirm_batch` | LOW | **corrigé** (comportement écrit : propagation, lot annulé, `500`) | P3 |
| LOW non numéroté de F — `user-manual.tex:2337` | LOW | **corrigé, porté à la 15-1a2-0** (AC8) | 15-1a2-0 |

**Découpage effectif** (décision de l'orchestrateur, C-15-1a2-19) : les refus du rang 2 bis des deux annulations
fournisseurs (ancienne T2, part de P4) et le test d'API `supplier_settlement_cancel_e2e.rs::cancel_blocked_by_closed_lettering`
sont partis à la **15-1a2-0**, qui les éprouve sur des groupes posés à la main ; AC7 garde l'**intégration** sur un
groupe posé par le paiement. **Propagation** : « P7 point 2 », `"supplier_invoice"`, « no-op strict », « 19 tests »,
« 4 modifiés », `000001_lettering` grepés sur les fiches, l'index, le registre et `crates/` (sites du code nommés en
T3). **Recompte** (depuis ce fichier) : **8 critères**, **7 tâches** (T0–T6), **18 tests neufs + 5 modifiés**.
Choix : C-15-1a2-19, 21, 22 (partagés). Prochaine passe : **P4** — **ciblée** possible (une lentille, braquée sur
ce commit) : la remédiation ne change aucune règle métier de la fiche (elle retire un périmètre, fixe des tests et
des textes) ; si l'orchestrateur la juge trop large du fait du découpage, complète (Opus).

### Validation P2 — 2026-10-09 (Opus 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, seul remédiateur des trois fiches, en autonomie)

**Rapports** : `kesh-gate-logs/15-1a2-ii-validate-p2-R.md` (**0 CRITICAL, 0 HIGH, 6 MEDIUM, 5 LOW**) et
`…-F.md` (**0 CRITICAL, 1 HIGH, 4 MEDIUM, 4 LOW**). Doublons : R-1 = F2-1, R-2 = F2-2, R-3 = F2-3,
R-4 = F2-4, R-8 = F2-6 → **1 HIGH et 6 MEDIUM distincts**, tous **nés de la remédiation P1**. **Trend** (la
passe 1 portait sur la fiche mère) : P1 **3 HIGH / 7 MEDIUM** (R) et **2 HIGH / 7 MEDIUM** (F) → P2 **1 HIGH
/ 6 MEDIUM distincts**. ⚠️ **Signal D5 levé** (amendement du 2026-10-06) : la sévérité ne baisse pas d'un
cran (HIGH → HIGH) **et** les défauts naissent du correctif précédent — c'est le **recyclage** qui
découpe. **Déclaré à l'orchestrateur, non découpé** : six des sept défauts MEDIUM+ ont **une** cause,
C-15-1a2-2, que la décision de l'orchestrateur remplace **à la racine** (refus, C-15-1a2-10) au lieu de la
rapiécer ; la fiche est elle-même le produit d'un découpage, compte cinq modules et n'en gagne aucun (les
textes et l'écran du refus sont à la 15-1a2-i). Si la P3 trouve encore un défaut né de cette remédiation,
le découpage naturel est **rattrapage** (P6, AC6, AC11, AC16) / **fournisseurs** (P2–P4, AC7–AC10).
Chaque finding relu au code (`056997b0`).

| finding | sévérité | verdict | où |
|---|---|---|---|
| F2-1 = R-1 — l'annulation d'une facture payée sous la borne laisse un groupe `document` orphelin, indissoluble | HIGH / MEDIUM | **corrigé** : refusée (rang 2 bis), comme l'annulation du paiement ; « aucun groupe orphelin » écrit | P4, AC7 |
| F2-2 = R-2 — AC6 (d) faux par construction (périodes d'aujourd'hui contre périodes du geste) | MEDIUM | **corrigé** : deux régimes classés au moment du rattrapage, différence attendue assertée pièce par pièce ; la seconde divergence (groupe gardé) disparaît (C-15-1a2-16) | AC6 |
| F2-3 = R-3 — découverte non définie hors `paid` ; `Invariant` sur l'achat `reversal` d'une facture annulée | MEDIUM | **corrigé** : découverte par statut, `cancelled` → vide (C-15-1a2-15) | P2, AC6 (e) |
| F2-4 = R-4 — AC9 part ii : règlement détaché contre-passé `reversal` omis ; reconnaissance non dite | MEDIUM | **corrigé** : anciens règlements jamais `document`, libres/`manual`/`reversal` ; reconnus par la trace d'audit (patron `modification_guard`) | AC9, AC7 |
| R-5 — « M1 no-op sur base à jour » sans test discriminant | MEDIUM | **corrigé** : `rows_affected == 0` de M1 asserté sur une base à jour portant des pièces lettrées ; `post_restore_class_a.rs` compte par entrée | AC16 (c), T3 |
| R-6 — après déverrouillage, `Invariant` au paiement suivant | MEDIUM | **corrigé à la racine** (plus de groupe gardé ; 15-1a2-i P3) | P4 |
| F2-5 — la classe A de M1 n'est pas un « no-op strict » | MEDIUM | **corrigé** : justification réécrite honnêtement — le rejeu peut lettrer ce que la base avait laissé ouvert à bon droit (exercice rouvert, compte devenu lettrable), jamais délettrer ni réécrire ; AC16 (d) et manuel d'administration | P6, AC12, AC16 |
| R-7 — « les cinq tests reprennent matière » | LOW | **corrigé** : trois sur cinq, et pourquoi | AC11 |
| R-8 = F2-6 — `documentNumber` d'une facture sans numéro | LOW | **corrigé** : `null` (C-15-1a2-17) | P3 part ii, tests |
| R-9 — « facture ouverte annulée : no-op » faux après un groupe gardé | LOW | **corrigé** (vrai de nouveau ; motif écrit) | P4 |
| R-10 — liste nominative des `yes` | LOW | **corrigé** | AC11 |
| R-11 — « après toute restauration » déborde M2 | LOW | **corrigé** (Story bornée, coût au CHANGELOG) | Story, AC12 |
| F2-7 — l'exception R3 omet le rejeu à l'import | LOW | **corrigé** | T3 |
| F2-8 — recouvrement M1 étape 3 / M2 | LOW | **corrigé** (exclusion explicite dans M2) | P6 |
| F2-9 — le compte de modules ne dit pas ce qu'il exclut | LOW | **corrigé** | Dev Notes |

**Décisions de l'orchestrateur appliquées** : refus au délettrage (C-15-1a2-10, historique écrit en
15-1a2-i P7) ; découverte par statut ; reconnaissance par l'audit ; AC6 en deux régimes ; justification de
classe A honnête. **Propagation** : « telle quelle … groupe gardé », « no-op strict », « libre ou `manual` »,
« après toute restauration », « les cinq tests » grepés sur les trois fiches et l'index. Choix consignés :
**C-15-1a2-15, 16, 17** (et 10, 12 partagés avec la 15-1a2-i). **Recompte** (depuis ce fichier) : **8
critères** (AC6, AC7, AC8–AC10 parts ii, AC11, AC12 part ii, AC16), **7 tâches** (T0–T6), **19 tests neufs +
4 modifiés**. Prochaine passe : **P3, Sonnet**, complète.

### Validation P1 — 2026-10-09 (Sonnet 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, en autonomie)

Fiche **née** de cette remédiation : découpage de la 15-1a2 (finding F-9 ; C-15-1a2-1). Findings de la
passe : **R** 3 HIGH, 7 MEDIUM, 4 LOW ; **F** 2 HIGH, 7 MEDIUM, 3 LOW — bilan par finding dans l'index.
Portés **ici** : R3/F-1 (classe du rattrapage : deux migrations, M1 classe A, M2 exemptée — P6, AC16),
R5/F-2 part ii (compte non lettrable : `B` et SQL — P3, P6, AC7), R2/F-6 part ii (abstention au rattrapage
— P6, AC6), R4 (Reçus 1, 3, 7, 10, 13 intégrés à P6), R10/F-13 (outillage : registre, exemption,
squash pour le suivi, `ALLOWED_REAL_MIGRATOR_FILES`, compteurs P5 76 → **78**, P6 — AC11), R11 part ii
(lexical fournisseur — AC8), F-10 part ii (`confirm_batch`, compte interne, tests existants — AC7, Dev
Notes), R9/F-7 part ii (manuels, rattrapage — AC12). Choix consignés : C-15-1a2-1 à -4 (registre).
**8 critères** (AC6, AC7, AC8–AC10 parts ii, AC11, AC12 part ii, AC16), **7 tâches** (T0–T6), **14 tests neufs + 2 modifiés** —
recomptés depuis ce fichier. Prochaine passe : P2, Opus, complète.
