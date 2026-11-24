# Story 49-1d : types de lignes de la facture client — la recette de montée avec factures réelles simulées, et le passage à la 0.20.0 au manuel d'administration (#1242)

Status: review

*Fiche rédigée le 2026-11-19 par l'agent de rédaction de la 49-1d (Opus 5.5), par le workflow `bmad-create-story`. Release
**0.20.0** (coupure jeudi 2026-11-26 au soir). **Aucune migration, aucun code de production** : la story touche l'outillage de
recette (`scripts/`), son test, `docs/testing.md` et le manuel d'administration FR. Code lu à la tête de la branche
`story/49-1d-types-de-lignes-recette-montee` = `f57f6ac80` (49-1a, avec la migration `20261118000002`, et 49-1b fusionnées dans
`epic/49-types-de-lignes-facture`). Sources : `epic-49-types-de-lignes-facture.md`, `release-0.20.0.md`, les fiches closes
`49-1a-types-de-lignes-migration-api.md` (§ *Transmis à la 49-1d*) et `49-1b-types-de-lignes-pdf.md`, le précédent de la 0.19.0
(`kesh-gate-logs/rel0190/recettes-rel0190.sh`, `recette-complement-rel0190.sh`). Les choix de cette fiche sont D1 à D11.
Validation : P1 (Sonnet) faite et remédiée le 2026-11-19, sur les arbitrages de l'orchestrateur ; branche à jour d'`epic/49`
(`d4f5bba7f` : 49-4 au fichier d'épique, #1400 rattachée, « la 49-1c ne sort pas sans la 49-4 »). P2 (Opus) faite le 2026-11-19 ;
remédiée le 2026-11-20 par l'**allègement décidé par Guy** (section *Allègement*) : les gestes rejoués après la montée, la
reprise d'une migration interrompue, la constatation du refus d'une sauvegarde 0.20.0 et leurs contrôles annexes sortent de la
story. P3 (Sonnet) et P4 (Opus) faites et remédiées le 2026-11-20 ; P5 (Sonnet) le 2026-11-20, sans finding au-dessus de LOW :
validation terminée, ses quatre LOW traités à l'étape 0 du développement. La 49-4 a été découpée le 2026-11-20 en 49-4a et 49-4b (décision de Guy, branche
`story/49-4-designation-longue-pdf`, non encore fusionnée dans `epic/49`) : la dérogation au rendu est celle de la **49-4b** (D8).*

**Issues** : `refs #1242` ; **aucun mot-clé de fermeture** dans un commit. #1242 ne se ferme pas avec cette story.

