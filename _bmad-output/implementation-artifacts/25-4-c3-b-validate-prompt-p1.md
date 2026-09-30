# Prompt — validation P1, Story 25-4-c3-b (l'arrondi au centime, et l'écart en écriture)

*Versionné le 2026-09-30. **Une lentille** (Sonnet), contexte frais.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-c3-b-arrondi-au-centime` (empilée sur la c3-a2, PR
#489). Fiche : `_bmad-output/implementation-artifacts/25-4-c3-b-arrondi-au-centime.md`. Sœurs :
`25-4-c3-a1-reglage-compte-arrondi.md`, `25-4-c3-a2-compte-arrondi-plans.md`, et la source des faits
`25-4-c-residuel-au-rapprochement.md` (Change Log : validation P1 F1–F3, P3 F4). Issue : `gh issue view 476`
(commentaires : arbitrages). Règles : `CLAUDE.md` (§ *Pattern batch*). Checklist :
`.claude/skills/bmad-create-story/checklist.md`.

L'arbitrage (l'écart s'écrit sur le compte désigné dans les paramètres) est **retenu** : en contester la
mise en œuvre, pas le principe. ⚠️ La fiche décrit du travail **à faire** : ne reproche pas au code de ne pas
encore le porter.

## Axes — tous obligatoires

1. **Chaque référence `fichier:ligne`** existe et dit ce que la fiche affirme.
2. **L'inventaire des sites** — pars du symptôme, pas de la liste des huit :
   `grep -rn "amount_due\|amountDue\|due_before\|due_after\|OVERPAYMENT\|overpayment" crates/ frontend/src`.
   Toute comparaison d'un reste dû à un montant payé, ou tout affichage d'un reste, hors des huit sites :
   légitime ou oublié ? Existe-t-il d'autres chemins qui **règlent** une facture client (import, avoir,
   écran d'échéancier, API) ?
3. **La mécanique de l'AC 3** : règlement au reste **brut**, trois lignes. Vérifie qu'elle ferme bien la
   créance à zéro (la ligne de créance = reste brut) et que `invoice_settlements.amount` (précision, CHECK
   `amount > 0`) l'accepte. Que deviennent les règlements **partiels** antérieurs (10.00 puis 0.01 sur un
   reste de 10.0050) : le second paiement déclenche-t-il bien l'écart ? Et un reste brut **négatif** hérité ?
   Le montant rapproché de la **transaction bancaire** (`bank_transactions.amount`) reste-t-il cohérent avec
   la ligne de banque de l'écriture ?
4. **L'AC 4** (compte manquant) : où lire le réglage (dans la transaction ? sous verrou ?) ; le code
   d'erreur respecte-t-il le pattern batch pour le rapprochement et le mapping d'erreurs pour le règlement
   manuel (`crates/kesh-api/src/routes/invoices.rs:1183`, `crates/kesh-api/src/errors.rs`) ?
5. **L'AC 6** (annulation) : `journal_entries::reverse_in_tx` (`:1437`) contre-passe-t-il toutes les lignes
   d'une écriture quelconque ? L'annulation d'un rapprochement (`reconciliation_cancel.rs`) passe-t-elle par
   le même chemin ? Le compte d'arrondi, archivé entre-temps, bloque-t-il la contre-passation (garde `active`
   de `create_in_tx`) — et est-ce acceptable ?
6. **Le verrou optimiste de la 25-4-c2** : l'écriture de l'écart change-t-elle quoi que ce soit à
   l'invariant « tout écrit qui change le reste dû incrémente `version` » ?
7. **Faisabilité des tests (AC 7)** : peut-on fabriquer un reste brut de 10.0050 et de 10.004 (lignes, TVA,
   `line_total` à 4 décimales) ? Mutations tuables ?
8. **Le manuel** : `docs/manual/fr/admin-manual.tex` et `user-manual.tex` (règlement, rapprochement),
   **PDF aplati** (`pdftotext … | tr '\n' ' ' | tr -s ' '` vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`).
9. **Périmètre** : modules recomptés depuis les tâches (seuil : plus de 5 → découpage).

## Ce que tu rends

- **Findings** : sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit exact, **preuve** (commande exécutée et sa
  sortie, ou code lu cité), correction proposée.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base
(`scripts/*`, `make`, `latexmk`, `git commit`/`push`/`add`/`stash`/`reset`/`rebase`/`checkout`/`switch`,
`sqlx migrate`, `cargo test`/`nextest`, `npm run`, `npx playwright`, `gh issue create`/`comment`/`edit`).
Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers le scratchpad,
`cargo check`.
