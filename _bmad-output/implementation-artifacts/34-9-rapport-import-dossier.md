# Story 34-9 : le rapport d'import du dossier nomme chaque fichier, et un doublon ne passe plus inaperçu (#969)

Status: ready-for-dev

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration** (extension additive du `details` d'un échec,
décision de Guy). Registre : `e34-choix-autonomes-e342.md`, C-e342-10 à C-e342-14. Commits en `refs #969` ; à la PR :
`closes #969`.*

## Story

En tant que Comptable qui importe le dossier de ses pièces, je veux un rapport qui nomme **chaque fichier** avec ce qui lui
est arrivé, qui me dise, pour un doublon, où se trouve le document déjà importé et que le fichier est parti dans `failed/`,
et un avertissement visible dès qu'un fichier n'entre pas, pour ne plus manquer un échec noyé sous « N document(s)
importé(s) ».

## Constat et décisions de Guy

- **Constat (production, 0.17.5, 2026-11-10)** : un import mêlant des documents neufs et des documents déjà importés ; aucun
  doublon créé, les fichiers partis dans `failed/`, mais **aucun message remarqué**. Précision de Guy : le rapport commençait
  par « N document(s) importé(s) » avec N > 0, les doublons ne venant qu'ensuite, en rouge. **L'E2E doit exercer ce cas mixte.**
- **Solution retenue (commentaire de Guy sur #969, 2026-11-10)**, qui remplace la « Proposition » du corps de l'issue :
  1. une ligne par fichier traité, avec son état : *importé*, *importé sans les données du QR* (avec motif), **déjà importé**,
     *refusé* (motif) ;
  2. un doublon n'est pas importé (inchangé) ; sa ligne dit **où est le document existant**, avec un lien — en attente dans
     « Documents à traiter », déjà transformé en facture, ou retiré de la liste —, grâce à l'identifiant du document existant
     ajouté au `details` de l'entrée `failed[]` (sans migration) ; et dit que le fichier a été **déplacé dans `failed/`**, à
     supprimer à la main ;
  3. une notification d'avertissement ;
  4. le rapport n'est pas conservé une fois la page quittée.
- **Fichier d'epic** : « Un avertissement apparaît dès qu'un fichier échoue. » (C-e342-13 pour la lecture retenue.)

## Ce que le code dit (prémisses établies)

- **Le contrat** (`crates/kesh-api/src/inbox_import.rs`) : `InboxImportReport { accepted, failed, warnings }` (`:91-98`) ;
  `AcceptedFile { importedSupplierInvoiceId, fileName, qr, qrError? }` (`:101-111`) ; `FailedFile { fileName, errorCode,
  details? }` (`:129-136`) — `details` existe déjà, et **tous** les échecs le posent à `None` aujourd'hui.
- **Le doublon a trois sites**, tous par `dispose_failed(…, ERR_DUPLICATE, None)` :
  - `:577` — la ligne était `discarded`, mais la réactivation n'a rien changé (course : elle n'est plus `discarded`, ou porte
    un QR) ; `existing.id` connu ;
  - `:581` — la ligne existe en `to_complete` ou `completed` ; `existing.id` et `existing.supplier_invoice_id` connus ;
  - `:658` — course sur `UNIQUE (company_id, file_hash)` à l'`INSERT` ; **l'identifiant n'est pas connu** : il se relit par
    `find_by_company_hash` (`kesh-db/src/repositories/imported_supplier_invoices.rs:121-134`) **après** le `rollback` (`:651`).
- **Les états du document existant** (`kesh-db/src/entities/imported_supplier_invoice.rs:34-38`) : `to_complete` (en attente,
  listé sur la page), `completed` (facture créée, `supplier_invoice_id` renseigné — `ON DELETE SET NULL`,
  `20260629000001_imported_supplier_invoices.sql:64-65` ; la suppression d'une facture rend le document à `to_complete`,
  `imported_supplier_invoices.rs:506`, `:533`), `discarded` (retiré de la liste ; un nouveau dépôt le **réactive**, `:493-575`,
  si bien qu'un doublon sur `discarded` n'arrive que par la course de `:577`).
- **Le déplacement** : `move_to_failed` (`:890-924`) renomme en `{stem}_{hash8}.{ext}` dans `<inbox>/failed/` : le nom sous
  lequel l'utilisateur retrouvera le fichier n'est **pas** son nom d'origine, et le rapport ne le dit pas.
- **Les fichiers non traités** (instables, disparus en course, au-delà de la limite) sortent en `warnings` : des **chaînes
  françaises** écrites par le serveur (`:396-399`, `:906-908`, `:318-323`), sans nom de fichier structuré.
- **L'écran** (`frontend/src/routes/(app)/supplier-invoices/import/+page.svelte`) :
  - `runImport` (`:89-128`) n'émet **aucune notification** après un import réussi ; `notifyWarning` est déjà importé (`:6`) ;
  - le rapport (`:216-269`) : « {$n} document(s) importé(s). », puis les seuls acceptés **sans** QR, puis « {$n} échec(s) : »
    en rouge, puis les avertissements ; un document importé **avec** son QR n'est pas nommé ;
  - la liste « Documents à traiter » (`:274-…`) ne porte pas d'ancre par document.
