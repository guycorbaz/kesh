# Prompt — validation P4 de la fiche v0172-correctifs (release 0.17.2), passe **ciblée** sur la remédiation de la P3, Opus

Tu valides **la seule remédiation de la validation P3** de la fiche de la Story v0172-correctifs de Kesh (comptabilité suisse,
Rust/Axum, SvelteKit, MariaDB 10.11 ; release corrective 0.17.2, sans migration ; sous-stories A2 #840, B #870/#851/#882/#885,
C #880/#883 ; #876 est dans la fiche sœur `v0172-session.md`, **hors de cette passe**). La P3 (Sonnet, complète) a rendu
0/0/4/5 ; rapport : `/home/gcorbaz/devel/kesh-gate-logs/v0172b-validate-p3.md`. Tous ses findings ont été appliqués.

**Passe ciblée** (`CLAUDE.md` § *La passe ciblée*) : une seule lentille, braquée sur ce que la remédiation a écrit — *la sévérité se
déplace vers ce qu'on vient d'écrire*. Le périmètre est le diff du commit de remédiation, premier commit après `2c34f396` sur la
branche `story/v0172-correctifs`, dont le message commence par `docs(v0172) : remédiation de la validation P3` :

```
git log --oneline 2c34f396..HEAD
git show <ce commit> -- _bmad-output/implementation-artifacts/v0172-correctifs.md _bmad-output/implementation-artifacts/v0172-choix-autonomes.md
```

Worktree `/home/gcorbaz/devel/kesh-0172b`. Langue du rapport : **français**. Modèle : **Opus**. Contexte frais : relis au code chaque
affirmation du diff ; ne t'appuie pas sur ce que la fiche dit d'elle-même. Le rapport P3 se lit pour savoir ce qui était demandé.

## Axes à exercer

**La remédiation — chaque hunk du diff**

- **P1, l'expiration d'une clé API** (§ 24, lignes 24 et 24 b ; § C.1, exception ; AC-C2, cas du 31.12 ; MC5 ; filtre d'AC-C3) :
  relis `frontend/src/routes/(app)/settings/api-keys/+page.svelte` en entier et `crates/kesh-api/src/routes/api_keys.rs`. L'expiration
  est-elle bien toujours une date de calendrier (création, modification, valeur par défaut `:51`, clé sans expiration) ? Les lignes
  `:94`, `:99`, `:104`, `:371`, `:372` sont-elles rangées juste (instant ou date) ? `isExpired` (`:56`) compare-t-il de façon
  cohérente avec l'affichage (une clé « Expirée le 31.12.2026 » est-elle bien expirée à l'instant affiché) ? Le filtre d'AC-C3 avec
  `formatExpiryDate\(` garde-t-il les décomptes 51 / 20, sur `d7ec2ea2` et après correction ?
- **P2, l'aspect de la barre** (conception de B.2, AC-B5 ter, MB3 ter, TB4 ter) : relis `routes/(app)/+layout.svelte:290-445`. Le
  conteneur étiré (`aside`) qui porte fond et filet dans `<div class="flex flex-1">` couvre-t-il la fenêtre pour tous les rôles,
  sans bandeau et avec un bandeau, page courte et page longue ? La `<nav>` `sticky top-14` à `max-h-[…]`, **dans** l'`aside`,
  reste-t-elle collée (le conteneur `sticky` est-il bien l'`aside` étiré) ? `padding`, `data-testid` et `aria-label` sont-ils gardés ?
  AC-B5 ter est-il décidable et vert avant correctif ? L'oracle du symptôme (AC-B5) reste-t-il juste avec cette forme ?
- **P3, le `page` réactif** (AC-B2 bis, tests) : relis `routes/(app)/journal-entries/[id]/page-state.test-helper.svelte.ts` et le test
  qui l'importe (`journal-entry-page.test.ts`, son `vi.mock('$app/state')`). La recette prescrite (`$state({ url })`, réaffectation
  par le simulacre de `goto`, `get page()`) est-elle réalisable avec `vi.mock` hissé et un module `.svelte.ts` ? La sonde sous MB8
  est-elle décidable ?
- **P4, la réactivation d'un taux** (§ 15 bis, manuel #882, C-v0172b-29) : relis `crates/kesh-api/src/routes/vat.rs:300-440`,
  `crates/kesh-api/src/lib.rs:275-290`, `crates/kesh-db/src/repositories/vat_rates.rs:180-220` et
  `frontend/src/routes/(app)/settings/vat-rates/+page.svelte`. Les lignes citées disent-elles ce que la fiche leur fait dire ? La
  phrase prescrite au manuel est-elle vraie ? Le choix de ne pas ajouter de bouton est-il cohérent avec #882 (l'utilisateur sait-il
  quoi faire de son brouillon) ?
- **Les LOW** : P5 (les assertions de requête vs les réponses simulées de `supplier-invoices-list-page.test.ts`, la ligne de
  `FiscalYearChecks.test.ts:176-178`) ; P6 (C-v0172b-1 et C-v0172b-10 récrits : « Retenu » et « Écartées » à l'état en vigueur, sans
  contradiction avec la fiche) ; P7 (la commande de contrôle du PDF, `sed "s/’/'/g"` : juste, et ne masque-t-elle rien ?) ; P8 (les
  **12** lignes listées pour #889 : recompte par `git grep -nE "formatSwissDate\(.*\.slice\(0, 10\)" -- frontend/src`, une ligne
  manque-t-elle, une est-elle en trop ; l'affirmation « la conversion de C.1 ne les touche pas » est-elle vraie ?) ; P9
  (`localStorage` vidé : clés exactes, `+layout.svelte:206-209` ; relevé (1) non bloquant).
- **Propagation** : greppe dans la fiche et le registre les valeurs et formes remplacées (`self-start`, « ne se réactive pas »,
  `vat_rates.rs` comme chemin de route, `limit: 100`, « 1 à 28 », « onze ») : une occurrence vivante hors des entrées datées et des
  mentions historiques est un résidu.

**Ce qui n'est pas dans la passe** : le reste de la fiche, déjà validé ; ne le reprends que si un hunk du diff le contredit.

## Ce qu'il faut rendre

Un rapport en Markdown à `/home/gcorbaz/devel/kesh-gate-logs/v0172b-validate-p4.md`, **écrit par l'outil d'écriture de fichier
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
