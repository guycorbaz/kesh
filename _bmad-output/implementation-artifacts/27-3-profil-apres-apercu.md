# Story 27.3 : changer de profil après l'aperçu recalcule l'aperçu — et le serveur refuse un import dont le profil, le compte ou le fichier a changé depuis l'aperçu

Status: ready-for-dev

<!-- Fiche créée le 2026-10-12 par bmad-create-story, en autonomie (aucune question). Choix au registre
`epic-27-choix-autonomes-27-3.md` (C-27-3-1 à C-27-3-45), à fusionner par l'orchestrateur dans
`epic-27-choix-autonomes.md` (branche de la 27-1). Validation P1 : `27-3-validate-prompt-p1.md` ;
P2 : `27-3-validate-prompt-p2.md` ; P3 : `27-3-validate-prompt-p3.md` ; P4 ciblée : `27-3-validate-prompt-p4-ciblee.md`. -->

**Issue** : [#626](https://github.com/guycorbaz/kesh/issues/626) (`bug`, `P2`). **PR : `closes #626`.**
**Epic** : 27 — recette de la v0.13.1, cible **v0.13.2** (décision de Guy, 2026-10-12 : la v0.13.2 ferme aussi
#624, #626 et #634, pour que le nombre de bugs baisse).
**Branche** : `story/27-3-profil-apres-apercu` (worktree `/home/gcorbaz/devel/kesh-27-3`, base `origin/main`
`a2743fd4`). **Port** 3036, bases `kesh_273` / `kesh_e2e_273`.

## Story

En tant que **comptable qui importe un relevé CSV**,
je veux que **la prévisualisation soit toujours celle du profil qui servira à l'import** — recalculée quand je
change de profil, et refusée par le serveur si le profil, le compte ou le fichier a changé entre-temps,
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
| D11 | *Annuler* pendant un aperçu en cours | — | **inatteignable** : le bouton `bank-import-cancel` (`:665`) est dans le bloc `{#if preview}` (`:354`), que l'AC2 retire pendant tout aperçu ; la sortie pendant un aperçu est le changement de compte (D5), qui passe par `reset()` (AC3). Aucun *Annuler* n'est ajouté à l'état de chargement (C-27-3-21) |
| D12 | Client qui confirme **sans** passer par l'aperçu (clé d'API, script, ancien onglet chargé avant la mise à jour) | importe | inchangé : l'empreinte est **facultative** (C-27-3-9) — **angle mort assumé**, écrit à `docs/api-external.md` (AC13) ; l'écran livré l'envoie toujours |
| D13 | Base qui change entre l'aperçu et la confirmation (un autre import ajoute des doublons, un fichier identique est importé, ou — après la fusion de la 27-1 — un import est **supprimé**, ce qui fait disparaître l'avertissement « fichier déjà importé ») | la création **recalcule** doublons de fichier (`create_csv:1781-1792`, `create:1108-1119`) et de lignes | inchangé — hors empreinte (C-27-3-12) |
| D14 | Cases de confirmation cochées sur l'ancien aperçu | — | remises à zéro à chaque recalcul (AC2) |
| D15 | *Annuler* **pendant la confirmation**, puis la réponse revient en `409` | *Annuler* actif (`onclick={reset}`, seul *Confirmer* dépend de `isLoading`, `:658`) ; la branche `409` prescrite aurait relancé l'aperçu depuis l'instantané **après** l'annulation | *Annuler* désactivé pendant la confirmation, comme les sélecteurs (AC3, C-27-3-21) — le chemin n'existe plus |
| D16 | Après un aperçu en **échec**, re-choisir **le même fichier** par « cliquer pour parcourir » (le manuel dit « redéposez le fichier », `user-manual.tex:1743`, `:1784`) | l'`<input type=file>` reste monté (zone visible, `preview` nul) et garde sa valeur : un navigateur n'émet pas `change` pour une sélection identique — **aucun aperçu** ; le glisser-déposer, lui, fonctionne | `handleFileInput` vide `input.value` après lecture du fichier (AC16, C-27-3-26) ; l'AC2 démonte en outre la zone pendant tout aperçu, ce qui remonte un `<input>` vierge |
| D17 | Un client d'API envoie l'empreinte **sans** `bankProfileId` à la création, alors que le profil résolu par la création peut différer de celui de l'aperçu : un **autre** profil à motif concordant est modifié ou **créé** entre-temps (il passe en tête du tri), ou l'aperçu portait un `bankProfileId` **explicite** B que la création n'envoie pas (la création résout alors par le nom du fichier : un autre profil, ou aucun) | — | `find_matching_profiles_for_filename` trie par `updated_at DESC` (`crates/kesh-db/src/repositories/bank_profiles.rs:180`) : le profil résolu change → `409`, **légitime** (le mapping appliqué peut différer) ; aucun profil concordant → `404 BANK_CSV_NO_PROFILE_MATCH`. Deux profils concordants de **même** `updated_at` ne sont pas retenus comme cas : la colonne est un `DATETIME(6)` (microseconde, `20260505000001_bank_profiles.sql:34`), l'égalité supposerait un seul `UPDATE` multi-lignes (C-27-3-39). L'écran n'est pas concerné : il envoie l'id résolu (AC5). Écrit à `docs/api-external.md` (AC13) |
| D18 | Fichier source **modifié sur le disque** entre l'aperçu et la confirmation (le manuel invite à « corriger le fichier source », `user-manual.tex:1826`) : l'objet `File` d'un `<input>` est lu paresseusement, la confirmation part avec d'autres octets **ou ne part pas**, selon le navigateur — Firefox relit le fichier ; Chrome échoue l'envoi en `ERR_UPLOAD_FILE_CHANGED`, qui tombe en `NETWORK_ERROR` (`api-client.ts:287-296`, « Impossible de contacter le serveur… »), **sans** `409`. *Comportement des navigateurs relevé à la validation P2, raisonné, non reproduit* | importe des octets jamais prévisualisés (navigateur qui relit) | **Serveur : fermé.** Si les octets arrivent, empreinte (hash du fichier, AC7-4) → `409`, recalcul AC9 et avis neutre ; s'ils n'arrivent pas, rien n'est importé. **Écran : non fermé, assumé.** Sous un navigateur qui échoue l'envoi, l'écran montre l'erreur réseau, *Confirmer* redevient actif et chaque clic échoue de même (l'objet `File` de l'instantané reste celui de l'aperçu) ; la sortie est *Annuler*, puis redéposer le fichier — écrite au manuel (AC12). Aucun risque pour les données ; pas de message dédié (C-27-3-35) |
| D19 | **Quitter la page** (menu, retour du navigateur) pendant une confirmation en vol | — | le composant est démonté ; si la réponse revient en `409`, la branche AC9 lance une requête d'aperçu sans effet visible ni écriture. **Inoffensif, assumé** : aucune garde de démontage (C-27-3-34) |

## Acceptance Criteria

### Écran (frontend)

**AC1 — Changer de profil relance l'aperçu.** Quand l'utilisateur change la valeur du sélecteur
`bank-profile-select` (un profil, ou *Auto-détection*) **alors qu'un fichier est retenu** (`selectedFile` non nul)
et qu'un compte est choisi, l'aperçu est **recalculé** : `previewBankImport(selectedFile, selectedAccountId,
{ bankProfileId })` avec la **nouvelle** valeur, et l'ancien aperçu est retiré avant la réponse. Sans fichier
retenu, changer de profil ne déclenche rien (comportement actuel). Le déclencheur est le **geste** de l'utilisateur
(le `onChange` du sélecteur), **jamais** un `$effect` sur `bankProfileId` (cf. AC4). Pour un **CAMT**, dont
l'aperçu ignore le profil, le changement relance aussi un aperçu — identique — et remet les cases à zéro (AC2) :
assumé, sans risque pour les données ; l'écran ne distingue pas le format avant la réponse du serveur (C-27-3-40).

**AC1-bis — Aussi après un aperçu en échec (D9).** Si le dernier aperçu a échoué (`preview === null`,
`errorCode` posé) et qu'un fichier est retenu, changer de profil relance l'aperçu avec ce fichier : l'utilisateur
n'a plus à redéposer pour essayer un autre profil. Exception : l'erreur `BANK_ACCOUNT_REQUIRED` (aucun fichier
retenu) ne relance rien.

**AC2 — État de l'écran pendant le recalcul.** Pendant qu'un aperçu est en cours (dépôt initial **ou** recalcul) :

- l'ancien aperçu (`bank-import-preview`) n'est **pas** affiché, ni donc le bouton *Confirmer l'import* ;
- la zone de dépôt (`bank-import-drop-zone`) n'est **pas** affichée (aujourd'hui elle l'est au premier dépôt :
  un second fichier déposé pendant le chargement lançait un aperçu concurrent) ;
