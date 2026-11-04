# Prompt — validation P5 de la fiche v0172-correctifs (release 0.17.2), passe **ciblée** sur la remédiation de la P4, Sonnet

Tu valides **la seule remédiation de la validation P4** de la fiche de la Story v0172-correctifs de Kesh (comptabilité suisse,
Rust/Axum, SvelteKit, MariaDB 10.11 ; release corrective 0.17.2, sans migration ; sous-stories A2 #840, B #870/#851/#882/#885,
C #880/#883 ; #876 est dans la fiche sœur `v0172-session.md`, **hors de cette passe**). La P4 (Opus, ciblée) a rendu 0/0/1/6 ;
rapport : `/home/gcorbaz/devel/kesh-gate-logs/v0172b-validate-p4.md`. Tous ses findings ont été appliqués.

**Passe ciblée** (`CLAUDE.md` § *La passe ciblée*) : une seule lentille, braquée sur ce que la remédiation a écrit — *la sévérité se
déplace vers ce qu'on vient d'écrire*. Le périmètre est le diff du commit de remédiation, premier commit après `0fb984c7` sur la
branche `story/v0172-correctifs`, dont le message commence par `docs(v0172) : remédiation de la validation P4` :

```
git log --oneline 0fb984c7..HEAD
git show <ce commit> -- _bmad-output/implementation-artifacts/v0172-correctifs.md _bmad-output/implementation-artifacts/v0172-choix-autonomes.md
```

Worktree `/home/gcorbaz/devel/kesh-0172b`. Langue du rapport : **français**. Modèle : **Sonnet**. Contexte frais : relis au code chaque
affirmation du diff ; ne t'appuie pas sur ce que la fiche dit d'elle-même. Le rapport P4 se lit pour savoir ce qui était demandé.

## Axes à exercer

**La remédiation — chaque hunk du diff**

- **P1, l'`aside` minoré** (conception de B.2, AC-B5 ter, MB3 quater) : relis `frontend/src/routes/(app)/+layout.svelte:290-450`. Avec
  `min-h-[calc(100vh-3.5rem)]` sur l'`aside`, la boîte est-elle bien celle d'aujourd'hui (bas à `100vh + B`, pied de page sous le pli)
  pour une page courte et une page longue, avec et sans bandeau ? La `<nav>` `sticky top-14 max-h-[…] overflow-y-auto`, dans l'`aside`,
  colle-t-elle toujours (le parent étiré permet-il le collage sur toute la hauteur de la page longue) ? Les assertions d'AC-B5 ter
  (bas de l'`aside`, hauteur, pied de page sous le pli, filet) sont-elles vertes avant et après, et rouges sous MB3 ter et MB3 quater ?
  L'oracle du symptôme (AC-B5) reste-t-il juste ?
- **P2, `isExpired` en UTC** (§ C.1, AC-C2, MC6) : relis `frontend/src/routes/(app)/settings/api-keys/+page.svelte` (`:51-110`, `:140-160`)
  et `crates/kesh-db/src/repositories/api_keys.rs:120-140`. La lecture UTC de `expiresAt` est-elle juste pour toutes les valeurs que le
  serveur rend (expiration posée par l'écran, clé sans expiration, clé créée par l'API avec une heure quelconque) ? Le cas de bord
  (`2026-12-31T23:30:00Z`) et les horloges fixées sont-ils décidables, et MC6 tuée ?
- **P3, les simulacres** (AC-B2 bis) : relis `routes/(app)/journal-entries/[id]/journal-entry-page.test.ts:1-40` et
  `page-state.test-helper.svelte.ts`. La forme prescrite (fabriques `async`, import dans la fabrique, réaffectation de `pageState.url`)
  est-elle réalisable telle quelle, y compris pour `goto` (simulacre déclaré par `vi.fn(async …)`) ?
- **P4, P5** (registre) : C-v0172b-8 (« Retenu » à l'état en vigueur), C-v0172b-9, C-v0172b-10 : plus de contradiction avec la fiche ?
- **P6, le contrôle du PDF** : la seconde substitution `sed -E 's/([[:alpha:]])- ([[:alpha:]])/\1\2/g'` est-elle juste (accents :
  `[[:alpha:]]` sous la locale du shell), et sa limite bien dite ?
- **P7** : `+layout.svelte:224-227` et `:242-246` disent-ils ce que la fiche leur fait dire ?
- **Propagation** : greppe `:371` (pour `+layout.svelte`), `:225-227`, « qui n'existe pas », « ci-dessous », `self-start`, « au moins celle
  de la fenêtre » : une occurrence vivante hors des entrées datées et des mentions historiques est un résidu.

**Ce qui n'est pas dans la passe** : le reste de la fiche, déjà validé ; ne le reprends que si un hunk du diff le contredit.

## Ce qu'il faut rendre

Un rapport en Markdown à `/home/gcorbaz/devel/kesh-gate-logs/v0172b-validate-p5.md`, **écrit par l'outil d'écriture de fichier
(`Write`), une seule fois**. Il contient :

1. **Les findings**, `P1`, `P2`…, avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le hunk visé, le lieu (`fichier:ligne`), **la
   ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède. Une affirmation de présence ou d'absence sans sortie
   `grep -nF` copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans cette liste ne
   compte pas.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture hors du rapport**, et le rapport par l'outil `Write` seulement : **aucune redirection** (`>`, `>>`, `2>`, `&>`,
  `| tee`), **ni vers un fichier, ni vers `/dev/null`** ; **aucune option qui écrit un fichier** (`-o`, `--output`, `pdftotext` vers
  un fichier) ; **aucun fichier temporaire** — ni dans `/tmp`, ni dans le scratchpad, ni ailleurs. `pdftotext` s'emploie avec `-`
  (sortie standard) seulement. Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous aucune forme.
- Aucune commande `gh` qui écrit ; `gh api` en `GET` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `npx`, ni `make`, ni `latexmk`, ni `python3`, ni `node`, ni
  `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- **Jamais `scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun autre script du dépôt.
- Jamais `rm`, `pkill`, `killall`. Jamais `~/kesh/`, `~/kesh-test/`, ni le NAS ; aucune donnée réelle dans le rapport.
- Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- Commandes permises : `cat`, `sed -n`, `/usr/bin/grep`, `git grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `awk` (lecture, sans
  redirection), `git log`, `git show`, `git diff` (en lecture), `pdftotext … -`, `gh api` en `GET`.
