# Prompt — revue de code P1, Story 27-3 (changer de profil après l'aperçu recalcule l'aperçu ; le serveur refuse un import dont le profil, le compte ou le fichier a changé)

*Versionné le 2026-10-12. Trois lentilles (Sonnet), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-27-3`, branche `story/27-3-profil-apres-apercu`. **Le diff à revoir** :
`git diff 1127c2a1 9bd5d039 -- . ':(exclude)_bmad-output'` (`1127c2a1` = validation close de la fiche ; `444ea236` =
dernier commit de **code** ; `d7cf88ca` et `9bd5d039` portent le manuel (`.tex` + PDF), le CHANGELOG et
`docs/api-external.md` ; les commits suivants ne portent que la fiche, le registre et ce prompt). ⚠️ Pathspec
`':(exclude)…'` et **non** `':!…'`, que le shell peut interpréter. Fiche :
`_bmad-output/implementation-artifacts/27-3-profil-apres-apercu.md` — inventaire clos D1-D19, AC1 à AC17 (dont
AC1-bis), Tâches, tests prescrits (U, I, V, E), mutations M1-M27 (vingt-neuf), Dev Notes, et le **Dev Agent Record**
(choix C-27-3-46 à 50 au registre `epic-27-choix-autonomes-27-3.md`, écarts déclarés, décomptes, gates, mutations).
Règles : `CLAUDE.md` du dépôt — § « Recompter ses propres comptes rendus », § « Propagation post-patch », § « Le prompt
d'une passe doit NOMMER le manuel », § « Une passe qui ne déclare pas ses axes exercés ne compte pas », § « Inventorier
les sites NON RÉSOLUS », § « Migration breaking policy » (aucune migration attendue, AC15). Issue : #626 (la PR la
fermera).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt (rien sous `~/kesh-test/`) ; la fixture
`crates/kesh-import/tests/fixtures/csv/ubs_synthetique.csv` est synthétique.

**Contexte** : story **pleine pile**. `kesh-api` : module neuf `routes/bank_import_fingerprint.rs`
(`preview_fingerprint`, `parse_fingerprint_field`, `check_preview_fingerprint`, `FingerprintFormat`, tests U1-U4) ;
`routes/bank_imports.rs` (`preview_fingerprint` de `BankImportPreviewResponse`, champ multipart `previewFingerprint`
et sa garde de doublon, calcul dans `preview` et `preview_csv`, comparaison dans `create` — avant la transaction — et
`create_csv` — après `resolve_csv_profile`, dans la transaction —, `normalize_iban` passée `pub(crate)`) ; `errors.rs`
(`AppError::BankImportPreviewStale`, `409 BANK_IMPORT_PREVIEW_STALE`) ; `routes/mod.rs` ; tests I1-I12, I6-bis
(`tests/bank_imports_e2e.rs`) ; entrée de la `TABLE` de `tests/textes_coherents.rs`. `kesh-i18n` : 24 clés neuves ×
quatre catalogues. Frontend : `BankImportUpload.svelte` (réécrit : `runPreview`, séquence `seq`, `previewReason`,
instantané, `handleProfileChange`, branche `409`, avis, désactivations, `input.value`, `autoMatchedId`, libellés de
l'AC17), `BankProfileSelector.svelte` (prop `disabled`), `BankImportList.svelte`, `routes/(app)/bank-import/+page.svelte`,
`bank-import.api.ts`, `bank-import.types.ts`, Vitest `BankImportUpload.test.ts` (V1-V13, V6-bis) et V11,
`i18n-keys.test.ts` (`sitesTotal` 2032 → 2062), E2E E1 et E2 (`tests/e2e/bank-csv-import.spec.ts`). Textes : manuel
utilisateur FR (`.tex` **et PDF**), `docs/api-external.md` (§ 7, sous-section *Importer un relevé — l'empreinte
d'aperçu*, et § 10), `CHANGELOG.md` (`## [0.13.2] — Non publié`).