**Couplage de coupure** : **la 49-1a ne sort jamais sans la 49-1d** (fichier d'épique, § *Règles de sortie*) ; les deux sortent
ensemble, ou la 0.20.0 ne porte aucune migration. **La 49-1d se rejoue au dernier commit de code de la release** (AC12).

## Story

En tant que Guy, qui tient ses livres réels en 0.19.0 sur le NAS et qui a déjà remis sur papier de vraies factures à un
locataire, je veux qu'avant de publier la 0.20.0 — la première release qui touche les lignes de facture — sa montée soit jouée,
depuis l'image publiée 0.19.0 et sur une installation jetable, sur une base peuplée **comme la mienne** : factures validées
payées et impayées, un avoir, un règlement groupé, des PDF déjà figés et d'autres jamais téléchargés ; et qu'elle prouve, facture
par facture, que **rien n'a bougé** — lignes, totaux, TVA, écritures, numéros, et le PDF, qu'il soit archivé ou rendu de
nouveau. Je veux aussi trouver au manuel d'administration, en tête de la section « Passer à la 0.20.0 », la sauvegarde à prendre
avant la mise à jour, et ce qu'il faut faire si elle tourne mal.

Le script `scripts/recette-montee.sh` existe (Story 33-10, #925) ; il sait monter, refuser, revenir et importer, mais son
peuplement ne porte **ni avoir, ni règlement groupé, ni PDF**, son empreinte ignore `credit_notes` et `credit_note_lines`, et
un fichier nommé par `--diff-attendu` cesse d'être comparé du tout. Cette story le rend capable d'éprouver la 0.20.0, sans en
faire un script propre à elle : tout ce qu'elle ajoute vaut pour les releases suivantes ; ce qui ne vaut que pour la 0.20.0 va
au crochet `--apres`.

## Ce qui se lit au code (`f57f6ac80`)

Chaque ligne citée est copiée du fichier, lue à la tête du worktree. `scripts/recette-montee.sh` est **identique** à celui du tag
v0.19.0 (`git show 773df80da:scripts/recette-montee.sh | diff - scripts/recette-montee.sh` : vide).

### Ce que le script peuple aujourd'hui

- Trois factures client, une ligne chacune, toutes au même taux et au même libellé :
  `scripts/recette-montee.sh:496` `  R=$(peupler facture POST /api/v1/invoices -d "{\"contactId\":$CLIENT,…\"lines\":[{\"description\":\"Conseil\",\"quantity\":\"$q\",\"unitPrice\":\"200.00\",\"vatRate\":\"8.10\"}]}") || exit $?`
  (boucle `for q in 4.5 1 2`, `:495`) ; la première et la deuxième sont validées, la deuxième réglée par un règlement unitaire
  (`:511`), la troisième reste brouillon.
- Les statuts relus en base, seule vérification du peuplement client :
  `:541` `[ "$(cat "$L/statuts-clients.txt")" = "validated,validated,draft" ] \` et
  `:543` `[ "$(SQL 'SELECT COUNT(*) FROM invoice_settlements')" = 1 ] || echec 1 "règlement client absent"`.
- **Aucun avoir, aucun règlement groupé, aucun PDF** : `git grep -n -e credit-notes -e invoice_group -e '/pdf' -- scripts/recette-montee.sh`
  ne rend rien. Le relevé importé (`scripts/fixtures/recette/releve-A.xml`) date ses deux crédits d'octobre 2026, avant les
  factures (datées du jour, `:476` `JOUR=$(date +%F)`) : il ne peut régler aucune d'elles.

### Ce que l'empreinte compare, et ce qu'elle ne compare pas

- `:414` `TABLES="journal_entries journal_entry_lines invoices invoice_lines invoice_settlements supplier_invoices supplier_invoice_lines bank_imports bank_transactions bank_accounts contacts accounts companies fiscal_years company_invoice_settings"`
  — **ni `credit_notes` ni `credit_note_lines`**.
- `:423` `    SQL "SELECT * FROM $t ORDER BY 1" > "$o/$t" || return 1` : toutes les colonnes. Une colonne ajoutée par la migration
  (`line_type`, `crates/kesh-db/migrations/20261118000002_invoice_line_types.sql:65` et `:76`
  `    ADD COLUMN IF NOT EXISTS line_type VARCHAR(16) NOT NULL DEFAULT 'priced',`) fait donc différer `invoice_lines` et
  `credit_note_lines` après la montée.
- `:437` `  for id in $(cat "$L/ids-factures"); do api GET "/api/v1/invoices/$id" | jq -S . >> "$o/api" || return 1; done` :
  la réponse d'une facture gagne `lineType` sur chaque ligne (`crates/kesh-api/src/routes/invoices.rs:192`
  `    pub line_type: LineType,`, champ de la réponse de ligne) : le fichier `api` diffère aussi.
- `:446` `  case ",$DIFF_ATTENDU," in *",$1,"*) return 0 ;; *) return 1 ;; esac` : un fichier nommé par `--diff-attendu` peut
  différer **de n'importe quelle façon**. Nommer `invoice_lines`, `credit_note_lines` et `api` ferait donc **cesser toute
  comparaison** des lignes de facture et des réponses de l'API — précisément ce que la 0.20.0 doit prouver. **Précédent** : la
  0.19.0 a dû nommer huit tables (`kesh-gate-logs/rel0190/recettes-rel0190.sh:8`
  `DIFF=${2:-migrations,accounts,companies,fiscal_years,journal_entry_lines,invoice_settlements,supplier_invoices,company_invoice_settings}`)
  et rattraper la perte par un script à part, hors dépôt, qui comparait « les colonnes relevées à l'état avant »
  (`recette-complement-rel0190.sh:107-113`, `information_schema.columns` par `ordinal_position`).
- Les documents : `:38-39` `# L'empreinte ne couvre pas tout non plus : `docs/testing.md` dit exactement ce qu'elle` /
  `# regarde, et aucun document n'est peuplé ni comparé.` ; `docs/testing.md:634` le dit aussi : « **Aucun document n'est
  peuplé ni comparé** : les pièces de l'image vivent dans son `/tmp` et disparaissent avec elle ». En effet l'image écrit ses
  documents dans le conteneur (`:391` `    -e "KESH_INBOX_DIR=/tmp/inbox" -e "KESH_DOCUMENTS_DIR=/tmp/documents" -e "KESH_ADMIN_BACKUP_DIR=/tmp/backup" \`),
  lancé `--rm`, sans montage (`:385` `# L'image précédente : aucun chemin de l'hôte, aucune variable héritée, `-e NOM=VALEUR`.`),
  tandis que le binaire neuf lit `$D/documents` (`:405`). Le test l'exige :
  `scripts/tests/recette-montee.test.sh:414` `if /usr/bin/grep -qE -- '--env-file|(^| )-v( |$)|--volume|--mount' "$FX/runs"; then fail "montage ou env-file"; else ok "aucun montage, aucun --env-file"; fi`.

### Le PDF d'une facture, figé ou non

- `crates/kesh-api/src/routes/issued_invoice_pdf.rs:9-14` (doc du module) : une facture **figée** relit le fichier sous
  `KESH_DOCUMENTS_DIR` et vérifie son SHA-256 ; une facture `validated` **non figée** se rend, puis se fige.
  `:104` `    if is_frozen(&invoice) {`. Le fichier porte le nom de son empreinte :
  `crates/kesh-api/src/document_storage.rs:117` `    let storage_path = format!("{sha256}.{}", ext.to_ascii_lowercase());`.
- Le regel : `issued_invoice_pdf.rs:238` `pub async fn refreeze(ctx: &PdfContext<'_>, invoice_id: i64) -> Result<IssuedPdf, AppError> {`
  ne rend un nouveau document que si le fichier **manque** (`:247` `        FrozenFile::Intact(_) => return Err(AppError::InvoicePdfPresent),`) ;
  route `crates/kesh-api/src/lib.rs:550` `            "/api/v1/invoices/{id}/pdf/refreeze",`.
- Routes (identiques au tag v0.19.0, `git show 773df80da:crates/kesh-api/src/lib.rs`, lignes 550, 1224, 1237) :
  `lib.rs:1224` `            "/api/v1/invoices/{id}/pdf",` ; `:1237` `            "/api/v1/credit-notes/{id}/pdf",`.
  **Le PDF d'un avoir n'est jamais figé** : il se rend à chaque demande (fiche 49-1b, § *Les chemins du PDF figé*).
- **Un rendu n'est pas reproductible à l'octet** : `crates/kesh-qrbill/tests/golden_test.rs:3-4`
  `//! Plan C (validé avec Guy) : on ne compare pas l'octet exact du PDF (printpdf` /
  `//! introduit un random dans le second élément du trailer `/ID`). À la place :` ; et la date de création suit l'horloge
  (`crates/kesh-qrbill/src/pdf.rs:52` `    generate_qr_bill_pdf_with_date(data, invoice, i18n, OffsetDateTime::now_utc())`),
  dans le seul dictionnaire `Info` : `grep -n 'now()\|today\|Local::now\|now_utc'` sur `invoice_pdf_service.rs`,
  `credit_notes.rs` et `pdf.rs` ne rend que `pdf.rs:46`, `:52`, `:133` — aucune date du jour n'est **dessinée**.
- La 49-1b a prouvé l'égalité du rendu **au niveau du code** et laisse explicitement à cette story la preuve sur base peuplée :
  fiche 49-1b, § *Où tourne le même rendu* : « la preuve 0.19.0 → 0.20.0 sur base peuplée est celle de la 49-1d ».
- La sauvegarde `.keshbackup` **porte les pièces** : `crates/kesh-api/src/admin_backup/import.rs:504`
  `/// Story 31-1a (AC8) — écrit chaque pièce de l'archive sous `documents_dir`,`.

### Les gestes du peuplement, dans la 0.19.0

Le peuplement se joue contre l'image **0.19.0** : chaque route est lue au tag (`git show 773df80da:crates/kesh-api/src/lib.rs`) :
`:723` `"/api/v1/credit-notes"` (création ; corps `CreateCreditNoteRequest { invoice_id, date }`,
`crates/kesh-api/src/routes/credit_notes.rs:144-147`, identique au tag) ; `:968` `"/api/v1/bank-imports/manual"` (relevé saisi à
la main, doc `docs/api-external.md:976`) ; `:987` `"/api/v1/reconciliation/accept"` (type `invoice_group`,
`docs/api-external.md:807-815` : « `invoiceIds` : 2 à 50 factures » ; une transaction au **crédit**, en CHF ; `:785` : seules
les factures « datées au plus tard du lendemain du paiement » ; le compte bancaire lié à un compte du grand livre, sinon
`BANK_ACCOUNT_NOT_CONFIGURED`, `:768`) ; `:1057`
`"/api/v1/bank-accounts/{id}"` (`PATCH` du compte du grand livre). Le précédent qui joue ces gestes sur une image publiée :
`kesh-gate-logs/rel0190/recette-complement-rel0190.sh:180-184` (compte bancaire lié au `1020`), `:237` (relevé saisi),
`:245` (groupe client accepté).

### Les fichiers de la 49-1a que cette story complète

- Le manuel d'administration porte le **squelette** : `docs/manual/fr/admin-manual.tex:2550`
  `\subsubsection{Passer à la 0.20.0}\label{sec:maj-0-20-0}`, son commentaire `:2551-2555` (« La Story 49-1d le complète : la
  sauvegarde d'avant la mise à jour, l'introduction […], « Une migration », « Compatibilité », « Le retour à la 0.19.0 », « Si la
  migration échoue au milieu » et « Ce qu'il faut savoir ».) et le seul paragraphe « Pour les intégrations ». Le modèle est la
  section 0.19.0, `:2564-2586`. Le paragraphe général « Compatibilité » des sauvegardes est `:2302` (il s'arrête à la 0.19.0).
- **Les manuels DE, EN, IT n'existent pas** (`docs/manual/de/README.md:3`, relevé par la 49-1a, V3-1) : FR seulement.
- Faits transmis par la 49-1a (§ *Transmis à la 49-1d*), relus au code :
  (a) **une sauvegarde 0.20.0 est refusée par la 0.19.0** : `git show 773df80da:crates/kesh-api/src/admin_backup/import.rs`
  `:615` `        // (c1) colonnes source inconnues côté destination.` puis `:633` `            return Err(AppError::ImportSchemaMismatch {`,
  message fr-CH (`crates/kesh-i18n/locales/fr-CH/messages.ftl:2460`) « Le schéma de ce backup est incompatible avec cette
  version de Kesh. Mettez à jour Kesh ou utilisez un backup compatible. ». Le refus **n'est pas écrit au journal du serveur** :
  `errors.rs:3167-3186` du tag construit la réponse `400` sans `tracing::` ; la table et la colonne sont dans le **corps de la
  réponse** (`error.details.table`, la première de l'ordre alphabétique — `credit_note_lines`, les tables étant tenues dans un
  `BTreeMap`, `import.rs:91` du tag — et `error.details.unknownColumns` = `["line_type"]`) (validation P2, V2-2). **L'écran
  n'affiche pas le message de l'API** (validation P3, V3-2) : l'écran de restauration de la 0.19.0 traduit le code lui-même,
  `git show 773df80da:frontend/src/lib/features/admin-restore/AdminRestorePanel.svelte`, `:189-193`
  `case 'IMPORT_SCHEMA_MISMATCH': {` … `'admin-restore-error-schema',` `'Schéma du backup incompatible avec cette version de Kesh (table { $table }).'`
  (clé fr-CH `messages.ftl:2435`) : l'administrateur lit « Schéma du backup incompatible avec cette version de Kesh (table
  credit_note_lines). » — la table est donc visible **à l'écran**, la colonne seulement dans la réponse. **Établi au code, non joué
  par cette story** (Allègement) ;
  (b) **reprise d'une migration interrompue** : `DELETE FROM _sqlx_migrations WHERE version = 20261118000002 AND success = 0`,
  puis redémarrer ; le rejeu est sans effet sur ce qui était posé (formes `IF [NOT] EXISTS`), prouvé par
  `migrations_upgrade_path.rs::line_types_migration_resumes_after_a_half_run`. **Le manuel ne la prescrit pas** (Allègement) : il
  garde la consigne de la 0.19.0, « ne reprenez pas le schéma à la main ». Kesh refuse alors de démarrer et écrit au journal
  `Échec des migrations : migration 20261118000002 is partially applied; fix and remove row from `_sqlx_migrations` table`
  (`crates/kesh-api/src/main.rs:218` ; `sqlx-core-0.8.6/src/migrate/error.rs:37-40`, version du `Cargo.lock`) — le mot *dirty*
  n'y est pas (V2-4).
- Décompte : `ls crates/kesh-db/migrations/*.sql | wc -l` → **96** à `f57f6ac80` (95 au tag v0.19.0).

### Les préconditions du PDF et de l'arrondi (validation P1, V1-1)

- Le rendu d'une facture exige un compte bancaire **principal** :
  `crates/kesh-api/src/routes/invoice_pdf_service.rs:245` `    let primary_bank = bank_accounts::find_primary(pool, company.id)`,
  qui ne retient que `crates/kesh-db/src/repositories/bank_accounts.rs:112`
  `            " FROM bank_accounts WHERE company_id = ? AND is_primary = TRUE AND archived = FALSE LIMIT 1",`. Le script crée son
  compte bancaire avec `"isPrimary":false` (`scripts/recette-montee.sh:550`) et le preset `with-company` n'en crée aucun : tout
  téléchargement de PDF serait refusé.
- Un arrondi non nul exige un compte de différences d'arrondi désigné : `crates/kesh-db/src/repositories/invoices.rs:2324`
  `            let account_id = company_invoice_settings::rounding_account_for_write(`. Le script n'écrit aux réglages que
  `defaultPayableAccountId` (`:514`) ; le champ est `default_rounding_account_id`
  (`crates/kesh-api/src/routes/company_invoice_settings.rs:149`, `defaultRoundingAccountId` en JSON).

### La 49-4 (fusionnée au fichier d'épique par `d4f5bba7f`)

`_bmad-output/planning-artifacts/epic-49-types-de-lignes-facture.md:130` : « désignation longue sur le PDF de facture et d'avoir,
retour à la ligne au lieu de la troncature à 44 caractères (#1363) ; capacité du PDF contrôlée avant la validation (#1364) ;
caractères que la police n'a pas […] signalés au lieu de disparaître (#1400, rattachée le 2026-11-19, décision de Guy) ;
s'applique aux factures émises à partir de la 0.20.0, le PDF archivé fait foi pour les anciennes ». `:154` : « **la 49-1c ne sort
pas sans la 49-4** ». Une facture d'avant **jamais téléchargée**, ou **regelée**, dont une désignation dépasse 44 caractères, se
rendra donc autrement en 0.20.0 si la 49-4 (aujourd'hui la 49-4b) y entre : c'est la dérogation de Guy (D8).

**Le découpage du 2026-11-20** (décision de Guy ; fiches lues sur la branche `story/49-4-designation-longue-pdf`, `55429142c`) :
la **49-4a** contrôle la capacité du PDF à la validation et **ne change aucun rendu** (« Toute facture, tout avoir, tout rappel se
rend octet pour octet comme avant », fiche 49-4a, `:26`) ; la **49-4b** imprime la désignation entière et replie ou signale
certains caractères — c'est elle qui porte la dérogation. Ses réponses de Guy du 2026-11-20 (fiche 49-4b, § *Réponses de Guy*) :
**Q3** « la dérogation s'étend à l'avoir » ; **Q1 (a, b)** « on imprime le rappel avec le nouveau rendu ». Ce qui **ne change
pas** avec la 49-4b (sa D3) : une désignation d'au plus 45 caractères tenant dans 88 mm, en caractères WinAnsi, sans blanc autre
que l'espace U+0020 — et, selon le remède de son V1-1 encore ouvert, sans espace double ni de bord. La règle de sortie de la 49-1c
devient « la 49-1c ne sort pas sans la 49-4a » (fiche 49-4a, `:28-29`).

## Inventaire par le symptôme

**Symptôme** : *une donnée d'une facture ou d'un avoir existants que la recette ne compare pas après la montée.* Liste close,
tirée des tables qui désignent une facture ou un avoir, non d'une liste de formes connues :

```
$ grep -nE 'REFERENCES `?(invoices|credit_notes|invoice_settlements)`?' crates/kesh-db/migrations/*.sql
20260827000001_invoice_settlements.sql:59:        FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE,
20260416000001_invoices.sql:53:        FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE,
20260715000001_invoice_reminders.sql:50:        FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE,
20260627000001_credit_notes.sql:46:        FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE RESTRICT,
20260627000001_credit_notes.sql:75:        FOREIGN KEY (credit_note_id) REFERENCES credit_notes(id) ON DELETE CASCADE,
```

| Site | Aujourd'hui | Ici |
|---|---|---|
| `invoices` (numéros, totaux, arrondi, `pdf_*`) | comparée (`:414`) | **reste comparée**, et le restera quand le fichier `invoices` n'est pas nommé par `--diff-attendu` |
| `invoice_lines` (`invoices.sql:53`) | comparée, mais **perdue** dès qu'elle est nommée par `--diff-attendu` | **résolu** : projection sur les colonnes d'avant (D2, AC3) |
| `invoice_settlements` | comparée | reste ; le règlement groupé y ajoute deux lignes (AC1) |
| `credit_notes` (`credit_notes.sql:46`) | **non comparée** | **résolu** : ajoutée à `TABLES` (AC2) |
| `credit_note_lines` (`credit_notes.sql:75`) | **non comparée** | **résolu** : ajoutée à `TABLES`, projection (AC2, AC3) |
| `invoice_reminders` (`invoice_reminders.sql:50`) | non comparée, **non peuplée** | **hors périmètre**, angle mort assumé : aucune migration de la 0.20.0 ne touche les rappels ; #1444 |
| écritures (`journal_entries`, `journal_entry_lines`) | comparées | restent ; l'écriture du groupe et la contre-passation de l'avoir y entrent (AC1) |
| réponse de l'API (`api`) | comparée, **perdue** si nommée | **résolu** : projection sur les clés d'avant (D2, AC3) ; les avoirs y entrent (AC2) |
| **le PDF** (dossier des documents) | **ni peuplé ni comparé** (`:39`, `testing.md:634`) | **résolu** : D3 à D5, AC4 à AC6 |
| séquences de numérotation (`invoice_number_sequences`, `credit_note_number_sequences`) | non comparées | hors périmètre : elles ne décrivent aucune pièce existante (le numéro de chaque pièce est dans `invoices` / `credit_notes`, comparées) ; même issue que les rappels, #1444 |

**Second symptôme (validation P1, V1-5 ; énoncé resserré par la P2, V2-11)** : *un lecteur ou un écrivain **filtré par le type
de ligne** (`line_type = 'priced'`, `is_priced()`), qui ne tourne jamais sur les données d'avant après la montée.* Des tables
identiques ne prouvent rien d'un lecteur filtré. Liste close, tirée du filtre lui-même :

```
$ git grep -n "line_type = 'priced'\|is_priced()" -- crates/*/src | grep -v test
crates/kesh-api/src/routes/invoices.rs:521:    if !req.line_type.is_priced() {
crates/kesh-db/src/repositories/credit_notes.rs:539:            .filter(|l| l.line_type.is_priced())
crates/kesh-db/src/repositories/credit_notes.rs:668:            let priced = || invoice_lines.iter().filter(|l| l.line_type.is_priced());
crates/kesh-db/src/repositories/invoice_settlements_write.rs:651:             WHERE invoice_id = ? AND line_type = 'priced'",
crates/kesh-db/src/repositories/invoices.rs:499:    if l.line_type.is_priced() {
crates/kesh-db/src/repositories/invoices.rs:511:        .filter(|l| l.line_type.is_priced())
crates/kesh-db/src/repositories/invoices.rs:523:        if l.line_type.is_priced() {
crates/kesh-db/src/repositories/invoices.rs:763:    l.line_type.is_priced() && l.revenue_account_id.is_none()
crates/kesh-db/src/repositories/invoices.rs:805:                .is_priced()
crates/kesh-db/src/repositories/invoices.rs:1369:                .is_priced()
crates/kesh-db/src/repositories/invoices.rs:2374:                    .is_priced()
crates/kesh-db/src/repositories/invoices.rs:2421:        // Story 49-1a (AC8 b) : `AND line_type = 'priced'` est OBLIGATOIRE —
crates/kesh-db/src/repositories/invoices.rs:2427:                 WHERE invoice_id = ? AND revenue_account_id IS NULL AND line_type = 'priced'",
crates/kesh-report/src/vat_report.rs:127:           AND il.line_type = 'priced' \
```

| Site | Après la montée, sur les données d'avant |
|---|---|
| `vat_report.rs:127` (rapport TVA) | **tourne** : `GET /api/v1/reports/vat` de la période du peuplement, dans `api` (projection par ajout, D10) |
| `invoices.rs:499`, `:511`, `:523`, `:763`, `:805`, `:1369`, `:2374`, `:2421-2427` (création, modification, validation, compte de produit) | **angle mort assumé** (Allègement) : la recette ne valide plus de brouillon d'avant. Sur des lignes toutes chiffrées, la parité avec la 0.19.0 repose sur la **suite existante passée inchangée** (contrôlée par T12 et au rejeu, AC12 ; perte assumée : `invoices_validate_vat`, `invoices_line_revenue_account`) ; le test de la 49-1a (`validation_ignores_unpriced_lines`) ne prouve que la neutralité des lignes non chiffrées ; #1445 |
| `credit_notes.rs:539`, `:668` (création d'avoir) | **angle mort assumé** (Allègement) : parité sur la suite existante inchangée (`credit_notes_repository`) ; `credit_note_copies_unpriced_lines` (49-1a) compare, dans le même code, un avoir avec et sans lignes non chiffrées — aucune facture d'avant la montée ; même issue, #1445 |
| `invoice_settlements_write.rs:651` (règlement) | **angle mort assumé** (Allègement) : filtre d'intention, sans effet sur des lignes toutes `priced` — sa suppression est une mutation **équivalente** (fiche 49-1a, tableau des mutations, `:624`) ; parité sur la suite existante inchangée (`invoice_settlement`, `grouped_settlements`, `invoice_amount_due_parity`) ; même issue, #1445 |
| `routes/invoices.rs:521` (garde de l'API, `lineType` du corps) | hors recette : un corps de la 0.19.0 est éprouvé par la 49-1a (T9 de sa fiche) |

Sites que la 49-1a a changés **hors du filtre**, relevés par une autre voie (`grep -rn "line_type" crates/*/src --include=*.rs`,
hors tests ; V2-11) :

| Site | Après la montée, sur les données d'avant |
|---|---|
| rendu PDF de la facture (`invoice_pdf_service.rs:41-50`, `:430`) et de l'avoir (`routes/credit_notes.rs:315`) | **tourne** : D4, D5 (rendus de F2, F4, F6, A1, regel de F8) |
| réponse d'avoir (`routes/credit_notes.rs:64`, `:77`) | **tourne** : `GET /api/v1/credit-notes/{id}` de A1 dans `api` (AC2) |
| export CSV (`exports/csv_tables.rs:650`, `:665`, `:1295`, `:1310`) | hors recette, éprouvé par la 49-1a (T10 de sa fiche) |
| sortie MCP (`mcp/outils/factures.rs:499`) | hors recette, éprouvée par la 49-1a (T11 de sa fiche) |

## Décisions

- **D1 — Le peuplement simule les factures réelles par l'API de la 0.19.0, sans un `INSERT`.** **Deux préconditions d'abord**
  (V1-1), contrôlées en base avant la première facture (code 11) : (a) le compte bancaire de la recette est **principal** — créé
  avec `"isPrimary":true`, ou rendu principal par `PUT` s'il existait déjà (`:548-552`) — et lié au compte `1020` (créé s'il
  manque) ; (b) les réglages de facturation désignent un **compte de différences d'arrondi** (`defaultRoundingAccountId`, dans le
  même `PUT` que `defaultPayableAccountId`, `:513-516`). Puis huit factures client (ordre de création = identifiants 1 à 8 sur une
  base neuve, celui que relit l'AC1) :

  | | Statut | Lignes | PDF en 0.19.0 | Rôle |
  |---|---|---|---|---|
  | F1 | validée, impayée | 1 ligne, 8.10 %, **description de plus de 44 caractères** | **figé** | PDF archivé servi tel quel |
  | F2 | validée, réglée (règlement unitaire, comme aujourd'hui) | 1 ligne | non figé | rendu 0.20.0 = rendu 0.19.0 |
  | F3 | brouillon | 1 ligne | — | brouillon d'avant, comparé par l'empreinte (en-tête et lignes) |
  | F4 | validée, impayée | **3 lignes**, taux 8.10, 2.60 et **0.00** (le cas d'un non-assujetti), arrondi à 5 centimes non nul | non figé | la facture de Guy jamais téléchargée |
  | F5, F6 | validées, **réglées ensemble** (`invoice_group`) | 1 ligne chacune | F5 **figé**, F6 non figé | règlement groupé |
  | F7 | validée, **figée**, puis **avoir** total (A1) → `cancelled` | 2 lignes, deux taux | **figé avant l'avoir** | facture annulée au PDF archivé ; PDF de l'avoir |
  | F8 | validée, **figée** ; son fichier **retiré** de la copie des documents (D3) | 1 ligne | figé, fichier manquant | **regel** par la 0.20.0 |

  Les descriptions de F2 à F8 restent dans ce que la 49-4b **ne change pas** (D8) : **au plus 44 caractères**, en ASCII imprimable
  (lettres, chiffres, ponctuation simple : ni trait d'union typographique, ni espace insécable, ni trait d'union conditionnel, ni
  saut de ligne), des espaces simples, aucune espace de tête ni de fin ; AC1 le contrôle. Chaque PDF téléchargé est contrôlé :
  statut HTTP 200, en-tête `%PDF-` (les cinq premiers octets), sinon la phase échoue en nommant la pièce et en gardant le corps reçu — jamais un refus JSON pris pour un PDF. Le règlement groupé passe par un
  relevé **saisi à la main** au jour de la recette (`POST /bank-imports/manual`, un crédit égal à la somme des TTC de F5 et F6) et
  `POST /reconciliation/accept` de type `invoice_group`, comme le précédent de la 0.19.0. *Écartées* : (a) un relevé CAMT en
  fixture — une date figée dans un fichier pourrirait (la facture doit dater au plus tard du lendemain du paiement,
  `api-external.md:785`) ; (b) des `INSERT` SQL de forme 0.19.0 comme la T1 de la 49-1a — c'est déjà sa preuve au niveau de la
  base ; la recette, elle, joue **l'application publiée** ; (c) une facture par cas croisé — huit factures couvrent les chemins du
  PDF (figé, non figé, annulé, regel) et des règlements (unitaire, groupé, avoir) sans doublon.
- **D2 — `--diff-attendu` n'admet plus, pour une table ou pour `api`, qu'un ajout.** Un fichier de table nommé par
  `--diff-attendu` peut différer, mais **sa projection sur les colonnes d'avant** — relevées à l'empreinte `avant` dans
  `information_schema.COLUMNS`, par `ORDINAL_POSITION`, chaque nom cité **entre apostrophes inverses** (`` `date` ``, `` `status` ``)
  — doit rester identique au fichier d'avant ; pour `api`, chaque document JSON d'après, **réduit aux clés présentes au même chemin
  dans le document d'avant** (récursivement, tableaux élément par élément, longueur égale et **même type JSON** exigés : un objet
  devenu scalaire est un écart), doit égaler celui d'avant. `migrations`, `min_required`, `users`, `audit_log` gardent leur sens
  (écart libre). Une colonne retirée ou renommée fait échouer la projection (erreur SQL) : la phase échoue, ce qui est voulu. La
  release se joue donc avec `--diff-attendu migrations,invoice_lines,credit_note_lines,api` (le fichier d'épique : « `--diff-attendu`
  des deux tables de lignes ») **sans rien perdre** de leur comparaison. *Écartées* : (a) nommer les deux tables sans projection
  (rien des lignes ne serait plus comparé) ; (b) une option neuve (`--ajouts-seuls`) à côté de `--diff-attendu` — deux listes pour
  un même fichier, et le sens libre actuel n'a servi qu'à perdre des comparaisons (précédent 0.19.0) ; (c) la projection au crochet
  `--apres`, hors dépôt : elle disparaîtrait avec la release, et ne joue pas à l'import. **Limites dites** : une release future qui
  **réécrit** une colonne existante verra la phase échouer et devra ajouter une option (aucune en 0.20.0 : la 49-1a est du DDL
  seul) ; la projection ne dit rien de la **valeur** d'une colonne neuve (le crochet, D7) ni du **schéma** (les `CHECK` assouplis
  sont éprouvés par la 49-1a, T2 de sa fiche). **Un écart légitime causé par une autre story** (V2-12) — un montant du rapport TVA
  arrondi autrement (48-11), une réponse de l'API changée — fait échouer la phase au rejeu de la release (AC12) comme tout autre
  écart : il ne se tolère pas en élargissant `--diff-attendu`, il revient à l'orchestrateur, qui le rapporte à la story qui l'a
  causé et décide avec le Project Lead, sur le modèle de D8.
- **D3 — Les documents de l'image sont copiés par `docker cp`, sans montage.** À la fin de la phase 1, avant d'arrêter l'image,
  `docker cp "$CT_IMAGE:/tmp/documents/." "$D/documents/"` (le conteneur porte le jeton du run, vérifié comme pour `docker stop`) ;
  puis le fichier de F8 est retiré de `$D/documents`. L'invariant « aucun chemin de l'hôte » (`:385`) et son test (`test.sh:414`)
  **restent**. *Écartée* : un montage `-v "$D/documents:/tmp/documents"` — il brise l'invariant et son test, et l'image tourne en
  `root` (le `Dockerfile` ne pose aucun `USER`) : le binaire neuf, lancé par l'utilisateur, ne pourrait plus écrire dans un
  dossier créé par `root`.
- **D4 — Un PDF archivé se compare à l'octet ; un rendu se compare par son contenu dessiné.** (a) **Figé** (F1, F5, F7) : les
  octets servis par le binaire neuf égalent ceux téléchargés en phase 1, et leur SHA-256 égale `pdf_sha256` relu en base.
  (b) **Rendu** (F2, F4, F6, A1, et le regel de F8) : chaque page est **tramée** par `mutool draw -q -r 300 -c rgb -F pnm`
  (**300 points par pouce**, fixés au script) et l'empreinte MD5 de chaque trame égale celle de la référence ; le **texte**
  (`mutool draw -q -F txt`) est égal ; le **nombre de pages** est égal et au moins 1 ; le code de sortie de `mutool` est contrôlé ; la
  trame est supprimée après son empreinte. Le texte sert à **lire** un écart, la trame à n'en laisser passer aucun.
  **Ce que « trame et texte identiques » prouve** : la page dessinée est la même au pixel près à 300 dpi — textes, positions,
  traits, couleurs, QR-facture — avec le même `mutool` (sa version, `mutool -v`, s'écrit au journal). **Ce qu'il ne prouve pas** :
  l'identité du **fichier** (le dictionnaire `Info` — producteur, dates —, le `/ID`, la structure des objets, les annotations et
  liens, l'ordre de dessin, un texte invisible ou hors page), ni un décalage inférieur à un tiers de pixel à 300 dpi (≈ 0,08 mm).
  Pour une facture qui sera réimprimée puis figée, c'est « le même document à l'œil et le même QR », non « le même PDF » : la fiche,
  `docs/testing.md` et le manuel le disent ainsi. *Écartées* : (a) l'octet du rendu (le `/ID` change, `golden_test.rs:3-4`) ;
  (b) l'empreinte de la 49-1b (`kesh-qrbill`, feature `test-support`) — aucun binaire ne l'expose au shell ; (c) `pdftotext` seul
  (ne voit ni position, ni trait, ni QR) ; (d) les octets après effacement du `/ID` et des dates (printpdf y mêle un identifiant
  d'instance aléatoire, `crates/kesh-qrbill/src/pdf.rs:55-56` « *modulo* printpdf's internal random / instance id » : fragile) ;
  (e) 72 dpi (un demi-millimètre du bloc de paiement y passerait).
- **D5 — La référence d'un rendu 0.19.0 se prend en phase 4, sur la base restaurée.** Télécharger F2, F4, F6 en phase 1 les
  figerait dans la base d'avant. La phase 4 restaure la base d'avant sous l'image 0.19.0 : **après** son empreinte (inchangée),
  elle rend F2, F4, F6 et l'avoir A1 — la référence 0.19.0 —, **avant** `journal_sans_erreur 4` et `arreter_image`
  (`scripts/recette-montee.sh:628-634`) : un rendu qui journalise ` ERROR ` fait échouer la phase 4. Le conteneur du retour est
  neuf (`--rm`) et son dossier des documents vide : F1, F5, F7, F8 y manquent, sans effet pour les quatre pièces non figées
  rendues ; aucune pièce figée n'y est demandée (elle y rendrait `410`) (V3-9). La phase 2 (binaire neuf) les rend **après** son empreinte et le
  crochet. La référence du regel de F8 est son fichier figé de la phase 1. Une **phase 6 « Rendus »** (code **16**) compare, après
  les phases 4 et 5, **un nombre exact de paires : 5**, avec ou sans `--import` (F2, F4, F6, A1, regel de F8) ; une paire ne compte
  que si les deux PDF ont passé D1 (200, `%PDF-`), ont au moins une page et si **leur texte contient le numéro de la pièce**
  (`invoiceNumber`, ou le numéro de l'avoir, relus à l'API) — un rendu vide ou étranger ne compte pas. Un décompte différent de 5
  échoue (code 16). *Écartée* : relancer l'image en phase 1 après le
  dump pour prendre les références puis restaurer (un cycle de plus, même preuve).
- **D6 — La phase 5 importe dans un dossier de documents neuf.** `lancer_binaire` reçoit le dossier des documents en argument ;
  l'import écrit dans `$D/documents-import`, vide : un PDF figé servi après l'import vient **de l'archive**. F1, F5, F7 **et F8**
  (dont l'archive a gardé le fichier) y sont servis à l'octet. *Écartées* : (a) réutiliser `$D/documents` ; (b) rendre de
  nouveau F2, F4, F6 et A1 après l'import (Allègement) : c'est le code de rendu de la phase 2 sur les mêmes données, et l'empreinte
  de la phase 5 prouve déjà ces données égales.
- **D7 — Ce qui ne vaut que pour la 0.20.0 va au crochet `--apres`**, écrit sous `kesh-gate-logs/rel0200/verifs-0200.sh` par
  cette story, copié **en entier** au Dev Agent Record et relu en revue comme du code (`docs/testing.md`, § `--apres`). Contrôles :
  toutes les lignes des deux tables sont `priced` (`SELECT COUNT(*) FROM invoice_lines WHERE line_type <> 'priced'` = 0, de même
  pour `credit_note_lines`) ; leur nombre égale celui d'avant, **lu au fichier** `$L/empreinte-avant/invoice_lines` (et
  `…/credit_note_lines`), une ligne par enregistrement — le crochet n'a pas d'autre mémoire ; `COLUMN_DEFAULT` vaut `'priced'`,
  **apostrophes comprises** (forme de MariaDB 10.11), sur les deux tables. **Chaque contrôle s'écrit `[ … ] || exit 1`** (jamais `[ … ] && …`, jamais
  dans un `if` ni un `local x=$(…)` : les exceptions de `set -e` que l'en-tête du script documente, `:32-37`). *Écartée* : ces
  contrôles dans le script (il deviendrait propre à une release).
- **D8 — Désignations : ce que la recette compare, et la dérogation de la 49-4b.** La 49-4 (fichier d'épique, `:130`, fusionné par
  `d4f5bba7f`), découpée le 2026-11-20, porte par sa **49-4b** l'impression d'une désignation longue en entier au lieu de la couper
  à 44 caractères, et le repli ou le signalement de certains caractères (sa D3, sa D4) ; Guy a admis la dérogation pour une facture
  d'avant **jamais téléchargée** ou **regelée** (2026-11-19), puis l'a étendue à **l'avoir** et au **rappel** (2026-11-20, 49-4b,
  Q3 et Q1). D'où : (a) F1, longue, est **figée** : son PDF archivé ne change jamais, 49-4b ou non, et se compare à l'octet ;
  (b) **F2 à F8, et donc A1** (qui recopie les lignes de F7 et se rend à chaque demande, jamais figé), restent dans ce que la
  49-4b ne change pas (D1 : au plus 44 caractères, ASCII imprimable, espaces simples, aucune espace de bord) : leur rendu doit être
  identique, quelle que soit la liste des stories de la release. La recette tenait déjà l'avoir pour couvert par la dérogation
  (A1 est sous 44 caractères pour cette raison) : la décision Q3 la confirme sans rien changer à ce qu'elle compare. **Le rappel**
  n'est ni peuplé ni rendu par la recette (inventaire, `invoice_reminders` hors périmètre) : la décision Q1 ne touche pas ce
  qu'elle compare. (c) **Une désignation longue rendue de nouveau** (facture jamais téléchargée, regel, avoir) **n'est pas jouée
  par la recette** (Allègement : l'option `--designation-longue` sort) : si la 49-4b entre, son rendu **change** par décision de
  Guy, et la preuve de ce changement est celle de la fiche de la 49-4b ; si elle n'entre pas, l'identité du rendu d'une
  désignation de plus de 44 caractères repose sur le test avant / après de la 49-1b (cas S7, « une description dépasse 45
  caractères », fiche 49-1b, `:299-300`), au niveau du code. Tout **autre** écart de rendu fait échouer la phase 6, quelle que soit
  la story qui l'a causé (48-15, #1327 de Beat, 49-3…), et revient à l'orchestrateur.
- **D9 — `mutool` est un outil de la recette, remplaçable au test par `KESH_MUTOOL`.** Ajouté à la liste des outils (`:123`,
  absent → code 5) avec `sha256sum` ; `KESH_MUTOOL` traverse la relance comme `KESH_DOCKER` et `KESH_CURL` (`:65`, `:81-83`), et
  l'enveloppe Rust le retire (`.env_remove`) comme elles. **Le test pose `KESH_MUTOOL` à chaque scénario** (vers son factice, ou
  vers un chemin absent) : le `mutool` de l'hôte n'est **jamais** atteint par le test, et la relance remettant `PATH=/usr/bin:/bin`
  (`:57`), un `PATH` de test ne suffirait pas à le masquer. Le factice consigne chacun de ses appels dans un journal du mini-dépôt :
  un scénario nominal exige ce journal non vide. *Motif* : la CI n'installe pas `mutool` (`grep -n 'apt-get'
  .github/workflows/ci.yml` : `mold`, `mariadb-client` seulement), la station l'a (`/usr/bin/mutool`) : sans ces deux règles, une
  preuve du rouge dépendrait de l'hôte (V1-6). *Écartée* : sauter la phase 6 si `mutool` manque.
- **D10 — Après la montée, le rapport TVA tourne sur les données d'avant, sous les deux versions.** Le rapport TVA de la période
  du peuplement entre dans `api` (`GET /api/v1/reports/vat`, `lib.rs:1443`, requête `ReportQuery`, `reports.rs:855`, bornée par
  `fiscal_year_id`, `period_start`, `period_end`, `reports.rs:64-68`), sous la projection par ajout (D2) : rendu par la 0.19.0 à
  l'empreinte `avant`, par la 0.20.0 à l'empreinte `apres`. C'est le seul lecteur filtré par `line_type` que la recette fait
  tourner (second inventaire) ; un lecteur filtré à tort laisserait les tables identiques, d'où ce rapport. *Écartées* : (a) se
  fier à l'égalité des tables ; (b) les **créances âgées** (V2-6) : la route ne prend aucune période (`reports.rs:455-462`), son
  `asOf` est le jour du serveur (`services/rapports.rs:733`) et ses tranches en dépendent (`kesh-report/src/aged_receivables.rs:84-89`)
  — une recette qui passe minuit rougirait à tort, et ce n'est pas un lecteur de lignes ; (c) **les gestes d'écriture rejoués
  après la montée** sur les données d'avant — validation d'un brouillon, création d'un avoir, règlement —, comparés à ceux de la
  0.19.0 : sortis par l'**Allègement** (décision de Guy du 2026-11-20) ; leur parité sur des lignes toutes chiffrées repose sur
  la suite de tests existante, passée inchangée (perte assumée, section *Allègement* ; contrôle à T12, à la tête de la story,
  rejoué sur la release assemblée par l'AC12).
- **D11 — Le retour que le manuel prescrit se joue, dans un script d'intégration sur le modèle de la 0.19.0.**
  `kesh-gate-logs/rel0200/recette-complement-rel0200.sh` (hors dépôt, comme `rel0190/recette-complement-rel0190.sh`), écrit et
  joué par cette story, rejoué par l'intégration ; conteneurs `kesh-rel0200c-*`, label `kesh.rel0200c`, base `mariadb:10.11` en
  tmpfs, valeurs factices ; il prend en argument le dossier `$D` laissé par la recette (AC10) et y lit `backup/pre-upgrade/`. **Un
  seul jeu, le retour par `pre-upgrade/`** : une image 0.19.1 neuve (0.19.0 avant la reprise du 2026-11-24), base vide, son administrateur créé au démarrage par
  `KESH_ADMIN_USERNAME` / `KESH_ADMIN_PASSWORD` (valeurs factices), importe la sauvegarde `pre-upgrade/` écrite par la montée par
  `POST /api/v1/admin/full-import` — la route qu'appelle le lien *Restaurer une sauvegarde* de l'assistant (`lib.rs:372` du tag,
  `frontend/src/lib/features/admin-restore/admin-restore.api.ts:64` du tag ; même route que le précédent,
  `rel0190/recette-complement-rel0190.sh:292`) (V2-5) : `200`, et l'empreinte des tables de `TABLES` égale **strictement** celle
  d'avant de la recette (`$D/journal/empreinte-avant/`).
  **Le nom de la sauvegarde** (V3-1) : **exactement un** fichier `kesh-pre-upgrade-*-0.19.1-to-*.keshbackup` sous
  `$D/backup/pre-upgrade/`, sinon échec ; la version d'arrivée se relève du nom et doit égaler la version que le binaire neuf
  rend à `/health` (`routes/health.rs:25`, `env!("CARGO_PKG_VERSION")`), que la phase 2 de la recette écrit au journal
  (`$D/journal/health-binaire.json`, T3). Le nom vient de `crates/kesh-api/src/main.rs:179` `let vers = env!("CARGO_PKG_VERSION");`
  et de `pre_upgrade_backup.rs` (`kesh-pre-upgrade-{…}Z-{de}-to-{vers}.keshbackup`) : **à la tête de la story, les crates sont en
  0.19.1** depuis la reprise de `main` (`crates/kesh-api/Cargo.toml:3`) et le fichier finit par `-0.19.1-to-0.19.1.keshbackup` ; il ne finit par
  `-0.19.1-to-0.20.0.keshbackup` qu'après le bump de `scripts/prepare-release.sh` (AC12).
  **Les PDF figés reviennent** (V3-4) : l'import remplace utilisateurs et société, et la session ouverte avant lui doit être
  rouverte (`admin-manual.tex`, juste avant `\subsubsection{Passer à la 0.20.0}`) ; le script fait donc, **après l'import, un
  nouveau login avec les identifiants de la base importée** — ceux de la recette, `admin` et le mot de passe factice de
  `scripts/recette-montee.sh:219-220` —, jamais avec le jeton d'avant l'import (V4-3). Puis il sert `GET
  /api/v1/invoices/{id}/pdf` de F1, F5 et F7 par la 0.19.1 neuve, et leurs octets égalent ceux téléchargés en phase 1 de la
  recette (gardés sous `$D`) ; F8, dont le fichier manquait quand la montée a écrit la sauvegarde (D3), compte seulement si sa
  réponse est `410` **et** porte le code `INVOICE_PDF_GONE` (`crates/kesh-api/src/errors.rs:2835`) — attendu, et dit au
  journal ; un `401`, un `404` ou un `410` d'un autre code est un échec. Les tables seules ne verraient
  pas une pièce perdue à l'écriture ou à l'import de la sauvegarde (une pièce absente n'y est qu'un `warn!`,
  `admin_backup/export.rs:363-368`). Journal attendu : `kesh-gate-logs/e491d/recette-complement.log` (à la
  story), `kesh-gate-logs/rel0200/recette-complement-*.log` (au rejeu). Le manuel ne dit « constaté » que ce que ce jeu et la
  recette ont établi (AC14). *Écartées* : (a) laisser le retour à #920 et ne l'établir qu'au code ; (b) le mettre dans
  `scripts/recette-montee.sh`, dont la phase 4 revient par le dump ; (c) **le refus, par une 0.19.0 neuve, d'une sauvegarde
  écrite par la 0.20.0** et **la reprise d'une migration interrompue avec le vrai binaire** : sortis par l'**Allègement**
  (décision de Guy du 2026-11-20) — le premier est établi au code du tag, le manuel le dit ainsi ; pour la seconde, le manuel garde
  la consigne de la 0.19.0.

## Allègement (décision de Guy du 2026-11-20)

**Pourquoi.** La validation P2 (`kesh-gate-logs/e491d/validation-49-1d-p2.md`) a rendu trois MEDIUM, tous nés de ce que la
remédiation de la P1 avait ajouté (D10 et D11) : le signal de recyclage de la règle de splitting préventif. Elle proposait de
sortir ces ajouts sans affaiblir la preuve que la story doit apporter — **factures et avoirs existants intacts à la montée, PDF
compris, et retour arrière possible**. Guy a accepté cet allègement le 2026-11-20.

**Ce qui reste** : la comparaison avant / après la montée de toutes les factures et de tous les avoirs, PDF compris, sans
comparaison possible à vide (chaque paire de rendus contient le numéro de la pièce ; les altérations volontaires R1 et R2, et
la trame de F2 contre F4, font échouer la recette) — D1 à D9, AC1 à AC7, AC9, AC10 ; le rapport TVA avant / après (D10, AC8) ; le
retour arrière par la sauvegarde d'avant la mise à jour (D11, AC11).

**Ce qui sort** :

1. **Les gestes rejoués après la montée sur les données d'avant** : validation du brouillon F3, avoir A2 sur F4, règlement de F1,
   joués en phase 2 et en phase 4, leurs extraits comparés en phase 6 (ex-D10, ex-AC8 pour sa part « gestes », ex-T15, ex-R4).
2. **La reprise d'une migration interrompue avec le vrai binaire** (ex-D11 c, ex-AC11 c) et le paragraphe du manuel qui la
   prescrivait. Le manuel garde la consigne de la 0.19.0 : « ne reprenez pas le schéma à la main » ; revenez par la sauvegarde
   d'avant la mise à jour (AC14).
3. **La constatation du refus, par la 0.19.0, d'une sauvegarde faite en 0.20.0** (ex-D11 a, ex-AC11 a, et l'export
   `apres-0.20.0.keshbackup` du crochet qui la servait). Le manuel dit ce refus **établi par le code** (`import.rs:615-633` du tag),
   non « constaté ».
4. **Les contrôles annexes désignés avec ces retraits** (avis de charge de la P2) : les **créances âgées** dans `api` (V2-6) ;
   l'option **`--designation-longue`** et son scénario (ex-T16) ; les **rouges R3 et R4 dans le script** (la trame reste éprouvée
   hors script, F2 contre F4, et par le factice, S6) ; les **rendus après l'import** (ex-paires 6 à 9 : le décompte passe de 9 à 5
   avec `--import`, les PDF figés restant servis à l'octet de l'archive).

**La perte assumée.** La recette ne prouve plus, sur l'image publiée et sur une base peuplée par elle, que valider un brouillon
d'avant, émettre un avoir sur une facture d'avant ou en encaisser une **écrit en 0.20.0 ce qu'écrivait la 0.19.0**. Après la
migration, une ligne d'avant est une ligne chiffrée (`DEFAULT 'priced'`) : sur de telles lignes, cette parité repose sur la
**suite de tests existante**, écrite pour la 0.19.0 et **passée inchangée** en 0.20.0 — au niveau du code, sur des données
synthétiques. « Inchangée en 0.20.0 » est contrôlé **deux fois** : à la tête de la story (T12) et **au rejeu de l'AC12**, sur la
release assemblée, toutes stories fusionnées (V4-1), **dès la coupure du jeudi 2026-11-26 au soir** (V5-3) ; d'autres stories de la release touchent ces fichiers ou le code qu'ils
exercent (la 48-14 et #1255 pour le règlement groupé, la 48-11 pour l'arrondi du rapport TVA). Elle tient en ces binaires : validation et compte de produit (`invoices_validate_vat`,
`invoices_line_revenue_account`), avoir (`credit_notes_repository`), règlement unitaire et groupé, reste dû (`invoice_settlement`,
`grouped_settlements`, `invoice_amount_due_parity`, `invoice_ttc_parity`, `reconciliation_group_e2e`), rapport TVA
(`vat_report_reconciliation`, `vat_report_e2e`). Ils passent au gate complet ; T12 prouve qu'aucune de leurs valeurs attendues n'a
changé depuis le tag v0.19.0, et l'AC12 le reprouve sur la release. **Un retrait d'une valeur attendue venu d'une autre story
n'est jamais toléré en silence** : il revient à l'orchestrateur, sur le modèle de D2, qui décide avec le Project Lead si la perte
assumée tient encore — la décision de Guy a été prise sur cette prémisse. Les tests ajoutés par la 49-1a dans ces fichiers (T5 à T8 de sa fiche) prouvent **autre chose** :
qu'une ligne non chiffrée est ignorée par ces calculs — non que le calcul sur des lignes chiffrées est resté celui de la 0.19.0
(V3-3). Le filtre du règlement (`invoice_settlements_write.rs:651`) est un filtre d'intention, dont la suppression est une
mutation équivalente (fiche 49-1a, `:624`). De même, la reprise d'une migration
interrompue n'est prouvée que par `migrations_upgrade_path.rs::line_types_migration_resumes_after_a_half_run` — c'est pourquoi le
manuel ne la prescrit pas —, et le refus d'une sauvegarde 0.20.0 par la 0.19.0, que par la lecture du code du tag. Si la 49-4b
n'entre pas dans la release, le rendu inchangé d'une désignation de plus de 44 caractères repose sur la 49-1b (D8 c). Issues
ouvertes pour ce qui sort : #1445, #1446, #1447 (Dev Notes).

## Critères d'acceptation

**Le script (`scripts/recette-montee.sh`)**

- **AC1 — Peuplement (D1).** Les deux préconditions d'abord, contrôlées en base (code 11) : exactement un compte bancaire
  `is_primary = TRUE AND archived = FALSE`, lié au `1020` ; `company_invoice_settings.default_rounding_account_id IS NOT NULL`. Puis
  la phase 1 crée, par l'API de l'image, les huit factures, l'avoir A1 et le règlement groupé de D1, et télécharge le PDF de F1, F5,
  F7 et F8 **avant** l'avoir (F7 se fige validée) ; chaque téléchargement : HTTP 200 et en-tête `%PDF-`, sinon code 11 nommant la
  pièce. Contrôles relus en base, chacun en code 11 : statuts
  `validated,validated,draft,validated,validated,validated,cancelled,validated` ; `invoice_settlements` = 3 (F2, F5, F6), dont 2
  portant `journal_entry_line_id` ; `credit_notes` = 1 ; `invoices.pdf_sha256 IS NOT NULL` pour exactement F1, F5, F7, F8 ; F4 a trois
  lignes, trois taux distincts dont `0.00`, et `rounding_amount <> 0` ; aucune description de F2 à F8 ne dépasse 44 caractères ni ne
  sort de la forme de D1 (ASCII imprimable, espaces simples, aucune espace de bord). Les factures fournisseur et le relevé A restent tels quels. Aucune donnée réelle (AC-7 du test
  inchangé).
- **AC2 — Empreinte.** `TABLES` gagne `credit_notes` et `credit_note_lines` ; `api` gagne `GET /api/v1/credit-notes/{id}` de chaque
  avoir peuplé et le rapport TVA de la période du peuplement (D10). `docs/testing.md` liste exactement ce que l'empreinte couvre.
- **AC3 — `--diff-attendu` par ajout (D2).** Pour une table ou `api` nommés, la projection sur l'avant est identique, sinon la
  phase échoue (code 12 à la montée, 15 à l'import) en nommant le fichier et en écrivant le diff de la projection au journal. Les
  autres noms gardent l'écart libre. La phase 4 (retour) reste à l'égalité **stricte** de tous les fichiers.
- **AC4 — Documents et PDF archivés (D3, D4 a).** Phase 1 : copie par `docker cp` du dossier des documents de l'image (conteneur
  du jeton seulement), puis retrait du fichier de F8 ; aucun montage (`test.sh:414` reste vert). Phase 2, **après** l'empreinte
  et le crochet : F1, F5, F7 servis par le binaire neuf, octets égaux à ceux de la phase 1 (SHA-256 égal aussi à `pdf_sha256`) ;
  F8 : `GET …/pdf` → `410` ; `POST …/pdf/refreeze` → `200` (une réponse JSON de facture, `routes/invoice_pdf.rs:78`, `:93`, non
  un PDF) ; puis `GET …/pdf` → `200`, en-tête `%PDF-`, SHA-256 égal au nouveau `pdf_sha256` relu en base — ce PDF est « le PDF du
  regel » (V2-7) ; F2, F4, F6 et A1 rendus et gardés pour la phase 6.
- **AC5 — Rendus (D4 b, D5) : phase 6, code 16.** Après la phase 4 (qui rend F2, F4, F6, A1 sous l'image 0.19.0, **après** son
  empreinte) : chaque rendu 0.20.0 a, page à page, la même trame à 300 dpi, le même texte et le même nombre de pages (≥ 1) que sa
  référence 0.19.0 (pour F8 : son fichier figé de la phase 1). **Décompte exact** : 5 paires, avec ou sans `--import` ; une paire
  ne compte que si ses deux PDF sont valides (D1) et que leur texte contient le numéro de la pièce. Un écart nomme la pièce et écrit le
  diff du texte au journal ; un décompte faux échoue. La version de `mutool` s'écrit au journal.
- **AC6 — Import (D6).** Avec `--import`, la phase 5 importe dans `$D/documents-import` (neuf) ; F1, F5, F7, F8 y sont servis à
  l'octet de la phase 1 ; l'empreinte suit AC3. Aucun rendu n'y est refait (Allègement).
- **AC7 — Outils (D9).** `mutool` et `sha256sum` sont exigés (code 5 s'ils manquent) ; `KESH_MUTOOL` traverse la relance et rien
  d'autre de neuf ; l'en-tête du script (`:10-19`, phases et codes) et `:38-39` disent la phase 6 et ce qui est désormais comparé.
- **AC8 — Rapport TVA sur les données d'avant (D10).** Le rapport TVA de la période du peuplement, rendu par la 0.19.0 à
  l'empreinte `avant` et par la 0.20.0 à l'empreinte `apres` (et après l'import), passe la projection de l'AC3 ; à l'empreinte
  `avant`, il porte un chiffre d'affaires non nul (contrôle de non-vacuité, code 11 ; le champ exact se relève à T0) — un rapport
  vide des deux côtés ne compte pas.

**Les factures réelles (consigne de production)**

- **AC9 — Les factures et avoirs existants restent identiques, et la recette le prouve sur l'image publiée.** Après la montée de
  0.19.0 à la tête de la story : lignes (toutes les colonnes d'avant), totaux, arrondi, TVA par ligne, rapport TVA, écritures,
  numéros, règlements (unitaires et groupé), avoir et ses lignes — **identiques** (AC3) ; PDF archivés servis à l'octet (AC4) ; PDF
  rendus de nouveau — facture jamais téléchargée, regel, avoir — **au même dessin** (AC5 ; ce que ce mot couvre et ne couvre pas :
  D4) pour les désignations de la forme de D1 (D8) ; rapport TVA identique (AC8) ; la même chose après l'import d'une sauvegarde
  0.19.0 (AC6), PDF figés servis à l'octet de l'archive. Ce que deviennent les gestes courants sur ces pièces après la montée n'est
  pas prouvé ici : la parité repose sur la suite existante passée inchangée (Allègement, perte assumée), contrôlée à la tête de
  la story (T12) et sur la release assemblée (AC12).

**Le jeu réel**

- **AC10 — Recette réelle, verte, à la tête de la story, avec ses rouges réels.** Sous les deux verrous de la voie « Base et
  Rust » et `mem-guard` (`docs/agents-orchestration.md:196`, ligne qui nomme les recettes de montée ; V2-10), depuis le worktree :
  ```sh
  flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh scripts/recette-montee.sh \
    --image gcorbaz/kesh:0.19.1 --binaire target/debug/kesh-api --static frontend/build \
    --port-api 3197 --port-base 3398 --migrations-attendues 1 \
    --diff-attendu migrations,invoice_lines,credit_note_lines,api \
    --apres ../kesh-gate-logs/rel0200/verifs-0200.sh --import --prefixe kesh-e491d
  ```
  → code 0, six phases jouées, la phase 3 « refusée : migration inconnue d'elle (sqlx) » (`20261118000002`), 5 paires de rendus
  comparées ; sortie copiée au Dev Agent Record (`kesh-gate-logs/e491d/recette-reelle.log`). **Rouges réels**, chacun son journal
  sous `kesh-gate-logs/e491d/` :
  (R1) `--diff-attendu migrations` seulement → code 12 nommant `invoice_lines`, `credit_note_lines` et `api`
  (`recette-reelle-rouge-diff.log`) — l'empreinte voit la migration ;
  (R2) **projection** : une copie mutée du script, **non suivie et retirée ensuite** (`scripts/recette-montee.mut.sh`, sous
  `RACINE`), exécute juste avant `empreinte apres` `SQL "UPDATE invoice_lines SET unit_price = unit_price + 0.0001 WHERE id = 1"` →
  code 12 nommant `invoice_lines` (`recette-reelle-rouge-projection.log`) — une valeur d'avant altérée rougit malgré `--diff-attendu`.
  **Sensibilité de la trame, hors script** : sur les vrais PDF du journal, la ligne `mutool` de D4 appliquée à F2 contre F2 rend des
  MD5 **égaux**, à F2 contre F4 des MD5 **différents** (commande et sortie au Dev Agent Record) — la trame distingue deux factures
  réelles ; la comparaison elle-même est éprouvée dans le script par le factice (S6).
  Après les rouges, `git status --porcelain` ne nomme pas la copie mutée.
- **AC11 — Le retour du manuel (D11).** `kesh-gate-logs/rel0200/recette-complement-rel0200.sh` joué sur le `$D` de l'AC10 :
  exactement une sauvegarde `kesh-pre-upgrade-*-0.19.1-to-*.keshbackup`, dont la version d'arrivée égale celle de `/health` du
  binaire neuf (à la tête de la story : `-0.19.1-to-0.19.1`, D11), importée par `POST /api/v1/admin/full-import` dans une 0.19.1
  neuve → `200`, empreinte égale **strictement** à celle d'avant ; nouveau login avec les identifiants de la recette, puis PDF de
  F1, F5, F7 servis à l'octet de la phase 1, F8 → `410` avec le code `INVOICE_PDF_GONE` (V3-4, V4-3). Texte du script et sortie (`recette-complement.log`) au Dev Agent Record.
- **AC12 — Rejeu au dernier commit de code de la release.** La fiche et `docs/testing.md` disent que l'intégration de la 0.20.0
  rejoue l'AC10 (vert et R1) et l'AC11 au dernier commit de code de la release (toutes stories fusionnées), journaux sous
  `kesh-gate-logs/rel0200/`, et que la section du manuel (AC14) ne dit « constaté » que ce que ce rejeu a établi. **Le rejeu se
  fait après le bump de `scripts/prepare-release.sh`** (lancé par l'intégration, jamais par cette story), binaire reconstruit
  sur le commit qui porte le bump : c'est alors seulement que `/health` rend `0.20.0` et que la sauvegarde `pre-upgrade/` finit
  par `-0.19.0-to-0.20.0.keshbackup` (V3-1). Il recompte aussi `ls crates/kesh-db/migrations/*.sql | wc -l` avant que le manuel
  écrive le total (V3-6). **Il rejoue aussi le contrôle de la perte assumée** (V4-1) : le contrôle de diff de T12, par
  `kesh-gate-logs/rel0200/perte-assumee.sh` (V5-4 ; commande restreinte aux dix binaires nommés à la perte assumée et aux modules
  qu'ils incluent, relevés par le script), sortie au journal `kesh-gate-logs/rel0200/perte-assumee.log`, retraits et ajouts triés
  par la règle de T12 ; un retrait d'une valeur attendue venu d'une autre story (48-14, 48-11 ou toute autre) revient à
  l'orchestrateur, sur le modèle de D2, qui décide avec le Project Lead si la perte assumée tient encore ; tant qu'il n'a pas
  tranché, le rejeu n'est pas vert. **Ce contrôle ne dépend pas du bump** (V5-3) : il se joue **dès la coupure du jeudi
  2026-11-26 au soir**, sur la branche assemblée, pour que la décision éventuelle se prenne le jeudi ; il se rejoue au dernier
  commit de code si des fusions ont suivi. Seuls la recette réelle et le script complémentaire attendent le bump, le vendredi
  avant midi, sur la voie « Base et Rust ». **Un rejeu non vert le vendredi à midi** — contrôle de la perte assumée, recette ou
  script complémentaire — **vaut le repli de l'ordre de coupe** : la migration et tout l'Epic 49 sortent de la 0.20.0, qui part
  sans migration ; l'orchestrateur le constate et le dit au compte rendu, le Project Lead garde le dernier mot. Un écart de
  rendu, de rapport ou de réponse causé par une autre story revient à l'orchestrateur (D2, D8), jamais toléré en élargissant
  `--diff-attendu`. Le crochet `verifs-0200.sh` et `recette-complement-rel0200.sh` existent et se relisent en revue.

**Documentation**

- **AC13 — `docs/testing.md`** (§ *La recette de montée et de retour*) : exemple d'appel à jour ; sens de `--diff-attendu` (D2) ;
  empreinte (AC2) ; documents et PDF (« Aucun document n'est peuplé ni comparé » retiré, remplacé par ce qui l'est et ce qui ne
  l'est pas, avec la réserve de D4 sur « même dessin ») ; le peuplement, ses préconditions et ses contrôles (AC1) ; le rapport TVA
  (AC8) ; la phase 6, le code 16 et le décompte exact ; `mutool`, `sha256sum`, `KESH_MUTOOL` ; la durée **mesurée** du test ; le
  rejeu au dernier commit de code (AC12) et le script complémentaire (D11) ; **ce que la recette ne prouve pas** (section
  *Allègement* : gestes après la montée — dont la parité repose sur la suite existante passée inchangée, contrôlée au rejeu —, reprise d'une migration
  interrompue, refus d'une sauvegarde plus récente, rendu d'une désignation longue).
- **AC14 — Manuel d'administration FR, section « Passer à la 0.20.0 »** (`docs/manual/fr/admin-manual.tex`, `.tex` seulement,
  sur le modèle de `:2564-2586`), **en tête** : « **Prenez avant la mise à jour une sauvegarde manuelle complète** — la base, un
  `.keshbackup` et le dossier des documents » et « le retour à la 0.19.1 passe uniquement par la sauvegarde d'avant la mise à
  jour » ; puis l'introduction (« La 0.20.0 se pose sur la 0.19.1 », ni compose ni variable neuve) ; **« Une migration »**
  (`20261118000002\_invoice\allowbreak\_line\allowbreak\_types` ; le total, 96 à la tête de la story, est celui que recompte le
  rejeu, AC12 ; non breaking, `kesh_version_min_required` reste à `0.17.6` ; aucune valeur existante réécrite : la colonne neuve
  prend sa valeur par défaut, « ligne chiffrée », sur chaque ligne existante, `ADD COLUMN … DEFAULT 'priced'`, V3-6 ; sauvegarde
  `pre-upgrade/` au nom finissant par `-0.19.0-to-0.20.0.keshbackup` — constaté seulement si le rejeu suit le bump, sinon établi
  par le code, D11 et AC12 ; `KESH_SKIP_PRE_UPGRADE_BACKUP=0.20.0`) ; **« Le retour à la 0.19.0 »** (la 0.19.0 ne démarre
  plus sur la base migrée — journal « migration 20261118000002 was previously applied but is missing in the resolved migrations »,
  constaté par la phase 3 de la recette ; elle refuse une sauvegarde 0.20.0 — **établi par le code** de la 0.19.0, non constaté —,
  dit comme la section 0.17.5 (`admin-manual.tex:2674`) par son code, `IMPORT\_SCHEMA\_MISMATCH` (`400`), **distinct**
  d'`INVALID\_BACKUP\_STRUCTURE`, et, si le texte de l'écran est cité, par celui que l'écran affiche réellement : « Schéma du backup
  incompatible avec cette version de Kesh (table credit\_note\_lines). » (§ *Les fichiers de la 49-1a* (a)) — jamais le message
  de la réponse de l'API ; la colonne en cause n'est que dans la réponse, rien n'est écrit au journal du serveur (V2-2, V3-2) ;
  retour sur une base neuve par la sauvegarde d'avant la mise à jour : **vidage de sécurité de la base actuelle**, puis
  installation 0.19.0 neuve sur une base vide, import depuis l'assistant, lien *Restaurer une sauvegarde* — constaté par l'AC11,
  PDF archivés compris ; **tout ce qui a été saisi depuis la mise à jour est perdu, et les numéros écrits sur le papier depuis la
  mise à jour sont à refaire** (phrases des sections 0.19.0 et 0.17.5, V3-5) ; une sauvegarde 0.19.0 s'importe dans la 0.20.0 —
  constaté par la phase 5) ; **« Si la migration échoue au milieu »**, sur la consigne de la 0.19.0 : Kesh ne redémarre
  plus, et le journal porte « Échec des migrations : migration 20261118000002 is partially applied; fix and remove row from
  `_sqlx_migrations` table » (message cité au code, D11 et § *Les fichiers de la 49-1a*, non le mot *dirty*) ; **ne reprenez pas le
  schéma à la main, même si ce message vous y invite** ; revenez par la sauvegarde d'avant la mise à jour, comme au retour
  ci-dessus. Le paragraphe **ne reprend pas** la phrase de la 0.19.0 sur l'erreur `1060` (fausse pour cette migration, écrite pour se
  rejouer) et **ne prescrit aucun** `DELETE` sur `_sqlx_migrations` ;
  **« Ce qu'il faut savoir »** : factures et avoirs existants inchangés ; PDF archivé servi tel quel ; une facture jamais téléchargée,
  une facture regelée et un avoir **se rendent au même dessin qu'en 0.19.1 pour des désignations de 44 caractères au plus** (en
  caractères courants : la forme de D1) ; **si la 49-4b est dans la release** (liste des stories fusionnées au dernier commit de
  code), la phrase nomme la dérogation de Guy : sur une facture jamais téléchargée, une facture regelée, **un avoir et un rappel**,
  une désignation de plus de 44 caractères s'imprime désormais en entier, sur plusieurs lignes, au lieu d'être coupée, et les
  caractères que la 49-4b replie s'impriment autrement (la liste est celle que la fiche de la 49-4b arrête, sa D3) — le PDF déjà
  archivé, lui, ne change pas ; sinon, la phrase vaut pour toute désignation, sans dire « constaté » au-delà de 44 caractères (preuve
  au niveau du code, 49-1b, D8 c). « Constaté » ne s'écrit que pour ce que l'AC10, l'AC11 et leur rejeu (AC12) ont joué. Le
  paragraphe « Pour les intégrations » de la 49-1a est **gardé** ; le commentaire `:2551-2555` retiré ou réécrit. Le paragraphe
  « Compatibilité » (`:2302`) gagne la phrase de la 0.20.0 : aucune table neuve, mais l'import compare aussi les **colonnes**
  (une colonne de la sauvegarde inconnue de la version qui l'importe, ou une colonne obligatoire absente de la sauvegarde) ; ce
  refus porte **son propre code**, `IMPORT\_SCHEMA\_MISMATCH`, et l'écran de restauration le dit « Schéma du backup incompatible
  avec cette version de Kesh (table …) », en nommant la table (V3-2) ; une sauvegarde 0.19.1 s'importe dans la 0.20.0, ses lignes devenant
  chiffrées ; l'inverse est refusé (\S\ref{sec:maj-0-20-0}). Le manuel compile (`make fr` vert, `Overfull \hbox` relevés), **PDF non
  commité**.

**Périmètre et gates**

- **AC15 — Aucun code de production.** `git diff --stat origin/epic/49-types-de-lignes-facture...HEAD -- crates frontend` (trois points :
  depuis la base de fusion, insensible aux stories fusionnées entre-temps dans `epic/49`, V4-5) ne nomme
  que `crates/kesh-db/tests/recette_montee_script.rs` (une ligne `.env_remove("KESH_MUTOOL")`, et la borne de durée si la mesure
  l'exige) ; aucune migration (`ls crates/kesh-db/migrations/*.sql | wc -l` = 96).
- **AC16 — Gates.** Pendant la boucle : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo nextest run -E 'binary(recette_montee_script)'` (gate ciblé) et `shellcheck` sur les scripts s'il est installé. Au dernier
  commit de code : bases `kesh_e491d`, `kesh_ci_e491d`, `kesh_e2e_e491d` remises à neuf ; `scripts/gate-ci.sh` sur `kesh_ci_e491d`
  vert ; **E2E Playwright complet** « vert » au sens de `docs/testing.md` § *Les échecs attendus* (règle du `CLAUDE.md` : la story
  touche du code exécutable) ; AC10 et AC11 rejoués à ce commit. Chaque commande lourde sous les verrous de sa voie
  (`docs/agents-orchestration.md:196-197`) : voie « Base et Rust » (`flock /tmp/station-heavy.lock flock "$GATE_LOGS/.gate.lock"
  scripts/mem-guard.sh …`) pour les recettes, le script complémentaire, `gate-ci` et l'E2E ; voie « Frontend et manuels »
  (`flock "$GATE_LOGS/.gate-front.lock" scripts/mem-guard.sh …`) pour `make fr` (V2-10). Recette visuelle : sans objet (aucun
  contrôle visible).

## Tests à écrire, et la preuve du rouge

Tous dans `scripts/tests/recette-montee.test.sh` (factices : `docker`, `curl`, binaire, et désormais `mutool`), lancé au gate par
`crates/kesh-db/tests/recette_montee_script.rs`. Les scénarios s'appellent **S1 à S14** (préfixe distinct des tâches T0 à T12,
V2-9). **Chaque scénario pose `KESH_MUTOOL`** (D9), y compris les lancements du test qui n'appellent pas `lancer()` (relevés par la
P2 : `test.sh:314`, `:321`, `:362`, `:458`, `:473`, `:487`, `:565`), et l'assertion « liste exacte » de la relance (`test.sh:368`)
gagne `KESH_MUTOOL`. Chaque preuve du rouge est une **mutation du script** (`scripts/recette-montee.sh` du mini-dépôt, copie
modifiée), le test relancé **sur la station** (où `/usr/bin/mutool` existe : la preuve ne doit pas en dépendre), le journal gardé
sous `kesh-gate-logs/e491d/mut/rouge-S<n>.log`, puis la mutation retirée. Les factices évoluent : le `docker` factice connaît `cp`
(copie un PDF factice dans la cible), les requêtes de colonnes (`information_schema`), les projections et les nouveaux décomptes ;
le `curl` factice sert `GET …/pdf` (corps commençant par `%PDF-` ; `410` pour F8 avant son regel), `POST …/pdf/refreeze` (`200`,
corps JSON), `/credit-notes`, `/reconciliation/accept`, `/bank-imports/manual`, `/reports/vat`, et consigne les corps reçus ; le
`mutool` factice écrit une trame, un texte (contenant le numéro de la pièce) et un nombre de pages qui dépendent d'un réglage, et
consigne chacun de ses appels.

| # | Scénario | Ce qu'il prouve | Preuve du rouge |
|---|---|---|---|
| S1 | nominal, `--diff-attendu migrations,invoice_lines,credit_note_lines,api`, `ECART=invoice_lines` (le factice ajoute une colonne : `SELECT *` change, la projection non) | code 0, six phases ; le journal du `mutool` factice est non vide | — (témoin des autres) |
| S2 | `ECART_ANCIEN=invoice_lines` (une colonne d'avant change) avec le même `--diff-attendu` | code 12, sortie nomme `invoice_lines` (AC3) | retirer la comparaison de projection → code 0 : **rouge** (`rouge-S2.log`) |
| S3 | `ECART_ANCIEN=api` (une clé d'avant change, dont une du rapport TVA) / `ECART=api` (une clé s'ajoute) / `ECART_TYPE=api` (un objet devient scalaire) | 12 / 0 / 12 (AC3, AC8) | projection `api` remplacée par `true` → S3a rend 0 : **rouge** ; contrôle de type retiré → S3c rend 0 : **rouge** |
| S4 | `ECART_ANCIEN=credit_note_lines` **sans** la nommer | code 12 (elle est dans `TABLES`, AC2) | retirer `credit_note_lines` de `TABLES` → code 0 : **rouge** |
| S5 | `PDF_FIGE_ALTERE=oui` (le `curl` factice sert d'autres octets en phase 2) | code 12, nomme la facture (AC4) | retirer la comparaison des octets → **rouge** |
| S6 | `RENDU_ECART=oui` (le `mutool` factice rend une autre trame pour une pièce) ; nominal : au journal des factices, les quatre `GET …/pdf` de la phase 4 précèdent l'arrêt du conteneur du retour (D5, V3-9) | code 16, nomme la pièce (AC5) ; ordre des rendus de la phase 4 | retirer la comparaison des trames → code 0 : **rouge** ; déplacer les rendus de la phase 4 après `arreter_image` → l'assertion d'ordre **rougit** |
| S7 | `RENDU_VIDE=oui` (le factice rend un texte sans le numéro, ou zéro page) ; `PDF_PAS_PDF=oui` (le `curl` factice sert un corps JSON en 200) | 16 ; 11 (D1, D5) | compter les paires sans contrôler le numéro → S7a rend 0 : **rouge** ; retirer le contrôle `%PDF-` → S7b ne rend pas 11 : **rouge** |
| S8 | `REGEL_ECHEC=oui` (refreeze `409`) | code 12 (AC4) | ignorer le statut du regel → **rouge** |
| S9 | nominal : `docker cp` appelé sur le conteneur du jeton, vers `$D/documents` ; `test.sh:414` (aucun montage) **inchangé et vert** | D3 | remplacer `docker cp` par un `-v` → l'assertion `:414` **rougit** |
| S10 | `STATUTS_CLIENTS=` autre / `NB_AVOIRS=0` / `PRINCIPAL=0` / `ARRONDI_COMPTE=null` / `TVA_VIDE=oui` (rapport TVA sans chiffre d'affaires) | code 11 (AC1, AC8) | retirer chaque contrôle → **rouge** |
| S11 | `KESH_MUTOOL` vers un chemin absent | code 5, zéro appel `docker` (AC7) | retirer `mutool` de la liste des outils → le script atteint la phase 6 et sort autrement qu'en 5 : **rouge** (le chemin absent est imposé par `KESH_MUTOOL`, le `mutool` de l'hôte n'est jamais pris) |
| S12 | `--import` nominal ; `PDF_IMPORT_ALTERE=oui` | 0 ; 15 (AC6) | importer dans `$D/documents` au lieu du dossier neuf : le factice le voit (il consigne le `KESH_DOCUMENTS_DIR` reçu) → **rouge** |
| S13 | la relance : `KESH_MUTOOL` traverse — le journal du `mutool` **factice** est non vide après le run —, une variable `KESH_AUTRE` ne traverse pas | D9 | retirer `KESH_MUTOOL` de la liste blanche → le factice n'est plus appelé, son journal reste vide : **rouge**, quel que soit l'hôte |
| S14 | décompte des paires : 5 sans `--import`, 5 avec ; `PAIRES_EN_MOINS=1` (le factice fait échouer un téléchargement d'une pièce rendue) | 0 ; 16 (AC5) | remplacer le décompte exact par « au moins une » → **rouge** |

**Le jeu réel** (AC10) est l'épreuve de bout en bout, avec ses rouges réels R1 et R2 et la sensibilité de la trame éprouvée hors
script ; le script complémentaire (AC11) a ses propres contrôles (exactement un fichier `pre-upgrade/`, `200`, empreinte stricte),
éprouvés par un rouge : une empreinte d'avant altérée d'une ligne fait échouer la comparaison, et une copie de la phase 1 du PDF de F1
altérée d'un octet fait échouer celle des PDF (V3-4) (journal `recette-complement-rouge.log`). **E2E Playwright neuf** : aucun — la story ne change ni écran ni réponse HTTP ; la suite complète
tourne tout de même au dernier commit de code (AC16).

## Tâches

- [x] **T0 — Sonde, avant tout changement** (journal `kesh-gate-logs/e491d/t0/`) : `docker image inspect gcorbaz/kesh:0.19.1`
  (tirer si absente) ; `command -v mutool sha256sum` et `mutool -v` ; jouer **à la main**, contre une image 0.19.0 jetable (préfixe
  `kesh-e491dt0`, base `mariadb:10.11` en tmpfs), les gestes de D1 et le rapport de D10 que le script ne fait pas encore : compte
  bancaire principal et lien au `1020`, compte d'arrondi désigné, relevé saisi au jour (continuité de solde avec le relevé A : les
  drapeaux `confirm*` du précédent), `invoice_group` accepté, avoir, téléchargement d'un PDF (le fichier apparaît-il sous
  `/tmp/documents` du conteneur ? `docker cp` le rend-il lisible par l'utilisateur ?), les taux `2.60` et `0.00` présents pour la
  société du preset et acceptés à la validation, un `410` puis un regel, le rapport TVA, ses paramètres (`ReportQuery`) et le champ
  du chiffre d'affaires (AC8) ; noter si un `410` ou un regel écrit une ligne ` ERROR ` (sinon `journal_sans_erreur` échouerait).
  Vérifier `mutool draw -F pnm` (sortie standard ou `-o`, sortie multipage), et que `mutool draw -q -F txt` sur le PDF d'une
  facture **et** sur celui d'un avoir de la sonde rend leur numéro (`invoiceNumber`, numéro de l'avoir ; police intégrée,
  `crates/kesh-qrbill/src/pdf.rs:329`) — sans quoi aucune paire de D5 ne compterait (V3-7). Toute surprise s'écrit au Dev Agent Record **avant**
  d'écrire le script ; une prémisse fausse de D1 à D11 arrête la story et se dit.
- [x] **T1 — Peuplement** (AC1) : préconditions, huit factures (descriptions de la forme de D1), avoir, groupe, contrôles.
- [x] **T2 — Empreinte et projection** (AC2, AC3, AC8) : relevé des colonnes à `avant`, projections, `api` par clés et types,
  rapport TVA et son contrôle de non-vacuité, diff au journal.
- [x] **T3 — Documents et PDF** (AC4, AC6) : `docker cp`, retrait de F8, phase 2 (servis, `410`, regel, rendus ; corps de
  `/health` du binaire neuf écrit à `$D/journal/health-binaire.json`, lu par D11), phase 5 (dossier neuf, PDF figés servis à
  l'octet).
- [x] **T4 — Phase 4 et phase 6** (AC5) : rendus 0.19.0 après l'empreinte du retour, comparaison trame (300 dpi) + texte + pages,
  décompte exact (5), code 16.
- [x] **T5 — Outils et relance** (AC7, D9) ; enveloppe Rust (`.env_remove("KESH_MUTOOL")`).
- [x] **T6 — Test shell** (S1 à S14) et mutations ; **durée mesurée avant la revue**, sous nextest, à la station (deux tirages) ;
  borne de `recette_montee_script.rs` (`timeout -k 10 600`, `:25`) relevée seulement si la mesure l'exige (dire la valeur).
- [x] **T7 — Crochet** `kesh-gate-logs/rel0200/verifs-0200.sh` (D7), texte intégral copié au Dev Agent Record.
- [x] **T8 — Recette réelle** (AC10), ses rouges réels R1 et R2, la sensibilité de la trame hors script ; copie mutée retirée.
- [x] **T9 — Script complémentaire** `kesh-gate-logs/rel0200/recette-complement-rel0200.sh` (D11, AC11) : nom `-0.19.1-to-*` et
  version d'arrivée égale à `/health`, import, empreinte stricte, **nouveau login après l'import** avec les identifiants de la
  recette (`admin`, mot de passe factice de `recette-montee.sh:219-220`), puis PDF de F1, F5, F7 à l'octet et F8 → `410` avec le
  code `INVOICE_PDF_GONE` (V4-3) ; joué avec son rouge,
  texte et sortie au Dev Agent Record (à la tête de la story, le nom finit par `-0.19.1-to-0.19.1`, D11).
- [x] **T10 — `docs/testing.md`** (AC13) ; grep du symptôme « aucun document » (`git grep -n -i -e "aucun document n'est peuplé" -e
  "vivent dans son" -- scripts docs crates`) : tous les sites traités.
- [x] **T11 — Manuel d'administration** (AC14) ; `make fr` sous le verrou de la voie « Frontend et manuels » et `mem-guard`
  (`flock "$GATE_LOGS/.gate-front.lock" scripts/mem-guard.sh make -C docs/manual fr`), puis `git checkout -- docs/manual/fr/*.pdf`
  (PDF jamais commités) ; vérifier le texte rendu (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`) avant
  le `checkout` : aucun `DELETE`, aucun « dirty », le message de sqlx et « établi » pour le refus d'import.
- [x] **T12 — Gates** (AC16) et Dev Agent Record (ce qui a tourné, et seulement cela). **Contrôle de la perte assumée** (V3-3) :
  aucune valeur attendue de la suite existante n'a changé depuis le tag. **Seuls les retraits se citent** au Dev Agent Record,
  avec leur fichier (V4-2 ; la sortie entière des ajouts et retraits ferait plus de 5 000 lignes à la tête de la story) :
  `git diff -U0 773df80da..HEAD -- 'crates/*/tests/*' | grep -E '^(\+\+\+ b/|-)' | grep -vE '^--- (a/|/dev/null)'` ; chaque
  ligne retirée d'un fichier de test qui existait au tag est citée et classée par la règle ci-dessous — aucune ne touche une
  valeur attendue d'un test de lignes chiffrées. Le pathspec se prouve par une sonde **sur le motif lui-même** (V4-2) :
  `git diff --numstat 773df80da..HEAD -- 'crates/*/tests/*' | grep migrations_upgrade_path` est non vide et rend des retraits
  non nuls (à `6bb2d5770` : `774 14`), alors que `-- 'crates/*/tests'` (sans `/*`) rend zéro ligne ; les deux sorties se citent.
  **Les dix binaires de la perte assumée** (liste rejouée par l'AC12) : `crates/kesh-db/tests/{invoices_validate_vat,
  invoices_line_revenue_account,credit_notes_repository,invoice_settlement,grouped_settlements,invoice_amount_due_parity,
  invoice_ttc_parity}.rs`, `crates/kesh-api/tests/{reconciliation_group_e2e,vat_report_e2e}.rs`,
  `crates/kesh-report/tests/vat_report_reconciliation.rs`, **et les modules qu'ils incluent**, relevés par
  `grep -hoE '#\[path = "[^"]+"\]|^mod common;'` sur ces dix fichiers (à `6bb2d5770` : `crates/kesh-db/tests/support/{jour,
  reglement_version,document_group}.rs`, `crates/kesh-db/tests/common/mod.rs`, `crates/kesh-api/tests/common/`). À
  `6bb2d5770`, leur seul retrait est une ligne de commentaire de `grouped_settlements.rs`.
  **Retrait de valeur attendue ou retrait sans effet** (V4-1, V5-1). Un retrait est **sans effet** seulement s'il est (a) un
  commentaire (`//`, `///`, `//!`) ; (b) une ligne `use` ou `mod` ; (c) **le décompte des migrations lui-même**, porté de la
  valeur de la release précédente à celle de la release qui ajoute la migration (un littéral comparé à
  `ls crates/kesh-db/migrations/*.sql | wc -l`, ou `migrations.len()`) — aucune autre valeur ; (d) une ligne reprise à
  l'identique, au blanc près, parmi les ajouts **de la même hunk** (`git diff -U0` : le même bloc de lignes retirées et ajoutées
  contiguës ; reformatage) — une ligne reprise ailleurs dans le fichier n'est **pas** couverte — et chaque cas se cite avec la
  ligne qui le remplace, s'il y en a une. **Toute autre ligne retirée est un retrait de valeur
  attendue** : une assertion (`assert…!`, `.expect…`), un littéral comparé (montant, taux, quantité, statut, chaîne attendue),
  une donnée de fixture qui nourrit un calcul comparé, un test entier ou son attribut ; **le doute se tranche en « valeur
  attendue »**. **Les ajouts se classent aussi** (V5-2), dans les mêmes fichiers : (e) un **ajout qui neutralise** un test
  existant — `#[ignore`, `#[cfg`, ou `return;` / `return Ok(());` seul sur sa ligne (le motif `return\b` est resserré : toute
  fonction auxiliaire en rendait) ; (f) un **ajout qui nomme un type de ligne non chiffré** — `LineType::Comment`,
  `LineType::Heading`, `'comment'`, `'heading'`, ou `line_type` / `lineType` sans le type chiffré — **dans une fonction qui
  existait au tag** : il change ce que prouve un test existant (des lignes non chiffrées là où la prémisse de Guy les veut toutes
  chiffrées) sans retirer une ligne. Un tel ajout dans une fonction **neuve** (absente du tag : les tests de la 49-1a, T5 à T8 de
  sa fiche) est sans effet sur la suite existante ; un ajout (e) ou (f) dans une fonction ancienne compte comme un retrait de
  valeur attendue. **La commande** (V5-4) est livrée par cette story, restreinte aux dix fichiers et à leurs modules :
  `kesh-gate-logs/rel0200/perte-assumee.sh <dépôt> [base=773df80da] [tête=HEAD]` — elle relève les modules (`#[path = …]`,
  `mod x;`, un module `mod.rs` pris avec tout son dossier), prouve son extracteur par une sonde positive
  (`migrations_upgrade_path.rs`, qui porte des retraits), sort chaque hunk qui retire une ligne avec ses retraits classés
  (a), (b), (d) ou « À TRIER », puis les ajouts (e) et (f), chacun rattaché à la fonction qui l'englobe et à sa présence au tag ;
  code 0 si rien n'est à trier ni à classer, 3 sinon (lire, trier par cette règle, citer), 4 si un fichier ou un module manque
  ou si la sonde ne rend rien. (c) se tranche à la main. Angle mort écrit dans le script : un ajout hors fonction se rattache
  à la fonction précédente. Un retrait de valeur
  attendue dans ces fichiers, à la tête de la story, arrête la story et se dit ; au rejeu (AC12), il revient à l'orchestrateur.
  Les binaires nommés à la perte assumée passent au gate complet.

## Note de version

*(Pour le CHANGELOG, assemblée à l'intégration ; la story n'écrit pas `CHANGELOG.md`.)*

> Le manuel d'administration décrit le passage à la 0.20.0 : la sauvegarde complète à prendre avant la mise à jour (base,
> `.keshbackup` et dossier des documents), le retour à la 0.19.1 par cette sauvegarde, et la conduite à tenir si la migration
> échoue au milieu. La montée a été éprouvée, depuis l'image publiée 0.19.1, sur une base de factures simulées — payées, impayées,
> réglées en groupe, annulées par un avoir, au PDF archivé ou jamais téléchargé — : chacune reste identique, comme le rapport TVA ;
> un PDF archivé est servi tel quel, et un PDF rendu de nouveau se dessine comme en 0.19.1 pour une désignation de 44 caractères au
> plus, en caractères courants. (refs #1242)

*(Si la 49-4b entre dans la release, l'intégration ajoute la dérogation : sur une facture jamais téléchargée ou regelée, un avoir
ou un rappel, une désignation plus longue s'imprime désormais en entier, et les caractères que la 49-4b replie s'impriment
autrement — la liste est celle de sa D3, comme à l'AC14 (V4-4).)*

## Dev Notes

- **Charge, au regard de la coupure du jeudi 2026-11-26** (estimation reprise après l'allègement du 2026-11-20) : script et test
  shell (D1 à D10, quatorze scénarios, mutations) **deux jours** ; script complémentaire (D11, un seul jeu) **un quart à un
  demi-jour** ; manuel et `docs/testing.md` **un demi-jour** ; gates, recette réelle (vert, R1, R2, trame hors script), E2E
  complet **un jour** (chaque recette réelle prend plusieurs minutes de station, sous les verrous de la voie base) ; plus la boucle
  de revue. Soit **trois jours et demi à quatre jours** à partir du lancement, contre cinq avant l'allègement, **boucle de revue
  non comprise** (au moins deux passes de modèles différents, gate complet et E2E complet au dernier commit) ; la remédiation
  de la P3 ajoute peu (contrôle du nom et des PDF au script complémentaire, sonde de T0, contrôle de diff de T12) : marge d'une
  journée environ si le développement part le 2026-11-21, absorbée en partie par la boucle de revue.
- **Si le temps manque — ordre de coupe** (V3-8) : **rien ne se coupe dans la story** — ni AC, ni gate, ni preuve du rouge, ni
  rejeu (AC12) ; les gates ne se rognent jamais pour tenir la date. **La 49-1a ne sort pas sans la 49-1d** : si la 49-1d n'est pas
  close le **jeudi 2026-11-26 au soir**, **tout l'Epic 49 sort de la 0.20.0**, avec la migration, et la release part sans
  migration (fichier d'épique, § *Ordre de sortie à la coupure* ; à dire en tête des notes). L'orchestrateur le constate à la
  coupure et le dit au compte rendu ; le Project Lead garde le dernier mot. Une prémisse fausse révélée par T0 arrête la story
  et se dit le jour même, pour que la décision se prenne avant jeudi. Le contrôle de la perte assumée
  (`rel0200/perte-assumee.sh`) se joue dès la coupure du jeudi soir ; la recette et le script complémentaire de l'AC12, après le
  bump, avant le vendredi midi, sur la voie « Base et Rust », que d'autres epics occupent aussi. **Un rejeu non vert le vendredi
  à midi vaut le même repli** : la migration et l'Epic 49 sortent de la 0.20.0 ; l'orchestrateur le constate, le Project Lead
  garde le dernier mot (V5-3).
- **Hors périmètre** : (a) la phase 4 du script revient par le dump ; le retour du manuel est joué par D11, #920 reste ouvert pour
  l'outillage du dépôt ; (b) rappels, séquences de numérotation : non peuplés ni comparés (inventaire) ; (c) la preuve de la
  dérogation de la 49-4b (sa fiche) ; (d) ce que l'**Allègement** a sorti (section dédiée). La story résout le premier point de #960
  (« La recette ne couvre pas les documents ») : à dire sur l'issue, qui reste ouverte pour ses autres points.
- **Issues** (ouvertes par l'orchestrateur le 2026-11-20, une par problème ; la n° 6, conditionnelle, ne l'est pas) :
  1. **#1444** — `technical-debt`, P3 — *la recette de montée ne peuple ni ne compare les rappels (`invoice_reminders`) ni les séquences de
     numérotation*.
  2. **#1445** — `technical-debt`, P3 — *recette de montée : rejouer l'usage réel après la montée* — valider un brouillon d'avant, émettre un
     avoir sur une facture d'avant, en encaisser une, sous la version d'avant et sous la neuve, et comparer ce qu'elles écrivent
     (ex-D10 de la 49-1d, sorti le 2026-11-20). Aujourd'hui, la parité sur des lignes toutes chiffrées repose sur la suite de
     tests existante, passée inchangée (binaires nommés à la perte assumée, contrôle de diff de T12, rejoué à l'AC12) ; les tests T5 à T8 de la
     49-1a ne prouvent que la neutralité des lignes non chiffrées. Points connus à reprendre : l'extrait doit porter
     `invoice_lines` (V2-1) et les colonnes d'instant en être exclues.
  3. **#1446** — `technical-debt`, P3 — *reprise d'une migration interrompue : jamais jouée avec le vrai binaire* — pour une migration écrite pour
     se rejouer (`20261118000002`), la reprise par `DELETE` de la ligne `success = 0` n'est prouvée que par
     `line_types_migration_resumes_after_a_half_run` ; le manuel ne la prescrit donc pas. Points connus : au redémarrage qui suit la
     reprise, Kesh écrit une **seconde** sauvegarde `pre-upgrade/`, du schéma à mi-chemin, que la version d'avant refuse (V2-3) ; le
     message de sqlx ne contient pas *dirty* (V2-4).
  4. **#1447** — `technical-debt`, P3 — *le refus, par la version d'avant, d'une sauvegarde de la version neuve n'est pas constaté à la release*
     (ex-D11 a de la 49-1d) : établi au code du tag ; le détail (table, colonne) est dans la réponse de l'API, non au journal (V2-2).
  5. **#1448** — `documentation`, P3 — *manuel d'administration, section « Passer à la 0.19.0 »* (`admin-manual.tex:2578`) : « migration *dirty*
     au journal » — le journal porte « migration … is partially applied; fix and remove row from `_sqlx_migrations` table », sans le
     mot *dirty* (V2-4).
  6. Conditionnelle, non ouverte — seulement si la 49-4b **sort** de la 0.20.0 : `technical-debt`, P4 — *la recette de montée ne rend aucune
     désignation de plus de 44 caractères sur une pièce non figée* (ex-option `--designation-longue`) ; la preuve repose sur le cas
     S7 de la 49-1b.
- **Écarts au fichier d'épique, déclarés** : (a) le tableau des stories écrit « manuel d'administration (×4) » ; le tableau des
  fichiers et la 49-1a (V3-1) disent **FR seul** : les manuels DE, EN, IT n'existent pas — FR seul ici ; (b) `--diff-attendu`
  nomme aussi `api` (la réponse gagne `lineType`), et n'admet qu'un ajout (D2) ; (c) fichiers touchés hors du tableau :
  `scripts/tests/recette-montee.test.sh` et `crates/kesh-db/tests/recette_montee_script.rs` (une ligne), plus deux scripts hors dépôt
  (`kesh-gate-logs/rel0200/verifs-0200.sh`, `recette-complement-rel0200.sh`) ; (d) le rapport TVA (D10) et le retour par
  `pre-upgrade/` (D11) dépassent la lettre du tableau (arbitrages de l'orchestrateur sur V1-4 et V1-5, réduits par l'Allègement).
  Modules : `scripts/`, `docs/` (testing, manuel), `kesh-db` (test seulement) — **trois**, sous le seuil de splitting.
- **Fichiers communs** : `admin-manual.tex` — la 49-1a a posé le squelette (fusionnée) ; la story de menu
  (`v0200-menu-sous-groupes`) fusionne en dernier et change des chemins du manuel ; les PDF ne se commitent pas.
  `scripts/recette-montee.sh` n'est touché par aucune autre story de la release. `sprint-status.yaml` : bloc de l'Epic 49.
- **Ce qui doit rester vrai** : le périmètre fermé du script (relance `env -i`, `HOME` neutralisé, démon local seul, écriture sous
  `$D` seulement, aucun chemin de la station, aucun montage, conteneurs du jeton seuls) ; les codes 2 à 5 et 11 à 15 ; la phase 4 à
  l'égalité stricte ; jamais `… | grep -q` sous `pipefail` (`:138-139`). **Jamais la base du NAS, jamais `~/kesh/`** : tout se joue
  dans les conteneurs de recette ; aucune donnée réelle de Guy dans les scripts, les journaux ou la fiche.
- **Station** : une commande lourde à la fois (construction de `kesh-api` et du frontend, recette réelle, script complémentaire,
  `gate-ci`, E2E), sous les verrous de sa voie (AC16 : `/tmp/station-heavy.lock` et `.gate.lock` pour la voie base,
  `.gate-front.lock` pour `make fr`) et `scripts/mem-guard.sh`, `CARGO_BUILD_JOBS=2`,
  `scripts/mem-guard.sh --status` avant chacune. La recette refuse si un conteneur `kesh.recette` d'un autre run existe (code 4) :
  attendre, ne jamais arrêter le conteneur d'un autre. Ports `3197`/`3398` (et `3399` pour l'import), préfixe `kesh-e491d` ; le
  script complémentaire prend ses propres ports, distincts.
- **Rejeu** : la recette est le gate de la **release** ; elle se rejoue au dernier commit de code de la 0.20.0 (AC12), après toutes
  les fusions (48-*, 49-*, menu, #1327 si fusionnée). Un rouge de la phase 6 à ce moment signale un changement de rendu d'une
  facture existante : il ne se tolère pas, il va à l'orchestrateur (D8). De même, un retrait d'une valeur attendue dans les dix
  binaires de la perte assumée (contrôle de T12 rejoué par `rel0200/perte-assumee.sh` dès la coupure du jeudi, AC12) va à
  l'orchestrateur, qui décide avec le Project Lead si la perte assumée tient encore (V4-1, V5-3).
- **Références** : `epic-49-types-de-lignes-facture.md` (§ *Contraintes de production*, point 4 ; § *49-1* ; `:130`, `:154`),
  `release-0.20.0.md` (§ *Gates de la release*), fiches 49-1a (§ *Transmis à la 49-1d*) et 49-1b (D4), `docs/testing.md`
  § *La recette de montée*, `kesh-gate-logs/rel0190/recette-complement-rel0190.sh`,
  `kesh-gate-logs/e491d/validation-49-1d-p1.md`, `validation-49-1d-p2.md`, fiches 49-4a et 49-4b (branche
  `story/49-4-designation-longue-pdf`, `55429142c`), `CLAUDE.md` § *Ce qu'impose la production*, § *Documentation à synchroniser*
  (point 7), #1242, #920, #925, #960, #1363, #1364, #1400.

## Dev Agent Record

### Reprise (pause décidée par Guy le 2026-11-20 ; reprise le mardi 2026-11-24 au matin)

**Où j'en suis.** Étape 0 faite et commitée (`53b4b2538` : LOW V5-1 à V5-4, issues #1444 à #1448, statut `in-progress`) ;
`kesh-gate-logs/rel0200/perte-assumee.sh` livré (hors dépôt), sondé (`e491d/dev/perte-assumee-sonde.log` : code 3 sur trois
mutations), joué à la tête (`e491d/dev/perte-assumee-etape0.log` : code 0). Code partiel commité par `e137ae1e9` (« pause »).
**T0 non jouée** : la sonde `kesh-gate-logs/e491d/t0/sonde-t0.sh` est écrite mais n'a jamais démarré (arrêtée dans la file du
verrou `.gate.lock` ; aucun conteneur créé). Ce qu'elle doit établir ne l'est donc **pas encore** : forme de la réponse de
création du compte bancaire, compte d'arrondi du preset, arrondi non nul de F4 (`100.00`/8.10 + `33.33`/2.60 + `10.01`/0.00),
statut `cancelled` de la facture après l'avoir, chemin `pdf_storage_path` (`<sha256>.pdf` à la racine ?), lisibilité des fichiers
après `docker cp`, `410` et regel sans ligne ` ERROR ` au journal, champ `totalBaseHt` du rapport TVA, numéro dans le texte
`mutool` d'une facture et d'un avoir, égalité stricte des tables après `full-import` dans une 0.19.0 neuve. Établi hors image :
`mutool 1.27.0` rend une trame PNM par page sur la sortie standard (`draw -q -r 300 -c rgb -F pnm -o - f.pdf <page>`),
`mutool info` donne `Pages: N`, le « warning: ICC » va sur la sortie d'erreur.

**Code commité en l'état, partiel et non éprouvé.** `scripts/recette-montee.sh` : préconditions, huit factures, avoir, groupe,
contrôles de la phase 1, `credit_notes`/`credit_note_lines` dans `TABLES`, projection par ajout (tables et `api`), rapport TVA,
`docker cp` et retrait de F8, PDF des phases 2, 4 et 5, phase 6 (code 16), `KESH_MUTOOL`. `scripts/tests/recette-montee.test.sh` :
factices refaits (`docker` : `cp` et nouvelles requêtes ; `curl` : pièces, PDF, `410`, regel, avoir, groupe, TVA ; `mutool`
factice ; `KESH_MUTOOL` à chaque lancement ; nominal à six phases, S6, S9, S12, S13). `bash -n` vert sur les deux ; **le test n'a
jamais tourné**. Manquent : S1 à S5, S7, S8, S10, S11, S14 comme scénarios propres ; `.env_remove("KESH_MUTOOL")` dans
`crates/kesh-db/tests/recette_montee_script.rs` ; T6 à T12.

**Prochaine étape exacte.** 1) Jouer T0 :
`cd /home/gcorbaz/devel/kesh-49-1d && flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh /home/gcorbaz/devel/kesh-gate-logs/e491d/t0/sonde-t0.sh > /home/gcorbaz/devel/kesh-gate-logs/e491d/t0/sonde-t0.log 2>&1`
(image `gcorbaz/kesh:0.19.0` déjà tirée, `sha256:2d82a944…`) ; écrire ici ses surprises avant tout autre code, corriger le script.
2) Lancer le test shell seul (`bash scripts/tests/recette-montee.test.sh`, factices seulement), le mettre au vert, ajouter les
scénarios manquants, puis les mutations (journaux sous `kesh-gate-logs/e491d/mut/`). 3) T5 à T12 dans l'ordre de la fiche.

**Bases à recréer** (`docs/testing.md` § *Base de dev jetable*) : `kesh_e491d`, `kesh_e2e_e491d`, `kesh_ci_e491d` — aucune n'a
encore été créée. Sonde et recette ont leurs propres conteneurs `mariadb:10.11` en tmpfs.

### Reprise du 2026-11-24 — ce qui a tourné (Opus 5.5)

Tout chiffre ci-dessous cite son journal (sous `/home/gcorbaz/devel/kesh-gate-logs/e491d/` sauf mention) et son commit.

**1. Reprise de la branche d'épique.** `git merge --no-ff origin/epic/49-types-de-lignes-facture` (`87e00ba05`) → `0d033d36e`.
Un conflit, `sprint-status.yaml` : bloc de l'épique gardé (49-1c `done`, renvois à l'Epic 52), ligne de la 49-1d gardée
`in-progress`. **La dernière image publiée est la 0.19.1** : la fiche et ses scripts partent d'elle (C-e491d-1 ; sites
opératoires corrigés, mentions historiques gardées). Faits relus au tag `v0.19.1` : rendu, routes, import, nom de la
sauvegarde, écran de restauration identiques à la 0.19.0 (`git diff --stat v0.19.0 v0.19.1` vide sur ces fichiers) ; `kesh-qrbill`
ne change que la visibilité de `largeur_mm`. **La 49-4b est passée à l'Epic 52** (`sprint-status.yaml` de l'épique, 2026-11-21) :
D8 (c) et l'AC14 jouent leur branche « la 49-4b n'entre pas » ; l'issue conditionnelle n° 6 des Dev Notes est donc à ouvrir.

**2. T0, la sonde** (`t0/sonde-t0.sh`, image `gcorbaz/kesh:0.19.1` `sha256:50b01c7cf627`, `mutool 1.27.0`). Premier tirage
(`t0/sonde-t0.log`) — **deux prémisses de D1 fausses, écrites ici avant le code qui en dépend** : (a) l'IBAN du relevé A,
`CH4431999123000889012`, est un **QR-IBAN** (IID 31999) ; en compte principal, tout PDF de facture est refusé
(`400 INVOICE_NOT_PDF_READY`, « QR-IBAN fourni sans référence QRR ») → C-e491d-2 (un second compte, principal, à l'IBAN
d'exemple ordinaire de SIX `CH9300762011623852957`) ; (b) `invoices.total_amount` est le **HT** ; un crédit égal à leur somme fait
refuser le groupe (`RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE`) → C-e491d-3 (somme des `totalTtc`). Aucune des deux ne touche
la preuve : elles changent le peuplement, non ce qui est comparé ; la story continue. Second tirage (`t0/sonde-t0-2.log`), tout
établi : compte bancaire rendu sous `{ "id": … }` à plat ; aucun compte au nom d'arrondi dans le preset (le premier compte de
charge est retenu) ; F4 (`100.00`/8,10 + `33.33`/2,60 + `10.01`/0,00) → `rounding_amount = -0.0100` ; les taux 2,60 et 0,00
existent et passent la validation ; l'avoir annule la facture (`cancelled`) ; PDF figé à `<sha256>.pdf` à la racine de
`/tmp/documents` (le dossier n'existe qu'après le premier), `docker cp` lisible par l'utilisateur ; deux rendus d'un avoir
diffèrent à l'octet et ont la même trame ; `410 INVOICE_PDF_GONE`, puis regel `200` (réponse JSON de facture), au journal un
**`WARN`** (« PDF de facture refigé »), aucune ligne ` ERROR ` ; `totalBaseHt` au rapport TVA ; le numéro de la pièce dans le
texte `mutool` d'une facture (2 fois) et d'un avoir (1 fois) ; après `full-import` dans une 0.19.1 neuve, les 17 tables
strictement égales, et le PDF figé servi à l'octet. Établi aussi : une facture de 40 lignes est refusée au PDF (`400`, capacité) :
aucune pièce de la recette n'a deux pages, la trame page à page n'est éprouvée que par le factice.

**3. Le code.** Commits `a5cf95297` (pièces lues dès leur création — `id_piece` lisait un fichier écrit après les
téléchargements de la phase 1 : défaut trouvé au premier tirage du test —, deux comptes bancaires, crédit au TTC ; factices
alignés), `a02b79bec` (scénarios S1 à S14 propres, `.env_remove("KESH_MUTOOL")`), `bd7e72ce8` (une trame vide des deux côtés
ou un fichier d'avant vide ne comptent plus : scénario **S6c** ajouté, défaut relevé en rédigeant le prompt de revue).
`shellcheck` n'est pas installé à la station : non joué.

**4. Le test shell** (`scripts/tests/recette-montee.test.sh`, factices seuls) : vert à `bd7e72ce8`, **155 assertions**,
`dev/test-shell-7.log` ; seul, 2 min 56 s (`dev/test-shell-6.log`, à `a02b79bec`). Durée sous nextest : § 7.

**5. Les mutations** (`mut/mutations.py`, copies de `scripts/` hors worktree, C-e491d-5) au commit `bd7e72ce8` : **21
mutations, 20 tuées, 1 équivalente** (`mut/juge.log`, rejugé par `mutations.py --juger`). Équivalente, déclarée d'avance :
S3c sans le contrôle de type — `jq` échoue sur l'objet devenu scalaire (`has`, indice), la projection échoue quand même
(`rouge-S3c-type-equivalente.log` : « S3c … (code 12) » sous mutation). Deux mutations retirent deux contrôles qui se couvrent
(S5 : octets et SHA-256 ; S10-principal : principal et lien au `1020`). **Deux défauts de l'outillage de mutation, trouvés et
corrigés en cours de route** : (a) le premier tirage jugeait « tuée » toute mutation, l'en-tête du journal citant l'assertion
attendue (relevé sur S3c, test vert) — le verdict exclut désormais l'en-tête et exige le résumé « assertion(s) en échec » ;
(b) deux tests en parallèle choisissent les mêmes ports et le scénario du port de la seconde base s'interrompt sous `set -e`
avant le résumé (onze journaux tronqués, `mut/juge-tirage-parallele.log`) — rejouées une à une (`mut/resume-2.log`), toutes
les onze ont rendu leur verdict, dix tuées et S3c équivalente. Le défaut (b) est **préexistant au test** (Story 33-10, ligne
`D=$(… grep -oE 'dossier .*$' … )` sous `pipefail`) : issue à ouvrir (une par problème), test défaillant, P3.

**6. La recette réelle (AC10)**, à `bd7e72ce8`, binaire construit à `0d033d36e` (code de production inchangé depuis :
`git diff --stat 0d033d36e..bd7e72ce8 -- crates` ne nomme que le test), frontend construit à la même tête
(`dev/build-kesh-api.log`, `dev/build-frontend.log`) :
```
flock /tmp/station-heavy.lock flock …/.gate.lock scripts/mem-guard.sh scripts/recette-montee.sh \
  --image gcorbaz/kesh:0.19.1 --binaire target/debug/kesh-api --static frontend/build \
  --port-api 3197 --port-base 3398 --migrations-attendues 1 \
  --diff-attendu migrations,invoice_lines,credit_note_lines,api \
  --apres ../kesh-gate-logs/rel0200/verifs-0200.sh --import --prefixe kesh-e491d
```
→ **code 0** (`recette-reelle.log`, dossier `/tmp/kesh-recette.fq4BbW`) : 95 migrations avant, une appliquée ; phase 3
« refusée : migration inconnue d'elle (sqlx) » ; **5 paires de rendus, chacune « 1 page(s), même dessin »** (F2, F4, F6, A1, F8).
Écarts relevés, tous attendus et **par ajout seulement** : `migrations`, `invoice_lines`, `credit_note_lines`, `api`, à la
montée comme à l'import ; les six diffs de projection sont vides, et non à vide — 11 lignes de facture, 2 d'avoir, 14 documents
`api` (8 factures, 4 factures fournisseur, 1 avoir, le rapport TVA), dont 13 lignes gagnent `lineType` ; `totalBaseHt` du
rapport TVA = 1813.34 aux empreintes `avant`, `apres` et `import`. Crochet (`journal/apres.log`) : « invoice_lines — 11
ligne(s), toutes priced, défaut 'priced' », « credit_note_lines — 2 ligne(s) … », « vert ». Sauvegarde
`kesh-pre-upgrade-20261124T004729872Z-0.19.1-to-0.19.1.keshbackup`, `/health` du binaire `0.19.1` (les crates sont en 0.19.1 :
`-0.19.1-to-0.20.0` n'est attendu qu'après le bump, AC12). **Sensibilité de la trame, hors script** (`trame-hors-script.log`) :
F2 contre F2 et la référence 0.19.1 de F2 contre son rendu 0.20.0 → MD5 égaux (`851d4f02…`) alors que leurs octets diffèrent ;
F2 contre F4 → différents (`ff35b8ba…`). Rouges réels R1 et R2 : § 8.

**7. Le script complémentaire (AC11)** (`rel0200/recette-complement-rel0200.sh`, joué sur `/tmp/kesh-recette.fq4BbW`) →
**code 0, 0 écart** (`recette-complement.log`) : exactement une sauvegarde `-0.19.1-to-*`, version d'arrivée 0.19.1 = `/health` ;
import `200` (`filesRestored 3`, `filesMissing 1` : F8) ; **17 tables strictement égales** à l'empreinte d'avant ; après un nouveau
login, F1, F5, F7 servis `200` à l'octet de la phase 1, F8 `410 INVOICE_PDF_GONE` ; journal de l'image sans ` ERROR `.

**Aucune facture ni aucun avoir existant ne change à la montée, et le retour par la sauvegarde fonctionne** : aucun point
d'arrêt du Project Lead (`alerte.md` le dit).

**8. Rouges réels** (`dev/chaine-reelle.sh`, une commande par prise ; journal `dev/chaine-reelle.log`) :
- **R1** (`--diff-attendu migrations` seul) → **code 12**, « écart hors de --diff-attendu (avant → apres) : api
  credit_note_lines invoice_lines » (`recette-reelle-rouge-diff.log`) : l'empreinte voit la migration.
- **R2** (copie mutée `scripts/recette-montee.mut.sh`, non suivie, `UPDATE invoice_lines SET unit_price = unit_price + 0.0001
  WHERE id = 1` juste avant `empreinte apres`) → **code 12**, « invoice_lines (valeurs d'avant changées …) » et `api` ; diff de
  la projection : `200.0000` → `200.0001` (`recette-reelle-rouge-projection.log`). Copie retirée ; `git status --porcelain`
  vide ensuite (fin de `dev/chaine-reelle.log`).
- **Rouge du script complémentaire**, sur une copie du dossier de la recette (`e491d/rouge-D/`) dont l'empreinte d'avant de
  `invoices` gagne une ligne et le PDF de F1 de la phase 1 un octet changé → **code 1, 2 écarts** : « invoices diffère », « F1 :
  HTTP 200, octets différents » (`recette-complement-rouge.log`).

**9. Gates de passage en `review`** (`gates-review.sh`, une commande par prise de la voie base ; `gates-review.log`) :
- bases `kesh_e491d`, `kesh_ci_e491d`, `kesh_e2e_e491d` remises à neuf (`bases-neuves-e491d.sh`, 04 h 56) ;
- **`scripts/test-fast.sh --ci`** à `a44055e74` (code identique à `bd7e72ce8`) → **EXIT=0**, « 6264 tests run: 6264 passed (1 slow),
  17 skipped », 566 s (`test-fast-ci.log`) ; le test de la recette y prend 268,6 s ;
- **mesure du test de la recette sous nextest**, seul binaire, deux tirages : **294,0 s** et **177,0 s**
  (`mesure-nextest-1.log`, `-2.log`) ; la borne de l'enveloppe (600 s) n'a pas à bouger ; `docs/testing.md` le dit ;
- **`scripts/gate-ci.sh kesh_ci_e491d`** à `a44055e74` (arbre : `docs/testing.md` modifié, documentation seule) → **code 0**,
  « 6283 passés, 0 échoués, 21 ignorés » (`gate-ci.log`, `gate-ci.log.entete`) ;
- frontend : non touché (`git diff --stat origin/epic/49-types-de-lignes-facture...HEAD -- frontend` vide), aucun gate ;
- **pas d'E2E** (il tourne à la clôture, après la revue) ; recette visuelle sans objet (aucun contrôle visible) ;
- manuel : `make fr` vert, aucun `Overfull` dans les lignes de la section 0.20.0 (`dev/make-fr.log`, `admin-manual.log`) ;
  texte aplati contrôlé : le message « partially applied » cité, « établi par la lecture du code de la 0.19.1, non constaté »,
  `-0.19.1-to-0.20.0.keshbackup` ; le seul *dirty* et le seul `DELETE FROM _sqlx_migrations` du PDF sont ceux des sections
  0.19.0 (#1448) et 0.17.x, non de la section 0.20.0. PDF du dépôt remis à leur état (`git checkout`) ; copie compilée pour la
  revue : `e491d/manuel/admin-manual.pdf` ;
- contrôle de la perte assumée (`rel0200/perte-assumee.sh`, base par défaut portée au tag `v0.19.1`, C-e491d-1) : **code 0**
  depuis `v0.19.1` et depuis `773df80da` (`dev/perte-assumee-v0.19.1.log`, `dev/perte-assumee-773df80da.log`) : 16 fichiers
  (les dix binaires, et six modules dont `support/caracterisation.rs`, venu avec la reprise de `main`, relevé seul),
  sonde positive 14 retraits, aucun retrait à trier, aucun ajout à classer ;
- AC15 : `git diff --stat origin/epic/49-types-de-lignes-facture...HEAD -- crates frontend` ne nomme que
  `crates/kesh-db/tests/recette_montee_script.rs` (4+, 2−) ; `ls crates/kesh-db/migrations/*.sql | wc -l` → 96 ;
- mots-clés de fermeture contre `origin/epic/49-types-de-lignes-facture` : aucun.

**10. Écarts aux consignes, déclarés.** (a) La résolution du conflit de `sprint-status.yaml`, la réécriture de la section de
`docs/testing.md` et la copie du crochet dans cette fiche ont été faites par un court script Python, non par l'outil Edit
(fichiers de documentation, aucun code) ; tout le code (`scripts/`, `crates/`) l'a été par Edit. (b) La recette écrit, par
construction, sous `/tmp/kesh-recette.*` (racine jetable imposée sous `/tmp`, D3 et Dev Notes) ; tout le reste sous le worktree
et `kesh-gate-logs/`. (c) Un `kill` par PID tiré de `pgrep -f` a touché, avec le jeu de mutations du premier tirage, le shell de
la commande qui le lançait (mes processus seulement) ; les deux tests orphelins ont été arrêtés par leur PID. (d) Le dossier
`e491d/mut/` garde deux racines de mutation du premier tirage interrompu (`racine-S5-octets`, `racine-S6a-trame`) : leur
suppression m'a été refusée ; elles sont inertes.

**11. À ouvrir** (une issue par problème, par l'orchestrateur) : (i) la conditionnelle n° 6 des Dev Notes, puisque la 49-4b est
à l'Epic 52 — `technical-debt`, P4, *la recette de montée ne rend aucune désignation de plus de 44 caractères sur une pièce non
figée* ; (ii) `technical-debt`/test défaillant, P3 — *`recette-montee.test.sh` : deux copies simultanées choisissent les mêmes
ports, et le scénario du port de la seconde base interrompt le test sous `set -e` avant son résumé* (ligne `D=$(/usr/bin/grep
-oE 'dossier .*$' …)`, Story 33-10) ; (iii) `technical-debt`, P4 — *`shellcheck` absent de la station : les scripts de recette ne
sont pas passés au linter*.

**Crochet `kesh-gate-logs/rel0200/verifs-0200.sh`, en entier (D7)** :
```sh
# verifs-0200.sh — crochet `--apres` de la recette de montée de la 0.20.0 (Story 49-1d, D7, #1242).
#
# Sourcé par `scripts/recette-montee.sh` en phase 2, après l'empreinte `apres`, dans un sous-shell
# sous `set -e`, depuis `$D/cwd`. Il dispose de `SQL` (base migrée), `api` et `L` (journal du run).
# Ce qu'il contrôle ne vaut que pour la migration 20261118000002 (types de lignes) : la projection de
# `--diff-attendu` (D2) prouve que les valeurs d'avant n'ont pas bougé ; ce crochet prouve la VALEUR
# de la colonne neuve et le nombre de lignes, sur les deux tables.
#
# Forme imposée (D7) : chaque contrôle s'écrit `[ … ] || exit 1`, jamais dans un `if`, un `&&` ni un
# `local x=$(…)` — les exceptions de `set -e` que l'en-tête du script documente. Valeurs relevées par
# des affectations simples, qui font sortir le crochet si la requête échoue.

for t in invoice_lines credit_note_lines; do
  # Aucune ligne existante n'a reçu un autre type que la ligne chiffrée.
  n_autres=$(SQL "SELECT COUNT(*) FROM $t WHERE line_type <> 'priced'")
  [ "$n_autres" = 0 ] || exit 1
  # Le nombre de lignes égale celui d'avant, lu au fichier de l'empreinte `avant` (une ligne par
  # enregistrement : `mariadb -N` en mode batch échappe les sauts de ligne).
  n_avant=$(wc -l < "$L/empreinte-avant/$t")
  n_apres=$(SQL "SELECT COUNT(*) FROM $t")
  [ "$n_apres" = "$n_avant" ] || exit 1
  # Une table vide ne prouverait rien : la recette peuple des lignes de facture et d'avoir.
  [ "$n_avant" -gt 0 ] || exit 1
  # La valeur par défaut, apostrophes comprises (forme de MariaDB 10.11).
  defaut=$(SQL "SELECT COLUMN_DEFAULT FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = '$t' AND COLUMN_NAME = 'line_type'")
  [ "$defaut" = "'priced'" ] || exit 1
  echo "verifs-0200 : $t — $n_apres ligne(s), toutes priced, défaut $defaut"
done
echo "verifs-0200 : vert"
```

## Change Log

| Date | Entrée |
|---|---|
| 2026-11-19 | Création de la fiche (workflow `bmad-create-story`), par l'agent de rédaction de la 49-1d (Opus 5.5) ; code lu à `f57f6ac80`. Neuf décisions (D1 à D9), quatorze AC, treize tests. Points arrêtés : peuplement par l'API de la 0.19.0 (huit factures, un avoir, un règlement groupé, quatre PDF figés dont un au fichier retiré) ; `--diff-attendu` réduit aux ajouts (projection sur les colonnes et les clés d'avant) ; documents copiés par `docker cp` sans montage ; PDF archivés à l'octet, rendus par trame et texte (`mutool`), référence 0.19.0 prise en phase 4, phase 6 « Rendus » (code 16) ; crochet de la release hors dépôt. Écarts au fichier d'épique déclarés (manuel FR seul, `api` nommé, test du script et enveloppe touchés). Une issue à ouvrir. Validation non lancée. Statut : `ready-for-dev`. |
| 2026-11-19 | **Passe de validation P1** (Sonnet, contexte frais, `kesh-gate-logs/e491d/validation-49-1d-p1.md` : 0 CRITICAL, 0 HIGH, 6 MEDIUM, 4 LOW). **Remédiation** (Opus 5.5, sur les arbitrages de l'orchestrateur ; branche à jour d'`epic/49` par `d4f5bba7f`) : **V1-1** préconditions écrites à D1 et AC1 — compte bancaire principal (`invoice_pdf_service.rs:245`, `bank_accounts.rs:112`) et compte de différences d'arrondi (`invoices.rs:2324`) —, contrôle `HTTP 200` + en-tête `%PDF-` de chaque PDF (code 11) ; **V1-2** résolution fixée à 300 dpi, nombre de pages et code de `mutool` contrôlés, décompte **exact** de paires (5, ou 9 avec `--import`) dont le texte contient le numéro de la pièce, ce que « même dessin » prouve et ne prouve pas (D4), rouges réels de sensibilité R2 (projection), R3 (rendu, et F2 contre F4 sur les vrais PDF), R4 (gestes) ; **V1-3** dérogation de la 49-4 (`d4f5bba7f`, fichier d'épique `:130`, `:154`) dite à D8, AC14 et à la Note de version, phrase « comme en 0.19.0 » réduite aux désignations de 44 caractères au plus et conditionnée à la liste des stories fusionnées ; **V1-4** D11 et AC11 : script d'intégration `kesh-gate-logs/rel0200/recette-complement-rel0200.sh`, écrit et joué par la story, rejoué à la release — refus d'import d'une sauvegarde 0.20.0 par la 0.19.0, retour par `pre-upgrade/` dans une 0.19.0 neuve, reprise d'une migration interrompue avec le vrai binaire —, le manuel ne dit « constaté » que ce qui a été joué, la reprise précédée d'une sauvegarde et dite propre à cette migration ; **V1-5** second inventaire par le symptôme (lecteurs filtrés par `line_type`, commande et sortie), D10 et AC8 : rapport TVA et créances âgées dans `api`, validation de F3, avoir A2 sur F4 et règlement de F1 joués en phase 2 et en phase 4, extraits comparés en phase 6 ; **V1-6** `KESH_MUTOOL` posé à chaque scénario, journal du factice exigé, T11 et T13 réécrits pour que le rouge ne dépende pas du `mutool` de l'hôte. **LOW appliqués** : V1-7 F7 et A1 sous 44 caractères, option `--designation-longue` si la 49-4 n'entre pas ; V1-8 contrôles du crochet en `[ … ] \|\| exit 1`, compte d'avant lu au fichier d'empreinte, `'priced'` avec apostrophes ; V1-9 noms de colonnes entre apostrophes inverses, même type JSON exigé, A1 rendu après l'import, durée du test mesurée avant la revue ; V1-10 paragraphe « Compatibilité » : l'import compare aussi les colonnes, message exact. Onze décisions (D1 à D11), seize AC, seize tests, douze tâches *(annotation du 2026-11-20 : treize tâches, T0 à T12 — V2-9 ; décompte faux à sa date)*. Estimation de charge écrite aux Dev Notes (cinq jours environ). Modèles : Opus (rédaction et remédiation), Sonnet (P1) ; la P2, complète, prend Opus en contexte frais. Statut : `ready-for-dev`. |
| 2026-11-20 | **Passe de validation P2** (Opus, contexte frais, `kesh-gate-logs/e491d/validation-49-1d-p2.md` : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 9 LOW ; les dix findings de la P1 résolus sur le fond). Les trois MEDIUM naissent de D10 et D11, ajoutés par la remédiation de la P1 : **signal de recyclage** de la règle de splitting préventif, déclaré au Project Lead. **Remède : l'allègement que la P2 proposait, accepté par Guy le 2026-11-20** (section *Allègement*), au lieu d'un découpage. **Sortent** : les gestes rejoués après la montée (validation de F3, avoir A2, règlement de F1, extraits comparés ; ex-AC8 « gestes », ex-T15, ex-R4) ; la reprise d'une migration interrompue avec le vrai binaire (ex-D11 c, ex-AC11 c) et son paragraphe prescriptif au manuel, remplacé par la consigne de la 0.19.0 ; la constatation du refus d'une sauvegarde 0.20.0 par la 0.19.0 (ex-D11 a, ex-AC11 a, export du crochet), dite « établie par le code » ; les contrôles annexes : créances âgées, `--designation-longue` et ex-T16, R3 et R4 dans le script, rendus après l'import (décompte 9 → 5). **Restent** : D1 à D9, D10 réduit au rapport TVA, D11 réduit au retour par `pre-upgrade/` ; AC1 à AC16 (AC8 devient « rapport TVA », AC11 « le retour du manuel »). **Perte assumée** écrite : la preuve des gestes repose sur les tests de la 49-1a (T5, T6, T7, T8). **MEDIUM** : V2-1 **tombé** (extrait des gestes sorti ; repris dans l'issue à ouvrir n° 2) ; V2-2 **tombé** pour le jeu (a), son fait corrigé là où il reste (« Les fichiers de la 49-1a » (a), AC14 : refus non écrit au journal, détail dans la réponse) ; V2-3 **tombé** (reprise sortie ; issue n° 3 ; contrôle « exactement une sauvegarde `pre-upgrade/` » ajouté à D11, l'installation neuve de la phase 5 n'en écrivant pas, `pre_upgrade_backup.rs:161`). **LOW** : V2-4 corrigé (message exact de sqlx cité, *dirty* retiré ; l'imprécision de la section 0.19.0 → issue n° 5) ; V2-5 corrigé (`POST /api/v1/admin/full-import`, administrateur par `KESH_ADMIN_USERNAME`) ; V2-6 corrigé par retrait des créances âgées ; V2-7 corrigé (AC4 : `GET` → `410`, `POST …/refreeze` → `200` JSON, `GET` → `200` `%PDF-`) ; V2-8 **tombé** (R3 sorti du script, trame éprouvée hors script et par S6) ; V2-9 corrigé (Dev Notes réécrites, annotation datée du décompte des tâches, scénarios renommés S1 à S14) ; V2-10 corrigé (deux verrous de la voie base à l'AC10 et à l'AC16, verrou de la voie front pour `make fr`) ; V2-11 corrigé (symptôme énoncé sur le filtre, sites hors filtre ajoutés : rendu, réponse d'avoir, CSV, MCP) ; V2-12 corrigé (D2 et AC12 : un écart légitime causé par une autre story revient à l'orchestrateur). Relevés de la P2 sur les factices intégrés (lancements hors `lancer()`, assertion « liste exacte »). **Cohérence avec le découpage de la 49-4** (2026-11-20) : la dérogation est celle de la 49-4b, étendue par Guy à l'avoir (Q3) et au rappel (Q1) ; la recette tenait déjà l'avoir pour couvert (A1 sous 44 caractères) et ne rend aucun rappel ; descriptions de F2 à F8 bornées à ce que la 49-4b ne change pas (ASCII, espaces simples, aucune de bord) ; AC14 et Note de version nomment l'avoir et le rappel. Six issues à ouvrir (Dev Notes, dont une conditionnelle). Charge réestimée : trois jours et demi à quatre jours plus la boucle de revue. Quatorze scénarios (S1 à S14), treize tâches. Modèles : Opus (P2), Opus 5.5 (remédiation) ; la P3, complète, prend Sonnet en contexte frais. Statut inchangé : `ready-for-dev`, validation en cours. |
| 2026-11-20 | **Passe de validation P3** (Sonnet, contexte frais, `kesh-gate-logs/e491d/validation-49-1d-p3.md` : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 5 LOW) — **validation P3 remédiée** (Opus 5.5, prémisse de chaque finding relue au code). Tendance des MEDIUM : P1 → 6, P2 → 3, P3 → 4. Trois des quatre MEDIUM naissent de la remédiation précédente (V3-1 et V3-3 de la P2, V3-2 en partie de la P1) : ce sont des **erreurs de fait** (un suffixe non confronté à la version Cargo, un texte d'écran pris pour celui de l'API, des tests cités pour une preuve qu'ils ne portent pas), non le retour d'une structure sous une autre forme — pas de recyclage au sens de la règle de splitting ; V3-4 est une lacune antérieure de D11. **MEDIUM** : **V3-1** corrigé — prémisse lue, `crates/kesh-api/src/main.rs:179` `let vers = env!("CARGO_PKG_VERSION");`, crates en 0.19.0 à la tête de la story ; D11, AC11 et T9 exigent exactement un `kesh-pre-upgrade-*-0.19.0-to-*.keshbackup` dont la version d'arrivée égale `/health` du binaire neuf (écrit au journal par la phase 2, T3) ; AC12 : rejeu **après** le bump de `scripts/prepare-release.sh`, binaire reconstruit ; AC14 : le nom `-0.19.0-to-0.20.0` n'est « constaté » que si le rejeu suit le bump. **V3-2** corrigé — prémisse lue, `AdminRestorePanel.svelte:189-193` (tête et tag), clé `admin-restore-error-schema` ; AC14 et « Compatibilité » disent le code `IMPORT\_SCHEMA\_MISMATCH` (précédent de la section 0.17.5, `admin-manual.tex:2674`) et, s'ils citent l'écran, son texte réel, jamais le message de l'API ; § *Les fichiers de la 49-1a* (a) corrigé (la table se voit à l'écran). **V3-3** corrigé — prémisse lue, `credit_note_copies_unpriced_lines` compare deux avoirs dans le même code, sans facture d'avant ; la perte assumée, D10 (c), les trois lignes « angle mort » du second inventaire, AC9, AC13 et l'issue n° 2 disent désormais que la parité des gestes sur des lignes toutes chiffrées repose sur la **suite existante passée inchangée** (binaires nommés), les tests T5 à T8 de la 49-1a ne prouvant que la neutralité des lignes non chiffrées ; contrôle de diff ajouté à T12 (`git diff -U0 773df80da..HEAD -- 'crates/*/tests/*'`, retraits cités et justifiés, sonde positive du pathspec) ; à signaler à Guy par l'orchestrateur : la prémisse de sa décision était plus étroite qu'écrite. **V3-4** corrigé — prémisse lue, `admin_backup/export.rs:363-368` (`warn!` sur une pièce manquante) ; le script complémentaire sert, après l'import, les PDF de F1, F5, F7 et les compare à l'octet à la phase 1, F8 → `410` ; son rouge gagne un PDF altéré d'un octet. **LOW**, tous corrigés : V3-5 (AC14 : vidage de sécurité, perte des saisies, numéros sur papier à refaire) ; V3-6 (formule « valeur par défaut », total recompté au rejeu) ; V3-7 (T0 sonde `mutool draw -F txt` sur une facture et un avoir, `pdf.rs:329`) ; V3-8 (Dev Notes : boucle de revue hors estimation, **ordre de coupe** — rien ne se coupe dans la story, si la 49-1d n'est pas close jeudi 2026-11-26 au soir tout l'Epic 49 sort de la release, jamais de rognage des gates —, rejeu AC12 avant vendredi midi) ; V3-9 (D5 : rendus de la phase 4 avant `journal_sans_erreur 4` et `arreter_image`, conteneur neuf ; S6 contrôle l'ordre). Grep du symptôme fait (`0.19.0-to-0.20.0`, `IMPORT_SCHEMA_MISMATCH`, « T5 », « perte assumée ») : aucun reste contradictoire dans la fiche ; la fiche close de la 49-1a (AC13 e) écrit le nom de la release, juste pour l'image publiée, laissée telle quelle. Décomptes inchangés : D1 à D11, AC1 à AC16, S1 à S14, T0 à T12. Modèles : Opus (P2), Sonnet (P3), Opus 5.5 (remédiation) ; la P4, complète, prend Opus en contexte frais. Statut inchangé : `ready-for-dev`, validation en cours. |
| 2026-11-20 | **Passe de validation P4** (Opus, contexte frais, `kesh-gate-logs/e491d/validation-49-1d-p4.md` : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 4 LOW) — **validation P4 remédiée** (Opus 5.5, prémisse de chaque finding relue au code). Tendance des MEDIUM : P1 → 6, P2 → 3, P3 → 4, P4 → 1. Le MEDIUM restant naît de la remédiation de la P3 (V3-3) : portée d'un contrôle juste (tête de la story contre release), non le retour d'une structure — pas de recyclage au sens de la règle de splitting. **MEDIUM** : **V4-1** corrigé — prémisse lue : `git diff --numstat 773df80da..origin/epic/48-corrections-0-20-0 -- 'crates/*/tests/*'` rend déjà des retraits (`invoice_send_email_e2e.rs` 37/3, `opening_balances_e2e.rs` 24/1), `origin/main` aussi (`account_currency_e2e.rs` 45/1) ; la 48-14 (#1255) refond le dé-rapprochement que couvrent `grouped_settlements` et `reconciliation_group_e2e`, la 48-11 l'arrondi que couvre `vat_report_e2e`. Le contrôle de diff de T12 se rejoue désormais au rejeu de l'AC12, sur la release assemblée, sur les dix binaires de la perte assumée **et les modules qu'ils incluent** (`#[path]`, `mod common`, relevés au rejeu ; ajout de cette remédiation, au-delà du remède proposé) ; journal `kesh-gate-logs/rel0200/perte-assumee.log` ; un retrait d'une valeur attendue venu d'une autre story revient à l'orchestrateur, sur le modèle de D2, qui décide avec le Project Lead si la perte assumée tient encore. Règle écrite à T12 pour distinguer un retrait de valeur attendue (assertion, littéral comparé, donnée de fixture, test ou attribut ; le doute tranche en « valeur attendue » ; un ajout qui neutralise — `#[ignore]`, `#[cfg]`, `return` — compte comme tel) d'un retrait sans effet (commentaire, `use`/`mod`, décompte de migrations, reformatage repris à l'identique). Propagé à *Allègement* (perte assumée), D10 (c), le second inventaire, AC9, AC12, AC13, T12 et les Dev Notes (issue n° 2, *Rejeu*). Joué à `6bb2d5770` : sur les dix fichiers et leurs modules, un seul retrait (commentaire de `grouped_settlements.rs`), aucun ajout neutralisant. **LOW**, tous corrigés : **V4-2** (T12 : sonde sur le motif lui-même, `-- 'crates/*/tests/*' \| grep migrations_upgrade_path` → `774 14`, contre zéro ligne sans `/*` ; seuls les retraits cités, 20 lignes à `6bb2d5770` au lieu de 5 328) ; **V4-3** (D11, AC11, T9 : nouveau login après l'import avec `admin` et le mot de passe factice de `recette-montee.sh:219-220` — non `:217-218` comme l'écrivait le rapport —, F8 compte seulement sur `410` **et** `INVOICE_PDF_GONE`, `errors.rs:2835`) ; **V4-4** (Note de version : « en caractères courants », et le complément conditionnel nomme les caractères repliés par la 49-4b, comme l'AC14) ; **V4-5** (AC15 : diff à trois points depuis la base de fusion). Grep du symptôme fait (« passée inchangée », `..HEAD -- crates frontend`, `F8 → 410`, « 44 caractères au plus ») : tous les sites traités. Décomptes inchangés : D1 à D11, AC1 à AC16, S1 à S14, T0 à T12. Modèles : Sonnet (P3), Opus (P4), Opus 5.5 (remédiation) ; la passe suivante, complète, prend Sonnet en contexte frais (le MEDIUM corrigé appelle une passe de plus). Statut inchangé : `ready-for-dev`, validation en cours. |
| 2026-11-20 | **Validation terminée** (entrée finale de la boucle). **Passe P5** (Sonnet selon l'alternance prévue par la P4, contexte frais, `kesh-gate-logs/e491d/validation-49-1d-p5.md` : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 4 LOW) : arrêt de la boucle, aucune passe ne rendant plus de finding au-dessus de LOW. **Tendance des findings par passe** (CRITICAL/HIGH/MEDIUM/LOW) : P1 0/0/6/4, P2 0/0/3/9, P3 0/0/4/5, P4 0/0/1/4, P5 0/0/0/4. **Modèles** : Sonnet (P1), Opus (P2), Sonnet (P3), Opus (P4), Sonnet (P5) — alternance tenue, aucune passe ciblée ; rédaction et remédiations par Opus 5.5. **Reclassements** : aucun finding MEDIUM+ reclassé en dette ; le signal de recyclage de la P2 a été traité par l'**Allègement** décidé par Guy (non par un découpage) ; les MEDIUM de la P3 et de la P4 jugés non recyclants (erreurs de fait, portée), justification à leurs entrées. **LOW de la P5, traités à l'étape 0 du développement** (Opus 5.5, prémisse relue) : **V5-1** — prémisse lue, T12 (d) « parmi les ajouts du même fichier » : (d) restreinte à **la même hunk** de `git diff -U0` ; (c) réécrite sans littéral qui vieillit (« le décompte des migrations lui-même, porté de la valeur de la release précédente à celle de la release qui ajoute la migration »). **V5-2** — prémisse lue, T12 ne cherchait que `^\+\s*(#\[ignore|#\[cfg|return\b)` ; ajouts classés en (e) neutralisants (motif `return` resserré à `return;` / `return Ok(());` seul sur sa ligne) et (f) types de ligne non chiffrés (`LineType::Comment`/`Heading`, `'comment'`, `'heading'`, `line_type`/`lineType` sans le type chiffré), chacun rattaché à sa fonction englobante : dans une fonction présente au tag, compté comme retrait de valeur attendue. **V5-3** — prémisse lue, AC12 « Il se planifie avant le vendredi midi » sans repli : le contrôle de la perte assumée se joue **dès la coupure du jeudi 2026-11-26 au soir** (il ne dépend pas du bump) ; un rejeu non vert le vendredi à midi vaut le repli de l'ordre de coupe (la migration et l'Epic 49 sortent de la 0.20.0 ; l'orchestrateur le constate, le Project Lead garde le dernier mot) — AC12, *Allègement*, Dev Notes (*Si le temps manque*, *Rejeu*). **V5-4** — prémisse lue, T12 ne donnait que la commande sur tout `crates/*/tests/*` : livré `kesh-gate-logs/rel0200/perte-assumee.sh` (hors dépôt, comme le crochet), restreint aux dix fichiers et à leurs modules relevés à la tête, sonde positive incorporée, codes 0/3/4 ; nommé par T12 et l'AC12. **Sondes du script** (`kesh-gate-logs/e491d/dev/perte-assumee-sonde.log`, clone jetable muté : une assertion retirée de `grouped_settlements.rs`, un `#[ignore]` et un `LineType::Comment` ajoutés dans une fonction du tag) → code 3, les trois lignes signalées ; la première version du script manquait la troisième (`git show … \| grep -q` sous `pipefail` : SIGPIPE, une fonction ancienne passait pour neuve) — corrigé par un décompte `grep -c`, la sonde rejouée. **Joué à la tête** (`153f6ba5c`, `kesh-gate-logs/e491d/dev/perte-assumee-etape0.log`) : 15 chemins (10 binaires, 5 modules), un retrait (commentaire de `grouped_settlements.rs`, (a)), 30 ajouts de type non chiffré tous dans des fonctions neuves de la 49-1a, code 0. **Issues des Dev Notes** ouvertes par l'orchestrateur : #1444 à #1448 (n° 6, conditionnelle, non ouverte), substituées à « à ouvrir » dans l'inventaire, la perte assumée et les Dev Notes (les entrées passées du Change Log gardent leur texte). Branche reprise d'`origin/epic/49-types-de-lignes-facture` (`abb246252`, documentation seule) par `153f6ba5c`. Décomptes inchangés : D1 à D11, AC1 à AC16, S1 à S14, T0 à T12. Statut : `ready-for-dev` → `in-progress` (fiche et `sprint-status.yaml`). |
| 2026-11-24 | **Développement repris et terminé** (Opus 5.5, `bmad-dev-story`). Branche d'épique reprise (`87e00ba05`, v0.19.1, par `0d033d36e`) ; **version de départ portée à la 0.19.1**, dernière image publiée (C-e491d-1), sites opératoires de la fiche corrigés. **T0** jouée sur `gcorbaz/kesh:0.19.1` : deux prémisses de D1 fausses, écrites au Dev Agent Record avant le code — QR-IBAN du relevé A (C-e491d-2), `total_amount` HT (C-e491d-3) — sans effet sur ce que la recette compare. Code : `a5cf95297`, `a02b79bec`, `bd7e72ce8` (dont deux défauts du script trouvés par le test : `id_piece`, et la comparaison à vide d'une trame ou d'un avant vides, scénario S6c ajouté). Test shell vert, 155 assertions ; **21 mutations, 20 tuées, S3c équivalente** (deux défauts de l'outillage de mutation corrigés en route, C-e491d-5). **Recette réelle verte** depuis l'image 0.19.1 (5 paires au même dessin, projections non vides et égales, crochet vert), **R1 et R2 rouges**, trame éprouvée hors script ; **script complémentaire vert** (17 tables, PDF figés à l'octet, F8 `410`) et son rouge (2 écarts) ; `test-fast --ci` vert (6264), `gate-ci kesh_ci_e491d` vert (6283), perte assumée code 0. `docs/testing.md` et manuel d'administration FR (section 0.20.0, « Compatibilité ») écrits ; `make fr` vert. **Aucune facture ni aucun avoir existant ne change ; le retour par la sauvegarde fonctionne** : aucun point d'arrêt. La 49-4b étant à l'Epic 52, la branche « elle n'entre pas » de D8 et de l'AC14 s'applique. Trois issues à ouvrir (Dev Agent Record § 11). Prompt de la revue P1 écrit (`49-1d-types-de-lignes-recette-montee-review-prompt-p1.md`), revue non lancée. Statut : `in-progress` → `review` (fiche ; `sprint-status.yaml` laissé à l'orchestrateur). |
