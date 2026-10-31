# Prompt — revue de code P1 de la Story v017-2 (#809, l'assistant affiche le motif d'un refus), trois lentilles, Sonnet

Tu relis le développement de la **Story v017-2** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11) :
*l'assistant d'installation dit pourquoi le serveur refuse* — **#809** : `errorMessageOf` reçoit un repli optionnel
(`frontend/src/lib/shared/utils/error-message.ts`, doc-comment de `api-client.ts:236`) ; les onze `catch` de
`frontend/src/routes/onboarding/+page.svelte` (S1 à S11) passent par lui — sept `catch` nus, les deux du compte
bancaire, la finalisation (`reportFinalizeError`, dont la branche `'VALIDATION'` était morte) ; une indication sous le
champ Pays (`#coord-country-hint`, `aria-describedby`, sans `maxlength`) ; deux clés aux quatre locales
(`crates/kesh-i18n/locales/*/messages.ftl`) ; deux compteurs de garde i18n recalés (`i18n-keys.test.ts`,
`i18n-un-repli-par-cle.test.ts`) ; tests neufs `onboarding-motif.test.ts`, `onboarding-catch-guard.test.ts` (garde
lexicale), cas ajoutés à `error-message.test.ts` ; test serveur `coordinates_rejects_country_name`
(`crates/kesh-api/tests/onboarding_path_b_e2e.rs`) ; spec E2E `frontend/tests/e2e/onboarding-motif.spec.ts` ;
`docs/testing.md` (une ligne) ; manuel utilisateur FR (`.tex` **et** PDF régénéré) ; `CHANGELOG.md` (en-tête
provisoire `## [0.17.0]`). **Aucune migration, aucun code serveur modifié.**

Worktree `/home/gcorbaz/devel/kesh-0172`, branche `story/v017-2-assistant-motif`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-0172 diff 6abf47dd...HEAD` (`6abf47dd` = fin de la validation de la fiche ; la
branche part de `c5101b07`). Langue du rapport : français. Modèle : **Sonnet**. Passe **complète** (première passe
d'une revue de code). Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Les trois propriétés centrales.**
1. **Le motif du serveur atteint l'écran à chaque étape de saisie** : pour toute `ApiError` à message non blanc, le
   toast montre ce message, tel quel — y compris à la finalisation, hors « déjà finalisé ».
2. **Le repli de l'étape remplace le générique**, et seulement quand le serveur ne donne pas de motif lisible (message
   vide, blanc, non-chaîne, ou valeur qui n'est pas une `ApiError` — toujours journalisée). Sans repli,
   `errorMessageOf` se comporte exactement comme avant (23 fichiers l'appellent).
3. **Le champ Pays n'est pas tronqué** : « Suisse » atteint le serveur et revient expliqué.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/v017-2-assistant-motif.md` — *Ce que le code dit*, l'*Inventaire clos des
`catch`*, les **AC-1 à AC-10**, *Tâches*, *Tests et mutations prévus* (M1 à M15), *E2E*, *Captures*, *Ce que les
gardes ne voient pas*, le **Dev Agent Record** et le **Change Log** ; puis le registre
`v017-2-assistant-motif-choix-autonomes.md`, **C-v017-2-1 à 13** (fiche) et **C-v017-2-14 et suivants** (choix du
développement). Le Dev Agent Record déclare des rouges, des gates, des mutations, un E2E et des captures : **ne les
prends pas pour argent comptant**, recoupe-les avec les journaux hors dépôt (lecture seule), sous
`/home/gcorbaz/devel/kesh-gate-logs/v0172/` : `rouges-etape0.txt`, `mutations-vitest.txt`, `mutation-M15.txt`,
`test-fast.log`, `vitest-complet.log`, `check.log`, `e2e-complet.log`, `make-fr.log` ; le script de mutations Vitest
est `mutations.py` dans ce même répertoire (à **lire**, jamais à exécuter). Les captures (thème clair) sont sous
`/home/gcorbaz/devel/kesh-gate-logs/v0172/captures/` (cinq PNG ; regarde-les si ton outil lit les images), leur spec
sous `/home/gcorbaz/devel/kesh-gate-logs/v0172/capture-spec/` (à lire seulement).

⛔ Ne lis rien sous `~/kesh-test/`.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`, `2>`, `&>`, `2>&1`) : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au
  rapport, il ne se supprime pas.
- **Aucun `rm`, sous aucune forme** (ni `rm -f`, ni `rm -rf`, ni `unlink`, ni `find -delete`) ; ni **`pkill`**, ni
  **`killall`**.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`, `merge`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/**`python3`**, ni `perl`, ni `ruby`,
  ni `node`/**`npm`**/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni **aucun script du dépôt** ni de `kesh-gate-logs` (dont
  `run-backend.sh`, `mutations.py`) ; ni **`make`**, ni `latexmk`, ni **`cargo`**, ni **`docker`**, ni
  `mysql`/`mariadb`, ni `sqlx`, ni **`curl`**, ni **`claude`** sous aucune forme, ni **`sed -i`**, ni `playwright`,
  ni `vitest`.
