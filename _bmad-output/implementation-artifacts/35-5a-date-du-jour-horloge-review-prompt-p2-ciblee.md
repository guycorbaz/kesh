# Prompt — revue de code P2 ciblée de la Story 35-5a (#1022, #907, #929 en partie), une lentille, braquée sur la remédiation de la P1

Passe **ciblée**, une seule lentille, contexte frais, **Sonnet** (P1 : Opus, trois lentilles), en lecture seule. Worktree
`/home/gcorbaz/devel/kesh-e355a`, branche `story/35-5a-date-du-jour-horloge`. La P1 a rendu 2 MEDIUM distincts et une douzaine de LOW
(rapports `/home/gcorbaz/devel/kesh-gate-logs/e355a-review-p1-{B,E,A}.md`). Déjà tracés, à ne pas rapporter de nouveau : #1125 (tests à
dates fixes de 2026), #1126 (test MCP rouge lancé seul), #1127 (rouges et mutations relevés sur un seul site des tests multi-sites, A4).
Ce qui reste au jour UTC (seed de démonstration, `closedOn` absent, balance âgée, borne d'un rapport de projet cumulé) relève de la
35-5b par décision : ne le rapporte pas comme défaut, vérifie seulement que les textes le disent vrai.

## Périmètre : le seul commit `527f57839`

`git -C /home/gcorbaz/devel/kesh-e355a show 527f57839` (10 fichiers). Compte rendu : `git show 7c2e1b686` (Change Log, annotations du
Dev Agent Record, registre C-e355a-*, PDF du manuel). Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e355a/` : `p1-*.log`,
`mut/M10.log`, `mut/M2-p1.log`, `mut/M1-p1.log`.

Il remédie :
- **E1** — l'écran du verrou (`settings/+page.svelte`) propose désormais `dayBefore(todayInZurich())` ; helpers neufs `todayInZurich`
  et `dayBefore` dans `frontend/src/lib/shared/utils/local-date.ts` ; Vitest New York, Athènes, nuit d'été ; M10 tuée.
- **B1 = A1** — le manuel et le CHANGELOG disent « le premier exercice que l'assistant crée pour votre société » et rangent la
  démonstration parmi ce qui reste au jour UTC ; seed non converti (option prudente, 35-5b).
- LOW : B2/A2/E5 (liste « notamment » de ce qui reste en UTC), B3/E6 (commentaires), E2 (premier minuit d'hiver à la seconde), E3
  (`ac4_verrouiller_jusqu_a_hier_une_nuit_d_ete`, M2 tuée côté serveur), E4 (mutation Europe/Paris équivalente, dite), A3, A5, A6, A7.

## Ce que tu vérifies

1. **`todayInZurich`** : `Intl.DateTimeFormat` avec `timeZone: 'Europe/Zurich'` — rend-il `AAAA-MM-JJ` sûrement (ordre des parties
   lu par `formatToParts` ou format `en-CA` ; pas de dépendance à la locale du poste, ni à `toLocaleDateString` dont la forme varie) ?
   Prend-il un instant injectable pour les tests ? Un helper équivalent existait-il déjà (`reports.api.ts:203` et ailleurs :
   `git grep -n "Europe/Zurich" -- frontend/src`) — doublon à signaler (règle DRY de CLAUDE.md) ? **`dayBefore`** : arithmétique sur la
   date civile, sans passage par un instant local (un `new Date(y, m-1, d-1)` sur un poste à changement d'heure, un `Date.UTC`) ;
   1er janvier, 1er mars d'une année bissextile et non bissextile, jour du changement d'heure ?
2. **L'écran** : `maxLockDate` est-il recalculé ou figé au montage (une page ouverte avant minuit et utilisée après) — et était-ce
   le cas avant ? Les autres lecteurs de « aujourd'hui » de la même page (aperçu, message, validation) suivent-ils le même jour, ou la page
   mêle-t-elle maintenant jour du poste et jour de Zurich ? La garde frontend des dates locales (`local-date-guard.test.ts`,
   allowlist 7 entrées) voit-elle le nouveau helper, et l'allowlist reste-t-elle juste (recompte) ?
3. **Les tests Vitest** : le fuseau du poste est-il réellement simulé (comment : `process.env.TZ`, `vi.setSystemTime`, injection) et
   le mécanisme agit-il sous jsdom/Node déjà démarré (un `TZ` posé après le premier `Date` est ignoré par V8 sur certaines versions) ? Les
   rouges de départ (`p1-rouges-vitest.log`, 2 sur 65) le prouvent-ils, avec les dates attendues ? M10 tuée par les tests nommés ?
4. **Côté serveur** : `ac4_verrouiller_jusqu_a_hier_une_nuit_d_ete` (instant 2026-07-01T22:30Z = 00 h 30 le 2 juillet à Zurich) —
   asserte-t-il ce qui rougit au jour UTC ? Le montage (exercice couvrant juillet) est-il cohérent ? E2 : la table AC1 de `horloge.rs` ;
   le doc-comment de `jour_suisse` (équivalence Europe/Paris) dit-il vrai ?
5. **Les textes** : `user-manual.tex` (les passages touchés, `:711-713` et la note *La date du jour est celle de Zurich*), le **PDF
   aplati** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` puis `/usr/bin/grep -o`), le CHANGELOG : chaque phrase
   vraie au code ? La liste de ce qui reste en UTC est-elle juste (chaque site nommé lit bien `Utc::now`, ligne lue citée) ? Reste-t-il
   ailleurs une promesse contraire (`git grep -n` sur « Zurich », « démonstration », « temps universel », « veille ») ?
