# Prompt — validation P7, Story 32-6 : passe ciblée « regression hunter »

*Versionné le 2026-10-25. **Passe ciblée** (septième passe sur huit — **il n'en restera qu'une** ; `CLAUDE.md` § *La passe
ciblée*) : **une seule lentille**, en contexte frais, **Sonnet** (la P6 était Opus), braquée sur **le seul commit de la
remédiation P6**. Lecture seule stricte,
**sans `gh`**.*

## Ce que tu relis

- Dépôt `/home/gcorbaz/devel/kesh-32-6`, branche `story/32-6`.
- **Le commit visé, en dur** : `3b884bd742e8d0b228d25ba9f30f00ba43cc75ca` (« Story 32-6 : remédiation de la validation
  P6 de la fiche »), dont le parent est `07a72200` (prompt de la P6, qui suit la remédiation P5 `ac52657a`). Le diff à
  relire :
  `git diff 07a72200 3b884bd742e8d0b228d25ba9f30f00ba43cc75ca -- _bmad-output/implementation-artifacts/32-6-endpoint-mcp.md _bmad-output/implementation-artifacts/epic-32-choix-autonomes-326.md _bmad-output/implementation-artifacts/sprint-status.yaml`
  (un **seul** diff, aplati : n'additionne pas des diffs intermédiaires).
- Pour juger le diff, la fiche entière à ce commit (`git show 3b884bd742e8d0b228d25ba9f30f00ba43cc75ca:_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md`)
  et le registre `epic-32-choix-autonomes-326.md` au même commit.
- Ce que la remédiation P6 corrige : `/home/gcorbaz/devel/kesh-gate-logs/326-validate-p6.md` (rapport de la P6, H1 à
  H9), et l'entrée « validation P6 » du Change Log de la fiche. **À tenir pour vraies** : les décisions de Guy (Q10 (c), Q11
  lecture seule applicative plus mode de session `READ ONLY`, Q12 par clé ; réseau local, filtre de défense en
  profondeur, `X-Forwarded-For` des seuls mandataires déclarés ; désactivé, `/mcp` indiscernable d'une route absente ;
  journal sans arguments ni données) et le montage de l'AC-a1 par `route_service` (P4, vérifié à la source).

## Ta lentille : la régression de la dernière remédiation

Le motif mesuré de ce dépôt : *la sévérité se déplace vers ce qu'on vient d'écrire.* Cherche les défauts que **le
commit `3b884bd7` a introduits**, ou qu'il a laissés en changeant un texte sans en changer les voisins. C'est
l'avant-dernière passe : un défaut que tu laisses ne sera relu que par une passe de plus.

1. **Chaque hunk** : la nouvelle phrase est-elle vraie au code, contradictoire avec une autre partie de la fiche ou du
   registre, ou plus large qu'elle ne le dit ? En particulier :
   - **la règle de `trusted_proxies`** (une entrée invalide écarte la société de S, journalisée en `error!`) :
     cohérente partout — AC-a2 (S), AC-a4 (une clé de cette société reçoit le 401 fixe), AC-a8, AC-a9 (n') et son
     témoin, Ma13 et Ma13b, la liste de la 32-7, C-32-6-6 annoté ? Le test (n') éprouve-t-il vraiment l'exclusion de
     **toute** la société (le pair `192.168.1.10` sans en-tête), et Ma13b y meurt-elle ?
   - la demande mal formée au **répartiteur** (AC-a19) : compatible avec l'ordre des contrôles des Dev Notes (étape 10,
     JSON-RPC ; étape 11, répartiteur), avec l'AC-a12 (liste close), l'AC-b1 (issue `arguments_invalides`, `outil` =
     `"(inconnu)"`) et l'AC-b4 ?
   - les témoins de `desactive_indiscernable_d_une_route_absente` par `oneshot` : reste-t-il une phrase (conventions,
     AC-a3, Gates) qui les met sur une vraie connexion ?
   - les citations neuves : `method_routing.rs:514`, `:436`, `:1168-1171`, `route.rs:164`, `hyper-1.10.1/src/proto/h1/
     role.rs:512-521`, `config.rs:443`, `:502`, `tests/accounts_e2e.rs:51`, `tests/api_keys_e2e.rs:47` ;
   - la convention du découpage (`Cargo.toml` avec `routes`) et la ligne `32-6a` de `sprint-status.yaml` ;
   - l'annotation H7 du Change Log (comptage de R12) : est-elle une annotation datée, non une réécriture de l'entrée
     P5 (`CLAUDE.md` § *Recompter n'est pas réécrire l'historique*) ?
2. **Propagation** (`CLAUDE.md` § *Propagation post-patch*) : pour chaque valeur que le commit change (« est
   ignorée » / « ignor », `debug!` → `error!`, « 200 », « les gestionnaires », `:1168-1170`, `:1623`, `test_config()`),
   greppe la **valeur** sur la fiche et le registre au commit visé ; vérifie le tri écrit au Change Log P6 (sites
   corrigés, sites laissés) et nomme tout site oublié.
3. **Recompte** : l'entrée « validation P6 » — trend, « le MEDIUM H1 et six des huit LOW nés du commit P5 », **60
   mutations** (54 / 6), 37 AC, 11 tâches, C-32-6-1 à 41 — recomptée à la source.

## Les deux axes obligatoires (inchangés)

- ⛔ **Motifs greppés sur le dépôt réel** : chaque motif que le commit ajoute ou modifie, et chaque mesure qu'il écrit
  (notamment les lignes de `config.rs`, `tests/accounts_e2e.rs`, `tests/api_keys_e2e.rs`, et l'absence de toute
  fonction de configuration dans `tests/common/`), est rejoué dans son périmètre, sortie copiée ; une mesure que ton
  grep ne reproduit pas est un finding.
- ⛔ **Chaque affirmation sur Axum, axum-core, tower-http, hyper ou sqlx se prouve à leur source**
  (`~/.cargo/registry/src/*/axum-0.8.9/`, `axum-core-0.5.6/`, `tower-http-0.6.11/`, `hyper-1.10.1/` — la version du
  `Cargo.lock`, `sqlx-core-0.8.6/`), ligne lue et copiée : notamment `post_service`, `any_service` et
  `skip_allow_header` (`routing/method_routing.rs`), `set_content_length` (`routing/route.rs`), le `content-length: 0`
  d'hyper (`src/proto/h1/role.rs:924-931`), l'erreur de `axum::body::to_bytes` (`body/mod.rs`), l'unification de la
  fonctionnalité `util` de `tower` (`axum-0.8.9/Cargo.toml`). Une affirmation dont tu ne trouves pas la ligne est un
  finding.

## Ce que tu rends

Findings `H1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, hunk ou ligne de la fiche, site du dépôt ou de la dépendance,
preuve (commande et sortie copiée), correction proposée — et, pour chacun, s'il est **né du commit `3b884bd7`** ou
antérieur. Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET non exercés** (1, 2, 3 et
les deux axes obligatoires) — sans elle, « 0 finding » ne compte pas comme passe. Rapport écrit **d'un seul jet**, dans
**un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/326-validate-p7.md`. Dernier message : chemin, bilan par
sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, **`scripts/test-fast.sh`** et
**`scripts/regen-test-schema.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, `wget`,
**`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, et **`gh` sous toutes ses formes**. `pdftotext` vers la sortie
standard seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh`
est hors de ton périmètre. Autorisés : lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`,
`pdftotext … -`, et les filtres en lecture seule `tr`, `sort`, `cut`, `head`, `tail`, `awk`, `wc`, `uniq` vers la
sortie standard.
