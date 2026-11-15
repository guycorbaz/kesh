# Prompt — revue de code P2 de la Story 36-5 (#931, KF-056), deux lentilles, passe complète

Tu relis la **Story 36-5** de Kesh (comptabilité suisse, SvelteKit 5 runes), release **0.17.8**, Epic 36 : le compte rendu d'une
restauration ne se perd plus sous un battement de session refusé. Correctif **entièrement frontend** : un maintien de session porté
par `authState` (`holdForRestore`, `shortenRestoreHold`, `releaseRestoreHold`, `restoreHold`), lu par le chemin 401 du client
(`refreshTokens`) et par le module de session, posé, réarmé, raccourci et levé par le seul panneau de restauration
(`AdminRestorePanel.svelte`). Aucune migration, aucun changement serveur, aucune clé i18n.

**Pourquoi une passe complète, et non ciblée** : la remédiation de la P1 **change la règle de la borne** (après un échec certain ou
une issue inconnue, l'échéance est désormais ramenée à au plus `maintenant + B`, au lieu de garder celle de la confirmation) et
touche trois fichiers de production. Relis donc la story entière, avec un œil particulier sur ce qui a changé.

Worktree `/home/gcorbaz/devel/kesh-36-5`, branche `story/36-5-restauration-compte-rendu`. Diffs, **aplatis** :
- la story entière : `git -C /home/gcorbaz/devel/kesh-36-5 diff f43562b4d HEAD` ;
- la remédiation P1 seule : `git diff 1fbda93a7 HEAD` — `f142298e7` (code et tests : E1-1, E1-2, E1-3/B1-3, E1-4, B1-2, A1-3 ;
  **dernier commit qui touche du code**), `85359e0f2` (fiche : formulation de la borne, AC3, mutations M24 à M26 ; manuel :
  incise), puis le compte rendu ;
- la production seule : `git diff f43562b4d HEAD -- frontend/src ':!*.test.ts'`.

Langue : français. Modèle : **Sonnet ou Opus** (passe complète : jamais Haiku). Contexte frais.

Lis d'abord la fiche `_bmad-output/implementation-artifacts/36-5-restauration-compte-rendu.md` en entier (D1 à D6, AC1 à AC9, la
table des mutations M1 à M26, Dev Agent Record et sa section *Remédiation de la revue P1*, Change Log), le registre
`e36-choix-autonomes-36-5.md` (C-36-5-1 à 13), et les rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/e36-5/review-p1-{B,E,A}.md`.
Recoupe le Dev Agent Record avec les journaux (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e36-5/` :
`r1-vitest-cible.log`, `r1-test-fast-ci.log`, `r1-front-*.log` (commit en première ligne), `repetition-apres-d5-r3.log`,
`mut/resume.log` (campagne 2), `mut/M24.log`, `mut/M25.log`, `mut/M26.log` et les autres `mut/M*.log`, `mut/mutations.py` et
`e2e-e36-5.sh` (à **lire**), `rouges.log`, `repetition-*.log`.

⚠️ **Non lancés, et c'est la consigne** : `gate-ci` et l'E2E complet (à la clôture). Le dire n'est pas un finding.
**Manuel** : `docs/manual/fr/admin-manual.tex:2340` (la demi-phrase et son incise « soixante minutes si l'écran ne la connaît pas
encore »). Le PDF **n'est pas** régénéré (à l'intégration, D6) : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`
doit rendre `1` pour « Prenez le temps de lire, puis cliquez » et `0` pour « Laissé ouvert, ou quitté ».
**Capture** : `/home/gcorbaz/devel/kesh-gate-logs/e36-5/captures/j-compte-rendu-apres-battement-refuse.png`.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se déclare.
- **Aucun `rm`** ; ni `pkill`, ni `killall`, ni `kill`. **Aucun `git` qui écrit** (`checkout`, `switch`, `restore`, `stash`…).
- **Aucun interpréteur, rien qui compile ou exécute** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq`,
  `unzip` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, ni aucun
  script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**,
  `sed -i`, `playwright`, `vitest`. **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et le contrôle du PDF
  ci-dessus. `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` ou `sed -n` copiée ; **tout chemin cité** se
  vérifie (`ls` ou `git cat-file -e`), ligne lue citée.

## Les deux lentilles

### E — Edge Case Hunter : les cas limites de la session

