# Prompt — revue de code P1 de la Story v0172-session, **lot S (serveur)**, trois lentilles, Sonnet

Tu relis le **lot S** (serveur) de la **Story v0172-session** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB
10.11), version **0.17.3** : **la durée d'inactivité de la session, réglée dans l'interface, exacte, remise à zéro par
l'activité** (#876, CR #891). Le lot S porte : une migration `20261104000001_installation_settings.sql` (table **système**
`_kesh_installation_settings`, DDL seul, sans ligne) ; le dépôt `installation_settings` (défaut 60, bornes 1–1440, verrou
optimiste où la version 0 signifie « aucune ligne », audit `admin.session_settings_updated`, valeur inchangée sans
écriture) ; `GET`/`PUT /api/v1/admin/session-settings` ; `jwt::SessionClaims` (`idle_exp`), `encode_session`,
`decode_session`, `jwt::encode` sous `#[cfg(test)]` ; le contrôle strict d'`idle_exp` au middleware ; `refresh_tokens.
expires_at` devenu l'**échéance d'inactivité** (connexion, assistant, changement de mot de passe à `now + N` ;
renouvellement qui **recopie** ; grâce de rotation de 30 s) ; le battement `POST /api/v1/auth/activity` (âge borné à N,
jeton réémis au **même `exp`**, aucun cookie si `exp` est passé) ; les champs `idleTimeoutMinutes`/`idleExpiresIn` de la
connexion, du renouvellement et de `/me` ; la **révocation de toutes les sessions** dans la transaction de la restauration
complète ; l'ancienne variable de durée d'inactivité **n'est plus lue** ; les registres et gardes recomptés.

Kesh tient des livres réels en production : **une migration** est ajoutée (non breaking) ; rien ne doit réécrire une donnée
existante hors des jetons de renouvellement.

Worktree `/home/gcorbaz/devel/kesh-0172s`, branche `story/v0172-session`. Le diff à relire, **aplati**, restreint au lot S :
`git -C /home/gcorbaz/devel/kesh-0172s diff 1e50fd03 1dc94968` (`1e50fd03` = étape 0, après la fusion de la v0.17.2 ;
`1dc94968` = commit du lot S). Les commits suivants (lot N, comptes rendus) sont hors de ta lentille, sauf pour relire le
manuel. Langue du rapport : français. Modèle : **Sonnet**. Passe **complète** (première passe). Contexte frais.

⚠️ **Les propriétés centrales.**
1. **Une seule échéance, exacte** : la ligne `refresh_tokens` porte `dernière activité + N` ; le jeton d'accès en porte
   la copie `idle_exp` (secondes, arrondi vers le bas, sur une échéance tronquée à la milliseconde) ; le middleware la
   refuse **sans** la tolérance de 60 s, quel que soit `exp` ; le renouvellement **recopie** ; seul le battement prolonge.
2. **Le battement ne prolonge rien d'autre** : même `sub`, `role`, `company_id`, **même `exp`** ; aucune rotation ; l'âge
   borné à `[0, N × 60 000]` ; `set_idle_deadline` exige le `user_id` du jeton présenté.
3. **Tout rejet de corps rend 400 `VALIDATION_ERROR`** (extracteurs `impl_validation_extractor!`), jamais 422.
4. **La restauration complète révoque dans sa transaction**, après le rejeu et avant le refus « aucun Admin » : un refus
   annule la révocation ; le réglage de la destination n'est pas restauré.
5. **L'ancienne variable n'est plus nommée** dans le corpus de `configuration_transmise.rs` (code de production, `.env.
   example`, compose, quatre catalogues, manuels français).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche `_bmad-output/implementation-artifacts/
