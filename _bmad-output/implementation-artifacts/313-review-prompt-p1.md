# Prompt — revue de code P1 de la Story 31-3 (Rust 1.97.1, garde G21), trois lentilles, Sonnet

Tu relis le développement de la **Story 31-3** de Kesh (comptabilité suisse) : passage de la chaîne d'outils à
**Rust 1.97.1**, correctif des lints nouveaux de clippy 0.1.97, garde **G21** (#697) qui compare le `FROM rust:` du
`Dockerfile` au `channel` de `rust-toolchain.toml`, documentation et manuels. Worktree `/home/gcorbaz/devel/kesh-31-3`,
branche `story/31-3`, base `origin/main` = `44ef2079`. Le diff à relire, **aplati** (un seul diff, pas la suite des
commits) : `git -C /home/gcorbaz/devel/kesh-31-3 diff origin/main...HEAD`. Langue du rapport : français.

Lis d'abord : la fiche `_bmad-output/implementation-artifacts/31-3-rust-1-97-1.md` (AC1 à AC7, Dev Agent Record,
Mutations) et le registre `_bmad-output/implementation-artifacts/epic-31-choix-autonomes-313.md`. Le Dev Agent Record
déclare des gates et des chiffres : **ne les prends pas pour argent comptant**, recoupe-les avec les journaux
`/home/gcorbaz/devel/kesh-gate-logs/313-*.log` (lecture seule).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. Aucun fichier temporaire : compare par
  des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`,
  ni `scripts/mem-guard.sh`, ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande), ni `rustc`, ni `npm`, ni
  `docker`, ni `python3`, ni `sed -i`, ni `perl`. Aucune commande `gh` qui écrit ; `gh issue view` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`, `gh issue view 697`.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. `crates/kesh-api/tests/textes_coherents.rs` : `version_rust`, `ligne_unique`, `rust_versions_accordees`, les deux
   tests G21. Les expressions régulières font-elles ce que disent leurs commentaires ? Ordre des contrôles, messages,
   `LazyLock`, `let … && …` (édition 2024), doublons avec des helpers existants du même fichier ou de
   `configuration_transmise.rs` (DRY) ? Le test du dépôt est-il adossé à des positives qui empêchent un vert muet ?
2. Les deux correctifs de lint (`crates/kesh-report/src/csv.rs`, `crates/kesh-api/tests/reports_export_e2e.rs`) : le rendu
   de l'assertion est-il inchangé ? Le type de l'argument change-t-il quelque chose au `{:?}` ?
3. `rust-toolchain.toml`, `Dockerfile` : cohérents entre eux et avec la CI (`.github/workflows/ci.yml`, `release.yml`) ?
4. Documentation : README, `website/index.html`, `DOCKER_START.md`, les deux `.tex` et **leurs PDF**
   (`docs/manual/fr/admin-manual.pdf`, `docs/manual/fr/marketing-brochure.pdf`), contrôlés aplatis : « Rust 1.97+ »
   présent, « 1.96 » absent ; `user-manual.pdf` non modifié (`git diff --stat origin/main...HEAD -- docs/manual`).

## Lentille E — Edge cases (chaque branche de G21)

1. Parcours chaque branche de `rust_versions_accordees` et dis, pour chacune, quel cas des seize de
   `g21_la_regle_d_accord_dockerfile_toolchain` l'exerce, ou qu'aucun ne l'exerce. Branches à couvrir au moins : zéro /
   une / plusieurs lignes `FROM rust:` ; ligne `FROM rust:` reconnue mais « illisible » (ex. `FROM rust:1.97 AS`, ou un
   troisième mot) ; tag sans mineure ; tag à suffixe exotique (`1.97-slim-bookworm`, `1.97.1-alpine3.20`) ; zéro / une /
   plusieurs lignes `channel` ; `channel` illisible (sans guillemets, guillemets mélangés `"1.97'`) ; `channel` non
   numérique ; mineure différente ; patch différent quand les deux en portent ; patch d'un seul côté.
