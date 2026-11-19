# Story 49-1d : types de lignes de la facture client — la recette de montée avec factures réelles simulées, et le passage à la 0.20.0 au manuel d'administration (#1242)

Status: ready-for-dev

*Fiche rédigée le 2026-11-19 par l'agent de rédaction de la 49-1d (Opus 5.5), par le workflow `bmad-create-story`. Release
**0.20.0** (coupure jeudi 2026-11-26 au soir). **Aucune migration, aucun code de production** : la story touche l'outillage de
recette (`scripts/`), son test, `docs/testing.md` et le manuel d'administration FR. Code lu à la tête de la branche
`story/49-1d-types-de-lignes-recette-montee` = `f57f6ac80` (49-1a, avec la migration `20261118000002`, et 49-1b fusionnées dans
`epic/49-types-de-lignes-facture`). Sources : `epic-49-types-de-lignes-facture.md`, `release-0.20.0.md`, les fiches closes
`49-1a-types-de-lignes-migration-api.md` (§ *Transmis à la 49-1d*) et `49-1b-types-de-lignes-pdf.md`, le précédent de la 0.19.0
(`kesh-gate-logs/rel0190/recettes-rel0190.sh`, `recette-complement-rel0190.sh`). Les choix de cette fiche sont D1 à D9.
Validation : non lancée.*

**Issues** : `refs #1242` ; **aucun mot-clé de fermeture** dans un commit. #1242 ne se ferme pas avec cette story.

