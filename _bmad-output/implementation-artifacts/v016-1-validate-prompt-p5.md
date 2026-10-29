# Prompt — validation P5 de la fiche v016-1 (correctif 0.16.1 : #811, #812), passe ciblée, une lentille, Sonnet

Tu valides la **dernière remédiation** de la fiche de la Story v016-1 de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit, MariaDB 10.11) : le correctif 0.16.1 du serveur MCP (#811 : retirer `structuredContent` de tout résultat de
`tools/call` ; #812 : une seule forme de montant ; retraits de « non éprouvé » au manuel d'après la recette Claude Code du
2026-10-28).

Worktree `/home/gcorbaz/devel/kesh-0161`, branche `story/v016-1-mcp-sorties`. Langue du rapport : **français**. Modèle :
**Sonnet**. Passe **ciblée** (cinquième passe de validation) : la P4 (Opus) a rendu 3 MEDIUM et 5 LOW, remédiés au commit
`687cf600` ; sur décision du Project Lead, le protocole de la recette Claude Code est sorti de la fiche, et la recette
se jouera sur une release candidate `v0.16.1-rc1`. **Contexte frais** : ne t'appuie sur aucune passe antérieure, ni sur ce que le Change Log dit avoir
corrigé.

## Le périmètre — le seul diff de la remédiation P4

```sh
git -C /home/gcorbaz/devel/kesh-0161 diff e9d2eefe..687cf600 -- _bmad-output/implementation-artifacts/
```

Le motif mesuré du dépôt : la sévérité **se déplace vers ce qu'on vient d'écrire**. Relis chaque passage ajouté ou
réécrit par ce diff, **contre le code, le dépôt et le reste de la fiche et du registre** (lis-les en entier pour le
contexte : `v016-1-mcp-sorties.md`, `v016-1-choix-autonomes.md`). En particulier :

- **Le point 3 de la release** (§ *Ce que la release doit contenir*) : la candidate `v0.16.1-rc1` est-elle possible
  telle qu'écrite ? Lis `.github/workflows/release.yml` en entier (déclencheur `v*.*.*`, extraction de la version,
  étiquettes, smoke `/health` et sa comparaison à la base du tag, push, GitHub Release) et `scripts/prepare-release.sh`
  (format de version accepté). Les lignes citées sont-elles justes ? Le correctif annoncé de `:latest` et de
  `prerelease` suffit-il pour qu'une candidate ne déplace pas l'image en service ? Un autre site du dépôt (CI, garde
  G19 de `crates/kesh-api/tests/textes_coherents.rs`, `prepare-release.sh`, manuel d'administration sur `:latest`)
  rend-il la candidate fausse ou dangereuse ?
- **La règle qui reste** (« une réserve ne se lève que sur une observation de la recette ») : est-elle cohérente
  avec l'AC-8 (items 5 et 7), l'AC-9 (*Corrigé*), l'AC-10, les points 8 et 10, et le registre C-v016-1-7 et C-v016-1-10 ?
  Un passage de la fiche ou du registre renvoie-t-il encore à des critères C1, C2 ou C3 hors des entrées passées du
  Change Log (qui sont des relevés datés et ne se corrigent pas) ?
- **L'ordre de la release** : les points 1 à 10 se suivent-ils sans contradiction avec la candidate (datation du
  CHANGELOG, `\keshReleaseDate`, URL du point 4 vers un tag pas encore posé, PDF, tag final) ?
- **Les LOW remédiés** : M7 et M9 (§ *Tests et mutations*) contre l'ordre des cas de l'AC-7 et `sortie.rs` ; la phrase
  « refus, échéance et numéro inconnu » contre `crates/kesh-api/src/mcp/reponse.rs` ; la tâche 5 (`fixture_tailles`,
  `mcp_outils_e2e.rs`) ; `reponse.rs:42-47`.
- **L'entrée P4 du Change Log** : ses décomptes et ses prémisses citées sont-ils justes ?

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0161-validate-p5.md` :

1. **Les findings**, numérotés `P5-1`, `P5-2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu
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
