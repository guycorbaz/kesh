# Story 49-1d : types de lignes de la facture client — la recette de montée avec factures réelles simulées, et le passage à la 0.20.0 au manuel d'administration (#1242)

Status: ready-for-dev

*Fiche rédigée le 2026-11-19 par l'agent de rédaction de la 49-1d (Opus 5.5), par le workflow `bmad-create-story`. Release
**0.20.0** (coupure jeudi 2026-11-26 au soir). **Aucune migration, aucun code de production** : la story touche l'outillage de
recette (`scripts/`), son test, `docs/testing.md` et le manuel d'administration FR. Code lu à la tête de la branche
`story/49-1d-types-de-lignes-recette-montee` = `f57f6ac80` (49-1a, avec la migration `20261118000002`, et 49-1b fusionnées dans
`epic/49-types-de-lignes-facture`). Sources : `epic-49-types-de-lignes-facture.md`, `release-0.20.0.md`, les fiches closes
`49-1a-types-de-lignes-migration-api.md` (§ *Transmis à la 49-1d*) et `49-1b-types-de-lignes-pdf.md`, le précédent de la 0.19.0
(`kesh-gate-logs/rel0190/recettes-rel0190.sh`, `recette-complement-rel0190.sh`). Les choix de cette fiche sont D1 à D11.
Validation : P1 (Sonnet) faite et remédiée le 2026-11-19, sur les arbitrages de l'orchestrateur ; branche à jour d'`epic/49`
(`d4f5bba7f` : 49-4 au fichier d'épique, #1400 rattachée, « la 49-1c ne sort pas sans la 49-4 »). Une P2 reste à lancer.*

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
  version de Kesh. Mettez à jour Kesh ou utilisez un backup compatible. » ;
  (b) **reprise d'une migration interrompue** : `DELETE FROM _sqlx_migrations WHERE version = 20261118000002 AND success = 0`,
  puis redémarrer ; le rejeu est sans effet sur ce qui était posé (formes `IF [NOT] EXISTS`), prouvé par
  `migrations_upgrade_path.rs::line_types_migration_resumes_after_a_half_run` ; le cas « premier `ALTER` échoué entièrement »
  se reprend de même.
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
rendra donc autrement en 0.20.0 si la 49-4 y entre : c'est la dérogation de Guy (D8).

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
| `invoice_reminders` (`invoice_reminders.sql:50`) | non comparée, **non peuplée** | **hors périmètre**, angle mort assumé : aucune migration de la 0.20.0 ne touche les rappels ; issue à ouvrir (Dev Notes) |
| écritures (`journal_entries`, `journal_entry_lines`) | comparées | restent ; l'écriture du groupe et la contre-passation de l'avoir y entrent (AC1) |
| réponse de l'API (`api`) | comparée, **perdue** si nommée | **résolu** : projection sur les clés d'avant (D2, AC3) ; les avoirs y entrent (AC2) |
| **le PDF** (dossier des documents) | **ni peuplé ni comparé** (`:39`, `testing.md:634`) | **résolu** : D3 à D5, AC4 à AC6 |
| séquences de numérotation (`invoice_number_sequences`, `credit_note_number_sequences`) | non comparées | hors périmètre : elles ne décrivent aucune pièce existante (le numéro de chaque pièce est dans `invoices` / `credit_notes`, comparées) ; même issue que les rappels |

**Second symptôme (validation P1, V1-5)** : *un lecteur ou un écrivain de lignes modifié par la 49-1a, qui ne tourne jamais sur
les données d'avant après la montée.* Des tables identiques ne prouvent rien d'un lecteur filtré. Liste close, tirée du filtre
lui-même :

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

| Site | Geste qui le fait tourner après la montée (D10) |
|---|---|
| `vat_report.rs:127` (rapport TVA) | `GET /api/v1/reports/vat` de la période du peuplement, dans `api` (projection par ajout) |
| `invoices.rs:499`, `:511`, `:523`, `:763`, `:805`, `:1369`, `:2374`, `:2421-2427` (création, modification, validation, compte de produit) | **validation du brouillon d'avant** F3 |
| `credit_notes.rs:539`, `:668` (création d'avoir) | **avoir** A2 sur la facture d'avant F4 |
| `invoice_settlements_write.rs:651` (règlement) | **règlement** unitaire de la facture d'avant F1 |
| `routes/invoices.rs:521` (garde de l'API, `lineType` du corps) | hors recette : un corps de la 0.19.0 est éprouvé par la 49-1a (T9 de sa fiche) |
| créances âgées (`lib.rs:1419`, `/api/v1/reports/aged-receivables`) | lecteur de factures, non de lignes : ajouté à `api` au même titre que le rapport TVA |

## Décisions

- **D1 — Le peuplement simule les factures réelles par l'API de la 0.19.0, sans un `INSERT`.** **Deux préconditions d'abord**
  (V1-1), contrôlées en base avant la première facture (code 11) : (a) le compte bancaire de la recette est **principal** — créé
  avec `"isPrimary":true`, ou rendu principal par `PUT` s'il existait déjà (`:548-552`) — et lié au compte `1020` (créé s'il
  manque) ; (b) les réglages de facturation désignent un **compte de différences d'arrondi** (`defaultRoundingAccountId`, dans le
  même `PUT` que `defaultPayableAccountId`, `:513-516`). Puis huit factures client (ordre de création = identifiants 1 à 8 sur une
  base neuve, celui que relit l'AC1) :

  | | Statut | Lignes | PDF en 0.19.0 | Rôle |
  |---|---|---|---|---|
  | F1 | validée, impayée | 1 ligne, 8.10 %, **description de plus de 44 caractères** | **figé** | PDF archivé servi tel quel ; réglée **après** la montée (D10) |
  | F2 | validée, réglée (règlement unitaire, comme aujourd'hui) | 1 ligne | non figé | rendu 0.20.0 = rendu 0.19.0 |
  | F3 | brouillon | 1 ligne | — | **validée après la montée** (D10) |
  | F4 | validée, impayée | **3 lignes**, taux 8.10, 2.60 et **0.00** (le cas d'un non-assujetti), arrondi à 5 centimes non nul | non figé | la facture de Guy jamais téléchargée ; **avoir A2 après la montée** (D10) |
  | F5, F6 | validées, **réglées ensemble** (`invoice_group`) | 1 ligne chacune | F5 **figé**, F6 non figé | règlement groupé |
  | F7 | validée, **figée**, puis **avoir** total (A1) → `cancelled` | 2 lignes, deux taux | **figé avant l'avoir** | facture annulée au PDF archivé ; PDF de l'avoir |
  | F8 | validée, **figée** ; son fichier **retiré** de la copie des documents (D3) | 1 ligne | figé, fichier manquant | **regel** par la 0.20.0 |

  Les descriptions de F2 à F8 restent **sous 44 caractères** (D8) ; l'option `--designation-longue` (D8) porte la première ligne
  de F4 au-delà. Chaque PDF téléchargé est contrôlé : statut HTTP 200, en-tête `%PDF-` (les cinq premiers octets), sinon la phase
  échoue en nommant la pièce et en gardant le corps reçu — jamais un refus JSON pris pour un PDF. Le règlement groupé passe par un
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
  sont éprouvés par la 49-1a, T2 de sa fiche).
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
  elle rend F2, F4, F6 et l'avoir A1 — la référence 0.19.0. La phase 2 (binaire neuf) les rend **après** son empreinte et le
  crochet. La référence du regel de F8 est son fichier figé de la phase 1. Une **phase 6 « Rendus et gestes »** (code **16**)
  compare, après les phases 4 et 5, **un nombre exact de paires** : **5** sans `--import` (F2, F4, F6, A1, regel de F8), **9** avec
  (plus F2, F4, F6, A1 rendus après l'import) ; une paire ne compte que si les deux PDF ont passé D1 (200, `%PDF-`), ont au moins une
  page et si **leur texte contient le numéro de la pièce** (`invoiceNumber`, ou le numéro de l'avoir, relus à l'API) — un rendu vide
  ou étranger ne compte pas. Un décompte différent de 5 ou de 9 échoue (code 16). *Écartée* : relancer l'image en phase 1 après le
  dump pour prendre les références puis restaurer (un cycle de plus, même preuve).
- **D6 — La phase 5 importe dans un dossier de documents neuf.** `lancer_binaire` reçoit le dossier des documents en argument ;
  l'import écrit dans `$D/documents-import`, vide : un PDF figé servi après l'import vient **de l'archive**. F1, F5, F7 **et F8**
  (dont l'archive a gardé le fichier) y sont servis à l'octet ; F2, F4, F6 et A1 y sont rendus et comparés en phase 6. *Écartée* :
  réutiliser `$D/documents`.
- **D7 — Ce qui ne vaut que pour la 0.20.0 va au crochet `--apres`**, écrit sous `kesh-gate-logs/rel0200/verifs-0200.sh` par
  cette story, copié **en entier** au Dev Agent Record et relu en revue comme du code (`docs/testing.md`, § `--apres`). Contrôles :
  toutes les lignes des deux tables sont `priced` (`SELECT COUNT(*) FROM invoice_lines WHERE line_type <> 'priced'` = 0, de même
  pour `credit_note_lines`) ; leur nombre égale celui d'avant, **lu au fichier** `$L/empreinte-avant/invoice_lines` (et
  `…/credit_note_lines`), une ligne par enregistrement — le crochet n'a pas d'autre mémoire ; `COLUMN_DEFAULT` vaut `'priced'`,
  **apostrophes comprises** (forme de MariaDB 10.11), sur les deux tables. Le crochet **exporte** aussi un `.keshbackup` du binaire
  neuf vers `$D/apres-0.20.0.keshbackup` (pour D11). **Chaque contrôle s'écrit `[ … ] || exit 1`** (jamais `[ … ] && …`, jamais
  dans un `if` ni un `local x=$(…)` : les exceptions de `set -e` que l'en-tête du script documente, `:32-37`). *Écartée* : ces
  contrôles dans le script (il deviendrait propre à une release).
- **D8 — Désignations : ce que la recette compare, et la dérogation de la 49-4.** La 49-4 (fichier d'épique, `:130`, fusionné par
  `d4f5bba7f`) imprime une désignation longue en entier au lieu de la couper à 44 caractères ; Guy a admis la dérogation pour une
  facture d'avant **jamais téléchargée** ou **regelée**. D'où : (a) F1, longue, est **figée** : son PDF archivé ne change jamais,
  49-4 ou non, et se compare à l'octet ; (b) **F2 à F8, et donc A1** (qui recopie les lignes de F7 et se rend à chaque demande,
  jamais figé), restent sous 44 caractères : leur rendu doit être identique, quelle que soit la liste des stories de la release ;
  (c) l'option **`--designation-longue`** du script porte la première ligne de F4 à plus de 44 caractères ; elle sert **seulement**
  si la 49-4 **n'entre pas** dans la release : l'intégration rejoue alors la recette avec elle, et le rendu de F4 doit rester
  identique (la troncature de la 0.19.0 tient). Si la 49-4 entre, l'option ne se joue pas : le rendu d'une désignation longue
  **change** par décision de Guy, et la preuve de ce changement est celle de la fiche de la 49-4. Tout **autre** écart de rendu fait
  échouer la phase 6, quelle que soit la story qui l'a causé (48-15, #1327 de Beat, 49-3…), et revient à l'orchestrateur.
- **D9 — `mutool` est un outil de la recette, remplaçable au test par `KESH_MUTOOL`.** Ajouté à la liste des outils (`:123`,
  absent → code 5) avec `sha256sum` ; `KESH_MUTOOL` traverse la relance comme `KESH_DOCKER` et `KESH_CURL` (`:65`, `:81-83`), et
  l'enveloppe Rust le retire (`.env_remove`) comme elles. **Le test pose `KESH_MUTOOL` à chaque scénario** (vers son factice, ou
  vers un chemin absent) : le `mutool` de l'hôte n'est **jamais** atteint par le test, et la relance remettant `PATH=/usr/bin:/bin`
  (`:57`), un `PATH` de test ne suffirait pas à le masquer. Le factice consigne chacun de ses appels dans un journal du mini-dépôt :
  un scénario nominal exige ce journal non vide. *Motif* : la CI n'installe pas `mutool` (`grep -n 'apt-get'
  .github/workflows/ci.yml` : `mold`, `mariadb-client` seulement), la station l'a (`/usr/bin/mutool`) : sans ces deux règles, une
  preuve du rouge dépendrait de l'hôte (V1-6). *Écartée* : sauter la phase 6 si `mutool` manque.
- **D10 — Après la montée, les lecteurs et les écritures changés par la 49-1a tournent sur les données d'avant, sous les deux
  versions.** Le rapport TVA et les créances âgées de la période du peuplement entrent dans `api` (`GET /api/v1/reports/vat`,
  `lib.rs:1443`, requête `ReportQuery`, `reports.rs:855` ; `GET /api/v1/reports/aged-receivables`, `lib.rs:1419`), sous la
  projection par ajout (D2) : rendus par la 0.19.0 à l'empreinte `avant`, par la 0.20.0 à l'empreinte `apres`. Puis trois
  **gestes d'écriture**, joués **à l'identique** en phase 2 (0.20.0, après les rendus) et en phase 4 (0.19.0, sur la base restaurée,
  après les rendus, dans le même ordre) : **validation** du brouillon d'avant F3 ; **avoir** A2 sur la facture d'avant F4 ;
  **règlement** unitaire de F1 (même compte, même date, même montant). Chaque geste exige son statut HTTP de succès. L'**extrait
  des gestes** de chaque phase : les lignes créées après l'empreinte (identifiant supérieur au maximum relevé à l'empreinte
  `avant`) de `journal_entries`, `journal_entry_lines`, `credit_notes`, `credit_note_lines`, `invoice_settlements`, et les lignes
  d'`invoices` de F1, F3, F4 ; projeté sur les colonnes d'avant, **sans** les colonnes d'instant (type `DATETIME` ou `TIMESTAMP`,
  lues dans `information_schema`) ni `pdf_storage_path`, `pdf_sha256` (les rendus de chaque phase diffèrent par leur `/ID`). La
  phase 6 exige les deux extraits **égaux** et non vides : **3** écritures neuves (validation, avoir, règlement), **1** avoir,
  **1** règlement — sinon code 16. C'est l'usage de Guy dès la mise à jour du NAS : valider ses brouillons d'avant, émettre un avoir
  ou encaisser une facture d'avant. *Écartées* : (a) se fier à l'égalité des tables (un lecteur filtré à tort les laisse
  identiques) ; (b) comparer à des valeurs écrites en dur dans le script (elles dépendraient du plan comptable du preset) ;
  (c) l'angle mort assumé (refusé par l'orchestrateur).
- **D11 — Les trois gestes que le manuel prescrit se jouent, dans un script d'intégration sur le modèle de la 0.19.0.**
  `kesh-gate-logs/rel0200/recette-complement-rel0200.sh` (hors dépôt, comme `rel0190/recette-complement-rel0190.sh` et
  `recette-refus-import-rel0190.sh`), écrit et joué par cette story, rejoué par l'intégration ; conteneurs `kesh-rel0200c-*`, label
  `kesh.rel0200c`, base `mariadb:10.11` en tmpfs, valeurs factices ; il prend en argument le dossier `$D` laissé par la recette
  (AC10) et y lit `backup/pre-upgrade/` et `apres-0.20.0.keshbackup` (D7). Trois jeux, chacun son code :
  (a) **refus d'import** — une image 0.19.0 neuve, base vide, importe `apres-0.20.0.keshbackup` : `400`, code
  `IMPORT_SCHEMA_MISMATCH`, message fr-CH exact (`messages.ftl:2460`), le journal du serveur nommant `line_type` ;
  (b) **retour par `pre-upgrade/`** — une image 0.19.0 neuve, base vide, importe la sauvegarde `pre-upgrade/` écrite par la montée
  (nom finissant par `-0.19.0-to-0.20.0.keshbackup`) depuis l'assistant (la route de l'assistant, comme le manuel le prescrit) :
  `200`, et l'empreinte des tables de `TABLES` égale **strictement** celle d'avant de la recette (`$D/journal/empreinte-avant/`) ;
  (c) **reprise d'une migration interrompue, avec le vrai binaire** — une base portée à 95 migrations par l'image 0.19.0, puis
  l'état que laisse une coupure entre les deux `ALTER` posé à la main exactement comme le fait
  `migrations_upgrade_path.rs::line_types_migration_resumes_after_a_half_run` (le premier `ALTER` de la migration appliqué, une ligne
  `success = 0` de la version `20261118000002` dans `_sqlx_migrations`) : le binaire 0.20.0 **refuse de démarrer** (*dirty* au
  journal, `/health` jamais servi) ; `DELETE FROM _sqlx_migrations WHERE version = 20261118000002 AND success = 0` ; le binaire
  démarre, 96 migrations `success = 1`, `line_type` présent sur les deux tables, contraintes de la migration présentes. Journal
  attendu : `kesh-gate-logs/e491d/recette-complement.log` (à la story), `kesh-gate-logs/rel0200/recette-complement-*.log` (au rejeu).
  Le manuel ne dit « constaté » que ce que ces jeux ont établi (AC14). *Écartées* : (a) laisser ces gestes à #920 et ne les établir
  qu'au code (le manuel prescrirait à Guy un `DELETE` sur le registre des migrations de sa production sans l'avoir vu tourner) ;
  (b) les mettre dans `scripts/recette-montee.sh` (le jeu (c) est propre à cette migration, le script deviendrait propre à la
  release).

## Critères d'acceptation

**Le script (`scripts/recette-montee.sh`)**

- **AC1 — Peuplement (D1).** Les deux préconditions d'abord, contrôlées en base (code 11) : exactement un compte bancaire
  `is_primary = TRUE AND archived = FALSE`, lié au `1020` ; `company_invoice_settings.default_rounding_account_id IS NOT NULL`. Puis
  la phase 1 crée, par l'API de l'image, les huit factures, l'avoir A1 et le règlement groupé de D1, et télécharge le PDF de F1, F5,
  F7 et F8 **avant** l'avoir (F7 se fige validée) ; chaque téléchargement : HTTP 200 et en-tête `%PDF-`, sinon code 11 nommant la
  pièce. Contrôles relus en base, chacun en code 11 : statuts
  `validated,validated,draft,validated,validated,validated,cancelled,validated` ; `invoice_settlements` = 3 (F2, F5, F6), dont 2
  portant `journal_entry_line_id` ; `credit_notes` = 1 ; `invoices.pdf_sha256 IS NOT NULL` pour exactement F1, F5, F7, F8 ; F4 a trois
  lignes, trois taux distincts dont `0.00`, et `rounding_amount <> 0` ; aucune description de F2 à F8 ne dépasse 44 caractères (sauf
  F4 sous `--designation-longue`). Les factures fournisseur et le relevé A restent tels quels. Aucune donnée réelle (AC-7 du test
  inchangé).
- **AC2 — Empreinte.** `TABLES` gagne `credit_notes` et `credit_note_lines` ; `api` gagne `GET /api/v1/credit-notes/{id}` de chaque
  avoir peuplé, le rapport TVA et les créances âgées de la période du peuplement (D10). `docs/testing.md` liste exactement ce que
  l'empreinte couvre.
- **AC3 — `--diff-attendu` par ajout (D2).** Pour une table ou `api` nommés, la projection sur l'avant est identique, sinon la
  phase échoue (code 12 à la montée, 15 à l'import) en nommant le fichier et en écrivant le diff de la projection au journal. Les
  autres noms gardent l'écart libre. La phase 4 (retour) reste à l'égalité **stricte** de tous les fichiers.
- **AC4 — Documents et PDF archivés (D3, D4 a).** Phase 1 : copie par `docker cp` du dossier des documents de l'image (conteneur
  du jeton seulement), puis retrait du fichier de F8 ; aucun montage (`test.sh:414` reste vert). Phase 2, **après** l'empreinte
  et le crochet : F1, F5, F7 servis par le binaire neuf, octets égaux à ceux de la phase 1 (SHA-256 égal aussi à `pdf_sha256`) ;
  F8 répond `410` puis son regel (`POST …/pdf/refreeze`) répond `200` ; F2, F4, F6 et A1 rendus et gardés pour la phase 6.
- **AC5 — Rendus (D4 b, D5) : phase 6, code 16.** Après la phase 4 (qui rend F2, F4, F6, A1 sous l'image 0.19.0, **après** son
  empreinte) : chaque rendu 0.20.0 a, page à page, la même trame à 300 dpi, le même texte et le même nombre de pages (≥ 1) que sa
  référence 0.19.0 (pour F8 : son fichier figé de la phase 1). **Décompte exact** : 5 paires sans `--import`, 9 avec ; une paire ne
  compte que si ses deux PDF sont valides (D1) et que leur texte contient le numéro de la pièce. Un écart nomme la pièce et écrit le
  diff du texte au journal ; un décompte faux échoue. La version de `mutool` s'écrit au journal.
- **AC6 — Import (D6).** Avec `--import`, la phase 5 importe dans `$D/documents-import` (neuf) ; F1, F5, F7, F8 y sont servis à
  l'octet de la phase 1 ; F2, F4, F6, A1 y sont rendus pour la phase 6 ; l'empreinte suit AC3.
- **AC7 — Outils (D9).** `mutool` et `sha256sum` sont exigés (code 5 s'ils manquent) ; `KESH_MUTOOL` traverse la relance et rien
  d'autre de neuf ; l'en-tête du script (`:10-19`, phases et codes) et `:38-39` disent la phase 6, l'option `--designation-longue`
  et ce qui est désormais comparé.
- **AC8 — Gestes sur les données d'avant (D10).** En phase 2 et en phase 4, dans le même ordre et après les rendus : validation de
  F3, avoir A2 sur F4, règlement de F1, chacun en succès HTTP ; la phase 6 exige les deux extraits des gestes égaux, avec 3 écritures
  neuves, 1 avoir, 1 règlement (code 16 sinon). Le rapport TVA et les créances âgées passent la projection de l'AC3.

**Les factures réelles (consigne de production)**

- **AC9 — Les factures et avoirs existants restent identiques, et la recette le prouve sur l'image publiée.** Après la montée de
  0.19.0 à la tête de la story : lignes (toutes les colonnes d'avant), totaux, arrondi, TVA par ligne, rapport TVA, écritures,
  numéros, règlements (unitaires et groupé), avoir et ses lignes — **identiques** (AC3) ; PDF archivés servis à l'octet (AC4) ; PDF
  rendus de nouveau — facture jamais téléchargée, regel, avoir — **au même dessin** (AC5 ; ce que ce mot couvre et ne couvre pas :
  D4) pour les désignations de 44 caractères au plus (D8) ; les gestes courants sur ces pièces écrivent ce qu'écrivait la 0.19.0
  (AC8) ; la même chose après l'import d'une sauvegarde 0.19.0 (AC6).

**Le jeu réel**

- **AC10 — Recette réelle, verte, à la tête de la story, avec ses rouges réels.** Sous le verrou de la voie base et `mem-guard`,
  depuis le worktree :
  ```sh
  flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh scripts/recette-montee.sh \
    --image gcorbaz/kesh:0.19.0 --binaire target/debug/kesh-api --static frontend/build \
    --port-api 3197 --port-base 3398 --migrations-attendues 1 \
    --diff-attendu migrations,invoice_lines,credit_note_lines,api \
    --apres ../kesh-gate-logs/rel0200/verifs-0200.sh --import --prefixe kesh-e491d
  ```
  → code 0, six phases jouées, la phase 3 « refusée : migration inconnue d'elle (sqlx) » (`20261118000002`), 9 paires de rendus et
  les gestes comparés ; sortie copiée au Dev Agent Record (`kesh-gate-logs/e491d/recette-reelle.log`). **Rouges réels**, chacun son
  journal sous `kesh-gate-logs/e491d/` :
  (R1) `--diff-attendu migrations` seulement → code 12 nommant `invoice_lines`, `credit_note_lines` et `api`
  (`recette-reelle-rouge-diff.log`) — l'empreinte voit la migration ;
  (R2) **projection** : une copie mutée du script, **non suivie et retirée ensuite** (`scripts/recette-montee.mut.sh`, sous
  `RACINE`), exécute juste avant `empreinte apres` `SQL "UPDATE invoice_lines SET unit_price = unit_price + 0.0001 WHERE id = 1"` →
  code 12 nommant `invoice_lines` (`recette-reelle-rouge-projection.log`) — une valeur d'avant altérée rougit malgré `--diff-attendu` ;
  (R3) **rendu** : la même copie mutée échange les références 0.19.0 de F2 et de F6 avant la phase 6 → code 16 nommant F2 et F6
  (`recette-reelle-rouge-rendu.log`) ; et, hors script, sur les vrais PDF du journal, la ligne `mutool` de D4 appliquée à F2 contre
  F2 rend des MD5 **égaux**, à F2 contre F4 des MD5 **différents** (commande et sortie au Dev Agent Record) — la trame distingue deux
  factures réelles ;
  (R4) **gestes** : la copie mutée change le montant du règlement de F1 en phase 2 seulement → code 16
  (`recette-reelle-rouge-gestes.log`).
  Après les rouges, `git status --porcelain` ne nomme pas la copie mutée.
- **AC11 — Jeux du manuel (D11).** `kesh-gate-logs/rel0200/recette-complement-rel0200.sh` joué sur le `$D` de l'AC10 : (a) refus
  d'import `400 IMPORT_SCHEMA_MISMATCH`, message exact ; (b) retour par `pre-upgrade/` dans une 0.19.0 neuve, empreinte égale à
  celle d'avant ; (c) reprise d'une migration interrompue avec le vrai binaire — refus de démarrer, `DELETE` de la ligne, démarrage,
  96 migrations. Texte du script et sortie (`recette-complement.log`) au Dev Agent Record.
- **AC12 — Rejeu au dernier commit de code de la release.** La fiche et `docs/testing.md` disent que l'intégration de la 0.20.0
  rejoue l'AC10 (vert et R1) et l'AC11 au dernier commit de code de la release (toutes stories fusionnées), journaux sous
  `kesh-gate-logs/rel0200/`, **avec** `--designation-longue` si la 49-4 n'est pas dans la release (D8), et que la section du manuel
  (AC14) ne dit « constaté » que ce que ce rejeu a établi. Le crochet `verifs-0200.sh` et `recette-complement-rel0200.sh` existent et
  se relisent en revue.

**Documentation**

- **AC13 — `docs/testing.md`** (§ *La recette de montée et de retour*) : exemple d'appel à jour ; sens de `--diff-attendu` (D2) ;
  empreinte (AC2) ; documents et PDF (« Aucun document n'est peuplé ni comparé » retiré, remplacé par ce qui l'est et ce qui ne
  l'est pas, avec la réserve de D4 sur « même dessin ») ; le peuplement, ses préconditions et ses contrôles (AC1) ; les gestes
  (AC8) ; la phase 6, le code 16 et le décompte exact ; `--designation-longue` ; `mutool`, `sha256sum`, `KESH_MUTOOL` ; la durée
  **mesurée** du test ; le rejeu au dernier commit de code (AC12) et le script complémentaire (D11).
- **AC14 — Manuel d'administration FR, section « Passer à la 0.20.0 »** (`docs/manual/fr/admin-manual.tex`, `.tex` seulement,
  sur le modèle de `:2564-2586`), **en tête** : « **Prenez avant la mise à jour une sauvegarde manuelle complète** — la base, un
  `.keshbackup` et le dossier des documents » et « le retour à la 0.19.0 passe uniquement par la sauvegarde d'avant la mise à
  jour » ; puis l'introduction (« La 0.20.0 se pose sur la 0.19.0 », ni compose ni variable neuve) ; **« Une migration »**
  (`20261118000002\_invoice\allowbreak\_line\allowbreak\_types`, 96 au total, non breaking, `kesh_version_min_required` reste à
  `0.17.6`, aucune ligne existante modifiée — chacune reçoit le type « ligne chiffrée » —, sauvegarde `pre-upgrade/` au nom finissant
  par `-0.19.0-to-0.20.0.keshbackup`, `KESH_SKIP_PRE_UPGRADE_BACKUP=0.20.0`) ; **« Le retour à la 0.19.0 »** (la 0.19.0 ne démarre
  plus sur la base migrée — journal « migration 20261118000002 was previously applied but is missing in the resolved migrations » ;
  elle refuse une sauvegarde 0.20.0 avec le message exact d'`IMPORT_SCHEMA_MISMATCH`, **distinct** d'`INVALID_BACKUP_STRUCTURE` ;
  retour sur une base neuve par la sauvegarde d'avant la mise à jour ; une sauvegarde 0.19.0 s'importe dans la 0.20.0) ; **« Si la
  migration échoue au milieu »** — **d'abord** une sauvegarde de la base dans son état (un `DELETE` sur `_sqlx_migrations` se fait sur
  une base déjà sauvegardée) ; puis la procédure jouée par l'AC11 (c) : Kesh refuse de démarrer, migration *dirty* ; retirer la ligne
  `success = 0` de la version `20261118000002` ; redémarrer ; le rejeu termine sans effet sur ce qui était posé, y compris si le
  premier `ALTER` a échoué entièrement ; la sauvegarde d'avant la mise à jour reste le recours. Le paragraphe dit en toutes lettres
  que **cette procédure ne vaut que pour cette migration** (écrite pour se rejouer sans effet) et qu'elle **ne contredit pas** le
  conseil de la section 0.19.0 (« ne reprenez pas le schéma à la main », `:2578`), qui reste vrai pour la migration de la 0.19.0 ;
  **« Ce qu'il faut savoir »** : factures et avoirs existants inchangés ; PDF archivé servi tel quel ; une facture jamais téléchargée,
  une facture regelée et un avoir **se rendent au même dessin qu'en 0.19.0 pour des désignations de 44 caractères au plus** ;
  **si la 49-4 est dans la release** (liste des stories fusionnées au dernier commit de code), la phrase nomme la dérogation : une
  désignation de plus de 44 caractères d'une facture jamais téléchargée ou regelée s'imprime désormais en entier, sur plusieurs
  lignes, au lieu d'être coupée — le PDF déjà archivé, lui, ne change pas ; sinon, la phrase vaut pour toute désignation, et le rejeu
  avec `--designation-longue` l'a établi. « Constaté » ne s'écrit que pour ce que l'AC10, l'AC11 et leur rejeu (AC12) ont joué. Le
  paragraphe « Pour les intégrations » de la 49-1a est **gardé** ; le commentaire `:2551-2555` retiré ou réécrit. Le paragraphe
  « Compatibilité » (`:2302`) gagne la phrase de la 0.20.0 : aucune table neuve, mais l'import compare aussi les **colonnes**
  (une colonne de la sauvegarde inconnue de la version qui l'importe, ou une colonne obligatoire absente de la sauvegarde) et ce refus
  a **son propre message**, « Le schéma de ce backup est incompatible avec cette version de Kesh. Mettez à jour Kesh ou utilisez un
  backup compatible. » (code `IMPORT\_SCHEMA\_MISMATCH`) ; une sauvegarde 0.19.0 s'importe dans la 0.20.0, ses lignes devenant
  chiffrées ; l'inverse est refusé (\S\ref{sec:maj-0-20-0}). Le manuel compile (`make fr` vert, `Overfull \hbox` relevés), **PDF non
  commité**.

**Périmètre et gates**

- **AC15 — Aucun code de production.** `git diff --stat origin/epic/49-types-de-lignes-facture..HEAD -- crates frontend` ne nomme
  que `crates/kesh-db/tests/recette_montee_script.rs` (une ligne `.env_remove("KESH_MUTOOL")`, et la borne de durée si la mesure
  l'exige) ; aucune migration (`ls crates/kesh-db/migrations/*.sql | wc -l` = 96).
- **AC16 — Gates.** Pendant la boucle : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo nextest run -E 'binary(recette_montee_script)'` (gate ciblé) et `shellcheck` sur les scripts s'il est installé. Au dernier
  commit de code : bases `kesh_e491d`, `kesh_ci_e491d`, `kesh_e2e_e491d` remises à neuf ; `scripts/gate-ci.sh` sur `kesh_ci_e491d`
  vert ; **E2E Playwright complet** « vert » au sens de `docs/testing.md` § *Les échecs attendus* (règle du `CLAUDE.md` : la story
  touche du code exécutable) ; AC10 et AC11 rejoués à ce commit. Recette visuelle : sans objet (aucun contrôle visible).

## Tests à écrire, et la preuve du rouge

Tous dans `scripts/tests/recette-montee.test.sh` (factices : `docker`, `curl`, binaire, et désormais `mutool`), lancé au gate par
`crates/kesh-db/tests/recette_montee_script.rs`. **Chaque scénario pose `KESH_MUTOOL`** (D9). Chaque preuve du rouge est une
**mutation du script** (`scripts/recette-montee.sh` du mini-dépôt, copie modifiée), le test relancé **sur la station** (où
`/usr/bin/mutool` existe : la preuve ne doit pas en dépendre), le journal gardé sous `kesh-gate-logs/e491d/mut/rouge-T<n>.log`, puis
la mutation retirée. Les factices évoluent : le `docker` factice connaît `cp` (copie un PDF factice dans la cible), les requêtes de
colonnes (`information_schema`), les projections et les nouveaux décomptes ; le `curl` factice sert `/pdf` (corps commençant par
`%PDF-`), `/pdf/refreeze` (410 puis 200), `/credit-notes`, `/reconciliation/accept`, `/bank-imports/manual`, `/reports/vat`,
`/reports/aged-receivables`, et consigne les corps reçus ; le `mutool` factice écrit une trame, un texte (contenant le numéro de la
pièce) et un nombre de pages qui dépendent d'un réglage, et consigne chacun de ses appels.

| # | Scénario | Ce qu'il prouve | Preuve du rouge |
|---|---|---|---|
| T1 | nominal, `--diff-attendu migrations,invoice_lines,credit_note_lines,api`, `ECART=invoice_lines` (le factice ajoute une colonne : `SELECT *` change, la projection non) | code 0, six phases ; le journal du `mutool` factice est non vide | — (témoin des autres) |
| T2 | `ECART_ANCIEN=invoice_lines` (une colonne d'avant change) avec le même `--diff-attendu` | code 12, sortie nomme `invoice_lines` (AC3) | retirer la comparaison de projection → code 0 : **rouge** (`rouge-T2.log`) |
| T3 | `ECART_ANCIEN=api` (une clé d'avant change) / `ECART=api` (une clé s'ajoute) / `ECART_TYPE=api` (un objet devient scalaire) | 12 / 0 / 12 (AC3) | projection `api` remplacée par `true` → T3a rend 0 : **rouge** ; contrôle de type retiré → T3c rend 0 : **rouge** |
| T4 | `ECART_ANCIEN=credit_note_lines` **sans** la nommer | code 12 (elle est dans `TABLES`, AC2) | retirer `credit_note_lines` de `TABLES` → code 0 : **rouge** |
| T5 | `PDF_FIGE_ALTERE=oui` (le `curl` factice sert d'autres octets en phase 2) | code 12, nomme la facture (AC4) | retirer la comparaison des octets → **rouge** |
| T6 | `RENDU_ECART=oui` | code 16, nomme la pièce (AC5) | retirer la phase 6 → code 0 : **rouge** |
| T7 | `RENDU_VIDE=oui` (le factice rend un texte sans le numéro, ou zéro page) ; `PDF_PAS_PDF=oui` (le `curl` factice sert un corps JSON en 200) | 16 ; 11 (D1, D5) | compter les paires sans contrôler le numéro → T7a rend 0 : **rouge** ; retirer le contrôle `%PDF-` → T7b ne rend pas 11 : **rouge** |
| T8 | `REGEL_ECHEC=oui` (refreeze `409`) | code 12 (AC4) | ignorer le statut du regel → **rouge** |
| T9 | nominal : `docker cp` appelé sur le conteneur du jeton, vers `$D/documents` ; `test.sh:414` (aucun montage) **inchangé et vert** | D3 | remplacer `docker cp` par un `-v` → l'assertion `:414` **rougit** |
| T10 | `STATUTS_CLIENTS=` autre / `NB_AVOIRS=0` / `PRINCIPAL=0` / `ARRONDI_COMPTE=null` | code 11 (AC1) | retirer chaque contrôle → **rouge** |
| T11 | `KESH_MUTOOL` vers un chemin absent | code 5, zéro appel `docker` (AC7) | retirer `mutool` de la liste des outils → le script atteint la phase 6 et sort autrement qu'en 5 : **rouge** (le chemin absent est imposé par `KESH_MUTOOL`, le `mutool` de l'hôte n'est jamais pris) |
| T12 | `--import` nominal ; `PDF_IMPORT_ALTERE=oui` | 0 ; 15 (AC6) | importer dans `$D/documents` au lieu du dossier neuf : le factice le voit (il consigne le `KESH_DOCUMENTS_DIR` reçu) → **rouge** |
| T13 | la relance : `KESH_MUTOOL` traverse — le journal du `mutool` **factice** est non vide après le run —, une variable `KESH_AUTRE` ne traverse pas | D9 | retirer `KESH_MUTOOL` de la liste blanche → le factice n'est plus appelé, son journal reste vide : **rouge**, quel que soit l'hôte |
| T14 | décompte des paires : sans `--import` 5, avec 9 ; `PAIRES_EN_MOINS=1` (le factice fait échouer un téléchargement d'une pièce rendue) | 0 ; 16 (AC5) | remplacer le décompte exact par « au moins une » → **rouge** |
| T15 | `GESTE_ECART=oui` (le `docker` factice rend un extrait de gestes différent en phase 4) ; `GESTE_VIDE=oui` | 16 ; 16 (AC8) | retirer la comparaison des gestes → **rouge** ; retirer le contrôle des décomptes (3, 1, 1) → T15b rend 0 : **rouge** |
| T16 | `--designation-longue` | le corps de création de F4 consigné par le `curl` factice porte une description de plus de 44 caractères ; sans l'option, aucune | ignorer l'option → **rouge** |

**Le jeu réel** (AC10) est l'épreuve de bout en bout, avec ses rouges réels R1 à R4 ; le script complémentaire (AC11) a ses propres
contrôles, éprouvés par son jeu (c) qui doit d'abord **refuser**. **E2E Playwright neuf** : aucun — la story ne change ni écran ni
réponse HTTP ; la suite complète tourne tout de même au dernier commit de code (AC16).

## Tâches

- [ ] **T0 — Sonde, avant tout changement** (journal `kesh-gate-logs/e491d/t0/`) : `docker image inspect gcorbaz/kesh:0.19.0`
  (tirer si absente) ; `command -v mutool sha256sum` et `mutool -v` ; jouer **à la main**, contre une image 0.19.0 jetable (préfixe
  `kesh-e491dt0`, base `mariadb:10.11` en tmpfs), les gestes de D1 et D10 que le script ne fait pas encore : compte bancaire
  principal et lien au `1020`, compte d'arrondi désigné, relevé saisi au jour (continuité de solde avec le relevé A : les drapeaux
  `confirm*` du précédent), `invoice_group` accepté, avoir, téléchargement d'un PDF (le fichier apparaît-il sous `/tmp/documents` du
  conteneur ? `docker cp` le rend-il lisible par l'utilisateur ?), le taux `0.00` accepté à la validation, un `410` puis un regel,
  le rapport TVA et ses paramètres (`ReportQuery`) ; noter si un `410` ou un regel écrit une ligne ` ERROR ` (sinon
  `journal_sans_erreur` échouerait). Toute surprise s'écrit au Dev Agent Record **avant** d'écrire le script ; une prémisse fausse de
  D1 à D11 arrête la story et se dit.
- [ ] **T1 — Peuplement** (AC1) : préconditions, huit factures, avoir, groupe, contrôles, `--designation-longue`.
- [ ] **T2 — Empreinte et projection** (AC2, AC3) : relevé des colonnes à `avant`, projections, `api` par clés et types, rapports,
  diff au journal.
- [ ] **T3 — Documents et PDF** (AC4, AC6) : `docker cp`, retrait de F8, phase 2 (servis, 410, regel, rendus), phase 5 (dossier
  neuf).
- [ ] **T4 — Phase 4, gestes et phase 6** (AC5, AC8) : rendus 0.19.0 après l'empreinte du retour, gestes dans les deux phases,
  comparaison trame (300 dpi) + texte + pages, décompte exact, extraits des gestes, code 16.
- [ ] **T5 — Outils et relance** (AC7, D9) ; enveloppe Rust (`.env_remove("KESH_MUTOOL")`).
- [ ] **T6 — Test shell** (T1 à T16) et mutations ; **durée mesurée avant la revue**, sous nextest, à la station (deux tirages) ;
  borne de `recette_montee_script.rs` (`timeout -k 10 600`, `:25`) relevée seulement si la mesure l'exige (dire la valeur).
- [ ] **T7 — Crochet** `kesh-gate-logs/rel0200/verifs-0200.sh` (D7), texte intégral copié au Dev Agent Record.
- [ ] **T8 — Recette réelle** (AC10) et ses rouges réels R1 à R4 ; copie mutée retirée.
- [ ] **T9 — Script complémentaire** `kesh-gate-logs/rel0200/recette-complement-rel0200.sh` (D11, AC11), joué, texte et sortie au
  Dev Agent Record.
- [ ] **T10 — `docs/testing.md`** (AC13) ; grep du symptôme « aucun document » (`git grep -n -i -e "aucun document n'est peuplé" -e
  "vivent dans son" -- scripts docs crates`) : tous les sites traités.
- [ ] **T11 — Manuel d'administration** (AC14) ; `make fr` sous `mem-guard`, puis `git checkout -- docs/manual/fr/*.pdf` (PDF jamais
  commités) ; vérifier le texte rendu (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`) avant le
  `checkout`.
- [ ] **T12 — Gates** (AC16) et Dev Agent Record (ce qui a tourné, et seulement cela).

## Note de version

*(Pour le CHANGELOG, assemblée à l'intégration ; la story n'écrit pas `CHANGELOG.md`.)*

> Le manuel d'administration décrit le passage à la 0.20.0 : la sauvegarde complète à prendre avant la mise à jour (base,
> `.keshbackup` et dossier des documents), le retour à la 0.19.0 et la reprise d'une migration interrompue. La montée a été
> éprouvée, depuis l'image publiée 0.19.0, sur une base de factures simulées — payées, impayées, réglées en groupe, annulées par un
> avoir, au PDF archivé ou jamais téléchargé — : chacune reste identique ; un PDF archivé est servi tel quel, et un PDF rendu de
> nouveau se dessine comme en 0.19.0 pour une désignation de 44 caractères au plus. (refs #1242)

*(Si la 49-4 entre dans la release, l'intégration ajoute la dérogation : une désignation plus longue d'une facture jamais
téléchargée ou regelée s'imprime désormais en entier.)*

## Dev Notes

- **Charge, au regard de la coupure du jeudi 2026-11-26** (estimation de la rédaction, après la P1) : script et test shell
  (D1 à D10, seize scénarios, mutations) **deux jours et demi à trois jours** ; script complémentaire (D11) **un demi-jour à un
  jour** ; manuel et `docs/testing.md` **un demi-jour** ; gates, recette réelle et ses rouges, E2E complet **un jour** (chaque
  recette réelle prend plusieurs minutes de station, sous le verrou de la voie base) ; plus la boucle de revue. Soit **cinq jours
  environ** à partir du lancement : tenable si le développement part le 2026-11-20, serré ensuite. Rien n'est coupé ici ; ce qui
  pourrait l'être si le temps manque (arbitrage de l'orchestrateur) : les gestes de D10 en phase 5, T16 (si la 49-4 entre, l'option
  ne se joue pas).
- **Hors périmètre** : (a) la phase 4 du script revient par le dump ; le retour du manuel est joué par D11 (b), #920 reste ouvert
  pour l'outillage du dépôt ; (b) rappels, séquences de numérotation : non peuplés ni comparés (inventaire) ; (c) la preuve de la
  dérogation de la 49-4 (sa fiche).
- **Issues à ouvrir** (demandées à l'orchestrateur, une par problème) : (1) `technical-debt`, P3 — *la recette de montée ne
  peuple ni ne compare les rappels (`invoice_reminders`) ni les séquences de numérotation*.
- **Écarts au fichier d'épique, déclarés** : (a) le tableau des stories écrit « manuel d'administration (×4) » ; le tableau des
  fichiers et la 49-1a (V3-1) disent **FR seul** : les manuels DE, EN, IT n'existent pas — FR seul ici ; (b) `--diff-attendu`
  nomme aussi `api` (la réponse gagne `lineType`), et n'admet qu'un ajout (D2) ; (c) fichiers touchés hors du tableau :
  `scripts/tests/recette-montee.test.sh` et `crates/kesh-db/tests/recette_montee_script.rs` (une ligne), plus deux scripts hors dépôt
  (`kesh-gate-logs/rel0200/verifs-0200.sh`, `recette-complement-rel0200.sh`) ; (d) les gestes de D10 et les jeux de D11 dépassent
  la lettre du tableau (arbitrages de l'orchestrateur sur V1-4 et V1-5). Modules : `scripts/`, `docs/` (testing, manuel),
  `kesh-db` (test seulement) — **trois**, sous le seuil de splitting.
- **Fichiers communs** : `admin-manual.tex` — la 49-1a a posé le squelette (fusionnée) ; la story de menu
  (`v0200-menu-sous-groupes`) fusionne en dernier et change des chemins du manuel ; les PDF ne se commitent pas.
  `scripts/recette-montee.sh` n'est touché par aucune autre story de la release. `sprint-status.yaml` : bloc de l'Epic 49.
- **Ce qui doit rester vrai** : le périmètre fermé du script (relance `env -i`, `HOME` neutralisé, démon local seul, écriture sous
  `$D` seulement, aucun chemin de la station, aucun montage, conteneurs du jeton seuls) ; les codes 2 à 5 et 11 à 15 ; la phase 4 à
  l'égalité stricte ; jamais `… | grep -q` sous `pipefail` (`:138-139`). **Jamais la base du NAS, jamais `~/kesh/`** : tout se joue
  dans les conteneurs de recette ; aucune donnée réelle de Guy dans les scripts, les journaux ou la fiche.
- **Station** : une commande lourde à la fois (construction de `kesh-api` et du frontend, recette réelle, script complémentaire,
  `gate-ci`, E2E), sous `flock` du verrou de la voie base et `scripts/mem-guard.sh`, `CARGO_BUILD_JOBS=2`,
  `scripts/mem-guard.sh --status` avant chacune. La recette refuse si un conteneur `kesh.recette` d'un autre run existe (code 4) :
  attendre, ne jamais arrêter le conteneur d'un autre. Ports `3197`/`3398` (et `3399` pour l'import), préfixe `kesh-e491d` ; le
  script complémentaire prend ses propres ports, distincts.
- **Rejeu** : la recette est le gate de la **release** ; elle se rejoue au dernier commit de code de la 0.20.0 (AC12), après toutes
  les fusions (48-*, 49-*, menu, #1327 si fusionnée). Un rouge de la phase 6 à ce moment signale un changement de rendu d'une
  facture existante : il ne se tolère pas, il va à l'orchestrateur (D8).
- **Références** : `epic-49-types-de-lignes-facture.md` (§ *Contraintes de production*, point 4 ; § *49-1* ; `:130`, `:154`),
  `release-0.20.0.md` (§ *Gates de la release*), fiches 49-1a (§ *Transmis à la 49-1d*) et 49-1b (D4), `docs/testing.md`
  § *La recette de montée*, `kesh-gate-logs/rel0190/recette-complement-rel0190.sh`, `recette-refus-import-rel0190.sh`,
  `kesh-gate-logs/e491d/validation-49-1d-p1.md`, `CLAUDE.md` § *Ce qu'impose la production*, § *Documentation à synchroniser*
  (point 7), #1242, #920, #925, #1363, #1364, #1400.

## Dev Agent Record

*(vide)*

## Change Log

| Date | Entrée |
|---|---|
| 2026-11-19 | Création de la fiche (workflow `bmad-create-story`), par l'agent de rédaction de la 49-1d (Opus 5.5) ; code lu à `f57f6ac80`. Neuf décisions (D1 à D9), quatorze AC, treize tests. Points arrêtés : peuplement par l'API de la 0.19.0 (huit factures, un avoir, un règlement groupé, quatre PDF figés dont un au fichier retiré) ; `--diff-attendu` réduit aux ajouts (projection sur les colonnes et les clés d'avant) ; documents copiés par `docker cp` sans montage ; PDF archivés à l'octet, rendus par trame et texte (`mutool`), référence 0.19.0 prise en phase 4, phase 6 « Rendus » (code 16) ; crochet de la release hors dépôt. Écarts au fichier d'épique déclarés (manuel FR seul, `api` nommé, test du script et enveloppe touchés). Une issue à ouvrir. Validation non lancée. Statut : `ready-for-dev`. |
| 2026-11-19 | **Passe de validation P1** (Sonnet, contexte frais, `kesh-gate-logs/e491d/validation-49-1d-p1.md` : 0 CRITICAL, 0 HIGH, 6 MEDIUM, 4 LOW). **Remédiation** (Opus 5.5, sur les arbitrages de l'orchestrateur ; branche à jour d'`epic/49` par `d4f5bba7f`) : **V1-1** préconditions écrites à D1 et AC1 — compte bancaire principal (`invoice_pdf_service.rs:245`, `bank_accounts.rs:112`) et compte de différences d'arrondi (`invoices.rs:2324`) —, contrôle `HTTP 200` + en-tête `%PDF-` de chaque PDF (code 11) ; **V1-2** résolution fixée à 300 dpi, nombre de pages et code de `mutool` contrôlés, décompte **exact** de paires (5, ou 9 avec `--import`) dont le texte contient le numéro de la pièce, ce que « même dessin » prouve et ne prouve pas (D4), rouges réels de sensibilité R2 (projection), R3 (rendu, et F2 contre F4 sur les vrais PDF), R4 (gestes) ; **V1-3** dérogation de la 49-4 (`d4f5bba7f`, fichier d'épique `:130`, `:154`) dite à D8, AC14 et à la Note de version, phrase « comme en 0.19.0 » réduite aux désignations de 44 caractères au plus et conditionnée à la liste des stories fusionnées ; **V1-4** D11 et AC11 : script d'intégration `kesh-gate-logs/rel0200/recette-complement-rel0200.sh`, écrit et joué par la story, rejoué à la release — refus d'import d'une sauvegarde 0.20.0 par la 0.19.0, retour par `pre-upgrade/` dans une 0.19.0 neuve, reprise d'une migration interrompue avec le vrai binaire —, le manuel ne dit « constaté » que ce qui a été joué, la reprise précédée d'une sauvegarde et dite propre à cette migration ; **V1-5** second inventaire par le symptôme (lecteurs filtrés par `line_type`, commande et sortie), D10 et AC8 : rapport TVA et créances âgées dans `api`, validation de F3, avoir A2 sur F4 et règlement de F1 joués en phase 2 et en phase 4, extraits comparés en phase 6 ; **V1-6** `KESH_MUTOOL` posé à chaque scénario, journal du factice exigé, T11 et T13 réécrits pour que le rouge ne dépende pas du `mutool` de l'hôte. **LOW appliqués** : V1-7 F7 et A1 sous 44 caractères, option `--designation-longue` si la 49-4 n'entre pas ; V1-8 contrôles du crochet en `[ … ] \|\| exit 1`, compte d'avant lu au fichier d'empreinte, `'priced'` avec apostrophes ; V1-9 noms de colonnes entre apostrophes inverses, même type JSON exigé, A1 rendu après l'import, durée du test mesurée avant la revue ; V1-10 paragraphe « Compatibilité » : l'import compare aussi les colonnes, message exact. Onze décisions (D1 à D11), seize AC, seize tests, douze tâches. Estimation de charge écrite aux Dev Notes (cinq jours environ). Modèles : Opus (rédaction et remédiation), Sonnet (P1) ; la P2, complète, prend Opus en contexte frais. Statut : `ready-for-dev`. |
