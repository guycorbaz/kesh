# Prompt — validation P1 de la fiche v0174-session (0.17.4 : sécurité de la session, #890, #895, #896, #898, #899), deux lentilles, Sonnet

Tu valides la **fiche** de la story `v0174-session` de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11)
et ses deux sous-fiches. **#890 (P1 sécurité)** : le jeton de renouvellement de session est stocké en clair dans
`refresh_tokens.token`. La fiche retient une empreinte **sans migration** — les 16 premiers octets de SHA-256, chiffre de
version forcé à `0`, mis en forme dans la colonne `CHAR(36)` existante —, calculée **par le dépôt** (les appelants
passent le jeton clair), et l'effacement des jetons d'avant au nettoyage du démarrage et dans la transaction d'import.
**#895** : un avertissement une minute avant la fermeture pour inactivité, avec « Rester connecté », sans prise de focus,
au-dessus d'un dialogue sans le fermer. **#896** : retrait d'un paramètre de test ignoré (85 appels). **#898, #899** : les
LOW des revues de la 0.17.3, triés un par un. Kesh **0.17.3** tient des livres réels en production : la montée, le
retour à la 0.17.3 et l'import d'une sauvegarde 0.17.3 sont des exigences.

Worktree `/home/gcorbaz/devel/kesh-0174s`, branche `story/v0174-session`, partie de `7bd4d5c0` ; fiche au commit
`cc1f7c2b`. Langue du rapport : **français**. Modèle : **Sonnet**. Passe **complète** (première passe de validation).
**Contexte frais**.

## À lire

1. La fiche parente `_bmad-output/implementation-artifacts/v0174-session.md`, **en entier**, et les sous-fiches
   `v0174-session-1.md` (serveur) et `v0174-session-2.md` (navigateur).
2. Le registre `_bmad-output/implementation-artifacts/v0174-choix-autonomes-v0174s.md` (C-v0174s-1 à 16).
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
     `frontend/src/lib/components/ui/dialog/`, `frontend/src/lib/shared/components/DegradedBanner.svelte`,
     `frontend/src/lib/features/imported-supplier-invoices/DocumentPreview.svelte` (aperçu), `frontend/scripts/lint-i18n-ownership.js`,
     `frontend/src/lib/shared/i18n-keys.test.ts`, `frontend/package.json` (Playwright), `crates/kesh-i18n/locales/*/messages.ftl`.
