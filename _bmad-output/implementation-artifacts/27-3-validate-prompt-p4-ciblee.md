# Prompt — validation P4 **ciblée** de la spec, Story 27-3 (changer de profil après l'aperçu recalcule l'aperçu)

*Versionné le 2026-10-12. **Une seule lentille**, **Haiku**, contexte frais, lecture seule, braquée sur **le seul
commit de la remédiation P3** (`CLAUDE.md`, § *La passe ciblée* : quand une boucle converge, ce qu'il reste à relire
n'est plus la story, c'est la dernière remédiation). Trend des défauts distincts au-dessus de LOW : P1 1 HIGH + 3
MEDIUM → P2 4 MEDIUM → P3 1 MEDIUM. ⚠️ Le Change Log P3 déclare le critère D5 de découpage rempli à la lettre (P2 :
M16 revenue ; P3 : D18 née de la remédiation P2) : l'arbitrage appartient à Guy ; cette passe ne le tranche pas.*

Dépôt `/home/gcorbaz/devel/kesh-27-3`, branche `story/27-3-profil-apres-apercu`.
**Périmètre : `git show aeb75b4c`** (remédiation P3), soit un **diff unique** — ne lis pas la séquence des commits
antérieurs (pour le contexte d'un hunk, lis le fichier **final** au `HEAD`, pas un diff intermédiaire). Fichiers
touchés : la fiche `_bmad-output/implementation-artifacts/27-3-profil-apres-apercu.md`, le registre
`epic-27-choix-autonomes-27-3.md`, `sprint-status.yaml`. Rapports P3 que ce commit remédie :
`/home/gcorbaz/devel/kesh-gate-logs/27-3-validate-p3-{R,F}.md`.

⚠️ La story est **en spécification** : la fiche prescrit du comportement d'écran (texte de l'avis, `previewReason`,
mutations M25 / M26) que le code n'a pas encore. Relis ces prescriptions comme **du comportement à venir**, et juge
si elles sont vraies, faisables et cohérentes entre elles — pas seulement si la phrase est bien écrite.

## La lentille — chasseur de régressions de la remédiation P3

Pour **chaque hunk** du commit, vérifie contre le code au sol (`HEAD` du worktree, base `a2743fd4`) :

1. **D18, AC9, AC12, C-27-3-35 (le MEDIUM de la P3).** Le fichier modifié sur le disque n'est plus promis comme
   recalculé. Vérifie : (a) `frontend/src/lib/shared/utils/api-client.ts:287-296` (`toFetchApiError` →
   `NETWORK_ERROR`, message « Impossible de contacter le serveur… ») et `fetchWithRetry` (`:329` et suiv. : un `POST`
   est-il rejoué ? `setDegraded()` est-il appelé ?) — la fiche dit-elle **plus** que ces lignes ? ; (b) que
   `createBankImport` passe bien par ce client (`frontend/src/lib/features/bank-import/bank-import.api.ts:63-72`) et
   que le `catch` de `handleConfirm` (`BankImportUpload.svelte:177-187`) affiche `err.message` ; (c) que « *Annuler*,
   puis redéposer » sort réellement de l'impasse (`reset()`, `:75-94`, purge le fichier ; un nouveau dépôt crée un
   nouvel objet `File`) ; (d) que le texte prescrit au manuel est vrai **quel que soit** le comportement du
   navigateur ; (e) **propagation** : reste-t-il dans la fiche ou le registre, **hors** Change Logs P1/P2 et hors
   contextes historiques des choix, une phrase qui promet un recalcul pour le fichier ? Refais :
   `grep -nE "sur le disque|D18|fichier modifié" _bmad-output/implementation-artifacts/27-3-profil-apres-apercu.md _bmad-output/implementation-artifacts/epic-27-choix-autonomes-27-3.md`.
