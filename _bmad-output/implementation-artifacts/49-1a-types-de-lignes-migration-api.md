# Story 49-1a : types de lignes de la facture client — la migration, le modèle, les dépôts et l'API gardée (#1242)

Status: ready-for-dev

*Fiche rédigée le 2026-11-18 par l'agent de rédaction de la 49-1 (Sonnet 5.5), par le workflow `bmad-create-story`, en découpant la
49-1 (`kesh-gate-logs/e491/decoupage-49-1.md`, règle de splitting préventif : neuf modules). Code lu à `773df80da` (v0.19.0), worktree
`kesh-49-1`, branche `story/49-1-types-de-lignes`. Release **0.20.0** (coupure jeudi 2026-11-26 au soir). **Story zéro de la 49-1 ;
porte LA migration de la release (la 96ᵉ).** **`kesh-db` touché : gate complet même en cours de boucle** (`CLAUDE.md` § *Pendant une
boucle de revue*). Décisions : fichier d'épique `epic-49-types-de-lignes-facture.md` (sur `origin/chore/kickoff-0.20.0`), décisions
de Guy du 2026-11-18 ; les choix de cette fiche sont D1 à D14. Aucune validation n'a encore eu lieu.*

**Issues** : `refs #1242` ; **aucun mot-clé de fermeture** dans un commit. #1242 ne se ferme pas avec cette story.

## Story

En tant que Guy, qui a déjà émis de vraies factures et en émettra d'autres avant la fin novembre, je veux que Kesh sache
distinguer une ligne **chiffrée** d'une ligne **de texte seul** (commentaire, titre de section) — sans qu'**aucune** facture ni
aucun avoir existant ne change d'un caractère, d'un centime ni d'un octet de PDF.

