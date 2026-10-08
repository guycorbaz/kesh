# Prompt — validation P7 ciblée de la spec, Story 15-5d

*Versionné le 2026-10-08. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur le seul commit de la remédiation P6. La fiche porte une dérogation écrite à la
règle de découpage : si cette passe trouve un défaut d'ORDRE DES VERROUS né d'une remédiation, la coupe AC5/AC6
s'applique — signale-le explicitement.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. **Objet** : `git show de4cf826`
(lis ce diff ; ouvre la fiche `_bmad-output/implementation-artifacts/15-5d-garde-usage-comptes-reglage.md` autour des
hunks). Rapports remédiés : `target/gate-logs/15-5d-p6-R.md` et `15-5d-p6-F.md`. Choix C87, C88.

## Lentille unique — chasseur de régressions de la remédiation

1. **Patron `owned_account_ids`** (AC1) : lecture non verrouillante dans la transaction, puis refus « absent de
   l'instantané ». Existe-t-il au code (`grep -rn "owned_account_ids" crates/`) et avec la signature citée ? Le refus
   rendu est-il cohérent avec l'AC4 ?
2. **Test « autre société »** : sonde, mutation « patron retiré », refus `InactiveOrInvalidAccounts` — le test
   rougit-il sous la mutation ?
3. **Motif d'attente** élargi (`FROM accounts WHERE`) : distingue-t-il encore les tests 1 et 2 quand l'accesseur est
   en lecture simple (lis `attendre_une_requete_en_cours`, `crates/kesh-db/src/test_fixtures.rs`) ? Peut-il attraper
   une autre requête et rendre un faux vert ?
4. **Montage** « `disable_rounding_to_5_centimes` sauf le test 3 », et sonde `NOWAIT` jugée sur l'erreur 1205.
5. **Cycle (a bis)**, paragraphe plan comptable / désarchivage (relis `accounts::reactivate` et `reset_demo` au
   code), section « Dérogation ».

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-5d-p7-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture.