- **Les libellés** : `IMPORT_ERROR_LABELS` (`frontend/src/lib/features/imported-supplier-invoices/error-label.ts:28-41`), lue par
  la garde de cardinalité `i18n-keys.test.ts` ; `DUPLICATE` → « Déjà importé (doublon) ». Type `FailedFile.details?:
  Record<string, unknown> | null` (`imported-supplier-invoices.types.ts:86-90`).
- **Les tests** : serveur, `import_no_qr_duplicate_and_reactivation` (`crates/kesh-api/tests/inbox_import_e2e.rs:897`),
  `reactivation_never_rewrites_stored_qr_and_duplicates_stay` (`:1068`), `import_duplicate_scoped_and_reactivates_discarded`
  (`:1265`) ; E2E, `inbox-import.spec.ts` ne vérifie que la présence de l'encadré (`:59`) ; aucun test ne montre un doublon à
  l'écran. Aide E2E : `helpers/inbox.ts` (fixtures synthétiques, copies rendues uniques par un suffixe après la fin logique).
- **Route** : `/supplier-invoices/[id]` existe (fiche d'une facture fournisseur). La route `POST /inbox-import` n'est pas
  décrite par `docs/api-external.md` (`git grep -n inbox docs/api-external.md` : rien).

## Acceptance Criteria

### Serveur

1. **AC-1 — le document existant** : toute entrée `failed[]` de code `DUPLICATE` porte
   `details = { existingDocumentId, existingStatus, supplierInvoiceId }` — `existingStatus` ∈ `to_complete | completed |
   discarded`, `supplierInvoiceId` le lien du document (`null` s'il n'en a pas). Les trois sites la posent ; celui de la course
   `UNIQUE` relit le document par son empreinte **après** le `rollback`, hors transaction (C-e342-10). Si la relecture ne
   trouve rien (document supprimé entre-temps, cas que rien ne produit aujourd'hui), `details` garde `movedTo` seul (AC-2) et
   l'écran dit « déjà importé » sans lien.
2. **AC-2 — où est le fichier** : toute entrée `failed[]`, quel que soit son code, porte `details.movedTo` : le nom sous lequel
   le fichier a été rangé dans `failed/` (`{stem}_{hash8}.{ext}`, rendu par `move_to_failed`), sans chemin absolu (C-e342-11).
3. **AC-3 — tests serveur** : un test par état du document existant — `to_complete`, `completed` avec `supplierInvoiceId`, et
   la course `discarded` (réactivation forcée à ne rien changer, patron de `:1068`) —, plus la course `UNIQUE` si elle se monte
   sans crochet de production (sinon le dire, comme la 33-6 pour sa fenêtre non reproductible) ; chacun asserte
   `existingDocumentId`, `existingStatus`, `supplierInvoiceId` et que `movedTo` désigne un fichier présent dans `failed/`.
   Les tests existants passent, le contrat n'étant qu'étendu.

### Écran

4. **AC-4 — une ligne par fichier** : le rapport liste **chaque** fichier de `accepted` et de `failed`, dans l'ordre du nom de
   fichier (C-e342-12), une ligne chacun (`data-testid="inbox-import-file-row"`, `data-state` = `imported | imported-no-qr |
   duplicate | refused`) :
   - *importé* : nom ;
   - *importé sans les données du QR* : nom, motif (`aucune QR-facture exploitable`, ou `QR illisible` et son libellé) et la
     consigne existante ;
   - **déjà importé** : nom ; « Déjà importé : » puis, selon `existingStatus` — `to_complete` : « en attente dans Documents à
     traiter » avec un lien vers la ligne du document plus bas (ancre `#imported-<id>`, posée sur chaque ligne de la liste) ;
     `completed` avec facture : « transformé en facture » avec un lien vers `/supplier-invoices/<supplierInvoiceId>` ;
     `completed` sans facture, ou `discarded` : « retiré de la liste » (`discarded`) ou « déjà traité » (sans facture), sans
     lien (C-e342-14) ; puis « Le fichier a été déplacé dans failed/ sous le nom { $movedTo } ; supprimez-le à la main. » ;
   - *refusé* : nom, motif (`importErrorLabel`), et la même phrase sur `failed/`.
   En tête du rapport, un résumé : « { $imported } importé(s), { $duplicates } déjà importé(s), { $refused } refusé(s). » — les
   compteurs nuls omis (forme Fluent par sélecteurs, quatre locales). Les avertissements (`warnings`) restent listés après,
   tels que le serveur les écrit.
5. **AC-5 — l'avertissement** : après un import qui rend au moins un échec (`failed.length > 0`) ou un avertissement
   (`warnings.length > 0`), une notification d'avertissement (`notifyWarning`) : « Import terminé : { $n } fichier(s) n'ont
   pas été importés. Voir le rapport. » ; sinon, une notification de succès « Import terminé : { $n } document(s)
   importé(s). » (C-e342-13). Un import **sans aucun fichier** n'émet rien de plus que l'encadré actuel. L'encadré du rapport
   porte une bordure d'avertissement quand il contient un échec.
6. **AC-6 — Vitest** (`import-page.test.ts`) : rapport mixte (un neuf avec QR, un neuf sans QR, un doublon `to_complete`, un
   doublon `completed`, un refus) : cinq lignes, dans l'ordre des noms, chacune avec son état, ses liens et `movedTo` ;
   notification d'avertissement appelée une fois, avec le nombre ; rapport sans échec : notification de succès ; rapport vide :
   aucune notification.
7. **AC-7 — E2E** (`inbox-import.spec.ts`, cas neuf) : un premier import d'un document A ; puis **un même import** d'un
   document neuf B et d'une copie **identique** de A (même octets, donc sans suffixe d'unicité) : le rapport porte deux lignes,
   B importé et A « déjà importé » avec le lien vers sa ligne dans « Documents à traiter », et la mention de `failed/` ; la
   notification d'avertissement est visible. Il rougit si la ligne du doublon disparaît (mutation M4). Nettoyage : documents
   retirés de la liste, fichier de `failed/` supprimé par le test lui-même (dossier de la recette, jamais `~/kesh/`).
8. **AC-8 — textes** : quatre catalogues (états, résumé, phrase `failed/`, notifications) ; manuel utilisateur (§ import du
   dossier, `docs/manual/fr/user-manual.tex:2161-2166`) : le rapport ligne par ligne, le doublon et son lien, le dossier
   `failed/` et le renommage `{nom}_{8 caractères}`, à vider à la main, et le fait que le rapport disparaît en quittant la page ;
   `user-manual.tex:2283` (le sous-dossier `failed/` et « le rapport en donne la raison ») aligné ; PDF régénéré, contrôlé
   aplati. Manuel d'administration : `admin-manual.tex:893`, `:905` (les refus rangés dans `failed/`) disent le renommage et
   que le dossier se vide à la main ; `:2276` (sauvegarde) relu, aligné s'il le faut ; PDF régénéré.

## Tâches

- [ ] **T0 — étape 0** : relire les prémisses ; reproduire sur une installation **jetable** (jamais la base du NAS) l'import
  mixte, avec une capture de l'écran actuel (« avant »), pour établir ce que Guy a vu.
- [ ] **T1 — rouge** : tests serveur AC-3, Vitest AC-6, E2E AC-7 ; rouges journalisés (`kesh-gate-logs/e342/rouges.log`).
- [ ] **T2** : `inbox_import.rs` — `dispose_failed` reçoit le `details` et y ajoute `movedTo` (lu de `move_to_failed`, qui
  rend le nom au lieu d'un booléen) ; une fonction qui construit le `details` du doublon depuis un `ImportedSupplierInvoice`
  (DRY, trois sites) ; relecture après `rollback` pour la course `UNIQUE`. Doc-comment du module (`:1-26`) à jour.
- [ ] **T3** : `imported-supplier-invoices.types.ts` (type du `details` d'un doublon), `+page.svelte` (lignes, résumé, ancres
  `id="imported-<id>"`, notifications), i18n, `error-label.ts` si un libellé change (la garde de cardinalité suit).
- [ ] **T4** : manuels, PDF ; CHANGELOG *Corrigé* (`refs #969`).
- [ ] **T5** : captures (thème clair, base jetable) : rapport mixte, notification d'avertissement, lien suivi jusqu'à la ligne.

### Mutations nommées

- **M1** : `details` du site `:581` remis à `None` → rouge AC-3 (`to_complete`, `completed`).
- **M2** : `supplierInvoiceId` toujours `null` → rouge AC-3 (`completed`) et AC-6 (lien).
- **M3** : `movedTo` portant le nom d'origine au lieu du nom rangé → rouge AC-3 (fichier absent de `failed/` sous ce nom).
- **M4** : l'écran n'affiche que les échecs hors doublon → rouge AC-6 et AC-7.
- **M5** : notification de succès quel que soit le rapport → rouge AC-6 et AC-7.

## Gates

`kesh-api` seul côté serveur (aucun fichier `kesh-db` modifié : `find_by_company_hash` existe). Gate ciblé
`binary(inbox_import_e2e)` entre les passes, gate complet et E2E complet au dernier commit de code.

## Ce qui ne se traite pas

- **Les avertissements du rapport sont écrits en français par le serveur** (`inbox_import.rs:318-323`, `:396-399`,
  `:906-908`), affichés tels quels dans les quatre locales, et ne nomment pas leur fichier de façon structurée. **Issue
  demandée** (une par problème).
- Conserver le rapport après avoir quitté la page : écarté par Guy (table et migration).
- `docs/api-external.md` ne décrit pas `POST /inbox-import` : rien à aligner ; le champ `details` s'ajoute au doc-comment.

## Recoupements

Aucun fichier commun avec les autres stories du groupe. `inbox_import.rs` n'est touché par aucune autre story de l'Epic 34.

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
