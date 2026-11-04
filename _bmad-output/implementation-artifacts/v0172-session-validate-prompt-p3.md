# Prompt — validation P3 de la fiche v0172-session (release 0.17.2 : #876, la session), lentille complète, Sonnet

Tu valides la **fiche** de la Story v0172-session de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11), release
corrective **0.17.2**, **sans migration**. La fiche soutient qu'avec les réglages par défaut (jeton d'accès et inactivité à
15 minutes) le renouvellement sur 401 **échoue toujours**, si bien que la session meurt 15 minutes après la connexion quelle que
soit l'activité. Elle prescrit : un champ `sessionExpiresIn` au serveur ; un renouvellement **à l'entrée de l'utilisateur**, au
plus une fois par pas `T`, **sans minuterie** ; la borne d'inactivité `max(N, jwt)` (aucun effacement tant que le jeton d'accès
vit) ; une garde du chemin 401 à **entrée récente** (module armé seulement) ; la tolérance des échecs transitoires ; un verrou
entre onglets, que la **déconnexion** prend aussi ; la **suspension** du renouvellement après un import complet.

Cette fiche a été **extraite** le 2026-11-04 de `v0172-correctifs.md` (dont elle était la sous-story A1) à la remédiation de la
validation P2 de celle-ci (Opus, 0/0/11/12) : les passes P1 et P2 ont donc porté sur ce texte quand il était dans la fiche mère
(rapports `/home/gcorbaz/devel/kesh-gate-logs/v0172b-validate-p1.md` et `…-p2.md`). C'est la **troisième** passe, **complète**.

Worktree `/home/gcorbaz/devel/kesh-0172b`, branche `story/v0172-correctifs`, partie de `d7ec2ea2` (le code applicatif n'a pas
bougé depuis : seuls des fichiers de `_bmad-output` diffèrent). Langue du rapport : **français**. Modèle : **Sonnet**. **Une seule
lentille**, R (la règle et sa preuve) et F (les textes et ce qu'ils affirment). **Contexte frais** : refais les relevés ; ne lis
les rapports P1 et P2 qu'**après** les tiens, pour l'axe de la remédiation.

## À lire

1. La fiche `_bmad-output/implementation-artifacts/v0172-session.md`, **en entier**.
2. Le registre `_bmad-output/implementation-artifacts/v0172-choix-autonomes.md`, section `v0172s` (C-v0172s-1 à 14) et les choix
   `C-v0172b-2, 3, 4, 5, 18, 23, 24` (déplacés, historique).
3. L'issue, en lecture seule : `gh api repos/guycorbaz/kesh/issues/876` et `…/876/comments`.
4. Le code : `crates/kesh-api/src/routes/auth.rs` (en entier), `routes/setup.rs`, `auth/jwt.rs`, `config.rs` (durées et bornes),
   `middleware/auth.rs`, `main.rs` (purge des jetons, pool), `crates/kesh-db/src/repositories/refresh_tokens.rs`,
   `crates/kesh-api/src/exports/global.rs` ; `frontend/src/lib/shared/utils/api-client.ts` et `api-client.test.ts`,
   `frontend/src/lib/app/stores/auth.svelte.ts` et son test, `frontend/src/hooks.client.ts`, `frontend/src/routes/login/+page.svelte`,
   `frontend/src/lib/features/setup/setup.api.ts`, `frontend/vitest-setup.ts`, `BackupScheduleBanner.svelte`,
   `api-health.svelte.ts`, `lib/features/admin-restore/AdminRestorePanel.svelte` et son test, `routes/(app)/admin/restore/+page.svelte`,
   `routes/onboarding/restore/+page.svelte`, `lib/features/imported-supplier-invoices/DocumentPreview.svelte`,
   `routes/(app)/bank-accounts/bank-accounts-page.test.ts:695-720` (précédent d'horloge et de fuseau simulés).
5. Le **manuel** : `docs/manual/fr/user-manual.tex:90-125` et `docs/manual/fr/admin-manual.tex` — tout l'inventaire du § *Manuel*
   de la fiche (64 lignes, 11 au tableau, 53 hors classe). **Le PDF aussi**, lu aplati vers la sortie standard seulement :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem `admin-manual.pdf`).
6. Pour contexte : `CLAUDE.md` du worktree (*Review Iteration Rule*, *Règle de splitting préventif*, *Propagation post-patch*,
   *Recompter ses propres comptes rendus*) ; `docs/agents-orchestration.md`.

## Axes à exercer

**La remédiation — axe obligatoire, à traiter en premier**

- Les findings **P1 à P5 et P12 à P17** de la validation P2 (rapport cité) et ceux de la P1 qui portaient sur #876 : le traitement
  écrit au Change Log de cette fiche est-il **réellement** dans la fiche et le registre, et **juste** ? Le refus partiel de P16
  (« la table n'est pas dans l'export global ») est-il vrai au code ? Un remède a-t-il introduit un défaut neuf ?
- **La borne** : refais les déroulés `N = jwt = 15` ; `N = 60, jwt = 15` ; `N = 1, jwt = 15` ; `N = 15, jwt = 1`. La formule
  « entre `max(N, jwt) − T` et `max(N, jwt)` après la dernière entrée qui a renouvelé » et le cas « lecture plus longue que `N`
  quand `N < jwt` » sont-ils vrais ? Le manuel prescrit le dit-il exactement ?
- **Le chemin 401 à entrée récente** : un parcours légitime est-il cassé (clic puis requête, soumission au clavier, requête
  déclenchée plus de 10 s après l'entrée par une attente réseau, rechargement de page, première requête après `hydrate`, onglet
  revenu au premier plan) ? Une sonde de fond peut-elle encore prolonger la session ?
- **Module armé / non armé** : les six tests `api-client.test.ts:205, :246, :270, :327, :488, :567` restent-ils verts **sans
  modification** sous la conception écrite ? Quels autres tests existants (store d'authentification, layout, pages qui appellent
  `authState.login(` ou `logout()`, `AdminRestorePanel.test.ts`) la conception change-t-elle — cherche-les et nomme-les ?
- **La déconnexion sous le verrou** : `navigator.locks` absent (jsdom), un verrou tenu par un onglet figé, un `logout()` appelé par
  `AdminRestorePanel.relogin()` alors que le module est suspendu : la déconnexion peut-elle bloquer ou échouer ?
- **La suspension après import** : l'appel est-il au bon endroit (`:156`), la reprise sur refus correcte, `unknownOutcome` couvert,
  la route de l'assistant (`/onboarding/restore`, sans session armée ?) cohérente ?

**La règle et sa preuve**

- La prémisse (§ 1 à 12) à la ligne près, y compris `git grep` large des renouvellements dans `frontend/src` (hors tests).
- `sessionExpiresIn` sur `/me` : cookie envoyé à `/api/v1/auth/me`, `require_auth`, `find_active_by_token`.
- Les AC-1 à AC-8 sont-ils décidables, et chaque mutation M1 à M15 tuée par le test qu'elle nomme ? Une régression probable sans
  mutation ?
- Le décompte des modules (cinq) et la complétude de la liste des fichiers : un fichier que les AC ou les Tâches font toucher
  manque-t-il ?
- Règles du dépôt : aucune migration (`ls crates/kesh-db/migrations/*.sql | wc -l` = 88) ; aucun mot-clé de fermeture prescrit ;
  gate complet et E2E complet au dernier commit de code.

**Les textes et ce qu'ils affirment**

- **Le manuel** — axe obligatoire : chaque phrase prescrite est-elle vraie au code après correctif ? Rejoue le relevé du manuel
  d'administration (64 lignes) : chaque classement et chaque verdict du tableau tiennent-ils (`SameSite`, stockage du jeton,
  `:2322`, `:2998`) ? Le PDF dit-il la même chose que le `.tex` ?
