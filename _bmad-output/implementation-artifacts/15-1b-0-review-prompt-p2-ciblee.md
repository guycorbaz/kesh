# Prompt — revue de code P2 CIBLÉE, Story 15-1b-0 (une lentille, la dernière remédiation)

*Versionné le 2026-10-10. Une lentille (Haiku), contexte frais, lecture seule. § « La passe ciblée » du `CLAUDE.md`.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-15-1b-0`, branche `story/15-1b-0-propriete-des-lignes-par-lot`, rebasée sur
`origin/main` (`56380066`, arbre identique à l'ancienne base `f8888750`). **Le seul diff à revoir** : `git show 83b3a026`
(la remédiation de la revue P1 — un commit unique, aplati : ne compare pas à d'autres commits). Fiche :
`_bmad-output/implementation-artifacts/15-1b-0-propriete-des-lignes-par-lot.md` — Change Log « Revue de code P1 » (le tableau
des findings et leurs verdicts) et D4/D5/AC1/AC3. Rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/15-1b-0-review-p1-{B,E,A}.md`.

## Lentille unique — chasse aux régressions de la remédiation

La remédiation n'ajoute aucune ligne exécutable de production : des doc-comments (`journal_entries.rs`,
`settlement_cancellation.rs`), la garde lexicale du `mod tests` de `crates/kesh-api/src/routes/journal_entries.rs`, et des
tests (`crates/kesh-db/tests/document_owners.rs`, `crates/kesh-db/tests/letterings.rs`). Vérifie :

1. **Les tests modifiés prouvent-ils encore ce qu'ils nomment ?** Fixture étendue (autre société à cinq types, écriture
   `e_sans_numero`) : l'oracle gelé reste-t-il déterministe (jamais deux pièces d'un même type sur une écriture de
   `ecritures()`) ? L'assertion « huit rangs vus » tient-elle ? `owners_are_scoped_by_company` : une table vide peut-elle
   venir d'une fixture creuse (assertion de montage présente ?) ? Bornes 500/501/1000/1001 : attendus justes ? Test de la
   première ligne : l'écriture « ancienne » a-t-elle vraiment le plus petit id ET la ligne lettrable la plus grande ? les
   deux lignes se soldent-elles sur le compte lettrable ? le refus attendu est-il bien R5 et non un rang antérieur ?
2. **La garde lexicale durcie** : rougit-elle encore sur `&state.pool`, sur `acquire()` ? Accepte-t-elle les trois appels
   réels (`git show 83b3a026:crates/kesh-api/src/routes/journal_entries.rs`, corps de `get_journal_entry`) ?
3. **Doc-comments** : chaque phrase neuve est-elle vraie du code ? Grep de la valeur sur `crates/` : « ordre des tests »,
   « rang » au sens de `rank()`.
4. **Change Log P1** : les décomptes (2 MEDIUM distincts, 8 LOW distincts, recoupements) se recomptent-ils depuis les trois
   rapports ? Les verdicts « sans changement » (B-3, E7) sont-ils argumentés ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1b-0-review-p2-ciblee.md`. Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, preuve. ⛔ Tout finding affirmant qu'un code est absent ou présent porte la
sortie d'un `grep -nF` copiée ; ⛔ un livrable que la fiche prescrit mais qui manquerait doit être cherché par `grep -rnF`
dans tout `crates/` avant d'être déclaré absent. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport ; aucune commande qui écrit dans le dépôt ou dans une base, ni ne compile, ni
n'exécute : `scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`, `sqlx`, `cargo`, `npm`, `npx`, `docker`, `gh`
en écriture, aucune requête SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`.
