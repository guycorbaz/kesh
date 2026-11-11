# Story 34-9 : le rapport d'import du dossier nomme chaque fichier, et un doublon ne passe plus inaperçu (#969)

Status: review

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration** (extension additive du `details` d'un échec,
décision de Guy). Registre : `e34-choix-autonomes-e342.md`, C-e342-10 à C-e342-14, C-e342-19 à C-e342-22. Commits en `refs #969`, `refs #974` ; à la PR :
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
  françaises** écrites par le serveur — fichier instable `:397-399`, fichier disparu `:882-884` (dans `dispose_failed`),
  troncature `:322-325` *(lignes corrigées à la validation P2, P2-9-2 ; vérifiées par `git show a75835db:… | sed -n`)* —,
  sans nom de fichier structuré (#974). Un fichier
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
   Un test `skipped` (`UNSTABLE`, fichier modifié pendant le délai de stabilité, `STABILITY_DELAY` de 25 ms `:87`, si le
   montage le permet, sinon `VANISHED`) ; **s'il ne se monte pas sans crochet de production, le dire au Change Log** *(P2-9-3)*,
   et un test unitaire de la conversion `FileOutcome::Skipped(code)` → `skipped[]` (et `warnings[]`), qui se monte toujours :
   *(Validation P3, P3-9-1)* l'agrégation vit aujourd'hui dans la boucle privée de `process_inbox` (`:301-317`) ; elle
   s'**extrait** en fonction pure (p. ex. `push_outcome(&mut InboxImportReport, file_name, FileOutcome)`), appelée par la
   boucle et testée seule, dans `mod tests` du fichier ;
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
   En tête du rapport, un **résumé composé par le Svelte** à partir d'une clé par segment (`…` = `imported-supplier-invoices-report`,
   le préfixe des clés voisines `imported-supplier-invoices-report-accepted` / `-failed`) — `…-summary-imported`,
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
   aucune notification ; rechargement en échec : une notification. *(P2-9-1)* Les assertions existantes de
   `import-page.test.ts:182-186` sont **réécrites** : le résumé « 3 document(s) importé(s) » devient le résumé composé, et
   `inbox-import-accepted-row` porte désormais **trois** lignes (le document avec QR compris), dans l'ordre des noms (`abime.png`
   en tête). `supplier-documents.spec.ts:110`, filtré par nom, tient sans modification.
