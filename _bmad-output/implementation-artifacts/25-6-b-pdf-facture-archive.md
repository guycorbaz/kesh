# Story 25.6-b : Le PDF d'une facture est figé — une pièce émise ne change plus

Status: ready-for-dev

**Issue : [#387]**, que cette story **ferme** : la PR porte `closes #387` dans le **titre ET le corps**. Branche
`story/25-6-b-pdf-facture-archive`, partie de `main` (`d7c74f02`). Seconde moitié de la 25-6 (la 25-6-a a traité #388 et
#389).

## Arbitrages (Guy, 2026-10-03, « ok » aux six recommandations)

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

**Hors périmètre, en CR** : figer les PDF d'avoir et de rappel ([#502]) ; emporter les fichiers dans `.keshbackup`
([#503]).

## Story

En tant que **personne qui émet des factures dans Kesh**,
je veux que **le PDF d'une facture, une fois émis, ne change plus** — même si je modifie ensuite mes coordonnées, celles
du client ou la mise en page,
afin de **pouvoir produire, dans cinq ans, la copie exacte de la facture envoyée**, ce qu'un contrôle demande.

## Les faits, vérifiés dans le code (cartographie du 2026-10-03)

- **Régénéré à chaque demande** : `GET /api/v1/invoices/{id}/pdf` (`crates/kesh-api/src/routes/invoice_pdf.rs:23-56`)
  appelle `invoice_pdf_service::render(…, state.config.locale, …)` ; `render` (`invoice_pdf_service.rs:177-269`) relit
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
- **Migrations** : 70 sur `main`. ⚠️ La chaîne 25-4 (non mergée) en ajoute quatre, datées `20261001000001`–`…04` : la
  migration de cette story est datée **`20261003000001`** pour s'ordonner après elles ; les compteurs (`upgrade_path`,
  audit d'idempotence) se recomposeront au merge.

## Acceptance Criteria

**AC 1 — Le schéma.** Migration `20261003000001_invoices_frozen_pdf.sql` : `ALTER TABLE invoices ADD COLUMN`
`pdf_storage_path VARCHAR(512) NULL`, `pdf_sha256 CHAR(64) NULL`, `pdf_frozen_at DATETIME(3) NULL`,
`pdf_language CHAR(2) NULL`, et une contrainte **tout-ou-rien** (`CHECK` : les quatre nuls, ou les quatre renseignés avec
`CHAR_LENGTH(pdf_sha256) = 64`). DDL pur, **non breaking** (`ADD COLUMN` nullable — P1/P3, pas de bump de
`kesh_version_min_required`). Squash de test (`crates/kesh-db/test-schema/`) et `test_schema_guard` à jour ; ligne au
tableau `docs/migrations-idempotence-audit.md` et compteurs **recomptés depuis la source** (P5) ; sites positionnels
inspectés (P6, `grep -rn "migrations.len()\|apply_migrations_up_to" crates/`) ; pas d'écriture de données (P7 sans
objet).

**AC 2 — Figer au premier rendu.** Un service unique `kesh-api` (ex. `issued_invoice_pdf::get_or_freeze`) est le **seul**
chemin vers le PDF d'une facture, pour le téléchargement **et** pour l'e-mail :
- facture **déjà figée** → relit le fichier, **vérifie** son SHA-256 contre `pdf_sha256`, rend ces octets ;
- facture `validated` **non figée** → rend le PDF dans la **langue du client** (`resolve_language`), l'écrit par
  `store_document` (`ext = "pdf"`), puis pose les quatre colonnes par un `UPDATE … WHERE id = ? AND company_id = ? AND
  pdf_storage_path IS NULL` ; **si zéro ligne** (un rendu concurrent a figé avant), relit et rend le PDF figé par
  l'autre — jamais deux documents pour une même facture ; journal d'audit `invoice.pdf_frozen` avec l'empreinte et la
  langue ;
- facture `cancelled` **figée** → rend le PDF figé ; `cancelled` **non figée** ou `draft` → refus inchangé
  (`InvoiceNotValidated`).

Le rendu, la validation des préconditions (adresse, banque principale, nombre de lignes) et leurs codes d'erreur sont
**inchangés** : `render` reste la fonction de rendu, le service ne fait que l'encadrer.

**AC 3 — L'intégrité.** Fichier figé **absent** → **410**, code dédié (`INVOICE_PDF_GONE`, sur le patron de
`SourceDocumentGone`) — jamais une régénération silencieuse, qui produirait un autre document. Empreinte **différente** →
**500** + `tracing::error!` (le fichier a été altéré) — jamais servi.

**AC 4 — L'e-mail joint le document figé.** `send_invoice_email` passe par le service : la pièce jointe est **octet pour
octet** celle du téléchargement. Le premier envoi d'une facture non figée la fige. Les rappels sont **inchangés**
(document distinct, [#502]).

**AC 5 — La dévalidation détache.** `unvalidate` remet les quatre colonnes à `NULL` dans sa transaction ; l'audit de
dévalidation porte l'empreinte détachée. Le fichier reste sur le disque (nommé par son empreinte, il peut être partagé ;
le supprimer n'est pas nécessaire). À la revalidation, le premier rendu fige un **nouveau** document.

**AC 6 — L'écran.** Le DTO de la facture expose `pdfFrozenAt` (et la langue) ; la fiche facture affiche le bouton PDF
pour une facture `validated` **ou** `cancelled` **figée**, et une mention discrète « Document figé le … » quand il l'est.
Libellés dans les 4 locales.

**AC 7 — Tests.** Chacun aurait échoué avant le patch :
- deux téléchargements successifs rendent **les mêmes octets** (aujourd'hui, non : date et `/ID`) ;
- le PDF est dans la **langue du client** (contact en allemand, installation en français) ;
- l'e-mail joint **les octets du téléchargement** (et le premier envoi fige) ;
- changer l'adresse du client **après** le gel ne change pas le PDF ;
- deux premiers rendus **concurrents** ne figent qu'un document ;
- fichier supprimé → 410 `INVOICE_PDF_GONE` ; fichier altéré → 500, rien servi ;
- dévalidation → colonnes nulles, audit portant l'empreinte ; revalidation → nouveau document ;
- facture annulée par un avoir **après** gel → PDF servi ; **sans** gel → refus ;
- isolation : un PDF figé d'une autre société n'est jamais servi (IDOR) ;
- la contrainte tout-ou-rien rejette un état partiel ;
- Vitest de la fiche : bouton pour `cancelled` figée, absent pour `cancelled` non figée ; mention « figé le » ;
- E2E : télécharger deux fois une facture rend le même fichier ; une facture annulée par un avoir garde son PDF.

**AC 8 — Le manuel et le CHANGELOG.** `user-manual.tex`, section des factures : le PDF est **figé** au premier
téléchargement ou envoi, dans la langue du client ; il ne suit plus les modifications ultérieures ; une facture
dévalidée puis revalidée reçoit un nouveau document ; une facture annulée garde le sien. Le manuel admin
(`admin-manual.tex`) : `KESH_DOCUMENTS_DIR` porte aussi les PDF figés et **doit être sauvegardé à part** tant que [#503]
n'est pas livrée. PDF régénérés, contrôlés **aplatis**. CHANGELOG `[0.12.1]` : `Fixed` (#387), `Changed` (le
téléchargement rend la langue du client).

## Tasks / Subtasks

- [ ] **T1 — migration** (AC 1) : SQL, squash, garde de schéma, audit d'idempotence, P6.
- [ ] **T2 — le service de gel** (AC 2, 3) : `kesh-db` (lecture/pose/détachement des colonnes, `UPDATE` conditionnel),
  `kesh-api` (service, code d'erreur 410, i18n de l'erreur).
- [ ] **T3 — les deux consommateurs** (AC 2, 4) : route de téléchargement, envoi par e-mail.
- [ ] **T4 — la dévalidation** (AC 5).
- [ ] **T5 — l'écran** (AC 6).
- [ ] **T6 — tests** (AC 7).
- [ ] **T7 — manuels, CHANGELOG** (AC 8).
- [ ] **T8 — gates** : backend complet, frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- **Ne jamais régénérer** un PDF figé introuvable : c'est fabriquer une pièce qui n'a pas été émise. 410, et le dire.
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

### Modules

`kesh-db` (migration, colonnes, dévalidation), `kesh-api` (service, routes PDF et e-mail, erreur), `kesh-i18n`,
`frontend` (fiche facture) — **quatre modules de code**, sous le seuil (+ `docs`, `CHANGELOG`).

### References

- Issue [#387] ; audit du 2026-08-26 (fiscaliste § II.4) ; CR [#502], [#503].
- Cartographie du 2026-10-03 (agent d'exploration, lecture seule) — références reprises ci-dessus.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-10-03** — Créée (Guy : « ok » aux six recommandations et aux deux CR, #502 et #503).

[#387]: https://github.com/guycorbaz/kesh/issues/387
[#502]: https://github.com/guycorbaz/kesh/issues/502
[#503]: https://github.com/guycorbaz/kesh/issues/503
