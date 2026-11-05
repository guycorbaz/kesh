# Prompt — validation R5 de la fiche v0172-session reprise (release 0.17.3 : #876, CR #891, CR #895), passe ciblée, Sonnet

Tu fais une **passe ciblée** sur la fiche reprise de la Story v0172-session de Kesh (comptabilité suisse, Rust/Axum, SvelteKit,
MariaDB 10.11), visée pour la release **0.17.3** : la durée d'inactivité de la session réglée dans l'interface d'administration, appliquée
exactement, remise à zéro par chaque activité de l'utilisateur. **Une seule lentille**, braquée sur **la seule remédiation de la
validation R4** : `git diff ec83dcdd HEAD` dans `/home/gcorbaz/devel/kesh-0172s` (fiche `v0172-session.md` et registre
`v0172-choix-autonomes.md`, section `v0172s`). Le reste de la fiche a été validé par R1, R2, R3 (complètes) et R4 (ciblée) ; tu ne le relis que là où la
remédiation le touche ou le contredit.

La validation R4 (Opus, ciblée, 0/0/2/6, rapport `/home/gcorbaz/devel/kesh-gate-logs/v0172s-validate-r4.md`) a été remédiée ainsi :
- **R4-1** : AC-6 (18) récrit sur la grille d'une seconde, valeurs exactes (`idleMs = 2 000`, `heartbeatAt = t + 2 000`).
- **R4-2** : branche « `idleExpiresIn` nul ou absent » de la conciliation ; clés non finies lues comme absentes, aucune écriture sans
  `Number.isFinite` ; AC-6 (12) complété ; mutation M29 ; C-v0172s-52.
- **LOW** : AC-6 (5 bis) avec une activité à 1,5 s (500, puis 7 000 au tic de 17 s) ; N tenu par le module à `§ 2` et `§ 7` ;
  `untrack` et nettoyage `() => sessionIdle.stop()`, `stop()` idempotente ; corps de C-35 ; ligne de généalogie après
  `migrations_upgrade_path.rs:210` ; R4-7 sans objet pour la fiche (l'orchestrateur commente #891).

Langue du rapport : **français**. Modèle : **Sonnet**. **Contexte frais** : fais tes propres relevés avant de lire le rapport R4.
**Ne lis rien sous `/home/gcorbaz/devel/kesh-0172a` ni `/home/gcorbaz/devel/kesh-0172b`.**

## À lire

1. `git diff ec83dcdd HEAD` (en lecture), puis, dans la fiche, les sections qu'il touche **en entier** (§ *Conception* 2, 5 et
   7, AC-6, *Tests et mutations*, *Tâches*, la dernière entrée du Change Log), et au registre C-v0172s-35, 50 et 52.
2. Le code que la remédiation cite : `frontend/src/lib/app/stores/auth.svelte.ts` (`login`, `_currentUser`, `:41`, `:113`),
   `frontend/src/lib/shared/utils/api-client.ts` (`:120-180`), `frontend/src/routes/+layout.svelte`,
   `crates/kesh-db/tests/migrations_upgrade_path.rs` (`:180-230`). Aucun manuel n'est touché par cette remédiation.

## Axes à exercer

- **Chaque remède est-il juste et complet** ? Pour chacun des huit findings, le traitement du Change Log (« Validation R4 (Opus,
  ciblée) ») est-il réellement dans la fiche et le registre ?
- **R4-1 / R4-3** : rejoue à la main, tic par tic, AC-6 (5 bis), (18) et (12) contre les règles du § 5 (régulation d'écriture à une
  seconde, régulation du battement à 15 s, conciliation, minuterie démarrée au démarrage du module). Les valeurs annoncées sont-elles
  exactement celles qu'une implémentation conforme produirait ? Les mutations M12 bis, M27 et M29 rougissent-elles ?
- **R4-2** : la règle « clé non finie = absente, aucune écriture sans `Number.isFinite` » couvre-t-elle les trois clés et tous les
  chemins d'écriture (événement, tic, conciliation, réponse et échec du battement) ? Reste-t-il un calcul sur `idleExpiresIn` nul ?
- **R4-4** : greppe toi-même la valeur — qui tient N — dans toute la fiche et le registre ; une phrase donne-t-elle encore N à `authState` ?
- **R4-5** : l'effet prescrit (dépendance au seul `sessionUserId`, `untrack`, nettoyage, `stop()` idempotente) est-il cohérent avec la
  fermeture du § 7 et avec AC-6 (19) ?
- **Régressions de la remédiation** : un remède contredit-il une autre section (mutations, Tâches, Dev Notes, registre) ?

## Ce qu'il faut rendre

Un rapport en Markdown à `/home/gcorbaz/devel/kesh-gate-logs/v0172s-validate-r5.md`, **écrit par l'outil d'écriture de fichier
(`Write`), une seule fois** :

1. **Les findings**, `R5-1`, `R5-2`…, avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), origine (**remédiation R4** ou **défaut
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