- **CHANGELOG** : la puce est-elle vraie et compatible avec la borne écrite ?
- **Les décomptes** (88, 64 / 53 / 11, six tests, cinq modules, quatorze choix, quinze mutations) : recompte-les depuis la source.

## Ce qu'il faut rendre

Un rapport en Markdown à `/home/gcorbaz/devel/kesh-gate-logs/v0172s-validate-p3.md`, **écrit par l'outil d'écriture de fichier
(`Write`), une seule fois**. Il contient :

1. **Les findings**, `P1`, `P2`…, avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), origine (**remédiation** ou **défaut
   d'origine**), lieu (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède. Une affirmation
   de présence ou d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans cette liste
   ne compte pas.
3. Un décompte final par sévérité et par origine, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture hors du rapport**, et le rapport par l'outil `Write` seulement : **aucune redirection** (`>`, `>>`, `2>`, `&>`,
  `| tee`), **aucune option qui écrit un fichier** (`-o`, `--output`, `pdftotext` vers un fichier), **aucun fichier temporaire**
  — ni dans `/tmp`, ni dans le scratchpad, ni ailleurs. `pdftotext` s'emploie avec `-` (sortie standard) seulement. Un fichier créé
  par erreur se **déclare** au rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous aucune forme.
- Aucune commande `gh` qui écrit ; `gh api` en `GET` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `npx`, ni `make`, ni `latexmk`, ni `python3`, ni `node`, ni
  `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- **Jamais `scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun autre script du dépôt.
- Jamais `rm`, `pkill`, `killall`. Jamais `~/kesh/`, `~/kesh-test/`, ni le NAS ; aucune donnée réelle dans le rapport.
- Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- Commandes permises : `cat`, `sed -n`, `/usr/bin/grep`, `git grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `awk` (lecture, sans
  redirection), `git log`, `git show`, `git diff` (en lecture), `pdftotext … -`, `gh api` en `GET`.