Cette story pose **la migration** (le type de ligne et ses contraintes, sur `invoice_lines` **et** `credit_note_lines`), le
**modèle** et ses **dépôts**, la **reprise des types dans l'avoir**, le **filtrage** des lecteurs qui regrouperaient ou
compteraient des lignes sans montant, les **exports** et le **MCP**, et une **API qui connaît les types mais n'en ouvre aucun
de neuf** : seul `priced` est accepté, `comment` et `heading` sont refusés `400` tant que leur rendu et leur écran n'existent pas.
**Rien ne change pour l'utilisateur.** **Couplage de coupure (décision de l'orchestrateur, 2026-11-18) : la 49-1a ne sort jamais sans la 49-1d** — la sauvegarde d'avant la mise à jour au manuel d'administration et la recette de montée (`--diff-attendu`) sont exigées de toute release qui porte une migration (`CLAUDE.md` § *Ce qu'impose la production*) ; à la coupure, les deux sortent ensemble, ou les deux restent. Le rendu PDF est la 49-1b ; l'écran, l'ouverture de `comment` et le manuel utilisateur la
49-1c ; le gate de recette et le manuel d'administration la 49-1d ; le titre de section la 49-3.

## Ce qui se lit au code (`773df80da`)

### Le schéma actuel, et ce qu'il ne dit pas

- `invoice_lines` n'a **aucune colonne de type** et **aucun lien au catalogue** : `crates/kesh-db/migrations/20260416000001_invoices.sql:54`
  `CONSTRAINT chk_invoice_lines_quantity_positive CHECK (quantity > 0),` ; `:57` `CONSTRAINT chk_invoice_lines_description_not_empty` ;
  `:59` `CONSTRAINT uq_invoice_lines_position UNIQUE (invoice_id, position),` ; `revenue_account_id` ajouté par
  `20260727000001_invoice_lines_revenue_account.sql` ; `chk_invoice_lines_line_total_non_negative` par `20260416000002`.
- `credit_note_lines` a la **même structure** : `20260627000001_credit_notes.sql:76`
  `CONSTRAINT chk_credit_note_lines_quantity_positive CHECK (quantity > 0),` ; `:79-81` description non vide, `line_total >= 0` ;
  `20260727000001` y ajoute `revenue_account_id`.
- **Preuve que l'origine d'une ligne existante n'est pas établissable** (règle de typage, D1) :
  - `git grep -rn "product_id" -- crates/kesh-db/migrations` : **vide** ; aucune migration n'ajoute une colonne d'article.
  - `crates/kesh-db/src/entities/invoice.rs:76` `pub struct InvoiceLine {` : champs `id, invoice_id, position, description, quantity,
    unit_price, vat_rate, line_total, revenue_account_id, created_at` — aucun article. `git grep -n -i "product" --
    crates/kesh-db/src/entities/invoice.rs crates/kesh-db/src/entities/credit_note.rs` ne rend qu'un doc-comment sur
    `products::update` (`invoice.rs:137`).
  - L'écran ne fait que **recopier** : `frontend/src/lib/components/invoices/InvoiceForm.svelte:480`
    `function onProductSelect(p: ProductResponse) {` pousse `description: p.name, quantity: '1', unitPrice: …, vatRate: p.vatRate,
    revenueAccountId: p.defaultRevenueAccountId ?? null` : une ligne ordinaire, sans identifiant d'article ; et
    `crates/kesh-api/src/routes/products.rs:306` : « son compte est recopié sur la ligne de facture ».
  - **Conclusion** : une ligne « produit » et une ligne « sans produit » sont **le même enregistrement**. Aucune migration, aucune
    règle ne peut les distinguer sans deviner (nom égal au nom d'un article ? prix égal ? faux dès que l'article a été renommé
    ou repricé). La règle de typage est donc celle que le kickoff prévoit pour le doute : **un seul type « chiffré » (`priced`)**,
    qui se rend et se compte **exactement** comme aujourd'hui.

### Les lecteurs et écrivains (inventaire par le symptôme, liste close)

Le symptôme : *« une ligne de facture ou d'avoir est lue, comptée, regroupée ou écrite sans qu'on ait à dire de quel type elle est »*.
Commandes (sorties copiées) :

```sh
$ git grep -lE "invoice_lines|credit_note_lines" -- 'crates/kesh-*/src/*' 'scripts'     # 18 fichiers (rejoué le 2026-11-18)
crates/kesh-api/src/exports/csv_tables.rs          crates/kesh-api/src/exports/global.rs
crates/kesh-api/src/routes/invoice_pdf_service.rs  crates/kesh-core/src/accounting/vat.rs   (commentaire)
crates/kesh-db/src/backup.rs                       crates/kesh-db/src/entities/credit_note.rs
crates/kesh-db/src/post_restore.rs                 crates/kesh-db/src/repositories/credit_notes.rs
crates/kesh-db/src/repositories/invoice_settlements.rs  crates/kesh-db/src/repositories/invoice_settlements_write.rs
crates/kesh-db/src/repositories/invoices.rs        crates/kesh-db/src/repositories/journal_entries.rs
crates/kesh-db/src/repositories/letterings/open_items.rs  crates/kesh-db/src/repositories/supplier_invoices.rs
crates/kesh-db/src/test_fixtures.rs                crates/kesh-report/src/vat_report.rs
crates/kesh-seed/src/pieges.rs                     scripts/recette-montee.sh
$ git grep -nE "\bInvoiceLine\b|NewInvoiceLine|CreditNoteLine\b|InvoiceLinePdf" -- 'crates/*/src/*'   # les porteurs du type
$ git grep -n "verify_vat_rates_against_db" -- 'crates/kesh-api/src/*'    # les consommateurs de `vat_rate` par ligne (V1-2)
crates/kesh-api/src/routes/invoices.rs:680   crates/kesh-api/src/routes/invoices.rs:773   (et sa définition, `routes/vat.rs`)
  (entités, dépôts, routes `invoices.rs` / `credit_notes.rs`, `invoice_email.rs`, `invoice_pdf_service.rs`, `services/factures.rs`,
   `mcp/outils/factures.rs`, `project_reassignment.rs`, `kesh-qrbill/src/{types,pdf}.rs`, `kesh-seed/src/pieges.rs`)
```

Chaque site est **résolu** (dans la story, avec son AC) ou écrit **hors périmètre** avec la story qui le prend :

| Site | Lit / écrit | Verdict au regard d'une ligne non chiffrée | Résolution |
|---|---|---|---|
| `kesh-db/src/repositories/invoices.rs:47` `LINE_COLUMNS` | liste de colonnes (partagée avec `credit_notes`) | doit porter `line_type` | **AC5** |
| `invoices.rs:464` `compute_total`, `:471` `insert_lines` (`INSERT INTO invoice_lines` `:480`) | écrit | doit poser le type, refuser une ligne non neutre | **AC6** |
| `invoices.rs:1067` `is_no_op_change` | compare | **sans le type, changer *seulement* le type d'une ligne serait vu comme un no-op** (même défaut que celui de la 16-1a pour le compte, cité `:1086-1090`) | **AC7** |
| `invoices.rs:77` `invoice_snapshot_json` | audit | le type doit figurer au « before / after » | **AC7** |
| `invoices.rs:2186` `let total_ht: Decimal = lines_before.iter().map(|l| l.line_total).sum();` | somme | **sûr par neutralité** (somme nulle) | AC8 (test) |
| `invoices.rs:2269` `if lines_before.iter().any(\|l\| l.revenue_account_id.is_none()) {` | **compte** | une ligne de commentaire (compte `NULL` par contrainte) déclencherait l'exigence du compte par défaut | **AC8** |
| `invoices.rs:2312-2314` `if lines_before.iter().any(…is_none()) { sqlx::query("UPDATE invoice_lines SET revenue_account_id = ? WHERE invoice_id = ? AND revenue_account_id IS NULL")` | **écrit** | **poserait un compte de produit sur une ligne de commentaire et violerait `chk_invoice_lines_unpriced_neutral` : 500 SQL à la validation** | **AC8** (le piège central) |
| `invoices.rs:1963` `generate_invoice_journal_lines_rounded`, `:1848` `generate_invoice_journal_lines` | regroupe par compte et par taux | **sûr par neutralité** (montants `> 0` seulement, `vat_by_rate` à zéro) ; un test l'épingle | AC8 (test) |
| `invoices.rs:3045` `list_all_lines_by_company` (liste de colonnes en dur `:3050-3053`) | export | doit porter `line_type` | **AC5** |
| `credit_notes.rs:420-430` (snapshot de `invoice_lines` avec `LINE_COLUMNS`) puis `:774` `INSERT INTO credit_note_lines` | copie | doit copier le type, **toutes** les lignes, à leur position | **AC9** |
| `credit_notes.rs:499` `let triplets … = invoice_lines.iter().map(|l| (l.line_total, l.vat_rate, l.revenue_account_id))` ; `:632` `if invoice_lines.iter().any(\|l\| l.revenue_account_id.is_none())` | regroupe / compte | comptes verrouillés et site « compte par défaut » : ne considérer que les lignes `priced` | **AC9** |
| `credit_notes.rs:148` `credit_note_vat_by_rate`, `:608` `total_ht` | regroupe / somme | sûrs par neutralité (aucune TVA, aucun montant) | AC9 (test) |
| `credit_notes.rs:66-76`, `:98-104`, `:919-935` (trois listes de colonnes en dur de `credit_note_lines`) | lit | **une seule constante** (la leçon de `LINE_COLUMNS`, `invoices.rs:42-46`) portant `line_type` | **AC5** |
| `invoice_settlements.rs:69` et `:93` (`FROM credit_note_lines cl`, macro `line_ttc_sql!`) ; `invoices.rs:217` et `:230` (TTC) | somme TTC | **sûrs par neutralité** : `line_total + ROUND(line_total * vat_rate / 100, 2)` = 0 | **AC10** (test de parité avec une ligne de commentaire) |
| `invoice_settlements_write.rs:647` `SELECT line_total, vat_rate FROM invoice_lines WHERE invoice_id = ?` → `write_off_vat_shares` | lit | sûr par neutralité (`vat_breakdown_by_rate` écarte `vat_rate <= 0`, `kesh-core/src/accounting/vat.rs:132`) ; on **filtre quand même** `AND line_type = 'priced'` (une intention lisible) | AC10 |
| `kesh-core/src/accounting/vat.rs` | TVA par taux | **aucun changement** : l'écart au fichier d'épique, qui déclarait ce module | aucun |
| `kesh-report/src/vat_report.rs:121` `FROM invoice_lines il … WHERE i.company_id = ? AND i.status = 'validated'` | **regroupe par taux** | **une ligne de commentaire (`vat_rate = 0`) ferait apparaître une ligne « 0 % » de base 0 dans le rapport TVA** (`by_rate.entry(vat_rate)`) : le « taux 0 % fantôme » du kickoff | **AC10** |
| `kesh-api/src/routes/invoices.rs:69` `CreateInvoiceLineRequest`, `:150` `InvoiceLineResponse`, `:475` `validate_line`, `:584` `validate_lines` | écrit / sert | type, validation par type, réponse | **AC11**, **AC12** |
| `kesh-api/src/routes/invoices.rs:679-680` et `:772-773` : `let rates: Vec<Decimal> = lines.iter().map(\|l\| l.vat_rate).collect(); vat::verify_vat_rates_against_db(…)` (`routes/vat.rs:96-110`, `COUNT(DISTINCT rate) … active = TRUE`) | **liste blanche des taux** | **un commentaire (`vat_rate = 0`) serait refusé `400` « taux de TVA » dans toute société dont le 0 % est désactivé** | **hors périmètre de la 49-1a, pris par la 49-1c** (qui ouvre `comment` : ne collecter que les taux des lignes `priced`, avec son AC). **En 49-1a le site est inatteignable** : la garde (AC11), dans `validate_lines`, rejette tout type non chiffré avant lui. |
| `kesh-api/src/routes/credit_notes.rs:33` `CreditNoteLineResponse` | sert | type dans la réponse | **AC12** |
| `kesh-api/src/exports/csv_tables.rs:623`, `:1271` ; `exports/global.rs:133`, `:146` | exporte | colonne `line_type` | **AC13** |
| `kesh-api/src/mcp/outils/factures.rs:237` `LigneFacture`, `:487` | sert | champ `type_ligne` | **AC13** |
| `kesh-db/src/backup.rs` (`TABLES_TO_TRUNCATE`, `:47` et `:50`), `admin_backup/import.rs:627` | sauvegarde / import | colonnes lues par introspection ; une colonne `NOT NULL DEFAULT` n'est pas « requise à la source » (`backup.rs:767`) | **AC4** (test) ; aucun changement de code |
| `kesh-db/src/post_restore.rs:210` `POST_RESTORE_BACKFILLS` | rejeu | aucun rejeu actif ne touche `invoice_lines` (une seule entrée, `20261106000001`) ; la migration n'écrit aucune donnée | **AC3** |
| `kesh-seed/src/pieges.rs:579` `lines: vec![NewInvoiceLine {` ; `:93` `champ("facture_ligne", "invoice_lines", "description", 1000, 10)` | écrit | un littéral à compléter ; le champ piégé est la **description**, inchangée | AC5 |
| `kesh-api/src/routes/invoice_pdf_service.rs:396`, `routes/credit_notes.rs:307` (`InvoicePdfData.lines`), `kesh-qrbill/src/{types,pdf}.rs` | **rend** | une ligne non chiffrée s'imprimerait `0 × 0.00 = 0.00` | **hors périmètre → 49-1b** ; **la garde de l'API (AC11) empêche d'en créer** d'ici là |
| `kesh-api/src/routes/invoice_email.rs:164,243,371` | TTC par `invoice_total_ttc` | sûr par neutralité | hors périmètre (aucun changement) |
| `kesh-db/src/repositories/project_reassignment.rs:85,564` | lignes de facture jointes à l'audit | aucune lecture de montant | hors périmètre (aucun changement) |
| `frontend/…/InvoiceForm.svelte`, `ProductPicker.svelte`, `features/invoices`, `features/credit-notes`, `routes/(app)/invoices`, `credit-notes` | affiche / envoie | ne se casse pas : `lineType` est **additif** à la lecture, optionnel à l'écriture | **hors périmètre → 49-1c** |
| `scripts/recette-montee.sh:414` (`TABLES="… invoice_lines …"`, `SELECT * FROM $t`) | empreinte | la nouvelle colonne **fait diverger** `invoice_lines` entre « avant » et « après » | **hors périmètre → 49-1d** (`--diff-attendu`) |
| `kesh-db/src/repositories/supplier_invoices.rs`, `journal_entries.rs:679`, `letterings/open_items.rs:666-692`, `test_fixtures.rs:377` | **faux positifs** du `git grep` : `supplier_invoice_lines` (table sœur, hors périmètre), commentaires, texte de test | aucune lecture de `invoice_lines` | aucun |

Fichiers de **tests** qui lisent ou écrivent ces tables : `git grep -lE "invoice_lines|credit_note_lines" -- 'crates/*/tests/*' | wc -l`
= **31** (le pathspec `'crates/*/tests'` rend 0 : muet). Ils compilent après l'ajout du champ — **67 littéraux `NewInvoiceLine {` dans
43 fichiers** (`git grep -F "NewInvoiceLine {" -- 'crates/*' | grep -v "pub struct" | wc -l` = 67 ; `git grep -lF … | wc -l` = 43), 4 littéraux `InvoiceLine {`
(`invoices.rs:3148`, `credit_notes.rs:1105`, `invoice_email.rs:1624`, `invoice_pdf_service.rs:1251`, tous de tests) ; ceux dont
**l'assertion** change sont nommés aux tâches.

### Ce que fait une ligne neutre, et ce qu'elle ne fait pas

- **Sûr par neutralité** : `quantity = 0`, `unit_price = 0`, `vat_rate = 0`, `line_total = 0` contribuent **zéro** à toute somme
  (`line_ttc_sql!` = `line_total + ROUND(line_total * vat_rate / 100, 2)`, `invoices.rs:186-197`), et `vat_breakdown_by_rate` écarte
  les taux `<= 0` (`vat.rs:132`). Aucun changement dans `kesh-core`.
- **Pas sûr** : un lecteur qui **regroupe** (rapport TVA), **compte** (`any(revenue_account_id.is_none())`), **écrit** (`UPDATE …
  SET revenue_account_id`) ou **rend** (PDF) une ligne par ligne. Ce sont exactement les sites des AC7 à AC10 et de la 49-1b.

### L'API, et la compatibilité des intégrations

- `crates/kesh-api/src/routes/invoices.rs:69` : `quantity`, `unit_price`, `vat_rate` sont des `Decimal` **obligatoires** à la
  désérialisation ; seul `revenue_account_id` porte `#[serde(default)]` (justification en doc-comment, `:77-86`).
- `validate_line` (`:475`) refuse `quantity <= 0` (`:490-495`) et `unit_price < 0` ; l'index d'erreur est `index + 1`, le rang dans
  la liste **tous types confondus** (le même que celui de l'écran, qui numérote ses lignes).
- La validation exécute `validate_lines` (`:584`) : au moins une ligne, au plus `MAX_LINES = 200` (`:61`).
- `validate_invoice` refuse déjà une pièce dont le HT est nul : `invoices.rs:2186-2188`
  `if total_ht == Decimal::ZERO { return Err(DbError::InvalidInput("invoiceTotalZero".into())); }` ; l'avoir de même
  (`credit_notes.rs:608-611` (`:610`), `creditNoteTotalZero`). **Une facture faite de commentaires seuls ne se valide donc pas** : aucun
  changement de règle.
- **Un avoir est toujours total** : `credit_notes.invoice_id` est `UNIQUE`, et `create_credit_note` copie **toutes** les lignes
  (`credit_notes.rs:771-791`, « (10) INSERT credit_note_lines (snapshot) »).

### Sauvegarde et import

- L'import insère par les **colonnes du manifeste** (`backup.rs:806`, doc de `TableRestore.column_names`) ; une colonne destination
  n'est exigée à la source que si elle est `NOT NULL`, sans `DEFAULT` (`backup.rs:767-769`, `is_required`) ; `import.rs:627`
  applique ce filtre. Une colonne `line_type VARCHAR(16) NOT NULL DEFAULT 'priced'` **n'est pas exigée** : une sauvegarde 0.19.0
  s'importe, ses lignes reçoivent `'priced'` — précédent : la colonne à `DEFAULT` de la 43-1a (`group_settlement_max_difference`).
- `POST_RESTORE_BACKFILLS` (`post_restore.rs:210`) ne porte **qu'une entrée active** (`20261106000001`, la numérotation interne des
  factures fournisseur) ; le rejeu de `20260729000001` (revenu de `invoice_lines`) est dans `RETIRED_BACKFILLS`
  (`post_restore.rs:288`). Rien ne rejoue donc un `UPDATE invoice_lines` sur une base restaurée, ce qui aurait violé la contrainte
  neutre sur une ligne de commentaire d'une sauvegarde 0.20.0.

### Le PDF (pour mémoire : 49-1b)

- `kesh-api/src/routes/invoice_pdf_service.rs:396` : `let invoice_lines_pdf: Vec<InvoiceLinePdf> = lines.iter().map(|l| InvoiceLinePdf { … })`
  sans filtre ; `credit_notes.rs:307` idem. Le PDF d'une facture validée se **fige au premier téléchargement**
  (`issued_invoice_pdf.rs:95` `get_or_freeze`) ; `refreeze_invoice_pdf` (`invoice_pdf.rs:74`) en fait un nouveau
  (`issued_invoice_pdf.rs:238` `refreeze`) ; **le PDF d'un avoir n'est pas figé** (rendu à la demande, `credit_notes.rs:307`).
  La variante déterministe `generate_qr_bill_pdf_with_date` existe (`kesh-qrbill/src/pdf.rs:58`), à une réserve près : l'identifiant
  d'instance interne de printpdf varie (doc de `:55-57`). Rien de cela ne change dans cette story : **aucun fichier de rendu n'est
  touché**, et les tests de PDF existants (`invoice_pdf_e2e`, `invoice_frozen_pdf_e2e`, `invoice_frozen_pdf`) sont l'épreuve.

## Décisions

- **D1 — Trois types stockés : `priced`, `comment`, `heading`.** « Produit » et « sans produit » sont **un seul enregistrement**
  (aucune colonne d'article, voir plus haut) ; ils ne diffèrent qu'à l'écran. *Écartée* : quatre types (`product`, `free`,
  `comment`, `heading`) comme le fichier d'épique l'écrit — il faudrait **deviner** le type des lignes existantes (nom égal à
  celui d'un article ? prix égal ?), et une ligne devinée à tort est un faux dans une facture remise sur papier ; rien ne
  distingue ensuite un « produit » d'un « sans produit » qui aurait le même nom. **Règle de typage des lignes existantes** : toutes
  `priced`, par le `DEFAULT` de la colonne (D2), sans un `UPDATE`. Cet écart est déclaré au rapport.
- **D2 — Colonne `line_type VARCHAR(16) NOT NULL DEFAULT 'priced'`, `CHECK (line_type IN ('priced','comment','heading'))`.**
  Le `DEFAULT` **est** le rétro-remplissage : `ADD COLUMN … DEFAULT` donne `'priced'` aux lignes existantes, et une sauvegarde 0.19.0
  (sans la colonne) donne `'priced'` à l'import. Aucun `UPDATE`, donc aucun triage P7 (D12). *Écartée* : un `ENUM` MariaDB (les
  énumérations se gèrent par `CHECK` texte dans ce dépôt, voir `20260416000001_invoices.sql:5`) ; une colonne `NULL` (une valeur
  qui veut dire « chiffrée » ne doit pas se lire `NULL`).
- **D3 — Les colonnes de montant restent `NOT NULL` ; une ligne non chiffrée porte des valeurs neutres** : `quantity = 0`,
  `unit_price = 0`, `vat_rate = 0`, `line_total = 0`, `revenue_account_id` `NULL`. Décision de l'orchestrateur du 2026-11-18
  (délégation de Guy), reprise : rendre les colonnes nullables rendrait une ligne écrite par la 0.20.0 illisible par un binaire
  0.19.0, cas breaking (P1, P3) qui exigerait de relever `kesh_version_min_required` et les crates (P2-bis).
- **D4 — Les contraintes : trois neuves par table, une retirée, le reste intact.** Pour `invoice_lines` (`credit_note_lines`
  identique, noms en `chk_credit_note_lines_…`) :
  1. `DROP CONSTRAINT chk_invoice_lines_quantity_positive` (la seule qui interdit un commentaire) ;
  2. `ADD CONSTRAINT chk_invoice_lines_line_type CHECK (line_type IN ('priced', 'comment', 'heading'))` ;
  3. `ADD CONSTRAINT chk_invoice_lines_priced_quantity CHECK (line_type <> 'priced' OR quantity > 0)` — **une ligne chiffrée garde
     exactement la règle d'aujourd'hui** ;
  4. `ADD CONSTRAINT chk_invoice_lines_unpriced_neutral CHECK (line_type = 'priced' OR (quantity = 0 AND unit_price = 0 AND
     vat_rate = 0 AND line_total = 0 AND revenue_account_id IS NULL))` ;
  `unit_price >= 0`, `vat_rate` entre 0 et 100, `line_total >= 0`, **la description non vide** (un commentaire a un texte) et
  `uq_*_position` **ne changent pas**. *Écartée* : une contrainte par type et par colonne (huit `CHECK`) — quatre suffisent et
  la panne se lit mieux.
- **D5 — Pas de type « sous-total » dans la migration** (la condition du kickoff : *seulement si la fiche établit qu'il ne
  préjuge pas de son sens TVA*). **La fiche l'établit dans le sens contraire.** #1330 laisse ouverts : HT ou TTC, ventilation par
  taux, calculé au rendu ou figé, portée depuis le titre ou le sous-total précédent, reprise dans l'avoir. Une valeur dans
  `CHECK … IN (…)` n'est pas neutre : elle fixerait qu'un sous-total est **une ligne stockée à valeurs neutres**, alors que le
  sens « juste en HT et TTC et par taux » demande peut-être des colonnes (taux, base) ou aucune ligne du tout (un sous-total
  **calculé au rendu**). Y revenir plus tard n'exige **qu'un `DROP CONSTRAINT` / `ADD CONSTRAINT` de `chk_*_line_type` et de
  `chk_*_unpriced_neutral`** — une opération qui n'est **pas** dans la liste de P3, donc non breaking. Le type reste dans #1330.
- **D6 — Le titre de section entre au schéma** (`heading`, mêmes contraintes qu'un commentaire), pour que la 49-3 n'ait pas à
  migrer. L'API le **refuse** jusqu'à la 49-3 (D9).
- **D7 — L'avoir recopie le type et la position de chaque ligne.** `CreditNoteLine.line_type` ; le snapshot de
  `credit_notes.rs:771-791` copie **toutes** les lignes, à leur position, comme aujourd'hui, avec leur type. Le débit de
  contre-passation et les comptes verrouillés ne considèrent que les lignes `priced` (AC9). *Écartée* : ne copier que les lignes
  chiffrées (l'avoir perdrait les commentaires, que Guy demande de reprendre : #1242).
- **D8 — Le contrat d'API : `lineType` optionnel, valeur par défaut `"priced"`.** `quantity`, `unitPrice`, `vatRate` deviennent
  `Option<Decimal>` à la **désérialisation** (avec `#[serde(default)]`) et sont **exigés pour `priced`** par `validate_line` ;
  pour `comment` et `heading` ils sont **absents ou nuls**, tout autre montant est refusé, de même qu'un `revenueAccountId`.
  Une intégration existante (PAT, Beat) qui n'envoie pas `lineType` reste **acceptée à l'identique**. Seul changement observable :
  l'omission d'un de ces trois champs sur une ligne chiffrée rend `400 VALIDATION_ERROR` nommant la ligne et le champ (« Ligne N :
  la quantité est obligatoire »), au lieu du `422` de l'extracteur `Json` (`errors.rs`, `StatusCode::UNPROCESSABLE_ENTITY`). **⚠️ Changement de contrat de l'API :
  le statut passe de `422` à `400 VALIDATION_ERROR`** pour un champ manquant (ou `null`) sur une ligne chiffrée, documenté comme
  `docs/api-external.md:1121` (« Une ligne CHF sans `debit` ou sans `credit` rend désormais `400 VALIDATION_ERROR`, non plus le `422` d'un
  champ manquant »). Un corps de 0.19.0 qui envoie les quatre champs n'est **pas** touché. **Un `lineType` inconnu** (`"subtotal"`,
  `"product"`) reste, lui, un rejet de l'extracteur : **`422`**, tel quel (V1-12). *Écartée* : un `enum` serde étiqueté par type
  (changerait la forme du corps pour tous les clients).
- **D9 — La garde : `LineType::open_to_api()`, une seule fonction.** En 49-1a elle rend `true` pour `priced` seulement ; la 49-1c
  l'ouvre pour `comment`, la 49-3 pour `heading`. Un type refusé donne `400 VALIDATION_ERROR` : « Ligne N : le type de ligne
  `comment` n'est pas encore disponible », **sans aucune écriture**, au `POST` comme au `PUT`. **La garde est dans `kesh-api`, non
  dans le dépôt** : le dépôt accepte les trois types (les tests de la 49-1b créent leurs lignes par le dépôt). *Écartée* : la garde au dépôt
  (elle bloquerait les tests de rendu) ; ne rien garder (un commentaire créé par l'API s'imprimerait `0 × 0.00 = 0.00` jusqu'à la 49-1b).
- **D10 — Le dépôt refuse une ligne non neutre.** `insert_lines` rend `DbError::Invariant` si une ligne `comment` ou `heading`
  porte un montant ou un compte (le gestionnaire HTTP a pré-validé, comme pour `lines` vide, `invoices.rs:691-697`) : il **ne
  normalise pas en silence**. *Écartée* : forcer les valeurs à zéro (cacherait une erreur de l'appelant).
- **D11 — La position reste l'index dans la liste, tous types confondus** (`insert_lines`, `.bind(idx as i32)`, `invoices.rs:480-486`) ;
  `uq_invoice_lines_position` inchangé. Le numéro de ligne des messages d'erreur est ce rang. Pas de numérotation séparée des
  lignes chiffrées.
- **D12 — Aucune ligne de `POST_RESTORE_BACKFILLS`, aucune de `EXEMPT_MIGRATIONS` (triage P7).** `every_data_backfill_migration_is_triaged`
  (`post_restore.rs:1018`) ne retient que les migrations qui **écrivent** des données : celle-ci ne contient que du DDL (`ALTER TABLE
  … ADD COLUMN … DEFAULT`, `DROP/ADD CONSTRAINT`), et le `DEFAULT` donne la même valeur à une ligne montée en place et à une
  ligne importée d'une sauvegarde 0.19.0. **Le fichier ne contient ni `UPDATE`, ni `INSERT`, ni `REPLACE`, ni `DELETE`, même en
  commentaire décommenté** (le détecteur cherche large, P7). *Si* le détecteur rougit malgré tout, le développeur **s'arrête**
  (la prémisse de D2 est fausse) et le dit.
- **D13 — Idempotence : `IF [NOT] EXISTS` (prouvé sur MariaDB 10.11), repli `tracked-by-sqlx`.** Le DDL MariaDB n'est pas
  transactionnel : un échec **entre** les deux `ALTER TABLE` laisse la première table migrée et la migration non inscrite ; le rejeu
  échouerait en 1060 (colonne déjà là). La forme **`ADD COLUMN IF NOT EXISTS`, `DROP CONSTRAINT IF EXISTS`, `ADD CONSTRAINT IF NOT
  EXISTS`** rend le rejeu **sans effet** (verdict `yes` à l'audit). **Sonde de l'agent de rédaction (2026-11-18, MariaDB `10.11.19-MariaDB-ubu2204` du conteneur de dev, schéma jetable
  `kesh_e491_probe`, supprimé ensuite)** : un `ALTER TABLE` unique — `ADD COLUMN IF NOT EXISTS line_type VARCHAR(16) NOT NULL DEFAULT
  'priced'`, `DROP CONSTRAINT IF EXISTS chk_t_quantity_positive`, trois `ADD CONSTRAINT IF NOT EXISTS … CHECK` dont l'une référence la
  colonne ajoutée dans le même `ALTER` — s'applique sur une table à deux lignes, **se rejoue sans erreur** ; les lignes existantes sont
  `priced` ; `COLUMN_DEFAULT` rend `'priced'` ; un commentaire neutre s'insère ; un commentaire à `quantity = 1` est refusé (`ERROR 4025
  (23000) : CONSTRAINT chk_t_unpriced_neutral failed`). **T0 du développeur** : rejouer cette sonde sur le MariaDB de son worktree
  (version citée au Change Log) ; si l'une des formes est refusée, retomber sur `tracked-by-sqlx` (erreurs 1060 / 1061 / 1091
  nommées) et écrire la reprise manuelle en en-tête. *Écartée* : deux
  `ALTER` par table (la fenêtre de panne grandit).
- **D14 — La colonne `line_type` se place en dernier dans les exports CSV** (après `created_at`), pour ne décaler **aucune**
  colonne existante (précédent : la 37-3a, `docs/api-external.md:1084`, « un lecteur qui lit par position doit être adapté » : ici
  personne n'a à l'être). MCP : un champ `type_ligne` **additif**.

## Critères d'acceptation

**Migration — la 96ᵉ (D2 à D6, D12, D13)**

- **AC1** — `crates/kesh-db/migrations/20261119000001_invoice_line_types.sql` (horodatage **postérieur** à
  `20261118000001_grouped_settlements.sql` ; s'il est écrit un autre jour, le nom suit ce jour), **DDL seul**, **un `ALTER TABLE`
  par table** (D13), dans cet ordre : `invoice_lines` puis `credit_note_lines` ; chacun porte `ADD COLUMN [IF NOT EXISTS] line_type
  VARCHAR(16) NOT NULL DEFAULT 'priced'`, le `DROP CONSTRAINT [IF EXISTS]` de `chk_*_quantity_positive` et les trois `ADD
  CONSTRAINT [IF NOT EXISTS]` de D4. **En-tête** : (a) non breaking au sens de P1 — « sans relever `kesh_version_min_required` et
  sans corrompre le schéma » (citer) ; (b) `DROP CONSTRAINT` n'est pas une opération de P3 ; (c) **le SQL qui défait la migration**
  (P1, « le retour en arrière passe par la sauvegarde… ou par le SQL qui défait la migration »), écrit en **commentaires `--`** (jamais
  `/* */`, que refuse `migrations_contain_no_block_comment`, `post_restore.rs:1420` ; `split_statements` ne retire que les `--`), **pour les
  deux tables** : *1°* (valable **seulement** si `SELECT COUNT(*) FROM invoice_lines WHERE line_type <> 'priced'` et le même sur
  `credit_note_lines` valent 0) `ALTER TABLE invoice_lines DROP CONSTRAINT chk_invoice_lines_unpriced_neutral, DROP CONSTRAINT
  chk_invoice_lines_priced_quantity, DROP CONSTRAINT chk_invoice_lines_line_type;` — **un `ALTER` qui retire d'abord les contraintes** (la
  contrainte neutre est multi-colonnes et référence `line_type`) ; *2°* `ALTER TABLE invoice_lines DROP COLUMN line_type, ADD CONSTRAINT
  chk_invoice_lines_quantity_positive CHECK (quantity > 0);` ; *3°* les deux mêmes `ALTER` sur `credit_note_lines` (`chk_credit_note_lines_…`) ;
  *4°* `DELETE FROM _sqlx_migrations WHERE version = 20261119000001;`. Le retour est **sondé à T0** comme la montée (T0 e) ; (d) la règle de typage des lignes existantes (D1) et pourquoi aucun `UPDATE` (D2) ; (e) le type sous-total absent et pourquoi (D5).
  **`kesh_version_min_required` inchangé** (0.17.6, vérifié : `git diff -- crates/kesh-db/migrations` ne contient aucun
  `UPDATE _kesh_version`) ; aucun `Cargo.toml` ne bouge.
- **AC2** — **Aucune facture ni aucun avoir ne change.** Sur une base peuplée par le schéma de la 0.19.0 (montée par
  `common::migrations_before(20261119000001, "invoice_line_types")`, qui applique tout **sauf** la migration neuve — donc `grouped_settlements`
  comprise, que le règlement groupé exige) — **factures validées payée, impayée, annulée par un avoir, avoir émis,
  facture réglée dans un règlement groupé**, avec 1, 3 et 8 lignes, deux taux de TVA et un arrondi à 5 centimes —, la migration
  appliquée : chaque ligne de `invoice_lines` et de `credit_note_lines` est **identique colonne à colonne** (empreinte des colonnes
  d'avant), `line_type = 'priced'` partout, `invoices.total_amount`, `rounding_amount`, `credit_notes.total_amount`, les écritures
  (`journal_entry_lines`), les numéros, `paid_at`, le TTC et le reste dû (macros `INVOICE_TTC_SUBQUERY_SQL`, `INVOICE_AMOUNT_DUE_*`)
  **inchangés**. Les contraintes existantes (`unit_price >= 0`, plage de `vat_rate`, `line_total >= 0`, description non vide,
  `uq_*_position`) **sont toujours là** (lues dans `information_schema.CHECK_CONSTRAINTS`).
- **AC3** — **Registres et garde-fous** : (a) `docs/migrations-idempotence-audit.md` : **une ligne** pour `20261119000001_…`
  (`yes` si D13 tient, sinon `tracked-by-sqlx` avec les erreurs 1060 / 1061 / 1091), totaux à **96** aux **deux** sites (en-tête
  `## Table d'audit (96 migrations)`, ligne `Total`, dont la ventilation gagne « + 1 Story 49-1a »), compteurs de partition
  recomptés (`yes + tracked-by-sqlx + no = 96`) ; (b) **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` relancé,
  chaque site inspecté, relevé au Change Log — **couplage positionnel avec garde** : `migrations_upgrade_path.rs:137`
  `assert_eq!(total, 95, …)` → **96** (message : « 95 précédentes + Story 49-1a »), et `:237` `let n_before_upgrade_window = total - 61;`
  → **`total - 62`** (frontière inchangée : 96 − 62 = 34), avec la ligne d'historique (« Story 49-1a : 61 → 62, frontière
  inchangée ») ; les autres sites résolvent **par version** (`common::migrations_before`, `pre_upgrade_backup/tests.rs`,
  `sauvegarde_avant_migration.rs`) ; (c) **P7** : `every_data_backfill_migration_is_triaged` **vert sans nouvelle entrée** à
  `POST_RESTORE_BACKFILLS` ni `EXEMPT_MIGRATIONS` ; la justification (« DDL seul ; le `DEFAULT` rétro-remplit ») va au Change Log et
  au tableau d'audit ; (d) **squash du schéma de test régénéré** par `scripts/regen-test-schema.sh` (jamais édité à la main),
  `test_schema_guard.rs` vert ; (e) **P8** : la migration n'est modifiée par personne après son premier démarrage ; un démarrage
  sur une base **persistante** (E2E ou `cargo run`) est joué avant de clore (le gate complet ne voit pas P8) ; (f)
  `account_columns_inventory.rs` (colonnes `*.revenue_account_id`, `:225`, `:234`) relu : l'inventaire **ne change pas**.
- **AC4** — **Sauvegarde, import, export de souveraineté** : (a) une sauvegarde **0.19.0** (manifeste sans `line_type`,
  `columnNames` des deux tables sans la colonne) s'importe, **chaque ligne de facture et d'avoir est `'priced'`** et identique aux
  valeurs d'avant (même assertion qu'AC2, par l'import) ; (b) une sauvegarde **0.20.0** portant une ligne `comment` fait l'aller-retour
  (export → import) sans perte ni refus de contrainte ; (c) `backup_inventory_matches_schema` et `export_table_serializes_rows_faithfully`
  restent verts **sans changement de code** de `backup.rs` (les colonnes se lisent par introspection) — **sinon la prémisse est
  fausse et le développeur s'arrête** ; (d) l'export global (AC13) porte la colonne.

**Entités et dépôts (D1, D7, D10, D11)**

- **AC5** — `enum LineType { Priced, Comment, Heading }` (`kesh-db/src/entities/`), `as_str()` / `FromStr` / sérialisation
  `"priced" | "comment" | "heading"` (camelCase des DTO inchangé) ; `InvoiceLine.line_type`, `NewInvoiceLine.line_type`,
  `CreditNoteLine.line_type` ; **une seule liste de colonnes par table** : `invoices::LINE_COLUMNS` (`:47`) gagne `line_type`, et les
  trois listes en dur de `credit_note_lines` (`credit_notes.rs:66-76`, `:98-104`, `:919-935`) ainsi que celle d'`invoices.rs:3050-3053`
  passent par **une constante partagée** (`CREDIT_NOTE_LINE_COLUMNS`, et `LINE_COLUMNS` préfixée pour la forme jointe) — il n'y a plus
  qu'**une seule liste à tenir** (un `SELECT` sans `line_type` échouerait encore à l'exécution, `FromRow` : la constante centralise, elle ne fait pas
  voir l'oubli à la compilation). Les **67** littéraux `NewInvoiceLine {` et les **4**
  `InvoiceLine {` sont complétés (`line_type: LineType::Priced`) ; **aucun test existant n'est modifié autrement que ceux que nomment AC3 (b) et AC13 (a)**. `kesh-seed`
  (`pieges.rs:579`) compile.
- **AC6** — `insert_lines` (`invoices.rs:471`) écrit `line_type` ; pour un type non chiffré, `line_total` n'est **pas** calculé
  (`compute_line_total` rend `0`) et `compute_total` (`:464`) ne compte que les lignes `priced` ; une ligne `comment` ou `heading`
  qui porte `quantity`, `unit_price`, `vat_rate` ou `revenue_account_id` non neutres → `DbError::Invariant` (D10). Une facture dont
  **toutes** les lignes sont non chiffrées **se crée** (brouillon) et **ne se valide pas** (`invoiceTotalZero`, `invoices.rs:2186`).
  `explicit_line_account_sites` (`:674`) ne considère que les lignes `priced` (le rang `idx + 1` reste celui de la liste entière).
- **AC7** — `is_no_op_change` (`invoices.rs:1067`) **compare `line_type`** : changer seulement le type d'une ligne est une
  modification (la `version` s'incrémente, l'audit s'écrit) ; `invoice_snapshot_json` (`:77`) porte `"lineType"` dans chaque ligne
  du « before » et du « after » (le snapshot d'audit de l'avoir, `credit_notes.rs:55-62`, de même).
- **AC8** — **`validate_invoice` ignore les lignes non chiffrées** (D3, le piège de `:2312`) : (a) `lines_before.iter().any(…
  revenue_account_id.is_none())` (`:2269`, `:2312`) et la boucle de mutation en mémoire (`:2335-2337`) ne portent que sur les lignes
  `priced` ; (b) l'`UPDATE invoice_lines SET revenue_account_id = ? WHERE invoice_id = ? AND revenue_account_id IS NULL` (`:2314`)
  gagne `AND line_type = 'priced'` — **sans quoi il viole `chk_invoice_lines_unpriced_neutral` : 500 SQL** ; (c) les lignes
  de l'écriture de vente d'une facture portant des commentaires sont **identiques** (comptes, montants, ordre) à celles de la même
  facture sans eux ; (d) le compte de produit par défaut n'est **pas** exigé quand toutes les lignes `priced` portent un compte
  explicite (une ligne de commentaire ne le fait pas exiger) ; (e) `ValidatedInvoice.lines` rend toutes les lignes, types compris ;
  (f) l'audit « after » est cohérent (`lineType` des commentaires, compte `null`).
- **AC9** — **L'avoir reprend les lignes non chiffrées** (D7) : (a) le snapshot (`credit_notes.rs:771-791`) copie `line_type` de
  chaque ligne de la facture, **toutes**, à leur `position` ; (b) `triplets` (`:499`), les comptes à verrouiller (`:510-545`) et le
  test d'« absence de compte » (`:632`) ne considèrent que les lignes `priced` ; (c) l'écriture de contre-passation est **identique**
  à celle de la même facture sans commentaires ; (d) `creditNoteTotalZero` (`:610`) inchangé ; (e) `get` et `list_all_lines_by_company`
  rendent `line_type` ; (f) la reprise vaut pour l'**avoir créé après la montée sur une facture d'avant la montée** : toutes ses lignes
  sont `priced`, l'avoir est identique à celui que la 0.19.0 aurait émis (test : mêmes lignes, même écriture).

**Lecteurs (inventaire)**

- **AC10** — (a) **Rapport TVA** : `kesh-report/src/vat_report.rs:121` filtre `AND il.line_type = 'priced'` ; une facture validée
  portant des commentaires donne **exactement** les mêmes lignes, la même base et la même TVA que sans eux, **sans ligne « 0 % »
  fantôme** (comparaison des `VatReport` entiers, et du `reconciliation_delta`) ; (b) **parité TTC** : `invoice_amount_due_parity`
  et `invoice_ttc_parity` rejouent une facture et un avoir portant une ligne de commentaire et un titre (créés **par le dépôt**) :
  TTC, crédité, réglé et reste dû **identiques** dans les formes SQL et en Rust (`kesh-core`) ; (c) `invoice_settlements_write.rs:647`
  gagne `AND line_type = 'priced'` (la part de TVA d'une perte ou d'un escompte est identique).

**API gardée (D8, D9)**

- **AC11** — `CreateInvoiceLineRequest` (`routes/invoices.rs:69`) : `line_type: LineType` (`#[serde(default)]`, défaut `Priced`),
  `quantity`, `unit_price`, `vat_rate` en `Option<Decimal>` (`#[serde(default)]`). `validate_line` (`:475`), par type : `priced` —
  exactement les règles d'aujourd'hui (messages inchangés), plus « Ligne N : la quantité / le prix unitaire / le taux de TVA est
  obligatoire » ; `comment` et `heading` — description obligatoire et bornée comme aujourd'hui, `quantity`/`unit_price`/`vat_rate`
  **absents ou égaux à zéro**, `revenueAccountId` absent ou `null`, sinon `400 VALIDATION_ERROR` nommant la ligne et le champ ;
  **puis la garde (D9)** : `LineType::open_to_api()` faux → `400 VALIDATION_ERROR`, « Ligne N : le type de ligne … n'est pas encore
  disponible », **aucune écriture**, au `POST` **et** au `PUT`. **Un corps de 0.19.0** (sans `lineType`, avec les quatre champs) est
  accepté et rend la **même réponse** qu'avant, au champ `lineType` près. Un `lineType` inconnu rend `422` (rejet de l'extracteur), dit tel quel à `docs/api-external.md`. `verify_vat_rates_against_db` (`:680`, `:773`) : voir l'inventaire (49-1c) ; ici, un test unitaire vérifie que `validate_lines` rejette un type non ouvert **avant** qu'aucun taux ne soit collecté.
- **AC12** — `InvoiceLineResponse` (`:150`) et `CreditNoteLineResponse` (`credit_notes.rs:33`) gagnent `lineType` (`"priced"`).
  Réponses de `POST`, `PUT`, `GET`, `validate`, `unvalidate`, `settlements` et de l'avoir : **le seul ajout est `lineType`**.
  `routes/invoice_email.rs`, `invoice_pdf_service.rs` et `credit_notes.rs:307` (PDF) **ne changent aucune ligne de rendu** (hors périmètre, 49-1b) ; **seule exception nommée** : le littéral de test `InvoiceLine {` de `invoice_pdf_service.rs:1251` (`#[cfg(test)]`), qui reçoit `line_type: LineType::Priced`.

**Exports, MCP, documentation**

- **AC13** — (a) `serialize_invoice_lines_csv` (`csv_tables.rs:623`) et `serialize_credit_note_lines_csv` (`:1271`) ajoutent
  `line_type` **en dernière colonne** (D14) ; `exports_global_e2e` et `admin_full_export_e2e` (`("invoice_lines.csv", 0)` à `exports_global_e2e.rs:991` est un *nombre de lignes* attendu, non un en-tête : **aucune assertion d'en-tête de
  `invoice_lines.csv` n'existe, T10 l'écrit**) sont mis à jour ; (b) MCP : `LigneFacture.type_ligne` (`mcp/outils/factures.rs:237`, `:487`)
  — additif, le MCP de 0.19.0 n'est pas contractuellement figé sur les champs (à confirmer au développement : un test d'exactitude
  du schéma, s'il existe, est mis à jour) ; (c) `docs/api-external.md` : `lineType` documenté sur la création et la modification
  d'une facture (défaut `priced`, valeurs `comment` et `heading` **réservées**, refusées `400` jusqu'à leur ouverture), le champ
  des réponses, la colonne des exports ; **le changement de contrat `422` → `400 VALIDATION_ERROR`** (champ manquant sur une ligne chiffrée), en « ⚠️ » sur le modèle de `:1121`, et la mention qu'un `lineType` inconnu rend `422` ; la phrase « les lignes d'une facture client portent un type depuis la v0.20.0 » ; (d)
  **aucun manuel utilisateur** n'est touché (l'écran ne change pas — `git grep -n "ligne de facture" -- docs/manual/fr/user-manual.tex`
  ne rend que `:515` (comptes imputables), qui ne décrit aucun type de ligne ; le manuel d'administration est la 49-1d).

**Non-régression**

- **AC14** — **Aucune ligne de rendu n'est changée** : `git diff origin/epic/49-types-de-lignes-facture..HEAD -- crates/kesh-qrbill
  crates/kesh-api/src/routes/issued_invoice_pdf.rs` est **vide** ; pour `crates/kesh-api/src/routes/invoice_pdf_service.rs`, le diff se
  **limite au module `#[cfg(test)]`** — le seul ajout permis est `line_type: LineType::Priced` au littéral `InvoiceLine {` de `:1251`
  (exception nommée : l'entité n'a pas `Default`, AC5 l'oblige) ; `invoice_pdf_e2e`,
  `invoice_frozen_pdf_e2e`, `invoice_frozen_pdf` (dépôt), `invoice_send_email_e2e` **verts sans modification**.
- **AC15** — **Gates** : gate complet (`fmt`, `build`, `clippy -D warnings`, `cargo nextest`/`gate-ci.sh`) **vert**, base remise à zéro
  avant ; E2E Playwright **complet** au dernier commit de code, « vert » au sens de `docs/testing.md` § *Les échecs attendus*
  (sept échecs de la KF-029) ; **un démarrage sur base persistante** (P8) ; frontend : `npm run check`, `lint-i18n-ownership`,
  `test:unit`, `build` verts **sans modification du frontend** (si le moindre fichier du frontend change, la fiche est fausse :
  l'écrire au Change Log).

## Tests à écrire, et la preuve du rouge

Chaque test neuf **rougit sans le correctif** : le journal attendu est dans `kesh-gate-logs/e491a/` (un fichier `rouge-<test>.log` par mutation).

| # | Test | Ce qu'il prouve | Mutation / retrait qui doit le rougir |
|---|---|---|---|
| T1 | `migrations_upgrade_path.rs::line_types_migration_keeps_real_invoices` (AC2) | base 0.19.0 peuplée (payée, impayée, annulée + avoir, groupée, 1/3/8 lignes, 2 taux, arrondi) ; empreinte d'avant = d'après sur les colonnes d'avant ; `line_type = 'priced'` partout ; totaux, TTC, reste dû, écritures inchangés ; contraintes existantes présentes | ajouter un `UPDATE invoice_lines SET unit_price = unit_price` à la migration ; mettre `DEFAULT 'comment'` ; oublier `credit_note_lines` |
| T2 | `kesh-db/tests/invoice_line_types.rs::constraints_by_type` (AC1, D4), pour **les deux tables** | `priced` : `quantity = 0` refusée, `quantity > 0` admise ; `comment` : `quantity = 1`, `unit_price = 1`, `vat_rate = 7.7`, `line_total = 1`, `revenue_account_id` posé, description vide **refusés** ; `heading` neutre admis ; `line_type = 'subtotal'` **refusé** ; **un `INSERT` de forme 0.19.0 (sans `line_type`, formes de `invoices.rs:480-482` et `credit_notes.rs:774-777`) passe après la migration et donne `priced`** (V1-13) ; description non vide **toujours** exigée pour un commentaire | retirer **chacune** des trois contraintes neuves, et remettre `chk_*_quantity_positive` : un test rougit par mutation (quatre journaux par table) |
| T3 | `admin_full_import_e2e.rs::import_sauvegarde_0_19_0_donne_les_memes_lignes` et `…_aller_retour_ligne_de_commentaire` (AC4) | manifeste sans `line_type` → `'priced'` ; manifeste 0.20.0 avec `comment` → identique | `NOT NULL` **sans** `DEFAULT` : l'import 0.19.0 échoue (`is_required`) |
| T4 | `kesh-db/tests/invoice_line_types.rs::draft_roundtrip_and_no_op` (AC6, AC7) | création et remplacement d'un brouillon avec un commentaire entre deux lignes chiffrées : positions `0..n`, total inchangé ; un `update` qui ne change **que** le type n'est **pas** un no-op ; audit « before / after » porte `lineType` ; ligne `comment` non neutre → `Invariant` | retirer `b.line_type == c.line_type` de `is_no_op_change` ; retirer le `lineType` du snapshot |
| T5 | `kesh-db/tests/invoices_validate_vat.rs::validation_ignores_unpriced_lines` (AC8) | écriture identique avec et sans commentaire (comptes, montants, ordre) ; le compte du commentaire reste `NULL` après validation ; compte par défaut **non exigé** si les lignes chiffrées ont un compte explicite ; pièce de commentaires seuls → `invoiceTotalZero` | **trois mutations, deux jeux** : jeu (1) une ligne chiffrée à compte `NULL` **plus** un commentaire → l'`UPDATE` s'exécute, le commentaire reste `NULL` ; tue le retrait de `AND line_type = 'priced'` de `:2314` (**500 par `chk_…_unpriced_neutral`**) ; jeu (2) lignes chiffrées à compte explicite **plus** un commentaire, **compte par défaut archivé** → la validation passe (le compte par défaut reste exigé *configuré*, `invoices.rs:2169-2171`, mais n'est re-validé que s'il est effectivement posté) ; tue le retrait du filtre du `any(…is_none())` (`:2269`, `:2312`) ; la boucle en mémoire (`:2335-2337`) est tuée par AC8 (e, f) |
| T6 | `kesh-db/tests/credit_notes_repository.rs::credit_note_copies_unpriced_lines` (AC9) | l'avoir d'une facture à commentaires : lignes **toutes** copiées, types et positions ; écriture identique à celle sans commentaires ; `get` rend `line_type` ; avoir d'une facture **d'avant la montée** : mêmes lignes, même écriture | ne pas copier `line_type` (`INSERT` sans la colonne → `priced` : l'assertion de type rougit) ; copier le commentaire avec un compte |
| T7 | `kesh-report/tests/vat_report_reconciliation.rs::vat_report_ignores_unpriced_lines` (AC10 a) | rapport identique avec / sans commentaire ; **aucune ligne « 0 % »** ; `reconciliation_delta` identique | retirer `AND il.line_type = 'priced'` : la ligne `0 %` apparaît |
| T8 | `invoice_amount_due_parity.rs`, `invoice_ttc_parity.rs` : cas « avec commentaire et titre » (AC10 b) | TTC, crédité, réglé, reste dû identiques (SQL ↔ `kesh-core`) | remplacer les valeurs neutres d'un commentaire par `unit_price = 1` (test de contre-épreuve du motif de neutralité) |
| T9 | `routes/invoices.rs` (tests unitaires de `validate_line`) + `kesh-api/tests/invoice_line_types_e2e.rs` (AC11, AC12) | par type et par champ (tableau) ; **corps de la 0.19.0 identique** accepté, réponse inchangée hors `lineType` ; `comment` et `heading` refusés `400` au `POST` **et** au `PUT`, **zéro écriture** (comptage de lignes, `version` de la facture inchangée) ; l'omission de `quantity` d'une ligne chiffrée → `400` nommant la ligne | `open_to_api()` rendant `true` pour `comment` : le test de refus rougit ; retirer l'`Option` : la désérialisation du commentaire échoue |
| T10 | `exports_global_e2e.rs`, `admin_full_export_e2e.rs` (AC13) | `line_type` en dernière colonne des deux CSV ; valeurs `priced` ; une ligne `comment` créée par le dépôt s'exporte | retirer la colonne de l'en-tête **ou** de la ligne : l'écart d'arité rougit |
| T11 | `mcp_outils_e2e.rs` (AC13 b) | `type_ligne` présent, `priced` | retirer le champ |
| T12 | `post_restore.rs` : `every_data_backfill_migration_is_triaged` (AC3 c), `test_schema_guard.rs` (AC3 d), `reset_partition_covers…` | verts **sans nouvelle entrée** ; le squash porte la colonne et les trois contraintes | ajouter un `UPDATE` à la migration : `every_data_backfill…` rougit (prouve que la migration est bien du DDL seul) |
| T13 | `invoice_pdf_e2e`, `invoice_frozen_pdf_e2e`, `invoice_frozen_pdf`, `invoice_send_email_e2e` (AC14) | **inchangés et verts** : le rendu n'a pas bougé | (épreuve, non test neuf) |

**E2E Playwright** : la story ne change **aucun** écran ; **aucun E2E neuf**, motif : le défaut se voit à la frontière HTTP, joué par
`invoice_line_types_e2e` (T9), et la 49-1c joue le parcours dans le navigateur. La **suite complète** tourne tout de même au dernier
commit de code (AC15) : elle prouve que l'écran de la 0.19.0 crée, modifie, valide et solde toujours une facture sur le schéma migré.

**Recette visuelle** (`CLAUDE.md`) : sans objet — aucun contrôle visible n'est ajouté ni modifié (dit au Dev Agent Record).

## Tâches

- [ ] **T0 — Étape 0, avant d'écrire.** (a) Relire les lignes citées de § *Ce qui se lit au code* à la tête du worktree
  (`git -C /home/gcorbaz/devel/kesh-49-1 log -1 --format=%H` : `773df80da` ou son descendant dans l'épique) ; une ligne qui a bougé se
  déclare au Change Log. (b) **Rejouer la sonde de D13** sur le MariaDB de dev (un `ALTER` unique par table, deux fois, trois formes `IF [NOT] EXISTS`) ; coller la sortie.
  (c) Rejouer que `ADD CONSTRAINT … CHECK` référençant la colonne **ajoutée dans le même `ALTER`** valide les lignes existantes
  (`quantity > 0`, `unit_price >= 0`) sans erreur. (d) Relever que `information_schema.COLUMNS.COLUMN_DEFAULT` rend `'priced'` (donc
  `has_default`, `backup.rs:795`). (e) **Sonder le retour (V1-5)** sur un schéma jetable `kesh_e491_*` (supprimé ensuite) : montée, retour par le SQL de l'en-tête (AC1 c) sur les deux tables, vérifier que les contraintes d'origine sont revenues (`information_schema.CHECK_CONSTRAINTS`) et qu'un `INSERT` de forme 0.19.0 passe, puis **remontée** ; si le `ALTER` du *1°* échoue, le scinder et le dire au Change Log. Si l'une des trois ne se rejoue pas, **s'arrêter et le dire** (rapport).
- [ ] **T1 — La migration** (AC1) et son audit (AC3 a), `regen-test-schema.sh` (AC3 d), P6 (AC3 b).
- [ ] **T2 — Entités, constantes de colonnes, littéraux** (AC5). Compiler d'abord (`cargo check --workspace --all-targets`) : les 67 +
  4 littéraux se trouvent au compilateur ; **un seul commit** pour cette mécanique.
- [ ] **T3 — Dépôts** : `insert_lines`, `compute_total`, `is_no_op_change`, snapshots (AC6, AC7) ; `validate_invoice` (AC8) ;
  `create_credit_note` (AC9) ; `vat_report` et `invoice_settlements_write` (AC10).
- [ ] **T4 — API** (AC11, AC12) : DTO, `validate_line` par type, `LineType::open_to_api()`.
- [ ] **T5 — Exports, MCP, `docs/api-external.md`** (AC13).
- [ ] **T6 — Tests** T1 à T12 ; **journaux de rouge** dans `kesh-gate-logs/e491a/`.
- [ ] **T7 — Gates** : `scripts/test-fast.sh` en cours de boucle (`kesh-db` touché : **gate complet** dès le premier patch de dépôt,
  `CLAUDE.md` § *Exception `kesh-db`*), base remise à zéro avant le gate complet, E2E complet au dernier commit de code ; démarrage
  sur base persistante (P8). **Le story file n'affirme que ce qui a tourné.**
- [ ] **T8 — Dev Agent Record, Change Log, statuts** ; ne **pas** toucher `sprint-status.yaml` (l'orchestrateur tient les statuts).

## Note de version

*(Pour le CHANGELOG, assemblée à l'intégration ; la story n'écrit pas `CHANGELOG.md`.)*

> **Migration : sauvegarde complète obligatoire avant la mise à jour** (base, `.keshbackup` et dossier des documents). Kesh sait
> désormais, au niveau du schéma, distinguer une ligne chiffrée d'une ligne de texte seul (commentaire, titre de section) sur les
> factures et les avoirs ; **chaque facture et chaque avoir existants restent identiques**, ligne pour ligne. Aucune ligne de texte
> seul ne peut encore être saisie : leur écran et leur impression arrivent avec les stories suivantes de #1242. L'API des factures
> accepte un champ optionnel `lineType` (défaut `priced`) ; ses valeurs `comment` et `heading` sont refusées pour l'instant, et un type inconnu rend `422`. **Changement de contrat de l'API** : un champ `quantity`, `unitPrice` ou `vatRate` manquant sur une ligne chiffrée rend désormais `400 VALIDATION_ERROR` (nommant la ligne), non plus le `422` de l'extracteur. Les
> exports CSV des lignes de facture et d'avoir gagnent la colonne `line_type`, en dernière position. (refs #1242)

## Dev Notes

- **Couplage 49-1a / 49-1d** : la 49-1a **ne sort pas sans la 49-1d** (V1-4) ; la 49-1d garde la recette et le manuel d'administration ×4.
- **Hors périmètre, avec la story qui le prend** : rendu PDF (facture, avoir) → **49-1b** ; écran, ouverture de `comment`, manuel
  utilisateur ×4, i18n → **49-1c** ; gate de recette, manuel d'administration « Passer à la 0.20.0 » ×4 → **49-1d** ; réordonnancement
  → **49-2** ; titre de section → **49-3** ; sous-totaux → **#1330** (« Plus tard »).
- **Fichiers communs** (tableau du fichier d'épique) : `crates/kesh-qrbill/src/pdf.rs`, `types.rs` (PR #1327 de Beat) **non touchés** par
  cette story ; `InvoiceForm.svelte` (48-13) non touché ; catalogues i18n non touchés ; `sprint-status.yaml` non touché ; compteurs
  à recompter **depuis la source après la fusion** : totaux de `docs/migrations-idempotence-audit.md` (96), `total` et `total - 62`
  de `migrations_upgrade_path.rs`, schéma de test.
- **Ordre de fusion** : la première de la 0.20.0 qui porte une migration ; **aucune autre migration n'entre dans la release**
  (décision de Guy, 2026-11-18). Un contributeur dont la migration partirait de `main` porte un horodatage postérieur et s'annonce.
- **Ce qui doit rester vrai** : (1) une ligne `priced` se crée, s'enregistre, se valide, se solde et se rend **comme avant** ; (2)
  `kesh_version_min_required` = 0.17.6 ; (3) la 49-1b et la 49-1c s'écrivent contre un dépôt qui accepte les trois types et une API
  qui n'en ouvre qu'un ; (4) la garde `open_to_api()` est **la seule** chose qui empêche un commentaire de s'imprimer `0 × 0.00` : un
  retrait de la garde avant la 49-1b est une régression visible (test T9 le garde) ; (5) le retour à la 0.19.0 passe par la
  sauvegarde d'avant la mise à jour, ou par le SQL de l'en-tête de la migration (P1).
- **Splitting (V1-15)** : la 49-1a compte **cinq** modules (`kesh-db`, `kesh-api`, `kesh-report`, `kesh-seed`, `docs/`), **au plafond** de la règle (conforme à la lettre : « plus de cinq ») ; c'est la plus grosse des quatre parties (15 AC, 13 tests). Si la passe 2 de validation recycle ses défauts, le signal se **déclare au Project Lead** (`CLAUDE.md` § *Règle de splitting préventif*).
- **Pièges à connaître** : (a) l'`UPDATE` de matérialisation du compte (`invoices.rs:2314`) est le piège central — la contrainte
  neutre le transformerait en 500 à la **validation** d'une facture à commentaire ; (b) `quantity = 0` sur un commentaire ne
  doit **jamais** atteindre un diviseur : aucun lecteur ne divise par la quantité (`git grep -n "/ .*quantity\|quantity.*/" --
  'crates/*/src/*'` à relancer à l'étape 0) ; (c) le tableau d'inventaire ci-dessus est **clos** : un site neuf découvert à l'écriture
  s'ajoute au Change Log et au tableau, avec sa résolution.
- **Aucune donnée réelle** de Guy dans la fiche, les tests ou les journaux : les factures de T1 et des gates sont **synthétiques**
  (`CLAUDE.md` de la mémoire : données réelles hors dépôt public). **`~/kesh/` est en lecture seule**, jamais lue ni écrite.
- **Gate de recette de la release (49-1d)** : cette story **ne le joue pas** ; elle fournit la migration et T1, qui en est l'épreuve
  au niveau base. La **recette de montée** avec factures simulées (payée, impayée, avoir, règlement groupé, PDF figés et non figés)
  est le gate de la 49-1d, rejoué au dernier commit de code de la release (`scripts/recette-montee.sh`).
- **Test avant / après du PDF** (consigne de production) : il est exigé **dans la story qui touche le rendu**, la 49-1b. Cette
  story garantit seulement (AC14) qu'**elle ne change aucune ligne de rendu**, de sorte que la référence « avant » de la 49-1b
  est le rendu de ce commit.
- **Références** : `epic-49-types-de-lignes-facture.md` (kickoff), `release-0.20.0.md`, #1242 et ses deux commentaires, #1330,
  `CLAUDE.md` § *Migration breaking policy* (P1 à P8) et § *Règle de splitting préventif*, modèle de fiche avec migration :
  `43-1a-reglement-groupe-modele-client.md` (colonne à `DEFAULT`, `CHECK` sur colonne neuve, P6), `20261001000004_invoice_settlements_write_off.sql`
  (précédent `DROP CONSTRAINT` + `ADD CONSTRAINT` d'un `CHECK`), `kesh-gate-logs/e491/decoupage-49-1.md`.

## Dev Agent Record

*(vide — à remplir au développement)*

## Change Log

| Date | Entrée |
|---|---|
| 2026-11-18 | Création de la fiche (workflow `bmad-create-story`), par l'agent de rédaction de la 49-1 (Sonnet 5.5) : découpage préventif de la 49-1 en 49-1a à 49-1d (`kesh-gate-logs/e491/decoupage-49-1.md`), fiche de la première partie ; code lu à `773df80da`. Écarts au fichier d'épique : trois types stockés (non quatre), pas de type sous-total (D5), `kesh-core` non touché, API **gardée**. Non encore validée. |
| 2026-11-18 | **Passe de validation P1** (Opus, contexte frais, `kesh-gate-logs/e491/validation-49-1a-p1.md` : 0 CRITICAL, 0 HIGH, 7 MEDIUM, 8 LOW). **Remédiation** (sept MEDIUM et huit LOW traités, décisions de l'orchestrateur) : V1-1 AC14 et AC12 bornés au code de rendu, exception nommée du littéral de test `invoice_pdf_service.rs:1251` ; V1-2 `verify_vat_rates_against_db` (`routes/invoices.rs:680`, `:773`) ajouté à l'inventaire, traitement pris par la 49-1c, inatteignable en 49-1a (garde) ; V1-3 changement de contrat `422` → `400 VALIDATION_ERROR` nommé (D8, AC13 c, note de version, `api-external.md`) ; V1-4 couplage « la 49-1a ne sort jamais sans la 49-1d » (fiche et découpage) ; V1-5 SQL de retour complet (deux tables, contraintes retirées avant `DROP COLUMN`) et sondé à T0 (e) ; V1-6 deux jeux de mutation pour T5 ; V1-7 pathspecs corrigés, inventaires rejoués (18 fichiers de `src`, 31 de tests, 67 littéraux dans 43 fichiers, 4 littéraux `InvoiceLine {`) ; V1-8 schéma 0.19.0 par `migrations_before(20261119000001, …)` ; V1-9 `:991` est un nombre de lignes, T10 écrit l'assertion d'en-tête ; V1-10 AC5 « hors AC3 b et AC13 a » ; V1-11 « une seule liste à tenir » ; V1-12 `lineType` inconnu = `422`, dit tel quel ; V1-13 `INSERT` de forme 0.19.0 après la migration (T2) ; V1-14 commentaires `--` obligatoires ; V1-15 cinq modules, au plafond, déclaré. Modèles : Sonnet (rédaction), Opus (passe 1). Une passe 2 reste à lancer (modèle différent). |
