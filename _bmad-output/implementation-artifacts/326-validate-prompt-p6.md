# Prompt — validation P6, Story 32-6 : passe ciblée « regression hunter »

*Versionné le 2026-10-25. **Passe ciblée** (sixième passe sur huit ; `CLAUDE.md` § *La passe ciblée*) : **une seule
lentille**, en contexte frais, **Opus**, braquée sur **le seul commit de la remédiation P5**. Lecture seule stricte,
**sans `gh`**.*

## Ce que tu relis

- Dépôt `/home/gcorbaz/devel/kesh-32-6`, branche `story/32-6`.
- **Le commit visé, en dur** : `ac52657a16e101148e330417b0898d5814297483` (« Story 32-6 : remédiation de la validation
  P5 de la fiche »), dont le parent est `c1ec0f3b` (remédiation P4). Le diff à relire :
  `git diff c1ec0f3b ac52657a16e101148e330417b0898d5814297483 -- _bmad-output/implementation-artifacts/32-6-endpoint-mcp.md _bmad-output/implementation-artifacts/epic-32-choix-autonomes-326.md`
  (un **seul** diff, aplati : n'additionne pas des diffs intermédiaires).
- Pour juger le diff, la fiche entière à ce commit (`git show ac52657a16e101148e330417b0898d5814297483:_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md`)
  et le registre `epic-32-choix-autonomes-326.md` au même commit.
- Ce que la remédiation P5 corrige : `/home/gcorbaz/devel/kesh-gate-logs/326-validate-p5-R.md` et `…-p5-F.md`, et
  l'entrée « validation P5 » du Change Log de la fiche. **À tenir pour vraies** : les décisions de Guy (Q10 (c), Q11
  lecture seule applicative plus mode de session `READ ONLY`, Q12 par clé ; réseau local, filtre de défense en
  profondeur, `X-Forwarded-For` des seuls mandataires déclarés ; désactivé, `/mcp` indiscernable d'une route absente ;
  journal sans arguments ni données) et le montage de l'AC-a1 par `route_service` (P4, vérifié à la source).

## Ta lentille : la régression de la dernière remédiation

Le motif mesuré de ce dépôt : *la sévérité se déplace vers ce qu'on vient d'écrire.* Cherche les défauts que **le
commit `ac52657a` a introduits**, ou qu'il a laissés en changeant un texte sans en changer les voisins :

1. **Chaque hunk** : la nouvelle phrase est-elle vraie au code, contradictoire avec une autre partie de la fiche ou du
   registre, ou plus large qu'elle ne le dit ? En particulier :
   - Ma20b, ses deux variantes `post_service(…)` et `any_service(…)` : compilent-elles **telles qu'écrites** sur
     `main_router` (`Router<AppState>`), et le test nommé les tue-t-il chacune par le canal dit ?
   - la matrice de `desactive_indiscernable_d_une_route_absente` entièrement par `oneshot` : reste-t-il, ailleurs dans
     la fiche (conventions, AC-a3, AC-a9, Gates), une phrase qui la fait passer par une connexion réelle ?
   - l'entrée invalide de `trusted_proxies` qui **écarte la société de S** : cohérente avec l'AC-a2, l'AC-a4 (401 fixe
     après authentification) et les tests de l'AC-a9 (n) ?
   - la conversion des réseaux IPv6 mappés (`::ffff:0:0/96`, préfixe `p − 96`) ; le port par défaut **du schéma** ;
     `400` pour toute autre erreur de lecture du corps (et la façon de distinguer le dépassement) ;
   - `tower` avec `util` en dépendance normale dans `crates/kesh-api/Cargo.toml` — le module compté, la convention du
     découpage ;
   - les AC-b6 et AC-a31 réécrits ; la ligne `route_service` avant le marqueur `// NOTE: …` de `lib.rs` ;
   - `params` mal formés → `-32602` et l'issue `arguments_invalides` : cohérent avec l'énumération fermée d'`issue` de
     l'AC-b1 et avec les tests nommés de la 32-6b ?
2. **Propagation** (`CLAUDE.md` § *Propagation post-patch*) : pour chaque valeur que le commit change (`"post"` →
   `"route_service"`, « deux gestionnaires », « onze », `:80`/`:443`, « jamais élargissante », `any(…)`), greppe la
   **valeur** sur la fiche et le registre au commit visé, et nomme tout site resté à l'ancienne valeur hors des
   mentions historiques datées.
3. **Recompte** : l'entrée « validation P5 » du Change Log — trend, doublons, « deux MEDIUM nés de la P4 », 59
   mutations, C-32-6-1 à 41 — recomptée à la source.

## Les deux axes obligatoires (inchangés)

- ⛔ **Motifs greppés sur le dépôt réel** : chaque motif que le commit ajoute ou modifie, et chaque mesure qu'il écrit
  (`grep -n "pub async fn lister" crates/kesh-api/src/services/*.rs` → 6, par exemple), est rejoué dans son
  périmètre, sortie copiée ; une mesure que ton grep ne reproduit pas est un finding.
- ⛔ **Chaque affirmation sur Axum, axum-core, tower-http, hyper ou sqlx se prouve à leur source**
  (`~/.cargo/registry/src/*/axum-0.8.9/`, `axum-core-0.5.6/`, `tower-http-0.6.11/`, `hyper-1.10.1/` — la version du
  `Cargo.lock`, `sqlx-core-0.8.6/`), ligne lue et copiée : notamment `post_service`, `any_service` et
  `skip_allow_header` (`routing/method_routing.rs`), `set_content_length` (`routing/route.rs`), le `content-length: 0`
  d'hyper (`src/proto/h1/role.rs:924-931`), l'erreur de `axum::body::to_bytes` (`body/mod.rs`), l'unification de la
  fonctionnalité `util` de `tower` (`axum-0.8.9/Cargo.toml`). Une affirmation dont tu ne trouves pas la ligne est un
  finding.

## Ce que tu rends

Findings `H1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, hunk ou ligne de la fiche, site du dépôt ou de la dépendance,
preuve (commande et sortie copiée), correction proposée — et, pour chacun, s'il est **né du commit `ac52657a`** ou
antérieur. Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET non exercés** (1, 2, 3 et
les deux axes obligatoires) — sans elle, « 0 finding » ne compte pas comme passe. Rapport écrit **d'un seul jet**, dans
**un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/326-validate-p6.md`. Dernier message : chemin, bilan par
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
