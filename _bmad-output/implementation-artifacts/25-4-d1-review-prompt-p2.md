# Prompt — revue de code P2 ciblée, Story 25-4-d1 (les comptes de solde)

*Versionné le 2026-10-01. Une lentille (Haiku), contexte frais — passe ciblée sur la seule remédiation de P1
(cf. `CLAUDE.md` § « La passe ciblée »).*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d1-comptes-de-solde`. **Diff à revoir : `git diff 1d95c91b b093bab1`**
— le commit de remédiation de la passe 1, aplati (un seul commit). Fiche :
`_bmad-output/implementation-artifacts/25-4-d1-comptes-de-solde.md` (Change Log, entrée « Revue de code P1 »).

## Lentille — regression hunter

La remédiation déplace le contrôle « deux marqueurs sur une même entrée » **avant** les validations par marqueur
dans `validate_chart` (`crates/kesh-core/src/chart_of_accounts/mod.rs`), ajoute un test, et remplace « sur ce
compte » par « sur le compte de différences d'arrondi » dans `docs/manual/fr/admin-manual.tex`.

- Le déplacement change-t-il le résultat d'un cas **valide** (plans livrés, plan sans marqueur) ou le message d'un
  autre refus ? Le contrôle utilise-t-il une variable déclarée après lui ?
- Le test neuf aurait-il **échoué** avant le patch (lis `git show 1d95c91b:crates/kesh-core/src/chart_of_accounts/mod.rs`
  et dis quel message l'ancien ordre aurait produit) ? Prouve-t-il ce qu'il dit ?
- Le manuel : la phrase modifiée est-elle juste ? Le PDF régénéré la porte-t-il (`pdftotext docs/manual/fr/admin-manual.pdf - |
  tr '\n' ' ' | tr -s ' '`, sortie vers `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`) ?
  Reste-t-il dans le manuel admin d'autres « ce compte » dont l'antécédent est désormais ambigu à cause du
  paragraphe *Comptes du solde du reste* ?

## Ce que tu rends

Findings avec sévérité, `fichier:ligne`, **preuve** (commande et sortie). Pour tout finding affirmant qu'un code est
absent ou présent : la sortie d'un `grep -nF`. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx migrate`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `cargo check`, `pdftotext` et expériences dans le scratchpad.
