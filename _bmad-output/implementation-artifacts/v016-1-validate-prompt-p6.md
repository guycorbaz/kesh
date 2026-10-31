# Prompt — validation P6 de la fiche v016-1 (correctif 0.16.1 : #811, #812), passe ciblée, une lentille, Opus

Tu valides la **dernière remédiation** de la fiche de la Story v016-1 de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit, MariaDB 10.11) : le correctif 0.16.1 du serveur MCP (#811 : retirer `structuredContent` de tout résultat de
`tools/call` ; #812 : une seule forme de montant ; retraits de « non éprouvé » au manuel d'après la recette Claude Code du
2026-10-28).

Worktree `/home/gcorbaz/devel/kesh-0161`, branche `story/v016-1-mcp-sorties`. Langue du rapport : **français**. Modèle :
**Opus**. Passe **ciblée** (sixième passe de validation) : la P5 (Sonnet) a rendu 1 MEDIUM et 5 LOW, remédiés au commit
`aacaa953`. La recette Claude Code de la release se joue sur une release candidate `v0.16.1-rc1` ; son protocole est
hors de la fiche, par décision du Project Lead. **Contexte frais** : ne t'appuie sur aucune passe antérieure, ni sur ce que le Change Log dit avoir
corrigé.

## Le périmètre — le seul diff de la remédiation P5

```sh
git -C /home/gcorbaz/devel/kesh-0161 diff cd011a49..aacaa953 -- _bmad-output/implementation-artifacts/
```

Le motif mesuré du dépôt : la sévérité **se déplace vers ce qu'on vient d'écrire**. Relis chaque passage ajouté ou
réécrit par ce diff, **contre le code, le dépôt et le reste de la fiche et du registre** (lis-les en entier :
`v016-1-mcp-sorties.md`, `v016-1-choix-autonomes.md`). En particulier :

- **Le point 8** (§ *Ce que la release doit contenir*) : l'ordre des gestes — PR de release ouverte, tag `v0.16.1-rc1`
  sur la tête de la branche, constats après la candidate, recette, points 2, 4 et 5 rejoués, fusion, tag final sur le
  commit de fusion — est-il exécutable et sans contradiction avec les points 1 à 10, le point 3 et le dépôt ?
  Lis `.github/workflows/release.yml` (quel fichier de workflow s'exécute pour un tag posé hors de `main`), `.github/workflows/deploy-pages.yml`
  et `.github/workflows/ci.yml` (déclencheurs), la règle de squash et de `closes` de `CLAUDE.md` (§ *Issue Tracking Rule*) :
  le tag final posé sur un commit de squash porte-t-il bien ce que la candidate a éprouvé ? Le smoke et la garde G19
  restent-ils justes ?
- **Le point 10** : la condition de `closes #811` et le moment où s'écrit le corps de la PR.
- **Le point 3** : `docs/ci.md:18`, `:163-164` et `docs/manual/fr/admin-manual.tex:2435` — les lignes citées
  sont-elles justes, et un autre site du dépôt dit-il qu'un tag pousse toujours `:latest` (`grep -rnF ":latest"`,
  hors `target/` et `node_modules/`) ?
- **Le registre C-v016-1-3** (l. 75-76) contre la fiche (l. 64).
- **L'entrée P5 du Change Log** : décomptes et prémisses citées.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0161-validate-p6.md` :

1. **Les findings**, numérotés `P6-1`, `P6-2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu
   (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) qui fonde l'affirmation, et le remède.
   Une affirmation de présence ou d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché pour chaque axe exercé.
   Un « 0 finding » sans cette liste ne clôt rien.
3. Un décompte final par sévérité, recompté depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture** hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier jetable ou le scratchpad. Un fichier créé par erreur se déclare au rapport.
- Aucun `git checkout`, `switch`, `reset`, `stash`, `commit`, `push`, `tag`, `rebase`.
- Aucune commande `gh` qui écrit ; `gh issue view` et `gh issue list` seulement.
- Aucune commande qui compile ou exécute : ni `cargo`, ni `npm`, ni `make`, ni `latexmk`, ni `python3`, ni `node`, ni
  `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`. Jamais `~/kesh-test/`. Ne lis pas `kesh-gate-logs/3211/capture-recette.log` (il porte une
  clé, révoquée) ; aucune donnée réelle dans le rapport.
- Pour un PDF aplati : `pdftotext … - | tr '\n' ' ' | tr -s ' ' | timeout 60 grep -cF "…"`, vers la sortie standard
  seulement.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`, `git diff`.
