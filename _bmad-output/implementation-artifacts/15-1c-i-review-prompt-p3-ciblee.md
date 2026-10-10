# Prompt — revue de code P3 ciblée, Story 15-1c-i (une lentille, braquée sur la remédiation P2)

*Versionné le 2026-10-10. Une lentille (Haiku), contexte frais, lecture seule. § « La passe ciblée » du `CLAUDE.md` : la
P2 a trouvé un défaut **né de la remédiation P1** ; ce qu'il reste à relire est la remédiation P2.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-15-1c-i`, branche `story/15-1c-i-ecran-postes-ouverts`.
**Le diff à revoir, et lui seul** : `git show 60e045a8` (un commit). Contexte si nécessaire : fiche
`_bmad-output/implementation-artifacts/15-1c-i-ecran-postes-ouverts.md`, Change Log « Revue de code P2 » (ce que le commit
prétend corriger) ; rapports P2 `/home/gcorbaz/devel/kesh-gate-logs/15-1c-i-review-p2-{B,E}.md`.

## Lentille — chasseur de régressions du dernier patch

Pour chaque hunk de `frontend/src/lib/features/open-items/OpenItemsScreen.svelte` :
1. **Saisie de la date** : `onDateChange` n'écrit plus le champ ; `onDateBlur` restaure une valeur non ISO. Une date
   valide saisie puis `blur` est-elle préservée ? un `blur` pendant que `busy` (champ désactivé) pose-t-il problème ?
   un `blur` après une navigation (l'URL a changé mais `asOf` pas encore) restaure-t-il une date périmée ?
2. **`disabled={busy}`** sur le sélecteur et la date : `busy` revient-il toujours à `false` (chemins `finally`) ? un `busy`
   resté vrai figerait l'écran.
3. **Rabattement** : condition `seq === listSeq && items vide && offset > 0` ; `total = 0` → `offset = 0` : une seconde
   lecture vide à l'offset 0 boucle-t-elle ? (une seule relecture prévue.)
4. **Code ressaisi** : `message = null` avant `loadGroup()`.
5. **Les tests ajoutés** (`OpenItemsScreen.test.ts`) prouvent-ils ce qu'ils nomment (mutations P1 à P6 du Change Log) ?
   le test « page vide tardive » place-t-il bien le nouveau compte en page 2 avant la réponse tardive ?
6. **Textes** : `open-items-no-account` dans les quatre FTL, repli Svelte = valeur fr-CH (G13), terme du menu de chaque
   locale (`nav-accounts`).
7. `frontend/tests/e2e/open-items.spec.ts` : le `finally` du scénario (6) délettre-t-il toujours son groupe ?

## Ce que tu rends

Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1c-i-review-p3-B.md`. Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW),
`fichier:ligne`, **preuve** (sortie `grep -nF` copiée, ou code relu cité). ⚠️ Un finding « code absent » ou « patch non
appliqué » exige la sortie d'un `grep -nF` sur le fichier **final** (pas sur le diff). ⛔ **Liste des axes exercés ET non
exercés** (les sept ci-dessus) — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport ; aucune commande qui écrit, compile ou exécute : `scripts/*` (dont
`scripts/prepare-release.sh`), `make`, `git commit`/`add`/`checkout`/`stash`/`reset`/`rebase`, `cargo`, `npm`, `npx`, `node`,
`docker`, `sqlx`, `gh` en écriture, aucune requête SQL ni HTTP. Autorisés : lecture, `grep`, `sed -n`, `git show`/`diff`/`log`.
