# Prompt — validation P2 de la spec, Story 25-7 (soldes de départ)

*Versionné le 2026-10-06. Deux lentilles (Opus), contexte frais chacune. Rotation (décision D6 de la rétro 25) :
P1 Sonnet ×3 → P2 Opus ×2 ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-7-soldes-de-depart`. **Fiche** :
`_bmad-output/implementation-artifacts/25-7-soldes-de-depart.md` — lire son **Change Log** (entrée « Validation P1 »).
La remédiation de P1 est le commit `bede9278` (`git show bede9278`) : elle a **réécrit** la fiche. Issue :
`gh issue view 445`. Règles : `CLAUDE.md`. Aucun code n'est encore écrit.

## Lentilles

- **R — Regression hunter** : `git show bede9278` et la fiche telle qu'elle est. La réécriture a-t-elle introduit
  des défauts ? Contradictions entre arbitrages, AC, tâches et Dev Notes (ex. : un AC qui dit « refus » là où un
  arbitrage dit « aucune contrepartie ») ; un code d'erreur de la table de l'AC 5 sans test à l'AC 7, ou un
  `completeReason` de l'AC 3 sans correspondance ; des faits recopiés de P1 sans vérification (numéros de ligne,
  noms de fonctions) — revérifie-les au code (`grep -nF`). La fiche est-elle implémentable sans deviner ?
- **F — Full-scope adversary** : la conception, et surtout ce que P1 a affirmé sans l'exécuter.
  - **La parade du CRITICAL B1** : la fiche affirme qu'InnoDB pose un verrou **partagé** sur la ligne parente
    `accounts` à l'insertion d'une ligne de `journal_entry_lines` (clé étrangère `fk_jel_account`), et qu'un
    `SELECT … FROM accounts … FOR UPDATE` attend donc toute écriture en vol et bloque les suivantes. Est-ce exact pour
    MariaDB 10.11 / InnoDB (documentation, comportement connu) ? Les deux sens (écriture en vol avant le complément ;
    complément avant l'écriture) sont-ils fermés ? Que fait une écriture ordinaire qui **lit** le compte puis insère :
    peut-elle interbloquer avec le complément (le complément tient `companies` puis `accounts` ; l'écriture tient
    `fiscal_years` puis la ligne parente) ? Le rejeu sur interblocage (`kesh_db::retry::retry_with`) est-il
    nécessaire à la route ?
  - **REPEATABLE READ** : l'ordre de l'AC 4 garantit-il qu'aucune lecture non verrouillante ne précède les verrous
    (la sentinelle `companies` est-elle une lecture verrouillante qui n'ouvre pas d'instantané ? les contrôles de
    forme de l'étape 2 lisent-ils la base ?) ; l'étape 6 (date, exercice, `books_locked_through`) et l'étape 8
    (`create_in_tx`, qui relit `books_locked_through` sans verrou, `journal_entries.rs:269-275`) voient-elles un état
    cohérent avec la sentinelle ?
  - **L'arbitrage 2** : chaque branche et ses bords ; « plusieurs exercices ouverts » ; un premier exercice ouvert
    mais **non** le premier par date ; la date UTC et minuit.
  - **L'AC 2** (formule) et **l'AC 1** (variantes) : justes, et cohérents avec la contrepartie de l'AC 4 étape 7 ?
  - **Les sites rendus faux** : le **manuel** (`docs/manual/fr/user-manual.tex` **et le PDF aplati**,
    `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers `target/gate-logs/`), les 4
    catalogues `crates/kesh-i18n/locales/*/messages.ftl`, les replis Rust et Svelte, le site `website/`. L'AC 8
    les nomme-t-il tous ?
  - La règle de découpage (§ « Règle de splitting préventif », amendée par D5) : la story tient-elle ?

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne` (de la fiche et du code), **preuve** (commande et
sortie, ou code cité relu, ou référence documentaire), correction proposée. Pour tout finding affirmant qu'un code est
absent ou présent : la sortie d'un `grep -nF`. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`, `sqlx`,
`cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, et **aucune requête SQL
d'écriture** (une base de dev existe : tu peux, si utile, y lire en `SELECT` seulement). Autorisés : lecture, `grep`,
`sed -n`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `target/gate-logs/`.
