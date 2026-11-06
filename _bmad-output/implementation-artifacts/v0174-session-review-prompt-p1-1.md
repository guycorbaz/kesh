# Prompt — revue de code P1 de la Story v0174-session, **sous-fiche 1 (serveur)**, trois lentilles, Sonnet

Tu relis la **sous-fiche 1** (serveur et textes) de la **Story v0174-session** de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit 5, MariaDB 10.11), version **0.17.4** : **le jeton de renouvellement n'est plus stocké qu'en empreinte** (#890,
P1 sécurité, sans migration), le paramètre ignoré de `Config::from_fields_for_test` retiré (#896), et les LOW serveur des
revues de la 0.17.3 (#898, #899). Elle porte : `crates/kesh-db/src/repositories/refresh_tokens.rs` (`token_digest`,
`LEGACY_CLEAR_TOKEN_PREDICATE`, liaisons de l'empreinte dans six fonctions, `delete_clear_tokens_in_tx`, effacement des
jetons clairs au démarrage) ; l'entité `entities/refresh_token.rs` (champ `token_digest`, `#[sqlx(rename = "token")]`) ;
`crates/kesh-db/Cargo.toml` (`sha2`) ; `routes/admin.rs` (effacement à l'import, point 5-quinquies) ; `main.rs` (message) ;
`routes/auth.rs` (lecture de N avant la rotation et avant l'écriture du mot de passe, `<=` au renouvellement, commentaires) ;
`routes/setup.rs` (lecture de N avant la section critique) ; `routes/session_settings.rs` (nom d'opération) ;
`middleware/auth.rs`, `auth/api_key.rs`, `admin_backup/mod.rs` (commentaires) ; `config.rs` et 80 fichiers de tests (#896) ;
les tests neufs et adaptés ; le manuel d'administration (`.tex` et PDF), le CHANGELOG `[0.17.4] — Non publié`, le README
de `kesh-api`, `docs/kesh-specifications.txt`, deux annotations de `v0172-session.md`.

Worktree `/home/gcorbaz/devel/kesh-0174s`, branche `story/v0174-session`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-0174s diff 5873112e 76f5007d -- crates docs CHANGELOG.md _bmad-output/implementation-artifacts/v0172-session.md`
(`5873112e` = dernière validation de la fiche ; `76f5007d` = dernier commit de la sous-fiche 1 ; les commits suivants
sont la sous-fiche 2, revue à part). Langue du rapport : français. Modèle : **Sonnet**. Passe **complète** (première passe).
Contexte frais.

⚠️ **Les propriétés centrales.**
1. **Aucune valeur claire n'est jamais liée** : chaque fonction du dépôt qui reçoit un jeton lie `token_digest(valeur)` ;
   aucun appelant de `kesh-api` ne hache lui-même ; le cookie et le corps de `RefreshResponse` restent le jeton clair.
2. **L'empreinte tient dans la colonne existante** (`CHAR(36)`, `chk_refresh_tokens_token_format`) et ne ressemble jamais à
   un UUID v4 (`0` à la position 15) ; **aucune migration** (89).
3. **Les jetons d'avant sont invalidés, puis effacés** — au démarrage (`delete_expired_and_revoked`) et dans la
   transaction de la restauration complète, **avant** `revoke_all_in_tx` (un refus ultérieur annule l'effacement).
4. **Le retour à la 0.17.3 exige le vidage de `refresh_tokens`** : sinon une empreinte présentée à la 0.17.3 ouvre une
   session ; le manuel et le CHANGELOG le disent, la recette (b) l'éprouve.
5. **#896 ne change le sens d'aucun test** : le septième argument retiré était bien `_refresh_inactivity_ignored`.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la parente `_bmad-output/implementation-artifacts/
v0174-session.md` (§ #890, § *Triage des LOW*, § *Textes*), la sous-fiche `v0174-session-1.md` en entier (Conception,
AC-1 à AC-10, *Tests existants qui changent*, *Tests et mutations* M1 à M17, *Recette*, *Manuel*, *README*, *CHANGELOG*,
**Change Log** et **Dev Agent Record**) ; le registre `v0174-choix-autonomes-v0174s.md` (C-v0174s-2 à 9, 17, 18, 19). Ne
prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous
`/home/gcorbaz/devel/kesh-gate-logs/v0174s/` : `rouges.log`, `cible-1.log`, `cible-2.log`, `896-avant.txt`,
`896-controle.txt`, `mut/campagne-S.log` et `mut/M*.log` (les scripts `scripts/mut-v0174s-S.py`, `scripts/check-896.py`,
`scripts/rejeu-sans-ecriture.py`, `scripts/recette-1-v0174s.sh` sont à **lire**, jamais à exécuter), `gate-test-fast.log`
(et `gate-test-fast-r1-mauvaise-base.log`, le run invalide, déclaré), `gate-build.log`, `e2e-r*/playwright.log`,
`manuels-1.log`, `manuels-2.log`, `recette-1.log`. Captures : sans objet pour cette sous-fiche (aucun contrôle visible).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle
  part ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`,
  `bash -c`/`sh -c`, `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun
  script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `curl`, `sha256sum` sur un
  fichier du dépôt n'est pas utile, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `printf '%s' '<valeur>' | sha256sum` (pour recalculer le vecteur de l'AC-1),
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⛔ Sur le PDF aplati, `grep -oF … | wc -l` ou `grep -cF` sur des motifs fixes, sous `timeout 60` ; l'apostrophe s'y rend
  `’` : compte les deux graphies.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **Le manuel d'administration (`.tex` et PDF aplati) contre le code** : § *Authentification JWT + refresh tokens*
   (`\label{sec:auth-jwt}`), les trois sites « jetons de session » (répétition de restauration, dumps du NAS, § du
   `.keshbackup`), l'écart (a) de § *Durée d'inactivité*, le paragraphe *Mise à jour*, le *Dépannage* des 401, et la
   sous-section neuve **« Passer à la 0.17.4 »** (`\label{sec:maj-0-17-4}` : aucune migration, les deux commandes du
   `lstlisting` contre `docker-compose.yml` — service `mariadb`, `kesh-api`, `MARIADB_*` —, la variante
   `docker-compose.prod.yml`, la reconnexion unique, « effacés au premier démarrage comme à l'import ») ; et le **PDF**
   (`docs/manual/fr/admin-manual.pdf`) : les motifs de la table de la sous-fiche (§ *Manuel*), sous leur forme réelle.
   Chaque phrase est-elle vraie au code de `refresh_tokens.rs`, `admin.rs`, `main.rs`, `routes/auth.rs` ?
2. **Les témoins et les mutations** : AC-1 à AC-9 — quel test couvre chaque cas ; peut-il passer **à vide** (un
   `UPDATE … WHERE token = ?` lié à la valeur claire qui touche 0 ligne ; un 401 « jeton inconnu » qui se confond avec le
   401 attendu ; une fixture d'archive qui ne porte pas les lignes claires) ; chaque mutation M1 à M17 est-elle tuée **par
   le test attendu** (`mut/M*.log`), M12 (équivalente) déclarée comme telle ? Mutations plausibles non jouées à proposer
   (par exemple : `delete_clear_tokens_in_tx` placé **après** le refus « aucun compte Admin » ; le prédicat appliqué à
   `revoke_all_in_tx` ; `independent_refresh_digest` qui forcerait un autre chiffre que le 13ᵉ).
3. **Les contrats** : `configuration_transmise.rs`, `audit_route_registry.rs` (le point (vi) « vingt-trois », recompté
   depuis la table et le corps des routes — vérifie le recompte toi-même), `textes_coherents.rs`, `test_schema_guard.rs`,
   `migrations_upgrade_path.rs`, `migrations_fresh_install.rs` ; `git diff --stat 7bd4d5c0 -- crates/kesh-db/migrations
   crates/kesh-db/test-schema docs/migrations-idempotence-audit.md` vide ; `Cargo.lock` : seule la liste de dépendances
   de `kesh-db` gagne `sha2`.
4. **Grep du symptôme, la valeur** : `git grep -nE "\.bind\(&?(new\.)?token\)" -- crates/kesh-db/src` (aucune liaison
   claire restante) ; `git grep -nF "refresh_tokens" -- crates/kesh-api/src` (aucun SQL brut de l'application sur la
   table hors du dépôt) ; `git grep -nE "jetons? de session" -- docs/manual/fr/admin-manual.tex` ;
   `git grep -nE "tel quel|non haché|en clair" -- docs/manual/fr/admin-manual.tex CHANGELOG.md crates/kesh-api/README.md`
   (trié) ; `git grep -n "from_fields_for_test(" -- crates | wc -l` contre 86 ; `git grep -nF "update_session_idle_minutes" -- crates`.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne `refresh_tokens.rs` (l'empreinte : 16 octets, hexadécimal, l'index forcé, la mise en forme ; chaque
liaison ; les deux effacements et leur `format!`), l'entité, `admin.rs` autour du point 5-quinquies, `routes/auth.rs`
(`refresh` : l'ordre des étapes 5 à 9, la lecture de N déplacée ; `change_password` : l'étape 2-bis ; `activity` : la doc),
`setup.rs` (l'étape 3-bis et le chemin 410), `main.rs`. Cherche : une liaison claire oubliée ; un effacement qui touche une
empreinte ; un ordre d'étapes qui laisse un état partiel ; un `format!` SQL qui accepterait une valeur externe ; une
régression du chemin 410 de l'assistant (la lecture de N avant le refus).

## Lentille E — Edge (chaque branche, chaque borne)

Un jeton présenté par le **cookie** et par le **corps** ; un jeton clair de la 0.17.3 (forme UUID v4) par chacune des
routes (`refresh`, `activity`, `logout`, `password`) ; une valeur qui n'est pas un UUID (`create("not-a-uuid")`) ; une
empreinte présentée **comme jeton** à la 0.17.4 (hachée deux fois : rien ne doit s'ouvrir) ; un import d'une sauvegarde
0.14.0 (format 1, fixture), d'une 0.17.3 (jetons clairs), d'une 0.17.4 (empreintes) ; un import refusé après l'effacement ;
le nettoyage du démarrage en échec (non fatal) ; la grâce de rotation à 29 s et 31 s ; `expires_at == now` au
renouvellement et `idle_exp == now` au middleware. Chaque cas : comportement au code, test qui le couvre (ou absence,
prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC-1 à AC-10 : livré, testé, au lieu prescrit ? Les rouges déclarés sont-ils réels (`rouges.log`) ?
Le contrôle 85/85 de #896 est-il établi par `896-controle.txt` et son script ? Les écarts déclarés au Change Log (Cargo.lock
au commit de `kesh-api`, AC-5 (c) en cas séparé, l'emplacement de l'annotation S2-A3, la sonde `.token`, l'ajout des tests
neufs d'`auth_e2e.rs` par un `cat >>` et non par l'outil Edit) sont-ils exacts et suffisants ? La recette (a), (b), (c) :
chaque volet joué, horodaté, au bon résultat (`recette-1.log`) ? Le CHANGELOG, le README, les spécifications : conformes
à la sous-fiche ? Le Dev Agent Record n'affirme-t-il que ce qui a tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/v0174s-review-p1-1-B.md`, `…-1-E.md`, `…-1-A.md`.
Chaque rapport : findings numérotés (`S-B1`, `S-E1`, `S-A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu
(`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le
décompte final **recompté depuis la liste**.
