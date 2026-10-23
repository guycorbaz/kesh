# Prompt — validation P3 complète, Story 31-3 (Rust 1.97.1)

*Versionné le 2026-10-23. **Passe COMPLÈTE**, contexte frais, **Sonnet**, **deux lentilles, R et F**, lancées chacune dans
son propre contexte. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-31-3`, branche `story/31-3` (reportée sur `origin/main`, `44ef2079` ; tout diff se borne à
`origin/main...HEAD`). Fiche : `_bmad-output/implementation-artifacts/31-3-rust-1-97-1.md` ; registre :
`_bmad-output/implementation-artifacts/epic-31-choix-autonomes-313.md`. Règles : `CLAUDE.md`, `docs/agents-orchestration.md`
§ 6. Contexte : la P1 (Haiku, F) a rendu 0/0/5/1 ; la P2 (Opus, R et F) 0/0/6/9 et 0/0/1/6, tous confirmés et corrigés (voir
le Change Log de la fiche). **Ne tiens pas la P2 pour acquise** : une remédiation introduit des défauts, la fiche a été
réécrite en profondeur (ordre des tâches T1 à T8 renumérotées, AC1/AC3/AC7 réécrites, C-31-3-8 et C-31-3-9 nouveaux). Cherche
d'abord ce que la remédiation a **cassé ou laissé** : renvois de tâches périmés (« T3-bis », « T4 », « T7 »), valeurs
non propagées, critères non décidables.

## Lentille R — cohérence et faisabilité

1. **Cohérence interne** : AC1 à AC7, tâches T1 à T8, registre C-31-3-1 à 9, *Mutations*, *Hors périmètre*, Change Log. Tout
   renvoi de tâche (`T3`, `T4`, `T5`, `T7`…) pointe-t-il la bonne tâche après la renumérotation ? Un compte (sept sites de
   documentation, 15 arbres de travail, 4 commits) est-il recompté ?
2. **Ordre des tâches (C-31-3-9)** : `rust-toolchain.toml` bumpé en T2, correctifs en T3, G21 + `Dockerfile` en T4. Chaque
   commit compile-t-il sous la chaîne qu'il porte ? Le « rouge » de G21 sur le dépôt est-il réel à T4 (1.96 contre 1.97.1) ?
   Le gate ciblé de T3 tourne-t-il sous 1.97.1 ?
3. **Garde G21 (AC7, C-31-3-8)** : la règle est-elle complète et sans ambiguïté (une seule ligne `FROM rust:` non commentée,
   formes `X.Y[.Z]-suffixe`, `AS`, `channel` numérique, comparaison de mineure puis de patch si les deux en portent) ? Les
   cas de la fonction pure couvrent-ils chaque branche de rouge ? Lis `crates/kesh-api/tests/textes_coherents.rs`
   (en-tête, helpers `racine`/`lire`, G19) et `Dockerfile`, `rust-toolchain.toml` : réalisable avec `regex` ? L'emplacement
   (C-31-3-8) est-il défendu à raison ? La mutation fait-elle rougir G21 et elle seule ?
4. **Décidabilité des inventaires** : les commandes d'AC1 (forme) et d'AC3 (lignes ajoutées, attributs) sont-elles
   syntaxiquement valides (ERE de `ugrep`/GNU grep), et rendent-elles bien ce que la fiche attend ? L'ensemble admis d'AC1
   est-il nommé par chemins et exact ?
5. **Règles du dépôt** : `refs` seulement dans les commits, `closes #697` sur la PR ; pas de migration ; `Cargo.lock` ; gates
   du `CLAUDE.md` (quatre gates frontend, `cargo publish --dry-run -p kesh-import`, exception `kesh-db`, `flock`,
   `mem-guard`, base remise à zéro) ; T8 et la section `## [0.15.0] — Non publié` du CHANGELOG (forme exigée par
   `scripts/prepare-release.sh`).