⛔ **Axes communs aux trois lentilles** (chacune les déclare exercés ou non) :
1. **L'empreinte, une seule fois.** Une seule sérialisation (DRY, AC8) : aperçu et création appellent-ils la même
   fonction avec les **mêmes** entrées (société, compte, hash, format, profil résolu) ? Une entrée qui diffère entre
   l'aperçu et la création sans que le contenu ait changé (ce qui rendrait tout import `409`) ? Déstructuration
   exhaustive sans `..` ; ce qui est couvert et ce qui ne l'est pas, contre l'AC7. Déterminisme (tableau JSON, pas de
   `Map`).
2. **Où et quand la création compare** (AC8) : CSV après `resolve_csv_profile`, sur la variable `profile` qui sert au
   parse, avant tout parse ; CAMT avant la transaction ; précédence des refus (`400`, `404` compte, `422` doublon CSV,
   `404` profil) puis `409` ; asymétrie CSV / CAMT. Rien d'écrit sur un `409` (transaction, ligne d'audit) ? Un client
   sans empreinte : comportement inchangé (D12) ?
3. **Inventaire clos D1-D19** : chaque divergence de la fiche est-elle fermée comme la colonne de droite le dit, au
   code ? **Cherche un chemin vers la confirmation absent de l'inventaire** (un geste, un état de l'écran, un appel
   d'API) — c'est le finding le plus cher.
4. **L'écran, état par état** (AC1-AC5, AC9, AC16) : séquence (variable ordinaire, jamais `$state`), réponse périmée
   — succès **et** erreur — sans effet ; `reset()` (séquence, `isPreviewing`, `isConfirming`, avis) ; instantané
   (fichier, compte, profil **résolu**, empreinte) ; cases remises à zéro à chaque aperçu ; présélection sans relance ;
   annotation « (auto-détecté) » ; sélecteurs et *Annuler* désactivés pendant la confirmation, actifs pendant tout
   aperçu ; texte de l'indicateur (priorité à `isConfirming`) ; trois sites d'effacement de l'avis ; `isConfirming`
   éteint **avant** la relance non attendue ; `input.value` vidé. Le `$effect` du compte dépend-il d'un état qu'il
   écrit ?
5. **i18n** : 24 clés × 4 (parité), replis Svelte et Rust = texte fr-CH (`i18n-repli-divergent-actif`, `TABLE` de
   `textes_coherents.rs`), préfixe `bank-import-` (`lint-i18n-ownership`), traductions de/it/en justes, terminologie du
   glossaire (`docs/i18n-glossaire.md`) ; `sitesTotal` 2062 recompté aux deux bornes, commentaires masqués, et l'écart
   d'un site avec la fiche (C-27-3-46) ; inventaire AC17 H1-H11 : **reste-t-il un libellé en dur** sur l'écran
   `/bank-import` (les trois composants et la page) hors des non-résolus déclarés ?
