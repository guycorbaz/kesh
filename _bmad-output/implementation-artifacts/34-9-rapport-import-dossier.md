# Story 34-9 : le rapport d'import du dossier nomme chaque fichier, et un doublon ne passe plus inaperçu (#969)

Status: ready-for-dev

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration** (extension additive du `details` d'un échec,
décision de Guy). Registre : `e34-choix-autonomes-e342.md`, C-e342-10 à C-e342-14, C-e342-19 à C-e342-21. Commits en `refs #969`, `refs #974` ; à la PR :
`closes #969`, `closes #974` (#974, avertissements du rapport écrits en français par le serveur, intégrée à la validation P1).*

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
  details? }` (`:129-136`) — `details` existe déjà ; tous les échecs le posent à `None` **sauf** `FILE_TOO_LARGE`, qui porte
  `{ byteSize, maxBytes }` (`:366-372`) *(prémisse corrigée à la validation P1, F9-2)*.
- **Le contrat est documenté aux clés d'API** : `docs/api-external.md:837-841` (`POST /api/v1/inbox-import`, `failed[]` avec
  `details.byteSize`/`maxBytes`, `warnings[]` « messages lisibles ») *(prémisse corrigée, F9-3 : la fiche disait le contraire)*.
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
- **Le déplacement** : `move_to_failed` (`:890-924`) renomme en `{stem}_{suffixe}.{ext}` dans `<inbox>/failed/` — suffixe =
  8 caractères de l'empreinte pour un échec après le hachage (`:490-491`), 8 caractères aléatoires (`random_suffix()`,
  `:856`) pour un échec d'avant (`:352-483`) : le nom sous lequel l'utilisateur retrouvera le fichier n'est **pas** son nom
  d'origine, et le rapport ne le dit pas.
- **Les fichiers non traités** (instables, disparus en course, au-delà de la limite) sortent en `warnings` : des **chaînes
  françaises** écrites par le serveur (`:396-399`, `:906-908`, `:321-324`), sans nom de fichier structuré (#974). Un fichier
  instable ou disparu est un `FileOutcome::Skipped(String)` (`:139-157`, poussé en `warnings` à `:317`) ; la troncature n'est
  liée à aucun fichier.
- **Les lignes du rapport sont lues par des tests** : `import-page.test.ts:183` (`inbox-import-accepted-row`),
  `frontend/tests/e2e/supplier-documents.spec.ts:110` (`getByTestId('inbox-import-accepted-row').filter({ hasText: name })`) ;
  `inbox-import-failed-row` (`+page.svelte:252`) ; les clés `imported-supplier-invoices-report-accepted` / `-failed` sont lues
  par la garde `i18n-keys.test.ts`.
- **Notifications existantes** : `runImport` émet déjà un avertissement quand le rechargement de la liste échoue
  (`+page.svelte:116-123`).
- **i18n côté écran** : le client reçoit les messages **pré-résolus sans arguments** (`crates/kesh-i18n/src/loader.rs:130-165`) ;
  `i18nMsg` ne remplace que `{ $nom }` (`frontend/src/lib/shared/utils/i18n.svelte.ts:19-27`). Un sélecteur Fluent
  (`{ $n -> … }`) arriverait figé sur sa variante par défaut.
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
- **Route** : `/supplier-invoices/[id]` existe (fiche d'une facture fournisseur).

## Acceptance Criteria

### Serveur

1. **AC-1 — le document existant** : toute entrée `failed[]` de code `DUPLICATE` porte
   `details = { existingDocumentId, existingStatus, supplierInvoiceId }` — `existingStatus` ∈ `to_complete | completed |
   discarded`, `supplierInvoiceId` le lien du document (`null` s'il n'en a pas). Les trois sites la posent. Les deux sites de
   **course** — `UNIQUE` (`:658`) et réactivation manquée (`:577`, où l'état `discarded` lu avant la course est périmé, F9-6) —
   relisent le document par son empreinte **après** le `rollback`, hors transaction, par la même fonction (C-e342-10) ; le site
   `:581` emploie la ligne déjà lue. Si la relecture ne trouve rien (document supprimé entre-temps, cas que rien ne produit
   aujourd'hui), `details` garde `movedTo` seul (AC-2) et l'écran dit « déjà importé » sans lien.
2. **AC-2 — où est le fichier** : toute entrée `failed[]`, quel que soit son code, porte `details.movedTo` : le nom sous lequel
   le fichier a été rangé dans `failed/` (`{stem}_{suffixe}.{ext}`, suffixe = 8 caractères d'empreinte ou d'aléa, rendu par
   `move_to_failed`), sans chemin absolu (C-e342-11). `movedTo` se **fusionne** dans un `details` existant : `FILE_TOO_LARGE`
   garde `byteSize` et `maxBytes` (F9-2).
2-bis. **AC-2-bis — fichiers non traités, structurés (#974)** (C-e342-19) : le rapport gagne deux champs additifs —
   `skipped: [{ fileName, reason }]`, `reason` ∈ `UNSTABLE` (fichier en cours d'écriture, laissé en place, retenté au prochain
   import) | `VANISHED` (disparu pendant le traitement) ; et `truncatedAt: number | null`, la limite par import quand elle est
   atteinte. `FileOutcome::Skipped` porte le code au lieu d'une chaîne. `warnings[]` reste, inchangé, pour les clients d'API
   (déprécié au doc et à `docs/api-external.md`) ; l'écran ne l'affiche plus.
3. **AC-3 — tests serveur** : un test par état du document existant — `to_complete`, `completed` avec `supplierInvoiceId`, et
   la course `discarded` (réactivation forcée à ne rien changer, patron de `:1068` : l'état rendu est celui **relu**, non
   `discarded`) —, plus la course `UNIQUE` si elle se monte sans crochet de production (sinon le dire, comme la 33-6 pour sa
   fenêtre non reproductible) ; chacun asserte `existingDocumentId`, `existingStatus`, `supplierInvoiceId` et que `movedTo`
   désigne un fichier présent dans `failed/`. Un test `FILE_TOO_LARGE` asserte `byteSize`, `maxBytes` et `movedTo` ensemble.
   Un test `skipped` (`UNSTABLE`, fichier modifié pendant le délai de stabilité si le montage le permet, sinon `VANISHED`) et
   un test `truncatedAt` (limite basse par `Config`). Les tests existants passent, le contrat n'étant qu'étendu.

### Écran

4. **AC-4 — une ligne par fichier** : le rapport liste **chaque** fichier de `accepted`, de `failed` et de `skipped`, dans
   l'ordre du nom de fichier (C-e342-12), une ligne chacun (`data-testid="inbox-import-file-row"`, `data-state` = `imported |
   imported-no-qr | duplicate | refused | not-processed`). **Les anciens testids restent** sur les lignes concernées
   (`inbox-import-accepted-row` sur les deux états *importé*, `inbox-import-failed-row` sur *déjà importé* et *refusé*), pour
   `import-page.test.ts:183` et `supplier-documents.spec.ts:110` (F9-5) :
   - *importé* : nom ;
   - *importé sans les données du QR* : nom, motif (`aucune QR-facture exploitable`, ou `QR illisible` et son libellé) et la
     consigne existante ;
   - **déjà importé** : nom ; « Déjà importé : » puis, selon `existingStatus` — `to_complete` : « en attente dans Documents à
     traiter » avec un lien vers la ligne du document plus bas (ancre `#imported-<id>`, posée sur chaque ligne de la liste) ;
     `completed` avec facture : « transformé en facture » avec un lien vers `/supplier-invoices/<supplierInvoiceId>` ;
     `completed` sans facture, ou `discarded` : « retiré de la liste » (`discarded`) ou « déjà traité » (sans facture), sans
     lien (C-e342-14) ; puis « Le fichier a été déplacé dans failed/ sous le nom { $movedTo } ; supprimez-le à la main. » ;
   - *refusé* : nom, motif (`importErrorLabel`), et la même phrase sur `failed/` ;
   - *non traité* (`skipped`) : nom, motif traduit — « en cours d'écriture, laissé dans le dossier : relancez l'import » ou
     « disparu pendant l'import » (#974).
   En tête du rapport, un **résumé composé par le Svelte** à partir d'une clé par segment — `…-summary-imported`,
   `…-summary-duplicates`, `…-summary-refused`, `…-summary-not-processed`, chacune « { $n } … » et au pluriel par « (s) »
   comme les messages voisins —, les segments nuls omis, joints par « , » et terminés par « . ». ⛔ **Pas de sélecteur Fluent**
   *(F9-1)* : le client reçoit les messages pré-résolus sans arguments, un sélecteur y arriverait figé. Si `truncatedAt` est posé,
   une phrase traduite sous le résumé : « La limite de { $n } fichiers par import est atteinte : relancez l'import pour traiter
   le reste. » (#974). Un test lit le **catalogue réel** (fr `messages.ftl`) des clés neuves et vérifie qu'aucune ne porte `->`.
5. **AC-5 — l'avertissement** : après un import qui rend au moins un échec, un fichier non traité ou une troncature, une
   notification d'avertissement (`notifyWarning`) : « Import terminé : { $n } fichier(s) non importé(s). Voir le rapport. », avec
   `n = failed.length + skipped.length` (la troncature ne désigne aucun fichier ; seule, elle donne « Import terminé : la limite
   par import est atteinte. Voir le rapport. ») ; sinon, une notification de succès « Import terminé : { $n } document(s)
   importé(s). » (C-e342-13). Un import **sans aucun fichier** n'émet rien de plus que l'encadré actuel. Si le rechargement de la
   liste échoue ensuite (`+page.svelte:116-123`), **une seule** notification d'avertissement, qui dit les deux (F9-8). L'encadré
   du rapport porte une bordure d'avertissement quand il contient un échec ou un fichier non traité.
6. **AC-6 — Vitest** (`import-page.test.ts`) : rapport mixte (un neuf avec QR, un neuf sans QR, un doublon `to_complete`, un
   doublon `completed`, un refus `FILE_TOO_LARGE`, un fichier `UNSTABLE`, `truncatedAt` posé) : six lignes, dans l'ordre des
   noms, chacune avec son état, ses liens, `movedTo` et ses anciens testids ; le résumé ; la phrase de troncature traduite ;
   notification d'avertissement appelée une fois, avec `n = 4` ; rapport sans échec : notification de succès ; rapport vide :
   aucune notification ; rechargement en échec : une notification. L'assertion de `:183` mise à jour au besoin (elle reste
   valide si le testid demeure).
