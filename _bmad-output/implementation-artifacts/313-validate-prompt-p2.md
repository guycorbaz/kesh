# Prompt — validation P2 complète, Story 31-3 (Rust 1.97.1)

*Versionné le 2026-10-23. **Passe COMPLÈTE** (la fiche a changé de périmètre : la garde G21 de #697 y est entrée, AC7,
T3-bis, C-31-3-6), contexte frais, **Sonnet**, **deux lentilles, R et F**, lancées chacune dans son propre contexte.
Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-31-3`, branche `story/31-3`. Fiche : `_bmad-output/implementation-artifacts/31-3-rust-1-97-1.md`
; registre : `_bmad-output/implementation-artifacts/epic-31-choix-autonomes-313.md`. Règles : `CLAUDE.md`,
`docs/agents-orchestration.md` § 6. Contexte de la passe précédente : P1 (Haiku, lentille F) a rendu 0/0/5/1, tous
confirmés et corrigés (voir le Change Log de la fiche) — **ne la tiens pas pour acquise, elle ne te dit pas ce qu'elle a
manqué**. La branche est fondée sur `chore/retros-29-30` et sera reportée sur `main` ; tout diff se borne à
`origin/main...HEAD`.

## Lentille R — cohérence et faisabilité

1. **Cohérence interne** : AC1 à AC7, tâches T1 à T7 (dont T3-bis), registre C-31-3-1 à 6, *Mutations*, *Hors
   périmètre*, Change Log. Une contradiction (ordre des tâches, conduite d'un lint non mécanique, périmètre de l'inventaire
   d'AC3), un critère invérifiable, un nombre non recompté ?
2. **Faisabilité de la garde G21 (AC7)** : lis `crates/kesh-api/tests/textes_coherents.rs` (en-tête et tableau des gardes,
   helpers `lire`, `racine`, G19 comme modèle). La garde est-elle réalisable avec ses helpers ? Le « rouge d'abord » par
   fonction pure sur un désaccord simulé est-il bien un rouge **avant** la lecture du dépôt ? La comparaison de la seule
   version mineure est-elle juste pour `FROM rust:1.97-bookworm` et `channel = "1.97.1"` (et pour un `channel` sans
   patch) ? Un `Dockerfile` à plusieurs `FROM`, un `FROM` avec `AS`, un tag sans mineur : cas non traités ? La mutation
   prévue fait-elle réellement rougir la garde (et pas une autre) ?
3. **Faisabilité du bump** : ordre T1 → T3-bis → T3 → T5 ; les gates annoncés (`scripts/test-fast.sh`, Vitest, E2E)
   sont-ils ceux que le `CLAUDE.md` exige ? **Exception `kesh-db`**, verrou `flock`, `mem-guard`, remise à zéro de la base
   : mentionnés là où il faut ? L'AC3 est-il décidable par la commande donnée (motif, bornes de chemins, `origin/main...HEAD`) ?
4. **Règles du dépôt** : « refs » seulement dans les commits, `closes #697` sur la PR ; pas de migration (donc P2 à P8 sans
   objet) ; `Cargo.lock` qui ne bouge pas (AC4) ; une nouvelle garde de texte exige-t-elle une ligne au tableau
   d'en-tête du fichier ?
5. **Registre** : chaque choix a-t-il Contexte / Retenu / Écartées / Réversible ? C-31-3-1 (révisé) et C-31-3-6 sont-ils
   cohérents avec la fiche ?

## Lentille F — les faits que la fiche affirme du dépôt

Pour **chaque fait** de « État des lieux », de l'AC1 et de l'AC7, vérifie-le par `grep -nF` / `grep -nE` / `sed -n` et
**copie la sortie**. Axes :

1. **Version actuelle** : `rust-toolchain.toml` dit-il `1.96.1` ? Autre configuration de chaîne (`.cargo/config.toml`,
   `rust-toolchain` sans extension, `.github/workflows/*.yml`, `scripts/`) ?
2. **Inventaire clos des sites** : `grep -rnE "\b1\.96\b" --exclude-dir=target --exclude-dir=node_modules
   --exclude-dir=.git --exclude=*.pdf .` ; chaque ligne est-elle à bumper (listée) ou historique/datée ? Cherche aussi
   `1\.9[0-9]`, `rust:`, `RUST_VERSION`, `msrv`, `rust-version` : un site oublié ? Le motif corrigé de la fiche
   (`\b1\.9[67]\b` sur `docs/ CLAUDE.md scripts/`) rend-il bien ce que la fiche dit ?
3. **`Dockerfile`** (l. 2 et l. 4), **CI** (`ci.yml` l. 75-79 ; `release.yml`, `codeql.yml`).
4. **`Cargo.toml`** : aucun `rust-version`, workspace et dix crates ?
5. **Worktrees** : `git worktree list` et lecture des `rust-toolchain.toml` : « 15 worktrees, tous sur 1.96.1 » est-il exact ?
6. **Manuel** : `admin-manual.tex` l. 106 et `marketing-brochure.tex` l. 343 disent-ils « Rust 1.96+ » ? PDF aplati :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` puis `grep -oF "Rust 1.96"` ; idem brochure.
7. **Garde G21** : le tableau d'en-tête de `textes_coherents.rs` (G1 à G20) est-il tel que la fiche le suppose (G21 libre,
   `closes #697` sur la PR) ? `gh issue view 697` (lecture seule) dit-il ce que le registre prétend ?
8. **Entrées datées** : la fiche réécrit-elle une entrée passée (CHANGELOG v0.4.0, tableau de versions) ?
9. **Ce que la fiche omet** : un fichier versionné qui fixe une version de Rust ou une image `rust:` sans être listé.

## Ce que tu rends

Findings `<lentille><n>` (R1…, F1…), sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, ligne du dépôt, preuve
(commande et sortie copiée), correction proposée. ⛔ **La liste des axes exercés ET non exercés** de ta lentille — sans
elle, « 0 finding » ne compte pas. Rapport écrit **d'un seul jet**, un seul fichier par lentille :
`/home/gcorbaz/devel/kesh-gate-logs/313-validate-p2-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/313-validate-p2-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

Ce que la fiche **prescrit** au développement (bump, build, garde) n'existe pas encore : son absence n'est pas un défaut.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit,
compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`,
`docker`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`,
`gh` en écriture. `pdftotext … -` vers la sortie standard seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`).
Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log|show|diff|blame|worktree list`, `gh issue view`, `pdftotext … -`.
