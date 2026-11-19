# Story 49-1a : types de lignes de la facture client — la migration, le modèle, les dépôts et l'API gardée (#1242)

Status: review

*Fiche rédigée le 2026-11-18 par l'agent de rédaction de la 49-1 (Sonnet 5.5), par le workflow `bmad-create-story`, en découpant la
49-1 (`kesh-gate-logs/e491/decoupage-49-1.md`, règle de splitting préventif : neuf modules). Code lu à `773df80da` (v0.19.0), worktree
`kesh-49-1`, branche `story/49-1-types-de-lignes`. Release **0.20.0** (coupure jeudi 2026-11-26 au soir). **Story zéro de la 49-1 ;
porte LA migration de la release (la 96ᵉ).** **`kesh-db` touché : gate complet même en cours de boucle** (`CLAUDE.md` § *Pendant une
boucle de revue*). Décisions : fichier d'épique `epic-49-types-de-lignes-facture.md` (sur `origin/chore/kickoff-0.20.0`), décisions
de Guy du 2026-11-18 ; les choix de cette fiche sont D1 à D14. Validations P1 à P5 faites et remédiées (2026-11-18) ; une P6 reste à lancer.*

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
  la quantité est obligatoire »), au lieu du `422` de l'extracteur **natif d'Axum** (`Json(req): Json<CreateInvoiceRequest>`, `routes/invoices.rs:671` et `:762` ; `docs/api-external.md:557` le dit : « refusé par l'extracteur (`422`) »). **⚠️ Changement de contrat de l'API :
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
  ligne importée d'une sauvegarde 0.19.0. **Le fichier ne contient aucun statement d'écriture (`UPDATE`, `INSERT`, `REPLACE`, `DELETE`) hors des commentaires `--`** : le SQL de retour de l'en-tête (AC1 c), écrit en `--`, porte un `DELETE FROM _sqlx_migrations` sans effet sur le détecteur (`split_statements` retire les `--`, `sql_lexical.rs:50-58`) (le détecteur cherche large, P7). *Si* le détecteur rougit malgré tout, le développeur **s'arrête**
  (la prémisse de D2 est fausse) et le dit.
- **D13 — Idempotence : `IF [NOT] EXISTS` (prouvé sur MariaDB 10.11), repli `tracked-by-sqlx`.** Le DDL MariaDB n'est pas
  transactionnel, et sqlx inscrit la migration **avant** son script (`success = 0`, validée implicitement par le premier `ALTER`) :
  un échec **entre** les deux `ALTER TABLE` laisse la première table migrée et une ligne `success = 0` ; Kesh refuse de redémarrer
  (`MigrateError::Dirty`). *(Corrigé après la revue P1, E1 : la rédaction d'origine disait « la migration non inscrite ».)* Reprise :
  `DELETE FROM _sqlx_migrations WHERE version = 20261118000002 AND success = 0`, puis redémarrer ; sans les formes gardées, le rejeu
  échouerait alors en 1060 (colonne déjà là). La forme **`ADD COLUMN IF NOT EXISTS`, `DROP CONSTRAINT IF EXISTS`, `ADD CONSTRAINT IF NOT
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
  personne n'a à l'être). MCP : un champ `type_ligne` (clé JSON `typeLigne`) **toujours présent** sur chaque ligne de la sortie `kesh_facture` (`"priced"`, `"comment"`, `"heading"`) — **un ajout de clé pour toute facture, existante comprise**, non cassant (un consommateur qui ignore les clés inconnues n'est pas touché) ; la macro `sortie!` n'admet aucun attribut de champ (`mcp/sortie.rs:90-114`, Ma26) : **aucune omission, aucune modification de la macro `sortie!`** ; `type_ligne: LineType` exige en revanche une ligne dans la liste `champ_enum!` de `sortie.rs:132` (`kesh_db::entities::LineType`, énumération sans champ : `sortie.rs` est modifiable, seule la macro ne l'est pas ; l'oubli est une erreur de compilation, jamais silencieuse).

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
  comprise, que le règlement groupé exige) **peuplée par des `INSERT` SQL directs de forme 0.19.0 (sans `line_type`), jamais par le dépôt** (le dépôt de la 0.20.0 écrit `line_type` : `1054 Unknown column` sur ce schéma, piège documenté à `invoice_lines_revenue_account_backfill.rs:1195-1199`) — **factures validées payée, impayée, annulée par un avoir, avoir émis,
  facture réglée dans un règlement groupé**, avec 1, 3 et 8 lignes, deux taux de TVA et un arrondi à 5 centimes, **dont au moins une facture validée au PDF figé** (`pdf_storage_path`, `pdf_sha256` de 64 caractères, `pdf_frozen_at`, `pdf_language = 'FR'` posés par `INSERT` direct, valeurs conformes à la `CHECK` tout-ou-rien de `20261003000001_invoices_frozen_pdf.sql:30-34`) **et au moins une non figée** — c'est le cas des factures réelles de Guy, remises sur papier, dont le PDF se fige au premier téléchargement —, la migration
  appliquée : chaque ligne de `invoice_lines` et de `credit_note_lines` est **identique colonne à colonne** (empreinte des colonnes
  d'avant), `line_type = 'priced'` partout, `invoices.total_amount`, `rounding_amount`, `credit_notes.total_amount` — **empreinte de toutes les colonnes d'`invoices`, de `credit_notes` et des deux tables de lignes d'avant**, dont les colonnes du PDF figé (`pdf_storage_path`, `pdf_sha256`, `pdf_frozen_at`, `pdf_language`, `20261003000001_invoices_frozen_pdf.sql:25-28`) —, les écritures
  (`journal_entry_lines`), les numéros, `paid_at`, le TTC et le reste dû (macros `INVOICE_TTC_SUBQUERY_SQL`, `INVOICE_AMOUNT_DUE_*`)
  **inchangés**. Les contraintes existantes (`unit_price >= 0`, plage de `vat_rate`, `line_total >= 0`, description non vide,
  `uq_*_position`) **sont toujours là** (les `CHECK` lus dans `information_schema.CHECK_CONSTRAINTS`, **`uq_*_position` dans `information_schema.TABLE_CONSTRAINTS`** (`CONSTRAINT_TYPE = 'UNIQUE'`), où il se trouve).
- **AC3** — **Registres et garde-fous** : (a) `docs/migrations-idempotence-audit.md` : **une ligne** pour `20261119000001_…`
  (`yes` si D13 tient, sinon `tracked-by-sqlx` avec les erreurs 1060 / 1061 / 1091), totaux à **96** aux **deux** sites (en-tête
  `## Table d'audit (96 migrations)`, ligne `Total`, dont la ventilation gagne « + 1 Story 49-1a »), compteurs de partition
  recomptés (`yes + tracked-by-sqlx + no = 96`) ; (b) **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` relancé,
  chaque site inspecté, relevé au Change Log — **couplage positionnel avec garde** : `migrations_upgrade_path.rs:137`
  `assert_eq!(total, 95, …)` → **96** (message : « 95 précédentes + Story 49-1a »), et `:237` `let n_before_upgrade_window = total - 61;`
  → **`total - 62`** ; **la valeur greppée**, par `git grep -nE "\b95\b|\b61\b" -- crates/kesh-db/tests/migrations_upgrade_path.rs` rejoué au développement (sortie au Change Log) : `:49` (`total - 61`, message de l'aide), `:51` (`total == 95`), `:68-69` (« `total - 61` migrations », « les **61** »), `:135`, `:138-139` (`total, 95`, message), `:142`, `:150`, `:157` (« les 61 restantes »), `:159` (« le laisser à 60 », à lire 61) et `:160` (`total = 95`, à lire 96), `:237` et `:240` (`.expect("apply_migrations_up_to(total - 61) failed")`), `:309` (« les 61 migrations restantes »), tous réécrits au même patch, **lignes d'historique datées exceptées** (`:100`, `:101`, `:166`, `:167`, `:228`, `:229` en sont ; `:162`, historique lui aussi, ne sort pas de la commande) ; **hors historique, les sorties sans rapport** (`dec!(99.95)` à `:1765` et `:1774`, `session_idle_minutes, 60` à `:1918`) **sont laissées et nommées au Change Log** ; la commande est rejouée avec le jeton `\b60\b` en plus, chaque site sorti étant classé (réécrit, historique, ou sans rapport) ; `test-schema/0001_schema_squash.sql:3` (« 95 migrations ») est régénéré par le script (frontière inchangée : 96 − 62 = 34), avec la ligne d'historique (« Story 49-1a : 61 → 62, frontière
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
  `InvoiceLine {` sont complétés (`line_type: LineType::Priced`) ; **les seuls tests existants modifiés** sont : ceux de AC3 (b) et AC13 (a) ; **deux tests qui montent un schéma partiel puis font tourner le dépôt**, dont le montage **pose** les colonnes `line_type` (sans les `CHECK`) avant d'exercer le dépôt, pour ne pas rendre `1054 Unknown column`, et **les retire avant `MIGRATOR.run()`** (`grouped_settlements.rs:1387` ; patron : `lettering_documents_backfill.rs:176-177`, « le retrait est **obligatoire** » — sinon `ADD COLUMN IF NOT EXISTS` saute en silence et le test passe sur une montée qui n'a pas créé la colonne, ou la montée échoue en `1060` si D13 retombe sur `tracked-by-sqlx`) — `grouped_settlements.rs::the_upgrade_keeps_unit_settlements_and_allows_a_group` (`:1301`, monte « avant `grouped_settlements` » puis appelle `facture()` = `invoices::create`) et `lettering_documents_backfill.rs`, par `colonne_posterieure` (`:178-246`, qui pose déjà les colonnes de la 43-1a ; `support/lettering_documents.rs`, `invoices::create` `:92`, `validate_invoice` `:113`, `create_credit_note` `:215`) — **liste ouverte** : tout autre test que le gate complet révélera s'ajoute, **chacun au Change Log avec son motif** ; les littéraux sont par ailleurs complétés sans autre changement. `kesh-seed`
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
  `routes/invoice_email.rs`, `invoice_pdf_service.rs` et `credit_notes.rs:307` (PDF) **ne changent aucune ligne de rendu** (hors périmètre, 49-1b) ; **seules exceptions nommées** : les littéraux de test `InvoiceLine {` de `invoice_pdf_service.rs:1251` et de `invoice_email.rs:1624` (`#[cfg(test)]`), qui reçoivent `line_type: LineType::Priced`.

**Exports, MCP, documentation**

- **AC13** — (a) `serialize_invoice_lines_csv` (`csv_tables.rs:623`) et `serialize_credit_note_lines_csv` (`:1271`) ajoutent
  `line_type` **en dernière colonne** (D14) ; `exports_global_e2e` et `admin_full_export_e2e` (`("invoice_lines.csv", 0)` à `exports_global_e2e.rs:991` est un *nombre de lignes* attendu, non un en-tête : **aucune assertion d'en-tête de
  `invoice_lines.csv` n'existe, T10 l'écrit**) sont mis à jour ; (b) MCP : `LigneFacture.type_ligne` (`mcp/outils/factures.rs:237`, `:487`)
  — **ajout de clé** `typeLigne` sur chaque ligne de la sortie, **pour toute facture, existante comprise** (D14), non cassant ; **mesure prévue au développement, discriminante** (V6-2) : le texte joli d'une sortie est borné à `MCP_ENVOI_OCTETS_MAX` (`services/envois_mcp.rs:55`, 24 576 octets ; `sortie.rs:498`) et une ligne pèse plusieurs centaines d'octets, donc 200 lignes franchissent le plafond **avec ou sans** le champ : on mesure **le nombre maximal de lignes qui passe sous le plafond, avec et sans `typeLigne`**, et la taille de la plus grosse facture de la base de démonstration avec et sans le champ ; **ordre de grandeur attendu de l'ajout : une trentaine d'octets par ligne** (`"typeLigne": "priced",` en JSON joli, relevé à confirmer). **Conduite** : si une facture **juste sous le plafond sans le champ** passe en `TropVolumineux` **du seul fait du champ**, la story **s'arrête et le dit** au rapport (**pas de troncature**) ; le seuil mesuré va au Change Log (précédent : `tailles_mesurees_en_devise`, `rapports.rs:26`) ; le MCP de 0.19.0 n'est pas contractuellement figé sur les champs (à confirmer au développement : un test d'exactitude
  du schéma, s'il existe, est mis à jour) ; (c) `docs/api-external.md` : `lineType` documenté sur la création et la modification
  d'une facture (défaut `priced`, valeurs `comment` et `heading` **réservées**, refusées `400` jusqu'à leur ouverture), le champ
  des réponses, la colonne des exports, **et la clé `typeLigne` ajoutée à chaque ligne de la sortie MCP `kesh_facture`** (ajout non cassant : un consommateur qui ignore les clés inconnues n'est pas touché) ; **le changement de contrat `422` → `400 VALIDATION_ERROR`** (champ manquant sur une ligne chiffrée), en « ⚠️ » sur le modèle de `:1121`, et la mention qu'un `lineType` inconnu rend `422` ; la phrase « les lignes d'une facture client portent un type depuis la v0.20.0 » ; (d)
  **aucun manuel utilisateur** n'est touché (l'écran ne change pas — `git grep -n "ligne de facture" -- docs/manual/fr/user-manual.tex`
  ne rend que `:515` (comptes imputables), qui ne décrit aucun type de ligne ; le manuel d'administration : voir (e) ci-dessous).

- **AC13 (e)** — **Manuel d'administration, FR seulement** (`docs/manual/fr/admin-manual.tex`, `.tex` seulement, PDF non commité) : les manuels DE, EN et IT **n'existent pas** (`docs/manual/{de,en,it}/` ne portent qu'un `README.md`, `docs/manual/de/README.md:3` « Traductions allemandes non encore créées ») : **aucun fichier n'est créé** pour eux : la 49-1a pose le **squelette de la section « Passer à la 0.20.0 »** avec **son paragraphe « Pour les intégrations »**, sur le modèle de `admin-manual.tex:2569` (0.19.0) : la colonne `line_type` en **dernière** position des CSV `invoice_lines` et `credit_note_lines` (D14), le champ MCP `type_ligne` (clé `typeLigne`, **présent sur chaque ligne**, ajout non cassant), le changement de contrat `422` → `400 VALIDATION_ERROR` pour un champ manquant sur une ligne chiffrée, un `lineType` inconnu → `422`. La **49-1d complète** la même section (sauvegarde d'avant la mise à jour, recette) : les deux touchent le même fichier, **la 49-1d part de la 49-1a fusionnée**. (V2-2 ; module `docs/` déjà compté.) **Ce qui est assigné à la 49-1d** (V3-7), outre la sauvegarde d'avant la mise à jour et la recette : les paragraphes « Compatibilité » (modèle `admin-manual.tex:2302`), « Le retour à la 0.19.0 » (`:2562`) et « Si la migration échoue au milieu » (`:2564`), **« Une migration »** (96 au total, non breaking, `kesh_version_min_required` inchangé à 0.17.6, nom du fichier de sauvegarde `…-0.19.0-to-0.20.0.keshbackup`, modèle `:2560`) et **« Ce qu'il faut savoir »**, la 49-1a posant dans le squelette **le `\label{sec:maj-0-20-0}`** que la phrase « Compatibilité » référence ; avec deux faits que la 49-1a lui transmet par son Change Log : **une sauvegarde 0.20.0 est refusée par la 0.19.0** (`IMPORT_SCHEMA_MISMATCH`, `400`, **message fr-CH exact** `error-import-schema-mismatch` — « Le schéma de ce backup est incompatible avec cette version de Kesh. Mettez à jour Kesh ou utilisez un backup compatible. » (`crates/kesh-i18n/locales/fr-CH/*.ftl:2460`), distinct du « Fichier de sauvegarde invalide ou corrompu » d'`INVALID_BACKUP_STRUCTURE` ; colonne source `line_type` inconnue de la destination : `admin_backup/import.rs:615-637`, `errors.rs:3174` ; précédent 0.17.5), et **le verdict de D13** (`yes` : le rejeu après un échec au milieu est sans effet ; sinon `1060`).

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
| T1 | `migrations_upgrade_path.rs::line_types_migration_keeps_real_invoices` (AC2) | base 0.19.0 peuplée **par `INSERT` SQL de forme 0.19.0, jamais par le dépôt** (**dont une facture au PDF figé** — les quatre `pdf_*` posés, `pdf_language = 'FR'` — **et une non figée** ; payée, impayée, annulée + avoir, groupée, 1/3/8 lignes, 2 taux, arrondi) ; empreinte d'avant = d'après sur **toutes** les colonnes d'avant (`invoices` comprise, colonnes `pdf_*` incluses) ; `line_type = 'priced'` partout ; totaux, TTC, reste dû, écritures inchangés ; contraintes existantes présentes | **cinq mutations numérotées, chacune changeant réellement une valeur et tuée par l'assertion d'empreinte (journal `rouge-T1-<n>.log`)** : (1) `UPDATE invoice_lines SET unit_price = unit_price + 0.0001 WHERE id = (SELECT MIN(id) …)` ; (2) `UPDATE credit_note_lines SET description = CONCAT(description, '.') …` ; (3) `UPDATE invoices SET rounding_amount = rounding_amount + 0.01, updated_at = updated_at …` ; (4) `UPDATE credit_notes SET total_amount = total_amount + 0.01, updated_at = updated_at …` ; (5) `UPDATE invoices SET pdf_language = 'DE', updated_at = updated_at WHERE pdf_language = 'FR'` (valeur permise par la `CHECK` tout-ou-rien). Les mutations (3) à (5) **figent `updated_at`** : sans cela `updated_at DATETIME(3) … ON UPDATE CURRENT_TIMESTAMP(3)` (`20260416000001_invoices.sql:27`, `20260627000001_credit_notes.sql:39`), qui fait partie de « toutes les colonnes », **rougirait l'empreinte à la place de la colonne visée** ; (1) et (2) visent des tables sans `updated_at` ; **sans facture figée au peuplement, (5) ne toucherait aucune ligne et survivrait en silence** (V5-3), d'où la facture figée et le témoin `pdf_sha256 IS NOT NULL`. **Déclarées comme telles** : `DEFAULT 'comment'` et « oublier `credit_note_lines` » ne rougissent **pas** par l'empreinte mais par l'échec de la montée (`chk_…_unpriced_neutral`) ou de la requête de type (`1054`) ; un `UPDATE x = x` sans effet n'est **pas** une mutation de T1 (il n'est tué que par T12); **témoin chiffré** : le test écrit **en constantes** le nombre de colonnes (> 0) et de lignes **par table, ces comptes étant ceux des lignes rendues par l'empreinte elle-même (avant et après), non d'un `COUNT(*)` séparé** (V5-4) (`invoices`, `invoice_lines`, `credit_notes`, `credit_note_lines` ; factures figées ; avoirs) et au moins une ligne d'`invoices` avec `pdf_sha256 IS NOT NULL` ; **le témoin est lui-même éprouvé** : une mutation du peuplement (une table vidée) et une empreinte sur une liste de colonnes vide le font rougir (journal `rouge-T1-temoin.log`). **Déclarées comme telles** : `DEFAULT 'comment'` et « oublier `credit_note_lines` » ne rougissent **pas** par l'empreinte mais par l'échec de la montée (`chk_…_unpriced_neutral`) ou de la requête de type (`1054`) ; un `UPDATE x = x` sans effet n'est **pas** une mutation de T1 (il n'est tué que par T12) |
| T2 | `kesh-db/tests/invoice_line_types.rs::constraints_by_type` (AC1, D4), pour **les deux tables** | `priced` : `quantity = 0` refusée, `quantity > 0` admise ; `comment` : `quantity = 1`, `unit_price = 1`, `vat_rate = 7.7`, `line_total = 1`, `revenue_account_id` posé, description vide **refusés** ; `heading` neutre admis ; une ligne **neutre** (quantité, prix, taux, total à 0, compte `NULL`) de `line_type = 'subtotal'` **refusée** (neutre, pour que seul `chk_*_line_type` puisse la refuser) ; **un `INSERT` de forme 0.19.0 (sans `line_type`, formes de `invoices.rs:480-482` et `credit_notes.rs:774-777`) passe après la migration et donne `priced`** (V1-13) ; description non vide **toujours** exigée pour un commentaire | retirer **chacune** des trois contraintes neuves, et remettre `chk_*_quantity_positive` : un test rougit par mutation (quatre journaux par table) |
| T3 | `admin_full_import_e2e.rs::import_sauvegarde_0_19_0_donne_les_memes_lignes` et `…_aller_retour_ligne_de_commentaire` (AC4) | manifeste sans `line_type` → `'priced'` ; manifeste 0.20.0 avec `comment` → identique | `NOT NULL` **sans** `DEFAULT` : l'import 0.19.0 échoue (`is_required`) |
| T4 | `kesh-db/tests/invoice_line_types.rs::draft_roundtrip_and_no_op` (AC6, AC7) | création et remplacement d'un brouillon avec un commentaire entre deux lignes chiffrées : positions `0..n`, total inchangé ; un `update` qui ne change **que** le type n'est **pas** un no-op ; audit « before / after » porte `lineType` ; ligne `comment` non neutre → `Invariant` | retirer `b.line_type == c.line_type` de `is_no_op_change` ; retirer le `lineType` du snapshot |
| T5 | `kesh-db/tests/invoices_validate_vat.rs::validation_ignores_unpriced_lines` (AC8) | écriture identique avec et sans commentaire (comptes, montants, ordre) ; le compte du commentaire reste `NULL` après validation ; compte par défaut **non exigé** si les lignes chiffrées ont un compte explicite ; pièce de commentaires seuls → `invoiceTotalZero` | **trois mutations, deux jeux** : jeu (1) une ligne chiffrée à compte `NULL` **plus** un commentaire → l'`UPDATE` s'exécute, le commentaire reste `NULL` ; tue le retrait de `AND line_type = 'priced'` de `:2314` (**500 par `chk_…_unpriced_neutral`**) ; jeu (2) lignes chiffrées à compte explicite **plus** un commentaire, **compte par défaut archivé** → la validation passe (le compte par défaut reste exigé *configuré*, `invoices.rs:2169-2171`, mais n'est re-validé que s'il est effectivement posté) ; tue le retrait du filtre du `any(…is_none())` (`:2269`, `:2312`) ; la boucle en mémoire (`:2335-2337`) est tuée par AC8 (e, f) |
| T6 | `kesh-db/tests/credit_notes_repository.rs::credit_note_copies_unpriced_lines` (AC9) | l'avoir d'une facture à commentaires : lignes **toutes** copiées, types et positions ; écriture identique à celle sans commentaires ; `get` rend `line_type` ; avoir d'une facture **d'avant la montée** : mêmes lignes, même écriture | ne pas copier `line_type` (`INSERT` sans la colonne → `priced` : l'assertion de type rougit) ; copier le commentaire avec un compte |
| T7 | `kesh-report/tests/vat_report_reconciliation.rs::vat_report_ignores_unpriced_lines` (AC10 a) | rapport identique avec / sans commentaire ; **aucune ligne « 0 % »** ; `reconciliation_delta` identique | retirer `AND il.line_type = 'priced'` : la ligne `0 %` apparaît |
| T8 | `invoice_amount_due_parity.rs`, `invoice_ttc_parity.rs` : cas « avec commentaire et titre » (AC10 b) | TTC, crédité, réglé, reste dû identiques (SQL ↔ `kesh-core`) | remplacer les valeurs neutres d'un commentaire par `unit_price = 1` (test de contre-épreuve du motif de neutralité) |
| T9 | `routes/invoices.rs` (tests unitaires de `validate_line`) + `kesh-api/tests/invoice_line_types_e2e.rs` (AC11, AC12) | par type et par champ (tableau) ; **corps de la 0.19.0 identique** accepté, réponse inchangée hors `lineType` ; `comment` et `heading` refusés `400` au `POST` **et** au `PUT`, **zéro écriture** (comptage de lignes, `version` de la facture inchangée) ; l'omission de `quantity` d'une ligne chiffrée → `400` nommant la ligne | `open_to_api()` rendant `true` pour `comment` : le test de refus rougit ; retirer l'`Option` : la désérialisation du commentaire échoue |
| T10 | `exports_global_e2e.rs`, `admin_full_export_e2e.rs` (AC13) | `line_type` en dernière colonne des deux CSV ; valeurs `priced` ; une ligne `comment` créée par le dépôt s'exporte | retirer la colonne de l'en-tête **ou** de la ligne : l'écart d'arité rougit |
| T11 | `mcp_outils_e2e.rs` (AC13 b) | la **clé elle-même** `typeLigne` est présente (`as_object().unwrap().contains_key("typeLigne")` ou le texte de la sortie, **jamais `is_null()`** : l'indexation d'un `Value` rend `Null` pour une clé absente) avec la valeur `"priced"` pour une ligne chiffrée, `"comment"` et `"heading"` pour des lignes créées par le dépôt | ne pas émettre le champ : l'absence de clé rougit ; le coder en dur à `priced` : `comment` et `heading` rougissent |
| T12 | `post_restore.rs` : `every_data_backfill_migration_is_triaged` (AC3 c), `test_schema_guard.rs` (AC3 d), `reset_partition_covers…` | verts **sans nouvelle entrée** ; le squash porte la colonne et les trois contraintes | ajouter un `UPDATE` à la migration : `every_data_backfill…` rougit (prouve que la migration est bien du DDL seul) |
| T13 | `invoice_pdf_e2e`, `invoice_frozen_pdf_e2e`, `invoice_frozen_pdf`, `invoice_send_email_e2e` (AC14) | **inchangés et verts** : le rendu n'a pas bougé | (épreuve, non test neuf) |

**E2E Playwright** : la story ne change **aucun** écran ; **aucun E2E neuf**, motif : le défaut se voit à la frontière HTTP, joué par
`invoice_line_types_e2e` (T9), et la 49-1c joue le parcours dans le navigateur. La **suite complète** tourne tout de même au dernier
commit de code (AC15) : elle prouve que l'écran de la 0.19.0 crée, modifie, valide et solde toujours une facture sur le schéma migré.

**Recette visuelle** (`CLAUDE.md`) : sans objet — aucun contrôle visible n'est ajouté ni modifié (dit au Dev Agent Record).

## Tâches

- [x] **T0 — Étape 0, avant d'écrire.** (a) Relire les lignes citées de § *Ce qui se lit au code* à la tête du worktree
  (`git -C /home/gcorbaz/devel/kesh-49-1 log -1 --format=%H` : `773df80da` ou son descendant dans l'épique) ; une ligne qui a bougé se
  déclare au Change Log. (b) **Rejouer la sonde de D13** sur le MariaDB de dev (un `ALTER` unique par table, deux fois, trois formes `IF [NOT] EXISTS`) ; coller la sortie.
  (c) Rejouer que `ADD CONSTRAINT … CHECK` référençant la colonne **ajoutée dans le même `ALTER`** valide les lignes existantes
  (`quantity > 0`, `unit_price >= 0`) sans erreur. (d) Relever que `information_schema.COLUMNS.COLUMN_DEFAULT` rend `'priced'` (donc
  `has_default`, `backup.rs:795`). (e) **Sonder le retour (V1-5)** sur un schéma jetable `kesh_e491_*` (supprimé ensuite) : montée, retour par le SQL de l'en-tête (AC1 c) sur les deux tables, vérifier que les contraintes d'origine sont revenues (`information_schema.CHECK_CONSTRAINTS`) et qu'un `INSERT` de forme 0.19.0 passe, puis **remontée** ; si le `ALTER` du *1°* échoue, le scinder et le dire au Change Log. Si l'une des trois ne se rejoue pas, **s'arrêter et le dire** (rapport).
- [x] **T1 — La migration** (AC1) et son audit (AC3 a), `regen-test-schema.sh` (AC3 d), P6 (AC3 b).
- [x] **T2 — Entités, constantes de colonnes, littéraux** (AC5). Compiler d'abord (`cargo check --workspace --all-targets`) : les 67 +
  4 littéraux se trouvent au compilateur ; **un seul commit** pour cette mécanique.
- [x] **T3 — Dépôts** : `insert_lines`, `compute_total`, `is_no_op_change`, snapshots (AC6, AC7) ; `validate_invoice` (AC8) ;
  `create_credit_note` (AC9) ; `vat_report` et `invoice_settlements_write` (AC10).
- [x] **T4 — API** (AC11, AC12) : DTO, `validate_line` par type, `LineType::open_to_api()`.
- [x] **T5 — Exports, MCP, `docs/api-external.md`** (AC13).
- [x] **T6 — Tests** T1 à T12 ; **journaux de rouge** dans `kesh-gate-logs/e491a/`.
- [ ] **T7 — Gates** : `scripts/test-fast.sh` en cours de boucle (`kesh-db` touché : **gate complet** dès le premier patch de dépôt,
  `CLAUDE.md` § *Exception `kesh-db`*), base remise à zéro avant le gate complet, E2E complet au dernier commit de code ; démarrage
  sur base persistante (P8). **Le story file n'affirme que ce qui a tourné.**
- [x] **T8 — Dev Agent Record, Change Log, statuts** ; ne **pas** toucher `sprint-status.yaml` (l'orchestrateur tient les statuts).

## Note de version

*(Pour le CHANGELOG, assemblée à l'intégration ; la story n'écrit pas `CHANGELOG.md`.)*

> **Migration : sauvegarde complète obligatoire avant la mise à jour** (base, `.keshbackup` et dossier des documents). Kesh sait
> désormais, au niveau du schéma, distinguer une ligne chiffrée d'une ligne de texte seul (commentaire, titre de section) sur les
> factures et les avoirs ; **chaque facture et chaque avoir existants restent identiques**, ligne pour ligne. Aucune ligne de texte
> seul ne peut encore être saisie : leur écran et leur impression arrivent avec les stories suivantes de #1242. L'API des factures
> accepte un champ optionnel `lineType` (défaut `priced`, absent ou `null`) ; ses valeurs `comment` et `heading` sont refusées pour l'instant, et un type inconnu rend `422`. **Changement de contrat de l'API** : un champ `quantity`, `unitPrice` ou `vatRate` manquant ou `null` sur une ligne chiffrée rend désormais `400 VALIDATION_ERROR` (nommant la ligne), non plus le `422` de l'extracteur. Les
> exports CSV des lignes de facture et d'avoir gagnent la colonne `line_type`, en dernière position, et la sortie `kesh_facture` du serveur MCP gagne, sur chaque ligne, la clé `typeLigne` (`priced`, `comment` ou `heading`) — un ajout de clé, sans effet sur un consommateur qui ignore les clés inconnues, sauf pour une très longue facture : la sortie restant plafonnée, une facture d'environ 85 à 94 lignes courtes (moins si les descriptions sont longues) n'est plus servie par `kesh_facture` (#1347). Une sauvegarde 0.20.0 ne s'importe pas dans une 0.19.0. (refs #1242)

## Dev Notes

- **Montage des tests partiels (V2-1)** : voir AC5 ; la liste des tests modifiés s'arrête au gate complet.
- **Couplage 49-1a / 49-1d** : la 49-1a **ne sort pas sans la 49-1d** (V1-4) ; la 49-1d garde la recette et le reste de la section « Passer à la 0.20.0 » du manuel d'administration (FR seulement : les manuels DE, EN, IT n'existent pas).
- **Hors périmètre, avec la story qui le prend** : rendu PDF (facture, avoir) → **49-1b** ; écran, ouverture de `comment`, manuel
  utilisateur (FR seulement), i18n → **49-1c** ; gate de recette, section « Passer à la 0.20.0 » du manuel d'administration (FR : introduction « La 0.20.0 se pose sur la 0.19.0 » (modèle `:2558`), « Une migration », sauvegarde d'avant la mise à jour, compatibilité, retour, échec au milieu, « Ce qu'il faut savoir » ; la 49-1a en pose le squelette, le `\label{sec:maj-0-20-0}` et les intégrations) → **49-1d** ; réordonnancement
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

*Agent de développement : Opus 5.5 (`bmad-dev-story`), 2026-11-18, worktree `kesh-49-1`, clé `e491`. Registre :
`_bmad-output/implementation-artifacts/e0200-choix-autonomes-e491.md` (C-e491-1 à 8). Journaux :
`/home/gcorbaz/devel/kesh-gate-logs/e491/`.*

### Nom de la migration (C-e491-1)

Écrite le 2026-11-18 : **`20261118000002_invoice_line_types.sql`**, non `20261119000001`. Partout où la fiche écrit
`20261119000001`, lire `20261118000002` (AC1, AC2, T1, SQL de retour de l'en-tête : `DELETE FROM _sqlx_migrations WHERE version =
20261118000002`). Propagé : en-tête de la migration, `docs/migrations-idempotence-audit.md`, `migrations_upgrade_path.rs`
(`LINE_TYPES_VERSION`), squash.

### T0 — sonde (verdict : montée, rejeu, retour, remontée **tiennent**)

MariaDB `10.11.19-MariaDB-ubu2204` (conteneur de dev). Schéma jetable `kesh_e491_t0` monté par les **95 migrations précédentes**
(`sqlx migrate run` sur une copie des fichiers, `kesh-gate-logs/e491/t0/mig95/`), deux lignes de facture et une d'avoir de forme
0.19.0, puis **le fichier de la migration lui-même** appliqué deux fois (`t0/sonde-montee.log`) : (b) les deux applications passent,
la seconde sans effet (formes `IF [NOT] EXISTS` acceptées dans un seul `ALTER`, y compris un `CHECK` qui référence la colonne
ajoutée dans le même `ALTER`) ; (c) les lignes existantes (`quantity` 1 et 2, `unit_price` 1200 et 50.5) passent les `CHECK` neufs ;
(d) `information_schema.COLUMNS.COLUMN_DEFAULT` = `'priced'`, `IS_NULLABLE` = `NO`, sur les deux tables ; toutes les lignes `priced` ;
contraintes d'avant présentes, `uq_*_position` dans `TABLE_CONSTRAINTS`. Contraintes (`t0/sonde-contraintes-retour.log`) :
commentaire neutre admis ; commentaire `quantity = 1` → `ERROR 4025 … chk_invoice_lines_unpriced_neutral` ; `priced` `quantity =
0` → `chk_invoice_lines_priced_quantity` ; `subtotal` neutre → `chk_invoice_lines_line_type`. (e) **Retour** par le SQL de
l'en-tête, deux tables, après suppression du commentaire (la requête de garde rendait 1, puis 0) : `retour-ok`, les cinq `CHECK`
d'origine par table revenus (dont `chk_*_quantity_positive`), colonne absente, un `INSERT` de forme 0.19.0 passe, `quantity = 0`
refusé par `chk_invoice_lines_quantity_positive` ; **remontée** : `remontee-ok`, lignes `priced`, 14 `CHECK`. Le `ALTER` du 1° du
retour (trois `DROP CONSTRAINT` en un) n'a pas eu à être scindé. Schéma `kesh_e491_t0` supprimé ensuite. Verdict d'idempotence :
**`yes`** (D13 tient).

### Étape 0 — lignes relues

- T0 (a) : `git diff --stat 773df80da..fe01a0179` ne nomme que la fiche ; les lignes citées de § *Ce qui se lit au code* sont donc
  celles de `773df80da`. Lignes relues à l'écriture (avant patch) : `invoices.rs:47-48` `pub(crate) const LINE_COLUMNS: &str = "id,
  invoice_id, position, description, quantity, \ unit_price, vat_rate, line_total, revenue_account_id, created_at";` ;
  `invoices.rs:2269`/`:2312` `if lines_before.iter().any(|l| l.revenue_account_id.is_none()) {` (à `:2337` et `:2380` du fichier
  après mes ajouts en tête) ; `invoices.rs:2382-2383` `"UPDATE invoice_lines SET revenue_account_id = ? \ WHERE invoice_id = ? AND
  revenue_account_id IS NULL"` ; `credit_notes.rs:515-518` `let triplets … .map(|l| (l.line_total, l.vat_rate,
  l.revenue_account_id))` ; `credit_notes.rs:793-799` `INSERT INTO credit_note_lines … VALUES (?, ?, ?, ?, ?, ?, ?, ?)` ;
  `vat_report.rs:119-125` ; `invoice_settlements_write.rs:647` ; `mcp/sortie.rs:90-114` (macro sans attribut de champ) et `:132`
  (`champ_enum!`) ; `routes/invoices.rs:69-87` (DTO) et `:475-543` (`validate_line`).
- Piège (b) : `git grep -n "/ .*quantity\|quantity.*/" -- 'crates/*/src/*'` rejoué : **sortie vide** — aucun lecteur ne divise par
  la quantité.
- LOW de la P6 (V6-1 à V6-4) : déjà portés par la fiche ; V6-1 vérifié au code (`LineType` ajouté à `champ_enum!`, la macro
  `sortie!` inchangée ; un doc-comment `///` sur le champ `type_ligne` ne compilait pas — attribut de champ, Ma26 — remplacé par
  `//`).

### P6 (AC3 b)

`grep -rn "migrations.len()\|apply_migrations_up_to" crates/` rejoué : tous les sites hors `migrations_upgrade_path.rs` résolvent
**par version** (`common::migrations_before`, `pre_upgrade_backup/tests.rs:186-194` — `restantes = len() - appliquees` avec
`appliquees` résolu par version et assertion de montage —, `sauvegarde_avant_migration.rs:28-43`, `grouped_settlements.rs:1301`,
backfills). Dans `migrations_upgrade_path.rs`, réécrits au même patch : `:49`, `:51`, `:68-69`, `:135-140` (`total, 96`, « 95
précédentes + Story 49-1a »), `:142`, `:150`, `:157`, `:159-160`, `:237`, `:240`, `:309` ; ligne d'historique ajoutée « Story 49-1a :
61 → 62, frontière inchangée (96 - 62 = 34) ». Rejeu `git grep -nE "\b95\b|\b61\b|\b60\b"` après patch
(`kesh-gate-logs/e491/p6-grep-apres.log`) : `:100`, `:101`, `:135`, `:167`, `:168`, `:229`, `:230`, `:232` = **historique** ;
`:140` = message (« 95 précédentes ») ; `:160` = la consigne réécrite (« Le laisser à 61 avec `total = 96` ») ; `:1767`, `:1776`
(`dec!(99.95)`), `:1920` (`session_idle_minutes, 60`) = **sans rapport**, laissés.

### P7 (AC3 c), P5, squash

Migration **DDL seul** : aucune ligne à `POST_RESTORE_BACKFILLS` ni à `EXEMPT_MIGRATIONS` ; « DDL seul ; le `DEFAULT` rétro-remplit »
au tableau d'audit. Audit : ligne `yes` ; `ls crates/kesh-db/migrations/*.sql | wc -l` = 96 ; `grep -c '^| \`20'` = 96 ;
partition `yes` 12 + `tracked-by-sqlx` 84 + `no` 0 = 96 ; en-tête et `Total` à 96 (« + 1 Story 49-1a »). Squash régénéré par
`scripts/regen-test-schema.sh` (« 96 migrations appliquées », rejeu vérifié, 50 tables).

### Ce qui a été fait

- **Migration** (AC1) : un `ALTER TABLE` par table, `ADD COLUMN IF NOT EXISTS line_type VARCHAR(16) NOT NULL DEFAULT 'priced'`,
  `DROP CONSTRAINT IF EXISTS chk_*_quantity_positive`, trois `ADD CONSTRAINT IF NOT EXISTS` ; en-tête (a) à (e), SQL de retour en
  `--`. Aucun `Cargo.toml`, aucun `UPDATE _kesh_version`.
- **Entités** (AC5) : `entities/line_type.rs` — `LineType { Priced, Comment, Heading }`, `Default = Priced`, `as_str`, `FromStr`,
  `Display`, serde minuscule, `Type`/`Encode`/`Decode` MySQL (patron d'`ApiKeyScope`), `is_priced()`, **`open_to_api()`** (D9) ;
  `InvoiceLine.line_type`, `NewInvoiceLine.line_type` (`#[serde(default)]`), `CreditNoteLine.line_type`. Une seule liste de
  colonnes par table (C-e491-2) : `invoice_line_columns!` (`LINE_COLUMNS` et la forme `il.` de l'export),
  `credit_note_line_columns!` (`SELECT_CREDIT_NOTE_LINES_SQL` pour les deux lectures, forme `cnl.` de l'export). Les **67 littéraux
  `NewInvoiceLine {`** existants (`git grep -F "NewInvoiceLine {" fe01a0179 -- 'crates/*' | grep -v "pub struct" | wc -l` = 67) et les
  **4 `InvoiceLine {`** sont complétés à l'outil Edit ; le même décompte à `HEAD` rend 84 (17 littéraux neufs : 2 de production,
  `validate_line` et `validate_unpriced_line`, et 15 des tests neufs).
- **Dépôts** : `new_line_total` (zéro pour une ligne non chiffrée), `compute_total` (lignes chiffrées), `ensure_unpriced_lines_are_neutral`
  (D10, appelée au début de `create`, d'`update` et dans `insert_lines`, C-e491-3), `insert_lines` écrit `line_type` ;
  `explicit_line_account_sites` alimenté par les seules lignes chiffrées (création, modification, validation) ; `is_no_op_change`
  compare `line_type` ; `"lineType"` aux snapshots d'audit de la facture et de l'avoir ; `validate_invoice` :
  `priced_line_without_account` pour le site « compte par défaut », la matérialisation (`UPDATE … AND line_type = 'priced'`) et la
  boucle en mémoire ; `create_credit_note` : `triplets` et sites « compte par défaut » sur les lignes chiffrées, copie de
  `line_type` ; `vat_report.rs` et `invoice_settlements_write.rs` filtrent `line_type = 'priced'`.
- **API** (AC11, AC12) : `CreateInvoiceLineRequest.line_type` (`#[serde(default)]`), `quantity`/`unit_price`/`vat_rate` en
  `Option<Decimal>` ; `validate_line` exige les trois montants d'une ligne chiffrée (« Ligne N : la quantité / le prix unitaire / le
  taux de TVA est obligatoire »), messages d'avant inchangés ; `validate_unpriced_line` (C-e491-4) ; `lineType` dans
  `InvoiceLineResponse` et `CreditNoteLineResponse`.
- **Exports, MCP, docs** (AC13) : `line_type` en dernière colonne des deux CSV ; `LigneFacture.type_ligne` et `LineType` à
  `champ_enum!` ; `docs/api-external.md` (sous-section « Les lignes d'une facture client », C-e491-8) ; manuel d'administration FR,
  squelette « Passer à la 0.20.0 » (`\label{sec:maj-0-20-0}`, « Pour les intégrations », C-e491-7) ; **aucun manuel utilisateur, aucun
  PDF** ; `docs/manual/{de,en,it}` non touchés (n'existent pas).
- **Non-régression** (AC14) : `git diff 773df80da..HEAD -- crates/kesh-qrbill crates/kesh-api/src/routes/issued_invoice_pdf.rs
  frontend` **vide** ; `invoice_pdf_service.rs` et `invoice_email.rs` : un seul ajout chacun, `line_type: …::Priced` dans le littéral
  de test.

### Transmis à la 49-1d (par ce Change Log)

- **Une sauvegarde 0.20.0 est refusée par la 0.19.0** : la colonne source `line_type` est inconnue de la destination
  (`admin_backup/import.rs:615-637`, `errors.rs:3174`) → `IMPORT_SCHEMA_MISMATCH` (`400`), message fr-CH « Le schéma de ce backup
  est incompatible avec cette version de Kesh. Mettez à jour Kesh ou utilisez un backup compatible. »
- **Verdict de D13 : `yes`, et la procédure vraie** *(corrigée après la revue P1, E1)* — un échec entre les deux `ALTER` laisse
  `invoice_lines` migrée et une ligne `success = 0` dans `_sqlx_migrations` : **Kesh refuse de redémarrer** (`MigrateError::Dirty`).
  Reprise : `DELETE FROM _sqlx_migrations WHERE version = 20261118000002 AND success = 0`, puis redémarrer ; le rejeu est sans effet
  sur `invoice_lines` et termine `credit_note_lines`. Prouvé par `migrations_upgrade_path.rs::line_types_migration_resumes_after_a_half_run`.
  Le paragraphe « Si la migration échoue au milieu » de la 0.19.0 (`admin-manual.tex`) disait la même erreur (`1060` sans ligne
  inscrite) : corrigé dans cette story ; pour la 0.19.0, retirer la ligne ne suffit pas (le rejeu échouerait en 1060), la sauvegarde
  `pre-upgrade/` reste la reprise.
- **Seuil MCP** (C-e491-6, **tranché par l'orchestrateur le 2026-11-19 : limite acceptée, suivie par #1347**,
  `kesh-gate-logs/e491/mesure-mcp.log`, `cible-p1-*.log`) : une ligne **minimale** de `kesh_facture` pèse 282 o dont 29 o pour
  `typeLigne` ; socle 672 o ; une facture passe sous les 24 576 o jusqu'à **94 lignes sans la clé, 84 avec** — **bornes hautes** :
  des descriptions plus longues abaissent les deux seuils, la bande touchée garde une dizaine de lignes. Documenté à
  `api-external.md`, au paragraphe « Pour les intégrations » du manuel et à la Note de version.

### Mutations (34, `kesh-gate-logs/e491/mut/`)

Banc versionné : `muter.py` (motif unique exigé, code commité avant, restauration `git restore --worktree` + `touch`), `specs.py`
(une spec JSON par mutation), `lancer.sh` (une mutation à la fois, sous le verrou de la voie base) ; résumé `resume.log` ; un
journal par mutation. Code muté au commit `16c168cfc` ; `git status` propre après la campagne. **Chaque mutation est tuée**
(`panicked` dans le journal), aucune par une erreur de compilation (`grep -c "could not compile\|error\[E"` = 0 sur les 32) ;
*corrigé après la revue P1 (A3)* : toutes ne le sont pas par l'assertion visée — **T10** l'est par un `500` de l'export (l'écrivain
CSV refuse l'arité inégale, l'assertion d'arité n'est pas atteinte), **T6 « type non copié »** par
`chk_credit_note_lines_priced_quantity` (la ligne copiée sans type devient `priced` à quantité 0), **T8** (campagne 1) par la garde
D10 à la création, le **témoin « colonnes vides »** de T1 par une erreur SQL 1064. Les autres le sont par l'assertion visée.

| Test | Mutations → verdict |
|---|---|
| T1 | (1) `unit_price + 0.0001` sur une ligne de facture ; (2) `description` d'une ligne d'avoir ; (3) `rounding_amount + 0.01` (`updated_at` figé) ; (4) `credit_notes.total_amount + 0.01` (`updated_at` figé) ; (5) `pdf_language = 'DE'` — chacune **ajoutée à la migration** : **ROUGE** ×5, par l'assertion d'empreinte (« une colonne d'avant a changé ») ; témoin : `credit_note_lines` vidée avant l'empreinte → **ROUGE** ; liste de colonnes vide → **ROUGE** (erreur SQL d'un `CONCAT_WS` vide, non une assertion : déclarée) |
| T2 | par table, `chk_*_line_type` retirée, `chk_*_priced_quantity` retirée, `chk_*_unpriced_neutral` retirée, `chk_*_quantity_positive` remise (au squash) : **ROUGE** ×8 |
| T3 | `line_type` `NOT NULL` sans `DEFAULT` (squash) : **ROUGE** |
| T4 | `b.line_type == c.line_type` retiré de `is_no_op_change` ; `"lineType"` retiré du snapshot ; neutralité non contrôlée (D10) : **ROUGE** ×3 |
| T5 | `AND line_type = 'priced'` retiré de l'`UPDATE` (jeu 1, 500 par `chk_invoice_lines_unpriced_neutral`) ; site « compte par défaut » sur `any(is_none)` (jeu 2) ; boucle en mémoire sur `is_none` : **ROUGE** ×3 |
| T6 | `line_type` non copié ; commentaire copié avec un compte ; sites « compte par défaut » de l'avoir sans filtre (tué par `credit_note_unpriced_line_does_not_require_default_account`) : **ROUGE** ×3 |
| T7 | filtre du rapport TVA retiré : **ROUGE** |
| T8 | contre-épreuve, commentaire à `unit_price = 1` : **ROUGE** (par D10, non par la parité : voir la campagne 2) |
| T9 | `open_to_api()` vrai pour `comment` : **ROUGE** |
| T10 | `line_type` retiré de l'en-tête de `invoice_lines.csv` : **ROUGE** |
| T11 | type codé en dur `Priced` ; clé absente : **ROUGE** ×2 |
| T12 | un `UPDATE` ajouté à la migration : `every_data_backfill_migration_is_triaged` **ROUGE** |
| Équivalentes (VERT, déclarées) | `triplets` de l'avoir sans filtre (une ligne neutre ne pèse rien dans la contre-passation ; le compte par défaut verrouillé en plus n'est refusé qu'aux sites, que le filtre gardé couvre) ; filtre de `invoice_settlements_write.rs:647` retiré (`vat_breakdown_by_rate` écarte le taux 0 — filtre d'intention, AC10 c) |

**Non jouée** : T9 « retirer l'`Option` » — un champ `Option` sans `#[serde(default)]` reste facultatif pour serde (absence =
`None`), et le rendre `Decimal` ne compile pas sans réécrire `validate_line` ; le refus du champ manquant est tenu par
`missing_amount_is_400_and_unknown_type_is_422` et `priced_line_requires_each_amount`.

**AC3 (f) et AC4 (c)** *(revue P1, A6)* : `crates/kesh-db/tests/account_columns_inventory.rs` relu, non touché ; ses entrées
`invoice_lines.revenue_account_id` et `credit_note_lines.revenue_account_id` (`:225`, `:234`) restent justes, `line_type` n'est
pas une colonne de compte. `backup.rs` ne change que d'un littéral de test ; `backup_inventory_matches_schema` et
`export_table_serializes_rows_faithfully` sont verts sans changement de code (`test-fast-review-2.log`, `gate-ci-review.log`).

### Mutations de la revue P1 (campagne 2, 7, code au commit `f283f018e`)

Specs `kesh-gate-logs/e491/mut/specs2.py`, journaux `rouge-P1-*.log`, résumé `resume.log`. **Toutes ROUGE, chacune par
l'assertion visée** (compilation : 0 erreur ; `git status` propre ensuite).

| Mutation | Tuée par |
|---|---|
| `BINARY` retiré de `chk_invoice_lines_line_type` / `chk_credit_note_lines_line_type` (squash) | `constraints_by_type` : `'PRICED'` admis (« l'insertion aurait dû être refusée ») ×2 |
| `IF NOT EXISTS` retiré du `ADD COLUMN` de `invoice_lines` (migration) | `line_types_migration_resumes_after_a_half_run` : le rejeu après le retrait de la ligne échoue en `1060` |
| `deserialize_with` retiré de `lineType` | `null_line_type_is_priced` (« invalid type: null ») |
| T8, la ligne de commentaire devenue chiffrée à 1.00 (`invoice_ttc_parity.rs`, `invoice_amount_due_parity.rs`) | les assertions de parité elles-mêmes : TTC `12629.36` ≠ `12628.36`, reste dû `143.30` ≠ `142.30` (A2 : la campagne 1 était tuée par D10) |
| `lineType` sauté à la sérialisation de `InvoiceLineResponse` | `every_invoice_response_carries_the_line_type` (« POST : clés de la ligne ») |

### Gates (tels qu'ils ont tourné ; dernier commit de code : `aae45788d`)

| Gate | Journal (`kesh-gate-logs/e491/`) | Commit | Résultat |
|---|---|---|---|
| `cargo check --workspace --all-targets` | `check-3.log` (vide = sans erreur), puis incrémental | avant `ac5471e07` | vert |
| `cargo fmt --check`, `clippy --workspace --all-targets -D warnings` | sortie de console (deux `type_complexity` de tests corrigés) | avant `ac5471e07` | vert |
| gate ciblé des tests neufs | `cible-1.log` (35 tests : 33 verts, 2 rouges de test — forme `"1.0000"`, arrondi à 5 centimes sans compte —, corrigés), `cible-2.log` (3/3), `cible-3.log` (1/1), `mesure-mcp.log` | `ac5471e07` et suivants | vert |
| `scripts/test-fast.sh --ci`, base `kesh_e491` remise à neuf | `test-fast-review.log` | `16c168cfc` | **rouge** : 6159/6160, `test_schema_guard::published_migrations_keep_their_checksums` (registre `migrations.sha384` à compléter) — corrigé en `aae45788d`, `cible-4.log` (7/7) |
| `scripts/test-fast.sh --ci`, base remise à neuf | `test-fast-review-2.log` | `aae45788d` | **vert** : 6160 passés, 14 ignorés (fmt et clippy compris) |
| `scripts/gate-ci.sh kesh_ci_e491` (voie 1) | `gate-ci-review.log` | `aae45788d` | **vert** : 6177 passés, 0 échoué, 18 ignorés |
| frontend (`check`, `lint-i18n-ownership`, `test:unit`, `build`) | — | — | **non lancé** : frontend non touché |
| E2E Playwright complet ; démarrage sur base persistante (P8) | — | — | **non lancés** : à la clôture |
| **Remédiation de la revue P1** : gate ciblé | `cible-p1-1.log` (31 : 29 verts, 2 rouges de test — corps de `unvalidate`, marque du semis —, corrigés), `cible-p1-2.log` (2/2) | avant `f283f018e` | vert |
| `scripts/test-fast.sh --ci`, base `kesh_e491` remise à neuf | `test-fast-p1.log` | `f283f018e` | **vert** : 6163 passés, 14 ignorés |
| `scripts/gate-ci.sh kesh_ci_e491` (exception `kesh-db` : la migration change) | `gate-ci-p1.log` | `f283f018e` | **vert** : 6180 passés, 0 échoué, 18 ignorés |

Contrôle des mots-clés de fermeture (`git log origin/epic/49-types-de-lignes-facture..HEAD --format=%B | grep -niE …`) : vide.

### File List

Neufs : `crates/kesh-db/migrations/20261118000002_invoice_line_types.sql`, `crates/kesh-db/src/entities/line_type.rs`,
`crates/kesh-db/tests/invoice_line_types.rs`, `crates/kesh-api/tests/invoice_line_types_e2e.rs`,
`_bmad-output/implementation-artifacts/e0200-choix-autonomes-e491.md`,
`_bmad-output/implementation-artifacts/49-1a-types-de-lignes-migration-api-review-prompt-p1.md`.
Modifiés (code) : `crates/kesh-db/src/entities/{mod,invoice,credit_note}.rs`,
`crates/kesh-db/src/repositories/{invoices,credit_notes,invoice_settlements_write}.rs`, `crates/kesh-db/src/backup.rs` (littéral de
test), `crates/kesh-db/test-schema/0001_schema_squash.sql` (régénéré), `crates/kesh-report/src/vat_report.rs`,
`crates/kesh-api/src/routes/{invoices,credit_notes}.rs`, `crates/kesh-api/src/routes/{invoice_email,invoice_pdf_service}.rs`
(littéraux de test), `crates/kesh-api/src/exports/csv_tables.rs`, `crates/kesh-api/src/mcp/{sortie.rs,outils/factures.rs}`,
`crates/kesh-seed/src/pieges.rs`. Modifiés (docs) : `docs/migrations-idempotence-audit.md`, `docs/api-external.md`,
`docs/manual/fr/admin-manual.tex`. Modifiés (tests) : ceux du Change Log (littéraux complétés, tests neufs, montages partiels),
`crates/kesh-db/tests/common/mod.rs` (`colonnes_types_de_ligne`).

## Change Log

| Date | Entrée |
|---|---|
| 2026-11-18 | Création de la fiche (workflow `bmad-create-story`), par l'agent de rédaction de la 49-1 (Sonnet 5.5) : découpage préventif de la 49-1 en 49-1a à 49-1d (`kesh-gate-logs/e491/decoupage-49-1.md`), fiche de la première partie ; code lu à `773df80da`. Écarts au fichier d'épique : trois types stockés (non quatre), pas de type sous-total (D5), `kesh-core` non touché, API **gardée**. Non encore validée. |
| 2026-11-18 | **Passe de validation P1** (Opus, contexte frais, `kesh-gate-logs/e491/validation-49-1a-p1.md` : 0 CRITICAL, 0 HIGH, 7 MEDIUM, 8 LOW). **Remédiation** (sept MEDIUM et huit LOW traités, décisions de l'orchestrateur) : V1-1 AC14 et AC12 bornés au code de rendu, exception nommée du littéral de test `invoice_pdf_service.rs:1251` ; V1-2 `verify_vat_rates_against_db` (`routes/invoices.rs:680`, `:773`) ajouté à l'inventaire, traitement pris par la 49-1c, inatteignable en 49-1a (garde) ; V1-3 changement de contrat `422` → `400 VALIDATION_ERROR` nommé (D8, AC13 c, note de version, `api-external.md`) ; V1-4 couplage « la 49-1a ne sort jamais sans la 49-1d » (fiche et découpage) ; V1-5 SQL de retour complet (deux tables, contraintes retirées avant `DROP COLUMN`) et sondé à T0 (e) ; V1-6 deux jeux de mutation pour T5 ; V1-7 pathspecs corrigés, inventaires rejoués (18 fichiers de `src`, 31 de tests, 67 littéraux dans 43 fichiers, 4 littéraux `InvoiceLine {`) ; V1-8 schéma 0.19.0 par `migrations_before(20261119000001, …)` ; V1-9 `:991` est un nombre de lignes, T10 écrit l'assertion d'en-tête ; V1-10 AC5 « hors AC3 b et AC13 a » ; V1-11 « une seule liste à tenir » ; V1-12 `lineType` inconnu = `422`, dit tel quel ; V1-13 `INSERT` de forme 0.19.0 après la migration (T2) ; V1-14 commentaires `--` obligatoires ; V1-15 cinq modules, au plafond, déclaré. Modèles : Sonnet (rédaction), Opus (passe 1). Une passe 2 reste à lancer (modèle différent). |
| 2026-11-18 | **Passe de validation P2** (Sonnet, `kesh-gate-logs/e491/validation-49-1a-p2.md` : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 8 LOW). **Remédiation** : V2-1 deux tests sur schéma partiel nommés (`grouped_settlements.rs::the_upgrade_keeps_unit_settlements_and_allows_a_group`, `lettering_documents_backfill.rs` / `colonne_posterieure`) et leur évolution, AC5 corrigé (liste exacte), T1/AC2 peuplés par `INSERT` SQL de forme 0.19.0 ; V2-2 paragraphe « Pour les intégrations » du manuel d'administration ×4 dans la 49-1a (AC13 e), la 49-1d complétant la section ; V2-3 en-tête ; V2-4 D12 « aucun statement d'écriture hors commentaires `--` » ; V2-5 sites P6 de la valeur 95 / `total - 61` listés (AC3 b) ; V2-6 D8 cite l'extracteur natif d'Axum (`routes/invoices.rs:671`, `api-external.md:557`) ; V2-7 ligne `subtotal` neutre (T2) ; V2-8 empreinte de toutes les colonnes d'`invoices` dont `pdf_*` (AC2) ; V2-9 littéral `invoice_email.rs:1624` ajouté aux exceptions (AC12) ; V2-10 `credit_note_lines` à `TABLES` de la recette : porté par la 49-1d (découpage). Modèles : Sonnet (rédaction), Opus (P1), Sonnet (P2) ; la P3 prend Opus. |
| 2026-11-18 | **Passe de validation P3** (Opus, `kesh-gate-logs/e491/validation-49-1a-p3.md` : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 6 LOW). **Remédiation** : V3-1 manuel d'administration **FR seulement** (les manuels DE, EN, IT n'existent pas, `docs/manual/de/README.md:3`) : AC13 (e), Dev Notes, découpage ; V3-2 T1 : cinq mutations qui changent réellement une valeur (une par table d'empreinte, `pdf_*` comprises), témoin « empreinte non vide », mutations qui rougissent par la montée déclarées comme telles ; V3-3 Dev Notes propagées (49-1a = squelette + intégrations, 49-1d = le reste), `type_ligne` à la Note de version ; V3-4 P6 : commande et sites réels, citation `:1195-1199` ; V3-5 montages partiels : colonnes posées **puis retirées avant `MIGRATOR.run()`**, liste ouverte avec motif au Change Log ; V3-6 `uq_*_position` dans `TABLE_CONSTRAINTS` ; V3-7 refus d'une sauvegarde 0.20.0 par la 0.19.0 (`IMPORT_SCHEMA_MISMATCH`) dit, paragraphes « Compatibilité », « Retour » et « Échec au milieu » assignés à la 49-1d avec le verdict de D13 ; V3-8 mesure de la taille d'un envoi `kesh_facture` sous `MCP_ENVOI_OCTETS_MAX`. **Signal de splitting « recycle partiel »** (V3-1 né du correctif V2-2, V3-2 d'origine) : à déclarer au Project Lead par l'orchestrateur ; les remèdes sont locaux. Modèles : Sonnet (rédaction), Opus (P1), Sonnet (P2), Opus (P3). |
| 2026-11-18 | **Passe de validation P4** (Sonnet, `kesh-gate-logs/e491/validation-49-1a-p4.md` : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 6 LOW). **Remédiation** : V4-1 T1/AC2 peuplent au moins une facture au **PDF figé** (quatre `pdf_*`, `pdf_language = 'FR'`, `CHECK` de `20261003000001:30-34`) et une non figée — le cas des factures réelles de Guy — ; mutation 5 = `SET pdf_language = 'DE' WHERE pdf_language = 'FR'` ; témoin `pdf_sha256 IS NOT NULL` ; V4-2 citation `lettering_documents_backfill.rs:176-177` ; V4-3 témoin chiffré en constantes par table, lui-même éprouvé (`rouge-T1-temoin.log`) ; V4-4 P6 : `:159` ajouté, jeton `\b60\b`, lignes d'historique `:101`, `:167`, `:229` exceptées ; V4-5 **décision de l'orchestrateur** : `type_ligne` **omis** de la sortie MCP pour une ligne `priced` (aucune facture existante ne grossit), mesure conservée, la story s'arrête et le dit si une facture à commentaires franchit le plafond (pas de troncature), T11 réécrit ; V4-6 « Une migration », « Ce qu'il faut savoir » et le `\label{sec:maj-0-20-0}` assignés (label dans le squelette de la 49-1a, paragraphes à la 49-1d), message exact de `IMPORT_SCHEMA_MISMATCH` transmis à la 49-1d ; V4-7 en-tête. Modèles : Sonnet (rédaction), Opus (P1), Sonnet (P2), Opus (P3), Sonnet (P4) ; la P5 prend Opus. |
| 2026-11-18 | **Passe de validation P5 ciblée** (Opus, `kesh-gate-logs/e491/validation-49-1a-p5-ciblee.md` : 0 CRITICAL, 1 HIGH, 1 MEDIUM, 5 LOW). **Remédiation** : V5-1 (HIGH) — **le défaut venait d'une décision de l'orchestrateur (V4-5), retirée** : la macro `sortie!` n'admet aucun attribut de champ (`sortie.rs:90-114`), un `Option` sérialise `null` ; retour au **champ toujours présent** : `typeLigne` sur chaque ligne de la sortie MCP `kesh_facture`, **ajout de clé non cassant pour toute facture, existante comprise** (D14, AC13 b et e, Note de version, `api-external.md`), mesure de taille conservée (la story s'arrête si le plafond est franchi du seul fait du champ), T11 vérifie la clé elle-même, jamais `is_null()` ; V5-2 `updated_at = updated_at` aux mutations visant `invoices` et `credit_notes` (`ON UPDATE CURRENT_TIMESTAMP(3)`, `20260416000001:27`, `20260627000001:39`) ; V5-3 motif exact de la facture figée (survie silencieuse) ; V5-4 comptes de lignes tirés de l'empreinte elle-même ; V5-5 Dev Notes alignées (introduction, « Une migration », « Ce qu'il faut savoir ») ; V5-6 AC13 (e) : `typeLigne` présent sur chaque ligne ; V5-7 sites du jeton `\b60\b` classés. Modèles : Sonnet (rédaction), Opus (P1), Sonnet (P2), Opus (P3), Sonnet (P4), Opus (P5 ciblée). |
| 2026-11-18 | **Validation terminée — Change Log final** (une entrée). **Tendance des findings par passe** (CRITICAL/HIGH/MEDIUM/LOW) : P1 Opus 0/0/7/8 → P2 Sonnet 0/0/2/8 → P3 Opus 0/0/2/6 → P4 Sonnet 0/0/1/6 → P5 ciblée Opus 0/1/1/5 → P6 ciblée Sonnet 0/0/0/4 ; arrêt sur une passe sans finding au-dessus de LOW. **Modèles** : Sonnet (rédaction), Opus (P1), Sonnet (P2), Opus (P3), Sonnet (P4), Opus (P5 ciblée), Sonnet (P6 ciblée). **Signaux** : « recycle partiel » (V3-1 né du correctif V2-2) déclaré au Project Lead par l'orchestrateur à la P3 ; **le HIGH de la P5 (V5-1) venait d'une décision de l'orchestrateur (V4-5, omission de `type_ligne`), retirée** au profit du champ toujours présent. **Reclassements** : aucun. **LOW de la P6 corrigés après la dernière passe** (non relus par une passe de plus : fiche seule, aucun code de production) : V6-1 `LineType` à la liste `champ_enum!` de `sortie.rs:132`, V6-2 mesure MCP discriminante et conduite, V6-3 T1 en une liste numérotée de cinq mutations, V6-4 `:162` hors de la liste des sites sortis. Statut : `ready-for-dev`. |
| 2026-11-18/19 | **Développement** (Opus 5.5, `bmad-dev-story`) — détail au Dev Agent Record. Migration nommée `20261118000002_invoice_line_types.sql` (C-e491-1 : écrite le 2026-11-18 ; lire ce nom partout où la fiche écrit `20261119000001`). T0 : montée, rejeu, retour et remontée **tiennent** sur MariaDB 10.11.19, verdict D13 `yes`. **Sites et tests hors de la fiche, découverts au développement** (inventaire « clos » complété, Dev Notes (c)) : (1) **`crates/kesh-db/migrations.sha384`**, registre des checksums des migrations publiées (garde P8, `test_schema_guard.rs::published_migrations_keep_their_checksums`, rouge au premier `test-fast`) — ligne `20261118000002` ajoutée (commit `aae45788d`) ; (2) `explicit_line_account_sites` (création, modification, validation) alimenté par les seules lignes chiffrées ; (3) le sous-cas AC9 (b) « compte par défaut archivé » n'avait aucun test : `credit_note_unpriced_line_does_not_require_default_account` ajouté (tue la mutation des sites de l'avoir) ; (4) `crates/kesh-db/tests/common/mod.rs` gagne `colonnes_types_de_ligne`, posé puis retiré par les deux montages partiels nommés (motif : `invoices::create` écrit `line_type`). **Tests existants modifiés au-delà des littéraux** : `grouped_settlements.rs::the_upgrade_keeps_unit_settlements_and_allows_a_group` et `lettering_documents_backfill.rs::colonne_posterieure` (motif ci-dessus) ; `migrations_upgrade_path.rs` (P6). Aucun autre, `test-fast --ci` vert. Journaux de rouge dans `kesh-gate-logs/e491/mut/` (non `e491a/`, qui n'existe pas). **Écarts** : `cargo check` lancé une fois sous `mem-guard` seul, sans le verrou de la voie base (compilation sans MariaDB, voie tenue 25 minutes par le `gate-ci` de la 48-13) ; un remplacement multiple (`replace_all`) de l'outil Edit par fichier pour les littéraux ; aucun `sed -i` ni script sur le code hors du banc de mutations. **Frontend non touché** (`git diff --stat` ne nomme aucun fichier sous `frontend/`) ; ses gates ne sont pas relancés (consigne : « frontend complet s'il est touché »). **Ce qui n'a pas tourné** : E2E Playwright complet et démarrage sur base persistante (P8, `cargo run`), à la clôture (T7 reste ouverte pour eux). Recette visuelle : sans objet (aucun contrôle visible). Statut : `review`. |
| 2026-11-19 | **Revue de code P1** (3 × Opus 5.5, contexte frais, `kesh-gate-logs/e491/review-p1-{B,E,A}.md`) : B 0/0/0/4, E 0/0/2/3, A 0/0/1/8 — **total 0/0/3/15** (CRITICAL/HIGH/MEDIUM/LOW). **Remédiation** (décisions de l'orchestrateur ; la migration n'est **pas publiée**, corrigée sur place, commit `f283f018e`) : **E1 (MEDIUM)** — un échec entre les deux `ALTER` laisse une ligne `success = 0` (sqlx inscrit la migration avant le script ; le premier `ALTER` la valide), Kesh refuse de démarrer ; reprise `DELETE FROM _sqlx_migrations WHERE version = 20261118000002 AND success = 0` puis redémarrage, rejeu sans effet : en-tête de la migration, ligne d'audit `:118`, D13 et note à la 49-1d corrigés ; **même symptôme déjà publié** corrigé : paragraphe « Si la migration échoue au milieu » de la 0.19.0 (`admin-manual.tex:2578`) et ligne d'audit de `20261118000001` (`:117`) — l'en-tête de `20261118000001_grouped_settlements.sql:38-41`, publiée, garde l'ancienne phrase (P8, l'octet ne se touche pas), signalé à l'audit ; grep du symptôme (`success = 0`, `Dirty`, « non inscrite », `1060`) : les autres `1060` décrivent une ré-exécution manuelle hors sqlx (justes). Prouvé par `migrations_upgrade_path.rs::line_types_migration_resumes_after_a_half_run` (couvre B3, A5, E4) ; sonde T0 rejouée et tracée (`t0/sonde-v2.sh`, `t0/sonde-v2.log` : `VERSION()` 10.11.19, deux applications « OK », `'PRICED'`/`'priced '`/`'Comment'` refusés, retour par les `ALTER` extraits de l'en-tête, remontée, 14 `CHECK`). **E2/B2 (MEDIUM)** — `chk_*_line_type` fermée à l'octet (`BINARY`, précédent `20261110000001_bank_accounts_without_iban.sql`) ; T2 refuse `'PRICED'`, `'priced '`, `'Priced'`. **P8** : `migrations.sha384` passé à `6fbae70f…021c17b86`, squash régénéré ; **bases de dev** : seules mes bases `kesh_e491*` avaient appliqué l'ancienne version, remises à neuf ; relevé en lecture de `_sqlx_migrations` (version `20261118000002`) sur toutes les bases `kesh*` des conteneurs `kesh-mariadb-dev` (`kesh`, `kesh_e2e`, `kesh_e4815`, `kesh_ci_e491`, `kesh_e2e_e491`, `kesh_e491`) et `kesh-mariadb-dev2` : **aucune** ne la portait au moment du relevé. **A1/E5/B4 (MEDIUM)** — C-e491-6 **tranché par l'orchestrateur le 2026-11-19 : limite acceptée, suivie par #1347** ; seuils 84/94 qualifiés de bornes hautes (lignes minimales) ; plus grosse facture des données semées (`seed_demo` n'en sème aucune ; facture des pièges, une ligne de 1 000 caractères) : **1 414 o avec la clé, 1 385 o sans** (`cible-p1-2.log`) ; « non cassant » nuancé dans `api-external.md`, le manuel d'administration et la Note de version. **B1/E3/A7** — `"lineType": null` vaut l'absence (`deserialize_with`), tests unitaire et E2E, documenté ; **A8** « manquant ou nul » au manuel. **LOW** : A2 (T8 : contre-épreuve qui atteint l'assertion de parité, deux fichiers), A3 (Dev Agent Record : par quoi chaque mutation est tuée), A4 (`every_invoice_response_carries_the_line_type` : objet de ligne comparé clé par clé, `POST`, `PUT` réussi, `GET`, `validate`, `unvalidate`, règlement), A6 (AC3 f, AC4 c attestés), **A9** : la ligne `comment` de T10 est insérée en SQL brut (l'export lit la base, sans effet sur la preuve) et `admin_full_export_e2e` ne reçoit qu'un littéral (aucune assertion d'en-tête n'y existe). Mutations neuves : 7, toutes tuées par l'assertion visée. Gates : `test-fast --ci` 6163/6163 (`test-fast-p1.log`), `gate-ci` 6180 passés, 0 échoué (`gate-ci-p1.log`), au commit `f283f018e`. Ce qui n'a pas tourné : E2E complet, démarrage P8 (clôture). **Une passe P2 complète suit** (la migration change). |
