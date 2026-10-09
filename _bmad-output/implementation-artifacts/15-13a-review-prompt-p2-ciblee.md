# Prompt — revue de code P2 ciblée, Story 15-13a

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur le seul commit de la remédiation P1 — documentation, configuration CI et
un test ; aucune ligne Rust de production.*

Worktree `/home/gcorbaz/devel/kesh-15-13a`, branche `story/15-13a-mariadb-non-publiee`. **Objet** :
`git show 7faa16bb` (diff unique ; ouvre les fichiers à `HEAD` autour des hunks). Rapports remédiés :
`target/gate-logs/15-13a-review-p1-{B,E,A}.md`. Choix C-15-13a-3, C-15-13a-4.

## Lentille unique — chasseur de régressions de la remédiation

1. **La distinction « installation neuve / base déjà créée »** : relis chaque site touché (`DOCKER_START.md`,
   `docs/manual/fr/admin-manual.tex`, `CHANGELOG.md`). Un passage dit-il encore de générer une valeur neuve sans
   condition ? Deux passages se contredisent-ils ? Rejoue le grep par la valeur
   (`git grep -nE "openssl rand|générez|choisissez|MARIADB_ROOT_PASSWORD|MARIADB_PASSWORD" -- . ':(exclude)_bmad-output'`)
   et vérifie le tri du rapport de remédiation.
2. **La recette `ALTER USER` refondue** (`admin-manual.tex`) : sous-shell `( set -e … )`, affichage avant
   application, `read`, `printenv MARIADB_USER`, ligne témoin, `up -d` en second bloc. Est-elle exécutable
   telle quelle (guillemets, `exec -T`, `\` de continuation) ? Un mot de passe peut-il se retrouver dans
   l'historique ou dans `ps` ? Le cas « compte applicatif refusé » laisse-t-il bien root inchangé ?
3. **`ci.yml`** : `refus.txt` sous `$RUNNER_TEMP` ; le commentaire sur M27 exact ; l'étape peut-elle passer à vide ?
4. **`configuration_transmise.rs`** : la recherche par nom remplaçant `contenus[2]` peut-elle passer à vide ?
5. **PDF du manuel** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `target/gate-logs/` — les phrases témoins du rapport sont-elles présentes, et `mot_de_passe_fort` absent ?

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; texte relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-13a-review-p2-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `docker`,
`git commit`/`add`/`checkout`/`switch`/`stash`/`reset`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`,
SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`grep`, `pdftotext`.
