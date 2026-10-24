# Prompt — revue de code P1 de la Story 31-1b (restaurer une installation neuve depuis l'assistant), trois lentilles, Sonnet

Tu relis le développement de la **Story 31-1b** de Kesh (comptabilité suisse) : après `/setup`, l'assistant propose
**« Restaurer une sauvegarde »** à l'Administrateur aux étapes 0 à 2, vers une page neuve `/onboarding/restore` qui rend
le panneau d'import existant ; les écrans de sauvegarde disent ce que le fichier ne contient pas (`.env`), ce que
l'export a omis (en-têtes de la 31-1a) et ce que l'import a rendu (pièces restaurées, manquantes), puis attendent
**« Se reconnecter »** ; l'export et l'import échappent au délai de 30 s du client HTTP (une heure, sans rejeu) et une
réponse d'import perdue (`TIMEOUT`, `NETWORK_ERROR`, 502, 504) affiche un message d'**issue inconnue** ; E2E de bout en
bout ; manuel d'administration pas à pas. Issue #503 (la 31-1b ne la ferme pas). Worktree
`/home/gcorbaz/devel/kesh-31-1b`, branche `story/31-1b`, partie de `story/31-1` = `1ee0583e` (la 31-1a). Le diff à
relire, **aplati** (un seul diff, pas la suite des commits) : `git -C /home/gcorbaz/devel/kesh-31-1b diff
1ee0583e...HEAD`. Langue du rapport : français. Modèle : **Sonnet**.

Lis d'abord : la fiche `_bmad-output/implementation-artifacts/31-1-sauvegarde-complete.md` — **partie b seulement**
(AC15 à AC22, AC18b ; tâches T-b1 à T-b7 ; mutations M-b1 à M-b7 ; Dev Agent Record de la 31-1b ; entrée « Revue de
code P1 de la 31-1a » du Change Log, dont la 31-1b reprend des LOW) — et le registre
`_bmad-output/implementation-artifacts/epic-31-choix-autonomes-311.md` (C-31-1-69 et suivants pour le développement ;
C-31-1-10, 11, 25, 39 pour la conception). Le Dev Agent Record déclare des gates, des chiffres et des mutations : **ne
les prends pas pour argent comptant**, recoupe-les avec les journaux `/home/gcorbaz/devel/kesh-gate-logs/311b-*.log`
(lecture seule). Les captures de l'AC21 sont dans `/home/gcorbaz/devel/kesh-gate-logs/311b-captures/` (lis-les comme
images). Les parties a et c de la fiche sont **hors périmètre** : un défaut qui ne vit qu'en elles n'est pas un finding.