7. **AC-7 — E2E** (`inbox-import.spec.ts`, deux cas neufs) :
   - **entre deux imports** : un premier import d'un document A ; puis **un même import** d'un document neuf B et d'une copie
     **identique** de A : le rapport porte deux lignes, B importé et A « déjà importé » avec le lien vers sa ligne dans
     « Documents à traiter », et la mention de `failed/` ; la notification d'avertissement est visible ;
   - **dans le même import** (le constat d'origine de #969, F9-7) : un document neuf C et **deux copies identiques** C' de C
     sous deux noms : une ligne *importé*, une ligne *déjà importé* (site `:581` sur une ligne née pendant le run).
   Les copies identiques demandent une aide neuve, `copieIdentique` dans `helpers/inbox.ts` (même octets, autre nom), à côté
   de celle qui rend chaque copie unique. Il rougit si la ligne du doublon disparaît (mutation M4). Nettoyage : documents
   retirés de la liste, fichiers de `failed/` supprimés par le test lui-même (dossier de la recette, jamais `~/kesh/`) ;
   `supplier-documents.spec.ts:110` vert sans modification.
8. **AC-8 — textes** : quatre catalogues (états, segments du résumé, motifs `UNSTABLE`/`VANISHED`, troncature, phrase
   `failed/`, notifications) ; `docs/api-external.md:837-841` : *depuis la v0.17.6*, `details` du `DUPLICATE`, `movedTo` sur
   tout échec, `skipped[]`, `truncatedAt`, et `warnings[]` gardé pour compatibilité (F9-3) ; manuel utilisateur (§ import du
   dossier, `docs/manual/fr/user-manual.tex:2161-2166`) : le rapport ligne par ligne, le doublon et son lien, le dossier
   `failed/` et le renommage `{nom}_{8 caractères}`, à vider à la main, et le fait que le rapport disparaît en quittant la page ;
   `user-manual.tex:2283` (le sous-dossier `failed/` et « le rapport en donne la raison ») aligné. Manuel d'administration :
   `admin-manual.tex:893`, `:905` (les refus rangés dans `failed/`) disent le renommage et que le dossier se vide à la main ;
   `:2276` (sauvegarde) relu, aligné s'il le faut. Les `.tex` changent dans la story ; les PDF de `user-manual` (touché par
   quatre stories de l'epic) et d'`admin-manual` se **régénèrent une fois à l'intégration** et s'y contrôlent aplatis (un PDF
   ne se fusionne pas).

## Tâches

- [ ] **T0 — étape 0** : relire les prémisses ; reproduire sur une installation **jetable** (jamais la base du NAS) l'import
  mixte, avec une capture de l'écran actuel (« avant »), pour établir ce que Guy a vu.
- [ ] **T1 — rouge** : tests serveur AC-3, Vitest AC-6, E2E AC-7 ; rouges journalisés (`kesh-gate-logs/e342/rouges.log`).
- [ ] **T2** : `inbox_import.rs` — `dispose_failed` reçoit le `details` et y **fusionne** `movedTo` (lu de `move_to_failed`,
  qui rend le nom au lieu d'un booléen) ; une fonction qui construit le `details` du doublon depuis un `ImportedSupplierInvoice`
  (DRY, trois sites) ; relecture après `rollback` pour les deux courses (`:577`, `:658`) ; `FileOutcome::Skipped` porte un code,
  le rapport `skipped` et `truncatedAt`, `warnings` garde ses chaînes. Doc-comment du module (`:1-26`) à jour.
- [ ] **T3** : `imported-supplier-invoices.types.ts` (`details` d'un doublon, `skipped`, `truncatedAt`), `+page.svelte` (lignes,
  anciens testids gardés, résumé composé, ancres `id="imported-<id>"`, notifications), i18n, `error-label.ts` si un libellé
  change (la garde de cardinalité suit) ; assertions existantes : `import-page.test.ts:183`, `supplier-documents.spec.ts:110`,
  clés `imported-supplier-invoices-report-accepted` / `-failed` (garde `i18n-keys.test.ts`) — gardées ou retirées avec leur
  garde.
- [ ] **T4** : manuels (`.tex` ; PDF à l'intégration), `docs/api-external.md` ; CHANGELOG *Corrigé* (`refs #969`, `refs #974`).
- [ ] **T5** : captures (thème clair, base jetable) : rapport mixte, notification d'avertissement, lien suivi jusqu'à la ligne.

### Mutations nommées

- **M1** : `details` du site `:581` remis à `None` → rouge AC-3 (`to_complete`, `completed`).
- **M2** : `supplierInvoiceId` toujours `null` → rouge AC-3 (`completed`) et AC-6 (lien).
- **M3** : `movedTo` portant le nom d'origine au lieu du nom rangé → rouge AC-3 (fichier absent de `failed/` sous ce nom).
- **M4** : l'écran n'affiche que les échecs hors doublon → rouge AC-6 et AC-7.
- **M5** : notification de succès quel que soit le rapport → rouge AC-6 et AC-7.
- **M6** : `movedTo` écrase `details` au lieu de s'y fusionner → rouge AC-3 (`FILE_TOO_LARGE` sans `byteSize`).
- **M7** : site `:577` rendant l'état lu avant la course (`discarded`) → rouge AC-3 (course `discarded`).
- **M8** : l'écran ignore `skipped` → rouge AC-6 (ligne *non traité*, `n = 4`).

## Gates

`kesh-api` seul côté serveur (aucun fichier `kesh-db` modifié : `find_by_company_hash` existe). Gate ciblé
`binary(inbox_import_e2e)` entre les passes, gate complet et E2E complet au dernier commit de code.

## Ce qui ne se traite pas

- Conserver le rapport après avoir quitté la page : écarté par Guy (table et migration).
- `warnings[]` n'est pas retiré du contrat : il reste pour les clients d'API, déprécié ; son retrait serait un changement de
  contrat à annoncer (hors périmètre).
- *(Les avertissements écrits en français par le serveur, #974, sont désormais traités ici : AC-2-bis.)*

## Recoupements

Aucun fichier commun avec les autres stories du groupe. `inbox_import.rs` n'est touché par aucune autre story de l'Epic 34.

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5).
- 2026-11-10 : Validation P1 (Sonnet), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p1-e342.md`, pour cette
  fiche 0 C, 1 H, 4 M, 3 LOW, tous retenus après lecture : **F9-1** HIGH (lus : `i18n.svelte.ts:19-27`, `loader.rs:130-165`) →
  résumé composé par segments, contrôle au catalogue réel ; **F9-2** (lus : `inbox_import.rs:366-372`, `FILE_TOO_LARGE` porte
  `details` ; `random_suffix()` `:856`) → fusion de `movedTo`, suffixe empreinte ou aléa, M6 ; **F9-3** (lu :
  `api-external.md:837-841`, la route y est décrite) → prémisse corrigée, AC-8 ; **F9-4** → **#974 intégrée** (AC-2-bis :
  `skipped[]`, `truncatedAt`, motifs traduits ; C-e342-19), la PR porte `closes #974` ; **F9-5** (lus : `import-page.test.ts:183`,
  `supplier-documents.spec.ts:110`) → anciens testids gardés, assertions listées ; **F9-6** (lu : `:493-577`) → relecture aussi au
  site `:577`, M7 ; **F9-7** (lu : `helpers/inbox.ts:1-20`) → second cas E2E dans le même import, aide `copieIdentique` ;
  **F9-8** (lu : `+page.svelte:116-123`) → valeur de `n` et une seule notification. Transversal **X-1** : PDF à l'intégration
  (AC-8).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