**Couplage de coupure** : **la 49-1a ne sort jamais sans la 49-1d** (fichier d'épique, § *Règles de sortie*) ; les deux sortent
ensemble, ou la 0.20.0 ne porte aucune migration. **La 49-1d se rejoue au dernier commit de code de la release** (AC10).

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

## Décisions

- **D1 — Le peuplement simule les factures réelles par l'API de la 0.19.0, sans un `INSERT`.** Huit factures client (ordre de
  création = identifiants 1 à 8 sur une base neuve, celui que relit l'AC1) :

  | | Statut | Lignes | PDF en 0.19.0 | Rôle |
  |---|---|---|---|---|
  | F1 | validée, impayée | 1 ligne, 8.10 %, **description de plus de 44 caractères** | **figé** | PDF archivé servi tel quel |
  | F2 | validée, réglée (règlement unitaire, comme aujourd'hui) | 1 ligne | non figé | rendu 0.20.0 = rendu 0.19.0 |
  | F3 | brouillon | 1 ligne | — | comme aujourd'hui |
  | F4 | validée, impayée | **3 lignes**, taux 8.10, 2.60 et **0.00** (le cas d'un non-assujetti), arrondi à 5 centimes non nul | non figé | la facture de Guy jamais téléchargée |
  | F5, F6 | validées, **réglées ensemble** (`invoice_group`) | 1 ligne chacune | F5 **figé**, F6 non figé | règlement groupé |
  | F7 | validée, **figée**, puis **avoir** total (A1) → `cancelled` | 2 lignes, deux taux | **figé avant l'avoir** | facture annulée au PDF archivé ; PDF de l'avoir |
  | F8 | validée, **figée** ; son fichier **retiré** de la copie des documents (D3) | 1 ligne | figé, fichier manquant | **regel** par la 0.20.0 |

  Les descriptions des factures **non figées** et de F8 restent **sous 44 caractères** (D8). Le règlement groupé passe par un
  relevé **saisi à la main** au jour de la recette (`POST /bank-imports/manual`, un crédit égal à la somme des TTC de F5 et F6)
  et `POST /reconciliation/accept` de type `invoice_group`, le compte bancaire lié au compte `1020` (créé s'il manque), comme le
  précédent de la 0.19.0. *Écartées* : (a) un relevé CAMT en fixture — une date figée dans un fichier pourrirait (la facture doit
  dater au plus tard du lendemain du paiement, `api-external.md:785`) ; (b) des `INSERT` SQL de forme 0.19.0 comme la T1 de la 49-1a — c'est déjà sa preuve
  au niveau de la base ; la recette, elle, joue **l'application publiée** ; (c) une facture par cas croisé (payée **et** figée,
  etc.) — huit factures couvrent les chemins du PDF (figé, non figé, annulé, regel) et des règlements (unitaire, groupé, avoir)
  sans doublon.
- **D2 — `--diff-attendu` n'admet plus, pour une table ou pour `api`, qu'un ajout.** Un fichier de table nommé par
  `--diff-attendu` peut différer, mais **sa projection sur les colonnes d'avant** — relevées à l'empreinte `avant` dans
  `information_schema.COLUMNS`, par `ORDINAL_POSITION` — doit rester identique au fichier d'avant ; pour `api`, chaque document
  JSON d'après, **réduit aux clés présentes au même chemin dans le document d'avant** (récursivement, tableaux élément par
  élément, longueur égale exigée), doit égaler celui d'avant. `migrations`, `min_required`, `users`, `audit_log` gardent leur sens
  (écart libre). Une colonne retirée ou renommée fait échouer la projection (erreur SQL) : la phase échoue, ce qui est voulu. La
  release se joue donc avec `--diff-attendu migrations,invoice_lines,credit_note_lines,api` (le fichier d'épique : « `--diff-attendu`
  des deux tables de lignes ») **sans rien perdre** de leur comparaison. *Écartées* : (a) nommer les deux tables sans projection
  (rien des lignes ne serait plus comparé : la recette serait verte sur une ligne altérée) ; (b) une option neuve
  (`--ajouts-seuls`) à côté de `--diff-attendu` — deux listes pour un même fichier, et le sens libre actuel de `--diff-attendu`
  pour une table n'a servi qu'à perdre des comparaisons (précédent 0.19.0) ; (c) la projection au crochet `--apres`, hors dépôt
  (rel0190) : elle disparaîtrait avec la release, et ne joue pas à l'import (phase 5). **Limite dite** : une release future qui
  **réécrit** une colonne existante (rétro-remplissage d'une valeur) verra la phase échouer et devra ajouter une option ; aucune
  ne le fait en 0.20.0 (la 49-1a est du DDL seul, D12 de sa fiche).
- **D3 — Les documents de l'image sont copiés par `docker cp`, sans montage.** À la fin de la phase 1, avant d'arrêter l'image,
  `docker cp "$CT_IMAGE:/tmp/documents/." "$D/documents/"` (le conteneur porte le jeton du run, vérifié comme pour `docker stop`) ;
  puis le fichier de F8 est retiré de `$D/documents`. L'invariant « aucun chemin de l'hôte » (`:385`) et son test (`test.sh:414`)
  **restent**. *Écartée* : un montage `-v "$D/documents:/tmp/documents"` — il brise l'invariant et son test, et l'image tourne en
  `root` (le `Dockerfile` ne pose aucun `USER`) : le binaire neuf, lancé par l'utilisateur, ne pourrait plus écrire dans un
  dossier créé par `root` (le gel de F2, F4, F6 échouerait).
- **D4 — Un PDF archivé se compare à l'octet ; un rendu se compare par son contenu dessiné.** (a) **Figé** (F1, F5, F7) : les
  octets servis par le binaire neuf égalent ceux téléchargés en phase 1, et leur SHA-256 égale `pdf_sha256` relu en base.
  (b) **Rendu** (F2, F4, F6, A1, et le regel de F8) : chaque page **tramée** par `mutool draw -F pnm` à résolution fixe a la même
  empreinte MD5 que le rendu de référence, et le **texte** (`mutool draw -F txt`) est égal — le texte sert à **lire** un écart,
  la trame à n'en laisser passer aucun (position, trait, QR). Ni le `/ID` ni la date du dictionnaire `Info`, qui changent à chaque
  rendu, ne sont dessinés. *Écartées* : (a) l'octet du rendu (le `/ID` change, `golden_test.rs:3-4`) ; (b) l'empreinte de la 49-1b
  (`kesh-qrbill`, feature `test-support`) — aucun binaire ne l'expose au shell, et en ajouter un serait du code Rust pour un script ;
  (c) `pdftotext` seul (ne voit ni position, ni trait, ni QR) ; (d) les octets après effacement du `/ID` et des dates (printpdf y
  mêle un identifiant d'instance aléatoire, `crates/kesh-qrbill/src/pdf.rs:55-56` « *modulo* printpdf's internal random /
  instance id » : fragile).
- **D5 — La référence d'un rendu 0.19.0 se prend en phase 4, sur la base restaurée.** Télécharger F2, F4, F6 en phase 1 les
  figerait dans la base d'avant : l'empreinte et la sauvegarde ne seraient plus celles d'une base où ces factures ne sont jamais
  téléchargées. La phase 4 restaure justement la base d'avant sous l'image 0.19.0 : **après** son empreinte (inchangée), elle
  rend F2, F4, F6 et l'avoir A1 — la référence 0.19.0. La phase 2 (binaire neuf) les rend **après** son empreinte et le crochet
  (le gel modifie `invoices`). La référence du regel de F8 est son fichier figé de la phase 1. Une **phase 6 « Rendus »** (code
  **16**) compare, après les phases 4 et 5 : 0.20.0 (phase 2, et phase 5 si `--import`) contre 0.19.0. *Écartée* : relancer
  l'image en phase 1 après le dump pour prendre les références puis restaurer (un cycle de plus, même preuve).
- **D6 — La phase 5 importe dans un dossier de documents neuf.** `lancer_binaire` reçoit le dossier des documents en argument ;
  l'import écrit dans `$D/documents-import`, vide : un PDF figé servi après l'import vient **de l'archive**, non de la copie de la
  phase 2. F1, F5, F7 **et F8** (dont l'archive a gardé le fichier) y sont servis à l'octet ; F2, F4, F6 y sont rendus et comparés
  en phase 6. *Écartée* : réutiliser `$D/documents` (un PDF servi n'y prouverait rien de la sauvegarde).
- **D7 — Ce qui ne vaut que pour la 0.20.0 va au crochet `--apres`**, écrit sous `kesh-gate-logs/rel0200/verifs-0200.sh` par
  cette story et relu en revue comme du code (`docs/testing.md`, § *`--apres*) : toutes les lignes des deux tables sont `priced`
  (`SELECT COUNT(*) FROM invoice_lines WHERE line_type <> 'priced'` = 0, de même pour `credit_note_lines`), leur nombre égale celui
  d'avant, `COLUMN_DEFAULT` vaut `'priced'` sur les deux tables. Le script, lui, ne nomme ni `line_type` ni la 0.20.0. *Écartée* :
  ces contrôles dans le script (il deviendrait propre à une release).
- **D8 — Les descriptions des factures rendues de nouveau restent sous 44 caractères.** La 49-4 (`sprint-status.yaml`, ajoutée par
  Guy le 2026-11-19 : #1363, #1364) peut changer le rendu d'une description longue, avec « dérogation assumée » pour une facture
  jamais téléchargée ou un regel. La recette compare donc à l'octet ce qui ne doit **jamais** changer (F1, longue, figée) et ne
  pose de description longue sur aucune facture rendue de nouveau : si la 49-4 entre dans la release, sa dérogation se prouve à sa
  propre fiche, non par un faux rouge de cette recette. Tout **autre** écart de rendu fait échouer la phase 6, quelle que soit la
  story qui l'a causé (48-15, #1327 de Beat, 49-3…), et revient à l'orchestrateur.
- **D9 — `mutool` est un outil de la recette, remplaçable au test par `KESH_MUTOOL`.** Ajouté à la liste des outils (`:123`,
  absent → code 5) avec `sha256sum` ; `KESH_MUTOOL` traverse la relance comme `KESH_DOCKER` et `KESH_CURL` (`:65`, `:81-83`), et
  l'enveloppe Rust le retire (`.env_remove`) comme elles. *Motif* : la CI n'installe pas `mutool`
  (`grep -n 'apt-get' .github/workflows/ci.yml` : `mold`, `mariadb-client` seulement) ; le test du script tourne en CI avec un
  `mutool` factice. *Écartée* : sauter la phase 6 si `mutool` manque (une recette verte sans comparaison des rendus).

## Critères d'acceptation

**Le script (`scripts/recette-montee.sh`)**

- **AC1 — Peuplement (D1).** La phase 1 crée, par l'API de l'image, les huit factures, l'avoir A1 et le règlement groupé de D1, et
  télécharge le PDF de F1, F5, F7 et F8 **avant** l'avoir (F7 se fige validée). Contrôles relus en base, chacun une sortie en
  code 11 : statuts `validated,validated,draft,validated,validated,validated,cancelled,validated` ; `invoice_settlements` = 3
  (F2, F5, F6), dont 2 portant `journal_entry_line_id` ; `credit_notes` = 1 ; `invoices.pdf_sha256 IS NOT NULL` pour exactement
  F1, F5, F7, F8 ; F4 a trois lignes, trois taux distincts dont `0.00`, et `rounding_amount <> 0`. Les factures fournisseur et le
  relevé A restent tels quels. Aucune donnée réelle : contacts, IBAN et libellés synthétiques (AC-7 du test inchangé).
- **AC2 — Empreinte.** `TABLES` gagne `credit_notes` et `credit_note_lines` ; `api` gagne `GET /api/v1/credit-notes/{id}` de
  chaque avoir peuplé. `docs/testing.md` liste exactement ce que l'empreinte couvre.
- **AC3 — `--diff-attendu` par ajout (D2).** Pour une table ou `api` nommés, la projection sur l'avant est identique, sinon la
  phase échoue (code 12 à la montée, 15 à l'import) en nommant le fichier et en écrivant le diff de la projection au journal. Les
  autres noms gardent l'écart libre. La phase 4 (retour) reste à l'égalité **stricte** de tous les fichiers.
- **AC4 — Documents et PDF archivés (D3, D4 a).** Phase 1 : copie par `docker cp` du dossier des documents de l'image (conteneur
  du jeton seulement), puis retrait du fichier de F8 ; aucun montage (`test.sh:414` reste vert). Phase 2, **après** l'empreinte
  et le crochet : F1, F5, F7 servis par le binaire neuf, octets égaux à ceux de la phase 1 (SHA-256 égal aussi à `pdf_sha256`) ;
  F8 répond `410` puis son regel (`POST …/pdf/refreeze`) répond `200` ; F2, F4, F6 et A1 rendus et gardés pour la phase 6.
- **AC5 — Rendus (D4 b, D5) : phase 6, code 16.** Après la phase 4 (qui rend F2, F4, F6, A1 sous l'image 0.19.0, **après** son
  empreinte) : chaque rendu 0.20.0 (F2, F4, F6, A1, regel de F8 ; et, avec `--import`, F2, F4, F6 rendus après l'import) a, page à
  page, la même trame et le même texte que sa référence 0.19.0 (pour F8 : son fichier figé de la phase 1). Un écart nomme la pièce
  et écrit le diff du texte au journal. **Témoin** : le nombre de paires comparées s'écrit, et **zéro paire échoue** (`echec 6
  "aucun rendu comparé"`) — jamais un vert à vide.
- **AC6 — Import (D6).** Avec `--import`, la phase 5 importe dans `$D/documents-import` (neuf) ; F1, F5, F7, F8 y sont servis à
  l'octet de la phase 1 ; l'empreinte suit AC3.
- **AC7 — Outils (D9).** `mutool` et `sha256sum` sont exigés (code 5 s'ils manquent) ; `KESH_MUTOOL` traverse la relance et
  rien d'autre de neuf ; l'en-tête du script (`:10-19`, phases et codes) et `:38-39` disent la phase 6 et ce qui est désormais
  comparé.

**Les factures réelles (consigne de production)**

- **AC8 — Les factures et avoirs existants restent identiques, et la recette le prouve sur l'image publiée.** Après la montée de
  0.19.0 à la tête de la story : lignes (toutes les colonnes d'avant), totaux, arrondi, TVA par ligne, écritures, numéros,
  règlements (unitaires et groupé), avoir et ses lignes — **identiques** (AC3) ; PDF archivés servis à l'octet (AC4) ; PDF rendus
  de nouveau, pour une facture jamais téléchargée, un regel et un avoir, au contenu dessiné identique (AC5) ; la même chose après
  l'import d'une sauvegarde 0.19.0 (AC6).

**Le jeu réel**

- **AC9 — Recette réelle, verte, à la tête de la story.** Sous le verrou de la voie base et `mem-guard`, depuis le worktree :
  ```sh
  flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh scripts/recette-montee.sh \
    --image gcorbaz/kesh:0.19.0 --binaire target/debug/kesh-api --static frontend/build \
    --port-api 3197 --port-base 3398 --migrations-attendues 1 \
    --diff-attendu migrations,invoice_lines,credit_note_lines,api \
    --apres ../kesh-gate-logs/rel0200/verifs-0200.sh --import --prefixe kesh-e491d
  ```
  → code 0, six phases jouées, la phase 3 « refusée : migration inconnue d'elle (sqlx) » (`20261118000002`), le témoin de la
  phase 6 non nul ; sortie et dossier du journal copiés au Dev Agent Record (`kesh-gate-logs/e491d/recette-reelle.log`). **Et son
  rouge réel** : la même commande avec `--diff-attendu migrations` seulement échoue en phase 2, code 12, en nommant
  `invoice_lines`, `credit_note_lines` et `api` (`recette-reelle-rouge.log`) — preuve que l'empreinte voit bien la migration.
- **AC10 — Rejeu au dernier commit de code de la release.** La fiche et `docs/testing.md` disent que l'intégration de la 0.20.0
  rejoue l'AC9 au dernier commit de code de la release (toutes stories fusionnées), journal sous `kesh-gate-logs/rel0200/`, et
  que la section du manuel (AC12) ne dit « constaté » que ce que ce rejeu a établi. Le crochet `verifs-0200.sh` existe et se relit
  en revue (D7).

**Documentation**

- **AC11 — `docs/testing.md`** (§ *La recette de montée et de retour*) : exemple d'appel à jour ; sens de `--diff-attendu` (D2) ;
  empreinte (AC2) ; documents et PDF (« Aucun document n'est peuplé ni comparé » retiré, remplacé par ce qui l'est et ce qui ne
  l'est pas) ; le peuplement et ses contrôles (AC1) ; la phase 6 et le code 16 ; `mutool`, `sha256sum`, `KESH_MUTOOL` ; la
  nouvelle durée du test ; le rejeu au dernier commit de code (AC10).
- **AC12 — Manuel d'administration FR, section « Passer à la 0.20.0 »** (`docs/manual/fr/admin-manual.tex`, `.tex` seulement,
  sur le modèle de `:2564-2586`), **en tête** : « **Prenez avant la mise à jour une sauvegarde manuelle complète** — la base, un
  `.keshbackup` et le dossier des documents » et « le retour à la 0.19.0 passe uniquement par la sauvegarde d'avant la mise à
  jour » ; puis l'introduction (« La 0.20.0 se pose sur la 0.19.0 », ni compose ni variable neuve) ; **« Une migration »**
  (`20261118000002_invoice\allowbreak\_line\allowbreak\_types`, 96 au total, non breaking, `kesh_version_min_required` reste à
  `0.17.6`, aucune ligne existante modifiée — chacune reçoit le type « ligne chiffrée » —, sauvegarde `pre-upgrade/` au nom finissant
  par `-0.19.0-to-0.20.0.keshbackup`, `KESH_SKIP_PRE_UPGRADE_BACKUP=0.20.0`) ; **« Le retour à la 0.19.0 »** (la 0.19.0 ne démarre
  plus sur la base migrée — journal « migration 20261118000002 was previously applied but is missing in the resolved migrations » ;
  elle refuse une sauvegarde 0.20.0 avec le message exact d'`IMPORT_SCHEMA_MISMATCH`, **distinct** d'`INVALID_BACKUP_STRUCTURE` ;
  retour sur une base neuve par la sauvegarde d'avant la mise à jour ; une sauvegarde 0.19.0 s'importe dans la 0.20.0) ; **« Si la
  migration échoue au milieu »** (la procédure exacte de `line_types_migration_resumes_after_a_half_run` : Kesh refuse de démarrer,
  migration *dirty* ; retirer la ligne `success = 0` de la version `20261118000002` ; redémarrer ; le rejeu termine sans effet sur
  ce qui était posé, y compris si le premier `ALTER` a échoué entièrement ; la sauvegarde d'avant la mise à jour reste le recours) ;
  **« Ce qu'il faut savoir »** (factures et avoirs existants inchangés, PDF archivé servi tel quel, facture jamais téléchargée et
  avoir rendus comme en 0.19.0 — avec ce que la recette a constaté, AC10) ; le paragraphe « Pour les intégrations » de la 49-1a
  **gardé** ; le commentaire `:2551-2555` retiré ou réécrit (il ne décrit plus une section à compléter). Le paragraphe
  « Compatibilité » (`:2302`) gagne la phrase de la 0.20.0 (aucune table neuve ; une sauvegarde 0.19.0 s'importe, ses lignes
  devenant chiffrées ; l'inverse est refusé avec `IMPORT_SCHEMA_MISMATCH`, \S\ref{sec:maj-0-20-0}). Le manuel compile (`make fr`
  vert, `Overfull \hbox` relevés), **PDF non commité**.

**Périmètre et gates**

- **AC13 — Aucun code de production.** `git diff --stat origin/epic/49-types-de-lignes-facture..HEAD -- crates frontend` ne nomme
  que `crates/kesh-db/tests/recette_montee_script.rs` (une ligne `.env_remove("KESH_MUTOOL")`, et la borne de durée si la mesure
  l'exige) ; aucune migration (`ls crates/kesh-db/migrations/*.sql | wc -l` = 96).
- **AC14 — Gates.** Pendant la boucle : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo nextest run -E 'binary(recette_montee_script)'` (gate ciblé) et `shellcheck` sur les deux scripts s'il est installé. Au
  dernier commit de code : bases `kesh_e491d`, `kesh_ci_e491d`, `kesh_e2e_e491d` remises à neuf ; `scripts/gate-ci.sh` sur
  `kesh_ci_e491d` vert ; **E2E Playwright complet** « vert » au sens de `docs/testing.md` § *Les échecs attendus* (règle du
  `CLAUDE.md` : la story touche du code exécutable) ; recette réelle (AC9) rejouée à ce commit. Recette visuelle : sans objet
  (aucun contrôle visible).

## Tests à écrire, et la preuve du rouge

Tous dans `scripts/tests/recette-montee.test.sh` (factices : `docker`, `curl`, binaire, et désormais `mutool`), lancé au gate par
`crates/kesh-db/tests/recette_montee_script.rs`. Chaque preuve du rouge est une **mutation du script** (`scripts/recette-montee.sh`
du mini-dépôt, copie modifiée), le test relancé, le journal gardé sous `kesh-gate-logs/e491d/mut/rouge-T<n>.log`, puis la
mutation retirée. Les factices évoluent : le `docker` factice connaît `cp` (copie un PDF factice dans la cible), les requêtes de
colonnes (`information_schema`), les projections (`SELECT <colonnes> FROM t`) et les nouveaux décomptes ; le `curl` factice sert
`/pdf`, `/pdf/refreeze` (410 puis 200), `/credit-notes`, `/reconciliation/accept`, `/bank-imports/manual` ; le `mutool` factice
écrit une trame et un texte qui dépendent d'un réglage (`RENDU_ECART=oui` sur les rendus de la phase 2).

| # | Scénario | Ce qu'il prouve | Preuve du rouge |
|---|---|---|---|
| T1 | nominal, `--diff-attendu migrations,invoice_lines,credit_note_lines,api`, `ECART=invoice_lines` (le factice ajoute une colonne : `SELECT *` change, la projection non) | code 0, six phases | — (témoin des autres) |
| T2 | `ECART_ANCIEN=invoice_lines` (une colonne d'avant change) avec le même `--diff-attendu` | code 12, sortie nomme `invoice_lines` (AC3) | retirer la comparaison de projection → code 0 : **rouge** (`rouge-T2.log`) |
| T3 | `ECART_ANCIEN=api` (une clé d'avant change) / `ECART=api` (une clé s'ajoute) | 12 / 0 (AC3) | projection `api` remplacée par `true` → T3a rend 0 : **rouge** |
| T4 | `ECART_ANCIEN=credit_note_lines` **sans** la nommer | code 12 (elle est dans `TABLES`, AC2) | retirer `credit_note_lines` de `TABLES` → code 0 : **rouge** |
| T5 | `PDF_FIGE_ALTERE=oui` (le `curl` factice sert d'autres octets en phase 2) | code 12, nomme la facture (AC4) | retirer la comparaison des octets → **rouge** |
| T6 | `RENDU_ECART=oui` | code 16, nomme la pièce (AC5) | retirer la phase 6 → code 0 : **rouge** |
| T7 | aucune pièce rendue (le factice rend une liste vide) | code 16, « aucun rendu comparé » (témoin) | retirer le témoin → code 0 : **rouge** |
| T8 | `REGEL_ECHEC=oui` (refreeze `409`) | code 12 (AC4) | ignorer le statut du regel → **rouge** |
| T9 | nominal : `docker cp` appelé sur le conteneur du jeton, vers `$D/documents` ; `test.sh:414` (aucun montage) **inchangé et vert** | D3 | remplacer `docker cp` par un `-v` → l'assertion `:414` **rougit** |
| T10 | `STATUTS_CLIENTS=` autre / `NB_AVOIRS=0` | code 11 (AC1) | retirer le contrôle → **rouge** |
| T11 | `KESH_MUTOOL` vers un chemin absent | code 5, zéro appel `docker` (AC7) | retirer `mutool` de la liste des outils → **rouge** |
| T12 | `--import` nominal ; `PDF_IMPORT_ALTERE=oui` | 0 ; 15 (AC6) | importer dans `$D/documents` au lieu du dossier neuf : le factice le voit (il consigne le `KESH_DOCUMENTS_DIR` reçu) → **rouge** |
| T13 | la relance : `KESH_MUTOOL` traverse, une variable `KESH_AUTRE` ne traverse pas | D9 | retirer `KESH_MUTOOL` de la liste blanche → **rouge** (code 5 après relance) |

**Le jeu réel** (AC9) est l'épreuve de bout en bout ; son rouge réel est `--diff-attendu migrations` (code 12 nommant les trois
fichiers). **E2E Playwright neuf** : aucun — la story ne change ni écran ni réponse HTTP ; la suite complète tourne tout de même
au dernier commit de code (AC14).

## Tâches

- [ ] **T0 — Sonde, avant tout changement** (journal `kesh-gate-logs/e491d/t0/`) : `docker image inspect gcorbaz/kesh:0.19.0`
  (tirer si absente) ; `command -v mutool sha256sum` ; jouer **à la main**, contre une image 0.19.0 jetable (préfixe
  `kesh-e491dt0`, base `mariadb:10.11` en tmpfs), les gestes de D1 que le script ne fait pas encore : création d'un `1020` si
  absent et lien du compte bancaire, relevé saisi au jour, `invoice_group` accepté, avoir, téléchargement d'un PDF (le fichier
  apparaît-il sous `/tmp/documents` du conteneur ? `docker cp` le rend-il lisible par l'utilisateur ?), le taux `0.00` accepté à la
  validation, un `410` puis un regel ; noter au journal si un `410` ou un regel écrit une ligne ` ERROR ` (sinon `journal_sans_erreur`
  échouerait). Toute surprise s'écrit au Dev Agent Record **avant** d'écrire le script ; une prémisse fausse de D1 à D6 arrête la
  story et se dit.
- [ ] **T1 — Peuplement** (AC1) : les huit factures, l'avoir, le groupe, les contrôles.
- [ ] **T2 — Empreinte et projection** (AC2, AC3) : relevé des colonnes à `avant`, projections, `api` par clés, diff au journal.
- [ ] **T3 — Documents et PDF** (AC4, AC6) : `docker cp`, retrait de F8, phase 2 (servis, 410, regel, rendus), phase 5 (dossier
  neuf).
- [ ] **T4 — Phase 4 et phase 6** (AC5) : rendus 0.19.0 après l'empreinte du retour, comparaison trame + texte, témoin, code 16.
- [ ] **T5 — Outils et relance** (AC7, D9) ; enveloppe Rust (`.env_remove("KESH_MUTOOL")`).
- [ ] **T6 — Test shell** (T1 à T13) et mutations ; durée mesurée ; borne de `recette_montee_script.rs` relevée seulement si la
  mesure l'exige (dire la valeur).
- [ ] **T7 — Crochet** `kesh-gate-logs/rel0200/verifs-0200.sh` (D7), son texte copié au Dev Agent Record.
- [ ] **T8 — Recette réelle** (AC9) et son rouge réel.
- [ ] **T9 — `docs/testing.md`** (AC11) ; grep du symptôme « aucun document » (`git grep -n -i -e "aucun document n'est peuplé" -e
  "vivent dans son" -- scripts docs crates`) : tous les sites traités.
- [ ] **T10 — Manuel d'administration** (AC12) ; `make fr` sous `mem-guard`, puis `git checkout -- docs/manual/fr/*.pdf` (PDF jamais
  commités) ; vérifier le texte rendu (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`).
- [ ] **T11 — Gates** (AC14) et Dev Agent Record (ce qui a tourné, et seulement cela).

## Note de version

*(Pour le CHANGELOG, assemblée à l'intégration ; la story n'écrit pas `CHANGELOG.md`.)*

> Le manuel d'administration décrit le passage à la 0.20.0 : la sauvegarde complète à prendre avant la mise à jour (base,
> `.keshbackup` et dossier des documents), le retour à la 0.19.0 et la reprise d'une migration interrompue. La montée a été
> éprouvée, depuis l'image publiée 0.19.0, sur une base de factures simulées — payées, impayées, réglées en groupe, annulées par un
> avoir, au PDF archivé ou jamais téléchargé — : chacune reste identique, PDF compris. (refs #1242)

## Dev Notes

- **Hors périmètre** : (a) la procédure de retour **du manuel** (sauvegarde `pre-upgrade/` importée dans une 0.19.0 neuve) reste
  #920, recette distincte ; la phase 4 revient par le dump, et le manuel ne dit « constaté » que ce qu'une recette a joué ; (b) le
  refus d'une sauvegarde 0.20.0 par une 0.19.0 est établi **au code** (v0.19.0, `import.rs:615-633`) ; l'intégration le rejoue
  comme pour la 0.19.0 (`kesh-gate-logs/rel0190/recette-refus-import-rel0190.sh`), et la phrase du manuel se règle sur ce rejeu ;
  (c) rappels, séquences de numérotation : non peuplés ni comparés (inventaire) ; (d) la dérogation de la 49-4 (D8).
- **Issues à ouvrir** (demandées à l'orchestrateur, une par problème) : (1) `technical-debt`, P3 — *la recette de montée ne
  peuple ni ne compare les rappels (`invoice_reminders`) ni les séquences de numérotation*.
- **Écarts au fichier d'épique, déclarés** : (a) le tableau des stories écrit « manuel d'administration (×4) » ; le tableau des
  fichiers et la 49-1a (V3-1) disent **FR seul** : les manuels DE, EN, IT n'existent pas — FR seul ici ; (b) `--diff-attendu`
  nomme aussi `api` (la réponse gagne `lineType`), et n'admet qu'un ajout (D2) ; (c) fichiers touchés hors du tableau :
  `scripts/tests/recette-montee.test.sh` (le test du script) et `crates/kesh-db/tests/recette_montee_script.rs` (une ligne), plus le
  crochet hors dépôt `kesh-gate-logs/rel0200/verifs-0200.sh`. Modules : `scripts/`, `docs/` (testing, manuel), `kesh-db` (test
  seulement) — **trois**, sous le seuil de splitting.
- **Fichiers communs** : `admin-manual.tex` — la 49-1a a posé le squelette (fusionnée) ; la story de menu
  (`v0200-menu-sous-groupes`) fusionne en dernier et change des chemins du manuel ; les PDF ne se commitent pas.
  `scripts/recette-montee.sh` n'est touché par aucune autre story de la release. `sprint-status.yaml` : bloc de l'Epic 49.
- **Ce qui doit rester vrai** : le périmètre fermé du script (relance `env -i`, `HOME` neutralisé, démon local seul, écriture sous
  `$D` seulement, aucun chemin de la station, aucun montage, conteneurs du jeton seuls) ; les codes 2 à 5 et 11 à 15 ; la phase 4 à
  l'égalité stricte ; jamais `… | grep -q` sous `pipefail` (`:138-139`). **Jamais la base du NAS, jamais `~/kesh/`** : tout se joue
  dans le conteneur de recette ; aucune donnée réelle de Guy dans le script, le crochet, les journaux ou la fiche.
- **Station** : une commande lourde à la fois (construction de `kesh-api` et du frontend, recette réelle, `gate-ci`, E2E), sous
  `flock` du verrou de la voie base et `scripts/mem-guard.sh`, `CARGO_BUILD_JOBS=2`, `scripts/mem-guard.sh --status` avant chacune.
  La recette refuse si un conteneur `kesh.recette` d'un autre run existe (code 4) : attendre, ne jamais arrêter le conteneur d'un
  autre. Ports `3197`/`3398` (et `3399` pour l'import), préfixe `kesh-e491d`.
- **Rejeu** : la recette est le gate de la **release** ; elle se rejoue au dernier commit de code de la 0.20.0 (AC10), après toutes
  les fusions (48-*, 49-*, menu, #1327 si fusionnée). Un rouge de la phase 6 à ce moment signale un changement de rendu d'une
  facture existante : il ne se tolère pas, il va à l'orchestrateur (D8).
- **Références** : `epic-49-types-de-lignes-facture.md` (§ *Contraintes de production*, point 4 ; § *49-1*), `release-0.20.0.md`
  (§ *Gates de la release*), fiches 49-1a (§ *Transmis à la 49-1d*) et 49-1b (D4), `docs/testing.md` § *La recette de montée*,
  `kesh-gate-logs/rel0190/recette-complement-rel0190.sh`, `CLAUDE.md` § *Ce qu'impose la production*, § *Documentation à
  synchroniser* (point 7), #1242, #920, #925.

## Dev Agent Record

*(vide)*

## Change Log

| Date | Entrée |
|---|---|
| 2026-11-19 | Création de la fiche (workflow `bmad-create-story`), par l'agent de rédaction de la 49-1d (Opus 5.5) ; code lu à `f57f6ac80`. Neuf décisions (D1 à D9), quatorze AC, treize tests. Points arrêtés : peuplement par l'API de la 0.19.0 (huit factures, un avoir, un règlement groupé, quatre PDF figés dont un au fichier retiré) ; `--diff-attendu` réduit aux ajouts (projection sur les colonnes et les clés d'avant) ; documents copiés par `docker cp` sans montage ; PDF archivés à l'octet, rendus par trame et texte (`mutool`), référence 0.19.0 prise en phase 4, phase 6 « Rendus » (code 16) ; crochet de la release hors dépôt. Écarts au fichier d'épique déclarés (manuel FR seul, `api` nommé, test du script et enveloppe touchés). Une issue à ouvrir. Validation non lancée. Statut : `ready-for-dev`. |