6. Les **manuels** : `docs/manual/fr/admin-manual.tex` (§ sauvegarde `:2255-2270`, § *Authentification JWT + refresh
   tokens* `:3021-3030`, § *Durée d'inactivité* `:3032-3056`) et `docs/manual/fr/user-manual.tex` (`:93-98`), et leurs PDF
   aplatis par `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem `user-manual.pdf`), **vers la
   sortie standard seulement** ; `docs/kesh-specifications.txt:2690-2706` ; `CHANGELOG.md` (section `[0.17.3]`).
7. `CLAUDE.md` du worktree (*Migration breaking policy* P1–P8, *Règle de splitting préventif*, *Review Iteration Rule*,
   *Issue Tracking Rule*, *Recette visuelle*) et `docs/agents-orchestration.md`.

## Deux lentilles, en contexte frais chacune

### Lentille R — la règle, les données et leur preuve (surtout `v0174-session-1` et la parente)

- **Les prémisses** : chaque `fichier:ligne` cité par la fiche et le registre dit-il ce qu'on lui fait dire ? En
  particulier : toutes les recherches et écritures de `refresh_tokens.token` (inventaire **clos** : cherche toi-même tout
  site qui lie une valeur de jeton à cette colonne, production, seed, scripts, frontend, tests) ; la prémisse corrigée sur
  le manuel (« tel quel » depuis la 0.17.2) ; l'export de souveraineté qui exclut déjà la table.
- **L'empreinte (C-v0174s-2, C-v0174s-5)** : la valeur prescrite satisfait-elle **toujours**
  `chk_refresh_tokens_token_format` ? Le chiffre forcé est-il bien à la position 15 de la chaîne mise en forme, et un UUID
  v4 y porte-t-il toujours `4` ? Le vecteur de l'AC-1 est-il juste (refais le calcul **de tête ou à la lecture**, sans
  exécuter de commande : sinon, dis-le non exercé) ? L'argument de force (124 bits pour 122) tient-il ? Le refus d'argon2,
  de sel et de poivre est-il justifié ?
- **Sans migration** : est-ce vrai au sens de P1 à P8 (aucun fichier sous `migrations/`, `test-schema/`,
  `migrations.sha384`, audit d'idempotence) ? La 0.17.3 redémarre-t-elle réellement sur une base passée par la 0.17.4
  (`_kesh_version`, `check_downgrade_protection`, `sqlx`) ? Une sauvegarde 0.17.3 s'importe-t-elle (contrôle des
  inventaires de tables, des colonnes, de la version du manifeste) ? Et une sauvegarde **0.17.4** dans une 0.17.3 ?
- **Le hachage dans le dépôt (C-v0174s-3)** : une fonction du dépôt qui reçoit un jeton est-elle oubliée ? Un appelant
  lit-il le champ `token` de l'entité (renommé) ? `bootstrap.rs`, `kesh-seed`, la réinitialisation de démonstration,
  `post_restore.rs` touchent-ils la colonne ? Le `Cargo.lock` reste-t-il sans crate neuve ?
- **Les effacements (C-v0174s-6)** : le prédicat efface-t-il une ligne qui ne devrait pas l'être (une empreinte) ou
  manque-t-il une forme d'avant ? Le placement dans `delete_expired_and_revoked` change-t-il le sens d'un test existant
  (`cleanup_removes_old_tokens`) ? Dans `full_import`, l'effacement est-il bien après la garde de comptage et avant le
  refus « aucun compte Admin », et l'annulation de la transaction le défait-elle ? La fixture 0.14.0 reste-t-elle
  importable ?
- **Les tests qui changent** : la liste est-elle complète (cherche le symptôme : toute requête brute sur
  `refresh_tokens.token`, tout test qui insère une ligne à la main puis présente sa valeur, tout test qui compare
  `.token`) ? `create_with_invalid_token_format_fails_check_constraint` est-il le seul test que le hachage rend muet ?
- **Les mutations M1 à M12** : chacune rougit-elle le test annoncé ? M1 (« stockage en clair ») est-elle tuée par un test
  qui **ne passe pas** par `token_digest` ? Un test qui passerait à vide ?
- **#896 (C-v0174s-8)** : 85 appels dans 80 fichiers, recompte-les ; le contrôle de la bonne ligne retirée est-il
  décidable ?
- **Le triage des LOW** : pour chaque ligne de la table de la parente, la décision est-elle juste au code (relis le lieu
  cité) ? Un « — » cache-t-il un défaut réel qui mériterait une issue ? Le décompte de la table est-il juste ?
- **Le découpage** (C-v0174s-1) : chaque AC dans une seule sous-fiche ; la 2 ne dépend pas du code de la 1.

### Lentille F — l'écran, les textes et ce qu'ils affirment (surtout `v0174-session-2` et les textes)

- **Le manuel** — axe obligatoire, au `.tex` **et** au PDF aplati : chaque phrase devenue fausse par #890 ou #895
  est-elle nommée (« tel quel », « jetons de session » dans la sauvegarde, « CR \#895 », « au moins une fois par durée
  réglée », le paragraphe *Mise à jour*) ? Une phrase manque-t-elle (glossaire, § *Dépannage*, manuel utilisateur
  § déconnexion, `kesh-specifications.txt`) ? Les textes prescrits disent-ils vrai au regard de la conception (une
  reconnexion, retour à la 0.17.3 sans sauvegarde, pas d'avis sans `localStorage`) ?
- **L'avertissement** : le délai `min(60 s, N/2)` ; l'état `warningRemainingMs` calculé par le tic ; `stayActive` (contrôle
  préalable de l'échéance, sans régulation) ; l'effacement par une activité d'un autre onglet. Un chemin laisse-t-il
  l'avis affiché après la fermeture, ou l'efface-t-il sans activité ? Le comportement d'un onglet en arrière-plan ?
- **Le focus et l'accessibilité (C-v0174s-11)** : la prémisse — un focus ramené d'un `<iframe>` vers la page fait
  recevoir à la fenêtre un `focus` compté comme activité — est-elle juste au code du module ? Une primitive bits-ui
  prendrait-elle le focus malgré la prescription ? `role="alert"` statique et compte à rebours `aria-hidden` : un lecteur
  d'écran a-t-il de quoi agir ?
- **Au-dessus d'un dialogue (C-v0174s-13)** : la prémisse (fermeture sur geste extérieur) est-elle juste pour les
  dialogues de Kesh ? La piste prescrite est-elle plausible avec bits-ui `^2.16` (lis `node_modules/bits-ui` si présent
  dans le worktree, en lecture) ? Une superposition modale (`pointer-events`) empêcherait-elle le clic ?
- **E2E sous `page.clock` (AC-4)** : la spec est-elle juste quand l'horloge de la page avance et non celle du serveur
  (battement, renouvellement, `idle_exp`, `exp` du jeton d'accès) ? Le cas (c) a-t-il un dialogue à saisie atteignable
  avec la démonstration ?
- **i18n (AC-6)** : clés aux quatre catalogues, replis égaux au fr-CH (G9), propriété des clés (`lint-i18n-ownership`
  pour `lib/shared/components`), compteurs de `i18n-keys.test.ts`, traductions (orthographe suisse).
- **Les LOW navigateur** (§ *Conception* 3 de la 2) : chaque remède est-il juste au code (N-E5 `removeItem` et le repli
  en mémoire ; Rec-e1 `pointerout` et `relatedTarget`) ? Un remède introduit-il un défaut (un `pointerout` émis entre deux
  éléments de la page) ?
- **CHANGELOG, README, captures** : la section `[0.17.4]` dit-elle la reconnexion, l'absence de migration, le retour à
  la 0.17.3 ? Les captures prescrites couvrent-elles chaque état visible ?
- **Les règles du dépôt** : aucun mot-clé de fermeture prescrit dans un message de commit ; aucune donnée réelle ;
  défaut voisin renvoyé à une issue s'il n'est pas mécanique ; issues demandées par la parente justes et suffisantes.
- **Les décomptes et numéros de ligne** : recompte-les.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, au chemin de ta lentille :
`/home/gcorbaz/devel/kesh-gate-logs/v0174s-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/v0174s-validate-p1-F.md` (lentille F). Il contient :

1. **Les findings**, numérotés `R1`… ou `F1`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu,
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
