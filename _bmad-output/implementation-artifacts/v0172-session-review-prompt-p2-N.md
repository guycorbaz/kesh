# Prompt — revue de code P2 de la Story v0172-session, **lot N (navigateur)**, remédiation de la P1, trois lentilles, Opus

Tu relis la **remédiation de la revue de code P1** du **lot N** (navigateur) de la **Story v0172-session** de Kesh
(comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11), version **0.17.3** : la durée d'inactivité de la session,
réglée dans l'interface, remise à zéro par l'activité dans tous les onglets (#876, CR #891). Modèle : **Opus**. Passe
**complète** sur un **périmètre restreint** : le diff de la remédiation et ses interactions avec le code qu'il touche.
Contexte frais. Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-0172s`, branche `story/v0172-session`. Le diff à relire, aplati :
`git -C /home/gcorbaz/devel/kesh-0172s diff ff20983d d38fcd8b` (`ff20983d` = remédiation du lot S, elle-même posée sur
`11e5a38b`, tête relue par la P1 ; `d38fcd8b` = dernier commit de la remédiation du lot N, manuel et PDF compris ; les
commits du lot N sont `ac666759`, `8d6561bc`, `d38fcd8b`). Le commit
de comptes rendus qui suit est hors de ta lentille, sauf pour relire la fiche, le registre et le Dev Agent Record.

**Ce que la remédiation prétend faire** (à vérifier, non à croire) :
1. **N-A1/N-E1** : `frontend/src/lib/app/stores/session-idle.svelte.test.ts` — un cas par écouteur (onze) tiré **après
   `stop()`** au-delà de l'échéance, chacun avec un témoin sans `stop()` qui ferme ; la minuterie comptée par
   `vi.getTimerCount()` (compte relatif : jsdom diffuse chaque écriture de `localStorage` par une minuterie à 0 ms).
   Mutations `MN1` (`input` non retiré), `MN1bis` (`blur` non retiré), `MN2` (`clearInterval` retiré).
2. **N-E2** : `localStorage` qui refuse **en cours de session** (écriture, lecture) : repli en mémoire, aucun utilisateur actif
   fermé, battement envoyé et régulé à 15 s. Mutations `MN3` (`shared = false` retiré de `writeKey`), `MN4` (`readKey` qui
   rend « absente » sans basculer : survivante au premier jeu, tuée après renforcement du test) et `MN4bis` (`shared = false`
   retiré à la lettre : récursion infinie).
3. **N-B1/N-E4** : `readKey` — une clé vide ou blanche vaut absente ; `concile` — une borne non finie (`idleExpiresIn`
   démesuré) vaut absente. Mutations `MN5`, `MN6`.
4. **N-B4/N-A9/N-E6** : `frontend/src/routes/login/+page.svelte` — `?minutes=` borné à 1..1440 (1 à 4 chiffres), singulier par
   la clé `login-session-idle-one` (quatre locales) et non par un sélecteur Fluent (choix **C-v0172s-60** : `i18nMsg` ne les
   évalue pas) ; `sitesTotal` 2825 → 2826. Mutations `MN7`, `MN8`.
5. **N-A2** : AC-10 (e), l'aperçu PDF, **joué en navigateur** (Chromium, nouveau headless) — `kesh-gate-logs/v0172s/
   recette-e-r*.log`, script `scripts/recette-e-v0172s.mjs` (à **lire**), captures `v0172s-07` et `-08` ; le Dev Agent Record
   ne dit plus « couvert par les tests ». Constat de la recette (r2) : deux sorties au même pixel, la seconde non comptée
   (filtre des mouvements fictifs) — tracé pour une issue (Rec-e1), non corrigé : juge si c'est assez.
6. **N-B2/N-E8** : `docs/manual/fr/user-manual.tex:97` — quitter la fenêtre (Alt-Tab) pendant que le document a la main « peut
   aussi compter, une fois » (formulation prudente : non observé en navigateur, cf. Change Log).
7. **N-A7** : registre, C-v0172s-58 — `doRefresh()` au lieu de `refreshCore()`.

Lis d'abord : la fiche `_bmad-output/implementation-artifacts/v0172-session.md` (*Conception retenue* § 5 à 7, AC-6 à AC-10,
le **Change Log**, entrée « Revue de code P1 », et le **Dev Agent Record**), le registre `v0172-choix-autonomes.md`
(C-v0172s-45, 50 à 52, 58 à 61), les six rapports de la P1 (`/home/gcorbaz/devel/kesh-gate-logs/v0172s-review-p1-{S,N}-
{B,E,A}.md`), `frontend/src/lib/shared/utils/i18n.svelte.ts` (`i18nMsg`). Journaux hors dépôt, en lecture seule, sous
`/home/gcorbaz/devel/kesh-gate-logs/v0172s/` : `rem-p1-cible-N*.log`, `mut-rem-p1/*.log` (le script
`scripts/mut-v0172s-rem-p1.py` est à **lire**, jamais à exécuter), `recette-e-r*.log`, `recette-e-backend-r*.log`,
`rem-p1-front-*.log`, `e2e-r4.log` et `rem-p1-e2e-run-r4.log` (le r2 a été interrompu par un OOM de la station le 2026-11-04 à 21:48 ; le r3, arrêté au démarrage sur une base non remise à neuf, ne compte pas ; le r4 a été rejoué le 2026-11-05 sur `d38fcd8b`, base `kesh_e2e_v0172s` recréée vide) (E2E complet de la remédiation), `rem-p1-make-fr.log`. Captures (thème clair) sous `captures/`.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`, `tee`,
  `2>`, `&>`, `2>&1`) : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au rapport.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**.
- **Aucun `git checkout`/`switch`/`restore`**, aucun `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`,
  `worktree`, `add`, `fetch`, `merge`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, ni `perl`, ni `node`/`npm`/`npx`, ni
  `bash -c`/`sh -c`, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`,
  ni **aucun script** du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, **`cargo`**, **`docker`**,
  `mysql`/`mariadb`, `sqlx`, **`curl`**, **`claude`**, **`sed -i`**, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep` (dans ce shell, `grep` est une
  fonction vers `ugrep`), `sed -n`, `awk` (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`),
  `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⛔ Sur le PDF aplati, `grep -oF … | wc -l` ou `grep -cF` sur des **motifs fixes**, sous `timeout 60` ; l'apostrophe s'y rend
  `’` : compte les deux graphies.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **Le manuel (`.tex` et PDF aplati) contre le code et contre la recette** : `user-manual.tex:95-97` (ce qui compte, l'aperçu
   PDF, le geste prescrit, la phrase neuve sur Alt-Tab) et `user-manual.pdf` aplati ; `admin-manual.tex` § *Durée
   d'inactivité de la session* (« Si le navigateur refuse `localStorage` … c'est le serveur qui ferme ») contre le repli
   en cours de session. La recette (e) établit-elle ce que le manuel prescrit (molette dans le document : fermeture à
   l'heure ; sortie du pointeur toutes les 90 s : session ouverte) ? La phrase sur Alt-Tab est-elle vraie au code
   (`onBlur` : `document.activeElement instanceof HTMLIFrameElement`) et ne promet-elle rien que le navigateur ne fasse ?
2. **Les témoins et les mutations** : chacun des onze cas « après `stop()` » peut-il passer **à vide** (le geste ne
   déclencherait rien même sans `stop()`) — son témoin l'exclut-il ? Le compte relatif de `vi.getTimerCount()` tue-t-il
   vraiment `clearInterval` retiré ? Les deux cas de `localStorage` refusé tuent-ils `MN3`, `MN4`, `MN4bis` **par le test
   attendu** (`mut-rem-p1/*.log`) ? Une mutation plausible survivrait-elle (`raw.trim()` remplacé par `raw.length`, borne
   `<= 1440` en `< 1440`, regex `{1,4}` élargie, `-one` choisi pour `minutes === 0`) ?
3. **Les gardes et compteurs, recomptés à la source** : `sitesTotal` 2825 → 2826 (`git show <borne>:<f> | grep -o
   'i18nMsg(' | wc -l` aux deux bornes) ; la parité des quatre catalogues (une clé de plus chacun) ; le repli de
   `login-session-idle-one` égal à la valeur fr-CH ; les compteurs de la fiche (Dev Agent Record : 40 mutations = 19 + 21 ;
   tests `it(` ajoutés par la remédiation, périmètre déclaré).
4. **Grep du symptôme, la valeur, sur tout le worktree** : `Number(raw)` et `Number(` sur une valeur de `localStorage`
   ailleurs dans `frontend/src` (même défaut de la chaîne vide ?) ; `minutes=` dans les liens de connexion ; « 1 minutes » et
   « 0 minutes » ; « couvert par les tests » au Dev Agent Record ; `refreshCore`.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne le diff : `readKey` (le `trim()`, l'ordre des gardes), `concile` (le `NaN` sentinelle, la branche
« jeton sans échéance » désormais prise aussi pour une borne non finie : `rep` y est-il juste ?), la page de connexion (la
regex, `Number`, les bornes, le choix de clé), les tests neufs (fuite d'état entre cas : `Object.defineProperty(document,
'visibilityState')`, espions de `Storage.prototype`, `<iframe>` laissés dans le `body`).

## Lentille E — Edge (chaque branche, chaque borne)

`lastActivity` à `'0'`, `' 5 '`, `'1e3'`, `'0x10'` ; `idleExpiresIn` à `-1e308`, `1e308`, `-0`, `NaN` (non fini, refusé en
amont ?) ; `?minutes=0001`, `?minutes=1440`, `?minutes=01`, `?minutes=` absent, `?reason=idle` seul ; `localStorage` qui
refuse puis accepte de nouveau (le module revient-il au partage ? doit-il ?) ; `stop()` appelé pendant un battement en vol
(`heartbeatInFlight` remis à `false` ?).

## Lentille A — Acceptance (le mandat de la remédiation contre le livré)

Chaque point 1 à 7 ci-dessus : livré, testé, au lieu prescrit, ligne lue citée au Change Log ? L'écart au mandat (clé `-one`
au lieu d'un sélecteur Fluent, C-v0172s-60) est-il prouvé au code ? La recette (e) : journal horodaté, résultats exacts
(écart à 120 s), captures présentes ; ce qui n'a pas été éprouvé (Alt-Tab) est-il dit « non éprouvé » au Dev Agent Record ?
Les LOW non corrigés sont-ils tous dans `/home/gcorbaz/devel/kesh-gate-logs/v0172s/issue-low-p1.md`, sans donnée réelle ?
Le Dev Agent Record n'affirme-t-il que ce qui a tourné (Vitest complet, E2E complet, échecs nommés contre
`docs/testing.md`) ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/v0172s-review-p2-N-B.md`, `…-N-E.md`, `…-N-A.md`.
Chaque rapport : findings numérotés (`N2-B1`, `N2-E1`, `N2-A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu
(`fichier:ligne`), ligne lue copiée, preuve, remède proposé ; **la liste des axes exercés ET non exercés** (une passe qui ne
la donne pas ne compte pas) ; le décompte final **recompté depuis la liste**.
