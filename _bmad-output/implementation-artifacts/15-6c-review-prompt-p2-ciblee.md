# Prompt — revue de code P2 ciblée, Story 15-6c

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur le seul commit de la remédiation P1. ⚠️ Les rapports de la P1
(`target/gate-logs/15-6c-review-p1-*.md`) ont été effacés par un `cargo clean` après le crash de la station :
le message de `73ba6211` et le Change Log de la fiche en tiennent lieu.*

Worktree `/home/gcorbaz/devel/kesh-15-6c`, branche `story/15-6c-configuration-sans-ecriture-nulle`. **Objet** :
`git show 73ba6211` (diff unique ; ouvre les fichiers à `HEAD` autour des hunks). Fiche :
`_bmad-output/implementation-artifacts/15-6c-configuration-sans-ecriture-nulle.md`. Choix C-15-6c-5 et C-15-6c-6
(`_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`).

⚠️ Tu relis du CODE livré ET une fiche : un défaut est ce que le code fait de faux, un test qui ne prouve pas ce
qu'il dit, ou un texte qui contredit le code. Ne signale pas comme manquant ce qu'une AUTRE story prescrit, ni
un livrable que la fiche prescrit à une tâche non encore faite.

## Lentille unique — chasseur de régressions de la remédiation

1. **Code de production** (`crates/kesh-api/src/errors.rs`, `crates/kesh-db/src/repositories/bank_accounts.rs`) :
   le commit affirme n'avoir touché aucune ligne exécutable. Vérifie-le hunk par hunk : un changement de texte
   d'erreur, de repli i18n ou de requête SQL est-il caché parmi les commentaires ? Si oui, est-il juste et testé ?
2. **Tests neufs** de `crates/kesh-db/tests/bank_accounts_repository.rs` (exemption au dépôt) : prouvent-ils ce
   que dit leur nom et leur doc-comment ? Peuvent-ils passer à vide (montage qui n'atteint pas la branche visée,
   assertion sur un état déjà vrai avant l'appel) ?
3. **Test frontend** ajouté à `bank-accounts-page.test.ts` (formulaire de création) : lit-il le bon élément, et
   rougirait-il si le comportement disparaissait ?
4. **Doc-comments** déplacés ou réécrits, notamment la « portée next-key de l'INSERT sans lien » : exacts contre
   le code et contre le comportement de MariaDB 10.11 décrit ailleurs dans le dépôt (`grep -rn "next-key"`) ?
5. **Textes** : manuel (`docs/manual/fr/user-manual.tex` et le PDF aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`), CHANGELOG — disent-ils ce que fait
   le code, ni plus ni moins ?

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `/home/gcorbaz/devel/kesh-gate-logs/15-6c-review-p2-ciblee.md` ; dernier message : le chemin, le
bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit dans le dépôt ou
dans une base, ni qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`,
SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `pdftotext` vers la sortie standard.