6. **Attendus retouchés** : `git show 527f57839 -- '*.rs' '*.ts'`, lignes `-` dans les tests (notamment `settings-page.test.ts`,
   −~20 lignes) : un attendu existant a-t-il été affaibli ou retiré sans motif ?
7. **Le compte rendu et les gates** au commit `527f57839` : test-fast 5358/5358 (13 ignorés), Vitest 3217 (232 fichiers), check, lint,
   build — chacun au journal, postérieur au commit ; mutations M10, M2-p1, M1-p1 au journal ; l'écart déclaré (correction défaite par
   `git checkout --` en restaurant M10, puis refaite) — la version commitée est-elle bien celle sur laquelle M10 a été jouée ? `gate-ci` et
   E2E déclarés « à la clôture », non affirmés ; entrées antérieures du Change Log annotées sans réécriture (`git show 7c2e1b686`,
   lignes `-`).

## Interdits absolus

Lecture seule : aucun fichier écrit hormis ton rapport (outil Write). **Aucune redirection, sous aucune forme** — ni `>`, ni `>>`, ni
`tee`, ni `2>`, ni `2>&1`, ni `2>/dev/null`, ni `> /dev/null` —, **aucune substitution de processus** (`<(…)`), aucun fichier temporaire.
Trois relecteurs de cette epic ont écrit `2>/dev/null` « par réflexe » : relis chaque commande avant de la lancer. Aucun `rm`, `pkill`,
`kill`, aucun `git` qui écrit. Aucun interpréteur ni script (`python3`, `perl`, `node`, `npm`, `npx`, `bash -c`, `sh -c`, `jq`,
`scripts/*` dont **`scripts/prepare-release.sh`**, les scripts de `kesh-gate-logs`, `make`, `latexmk`, `cargo`, `docker`, `mariadb`,
`curl`, **`claude`**, `sed -i`, `vitest`, `playwright`), aucune commande `gh`, aucun accès réseau. Ne lis rien sous `~/kesh/` ni
`~/kesh-test/`. Permis : `git show/diff/log/grep/ls-files`, `/usr/bin/grep`, `sed -n`, `awk` sans redirection ni `system()`, `cat`, `ls`,
`find` sans `-exec`/`-delete`, `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`. ⚠️ `grep` est une fonction vers
`ugrep` : emploie `/usr/bin/grep` ou `git grep`. Toute affirmation de présence ou d'absence s'appuie sur une sortie copiée au rapport. Un
écart se déclare.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e355a-review-p2-ciblee.md` : findings numérotés (`C1`…), sévérité (CRITICAL/HIGH/MEDIUM/LOW), lieu,
ligne lue copiée, preuve, remède ; **axes exercés et non exercés** ; décompte final recompté depuis la liste.
