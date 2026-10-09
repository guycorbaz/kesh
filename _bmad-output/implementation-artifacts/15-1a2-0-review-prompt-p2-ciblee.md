# Prompt — revue de code P2 CIBLÉE, Story 15-1a2-0 (une lentille, Haiku)

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille, contexte frais,
braquée sur **le seul commit de la remédiation P1**.*

Worktree `/home/gcorbaz/devel/kesh-15-1a2-0`. **Diff à revoir, aplati** : `git diff 9b51bab1 4ff48b03`. Contexte :
fiche `_bmad-output/implementation-artifacts/15-1a2-0-lettrage-fige-avec-la-periode.md` (§ Change Log « Revue de
code P1 », qui dit ce que chaque hunk corrige).

## Lentille — chasseur de régression de la remédiation

1. `crates/kesh-db/src/repositories/letterings.rs`, `document_group_frozen_by_periods` : la branche
   `if lignes.is_empty() { continue; }` peut-elle laisser passer un groupe réellement figé (faux négatif) ? Lis la
   requête des clés (`DOCUMENT_KEYS_OF_ENTRY_SQL`) et celle des lignes (`GROUP_LINE_PERIODS_SQL`) : un groupe
   existant peut-il rendre zéro ligne à la seconde requête autrement que par dissolution (filtre de société,
   jointure) ?
2. `crates/kesh-db/src/repositories/fiscal_years.rs`, doc de `find_later_closed` : la liste des appelants est-elle
   exacte (`grep -rn "find_later_closed(" crates --include=*.rs`) ? Le paragraphe « Sans verrou par choix » est-il
   resté attaché à l'appelant qu'il justifie ?
3. `crates/kesh-db/tests/letterings.rs` : l'assertion ajoutée prouve-t-elle ce qu'elle nomme (groupe à cheval sur
   deux exercices) ?
4. `_bmad-output/implementation-artifacts/15-1c-i-ecran-postes-ouverts.md` : le texte cité égale-t-il la valeur
   fr-CH de `error-lettering-is-document` (`crates/kesh-i18n/locales/fr-CH/messages.ftl`) ?

## Ce que tu rends

Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-0-review-p2-B.md` : findings (sévérité, `fichier:ligne`,
preuve — sortie `grep -nF` copiée pour toute affirmation de présence/absence —, correction), et ⛔ **la liste des
axes exercés ET non exercés**.

## Interdits

⛔ Aucun fichier écrit hors de ton rapport ; aucune commande `cargo`, `npm`, `npx`, `scripts/*` (dont
`scripts/prepare-release.sh`), `docker`, `sqlx`, SQL, `git` qui écrit (`commit`, `add`, `checkout`, `stash`,
`reset`), `gh` en écriture. Autorisés : lecture, `grep`, `sed -n`, `git diff`/`show`/`log`.
