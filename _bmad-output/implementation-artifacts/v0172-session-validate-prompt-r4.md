# Prompt — validation R4 de la fiche v0172-session reprise (release 0.17.3 : #876, CR #891, CR #895), passe ciblée, Opus

Tu fais une **passe ciblée** sur la fiche reprise de la Story v0172-session de Kesh (comptabilité suisse, Rust/Axum, SvelteKit,
MariaDB 10.11), visée pour la release **0.17.3** : la durée d'inactivité de la session réglée dans l'interface d'administration, appliquée
exactement, remise à zéro par chaque activité de l'utilisateur. **Une seule lentille**, braquée sur **la seule remédiation de la
validation R3** : `git diff da16b94e HEAD` dans `/home/gcorbaz/devel/kesh-0172s` (fiche `v0172-session.md` et registre
`v0172-choix-autonomes.md`, section `v0172s`). Le reste de la fiche a été validé par R1, R2, R3 (complètes) ; tu ne le relis que là où la
remédiation le touche ou le contredit.

La validation R3 (Sonnet, 0/0/2/7, rapport `/home/gcorbaz/devel/kesh-gate-logs/v0172s-validate-r3.md`) a été remédiée ainsi :
- **R3-1** : au démarrage, `reportedActivity = min(now, max(reportedActivity stockée, serverLast))`, non plus l'activité stockée ;
  AC-6 (18), M27, C-v0172s-51.
- **R3-2** : la fiche ne promet plus « défilement » (FR13, manuel, CHANGELOG) mais « clavier, souris (mouvement, clic, molette), toucher,
  retour sur l'onglet » ; le glisser de la barre ne compte qu'à son début, écart déclaré ; `scroll` reste non écouté ; C-v0172s-49.
- **R3-3 / R3-9** : le layout racine s'abonne à `authState.currentUser?.userId` ; interface `sessionIdle` (`start`, `stop`, `setMinutes`,
  `minutes`) ; N tenu par le module ; AC-6 (19), AC-8, M28, C-v0172s-50.
- **LOW** : AC-6 (5 bis) sur la grille d'une seconde ; précision du serveur récrite ; sonde d'AC-2 et quatre commentaires ; prose de
  `migrations_upgrade_path.rs` ; registre C-33, C-47 ; dérogation de splitting déclarée au Project Lead le 2026-11-04.

Langue du rapport : **français**. Modèle : **Opus**. **Contexte frais** : fais tes propres relevés avant de lire le rapport R3.
**Ne lis rien sous `/home/gcorbaz/devel/kesh-0172a` ni `/home/gcorbaz/devel/kesh-0172b`.**

## À lire

1. `git diff da16b94e HEAD` (en lecture), puis, dans la fiche, les sections qu'il touche **en entier** (§ *Modules*, § *Conception* 5 et
   7, AC-2, AC-6, AC-8, *Tests et mutations*, *Tâches*, *PRD*, *Manuel*, *CHANGELOG*, *Dev Notes*, la dernière entrée du Change Log).
2. Le code que la remédiation cite : `frontend/src/lib/app/stores/auth.svelte.ts` (`login`, `_currentUser`, `hydrate`),
   `frontend/src/lib/shared/utils/api-client.ts` (`:120-180`), `frontend/src/routes/+layout.svelte`, `crates/kesh-api/src/routes/auth.rs`
   (`:20-70`), `crates/kesh-api/tests/auth_cookies_e2e.rs` (`:240-280`), `crates/kesh-db/tests/migrations_upgrade_path.rs` (en entier).
3. Le **manuel** que la remédiation fait changer : `docs/manual/fr/user-manual.tex` (`:85-125`) ; **le PDF aussi**, aplati vers la sortie
   standard : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`.

## Axes à exercer

- **Chaque remède est-il juste et complet** ? Pour chacun des neuf findings, le traitement du Change Log (« Validation R3 (Sonnet) ») est-il
  réellement dans la fiche et le registre ?
- **R3-1** : la formule neuve garde-t-elle les propriétés acquises (jamais d'activité future ; `null` sans calcul ; connexion
  `serverLast = now`) ? Le cas AC-6 (18) est-il décidable tel qu'écrit, et M27 le fait-il rougir ?
- **R3-2** : le texte de FR13 (`prd.md:391`, `epics.md:63`, et ses copies dans `docs/kesh-specifications.txt`), le manuel prescrit et le
  CHANGELOG disent-ils la même chose, sans plus promettre « défilement » ? Reste-t-il une promesse de défilement ailleurs dans la fiche
  (exigences, § 5, recette) ? L'écart est-il écrit là où l'utilisateur le lira ?
- **R3-3 / R3-9** : l'abonnement par `userId` redémarre-t-il bien le module au changement d'utilisateur et l'arrête-t-il à
  `clearSession()` ? Un `$derived` de chaîne se propage-t-il quand la valeur ne change pas (Svelte 5, comparaison d'égalité) ?
  L'interface `sessionIdle` est-elle cohérente avec tous les appels que la fiche prescrit (fermeture, battement, écran, `?minutes=`) ?
- **Les LOW** : la grille d'une seconde de AC-6 (5 bis) tient-elle avec `vi.useFakeTimers()` et une minuterie démarrée à t = 0 ? Les
  numéros de ligne prescrits de `migrations_upgrade_path.rs` et de `auth_cookies_e2e.rs` sont-ils justes ?
- **Régressions de la remédiation** : un remède contredit-il une autre section (mutations, Tâches, Dev Notes, registre) ?

## Ce qu'il faut rendre

Un rapport en Markdown à `/home/gcorbaz/devel/kesh-gate-logs/v0172s-validate-r4.md`, **écrit par l'outil d'écriture de fichier
(`Write`), une seule fois** :

1. **Les findings**, `R4-1`, `R4-2`…, avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), origine (**remédiation R3** ou **défaut
   d'origine**), lieu (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède. Une affirmation de
   présence ou d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans cette liste ne
   compte pas.
3. Un décompte final par sévérité et par origine, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture hors du rapport**, et le rapport par l'outil `Write` seulement : **aucune redirection** (`>`, `>>`, `2>`, `&>`,
  `| tee`), **ni vers un fichier, ni vers `/dev/null`** ; **aucune option qui écrit un fichier** (`-o`, `--output`, `pdftotext` vers un
  fichier) ; **aucun fichier temporaire** — ni dans `/tmp`, ni dans le scratchpad, ni ailleurs. `pdftotext` s'emploie avec `-` (sortie
  standard) seulement. Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous aucune forme.
- Aucune commande `gh` qui écrit ; `gh api` en `GET` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `npx`, ni `node`, ni `make`, ni `latexmk`, ni `python3`, ni
  `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- **Jamais `scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun autre script du dépôt.
- Jamais `rm`, `pkill`, `killall`. Jamais `~/kesh/`, `~/kesh-test/`, ni le NAS, ni `/home/gcorbaz/devel/kesh-0172a`,
  `/home/gcorbaz/devel/kesh-0172b` ; aucune donnée réelle dans le rapport.
- Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- Commandes permises : `cat`, `sed -n`, `/usr/bin/grep`, `git grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `awk` (lecture, sans
  redirection), `git log`, `git show`, `git diff` (en lecture), `pdftotext … -`, `gh api` en `GET`.
