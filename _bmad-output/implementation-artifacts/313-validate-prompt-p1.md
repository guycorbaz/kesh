# Prompt — validation P1 ciblée, Story 31-3 (Rust 1.97.1)

*Versionné le 2026-10-23. **Une seule lentille F**, contexte frais, **Haiku**. Lecture seule stricte. Pas de lentille R :
la fiche est courte et sans logique (C-31-3-1).*

Dépôt `/home/gcorbaz/devel/kesh-31-3`, branche `story/31-3`. Fiche : `_bmad-output/implementation-artifacts/31-3-rust-1-97-1.md`
; registre : `epic-31-choix-autonomes-313.md`. Règles : `CLAUDE.md`, `docs/agents-orchestration.md` § 6.

## Lentille F — les faits que la fiche affirme du dépôt

Pour **chaque fait** de la section « État des lieux » et de l'AC1, vérifie-le par `grep -nF` ou `sed -n` et **copie la
sortie**. Axes, tous à exercer :

1. **La version actuelle** : `rust-toolchain.toml` dit-il `1.96.1` ? Y a-t-il une autre configuration de chaîne
   (`.cargo/config.toml`, `rust-toolchain` sans extension, `rustup` dans `.github/workflows/*.yml`, `scripts/`) ?
2. **L'inventaire clos des sites** : `grep -rnE "1\.96" --exclude-dir=target --exclude-dir=node_modules --exclude-dir=.git
   --exclude=*.pdf .` — chaque ligne rendue est-elle dans la liste de la fiche (à bumper) ou classée historique/datée ?
   Cherche aussi `1.96.1`, `1\.9[0-9]`, `rust:`, `RUST_VERSION`, `msrv`, `rust-version` : un site oublié ?
3. **`Dockerfile`** (l. 2 et l. 4) et **CI** (`ci.yml` l. 75-79 ; `release.yml`, `codeql.yml`) : disent-ils ce que la fiche
   prétend ?
4. **`Cargo.toml`** : aucun `rust-version`, vraiment (workspace et dix crates) ?
5. **Cohérence interne** : AC1 à AC6, tâches T1-T7, registre C-31-3-1 à 5, *Hors périmètre* — une contradiction, un
   critère invérifiable, un nombre non recompté ? Le critère AC3 (« aucun `allow` nouveau ») est-il décidable par la
   commande donnée ?
6. **Manuel** : les deux `.tex` cités (`docs/manual/fr/admin-manual.tex`, `marketing-brochure.tex`) disent-ils bien
   « Rust 1.96+ » aux lignes données ? Contrôle le **PDF aplati** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n'
   ' ' | tr -s ' '` puis `grep -oF "Rust 1.96"` ; idem brochure.
7. **Entrées datées** : la fiche réécrit-elle une entrée passée ?
8. **Ce que la fiche omet** : un fichier versionné (docs/, scripts/, `.github/`, `docker-compose*.yml`, `website/`) qui
   fixe une version de Rust ou d'une image `rust:` et n'est pas listé.

**Haiku — garde-fous** : tout finding d'absence ou de présence s'appuie sur une sortie `grep -nF` copiée ; ce que la fiche
**prescrit** au développement (bump, build) n'existe pas encore : son absence n'est pas un défaut.

## Ce que tu rends

Findings `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, ligne du dépôt, preuve (commande et sortie
copiée), correction proposée. ⛔ **La liste des axes exercés ET non exercés** (1 à 8) — sans elle, « 0 finding » ne
compte pas. Rapport écrit **d'un seul jet**, un seul fichier :
`/home/gcorbaz/devel/kesh-gate-logs/313-validate-p1-F.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit,
compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`,
`docker`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`,
`gh` en écriture. `pdftotext` vers la sortie standard seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`).
Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log|show|diff|blame`, `pdftotext … -`.
