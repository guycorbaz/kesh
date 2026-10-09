# Prompt — validation P5 ciblée de la spec, Story 15-1a-i

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur la seule remédiation P4 de la fiche 15-1a-i. Aucune règle métier ni ordre de
verrous n'a changé en P4.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. **Objet** :
`git show 329ab812 -- _bmad-output/implementation-artifacts/15-1a-i-marque-du-lettrage.md` (lis ce diff ; ouvre la
fiche autour des hunks). Rapports remédiés : `target/gate-logs/15-1a-i-p4-R.md` et `15-1a-i-p4-F.md`. Choix C128,
C130, C131. Code de référence : `origin/main` (`git show origin/main:<chemin>`).

⚠️ **La story n'est pas développée** : tu relis une SPÉCIFICATION. Un test, une constante ou un texte que la fiche
prescrit et qui n'existe pas encore dans le code n'est PAS un défaut. Un défaut est une contradiction, une
prescription inexécutable, un fait cité faux, ou un site oublié.

## Lentille unique — chasseur de régressions de la remédiation

1. **C128** : la lecture sans verrou `LETTERING_FISCAL_YEAR_NAMES_SQL` ajoutée en mode `System` après l'acte 1.
   Est-elle cohérente partout (R7 point 3, AC6, AC10, T3, « Coût » de R6) ? Peut-elle rendre un nom faux ou
   manquant (exercice renommé ou supprimé entre l'acte 1 et la lecture) ? Le reste de la phrase « sans requête »
   ne se contredit-il nulle part ?
2. **C130** : la promesse « à lettrer » retirée. Rejoue le `git grep` de la fiche sur `origin/main` et vérifie
   que les 5 lignes restantes sont bien légitimes ; les textes de remplacement sont-ils donnés dans les 4 locales ?
   Les tests qui assertent l'ancien texte sont-ils tous nommés ?
3. **C131 / F4-4** : `check_rows_affected` testée sans base, au lieu d'un déclencheur. Vérifie l'argument
   `CLIENT_FOUND_ROWS` dans `~/.cargo/registry/src/*/sqlx-mysql-0.8.6/src/connection/stream.rs`. L'angle mort
   est-il écrit ?
4. **R2** : `to_ascii_uppercase` puis validation `A-Z` ; les tests nommés couvrent-ils `ß`, `ſ`, `ı` ?
5. **Fixtures frontend** nommées (`JournalEntryForm.edit.test.ts`, `form-helpers.test.ts`) : existent-elles aux
   lignes citées ?

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-1a-i-p5-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`grep`, lecture des sources dans
`~/.cargo/registry/src/`.
