# Prompt — validation P2 de la fiche v0174-session (0.17.4 : sécurité de la session, #890, #895, #896, #898, #899), passe complète, Opus

Tu valides, en **passe complète**, la fiche de la story `v0174-session` de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit, MariaDB 10.11) et ses deux sous-fiches, **après** la remédiation de la validation P1 (Sonnet, lentilles R et F :
R 0/0/2/9, F 0/0/3/13). **#890 (P1 sécurité)** : le jeton de renouvellement est stocké en clair ; la fiche retient une
empreinte **sans migration** (16 premiers octets de SHA-256, chiffre de version forcé à `0`, mise en forme dans la colonne
`CHAR(36)` existante), calculée **par le dépôt**, et l'effacement des jetons d'avant au nettoyage du démarrage et dans la
transaction d'import. **#895** : un avertissement une minute avant la fermeture pour inactivité, « Rester connecté », sans
prise de focus, au-dessus d'un dialogue sans le fermer (élément ordinaire : arrêt du `pointerdown`, `pointer-events: auto`,
`z-index` > 50). **#896** : retrait d'un paramètre de test ignoré (85 appels). **#898, #899** : les LOW des revues de la
0.17.3, triés. Kesh **0.17.3** tient des livres réels en production : la montée, le retour à la 0.17.3 et l'import d'une
sauvegarde 0.17.3 sont des exigences.

Worktree `/home/gcorbaz/devel/kesh-0174s`, branche `story/v0174-session`, partie de `7bd4d5c0` ; fiche à `b30d8b3e`
(remédiation P1). Langue du rapport : **français**. Modèle : **Opus**. Passe **complète**, **une seule lentille** qui
couvre tout. **Contexte frais.**

## À lire

0. Les rapports de la validation P1 : `/home/gcorbaz/devel/kesh-gate-logs/v0174s-validate-p1-R.md` et
   `/home/gcorbaz/devel/kesh-gate-logs/v0174s-validate-p1-F.md`, et leur remédiation (`git show b30d8b3e`, Change Log de
   la parente).
1. La fiche parente `_bmad-output/implementation-artifacts/v0174-session.md`, **en entier**, et les sous-fiches
   `v0174-session-1.md` (serveur) et `v0174-session-2.md` (navigateur).
2. Le registre `_bmad-output/implementation-artifacts/v0174-choix-autonomes-v0174s.md` (C-v0174s-1 à 20).
3. Les issues, **en lecture seule** : `gh issue view <N> --json title,body,comments` pour 890, 895, 896, 898, 899.
4. La fiche de la 0.17.3 qui a posé la conception de la session : `_bmad-output/implementation-artifacts/v0172-session.md`
   (§ *Conception* 3 à 8, § *Acceptance Criteria*).
