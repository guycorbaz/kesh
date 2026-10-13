# Orchestration des agents — consignes communes et verrou de gate

Ce document porte les consignes que l'orchestrateur cite dans **chaque** prompt d'agent quand une epic
est menée en mode autonome, plusieurs stories en parallèle, chacune dans son worktree : agent de
fiche, de développement, lentille de revue, remédiateur. Elles ont été rodées sur les Epics 15, 26
et 27, et ne vivaient jusqu'au kickoff de l'Epic 28 que dans la mémoire de travail de
l'orchestrateur — c'est-à-dire hors de tout historique (perdues une fois dans `/tmp` au crash du
2026-10-09, reconstituées depuis le journal).

Elles **complètent** le `CLAUDE.md`, qui fait foi : en cas d'écart, c'est lui qui gagne, et l'écart
se corrige ici.

*(Versé au dépôt le 2026-10-13, Story 28-0 — action A8 de la rétrospective de l'Epic 15, B6 de celle
de l'Epic 26, Q5 de celle de l'Epic 27.)*

**Conventions de ce document.** `<clé>` désigne la clé courte de la story (`281` pour la 28-1) ;
`<worktree>` le répertoire du worktree assigné ; `$GATE_LOGS` le répertoire **commun** des journaux
de gate et des rapports de lentille, placé **à côté** des worktrees et hors de toute cible cargo
(sur la station de dev : `kesh-gate-logs/`, voisin de `kesh/` et des `kesh-<clé>/`).

---

## 1. Autonomie et registre des choix

1. **Aucune question** à Guy ni à l'orchestrateur en cours de travail : trancher, puis écrire le
   choix au registre de l'epic, `_bmad-output/implementation-artifacts/epic-<N>-choix-autonomes.md`
   — contexte, option retenue, alternatives écartées, réversibilité —, numéroté `C-<story>-<k>`.
2. **Le registre se met à jour par UNION**, jamais par écrasement : chaque branche y ajoute ses
   entrées, et la fusion réunit celles de `main` et de la branche. Un numéro en doublon se
   **renumérote**, il ne s'écrase pas. La même règle vaut pour `sprint-status.yaml`.
3. **Le rapport final** de l'agent porte : les commits produits, les gates **réellement exécutés**
   avec leurs chiffres, les choix consignés, et les actions **demandées** à l'orchestrateur (push,
   PR avec ses mots-clés `closes`/`refs`, issues à ouvrir).

## 2. Interdits absolus

Sauf autorisation écrite dans le prompt de l'agent :

- `git push`, et toute commande `gh` qui **écrit** (`pr`, `merge`, `issue create|edit|close`,
  `comment`) ;
- `scripts/prepare-release.sh`, `git tag`, `cargo publish` ;
- `git checkout`, `switch`, `reset --hard`, `rebase` **hors du worktree assigné** ;
- `docker restart`, `stop` ou `compose down` du MariaDB de dev partagé : son datadir est en tmpfs
  (cf. `docs/testing.md` § *Base de dev jetable*), l'arrêter **efface les bases de tous les
  agents** ;
- `pkill` ou `killall` à motif large — cf. § 4.

Le push, les PR, les merges et les issues reviennent à l'orchestrateur ; l'agent les **demande**.

⛔ **Interdire l'écriture ne suffit pas : il faut nommer les commandes qui écrivent.** Une lentille à
qui l'on avait dit « n'écris aucun fichier » a lancé `scripts/prepare-release.sh`, qui a bumpé les
dix crates (cf. `CLAUDE.md` § *Une passe qui ne déclare pas ses axes exercés…*). Une autre a
exécuté un script `python3` en lecture seule malgré l'interdit d'exécuter (Epic 27) : le prompt
d'une lentille nomme `python3` parmi les interdits.

## 3. Processus

Le `CLAUDE.md` **tel qu'écrit** : boucles `validate` et `review` jusqu'à 0 MEDIUM+ ou 8 passes ;
Sonnet ↔ Opus pour les passes complètes, Haiku pour la passe ciblée seulement ; gate complet **et**
E2E complet au **dernier commit de code** ; ne déclarer que ce qui a tourné ; recompter les
décomptes ; greper le symptôme — la **valeur** — après chaque patch ; un commit par étape BMAD,
messages `refs #N`.

- **Ne jamais réécrire l'entrée d'une passe passée** en propageant un décompte : un compte rendu de
  passe est un relevé daté, il se corrige quand il était faux **à sa date** — et alors par une
  annotation datée —, jamais pour le mettre à jour. *(Epic 27 : une propagation a réécrit l'entrée
  P3 d'un Change Log, qui disait juste à sa date, tout en oubliant la note vivante qu'elle devait
  mettre à jour.)*
- **Une passe dont le résultat n'est pas écrit dans la fiche n'a pas eu lieu**, même si son prompt
  est versionné : elle ne compte pas dans la boucle.

## 4. Travail parallèle : base, port, cible, processus

1. **Une base et un port par worktree.** Bases `kesh_<clé>` et `kesh_e2e_<clé>`, créées par l'agent,
   migrées depuis **les migrations du worktree** (`sqlx migrate run --source
   crates/kesh-db/migrations`), seedées par `scripts/seed-dev-db.sql` ; port `KESH_PORT` attribué
   par l'orchestrateur. `DATABASE_URL` en variable d'environnement, **jamais** dans un `.env`
   commité.