6. ⛔ **LE MANUEL — `docs/manual/fr/user-manual.tex` ET `user-manual.pdf`.** Les étapes 3 de *Import dans Kesh* et
   *Import avec un profil existant*, et l'encadré réécrit : chaque affirmation (recalcul « aussitôt », y compris après un
   refus ; cases décochées ; refus d'un profil ou d'un compte modifié ailleurs ; texte de l'avis cité ; fichier modifié
   sur le disque « selon le navigateur » ; *Annuler* garde le profil) est-elle **vraie au code** ? Les phrases laissées
   telles quelles (`:1743`, `:1784`, `:1826` à la base — « redéposez le fichier », « corriger le fichier source ») le
   restent-elles ? **Cherche une phrase du manuel, de `api-external.md`, du README ou du site (`website/`), hors des
   sections touchées, que la story rend fausse** (ex. : une description de la prévisualisation, du sélecteur de profil,
   de la mention « auto-détecté », de l'écran pendant le chargement). Contrôle le **PDF aplati**
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers `/home/gcorbaz/devel/kesh-gate-logs/`)
   — l'apostrophe y est typographique, écrire `.` à sa place dans un motif ; les coupures de mots restent (`réinitia-
   lisation`) — et sa **table des matières** (le `Makefile` de la base fait deux passes fixes, AC12).
7. **CHANGELOG et `api-external.md`** : une seule section `## [0.13.2] — Non publié` ; rien qui promette au-delà du
   code (« depuis l'écran », C-27-3-27) ; « depuis la v0.13.2 » et jamais « Unreleased » ni « à venir » ; l'angle mort
   D12, le cas D17, l'ordre des refus et son asymétrie, la ligne `409` du § 10, au patron des voisines.

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. Le module d'empreinte (bornes de
  `parse_fingerprint_field`, `unwrap_or_default` de la sérialisation), le multipart (doublon, champ vide, ordre des
  champs), les quatre handlers (une branche qui calcule l'empreinte avant une variable modifiée ensuite — encodage
  forcé, profil relu ?), le composant (un geste qui laisse l'indicateur allumé ou la zone masquée ; un état où
  *Confirmer* est visible avec un instantané nul ou d'un autre aperçu ; un `409` pendant que l'utilisateur change de
  compte ; `onSuccess` qui lève ; deux `409` de suite ; un CAMT et la présélection), les types TS contre les DTO Rust.
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Rejoue mentalement les vingt-neuf
  mutations du Dev Agent Record (journal `/home/gcorbaz/devel/kesh-gate-logs/27-3-mutations.log`, diffs sous
  `27-3-mut/`) et cherche une **mutation plausible qui resterait verte** (ex. : l'empreinte de l'aperçu CSV calculée
  sur le profil demandé et non résolu ; `create` comparant après la transaction ; `appendFlags` envoyant une empreinte
  vide ; `reset()` n'incrémentant pas `seq` ; l'instantané pris avant la présélection ; l'avis rendu dans le bloc de
  l'aperçu). Tests qui passent à vide ou dont le montage ne construit pas l'état voulu (`tests-qui-prouvent-moins`) :
  I4 restaure-t-il vraiment le profil entre deux cas ; V3 ordonne-t-il vraiment les réponses ; V13 voit-il chaque
  clé ? L'E2E : indépendance, rejouabilité (noms de profils fixes, deuxième passage sur la même base ?),
  `data-testid` seuls.
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (1 à 17, et 1-bis) contre le code :
  satisfait, partiel, absent. Les tests prescrits (U1-U4, I1-I12, I6-bis, V1-V13, V6-bis, E1, E2) existent-ils et
  prouvent-ils ce qu'ils nomment ? AC14 (chaque mutation exécutée, résultat rapporté). AC15 (`git diff --stat` : ni
  `migrations/`, ni `test-schema/`, ni `post_restore.rs`, ni `Cargo.toml`). **Décomptes recomptés depuis la source**
  (24 clés × 4, `sitesTotal`, tests ajoutés aux deux bornes, nextest / Vitest / E2E déclarés, fichiers touchés) et
  périmètre de chaque décompte. Les choix C-27-3-46 à 50 et les écarts déclarés : justifiés, ou défaut ? Les fichiers
  communs avec la 27-1 (tableau des Dev Notes) : les emplacements prescrits pour s'écarter de ses hunks sont-ils tenus
  (variante d'erreur près de `BankCsvProfileNotFound`, clés après l'auto-détection, bloc `use` intact) ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/27-3-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour
tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les sept axes communs et ceux de ta lentille) — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati du PDF au
même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute : `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `xelatex`,
`git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`, `sqlx`, `cargo` (aucune sous-commande), `npm`, `npx`,
`node`, **`python3`, `python`, `perl`, `ruby`, `bash -c`/`sh -c` et tout autre interpréteur**, `docker`, `gh` en
écriture, aucune requête SQL ni HTTP. Aucun fichier hors du dépôt. Autorisés : lecture, `grep`, `sed -n`, `wc`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`.
