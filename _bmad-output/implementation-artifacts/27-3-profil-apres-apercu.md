# Story 27.3 : changer de profil après l'aperçu recalcule l'aperçu — et le serveur refuse un import jamais prévisualisé

Status: ready-for-dev

<!-- Fiche créée le 2026-10-12 par bmad-create-story, en autonomie (aucune question). Choix au registre
`epic-27-choix-autonomes-27-3.md` (C-27-3-1 à C-27-3-19), à fusionner par l'orchestrateur dans
`epic-27-choix-autonomes.md` (branche de la 27-1). Validation P1 : `27-3-validate-prompt-p1.md`. -->

**Issue** : [#626](https://github.com/guycorbaz/kesh/issues/626) (`bug`, `P2`). **PR : `closes #626`.**
**Epic** : 27 — recette de la v0.13.1, cible **v0.13.2** (décision de Guy, 2026-10-12 : la v0.13.2 ferme aussi
#624, #626 et #634, pour que le nombre de bugs baisse).
**Branche** : `story/27-3-profil-apres-apercu` (worktree `/home/gcorbaz/devel/kesh-27-3`, base `origin/main`
`a2743fd4`). **Port** 3036, bases `kesh_273` / `kesh_e2e_273`.

## Story

En tant que **comptable qui importe un relevé CSV**,
je veux que **la prévisualisation soit toujours celle du profil qui servira à l'import** — recalculée quand je
change de profil, et refusée par le serveur si le profil a changé entre-temps ailleurs,
afin de **ne jamais importer des mouvements lus avec un mapping (colonnes, sens des débits) que je n'ai pas vu**.

## Le défaut, au code

Constat de l'issue, revérifié le 2026-10-12 sur `a2743fd4` :

- `frontend/src/lib/features/bank-import/BankImportUpload.svelte:107-157` — `handleFileSelect` calcule l'aperçu
  **au dépôt** avec `bankProfileId` de cet instant.
- `:248-257` — le sélecteur de profil fait seulement `onChange={(id) => (bankProfileId = id)}` : **rien ne relance
  l'aperçu**, qui reste affiché.
- `:96-105` + `:174` — `handleConfirm` envoie `buildFlags()`, donc le **nouveau** `bankProfileId` : le serveur
  importe avec un mapping jamais prévisualisé (`create_csv`, `crates/kesh-api/src/routes/bank_imports.rs:1794-1809`,
  relit le profil et parse à nouveau).
- Côté serveur, **rien ne lie la confirmation à l'aperçu** : `POST /api/v1/bank-imports` est sans état, il ne sait pas
  ce que l'utilisateur a vu. Même un écran corrigé reste exposé si le profil est **modifié dans un autre onglet**
  (ou par un autre utilisateur) entre l'aperçu et la confirmation : `create_csv` lit le profil **du moment**.

Le contournement documenté par la 26-3 (choix **C-26-3-15**, `docs/manual/fr/user-manual.tex:1697-1698`,
`:1794-1796` et l'encadré `:1801-1807`) — « choisir le profil *avant* de déposer ; sinon *Annuler*, choisir,
redéposer » — **devient faux** et se réécrit (AC12).

## Inventaire clos des chemins qui mènent à la confirmation

La confirmation (`handleConfirm`) n'est atteignable que par le bouton `bank-import-confirm`, rendu seulement
quand `preview` n'est pas `null` (`:354`, `:653-662`). L'aperçu affiché dépend de **cinq entrées** :
le fichier, le compte bancaire, le profil choisi, **le contenu** du profil en base, et l'état de la base (doublons).
Chaque façon de faire diverger l'une d'elles entre l'aperçu et la confirmation est listée ci-dessous ; la colonne
de droite dit ce qui la ferme. **Un chemin absent de ce tableau est un finding.**

| # | Divergence | Aujourd'hui | Après la story |
|---|---|---|---|
| D1 | Changer de profil dans le sélecteur après l'aperçu | aperçu périmé, confirmation avec le nouveau profil (**le défaut**) | aperçu relancé (AC1) ; empreinte serveur (AC6) |
| D2 | Changer de profil **pendant** qu'un aperçu est en cours, ou deux changements rapides (clavier : certains navigateurs émettent `change` à chaque flèche sur un `<select>` fermé) | — (aucun recalcul) | dernier demandé gagne (AC3) |
| D3 | Profil **modifié** (ou renommé) dans un autre onglet / par un autre utilisateur entre l'aperçu et la confirmation | importe avec le contenu modifié | `409 BANK_IMPORT_PREVIEW_STALE` (AC6), recalcul automatique à l'écran (AC9) |
| D4 | Profil **supprimé** ailleurs entre l'aperçu et la confirmation | `404 BANK_CSV_NO_PROFILE_MATCH` (`resolve_csv_profile:716-724`) | inchangé — le refus existe déjà, aucun import |
| D5 | Changer de **compte bancaire** après l'aperçu | `$effect` → `reset()` (`:75-94`) purge aperçu et fichier | inchangé ; **plus** : réponse d'aperçu en vol ignorée (AC3) ; empreinte (compte inclus, AC5) |
| D6 | **Redéposer** un fichier (même compte) | `handleFileSelect` recalcule ; la zone de dépôt n'est visible que sans aperçu (`:259`) | inchangé ; la zone reste masquée pendant un aperçu en cours (AC2) ; empreinte (hash du fichier, AC5) |
| D7 | Compte bancaire **modifié** ailleurs (IBAN) entre l'aperçu et la confirmation | CAMT : autre relevé sélectionné, ou `422` ; CSV : sans effet | empreinte (IBAN normalisé inclus, AC5) |
| D8 | Choisir *Auto-détection* après un aperçu | aucun recalcul | aperçu relancé sans profil (AC1) ; la présélection qui suit ne relance pas (AC4) |
| D9 | Aperçu **en échec** (refus de signe, aucun profil reconnu…), puis changer de profil | rien : il faut redéposer | aperçu relancé avec le fichier retenu (AC1-bis) |
| D10 | Changer de profil **pendant la confirmation** (requête `POST` en vol) | possible (rien n'est désactivé) | sélecteurs désactivés pendant la confirmation (AC3) |
| D11 | *Annuler* pendant un aperçu en cours | — | réponse en vol ignorée (AC3) |
| D12 | Client qui confirme **sans** passer par l'aperçu (clé d'API, script, ancien onglet chargé avant la mise à jour) | importe | inchangé : l'empreinte est **facultative** (C-27-3-9) — **angle mort assumé**, écrit à `docs/api-external.md` (AC13) ; l'écran livré l'envoie toujours |
| D13 | Base qui change entre l'aperçu et la confirmation (un autre import ajoute des doublons, un fichier identique est importé) | la création **recalcule** doublons de fichier (`create_csv:1781-1792`, `create:1108-1117`) et de lignes | inchangé — hors empreinte (C-27-3-12) |
| D14 | Cases de confirmation cochées sur l'ancien aperçu | — | remises à zéro à chaque recalcul (AC2) |

## Acceptance Criteria

### Écran (frontend)

**AC1 — Changer de profil relance l'aperçu.** Quand l'utilisateur change la valeur du sélecteur
`bank-profile-select` (un profil, ou *Auto-détection*) **alors qu'un fichier est retenu** (`selectedFile` non nul)
et qu'un compte est choisi, l'aperçu est **recalculé** : `previewBankImport(selectedFile, selectedAccountId,
{ bankProfileId })` avec la **nouvelle** valeur, et l'ancien aperçu est retiré avant la réponse. Sans fichier
retenu, changer de profil ne déclenche rien (comportement actuel). Le déclencheur est le **geste** de l'utilisateur
(le `onChange` du sélecteur), **jamais** un `$effect` sur `bankProfileId` (cf. AC4).

**AC1-bis — Aussi après un aperçu en échec (D9).** Si le dernier aperçu a échoué (`preview === null`,
`errorCode` posé) et qu'un fichier est retenu, changer de profil relance l'aperçu avec ce fichier : l'utilisateur
n'a plus à redéposer pour essayer un autre profil. Exception : l'erreur `BANK_ACCOUNT_REQUIRED` (aucun fichier
retenu) ne relance rien.

**AC2 — État de l'écran pendant le recalcul.** Pendant qu'un aperçu est en cours (dépôt initial **ou** recalcul) :

- l'ancien aperçu (`bank-import-preview`) n'est **pas** affiché, ni donc le bouton *Confirmer l'import* ;
- la zone de dépôt (`bank-import-drop-zone`) n'est **pas** affichée (aujourd'hui elle l'est au premier dépôt :
  un second fichier déposé pendant le chargement lançait un aperçu concurrent) ;
- l'indicateur `bank-import-loading` est affiché, `role="status"`, avec le texte
  `bank-import-labels-preview-recomputing` (« Recalcul de la prévisualisation avec le profil choisi… ») quand il
  s'agit d'un **recalcul** (AC1, AC1-bis, AC9), et le « Chargement… » actuel au dépôt d'un fichier ;
- l'erreur précédente (`bank-import-error`) est effacée ;
- **les cases et le choix de doublons** (`confirmBalanceMismatch`, `confirmDuplicateFile`, `confirmDuplicateLines`
  remis à `skip`, `confirmPartialImport`, `confirmEncodingMismatch`) sont **remis à zéro** — les avertissements du
  nouvel aperçu ne sont pas ceux de l'ancien (même raison que F1 Pass 3, `:114-125`).

**AC3 — Le dernier demandé gagne ; rien de périmé ne s'affiche (D2, D5, D10, D11).** Chaque demande d'aperçu porte
un numéro de séquence ; une réponse (succès **ou** erreur) dont le numéro n'est plus le dernier est **ignorée** —
elle ne pose ni `preview`, ni l'erreur, ni la présélection, ni ne termine l'indicateur de chargement. `reset()`
(*Annuler*, changement de compte, succès de l'import) **invalide** la séquence en cours. Pendant une **confirmation**
en vol, les sélecteurs de compte (`bank-account-select`) et de profil (`bank-profile-select`) sont **désactivés**
(`disabled`) ; pendant un **aperçu**, ils restent actifs (C-27-3-3).

**AC4 — La présélection automatique ne relance pas l'aperçu.** Le comportement KF #70 (`:135-142`) est conservé :
après un aperçu demandé sur *Auto-détection* et rendu `autoMatched`, `bankProfileId` prend `csvProfileMatch.profileId`
**sans** nouvel appel à `previewBankImport` (un seul appel pour un dépôt).

**AC5 — La confirmation envoie l'aperçu affiché, pas l'état du sélecteur.** `handleConfirm` construit sa requête
depuis un **instantané** pris à la réception de l'aperçu affiché : le fichier, le compte, le profil **résolu** par
cet aperçu (`csvProfileMatch.profileId` pour un CSV ; aucun `bankProfileId` pour un CAMT — le serveur l'ignore
déjà), l'**empreinte** `previewFingerprint` (AC6), et les cases de confirmation courantes.
`createBankImport` / `appendFlags` (`bank-import.api.ts:28-46`) transmettent `previewFingerprint` en champ
multipart quand il est fourni.

### Serveur (défense — un client périmé ne peut pas importer avec un mapping jamais vu)

**AC6 — L'aperçu rend une empreinte ; la confirmation la vérifie.** `POST /api/v1/bank-imports/preview` rend un
champ `previewFingerprint` (64 caractères hexadécimaux minuscules), pour un CSV **et** pour un CAMT.053.
`POST /api/v1/bank-imports` accepte un champ multipart **facultatif** `previewFingerprint` :

- **présent et égal** à l'empreinte recalculée par la création sur ses propres entrées → l'import suit son cours ;
- **présent et différent** → **`409 BANK_IMPORT_PREVIEW_STALE`**, aucun import, aucune transaction, aucune ligne
  d'audit ; message i18n `bank-import-errors-preview-stale` ;
- **absent** → comportement actuel, inchangé (D12, C-27-3-9) ;
- **mal formé** (longueur ≠ 64, caractère hors `[0-9a-f]`) → `400 VALIDATION_ERROR` ; **dupliqué** dans le
  multipart → `400` (parité avec les autres champs, `parse_multipart:373-545`).

**AC7 — Ce que couvre l'empreinte, et rien d'autre.** SHA-256 (hex) d'une sérialisation déterministe
(`serde_json` d'un tableau ordonné) de :

1. la balise de domaine `"kesh-bank-import-preview/v1"` ;
2. `company_id` ;
3. `bank_account.id` et `normalize_iban(&bank_account.iban)` (`bank_imports.rs:815-820`) ;
4. le hash SHA-256 du fichier (`compute_sha256_hex`, `:1346`) ;
5. le format détecté (`"csv"` / `"camt053"`) ;
6. pour un CSV, le **profil résolu** — tous les champs de `kesh_db::entities::bank_profile::BankProfile`
   **sauf** `created_at` et `updated_at`, obtenus par **déstructuration exhaustive** (sans `..`, les deux horodatages
   liés à `_`) : `id`, `company_id`, `bank_name`, `filename_pattern`, `column_mapping_json` (chaîne brute),
   `date_format`, `decimal_separator`, `field_separator`, `encoding`, `header_row_count`, `debit_sign`. Un champ
   ajouté plus tard à `BankProfile` **casse la compilation** de la fonction d'empreinte au lieu d'en être absent en
   silence. Pour un CAMT, `null` à cette place.

L'empreinte n'est **pas un secret** et n'authentifie rien : elle détecte une divergence, elle ne protège pas d'un
client malveillant, qui a de toute façon le droit d'importer (C-27-3-7). Elle **n'inclut pas** l'état de la base
(doublons, D13), ni les cases de confirmation, ni le nom du fichier (le profil **résolu** y est déjà ; deux aperçus
sur *Auto-détection* puis explicite donnent la même empreinte si le profil résolu est le même — AC8).

**AC8 — Où la création calcule et compare.** Une seule fonction calcule l'empreinte, appelée par `preview`,
`preview_csv`, `create` et `create_csv` — **jamais** deux sérialisations écrites deux fois (DRY ; une divergence
entre elles rendrait tout import `409`).

- **CSV** (`create_csv`) : **après** `resolve_csv_profile` (`:1795-1802`), sur le profil **lu dans la transaction**
  — celui-là même qui sert au parse (`:1804-1809`). Pas de fenêtre entre la comparaison et l'usage.
- **CAMT** (`create`) : après la détection du format, avant l'ouverture de la transaction (aucun profil en jeu).
- **Précédence** : les refus qui précèdent la résolution du profil restent prioritaires — `400` multipart,
  `404 BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`, et pour un CSV le `422 BANK_IMPORT_DUPLICATE_FILE` non confirmé
  (`:1781-1792`) et le `404 BANK_CSV_NO_PROFILE_MATCH` (D4). Le `409` vient ensuite, **avant** tout parse.
- Un aperçu sur *Auto-détection* (profil `X` résolu par le nom du fichier) suivi d'une création avec
  `bankProfileId = X` explicite donne la **même** empreinte : `201`.

### Réaction de l'écran au refus

**AC9 — Sur `409 BANK_IMPORT_PREVIEW_STALE`, l'écran recalcule et le dit.** Si la confirmation reçoit ce code,
l'écran **relance l'aperçu** (même fichier, même compte, profil de l'instantané — AC5) et affiche, au-dessus de
l'aperçu, l'avis `bank-import-preview-recomputed` (`role="status"`, clé `bank-import-labels-preview-recomputed` :
« Le profil a été modifié depuis la prévisualisation : elle a été recalculée. Vérifiez-la avant de confirmer. »).
**Aucune nouvelle confirmation automatique** : l'utilisateur reconfirme. `onSuccess` n'est pas appelé. L'avis
disparaît au prochain geste de l'utilisateur (dépôt, changement de profil ou de compte, *Annuler*, nouvelle
confirmation). Si ce recalcul échoue (profil devenu inadapté : refus de signe, par exemple), l'erreur s'affiche
**avec** l'avis.

### Textes

**AC10 — i18n, quatre locales.** Trois clés neuves, dans `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`,
placées après `bank-import-labels-bank-profile-auto-detect-placeholder` (fr-CH `:1094` — **hors** des hunks de la
27-1, qui ajoute après `:1057` et `:1078`) :

| Clé | fr-CH |
|---|---|
| `bank-import-errors-preview-stale` | Le profil, le fichier ou le compte a changé depuis la prévisualisation : recalculez-la avant d'importer. |
| `bank-import-labels-preview-recomputing` | Recalcul de la prévisualisation avec le profil choisi… |
| `bank-import-labels-preview-recomputed` | Le profil a été modifié depuis la prévisualisation : elle a été recalculée. Vérifiez-la avant de confirmer. |

Les replis des `i18nMsg` sont **identiques** au texte fr-CH (garde du moissonneur). Préfixe `bank-import-` :
conforme à `lint-i18n-ownership` (feature `bank-import`). Le repli serveur de `t()` est le texte fr-CH.

**AC11 — Borne `sitesTotal`.** Deux sites `i18nMsg(` neufs dans `BankImportUpload.svelte` (23 aujourd'hui,
`grep -o "i18nMsg(" … | wc -l`) → **25** ; `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` passe de
**2032 à 2034** sur la base `a2743fd4`, avec son commentaire ventilé (le patron du fichier). ⚠️ La 27-1 le porte à
2041 : à la fusion, **2043** — l'orchestrateur additionne les deltas, il ne prend pas l'un des deux nombres.
Recompter au dernier commit, ne pas recopier ce chiffre.

**AC12 — Manuel utilisateur (FR, `.tex` **et** PDF régénéré).** `docs/manual/fr/user-manual.tex` :

- `:1697-1698` (§ *Import dans Kesh*, étape 3) : retirer « \emph{avant} de déposer le fichier » — le profil se
  choisit avant ou après le dépôt ;
- `:1794-1796` (§ *Import avec un profil existant*, étape 3) : idem ; garder la phrase sur *Auto-détection* et la
  mention « auto-détecté » (toujours vraie, AC4) ;
- l'encadré `keshwarning` `:1801-1807` est **remplacé** par un texte vrai au code livré :
  la prévisualisation est toujours celle du profil choisi ; changer de profil après le dépôt la **recalcule**
  aussitôt (« Recalcul de la prévisualisation… »), y compris après un refus — inutile de redéposer ; les
  avertissements déjà confirmés sont **à confirmer de nouveau** ; si le profil est modifié ailleurs (autre onglet,
  autre utilisateur) entre la prévisualisation et la confirmation, Kesh **n'importe pas** : il recalcule la
  prévisualisation et l'annonce ; vérifier, puis confirmer de nouveau. *Annuler* garde le profil choisi (vrai,
  `reset()` ne touche pas `bankProfileId`).
- **Restent vraies, ne pas toucher** : `:1743` et `:1784` (« réglez le sens des débits du profil, puis redéposez le
  fichier ») — modifier un profil se fait dans *Administration*, hors de l'écran d'import, et re-choisir la même
  valeur du sélecteur n'émet aucun `change` ; le message serveur `bank-import-csv-errors-amount-sign-refused-profile`
  (`errors.rs:2367`, ftl `:1120`) idem (C-27-3-14).
- Régénérer **les PDF** (`make fr` dans `docs/manual/`, sous `flock …/.gate.lock`) et contrôler le PDF **aplati** :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` ne contient plus « ne la recalcule pas »
  ni « avant de déposer », et contient « recalcul ».
- Manuels DE/IT/EN : vides (README seuls) — rien à faire.

**AC13 — CHANGELOG et doc d'API.**

- `CHANGELOG.md`, section `## [0.13.2] — Non publié` (à **créer** sur cette branche si absente : elle n'existe pas
  sur `a2743fd4` ; la 27-1 la crée aussi → conflit de fusion à résoudre par union), sous `### Corrigé` : changer de
  profil après la prévisualisation la recalcule ; un import ne se fait plus avec un profil modifié depuis la
  prévisualisation (`409 BANK_IMPORT_PREVIEW_STALE`, champ `previewFingerprint`) ([#626](…/issues/626)).
- `docs/api-external.md` : sous § 7 (après la sous-section *Annuler un rapprochement bancaire* ou en sous-section
  propre *Importer un relevé*), décrire `previewFingerprint` (rendu par l'aperçu, facultatif à la création,
  conseillé ; `409` / `400`), et l'**angle mort assumé** D12 : une création sans empreinte n'est pas contrôlée.

### Tests

**AC14 — Chaque AC a un test qui rougirait** (§ Tests ci-dessous), et les mutations M1-M16 rougissent.

**AC15 — Aucune migration, aucun bump.** Ni fichier sous `crates/kesh-db/migrations/`, ni `kesh_version_min_required`,
ni version Cargo (la release 0.13.2 s'en charge). Une migration au diff signale une dérive.

## Tasks / Subtasks

- [ ] **T0 — Préalables** (AC15)
  - [ ] Bases `kesh_273` / `kesh_e2e_273` (migrations du worktree, seed `scripts/seed-dev-db.sql`) ; `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-27-3/target` ; `node_modules` par `cp -al` depuis le dépôt principal (consignes 6, 8, 11).
- [ ] **T1 — Empreinte serveur** (AC6, AC7, AC8)
  - [ ] Nouveau module `crates/kesh-api/src/routes/bank_import_fingerprint.rs` (déclaré dans `routes/mod.rs`) — **hors** de `bank_imports.rs` pour limiter le recouvrement avec la 27-1 : `pub(crate) fn preview_fingerprint(company_id, &BankAccount, file_hash: &str, format: FingerprintFormat, profile: Option<&BankProfile>) -> String`, déstructuration exhaustive de `BankProfile` ; `pub(crate) fn parse_fingerprint_field(&str) -> Result<String, AppError>` (64 hex minuscules). Doc-comments `///` (ce que couvre l'empreinte, ce qu'elle ne couvre pas, pourquoi pas de secret).
  - [ ] `BankImportPreviewResponse` (`bank_imports.rs:54`) : champ `preview_fingerprint: String` ; rempli par `preview` (CAMT, `:1024`) et `preview_csv` (`:1738`).
  - [ ] `MultipartFields` + `parse_multipart` : `preview_fingerprint: Option<String>`, garde de doublon, validation de forme.
  - [ ] `create` (CAMT) et `create_csv` (après `resolve_csv_profile`) : comparaison → `AppError::BankImportPreviewStale`.
  - [ ] `errors.rs` : variante `BankImportPreviewStale` (placée près de `BankCsvProfileNotFound`, `:740`, **pas** après `BankImportDuplicateFile` où la 27-1 insère) ; mapping `409`, code `BANK_IMPORT_PREVIEW_STALE`, `t("bank-import-errors-preview-stale", …)` (placé près de `:2237`).
  - [ ] Tests unitaires dans le module neuf (U1-U4) ; tests d'intégration dans `crates/kesh-api/tests/bank_imports_e2e.rs` (I1-I10).
- [ ] **T2 — Client API et types** (AC5, AC6)
  - [ ] `bank-import.types.ts` : `previewFingerprint: string` dans `BankImportPreviewResponse`.
  - [ ] `bank-import.api.ts` : `BankImportFlags.previewFingerprint?: string`, ajouté par `appendFlags` ; test dans `bank-import.api.test.ts` (V11).
- [ ] **T3 — Écran** (AC1-AC5, AC9)
  - [ ] `BankImportUpload.svelte` : séparer `runPreview(file, reason)` (séquence, retrait de l'ancien aperçu, remise à zéro des cases, instantané à la réception, présélection AC4) des gestes `handleFileSelect` (dépôt) et `handleProfileChange(id)` (sélecteur) ; états `isPreviewing` / `isConfirming` (l'indicateur et le `disabled` du bouton en dérivent) ; `reset()` invalide la séquence ; `handleConfirm` depuis l'instantané, branche `BANK_IMPORT_PREVIEW_STALE` → `runPreview(…, 'stale')` + avis.
  - [ ] `BankProfileSelector.svelte` : prop `disabled` (défaut `false`) ; le `<select>` de compte reçoit `disabled={isConfirming}`.
  - [ ] Mettre à jour le commentaire d'en-tête (machine d'état) et le commentaire L3 de `handleConfirm` (`:160-167`).
  - [ ] Nouveau `BankImportUpload.test.ts` (V1-V10), patron `BankProfileForm.test.ts` (mocks avant import).
- [ ] **T4 — i18n** (AC10, AC11) : trois clés × quatre locales ; `sitesTotal` 2032 → 2034 recompté.
- [ ] **T5 — E2E** (AC1, AC6, AC9) : deux tests dans `frontend/tests/e2e/bank-csv-import.spec.ts` (E1, E2).
- [ ] **T6 — Manuel** (AC12) : `.tex`, `make fr`, PDF aplati contrôlé.
- [ ] **T7 — CHANGELOG, `docs/api-external.md`** (AC13).
- [ ] **T8 — Mutations** M1-M16 : chacune appliquée, test rouge constaté, restaurée (et le fichier **touché** après restauration : cargo garde sinon le binaire muté) ; tableau au Dev Agent Record.
- [ ] **T9 — Gates** au dernier commit de code : `scripts/test-fast.sh` (sous `flock`), `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`, **E2E complet** (8 échecs attendus de `docs/testing.md`, jugés fichier par fichier). Base remise à zéro avant le gate complet.

## Tests prescrits

### Unitaires Rust (module `bank_import_fingerprint`)

- **U1** déterminisme : deux appels mêmes entrées → même empreinte ; 64 hex minuscules.
- **U2** sensibilité, **table** : pour chaque champ de `BankProfile` hors horodatages (11), une copie qui ne diffère que
  par lui → empreinte différente ; idem pour `company_id`, compte (`id`, IBAN), hash du fichier, format.
- **U3** insensibilité : `created_at` / `updated_at` changés → même empreinte ; IBAN avec espaces / minuscules →
  même empreinte (normalisé).
- **U4** `parse_fingerprint_field` : accepte 64 hex minuscules ; refuse 63, 65, majuscules, `g`, vide.

### Intégration (`crates/kesh-api/tests/bank_imports_e2e.rs`, `#[sqlx::test(migrations = "../kesh-db/test-schema")]`)

Helpers existants : `send_csv` (`:1607`), `create_test_bank_profile_csv` (`:690`), `build_csv_multipart` (`:711`).

- **I1** l'aperçu CSV **et** CAMT rend `previewFingerprint` (64 hex) ; deux aperçus identiques → même valeur.
- **I2** création avec l'empreinte de l'aperçu → `201`.
- **I3** création **sans** empreinte → `201` (compatibilité D12).
- **I4** **table** : pour chaque colonne du profil modifiable — `bank_name`, `filename_pattern`, `column_mapping_json`,
  `date_format`, `decimal_separator`, `field_separator`, `encoding`, `header_row_count`, `debit_sign` —, un `UPDATE`
  SQL entre l'aperçu et la création → `409 BANK_IMPORT_PREVIEW_STALE` ; `bank_imports`, `bank_transactions` et
  `audit_log` **inchangés** (compter avant / après).
- **I5** création avec un **autre** `bankProfileId` explicite que celui de l'aperçu → `409`.
- **I6** aperçu sur *Auto-détection* (profil à `filename_pattern` reconnu), création avec l'id résolu → `201`.
- **I7** autre fichier (octets différents) avec l'empreinte du premier → `409` ; autre compte bancaire de la même
  société → `409`.
- **I8** empreinte mal formée → `400` ; champ dupliqué → `400`.
- **I9** seul `updated_at` modifié entre aperçu et création → `201`.
- **I10** précédence : fichier déjà importé **et** empreinte périmée, sans `confirmDuplicateFile` → `422
  BANK_IMPORT_DUPLICATE_FILE` (AC8) ; CAMT avec empreinte d'un autre fichier → `409`.

### Vitest (`frontend/src/lib/features/bank-import/BankImportUpload.test.ts`, neuf)

Mocks : `./bank-import.api` (`previewBankImport`, `createBankImport`), `./bank-profile.api` (`listBankProfiles` → deux
profils), `$lib/shared/utils/i18n.svelte` (repli), et un `File` synthétique.

- **V1** aperçu avec le profil A, puis choix de B → `previewBankImport` rappelé avec `bankProfileId: B` ; l'aperçu
  affiché est celui de B ; *Confirmer* envoie `bankProfileId: B` et l'empreinte de B.
- **V2** pendant le recalcul (promesse retenue) : `bank-import-preview` et `bank-import-confirm` absents,
  `bank-import-drop-zone` absente, `bank-import-loading` porte le texte de recalcul.
- **V3** course : A → B → C rapides, réponses rendues dans l'ordre C, A, B → l'aperçu affiché est C, et l'indicateur
  ne s'éteint qu'à la réponse C.
- **V4** *Auto-détection* + réponse `autoMatched` → **un seul** appel à `previewBankImport`, sélecteur sur le profil
  présélectionné.
- **V5** aperçu en erreur (`BANK_CSV_AMOUNT_SIGN_MISMATCH`), puis choix d'un autre profil → aperçu relancé avec le
  même fichier, erreur effacée.
- **V6** *Confirmer* → `409 BANK_IMPORT_PREVIEW_STALE` → `previewBankImport` rappelé, avis
  `bank-import-preview-recomputed` visible, `createBankImport` appelé **une** fois, `onSuccess` non appelé.
- **V7** case `confirm-duplicate-file` cochée, puis changement de profil → la case du nouvel aperçu est **décochée**
  et *Confirmer* désactivé.
- **V8** changement de compte pendant un aperçu en vol → la réponse tardive n'affiche rien.
- **V9** pendant la confirmation (promesse retenue) : `bank-account-select` et `bank-profile-select` `disabled`.
- **V10** *Annuler* pendant un aperçu en vol → rien ne s'affiche à la réponse.
- **V11** (`bank-import.api.test.ts`) `createBankImport(…, { previewFingerprint })` ajoute le champ ; absent sinon.

### E2E (`frontend/tests/e2e/bank-csv-import.spec.ts`)

Patron des tests 26-1 (`:315-404`, `createProfileViaUi`, `UBS_COLUMNS`, `importTotal`). ⚠️ La fixture
`ubs_synthetique.csv` a déjà été **importée** par le test 26-1 (`:315`) : la réutiliser telle quelle ferait
apparaître l'avertissement « fichier déjà importé » et ignorer les 11 lignes comme doublons. Dériver le contenu
**en mémoire** (`buffer` de `setInputFiles`) en remplaçant l'année des dates (`2026-` → `2027-` pour E1, `2028-`
pour E2 ; vérifier que l'année n'apparaît nulle part ailleurs dans la fixture) et un nom de fichier propre à chaque
test (C-27-3-17). Noms de profils propres (`bank_name` unique par société).

- **E1** deux profils (« 27-3 moins » `negative`, « 27-3 sans signe » `positive`) ; choisir *sans signe*, déposer →
  refus `BANK_CSV_AMOUNT_SIGN_MISMATCH`, aucun aperçu ; choisir *moins* **sans redéposer** → aperçu, 11 transactions ;
  confirmer → l'import porte 11 montants dont 9 négatifs (somme en centimes, comme `:345-352`).
- **E2** profil *moins*, dépôt → aperçu de 11 ; modifier le profil **par l'API** (`PUT /api/v1/bank-profiles/{id}`,
  `debitSign: 'positive'`) — l'« autre onglet » ; *Confirmer* → avis `bank-import-preview-recomputed` visible, refus de
  signe affiché, `importTotal` inchangé.

## Mutations (T8) — chacune doit faire rougir au moins le test nommé

| # | Mutation | Rougit |
|---|---|---|
| M1 | `handleProfileChange` ne relance plus l'aperçu (pose seulement `bankProfileId`) | V1, V5, E1 |
| M2 | relance par `$effect` sur `bankProfileId` au lieu du geste | V4 (deux appels) |
| M3 | garde de séquence retirée (toute réponse s'applique) | V3, V8, V10 |
| M4 | l'ancien aperçu n'est pas retiré au début du recalcul | V2 |
| M5 | cases non remises à zéro au recalcul | V7 |
| M6 | `handleConfirm` envoie `bankProfileId` du sélecteur au lieu de l'instantané | V1 (variante : sélecteur changé pendant un recalcul en vol, confirmer impossible) — à défaut, I5 côté serveur |
| M7 | branche `BANK_IMPORT_PREVIEW_STALE` retirée (erreur affichée comme une autre) | V6, E2 |
| M8 | sélecteurs non désactivés pendant la confirmation | V9 |
| M9 | `appendFlags` n'envoie pas `previewFingerprint` | V11 |
| M10 | création : comparaison d'empreinte retirée | I4, I5, I7, E2 |
| M11 | empreinte : `debit_sign` lié à `_` (champ omis) | U2, I4 (ligne `debit_sign`) |
| M12 | empreinte : `column_mapping_json` omis | U2, I4 |
| M13 | empreinte : hash du fichier omis | U2, I7 |
| M14 | empreinte : compte omis | U2, I7 |
| M15 | empreinte : horodatages **inclus** | U3, I9 |
| M16 | création CSV : comparaison **avant** `resolve_csv_profile`, sur un profil relu hors transaction — ou empreinte calculée sur le `bankProfileId` demandé au lieu du profil résolu | I6 |

Une mutation qui survit se documente (équivalente, ou test ajouté), jamais en silence.

## Dev Notes

### Ce qu'il faut préserver

- **KF #70** : présélection `autoMatched` (`:135-142`), annotation « auto-détecté » du sélecteur
  (`BankProfileSelector.svelte:53-55`, alimentée par `preview?.csvProfileMatch?.profileId`).
- **F1 Pass 3** : remise à zéro des cases à chaque nouveau fichier (`:114-125`) — étendue au recalcul.
- **Purge au changement de compte** (`$effect` `:75-81`) : conservée ; elle s'ajoute à la séquence.
- **Refus de signe** (Story 26-1) : `errorDetails` (`:51`, `:55-64`) posé et effacé avec `errorCode`, y compris par
  la branche `409` (l'avis n'est pas `errorDetails`).
- Les tests d'intégration existants créent des imports **sans** aperçu préalable : l'empreinte facultative les laisse
  verts (I3). Les E2E existants (`bank-import.spec.ts`, `bank-import-confirms.spec.ts`, `bank-csv-import.spec.ts`)
  passent par l'aperçu, donc enverront l'empreinte : ils doivent rester verts sans modification — un rouge y est un
  défaut de l'empreinte (preview et création qui ne calculent pas la même chose), pas du test.
- `parse_multipart` vide les champs inconnus (`:512-520`) : un client qui envoie déjà `previewFingerprint` à une
  v0.13.1 n'obtient pas d'erreur — sans intérêt pratique, mais aucune rupture dans l'autre sens.

### Pourquoi une empreinte sans état, et pas un jeton

Un jeton d'aperçu stocké (table, cache) imposerait une migration, une expiration et un nettoyage, et ne fermerait
rien de plus : ce qui doit être identique entre l'aperçu et la création, ce sont **les entrées du calcul**, que la
création possède toutes. Les recalculer et comparer suffit, sans état (C-27-3-7). Un HMAC n'apporterait rien : la
menace est le client **périmé**, pas le client malveillant (qui a le rôle d'importer).

### Fichiers communs avec la 27-1 (non mergée, `story/27-1-supprimer-import`)

| Fichier | 27-1 | 27-3 | Risque de fusion |
|---|---|---|---|
| `crates/kesh-api/src/routes/bank_imports.rs` | `DELETE` après `detail` (~`:1339`), imports (`:9`, `:33`) | champ de réponse `:54`, multipart `:296-545`, `preview`/`create`/`preview_csv`/`create_csv` | hunks distincts ; vérifier le bloc `use` |
| `crates/kesh-api/src/errors.rs` | variantes `:216`, `:693` ; mappings `:1496`, `:2180`, `:2466` | variante près de `:740`, mapping près de `:2237` | faible si l'emplacement prescrit est tenu |
| `crates/kesh-i18n/locales/*/messages.ftl` | après `:1057`, `:1078`, `:1248`, `:2362`, `:11` | après `:1094` | faible |
| `docs/manual/fr/user-manual.tex` + `.pdf` | `:248`, `:270`, `:1640`, insertion de 40 lignes après `:1718`, `:1870` | `:1697-1698`, `:1794-1807` | `.tex` : faible ; **PDF : conflit binaire certain** → régénérer après fusion |
| `CHANGELOG.md` | crée `## [0.13.2] — Non publié` | idem | conflit certain → union |
| `docs/api-external.md` | route `DELETE` | `previewFingerprint` | à vérifier |
| `frontend/src/lib/shared/i18n-keys.test.ts` | `sitesTotal` 2032 → 2041 | 2032 → 2034 | conflit certain → **2043**, commentaires des deux |
| `frontend/src/lib/features/bank-import/bank-import.api.ts` (+ `.test.ts`) | `deleteBankImport` | `previewFingerprint` | faible |
| `crates/kesh-api/tests/bank_imports_e2e.rs` | +327 lignes | I1-I10 | ajout en fin de fichier des deux côtés → conflit probable, union |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | section `epic-27` | section `epic-27` créée ici aussi | conflit certain → union |

`BankImportUpload.svelte`, `BankProfileSelector.svelte`, le module d'empreinte et `bank-csv-import.spec.ts` ne sont
pas touchés par la 27-1.

### Pièges

- **Svelte 5** : `bind:value` du `<select>` de compte et le `value={…}`/`onchange` du sélecteur de profil — garder le
  sélecteur de profil **non lié** (`value` + `onChange`), c'est ce qui permet de distinguer le geste (AC1) de
  l'affectation programmatique (AC4).
- `isLoading` sert aujourd'hui aux deux requêtes ; une réponse d'aperçu périmée qui l'éteint en `finally` rallumerait
  le bouton pendant que le dernier aperçu est en vol — d'où l'extinction conditionnée à la séquence (AC3, V3).
- Le texte « Chargement… » (`:305`) est en dur : hors périmètre (pas de nouvelle clé pour lui), mais l'élément porte
  désormais `role="status"`.
- `serde_json::to_vec(&json!([...]))` est déterministe pour un tableau ; ne pas sérialiser un `HashMap`.
- Après une mutation restaurée par `cp`/`mv`, **toucher** le fichier (mémoire « mutation restaurée, binaire périmé »).

### Project Structure Notes

Module neuf sous `crates/kesh-api/src/routes/` (le crate n'a pas de sous-dossier par domaine) ; test Vitest à côté du
composant (patron du dossier). Aucune dépendance neuve (`sha2`, `serde_json` déjà présents dans `kesh-api`).

### References

- Issue #626 ; choix C-26-3-15 (`epic-26-choix-autonomes.md:1045-1057`) ; fiche 26-3 `:125`, `:996`.
- `frontend/src/lib/features/bank-import/BankImportUpload.svelte`, `BankProfileSelector.svelte`, `bank-import.api.ts`,
  `bank-import.types.ts:29-40`.
- `crates/kesh-api/src/routes/bank_imports.rs` (lignes citées ci-dessus), `crates/kesh-api/src/errors.rs:2237-2251`,
  `crates/kesh-db/src/entities/bank_profile.rs:26-46`.
- `CLAUDE.md` § Review Iteration Rule (manuel nommé, axes déclarés, sites non résolus, propagation, recompte), § Test
  Locally First (E2E au dernier commit de code), § Issue Tracking Rule (`closes #626` dans la PR).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-12 — Fiche créée (bmad-create-story, Opus 5.5, autonomie) : 16 AC (AC1-AC15 + AC1-bis), tâches T0-T9,
  tests U1-U4 / I1-I10 / V1-V11 / E1-E2, mutations M1-M16, inventaire D1-D14 ; choix C-27-3-1 à 19. Validation P1 à
  lancer (`27-3-validate-prompt-p1.md`).
