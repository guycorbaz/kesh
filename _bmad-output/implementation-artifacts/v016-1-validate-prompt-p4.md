# Prompt — validation P4 de la fiche v016-1 (correctif 0.16.1 : #811, #812), passe ciblée, une lentille, Opus

Tu valides la **dernière remédiation** de la fiche de la Story v016-1 de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit, MariaDB 10.11) : le correctif 0.16.1 du serveur MCP (#811 : retirer `structuredContent` de tout résultat de
`tools/call` ; #812 : une seule forme de montant ; retraits de « non éprouvé » au manuel d'après la recette Claude Code du
2026-10-28).

Worktree `/home/gcorbaz/devel/kesh-0161`, branche `story/v016-1-mcp-sorties`. Langue du rapport : **français**. Modèle :
**Opus**. Passe **ciblée** (quatrième passe de validation) : la P3 (Sonnet) a rendu 1 MEDIUM et 11 LOW, remédiés au
commit `0e1f25f3`. **Contexte frais** : ne t'appuie sur aucune passe antérieure, ni sur ce que le Change Log dit avoir
corrigé.

## Le périmètre — le seul diff de la remédiation P3

```sh
git -C /home/gcorbaz/devel/kesh-0161 diff 324b6986..0e1f25f3 -- _bmad-output/implementation-artifacts/
```

Le motif mesuré du dépôt : la sévérité **se déplace vers ce qu'on vient d'écrire**. Relis chaque passage ajouté ou
réécrit par ce diff, **contre le code et contre le reste de la fiche et du registre** (lis-les en entier pour le
contexte : `v016-1-mcp-sorties.md`, `v016-1-choix-autonomes.md`). En particulier :

- **Les critères de la recette de la release** (fiche, § *Ce que la release doit contenir*, point 3 : C1, C2, C3, et la
  règle « chaque réserve se lève sur son critère ») : chacun est-il **décidable** sur une réponse du modèle ? C3 est-il
  vrai — le chemin `/admin/mcp/sends/<n>` n'est-il écrit que dans le texte de l'accusé (`crates/kesh-api/src/mcp/textes.rs`,
  `grep -rnF "/admin/mcp/sends" crates/kesh-api/src`), ni dans `INSTRUCTIONS`, ni dans une description d'outil, ni dans un
  prompt standard ? La correspondance critère → réserve est-elle la même au point 3, au point 10, à l'AC-8 (items 5 et 7),
  à l'AC-9, à l'AC-10 et au registre C-v016-1-7 ?
- **C2** : une écriture de démonstration sur une base neuve fait-elle bien paraître son montant dans `kesh_bilan`
  (`crates/kesh-api/src/mcp/outils/`, l'outil du bilan) ? La forme attendue `4321.09` est-elle celle que produit la règle
  de l'AC-6 ?
- **M7 et M15** (§ *Tests et mutations*) : la mutation décrite produit-elle, au code (`sortie.rs` pour `Montant`,
  `reponse.rs:38-80` pour `un_bloc`), le rouge annoncé, et seulement celui-là ? `echeance_ne_transmet_rien` compare-t-il
  bien le corps entier (sinon M15 ne le ferait pas rougir) ?
- **Les phrases de chaque relecture** (« Passer à la 0.16.1 », CHANGELOG *Modifié*, registre C-v016-1-1) contre
  `textes.rs:196-260`.
- **La tâche 5** (tests de taille, `mcp_outils_e2e.rs:1342`, `:1385`) : les tests nommés mesurent-ils une longueur que la
  forme des montants change ?
- **Le décompte d'issues du point 7** : recompte (`gh issue list --state open --limit 500 --json number --jq length`) et
  vérifie la décomposition écrite.
- **L'AC-8, ligne l. 3227** : la phrase prescrite est-elle cohérente avec le `.tex` actuel
  (`docs/manual/fr/admin-manual.tex`, l. 3225-3228) ?

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0161-validate-p4.md` :

1. **Les findings**, numérotés `P4-1`, `P4-2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu
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
