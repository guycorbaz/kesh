# Story 15.1a2 : Le lettrage des pièces — une facture soldée est lettrée, et seulement elle

## Status

ready-for-dev *(créée le 2026-10-08 à la reprise du lettrage, à valider — `bmad-create-story validate` avant tout développement)*

## Story

**As a** indépendant, PME ou fiduciaire qui encaisse ses factures et paie ses fournisseurs dans Kesh,
**I want** que les lignes d'une facture soldée — la créance, ses règlements, son solde, son avoir —
soient lettrées entre elles sans que j'aie à le refaire, et délettrées si un règlement est annulé,
**so that** le grand livre dise ce que disent la fiche facture, la balance âgée et les relances :
une facture est soldée **si et seulement si** son reste dû est nul.

Deuxième des quatre sous-stories du lettrage (#518). **Suppose la 15-1a livrée** (colonnes,
primitive `create_group_in_tx` / `dissolve_group_in_tx`, origine `document`) — **découpée le 2026-10-09
(C124) en 15-1a-i** (la marque : colonnes, primitive, routes) **et 15-1a-ii** (les gardes : gel
`ENTRY_LETTERED`, contre-passation qui lettre, R6) ; cette fiche suppose **les deux**. Les renvois
« 15-1a Rn / ACn » gardent leur numéro, porté par la sous-fiche que donne la table de
`15-1a-socle-lettrage.md` (index). Ordre : 15-1a-i → 15-1a-ii → **15-1a2** → 15-1b → 15-1c.

## Pourquoi cette story existe

`invoice_settlements` (Story 24-2) est **déjà** un lettrage : chaque ligne y rattache une
écriture de règlement à **une** facture, et c'est l'utilisateur qui l'a décidé — en saisissant le
règlement sur la facture, ou en acceptant une proposition de rapprochement. La migration qui a
créé la table le dit : *« Cette table est aussi le SUBSTRAT DU LETTRAGE »*
(`20260827000001_invoice_settlements.sql:15-18`). Côté fournisseur, `supplier_invoices` porte
l'écriture d'achat **et** celle du règlement (`purchase_journal_entry_id`,
`settlement_journal_entry_id`).

Faire relettrer à la main ce que l'utilisateur a déjà apparié serait une double saisie ; laisser
le grand livre muet sur ce qui est soldé ferait **mentir la vue des postes ouverts** (15-1b) sur
le cas le plus fréquent. Ce n'est **pas** un appariement automatique au sens du `CLAUDE.md` :
Kesh ne rapproche aucune donnée entrante, il **recopie au grand livre un rattachement que
l'utilisateur a déclaré** (C93).

## Le modèle réel — relevé sur `origin/main` (`9cb5083b`)

**Client.** Le compte de créance `A` d'une facture est celui de **la première ligne au débit de
son écriture de vente** — c'est la lecture que font déjà le règlement et le rapprochement
(`invoice_settlements_write.rs:104-115`, `routes/reconciliation.rs:1475-1502`), et non le réglage
du moment.

