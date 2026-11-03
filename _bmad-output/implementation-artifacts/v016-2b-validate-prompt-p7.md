# Prompt — validation P7 **ciblée** de la sous-fiche v016-2b (#816), une lentille, Sonnet

Tu fais une **passe ciblée** (`CLAUDE.md` § *La passe ciblée*) sur la sous-fiche `v016-2b-releve-manuel-ecran.md` de Kesh (comptabilité suisse,
Rust/Axum, SvelteKit, MariaDB 10.11) : la saisie manuelle d'un relevé bancaire (#816), découpée en `v016-2a` (backend)
et `v016-2b` (écran), release 0.17.0. **Une seule lentille**, braquée sur **le seul diff de la remédiation P6**. Modèle :
**Sonnet**. La sous-fiche `v016-2a` est **validée** et en cours de développement dans ce
worktree : tu ne la juges pas, tu ne lis d'elle que ce que la 2b invoque. **Contexte frais**. Langue : **français**.

Worktree `/home/gcorbaz/devel/kesh-0162`, branche `story/v016-2-releve-manuel` ; code et manuels de `c5101b07` (la branche
ne touche que `_bmad-output/`).

## À lire

1. Le diff de la remédiation P6 : `git log --oneline -15`, puis
   `git show <commit « docs(v016-2b) : remédiation de la validation P6 »> -- _bmad-output/` — toutes les lignes
   ajoutées ou récrites de `v016-2b-releve-manuel-ecran.md`, de `v016-2a-releve-manuel-backend.md`, de la parente `v016-2-releve-manuel.md` et du registre
   `v016-2-choix-autonomes.md` (dont l'entrée **C-v016-2-19**).
2. Le rapport P6 qui l'a motivé : `/home/gcorbaz/devel/kesh-gate-logs/v0162b-validate-p6.md`, et l'entrée « Validation P6 »
   du Change Log de la 2b.
3. Le code que le diff cite, ligne par ligne (`sed -n`, `grep -nF`).

## Axes (tous obligatoires)

1. **Le diff P6 de cette sous-fiche.** Chaque passage ajouté ou récrit : chaque `fichier:ligne` dit-il ce qu'on lui fait
   dire ? Chaque remède tient-il au code sans contredire un autre AC ? Chaque finding déclaré « appliqué » l'est-il
   entièrement (greper la **valeur** corrigée dans les trois fiches et le registre) ? Une entrée passée du Change Log
   a-t-elle été réécrite (interdit) ? Le « LOW non pris » est-il motivé ?
2. **Le point bloquant de « À recontrôler »** — axe principal. Est-il décidable (quoi vérifier, où, et quoi faire si
   ce n'est pas le cas) ? Le renvoi de l'AC-24 vers lui est-il exact, et ne reste-t-il, nulle part dans la 2b, la parente
   ou le registre, une phrase qui tienne encore la garantie de `ALL` pour acquise hors de ce point (`grep -n "C-v016-2-20"`,
   `grep -n "Garantie de la 2a"`) ?
3. **Les gardes i18n**, ligne lue dans `frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts` : la glose corrigée
   (premier test : `:142` ; G13 : `REPLIS_A_SITE_UNIQUE`, `:210`, `:217-236` ; `PREFIXES_REPLI_EGAL_AU_CATALOGUE`, `:279`)
   est-elle exacte ? L'ajout nommé des clés statiques neuves à `REPLIS_A_SITE_UNIQUE` est-il compatible avec la mécanique
   de G13 (site unique, repli égal au catalogue) ? L'angle mort de la famille `bank-import-manual-reason-` est-il écrit
   juste ? Les citations `i18n-harvest.js:32`, `i18n-literal-reader.js:491` et `crates/kesh-i18n/src/loader.rs:1193`,
   `:1308` (G8, G8-bis, « rouvrir ») disent-elles ce qu'on leur fait dire ?
4. **La phrase de l'AC-23 remise en ordre** et le point 10 de la liste C de la parente : lisibles, sans perte de sens, sans
   réécriture d'une entrée passée du Change Log ?

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0162b-validate-p7.md`. Il
contient :

1. **Les findings**, numérotés `P1`, `P2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu
   (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède proposé. Une affirmation de
   présence ou d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans
   cette liste ne clôt rien et ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au
  rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit (`issue create|edit|close|comment`, `pr …`) ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni `make`, ni
  `latexmk`, ni `python3`, ni `node`, ni `sqlx`, ni `docker`, ni `mariadb`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`,
  ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
