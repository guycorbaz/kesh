# Prompt — revue de code P2 ciblée, Story 15-12a

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur le seul commit de la remédiation P1.*

Worktree `/home/gcorbaz/devel/kesh-15-12a`, branche `story/15-12a-cloture-dans-l-ordre`. **Objet** :
`git show 37784da4` (diff unique ; ouvre les fichiers à `HEAD` autour des hunks). Rapports remédiés :
`target/gate-logs/15-12a-review-p1-{B,E,A}.md`. Choix C-15-12a-2 à 4.

⚠️ Tu relis du CODE livré ET une fiche : un défaut est ce que le code fait de faux, un test qui ne prouve pas ce
qu'il dit, ou un texte qui contredit le code. Ne signale pas comme manquant ce qu'une AUTRE story prescrit.

## Lentille unique — chasseur de régressions de la remédiation

1. **Test 13 b3** (`close_waits_for_a_creation_whose_guard_holds_the_later_year`,
   `crates/kesh-db/tests/fiscal_years_repository.rs` ou voisin — `grep -rn` le nom) : W rejoue-t-il exactement la
   garde de `create` (même requête, `FOR UPDATE`) ? L'attente de la clôture à l'étape (c) est-elle observée
   (motif de `attendre_une_requete_en_cours`) et non supposée ? Le test peut-il passer à vide (clôture qui
   n'attend pas, verdict obtenu par une autre voie) ? Dépend-il du plan de l'optimiseur ?
2. **Message `error-later-fiscal-year-closed`** (4 `messages.ftl` + repli `crates/kesh-api/src/errors.rs`) :
   le texte borné « modifiée ni supprimée » est-il juste dans les 4 langues, et cohérent avec ce que le code
   refuse ? Un seul registre par message (l'italien tutoyé, cf. `docs/i18n-glossaire.md`) ?
3. **Enveloppe `<span class="inline-flex" title=…>`** autour des boutons « Clôturer »/« Réouvrir »
   (`frontend/src/routes/(app)/settings/fiscal-years/+page.svelte`) : le bouton reste-t-il cliquable quand il
   est actif ? La mise en page de la cellule change-t-elle ? Les tests Vitest et E2E lisent-ils le bon élément ?
4. **Tests neufs** `create_ignores_the_closed_years_of_another_company`, `create_right_after_a_closed_year_is_allowed`,
   `both_refusals_reach_a_read_write_api_key` : prouvent-ils ce qu'ils disent ?
5. **Doc-comments neufs** de `crates/kesh-db/src/repositories/fiscal_years.rs` (module, `create`, `close`,
   `find_later_closed_in_tx`) : exacts contre le code, en particulier « aucun montage simple ne force le
   fantôme » et le coût de (b') ?

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-12a-review-p2-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`.
