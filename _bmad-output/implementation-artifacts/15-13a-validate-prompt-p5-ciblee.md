# Prompt — validation P5 ciblée de la spec, Story 15-13a

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur la seule remédiation P4 de la fiche 15-13a. Aucune règle métier n'a changé.*

Worktree `/home/gcorbaz/devel/kesh-15-13`, branche `story/15-13-mariadb-et-sauvegarde`. **Objet** :
`git show 3ebedca9 -- _bmad-output/implementation-artifacts/15-13a-mariadb-non-publiee-mots-de-passe-obligatoires.md`
(lis ce diff ; ouvre la fiche autour des hunks). Rapports remédiés : `target/gate-logs/15-13a-p4-{R,F}.md`.
Choix C-15-13-22, C-15-13-23.

⚠️ **La story n'est pas développée** : tu relis une SPÉCIFICATION. Un test, un texte, une mutation ou une
constante que la fiche prescrit et qui n'existe pas encore dans le code n'est PAS un défaut. Un défaut est une
contradiction, une prescription inexécutable, un fait cité faux, ou un site oublié.

## Lentille unique — chasseur de régressions de la remédiation

1. **Grep restreint du T9 (1)** `(compose|décrit les)[^.]{0,20}deux gestes` : rejoue-le
   (`git grep -nE "…" -- . ':(exclude)_bmad-output'`) et compare au texte de la fiche (seuls `CHANGELOG.md:46`
   et `admin-manual.tex:1793` attendus). Rejoue aussi le grep (1) entier du T9 : un site manque-t-il à
   l'inventaire (48 lignes) ?
2. **`decode_utf8_lossy()` (C-15-13-22)** et le témoin `%FF` du test 7 : cohérent avec l'AC 5 (b) (le démarrage
   n'échoue pas) ? La crate `url`/`percent-encoding` fournit-elle bien ce décodage (lis
   `~/.cargo/registry/src/*/percent-encoding-*/src/lib.rs`) ?
3. **M43, M44** : chacune fait-elle rougir le test qu'elle vise, telle qu'écrite ?
4. **Recette `ALTER USER` (C-15-13-23)** : le mot de passe n'apparaît-il vraiment ni dans l'historique du shell
   ni dans `ps` (variable lue au clavier, puis entrée standard) ? La recette est-elle exécutable telle quelle
   (`docker compose exec -T`, guillemets) ?
5. **Recomptes** (11 AC, 11 tâches, 9 lignes de test, 24 mutations, 10 cas, 48 lignes d'inventaire, 5 angles
   morts, 2 modules).

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-13a-p5-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `docker`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`grep`, lecture de `~/.cargo/registry/src/`.
