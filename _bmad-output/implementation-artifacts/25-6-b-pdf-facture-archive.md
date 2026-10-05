# Story 25.6-b : Le PDF d'une facture est figé — une pièce émise ne change plus

Status: review

**Issue : [#387]**, que cette story **ferme** : la PR porte `closes #387` dans le **titre ET le corps**. Branche
`story/25-6-b-pdf-facture-archive`, partie de `main` (`d7c74f02`). Seconde moitié de la 25-6 (la 25-6-a a traité #388 et
#389).

## Arbitrages (Guy — 1 à 6 le 2026-10-03, 7 et 8 le 2026-10-04)

1. **Quand figer** : au **premier rendu après validation** (premier téléchargement OU premier envoi par e-mail) — pas
   dans la transaction de validation. Les factures déjà validées sont figées à leur prochain rendu, avec les données du
   moment (leurs PDF passés n'ont jamais été conservés).
2. **Langue** : celle du **client** (`resolve_language` : `contact.language`, sinon la langue d'instance), la règle de
   l'e-mail. Le téléchargement rend désormais **le document envoyé**.
3. **Stockage** : des **colonnes nullables sur `invoices`**, pas de nouvelle table (une table neuve rendrait toutes les
   sauvegardes existantes non importables, `post_restore.rs:20-36`).
4. **Dévalidation** : le PDF figé est **détaché** ; son empreinte va au journal d'audit. (Elle reste refusée pour une
   facture envoyée par e-mail — inchangé.)
5. **Facture annulée par un avoir** : son PDF figé, s'il existe, reste servi.
6. **Empreinte** SHA-256 au journal d'audit au gel, **vérifiée à chaque lecture**.

7. **(2026-10-04, validation P1)** **Refiger, geste d'administrateur** : un PDF figé dont le fichier a disparu (le cas
   de toute restauration tant que [#503] n'est pas livrée) peut être **refigé** par un administrateur — nouveau document,
   ancienne et nouvelle empreinte au journal d'audit. Sans ce geste, une facture envoyée dont le fichier manque serait
   sans issue : la dévalidation est refusée, et rien ne refige.
8. **(2026-10-04, validation P1)** **Le gel par tout rôle** : le premier téléchargement fige, que l'acteur soit
   Consultation ou une clé d'API en lecture — une pièce sortie de Kesh est figée, qui que ce soit qui l'ait fait sortir ;
   l'audit nomme l'acteur.

**Hors périmètre, en CR** : figer les PDF d'avoir et de rappel ([#502]) ; emporter les fichiers dans `.keshbackup`
([#503]).

## Story

En tant que **personne qui émet des factures dans Kesh**,
je veux que **le PDF d'une facture, une fois émis, ne change plus** — même si je modifie ensuite mes coordonnées, celles
du client ou la mise en page,
afin de **pouvoir produire, dans cinq ans, la copie exacte de la facture envoyée**, ce qu'un contrôle demande.

## Les faits, vérifiés dans le code (cartographie du 2026-10-03)

- **Régénéré à chaque demande** : `GET /api/v1/invoices/{id}/pdf` (`crates/kesh-api/src/routes/invoice_pdf.rs:23-56`)
  appelle `invoice_pdf_service::render(…, state.config.locale, …)` ; `render` (`crates/kesh-api/src/routes/invoice_pdf_service.rs:177-269`) relit
  **en direct** la facture, le contact (nom, adresse, `client_number`), le compte bancaire principal, la société, les
  textes i18n. Il exige le statut `validated` (`:211-213`).
- **Deux langues** : le téléchargement rend dans la langue de l'**installation** ; l'envoi par e-mail
  (`invoice_email.rs:732`, rendu `:785-787`, pièce jointe `:797-801`) dans celle du **client** (`resolve_language`,
  `:69-71`). Aujourd'hui, le PDF téléchargé n'est donc pas forcément celui qui a été envoyé.
- **Non reproductible** : `kesh-qrbill` inscrit `now_utc()` comme date de création (`crates/kesh-qrbill/src/pdf.rs:28-35`,
  `:78-83`) et printpdf 0.7 un `/ID` aléatoire. Deux rendus identiques n'ont pas le même SHA-256 : **seul un fichier figé
  une fois fait preuve**.
- **Le stockage réutilisable** : `crates/kesh-api/src/document_storage.rs` — `store_document(dir, bytes, ext,
  original_filename, mime_type) -> io::Result<DocumentMeta>` (`:84-143`), nommage `{sha256}.{ext}`, écriture atomique
  (temporaire + `sync_all` + `rename`), refus d'un contenu vide, extension contrôlée ; `read_document(dir, storage_path)`
  (`:154-167`) contrôle le chemin et distingue `NotFound` / `InvalidPath` / `Io`. Répertoire : `KESH_DOCUMENTS_DIR`
  (`config.rs:253-255`, défaut `/data/documents`). Patron de service HTTP : `serve_document`
  (`routes/imported_supplier_invoices.rs:415-445`) — fichier absent → **410** `SourceDocumentGone` (`errors.rs:914`).
- **Cycle de vie** : validation `validate_invoice` (`kesh-db/src/repositories/invoices.rs:1836`, commit `:2205-2208`) —
  dans **kesh-db**, alors que le rendu est dans **kesh-api** ; dévalidation `unvalidate` (`invoices.rs:1441`, refus si
  envoyée par e-mail `:1529-1541`, `UPDATE … status='draft'` `:1576-1578`, audit `:1602`) ; l'avoir passe la facture en
  `cancelled` (`credit_notes.rs:583-586`) — **son PDF n'est plus servi** (`render` refuse).
- **Frontend** : bouton PDF de la fiche facture affiché seulement si `validated` (`invoices/[id]/+page.svelte:756-767`,
  `downloadPdf` `:671-675`).
- ⚠️ **La chaîne 25-4 (non mergée) touche les mêmes lignes** (validation P2, M6) : elle ajoute `rounding_amount` à
  `Invoice`, à `INVOICE_COLUMNS`, à `FIND_INVOICE_SCOPED_SQL`, aux `SELECT` en dur de `invoices.rs`, à l'en-tête CSV et
  à son test, et modifie l'`UPDATE` de `unvalidate`. Conflits textuels attendus au merge ; la constante unique de
  colonnes (AC 1-bis) inclura `rounding_amount` si la chaîne est mergée la première, et les références de lignes
  ci-dessous se décaleront.
- **Migrations** : 70 sur `main`. ⚠️ La chaîne 25-4 (non mergée) en ajoute quatre, datées `20261001000001`–`…04` : la
  migration de cette story est datée **`20261003000001`** pour s'ordonner après elles ; les compteurs (`upgrade_path`,
  audit d'idempotence) se recomposeront au merge.

## Acceptance Criteria

**AC 1 — Le schéma.** Migration `20261003000001_invoices_frozen_pdf.sql` : `ALTER TABLE invoices ADD COLUMN`
`pdf_storage_path VARCHAR(512) NULL`, `pdf_sha256 CHAR(64) NULL`, `pdf_frozen_at DATETIME(3) NULL`,
`pdf_language CHAR(2) NULL`, et une contrainte **tout-ou-rien** (`CHECK` : les quatre nuls, ou les quatre renseignés avec
`CHAR_LENGTH(pdf_sha256) = 64`). DDL pur, **non breaking** (`ADD COLUMN` nullable — P1/P3, pas de bump de
`kesh_version_min_required`). Squash de test **régénéré par `scripts/regen-test-schema.sh`**, jamais édité à la main
(`crates/kesh-db/test-schema/README.md`) — sans quoi `the_real_migrator_matches_the_migrations_directory`
(`test_schema_guard.rs:497`) rougit ; `migrations_upgrade_path.rs:100-104` (`assert_eq!(total, 70)`, fenêtre positionnelle
`total - 36`, `:38`, `:57`) relevé — **70 → 71** sur cette branche, **74 → 75** si la chaîne 25-4 est mergée d'abord ; ligne au
tableau `docs/migrations-idempotence-audit.md` et compteurs **recomptés depuis la source** (P5) ; sites positionnels
inspectés (P6, `grep -rn "migrations.len()\|apply_migrations_up_to" crates/`) ; pas d'écriture de données (P7 sans
objet).

**AC 1-bis — L'entité et ses colonnes (validation P1, B1).** `Invoice` (`entities/invoice.rs:15`, `FromRow`) gagne les
quatre champs. Ses colonnes sont aujourd'hui **énumérées en dur** — `FIND_INVOICE_SCOPED_SQL`
(`invoices.rs:52-56`), les `SELECT` de `invoices.rs:1304-1308` et `:2511-2516`, `INVOICE_COLUMNS`
(`reconciliation.rs:49-51`) — et un oubli échoue **à l'exécution**, pas à la compilation (`invoices.rs:277`). Ces listes —
**quatre** sites : `invoices.rs:54`, `:1306`, `:2513`, `reconciliation.rs:50` (`grep -rnE "emailed_at, *emailed_to" crates`)
— sont ramenées à **une seule constante** de colonnes (DRY), et les littéraux `Invoice { … }` des tests complétés
(`invoice_email.rs:1452`, `exports/csv_tables.rs:1665`, `invoice_pdf_service.rs:633`, `grep -rn "dunning_paused_note:"
crates`). `invoice_snapshot_json` (`invoices.rs:58`) porte l'empreinte.

**AC 2 — Figer au premier rendu.** Un service unique `kesh-api` (ex. `issued_invoice_pdf::get_or_freeze`) est le **seul**
chemin vers le PDF d'une facture, pour le téléchargement **et** pour l'e-mail :
- facture **déjà figée** → relit le fichier, **vérifie** son SHA-256 contre `pdf_sha256`, rend ces octets ;
- facture `validated` **non figée** → rend le PDF dans la **langue du client** (`resolve_language`), l'écrit par
  `store_document` (`ext = "pdf"`), puis pose les quatre colonnes par une fonction `kesh-db` qui fait, **dans une seule
  transaction**, l'`UPDATE … WHERE id = ? AND company_id = ? AND pdf_storage_path IS NULL AND status = 'validated' AND version = ?`
  (la **version lue au rendu** — c'est `render_document`, qui charge la facture (`invoice_pdf_service.rs:198-215`), qui
  la rend ; ses appelants « rappel » (`invoice_email.rs:557`, `:1239`) l'ignorent ; validation P2, M1) et
  l'audit `invoice.pdf_frozen` (empreinte, langue, **acteur** — utilisateur ou clé d'API) — l'empreinte n'existe jamais
  sans sa trace. ⚠️ **La garde `status = 'validated'` est indispensable** (validation P1, A1) : sans elle, une dévalidation
  intercalée entre le rendu et l'`UPDATE` laisserait un PDF figé sur un brouillon, servi comme émis à la revalidation.
  ⚠️ **Et la garde `version` aussi** : une séquence dévalidation → modification → revalidation tenant dans la fenêtre
  passerait la seule garde de statut et figerait un PDF rendu sur les **anciennes** lignes (le numéro est conservé).
  Le gel **ne touche pas `version`** (B2 : sinon une dévalidation ou un avoir lancés depuis une fiche ouverte avant le
  téléchargement tomberaient en 409) ; `updated_at` bouge (`ON UPDATE`). `store_document` est synchrone et fait `fsync` :
  appelé par `spawn_blocking` (B3) ;
- **si zéro ligne** : relire la facture. Colonnes renseignées (un rendu concurrent a figé) → vérifier l'empreinte et
  servir **le document de l'autre**, jamais son propre rendu ; colonnes nulles et facture plus `validated` (dévalidée, ou
  **annulée par un avoir** intercalé) → `InvoiceNotValidated` (ou `InvoiceCancelled` pour une annulée — un code qui dit
  vrai, P2 L5), **sans nouvelle tentative** ; colonnes nulles et facture **`validated` à une autre version** (elle a
  changé pendant le rendu) → **une seule** nouvelle tentative (rendu + pose), puis **409** `INVOICE_CHANGED` si la
  course se répète. En pratique : une pause des rappels, un règlement ou un avoir **concurrents**, dans la seconde du
  rendu, déclenchent la nouvelle tentative ; un envoi par e-mail non (`mark_emailed` ne touche pas `version`) — pas de
  409 en usage normal (P3, L2) ;
- **échec d'écriture du fichier** (`KESH_DOCUMENTS_DIR` non inscriptible) → **500**, rien n'est servi — jamais un rendu
  non figé présenté comme le document (C-F6) ;
- facture `cancelled` **figée** → rend le PDF figé ; `cancelled` **non figée** ou `draft` → refus inchangé
  (`InvoiceNotValidated`).

Le rendu, la validation des préconditions (adresse, banque principale, nombre de lignes) et leurs codes d'erreur sont
**inchangés** : `render` reste la fonction de rendu, le service ne fait que l'encadrer.

**AC 3 — L'intégrité.** Fichier figé **absent** → **410**, code dédié (`INVOICE_PDF_GONE`, sur le patron de
`SourceDocumentGone`) — jamais une régénération silencieuse, qui produirait un autre document. Son message **nomme la
cause et le remède** : le fichier `{empreinte}.pdf` manque sous `KESH_DOCUMENTS_DIR`, à restaurer depuis la sauvegarde de ce
répertoire. Empreinte **différente** →
**500** + `tracing::error!` (le fichier a été altéré) — jamais servi.

**AC 3-bis — Refiger (arbitrage 7).** `POST /api/v1/invoices/{id}/pdf/refreeze`, **Administrateur seulement** (bloc des
routes d'administration, donc refusé aux clés d'API — `require_not_pat`), **sans corps** :
- **accepté seulement si** la facture est figée **et** que son fichier est **absent** (le cas du 410) ; fichier présent →
  **409** `INVOICE_PDF_PRESENT` (on ne remplace pas un document qui existe) ; facture non figée → **409**
  `INVOICE_PDF_NOT_FROZEN` (le prochain rendu la figera) ; fichier présent mais **altéré** → **409**
  `INVOICE_PDF_INTEGRITY` (le 500 d'intégrité se diagnostique, il ne se recouvre pas d'un nouveau document) ; le manuel
  admin dit la procédure : restaurer `{empreinte}.pdf` depuis la sauvegarde ; à défaut, **écarter** le fichier altéré
  (le déplacer hors du répertoire), ce qui ramène au cas 410, puis refiger (P2, M7) ;
- rend un nouveau document par le même chemin que le gel (langue du client **actuelle**, données du moment), l'écrit,
  et remplace les quatre colonnes **dans une transaction** avec l'audit `invoice.pdf_refrozen` portant l'**ancienne** et
  la **nouvelle** empreinte et l'acteur ; gardes `pdf_sha256 = <ancienne empreinte>` **et `status = 'validated'`** sur
  l'`UPDATE` (deux refigeages concurrents : un seul réussit ; un avoir intercalé : refusé — P2 L4) ;
- une facture **`cancelled`** dont le fichier manque **ne peut pas** être refigée — `render` refuse ce statut : limite
  assumée, écrite au manuel ;
- à l'écran, la fiche facture d'un administrateur affiche, quand le téléchargement répond 410, un bouton **« Refiger le
  document »** avec une confirmation qui dit ce que le geste fait — un nouveau document, **pas l'original**, tracé — et
  qui **conseille de restaurer d'abord** le fichier (un volume de documents mal monté après une mise à jour produirait
  des 410 en série ; refiger remplacerait définitivement des originaux qui existent encore ailleurs, P2 L6) ;
- ⚠️ **gardes structurelles** que la route fait rougir (P2, M3) : le compte des constructeurs du bloc d'administration
  (`lib.rs:186`, **25 → 26**) et `admin_pat_denied_e2e.rs` (`:46`, `:757`, `ADMIN_COUPLES`) ; le registre des routes
  d'audit (`audit_route_registry.rs`, `LIB_ROUTES` : `("post", "…refreeze…", Traced)`) **et ses trois totaux**
  (`:436-470`) — `LIB_ROUTES.len()` **109 → 110**, routes tracées **91 → 92** (message de l'assertion réécrit),
  `LIB_ROUTES.len() + TEST_ENDPOINT_ROUTES.len()` **112 → 113** ; le registre des libellés
  (`audit_label_registry.rs` : `invoice.pdf_frozen` et `invoice.pdf_refrozen` dans `audit_labels::ACTIONS`, libellé dans
  les 4 `.ftl` ; si l'action est un paramètre d'une fonction commune, le site entre dans `SITES_INDIRECTS`).

**AC 4 — L'e-mail joint le document figé.** `send_invoice_email` passe par le service : la pièce jointe est **octet pour
octet** celle du téléchargement. Le premier envoi d'une facture non figée la fige — **et le gel survit à un échec SMTP**
(le document a été produit ; la facture n'est simplement pas marquée envoyée, C-F3). Le corps du message reste rédigé dans
la langue **actuelle** du client ; si celle-ci a changé depuis le gel, la pièce jointe reste dans la langue **figée**
(limite assumée, A4). Un client **archivé** : l'envoi reste refusé (`ContactArchived`, inchangé) ; le téléchargement, lui,
fige avec les données du client archivé (C-F5). Les rappels sont **inchangés**
(document distinct, [#502]).

**AC 5 — La dévalidation détache.** `unvalidate` remet les quatre colonnes à `NULL` dans sa transaction ; l'audit de
dévalidation porte l'empreinte détachée. Le fichier reste sur le disque (nommé par son empreinte, il peut être partagé ;
le supprimer n'est pas nécessaire). À la revalidation, le premier rendu fige un **nouveau** document.

**AC 5-bis — L'export de souveraineté.** `serialize_invoices_csv` (`exports/csv_tables.rs:495-521`) exporte
`pdf_sha256`, `pdf_frozen_at` **et `pdf_language`** (un fait du document émis) ; `pdf_storage_path` est **exempté
explicitement**, dérivable de l'empreinte, et le doc-comment de la garde d'en-tête (`:1690-1691`, « exactement les
colonnes de la struct `Invoice` ») est réécrit avec cette exception (P2, M5) (la preuve d'émission appartient aux données de l'utilisateur ; précédent #262), et son
test d'en-tête (`:1696-1705`) est mis à jour. Les fichiers eux-mêmes ne sont pas dans l'export (CSV). La sauvegarde
`.keshbackup` lit les colonnes dans `INFORMATION_SCHEMA` : rien à faire, les colonnes nullables sont couvertes, et une
sauvegarde antérieure reste importable (`check_schema_compat`, colonnes nullables non requises).

**AC 6 — L'écran.** Le DTO de la facture expose `pdfFrozenAt` (et la langue) ; la fiche facture affiche le bouton PDF
pour une facture `validated` **ou** `cancelled` **figée**, et une mention discrète « Document figé le … » quand il l'est.
Libellés dans les 4 locales.

**AC 7 — Tests.** Chacun aurait échoué avant le patch :
- deux téléchargements successifs rendent **les mêmes octets** (aujourd'hui, non : date et `/ID`) ;
- le PDF est dans la **langue du client** (contact en allemand, installation en français) ;
- l'e-mail joint **les octets du téléchargement** (et le premier envoi fige) ;
- changer l'adresse du client **après** le gel ne change pas le PDF ;
- **l'`UPDATE` conditionnel, de façon déterministe** : colonnes pré-posées par une autre voie, puis pose → zéro ligne,
  et le service relit et sert le document déjà posé ; mutation « garde `pdf_storage_path IS NULL` retirée » tuée (un
  `tokio::join!` passerait aussi sans la garde, C-F7) ;
- **la garde de statut** : facture dévalidée entre le rendu et la pose (simulée en appelant la pose après la
  dévalidation) → zéro ligne, rien de figé sur le brouillon ; mutation « garde `status` retirée » tuée (A1) ;
- le gel **ne change pas `version`** ;
- **la garde `version`** (P3, M2) : dévalidation → modification → revalidation, puis pose avec l'**ancienne** version →
  zéro ligne, la nouvelle tentative rend et fige les **nouvelles** lignes ; une course qui se répète → 409
  `INVOICE_CHANGED` ; mutation « garde `version` retirée » tuée ;
- l'audit `invoice.pdf_frozen` nomme l'acteur, et partage la transaction de la pose ;
- un envoi e-mail en **échec SMTP** laisse la facture figée, et le téléchargement suivant rend les mêmes octets ;
- changer la **langue** du client après le gel ne change pas le PDF ;
- une facture à **10 lignes** : erreur inchangée, rien de figé ;
- `KESH_DOCUMENTS_DIR` non inscriptible → 500, colonnes nulles ;
- l'export CSV porte `pdf_sha256` et `pdf_frozen_at` ;
- fichier supprimé → 410 `INVOICE_PDF_GONE` ; fichier altéré → 500, rien servi ;
- dévalidation → colonnes nulles, audit portant l'empreinte ; revalidation → nouveau document ;
- facture annulée par un avoir **après** gel → PDF servi ; **sans** gel → refus ;
- isolation : un PDF figé d'une autre société n'est jamais servi (IDOR) ;
- la contrainte tout-ou-rien rejette un état partiel ;
- Vitest de la fiche : bouton pour `cancelled` figée, absent pour `cancelled` non figée ; mention « figé le » ;
- **refiger** : fichier supprimé → 410 → refigeage par un Admin → 200, nouveau document, audit avec les deux
  empreintes ; refusé pour un Comptable (403), une clé d'API, une facture non figée (409), un fichier présent (409), un
  fichier **altéré** (409 `INVOICE_PDF_INTEGRITY`), une facture `cancelled` ; deux refigeages concurrents → un seul ;
- **le gel par un rôle Consultation** : le premier téléchargement d'un Consultation fige, et l'audit le nomme ;
- E2E (`authedApiContext`, `pdfRes.body()`) : télécharger deux fois une facture rend le même fichier ; une facture
  annulée par un avoir garde son PDF. Chaque test crée **sa propre** facture : les fichiers figés persistent sous
  `/tmp/kesh-e2e/documents` d'une exécution à l'autre.

**AC 8 — Le manuel et le CHANGELOG.** Sites que le patch rend faux, **nommés** (validation P1, C-F2 ; P2, H1 et M2) —
`grep -nE "génère le PDF|PDF QR Bill généré|téléchargeable depuis|PDF joint|ne laisse aucune trace" docs/manual/fr/user-manual.tex` :
- `user-manual.tex:769` (« À la validation, Kesh … génère le PDF QR Bill »), `:780` (« Validée : … PDF QR Bill
  généré ») et `:846` (« À la validation, Kesh génère un PDF conforme… ») : le PDF est **figé au premier téléchargement
  ou envoi** ;
- `:789`, `:797` (coordonnées de l'émetteur « apparaissent sur le PDF ») et `:815` (le numéro de client « apparaît
  ensuite sur toutes les factures ») : **sur les factures non encore figées** — un PDF figé ne change plus ;
- le bouton **« Refiger le document »** et le message du 410, visibles à l'écran (côté administrateur) ;
- `:855` (« téléchargeable depuis la facture validée … pour l'archiver ») : figé, identique d'un téléchargement à
  l'autre, et servi aussi pour une facture **annulée** qui l'a été ; un PDF figé avant l'avoir ne porte pas de mention
  d'annulation (A8) ;
- `:879` (« Pièce jointe : la QR-facture PDF, attachée ») : c'est le PDF **figé**, octet pour octet celui du
  téléchargement ;
- `:928` (« Langue de correspondance … détermine la langue de l'e-mail et du PDF joint ») : aussi du téléchargement, et
  figée au premier rendu ;
- `:1136` (« un PDF téléchargé puis transmis à la main ne laisse aucune trace ») : il laisse désormais une trace (audit
  `invoice.pdf_frozen`, empreinte) ; la dévalidation reste possible tant que la facture n'a pas été envoyée **par Kesh**,
  et détache le document — un PDF déjà transmis par vous reste valable mais n'est plus celui que Kesh conserve (A7) ;
- `:1011` et suivantes : le **rappel**, lui, est toujours régénéré ([#502]).

Manuel admin : `admin-manual.tex:741`, `:790` (`KESH_DOCUMENTS_HOST_DIR`), `:811` et la § 5.1.1 — `KESH_DOCUMENTS_DIR`
porte aussi les **PDF de factures émises**, pièces à conserver ; il **doit être sauvegardé à part** tant que [#503] n'est
pas livrée. ⚠️ **Le chapitre « Sauvegarde et restauration » le rend faux, et c'est le site le plus cher** (P2, H1) — un
administrateur qui suit la procédure documentée perd tous les PDF émis à la première restauration :
- `:1376-1381` (« Stratégie recommandée » : `mariadb-dump`, volume `kesh_db_data`, export ZIP) : **ajouter
  `KESH_DOCUMENTS_DIR`** ;
- `:1444` (restauration) et `:1480` et suivantes (Hyper Backup) : **sauvegarder et restaurer le volume des documents avec
  la base** ;
- `:1583` (le `.keshbackup` contient « toutes les données d'installation ») et le tableau `:1648-1654` (« Installation
  Kesh complète ») : **sauf les fichiers** ([#503]) ;
- `:1353` (rôle Consultation, « aucune mutation ») et `:1774` (portée `read` d'une clé d'API, « GET/HEAD/OPTIONS ») : un
  premier téléchargement **fige** le document et écrit au journal d'audit (arbitrage 8).

Pour la suite : un 410 `INVOICE_PDF_GONE` se répare **de préférence** en restaurant le fichier `{empreinte}.pdf` dans ce répertoire —
c'est le seul moyen de retrouver le document **original** ; à défaut, un administrateur peut **refiger** la facture
(nouveau document, réémission tracée au journal d'audit) — sauf une facture annulée. PDF régénérés, contrôlés **aplatis**. CHANGELOG `[0.12.1]` : `Fixed` (#387), `Changed` (le
téléchargement rend la langue du client).

## Tasks / Subtasks

- [x] **T1 — migration** (AC 1) : SQL, squash, garde de schéma, audit d'idempotence, P6.
- [x] **T2 — le service de gel** (AC 1-bis, 2, 3) : `kesh-db` (entité `Invoice` et **constante unique** de colonnes,
  pose conditionnelle + audit en une transaction, détachement), `kesh-api` (service, `spawn_blocking`) et **six variantes d'erreur neuves** (P3, M3 — aucune n'existe ; seule
  `InvoiceNotValidated` existe, `errors.rs:338`), chacune avec statut HTTP, code JSON et message dans les 4 locales :
  `InvoicePdfGone` (410 `INVOICE_PDF_GONE`), `InvoiceChanged` (409 `INVOICE_CHANGED`), `InvoicePdfPresent` (409
  `INVOICE_PDF_PRESENT`), `InvoicePdfNotFrozen` (409 `INVOICE_PDF_NOT_FROZEN`), `InvoicePdfIntegrity` (409
  `INVOICE_PDF_INTEGRITY`, au refigeage ; 500 à la lecture), `InvoiceCancelled` (code à fixer sur le patron
  d'`InvoiceNotValidated`).
- [x] **T3 — les deux consommateurs** (AC 2, 4) : route de téléchargement, envoi par e-mail.
- [x] **T3-bis — refiger** (AC 3-bis) : route d'administration, trois codes 409, audit, bouton et confirmation à l'écran,
  et les gardes structurelles (compte du bloc d'administration, `admin_pat_denied_e2e`, registres d'audit et de libellés).
- [x] **T4 — la dévalidation** (AC 5).
- [x] **T5 — l'écran** (AC 6).
- [x] **T6 — tests** (AC 7).
- [x] **T6-bis — export de souveraineté** (AC 5-bis).
- [x] **T7 — manuels, CHANGELOG** (AC 8).
- [x] **T8 — gates** : backend complet, frontend complet, **E2E complet** — le backend E2E doit avoir
  `KESH_DOCUMENTS_DIR` inscriptible (`docs/testing.md:167`, `:197`) : sans lui, tout téléchargement de PDF répondra 500.

## Dev Notes

### Ce qu'il ne faut pas faire

- **Ne jamais régénérer automatiquement ni silencieusement** un PDF figé introuvable : c'est fabriquer une pièce qui
  n'a pas été émise. 410, et le dire. **Seul le refigeage explicite d'un administrateur**, tracé avec les deux
  empreintes (arbitrage 7), produit un nouveau document.
- **Ne pas** figer dans la transaction de validation (arbitrage 1) : la validation ne doit pas dépendre du rendu.
- **Ne pas** créer de table (arbitrage 3).
- **Ne pas** toucher aux rappels ni aux avoirs ([#502]).
- **Ne pas** recopier `serve_document` : en extraire la partie commune si besoin (DRY).

### Limites assumées

- Une facture validée **avant** cette version est figée à son prochain rendu, avec les données du moment : son PDF
  d'origine n'a jamais été conservé.
- Les fichiers figés ne sont pas dans la sauvegarde `.keshbackup` ([#503]) : après une restauration sans
  `KESH_DOCUMENTS_DIR`, ils répondent 410.
- Une facture annulée **jamais rendue** n'a pas de PDF (refus inchangé).
- **Fichiers orphelins** : deux premiers rendus concurrents, ou un arrêt entre l'écriture et la pose, laissent un fichier
  jamais référencé (les rendus ne sont pas reproductibles, donc nommés différemment). Inoffensif — nommé par son
  empreinte, jamais servi — et non nettoyé.
- **Langue** : le corps d'un e-mail suit la langue actuelle du client, la pièce jointe la langue figée.
- Une facture **annulée** dont le fichier figé manque ne peut pas être refigée (`render` refuse ce statut) : seule la
  restauration du fichier la répare.
- Le **refigeage est irréversible** : il remplace la référence au document d'origine ; l'empreinte d'origine ne reste
  qu'au journal d'audit.
- **`GET /api/v1/invoices/{id}/pdf` écrit** désormais (gel, audit) — et `HEAD` aussi, axum le servant par le handler
  `get`. Le registre des routes d'audit (`audit_route_registry.rs:186`) ne balaie que les méthodes d'écriture : cette
  route mutante lui échappe. **Angle mort assumé**, écrit dans le registre par un commentaire nommant la route.
- `store_document` fait `sync_all` puis `rename`, sans `fsync` du répertoire parent : après un arrêt brutal, le nom peut
  se perdre alors que la base a commité la pose — un 410, réparable par refigeage (P2, L7).
- Tests : les cas qui suppriment ou altèrent un fichier utilisent **leur propre** `documents_dir` (patron
  `inbox_import_e2e.rs:78`), pas le répertoire partagé `/tmp/kesh-documents-test` ; la contrainte de l'AC 1 borne aussi
  `pdf_language` à `FR`, `DE`, `IT`, `EN` (P2, L8).

### Modules

`kesh-db` (migration, entité et colonnes, pose et dévalidation), `kesh-api` (service, routes PDF et e-mail, erreur,
export CSV), `kesh-i18n`,
`frontend` (fiche facture) — **quatre modules de code**, sous le seuil (+ `docs`, `CHANGELOG`).

### References

- Issue [#387] ; audit du 2026-08-26 (fiscaliste § II.4) ; CR [#502], [#503].
- Cartographie du 2026-10-03 (agent d'exploration, lecture seule) — références reprises ci-dessus.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`).

### Debug Log References

Journaux sous `target/gate-logs/` : `25-6-b-backend.log`, `25-6-b-frontend.log`, `25-6-b-e2e.log`,
`manual-25-6-b.log`.

### Completion Notes List

- **Service unique** `kesh-api/src/routes/issued_invoice_pdf.rs` (`get_or_freeze`, `pose`, `refreeze`) : seul chemin
  vers le PDF d'une facture. L'ancienne fonction `invoice_pdf_service::render` a été **retirée** pour qu'aucun appel
  ne contourne le gel ; `render_document` reste (rappels, #502) et porte désormais `invoice_version`. La nouvelle
  tentative unique est extraite (`with_one_retry`) pour être testée de façon déterministe, sans course réelle.
- **`kesh-db`** : `freeze_pdf` (trois gardes + audit `invoice.pdf_frozen` dans la transaction), `refreeze_pdf`
  (gardes `pdf_sha256 = old` et `status`, audit `invoice.pdf_refrozen` avec les deux empreintes), `find_by_id`
  (facture sans ses lignes). `unvalidate` détache les quatre colonnes ; `invoice_snapshot_json` porte `pdfSha256`.
- **Liste unique de colonnes** (AC 1-bis) : `macro_rules! invoice_columns` + `const INVOICE_COLUMNS` dans
  `repositories/invoices.rs`. Macro parce que `concat!` n'accepte que des littéraux. Les quatre sites en dur sont
  remplacés.
- **Six erreurs** dans `errors.rs`, messages dans les 4 locales : `INVOICE_CANCELLED` (400), `INVOICE_PDF_GONE`
  (410, nomme `{empreinte}.pdf`), `INVOICE_CHANGED`, `INVOICE_PDF_PRESENT`, `INVOICE_PDF_NOT_FROZEN`,
  `INVOICE_PDF_INTEGRITY` (409). À la **lecture**, un fichier altéré répond `Internal` (500), conformément à l'AC 3.
- **Écart avec la spec, recompté depuis la source** : l'AC 3-bis annonçait le bloc d'administration à « 25 → 26 ». Il
  comptait déjà **27** couples sur `main` (5 `GET` + 22 mutants) — la prose de `lib.rs` et de
  `admin_pat_denied_e2e.rs` était périmée. Il passe à **28** (5 + 23) ; le test asserte `ADMIN_COUPLES.len()` et non
  un littéral, il était donc vert malgré la prose. Prose et compteurs recalés. Les trois totaux du registre d'audit
  sont ceux de la spec : 110 / 92 / 113.
- **Garde non anticipée par la spec** : `chaque_colonne_du_schema_est_exportee_ou_ecartee` (25-5-a) a rougi au gate
  complet sur `invoices.pdf_storage_path`. La colonne est inscrite à `COLONNES_HORS_EXPORT` avec son motif.
- **Angle mort assumé** écrit dans `audit_route_registry.rs` : `GET /invoices/{id}/pdf` (et `HEAD`) écrit désormais.
- **`MockMailer`** capture les octets de la pièce jointe (`attachment_bytes`) : sans cela, « l'e-mail joint les
  octets du téléchargement » n'était pas prouvable.
- **Écran** : bouton PDF extrait en snippet (facture validée, ou annulée figée), mention « Document figé le »,
  bouton « Refiger le document » pour un admin après un 410, avec une confirmation qui conseille de restaurer
  d'abord. `sitesTotal` 1758 → 1767 (neuf sites, ventilés dans le test).
- **Tests neufs** (périmètre : `main` → ce commit) : 9 `kesh-db` (`tests/invoice_frozen_pdf.rs`), 23 `kesh-api` HTTP
  et service (`tests/invoice_frozen_pdf_e2e.rs`), 3 unitaires (`with_one_retry`), 1 CSV
  (`serialize_invoices_csv_porte_le_pdf_fige`), 8 Vitest (`invoice-pdf-page.test.ts`), 2 Playwright
  (`invoice-frozen-pdf.spec.ts`).
- **Mutations** (chacune restaurée puis `touch`) :
  - 7 sur les gardes `kesh-db`, chacune tuée par le test qui la nomme : `IS NULL`, `status` (pose), `version`,
    `pdf_sha256 = old`, `status` (refigeage), détachement, empreinte au snapshot ;
  - 6 sur le service : gel ignoré, langue de l'instance, empreinte non contrôlée, adoption de notre propre rendu,
    refigeage d'un fichier présent, refigeage d'une annulée — toutes tuées ;
  - 4 sur la fiche facture : condition de gel retirée, garde `isAdmin` retirée, tout échec pose `pdfGone`, envoi
    sans confirmation — toutes tuées.
  - ⚠️ « Gel ignoré » n'est tuée que par la spec de la facture annulée : sur une facture validée, la seconde pose
    échoue sur `IS NULL` et le service **adopte** le document déjà figé. La conception se défend elle-même, et ce
    test-là ne le distingue pas.
- **Gates**, base remise à zéro avant chacun :
  - backend `scripts/test-fast.sh --ci` : **2583/2583**, 4 skipped. Un premier passage s'était arrêté sur la garde
    d'export ci-dessus, corrigée avant le second ;
  - frontend : `check` 0 erreur, `lint-i18n-ownership` PASS, `test:unit` **853/853**, `build` OK ;
  - **E2E complet** (06:40 UTC) : 227 passés, 11 échecs, 19 skipped. Neuf sont **attendus** : KF-029 ×7, et KF-051
    ×2 (`invoices.spec.ts:415`, `:439`, avant 12:00 UTC). Les deux autres étaient **la spec neuve**, dont le montage
    omettait `ensurePrimaryBankAccountViaApi` (400 `INVOICE_NOT_PDF_READY`). Corrigée, elle est **rejouée seule** :
    2/2. ⚠️ La suite complète n'a pas été relancée après cette correction, qui ne touche que le montage de la spec.
- Manuels FR régénérés (`make admin user`), 0 référence indéfinie, texte contrôlé dans les PDF aplatis.

### File List

- `CHANGELOG.md`, `README.md`, `docs/testing.md`, `docs/migrations-idempotence-audit.md`
- `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/admin-manual.pdf`, `docs/manual/fr/user-manual.tex`,
  `docs/manual/fr/user-manual.pdf`
- `crates/kesh-db/migrations/20261003000001_invoices_frozen_pdf.sql` (nouveau), `crates/kesh-db/migrations.sha384`,
  `crates/kesh-db/test-schema/0001_schema_squash.sql`, `crates/kesh-db/tests/migrations_upgrade_path.rs`
- `crates/kesh-db/src/entities/invoice.rs`, `crates/kesh-db/src/repositories/invoices.rs`,
  `crates/kesh-db/src/repositories/reconciliation.rs`, `crates/kesh-db/tests/invoice_frozen_pdf.rs` (nouveau)
- `crates/kesh-api/src/routes/issued_invoice_pdf.rs` (nouveau), `crates/kesh-api/src/routes/mod.rs`,
  `crates/kesh-api/src/routes/invoice_pdf.rs`, `crates/kesh-api/src/routes/invoice_pdf_service.rs`,
  `crates/kesh-api/src/routes/invoice_email.rs`, `crates/kesh-api/src/routes/invoices.rs`,
  `crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/audit_labels.rs`, `crates/kesh-api/src/lib.rs`,
  `crates/kesh-api/src/mail/mod.rs`, `crates/kesh-api/src/exports/csv_tables.rs`
- `crates/kesh-api/tests/invoice_frozen_pdf_e2e.rs` (nouveau), `crates/kesh-api/tests/admin_pat_denied_e2e.rs`,
  `crates/kesh-api/tests/audit_route_registry.rs`
- `crates/kesh-reconciliation/src/matching.rs`
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`
- `frontend/src/lib/features/invoices/invoices.api.ts`, `frontend/src/lib/features/invoices/invoices.types.ts`,
  `frontend/src/lib/shared/i18n-keys.test.ts`, `frontend/src/routes/(app)/invoices/[id]/+page.svelte`,
  `frontend/src/routes/(app)/invoices/[id]/invoice-pdf-page.test.ts` (nouveau),
  `frontend/src/routes/(app)/invoices/[id]/invoice-settlements-page.test.ts`,
  `frontend/tests/e2e/invoice-frozen-pdf.spec.ts` (nouveau)

## Change Log

- **2026-10-05** — Revue de code P3 (Sonnet ×2 : R = la remédiation `b2eeaab9`, C = acceptation ; prompt
  `25-6-b-review-prompt-p3.md`) : **0 CRITICAL/HIGH/MEDIUM, 8 LOW** — **critère d'arrêt atteint**. Trend : P1 3M/7L
  → P2 5M/8L (dont 2 MED nés de P1) → P3 0 > LOW. Recomptes de la lentille C (migrations 71, bloc admin 28, registre
  110/92/113, parité des 15 clés neuves, tests de P2) **tous exacts**.
  - **Appliqués** (quatre touchent la production, d'où une passe ciblée P4) :
    - relecture après refigeage isolée de son `try` : un échec de lecture ne se fait plus passer pour un échec du
      refigeage (clé `invoice-pdf-refreeze-reload-failed`, 4 locales ; `sitesTotal` 1765 → 1766) ;
    - une facture figée entre deux tentatives est **adoptée** (`render_and_pose`), sous la garde de l'usage ;
    - la garde `Usage::Send` vaut aussi à l'adoption dans `pose` (avoir intercalé après un gel concurrent) ;
    - refus du refigeage : le statut est examiné avant l'empreinte — une dévalidation intercalée ne répond plus
      « rien à refiger » ;
    - décompte de P1 corrigé (test inbox durci, pas neuf) ; CHANGELOG : clé d'API, `HEAD`, restauration, annulée.
  - **Non appliqué** : retrait des marques d'isolation de tous les messages à argument (`t_args`), y compris un
    texte de droite à gauche — même choix que `contacts.rs`, marginal en contexte suisse.
  - Tests : 1 service neuf (`un_envoi_n_adopte_pas_le_gel_d_une_facture_annulee_entre_temps`), 1 Vitest neuf. Deux
    mutations tuées (garde d'adoption retirée, relecture remise dans le même `try`). Le réordonnancement du refus de
    refigeage n'a **pas** de test : la course n'est pas reproductible sans point d'injection.
- **2026-10-05** — Revue de code P2 (Opus ×2 : R = la remédiation `8ac0a4de`, F = périmètre complet ; prompt
  `25-6-b-review-prompt-p2.md`) : **0 CRITICAL/HIGH, 5 MED, 8 LOW**. **Deux des MED viennent de la remédiation P1** :
  R2-2 directement, et F-M1 parce que P1 affiche désormais le message du serveur, avec ses marques invisibles.
  - **MED** :
    - **R2-2** : le 410 d'une facture **annulée** proposait de refiger, ce que l'écran ne permet plus. `InvoicePdfGone`
      porte `refreezable` ; une clé `error-invoice-pdf-gone-cancelled` (4 locales) ne propose que la restauration ;
    - **F-M1** : le message du 410 entourait l'empreinte des marques d'isolation BiDi de Fluent — copiée pour chercher
      le fichier dans une sauvegarde, elle ne trouvait rien, et l'administrateur pouvait refiger un original qui
      existait encore. Marques retirées dans `t_args` (tous les messages d'erreur à argument) ;
    - **F-M2** : aucun test du gel par une clé d'API, que les deux manuels promettent tracé au nom de la clé. Test
      HTTP ajouté (clé `read`) ;
    - **F-M3** : le test de langue ne vérifiait que la colonne. Il vérifie désormais le document (« Rechnung »,
      « Zahlteil », pas « Section paiement ») ;
    - **R2-1**, **antérieur à la story** et trouvé par propagation du symptôme de M1 : « trop de lignes » affichait
      `{ $count }` brut sur les fiches facture **et** avoir. Les deux tables sont fusionnées dans
      `shared/utils/pdf-error.ts` (`pdfErrorMessage`), qui reprend le message du serveur pour les codes à variable.
  - **LOW** :
    - **R2-4** : la garde d'envoi est aussi tenue **dans le service** (`Usage::Send`), sur la facture qu'il relit ;
    - **F-L1** : la facture est relue à chaque tentative, et un rendu d'une autre version que celle dont la langue
      est tirée est rejoué ;
    - **F-L2** : après un refigeage, la fiche se relit (la réponse ne porte pas « Déjà réglé / Reste dû »). ⚠️ Le
      même défaut existe, **antérieur**, après un envoi par e-mail (`+page.svelte`, `sendInvoiceEmail`) : non traité ;
    - **F-L3** : le message d'`INVOICE_CANCELLED` ne prétend plus que l'annulation est survenue « pendant la
      préparation » ;
    - **R2-3** : ordre des refus de l'envoi et commentaire de test mis à jour ;
    - **R2-5** : tests Vitest — `setTimeout(0)` avant une assertion d'absence, `vi.spyOn` restauré ;
    - **F-L4, F-L5** (manuels) : `HEAD` fige aussi ; une facture envoyée avant la v0.12.1 puis annulée n'a pas de
      PDF ; une sauvegarde restaurée antérieure au gel fera figer un nouveau document.
  - **Défaut antérieur relevé, non traité** : la communication de la QR (« Facture {numéro} ») est écrite en
    français quelle que soit la langue du client (`invoice_pdf_service.rs:369`).
  - Tests neufs : 3 HTTP/service, 5 Vitest (`pdf-error.test.ts`), assertions durcies sur 3 tests. **Mutations tuées**
    (7) : marques non retirées, `refreezable` toujours vrai, garde `Usage::Send`, clé d'API non transmise, rendu dans
    la langue de l'installation, « trop de lignes » tiré du catalogue, fiche non relue après refigeage. La relecture
    par tentative (F-L1) n'a **pas** de test : la course n'est pas reproductible sans point d'injection.
  - Gardes i18n recalées et ventilées : `sitesTotal` 1767 → 1765, `sitesNonResolus` 34 → 33 ; entrée tolérée
    `invoice-pdf-error-generic` retirée de `i18n-repli-divergent-actif` (plus divergente).
  - Gate ciblé : `fmt`, `clippy` workspace, 144 tests Rust (`invoice_frozen_pdf_e2e`, `invoice_send_email_e2e`,
    `inbox_import_e2e`, `invoice_pdf_e2e`, `test(error)`), Vitest `shared` + fiches 210/210, `check` 0 erreur.
    Gate complet au push.
- **2026-10-05** — Revue de code P1 (Sonnet ×3, prompt `25-6-b-review-prompt-p1.md`) : **0 CRITICAL/HIGH, 3 MED,
  7 LOW**, tous retenus et corrigés.
  - **MED** :
    - **B1** : `send-email` envoyait une facture **annulée** figée — la seule garde de statut vivait dans le rendu, que
      le service court-circuite pour servir le PDF figé. Garde rétablie dans le handler ;
    - **M1** (A et C, convergents) : le toast du 410 affichait `{ $sha256 }` brut — le catalogue servi au frontend est
      résolu sans arguments et l'emporte sur `err.message`. Le message du serveur est repris tel quel ; le mock
      Vitest honore désormais un catalogue (il rendait toujours le repli, ce qui cachait le défaut) ;
    - **M2** : la note de l'export de souveraineté (manuel utilisateur) ne disait pas que les PDF émis n'y sont pas.
  - **LOW** :
    - bouton « Refiger » proposé sur une facture annulée, où le geste échoue toujours : retiré ;
    - `pdfGone` jamais remis à faux : un téléchargement réussi le remet ;
    - repli d'erreur du refigeage qui parlait de « téléchargement » : clé `invoice-pdf-refreeze-error` (4 locales) ;
    - **L3** : l'import de l'inbox supprimait, sur un champ QR trop long, un fichier archivé nommé par son contenu —
      possiblement un PDF figé ou le justificatif d'une autre société. Suppression retirée ;
    - facture déjà envoyée avant la v0.12.1 : le document figé n'est pas forcément celui reçu — écrit au manuel ;
    - le bouton n'apparaît qu'après un téléchargement refusé — écrit au manuel ;
    - CHANGELOG : trois colonnes insérées dans `invoices.csv` (lecture par position), route et champs d'API.
  - Tests (périmètre `9d842984` → `8ac0a4de`) : 1 HTTP neuf (B1), 3 Vitest neufs, et 1 test inbox existant
    **durci** (`import_field_too_long_returns_failed_not_500`, pas un test neuf — corrigé en P3). **Mutations tuées** : garde de B1 retirée, suppression de
    l'inbox remise, message du 410 tiré du catalogue, condition `validated` du bouton retirée, `pdfGone` non remis.
  - Recomptes de la lentille C (migrations 71, bloc admin 28, registre 110/92/113, `sitesTotal` 1767, parité i18n,
    décomptes de tests) : **tous exacts**.
  - Gate ciblé : `fmt`, `clippy` workspace, `binary(invoice_frozen_pdf_e2e | invoice_send_email_e2e |
    inbox_import_e2e | invoice_pdf_e2e)` 95/95, Vitest de la fiche et `i18n-keys` 27/27. Gate complet au push.
- **2026-10-05** — Développement (dev-story) : T1 à T8. Service de gel, refigeage, détachement, export, écran,
  manuels. Gates : backend 2583/2583, frontend 853/853, E2E 227 passés + 9 échecs attendus (+ la spec neuve rejouée
  seule après correction de son montage, 2/2). Écart de la spec sur le bloc admin (27 → 28, et non 25 → 26), recompté
  depuis la source. → `review`.
- **2026-10-05** — Validation P4 ciblée (Haiku, prompt `25-6-b-validate-prompt-p4.md`) : **0 finding**. Sa première
  vérification résumait sa sortie au lieu de la citer : **reprise par l'orchestrateur** (`audit_route_registry.rs` porte
  bien 109, 91 et 112) ; le « six sites » restant est celui de P1, exact. **Boucle close** : P1 4H/12M (Sonnet ×3) → P2
  1H/7M (Opus) → P3 3M (Sonnet) → P4 0 (Haiku).
- **2026-10-05** — Validation P3 (Sonnet, prompt `25-6-b-validate-prompt-p3.md`) : **0 CRITICAL/HIGH, 3 MED, 4 LOW**,
  retenus.
  - **MED** :
    - les trois totaux du registre des routes d'audit (109, 91, 112), que la route fera rougir (M1) ;
    - les tests de la garde `version` et du refigeage d'un fichier altéré (M2) ;
    - les six variantes d'erreur à créer, nommées à la T2 (M3).
  - **LOW** :
    - « cinq » sites et non « six » pour H1 de P2 ;
    - la fenêtre de la garde `version` écrite — pas de 409 en usage normal, `mark_emailed` ne touche pas `version` ;
    - `render_document` porte la version ;
    - le site `:879` (pièce jointe).
  - La P3 a vérifié chaque site de manuel cité et chaque garde structurelle. Trend : P1 4H/12M → P2 1H/7M → P3 3M.
- **2026-10-05** — Arbitrage de Guy (« d'accord avec tes recommandations ») : **pas de découpage**.
- **2026-10-04** — Validation P2 (Opus, prompt `25-6-b-validate-prompt-p2.md`) : **1 HIGH, 7 MED, 8 LOW**, retenus.
  - **H1** : le chapitre « Sauvegarde et restauration » du manuel admin n'était pas nommé, alors que c'est lui qui fait
    perdre les PDF émis à la première restauration. Cinq sites sont ajoutés à l'AC 8.
  - **M1** : la garde de statut ne fermait pas la séquence dévalidation → modification → revalidation. La pose est aussi
    gardée par la version lue au rendu, avec une seule nouvelle tentative, puis `INVOICE_CHANGED`.
  - **M2** : six autres sites du manuel, dont « Consultation : aucune mutation » et la portée `read` des clés d'API.
  - **M3** : les gardes structurelles de la nouvelle route, et le `GET` mutant hors registre (angle mort écrit).
  - **M4** : Dev Notes réconciliées avec l'arbitrage 7.
  - **M5** : `pdf_language` exporté, `pdf_storage_path` exempté explicitement.
  - **M6** : la chaîne 25-4 touche les mêmes lignes.
  - **M7** : refigeage refusé pour un fichier altéré, avec code et procédure.
  - **LOW** : quatre sites et non six ; « ~15 » recompté à douze ; titre des arbitrages ; course avoir/refigeage ; code
    vrai pour une facture annulée ; impasse et irréversibilité en limites ; `fsync` du répertoire ; répertoire de tests
    dédié ; `pdf_language` borné.
  - ⚠️ **Signal de découpage** : HIGH → HIGH (P1 → P2), formellement atteint. Le HIGH de P2 est un site de manuel non
    nommé, pas un défaut de conception ; aucun défaut recyclé ; toujours quatre modules. **Arbitrage à Guy**,
    recommandation : ne pas découper.
- **2026-10-04** — Validation P1 (Sonnet ×3, prompt `25-6-b-validate-prompt-p1.md`) : **4 HIGH, 12 MED**, LOW.
  - **HIGH retenus** :
    - **A1** : un gel pouvait atterrir sur un **brouillon**, si une dévalidation s'intercalait entre le rendu et la pose.
      La garde `status = 'validated'` est ajoutée à l'`UPDATE` ;
    - **B1** : les colonnes d'`Invoice`, écrites en dur à six endroits, sont ramenées à une constante unique ;
    - **C-F1** : le 410 sur un fichier figé perdu était une impasse. Le geste « refiger » a été soumis à Guy et retenu
      (arbitrage 7) ;
    - **C-F2** : six sites du manuel sont désormais nommés.
  - **MEDIUM retenus** (douze) :
    - le cas « zéro ligne » défini ;
    - l'audit dans la transaction de la pose, avec l'acteur ;
    - le gel par Consultation, soumis à Guy et accepté (arbitrage 8) ;
    - le gel ne touche pas `version` ;
    - `spawn_blocking` ;
    - le squash régénéré par script, et les compteurs de `migrations_upgrade_path` ;
    - l'export de souveraineté ;
    - l'échec SMTP après le gel, et le changement de langue ;
    - le client archivé ;
    - l'écriture de fichier impossible ;
    - un test déterministe de l'`UPDATE` conditionnel ;
    - le PDF transmis hors de Kesh, puis détaché.
  - **LOW** : les références (répertoire `routes/`), l'E2E et `KESH_DOCUMENTS_DIR`, la facture à 10 lignes, les fichiers
    orphelins.
- **2026-10-03** — Créée (Guy : « ok » aux six recommandations et aux deux CR, #502 et #503).

[#387]: https://github.com/guycorbaz/kesh/issues/387
[#502]: https://github.com/guycorbaz/kesh/issues/502
[#503]: https://github.com/guycorbaz/kesh/issues/503
