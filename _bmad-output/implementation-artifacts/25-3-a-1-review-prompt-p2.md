# Prompt — passe 2 de `bmad-code-review`, Story 25-3-a-1

*Versionné le 2026-09-24. Trois lentilles en contexte frais (Haiku 4.5) — cycle Sonnet → Haiku →
Opus. ⚠️ Haiku : le diff est fourni **aplati en un seul fichier** (pas de séquence de commits),
pour éviter la confusion d'indexation d'un diff multi-commit.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-3-a-annuler-reglement`. **Diff revu** :
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/5bef2124-6ec8-41d8-9004-39b0653420a3/scratchpad/25-3-a-1-p2.diff`
(= `git diff a1534b63..dd29630d` sur le code et la documentation, 3711 lignes). Fiche :
`_bmad-output/implementation-artifacts/25-3-a-1-annuler-reglement-client.md` — son Change Log dit
ce que la passe 1 a trouvé et corrigé.

Objet : **annuler un règlement client par contre-passation** (cf. fiche). ⛔ La passe 1 a trouvé
une **course** (clôture d'exercice concurrente) que la lentille chargée de la concurrence n'avait
pas vue ; le correctif est l'étape **2-bis** de `cancel_settlement_in_tx`
(`crates/kesh-db/src/repositories/invoice_settlements_write.rs`) et le test
`une_cloture_concurrente_attend_l_annulation`. **Motif mesuré : la sévérité se déplace vers ce qu'on
vient d'écrire.**

## Lentille 1 — Blind Hunter (diff SEUL)

Tu ne lis **que** le fichier de diff. Revue adversariale générale : logique, erreurs, concurrence,
multi-tenant (`company_id` sur chaque requête), tests qui ne prouvent rien.

## Lentille 2 — Edge Case Hunter (diff + dépôt)

Parcours chaque branche du code neuf en lisant l'appelé et l'appelant. ⛔ **Priorité : la
concurrence**, que la passe 1 a manquée dans cette lentille-ci. Pour **chaque** lecture sans
verrou qui décide d'un refus (`settlement_cancel_blocker`, `settlement_entry_cancel_blocker`),
dis quel verrou, posé avant, la rend sûre — ou qu'aucun ne le fait. Vérifie l'ordre des verrous
de `cancel_settlement_in_tx` (facture → règlement → écriture + exercice → socle : écriture
d'origine + exercice, exercice du jour) contre celui de `settle_invoice`, `create_credit_note`,
`accept_one_invoice`, `fiscal_years::close` / `reopen`, `journal_entries::create_in_tx_inner` :
un **interblocage** est-il possible ? Le test d'entrelacement prouve-t-il ce qu'il dit, et est-il
**fiable** (il repose sur un délai de 500 ms) ?

## Lentille 3 — Acceptance Auditor (diff + fiche + documentation)

Chaque AC (1 à 13) contre le diff ; les décomptes du Dev Agent Record et du Change Log
**recomptés** ; le manuel `docs/manual/fr/user-manual.tex` et son **PDF aplati**
(`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`), `docs/api-external.md`,
`CHANGELOG.md`, `README.md`. ⚠️ La fiche **sœur** `25-3-a-2-annuler-reglement-fournisseur.md` a
été modifiée par la passe 1 (AC 3, verrou) : l'ajout est-il juste et suffisant ?

## Ce que tu rends

- Findings : sévérité (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`, **preuve**, correction.
  ⛔ Pour tout CRITICAL ou HIGH affirmant qu'une chose est **absente** ou **présente** dans un
  fichier : la commande `grep -nF "<chaîne>" <fichier>` exécutée et son résultat.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`/`cargo nextest`, `npm run`,
`npx playwright`. Lecture, `grep`, `git log`/`show`/`diff`, `pdftotext` (vers le scratchpad) et
`cargo check` sont autorisés.