5. Le code (à `7bd4d5c0`, inchangé par la fiche) :
   - serveur : `crates/kesh-db/src/repositories/refresh_tokens.rs` (en entier), `entities/refresh_token.rs`,
     `repositories/password_reset_tokens.rs` (en-tête), `crates/kesh-db/Cargo.toml`, `src/backup.rs` (listes de tables),
     `migrations/20260405000001_auth_refresh_tokens.sql`, `20260406000001_refresh_tokens_revoked_reason.sql`,
     `test-schema/0001_schema_squash.sql` (table `refresh_tokens`) ; `crates/kesh-api/src/routes/auth.rs` (connexion,
     déconnexion, `refresh`, changement de mot de passe, `activity`, `new_session_deadline`), `routes/setup.rs`,
     `routes/admin.rs` (`full_import`, points 5 à 5-quater), `routes/session_settings.rs`, `middleware/auth.rs`,
     `auth/api_key.rs`, `auth/bootstrap.rs` (tests `:790-870`), `admin_backup/` (`mod.rs`, `export.rs`, `import.rs`),
     `exports/global.rs`, `config.rs` (`from_fields_for_test`), `main.rs` (nettoyage du démarrage), `lib.rs` (routes
     d'authentification et limiteurs), `crates/kesh-api/README.md` ;
   - tests : `crates/kesh-db/tests/refresh_tokens_repository.rs`, `crates/kesh-api/tests/auth_e2e.rs`,
     `auth_cookies_e2e.rs`, `session_idle_e2e.rs`, `admin_full_import_e2e.rs` (`:5140-5310`), `admin_backup_e2e.rs`,
     `setup_admin_e2e.rs`, `password_recovery_e2e.rs`, `audit_route_registry.rs`, `textes_coherents.rs`,
     `configuration_transmise.rs` ;
   - navigateur : `frontend/src/lib/app/stores/session-idle.svelte.ts` et son test, `frontend/src/routes/+layout.svelte`,
     `routes/root-layout-session.test.ts`, `frontend/src/lib/shared/utils/api-client.ts` (chemin 401, `signal`),
     `frontend/src/lib/components/ui/dialog/` et, **en lecture**, `frontend/node_modules/bits-ui/dist/` (2.16.5 :
     `bits/utilities/dismissible-layer/use-dismissable-layer.svelte.js`, `internal/body-scroll-lock.svelte.js`,
     `bits/dialog/dialog.svelte.js`, `bits/dialog/components/dialog-content.svelte`), `frontend/src/lib/shared/components/DegradedBanner.svelte`,
     `frontend/src/lib/features/imported-supplier-invoices/DocumentPreview.svelte` (aperçu), `frontend/scripts/lint-i18n-ownership.js`,
     `frontend/src/lib/shared/i18n-keys.test.ts`, `frontend/package.json` (Playwright), `crates/kesh-i18n/locales/*/messages.ftl`.
6. Les **manuels** : `docs/manual/fr/admin-manual.tex` (§ sauvegarde `:2255-2270`, § *Authentification JWT + refresh
   tokens* `:3021-3030`, § *Durée d'inactivité* `:3032-3056`) et `docs/manual/fr/user-manual.tex` (`:93-98`), et leurs PDF
   aplatis par `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem `user-manual.pdf`), **vers la
   sortie standard seulement** ; `docs/kesh-specifications.txt:2690-2706` ; `CHANGELOG.md` (section `[0.17.3]`).
7. `CLAUDE.md` du worktree (*Migration breaking policy* P1–P8, *Règle de splitting préventif*, *Review Iteration Rule*,
   *Issue Tracking Rule*, *Recette visuelle*) et `docs/agents-orchestration.md`.

## Ce que tu cherches

D'abord **la dernière remédiation** (`git diff cc1f7c2b b30d8b3e -- _bmad-output/`) : la sévérité se déplace vers ce
qu'on vient d'écrire. Chaque texte neuf est-il juste au code (ligne relue) ? A-t-il introduit une contradiction avec une
autre partie des fiches ou du registre ? Puis le reste, à neuf :

- **Les prémisses** : chaque `fichier:ligne` cité dit-il ce qu'on lui fait dire (code de `7bd4d5c0`, et bits-ui 2.16.5) ?
  Inventaire **clos** des sites qui lient une valeur de jeton à `refresh_tokens.token` (production, seed, scripts, tests).
  La prémisse corrigée (« manuel faux jusqu'à la 0.17.2, corrigé en 0.17.3 ») : `git show v0.17.2:docs/manual/fr/admin-manual.tex`.
- **L'empreinte et l'absence de migration** : contrainte de format, position 15, force, argon2 écarté ; P1–P8 réellement
  sans objet ; redémarrage de la 0.17.3 ; import d'une sauvegarde 0.17.3 dans la 0.17.4 et l'inverse.
- **Les effacements et les tests** : le prédicat ; `cleanup_removes_old_tokens` réécrit (survivantes) ; `rows_affected` ;
  l'assistant `independent_refresh_digest` (vraiment indépendant de `token_digest`) ; AC-5 (c) vert avant correctif et
  M14 ; chaque mutation M1 à M17 rougit-elle le test nommé ? Un test passe-t-il encore à vide ?
- **S-E5, trois sites** : l'ordre prescrit pour `refresh`, `setup`, `change_password` est-il possible sans changer un
  comportement (tolérance de ± 2 s, audit, rollback) ? Le CHANGELOG ne promet-il que la lecture de N ?
- **L'avertissement** : `warningRemainingMs` placé avant `:292` ; `stayActive` ; composant à propriétés et condition
  `typeof` ; douzième écouteur (`pointerout`) et le garde « onze » ; N = 1 ; onglet en arrière-plan ; aucune prise de
  focus ; la phrase annoncée.
- **Au-dessus d'un dialogue (C-v0174s-13 récrit)** : relis bits-ui. L'arrêt de propagation d'un `pointerdown` natif sur
  la racine de l'avis empêche-t-il bien la fermeture (capture/bouillonnement, `#isAnyEventIntercepted`, `focusin`
  extérieur et `onFocusOutside`, `click` tactile) ? `pointer-events: auto` sous `ScrollLock` ? `z-[60]` contre la
  superposition `z-50` montée par portail ? La délégation d'événements de Svelte 5 est-elle bien contournée ? Le piège à
  focus (`FocusScope`) ramène-t-il le focus sans dommage après le clic ? Échap ?
- **E2E sous `page.clock`** (AC-4) : `GET` puis `PUT`, remise dans `afterAll`, `waitForResponse`, dialogue à champ
  atteignable avec la démonstration, aperçu PDF par la boîte d'entrée.
- **Le manuel** — axe obligatoire, au `.tex` **et** au PDF aplati : les trois sites « jetons de session », la
  sous-section « Passer à la 0.17.4 » (parts des deux sous-fiches, rédaction commune à l'intégration), le paragraphe
  *Mise à jour* qui y renvoie, § *Dépannage*, manuel utilisateur (seule la phrase en gras remplacée), N = 1, les motifs
  **locaux** de contrôle du PDF (décidables ? réussiraient-ils avant correctif ?). `docs/kesh-specifications.txt`.
- **i18n** : trois clés, quatre locales, gardes Svelte nommées (`i18n-entrees-a-variables.test.ts` et la variable
  `remaining`), vocabulaire (« beendet »), orthographe suisse.
- **Le triage des LOW et les issues demandées** (parente, points 1 à 6) : justes et suffisants ? Un « — » cache-t-il un
  défaut ? Décompte de la table.
- **Les décomptes** (neuf appels, six fonctions, 85 appels, M1 à M17, W1 à W15, C-v0174s-1 à 20) : recompte-les.
- **Les règles du dépôt** : aucun mot-clé de fermeture prescrit dans un message de commit ; aucune donnée réelle ;
  historique non réécrit (les entrées datées sont annotées, non récrites).

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0174s-validate-p2.md`. Il
contient :

1. **Les findings**, numérotés `P2-1`, `P2-2`…, chacun marqué **« de la remédiation »** s'il naît du commit `b30d8b3e`, avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu,
   **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède proposé. Une affirmation de présence ou
   d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché — **le manuel et son PDF
   compris**. Un « 0 finding » sans cette liste ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au
  rapport, il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `npx`, ni `make`, ni `latexmk`, ni `python3`, ni
  `node`, ni `sqlx`, ni `docker`, ni `curl`, ni `sha256sum`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`,
  `scripts/install-hooks.sh`, ni aucun autre script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/`, ni `/home/gcorbaz/travail/` ; aucune donnée réelle dans le rapport.
- Rien dans `/home/gcorbaz/devel/kesh` : lecture seule.
- Commandes permises : `cat`, `sed -n`, `/usr/bin/grep` (dans ce shell, `grep` est une fonction vers `ugrep`), `git
  grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`, `git diff` (en lecture), `pdftotext … -` vers
  la sortie standard, `gh issue view`.
