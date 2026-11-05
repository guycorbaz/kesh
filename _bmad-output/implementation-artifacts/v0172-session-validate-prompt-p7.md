# Prompt — validation P7 de la fiche v0172-session (release 0.17.3 : #876 ; CR #891), passe **ciblée** sur la remédiation de la P6, Sonnet

Tu valides **la seule remédiation de la validation P6** de la fiche de la Story v0172-session de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit, MariaDB 10.11 ; release 0.17.3, sans migration). La P6 (Opus, complète) a rendu 0/0/3/13 ; rapport :
`/home/gcorbaz/devel/kesh-gate-logs/v0172s-validate-p6.md`. Sa remédiation est **textuelle** : aucun mécanisme ajouté ; des assertions et
des mutations retirées (« `postMessage` non appelé », M7 bis, M7 ter, le cas 4 de (c) pour M4) ; l'import de `tryRefresh()` rendu
statique ; des phrases corrigées (contournement `KESH_JWT_EXPIRY_MINUTES=480`, effacements multiples, succès d'`hydrate()`, course entre
onglets, inventaires, citations) ; le signal de non-convergence écrit au Change Log.

⚠️ **Septième passe** : il reste celle-ci et la P8 (`CLAUDE.md`, *Review Iteration Rule*). Ce qui serait mieux **retiré** que corrigé,
dis-le.

**Passe ciblée** (`CLAUDE.md` § *La passe ciblée*) : une seule lentille, braquée sur le diff de la remédiation. Périmètre :

```
git diff d556bc25 HEAD -- _bmad-output/implementation-artifacts/v0172-session.md _bmad-output/implementation-artifacts/v0172-choix-autonomes.md
```

Worktree **`/home/gcorbaz/devel/kesh-0172s`**, branche `story/v0172-session`. **Ne lis rien sous `/home/gcorbaz/devel/kesh-0172b`.** Langue
du rapport : **français**. Modèle : **Sonnet**. Contexte frais : relis au code chaque affirmation du diff ; le rapport P6 se lit pour
savoir ce qui était demandé.

## Axes à exercer

**La remédiation — chaque hunk du diff**

- **Le contournement 480** (§ 2, manuel `:779`, CHANGELOG) : le commentaire de #876 (`gh api repos/guycorbaz/kesh/issues/876/comments`) le
  dit-il ainsi ? Le calcul (960 minutes, 8 à 16 heures) est-il juste avec `effective_refresh_inactivity` ? Les deux libellés de
  l'avertissement (variable posée, variable absente) sont-ils décidables par un test d'AC-1 ?
- **Les retraits** (AC-5 bis, M7 bis, M7 ter, M4) : un comportement voulu (aucune diffusion `auth-change` depuis `hydrate()`, aucune
  redirection) reste-t-il sans aucun garde-fou ? La phrase qui explique le retrait est-elle vraie (`frontend/src/lib/app/stores/auth.svelte.ts:30-37`,
  `auth.svelte.test.ts:170-180`) ? M4 reste-t-elle tuée par T3 (b) ?
- **L'import statique** : le cycle cité existe-t-il (`auth.svelte.ts:17`, `frontend/src/lib/features/vat-rates/`, `api-client.ts:15`) ? Aucun
  des deux modules n'utilise-t-il l'autre au chargement (cherche un usage au niveau supérieur) ?
- **`tryRefresh()` et `hydrate()`** : la branche « refresh réussi, `/me` en échec » (`api-client.ts:158-172`) est-elle décrite juste ;
  `hydrate()` qui juge sur `currentUser !== null` couvre-t-il les trois issues ? Les N effacements déclarés sont-ils exacts ?
- **Les textes** : `auth.svelte.test.ts:147-163` (compte d'appels), `crates/kesh-api/README.md:40`, `docs/kesh-specifications.txt:44, :219, :1453`,
  #890 (`gh api repos/guycorbaz/kesh/issues/890`), la sonde élargie (`git grep -n refresh_tokens -- crates/kesh-db/src crates/kesh-api/src/routes/admin.rs`),
  la citation `auth.svelte.ts:209-316` / `:261-267`, la phrase de la course (C-v0172s-24), la note sur FR13, la puce *Modifié* du CHANGELOG.
- **Le signal de non-convergence** (Change Log) : la tendance et l'origine du HIGH de P5 sont-elles exactes (relis les entrées du Change Log
  et le registre C-v0172s-23) ?
- **Le registre** : C-v0172s-14, 24, 26 annotés sans contradiction avec la fiche ?
- **Propagation** : greppe dans la fiche et le registre `480`, `postMessage`, `M7 bis`, `M7 ter`, `cas 4 de (c)`, `dynamique`, `:229-265`,
  `issue demandée`, `deux gestes` : une occurrence vivante hors des entrées datées et des mentions historiques est un résidu.

**Ce qui n'est pas dans la passe** : le reste de la fiche, déjà validé ; ne le reprends que si un hunk du diff le contredit.

## Ce qu'il faut rendre

Un rapport en Markdown à `/home/gcorbaz/devel/kesh-gate-logs/v0172s-validate-p7.md`, **écrit par l'outil d'écriture de fichier (`Write`),
une seule fois**. Il contient :

1. **Les findings**, `P1`, `P2`…, avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le hunk visé, le lieu (`fichier:ligne`), **la ligne
   lue citée** (sortie `grep -nF` ou extrait copié) et le remède. Une affirmation de présence ou d'absence sans sortie `grep -nF` copiée ne
   compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans cette liste ne compte
   pas.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture hors du rapport**, et le rapport par l'outil `Write` seulement : **aucune redirection** (`>`, `>>`, `2>`, `&>`,
  `| tee`), **ni vers un fichier, ni vers `/dev/null`** ; **aucune option qui écrit un fichier** (`-o`, `--output`, `pdftotext` vers un
  fichier) ; **aucun fichier temporaire** — ni dans `/tmp`, ni dans le scratchpad, ni ailleurs. `pdftotext` s'emploie avec `-` (sortie
  standard) seulement. Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous aucune forme.
- Aucune commande `gh` qui écrit ; `gh api` en `GET` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `npx`, ni **`node`**, ni `make`, ni `latexmk`, ni `python3`, ni
  `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- **Jamais `scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun autre script du dépôt.
- Jamais `rm`, `pkill`, `killall`. Jamais `~/kesh/`, `~/kesh-test/`, ni le NAS, ni `/home/gcorbaz/devel/kesh-0172b` ; aucune donnée réelle
  dans le rapport.
- Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- Commandes permises : `cat`, `sed -n`, `/usr/bin/grep`, `git grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `awk` (lecture, sans
  redirection), `git log`, `git show`, `git diff` (en lecture), `pdftotext … -`, `gh api` en `GET`.
