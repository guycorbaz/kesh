# Prompt — validation P4 de la fiche v0172-session (release 0.17.2 : #876, la session), lentille complète, Opus

Tu valides la **fiche** de la Story v0172-session de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11), release
corrective **0.17.2**, **sans migration**. **La conception a été remplacée** à la remédiation de la validation P3 (Sonnet, 0/0/7/10) :
le renouvellement anticipé à l'entrée de l'utilisateur (verrous entre onglets, module armé, suspension pendant l'import) est
abandonné. La fiche prescrit désormais : l'inactivité du jeton de renouvellement à **60 minutes par défaut** et **au moins le double**
du jeton d'accès (une valeur plus courte est portée au double, avec un avertissement, sans refus de démarrage) ; une **grâce de rotation**
de 30 s côté serveur (deux onglets) ; un **repli `GET /auth/me`** après un refus de renouvellement ; un **renouvellement silencieux** au
démarrage d'un onglet ; des **lectures de fond** qui ne renouvellent pas. Le chemin existant (401, renouvellement, rejeu) fait le reste.
La borne écrite : la session prend fin entre `N − jwt` et `N` après la dernière requête de l'utilisateur.

C'est une passe **complète**, la conception étant neuve : relis-la comme un texte neuf. Worktree `/home/gcorbaz/devel/kesh-0172b`,
branche `story/v0172-correctifs`, partie de `d7ec2ea2` (le code applicatif n'a pas bougé depuis). Langue du rapport : **français**.
Modèle : **Opus**. **Une seule lentille**, R (la règle et sa preuve) et F (les textes et ce qu'ils affirment). **Contexte frais** : refais
les relevés ; ne lis le rapport P3 (`/home/gcorbaz/devel/kesh-gate-logs/v0172s-validate-p3.md`) qu'**après** les tiens.

## À lire

1. La fiche `_bmad-output/implementation-artifacts/v0172-session.md`, **en entier**.
2. Le registre `_bmad-output/implementation-artifacts/v0172-choix-autonomes.md`, section `v0172s` (en vigueur : C-v0172s-14 à 21 ;
   C-v0172s-1 à 13 écartés).
3. L'issue, en lecture seule : `gh api repos/guycorbaz/kesh/issues/876` et `…/876/comments`.
4. Le code : `crates/kesh-api/src/config.rs` (lecture des durées, `from_fields_for_test`, tests `:2680-2730`), `routes/auth.rs` (en
   entier), `routes/setup.rs`, `middleware/auth.rs`, `auth/jwt.rs`, `main.rs` (avertissement inactivité / durée de vie, purge des
   jetons), `crates/kesh-db/src/repositories/refresh_tokens.rs`, `crates/kesh-db/src/entities/refresh_token.rs`,
   `crates/kesh-api/tests/{auth_e2e,auth_cookies_e2e,setup_admin_e2e}.rs` ; `frontend/src/lib/shared/utils/api-client.ts` et
   `api-client.test.ts`, `frontend/src/lib/app/stores/auth.svelte.ts` et son test, `frontend/src/hooks.client.ts`,
   `frontend/src/routes/login/+page.svelte`, `frontend/src/lib/features/admin-backup/{BackupScheduleBanner.svelte,backup-schedule.svelte.ts,backup-schedule.api.ts}`,
   `frontend/src/lib/shared/utils/api-health.svelte.ts`, `lib/features/admin-restore/AdminRestorePanel.svelte` ; `.env.example`,
   `docker-compose.yml`, `docker-compose.prod.yml`.
5. Le **manuel** : `docs/manual/fr/user-manual.tex` et `docs/manual/fr/admin-manual.tex` — l'inventaire du § *Manuel* de la fiche
   (relevés insensibles à la casse : 82 et 16 lignes). **Le PDF aussi**, lu aplati vers la sortie standard seulement :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem `admin-manual.pdf`).
6. Pour contexte : `CLAUDE.md` du worktree ; `docs/agents-orchestration.md` ; `kesh-gate-logs/prompts/v017-donnees-a-garder.md`.

## Axes à exercer

**La conception neuve — axe obligatoire, à traiter en premier**

- **La prémisse de la simplification** : le chemin 401 renouvelle-t-il et rejoue-t-il vraiment toute requête de l'utilisateur
  (`request` **et** `requestRaw`, téléchargements, envois de fichiers, `fetch` directs hors `api-client`) ? Le rejeu d'un `POST` est-il
  sûr partout (cherche les handlers qui rendent 401 eux-mêmes, et les corps non rejouables) ? Une saisie peut-elle encore être perdue
  avec les nouvelles durées (formulaire ouvert plus de `N − jwt` sans requête, rechargement, onglet en veille) ?
- **La borne** : refais les déroulés 60/15, 15/15 (porté à 30), 90/15, 2/1. La formule « entre `N − jwt` et `N` après la dernière
  requête » est-elle vraie ? Le manuel et le CHANGELOG prescrits la disent-ils exactement ?
- **La règle du double** : où s'applique-t-elle (lecture de l'environnement seulement, ou aussi `from_fields_for_test`) ? Interagit-elle
  avec l'avertissement existant de `main.rs` (inactivité > durée de vie) et avec les bornes [1, 1440] ? Les tests existants que la fiche
  nomme sont-ils tous ceux qui changent (cherche ce qui dépend d'une inactivité de 15 min : tests Rust, E2E, captures, `docs/`) ?
- **La grâce de rotation** : est-elle bien bornée (horloges : `revoked_at` en `NOW(3)` sous une session `+00:00`, `Utc::now()` côté
  Rust) ? Ouvre-t-elle un contournement de la détection de vol ? Le chemin `"rotation"` est-il le seul concerné ?
- **Le repli `/me`** : peut-il tenir pour réussi un renouvellement alors que la session est morte (jeton d'accès encore valable, ligne
  révoquée par un changement de mot de passe ou une déconnexion ailleurs) ? Boucle possible avec le rejeu ?
- **`hydrate()` silencieux** : la page de connexion, l'assistant (`/setup`, `/onboarding`), le canal `auth-change` entre onglets, le
  423 : un parcours change-t-il de façon non voulue ?
- **Les lectures de fond** : `background` est-il conservé au rejeu ; la lecture du montage du bandeau est-elle bien ordinaire ; une autre
  lecture périodique ou différée existe-t-elle (`setTimeout`, `$effect` qui relance) ?
- **Ce qui a été retiré** : la fiche garde-t-elle, hors § 8 *Écarté* et Change Log, une trace de la conception écartée (AC, mutation,
  tâche, manuel) ? Les findings de la P3 déclarés « sans objet » le sont-ils vraiment ?

**La règle et sa preuve**

- Chaque `fichier:ligne` de la fiche dit-il ce qu'elle lui fait dire ?
- AC-1 à AC-9 décidables ; mutations M1 à M7 tuées par le test qu'elles nomment ; une régression probable sans mutation ?
- Le décompte des modules (cinq) et la liste des fichiers : un fichier que les AC ou les Tâches font toucher manque-t-il ?
- Règles du dépôt : aucune migration (`ls crates/kesh-db/migrations/*.sql | wc -l` = 88) ; aucun mot-clé de fermeture prescrit ; gate
  complet et E2E complet au dernier commit de code ; aucune donnée réelle.

**Les textes et ce qu'ils affirment**

- **Le manuel** — axe obligatoire : chaque phrase prescrite est-elle vraie au code après correctif ? Rejoue les deux relevés insensibles
  à la casse : décomptes (82 = 75 + 7 ; 16 ; 13 au tableau et 69 hors classe pour l'administration ; 2 et 14 pour l'utilisateur) et
  classements. Le PDF dit-il la même chose que le `.tex` ?
- **La configuration livrée** : les commentaires prescrits de `.env.example` sont-ils vrais ?
- **CHANGELOG** : la puce est-elle vraie, y compris la phrase sur un `.env` recopié ?

## Ce qu'il faut rendre

Un rapport en Markdown à `/home/gcorbaz/devel/kesh-gate-logs/v0172s-validate-p4.md`, **écrit par l'outil d'écriture de fichier
(`Write`), une seule fois**. Il contient :

1. **Les findings**, `P1`, `P2`…, avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), origine (**conception neuve** ou **reprise de la
   fiche précédente**), lieu (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède. Une affirmation
   de présence ou d'absence sans sortie `grep -nF` copiée ne compte pas.
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
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `npx`, ni `make`, ni `latexmk`, ni `python3`, ni `node`, ni
  `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- **Jamais `scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun autre script du dépôt.
- Jamais `rm`, `pkill`, `killall`. Jamais `~/kesh/`, `~/kesh-test/`, ni le NAS ; aucune donnée réelle dans le rapport.
- Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- Commandes permises : `cat`, `sed -n`, `/usr/bin/grep`, `git grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `awk` (lecture, sans
  redirection), `git log`, `git show`, `git diff` (en lecture), `pdftotext … -`, `gh api` en `GET`.