- **Aucune commande `gh`**, pas même en lecture. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`, et `git show <réf>:<f>.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`.
- ⛔ **Sur le PDF aplati, jamais de `grep -oE` ni de `grep -E` à quantificateur `{0,N}` ou `{N,M}`** : `grep -oF … |
  wc -l` ou `grep -cF` sur des **motifs fixes**, sous `timeout 60`. Exemple :
  `timeout 60 pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF 'code ISO à deux lettres' | wc -l`.
  ⚠️ Le PDF rend l'apostrophe en `’` (U+2019) : `grep -oF "l’interface"`, non `l'interface`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée dans le
  rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques (« Exemple Sàrl », « Banque Exemple », 1000 Lausanne).

## Les axes obligatoires, pour chaque lentille

1. **Le manuel (`.tex` et PDF aplati) contre le code.** La puce *Coordonnées légales* (`user-manual.tex`, § *Chemin B*)
   et l'incise à l'avertissement du § *Choix de la langue* : chaque affirmation est-elle vraie au code (le pays par
   défaut `CH` — `+page.svelte` ; le motif affiché — `errorMessageOf` ; « rédigé en français quelle que soit la langue
   de l'interface » — `address_input.rs`, `routes/onboarding.rs`, `errors.rs`, #710) ? Le PDF dit-il la même chose que
   le `.tex`, et les comptes « zéro avant, un après » du Dev Agent Record se reproduisent-ils
   (`git show 6abf47dd:docs/manual/fr/user-manual.pdf | pdftotext - - | …` pour l'avant) ? Le manuel d'administration
   (`admin-manual.tex`, mentions de l'assistant) dit-il quelque chose de devenu faux ? Un autre lieu du manuel
   utilisateur parle-t-il de l'assistant ou des messages d'erreur de façon désormais fausse
   (`grep -nF 'onboarding' docs/manual/fr/user-manual.tex`) ?
2. **Les témoins et les mutations.** Pour chaque site S1 à S11 : quel test le couvre, sur quel déclencheur, et peut-il
   passer **à vide** (un `toast.error` appelé par une autre branche, un `vi.waitFor` qui se contente d'un appel
   antérieur, un `mockRejectedValueOnce` consommé ailleurs, l'état `_loading` de module laissé à `true` par un
   scénario de finalisation) ? Les mutations M1 à M15 sont-elles tuées **par le test nommé** (journaux) et pour la
   raison visée ? M15 a rougi sur le **code** d'erreur (la colonne de la base refuse la valeur longue) et non sur le
   statut : le test serveur prouve-t-il ce qu'il annonce ? Une mutation plausible non jouée survivrait-elle (par
   exemple `errorMessageOf(err, msg(...))` remplacé par `errorMessageOf(autre, …)`, ou un `catch` dont le toast est
   retiré mais l'appel à `errorMessageOf` gardé dans une variable) ?
3. **Les gardes et compteurs.** La garde `onboarding-catch-guard.test.ts` : motif de recueil, calcul de la position de
   la parenthèse, lecture du bloc, plancher, témoins ; ses angles morts sont-ils écrits ? Les deux compteurs recalés
   (`sitesTotal` 2619 → 2620, `CLES_RELEVEES` 344 → 346) se recomptent-ils aux deux bornes (`c5101b07` → `HEAD`) par la
   commande du bloc daté, et le bloc daté dit-il vrai ?
4. **Grep du symptôme sur tout le worktree** : `git grep -nF "'VALIDATION'" -- frontend/src` ; `git grep -nF 'catch {'
   -- frontend/src/routes/onboarding` ; `git grep -nE "serveur l'a (déjà )?traduit" -- frontend/src` ; `git grep -nF
   'onboarding-language-error'` et `'onboarding-field-country-hint'` (quatre locales et un site chacune) ;
   d'autres écrans de l'application qui affichent le motif « à la main » (`isApiError(err) && err.message ?`) et que
   la story aurait dû ou non toucher (#644) — chaque occurrence restante est-elle justifiée ?

## Lentille B — Blind (le diff seul, sans la fiche)

1. **Le code de production** (`error-message.ts`, `+page.svelte`, `api-client.ts`) : le comportement de
   `errorMessageOf` avec et sans repli (repli vide, blanc, non-chaîne à l'exécution) ; le doc-comment dit-il vrai ;
   `reportFinalizeError` (l'ordre des branches, un `isApiError` devenu inutile ou manquant) ; l'indication du pays
   (accessibilité : `aria-describedby` sur l'élément `<input>` réel du composant `Input`) ; un import mort ?
2. **Les tests** : `onboarding-motif.test.ts` (mocks, `atStep`, `jouer`, `finaliser`, le `setTimeout(0)` entre deux
   scénarios), la garde, les cas ajoutés à `error-message.test.ts`, `coordinates_rejects_country_name`, la spec E2E
   (sélecteurs d'`id`, aucun sélecteur de texte, contexte libéré).
3. **Les textes** : CHANGELOG (en-tête provisoire, commentaire HTML, section *Corrigé*), `docs/testing.md`, LaTeX
   (`\#710`, tirets cadratins, `~;`) ; aucun mot-clé de fermeture (`closes`, `fixes`, `resolves`) dans les messages de
   commit (`git log 6abf47dd..HEAD --format=%B`).

## Lentille E — Edge cases

1. **Les erreurs** : une `ApiError` de statut 0 (réseau, message fabriqué par le client), un 500 (« Erreur interne »),
   un 403, un 409, `ONBOARDING_STEP_ALREADY_COMPLETED` aux étapes S1 à S8/S10 (C-v017-2-6) ; un rejet par une chaîne,
   par `null` ; la démonstration (S3) et ses écritures partielles (#538).
2. **La finalisation** : après S9/S11, l'état vaut 7 et l'écran est vide (#821) ; un double clic ; le verrou de
   `finalize` ; un refus qui n'est pas une `ApiError`.
3. **Le champ Pays** : « ch » en minuscules, « CH  » avec blancs, chaîne vide (`|| 'CH'`), « Suisse » ; la longueur
   de l'indication en de-CH et it-CH (capture 4) ; le comportement du serveur pour une valeur longue (M15 :
   `DATA_LENGTH_OR_RANGE` si la validation disparaissait).

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC (**AC-1 à AC-10**), rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve.
En particulier : AC-1 (signature, quatre règles, repli blanc, doc-comment corrigé et sa propagation à
`api-client.ts:236`) ; AC-2 (S1 à S7, replis inchangés, clé neuve pour S1) ; AC-3 (S8, S10) ; AC-4 (la forme
**exacte** prescrite de `reportFinalizeError`) ; AC-5 (garde : motif, plancher, trois témoins) ; AC-6 (élément,
classes, `aria-describedby`, pas de `maxlength`, `CH` et `placeholder` inchangés) ; AC-7 (les huit valeurs, places dans
les fichiers) ; AC-8 (test serveur) ; AC-9 (manuel, PDF, comptes, fragment cité) ; AC-10 (CHANGELOG, texte prescrit).
Les tâches 3 (compteurs, import) et 7 (propagation) ; les **mutations M1 à M15** (chacune jouée ? tuée par le test
attendu ?) ; l'E2E (§ *E2E* : état amené par l'API, sélecteurs, `disposeContextSafe`, `clearAuthStorage`, place dans
`docs/testing.md`) et ses échecs déclarés, jugés contre `docs/testing.md` § *Les échecs attendus* ; les **captures**
(§ *Captures* : cinq états, thème clair, de-CH sous `KESH_LANG=de`). L'étape 0 : les sept LOW de la validation P3
(`/home/gcorbaz/devel/kesh-gate-logs/v017-2-assistant-motif-validate-p3.md`) sont-ils appliqués, ligne lue citée ? Les
entrées passées du Change Log sont-elles restées des relevés datés (annotées, non réécrites :
`git diff 6abf47dd...HEAD -- _bmad-output | grep '^-'`) ?

Et : les choix C-v017-2-14 et suivants sont-ils justifiés, l'un contredit-il un AC, la fiche ou le registre ? Chaque
nombre du Dev Agent Record (tests, mutations, gates, E2E) se recompte-t-il depuis sa source, avec son périmètre ? Les
**défauts observés** que le Dev Agent Record demande en issue sont-ils réels (prémisse lue au code) ?

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/v017-2-assistant-motif-review-p1-B.md`
(lentille B), `/home/gcorbaz/devel/kesh-gate-logs/v017-2-assistant-motif-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/v017-2-assistant-motif-review-p1-A.md` (lentille A), selon la lentille qui t'est
confiée — par l'outil d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier
   et ligne, la preuve (sortie de commande copiée, `grep -nF` pour toute présence ou absence), la correction proposée,
   et s'il est **né du développement** de la v017-2 ou **antérieur** (un défaut antérieur se **signale**, il ne se
   corrige pas ici).
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, et toute commande lancée hors de la liste permise, s'il y en a.
