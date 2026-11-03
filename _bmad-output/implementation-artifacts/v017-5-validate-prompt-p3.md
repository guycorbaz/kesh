# Prompt — validation P3 de la fiche v017-5 (release 0.17.0 : #854), deux lentilles, Sonnet

Tu valides la **fiche** de la Story v017-5 de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11).
**#854 (P2)** : à l'import de pièces (*Documents à traiter → Importer le dossier*), la QR-facture d'un document
**scanné** (archive PDF/A, QR en image matricielle) n'est pas lue. La fiche retient : **cause H1**, la page PDF rendue à
**72 dpi** (échelle 1,0 par défaut de `pdfium-render`), à prouver par fixtures synthétiques et un test pdfium avant tout
correctif, avec des paliers de dégradation écrits d'avance ; **remède** : deux passes **par document** — toutes les
pages à 72 dpi, puis, seulement si aucune n'a livré de QR-facture, une seconde passe à `render_scale` (≈ 171 dpi en A4,
au plus 300 dpi, sous la borne de 2000 px) —, boucle extraite sur un rendeur injecté et testée en CI, décodeur restreint
au format QR ; **délai du client** de l'import aligné sur le proxy (300 s) et conduite à l'abandon vérifiée par
exécution (verrou `RELEASE_LOCK`) ; **écran** : « Aucune QR-facture lue » ; **relecture** d'un document sans QR à sa
réactivation ; #854 en `refs` jusqu'au constat du Project Lead. Aucune migration.

La fiche a passé deux validations (P1 Sonnet : 8 MEDIUM, 23 LOW ; P2 Opus : 1 HIGH, 10 MEDIUM, 12 LOW), remédiées par
l'auteur de la fiche (Change Log, entrées « Validation P1 » et « Validation P2 »). **Ne lis pas les rapports des passes
précédentes** : juge la fiche telle qu'elle est.

Worktree `/home/gcorbaz/devel/kesh-0176`, branche `story/v017-5-qr-scanne`, partie de `8e126cfd`. Langue du rapport :
**français**. Modèle : **Sonnet**. Passe **complète** (troisième passe de validation). **Contexte frais** : ne t'appuie
sur aucune passe antérieure, ni sur ce que la fiche dit d'elle-même.

## À lire

