# Prompt — validation P1 de la fiche v017-5 (release 0.17.0 : #854), deux lentilles, Sonnet

Tu valides la **fiche** de la Story v017-5 de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11).
**#854 (P2)** : à l'import de pièces (*Documents à traiter → Importer le dossier*), le QR d'une facture QR **scannée**
(archive PDF/A d'un gestionnaire de documents, QR en image matricielle) n'est pas lu ; le document entre « sans
QR-code ». La fiche retient : **cause H1**, la page PDF rendue à **72 dpi** (échelle 1,0 par défaut de `pdfium-render`,
la borne de 2000 px n'agissant jamais), à **prouver** par des fixtures synthétiques et un test pdfium joué avant tout
correctif ; **remède** : une fonction pure `render_scale` (plus haute résolution que permet la borne existante, au plus
300 dpi), décodeur restreint au format QR, échelon supplémentaire seulement sur preuve ; **écran** : « Aucun QR lu »
dans les quatre locales ; **relecture** d'un document sans QR à sa réactivation (retirer puis redéposer), sans bouton.
Aucune migration.

Worktree `/home/gcorbaz/devel/kesh-0176`, branche `story/v017-5-qr-scanne`, partie de `8e126cfd`. Langue du rapport :
**français**. Modèle : **Sonnet**. Passe **complète** (première passe de validation). **Contexte frais** : ne t'appuie
sur aucune passe antérieure, ni sur ce que la fiche dit d'elle-même.

## À lire

1. La fiche `_bmad-output/implementation-artifacts/v017-5-qr-scanne.md`, **en entier**.
2. Le registre `_bmad-output/implementation-artifacts/v017-5-choix-autonomes.md` (C-v017-5-1 à 11).
3. L'issue, **en lecture seule** : `gh issue view 854 --json title,body,comments`.
4. Le code : `crates/kesh-api/src/qr_decode.rs` (en entier), `crates/kesh-api/src/inbox_import.rs` (l. 55-160 et
   320-600, `map_decode_error`), `crates/kesh-db/src/repositories/imported_supplier_invoices.rs` (`find_by_company_hash`,
   `reactivate_to_complete`, `mark_discarded`), `crates/kesh-db/src/entities/` (l'entité des documents importés :
   `has_qr`, `from_scanned`, `without_qr`, `check_scanned_fits_columns`),
   `crates/kesh-db/migrations/20261015000001_imported_documents_without_qr.sql`, `crates/kesh-api/tests/inbox_import_e2e.rs`
   (assistants `qr_invoice_png*`, `blank_png`, `qr_columns_all_null`, `audit_details` ; tests de réactivation),
   `crates/kesh-api/Cargo.toml`, `Cargo.lock` (présence de `printpdf 0.7.0`), `Dockerfile` (l. 19-45),
   `crates/kesh-api/src/services/pieces.rs` (`qr_lu`) ; le frontend
   `frontend/src/routes/(app)/supplier-invoices/import/+page.svelte` (+ `import-page.test.ts`),
   `frontend/src/lib/features/imported-supplier-invoices/AttachDocumentPicker.svelte`,
   `frontend/tests/e2e/supplier-documents.spec.ts`, les gardes i18n (`frontend/src/lib/shared/i18n-*.test.ts`) ; les
   quatre `crates/kesh-i18n/locales/*/messages.ftl` (clés `imported-supplier-invoices-*`).
5. La source des dépendances, **en lecture** sous `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` :
   `pdfium-render-0.8.37/src/pdf/document/page/render_config.rs` (`apply_to_page`, `scale_page_by_factor`, maxima,
   `clip`, matrice), `rxing-0.7.1/src/helpers.rs`, `rxing-0.7.1/src/decode_hints.rs`,
   `rxing-0.7.1/src/filtered_image_reader.rs`, `printpdf-0.7.0/src/xobject.rs`, `printpdf-0.7.0/Cargo.toml`.
6. Le **manuel** : `docs/manual/fr/user-manual.tex` l. 1838-1900, `docs/manual/fr/admin-manual.tex` l. 879-910 et
   2481-2500 ; `docs/api-external.md` l. 774-780 ; les PDF lus aplatis, **vers la sortie standard seulement** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem `admin-manual.pdf`).
7. Pour contexte : `CLAUDE.md` du worktree (*Review Iteration Rule*, *Issue Tracking Rule*, *Recette visuelle*,
   *Test Locally First* et son exception `kesh-db`, *Migration breaking policy*) ; `docs/agents-orchestration.md`
   (§ 4, § 7, § 8) ; `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md` ; `CHANGELOG.md`
   (`[0.17.0] — Non publié`).

## Deux lentilles, en contexte frais chacune

L'orchestrateur lance **deux agents distincts** sur ce même prompt ; chacun ne tient **que** sa lentille.

### Lentille R — la cause, le remède et leur preuve

- **La cause (C-v017-5-1)** : lis `apply_to_page` toi-même. Sans facteur d'échelle ni taille cible, l'échelle vaut-elle
  bien 1,0, et les maxima ne font-ils que réduire ? Une autre voie du code de Kesh (autre appel de rendu, autre
  configuration) rendrait-elle les pages autrement ? Le calcul « 1,7 à 2,3 px par module » tient-il (46 mm, versions 10
  à 15) ? La branche d'arrêt d'AC-2 est-elle décidable et suffisante si H1 est fausse ?
- **AC-2, le rouge d'abord** : le commit rouge prescrit (fixtures, extraction de `render_scale` rendant 1,0) rend-il
  vraiment T1 rouge et T4 « non lus » sans aucun correctif ? Le témoin de T2 (non lu à 595 × 842) est-il un test sain
  ou un test qui dépend d'une faiblesse de `rxing` (que se passe-t-il à sa montée de version) ?
- **AC-3** : recalcule les six valeurs de `render_scale` (A4 portrait et paysage, section paiement 210 × 105 mm, A6
  portrait, A3, page minuscule). `scale_page_by_factor` combiné aux maxima donne-t-il bien la sortie annoncée dans
  `apply_to_page` (aspect ratio, `do_maintain_aspect_ratio`) ? Une page **tournée** (`/Rotate 90`) ou à `MediaBox`
  décalée change-t-elle quelque chose ?
- **AC-4** : `detect_multiple_in_luma_with_hints` respecte-t-il `PossibleFormats` (lis `MultiUseMultiFormatReader`) ?
  Le rééchantillonnage `Triangle` de T2 représente-t-il honnêtement le rendu pdfium ? La règle « échelon seulement sur
  preuve » est-elle décidable, et M5 la tient-elle ?
- **AC-6, le coût** : les plafonds (1,0 s par page, 20 s pour 20 pages, + 128 Mio) sont-ils mesurables tels qu'écrits
  (commande, mode `--release`, `/usr/bin/time -v`) ? La borne « 16 Mio par bitmap » est-elle exacte, et la pointe réelle
  (copie `as_image`, luminance, image source décodée par pdfium) reste-t-elle sous le plafond ? Le refus d'un budget
  d'horloge (C-v017-5-4) est-il bien motivé ?
- **AC-8 / AC-9, la relecture** : l'instruction unique gardée par `creditor_iban IS NULL` est-elle correcte contre la
  contrainte `chk_imported_si_qr_all_or_nothing` ? Les colonnes **facultatives** du QR (adresse, montant, référence,
  message) sont-elles toutes couvertes ? Le décodage avant la transaction et le verrou de run (`GET_LOCK`) laissent-ils
  une course ? Une ligne `discarded` peut-elle porter des colonnes facultatives non vides sans `creditor_iban` (un état
  où la garde écrirait par-dessus) ? Le comportement `ImageDecode` → `none` est-il juste ? La trace `reactivated` — un
  test existant compare-t-il ses `details` à l'égalité stricte ?
- **Les tests et mutations** : T6 (colonnes remises à `NULL` en SQL) simule-t-il fidèlement une ligne entrée sans QR ?
  T7 tue-t-il M7 tel que M7 est écrit ? Chaque mutation M1 à M10 rougit-elle le test annoncé ? Une régression probable
  reste-t-elle sans mutation ?
- **Le générateur (AC-1)** : `printpdf 0.7` sans la fonctionnalité `embedded_images` permet-il un `ImageXObject` JPEG
  (filtre DCT) ? L'ajouter aux `dev-dependencies` de `kesh-api` n'ajoute-t-il vraiment aucun paquet au `Cargo.lock`
  (fonctionnalités par défaut, `js-sys`) ? « Payload ≥ 280 caractères, version ≥ 12 » est-il atteignable avec les champs
  de `QrBillData` ? La régénération « octets comparés » est-elle déterministe (horodatage, identifiant de document que
  `printpdf` écrit) ?
- **Le périmètre** : les défauts voisins I1, I2 sont-ils justement exclus ? La réparation du test cassé
  (`qr_decode.rs:301-313`) est-elle bien mécanique ? Un défaut voisin mécanique manque-t-il ?

### Lentille F — les textes, l'écran, les données à garder et ce qu'ils affirment

- **Le manuel (AC-10, AC-11)** — axe obligatoire : chaque phrase prescrite est-elle vraie au code après la story
  (« environ 170 dpi », « au plus 300 dpi », « sans doublon », « n'est jamais relu » pour un document lu ou dont la
  facture est créée, *Télécharger* rend-il les **mêmes octets** que le fichier déposé — lis la route de téléchargement et
  `document_storage`) ? Une phrase actuelle du manuel utilisateur ou d'administration, au `.tex` **et** au PDF aplati,
  devient-elle fausse ou reste-t-elle muette sur ce qui change (badge, rapport, *Retirer de la liste*, « Sans QR-code »
  ailleurs dans le manuel, l. 807, 1609, 1889-1898 ; admin l. 818, 886, 905, 1492, 2541-2551) ?
- **`docs/api-external.md` (AC-12)** et **CHANGELOG (AC-13)** : vrais et complets ? Le CHANGELOG dit-il ce qu'un
  utilisateur doit faire pour ses pièces déjà importées ? Aucun mot-clé de fermeture prescrit dans un commit ?
- **Les locales (AC-7)** : les huit traductions sont-elles correctes et cohérentes avec les autres clés
  `imported-supplier-invoices-*` de chaque locale ; la règle « repli égal à la valeur fr-CH » est-elle celle des gardes
  i18n du dépôt (lis-les) ; un test (Vitest, E2E, garde) épingle-t-il l'ancienne valeur ailleurs que dans les fichiers
  cités ? Le choix de garder les clés et la valeur d'API `none` (C-v017-5-7) tient-il ?
- **Le bouton (C-v017-5-8)** : l'argument « le décodeur est déterministe, relire à version égale rend le même
  résultat » est-il vrai (pdfium, `rxing`, aucune source d'aléa) ? Le geste « Télécharger, Retirer, déposer, Importer »
  est-il réellement praticable par un utilisateur (droits, rôles, le fichier quitte le dossier à l'import) ?
- **Les données à garder** : « aucune migration, aucune table, aucun bump » est-il vrai ; une ligne existante peut-elle
  être réécrite par un chemin que la fiche n'a pas vu (liste, export, sauvegarde, MCP `services/pieces.rs`) ; la recette
  prescrite reste-t-elle hors de la base du NAS ; **aucune donnée réelle** n'est-elle prescrite (fixtures, captures,
  journaux versés) ?
- **Les captures** couvrent-elles chaque état visible touché (liste, rapport, sélecteur de document à joindre, une
  locale non française) ?
- **Les décomptes** de la fiche et du registre (sept colonnes QR, six cas de T1, dix mutations, quatre fixtures, numéros
  de ligne) : recompte-les depuis la source ; chaque `fichier:ligne` cité dit-il ce que la fiche lui fait dire ?
- **Les issues** I1, I2, I3 et les questions Q1 à Q4 : justifiées, bien priorisées, sans doublon d'une issue ouverte
  (`gh issue list --state open --search "pdfium"` et `"spawn_blocking"`, en lecture) ?

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, au chemin de ta lentille :
`/home/gcorbaz/devel/kesh-gate-logs/v0175-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/v0175-validate-p1-F.md` (lentille F). Il contient :

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
- **Aucune pièce réelle** : n'ouvre ni `~/kesh/`, ni `~/kesh-test/`, ni le dossier de projet du Project Lead, ni
  Paperless, ni aucun PDF hors du dépôt ; aucune donnée réelle dans le rapport.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`, `gh issue list`.