v0172-session.md` — *Contrainte*, *Modules*, *Ce que le code dit*, *Conception retenue* § 1 à 4, 8, 9, 12, **AC-1 à AC-5,
AC-10, AC-11**, *Tests et mutations* (M1 à M8 bis, M15 à M17, M22), *Tâches* (lot S, *Registres et compteurs*),
*Configuration livrée*, *Manuel* (sites gardés par des tests Rust), *Dev Notes*, le **Dev Agent Record** et le **Change
Log** ; puis le registre `v0172-choix-autonomes.md`, section `v0172s`, en vigueur **C-v0172s-14, 17, 23, 24, 26, 28 à
58** (C-53 à 58 : choix du développement, dont deux **écarts à la fiche** : C-54, la garde `configuration_transmise.rs`
change de deux nombres ; C-55, deux tests « n'ajoute aucune table » appliqués jusqu'à leur migration incluse). Le Dev
Agent Record déclare des rouges, des gates, des mutations, un E2E : **ne les prends pas pour argent comptant**, recoupe-les
avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/v0172s/` : `rouges.log` (et
`rouges-S.log`), `cible-S-*.log`, `mut/S-resume.log` et `mut/M*.log` (le script `scripts/mut-v0172s-S.py` est à **lire**,
jamais à exécuter), `gate-backend-*.log`, `build-*.log`, `regen-squash.log`, `e2e-r*.log`, `recette-r*.log` ; la recette
`scripts/recette-v0172s.mjs` et `scripts/e2e-v0172s.sh` sont à **lire** seulement. Captures (thème clair) sous
`/home/gcorbaz/devel/kesh-gate-logs/v0172s/captures/` : `v0172s-01` à `v0172s-06`. La validation R5 de la fiche :
`/home/gcorbaz/devel/kesh-gate-logs/v0172s-validate-r5.md`.

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
  ni `scripts/regen-test-schema.sh`, ni **aucun script** du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`,
  **`cargo`**, **`docker`**, `mysql`/`mariadb`, `sqlx`, **`curl`**, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep` (dans ce shell, `grep` est une
  fonction vers `ugrep`), `sed -n`, `awk` (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`),
  `wc`, `sort`, `uniq`, `cmp`, `sha384sum`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⛔ Sur le PDF aplati, `grep -oF … | wc -l` ou `grep -cF` sur des **motifs fixes**, sous `timeout 60` ; l'apostrophe
  s'y rend `’` : compte les deux graphies.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **Le manuel (`.tex` et PDF aplati) contre le code du lot S** : `docs/manual/fr/admin-manual.tex` — le tableau des
   variables (`KESH_JWT_EXPIRY_MINUTES`, `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS`, la note sous le tableau), la note de la
   0.17.2 (« La session, non corrigée dans cette version »), le compte du registre des routes (« 136 des 144 », « 136 + 5
   + 3 = 144 »), § *Authentification JWT + refresh tokens*, § *Durée d'inactivité de la session*
   (`\label{sec:duree-inactivite}`), la restauration complète (« ne peuvent plus être renouvelées »), le dépannage des 401,
   le glossaire ; `user-manual.tex` (la session, la récupération par courriel, la déconnexion) ; et leurs **PDF** (le
   PDF est régénéré au lot N). Chaque affirmation est-elle vraie au code (« à la seconde près, sauf quatre écarts », « le
   serveur refuse … quelle que soit la durée du jeton d'accès », « la mise à jour ne déconnecte personne », « une
   restauration … ne le restaure pas ») ?
2. **Les témoins et les mutations** : T1 à T5 (AC-1 à AC-5) — quel test couvre chaque propriété centrale, peut-il passer
   **à vide** (un 401 rendu par une autre règle, un jeton forgé sans `idle_exp`, une ligne déjà expirée) ; chaque mutation
   nommée est-elle tuée **par le test attendu** (`mut/M*.log`) ? Une mutation plausible non jouée survivrait-elle
   (`trunc_subsecs` retiré ; `<=` au lieu de `>=` au middleware ; `idle_exp` du renouvellement calculé sur `now + N` ;
   `set_idle_deadline` sans `expires_at > ?` ; la révocation placée après l'audit ; `installation_settings::get` qui écrit
   une ligne) ?
3. **Les gardes et compteurs, recomptés à la source aux deux bornes** (`1e50fd03` → `1dc94968`) : `audit_route_registry.rs`
   (`LIB_ROUTES` 142 → 144, `traced` 135 → 136, `exempt` 4 → 5, `sans_ecriture` 117 → 119, 145 → 147 deux fois),
   `admin_pat_denied_e2e.rs` (50 → 52 constructeurs, `(14, 36)` → `(15, 37)`, HEAD discriminants 12 → 13),
   `configuration_transmise.rs` (31 → 30, 41 → 40), `textes_coherents.rs` (G9 : 7 → 8 entrées),
   `migrations_upgrade_path.rs` (89, `total - 55`, frontière 34), `docs/migrations-idempotence-audit.md` (89 = 11 + 78 +
   0, aux deux sites du total), `migrations.sha384` (le checksum de la migration neuve), le squash régénéré.
4. **Grep du symptôme, la valeur, sur tout le worktree** : `git grep -nE 'refresh_inactivity|REFRESH_INACTIVITY' -- crates
   frontend scripts docs/manual .env.example docker-compose.yml docker-compose.prod.yml` (attendu : le paramètre ignoré de
   `from_fields_for_test` et `crates/kesh-api/README.md`) ; `git grep -n "jwt::encode(" -- crates/kesh-api/src` (attendu :
   des lignes sous `#[cfg(test)]` seulement) ; `git grep -nE '"(logout|rotation|password_change|admin_disable|theft_detected)"' -- crates/kesh-api/src` ;
   `git grep -nF '1440' -- crates/kesh-i18n/locales` ; `git grep -nF 'Utc::now() + state.config' -- crates/kesh-api/src`.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne : `crates/kesh-db/src/repositories/installation_settings.rs` (le `FOR UPDATE` sur une table vide, la
course de deux `INSERT` à version 0, le rollback de chaque branche), `refresh_tokens.rs` (`set_idle_deadline`,
`was_rotated_within` et l'horloge de la base, `revoke_all_in_tx`), `crates/kesh-api/src/auth/jwt.rs` (`#[serde(flatten)]`
et `required_spec_claims`), `middleware/auth.rs` (le `let`-chain), `routes/auth.rs` (`new_session_deadline`, le
renouvellement, la grâce placée **avant** la détection de vol, `activity` : l'ordre `ensure_not_pat` → cookie → N →
écriture → réémission, les conversions `u64`/`i64`, `exp − now ≤ 0`), `routes/session_settings.rs`, `routes/setup.rs`,
`routes/admin.rs` (le site de la révocation), `config.rs` et `main.rs` (ce qui est retiré), la migration. Cherche : un
chemin de production qui émet un jeton sans `idle_exp` ; un cas où le battement prolonge une ligne d'un autre utilisateur
ou une ligne révoquée ; un débordement ; un 500 là où un 400/401 est dû ; une écriture hors transaction.

## Lentille E — Edge (chaque branche, chaque borne)

Pour chaque fonction du lot S, chaque branche et chaque borne : N = 1 et 1440 ; `idleMs` = 0, `N × 60 000`, au-delà,
`u64::MAX` ; un jeton présenté dans la tolérance (`exp` passé de 30 s) ; `idle_exp` égale à `now` ; une ligne
`expires_at = now` ; deux battements simultanés ; un battement qui croise un renouvellement (la ligne neuve recopie
l'échéance d'avant) ; un renouvellement présenté 29 s et 31 s après la rotation ; une restauration d'une sauvegarde sans
ligne `refresh_tokens` ; un `PUT` à version 0 quand une ligne existe ; un `PUT` concurrent ; le passage d'une 0.17.2
(jetons sans `idle_exp`) à la 0.17.3 ; un `KESH_JWT_EXPIRY_MINUTES` de 1440. Chaque cas : comportement au code, test qui
le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère d'AC-1 à AC-5 et la part serveur d'AC-10 et d'AC-11 : livré, testé, au lieu prescrit ? Les écarts déclarés
(C-v0172s-53 à 58) sont-ils justifiés et complets ? La fiche prescrivait AC-2 « vert sans modification » : l'écart C-54
est-il le seul ? *Registres et compteurs* : chaque nombre conforme ? *Configuration livrée* : `.env.example`, les deux
compose, `crates/kesh-api/README.md` ? Le CHANGELOG `[0.17.3] — Non publié` dit-il vrai pour le serveur (migration,
retour par la sauvegarde, variable ignorée, contournement à retirer, restauration qui révoque) ? Le Dev Agent Record
n'affirme-t-il que ce qui a tourné (chiffres recoupés aux journaux) ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/v0172s-review-p1-S-B.md`, `…-S-E.md`, `…-S-A.md`.
Chaque rapport : findings numérotés (`S-B1`, `S-E1`, `S-A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu
(`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** (une
passe qui ne la donne pas ne compte pas) ; le décompte final **recompté depuis la liste**.