2. **Avis du `409`, C-27-3-36.** Le texte « … Kesh la recalcule. Vérifiez le résultat avant de confirmer. » : vrai
   dans les trois états (recalcul en cours, réussi, en échec) ? Les quatre locales (AC10, table des clés) disent-elles
   la même chose, sans contresens (de-CH « berechnet sie neu », it-CH « la ricalcola », en-CH « recomputes it ») ? Le
   repli est-il prescrit identique au fr-CH ? Reste-t-il l'ancien texte ailleurs que dans les Change Logs et le
   contexte de C-27-3-36 ? `grep -nF "a été recalculée"` sur la fiche et le registre.
3. **Tests et mutations neufs ou modifiés** — chacun est-il **faisable**, et ferait-il **rougir** la mutation qu'on
   lui attribue (et **seulement** par le test nommé, sans être tué par erreur de catégorie) ?
   - variante « cases » de V6 / M26 : `warnings.duplicateFile` (`bank-import.types.ts:46`), case
     `confirm-duplicate-file` (`BankImportUpload.svelte:399`), *Confirmer* désactivé tant qu'elle n'est pas cochée
     (`:220`) ;
   - M25 → V6-bis (c) ; M20 → V6-bis (a) et (b) seulement ; V6-bis en trois `it` distincts ;
   - M10a (CSV → I4, I5, I7, E2) et M10b (CAMT → I10) : chaque sous-cas CAMT d'I10 rougit-il sous M10b ?
   - V13 et son `afterEach` ;
   - `previewReason` (AC2, T3, C-27-3-41) : « posé par chaque `runPreview` avant son premier `await`, jamais lu
     périmé, `reset()` n'y touche pas » — vrai, y compris après un changement de compte pendant un aperçu (V8) et
     pendant une confirmation (texte `common-loading`, V9) ? V2 et V13 (b) tiennent-ils vraiment le texte ?
   - `isConfirming` dans `reset()` déclaré **défensif, non éprouvé** (AC3, C-27-3-38) : énumère **tous** les sites
     d'appel de `reset()` au code (`grep -nF "reset" frontend/src/lib/features/bank-import/BankImportUpload.svelte`)
     et ceux que la fiche prescrit ; un seul est-il atteignable pendant une confirmation ?
4. **D17, AC13, C-27-3-7, C-27-3-39.** `crates/kesh-db/migrations/20260505000001_bank_profiles.sql:34`
   (`DATETIME(6)`), `crates/kesh-db/src/repositories/bank_profiles.rs:175-190` (`ORDER BY updated_at DESC`). Le cas
   neuf « aperçu sur un profil explicite, création avec l'empreinte et **sans** `bankProfileId` → `409` ou `404` » est-il
   vrai à `create_csv` (`crates/kesh-api/src/routes/bank_imports.rs:1781-1809`, `resolve_csv_profile:700-724`) ?
   Placement prescrit dans `docs/api-external.md` : § 7 après *Annuler un rapprochement bancaire* (`:543` à la base),
   § 10 (`:633`) ; la ligne « Imports bancaires » est-elle bien absente de la base
   (`git show a2743fd4:docs/api-external.md | grep -nF "bank-imports"`) ?