7. **AC-7 — E2E** (`inbox-import.spec.ts`, deux cas neufs) :
   - **entre deux imports** : un premier import d'un document A ; puis **un même import** d'un document neuf B et d'une copie
     **identique** de A : le rapport porte deux lignes, B importé et A « déjà importé » avec le lien vers sa ligne dans
     « Documents à traiter », et la mention de `failed/` ; la notification d'avertissement est visible ;
   - **dans le même import** (le constat d'origine de #969, F9-7) : **deux fichiers de mêmes octets**, `C-1` et `C-2`, neufs
     tous deux : une ligne *importé*, une ligne *déjà importé* (site `:581` sur une ligne née pendant le run). `read_dir`
     (`:255`) ne garantit aucun ordre : le test asserte **l'ensemble des états** (un *importé*, un *déjà importé*), non le nom
     qui porte chacun *(P2-9-4)*.
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

- [x] **T0 — étape 0** : relire les prémisses ; reproduire sur une installation **jetable** (jamais la base du NAS) l'import
  mixte, avec une capture de l'écran actuel (« avant »), pour établir ce que Guy a vu.
- [x] **T1 — rouge** : tests serveur AC-3, Vitest AC-6, E2E AC-7 ; rouges journalisés (`kesh-gate-logs/e342/rouges.log` ;
  journal réel : `kesh-gate-logs/e349/rouges.log`, la clé de l'agent de développement étant `e349`).
- [x] **T2** : `inbox_import.rs` — `dispose_failed` reçoit le `details` et y **fusionne** `movedTo` (lu de `move_to_failed`,
  qui rend le nom au lieu d'un booléen) ; une fonction qui construit le `details` du doublon depuis un `ImportedSupplierInvoice`
  (DRY, trois sites) ; relecture après `rollback` pour les deux courses (`:577`, `:658`) ; `FileOutcome::Skipped` porte un code,
  le rapport `skipped` et `truncatedAt`, `warnings` garde ses chaînes ; agrégation extraite en fonction pure (AC-3, P3-9-1). Doc-comment du module (`:1-26`) à jour.
- [x] **T3** : `imported-supplier-invoices.types.ts` (`details` d'un doublon, `skipped`, `truncatedAt`), `+page.svelte` (lignes,
  anciens testids gardés, résumé composé, ancres `id="imported-<id>"`, notifications), i18n, `error-label.ts` si un libellé
  change (la garde de cardinalité suit) ; assertions existantes : `import-page.test.ts:183`, `supplier-documents.spec.ts:110`,
  clés `imported-supplier-invoices-report-accepted` / `-failed` (garde `i18n-keys.test.ts`) — gardées ou retirées avec leur
  garde.
- [x] **T4** : manuels (`.tex` ; PDF à l'intégration), `docs/api-external.md` ; CHANGELOG *Corrigé* (`refs #969`, `refs #974`).
- [x] **T5** : captures (thème clair, base jetable) : rapport mixte, notification d'avertissement, lien suivi jusqu'à la ligne.

### Mutations nommées

- **M1** : `details` du site `:581` remis à `None` → rouge AC-3 (`to_complete`, `completed`).
- **M2** : `supplierInvoiceId` toujours `null` → rouge AC-3 (`completed`) et AC-6 (lien).
- **M3** : `movedTo` portant le nom d'origine au lieu du nom rangé → rouge AC-3 (fichier absent de `failed/` sous ce nom).
- **M4** : l'écran n'affiche que les échecs hors doublon → rouge AC-6 et AC-7.
- **M5** : notification de succès quel que soit le rapport → rouge AC-6 et AC-7.
- **M6** : `movedTo` écrase `details` au lieu de s'y fusionner → rouge AC-3 (`FILE_TOO_LARGE` sans `byteSize`).
- **M7** : site `:577` rendant l'état lu avant la course (`discarded`) → rouge AC-3 (course `discarded`).
- **M8** : l'écran ignore `skipped` → rouge AC-6 (ligne *non traité*, `n = 4`).
- **M9** : `skipped` jamais rempli côté serveur (la fonction d'agrégation pousse en `warnings` seulement) → rouge AC-3 (test
  unitaire de la conversion), même si le test d'instabilité ne se monte pas.

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
- 2026-11-10 : Validation P2 (Opus), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p2-e342.md`, pour cette
  fiche 0 C, 0 H, 0 M, 4 LOW, tous retenus après lecture : **P2-9-1** (lu : `import-page.test.ts:182-186`, résumé et deux
  lignes) → assertions à réécrire nommées (trois lignes, ordre, résumé) ; **P2-9-2** (lu : `git show a75835db:crates/kesh-api/src/inbox_import.rs`,
  `:322-325`, `:397-399`, `:882-884`) → lignes de #974 corrigées ; **P2-9-3** (lu : `:87`, 25 ms) → sortie écrite et test de
  conversion ; **P2-9-4** (lu : `:255`) → `C-1`/`C-2`, ensemble des états. CHANGELOG : *Corrigé* (C-e342-22).
- 2026-11-10 : Validation P3 (Sonnet), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p3-e342.md`, pour cette
  fiche 0 C, 0 H, 0 M, 1 LOW, retenu : **P3-9-1** (lu : `inbox_import.rs:301-317`, agrégation dans la boucle privée) →
  extraction en fonction pure prescrite à l'AC-3, M9. En passant, le préfixe `…` du résumé est écrit
  (`imported-supplier-invoices-report`), comme pour la 34-4 (P3-4-1).
- 2026-11-10/11 : Développement (Opus 5.5, worktree `/home/gcorbaz/devel/kesh-e349`, clé `e349`, base `665de151`).
  **Étape 0** : aucun LOW de validation en suspens pour cette fiche (P3-9-1 déjà prescrit à l'AC-3 ; P4 et P5 ne portent pas
  sur la 34-9). Prémisses relues au commit de base par `git show 665de151:crates/kesh-api/src/inbox_import.rs | sed -n` :
  `:87` (`STABILITY_DELAY` = 25 ms), `:301-317` (agrégation dans la boucle, `Skipped(warning) => report.warnings.push`),
  `:322-325` (troncature), `:366-372` (`FILE_TOO_LARGE` avec `byteSize`, `maxBytes`), `:397-399` (instable), `:577`, `:581`,
  `:658` (les trois `dispose_failed(…, ERR_DUPLICATE, None)`), `:856` (`random_suffix`), `:882-884` (disparu) — toutes
  justes. Capture « avant » de l'import mixte sur base jetable (`kesh_e2e_e349`) : `kesh-gate-logs/e349/captures/avant-rapport-mixte.png`
  — « 1 document(s) importé(s). » en tête, le doublon en dernière ligne rouge, sans lien ni mention de `failed/` : le
  constat de Guy reproduit. **Rouges** (`kesh-gate-logs/e349/rouges.log`) : 7/7 tests serveur, 13/20 Vitest, 2/5 E2E
  (nouvelle spec contre l'écran du commit de base). Choix neufs : C-e349-1 à C-e349-10
  (`e34-choix-autonomes-e349.md`). **Consigne Edit** : tout le code a été édité à l'outil Edit ; un seul écart, `cargo fmt`
  (le formateur du projet, non un script de remplacement) sur `inbox_import_e2e.rs`, commit `de4e7514`. Les mutations
  passent par un script (`kesh-gate-logs/e349/mut-e349.py`, motif vérifié unique, restauration `git checkout --` + `touch`),
  comme le permet la consigne.
- 2026-11-11 : **Revue de code P1** (trois lentilles B, E, A ; rapports `kesh-gate-logs/e349-review-p1-{B,E,A}.md`) :
  **0 C, 0 H, 0 M** ; LOW : B 5, E 4, A 4, soit neuf points une fois dédoublonnés, tous traités dans la story, aucune
  issue à ouvrir. Remédiation (Opus 5.5), commits `8ddeee5c` et `7740ad67` :
  1. **A-1 = B-5** — `duplicate_details_names_the_document_or_nothing` exerce aussi la branche `Some` (fixture,
     `to_complete` sans facture, `completed` avec facture 42) ;
  2. **A-2 ≈ B-1** — `docs/api-external.md` : `existingStatus` est « l'état lu par l'import, relu après l'échec quand le
     doublon naît d'une course » (au site `:581` la ligne est celle déjà lue) ; l'absence des trois champs vient d'un
     « document disparu entre-temps, ou relecture en erreur » (C-e349-1), non d'« aucun chemin » ;
  3. **E-1 ≈ B-2** — un doublon `to_complete` dont la ligne n'est pas dans la liste affichée ne porte plus de lien mort :
     « absent de la liste ci-dessous : actualisez la page » (clé `-report-duplicate-not-in-list`) ; ⚠️ cette
     remédiation faisait clignoter le texte entre l'arrivée du rapport et le rechargement de la liste — relevé en
     écrivant le prompt P2, corrigé au commit `7740ad67` (`!importing`) ;
  4. **E-2** — toast « {$n} fichier(s) non importé(s), et la limite par import est atteinte » quand les deux arrivent
     ensemble (clé `-notice-not-imported-truncated`) ;
  5. **B-3** — le cadre est bordé d'avertissement aussi à la seule troncature ;
  6. **A-3** — manuel utilisateur : la notification de succès ne paraît que si des documents ont été importés ; le
     lien « tant que la ligne figure dans la liste » ; le cadre bordé ;
  7. **E-3** — `with_moved_to(Some(null))` vaut `None` (garde, et cas ajouté au test unitaire) ;
  8. **E-4 ≈ B-4** — test d'instabilité : fil arrêté et rejoint par un garde `Drop`, panique comprise ; cinq essais au
     plus, à fichiers distincts, si l'ordonnanceur prive le fil de la fenêtre de 25 ms ; il échoue si aucun essai
     n'exerce l'instabilité ;
  9. **Double cadre derrière le toast** (A, capture `apres-notification.png`) : **un empilement de sonner, non deux
     notifications**. Le script de captures lançait deux imports à moins d'une seconde, et le toast de succès du
     premier dure 4 s. Journal de la capture reprise (`e2e-r5/capture.log`) : 1 toast après le premier import, 0 avant
     le second, 1 après le second ; la capture reprise n'en montre qu'un.
  Textes : deux clés neuves aux quatre catalogues ; gardes recomptées, `i18n-keys` 2942 → 2944 sites,
  `i18n-un-repli-par-cle` 426 → 428 clés. Vitest de la page : 20 → 21 tests (un neuf, trois ajustés : la liste du cas
  mixte porte le document 7, le toast mixte dit la limite, le cas du rechargement en échec sans troncature).
  **Gates au dernier commit de code, `7740ad67`** (`kesh-gate-logs/e349/gates3.log`) :
  - gate ciblé `binary(inbox_import_e2e)` + unitaires : 54/54 ;
  - bases remises à neuf, `test-fast --ci` 5149/5149, 12 ignorés ; `cargo build --workspace --all-targets` vert ;
    `gate-ci.sh kesh_ci_e349` 5156 passés, 0 échoué, 16 ignorés ;
  - `npm run check` 0 erreur (29 avertissements préexistants), `lint-i18n-ownership` vert, `test:unit` 2895/2895
    (222 fichiers), `build` vert ;
  - E2E complet, base vide, runner avec `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR` (`e2e-r4`) : 387 réussis, 7 échecs,
    18 ignorés ; les sept échecs sont ceux de la KF-029 (#97) de `docs/testing.md` (`mode-expert:26`, `:41`,
    `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`) ; les deux cas de la 34-9 passent ;
  - captures reprises (`e2e-r5`).
  **Tendance** : P1 0 C / 0 H / 0 M / 13 LOW (9 distincts). La remédiation touche le code de production (Svelte,
  `with_moved_to`) : la boucle se poursuit par une **passe ciblée P2** (prompt
  `34-9-rapport-import-dossier-review-prompt-p2-ciblee.md`, rapport attendu `kesh-gate-logs/e349-review-p2-ciblee.md`).
- 2026-11-11 : **Passe ciblée P2** (Opus 5.5, une lentille sur `e74f55f6..7740ad67` ; rapport
  `kesh-gate-logs/e349-review-p2-ciblee.md`) : **0 C, 0 H, 1 M, 3 LOW**, tous introduits ou laissés par la remédiation
  P1. Remédiation (Opus 5.5), commit `f1b8e65c` :
  - **P2-1 MEDIUM** — « absent de la liste ci-dessous : actualisez la page » poussait à effacer le rapport, non
    conservé, sans ramener le document. La ligne dit désormais « retiré de la liste » si cette page vient de retirer
    le document (`discardedHere`), « pas dans la liste ci-dessous, qui n'a pas pu être rechargée : notez ce rapport,
    puis actualisez la page » si le rechargement de la liste a échoué (`listReloadFailed`), et sinon « n'est plus dans
    la liste ci-dessous » (C-e349-12). Quatre catalogues (valeur de `-report-duplicate-not-in-list` changée, clé neuve
    `-report-duplicate-not-in-list-reload-failed`), replis Svelte, manuel utilisateur. Symptôme grepé sur tout le dépôt
    (`git grep` de `absent de la liste`, `actualisez la page`, `aktualisieren Sie die Seite`, `aggiorna la pagina`,
    `refresh the page`) : restent les seules notifications d'un rechargement en échec, le cas légitime ;
  - **P2-2 LOW** — un second import retire l'ancien rapport dès son lancement (`report = null`, C-e349-11) ;
  - **P2-3 LOW** — tests Vitest : la garde `!importing` (second `listImported` en suspens : lien avant, texte après),
    le second import, le retrait depuis la page, la liste non rechargée ; mutations **M10** (garde retirée) et **M11**
    (`report = null` retiré) — chacune rougit son test (`mut/M10-vitest.log`, `mut/M11-vitest.log`), worktree restauré ;
  - **P2-4 LOW** — captures des trois états neufs : `p2-limite-seule.png` (cadre bordé, toast de la limite),
    `p2-toast-combine.png` (toast « … non importé(s), et la limite … »), `p2-doublon-retire.png` (« retiré de la
    liste », sans lien) ; backend à `KESH_INBOX_MAX_FILES_PER_RUN=2`, un seul toast à chaque capture
    (`e2e-r7/capture-p2.log`).
  Gardes recomptées par leur exécution : `i18n-keys` 2944 → 2946 sites, `i18n-un-repli-par-cle` 428 → 429 clés.
  Vitest de la page : 21 → 25 tests.
  **Gates au dernier commit de code, `f1b8e65c`** (frontend seul touché ; `kesh-gate-logs/e349/gates4.log`) :
  `npm run check` 0 erreur (29 avertissements préexistants), `lint-i18n-ownership` vert, `test:unit` 2899/2899
  (222 fichiers), `build` vert ; E2E complet sur base recréée (`e2e-r6`) : 387 réussis, 7 échecs, 18 ignorés — les sept
  de la KF-029 (#97), nommés dans `docs/testing.md` (`mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`,
  `onboarding:57`, `:77`, `:150`). Backend non rejoué : aucun fichier Rust touché depuis `7740ad67`, où il est vert.
  ⚠️ **Relevé en écrivant le prompt P3, non corrigé** (la chaîne des gates mutait alors le fichier) : un rechargement
  réussi par `safeReloadList` (après un conflit de retrait) ne remet pas `listReloadFailed` à `false` ; la ligne peut
  alors inviter à actualiser alors que la liste est à jour. Soumis à la passe P3 (axe 1).
  **Tendance** : P1 0/0/0/13 LOW → P2 ciblée 0/0/1 M/3 LOW. La remédiation touche le code de production : la boucle se
  poursuit par une **passe ciblée P3** (Sonnet ; prompt `34-9-rapport-import-dossier-review-prompt-p3-ciblee.md`,
  rapport attendu `kesh-gate-logs/e349-review-p3-ciblee.md`).
- 2026-11-11 : **Correctif avant la P3** (Opus 5.5, à la demande de l'orchestrateur : le défaut relevé ci-dessus ne
  reste pas soumis à la P3). Commit `e071f051` : `listReloadFailed` remis à `false` dans `reloadList`, commune aux trois
  sites de rechargement, recensés par `git grep -n "reloadList\|listImported("` sur la page (`onMount`, `safeReloadList`,
  `runImport`, dont le `finally` réaffecte ensuite l'issue du rechargement de l'import). Test Vitest neuf : rechargement
  en échec à l'import, puis conflit de retrait suivi d'un rechargement réussi — la ligne passe de « … qui n'a pas pu
  être rechargée … actualisez » à « n'est plus dans la liste ci-dessous » ; mutation **M12** (remise à faux retirée)
  rougit ce test (`mut/M12-vitest.log`), fichier restauré et touché, worktree propre. Vitest de la page : 25 → 26 tests.
  **Gates au dernier commit de code, `e071f051`** (`kesh-gate-logs/e349/gates5.log` ; voie `.gate-front.lock` pour le
  frontend et la mutation, voie base pour l'E2E, consigne du 2026-11-11) : `npm run check` 0 erreur (29 avertissements
  préexistants), `lint-i18n-ownership` vert, `test:unit` 2900/2900 (222 fichiers), `build` vert ; E2E complet sur base
  recréée (`e2e-r8`) : 387 réussis, 7 échecs, 18 ignorés — les sept de la KF-029 (#97) de `docs/testing.md`
  (`mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`). Backend non rejoué : aucun
  fichier Rust touché depuis `7740ad67`. Le prompt P3 porte désormais sur `f1b8e65c` **et** `e071f051`.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (agent de développement autonome de l'Epic 34).

### Completion Notes List

- **Serveur** (`inbox_import.rs`) : `details.movedTo` fusionné sur toute entrée `failed[]` (`with_moved_to`, `move_to_failed`
  rend le nom rangé) ; un `DUPLICATE` porte `existingDocumentId`, `existingStatus`, `supplierInvoiceId`
  (`duplicate_details`, une définition pour les trois sites) ; relecture par l'empreinte après le `rollback` aux deux sites
  de course (`reread_duplicate`, une erreur journalisée et non escaladée, C-e349-1) ; `SkipReason` (`UNSTABLE`,
  `VANISHED`), `skipped[]`, `truncatedAt` ; `warnings[]` gardé, mêmes chaînes, déprécié ; agrégation extraite en fonctions
  pures `push_outcome` et `mark_truncated`, testées dans `mod tests`.
- **AC-3** : les deux courses **se montent sans crochet de production** (C-e349-6) — verrou de ligne tenu par une
  transaction du test, attente lue dans `information_schema.PROCESSLIST` ; l'instabilité aussi (un fil qui écrit dans le
  fichier pendant l'import). `VANISHED` ne se monte pas sans crochet : couvert par le test unitaire de `push_outcome` (M9).
- **Écran** : résumé composé segment par segment (aucun sélecteur Fluent), une ligne par fichier dans l'ordre des noms
  (`data-testid="inbox-import-file-row"`, `data-state`), anciens testids portés par l'intérieur de la ligne (C-e349-4),
  lien `#imported-<id>` ou `/supplier-invoices/<id>`, ancre sur chaque ligne de la liste, phrase `failed/`, notification
  d'avertissement ou de succès, une seule notification si le rechargement échoue (C-e349-7), bordure d'avertissement
  (C-e349-8). `warnings[]` n'est plus affiché.
- **Textes** : 21 clés neuves dans les quatre catalogues, 2 retirées (`-report-accepted`, `-report-failed`, C-e349-5) ;
  aucune ne porte `->` (test du catalogue réel). Gardes ajustées : `i18n-keys.test.ts` sites 2922 → 2942 (+20),
  `i18n-un-repli-par-cle.test.ts` clés 407 → 426 (+19), recomptés par les gardes elles-mêmes.
- **Documentation** : `docs/api-external.md` (contrat étendu, `warnings[]` déprécié) ; `user-manual.tex` (paragraphe « Le
  rapport d'import », « Ce qui reste refusé ») ; `admin-manual.tex` (`sec:inbox-import` : renommage, vidage à la main ;
  `:2283` « Ce qui reste hors du fichier » relu, juste sans changement). PDF non régénérés (C-e342-20 : à l'intégration).
  CHANGELOG `## [0.17.6] — Non publié`, *Corrigé*, `refs #969`, `refs #974`.
- **Tests ajoutés** (`665de151` → `HEAD`, recomptés par `git show <ref>:<fichier> | grep -c`) : `inbox_import_e2e.rs`
  44 → 50 `#[sqlx::test]` (+6 : quatre doublons, instable, troncature) et un test étendu
  (`import_file_too_large_and_directory_ignored`) ; `inbox_import.rs` 0 → 5 `#[test]` ; `import-page.test.ts` 7 → 20
  tests exécutés (un réécrit, treize neufs dont quatre par locale) ; `inbox-import.spec.ts` 3 → 5 tests.
- **Mutations** (`kesh-gate-logs/e349/mut/*.log`, chaîne `mut/chaine.log`) : les dix rougissent — M1 (2 tests), M2 (1),
  M2-écran (Vitest mixte), M3 (3), M4 (2 Vitest), M5 (3 Vitest), M6 (unitaire + `FILE_TOO_LARGE`), M7 (course
  `discarded`), M8 (2 Vitest), M9 (unitaire + instable). M4 et M5 n'ont **pas** été rejouées en E2E (un `npm run build`
  muté chacune) : le rouge E2E de `e2e-r1`, où l'écran du commit de base n'affiche aucune ligne `inbox-import-file-row`,
  établit déjà que la spec rougit sans la ligne du doublon.
- **Gates au dernier commit de code** — déclarés tels qu'ils ont tourné :
  - backend, au commit `de4e7514` (le commit suivant, `51709afd`, ne touche que le frontend) : bases remises à neuf ;
    `scripts/test-fast.sh --ci` **5149 exécutés, 5149 passés, 12 ignorés** (`gate-test-fast.log`) ;
    `cargo build --workspace --all-targets` vert (`gate-build.log`) ; `scripts/gate-ci.sh kesh_ci_e349` **5156 passés,
    0 échoué, 16 ignorés** (`gate-ci.log`) ;
  - frontend, au commit `de4e7514` : `npm run check` **rouge** (1 erreur de type, `row.skipped` dans le `{:else}`),
    `npm run test:unit` **rouge** (2 gardes i18n de décompte, 2892/2894), `npm run build` vert, `lint-i18n-ownership`
    vert — corrigés au commit `51709afd` ; depuis, `lint-i18n-ownership` vert, et les trois fichiers Vitest concernés
    rejoués isolément, verts (11, 4 et 20 tests, `verts-vitest-gardes.log`). Après la pause `PAUSE-HEAVY` de la station
    (décision de Guy du 2026-11-11 ; la chaîne en attente du verrou a été arrêtée, puis reprise à la levée), au commit
    `f05dc5bd` (même code que `51709afd`) : `npm run check` **0 erreur**, 29 avertissements préexistants hors de la story
    (`front-check-2.log`) ; `npm run test:unit` **2894/2894, 222 fichiers** (`front-test-unit-2.log`) ; `npm run build`
    vert ;
  - **E2E complet**, base `kesh_e2e_e349` recréée vide, runner avec `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR`
    (`e2e-r2/playwright.log`) : **387 réussis, 7 échecs, 18 ignorés**. Les sept échecs sont, un par un, ceux de la KF-029
    (#97) listés dans `docs/testing.md` : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
    `onboarding.spec.ts:57`, `:77`, `:150`. Les deux cas neufs de `inbox-import.spec.ts` et `supplier-documents.spec.ts`
    (sans modification) passent.
- **Captures** (thème clair, base jetable, `kesh-gate-logs/e349/captures/`) : `avant-rapport-mixte.png` (commit de base),
  `apres-rapport-mixte.png`, `apres-notification.png`, `apres-lien-suivi.png` (la ligne visée encadrée).

### File List

- `crates/kesh-api/src/inbox_import.rs`
- `crates/kesh-api/src/routes/imported_supplier_invoices.rs`
- `crates/kesh-api/tests/inbox_import_e2e.rs`
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`
- `frontend/src/routes/(app)/supplier-invoices/import/+page.svelte`
- `frontend/src/routes/(app)/supplier-invoices/import/import-page.test.ts`
- `frontend/src/lib/features/imported-supplier-invoices/imported-supplier-invoices.types.ts`
- `frontend/src/lib/features/imported-supplier-invoices/imported-supplier-invoices.api.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/src/lib/shared/i18n-un-repli-par-cle.test.ts`
- `frontend/tests/e2e/inbox-import.spec.ts`
- `frontend/tests/e2e/helpers/inbox.ts`
- `docs/api-external.md`
- `docs/manual/fr/user-manual.tex`
- `docs/manual/fr/admin-manual.tex`
- `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/e34-choix-autonomes-e349.md`
- `_bmad-output/implementation-artifacts/34-9-rapport-import-dossier-review-prompt-p1.md`
