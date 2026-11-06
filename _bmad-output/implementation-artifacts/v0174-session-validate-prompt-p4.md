# Prompt — validation P4 de la fiche v0174-session : passe **ciblée** sur la remédiation P3, une lentille, Haiku

Tu fais une **passe ciblée** de validation de la fiche `v0174-session` de Kesh (0.17.4 : jeton de renouvellement stocké en
empreinte, #890 ; avertissement avant la fermeture pour inactivité, #895). La boucle a convergé : P2 (Opus) 0/0/6/9, P3
(Sonnet) 0/0/1/7, toutes deux remédiées. **Ton seul périmètre est le commit de la remédiation P3**, `5d52015d` :

```
git show 5d52015d -- _bmad-output/implementation-artifacts/
```

Ne relis pas la fiche en entier : relis **ce diff aplati** (un seul commit, aucun diff multi-commit à combiner), et, pour
chaque ligne neuve, ce qu'elle affirme. Worktree `/home/gcorbaz/devel/kesh-0174s`, branche `story/v0174-session`, code de
`7bd4d5c0`. Langue du rapport : **français**. Modèle : **Haiku**. Contexte frais.

## À lire

1. `git show 5d52015d` (le diff).
2. Le rapport qui l'a motivé : `/home/gcorbaz/devel/kesh-gate-logs/v0174s-validate-p3.md` (findings P3-1 à P3-8).
3. Pour vérifier une affirmation du diff, et seulement pour cela : les fichiers qu'elle cite (`docker-compose.yml`,
   `docker-compose.prod.yml`, `docs/manual/Makefile`, `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/user-manual.tex`,
   `crates/kesh-api/src/admin_backup/export.rs`, `crates/kesh-i18n/locales/*/messages.ftl`), et le PDF aplati
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers la sortie standard seulement.

## Ce que tu cherches — une seule lentille : la remédiation a-t-elle introduit un défaut ?

- **P3-1** : l'annonce (quatre catalogues prescrits) dit-elle bien la même chose dans les quatre langues, et rien de plus
  que « cliquez sur « Rester connecté » » ? Le manuel utilisateur prescrit est-il descriptif, sans impératif, et vrai (une
  activité « hors du document affiché » garde-t-elle la session au code, `frontend/src/lib/app/stores/session-idle.svelte.ts`) ?
  Reste-t-il, dans les fiches ou le registre, une phrase **vivante** (hors Change Log daté) qui promet la touche ou
  « n'importe où » ?
- **P3-2, P3-3** : la commande en `lstlisting` et la phrase pour `docker-compose.prod.yml` sont-elles justes (services,
  compte, base, mot de passe demandé) ? La recette (b) est-elle exécutable par un agent sans toucher au conteneur du dépôt
  principal ?
- **P3-4, P3-5** : C-v0174s-1 et la dérogation de la sous-fiche 1 disent-ils le même motif ? L'ordre des tâches cité
  est-il celui du § *Tâches* de la sous-fiche 1 ?
- **P3-6** : `make admin` et `make admin user` existent-ils (`docs/manual/Makefile`) ?
- **P3-7** : l'assertion préalable de l'AC-5 (a) est-elle décidable (nom de l'entrée de l'archive) ?
- **P3-8** : le motif `garde qu’une empreinte` est-il absent du PDF **actuel** (attendu « avant » = 0) et présent dans la
  phrase prescrite pour `:3027` (attendu « après » = 1) ?
- **Cohérence** : une ligne du diff contredit-elle une autre partie des fiches ou du registre ?

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0174s-validate-p4.md`. Il
contient :

1. **Les findings**, numérotés `P4-1`, `P4-2`…, avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), lieu, remède.
   ⛔ **Ground-truth obligatoire** : **toute** affirmation de présence ou d'absence d'un texte (dans le diff, une fiche, le
   code, un catalogue, le manuel ou le PDF) s'appuie sur une sortie **`grep -nF "<chaîne exacte>" <fichier>`** copiée dans
   le rapport (`-F` obligatoire : pas de métacaractères) — ou, pour le PDF, sur la sortie de
   `pdftotext <pdf> - | tr '\n' ' ' | tr -s ' ' | grep -oF "<chaîne>" | wc -l`. Un finding sans cette sortie ne compte pas.
   N'affirme jamais qu'une remédiation « n'a pas été appliquée » sans le `grep -nF` qui le prouve sur le fichier **final**.
2. **Les axes exercés** et **non exercés**, un par un — **le manuel et son PDF compris**. Un « 0 finding » sans cette liste
   ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au
  rapport, il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `npx`, ni `make`, ni `latexmk`, ni `python3`, ni
  `node`, ni `sqlx`, ni `docker`, ni `curl`, ni `sha256sum`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`,
  `scripts/install-hooks.sh`, ni aucun autre script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/`, ni `/home/gcorbaz/travail/` ; aucune donnée réelle dans le rapport.
- Rien dans `/home/gcorbaz/devel/kesh` : lecture seule.
- Commandes permises : `cat`, `sed -n`, `/usr/bin/grep` (dans ce shell, `grep` est une fonction vers `ugrep`), `git
  grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`, `git diff` (en lecture), `pdftotext … -` vers
  la sortie standard, `gh issue view`.
