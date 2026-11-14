# Prise en main — développer sur Kesh

Cette note s'adresse à un développeur qui rejoint Kesh. Elle dit **ce qu'il faut savoir avant d'écrire la première ligne**,
puis comment monter le poste, vérifier son travail et le proposer. Elle renvoie aux documents du dépôt plutôt que de les
recopier : quand une règle y est écrite, c'est eux qui font foi.

## 1. Avant tout : Kesh tient des livres réels

Kesh est **en production** : le Project Lead y tient sa comptabilité réelle depuis le 2026-10-31. Trois règles en
découlent, sans exception :

- **Aucune donnée réelle** dans le dépôt, les issues, les commits, les captures ou les fixtures : ni IBAN, ni nom, ni
  montant, ni contenu d'une pièce. Le dépôt est **public**. Les tests emploient des données synthétiques.
- **Jamais l'installation de production** : aucun essai, aucune recette, aucune clé d'essai ne se joue sur l'instance du
  Project Lead. Tout se joue sur une installation jetable (la vôtre, § 3).
- **Une migration n'est jamais anodine** : elle s'appliquera à des livres qu'on ne refait pas, et toute migration rend le
  retour à la version précédente dépendant de la sauvegarde d'avant la mise à jour. Avant d'en écrire une, parlez-en
  (§ 6).

## 2. Ce qu'il faut lire, dans cet ordre

1. **`CLAUDE.md`** (racine) — les règles du dépôt : tests, revues, migrations, priorités des défauts, commits. Il est
   long ; lisez au moins *Test Locally First*, *Review Iteration Rule*, *Priorités des défauts*, *Issue Tracking Rule*,
   *Migration breaking policy* et *Règle de commit et push*.
2. **`README.md`** — l'aperçu, la structure du dépôt, le démarrage rapide.
3. **`docs/testing.md`** — les patterns de tests, la base de dev jetable, la recette E2E et **la liste des échecs E2E
   attendus** (§ *Les échecs attendus*).
4. **`docs/ci.md`** — ce que la CI exécute, et ce qu'elle n'exécute pas.
5. Selon votre sujet : `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (toute requête est scopée par société),
   `docs/optimistic-locking-patterns.md`, `docs/i18n-key-ownership-pattern.md`, `docs/api-external.md`, le PRD et
   l'architecture sous `_bmad-output/planning-artifacts/`.

## 3. Le poste de dev

**Prérequis** : Rust (la version est fixée par `rust-toolchain.toml`, installée d'office par rustup), Node.js ≥ 20, Docker
et Docker Compose. `cargo install cargo-nextest` est recommandé.

```sh
git clone https://github.com/guycorbaz/kesh.git && cd kesh
bash scripts/install-hooks.sh        # refuse les commits directs sur main
docker compose -f docker-compose.dev.yml up -d mariadb
```

- **La base de dev est en RAM** (tmpfs) : rien ne survit à un redémarrage du conteneur, et c'est voulu. Le conteneur se
  crée **depuis le clone principal, jamais depuis un worktree** (son montage des droits est relatif au répertoire de
  lancement). Détail et remise à zéro : `docs/testing.md` § *Base de dev jetable*.
- **Migrations et données de dev** : `sqlx migrate run --source crates/kesh-db/migrations`, puis
  `docker exec -i kesh-mariadb-dev mariadb -uroot -pkesh_dev_root kesh < scripts/seed-dev-db.sql`.
- **Lancer l'application** : `README.md` § *Démarrage rapide* (backend `cargo run -p kesh-api`, frontend
  `cd frontend && npm install && npm run dev`). Au premier accès, l'écran `/setup` crée l'administrateur.
- **Avant chaque gate complet, remettez la base à zéro**, même après un run vert : un test peut laisser des lignes qui
  font rougir le suivant (`CLAUDE.md` § *Un gate laisse la base piégée*).

## 4. Vérifier son travail en local

La règle est simple : **avant chaque `git push`, la même série de contrôles que la CI**.

**Backend** (racine du dépôt) :

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
scripts/test-fast.sh --ci        # fmt + clippy + nextest, MariaDB de dev démarré
```