2. **Recette E2E** : celle de `docs/testing.md` § *Prérequis Playwright local*, **à la lettre** —
   `KESH_COOKIE_SECURE=false`, `KESH_HOST=127.0.0.1`, `KESH_TEST_MODE=true` des deux côtés,
   `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`, `KESH_BACKEND_URL` sur le port attribué,
   `KESH_INBOX_DIR`, `KESH_DOCUMENTS_DIR`, les quatre `KESH_SMTP_*`, et `KESH_ADMIN_BACKUP_DIR` vers
   un répertoire inscriptible (son défaut `/data/backup` ne l'est pas hors Docker). Un montage
   reconstitué de tête rend 18 échecs au lieu de la baseline.
3. **Une cible cargo par worktree** : `CARGO_TARGET_DIR=<worktree>/target`. Une cible partagée mêle
   les binaires de plusieurs branches (incident C82, Epic 15).
4. **Attendre le backend** en interrogeant `http://127.0.0.1:<port>/health` en boucle, avec délai ;
   identifier son processus par son **PID**, ou par un motif qui contient le port — jamais
   `pgrep kesh-api` nu.
5. **Tuer par PID uniquement.** Un `pkill -f kesh-api` a tué les backends des autres agents ; il tue
   aussi le shell appelant, dont la ligne de commande contient la chaîne cherchée.
6. **`node_modules` d'un nouveau worktree** : `cp -al` depuis le dépôt principal. Un lien
   symbolique casse Vitest.

## 5. Le verrou de gate — les gates lourds passent un par un

**Règle** : tout travail **lourd** s'exécute sous un verrou exclusif commun à tous les worktrees :

```sh
flock "$GATE_LOGS/.gate.lock" scripts/test-fast.sh
flock "$GATE_LOGS/.gate.lock" bash -c 'cmd1 && cmd2'     # plusieurs commandes
```

Sont lourds : `scripts/test-fast.sh` complet, `cargo build --workspace`, Vitest complet, E2E
Playwright complet, une campagne de mutations, le `make` des manuels. Hors `test-fast.sh` (qui l'est
déjà), ils passent aussi sous `scripts/mem-guard.sh`. Le **backend E2E** se lance **dans** le verrou
et s'arrête — par PID — **avant** de le relâcher. Restent hors verrou : `cargo fmt`, `cargo clippy`
et les gates ciblés `cargo nextest run -E 'binary(x)'`.

**Pourquoi.** Le plafond de `mem-guard.sh` choisit le mort à l'intérieur d'**un** gate ; il ne borne
pas la **somme** de plusieurs. Le 2026-10-11 à 01:40, trois agents en gate simultané ont saturé la
tranche utilisateur, et `systemd-oomd` a tué la session — GNOME Shell compris (cf. `CLAUDE.md`
§ *Plafonds mémoire*). Depuis la pose du verrou, sur toute la reprise de l'Epic 26 puis tout l'Epic
27, trois stories actives à la fois : **aucun OOM**.

⚠️ Le verrou sérialise les gates, il ne les rend pas plus courts : une file de trois gates complets
attend. C'est le prix, et il est bas devant une session perdue.

## 6. Lentilles de revue

- **Lecture seule** : aucune commande qui écrit, compile ou exécute, sauf mention contraire du
  prompt.
- **Rapport hors de toute cible cargo** : `$GATE_LOGS/<clé>-<validate|review>-p<n>-<R|F|B|E|A>.md`.
  **Jamais sous `target/`**, qu'un `cargo clean` efface — les rapports P1 de la 15-6c ont été perdus
  ainsi le 2026-10-09.
- Le rapport liste les **axes exercés et ceux qui ne l'ont pas été** ; toute affirmation de présence
  ou d'absence s'appuie sur une sortie `grep -nF` copiée.
- **Haiku** : ses MEDIUM qui reprochent au code **l'absence d'un livrable prescrit par la fiche**
  sont présumés **erreur de catégorie** — le livrable n'est pas encore écrit, ce n'est pas un défaut
  — et se vérifient au `grep -nF` avant tout patch. Quatre occurrences sur les Epics 15 et 27, toutes
  fausses. Vérifier un finding Haiku, c'est vérifier aussi **son remède**.

## 7. Édition du code et mutations

- **Le code s'édite par l'outil d'édition, jamais par substitution de motifs** : ni `perl -pi`, ni
  `sed -i`, ni script `python3` de remplacement sur un fichier source. Sur l'Epic 27, un seul `\|`
  mal échappé dans un `perl` a inséré du texte en tête de **chaque ligne** d'un composant Svelte.
  Les scripts restent permis pour les artefacts de `_bmad-output/` quand l'outil d'édition ne
  convient pas, et pour les mutations.
- **Mutations** : motif vérifié **unique** avant toute écriture ; restauration par
  `git checkout -- <fichier>` **puis `touch <fichier>`** avant le gate suivant. Cargo compare des
  dates, pas des contenus : une source restaurée depuis une copie faite **avant** le build muté
  (`cp f f.bak … mv f.bak f`) est juste mais plus ancienne que le binaire, qui n'est pas
  recompilé — le gate suivant mesure alors la mutation, pas le code (revue P1 de la 25-4-b1 : un
  gate complet perdu, `git diff HEAD` vide). Un rouge qui reproduit le message d'une mutation
  récente se diagnostique d'abord par les dates.

## 8. Données réelles

Le dépôt et ses issues sont **publics**. Rien d'un relevé ou d'une pièce réelle de l'utilisateur —
IBAN, numéro de compte, solde, ligne de transaction, ni même un agrégat qui en dérive — n'entre
dans le dépôt, un commit, une issue, une fiche, un registre ou un prompt versionné. Les fixtures
sont **synthétiques**, posées **avant** le premier diagnostic sur un fichier réel ; un contrôle sur
le fichier réel se fait en local et se rapporte en « conforme » ou « non conforme », sans chiffre.
Les journaux de `$GATE_LOGS` peuvent contenir de tels agrégats : ils ne se versent jamais au dépôt.
