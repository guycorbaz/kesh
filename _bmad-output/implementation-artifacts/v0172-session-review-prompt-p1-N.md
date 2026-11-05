# Prompt — revue de code P1 de la Story v0172-session, **lot N (navigateur)**, trois lentilles, Sonnet

Tu relis le **lot N** (navigateur et textes) de la **Story v0172-session** de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit 5 runes, MariaDB 10.11), version **0.17.3** : **la durée d'inactivité de la session, réglée dans l'interface,
exacte, remise à zéro par l'activité** (#876, CR #891). Le lot N porte : le module `frontend/src/lib/app/stores/
session-idle.svelte.ts` (écouteurs d'activité, trois clés `localStorage` partagées par les onglets — `lastActivity`,
`reportedActivity`, `heartbeatAt` —, minuterie d'une seconde, battement au plus toutes les 15 s, conciliation au démarrage
bornée à maintenant, aucune valeur non finie lue ni écrite, fermeture qui attend la déconnexion au serveur jusqu'à 2 s puis
`/login?reason=idle&minutes=N`, repli en mémoire **sans fermeture locale**) ; l'abonnement du **layout racine** à
l'identifiant de l'utilisateur ; `tryRefresh()` dans `api-client.ts` (sous le mutex du chemin 401, sans effacement ni
redirection) et `hydrate()` (un renouvellement sur 401 de `/me`, **aucune requête** sur `/login?reason=idle`) ; les champs
facultatifs `idleTimeoutMinutes`/`idleExpiresIn` du `LoginPayload` et ses trois constructeurs ; la section **« Session »**
d'Administration → Paramètres (Administrateur seul, verrou optimiste) ; le message de la page de connexion ; les clés i18n
(quatre locales) ; les manuels `.tex` et PDF ; le CHANGELOG `[0.17.3] — Non publié` ; FR13 et ses copies (CR #891).

Worktree `/home/gcorbaz/devel/kesh-0172s`, branche `story/v0172-session`. Le diff à relire, **aplati** : `git -C
/home/gcorbaz/devel/kesh-0172s diff 1dc94968 HEAD` (`1dc94968` = commit du lot S ; le lot N et les comptes rendus
suivent). Langue du rapport : français. Modèle : **Sonnet**. Passe **complète** (première passe). Contexte frais.

⚠️ **Les propriétés centrales.**
1. **L'échéance est partagée et exacte** : `deadline = lastActivity + N`, contrôlée **avant** toute écriture d'activité ;
   une activité au-delà ferme, elle ne ranime rien.
2. **Ce qui compte, et ce qui ne compte pas** : `pointermove` seulement si les coordonnées d'écran changent ; `focus` de la
   fenêtre elle-même ; `blur` vers un `<iframe>` ; `scroll` **non** écouté ; les requêtes de fond ne comptent pas.
3. **Aucun `NaN`**, aucune activité future, aucune activité fantôme au démarrage (`reportedActivity` concilié avec ce que
   le serveur a reçu, `serverLast`, jamais avec l'activité stockée).
4. **La fermeture déconnecte, puis redirige** ; sans `localStorage`, le module bat mais ne ferme rien.
5. **Le layout ne redémarre pas le module à chaque renouvellement** (abonnement à l'identifiant, arguments sous `untrack`).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche `_bmad-output/implementation-artifacts/
v0172-session.md` — *Conception retenue* § 5, 6, 7, 10, 11, 13, **AC-6 à AC-11**, *Tests et mutations* (M9 à M14, M18 à
M30), *Tâches* (lot N), *Manuel*, *Captures*, *CHANGELOG*, *PRD*, le **Dev Agent Record** et le **Change Log** ; puis le
registre `v0172-choix-autonomes.md`, section `v0172s`, en vigueur **C-v0172s-14, 17, 23, 24, 26, 28 à 58** (C-57 :
`Promise.race` de la déconnexion ; C-58 : le chemin 401 efface et redirige lui-même). Ne prends pas le Dev Agent Record
pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/
v0172s/` : `rouges.log` (et `rouges-N.log`), `cible-N-*.log`, `mut/N-resume.log` et `mut/M*.log` (le script
`scripts/mut-v0172s-N.py` est à **lire**, jamais à exécuter), `front-*.log`, `e2e-r*.log`, `make-fr-*.log`,
`recette-r*.log` (la recette de durée, horodatée ; `scripts/recette-v0172s.mjs` à **lire**). Captures (thème clair) sous
`/home/gcorbaz/devel/kesh-gate-logs/v0172s/captures/` : `v0172s-01-parametres-session-60`, `-02-saisie-hors-bornes-
refusee`, `-03-enregistrement-reussi-2`, `-04-parametres-comptable-sans-session`, `-05-connexion-session-fermee-2-min`,
`-06-journal-audit-reglage` (regarde-les si ton outil lit les images).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle
  part ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**.