6. **Registre** : Contexte / Retenu / Écartée(s) / Réversible pour chaque choix ; C-31-3-1 (annotation datée), C-31-3-6
   (Justification) cohérents avec la fiche et avec `docs/agents-orchestration.md` § 3.
7. **Le manuel** (axe nommé) : la fiche promet-elle quelque chose sur les manuels (`Rust 1.97+`, PDF régénérés, macros
   `\keshVersion` intactes) que le code ou le `Makefile` de `docs/manual/` contredit ? Le contrôle du **PDF aplati** est-il
   prévu ?

## Lentille F — les faits que la fiche affirme du dépôt

Pour **chaque fait** de « État des lieux », de l'AC1 et de l'AC7, vérifie-le par `grep -nF` / `grep -nE` / `sed -n` et
**copie la sortie**. Axes :

1. **Version actuelle** : `rust-toolchain.toml` à `1.96.1` ? Autre configuration de chaîne (`.cargo/config.toml`,
   `rust-toolchain`, workflows, `scripts/`, `DOCKER_START.md`, compose) ?
2. **Inventaire clos sur la forme** : exécute les deux commandes d'AC1 telles qu'écrites et compare leur sortie à
   l'ensemble admis et à la liste des sept sites de documentation ; un site manquant ? un site listé qui n'existe pas à la
   ligne dite ? `DOCKER_START.md:192` dit-il bien « Rust: 1.85 » ?
3. **`Dockerfile`** (l. 2, 4, 6, les trois `FROM`), **CI** (`ci.yml` l. 75-79 ; `release.yml`, `codeql.yml`).
4. **`Cargo.toml`** : aucun `rust-version`, dix crates en édition 2024.
5. **Worktrees** : `git worktree list` et lecture des `rust-toolchain.toml` : « 15 arbres de travail, dépôt principal
   compris, tous sur 1.96.1 » est-il exact ?
6. **Manuel** : `admin-manual.tex` l. 106 et `marketing-brochure.tex` l. 343 ; PDF aplatis
   (`pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '` puis `grep -oF "Rust 1.96"`).
7. **Garde G21** : le tableau d'en-tête de `textes_coherents.rs` (G21 libre) ; les affirmations sur
   `configuration_transmise.rs` (C-31-3-8 : `lire("Dockerfile")` l. 1897, `dockerfile_fixe` l. 468-480) ; `gh issue view 697`.
8. **CHANGELOG / `prepare-release.sh`** : l'en-tête `## [0.15.0] — Non publié` et la règle « une seule section » sont-ils
   tels que T8 le dit (`scripts/prepare-release.sh` lu, non exécuté) ? Première section du CHANGELOG ?
9. **Entrées datées** : la fiche réécrit-elle une entrée passée (CHANGELOG v0.4.0, tableau de versions, entrée de Change
   Log) ?
10. **Ce que la fiche omet** : un fichier versionné qui fixe une version de Rust ou une image `rust:` sans être listé.

## Ce que tu rends

Findings `<lentille><n>` (R1…, F1…), sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, ligne du dépôt, preuve
(commande et sortie copiée), correction proposée. ⛔ **La liste des axes exercés ET non exercés** de ta lentille — sans
elle, « 0 finding » ne compte pas (un « 0 » se vérifie comme un finding). Rapport écrit **d'un seul jet**, un seul fichier
par lentille : `/home/gcorbaz/devel/kesh-gate-logs/313-validate-p3-R.md` et
`/home/gcorbaz/devel/kesh-gate-logs/313-validate-p3-F.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

Ce que la fiche **prescrit** au développement (bump, build, garde) n'existe pas encore : son absence n'est pas un défaut.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit,
compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`,
`docker`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`,
`gh` en écriture. `pdftotext … -` vers la sortie standard seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`).
Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log|show|diff|blame|worktree list`, `gh issue view`, `pdftotext … -`.