5. **Fichiers communs avec la 27-1, AC12 `make user`, numéros de ligne.** Recompte **depuis la source** les
   chiffres du tableau des fichiers communs : `git -C /home/gcorbaz/devel/kesh-27-1 diff --numstat a2743fd4 3c36f721 -- <fichier>`
   et `git -C /home/gcorbaz/devel/kesh-27-1 diff -U0 a2743fd4 3c36f721 -- <fichier> | grep '^@@'` (le **commit**
   `3c36f721`, pas l'arbre de travail de la 27-1, qui bouge). `docs/manual/Makefile:44` (`fr`) et `:48` (`user`).
   I4 : `crates/kesh-db/test-schema/0001_schema_squash.sql:175` (`column_mapping`), `bank_profiles.rs:21-24` (alias).
   AC10 : `frontend/src/lib/shared/i18n-un-repli-par-cle.test.ts:51-71` (six préfixes). Dev Notes : `bank_imports.rs:514-521` ;
   AC8 / D13 : `:1108-1119`.
6. **Manuel contre code (obligatoire), `.tex` ET PDF aplati.** `docs/manual/fr/user-manual.tex:1697-1698`,
   `:1794-1807`, `:1743`, `:1784`, `:1826` contre le texte prescrit par l'AC12 (encadré neuf, « Restent vraies ») ; le
   PDF de la base aplati — `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (aplatir **avant**
   de greper) — : les contrôles prescrits sont-ils **discriminants** (faux à la base, vrais après), y compris le
   contrôle neuf « selon le navigateur » ?
7. **Recomptes** écrits par le commit — recompte depuis les fichiers, ne relis pas les nombres : 21 findings P3,
   5 doublons, 16 distincts (1 MEDIUM, 15 LOW) contre les deux rapports P3 ; 18 AC
   (`grep -cE "^\*\*AC[0-9]+(-bis)? "`) ; 33 tests (`grep -oE "^- \*\*[UIVE][0-9]+(-bis)?\*\*" | wc -l`) ; 28 mutations
   (`grep -cE "^\| M[0-9]+[ab]? \|"`) et leur liste « M1-M26 » partout où elle est écrite (AC14, T8, pied du tableau) ;
   D1-D19 ; 44 choix (`grep -c "^### C-27-3-"` du registre) ; 24 clés ; lignes 1 et 583 de `sprint-status.yaml` ; et
   les **faits** du constat D5 (P2 : `git show 19b51ea3:_bmad-output/implementation-artifacts/27-3-profil-apres-apercu.md | grep -nF "M16"` ;
   P3 : `git show 3f5a3c15 | grep -nF "sur le disque"`). Tu ne juges **pas** l'arbitrage D5 : seulement si les faits
   cités sont exacts.

⚠️ **Erreur de catégorie** (consigne 5 des agents) : un livrable **prescrit par la fiche mais absent du code** n'est
**pas** un défaut — la fiche est une spec, le code n'est pas encore écrit (« `previewReason` n'existe pas dans
`BankImportUpload.svelte` », « M25 n'est pas jouée », « la sous-section n'est pas dans `api-external.md` » sont
attendus). Un finding ne porte que sur ce que **le texte du commit affirme du code existant**, ou sur ce qu'il
**prescrit de faux, d'infaisable ou de contradictoire**.
⚠️ **Grep ground-truth obligatoire** : toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée
(`-F` toujours : le code Rust et TS est plein de métacaractères). Un CRITICAL ou HIGH qui affirme qu'un patch « n'a pas
été appliqué » ou qu'une phrase « est encore là » se vérifie au `grep -nF` sur le fichier **final** (`HEAD`) avant
d'être écrit — c'est le mode d'échec connu de Haiku sur les diffs (`CLAUDE.md`, § *Haiku-specific guardrails*).

## Ce que tu rends

Findings numérotés `P4-1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve** (commande et
sortie copiée), origine (**régression P3** : introduit par `aeb75b4c` ; ou **antérieur**), correction proposée.
⛔ **La liste des axes exercés ET non exercés** (les sept ci-dessus, sous-point par sous-point quand l'axe en a) — un
« 0 finding » sans elle ne compte pas comme passe, et un axe « survolé » se déclare non exercé.
Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/27-3-validate-p4-ciblee.md` ; dernier message : le chemin du
rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`,
`latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`merge`/`tag`/`push`,
`cargo`, `npm`, `npx`, `docker`, `gh` en écriture, toute requête SQL. ⛔ **Aucun interpréteur**, sous aucune forme :
`python`, `python3`, `perl`, `ruby`, `node`, `deno`, `bun`, `php`, `lua`, `awk` avec un programme, `bash`/`sh` sur un
script ou avec `-c`, ni un script écrit dans `kesh-gate-logs/` puis lancé. Ne touche à **aucun** autre worktree
qu'en lecture (`/home/gcorbaz/devel/kesh-27-1` : `git -C … diff`/`show`/`log` seulement). Rien de `~/kesh-test/`
(relevé réel de Guy). Autorisés : lecture (`cat`, `head`, `tail`, `sed -n`), `grep`, `wc`, `sort`, `uniq`, `tr`,
`git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard ou vers un fichier de
`kesh-gate-logs/`.
