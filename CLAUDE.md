# CLAUDE.md

Règles et décisions en vigueur pour travailler sur Kesh. Ce fichier ne porte ni historique ni précédent : ceux-ci sont dans les rétrospectives (`_bmad-output/implementation-artifacts/`), le `CHANGELOG.md` et `git log -p CLAUDE.md`. Une règle peut tenir une ligne de motif ; une décision nomme qui l'a prise et quand. **Quand une règle change, on la réécrit ; on n'y ajoute pas d'annotation.**

## Le projet

**Kesh** est un logiciel de comptabilité suisse pour particuliers et petites entreprises, développé avec le cadre BMAD. Backend Rust (Axum), frontend SvelteKit, MariaDB ; application web seulement ; configuration par variables d'environnement ; déploiement par docker-compose. PRD : `_bmad-output/planning-artifacts/prd.md` (QR-facture 2.2, pain.001.001.03, CAMT.053.001.04, FR/DE/IT/EN).

- **Dernière release : [v0.19.0](https://github.com/guycorbaz/kesh/releases/tag/v0.19.0)** (2026-11-18), **avec migrations** ; `kesh_version_min_required` = 0.17.6.
- **Kesh est en production** : Guy tient ses livres réels sur le NAS depuis le 2026-10-31. Version en fonction : **0.19.0**, depuis le 2026-11-18. Ces deux lignes se mettent à jour à chaque release et à chaque mise à jour du NAS.
- **Jalon ouvert** : « Première clôture d'exercice tenue dans Kesh » (J9 de la fiche de projet). Le jour où il est atteint, cette section se réécrit.
- **Les décomptes** (migrations, tests, stories, issues) ne s'écrivent pas ici : ils vont à la PR de release, au journal `kesh-gate-logs/rel<version>/` et à la rétrospective. Ils se recomptent depuis la source :
  - migrations : `ls crates/kesh-db/migrations/*.sql | wc -l` ;
  - stories livrées : clés de `development_status:` de `sprint-status.yaml` au statut `done`, hors `epic-N` et `*-retrospective`, lues par un parseur YAML ; epics : clés `epic-<entier>` ; un epic est ouvert s'il porte une story `backlog`, `ready-for-dev`, `in-progress` ou `review` ;
  - issues ouvertes : `gh issue list --state open --limit 600 --json number --jq length` ; bugs : commande de § *Priorités des défauts* ; au tag passé, reconstituer depuis `createdAt`/`closedAt`.

### Ce qu'impose la production

- **Un défaut qui perd ou altère des données est un P1** : `.keshbackup`, import d'installation, sauvegarde automatique, migrations. Un défaut qui a écrit faux se corrige dans les livres, par une écriture.
- **Toute release qui porte une migration exige la sauvegarde d'avant la mise à jour** — base, `.keshbackup` et dossier des documents —, écrite en tête de ses notes et au manuel d'administration. La sauvegarde que Kesh écrit seul dans `backup/pre-upgrade/` (#847) n'en dispense pas.
- **Rien de jetable sur la base du NAS** : recettes, écritures de démonstration et clés d'essai se jouent sur une installation jetable. Une montée de version se répète d'abord sur une base peuplée par la version d'où l'on part (`scripts/recette-montee.sh`).
- **`~/kesh/` est en lecture seule** : ce sont les pièces de production de Guy.

## Communication

Guy travaille en **français** : conversation et documents en français, sauf indication contraire.

## Structure du dépôt

```
crates/                 # Backend Rust — workspace de 10 crates
  kesh-core/            # Logique métier pure, ZÉRO I/O
  kesh-db/              # Persistance MariaDB via SQLx + les migrations
  kesh-api/             # Serveur HTTP Axum — routes, auth, middleware
  kesh-qrbill/          # QR-facture SIX 2.2 + génération des PDF
  kesh-import/          # Parseurs CAMT.053 et CSV (publiable, zéro dépendance interne)
  kesh-payment/         # Fichiers de paiement ISO 20022 (pain.001)
  kesh-reconciliation/  # Réconciliation bancaire
  kesh-report/          # Générateurs de rapports comptables
  kesh-i18n/            # i18n et formatage suisse (Fluent, 4 locales)
  kesh-seed/            # Données de démonstration
frontend/               # SvelteKit — src/lib/features/ par domaine, tests/e2e/ Playwright
scripts/                # Outillage : test-fast.sh, mem-guard.sh, prepare-release.sh, hooks
docs/                   # Documentation, dont manual/{fr,de,en,it}/ (LaTeX + PDF)
website/                # Site GitHub Pages (HTML statique)
_bmad/                  # Cadre BMAD (agents, workflows, skills, config)
_bmad-output/           # Artefacts BMAD : planning-artifacts/, implementation-artifacts/
.claude/skills/         # Skills BMAD installées pour Claude Code
```

Les chemins se vérifient, ils ne se supposent pas. Ne pas écrire ici de nombres que rien ne contrôle (skills, workflows) : `wc -l` sur les manifestes de `_bmad/_config/` dit la vérité.

## BMAD

- Les workflows exécutent leurs étapes dans l'ordre, sans en sauter ; leur état vit dans le frontmatter (`stepsCompleted`).
- Les skills s'invoquent par `/nom-du-skill`.
- Toute sortie engendrée va dans `_bmad-output/`, jamais mêlée aux fichiers du cadre.
- Configuration : `_bmad/bmm/config.yaml` ; manifestes : `_bmad/_config/`.

## Qualité du code

- **DRY** : pas de code dupliqué ; extraire la logique partagée.
- **Documentation** : API publiques, logique complexe et modules documentés (`///` en Rust, JSDoc en Svelte).
- **Tests** : tout ce qui se teste se teste — unitaires pour la logique métier (moteur comptable, TVA, calculs financiers), intégration pour les parseurs (CAMT.053, QR-facture, pain.001), Playwright pour chaque parcours du PRD.
- **Endpoints batch** : § *Pattern batch — FailedProposal per-proposal*.
- **Un appariement automatique propose, il ne crée jamais.** Quand Kesh rapproche une donnée entrante d'une entité existante (fournisseur, débiteur, article), il propose un rattachement ; il ne crée jamais l'entité d'office et ne tranche jamais seul un cas douteux, qui va dans une file « à rattacher ». La prévention se place à la saisie : signaler un contact proche, jamais bloquer. *Motif : le faux rattachement est muet, le doublon ne l'est pas.*

## Test Locally First

**Avant chaque `git push` qui ouvre ou met à jour une PR, lancer localement les mêmes contrôles que la CI.** Ne s'applique pas aux commits qui ne touchent aucun code exécutable (Markdown, YAML de planning, README, CLAUDE.md).

### Backend (Rust)

Depuis la racine — exactement le job `Backend (Rust)` de `.github/workflows/ci.yml` :

```sh
cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace            # la CI : -j1 -- --test-threads=1
```

#### Gate rapide — `cargo-nextest`

```sh
scripts/test-fast.sh            # fmt + clippy + nextest (MariaDB de dev démarré)
scripts/test-fast.sh --no-lint  # nextest seul
scripts/test-fast.sh --ci       # profil ci
scripts/gate-ci.sh <base>       # la commande de la CI, en environnement fermé (docs/ci.md)
```

- nextest lance un processus par test, la CI un par binaire : ce qu'un test laisse dans l'état du processus ne se voit qu'avec `gate-ci.sh`.
- Les tests `#[sqlx::test]` montent le squash `crates/kesh-db/test-schema/`, sauf ceux qui testent le chemin des migrations ; la liste est tenue par `crates/kesh-db/tests/test_schema_guard.rs`.
- `test-threads = 8` (`.config/nextest.toml`, décision de Guy du 2026-08-16). Son innocuité n'est pas établie : si un `reconciliation_*_e2e` rougit, soupçonner d'abord ce plafond.

### Frontend (Svelte)

```sh
cd frontend
npm run check
npm run lint-i18n-ownership
npm run test:unit
npm run build
```

### E2E (Playwright)

- **La suite complète tourne en local avant tout `git push`**, et au **dernier commit de code** de la story ; « rejouée au push » est interdit. Un commit qui ne touche que des comptes rendus ou de la documentation après l'E2E ne l'oblige pas à repasser.
- **Montage** : `docs/testing.md` § *Prérequis Playwright local* fait foi. `KESH_COOKIE_SECURE=false` y est obligatoire (Playwright ne joint pas un cookie `Secure` sur `http://127.0.0.1`), ses valeurs factices ne se remplacent pas par des gabarits `<…>` (refusés au démarrage), et `KESH_TEST_MODE=true` se pose des deux côtés.
- **« Vert » veut dire : seulement les échecs attendus** (sept de la KF-029, #97, plus zéro à deux échecs de pollution). Un rouge se juge fichier par fichier contre `docs/testing.md` § *Les échecs attendus*, jamais au nombre.
- **La CI ne lance qu'un smoke** (`e2e-smoke`, `frontend/tests/e2e/smoke.spec.ts`, contre l'image Docker de la PR), requis pour la fusion (décision de Guy du 2026-10-24). Il vérifie que le harnais est vivant, pas les fonctionnalités.
- **Avant de diagnostiquer, lire la doc du dépôt** ; ne jamais déclarer une hypothèse écartée sans l'avoir exécutée.

#### Recette visuelle

Toute story qui ajoute ou modifie un **contrôle visible** (bouton, avis, dialogue, colonne, état d'un écran) livre des captures de chaque état touché, prises sur une base de démonstration, jointes à son Dev Agent Record et au prompt de la revue de code. Thème clair seulement : Kesh n'a pas de thème sombre. Les captures vivent hors du dépôt (`kesh-gate-logs/`). *Motif : un test vérifie qu'un élément existe, non qu'on le voit.*

### Pendant une boucle de revue — gate ciblé, gate complet au push

- **Entre deux passes**, un gate ciblé suffit :
  ```sh
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo nextest run -E 'binary(<le binaire de test touché>)'
  ```
- **Le gate complet est obligatoire** avant tout `git push`, avant de déclarer une story `done`, et au dernier commit d'une boucle de revue.
- **Exception `kesh-db`** : dès qu'un patch touche `crates/kesh-db/migrations/`, `post_restore.rs` ou un repository, gate complet même en cours de boucle.
- **Le story file n'affirme que ce qui a tourné** : écrire « gate ciblé `binary(x)` vert, gate complet au push » quand seul le binaire ciblé a tourné.

### Base de gate

- **Remettre la base à zéro avant tout gate complet**, sans condition, et avant de diagnostiquer tout rouge sur des tests que la branche ne touche pas (un gate, même terminé en vert, peut laisser des résidus) :
  ```sh
  docker compose -f docker-compose.dev.yml restart mariadb   # depuis le dépôt principal ; tmpfs : efface tout
  sqlx migrate run --source crates/kesh-db/migrations
  docker exec -i kesh-mariadb-dev mariadb -uroot -pkesh_dev_root kesh < scripts/seed-dev-db.sql
  ```
- **`docker compose` sur ce conteneur se lance depuis le dépôt principal, jamais depuis un worktree** (le montage des droits est relatif). En travail parallèle, un agent ne redémarre ni ne recrée le conteneur : il travaille sur ses bases `kesh_<clé>` (`docs/agents-orchestration.md`, `docs/testing.md` § *Base de dev jetable*).

### CI et commits de documentation

- Les quatre jobs (`backend`, `frontend`, `docker-build`, `e2e-smoke`) tournent sur les PR vers `main` et `epic/**` et sur `push` vers `main`, quel que soit le fichier touché — sauf une PR dont le diff ne nomme **que** `_bmad-output/` (filtre `scripts/ci-seuls-artefacts.sh`, qui échoue fermé ; `docs/ci.md`).
- Un commit de documentation est donc gratuit en local mais coûte un cycle de CI : les regrouper, ou les faire voyager avec la PR qui les motive.

## Plafonds mémoire

- **Tout travail lourd sur la station passe par `scripts/mem-guard.sh`** (cgroup borné) ; `scripts/test-fast.sh` le fait déjà. Il délègue à l'outil global `mem-guard` (`~/.local/bin`) : 10G + 1G de swap par exécution, deux exécutions à la fois sur toute la station.
  ```sh
  scripts/mem-guard.sh --protect-shell        # une fois par nouvelle fenêtre
  scripts/mem-guard.sh cargo build --workspace --all-targets
  scripts/mem-guard.sh npm --prefix frontend run test:unit
  scripts/mem-guard.sh --status               # état, politique oomd, derniers OOM
  ```
  *Motif : sans plafond, oomd tue l'onglet entier, session de travail comprise.*
- **Tout gate lourd prend aussi le verrou** `flock <kesh-gate-logs>/.gate.lock`.
- **Avant d'incriminer le gate en cours d'un OOM**, lire `scripts/mem-guard.sh --status` et `ps -eo pid,rss,args --sort=-rss | head` : la pression vient souvent d'une autre fenêtre.
- **Ne pas activer `MemoryHigh`** sans refaire la mesure : il étrangle sans tuer.
- **Réglages de parallélisme** : `[build] jobs = 4` et mold `--thread-count=4` (`.cargo/config.toml`, qui écrase celui de la station : toute modification de la station s'y répercute), `[profile.dev] debug = "line-tables-only"` (pour un débogage pas à pas : `CARGO_PROFILE_DEV_DEBUG=full`), vitest `maxWorkers: 4`, nextest `test-threads = 8` (borné par MariaDB, pas par la mémoire).
- Lire `memory.stat/anon`, pas `memory.peak` (qui compte le cache de pages), pour juger d'un besoin en mémoire.

### Worktrees et agents

- **Un agent n'écrit jamais hors de son worktree** (et de `kesh-gate-logs/`), toujours en chemin absolu ; le dépôt principal est en lecture seule pour lui. Consignes communes : `docs/agents-orchestration.md`.
- **Un worktree se retire à la clôture de sa story** (`git worktree remove`, après un `git status --porcelain` vide) ; sa branche reste (décision de Guy du 2026-11-14). Chacun garde 20 à 50 Go de cible cargo.

## Review Iteration Rule

S'applique à `bmad-create-story validate`, à `bmad-code-review` et à toute revue adversariale du même genre.

- **On relance une passe tant qu'une passe rend un finding au-dessus de LOW** (CRITICAL, HIGH, MEDIUM), patches appliqués avant la passe suivante. Arrêt : une passe qui ne rend **aucun** finding au-dessus de LOW — même un MEDIUM corrigé hors production appelle une passe de plus (décision de Guy du 2026-10-23) — ou **8 passes**.
- **Contexte frais à chaque passe, et modèle différent** de la passe précédente : Sonnet ↔ Opus pour les passes complètes ; **Haiku seulement pour une passe ciblée** ; **une première passe n'est jamais ciblée** et ne va jamais à Haiku (décisions de Guy des 2026-10-06 et 2026-10-24).
- **Change Log final** (une entrée, pas une par passe) : tendance des findings par passe, modèles employés, reclassements.
- **Exception** : un finding MEDIUM+ reclassé en dette documentée (section `Security debt`, `Performance debt` ou équivalente), avec propriétaire et story de remédiation planifiée, compte comme résolu.

### La passe ciblée

- Quand une boucle converge, la passe suivante peut être une **passe ciblée** : une seule lentille, en contexte frais, sur **le seul commit de la dernière remédiation**. *Motif : la sévérité se déplace vers ce qu'on vient d'écrire.*
- **À employer** quand la sévérité est retombée, ou ne vient plus que du patch précédent, et que la remédiation touche du code. **Ne suffit pas** si le patch touche plusieurs modules ou change une règle métier : reprendre le protocole complet.
- **Une boucle ne se clôt pas tant que la dernière remédiation touche du code de production** ; et elle ne se clôt que sur une passe sans finding au-dessus de LOW.
- **Le prompt de la lentille se versionne** à côté du story file.
- **Les remédiations de l'orchestrateur suivent la même boucle** que celles d'un agent : il lit au code la prémisse du finding (source d'une dépendance comprise, même pour un LOW), puis sa remédiation passe sous une passe ciblée.
- **À l'approche du plafond**, retirer ou renvoyer au développement plutôt que corriger : une remédiation écrite après la dernière passe n'est relue par personne. **Dès la sixième passe, l'orchestrateur ne remédie plus lui-même.** Une validation peut se clore au plafond sur des MEDIUM corrigés sans passe de plus si leur recontrôle se fait à l'étape 0 du développement et se déclare au Change Log, ligne lue citée (décision de Guy du 2026-10-24).

### Le prompt d'une passe doit NOMMER le manuel

Tout prompt de passe portant sur du code visible de l'utilisateur inscrit parmi ses axes la vérification du manuel (`docs/manual/`) contre le code, **PDF compris**, aplati avant de greper : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`. *Motif : aucune passe ne lit le manuel si on ne le lui demande pas, et c'est là que se voient les écarts.*

### Une passe déclare ses axes exercés

- Tout prompt de passe exige la liste des axes **exercés et non exercés**. Un « 0 finding » sans cette liste ne compte pas comme passe.
- L'orchestrateur vérifie un « 0 finding » comme un finding : il reprend lui-même les axes déclarés non exercés.
- Les interdits d'un prompt nomment les **commandes** qui écrivent (`scripts/prepare-release.sh` et tout script mutant), pas seulement « n'écris aucun fichier ».

### Inventorier les sites NON RÉSOLUS

Pour prouver qu'une classe de défauts est fermée, inventorier **l'ensemble clos des sites qui ne résolvent pas**, chacun résolu ou écrit comme angle mort assumé — et non énumérer les formes traitées. Une assertion dont les deux côtés sortent du même extracteur est vraie par construction et ne prouve rien.

### Propagation post-patch — grep du symptôme

- Après un patch de remédiation et avant la passe suivante, **greper le symptôme corrigé sur tout le dépôt** (code, fiches, doc-comments, tests, i18n des quatre locales, fallbacks Svelte, manuels) et traiter tous les sites dans le même patch.
- **Greper la valeur, pas la formulation** : pour un nombre, une version ou un identifiant, le jeton (`\b25\b`, `\b0\.8\.0\b`). Un grep vide sur un symptôme qu'on vient de corriger éveille le soupçon.

### Recompter

- Tout **décompte** écrit dans un Change Log, un Dev Agent Record ou un rapport de passe se **recompte depuis la source**, avec son **périmètre** (de quelle borne à quelle borne). Un total est cohérent avec sa ventilation.
- **Recompter n'est pas réécrire l'historique** : une nouvelle valeur va dans l'entrée vivante, jamais dans l'entrée d'une passe passée ; celle-ci ne se corrige que si elle était fausse à sa date, par une annotation datée.

### Haiku-specific guardrails — grep ground-truth

Pour tout finding CRITICAL ou HIGH qui affirme l'absence d'un code attendu ou la présence d'un anti-pattern, **vérifier avant de le traiter comme réel** — pour tous les modèles, Haiku étant le cas connu (indexation des diffs multi-commits) :

- motif sur une ligne : `grep -nF "<chaîne>" <fichier>` (`-F` obligatoire) ; motif sur plusieurs lignes : une ligne discriminante, ou `grep -nFA <N>` ;
- flux sur plusieurs fonctions ou fichiers : lecture directe, extrait cité au Change Log ;
- finding réfuté : écarté comme faux positif, consigné au Change Log.

À partir de la deuxième passe, donner à Haiku un diff unique aplati (`HEAD` contre `main`).

### Pattern batch — FailedProposal per-proposal

Pour tout endpoint qui traite N propositions et rend `{ accepted, failed }` :

- **Aucune erreur propre à une proposition ne devient une `AppError` globale** : elle va dans `failed[]`, et la requête rend **200**. Seules restent globales les erreurs qui invalident toute la requête : 401, 403, 400 (corps illisible), 500 (pool fermé, panique, I/O).
- **`FailedProposal`** porte : l'identifiant métier de la proposition (jamais un index positionnel), `error_code` (constante, jamais une chaîne interpolée), `details: Option<serde_json::Value>` pour le contexte.
- **Pas de `unreachable!()`** dans un `match` sur les variantes : `tracing::error!` puis `AppError::Internal`. Critère : un `match` incomplet après refactor est un bug structurel (500) ; une donnée inattendue sur un code qui compile est une erreur métier (`FailedProposal`).
- Références : `accept_one_invoice`, `accept_one_split`, `accept_one_rule`. Un import brut (CAMT.053) ne suit pas ce pattern.

### Règle de splitting préventif

**Découper une story avant `bmad-create-story`** si elle touche **plus de cinq modules** distincts, ou si sa validation **recycle** ses défauts — une passe N+1 de sévérité égale ou supérieure dont les défauts reviennent sous une autre forme ou naissent du correctif précédent. Des défauts d'origine neufs et distincts ne découpent pas. Le signal se déclare toujours au Project Lead, avec le constat au Change Log.

- **Comment** : une story zéro qui pose le motif sur un ou deux modules pilotes, puis des stories de déploiement mécaniques, revues fichier par fichier.
- **Dérogation** : si le découpage crée des cycles de dépendances ou des fusions intermédiaires intestables, garder la story unique et l'écrire dans une section `Dérogation règle de splitting`.

## Priorités des défauts

Chaque défaut ouvert porte sa priorité dès sa création, et c'est elle qui ordonne le travail.

| Label | Niveau | Critère |
|---|---|---|
| `P1` | critique | erreur comptable (écriture, solde ou compte faux), perte ou corruption de données, conformité légale, sécurité |
| `P2` | haute | comportement faux sans effet sur les comptes — erreur 500, opération enregistrée mais annoncée en échec, piste de contrôle incomplète |
| `P3` | moyenne | manuel qui décrit autre chose que le code, test défaillant, dette de code |
| `P4` | basse | orthographe, traduction, cosmétique |

**Trois engagements par epic**, dans cet ordre (décision de Guy du 2026-10-08) :

1. **Le nombre de bugs baisse.** Bug = issue ouverte portant `bug` ou `known-failure`, comptée au kickoff et à la rétrospective :
   ```sh
   gh issue list --state open --limit 600 --json labels \
     --jq '[.[]|select([.labels[].name]|any(.=="bug" or .=="known-failure"))]|length'
   ```
2. **Aucun P1 ne passe d'un epic à l'autre**, autant que possible ; celui qui reste est nommé à la rétrospective, avec sa raison.
3. **L'epic livre une fonctionnalité nouvelle si c'est possible**, la baisse des défauts restant prioritaire.

Et :

- **Report assumé des dix P1 de la TVA** (jalon « Vague 2 » : #390 à #397, #401, #525) : Guy, seul utilisateur réel, n'est pas assujetti. Ils sont nommés à chaque rétrospective ; le report tombe dès que Kesh tient, ou est proposé pour tenir, les livres d'un assujetti (décision de Guy du 2026-10-08).
- **Moins d'issues ouvertes au tag qu'au tag précédent est un objectif, pas une barrière** : le dépasser n'empêche pas de publier. Le décompte (toutes les issues) s'écrit : celui du tag précédent et l'objectif au fichier d'epic dès le kickoff, celui du tag à la PR de release et à la rétrospective, avec l'écart et sa cause (décision de Guy du 2026-10-24).
- **On trace toujours** : un point qu'on décide de ne pas traiter s'ouvre en issue, même s'il fait remonter le compte ; on ne ferme jamais un défaut réel pour tenir le chiffre ; **une issue par problème**, sans regroupement (décisions de Guy des 2026-10-23 et 2026-11-10).
- **Un LOW de revue qui touche la production peut se corriger dans la story**, suivi d'une passe ciblée de ce seul correctif ; l'issue reste le choix par défaut quand le correctif sort du périmètre (décision de Guy du 2026-11-14).
- **Une story de solde à la fin de chaque epic**, avant le tag : seulement la dette des stories de l'epic, plafonnée par un nombre d'issues fixé au kickoff (décision de Guy du 2026-11-14).
- **L'étiquette `bug` décrit un comportement faux visible** : un test manquant, un commentaire, une phrase de fiche prennent `technical-debt` ou `documentation`. Les anciennes issues ne se réétiquettent que sur demande de Guy, et un tel réétiquetage s'écrit à côté du décompte des bugs (décision de Guy du 2026-11-14).

Les anciennes rétrospectives classent la dette en **A** (vraie dette), **B** (limitation documentée, remédiation planifiée) et **C** (choix de conception, pas une dette).

## Clôture d'epic

À la fin de chaque `bmad-retrospective`, enchaîner sur la **revue de projet** dans `/home/gcorbaz/travail/Projets actuels/kesh` : la rétrospective regarde le dépôt, la revue de projet le dossier (échéances, dépendances, engagements hors dépôt). La rétrospective d'abord, ses conclusions étant des entrées de la revue.

## Issue Tracking Rule

- **GitHub Issues est l'unique source de vérité** pour les CR, KF et bugs ([guycorbaz/kesh/issues](https://github.com/guycorbaz/kesh/issues)), avec les modèles de `.github/ISSUE_TEMPLATE/`. Aucun fichier du dépôt ne tient sa propre liste. `docs/change_request.md` et `docs/known-failures.md` sont archivés : on n'y écrit plus.

| Type | Modèle | Labels |
|------|--------|--------|
| Bug | `bug_report.yml` | `bug`, `triage` |
| KF | `known_failure.yml` | `known-failure`, `triage` (+ `technical-debt` si dette persistante) |
| CR | `feature_request.yml` | `enhancement`, `triage` |

- **Titre d'une KF : `[KF-NNN] description`.** Le numéro se lit **depuis GitHub** avant d'être attribué, sur le seul préfixe :
  ```sh
  gh issue list --state all --limit 600 --json title --jq '.[].title' \
    | grep -oE '^\[KF-[0-9]{3}\]' | grep -oE '[0-9]{3}' | sort -n | tail -1
  ```
- **Quand créer** : un bug reproduit hors de la story en cours (un bug trouvé dans la story se corrige dans la story, au Change Log) ; une KF détectée hors du travail courant ; un CR **avant** tout changement de périmètre d'une story validée.
- **Commits** : chaque commit qui touche une issue la cite en `refs #N`. **Aucun mot-clé de fermeture (`closes`, `fixes`, `resolves` et variantes) dans un message de commit**, pas même en prose : il ne s'écrit que dans le titre ou le corps de la PR, puisque le dépôt fusionne en squash. Contrôle avant d'ouvrir ou de fusionner une PR :
  ```sh
  git log origin/main..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'
  ```
  Écrire « validation terminée », non « validation close ». Après fusion, vérifier : `gh issue view <N> --json state --jq .state`.

## Migration breaking policy

- **(P1) Définition.** Une migration est **breaking** si un binaire antérieur ne peut plus consommer le schéma correctement (`DROP COLUMN` lue, `RENAME TABLE`, `MODIFY`/`CHANGE COLUMN` vers un type incompatible). **Toute** migration, breaking ou non, empêche le binaire précédent de démarrer (sqlx refuse une migration appliquée qu'il ne connaît pas) : le retour en arrière passe par la sauvegarde d'avant la mise à jour, ou par le SQL qui défait la migration et retire sa ligne de `_sqlx_migrations`. « Non breaking » veut dire : sans relever `kesh_version_min_required` et sans corrompre le schéma.
- **(P2) Bump.** Une migration breaking finit par `UPDATE _kesh_version SET kesh_version_min_required = '<version de la PR>' WHERE id = 1;`, version figée dans le SQL.
- **(P2-bis) Le bump `min_required` va toujours avec le bump Cargo** : si `min_required` passe à `X.Y.Z`, tous les crates (`crates/*/Cargo.toml`) sont ≥ `X.Y.Z` dans la même PR, sinon le binaire refuse de démarrer et d'importer une sauvegarde. Le défaut ne se voit qu'au gate complet : jamais `done` sans lui.
- **(P3) Revue.** Une migration `DROP TABLE`, `DROP COLUMN`, `RENAME TABLE`, `RENAME COLUMN`, `MODIFY COLUMN <type>` ou `CHANGE COLUMN <nom> <type>` sans bump de `kesh_version_min_required` est un finding **CRITICAL**. (MariaDB : `MODIFY`/`CHANGE COLUMN`, pas `ALTER COLUMN … TYPE`.)
- **(P4) Exception.** Une telle opération restée compatible porte `-- breaking-skip-bump: <justification>`, confirmée en revue.
- **(P5) Audit d'idempotence.** Toute nouvelle migration ajoute sa ligne au tableau de `docs/migrations-idempotence-audit.md` (`yes` / `no` / `tracked-by-sqlx`, avec justification) ; à défaut, finding **MEDIUM**. Le total apparaît deux fois (en-tête `## Table d'audit (N migrations)` et ligne `Total`) et égale la somme des trois compteurs de partition ; tous se recomptent depuis le tableau :
  ```sh
  ls crates/kesh-db/migrations/*.sql | wc -l
  grep -c '^| `20' docs/migrations-idempotence-audit.md
  ```
- **(P6) Couplage positionnel.** Toute nouvelle migration oblige à lancer `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et à inspecter chaque site : index résolu par version (`.position(|m| m.version == …)`, à préférer), ou couplage positionnel assumé avec garde qui échoue bruyamment (`assert_eq!(total, N)` ou assertion de montage). Un site positionnel sans garde est un finding **MEDIUM**. *Motif : un test indexé par position peut se mettre à passer à vide.*
- **(P7) Backfills de données.** Toute migration qui écrit des données se trie au registre `POST_RESTORE_BACKFILLS` ou à `EXEMPT_MIGRATIONS` avec justification écrite (`crates/kesh-db/src/post_restore.rs`) ; `every_data_backfill_migration_is_triaged` l'impose ; un manquement est un finding **MEDIUM**.
  - Le détecteur cherche large (`UPDATE`, `INSERT` sous toutes ses formes, `REPLACE`, `DELETE`, SQL décommenté, instructions sur plusieurs lignes) et classe sur le premier mot-clé ; `ON UPDATE CURRENT_TIMESTAMP` est du DDL.
  - Une migration hors de la fenêtre d'import ne s'inscrit pas au registre (`registry_entries_are_within_import_window`). Une justification « hors fenêtre » commence par la chaîne `Hors fenêtre`, faute de quoi `exemptions_claiming_out_of_window_really_are_out_of_window` ne la contrôle pas.
  - **Classe A** (rejeu inconditionnel) seulement si toutes les instructions sont gardées contre l'écrasement d'une valeur de l'utilisateur ; sinon **classe B** (rejeu conditionné à une colonne sentinelle), valide seulement si DDL et backfill sont dans le même fichier. Ne jamais classer sur la présence textuelle de `IS NULL` / `NOT EXISTS`.
- **(P8) Une migration appliquée ne se modifie plus, pas même un commentaire** : sqlx compare l'empreinte de l'octet et refuse de démarrer. Ce qu'on voudrait y ajouter va dans `docs/migrations-idempotence-audit.md`, les Dev Notes ou une migration suivante. Le gate complet ne le voit pas (bases éphémères) ; seul un démarrage sur une base persistante (E2E, `cargo run`) le révèle.

## Commits, branches et push

- **Brancher avant le premier `Edit`/`Write`** d'une story, d'un correctif ou d'une maintenance :
  ```sh
  git checkout main && git pull --ff-only
  git checkout -b story/X-Y-slug    # ou chore/…, fix/…, docs/…
  ```
  Le hook `scripts/hooks/pre-commit` refuse les commits sur `main`/`master` (installation : `bash scripts/install-hooks.sh`, à chaque clone et après toute mise à jour du hook). Contournement exceptionnel : `--no-verify`, justifié dans le message.
- **Branches** : `story/X-Y-slug`, `chore/…`, `fix/…` (rare), `docs/…`, `epic/N-slug`, `hotfix/X.Y.Z` (§ *Travailler à plusieurs développeurs*).
- **Un commit par étape BMAD** : après `bmad-create-story`, après chaque passe de validation, après `bmad-dev-story`, après chaque passe de `bmad-code-review`.
- **Push** : sur demande de Guy ; à chaque story close, sans redemander — sa branche `story/…`, puis la branche d'epic une fois la fusion contrôlée (décision de Guy du 2026-11-16) ; à la fin d'un epic ; ou quand un workflow l'impose, justifié.

### Documentation à synchroniser

- Les documents pour l'utilisateur et l'intégrateur (manuels, guide de l'API, CHANGELOG, notes de version, site) ne nomment que des numéros de release, jamais un epic ; `README.md` et `CLAUDE.md` peuvent nommer les epics, chacun avec le lien de sa release (décision de Guy du 2026-11-18).
- **À chaque commit** : la « Feuille de route » de `README.md` reflète l'état du projet (statut des epics, marqueurs *(à venir)*, renumérotations, changements de périmètre), mise à jour dans le commit qui la change.
- **Toute mise à jour de documentation voyage dans le commit ou la PR du code qui la motive.** Pas de commit de documentation cosmétique quand rien n'a changé.
- **À tout push de PR**, vérifier la cohérence des docs que la branche touche ; **à toute release et toute clôture d'epic**, audit complet de :
  1. `README.md` — feuille de route et « Fonctionnalités » ;
  2. `website/index.html`, `website/roadmap.html` — rien de promis qui ne soit dans `main`, rien de livré qui manque ; `website/issues.html` aligné sur les labels GitHub ;
  3. manuels LaTeX (`docs/manual/fr/` : `admin-manual`, `user-manual`, `marketing-brochure` ; DE/IT/EN) — mis à jour si la story touche installation, configuration, sécurité ou usage ;
  4. **(4-bis) macros de version, à toute release, sans condition** : `\keshVersion{X.Y.Z}` (= version Cargo et `/health`), `\keshReleaseDate{AAAA-MM-JJ}`, `\keshTargetRelease{vX.Y}` dans `docs/manual/shared/kesh-style.sty`, puis les PDF régénérés (`make fr` dans `docs/manual/`). Un tag dont la version diffère de `\keshVersion` est un défaut de release. Les URL `raw.githubusercontent.com/guycorbaz/kesh/<réf>/…` des manuels, du README et de `website/` portent le même tag `vX.Y.Z`, jamais `main` (garde G19 de `crates/kesh-api/tests/textes_coherents.rs`) ;
  5. `CHANGELOG.md` — entrée de la release, au format Keep a Changelog, assemblée depuis les *Notes de version* des fiches ;
  6. **`scripts/prepare-release.sh`, à lancer** : bump Cargo, datation du CHANGELOG, rappels sans autre filet ; il refuse la release quand une exemption datée de `EXEMPT_MIGRATIONS` est démentie par un tag publié. Poser un tag à la main contourne ce contrôle ;
  7. `scripts/recette-montee.sh` — montée et retour depuis la dernière image publiée (`docs/testing.md`).

## Travailler à plusieurs développeurs

Kesh a plusieurs développeurs depuis le 2026-11-15 : le Project Lead, l'orchestrateur qui travaille sur son mandat, et des contributeurs à qui des epics sont réservés (37 à 39, 40 ; le prochain epic interne est le 41, et le reste de l'Epic 36 part en epics neufs à partir du 42). Décisions de Guy des 2026-11-15 et 2026-11-16 ; `docs/developpement-a-plusieurs.md` les commente pour un contributeur, mais **elles font foi ici**.

### Planification — GitHub Projects

- Le projet GitHub « Kesh » porte la planification au niveau des **issues** : **release = milestone** (une issue n'appartient qu'à une release), **epic = champ du projet**, **qui = l'assigné**.
- **Les stories BMAD restent dans `sprint-status.yaml`** ; le projet ne les recopie pas.
- Seules les deux ou trois prochaines releases sont datées ; le reste va au milestone « Plus tard ». N'entrent au projet que les issues qu'un epic ou une release retient.
- Réserver un epic, c'est renseigner le champ *Epic* avec responsable et objet. **Rattacher une issue à une release est une décision du Project Lead** ; les contributeurs proposent.
- On parle d'epics, non de numéros de release : le numéro n'est acquis qu'au tag.

### Numérotation des versions

- **Une release planifiée monte le chiffre mineur** (0.19.0, 0.20.0) et reçoit son milestone d'avance.
- **Un correctif urgent monte le chiffre de correction** de la dernière release publiée (0.18.1) : branche `hotfix/X.Y.Z` partie du tag, **aucune migration** (sinon il va à la prochaine release planifiée), reporté dans `main` après publication.
- **Un epic = une release**, jamais plus : un chantier trop grand se découpe en plusieurs epics au kickoff ; un epic qui n'aboutit pas se réduit (§ *Cadence*). Une release peut porter plusieurs epics.
- **Un epic reste petit** : un epic de correction ou d'amélioration compte **au plus six stories**, story de solde comprise ; un epic de fonctionnalité nouvelle peut dépasser, si le kickoff le dit. La taille des stories compte d'abord : le splitting préventif s'applique dès la fiche.

### Ordre des releases

- **La première release terminée sort la première** ; si besoin, on échange les numéros et les milestones.
- **Le second arrivé reprend `main`** dans sa branche, résout, puis rejoue tous les gates et l'E2E complet ; si les deux releases portent des migrations, leur ordre se contrôle sur une base peuplée par la release du premier. La recette de montée part toujours de la dernière image publiée.
- Un correctif dont la base a été dépassée par une release planifiée se publie depuis celle-ci, sauf décision du Project Lead.
- **Seul le Project Lead pose un tag et publie une image**, ou l'orchestrateur sur son mandat explicite.

### Cadence — au moins une release par semaine

- **Au moins une release par semaine de présence du Project Lead** : la date tient, le périmètre s'ajuste.
- **La semaine type** : coupure le **jeudi soir** (n'entrent que les stories closes — boucle de revue terminée, gates de clôture et E2E complet verts) ; intégration le **vendredi matin** (dernières fusions, gates, recette de montée, revue du code d'intégration, PR, CI) ; **accord de fusion du Project Lead vers midi**, puis tag et image ; mise à jour du NAS par le Project Lead.
- **Chaque story se fusionne dans la branche de son epic dès sa clôture**, puis on pousse (§ *Commits, branches et push*) ; les branches `story/…` poussées se suppriment sur GitHub après la fusion de la release.
- **Une branche d'epic n'est pas une base** : un contributeur part de `main` ; bâtir sur une branche d'epic non publiée demande l'accord du Project Lead.
- **Un epic qui n'aboutit pas se réduit** : à la coupure, l'orchestrateur sort les stories non closes (`backlog`, pour une semaine suivante) et le dit au compte rendu de la PR ; le Project Lead garde le dernier mot. Il ne touche jamais à l'epic d'un contributeur.
- **Une story sortie emporte sa migration.** Une migration est annoncée d'avance au projet GitHub et dite en tête des notes ; les migrations voisines se regroupent quand c'est possible.
- **Chaque release planifiée corrige au moins un ou deux bugs**, dans une story de correction en plus de ses epics, comptée dans la borne de six : un P1 d'abord, sinon ce qui touche l'usage réel du Project Lead, sinon le plus ancien. Elle est petite et part en premier. Le dosage s'adapte au stock de bugs. Elle s'ajoute à la story de solde et à la correction des LOW, sans les remplacer.
- **Une release urgente** (`hotfix/X.Y.Z`) reste possible pour un bug majeur — P1 qui touche les livres ou la conformité, ou défaut qui bloque le Project Lead —, sur sa décision ; elle ne compte pas comme la release de la semaine.
- **Absence du Project Lead** : pas de release cette semaine-là ; il annonce ses dates d'avance, les stories continuent.

### Branches

| Branche ou tag | Rôle |
|---|---|
| `epic/N-slug` | un epic, par son développeur ; `story/…` en partent et y reviennent |
| `main` | intégration : ce qui est terminé, revu, et attend sa release |
| tag `vX.Y.Z` | production : le code exact de l'image publiée |
| `hotfix/X.Y.Z` | correctif urgent parti du tag, reporté ensuite dans `main` |

- Pas de branche `develop`.
- **Une branche par epic, fusionnée une fois** (le squash rend toute suite après fusion source de faux conflits).
- **Reprendre `main` dans sa branche au moins après chaque release publiée** par un autre.
- **On ne touche ni aux branches ni aux bases de test d'un autre.**

### Fusions et fichiers communs

- **Au kickoff**, le fichier d'epic déclare les fichiers et modules qu'il touchera ; deux epics sur le même module s'arbitrent avec le Project Lead.

| Fichier | Règle |
|---|---|
| `CHANGELOG.md` | pas écrit dans les stories : assemblé à l'intégration depuis les *Notes de version* des fiches |
| PDF des manuels | jamais commités dans `epic/…` ou `story/…` : seul le `.tex` change, les PDF se régénèrent à la release |
| catalogues i18n (quatre locales), `sprint-status.yaml` | chaque epic dans son bloc ; un conflit se résout en gardant les deux côtés |
| compteurs (`sitesTotal` de `i18n-keys.test.ts`, totaux de `docs/migrations-idempotence-audit.md`, schéma de test) | se recomptent depuis la source après la fusion |
| migrations | un fichier chacune, horodaté ; l'ordre se contrôle à l'intégration |
| `Cargo.lock` | se régénère après la fusion |

- **Le second arrivé résout, dans sa branche, jamais dans `main`**, en gardant les deux intentions ; s'il y a contradiction, il en parle à l'autre et le Project Lead arbitre.
- **Toute PR d'un epic vers `main` passe une revue du code d'intégration** : commits de fusion relus en diff combiné, et recherche des interactions sans conflit de texte (deux epics qui changent la même règle dans deux fichiers).

### CI

- `ci.yml` tourne aussi sur les PR vers `epic/**`.
- **Le cache Rust ne s'écrit que depuis `main`** (`save-if`, `shared-key`) ; les PR le lisent. Le cache Docker (`type=gha`) n'est pas restreint.
- Une CI allégée sur les PR vers `epic/**` reste à décider avec les contributeurs.