- **Aucun `git checkout`/`switch`/`restore`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`,
  `bash -c`/`sh -c`, `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun
  script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `curl`, **`claude`**,
  `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⛔ Sur le PDF aplati, `grep -oF … | wc -l` ou `grep -cF` sur des motifs fixes, sous `timeout 60` ; l'apostrophe s'y rend
  `’` : compte les deux graphies.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **Les manuels (`.tex` et PDF aplati) contre le code** : `docs/manual/fr/user-manual.tex` (§ de la session : ce qui
   compte, ce qui ne compte pas, l'aperçu PDF et le geste à faire, « le message s'affiche en français », « recharger … ne
   déconnecte pas », la récupération par courriel, la déconnexion) ; `admin-manual.tex` § *Durée d'inactivité de la
   session* (`\label{sec:duree-inactivite}` : les quatre écarts, « à la seconde près », la déconnexion attendue deux
   secondes, `localStorage` indisponible, « la mise à jour ne déconnecte personne »), § *Authentification JWT + refresh
   tokens*, le dépannage des 401, la restauration complète ; et leurs **PDF** régénérés (`make fr`). Chaque phrase est-elle
   vraie au code de `session-idle.svelte.ts`, `+layout.svelte`, `login/+page.svelte`, `SessionIdleSection.svelte` ?
2. **Les témoins et les mutations** : AC-6 (1) à (19), AC-7 (1) à (4), AC-8, AC-9 — quel test couvre chaque cas, peut-il
   passer **à vide** (minuteries simulées mal avancées, `localStorage` partagé entre cas, un `<iframe>` laissé focalisé par
   un cas précédent, un `fetch` simulé qui résout trop tôt) ; chaque mutation nommée est-elle tuée **par le test attendu**
   (`mut/M*.log`) ? Mutations plausibles non jouées : la régulation d'écriture à une seconde retirée ; `stop()` qui ne retire
   pas un écouteur (`blur`, `input` sur `document`) ; la minuterie non arrêtée par `stop()` ; `generation` ignoré dans
   `learnFromMe` ; `heartbeatInFlight` retiré ; `minutes` lu du serveur sans `isValidMinutes`.
3. **Les gardes et compteurs** : `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal` 2813 → 2825, recompté à la
   source aux deux bornes : `git show 1dc94968:<f> | grep -o 'i18nMsg(' | wc -l`), la parité des quatre catalogues (10
   clés neuves par locale), `npm run lint-i18n-ownership`, les gardes `i18n-un-repli-par-cle` et `i18n-libelle-en-dur`
   (vertes ?), le repli de chaque `i18nMsg` égal à la valeur fr-CH.
4. **Grep du symptôme, la valeur** : `git grep -nF "kesh:session:" -- frontend/src` ; `git grep -nE "addEventListener\('(scroll)" -- frontend/src` ;
   `git grep -nE "LoginPayload = \{|authState.login\(|_currentUser = \{" -- frontend/src ':!*.test.ts'` (trois
   constructeurs et `hydrate()` transmettent les deux champs) ; `git grep -nF "reason=idle" -- frontend/src` ;
   `git grep -nF "défilement" -- docs/manual/fr/*.tex CHANGELOG.md _bmad-output/planning-artifacts` (ne plus promettre le
   défilement : « molette, clavier, souris, toucher ») ; `git grep -nE '\b15 min(utes)?\b' -- _bmad-output/planning-artifacts docs/kesh-specifications.txt` (trié).

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne `session-idle.svelte.ts` (lecture et écriture des clés, `memory` et `shared`, `currentLast`,
`concile`, `recordActivity`, `tick`, `sendHeartbeat` et son `catch`, `closeSession` et son `Promise.race`, `start`/`stop`
et `generation`), `api-client.ts` (`sharedRefresh`, `refreshTokens`, `tryRefresh`, le `.finally` qui libère le mutex),
`auth.svelte.ts` (`isIdleClosurePage`, les champs de transit, la branche 401 d'`hydrate()`), `+layout.svelte` (racine),
`login/+page.svelte`, `SessionIdleSection.svelte`, `(app)/settings/+page.svelte`, `setup.api.ts`, `settings.api.ts`. Cherche :
une course entre deux onglets qui écrit une valeur fausse ; un écouteur posé deux fois ; une fuite de minuterie ; une
fermeture qui n'attend pas la déconnexion ; une redirection perdue quand le chemin 401 rejoint un `tryRefresh` ; un état
Svelte lu dans un effet sans `untrack`.

## Lentille E — Edge (chaque branche, chaque borne)

Chaque branche du module : `idleExpiresIn` nombre, `null`, absent, négatif, très grand ; N inconnu puis appris par `/me`
ou par un battement ; clés absentes, `NaN`, texte, futures ; `localStorage` qui refuse à la lecture, à l'écriture, en
cours de session ; un onglet en arrière-plan (minuterie ralentie) ; l'échéance atteinte pendant un battement en vol ; deux
onglets qui ferment ensemble ; un `stop()` pendant `/me` ; une activité dans la seconde de l'échéance ; un abaissement à 1
minute depuis l'écran ; un rechargement de `/login?reason=idle` ; `minutes=0`, `minutes=1.5`, `minutes=` vide. Chaque cas :
comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère d'AC-6 à AC-11 : livré, testé, au lieu prescrit ? Les rouges déclarés sont-ils réels (`rouges-N.log`) — et
AC-7 (3), déclaré vert avant correctif (garde du mutex, rougie par M13 bis), est-ce juste ? La recette AC-10 : chaque
volet joué, horodaté, au bon écart (`recette-r*.log`) ; les volets non joués (e, f) déclarés ? Les captures (§ *Captures*) :
les six présentes, thème clair ? Le CHANGELOG, FR13 et ses copies, le PDF : conformes ? Le Dev Agent Record n'affirme-t-il
que ce qui a tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/v0172s-review-p1-N-B.md`, `…-N-E.md`, `…-N-A.md`.
Chaque rapport : findings numérotés (`N-B1`, `N-E1`, `N-A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu
(`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le
décompte final **recompté depuis la liste**.