2. Formes réelles d'un `Dockerfile` ou d'un `rust-toolchain.toml` qui échapperaient à la règle **en rendant un vert
   faux** (le seul mode d'échec grave d'une garde) : `FROM --platform=…`, `ARG RUST_VERSION` puis `FROM rust:${…}`,
   digest `@sha256:`, continuation `\`, tabulations, `\r\n`, BOM, `[toolchain]` en table en ligne, `channel` dans une
   autre table que `[toolchain]`, `rust-toolchain` (sans extension) présent à côté. Pour chacune : vert faux, refus
   explicite ou sans objet — et l'angle mort est-il écrit dans la doc-comment ?
3. La mutation déclarée (`Dockerfile` → `rust:1.96-bookworm`) fait-elle rougir G21 **et elle seule** ? Une autre
   mutation plausible (le `Dockerfile` à `rust:1.97.0-bookworm`, `rust-toolchain.toml` à `"stable"`) serait-elle tuée ?
   Raisonne sur le code, n'exécute rien.
4. Cas limites du correctif de lint : `body.get(..16).unwrap_or(&body[..body.len().min(16)])` quand `body` est vide ou
   plus court que 16 octets.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC1 à AC7, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve :

- **AC1** : rejoue les deux commandes d'inventaire de la fiche (elles sont en lecture seule) et compare leur sortie à
  l'ensemble admis, nommé par chemins ; PDF aplatis.
- **AC2** : les chiffres du Dev Agent Record (nextest, `cargo publish --dry-run`, quatre gates frontend, E2E et liste des
  échecs) correspondent-ils **aux journaux** `313-t6-*.log` ? Les échecs E2E sont-ils jugés fichier par fichier contre
  `docs/testing.md` § *Les échecs attendus* ? Le gate et l'E2E ont-ils tourné **au dernier commit de code** (compare le
  commit cité au `git log --oneline origin/main..HEAD` : les commits postérieurs ne doivent toucher que de la
  documentation) ?
- **AC3** : rejoue la commande d'inventaire des exceptions (pathspec de la fiche) et le `find … clippy.toml` ; zéro
  attendu. Le relevé « un lint, deux sites » est-il étayé par `313-t1-clippy-1971-full.log` ?
- **AC4** : `git diff --stat origin/main...HEAD -- Cargo.lock` vide ? Le diff de production (hors tests, docs,
  `_bmad-output`) est-il vide ou mécanique ?
- **AC5** : le constat du `docker build` (T7) est-il écrit, avec la chaîne réellement employée par l'image (journal
  `313-t7-*.log`) ? Le smoke CI relève de l'orchestrateur : « non vérifiable » est une réponse admise.
- **AC6** : le Dev Agent Record dit-il ce qui a été nettoyé et par qui, sans que le dev ait touché aux worktrees des
  autres ?
- **AC7** : la règle écrite dans la fiche (casse, suffixe facultatif, `channel` commenté, guillemets simples, deux
  `channel`, comparaison des patchs) est-elle **exactement** celle du code ? Chaque cas « rouge d'abord » de la fiche
  figure-t-il parmi les seize ? La ligne G21 du tableau d'en-tête de `textes_coherents.rs` est-elle juste ?
- **CHANGELOG** : une seule section `## [0.15.0] — Non publié`, placée sous le `---` qui précède `## [0.14.0]`, avec la
  ligne de la chaîne sous `### Changed` ; aucune mention historique de « Rust 1.96 » (v0.4.0) réécrite.
- **Manuel** : les manuels FR et **leurs PDF** disent-ils vrai sur la chaîne requise ? Le README « Prérequis » dit
  « installé automatiquement via `rust-toolchain.toml` » : est-ce toujours vrai ?

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/313-review-p1-<B|E|A>.md` (selon ta lentille) :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `B1…`/`E1…`/`A1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`,
  `sed -n` ou `git diff` — toute affirmation de présence ou d'absence s'appuie sur une sortie), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- rien d'autre n'est écrit nulle part.