- l'indicateur `bank-import-loading` est affiché, `role="status"`, avec le texte
  `bank-import-labels-preview-recomputing` (« Recalcul de la prévisualisation… » — sans « avec le profil choisi » :
  après un `409` sur un CAMT, aucun profil n'est en jeu, C-27-3-28) quand il s'agit d'un **recalcul** (AC1, AC1-bis,
  AC9), et, au dépôt d'un fichier **comme pendant une confirmation**, le « Chargement… » actuel — **traduit** par la
  clé globale existante `common-loading` (AC17 ; aujourd'hui en dur, `:305`, affiché pour les deux requêtes). Le
  gabarit choisit le texte ainsi : **`isConfirming` d'abord** (→ `common-loading`), puis la raison de l'aperçu, lue
  dans un état `previewReason` (`$state`, `'drop' | 'profile' | 'stale'`) que **chaque** appel de `runPreview` pose
  avant son premier `await` ; `reset()` n'a pas à y toucher (C-27-3-41). ⚠️ `previewReason` garde sa valeur après
  l'aperçu : sans la priorité à `isConfirming`, une confirmation qui suit un recalcul afficherait « Recalcul de la
  prévisualisation… » pendant l'import (validation P4, P4-1 ; C-27-3-45). Le texte est tenu par V2 (recalcul), V9
  (confirmation après un recalcul) et V13 (b) (dépôt, `common-loading`) ;
- l'erreur précédente (`bank-import-error`) est effacée ;
- **les cases et le choix de doublons** (`confirmBalanceMismatch`, `confirmDuplicateFile`, `confirmDuplicateLines`
  remis à `skip`, `confirmPartialImport`, `confirmEncodingMismatch`) sont **remis à zéro** — les avertissements du
  nouvel aperçu ne sont pas ceux de l'ancien (même raison que F1 Pass 3, `:114-125`).

**AC3 — Le dernier demandé gagne ; rien de périmé ne s'affiche (D2, D5, D10, D15).** Chaque demande d'aperçu porte
un numéro de séquence ; une réponse (succès **ou** erreur) dont le numéro n'est plus le dernier est **ignorée** —
elle ne pose ni `preview`, ni l'erreur, ni la présélection, ni ne termine l'indicateur de chargement. Le compteur
de séquence est une variable **ordinaire** (`let seq = 0`), **jamais** un `$state` : `reset()` est appelée depuis le
`$effect` du compte (`:75-81`), qui dépendrait sinon du compteur.

`reset()` (changement de compte, *Annuler*, succès de l'import) **invalide** la séquence en cours **et** remet
`isPreviewing` et `isConfirming` à faux et efface l'avis de l'AC9 (C-27-3-20) : la réponse ignorée ne pouvant plus
éteindre l'indicateur, c'est `reset()` qui le fait — sans quoi un changement de compte pendant un aperçu laisserait
l'indicateur allumé et la zone de dépôt masquée (AC2) jusqu'au rechargement de la page. ⚠️ L'extinction
d'`isConfirming` par `reset()` est **défensive, non éprouvée** : pendant une confirmation, le compte et *Annuler* sont
désactivés (ci-dessous), si bien que `reset()` n'y est appelée que par le succès de l'import, dont le `finally` éteint
déjà `isConfirming`. Aucun test ne peut l'observer, aucune mutation ne la vise — angle mort déclaré (C-27-3-38).
L'extinction d'`isPreviewing` (M19) et l'effacement de l'avis (M25), eux, sont éprouvés.

Pendant une **confirmation** en vol, les sélecteurs de compte (`bank-account-select`) et de profil
(`bank-profile-select`) **et** le bouton *Annuler* (`bank-import-cancel`) sont **désactivés** (`disabled`) : un geste
à ce moment ne peut plus rien corriger, et *Annuler* suivi d'un `409` aurait fait réapparaître l'aperçu après
l'annulation (D15, C-27-3-21). Pendant un **aperçu** — y compris le recalcul qui suit un `409` (AC9) —, les sélecteurs
restent actifs (C-27-3-3) ; *Annuler* n'y est pas rendu (D11).

**AC4 — La présélection automatique ne relance pas l'aperçu.** Le comportement KF #70 (`:135-142`) est conservé :
après un aperçu demandé sur *Auto-détection* et rendu `autoMatched`, `bankProfileId` prend `csvProfileMatch.profileId`
**sans** nouvel appel à `previewBankImport` (un seul appel pour un dépôt).

L'annotation « (auto-détecté) » du sélecteur (`BankProfileSelector.svelte:53-55`) ne s'affiche que pour un aperçu
**`autoMatched`** : `autoMatchedId={preview?.csvProfileMatch?.autoMatched ? preview.csvProfileMatch.profileId : null}`.
Aujourd'hui elle suit `csvProfileMatch.profileId` (`BankImportUpload.svelte:252`), que `preview_csv` pose **toujours**
pour un CSV (`bank_imports.rs:1688-1693`) : un profil **choisi** avant le dépôt est déjà annoté à tort, et l'AC1 —
qui relance l'aperçu à chaque choix — rendrait l'erreur systématique (C-27-3-32).

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
- ⚠️ **Asymétrie assumée entre CSV et CAMT** (C-27-3-24) : pour un CAMT, le `409` précède le
  `422 BANK_IMPORT_DUPLICATE_FILE`, dont la détection se fait **dans** la transaction (`create`, `:1108-1119`) ;
  pour un CSV, il le suit. Aucune incidence (les deux refusent sans importer) ; l'ordre est écrit à
  `docs/api-external.md` (AC13) et tenu par I10.
- La comparaison CSV est **sans fenêtre par construction** : la même variable `profile`, lue dans la transaction,
  est comparée puis parsée (`:1795-1809`). Ce n'est pas testable sans injection de concurrence, et ce n'est pas
  compté comme mutation (C-27-3-23).
- Un aperçu sur *Auto-détection* (profil `X` résolu par le nom du fichier) suivi d'une création avec
  `bankProfileId = X` explicite donne la **même** empreinte : `201`.

### Réaction de l'écran au refus

**AC9 — Sur `409 BANK_IMPORT_PREVIEW_STALE`, l'écran recalcule et le dit.** Si la confirmation reçoit ce code,
l'écran **relance l'aperçu** (même fichier, même compte, profil de l'instantané — AC5) et affiche l'avis
`bank-import-preview-recomputed` — rendu **hors** du bloc `{#if preview}` (`:354`), au-dessus de l'aperçu s'il y en
a un et au-dessus de l'erreur sinon, puisqu'il doit rester visible quand le recalcul échoue (E2) — (`role="status"`, clé `bank-import-labels-preview-recomputed` :
« Le profil, le fichier ou le compte a changé depuis la prévisualisation : Kesh la recalcule. Vérifiez le résultat
avant de confirmer. »). Le texte est **vrai dans les trois états** où l'écran le montre — recalcul en cours (à côté
de « Recalcul de la prévisualisation… »), recalcul réussi (le résultat est le nouvel aperçu), recalcul en échec (le
résultat est l'erreur) — : il ne dit ni que le recalcul est fini, ni qu'il a réussi (C-27-3-36). L'avis est
**neutre**, aligné sur le message serveur (`bank-import-errors-preview-stale`) : depuis l'écran, l'instantané fige
l'objet `File` et l'id du compte, mais un `409` peut venir du **contenu** du profil (D3), de l'**IBAN** du compte
modifié ailleurs (D7) ou, sous un navigateur qui relit le fichier, de ses **octets** modifiés sur le disque (D18) — et,
pour un CAMT, jamais du profil (C-27-3-28). ⚠️ Un navigateur qui **échoue l'envoi** d'un fichier modifié (Chrome,
`ERR_UPLOAD_FILE_CHANGED`) ne produit **aucun** `409` : la confirmation tombe en `NETWORK_ERROR`, affichée comme
toute erreur, **sans** recalcul ni avis ; la sortie est *Annuler* puis redéposer (D18, AC12, C-27-3-35).

`isConfirming` est remis à faux **avant** que le recalcul ne soit lancé (pas d'`await runPreview(…)` dans le `catch`
avant le `finally`) : pendant le recalcul, les sélecteurs sont actifs comme pendant tout aperçu (AC3), et un recalcul
qui traîne ne fige pas l'écran (C-27-3-33). Ce recalcul remet les cases et le choix de doublons à zéro comme tout
autre (AC2, D14) : tenu par la variante « cases » de V6 et par M26 (C-27-3-37).
**Aucune nouvelle confirmation automatique** : l'utilisateur reconfirme. `onSuccess` n'est pas appelé. L'avis
disparaît au prochain geste de l'utilisateur (dépôt, changement de profil ou de compte, *Annuler*, nouvelle
confirmation) — effacé en **trois** sites, tous prescrits à T3 : `reset()` (compte, *Annuler*), `runPreview` pour
toute raison autre que `'stale'` (dépôt, profil), et `handleConfirm` dès le clic (nouvelle confirmation) ; chacun
tenu par un test et une mutation — V6-bis (a) et (b) par M20, V6-bis (c) par M25 (C-27-3-31, C-27-3-38). Si ce
recalcul échoue (profil devenu inadapté : refus de signe, par exemple), l'erreur s'affiche **avec** l'avis.

### Textes

**AC10 — i18n, quatre locales : trois clés pour le défaut.** Trois clés neuves, dans
`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`, placées après
`bank-import-labels-bank-profile-auto-detect-placeholder` — fr-CH `:1094`, de-CH / it-CH / en-CH `:1033` —, **hors**
des hunks de la 27-1 (fr-CH après `:1059` et `:1080` ; de/it/en-CH après `:1002` et `:1021`) :

| Clé | fr-CH | de-CH | it-CH | en-CH |
|---|---|---|---|---|
| `bank-import-errors-preview-stale` | Le profil, le fichier ou le compte a changé depuis la prévisualisation : recalculez-la avant d'importer. | Profil, Datei oder Konto haben sich seit der Vorschau geändert: Berechnen Sie die Vorschau vor dem Import neu. | Il profilo, il file o il conto sono cambiati dopo l'anteprima: ricalcolatela prima di importare. | The profile, file or account has changed since the preview: recompute it before importing. |
| `bank-import-labels-preview-recomputing` | Recalcul de la prévisualisation… | Die Vorschau wird neu berechnet… | Ricalcolo dell'anteprima… | Recomputing the preview… |
| `bank-import-labels-preview-recomputed` | Le profil, le fichier ou le compte a changé depuis la prévisualisation : Kesh la recalcule. Vérifiez le résultat avant de confirmer. | Profil, Datei oder Konto haben sich seit der Vorschau geändert: Kesh berechnet sie neu. Prüfen Sie das Ergebnis vor dem Bestätigen. | Il profilo, il file o il conto sono cambiati dopo l'anteprima: Kesh la ricalcola. Verificate il risultato prima di confermare. | The profile, file or account has changed since the preview: Kesh recomputes it. Check the result before confirming. |

Les replis des `i18nMsg` sont **identiques** au texte fr-CH (garde du moissonneur). Préfixe `bank-import-` :
conforme à `lint-i18n-ownership` (feature `bank-import`). Le repli serveur de `t()` est le texte fr-CH, **et il
est inscrit** à la `TABLE` de `les_replis_rust_suivent_le_catalogue`
(`crates/kesh-api/tests/textes_coherents.rs:679-735`, groupe `crates/kesh-api/src/errors.rs`) :
`("bank-import-errors-preview-stale", 1, &[])` — c'est le seul repli Rust neuf de la story (les autres textes neufs
sont des replis `i18nMsg`, tenus par le moissonneur). La 27-1 ajoute trois entrées au même groupe (`:699`) : union à
la fusion.

**Gardes existantes que ces textes engagent** (nommées pour que leur rouge ne surprenne pas) :
`crates/kesh-i18n/src/loader.rs:733` `parity_between_locales` rougit si l'une des 24 clés manque à une locale ;
`frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts` exige des replis identiques au catalogue (il tolère déjà
l'écart historique de `common-loading`, « Chargement... » contre « Chargement… » : le repli neuf de H4/H11 prend la
graphie du catalogue fr-CH) ; `lint-i18n-ownership` (préfixe `bank-import-`, globales `common-` / `error-` dans
`GLOBAL_NAMESPACES`). **Sans incidence**, vérifiées : `i18n-un-repli-par-cle.test.ts` (six préfixes, `PREFIXES`, `:51-71` :
`supplier-invoices-`, `imported-supplier-invoices-`, `payment-batches-`, `onboarding-`, `open-items-`, `error-lettering-` — ni `bank-import-`, ni `common-`, ni `error-unexpected`), `i18n-libelle-en-dur.test.ts` (fonctions `*Label` seules), `crates/kesh-api/tests/audit_route_registry.rs`
(aucune route neuve).

**AC11 — Borne `sitesTotal`.** Sites `i18nMsg(` neufs, recomptés **comme la garde les compte : commentaires
masqués** (`masquerCommentaires`, importé à `i18n-keys.test.ts:32` depuis `i18n-literal-reader.js` ; précédent de la
26-1, où un second `i18nMsg(` vivait dans un commentaire), sur la base `a2743fd4` puis au dernier commit. À la base,
aucun des trois fichiers ne porte `i18nMsg(` en commentaire, si bien que `grep -o "i18nMsg(" <fichier> | wc -l` y
donne les mêmes nombres ; T3 réécrit le commentaire d'en-tête de `BankImportUpload.svelte` — **ne pas** y écrire
`i18nMsg(`, ou recompter commentaires masqués :

| Fichier | base | après | delta | ventilation |
|---|---|---|---|---|
| `features/bank-import/BankImportUpload.svelte` | 23 | 49 | **+26** | 2 pour le défaut (AC10 : recalcul, avis) ; 24 pour l'AC17 (table ci-dessous, lignes H1-H9) |
| `features/bank-import/BankImportList.svelte` | 1 | 5 | **+4** | AC17, H10 |
| `routes/(app)/bank-import/+page.svelte` | 3 | 4 | **+1** | AC17, H11 |

`sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` passe donc de **2032 à 2063** (+31) sur la base
`a2743fd4`, avec son commentaire ventilé (le patron du fichier). ⚠️ La 27-1 le porte à 2041 (+9) : à la fusion,
2032 + 31 + 9 = **2072** — l'orchestrateur additionne les deltas, il ne prend pas l'un des deux nombres. Le
refactor de `BankImportUpload.svelte` (T3) peut fusionner ou dédoubler des sites (deux `catch`, par exemple) :
**recompter au dernier commit et ventiler l'écart**, ne pas recopier ces chiffres. `sitesNonResolus`, `relais`,
`sitesGabarit` inchangés (que des littéraux), à mesurer.

**AC12 — Manuel utilisateur (FR, `.tex` **et** PDF régénéré).** `docs/manual/fr/user-manual.tex` :

- `:1697-1698` (§ *Import dans Kesh*, étape 3) : retirer « \emph{avant} de déposer le fichier » — le profil se
  choisit avant ou après le dépôt ;
- `:1794-1796` (§ *Import avec un profil existant*, étape 3) : idem ; garder la phrase sur *Auto-détection* et la
  mention « auto-détecté » — vraie au code livré, qui ne l'affiche plus que pour un profil reconnu sur
  *Auto-détection* (AC4, C-27-3-32), ce que la phrase du manuel dit déjà ;
- l'encadré `keshwarning` `:1801-1807` est **remplacé** par un texte vrai au code livré :
  la prévisualisation est toujours celle du profil choisi ; changer de profil après le dépôt la **recalcule**
  aussitôt (« Recalcul de la prévisualisation… »), y compris après un refus — inutile de redéposer ; les
  avertissements déjà confirmés sont **à confirmer de nouveau** ; si le profil ou le compte bancaire est modifié
  ailleurs (autre onglet, autre utilisateur) entre la prévisualisation et la confirmation, Kesh **n'importe pas** :
  il recalcule la prévisualisation et l'annonce — l'avis ne dit pas ce qui a changé, l'écran ne le sait pas
  (C-27-3-28) ; vérifier le résultat, puis confirmer de nouveau. Si c'est le **fichier** qui est modifié sur le disque
  après la prévisualisation, Kesh n'importe pas non plus, mais **selon le navigateur** il recalcule la
  prévisualisation, ou l'envoi échoue avec un message d'erreur de connexion (« Impossible de contacter le
  serveur… ») qui se répète à chaque clic : dans ce cas, *Annuler*, puis redéposer le fichier (D18, C-27-3-35).
  *Annuler* garde le profil choisi (vrai, `reset()` ne touche pas `bankProfileId`).
- **Restent vraies, ne pas toucher** : `:1743` et `:1784` (« réglez le sens des débits du profil, puis redéposez le
  fichier ») — modifier un profil se fait dans *Administration*, hors de l'écran d'import, et re-choisir la même
  valeur du sélecteur n'émet aucun `change` ; le message serveur `bank-import-csv-errors-amount-sign-refused-profile`
  (`errors.rs:2367`, ftl `:1120`) idem (C-27-3-14) ; `:1826` (« corriger le fichier source et le redéposer ») —
  redéposer crée un nouvel objet `File`, donc un nouvel aperçu.
- Régénérer le **seul manuel utilisateur** — `make user` dans `docs/manual/`, sous `flock …/.gate.lock` — et ne
  committer que `docs/manual/fr/user-manual.pdf` : `make fr` reconstruit aussi `admin-manual.pdf` et
  `marketing-brochure.pdf` (`Makefile:44`), que la 27-1 modifie, ce qui ajouterait deux conflits binaires sans contenu
  (C-27-3-42). ⚠️ Le `Makefile` de la base fait **deux passes fixes** ; la 27-1 le remplace par des passes jusqu'à
  stabilisation (C-27-1-27 : deux passes laissaient des tables des matières périmées). Sur cette branche, contrôler la
  table des matières du PDF produit (aplatie, entrées de la section *Import* présentes et paginées) ; après la fusion,
  le PDF se régénère de toute façon (conflit binaire certain, tableau des fichiers communs).
- Contrôler le PDF **aplati** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` ne contient
  plus « ne la recalcule pas » ni « avant de déposer », et contient « Recalcul de la prévisualisation » (le libellé
  cité par l'encadré neuf ; **0** occurrence dans le PDF de la base, alors que « recalcul » seul y figure déjà trois
  fois, dont deux hors de l'import — un contrôle sur ce mot serait vert dès la base) et « selon le navigateur ». ⚠️ Le
  texte aplati garde les **coupures de mots** (la base porte `réinitia- lisation`) : en cas d'échec du contrôle
  positif, chercher `Recalcul de la` et `prévisualisation…` séparément, puis relire la ligne, avant de conclure.
- Manuels DE/IT/EN : vides (README seuls) — rien à faire.

**AC13 — CHANGELOG et doc d'API.**

- `CHANGELOG.md`, section `## [0.13.2] — Non publié` (à **créer** sur cette branche si absente : elle n'existe pas
  sur `a2743fd4` ; la 27-1 la crée aussi → conflit de fusion à résoudre par union), sous `### Corrigé` : changer de
  profil après la prévisualisation la recalcule ; **depuis l'écran**, un import ne se fait plus avec un profil, un
  compte ou un fichier modifié depuis la prévisualisation (`409 BANK_IMPORT_PREVIEW_STALE`, champ
  `previewFingerprint`, facultatif pour un client d'API) ; re-choisir le même fichier après un refus relance la
  prévisualisation ; les libellés de l'écran d'import qui restaient en français sont traduits
  ([#626](…/issues/626)) (C-27-3-44).
- `docs/api-external.md` : au § 7, en sous-section propre *Importer un relevé* placée **après** la sous-section
  *Annuler un rapprochement bancaire* (`:543` à la base) — la ligne « Imports bancaires » du tableau d'intro du § 7
  (`POST /bank-imports/preview`, `POST /bank-imports ⁸`) et sa note ⁸ n'existent pas à la base : c'est la 27-1 qui
  les crée (`@@ -229,0 +233,2 @@`) ; la sous-section renvoie à cette ligne après la fusion, sans la créer ici —,
  décrire `previewFingerprint` (rendu par l'aperçu, facultatif à la création, conseillé ; `409` / `400`), l'**angle
  mort assumé** D12 (une création sans empreinte n'est pas contrôlée), l'ordre des refus et son **asymétrie** CSV /
  CAMT (AC8, C-27-3-24), et le cas D17 (empreinte envoyée **sans** `bankProfileId` : un autre profil à motif
  concordant modifié ou créé entre-temps, ou un aperçu fait sur un profil explicite, peuvent faire résoudre à la
  création un autre profil — `409`, à juste titre — ou aucun — `404 BANK_CSV_NO_PROFILE_MATCH` ; envoyer l'id résolu
  par l'aperçu, `csvProfileMatch.profileId`, l'évite). **Ne pas** écrire de cas « deux profils de même
  `updated_at` » (`DATETIME(6)`, C-27-3-39). Et, dans la table du **§ 10 *Gestion des erreurs*** (`:633` à la base),
  une ligne `| 409 | BANK_IMPORT_PREVIEW_STALE | … |` (`POST /bank-imports` : l'empreinte envoyée ne correspond plus au
  profil, au compte ou au fichier ; renvoie à la sous-section du § 7 ; *(Depuis la v0.13.2.)*), au patron des lignes
  voisines ; la 27-1 ajoute au même tableau la ligne `403 API_KEY_OPERATION_FORBIDDEN` (`@@ -648,0 +658 @@`) →
  **conflit probable**, union des deux lignes ; le `400 VALIDATION_ERROR` d'une empreinte mal formée relève de la
  ligne `VALIDATION_ERROR` existante, cité seulement au § 7.

### Tests

**AC14 — Chaque AC a un test qui rougirait** (§ Tests ci-dessous), et les vingt-neuf mutations M1-M27 (M10 et M16 en
deux : M10a, M10b, M16a, M16b) rougissent.

**AC15 — Aucune migration, aucun bump.** Ni fichier sous `crates/kesh-db/migrations/`, ni `kesh_version_min_required`,
ni version Cargo (la release 0.13.2 s'en charge). Une migration au diff signale une dérive.

### Ajouts de la validation P1

**AC16 — Re-choisir le même fichier relance l'aperçu (D16).** `handleFileInput` lit `input.files?.[0]`, puis remet
`input.value = ''` (l'ordre avec l'appel de `handleFileSelect` est indifférent : le `File` est déjà capturé, et
l'ordre n'est pas observable — V12 ne l'asserte pas) : une nouvelle sélection du même fichier par « cliquer pour
parcourir » émet alors `change`. C'est ce que promettent le manuel (« redéposez le fichier », `:1743`, `:1784`) et le
message serveur `bank-import-csv-errors-amount-sign-refused-profile` (C-27-3-14, C-27-3-26). ⚠️ Constat **raisonné,
non exécuté** à la validation : à la base `a2743fd4`, après un refus (`preview` nul), la zone et son `<input>` restent
montés avec leur valeur, et les navigateurs n'émettent pas `change` pour une sélection identique ; l'AC2 démonte en
outre la zone pendant tout aperçu, ce qui suffirait en pratique — la ligne explicite tient la promesse sans dépendre
du rendu.

**AC17 — Les libellés restés en dur sur l'écran d'import sont traduits (C-27-3-25).** Périmètre : l'écran
`/bank-import` — `routes/(app)/bank-import/+page.svelte` et les trois composants qu'il rend,
`BankImportUpload.svelte`, `BankImportList.svelte`, `BankProfileSelector.svelte`. **Inventaire clos** des nœuds de
texte et littéraux affichés qui n'appellent pas `i18nMsg` à la base `a2743fd4` (relevé par
`grep -nE ">[^<{}]*[A-Za-zé]{2,}[^<{}]*<"` et lecture des `errorMessage = '…'`), chacun résolu ou écrit comme non
résolu. Les textes fr-CH sont **repris à l'identique** (aucune reformulation : le manuel, les E2E et les captures
restent vrais) ; aucun E2E ne lit ces libellés (`grep` des `bank-*.spec.ts` : aucun).

| # | Site (base `a2743fd4`) | Texte | Clé | Sites |
|---|---|---|---|---|
| H1 | `BankImportUpload.svelte:241` | `— Sélectionner un compte —` | `bank-import-labels-bank-account-placeholder` (« Sélectionner un compte » ; cadratins dans le balisage, patron `BankProfileSelector.svelte:50`) | 1 |
| H2 | `:110` | `Sélectionnez d'abord un compte bancaire.` | `bank-import-errors-bank-account-required` | 1 |
| H3 | `:150`, `:185` | `Erreur inattendue.` | **`error-unexpected`** (globale existante, repli identique) | 2 |
| H4 | `:305` | `Chargement…` | **`common-loading`** (globale existante, repli identique) | 1 |
| H5 | `:331-333`, `:503-505` | `Ligne`, `Code`, `Valeur` (deux tables : refus de signe, lignes invalides) | `bank-import-labels-col-line`, `-col-code`, `-col-value` | 6 |
| H6 | `:360-371` | `IBAN`, `Période`, `Devise`, `Solde ouverture`, `Solde clôture`, `Nombre de transactions` | `bank-import-labels-iban`, `-period`, `-currency`, `-opening-balance`, `-closing-balance`, `-transaction-count` | 6 |
| H7 | `:426-428` | `Index`, `ID existant`, `Clé` | `bank-import-labels-duplicate-col-index`, `-duplicate-col-existing-id`, `-duplicate-col-key` | 3 |
| H8 | `:629` | `statement(s) ignoré(s) (autres IBAN dans le fichier).` (précédé du nombre) | `bank-import-labels-ignored-statements-summary` — le nombre reste **hors** du message, patron de `bank-import-warnings-duplicate-lines-summary` (`:416-421`) : aucune variable Fluent | 1 |
| H9 | `:637-639` | `Date`, `Montant`, `Détails` | `bank-import-labels-tx-col-date`, `-tx-col-amount`, `-tx-col-details` | 3 |
| H10 | `BankImportList.svelte:24-27` | `Date import`, `Fichier`, `Période`, `Transactions` | `bank-import-labels-list-col-imported-at`, `-list-col-filename`, **`-period`** (la clé de H6, même sens), `-list-col-transactions` | 4 |
| H11 | `routes/(app)/bank-import/+page.svelte:54` | `Chargement…` | **`common-loading`** | 1 |

**Non résolus, assumés** : `Δ =` (`BankImportUpload.svelte:579`, avertissement d'écart de solde — symbole), les
cadratins `—` et flèches `→` des cellules, les codes bruts `<code>{line.code}</code>` (le libellé traduit est
juste à côté, `csvLineErrorLabel`) et le repli brut du gabarit `bank-import-info-*` (existant, borné par
`sitesGabarit`). `BankProfileSelector.svelte` n'a aucun libellé en dur. **Hors périmètre** : la fiche d'import
`routes/(app)/bank-import/[id]/+page.svelte` (une quinzaine de libellés en dur, `:55-104`) et `profiles/[id]/+page.svelte:40`
— autres écrans ; la 27-1 modifie la fiche d'import juste après le bloc `<dl>` (`@@ -93,6 +103,8 @@`), d'où un conflit
certain. Signalé à l'orchestrateur (C-27-3-25).

Clés neuves de l'AC17, placées avec celles de l'AC10 :

| Clé | fr-CH | de-CH | it-CH | en-CH |
|---|---|---|---|---|
| `bank-import-labels-bank-account-placeholder` | Sélectionner un compte | Konto auswählen | Selezionare un conto | Select an account |
| `bank-import-errors-bank-account-required` | Sélectionnez d'abord un compte bancaire. | Wählen Sie zuerst ein Bankkonto aus. | Selezionate prima un conto bancario. | Select a bank account first. |
| `bank-import-labels-col-line` | Ligne | Zeile | Riga | Line |
| `bank-import-labels-col-code` | Code | Code | Codice | Code |
| `bank-import-labels-col-value` | Valeur | Wert | Valore | Value |
| `bank-import-labels-iban` | IBAN | IBAN | IBAN | IBAN |
| `bank-import-labels-period` | Période | Zeitraum | Periodo | Period |
| `bank-import-labels-currency` | Devise | Währung | Valuta | Currency |
| `bank-import-labels-opening-balance` | Solde ouverture | Anfangssaldo | Saldo iniziale | Opening balance |
| `bank-import-labels-closing-balance` | Solde clôture | Schlusssaldo | Saldo finale | Closing balance |
| `bank-import-labels-transaction-count` | Nombre de transactions | Anzahl Transaktionen | Numero di transazioni | Number of transactions |
| `bank-import-labels-duplicate-col-index` | Index | Index | Indice | Index |
| `bank-import-labels-duplicate-col-existing-id` | ID existant | Bestehende ID | ID esistente | Existing ID |
| `bank-import-labels-duplicate-col-key` | Clé | Schlüssel | Chiave | Key |
| `bank-import-labels-ignored-statements-summary` | statement(s) ignoré(s) (autres IBAN dans le fichier). | Auszug/Auszüge ignoriert (andere IBAN in der Datei). | estratto/i ignorato/i (altri IBAN nel file). | statement(s) ignored (other IBANs in the file). |
| `bank-import-labels-tx-col-date` | Date | Datum | Data | Date |
| `bank-import-labels-tx-col-amount` | Montant | Betrag | Importo | Amount |
| `bank-import-labels-tx-col-details` | Détails | Details | Dettagli | Details |
| `bank-import-labels-list-col-imported-at` | Date import | Importdatum | Data importazione | Import date |
| `bank-import-labels-list-col-filename` | Fichier | Datei | File | File |
| `bank-import-labels-list-col-transactions` | Transactions | Transaktionen | Transazioni | Transactions |

Soit **21** clés neuves pour l'AC17 (H1 1, H2 1, H5 3, H6 6, H7 3, H8 1, H9 3, H10 3 — `-period` y est partagée
avec H6 — = **21**, une ligne de table chacune) et **24** avec l'AC10, dans chacune des quatre locales ; deux
clés globales réutilisées (`common-loading`, `error-unexpected`), sans clé neuve. Les replis sont identiques au
fr-CH, y compris l'apostrophe droite de « d'abord » (texte actuel du site).

## Tasks / Subtasks

- [ ] **T0 — Préalables** (AC15)
  - [ ] Bases `kesh_273` / `kesh_e2e_273` (migrations du worktree, seed `scripts/seed-dev-db.sql`) ; `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-27-3/target` ; `node_modules` par `cp -al` depuis le dépôt principal (consignes 6, 8, 11).
- [ ] **T1 — Empreinte serveur** (AC6, AC7, AC8)
  - [ ] Nouveau module `crates/kesh-api/src/routes/bank_import_fingerprint.rs` (déclaré dans `routes/mod.rs`) — **hors** de `bank_imports.rs` pour limiter le recouvrement avec la 27-1 : `pub(crate) fn preview_fingerprint(company_id, &BankAccount, file_hash: &str, format: FingerprintFormat, profile: Option<&BankProfile>) -> String`, déstructuration exhaustive de `BankProfile` ; `pub(crate) fn parse_fingerprint_field(&str) -> Result<String, AppError>` (64 hex minuscules). Doc-comments `///` (ce que couvre l'empreinte, ce qu'elle ne couvre pas, pourquoi pas de secret). `normalize_iban` (`bank_imports.rs:815`, privée) passe **`pub(crate)`** pour être appelée du module neuf — ni copie (DRY), ni déplacement (hunk plus large dans un fichier que touche la 27-1).
  - [ ] `BankImportPreviewResponse` (`bank_imports.rs:54`) : champ `preview_fingerprint: String` ; rempli par `preview` (CAMT, `:1024`) et `preview_csv` (`:1738`).
  - [ ] `MultipartFields` + `parse_multipart` : `preview_fingerprint: Option<String>`, garde de doublon, validation de forme.
  - [ ] `create` (CAMT) et `create_csv` (après `resolve_csv_profile`) : comparaison → `AppError::BankImportPreviewStale`.
  - [ ] `errors.rs` : variante `BankImportPreviewStale` (placée près de `BankCsvProfileNotFound`, `:740`, **pas** après `BankImportDuplicateFile` où la 27-1 insère) ; mapping `409`, code `BANK_IMPORT_PREVIEW_STALE`, `t("bank-import-errors-preview-stale", …)` (placé près de `:2237`).
  - [ ] `crates/kesh-api/tests/textes_coherents.rs` : entrée `("bank-import-errors-preview-stale", 1, &[])` dans la `TABLE` de `les_replis_rust_suivent_le_catalogue`, groupe `errors.rs` (AC10).
  - [ ] Tests unitaires dans le module neuf (U1-U4) ; tests d'intégration dans `crates/kesh-api/tests/bank_imports_e2e.rs` (I1-I12, I6-bis), avec le helper **existant** `build_multipart_with_text_fields` (`:677`) pour I6, I6-bis et les créations CAMT à empreinte (multipart **sans** `bankProfileId`, nom de fichier choisi, champs texte en plus) — sans modifier la signature de `send_csv` / `build_csv_multipart`, que la 27-1 emploie aussi.
- [ ] **T2 — Client API et types** (AC5, AC6)
  - [ ] `bank-import.types.ts` : `previewFingerprint: string` dans `BankImportPreviewResponse`.
  - [ ] `bank-import.api.ts` : `BankImportFlags.previewFingerprint?: string`, ajouté par `appendFlags` ; test dans `bank-import.api.test.ts` (V11).
- [ ] **T3 — Écran** (AC1-AC5, AC9)
  - [ ] `BankImportUpload.svelte` : séparer `runPreview(file, reason)` (séquence, retrait de l'ancien aperçu, remise à zéro des cases, instantané à la réception, présélection AC4) des gestes `handleFileSelect` (dépôt) et `handleProfileChange(id)` (sélecteur) ; états `isPreviewing` / `isConfirming` (l'indicateur et le `disabled` du bouton en dérivent) et `previewReason` (`$state`, posé par chaque `runPreview` avant son premier `await`, lu par le texte de l'indicateur, AC2, C-27-3-41) ; `runPreview` remet les cases à zéro pour **toute** raison, `'stale'` comprise (AC2, AC9, M26) ; compteur de séquence en variable ordinaire (`let seq = 0`, pas `$state`) ; `reset()` invalide la séquence **et** remet `isPreviewing`, `isConfirming` et l'avis à leur état initial (AC3 ; l'extinction d'`isConfirming` y est défensive, non éprouvée, C-27-3-38) ; une réponse périmée — succès **ou** erreur — ne touche ni `preview`, ni l'erreur, ni `isPreviewing` (le `finally` n'éteint l'indicateur que si sa séquence est la dernière) ; `handleConfirm` depuis l'instantané, **efface l'avis dès le clic**, branche `BANK_IMPORT_PREVIEW_STALE` → `isConfirming = false` **puis** `void runPreview(…, 'stale')` (pas d'`await` avant le `finally`, AC9, C-27-3-33) + avis rendu hors du bloc `{#if preview}` (AC9) ; `runPreview` **efface l'avis** pour toute raison autre que `'stale'` (dépôt, profil) — l'avis a donc trois sites d'effacement : `reset()`, `runPreview` hors `'stale'`, `handleConfirm` (C-27-3-31) ; *Annuler* `disabled={isConfirming}` ; `handleFileInput` vide `input.value` (AC16) ; `autoMatchedId` ne vaut l'id du profil que si `csvProfileMatch.autoMatched` (AC4, C-27-3-32).
  - [ ] `BankProfileSelector.svelte` : prop `disabled` (défaut `false`) ; le `<select>` de compte reçoit `disabled={isConfirming}`.
  - [ ] Mettre à jour le commentaire d'en-tête (machine d'état) et le commentaire L3 de `handleConfirm` (`:160-167`).
  - [ ] Nouveau `BankImportUpload.test.ts` (V1-V10, V6-bis, V12, V13), patron `BankProfileForm.test.ts` (mocks avant import).
- [ ] **T4 — i18n** (AC10, AC11, AC17) : 24 clés × quatre locales (3 de l'AC10, 21 de l'AC17) ; sites H1-H11 de l'AC17 dans `BankImportUpload.svelte`, `BankImportList.svelte`, `routes/(app)/bank-import/+page.svelte` (deux clés globales réutilisées) ; `sitesTotal` 2032 → 2063 recompté commentaires masqués et ventilé ; gardes engagées nommées à l'AC10 (`parity_between_locales`, `i18n-repli-divergent-actif`, `lint-i18n-ownership`).
- [ ] **T5 — E2E** (AC1, AC6, AC9) : deux tests dans `frontend/tests/e2e/bank-csv-import.spec.ts` (E1, E2).
- [ ] **T6 — Manuel** (AC12) : `.tex`, `make user` (sous `flock`), seul `user-manual.pdf` committé, PDF aplati contrôlé (table des matières comprise : Makefile de la base à deux passes fixes, C-27-3-42).
- [ ] **T7 — CHANGELOG, `docs/api-external.md`** (AC13).
- [ ] **T8 — Mutations** M1-M27 (vingt-neuf, M10 et M16 en deux) : chacune appliquée, test rouge constaté, restaurée (et le fichier **touché** après restauration : cargo garde sinon le binaire muté) ; tableau au Dev Agent Record.
- [ ] **T9 — Gates** au dernier commit de code : `scripts/test-fast.sh` (sous `flock`), `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`, **E2E complet** (8 échecs attendus de `docs/testing.md`, jugés fichier par fichier). Base remise à zéro avant le gate complet. Les gardes de l'AC10 (`parity_between_locales` dans le gate Rust, `i18n-repli-divergent-actif` et `i18n-keys` dans Vitest) y passent.

## Tests prescrits

### Unitaires Rust (module `bank_import_fingerprint`)

- **U1** déterminisme : deux appels mêmes entrées → même empreinte ; 64 hex minuscules.
- **U2** sensibilité, **table** : pour chaque champ de `BankProfile` hors horodatages (11), une copie qui ne diffère que
  par lui → empreinte différente ; idem pour `company_id`, compte (`id`, IBAN), hash du fichier, format.
- **U3** insensibilité : `created_at` / `updated_at` changés → même empreinte ; IBAN avec espaces / minuscules →
  même empreinte (normalisé).
- **U4** `parse_fingerprint_field` : accepte 64 hex minuscules ; refuse 63, 65, majuscules, `g`, vide.

### Intégration (`crates/kesh-api/tests/bank_imports_e2e.rs`, `#[sqlx::test(migrations = "../kesh-db/test-schema")]`)

Helpers existants : `send_csv` (`:1607`), `create_test_bank_profile_csv` (`:690`), `build_csv_multipart` (`:711`),
`post_ubs_profile` (`:1574`). L'empreinte passe par le paramètre `extra` (`("previewFingerprint", fp)`). ⚠️ `send_csv`
envoie **toujours** `bankProfileId` et fixe le nom `ubs_synthetique.csv`, et `post_ubs_profile` crée le profil avec
`"filenamePattern": null` : I6 et I6-bis (aperçu sur *Auto-détection*) passent par le helper **existant**
`build_multipart_with_text_fields` (`:677`) — multipart sans `bankProfileId`, nom de fichier choisi, champs texte en
plus (dont `previewFingerprint`) ; il sert aussi aux créations CAMT à empreinte (I10). Le profil à motif est posé
par `UPDATE` ou par un `POST` dédié. La détection CSV passe par l'extension avant le MIME `application/xml` que pose
`build_multipart` (`bank_imports.rs:566-571`). Ne pas modifier la signature des helpers existants (la 27-1 les
emploie).

- **I1** l'aperçu CSV **et** CAMT rend `previewFingerprint` (64 hex) ; deux aperçus identiques → même valeur.
- **I2** création avec l'empreinte de l'aperçu → `201`.
- **I3** création **sans** empreinte → `201` (compatibilité D12).
- **I4** **table** : pour chaque colonne du profil modifiable — `bank_name`, `filename_pattern`, `column_mapping` (colonne SQL réelle ; `column_mapping_json` n'est que l'alias de lecture, `bank_profiles.rs:21-24`, et la valeur posée doit rester du JSON valide, `CHECK (json_valid(column_mapping))`),
  `date_format`, `decimal_separator`, `field_separator`, `encoding`, `header_row_count`, `debit_sign` —, un `UPDATE`
  SQL entre l'aperçu et la création → `409 BANK_IMPORT_PREVIEW_STALE` ; `bank_imports`, `bank_transactions` et
  `audit_log` **inchangés** (compter avant / après).
- **I5** création avec un **autre** `bankProfileId` explicite que celui de l'aperçu → `409`.
- **I6** aperçu sur *Auto-détection* (profil à `filename_pattern` reconnu), création avec l'id résolu → `201`
  (AC8, dernier point : *Auto-détection* puis explicite, même empreinte).
- **I6-bis** aperçu **et** création sur *Auto-détection* : aucun `bankProfileId` des deux côtés, même nom de fichier,
  **un seul** profil à motif concordant, empreinte de l'aperçu envoyée → `201`. C'est le seul test où l'id demandé à la
  création (`None`) diffère du profil résolu : il tue M16b (C-27-3-29).
- **I7** autre fichier (octets différents) avec l'empreinte du premier → `409` ; autre compte bancaire de la même
  société → `409`.
- **I8** empreinte mal formée → `400` ; champ dupliqué → `400`.
- **I9** seul `updated_at` modifié entre aperçu et création → `201`.
- **I10** précédence : CSV déjà importé **et** empreinte périmée, sans `confirmDuplicateFile` → `422
  BANK_IMPORT_DUPLICATE_FILE` (AC8) ; CAMT avec empreinte d'un autre fichier → `409` ; CAMT déjà importé **et**
  empreinte périmée, sans `confirmDuplicateFile` → `409` (l'asymétrie assumée de l'AC8, C-27-3-24).
- **I11** profil **supprimé** entre l'aperçu et la création (D4), empreinte de l'aperçu envoyée → `404
  BANK_CSV_NO_PROFILE_MATCH`, et non `409` (précédence AC8, mutation M16a).
- **I12** compte bancaire inconnu de la société, empreinte quelconque bien formée → `404
  BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` (précédence AC8).

### Vitest (`frontend/src/lib/features/bank-import/BankImportUpload.test.ts`, neuf)

Mocks : `./bank-import.api` (`previewBankImport`, `createBankImport`), `./bank-profile.api` (`listBankProfiles` → deux
profils), `$lib/shared/utils/i18n.svelte` (repli), et un `File` synthétique.

- **V1** aperçu avec le profil A, puis choix de B → `previewBankImport` rappelé avec `bankProfileId: B` ; l'aperçu
  affiché est celui de B ; *Confirmer* envoie `bankProfileId: B` et l'empreinte de B. Les deux aperçus sont rendus
  **sans** `autoMatched` (profil choisi) : l'option B ne porte **pas** l'annotation « (auto-détecté) » (AC4,
  C-27-3-32) ; V4, lui, la voit sur le profil présélectionné.
- **V2** pendant le recalcul (promesse retenue) : `bank-import-preview` et `bank-import-confirm` absents,
  `bank-import-drop-zone` absente, `bank-import-loading` porte `role="status"` et le texte de recalcul ;
  `bank-account-select` et `bank-profile-select` **actifs** (AC3 : un aperçu ne désactive rien).
- **V3** course : A → B → C rapides, réponses rendues dans l'ordre **A, C, B** → après la résolution de A,
  `bank-import-loading` **présent** et aucun aperçu affiché (la réponse périmée A n'éteint pas l'indicateur) ; après C,
  l'aperçu affiché est C et l'indicateur absent ; après B, l'aperçu reste C. L'ordre A, C, B fait arriver une réponse
  périmée **pendant** qu'un aperçu plus récent est en vol, ce que l'ordre C, A, B de la P1 n'exerçait pas (C-27-3-30).
- **V4** *Auto-détection* + réponse `autoMatched` → **un seul** appel à `previewBankImport`, sélecteur sur le profil
  présélectionné, qui porte l'annotation « (auto-détecté) ».
- **V5** aperçu en erreur (`BANK_CSV_AMOUNT_SIGN_MISMATCH`), puis choix d'un autre profil → aperçu relancé avec le
  même fichier, erreur effacée.
- **V6** *Confirmer* → `409 BANK_IMPORT_PREVIEW_STALE` → `previewBankImport` rappelé, avis
  `bank-import-preview-recomputed` visible, `createBankImport` appelé **une** fois, `onSuccess` non appelé. L'erreur
  simulée porte `status: 409` **numérique** en plus de `code` : `isApiError` (`api-client.ts:19-27`) exige les deux,
  faute de quoi la branche n'est jamais prise. Pendant le recalcul (promesse retenue), `bank-account-select` et
  `bank-profile-select` sont **actifs** (`isConfirming` éteint avant la relance, C-27-3-33). Variante : le recalcul
  échoue (refus de signe) → l'avis **et** l'erreur sont visibles (AC9).
  Variante « cases » (C-27-3-37) : l'aperçu porte `warnings.duplicateFile`, la case `confirm-duplicate-file` est
  cochée, *Confirmer* → `409` → après le recalcul (qui porte de nouveau `duplicateFile`), la case est **décochée** et
  *Confirmer* désactivé — la voie `409` remet les cases à zéro comme le changement de profil (V7) ; tue M26.
- **V6-bis** (AC9, disparition de l'avis) — après l'avis de V6, recalcul résolu, **trois `it` distincts**, un rendu
  chacun (`@testing-library/svelte` ne démonte qu'en `afterEach` : deux `render` dans un même `it` laisseraient deux
  `bank-import-preview-recomputed` dans `document.body`) : (a) changer de profil → avis absent dès le geste ;
  (b) cliquer *Confirmer* (promesse retenue) → avis absent **dès le clic**, et `bank-import-loading` porte `common-loading` (aperçu de raison `'stale'` : variante de M27, C-27-3-45) ; (c) changer de compte → avis absent. Un
  site par geste : `runPreview` hors `'stale'` (M20), `handleConfirm` (M20), `reset()` (M25) (C-27-3-31, C-27-3-38).
- **V7** case `confirm-duplicate-file` cochée, puis changement de profil → la case du nouvel aperçu est **décochée**
  et *Confirmer* désactivé.
- **V8** changement de compte pendant un aperçu en vol (promesse retenue) → dès le changement,
  `bank-import-loading` **absent** et `bank-import-drop-zone` **présente** ; la réponse tardive résolue ensuite
  n'affiche ni aperçu ni erreur, et laisse la zone de dépôt affichée (AC3, C-27-3-20). Variante : la réponse tardive
  est **rejetée** (une `ApiError`, `status` numérique) → ni `bank-import-error`, ni `bank-import-loading`, zone de
  dépôt affichée (une garde de séquence posée sur la seule branche de succès laisserait passer l'erreur,
  C-27-3-30).
- **V9** pendant la confirmation (promesse retenue) : `bank-account-select`, `bank-profile-select` **et**
  `bank-import-cancel` `disabled` (D10, D15) ; `bank-import-loading` porte le texte de `common-loading` (AC2). La
  confirmation est lancée **après un recalcul** (changement de profil, puis *Confirmer*) : c'est le seul ordre où
  `previewReason` vaut `'profile'` pendant la confirmation, donc le seul qui tue M27 (C-27-3-45).
- **V10** CAMT : aperçu rendu **sans** `csvProfileMatch`, sélecteur sur le profil A, *Confirmer* →
  `createBankImport` appelé **sans** `bankProfileId` (instantané, AC5). C'est le seul état atteignable où le
  sélecteur et l'instantané divergent ; il tue M6.
- **V11** (`bank-import.api.test.ts`) `createBankImport(…, { previewFingerprint })` ajoute le champ ; absent sinon.
- **V12** (AC16) choisir un fichier par l'`<input>` `bank-import-file-input` : le **setter** de `value` de cet élément
  (espionné par `Object.defineProperty(input, 'value', { set, get, configurable: true })` avant l'événement) est
  appelé avec `''`, **et** `previewBankImport` reçoit ce fichier — sans assertion d'ordre entre les deux (l'ordre
  n'est pas observable et l'AC16 ne l'impose pas). Le dépôt ne passe pas par `@testing-library/user-event`
  (absent de `frontend/package.json`) : `files` posé par `Object.defineProperty`, puis `fireEvent.change`.
- **V13** (AC17) rendus avec un `i18nMsg` simulé qui rend **la clé** (mode du mock basculé pour ce seul test) :
  (a) glisser un fichier **sans** compte choisi → `bank-import-errors-bank-account-required` ; (b) aperçu en vol →
  `common-loading` ; (c) aperçu rejeté par une erreur qui n'est pas une `ApiError` → `error-unexpected`, et de même pour une
confirmation rejetée ainsi (les deux sites de H3, `:150` et `:185`) ; (d) aperçu
  qui porte doublons de lignes, lignes invalides et relevés ignorés ; (e) refus de signe avec lignes ; (f)
  `BankImportList` avec un import. Chacune des 21 clés de l'AC17 et les deux clés globales apparaissent dans le DOM
  d'au moins un rendu, et aucun des textes fr de H1-H10 (`Ligne`, `Période`, `Nombre de transactions`…) n'y figure.
  Le site H11 (`routes/(app)/bank-import/+page.svelte`) n'est tenu que par la borne `sitesTotal` (aucun test de
  route dans ce dossier). Le mode « clé » du mock est remis au mode repli par un `afterEach` (sinon il fuit sur V2, V9 et les autres rendus du fichier).

### E2E (`frontend/tests/e2e/bank-csv-import.spec.ts`)

Patron des tests 26-1 (`:315-404`, `createProfileViaUi`, `UBS_COLUMNS`, `importTotal`). ⚠️ La fixture
`ubs_synthetique.csv` a déjà été **importée** par le test 26-1 (`:315`) : la réutiliser telle quelle ferait
apparaître l'avertissement « fichier déjà importé » et ignorer les 11 lignes comme doublons. Dériver le contenu
**en mémoire** (`buffer` de `setInputFiles`) en remplaçant l'année des dates (`2026-` → `2027-` pour E1, `2028-`
pour E2) et un nom de fichier propre à chaque test (C-27-3-17). Constaté à la validation P1 :
`grep -o "2026-" crates/kesh-import/tests/fixtures/csv/ubs_synthetique.csv | wc -l` → 36, dont une seule occurrence
hors date (le libellé `Facture 2026-014`, texte libre) : le remplacement global est inoffensif. Le dédoublonnage de
lignes porte sur une plage de dates (`find_in_dedup_window`) : 2027 et 2028 sont disjointes de 2026. Noms de
profils propres (`bank_name` unique par société), **motif vide** (`pattern: ''`, enregistré `null` par
`BankProfileForm.svelte:73`) : aucun profil des deux tests n'est auto-détecté.

- **E1** deux profils (« 27-3 moins » `negative`, « 27-3 sans signe » `positive`) ; choisir *sans signe*, déposer →
  refus `BANK_CSV_AMOUNT_SIGN_MISMATCH`, aucun aperçu ; choisir *moins* **sans redéposer** → aperçu, 11 transactions ;
  confirmer → l'import porte 11 montants dont 9 négatifs (somme en centimes, comme `:345-352`).
- **E2** profil *moins*, dépôt → aperçu de 11 ; modifier le profil **par l'API** — l'« autre onglet » :
  `GET /api/v1/bank-profiles/{id}`, puis `PUT` du même chemin avec un corps **complet** reconstruit des seuls champs de
  `CreateOrUpdateBankProfileRequest` (`bankName`, `filenamePattern`, `columnMapping`, `dateFormat`,
  `decimalSeparator`, `fieldSeparator`, `encoding`, `headerRowCount`, `debitSign: 'positive'`). ⚠️ Le `PUT` est un
  remplacement complet en `deny_unknown_fields` (`crates/kesh-api/src/routes/bank_profiles.rs:86-100`), sans défaut pour `debitSign` : renvoyer
  la réponse du `GET` telle quelle (`id`, `createdAt`, `updatedAt`) est refusé. *Confirmer* → avis
  `bank-import-preview-recomputed` visible, refus de signe affiché, `importTotal` inchangé.

## Mutations (T8) — chacune doit faire rougir au moins le test nommé

| # | Mutation | Rougit |
|---|---|---|
| M1 | `handleProfileChange` ne relance plus l'aperçu (pose seulement `bankProfileId`) | V1, V5, E1 |
| M2 | relance par `$effect` sur `bankProfileId` au lieu du geste | V4 (deux appels) |
| M3 | garde de séquence retirée (toute réponse s'applique) | V3, V8 |
| M4 | l'ancien aperçu n'est pas retiré au début du recalcul | V2 |
| M5 | cases non remises à zéro au recalcul | V7 |
| M6 | `handleConfirm` envoie `bankProfileId` du sélecteur au lieu de l'instantané | V10 (CAMT : le sélecteur vaut A, l'instantané n'a aucun profil) |
| M7 | branche `BANK_IMPORT_PREVIEW_STALE` retirée (erreur affichée comme une autre) | V6, E2 |
| M8 | sélecteurs **ou** *Annuler* non désactivés pendant la confirmation | V9 |
| M9 | `appendFlags` n'envoie pas `previewFingerprint` | V11 |
| M10a | création **CSV** (`create_csv`) : comparaison d'empreinte retirée | I4, I5, I7, E2 |
| M10b | création **CAMT** (`create`) : comparaison d'empreinte retirée | I10 (« CAMT avec empreinte d'un autre fichier → `409` », C-27-3-43) |
| M11 | empreinte : `debit_sign` lié à `_` (champ omis) | U2, I4 (ligne `debit_sign`) |
| M12 | empreinte : `column_mapping_json` omis | U2, I4 |
| M13 | empreinte : hash du fichier omis | U2, I7 |
| M14 | empreinte : compte omis | U2, I7 |
| M15 | empreinte : horodatages **inclus** | U3, I9 |
| M16a | création CSV : comparaison déplacée **avant** `resolve_csv_profile` (profil relu par l'id demandé hors transaction ; absent → `null` dans l'empreinte), si bien qu'un profil supprimé rend `409` au lieu de `404` | I11 |
| M16b | création CSV : profil de l'empreinte = `fields.bank_profile_id.and_then(<relecture par id>)` — le profil **demandé**, `null` si aucun id — au lieu du profil **résolu** | I6-bis (création sans id : `null` ≠ profil résolu de l'aperçu → `409` au lieu de `201`). I6 ne la tue **pas** : l'id demandé y est l'id résolu (C-27-3-29) |
| M17 | `handleFileInput` ne vide plus `input.value` | V12 |
| M18 | un libellé de l'AC17 rendu en dur (par exemple `Ligne` remis dans la table du refus de signe) | V13, borne `sitesTotal` |
| M19 | `reset()` ne remet plus `isPreviewing` à faux | V8 |
| M20 | l'avis de l'AC9 n'est effacé que par `reset()` (ni par `runPreview` hors `'stale'`, ni par `handleConfirm`) | V6-bis (a) et (b) |
| M21 | `finally` de l'aperçu : `isPreviewing = false` **sans** condition de séquence (la garde reste sur `preview`) | V3 (indicateur absent après la réponse périmée A) |
| M22 | garde de séquence posée sur la seule branche de **succès** : le `catch` d'une réponse périmée pose `errorCode` / `errorMessage` | V8 (variante rejetée) |
| M23 | `autoMatchedId` suit `csvProfileMatch.profileId` sans regarder `autoMatched` (comportement de la base) | V1 (annotation sur B) |
| M24 | branche `409` : `await runPreview(…, 'stale')` avant le `finally` — `isConfirming` reste vrai pendant le recalcul | V6 (sélecteurs désactivés pendant le recalcul retenu) |
| M25 | `reset()` n'efface plus l'avis de l'AC9 (les deux autres sites restent) | V6-bis (c) (avis encore visible après le changement de compte, C-27-3-38) |
| M26 | `runPreview` ne remet pas les cases à zéro pour `reason === 'stale'` (les autres raisons restent) | V6, variante « cases » (case `confirm-duplicate-file` encore cochée après le recalcul, C-27-3-37) |
| M27 | le texte de l'indicateur ne donne pas la priorité à `isConfirming` (lit `previewReason` seul) | V9 (raison `'profile'`) et V6-bis (b) (raison `'stale'`) : « Recalcul de la prévisualisation… » au lieu de `common-loading` (C-27-3-45) |

Vingt-neuf mutations (M1-M9, M10a, M10b, M11-M15, M16a, M16b, M17-M27). ⚠️ **Non comptée, et écrite comme telle** : la variante « profil
comparé relu **hors** transaction mais présent » est **équivalente** dans tout scénario déterministe — même valeur
lue —, et la fenêtre qu'elle ouvrirait n'est observable que par une course ; elle est tenue par construction (une
seule variable `profile`, comparée puis parsée, AC8, C-27-3-23).

Une mutation qui survit se documente (équivalente, ou test ajouté), jamais en silence.

## Dev Notes

### Ce qu'il faut préserver

- **KF #70** : présélection `autoMatched` (`:135-142`), annotation « auto-détecté » du sélecteur
  (`BankProfileSelector.svelte:53-55`) — conservée **pour un profil auto-détecté seulement** : son alimentation
  (`preview?.csvProfileMatch?.profileId`, `BankImportUpload.svelte:252`) se corrige pour regarder `autoMatched` (AC4,
  C-27-3-32).
- **F1 Pass 3** : remise à zéro des cases à chaque nouveau fichier (`:114-125`) — étendue au recalcul.
- **Purge au changement de compte** (`$effect` `:75-81`) : conservée ; elle s'ajoute à la séquence.
- **Refus de signe** (Story 26-1) : `errorDetails` (`:51`, `:55-64`) posé et effacé avec `errorCode`, y compris par
  la branche `409` (l'avis n'est pas `errorDetails`).
- Les tests d'intégration existants créent des imports **sans** aperçu préalable : l'empreinte facultative les laisse
  verts (I3). Les E2E existants (`bank-import.spec.ts`, `bank-import-confirms.spec.ts`, `bank-csv-import.spec.ts`)
  passent par l'aperçu, donc enverront l'empreinte : ils doivent rester verts sans modification — un rouge y est un
  défaut de l'empreinte (preview et création qui ne calculent pas la même chose), pas du test.
- `parse_multipart` vide les champs inconnus (bras `_ =>`, `:514-521` : le champ est lu puis jeté) : un client qui envoie déjà `previewFingerprint` à une
  v0.13.1 n'obtient pas d'erreur — sans intérêt pratique, mais aucune rupture dans l'autre sens.

### Pourquoi une empreinte sans état, et pas un jeton

Un jeton d'aperçu stocké (table, cache) imposerait une migration, une expiration et un nettoyage, et ne fermerait
rien de plus : ce qui doit être identique entre l'aperçu et la création, ce sont **les entrées du calcul**, que la
création possède toutes. Les recalculer et comparer suffit, sans état (C-27-3-7). Un HMAC n'apporterait rien : la
menace est le client **périmé**, pas le client malveillant (qui a le rôle d'importer).

### Fichiers communs avec la 27-1 (non mergée, `story/27-1-supprimer-import`)

*(Relevé le 2026-10-12 à la validation P3, commit `3c36f721` de la 27-1 contre la base commune `a2743fd4` (`git diff a2743fd4 3c36f721` — l'arbre de travail de la 27-1, en cours de modification, est exclu) ; positions « `@@ -base +27-1 @@` ». La 27-1 bouge encore : **recompter à la fusion**, ne pas recopier ces nombres.)*

| Fichier | 27-1 | 27-3 | Risque de fusion |
|---|---|---|---|
| `crates/kesh-api/src/routes/bank_imports.rs` | +118 / −1 : imports (`@@ -11,0 +12,6 @@`, `@@ -36 +42 @@`), `DELETE` après `detail` (`@@ -1341,0 +1348,111 @@`) | champ de réponse `:54`, multipart `:296-545`, `normalize_iban` passée `pub(crate)` (`:815`), `preview`/`create`/`preview_csv`/`create_csv` | hunks distincts ; vérifier le bloc `use` |
| `crates/kesh-api/src/errors.rs` | variantes après `:218` et `:695` ; mappings après `:1498`, `:2182`, et `:2469` remplacée (positions de la base) | variante près de `:740`, mapping près de `:2237` | faible si l'emplacement prescrit est tenu |
| `crates/kesh-i18n/locales/*/messages.ftl` | fr-CH après `:13`, `:1059`, `:1080`, `:1251`, `:2364` ; de/it/en-CH après `:13`, `:1002`, `:1021`, `:1187`, `:2310`-`:2316` | 24 clés après l'auto-détection : fr-CH `:1094`, de/it/en-CH `:1033` | faible (hunks distincts) |
| `crates/kesh-api/tests/textes_coherents.rs` | +25 lignes (`@@ -701,0 +702,25 @@`), dont trois entrées dans la `TABLE` de `les_replis_rust_suivent_le_catalogue`, groupe `errors.rs` | une entrée dans le même groupe | **conflit probable** (même liste) → union des entrées |
| `crates/kesh-i18n/src/loader.rs` | test `operation_forbidden_message_is_translated_and_distinct_from_its_neighbours` (+38) | non touché | aucun (listé pour que l'inventaire soit complet) |
| `docs/manual/fr/user-manual.tex` + `.pdf` | `:251`, `:273`, `:1643`, insertion de 42 lignes après `:1720`, `:1873` (positions de la base) | `:1697-1698`, `:1794-1807` | `.tex` : faible ; **PDF : conflit binaire certain** → régénérer après fusion (`make user`) |
| `docs/manual/Makefile` | +25 / −6 : passes jusqu'à stabilisation au lieu de deux passes fixes (C-27-1-27) | non touché ; `make user` l'emploie tel qu'il est à la base | aucun conflit ; PDF produit ici avec l'ancien `Makefile` → table des matières à contrôler (AC12), PDF régénéré après fusion |
| `docs/manual/fr/admin-manual.tex` + `.pdf`, `marketing-brochure.pdf` | modifiés | **non committés** : `make user`, pas `make fr` (AC12, C-27-3-42) | aucun, si seul `user-manual.pdf` est committé |
| `CHANGELOG.md` | crée `## [0.13.2] — Non publié` | idem | conflit certain → union |
| `docs/api-external.md` | +11 / −1 : § 4 (note, `@@ -84,0 +85,2 @@`), § 7 (phrase d'intro « trois exceptions », `@@ -205 +207 @@` ; ligne « Imports bancaires » du tableau et note ⁸, `@@ -218,0 +221 @@` et `@@ -229,0 +233,2 @@` ; rejet en lot, `@@ -542,0 +548,2 @@`), § 9 (deux lignes, `@@ -622,0 +630,2 @@`), § 10 (ligne `403 API_KEY_OPERATION_FORBIDDEN`, `@@ -648,0 +658 @@`) | sous-section § 7 après *Annuler un rapprochement bancaire* ; ligne `409` au § 10 (AC13) | § 7 : faible (hunks distincts) ; **§ 10 : conflit probable** (deux lignes ajoutées au même tableau) → union |
| `frontend/src/lib/shared/i18n-keys.test.ts` | `sitesTotal` 2032 → 2041 | 2032 → 2063 | conflit certain → **2072**, commentaires des deux |
| `frontend/src/lib/features/bank-import/bank-import.api.ts` (+ `.test.ts`) | `deleteBankImport` | `previewFingerprint` | faible |
| `crates/kesh-api/tests/bank_imports_e2e.rs` | +384 / −2 (`@@ -550,2 +550,3 @@`, et ajout de 381 lignes en fin de fichier, `@@ -1825,0 +1827,381 @@`) | I1-I12 | ajout en fin de fichier des deux côtés → conflit probable, union |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | section `epic-27` | section `epic-27` créée ici aussi | conflit certain → union |
| `frontend/src/lib/features/bank-import/BankImportList.svelte` | un `data-testid="bank-import-list-link"` sur le lien de la ligne (`@@ -31,7 +31,7 @@`, ligne 34) | en-têtes `:24-27` traduits (AC17, H10) | faible (hunks distincts, six lignes d'écart) |

`BankImportUpload.svelte`, `BankProfileSelector.svelte`, `routes/(app)/bank-import/+page.svelte`, le module
d'empreinte, `BankImportUpload.test.ts` et `bank-csv-import.spec.ts` ne sont pas touchés par la 27-1. La 27-1 touche
en revanche `routes/(app)/bank-import/[id]/+page.svelte`, que la 27-3 laisse tel quel (AC17, hors périmètre), ainsi
que `README.md`, `crates/kesh-db/src/repositories/bank_imports.rs` et `crates/kesh-api/src/routes/bank_accounts.rs`,
que la 27-3 ne touche pas (relevé au `diff --stat` de la validation P2 ; la 27-1 bouge encore).

### Pièges

- **Svelte 5** : `bind:value` du `<select>` de compte et le `value={…}`/`onchange` du sélecteur de profil — garder le
  sélecteur de profil **non lié** (`value` + `onChange`), c'est ce qui permet de distinguer le geste (AC1) de
  l'affectation programmatique (AC4).
- `isLoading` sert aujourd'hui aux deux requêtes ; une réponse d'aperçu périmée qui l'éteint en `finally` rallumerait
  le bouton pendant que le dernier aperçu est en vol — d'où l'extinction conditionnée à la séquence (AC3, V3).
- Le texte « Chargement… » (`:305`) était en dur : il passe par la clé globale `common-loading` (AC17, H4), et
  l'élément porte désormais `role="status"`.
- Le compteur de séquence en `$state` ferait dépendre le `$effect` du compte (`:75-81`) de lui, via `reset()` :
  variable ordinaire (AC3).
- *Annuler* n'est rendu qu'avec un aperçu (`:354`, `:665`) : ne pas tester ni documenter un *Annuler* pendant un aperçu
  en vol (D11).
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

### Validation P1 (2026-10-12) — Sonnet, lentilles R et F

Rapports hors dépôt : `kesh-gate-logs/27-3-validate-p1-F.md` (0 CRITICAL, 1 HIGH, 3 MEDIUM, 4 LOW) et
`27-3-validate-p1-R.md` (0 CRITICAL, 0 HIGH, 4 MEDIUM, 8 LOW). Sept convergences (F1 = R3, F2 = R1, F3 = R2, F4 ≈ R4,
F5 = R6, F6 ≈ R7, F7 = R11) : **4 défauts distincts au-dessus de LOW** (1 HIGH, 3 MEDIUM — les quatre MEDIUM de R
recoupent le HIGH et les trois MEDIUM de F), tous appliqués, ainsi que les 9 LOW distincts. Remédiation par l'agent de
remédiation (Opus 5.5), aucun code, aucun gate.

| Finding | Sévérité | Décision |
|---|---|---|
| F1 / R3 — `reset()` n'éteint pas `isPreviewing` : écran figé après un changement de compte pendant un aperçu | HIGH | **Appliqué** : AC3 (reset éteint `isPreviewing`, `isConfirming`, l'avis) ; V8 asserte indicateur absent et zone présente ; T3 ; mutation M19 ; C-27-3-20 |
| F2 / R1 — D11 / V10 inatteignables (*Annuler* n'existe pas pendant un aperçu) | MEDIUM | **Appliqué** : D11 réécrit « inatteignable » ; V10 retiré de M3 et réaffecté (F3) ; aucun *Annuler* ajouté à l'état de chargement ; C-27-3-21 |
| F3 / R2 — M6 survit ; « à défaut, I5 » est une erreur de catégorie (test Rust contre une mutation Svelte) | MEDIUM | **Appliqué** : V10 = test CAMT (sélecteur sur A, instantané sans profil, `createBankImport` sans `bankProfileId`) ; M6 → V10 ; C-27-3-22 |
| F4 / R4 — M16 (première forme) tuée par aucun test ; précédence 404 non testée | MEDIUM | **Appliqué** : I11 (profil supprimé → `404`), I12 (compte inconnu → `404`) ; M16 → M16a (→ I11) et M16b (→ I6) ; variante « hors transaction mais présent » écrite équivalente, non comptée, tenue par construction (AC8) ; C-27-3-23 |
| F5 / R6 — *Annuler* pendant la confirmation, puis `409` : l'aperçu ressuscite | LOW | **Appliqué** : *Annuler* désactivé pendant la confirmation (AC3) ; ligne D15 ; V9 et M8 étendus ; C-27-3-21 |
| F6 / R7 — `PUT` complet en `deny_unknown_fields` ; motif des profils E2E ; fixture ; `status` numérique de V6 ; avis hors `{#if preview}` | LOW | **Appliqué** : E2 décrit le corps reconstruit ; motif vide pour E1/E2 ; constat de la fixture écrit (36 « 2026- », un hors date, inoffensif) ; V6 exige `status: 409` ; AC9 rend l'avis hors du bloc |
| F7 / R11 — précédence asymétrique CSV / CAMT, non documentée | LOW | **Appliqué** : asymétrie écrite à l'AC8 et à l'AC13 (`docs/api-external.md`), I10 étendu au CAMT ; C-27-3-24 |
| F8 — compteur de séquence réactif | LOW | **Appliqué** : `let seq = 0` prescrit (AC3, T3, Pièges) |
| F (note) — auto-détection côté API, `updated_at DESC` | — | **Appliqué** : ligne D17, écrite à l'AC13 |
| R5 — repli `bank-import-errors-preview-stale` hors de la table des replis ; `textes_coherents.rs` / `loader.rs` absents des recouvrements | LOW | **Appliqué** : entrée à la `TABLE` de `les_replis_rust_suivent_le_catalogue` (AC10, T1) ; deux lignes au tableau des recouvrements (`textes_coherents.rs` : conflit probable, union ; `loader.rs` : non touché par la 27-3) |
| R8 — I6 ne s'écrit pas avec les helpers existants | LOW | **Appliqué** : helper neuf prescrit, signatures existantes intactes (T1, § Intégration) |
| R9 — titre et CHANGELOG sur-promettent (D12) | LOW | **Appliqué** : titre reformulé, CHANGELOG « depuis l'écran » ; C-27-3-27 |
| R10 — numéros de lignes `.ftl` mélangés entre locales | LOW | **Appliqué** : numéros par locale (AC10, recouvrements) |
| R12 — `handleFileInput` ne vide pas `input.value` | LOW | **Appliqué**, sur consigne de l'orchestrateur : défaut **réel au raisonnement** à la base (non exécuté — validation en lecture seule) ; AC16, D16, V12, M17 ; C-27-3-26 |

**Élargissement du périmètre (consigne de l'orchestrateur)** : les libellés restés en dur sur l'écran d'import sont
traduits ici (AC17, inventaire clos H1-H11, 21 clés neuves, deux globales réutilisées) au lieu d'être laissés hors
périmètre — C-27-3-19 révisé par C-27-3-25. La fiche d'import `/bank-import/{id}` reste hors périmètre (conflit
certain avec la 27-1) et est **signalée à l'orchestrateur**.

**Nouveaux décomptes**, recomptés depuis la fiche (`grep -cE "^\*\*AC[0-9]+(-bis)? "`, `grep -oE "^- \*\*[UIVE][0-9]+\*\*"`,
`grep -cE "^\| M[0-9]+[ab]? \|"`, `grep -cE "^\| D[0-9]+ \|"`, `grep -c "^### C-27-3-"` du registre) : **18 AC**
(AC1-AC17 + AC1-bis), tests **U1-U4 / I1-I12 / V1-V13 / E1-E2** (31), **20 mutations** (M1-M15, M16a, M16b,
M17-M19), inventaire **D1-D17**, choix **C-27-3-1 à 27**. i18n : **24 clés** neuves × 4 locales ; `sitesTotal`
**2032 → 2063** (+31 : `BankImportUpload.svelte` 23 → 49, `BankImportList.svelte` 1 → 5,
`routes/(app)/bank-import/+page.svelte` 3 → 4, comptés par `grep -o "i18nMsg("` sur `a2743fd4`), **2072** à la fusion
avec la 27-1 (+9). Signal de découpage (CLAUDE.md, § splitting préventif) : non levé — première passe ; le périmètre
grandit d'un écran (trois fichiers Svelte et les quatre `.ftl`), sans module neuf. Prompt de la P2 :
`27-3-validate-prompt-p2.md` (Opus, lentilles R et F).

### Validation P2 (2026-10-12) — Opus, lentilles R et F

Rapports hors dépôt : `kesh-gate-logs/27-3-validate-p2-F.md` (0 CRITICAL, 0 HIGH, 2 MEDIUM, 8 LOW) et
`27-3-validate-p2-R.md` (0 CRITICAL, 0 HIGH, 4 MEDIUM, 10 LOW). Quatre convergences (F1 = R2, F2 = R1, F6 = R5,
F7 = R6) : **4 défauts distincts au-dessus de LOW** (4 MEDIUM) et **16 LOW distincts** — 24 findings, moins les 4
doublons, font 20 —, tous appliqués. Remédiation par l'agent de remédiation (Opus 5.5), aucun code, aucun gate.

| Finding | Sévérité | Décision |
|---|---|---|
| F1 / R2 — l'avis du `409` impute tout refus au profil ; depuis l'écran il peut venir du compte (IBAN) ou du fichier, et pour un CAMT jamais du profil | MEDIUM | **Appliqué** : avis neutre aligné sur le message serveur (« Le profil, le fichier ou le compte a changé… »), quatre locales et repli (AC9, AC10) ; texte de recalcul sans « avec le profil choisi » ; manuel (AC12) cite profil, compte et fichier ; C-27-3-28 |
| F2 / R1 — M16b n'est pas tuée par I6 (l'id demandé y est l'id résolu) | MEDIUM | **Appliqué** : I6-bis (aucun id des deux côtés, un seul profil concordant → `201`) ; M16b réécrite et réaffectée ; C-27-3-23 corrigé, C-27-3-29 |
| R3 — garde de séquence éprouvée sur des succès seulement, et jamais par une réponse périmée arrivant avant la plus récente | MEDIUM | **Appliqué** : V3 dans l'ordre A, C, B (indicateur présent après A) ; V8 avec une variante rejetée ; T3 précisé ; mutations M21, M22 ; C-27-3-30 |
| R4 — « l'avis disparaît au prochain geste » : ni test ni mutation, un seul site d'effacement prescrit | MEDIUM | **Appliqué** : trois sites prescrits à T3 et à l'AC9 (`reset()`, `runPreview` hors `'stale'`, `handleConfirm`) ; V6-bis ; mutation M20 ; C-27-3-31 |
| F3 — D18 (fichier modifié sur le disque) absent ; suppression d'un import par la 27-1 non nommée | LOW | **Appliqué** : ligne D18 ; D13 nomme la suppression ; C-27-3-34 |
| F4 — D17 : profil résolu changé sans modification (égalité d'`updated_at`, profil créé) | LOW | **Appliqué** : D17 et AC13 élargis |
| F5 — `isConfirming` éteint après le recalcul fige les sélecteurs | LOW | **Appliqué** : `isConfirming = false` puis `void runPreview` (AC9, T3) ; assertion dans V6 ; mutation M24 ; AC3 précisé ; C-27-3-33 |
| F6 / R5 — V12 contredit l'ordre de l'AC16 | LOW | **Appliqué** : AC16 et V12 sans ordre (non observable) |
| F7 / R6 — contrôle PDF « recalcul » vert dès la base | LOW | **Appliqué** : contrôle sur « Recalcul de la prévisualisation » (0 dans le PDF de la base, recompté) |
| F8 — le « helper neuf » d'I6 existe (`build_multipart_with_text_fields`, `:677`) | LOW | **Appliqué** : helper existant nommé (T1, § Intégration) ; corrige la décision R8 de la P1 |
| F9 — annotation « (auto-détecté) » sur un profil choisi, rendue systématique par l'AC1 | LOW | **Appliqué** : `autoMatchedId` conditionné à `autoMatched` (AC4, T3, Dev Notes) ; V1, V4 ; mutation M23 ; AC12 sans « toujours vraie » ; C-27-3-32 |
| F10 — `BankImportList.svelte` touché par la 27-1 ; `textes_coherents.rs` +25 ; autres fichiers 27-1 | LOW | **Appliqué** : ligne au tableau, phrase des fichiers non communs corrigée, `textes_coherents.rs` à recompter à la fusion |
| R7 — numéros de ligne (`post_ubs_profile`, `[id]/+page.svelte`, insertion 27-1 du manuel) | LOW | **Appliqué** : `:1574`, `:55-104`, après `:1720` |
| R8 — `normalize_iban` privée | LOW | **Appliqué** : `pub(crate)` à T1 et au tableau des fichiers communs |
| R9 — pas de ligne `409` au § 10 d'`api-external.md` | LOW | **Appliqué** : ligne prescrite à l'AC13 |
| R10 — gardes existantes non nommées | LOW | **Appliqué** : nommées à l'AC10, T4, T9 (dont celles sans incidence) |
| R11 — recompte `sitesTotal` autrement que la garde | LOW | **Appliqué** : commentaires masqués (AC11, T4) |
| R12 — `role="status"`, sélecteurs actifs pendant l'aperçu, second site de H3, texte de l'indicateur de confirmation | LOW | **Appliqué** : V2, V13 (c), AC2 (`common-loading` pendant la confirmation), V9 |
| R13 — C-27-3-7 attribue à l'issue une demande absente | LOW | **Appliqué** : contexte reformulé |
| R14 — la navigation invalide encore une confirmation en vol | LOW | **Appliqué** : ligne D19 (inoffensif, assumé) ; C-27-3-21 précisé ; C-27-3-34 |

**Signal de découpage (CLAUDE.md, § splitting préventif, amendement D5).** La P2 rend **autant** de défauts distincts
au-dessus de LOW que la P1 (4 → 4 ; 1 HIGH et 3 MEDIUM, puis 4 MEDIUM) : le signal « sévérité égale ou supérieure » est levé — au nombre, la sévérité maximale baissant —, et il est
déclaré ici au Project Lead. **Constat** : les quatre défauts sont **distincts** de ceux de la P1 — aucun n'est un
finding P1 revenu sous une autre forme. Les lentilles marquent deux d'entre eux « régression P1 » (M16b / I6, V3) ;
relu contre la fiche d'origine (`git show 19b51ea3:…`), **les deux y figuraient déjà** — l'ancienne M16 affectait
déjà sa seconde forme à I6, et V3 portait déjà l'ordre C, A, B. La remédiation P1 a **réécrit ces lignes sans voir le
défaut** (scission de M16, retrait de V10 de M3, qui a fait reposer M3 sur V3 et V8 seuls) : elle ne les a pas créés,
elle les a laissés passer. Les deux autres (avis imputé au profil, disparition de l'avis) sont d'origine. Aucune
dispersion nouvelle : la remédiation n'ajoute aucun module, seulement des tests, des mutations et des textes.
**Conclusion : pas de découpage.** La revue trouve des défauts d'origine neufs, de sévérité décroissante (plus aucun
HIGH) : elle ne tourne pas en rond. P3 en passe complète (Sonnet), axe premier sur la présente
remédiation.

**Nouveaux décomptes**, recomptés depuis la fiche (mêmes commandes que la P1, la regex des tests élargie aux `-bis` :
`grep -oE "^- \*\*[UIVE][0-9]+(-bis)?\*\*"`) : **18 AC** ; tests **U1-U4 / I1-I12 + I6-bis / V1-V13 + V6-bis / E1-E2**
(**33**) ; **25 mutations** (M1-M15, M16a, M16b, M17-M24) ; inventaire **D1-D19** ; choix **C-27-3-1 à 34** ; **24 clés**
neuves (3 + 21, `grep -cE "^\| \`bank-import-"` → 24) ; aucun site `i18nMsg(` neuf ni retiré par la P2 (textes
seuls), donc `sitesTotal` **2032 → 2063** inchangé, **2072** à la fusion. Prompt de la P3 : `27-3-validate-prompt-p3.md`
(Sonnet, lentilles R et F).

### Validation P3 (2026-10-12) — Sonnet, lentilles R et F

Rapports hors dépôt : `kesh-gate-logs/27-3-validate-p3-F.md` (0 CRITICAL, 0 HIGH, 1 MEDIUM, 9 LOW — F2 à F10 ; le
bilan d'en-tête du rapport écrit « 8 LOW », la liste en porte neuf, recomptée) et `27-3-validate-p3-R.md`
(0 CRITICAL, 0 HIGH, 1 MEDIUM, 10 LOW — R2 à R11). Cinq convergences (F1 = R1, F3 = R2, F2 = R3, F7 = R5,
F10 = R7) : 21 findings, moins 5 doublons, font **16 distincts — 1 MEDIUM et 15 LOW** —, tous traités (R11 est une
question de processus, répondue ci-dessous). Remédiation par l'agent de remédiation (Opus 5.5), aucun code, aucun
gate.

**Trend des défauts distincts au-dessus de LOW** : P1 **1 HIGH + 3 MEDIUM** → P2 **4 MEDIUM** → P3 **1 MEDIUM**.

| Finding | Sévérité | Décision |
|---|---|---|
| F1 / R1 — AC12 et AC9 promettent un recalcul quand le fichier est modifié sur le disque ; D18 dit elle-même que Chrome échoue l'envoi (`ERR_UPLOAD_FILE_CHANGED` → `NETWORK_ERROR`, sans `409`) | MEDIUM (régression P2) | **Appliqué** : le recalcul n'est promis que pour le profil et le compte ; pour le fichier, « selon le navigateur » recalcul ou erreur de connexion répétée, sortie *Annuler* puis redéposer — AC12 (encadré du manuel, contrôle PDF « selon le navigateur »), AC9 (D18 cité pour un navigateur qui relit ; erreur réseau nommée), D18 (« serveur : fermé ; écran : non fermé, assumé »), C-27-3-34 annoté ; C-27-3-35 |
| R2 / F3 — l'avis « elle a été recalculée. Vérifiez-la » est faux pendant le recalcul et quand il échoue | LOW | **Appliqué** : « Kesh la recalcule. Vérifiez le résultat avant de confirmer. », quatre locales et repli (AC9, AC10) ; C-27-3-36 |
| R6 — remise à zéro des cases sur la voie `409` sans test | LOW | **Appliqué** : variante « cases » de V6, mutation M26, T3 (« toute raison, `'stale'` comprise ») ; C-27-3-37 |
| R5 / F7 — 3e site d'effacement de l'avis sans mutation ; `isConfirming` dans `reset()` inobservable | LOW | **Appliqué** : M25 → V6-bis (c) ; `isConfirming` dans `reset()` écrit défensif, non éprouvé, angle mort déclaré (AC3, T3) ; C-27-3-38 |
| R7 / F10 — tableau des fichiers communs : `Makefile` absent, PDF de la 27-1, `api-external.md` « à vérifier », chiffres périmés | LOW | **Appliqué** : `make user`, seul `user-manual.pdf` committé, table des matières contrôlée (AC12, T6) ; lignes `Makefile` et PDF ajoutées ; `api-external.md` relevé § 4 / 7 / 9 / 10 (conflit probable au § 10) et AC13 précisé (sous-section après *Annuler un rapprochement bancaire*, la ligne « Imports bancaires » est de la 27-1) ; chiffres relevés au commit `3c36f721` de la 27-1 ; C-27-3-42 |
| R8 — `sprint-status.yaml` périmé (16 AC, choix 1 à 19, « P1 à lancer ») | LOW | **Appliqué** : lignes 1 et 583 recomptées (18 AC, 33 tests, 28 mutations, D1-D19, choix 1 à 44, P4 ciblée à lancer) |
| R3 / F2 — « précision à la seconde » faux (`DATETIME(6)`) ; cas « aperçu explicite, création sans id » non nommé | LOW | **Appliqué** : D17 et AC13 sans le sous-cas, cas voisin nommé (`409` ou `404`) ; C-27-3-7 corrigé ; C-27-3-39 |
| F4 — titre, énoncé, CHANGELOG imputent le refus au seul profil | LOW | **Appliqué** : « le profil, le compte ou le fichier » au titre, à l'énoncé et au CHANGELOG ; C-27-3-27 annoté ; C-27-3-44 |
| F5 — V6-bis : trois rendus dans un même test ; mock de V13 qui fuit | LOW | **Appliqué** : V6-bis en trois `it` distincts ; V13 remet le mock en `afterEach` |
| F6 — raison de l'indicateur non nommée comme état | LOW | **Appliqué** : `previewReason` (`$state`), posé par chaque `runPreview` (AC2, T3) ; C-27-3-41 |
| F8 — un CAMT relancé à chaque changement de profil | LOW | **Assumé**, écrit à l'AC1 ; C-27-3-40 |
| F9 — M10 ne nomme que des tests CSV | LOW | **Appliqué** : M10a (CSV) et M10b (CAMT → I10) ; C-27-3-43 |
| R4 — `column_mapping_json` n'est pas une colonne SQL | LOW | **Appliqué** : I4 écrit `column_mapping` (alias de lecture `column_mapping_json`, JSON valide exigé) |
| R9 — `i18n-un-repli-par-cle` décrite « `supplier-invoices` seul » | LOW | **Appliqué** : six préfixes nommés (AC10) |
| R10 — numéros de ligne ; coupure de mots dans le PDF aplati | LOW | **Appliqué** : `:514-521`, `create:1108-1119` (AC8, D13, Dev Notes) ; contrôle PDF en deux morceaux en cas d'échec (AC12) |
| R11 — constat D5 de la P2 à arbitrer | LOW (processus) | **Répondu** ci-dessous |

**Réponse à R11, et constat D5 de la P3 (CLAUDE.md, § splitting préventif, amendement D5).** L'amendement fait
découper sur le **recyclage** : « un défaut qui revient sous une autre forme, ou qui naît du correctif précédent ».
Relus honnêtement, les deux derniers tours y tombent **à la lettre** :

- **P1 → P2, M16.** La P1 a relevé M16 survivante (F4 / R4 : sa première forme n'était tuée par aucun test) ; sa
  remédiation l'a scindée en M16a / M16b et a laissé M16b affectée à I6, où elle est équivalente ; la P2 l'a relevée
  **de nouveau** survivante (F2 / R1). Le constat de la P2 — « les deux figuraient déjà à l'origine, la remédiation
  P1 les a laissés passer sans les créer » — est **vrai au fait** (`git show 19b51ea3:…`, ligne 346), mais il ne
  change pas la qualification : c'est le même défaut (une forme de M16 que rien ne tue) **revenu sous une autre
  forme** après un correctif qui le visait. C'est du recyclage, et la conclusion « aucun n'est un finding P1 revenu
  sous une autre forme » de la P2 était **inexacte**.
- **P2 → P3, D18.** L'unique MEDIUM de la P3 (F1 = R1) **naît du correctif précédent** : la ligne D18 et la phrase
  « ou le fichier modifié sur le disque » de l'AC12 ont été écrites par la remédiation P2 (`3f5a3c15`). La sévérité
  maximale reste égale (MEDIUM → MEDIUM). Le signal est donc levé, et le critère de recyclage rempli.

Ce qui pèse dans l'autre sens, écrit pour l'arbitrage et non pour l'écarter : le nombre de défauts au-dessus de LOW
décroît (4 → 4 → 1) et le HIGH a disparu depuis la P1 ; aucun module neuf ni dispersion (les remédiations n'ont
ajouté que des tests, des mutations et des textes) ; le recyclage porte sur **ce qu'on vient d'écrire** — le motif
exact que la passe ciblée (§ *La passe ciblée*) a été inventée pour traiter —, non sur la conception d'origine ; et
un découpage séparerait l'écran et la défense serveur, alors que D18, l'avis et l'AC12 relèvent des deux à la fois.
**Constat : le critère D5 de découpage est rempli à la lettre en P2 et en P3 ; l'agent de remédiation ne découpe
pas** (ce n'est pas son arbitrage) **et le déclare au Project Lead.** Proposition, si Guy ne découpe pas : P4 en
passe **ciblée** (Haiku, une lentille, braquée sur le seul commit de la présente remédiation), dont le prompt est
versionné (`27-3-validate-prompt-p4-ciblee.md`). ⚠️ La présente remédiation **touche des règles de l'écran** (texte
de l'avis, mutations M25 / M26, `previewReason`) : le critère de clôture d'une passe ciblée — une remédiation qui ne
touche aucune ligne de production — est ici sans objet (la story est en spécification), mais la P4 doit relire ces
prescriptions comme du comportement, pas comme du texte.

**Nouveaux décomptes**, recomptés depuis la fiche (mêmes commandes que la P2) : **18 AC** ; tests **U1-U4 / I1-I12 +
I6-bis / V1-V13 + V6-bis / E1-E2** (**33** — la variante « cases » vit dans V6, sans numéro neuf) ; **28 mutations**
(M1-M9, M10a, M10b, M11-M15, M16a, M16b, M17-M26 ; `grep -cE "^\| M[0-9]+[ab]? \|"` → 28) ; inventaire **D1-D19** ;
choix **C-27-3-1 à 44** (`grep -c "^### C-27-3-"` du registre → 44) ; **24 clés** neuves (`grep -cE "^\| \`bank-import-"`
→ 24) ; aucun site `i18nMsg(` neuf ni retiré par la P3 (textes et prescriptions seuls), donc `sitesTotal`
**2032 → 2063** inchangé, **2072** à la fusion. Prompt de la P4 : `27-3-validate-prompt-p4-ciblee.md` (Haiku, une lentille).

### Validation P4 ciblée (Haiku, une lentille, sur `aeb75b4c`) — 2026-10-12

Rapport : `kesh-gate-logs/27-3-validate-p4-ciblee.md` — 2 MEDIUM, 2 LOW. Vérifiés par l'orchestrateur au `grep` :

| Finding | Sév. | Décision |
|---|---|---|
| P4-1 — l'indicateur lit `previewReason` seul : « Recalcul… » pendant une confirmation qui suit un recalcul | MEDIUM | **Appliqué** : priorité à `isConfirming` (AC2), V9 après un recalcul, mutation M27 ; C-27-3-45 |
| P4-2 — le contrôle « ne contient plus « avant de déposer » » resterait rouge | MEDIUM | **Réfuté** : l'AC12 prescrit le retrait des deux occurrences (`:1697-1698`, `:1794-1796`, fiche l. 282 et 284) — erreur de catégorie (livrable prescrit, pas encore écrit) |
| P4-3 — « selon le navigateur » non exhaustif (un navigateur qui renverrait les octets de l'aperçu importerait sans erreur) | LOW | Laissé : le serveur reste fermé (l'empreinte couvre le hash), seule la phrase de l'écran serait incomplète ; le dev la nuance s'il le juge utile |
| P4-4 — `fetchWithRetry` passe l'API en mode dégradé sur toute erreur réseau, POST compris | LOW | Noté : comportement préexistant, hors périmètre |

Trend de la validation : P1 1 HIGH + 3 MEDIUM → P2 4 MEDIUM → P3 1 MEDIUM → P4 ciblée 1 MEDIUM réel (1 réfuté). Décision
de Guy (2026-10-12) sur le signal D5 : **ne pas découper**. La remédiation P4 ne touche que la fiche et le registre ;
une passe ciblée P5 (Haiku) la relit avant le développement.

### Validation P5 ciblée (Haiku, sur `34de94b7..ac2c7e4c`) — 2026-10-12

Rapport : `kesh-gate-logs/27-3-validate-p5-ciblee.md` — 1 MEDIUM, 6 LOW, tous appliqués par l'orchestrateur : note du
tableau des mutations (« vingt-neuf, M17-M27 ») ; en-tête, provenance du registre, C-27-3-41 et C-27-3-43 annotés
(45 choix, 29 mutations) ; V6-bis (b) tient la variante `'stale'` de M27 ; renvoi « l. 282 et 284 » ; `sprint-status`.
La remédiation ne touche que la fiche, le registre et le registre de sprint : **validation close**.

**Trend complet de la validation** : P1 (Sonnet) 1 HIGH + 3 MEDIUM → P2 (Opus) 4 MEDIUM → P3 (Sonnet) 1 MEDIUM →
P4 ciblée (Haiku) 1 MEDIUM réel, 1 réfuté → P5 ciblée (Haiku) 1 MEDIUM de décompte → close. Signal D5 levé en P2 et
P3, déclaré à Guy : « ne pas découper » (2026-10-12).
