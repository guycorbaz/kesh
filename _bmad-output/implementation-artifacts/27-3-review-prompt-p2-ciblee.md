# Prompt — revue de code P2 ciblée, Story 27-3 (une lentille, Haiku)

*Versionné le 2026-10-12, par l'agent de remédiation de la revue de code P1. Contexte frais, lecture seule.*

Worktree `/home/gcorbaz/devel/kesh-27-3`, branche `story/27-3-profil-apres-apercu`. **Le seul diff à relire**, aplati
(un seul diff, pas la suite des commits — CLAUDE.md, § *Haiku-specific guardrails*) :

```
git diff 3800e4d0 8c53cf80 -- . ':(exclude)_bmad-output'
```

`3800e4d0` = état revu par la P1 ; `8c53cf80` = **dernier commit de code** de la remédiation P1. Les commits qui suivent
ne portent que des textes : manuel (`.tex` + PDF), `CHANGELOG.md`, `docs/api-external.md`, fiche, registre, ce prompt —
relis-les avec `git diff 3800e4d0 HEAD -- docs CHANGELOG.md` pour l'axe 5 seulement. ⚠️ Pathspec `':(exclude)…'`, et
**non** `':!…'`. Fiche : `_bmad-output/implementation-artifacts/27-3-profil-apres-apercu.md` (Change Log, section
*Revue de code P1*) ; registre : `_bmad-output/implementation-artifacts/epic-27-choix-autonomes-27-3.md` (C-27-3-51 à
56) ; rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/27-3-review-p1-{B,E,A}.md` ; journal des mutations :
`/home/gcorbaz/devel/kesh-gate-logs/27-3-mutations.log` (lignes M28 et suivantes) et `27-3-mut/M28.diff` etc.

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de `kesh-gate-logs/` (rien sous
`~/kesh-test/`).

## Ce que la remédiation a changé (à vérifier, pas à croire)

- **Production** — `frontend/src/lib/features/bank-import/BankImportUpload.svelte` : `loadProfiles()` partagée par
  `onMount` et la branche `409 BANK_IMPORT_PREVIEW_STALE` de `handleConfirm` (appel non attendu, après le recalcul) ; un
  échec de la relecture **garde** la liste en place (B-1, C-27-3-51). C'est la **seule** modification de code de
  production.
- **Tests** — `BankImportUpload.test.ts` : V14 (confirmation rejetée hors `409`, E-1), V4 prolongé par une confirmation
  (profil résolu, E-2), V9 + `bank-import-confirm` désactivé (E-3), `previewWithA` + `profileFlag(calls) === A` (E-4),
  V6-ter et sa variante (B-1) ; `crates/kesh-api/tests/bank_imports_e2e.rs` : I8, cas « vide » (E-5) ;
  `frontend/tests/e2e/bank-csv-import.spec.ts` : `RUN`, `E1_YEAR`, noms suffixés (E-6) ; neuf
  `frontend/src/routes/(app)/bank-import/bank-import-page.test.ts` (E-7).

## Axes (chacun déclaré exercé ou non)

1. **`loadProfiles` ne régresse pas le montage.** Avant : `catch { availableProfiles = [] }` ; après : la liste en place
   est gardée. Au montage, est-ce bien équivalent (valeur initiale `[]`) ? `onMount(loadProfiles)` : une fonction
   `async` passée directement à `onMount` — un `Promise` rendu y est-il inoffensif (Svelte 5 n'appelle que le retour
   *fonction*) ? Le sélecteur (`{#if availableProfiles.length > 0}`) peut-il disparaître ou apparaître au mauvais moment
   après un `409` ?
2. **La relecture à la branche `409`, et ses courses.** Elle n'est pas attendue et n'est gardée par aucune séquence :
   une réponse qui arrive **après** un changement de compte, un `reset()`, un second `409` ou un démontage peut-elle
   poser un état faux ? (La liste est celle de la société, pas du compte : dis si c'est sans effet, et pourquoi.) Le
   profil de l'instantané supprimé ailleurs : que montre le sélecteur une fois la liste relue (valeur `bankProfileId`
   absente des options), et le manuel le dit-il juste ?
3. **Les tests neufs prouvent-ils ce qu'ils nomment ?** Pour chacun (V14, V4 prolongé, V9, `previewWithA`, V6-ter et
   variante, I8 « vide », test de page), le montage construit-il l'état voulu, et la mutation journalisée (M28 à M35)
   le fait-elle rougir **pour la raison nommée** ? Cherche une **mutation plausible du diff qui resterait verte** (ex. :
   la relecture déplacée avant `runPreview` ; `loadProfiles` appelée aussi hors `409` ; I8 « vide » rendu vert par un
   autre champ manquant ; le test de page vert parce que `Chargement` n'apparaît jamais, quel que soit le site). Un
   `mockResolvedValueOnce` / `mockRejectedValueOnce` posé **avant** l'appel qu'il vise est-il bien consommé par lui, et
   non par l'`onMount` ?
4. **E2E rejouable (E-6).** `RUN` et `E1_YEAR` sont évalués une fois par fichier : deux tests du même fichier
   partagent-ils un nom ? `E1_YEAR` (2100 à 9099) est-il toujours une année valide pour le format `%Y-%m-%d` du profil,
   et jamais 2026, 2027 ou 2028 ? La fixture remplacée (`replaceAll('2026-', …)`) garde-t-elle 11 transactions et la
   somme attendue ?
5. ⛔ **LE MANUEL — `docs/manual/fr/user-manual.tex` ET `user-manual.pdf`**, et `CHANGELOG.md`, `docs/api-external.md`.
   Chaque phrase neuve est-elle vraie au code : « l'IBAN du compte bancaire » (l'empreinte, `bank_import_fingerprint.rs`,
   ne voit du compte que son id et son IBAN normalisé) ; « relit la liste des profils » ; « si le recalcul échoue …, le
   message d'erreur s'affiche avec l'avis » ; « nom de la banque, description … ne bloquent pas l'import » ; « avant tout
   parse du fichier (ses octets sont déjà lus) » pour CSV **et** CAMT ; le titre « Notes du tableau des ressources » et
   ses renvois ¹ ² ³. Contrôle le **PDF aplati** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`
   vers `/home/gcorbaz/devel/kesh-gate-logs/27-3-review-p2-ciblee-manuel.txt` ; l'apostrophe y est typographique,
   écrire `.` à sa place dans un motif).
6. **Propagation** : le symptôme corrigé (« compte bancaire est modifié », « avant toute lecture du fichier »,
   « 2063 », « 2072 ») subsiste-t-il ailleurs hors des Change Logs historiques ? (`grep -rnE`, en excluant
   `node_modules` et `target`.)

⚠️ **Erreur de catégorie connue** : un livrable que la fiche prescrit mais qui n'est pas dans ce diff n'est pas un
défaut de la remédiation. Toute affirmation de présence ou d'absence d'un code ou d'un texte s'appuie sur une sortie
`grep -nF` **copiée**, prise sur le fichier final (`grep -nF "<chaîne>" <fichier>`, `-A <N>` pour un bloc) ; un finding
CRITICAL ou HIGH sans elle sera écarté.

## Interdits

Lecture seule. Tu n'écris que `/home/gcorbaz/devel/kesh-gate-logs/27-3-review-p2-ciblee.md` (et le texte aplati du PDF
nommé à l'axe 5). ⛔ **Aucune commande qui compile, exécute ou écrit** : ni `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), ni `make`, `latexmk`, `xelatex`, ni
`cargo` (aucune sous-commande), `sqlx`, `npm`, `npx`, `node`, ni **aucun interpréteur** — `python`, `python3`, `perl`,
`ruby`, `awk` avec un programme, `bash -c`, `sh -c` —, ni `docker`, ni `git` en écriture (`commit`, `add`, `checkout`,
`switch`, `stash`, `reset`, `rebase`, `apply`), ni `gh` en écriture, ni SQL, ni HTTP. Autorisés : lecture (`Read`),
`grep`, `sed -n`, `wc`, `git log` / `show` / `diff`, `pdftotext` vers `kesh-gate-logs/`.

## Rapport

Dans `/home/gcorbaz/devel/kesh-gate-logs/27-3-review-p2-ciblee.md` : décompte par sévérité (CRITICAL / HIGH / MEDIUM /
LOW) ; chaque finding avec `fichier:ligne`, **preuve** (commande et sortie copiées, ou code relu cité), correction
proposée ; ⛔ **la liste des axes exercés ET non exercés** (les six ci-dessus) — un « 0 finding » sans elle ne compte
pas comme passe.