Chaque branche et chaque borne, contre le code, avec ce qui a changé en P1 au premier plan :
1. **La borne après un échec** (`shortenRestoreHold`, `auth.svelte.ts` ; le `finally` de `confirmImport`) : `min`, jamais
   d'allongement, rien sans maintien ; échec rapide, échec après un import plus long que B, issue inconnue (`TIMEOUT` après une
   heure, puis rejeu), échec **après démontage** (le `finally` raccourcit-il un maintien déjà levé, ou en pose-t-il un ?) ; une
   exception levée dans le `catch` ; la borne finale `confirmation + max(importHoldMs(B), durée de l'import + B)` tient-elle
   dans chaque branche ?
2. **L'écouteur de geste** : retiré en tête de `confirmImport` — avant ou après la garde de ré-entrance, et que se passe-t-il
   si la garde renvoie (double clic, `imported` posé) ? Écouteur retiré pendant l'issue inconnue (import non reproposé) ? Un
   geste dans le `Dialog` de confirmation lui-même ? `capture: true` et l'ordre avec les écouteurs de `session-idle`.
3. **La lecture du compte rendu** : minuterie armée avant le maintien (B1-3) — un délai du navigateur (onglet en arrière-plan)
   qui fait passer l'échéance du maintien avant la minuterie : que se passe-t-il, et le commentaire du code le dit-il juste ?
   B relu au succès (`readingMs()` rappelé) : une durée apprise entre la confirmation et le succès change-t-elle quelque chose
   de faux (phase 1 calculée sur l'ancienne, phase 2 sur la nouvelle) ?
4. **Le chemin 401 et le module de session** (D1, D2) sous maintien raccourci : à `échec + B`, un battement 401 renvoie-t-il à
   la connexion, et la fermeture d'inactivité a-t-elle lieu au tic qui suit, si `last + w` est passé ?
5. **Deux onglets, `hydrate()`** (commentaire ajouté, B1-2) : dit-il vrai ?
6. **Les tests** de la remédiation (deux tests au panneau, deux au store) : prouvent-ils ce qu'ils disent, ou passent-ils à vide
   (horloge simulée, `toFake`, ordre des appels) ? M24, M25, M26 sont-elles tuées par eux, et les 33 autres toujours ?

### A — Acceptance Auditor : la fiche contre le livré

1. La **formulation unique de la borne** est-elle identique aux trois endroits (D2, D3, *Ce qui doit rester vrai*), et juste
   au code dans chaque branche ? Ce qui dépend d'elle ailleurs dans la fiche (D3 « issue inconnue et échec certain »,
   *Écartées*, AC3, AC5 bis, *Hors périmètre*, Dev Notes) a-t-il suivi, ou reste-t-il une phrase qui dit encore « l'échéance de
   la confirmation est gardée » ? Greppe la valeur, pas la formulation.
2. Chaque **AC** (AC1 à AC9) et chaque **mutation** M1 à M26 contre ce qui est livré et exécuté (`mut/resume.log`, campagne 2 :
   36 sur 36 ?), y compris les annotations M8 (A1-4) et M18 (A1-1).
3. Le **Dev Agent Record** : chaque chiffre (tests ajoutés de `f43562b4d` à `HEAD`, tests de la remédiation, mutations, gates)
   se recompte-t-il depuis la source, avec son périmètre, et chaque gate cite-t-il un journal dont la première ligne porte le
   commit ? Le sort de chaque finding de la P1 (E1-1 à E1-4, B1-1 à B1-3, A1-1 à A1-5) y est-il écrit et vrai au code ?
4. **Le manuel** : la demi-phrase et son incise disent-elles ce que fait le code (B relu, 60 min si inconnue ; « aussitôt »
   quand on quitte) ? Le paragraphe de l'issue inconnue (`:2342`) reste-t-il juste avec la borne ramenée à B ?
5. **L'E2E de D5** (`backup-restore-new-install.spec.ts`, test « KF-056 ») : le message de rouge nomme-t-il désormais la page
   (A1-3) ? Les journaux « avant » annotés (A1-2) et le script `e2e-e36-5.sh` disent-ils le commit servi ?
6. `git log origin/main..HEAD --format=%B | /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` est-il vide ?

## Rapport

Un rapport par lentille, et seulement là :
- E : `/home/gcorbaz/devel/kesh-gate-logs/e36-5/review-p2-E.md`
- A : `/home/gcorbaz/devel/kesh-gate-logs/e36-5/review-p2-A.md`

Chacun porte : l'en-tête (lentille, modèle, commit relu, date) ; les findings numérotés (`E2-1`, `A2-1`…), chacun avec sa
**sévérité** (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), son lieu (`fichier:ligne`), la ligne lue citée, la preuve copiée et le
remède ; la liste des **axes exercés** et celle des **axes non exercés** (un « 0 finding » sans elle ne compte pas) ; le décompte
par sévérité, recompté depuis la liste.
