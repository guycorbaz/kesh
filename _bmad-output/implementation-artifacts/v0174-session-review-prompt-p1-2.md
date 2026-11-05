# Prompt — revue de code P1 de la Story v0174-session, **sous-fiche 2 (navigateur)**, trois lentilles, Sonnet

Tu relis la **sous-fiche 2** (navigateur et textes) de la **Story v0174-session** de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit 5 runes, MariaDB 10.11), version **0.17.4** : **l'avertissement avant la fermeture de la session pour
inactivité, avec « Rester connecté »** (#895, CR accepté), et les LOW navigateur des revues de la 0.17.3 (#898). Elle
porte : le module `frontend/src/lib/app/stores/session-idle.svelte.ts` (`WARNING_LEAD_MS`, `warningRemainingMs` calculé par
`tick`, `stayActive()`, `stop()` qui remet l'avis à `null`, `removeKey` après un battement en échec sans activité rapportée
— N-E5 —, écouteur `pointerout` — Rec-e1) ; le composant neuf `frontend/src/lib/shared/components/SessionIdleWarning.svelte`
(propriétés `remainingMs`, `onStay` ; annonce statique `role="alert"`, compte à rebours `aria-hidden`, aucun déplacement du
focus ; au-dessus d'un dialogue : écouteur **natif** `pointerdown` qui arrête la propagation, `pointer-events-auto`,
`z-[60]`) ; le layout racine (`routes/+layout.svelte`, rendu sur `typeof … === 'number'`) ; trois clés i18n aux quatre
catalogues ; les tests (Vitest du module, du composant, du layout ; compteur d'`i18n-keys.test.ts`) ; la spec E2E neuve
`frontend/tests/e2e/session-idle-warning.spec.ts` ; les manuels utilisateur et d'administration (`.tex` et PDF), le
CHANGELOG, la ligne d'acceptation du CR #895 aux spécifications.

Worktree `/home/gcorbaz/devel/kesh-0174s`, branche `story/v0174-session`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-0174s diff 76f5007d 92696469` (`76f5007d` = dernier commit de la sous-fiche 1 ;
`92696469` = dernier commit de textes de la sous-fiche 2 ; les comptes rendus suivent). Langue du rapport : français.
Modèle : **Sonnet**. Passe **complète** (première passe). Contexte frais.

⚠️ **Les propriétés centrales.**
1. **L'avis ne change ni l'échéance ni sa précision** : il n'est qu'un affichage et une activité de plus ; à l'échéance,
   la fermeture a lieu comme en 0.17.3.
2. **L'avis ne se renouvelle jamais seul** : aucun déplacement programmatique du focus (un `focus` de la fenêtre compte
   comme activité) ; aucune région `aria-live` sur le compte à rebours ; pas de `role="alertdialog"`.
3. **Au-dessus d'un dialogue ouvert, sans le fermer ni en perdre la saisie** (les trois propriétés de C-v0174s-13,
   établies dans `frontend/node_modules/bits-ui` 2.16.5) ; Échap reste au dialogue.
4. **Sans `localStorage`, pas d'avis** (le serveur ferme, sans avertissement).
5. **`stayActive()` n'envoie rien lui-même** : le battement part au tic suivant par le chemin ordinaire.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la parente `_bmad-output/implementation-artifacts/
v0174-session.md` (§ #895, § *Triage des LOW*, § *Textes*), la sous-fiche `v0174-session-2.md` en entier (Conception,
AC-1 à AC-8, *Tests et mutations* W1 à W15, *Captures*, *Manuel*, *CHANGELOG*, **Change Log** et **Dev Agent Record**) ;
le registre `v0174-choix-autonomes-v0174s.md` (C-v0174s-10 à 15, 17, 18, 20). Ne prends pas le Dev Agent Record pour
argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/
v0174s/` : `rouges.log` (section « rouges navigateur »), `vitest-cible-2.log`, `mut/campagne-N.log` et `mut/W*.log` (le
script `scripts/mut-v0174s-N.py` est à **lire**, jamais à exécuter), `front-check.log`, `front-lint-i18n.log`,
`front-unit.log`, `front-build.log`, `gate-test-fast.log`, `e2e-r*/playwright.log`, `manuels-2.log`, `recette-2.log`
(`scripts/recette-2-v0174s.mjs` à **lire**). **Captures** (thème clair, au viewport) sous
`/home/gcorbaz/devel/kesh-gate-logs/v0174s/captures/` : `v0174s-01-avis.png`, `v0174s-02-avis-apercu-pdf.png`,
`v0174s-03-avis-dialogue.png`, `v0174s-04-apres-clic.png`, `v0174s-05-fermeture.png` — regarde-les si ton outil lit
les images : l'avis est-il lisible, contrasté, au-dessus du dialogue et de l'aperçu sans les masquer entièrement ?

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle
  part ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
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

1. **Les manuels (`.tex` et PDF aplati) contre le code** : `docs/manual/fr/user-manual.tex` (§ de la session : la phrase
   de l'avertissement — « environ une minute avant », « trente secondes » pour une durée d'une minute, « Toute autre
   activité dans Kesh hors du document affiché la garde aussi » — et la phrase en gras de l'aperçu) ; `admin-manual.tex`
   § *Durée d'inactivité* (« Ce qui compte », `localStorage` et « aucun avertissement »), la part « Ce qui change à
   l'écran » de « Passer à la 0.17.4 » ; et les **PDF** régénérés (`make admin user`) : « 895 » absent, « Rester connecté »
   présent. Chaque phrase est-elle vraie au code (le calcul de `tick`, `stayActive`, le composant) ? « Toute autre
   activité … la garde aussi » est-il vrai pour un clic **hors** d'un dialogue ouvert (qui le fermerait) ?
2. **Les témoins et les mutations** : AC-1 à AC-6 — quel test couvre chaque cas ; peut-il passer **à vide** (minuteries
   simulées mal avancées ; `vi.setSystemTime` mal placé ; un espion de `setItem` posé après l'écriture ; un simulacre du
   layout dont le getter n'est pas lu) ; chaque mutation W1 à W15 (W13 retirée) est-elle tuée **par le test attendu**
   (`mut/W*.log`) ? `Wx-nullish` (hors liste, la prémisse « un `??` muté en `||` rougit ») survit : la conclusion
   « équivalente en pratique » est-elle juste ? Mutations plausibles non jouées à proposer (par exemple : l'avis calculé
   **après** le `return` de `heartbeatInFlight` ; `stayActive` sans le garde `!running` ; l'écouteur `pointerdown` posé en
   **capture** ; `onpointerdown` délégué au lieu de l'écouteur natif ; `z-50` au lieu de `z-[60]`).
3. **Les gardes et compteurs** : `i18n-keys.test.ts` (`sitesTotal` 2826 → 2829, recompté à la source :
   `grep -o 'i18nMsg(' frontend/src/lib/shared/components/SessionIdleWarning.svelte | wc -l`), la parité des quatre
   catalogues (trois clés neuves par locale), `i18n-repli-divergent-actif`, `i18n-un-repli-par-cle`,
   `i18n-entrees-a-variables` (le site passe `{ remaining }`), `jetons-non-definis` (les classes de couleur de l'avis
   sont-elles définies ?), `lint-i18n-ownership` (vert par construction pour ce composant).
4. **Grep du symptôme, la valeur** : `git grep -nF "895" -- docs/manual/fr/*.tex CHANGELOG.md` (trié : historique ou
   vivant) ; `git grep -nF "sortez le pointeur" -- docs/manual/fr/*.tex` (la phrase de la 0.17.3 restée dans « Passer à
   la 0.17.3 » : historique ou à reprendre ?) ; `git grep -nF "warningRemainingMs" -- frontend/src` ;
   `git grep -nE "\.focus\(\)|autofocus" -- frontend/src/lib/shared/components/SessionIdleWarning.svelte` ;
   `git grep -nF "onze" -- frontend/src/lib/app/stores/session-idle.svelte.test.ts`.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne le module (le calcul de l'avis dans `tick` et sa place, `stayActive`, `stop`, `removeKey`,
`onPointerOut`, `addListeners`/`removeListeners`), le composant (le `$effect` et son nettoyage, `$derived.by`, le
`bind:this`, les classes), le layout, les tests neufs et la spec E2E (`page.clock`, l'ordre des `fastForward` et des
`waitForResponse`, `afterAll` qui remet N à 60). Cherche : un état Svelte écrit hors du module ; un écouteur jamais retiré ;
un `$effect` qui se relance et repose l'écouteur ; une fermeture qui laisse l'avis affiché ; un test E2E qui avance
l'horloge pendant une requête en vol.

## Lentille E — Edge (chaque branche, chaque borne)

N = 1 (avis à 30 s), N = 2, N = 1440 ; `remainingMs` à 59 999, 60 000, 60 001, 1, 0 ; le compte à rebours au passage de la
minute (`1:00` → `0:59`) ; une activité dans un autre onglet ; `localStorage` qui refuse en cours de session (l'avis
disparaît-il ?) ; `stayActive()` après l'échéance, après `stop()`, dix fois en 500 ms ; un battement en échec quand
`reportedActivity` est absente, présente ; `pointerout` vers `null`, vers un `<iframe>`, vers un `<div>` ; l'avis au-dessus
d'un dialogue à `preventScroll` vrai (ScrollLock) et à `trapFocus` vrai (la tabulation, écart tracé par #912) ; un onglet en
arrière-plan. Chaque cas : comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC-1 à AC-8 : livré, testé, au lieu prescrit ? Les rouges déclarés sont-ils réels (`rouges.log`) ? L'étape 0
(le dialogue à champ choisi pour l'AC-4 (c) — la création de contact —, la justesse de l'AC-4 sous `page.clock`, les trois
propriétés éprouvées en Chromium par l'AC-4 (c)) est-elle écrite au Change Log, ligne lue citée ? La recette AC-8 (a) à
(f) : chaque volet joué, horodaté, au bon écart (`recette-2.log`) ; les captures 01 à 05 présentes, thème clair, au
viewport ? Le CHANGELOG, la ligne des spécifications : conformes ? Le Dev Agent Record n'affirme-t-il que ce qui a tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/v0174s-review-p1-2-B.md`, `…-2-E.md`, `…-2-A.md`.
Chaque rapport : findings numérotés (`N-B1`, `N-E1`, `N-A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu
(`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le
décompte final **recompté depuis la liste**.