| pièce | lignes sur `A` | source |
|---|---|---|
| écriture de vente | **un** débit = TTC arrondi (l'arrondi à 5 centimes est porté par cette ligne, `invoices.rs:1898`) | `generate_invoice_journal_lines(_rounded)`, `invoices.rs:1789/1884` |
| chaque règlement (`bank_transfer`, `internal_account`) | **un** crédit = montant **réglé** (brut ; l'écart d'arrondi éventuel va sur une 3ᵉ ligne, hors `A`) | `settlement_journal_lines`, `invoice_settlements.rs:198-240` |
| solde du reste (`write_off`) | **un** crédit = reste exact | `write_off_journal_lines`, `invoice_settlements.rs:259-340` |
| avoir (total, unique, refusé s'il existe un règlement) | **un** crédit = TTC + arrondi de la facture | `generate_credit_note_journal_lines`, `credit_notes.rs:187-257`, `:523-546` |

⚠️ **L'avoir crédite aujourd'hui le compte des RÉGLAGES** (`credit_notes.rs:362-364`), pas `A` —
défaut **#473**, corrigé par la **15-6a**. Tant qu'elle n'est pas mergée, un avoir émis après un
changement de réglage tombe sur un autre compte : le groupe n'est alors **pas formé** (AC1 ne
regarde que `A`), la facture et l'avoir restent ouverts chacun sur leur compte — ce qui est la
vérité du grand livre. **Ordre recommandé : 15-6a avant 15-1a2.**

⚠️ **#474** (15-6b) : un règlement « compte interne » peut aujourd'hui viser `A` lui-même — son
écriture porte alors un débit **et** un crédit sur `A`. Le groupe ne se forme pas (la somme ne
s'annule pas) ; la 15-6b ferme la porte.

**Écrivains de `invoice_settlements`** — inventaire **fermé**, relevé par
`grep -rn "INTO invoice_settlements\|DELETE FROM invoice_settlements" crates/ --include=*.rs`
hors tests : **un** `INSERT` (`invoice_settlements::create_in_tx`, `invoice_settlements.rs:397`,
appelé par `settle_invoice` `:250`, `write_off_invoice` `:540` et `accept_one_invoice`
`routes/reconciliation.rs:1645`) et **un** `DELETE` (`cancel_settlement_in_tx`,
`invoice_settlements_write.rs:783`, aussi appelé par le dé-rapprochement
`reconciliation_cancel.rs:354`). La FK `ON DELETE CASCADE` depuis `invoices` ne joue que sur un
brouillon (`invoices::delete` refuse une facture validée, `invoices.rs:1384-1387`).

**Fournisseur.** Compte fournisseurs `B` = celui de **la première ligne au crédit de l'écriture
d'achat** (lecture déjà faite par `pay_in_tx`, `supplier_invoices.rs:657-668`). Achat : **un**
crédit sur `B` = TTC ; règlement : **un** débit sur `B` = TTC, **toujours complet** (pas de
paiement partiel fournisseur, une seule colonne de règlement). Écrivains : `pay_in_tx` (`:629`,
appelé par `pay` et par `payment_batches::confirm_batch`, `payment_batches.rs:359-371`),
`cancel_in_tx` (`:935` — contre-passe l'achat ; une facture **payée** voit son règlement
**détaché**, `:992-996`), `cancel_settlement_in_tx` (`:1132` — contre-passe le règlement).

## Décisions

### P1 — Le groupe `document` d'une facture client

Pour une facture `I` (statut `validated` ou `cancelled` par avoir) dont l'écriture de vente
existe : `C(I)` = les lignes **sur `A`** de l'écriture de vente, des écritures de **tous** ses
règlements en vigueur (lignes de `invoice_settlements`, soldes compris) et de l'écriture de son
**avoir** émis. **Le groupe existe si et seulement si** `|C(I)| ≥ 2` **et** `Σ(débit − crédit)`
sur `C(I)` **= 0**. Il porte l'origine `document`.

⚠️ **Le critère est la somme au grand livre, pas le reste dû** — le reste dû est **dérivé** des
pièces (`INVOICE_AMOUNT_DUE_DERIVED_SQL`, `invoice_settlements.rs:120`) et peut dire 0 alors que
les lignes ne s'annulent pas sur `A` (avoir sur un autre compte, #473). Lettrer sur le reste dû
violerait la règle du groupe (somme nulle, 15-1a R3). AC5 vérifie que les deux **s'accordent**
sur tous les cas normaux, et nomme ceux où ils divergent.

⚠️ **Données héritées** : une facture marquée `paid_at` **sans** ligne de règlement (antérieure à
la 24-2, `invoices.rs:1524-1528`) a `|C(I)| = 1` → **pas de groupe** ; sa créance reste ouverte
au grand livre, ce qui est **vrai** (aucune écriture d'encaissement n'existe). La 15-1b l'affiche
avec ce motif.

### P2 — Le groupe `document` d'une facture fournisseur

Pour une facture fournisseur **payée** (`settlement_journal_entry_id` non nul) : `C(S)` = les
lignes **sur `B`** de l'achat et du règlement. Même règle (≥ 2 lignes, somme nulle).

### P3 — Une fonction de synchronisation par sorte de pièce, idempotente

`letterings::sync_invoice_in_tx(tx, company_id, invoice_id, actor)` et
`letterings::sync_supplier_invoice_in_tx(tx, company_id, supplier_invoice_id, actor)` :

1. calculent `C` (P1/P2) ;
2. trouvent le groupe `document` existant qui contient une ligne de `C` ou la créance de la
   pièce ;
3. si `C` qualifie et égale ce groupe → **rien** ; sinon dissolvent ce groupe
   (`dissolve_group_in_tx`, mode `System`) puis, si `C` qualifie, créent le groupe
   (`create_group_in_tx`, origine `document`).

⛔ **Une ligne de `C` déjà lettrée hors `document`** (manuel ou contre-passation) est un état
**impossible** (une ligne de pièce n'est pas lettrable à la main, 15-1a R5 ; une ligne de
règlement n'est contre-passée qu'après le retrait de sa ligne `invoice_settlements`) →
`DbError::Invariant`, jamais un écrasement.

### P4 — Où la synchronisation est appelée — inventaire FERMÉ des écrivains

| écrivain | appel | moment |
|---|---|---|
| `invoice_settlements::create_in_tx` (`:397`) — **couvre** `settle_invoice`, `write_off_invoice`, `accept_one_invoice` | `sync_invoice_in_tx` | après l'`INSERT` |
| `invoice_settlements_write::cancel_settlement_in_tx` (`:702`) — **couvre** le dé-rapprochement (`reconciliation_cancel.rs:354`) | ① dissolution du groupe `document` de la facture, **avant** `reverse_owned_in_tx` (`:773`) ; ② `sync_invoice_in_tx` après le `DELETE` (`:783`) | ① **avant** le socle — sinon la ligne de règlement, encore lettrée, garderait son groupe et son miroir resterait ouvert (15-1a R6, seconde branche) |
| `credit_notes::create_credit_note` (`:261`) | `sync_invoice_in_tx` | après l'écriture d'avoir (`:548`) |
| `supplier_invoices::pay_in_tx` (`:629`) — **couvre** `pay` et `confirm_batch` | `sync_supplier_invoice_in_tx` | après l'`UPDATE … status='paid'` (`:797-801`) |
| `supplier_invoices::cancel_settlement_in_tx` (`:1132`) | dissolution **avant** la contre-passation (`:1188`) | idem ① |
| `supplier_invoices::cancel_in_tx` (`:935`), facture **payée** | dissolution **avant** la contre-passation de l'achat (`:979-988`) | le règlement détaché reste **ouvert** : c'est un paiement sans facture, lettrable à la main (15-1a R5) |

⛔ **La contre-passation lettre ensuite ce qui est libre** (15-1a R6) : règlement annulé ↔ son
miroir ; achat annulé ↔ son miroir. La créance (ou la dette) **redevient ouverte**.

⚠️ **Le passage par `create_in_tx` est ce qui ferme l'inventaire** : un futur créateur de
règlement qui passerait par lui serait couvert d'office ; un `INSERT INTO invoice_settlements`
écrit ailleurs ne le serait pas — d'où le test lexical d'AC8.

### P5 — Les dissolutions système ne butent jamais sur un exercice clos — et c'est vérifié

Toute dissolution de P4 accompagne l'annulation d'une écriture **sur exercice ouvert** : la queue
commune des annulations refuse un exercice clos au rang 2 (`settlement_cancellation.rs:60` ;
`cancel_settlement_in_tx` client, et les deux annulations fournisseur la partagent). Le groupe
contient donc au moins une ligne sur exercice ouvert, condition du mode `System` (15-1a AC5).
⚠️ **L'ordre compte** : la dissolution est appelée **après** les gardes d'annulation (sinon un
refus d'annulation laisserait… rien, la transaction étant annulée — mais un `Invariant` levé
avant le bon refus masquerait la vraie cause).

### P6 — Rattrapage des données existantes : une migration, classe A, rejouée après restauration

Les pièces déjà soldées et les contre-passations déjà passées avant cette story doivent être
lettrées. ⛔ **Une migration qui écrit des données** → triage **P7** obligatoire.

- **Contenu** : (1) les groupes `document` client (P1) ; (2) les groupes `document` fournisseur
  (P2) ; (3) les paires `reversal` (15-1a R6, première branche) pour les lignes **encore libres**
  après (1) et (2) — appariées à leur miroir par `reverses_entry_id` et `line_order`, sur compte
  lettrable. Chaque `UPDATE` pose `lettering_key = MIN(id)` du groupe — calculable en SQL **sans
  compteur**, ce que la clé de la 15-1a rend possible.
- **Classe A** (rejeu inconditionnel) : **chaque** statement est gardé par
  `lettering_key IS NULL` sur **toutes** les lignes du groupe candidat (aucune valeur posée ne
  peut être écrasée). Inscription au registre `POST_RESTORE_BACKFILLS` (`post_restore.rs:204`,
  **vide** à ce jour — ce sera sa première entrée) ; `registry_entries_are_within_import_window`
  est satisfait (aucune table créée depuis). Une sauvegarde antérieure à la 15-1a, restaurée,
  se voit ainsi relettrée.
- **Pas d'audit** pour le rattrapage (une migration n'a pas d'acteur) ; le dire dans le manuel.
- ⛔ **Deux implémentations de la même règle** (SQL de rattrapage, Rust de P3) : c'est la
  duplication que le `CLAUDE.md` interdit, acceptée **parce que P8 fige la migration** — et
  **tenue par un test d'accord** (AC6), sans lequel elle ne serait pas acceptable.

## Reçu de la 15-1a — validation P1 du socle (2026-10-08)

*Section ajoutée par la remédiation de la validation P1 de la 15-1a (registre C101 à C106). Elle
ne réécrit pas cette fiche : elle liste ce que le socle a changé et que **cette** story doit
intégrer à sa propre validation.*

1. **`kesh_version_min_required` est déjà relevé à `'0.13.0'` par la migration de la 15-1a**
   (C101) — motif : une v0.12.1 relancée sur une base où **cette** story a posé des groupes
   `document` annulerait un règlement sans dissoudre le groupe (facture due, grand livre
   « soldée »). La migration de rattrapage (P6) n'a **pas** à relever de nouveau ; les crates sont
   déjà en `0.13.0`. À écrire dans le risque de P6.
2. **Mode `System { held_open_fiscal_year_id }`** (15-1a R7, C103) : la primitive ne prend
   **aucun** verrou d'exercice en mode `System` ; l'appelant tient, `FOR UPDATE`, un exercice
   **ouvert** couvrant une ligne du groupe et le passe. Pour chaque site de P4, nommer cet
   exercice et le moment où il est verrouillé : pour une **création** après un règlement, celui de
   l'écriture de règlement ; pour une **dissolution avant** contre-passation, celui de l'écriture
   annulée (P5 le dit ouvert — le **verrouiller** au lieu de seulement le lire, dans le sens
   `start_date` croissant avant l'exercice du jour que prendra `reverse_owned_in_tx`).
3. **Règle symétrique des exercices** (C105) : un groupe entièrement sur des exercices clos ne se
   crée ni ne se défait en mode `Manual`. ⚠️ **La migration de rattrapage (P6) n'est pas soumise
   à la primitive** et poserait des groupes historiques entièrement clos — décider ici si elle
   s'en abstient (cohérence avec la vue « au » de la 15-1b, qui ne connaît pas la date du
   lettrage) ou si elle les pose (l'historique était soldé ; aucune vue ne l'a montré ouvert
   avant la 15-1b). Recommandation du socle : s'en **abstenir**, par symétrie.
4. **Lettrabilité à la création seule** (C104) : un groupe `document` survit au retypage ou au
   rattachement bancaire de son compte ; `sync_*` ne doit pas le dissoudre pour ce motif.
5. **Groupe `reversal` contenant une ligne de pièce** (C106) : non dissoluble à la main (409
   `LETTERING_LINE_OWNED_BY_DOCUMENT`). Les paires posées par la contre-passation après une
   annulation de pièce (P4, « la contre-passation lettre ensuite ce qui est libre ») en relèvent.
6. **Le règlement d'une facture créditée** (C106, finding F10 de la 15-1a) : il garde sa ligne
   `invoice_settlements` (motif `OwnedBySettlement`, donc refusé au lettrage manuel) et ne peut
   entrer dans le groupe `document` (facture créditée **et** réglée : somme ≠ 0 au compte
   débiteurs). La 15-1a retire du manuel (`user-manual.tex:1169`, `:1696` — `:1171`, `:1711` au 2026-10-09) et d'`api-external.md`
   (`:325`, `:386`) la promesse « paiement **à lettrer** ». **Cette story tranche le cas** : soit
   une exception à R5 pour ce seul règlement, soit un groupe `document` qui l'inclut avec la
   contrepartie de l'avoir, soit le cas reste ouvert et le manuel le dit.
7. **Appariement d'une contre-passation par POSITION** (15-1a R6) et non par `line_order` égal :
   P6 (3) dit « par `reverses_entry_id` et `line_order` » — à aligner (rang dans `ORDER BY
   line_order`), la contre-passation renumérotant `idx + 1`.
8. **Ordre de développement** : **15-12a → 15-12b → 15-1a-i → 15-1a-ii → 15-1a2 → 15-1b → 15-1c** (C105 ; la 15-12 découpée le 2026-10-09, C107/C112 ; la 15-1a, C124).

*Ajouts de la remédiation de la validation P2 du socle (2026-10-09, registre C113 à C117) :*

9. **La règle des périodes remplace la règle des exercices** (C113). Une ligne est « en période
   ouverte » si son exercice est ouvert, qu'aucun exercice postérieur n'est clos et que sa date est
   postérieure à `books_locked_through`. En mode `Manual`, lettrer comme délettrer en exigent une ;
   le refus devient `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` (`LETTERING_FISCAL_YEARS_CLOSED` est
   retiré). En mode `System`, la primitive **ne l'évalue pas**. **À trancher ici** : une annulation de
   règlement datée du jour peut dissoudre un groupe `document` dont **toutes** les lignes sont dans
   une période verrouillée (facture et règlement du premier trimestre, verrou au 31 mars) — la liste
   des postes ouverts « au 31.03 » en serait réécrite (deux lignes ouvertes au lieu d'aucune ; la
   somme, elle, ne change pas). Le socle ne le refuse pas : garder le groupe rendrait la facture
   « soldée » alors qu'elle est due. Dire si on l'accepte (et l'écrire au manuel) ou si l'annulation
   d'un règlement de période verrouillée est refusée en amont.
10. **Le point 3 s'étend** : la migration de rattrapage (P6) devrait aussi s'abstenir des groupes
    dont toutes les lignes sont sous le verrou de période ou sous un exercice postérieur clos (même
    symétrie que pour les exercices clos).
11. **Verrous d'exercice dans l'ordre** (C114) : un `ORDER BY start_date` sur une liste d'`id` ne fixe
    pas l'ordre d'acquisition. Là où le point 2 demande de verrouiller l'exercice de l'écriture annulée
    « dans le sens `start_date` croissant avant l'exercice du jour », employer un parcours de l'index
    `(company_id, start_date)` (forme de R7 point 2 du socle) ou un verrou par `id` unique, avec
    `EXPLAIN` en T0.
12. **La contre-passation rend des lignes relues** (C116) : après R6, `reverse_in_tx` rend l'écriture
    inverse **avec** ses marques `reversal`. Une synchronisation appelée après elle peut s'y fier.
13. **Motif du bump** (C115) : le point 1 ci-dessus parle d'« une v0.12.1 relancée » ; le motif exact
    vise tout binaire publié depuis la v0.10.0 (les v0.10.0 à v0.11.1 modifient et suppriment des
    écritures manuelles). À reprendre tel quel si P6 en parle.

*Ajouts de la remédiation de la validation P3 du socle (2026-10-09, registre C124 à C127) :*

14. **Le socle est découpé** (C124) : la primitive, ses modes et les routes sont dans la **15-1a-i** ;
    R6 (la contre-passation qui lettre ce qui est libre, sur laquelle reposent les annulations de pièce
    — tableau des chemins ci-dessus) et le gel `ENTRY_LETTERED` sont dans la **15-1a-ii**. Cette fiche
    suppose les deux mergées.
15. **Le point 11 est périmé** (C125, qui révise C114) : le socle ne verrouille plus ses exercices par un
    parcours `(company_id, start_date)` — un parcours d'intervalle ne fixe pas l'ordre (C119). Forme
    retenue : lire sans verrou les `id` triés par `start_date`, puis verrouiller **un par un** par clé
    primaire (`SELECT … WHERE id = ? AND company_id = ? FOR UPDATE`) dans cet ordre ; et le socle ne
    verrouille plus que les exercices **du groupe**, l'information « exercice postérieur clos » se
    lisant sans verrou (`find_later_closed`, preuve (α)-(β) de la 15-12b AC 8). Là où le point 2 demande
    de verrouiller l'exercice de l'écriture annulée avant celui du jour, employer cette forme (pas
    d'`EXPLAIN` exigé : l'ordre est tenu par le code).
16. **`ENTRY_LETTERED` parle en dernier** (C126) : après le verrou de période, sur le `PUT`, le `DELETE`,
    la dévalidation et le motif d'écran. Si cette fiche ajoute un refus à `delete_in_tx` ou à
    `invoices::unvalidate`, elle le place **avant** la marque (« tout refus que le délettrage ne peut
    lever parle d'abord »).
17. **L'audit des groupes porte l'exercice de chaque ligne** (C127) : les `details` de
    `lettering.created` / `lettering.removed` ont, par ligne, `fiscalYearId` et `fiscalYearName` ; l'AC10
    de cette fiche (`documentType`… en plus) les hérite.

*Ajouts de la remédiation de la validation P4 du socle (2026-10-09, registre C128 à C131) :*

18. **D'où vient `fiscalYearName` en mode `System`** (C128) : la primitive lit le nom des exercices du
    groupe par une lecture **ordinaire, non verrouillante** (`SELECT id, name FROM fiscal_years WHERE
    company_id = ? AND id IN (…)`, après son acte 1 — R7 point 3 de la 15-1a-i). Les groupes `document`
    posés et dissous par `sync_*` (mode `System`) portent donc le nom par ligne sans que cette fiche ait
    rien à lire ; ⛔ ne pas y ajouter de lecture **verrouillante** d'exercice pour ce nom (elle
    inverserait l'ordre `start_date` derrière l'exercice tenu).
19. **La promesse « paiement à lettrer » est retirée de l'écran aussi** (C130 ; complète le point 6) : la
    15-1a-i réécrit, outre le manuel et `api-external.md`, les deux clés
    `invoices-settlement-cancel-blocked-credited` et `reconciliation-cancel-blocked-credited` (quatre
    locales), leurs replis Rust (`kesh-api/src/errors.rs:2966`, `:3495`) et frontend, quatre tests
    frontend et le doc-comment `kesh-db/src/errors.rs:327`, en « le règlement **reste ouvert au compte
    débiteurs**, il ne s'annule pas ». Si cette story **traite** le cas (exception à R5, ou groupe
    `document` qui inclut le règlement), c'est elle qui réécrit de nouveau ces sites — par la valeur,
    dans les quatre langues (`git grep -nE "reste ouvert au compte débiteurs|Debitorenkonto offen|stays
    open on the receivables|resta aperto sul conto debitori"`).
20. **« L'origine reste intacte » — la réserve de la marque** (C129) : la 15-1a-ii écrit que la
    contre-passation laisse l'origine intacte dans ses montants, comptes, dates et libellés, ses lignes
    recevant seulement la marque de lettrage. Le groupe `document` fait de même sur les lignes d'une
    facture et d'un avoir : `user-manual.tex:1230` et `:1235` (« l'écriture d'origine reste intacte »,
    « sa propre écriture reste intacte », § des avoirs) sont à relire dans ce sens ici.

## Reçu de la 15-1a-i — revue de code P2 (2026-10-09) : textes provisoires à retirer

*Section ajoutée par la remédiation de la revue de code P2 de la 15-1a-i (finding A2-1, registre
C-15-1a-i-7 et C-15-1a-i-11). Elle ne réécrit pas cette fiche : elle transmet ce que la 15-1a-i a écrit
au présent de son code livré, et que **cette** story rend faux.*

La revue de code P1 de la 15-1a-i a ramené les textes publics à ce que fait son code : seul le lettrage
`manual` existe, les origines `reversal` et `document` sont « réservées ». Les numéros de ligne sont ceux de la branche `story/15-1a-i-marque-du-lettrage` au commit de sa revue de
code P2 ; ils bougeront au merge — **re-greper par la valeur**. Relevé de C-15-1a-i-7, tel qu'écrit au
registre : `git grep -nE "Seul le lettrage manuel|ne lettre (pas encore|rien|encore rien)|réservé|n'existe encore" -- CHANGELOG.md docs` (62 lignes, dont 56 hors sujet — « réservé à l'administrateur » pour la plupart —, à
trier). Forme resserrée, qui rend **exactement les six sites** ci-dessous sur la branche de la 15-1a-i :
`git grep -nE "Seul le lettrage manuel|ne lettre (pas encore|rien|encore rien)|sont réservés aux lettrages|n'existe encore" -- CHANGELOG.md docs website README.md` ; et le PDF aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "Kesh ne lettre encore rien[^.]*\."`.

| site | texte provisoire |
|---|---|
| `CHANGELOG.md:15` | « **Seul le lettrage manuel existe à ce stade** : Kesh ne lettre pas encore de lui-même une facture soldée par ses règlements, ni une écriture et sa contre-passation. » |
| `docs/api-external.md:222` | `letteringOrigin` (« `manual` ; `document` et `reversal` sont réservés aux lettrages que Kesh posera de lui-même ») |
| `docs/api-external.md:288` | « L'origine d'un groupe est `manual` … ⚠️ **À ce stade, Kesh ne lettre rien de lui-même** : les origines `reversal` … et `document` … sont réservées … ; aucune route ne les rend encore. » |
| `docs/api-external.md:321` | refus `LETTERING_IS_DOCUMENT` du `DELETE`, annoté « *(aucun groupe `document` n'existe encore)* » |
| `docs/api-external.md:322` | refus `LETTERING_LINE_OWNED_BY_DOCUMENT` du `DELETE`, annoté « *(aucun groupe `reversal` n'existe encore)* » |
| `docs/manual/fr/user-manual.tex:2405-2406` + PDF | glossaire, entrée *Lettrage* : « Kesh ne lettre encore rien de lui-même~: ni une facture soldée par ses règlements, ni une écriture et sa contre-passation. » |

⛔ **Règle de relais** : chaque story réécrit ces textes **dans le même commit** que le comportement qui
les rend faux — non dans un commit de documentation ultérieur. Aucun test ne lit ces fichiers : rien ne
rougira si l'un d'eux est oublié, et C124 interdit tout tag entre les merges, si bien qu'un texte oublié
part dans la v0.13.0.

**Ce qui revient à cette story** (le groupe `document`) : CHANGELOG `:15` — retirer « une facture soldée
par ses règlements » de ce que Kesh « ne lettre pas encore », et réintroduire l'exemple « une facture et ses
règlements » ; `api-external.md:222` et `:288` — sortir `document` de la réserve ; `:321` — retirer
l'annotation « (aucun groupe `document` n'existe encore) » ; glossaire `.tex` `:2405-2406` et PDF régénéré —
retirer le premier « ni ». Les sites de `reversal` (`:222`, `:288`, `:322`, le second « ni ») reviennent à
la 15-1a-ii, qui se merge avant : au développement de cette story, relire ce qu'elle en a laissé — si les
deux sont faites, la phrase « Kesh ne lettre … de lui-même » disparaît entière.

## Critères d'acceptation

**AC1** — Une facture client entièrement réglée (un, puis **trois** règlements partiels, dont un
`internal_account`) est lettrée `document` : créance + lignes de règlement sur `A`, une seule clé.
Un règlement partiel seul ne lettre **rien**.

**AC2** — Solde du reste (`write_off`, chacune des quatre natures) qui éteint la facture → lettrée,
la ligne de solde comprise. Arrondi à 5 centimes (`round_to_5_centimes`) et règlement
`SettlesWithRounding` → lettrée, la ligne d'écart d'arrondi **exclue** (elle n'est pas sur `A`).

**AC3** — Avoir total sur une facture validée → facture et avoir lettrés `document`. *(Avec la
15-6a mergée. Sans elle, test de la divergence : réglage de créance changé avant l'avoir → aucun
groupe, aucune erreur.)*

**AC4** — Annulation d'un règlement d'une facture soldée → le groupe `document` est **dissous**,
le règlement et son miroir sont lettrés `reversal`, la créance est **ouverte** (et les autres
règlements partiels aussi). Même résultat par le **dé-rapprochement** d'une facture encaissée par
rapprochement. Facture de l'exercice N **clos**, règlement et annulation en N+1 → **réussit**
(P5).

**AC5** — **Accord grand livre ↔ pièce**, test sur une base de scénarios mêlés (paiement total,
partiels, solde, avoir, annulation, rapprochement, données héritées `paid_at` sans règlement) :
pour chaque facture validée ou créditée, *lettrée `document`* ⇔ *reste dû dérivé nul* — avec
**une seule** exception nommée : l'avoir crédité sur un autre compte (#473, tant que la 15-6a
n'est pas mergée). ⚠️ L'héritage `paid_at` sans règlement **n'est pas** une exception : son reste
dû dérivé vaut le TTC (aucune ligne de règlement, `INVOICE_AMOUNT_DUE_DERIVED_SQL` ne lit pas
`paid_at`) et il n'est pas lettré — les deux côtés s'accordent. Toute autre divergence fait
rougir.

**AC6** — **Accord rattrapage ↔ synchronisation** : sur la même base de scénarios, écrite **sans**
lettrage puis passée par la migration de rattrapage, l'appel de `sync_invoice_in_tx` /
`sync_supplier_invoice_in_tx` sur **chaque** pièce est un **no-op** (aucune ligne modifiée, aucune
entrée d'audit), et les paires `reversal` sont celles qu'aurait posées le socle.

**AC7** — Fournisseur : paiement (direct **et** par lot pain.001 confirmé) → achat et règlement
lettrés ; annulation du règlement → dissous, règlement ↔ miroir lettrés, achat ouvert ; annulation
d'une facture payée → dissous, achat ↔ miroir lettrés, **règlement détaché ouvert** et lettrable
à la main (15-1a R5).

**AC8** — **Inventaire fermé, tenu par un test lexical** : tout `INSERT INTO invoice_settlements`
ou `DELETE FROM invoice_settlements` du code de production (hors tests) est dans
`invoice_settlements.rs` / `invoice_settlements_write.rs` ; tout `UPDATE supplier_invoices … SET
… settlement_journal_entry_id` est dans une fonction qui appelle la synchronisation ou la
dissolution. Un site neuf fait rougir le test **en le nommant**.

**AC9** — Aucune ligne d'une pièce n'est jamais lettrée hors `document`, et réciproquement — 
ajout au test d'invariant de la 15-1a (AC13 de la 15-1a) : une ligne d'origine `document` est sur
l'écriture d'une pièce ; une ligne d'une écriture possédée par une pièce n'est ni `manual` ni
`reversal`, **sauf** la ligne d'un règlement annulé (plus possédée) et l'achat d'une facture
fournisseur annulée.

**AC10** — Audit : chaque création/dissolution `document` produit `lettering.created` /
`lettering.removed` (15-1a AC10) dont les `details` portent en plus `documentType`
(`invoice`, `supplier_invoice`) et `documentId` / `documentNumber`. Acteur : l'auteur du geste.

**AC11** — Migration P6 : P7 (registre, classe A, rejeu après restauration testé par
`admin_full_import_e2e` ou équivalent sur une sauvegarde **sans** lettrage), P5 (ligne d'audit
d'idempotence, verdict `yes`, compteurs recomptés), squash régénéré, `migrations.sha384`, P6
(inspection des sites positionnels). Non-breaking (P1 : `UPDATE` de colonnes nullables neuves).

**AC12** — Documentation : CHANGELOG (complète l'entrée de la 15-1a : « les factures soldées et les
paiements fournisseurs sont lettrés d'office ; l'annulation d'un règlement délettre »),
`api-external.md` (le lettrage `document` n'est **pas** délettrable par l'API —
`LETTERING_IS_DOCUMENT` —, il suit les règlements).

## Tasks

- [ ] **T1** (P3) — `sync_invoice_in_tx`, `sync_supplier_invoice_in_tx`, et
      `dissolve_document_group_for_*_in_tx`, dans `letterings.rs`.
- [ ] **T2** (P4) — Les six appels du tableau P4, chacun à la place indiquée.
- [ ] **T3** (P6, AC11) — Migration de rattrapage + entrée `POST_RESTORE_BACKFILLS` (classe A) ;
      squash ; sha384 ; audit d'idempotence.
- [ ] **T4** (AC8) — Test lexical d'inventaire.
- [ ] **T5** — Tests AC1 à AC7, AC9, AC10 ; la base de scénarios d'AC5/AC6 est **une** fixture
      partagée.
- [ ] **T6** (AC12) — CHANGELOG, `api-external.md`.

## Dev Notes

- **Gate `kesh-db` complet** (migration de données + repositories) ; base remise à zéro avant,
  sans redémarrer le conteneur.
- **Rejeu** : aucune route neuve ; les routes appelantes sont déjà `Rejouee`
  (`audit_route_registry.rs`) — vérifier qu'elles le restent.
- **Verrous** : la synchronisation s'exécute **après** les verrous du geste (facture `FOR UPDATE`,
  écriture `FOR UPDATE`) et prend ceux de la primitive (en-têtes par `id` croissant, puis
  lignes). Un second règlement concurrent sur la même facture est déjà sérialisé par le verrou de
  la facture ; une ligne de pièce n'est jamais dans un groupe manuel (15-1a R5) : pas de cycle
  identifié, le rejeu reste la défense.
- **Dépendances** : 15-1a ; **15-6a recommandée avant** (#473) ; 15-6b (#474) sans dépendance
  dure.
- **Modules** : `kesh-db` (repositories, migration, `post_restore`), `kesh-api` (aucun code si
  `accept_one_invoice` reste couvert par `create_in_tx` — à vérifier), docs. Sous le seuil de
  découpage.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

### Reçu de la 15-1a-i — 2026-10-09 (Opus 5.5, remédiation de la revue de code P2 de la 15-1a-i)

Section « Reçu de la 15-1a-i — revue de code P2 » ajoutée (finding A2-1, registre C-15-1a-i-7 et
C-15-1a-i-11) : six textes provisoires écrits par la 15-1a-i (« Kesh ne lettre rien de lui-même »,
origines « réservées »), dont quatre à réécrire ici au même commit que le groupe `document`. Corps de
cette fiche non réécrit. Édition hors passe.

### Reçu de la validation P4 du socle — 2026-10-09 (Opus 5.5, remédiation de la 15-1a-i et de la 15-1a-ii)

Section « Reçu de la 15-1a » complétée des points 18 à 20 (registre C128 à C130) : nom des exercices en
mode `System` lu sans verrou, promesse « à lettrer » retirée de l'écran par la 15-1a-i, réserve « intacte
hors la marque » pour l'origine d'une contre-passation, à transposer aux avoirs. Corps de cette fiche
non réécrit.

### Reçu de la validation P3 du socle — 2026-10-09 (Opus 5.5, remédiation de la 15-1a)

Section « Reçu de la 15-1a » complétée des points 14 à 17 (registre C124 à C127) : la 15-1a est découpée
en 15-1a-i et 15-1a-ii (cette fiche suppose les deux) ; le point 11 est périmé (verrous d'exercice un par
un, bornés au groupe, C125) ; `ENTRY_LETTERED` parle en dernier (C126) ; l'audit porte l'exercice par
ligne (C127). Dépendance et ordre de tête mis à jour ; un numéro du manuel actualisé au point 6. Corps
non réécrit : à intégrer à la validation de cette fiche.

### Création — 2026-10-08 (reprise du lettrage, Opus 5.5, en autonomie)

Story **nouvelle**, née de la relecture du lettrage contre `invoice_settlements` (registre C93,
C98). Elle n'a pas d'ancêtre dans les fiches d'août, qui supposaient qu'aucune écriture
d'encaissement n'existait. **12 critères** (AC1–AC12), **6 tâches** (T1–T6), recomptés depuis ce
fichier.