1. La fiche `_bmad-output/implementation-artifacts/v017-5-qr-scanne.md`, **en entier**.
2. Le registre `_bmad-output/implementation-artifacts/v017-5-choix-autonomes.md` (C-v017-5-1 à 15).
3. Les issues, **en lecture seule** : `gh issue view 854 --json title,body,comments`, idem `855`, `856`, `858`, `820`.
4. Le code : `crates/kesh-api/src/qr_decode.rs` (en entier), `crates/kesh-api/src/inbox_import.rs` (l. 55-160 et
   320-600, `map_decode_error`), `crates/kesh-db/src/repositories/imported_supplier_invoices.rs` (`find_by_company_hash`,
   `reactivate_to_complete`, `mark_discarded`), `crates/kesh-db/src/entities/` (l'entité des documents importés :
   `has_qr`, `from_scanned`, `without_qr`, `check_scanned_fits_columns`),
   `crates/kesh-db/migrations/20261015000001_imported_documents_without_qr.sql`, `crates/kesh-api/tests/inbox_import_e2e.rs`
   (assistants `qr_invoice_png*`, `blank_png`, `qr_columns_all_null`, `audit_details` ; tests de réactivation),
   `crates/kesh-api/Cargo.toml`, `Cargo.lock` (présence de `printpdf 0.7.0`), `Dockerfile` (l. 19-45),
   `crates/kesh-api/src/services/pieces.rs` (`qr_lu`), `crates/kesh-api/src/config.rs` (`inbox_max_*`),
   `crates/kesh-api/src/routes/imported_supplier_invoices.rs` (route d'import, téléchargement, retrait, détachement),
   `README.md`, `crates/kesh-seed/src/pieges.rs` (littéraux `NewImportedSupplierInvoice`), `crates/kesh-qrbill/src/generator.rs`,
   `crates/kesh-qrbill/src/pdf.rs` (`generate_qr_bill_pdf_with_date`), `crates/kesh-qrbill/tests/golden_test.rs` ;
   `frontend/src/lib/shared/utils/api-client.ts` (délais, `CallOptions`, `post`),
   `frontend/src/lib/features/imported-supplier-invoices/imported-supplier-invoices.api.ts` ; le frontend
   `frontend/src/routes/(app)/supplier-invoices/import/+page.svelte` (+ `import-page.test.ts`),
   `frontend/src/lib/features/imported-supplier-invoices/AttachDocumentPicker.svelte`,
   `frontend/tests/e2e/supplier-documents.spec.ts`, les gardes i18n (`frontend/src/lib/shared/i18n-*.test.ts`) ; les
   quatre `crates/kesh-i18n/locales/*/messages.ftl` (clés `imported-supplier-invoices-*`).
5. La source des dépendances, **en lecture** sous `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` :
   `pdfium-render-0.8.37/src/pdf/document/page/render_config.rs` (`apply_to_page`, `scale_page_by_factor`, maxima,
   `clip`, matrice), `rxing-0.7.1/src/helpers.rs`, `rxing-0.7.1/src/decode_hints.rs`,
   `rxing-0.7.1/src/filtered_image_reader.rs`, `rxing-0.7.1/src/multi/generic_multiple_barcode_reader.rs`,
   `rxing-0.7.1/src/qrcode/qr_code_writer.rs`, `printpdf-0.7.0/src/xobject.rs`, `printpdf-0.7.0/Cargo.toml`, la source de
   `sqlx` (retour d'une `PoolConnection` au pool, `detach`) et de `hyper` (abandon du futur à la fermeture du client).
6. Le **manuel** : `docs/manual/fr/user-manual.tex` l. 800-810, 1600-1612, 1838-1900, `docs/manual/fr/admin-manual.tex`
   l. 330-360 (proxy), 810-820, 879-925 et 2481-2545 ; `docs/api-external.md` l. 774-780 ; les PDF lus aplatis, **vers la sortie standard seulement** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem `admin-manual.pdf`).
7. Pour contexte : `CLAUDE.md` du worktree (*Review Iteration Rule*, *Issue Tracking Rule*, *Recette visuelle*,
   *Test Locally First* et son exception `kesh-db`, *Migration breaking policy*) ; `docs/agents-orchestration.md`
   (§ 4, § 7, § 8) ; `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md` ; `CHANGELOG.md`
   (`[0.17.0] — Non publié`).

## Deux lentilles, en contexte frais chacune

L'orchestrateur lance **deux agents distincts** sur ce même prompt ; chacun ne tient **que** sa lentille.

### Axe obligatoire, pour les deux lentilles — la dernière remédiation

La remédiation de la P2 (`git diff 911c8982 HEAD -- _bmad-output/implementation-artifacts/v017-5-qr-scanne.md
_bmad-output/implementation-artifacts/v017-5-choix-autonomes.md`) a récrit la plus grande partie des critères. La
sévérité se déplace vers ce qu'on vient d'écrire : cherche d'abord les défauts **introduits** par elle — contradictions
entre AC-3, AC-6, AC-6-bis, T10, le tableau des mutations, les manuels prescrits et le registre (C-v017-5-2, -4, -5,
-7, -14, -15) ; décomptes nouveaux (onze cas de T1, cas (a) à (g) de T10, mutations M1 à M14 avec leurs variantes,
D1 à D4, quinze colonnes) recomptés ; renvois `fichier:ligne` nouveaux vérifiés un à un ; et, pour la lentille F,
tout reste de l'ancien libellé (« Aucun QR lu ») ou de l'ancien remède (« pour chaque page », « au lieu de 72 ») hors
des entrées datées du Change Log. Le rapport dit, pour chaque finding, s'il vient de cette remédiation.

### Lentille R — la cause, le remède et leur preuve

- **La cause (C-v017-5-1)** : lis `apply_to_page` toi-même. Sans facteur d'échelle ni taille cible, l'échelle vaut-elle
  bien 1,0, et les maxima ne font-ils que réduire ? Une autre voie du code de Kesh (autre appel de rendu, autre
  configuration) rendrait-elle les pages autrement ? Le calcul « 1,7 à 2,3 px par module » tient-il (46 mm, versions 10
  à 15) ? La branche d'arrêt d'AC-2 est-elle décidable et suffisante si H1 est fausse ?
- **AC-2, le rouge d'abord** : le commit rouge prescrit (fixtures, `render_scale` extraite et non appelée) rend-il
  vraiment T1 rouge et T4 « non lus » sans aucun correctif ? Le témoin vectoriel (payload de `render_spc_qr_png`,
  `generate_qr_bill_pdf_with_date`) est-il constructible tel qu'écrit (lis la signature) et lisible à 72 dpi ? Les
  trois branches d'arrêt sont-elles décidables ? T1-bis (grandeur causale) est-il calculable depuis le générateur ?
- **AC-3, les deux passes par document et `decode_pages`** : la signature et la conduite (étapes 1 à 3) sont-elles
  implémentables telles qu'écrites, y compris le cap (`page_count > max_pages`) et `last_render_err` entre les deux
  passes ? T10 (a) à (g) couvre-t-il chaque branche, et chaque mutation M11, M12, M12a à M12d rougit-elle le cas
  annoncé ? Le faux rendeur peut-il produire « un QR non SPC » (g) sans pdfium ?
- **AC-3, `render_scale`** : recalcule les onze cas de `render_scale` (six pages, cinq `None`) ; la garde `is_finite` est-elle bien écrite ?
  `scale_page_by_factor` combiné aux maxima donne-t-il la sortie annoncée dans `apply_to_page` (arrondi,
  `do_maintain_aspect_ratio`) ? Le déclencheur du second temps (« aucun payload SPC ») est-il juste pour chaque issue du
  premier — payload inexploitable, erreur de rendu, cap de pages (`last_render_err`, `qr_decode.rs:171-198`) ? Le
  second temps pourrait-il masquer ou changer un diagnostic existant (`PDF_RENDER_ERROR`) ? Une page **tournée**
  (`/Rotate 90`) change-t-elle quelque chose ?
- **AC-4** : `detect_multiple_in_luma_with_hints` respecte-t-il `PossibleFormats` (lis `MultiUseMultiFormatReader`) ?
  Le rééchantillonnage `Triangle` de T2 représente-t-il honnêtement le rendu pdfium ? La règle « échelon seulement sur
  preuve » est-elle décidable, et M5 la tient-elle ?
- **AC-6-bis, l'abandon** : la lecture du code (`inbox_import.rs:169-207`, route, hyper, `sqlx`) soutient-elle le risque
  décrit (verrou tenu par une session rendue au pool) ? La vérification prescrite (test `reqwest` à délai court, ou
  `curl --max-time` sur l'installation jetable) est-elle réalisable et son AC décidable (« second import → 200 ») ?
  La correction « mécanique » suggérée (`PoolConnection::detach`) existe-t-elle dans la version de `sqlx` du
  `Cargo.lock` ?
- **AC-6, le délai du client** : `apiClient.post(url, body, call)` accepte-t-il `{ timeoutMs, retry: false }` tel quel ;
  le délai court-il jusqu'aux en-têtes (`api-client.ts:44-52`) ; T11 est-il écrivable (mock de `apiClient`) ?
- **AC-6, le coût** : la commande du banc est-elle exécutable telle qu'écrite (sélecteur `jq` du binaire de test de la
  bibliothèque, `--exact` avec le chemin de module, `LD_LIBRARY_PATH`) ? Le pire document (20 pages scannées sans QR,
  deux rendus chacune) est-il le pire réel ? Les plafonds et le coût agrégé (200 fichiers, proxy à 300 s, NAS) sont-ils
  mesurables et honnêtes ? Le refus d'un budget d'horloge (C-v017-5-4) tient-il encore ?
- **AC-8 / AC-9, la relecture** : l'instruction unique gardée par `creditor_iban IS NULL` est-elle correcte contre la
  contrainte `chk_imported_si_qr_all_or_nothing` ? Les **quinze** colonnes sont-elles exactement celles de
  `from_scanned` ? L'extraction de `QrColumns` est-elle sans effet sur l'import (`create_in_tx`) ? Le décodage avant la transaction et le verrou de run (`GET_LOCK`) laissent-ils
  une course ? Une ligne `discarded` peut-elle porter des colonnes facultatives non vides sans `creditor_iban` (un état
  où la garde écrirait par-dessus) ? Le comportement `ImageDecode` → `none` est-il juste ? La trace `reactivated` — un
  test existant compare-t-il ses `details` à l'égalité stricte ?
- **Les tests et mutations** : T6 (quinze colonnes remises à `NULL` en SQL) simule-t-il fidèlement une ligne entrée
  sans QR ? T9 (dépôt) tue-t-il M7a seul ? Chaque mutation M1 à M14 (variantes M3a, M7a, M10a, M12a à M12d, M13a comprises) rougit-elle le test
  annoncé, et M4, seule déclarée sans test fonctionnel, l'est-elle à juste titre ? Une régression probable
  reste-t-elle sans mutation ?
- **Le générateur (AC-1)** : `printpdf 0.7` sans `embedded_images` permet-il un `ImageXObject` JPEG (filtre DCT) ?
  « Payload ≥ 280 caractères, version ≥ 12 » est-il atteignable avec `QrBillData` ? Le plafond de 1 Mio est-il tenable
  pour une page A4 à 300 dpi bruitée, avec les leviers permis (qualité ≥ 60, bruit restreint à la section paiement) ?
- **Le périmètre** : les défauts voisins #855, #856 sont-ils justement exclus ? La réparation du test cassé
  (`qr_decode.rs:301-313`) est-elle bien mécanique ? Un défaut voisin mécanique manque-t-il ?

### Lentille F — les textes, l'écran, les données à garder et ce qu'ils affirment

- **Le manuel (AC-10, AC-11)** — axe obligatoire : chaque phrase prescrite est-elle vraie au code après la story
  (« d'abord à 72 dpi, puis une seconde fois », « environ 170 dpi », « au plus 300 dpi », « sans doublon », « n'est
  jamais relu » pour un document lu ou joint à une facture — et un document détaché ? —, « une fois retiré, il ne se
  télécharge plus depuis la liste », *Télécharger* rend-il les **mêmes octets** que le fichier déposé) ? Le paragraphe
  du réglage `KESH_INBOX_MAX_PDF_PAGES` dit-il juste le coût et le conseil des petits lots ? Une phrase actuelle du manuel utilisateur ou d'administration, au `.tex` **et** au PDF aplati,
  devient-elle fausse ou reste-t-elle muette sur ce qui change (badge, rapport, *Retirer de la liste*, « Sans QR-code »
  ailleurs dans le manuel, l. 807, 1609, 1889-1898 ; admin l. 818, 886, 905, 1492, 2541-2551) ?
- **`docs/api-external.md` (AC-12)** et **CHANGELOG (AC-13)** : vrais et complets ? Le CHANGELOG dit-il ce qu'un
  utilisateur doit faire pour ses pièces déjà importées, sans promettre plus que la preuve synthétique ? Aucun mot-clé de
  fermeture prescrit, ni dans un commit ni dans la PR (§ *Fermeture de #854*) ?
- **Le libellé « Aucune QR-facture lue » (AC-7, Q1)** : les huit traductions sont-elles correctes et cohérentes avec
  `badge-qr` et `report-qr-unreadable` de chaque locale ? L'ajout des deux clés à `REPLIS_A_SITE_UNIQUE` suffit-il à
  tuer M10 et M10a (lis G13, `:330-360`) ? Le libellé est-il exact dans chaque cas où le badge s'affiche ?
- **Les textes du délai** (AC-10 *Un import long*, AC-11 *les délais*, AC-12, AC-13) : vrais au code livré (« cinq
  minutes », « 300 s »), cohérents entre eux, et subordonnés au résultat d'AC-6-bis là où il faut ?
- **L'inventaire de la valeur (AC-7)** : rejoue `grep -rnF "Sans QR-code"` et `grep -rnF "sans QR-code"` sur tout le
  dépôt ; chaque occurrence est-elle dans le tableau, avec la bonne décision (libellé d'écran ou emploi générique) ?
- **La fermeture de #854 et la recette** : la recette du Project Lead est-elle praticable, sans aucune donnée réelle, et
  la recette de l'orchestrateur reste-t-elle sur une installation jetable ?
- **Les locales (AC-7)** : les huit traductions sont-elles correctes et cohérentes avec les autres clés
  `imported-supplier-invoices-*` de chaque locale ; la règle « repli égal à la valeur fr-CH » est-elle celle des gardes
  i18n du dépôt (lis-les) ; un test (Vitest, E2E, garde) épingle-t-il l'ancienne valeur ailleurs que dans les fichiers
  cités ? Le choix de garder les clés et la valeur d'API `none` (C-v017-5-7) tient-il ?
- **Le bouton (C-v017-5-8, #858)** : l'argument du déterminisme, tel qu'il est maintenant écrit (version, réglage de
  pages, bibliothèque), est-il vrai ? Le geste « Télécharger, Retirer, déposer, Importer » est-il praticable ?
- **Les données à garder** : « aucune migration, aucune table, aucun bump » est-il vrai ; une ligne existante peut-elle
  être réécrite par un chemin que la fiche n'a pas vu (liste, export, sauvegarde, MCP `services/pieces.rs`) ; la recette
  prescrite reste-t-elle hors de la base du NAS ; **aucune donnée réelle** n'est-elle prescrite (fixtures, captures,
  journaux versés) ?
- **Les captures** couvrent-elles chaque état visible touché (liste, rapport, sélecteur, avant / après relecture,
  de-CH) ?
- **Les décomptes** de la fiche et du registre (sept et quinze colonnes QR, onze cas de T1, cas (a) à (g) de T10, mutations
  M1 à M14 et leurs variantes, D1 à D4, quatre fixtures, huit traductions, numéros de ligne, entrées « Validation P1 » et
  « Validation P2 » du Change Log) : recompte-les depuis la source ; chaque `fichier:ligne` cité dit-il ce que la fiche lui
  fait dire ?
- **Les issues** #855, #856, #858 et les questions Q1, Q2 : la fiche les cite-t-elle justement, et reste-t-il un point
  décidé non traité qui ne serait tracé nulle part (« on trace toujours ») ?

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, au chemin de ta lentille :
`/home/gcorbaz/devel/kesh-gate-logs/v0175-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/v0175-validate-p3-F.md` (lentille F). Il contient :

1. **Les findings**, numérotés `R1`, `R2`… ou `F1`, `F2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`),
   le lieu (`fichier:ligne` de la fiche, du registre ou du code), **la ligne lue citée** (sortie `grep -nF` ou extrait
   copié) qui fonde l'affirmation, et le remède proposé. Une affirmation de présence ou d'absence sans sortie `grep -nF`
   copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés** de ta lentille, un par un, avec, pour chaque axe exercé, ce que tu as
   lu ou cherché. Un « 0 finding » sans cette liste ne clôt rien et ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au
  rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit (`issue create|edit|close|comment`, `pr …`) ; `gh issue view` et `gh issue list`
  seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni `make`, ni
  `latexmk`, ni `python3`, ni `node`, ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Ne lis pas `/home/gcorbaz/devel/kesh-gate-logs/v0175-validate-p1-*.md` ni `v0175-validate-p2-*.md` (contexte frais).
- **Aucune pièce réelle** : n'ouvre ni `~/kesh/`, ni `~/kesh-test/`, ni le dossier de projet du Project Lead, ni
  Paperless, ni aucun PDF hors du dépôt ; aucune donnée réelle dans le rapport.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`, `gh issue list`.