Rapports, un par lentille : `/home/gcorbaz/devel/kesh-gate-logs/311b-review-p1-B.md`, `…-E.md`, `…-A.md`. Chaque
rapport porte : un bilan CRITICAL / HIGH / MEDIUM / LOW ; chaque finding avec sa preuve (sortie `grep -nF` ou `sed -n`
copiée) ; la liste des **axes exercés** et celle des **axes non exercés** — un « 0 finding » sans cette liste ne compte
pas comme passe.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. Aucun fichier temporaire : compare
  par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`,
  ni `scripts/mem-guard.sh`, ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni
  `vitest`, ni `playwright`, ni `node`, ni `docker`, ni `python3`, ni `sed -i`, ni `perl`. Aucune commande `gh` qui
  écrit ; `gh issue view 503` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. `frontend/src/lib/shared/utils/api-client.ts` : `CallOptions`, `LONG_OPERATION_TIMEOUT_MS`, `fetchWithTimeout`
   (le délai ne court-il que jusqu'aux en-têtes ?), `fetchWithRetry` (`retry: false` respecté pour **toute** méthode ?),
   `request` / `requestRaw` (les options survivent-elles au rejeu après un `401` + refresh ?), `getBlob`,
   `postFormData`. Les autres appelants de `getBlob` / `postFormData` (`git grep -n "getBlob(\|postFormData("`) gardent-ils
   les défauts d'avant ?
2. `frontend/src/lib/features/admin-backup/` : `admin-backup.api.ts` (lecture des en-têtes `x-kesh-backup-*`,
   `headerCount` : valeur absente, vide, non numérique, énorme ; casse des noms d'en-têtes), `AdminBackupPanel.svelte`
   (avis effacés à chaque export ? affichés seulement si > 0 ?), `BackupEnvNotice.svelte` (neuf).
3. `frontend/src/lib/features/admin-restore/` : `AdminRestorePanel.svelte` — `issueInconnue`, `importErrorMessage`,
   `confirmImport` (aucune redirection avant « Se reconnecter » ; ré-entrance ; second import impossible après un
   succès), `relogin` (logout best-effort puis redirection **garantie**), rendu des avis ; `admin-restore.api.ts`
   (`filesRestored` / `filesMissing` optionnels : un serveur sans ces champs casse-t-il l'écran ?).
4. `frontend/src/routes/onboarding/+page.svelte` (`showRestoreLink` : étapes, rôle, état non chargé),
   `routes/onboarding/restore/+page.svelte` et `+page.ts` (garde de rôle ; la garde de `routes/onboarding/+layout.ts`
   couvre-t-elle bien la page — non authentifié, assistant terminé ?). Un non-Admin peut-il voir l'UI destructrice ?
5. i18n : les quatre `crates/kesh-i18n/locales/*/messages.ftl` — clés neuves présentes partout, `admin-restore-toast-success`
   retirée partout et de tout le code, replis Svelte **identiques** au fr-CH (compare au caractère près), variables
   `{ $count }` / `{ $mib }` présentes dans chaque traduction ; `frontend/src/lib/shared/i18n-keys.test.ts`
   (`sitesTotal` 2279 → 2289) et `i18n-un-repli-par-cle.test.ts` (341 → 344) : les ventilations écrites sont-elles
   justes (recompte par `git show 1ee0583e:<f> | grep -oE '(i18nMsg|[^a-zA-Z]msg)\(' | wc -l`) ?
6. `crates/kesh-api/tests/textes_coherents.rs` (G18-ter : fragments assumés réécrits — invariant gardé ou affaibli ?),
   `crates/kesh-db/src/backup.rs` (commentaire seulement).
7. Documentation : `docs/manual/fr/admin-manual.tex` **et son PDF** (`docs/manual/fr/admin-manual.pdf`, aplati) — la
   sous-section *Restaurer sur une nouvelle machine* (`sec:restauration-machine-neuve`), *Ce que dit l'écran à la fin
   de l'import*, la liste des refus de l'import et ses deux limites, le paragraphe du premier démarrage ;
   `docs/api-external.md` (le 410) ; `CHANGELOG.md` (§ `[0.15.0] — Non publié`) ; `README.md`. **Le manuel promet-il ce
   que le code ne fait pas** — par exemple : `KESH_JWT_SECRET` neuf sans conséquence autre que les sessions (vérifie
   ses usages : `git grep -n jwt_secret crates/kesh-api/src`), le lien visible « jusqu'au choix entre démonstration et
   production », « l'application s'ouvre sans l'assistant », « environ deux fois la taille du fichier », un 504 « rien
   n'est touché » ?

## Lentille E — Edge cases (chaque branche)

1. Client HTTP : pour `retry` absent / `true` / `false` et méthode `GET` / `POST`, combien de tentatives ? `timeoutMs`
   absent / `0` / négatif / une heure. Un `401` suivi d'un refresh réussi : le second essai garde-t-il le délai d'une
   heure et `retry: false` ? Un `503` sur un `getBlob` à `retry: false` : rendu au panneau tel quel ?
2. `downloadFullExport` : en-têtes absents, `"0"`, `"abc"`, `""`, `" 3 "`, `"1e3"`, `"-1"`, `"99999999999999999999"`.
   Pour chacun, la valeur rendue et l'avis affiché (ou non).
3. `AdminRestorePanel` : chaque forme d'erreur (`ApiError` à `status` 0 de code `TIMEOUT` / `NETWORK_ERROR` / autre ;
   statut 400, 409, 413, 500, 502, 503, 504 ; erreur non `ApiError`) → quel message, et « Se reconnecter » proposé ou
   non ; réponse `200` sans `filesRestored` / `filesMissing` ; `filesMissing` 0 / > 0 ; double clic sur « Se
   reconnecter » ; `logout()` qui rejette ; nouvel essai après une issue inconnue.
4. Assistant : `stepCompleted` 0, 1, 2, 3, 8 ; démo à l'étape 3 ; rôle `Admin`, `Comptable`, `Consultation`, aucun
   utilisateur ; état d'onboarding non chargé (échec de `fetchState`) ; accès direct à `/onboarding/restore` d'un
   non-Admin, d'un non-authentifié, une fois l'assistant terminé.
5. E2E `frontend/tests/e2e/backup-restore-new-install.spec.ts` : peut-il passer **à vide** (garde de suppression des
   pièces ; pièces comptées « 2 » alors qu'une troisième pourrait être référencée — sonde d'import, résidu d'un run
   précédent du dossier des documents ; comparaison des octets du PDF ; état laissé aux specs suivantes) ? Les deux
   parcours d'interception (`page.route`) prouvent-ils ce qu'ils disent (le 504 n'atteint pas Kesh) ?
6. Mutations M-b1 à M-b10 : pour chacune, le diff du journal `311b-mut-<M>.log` est-il la mutation décrite, le test qui
   la tue est-il celui que le Dev Agent Record nomme, et l'échec porte-t-il sur l'assertion qui compte ?

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC15 à AC22 et AC18b, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve.
En particulier :

- **AC15** : lien aux étapes 0–2, Admin seulement ; page `/onboarding/restore` rendant `AdminRestorePanel` ; garde de
  rôle identique à `(app)/admin/backup/+page.ts` (redirection : `/onboarding`, et non `/` — le registre le justifie-t-il ?).
- **AC16** : prouvé par l'E2E (connexion avec les identifiants de la sauvegarde, application sans assistant).
- **AC17** : texte de `admin-backup-not-included-env` au caractère près de la fiche ; présent sur les deux panneaux
  (composant partagé, C-31-1-69 : la règle `lint-i18n-ownership` le justifie-t-elle ?).
- **AC18** : textes des avis — la fiche donne « N pièces référencées n'ont pas pu être incluses … » ; le code dit
  « Pièces référencées que le fichier n'a pas pu inclure : N … » (écart déclaré, C-31-1-70) : acceptable ? En-têtes
  absents → aucun avis.
- **AC18b** : options, constante, tests à horloge simulée (30 s, 59 min, 60 min, un seul `fetch`), espions sur
  `apiClient`, message d'issue inconnue pour les quatre cas, 503 au défaut.
- **AC19** : toast retiré (code **et** quatre catalogues), « Se reconnecter » après tout succès, avis `filesMissing`.
- **AC20** : chaque étape (1) à (6) de la fiche retrouvée dans la spec ; « 2 pièces restaurées » ; facture, écriture,
  aperçu, PDF identique, fichiers revenus ; état remis en fin de spec.
- **AC21** : sous-section du manuel, étapes (1) à (5) de la fiche, renvoi depuis *Test de restauration périodique* ;
  PDF régénéré et à jour (aplati) ; captures (a) à (i) présentes et conformes à leur état, voie d'obtention dite au
  Dev Agent Record.
- **AC22** : quatre locales, replis identiques au français, `lint-i18n-ownership`, description complétée.
- **LOW confiés par la revue P1 de la 31-1a** (Change Log) : commentaire `backup.rs` ; refus du format 2, bornes de
  lecture, 410 d'une pièce non emportée, angles morts E-3 et E-4 — au manuel et à `docs/api-external.md` (le Dev Agent
  Record explique pourquoi les refus ne vont pas dans `api-external.md` : est-ce exact ?).
- **Gates** : les chiffres du Dev Agent Record (nextest, Vitest, `check`, `lint-i18n-ownership`, `build`, E2E et liste
  des échecs jugés contre `docs/testing.md` § *Les échecs attendus*) correspondent-ils **aux journaux** ? Le gate
  complet et l'E2E ont-ils tourné **au dernier commit de code** (compare au `git log --oneline 1ee0583e..HEAD`) ?
- **Écarts déclarés** au Dev Agent Record : sont-ils exacts et complets ?