La CI lance `cargo test --workspace -j1 -- --test-threads=1` (un processus par binaire) ; `scripts/gate-ci.sh <base>` la
rejoue en local quand un doute existe (`docs/ci.md`).

**Frontend** (`frontend/`) :

```sh
npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build
```

**E2E** (Playwright) : la recette exacte est dans `docs/testing.md` § *Prérequis Playwright local* — recopiez-la, ne la
reconstituez pas (`KESH_COOKIE_SECURE=false` y est indispensable en HTTP local). **La suite ne passe pas au vert** : sept
échecs connus (KF-029, #97) sont attendus. Un rouge se juge fichier par fichier contre la liste de `docs/testing.md`, jamais
au nombre.

**Mémoire** : un gate complet est gourmand. `scripts/mem-guard.sh <commande>` l'exécute sous un plafond de mémoire (sans
effet si systemd utilisateur est absent) ; `scripts/test-fast.sh` le fait seul.

## 5. Proposer un changement

- **Une issue d'abord**, sur GitHub, avec le gabarit qui convient (`.github/ISSUE_TEMPLATE/` : bug, known failure,
  demande). GitHub Issues est l'unique registre : pas de liste de défauts dans le dépôt. **Un point qu'on décide de ne pas
  traiter s'ouvre aussi en issue** (« on trace toujours »), une issue par problème.
- **Une branche par travail**, créée depuis `main` à jour : `story/…`, `fix/…`, `chore/…`, `docs/…`.
- **Commits** : un par étape, au format Conventional Commits (`fix(kesh-api) : …`), chacun en `refs #N`. ⛔ **Aucun mot-clé
  de fermeture** (`closes`, `fixes`, `resolves`) dans un message de commit : il ne s'écrit que dans le corps de la **PR**,
  que le dépôt fusionne en squash.
- **PR** : gabarit `.github/PULL_REQUEST_TEMPLATE.md` ; la CI exécute quatre jobs (backend, frontend, image Docker, smoke
  E2E), tous requis pour la fusion. Une PR qui ne touche que `_bmad-output/` les saute.
- **Documentation dans la même PR que le code** : un comportement visible qui change met à jour le manuel concerné
  (`docs/manual/fr/*.tex`, PDF régénéré), les **quatre locales** (`crates/kesh-i18n/locales/*/messages.ftl`), et la note de
  version au CHANGELOG à la release.
- **Qualité** : DRY, code documenté (`///` en Rust), tout ce qui se teste est testé ; un test neuf doit **rougir sans le
  correctif** (le montrer). Une fonctionnalité visible s'accompagne de captures d'écran.

**La revue.** Les changements de Kesh passent par une boucle de revue adversariale (`CLAUDE.md` § *Review Iteration Rule*) :
elle se relance tant qu'une passe trouve un défaut au-dessus de LOW. Le Project Lead fixe avec vous le niveau de revue
attendu pour vos PR.

## 6. Coordination

- **Vos epics portent les numéros 37, 38 et 39**, réservés pour vous (`_bmad-output/planning-artifacts/epics-37-39-reserves.md`,
  versé par la PR #1160).
  Leur contenu se décide avec le Project Lead.
- **D'autres travaux avancent en parallèle** (l'Epic 36, puis 40 et suivants). Deux points se règlent au début de chaque
  epic : les **fichiers communs** (qui touche quoi, et dans quel ordre les fusionner) et les **migrations**, numérotées par
  horodatage et dont l'ordre se contrôle à l'intégration.
- **Migrations** : toute PR qui en ajoute une suit la politique P1 à P8 du `CLAUDE.md` (bump de version minimale pour une
  migration *breaking*, ligne à `docs/migrations-idempotence-audit.md`, triage des backfills, une migration appliquée ne se
  modifie plus jamais). Annoncez-la avant de l'écrire.
- **Ne touchez pas aux branches d'un autre**, ni à ses bases de test.
- **Une question, un désaccord, un défaut trouvé** : une issue, ou le Project Lead directement.
