# Prompt — validation P2, Story 25-4-a (le résiduel juste)

*Versionné le 2026-09-26. **Une lentille** (Haiku), contexte frais.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-propager-le-residuel`. Fiche à valider :
`_bmad-output/implementation-artifacts/25-4-a-residuel-juste.md`. Fiche mère (inventaire,
découpage) : `25-4-propager-le-residuel.md`. Issues : `gh issue view 455`, `gh issue view 456`.
Contexte de la grandeur : `24-2-encaissement-client.md` § D3, D4, AC 5-7. Règles du dépôt :
`CLAUDE.md`. Checklist BMAD : `.claude/skills/bmad-create-story/checklist.md`.

## Axes — tous obligatoires

1. **Chaque référence `fichier:ligne`** existe et dit ce que la fiche affirme — relis le code.
2. **La formule** : la sous-requête TTC de l'avoir prescrite (AC 1-2) concorde-t-elle réellement,
   centime par centime, avec l'écriture de l'avoir (`generate_credit_note_journal_lines`,
   `line_vat_amount` — lis `kesh-core`, sens d'arrondi de `ROUND` MariaDB vs Rust) ? Et avec
   `INVOICE_TTC_SUBQUERY_SQL` ? Un écart d'un centime possible est un finding.
3. **La garde** (AC 9-11) : inventorie **tous** les chemins qui créent un règlement ou un avoir
   (pars du symptôme : `grep -rn "INSERT INTO invoice_settlements\|create_credit_note\|INSERT INTO credit_notes" crates/`),
   et leur sérialisation (verrou pessimiste ou optimiste). La fiche en nomme trois : en manque-t-il ?
   Le raisonnement REPEATABLE READ de § Courses est-il juste ?
4. **L'état hérité** : l'import d'une sauvegarde ramène-t-il vraiment l'état (règlement + avoir) ?
   Les textes prescrits (AC 17) le disent-ils juste ? D'autres lecteurs de `amount_due` ou des
   constantes changent-ils de valeur avec l'AC 1 (inventaire : `grep -rn "INVOICE_CREDITED\|amount_due" crates/ frontend/src`) ?
5. **Les tests** (AC 16) : chacun prouve-t-il ce qu'il annonce, ou peut-il passer à vide ? Les
   mutations m1-m6 sont-elles tuables par les tests listés ? Une mutation que rien ne tuerait est
   un finding.
6. **Le manuel** : les lignes citées (AC 17) existent-elles et disent-elles ce qu'on veut changer ?
   Contrôle aussi le **PDF aplati** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`,
   vers `/tmp/claude-1000/-home-gcorbaz-devel-kesh/ca5ce2e2-67a3-4eeb-817f-c2de35620a1c/scratchpad/`).
   Une autre phrase du manuel ou de l'admin-manual, de l'api-external, du README, promet-elle
   l'ancien comportement ?
7. **Le découpage** (fiche mère) : quelque chose de 25-4-a appartient-il en fait à b ou c, ou
   l'inverse ? La latence de #455 est-elle correctement établie ?

## Ce que tu rends

- **Findings** : sévérité (CRITICAL / HIGH / MEDIUM / LOW), l'endroit exact de la fiche, **la
  preuve** (commande et résultat, code lu), la correction proposée.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding »
  sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt ; n'exécute aucune commande qui écrit dans le dépôt ou dans une
base — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`. Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`,
`pdftotext` vers le scratchpad, `cargo check`.

## Contexte P1 — à vérifier, pas à re-signaler

La passe 1 a trouvé 1 HIGH (récit « #455 latent » faux : `get_invoice` rend `amountDue` aux clés
API), 3 MEDIUM (`error_code()` et non `code()` ; AC 14 aligné sur le précédent « Dévalider » ;
AC 13, `monter` sert quinze cas et un avoir `issued` exige une écriture) et 1 LOW. Les corrections
sont dans `git diff 0f4b9ec0 HEAD -- _bmad-output/implementation-artifacts/25-4-*.md` — **diff unique
aplati, à lire en priorité** : une correction introduit souvent le défaut suivant.

**Tes priorités** : (1) le **gabarit « détacher, créditer, rattacher »** de l'AC 13 — est-il
réalisable (contraintes d'unicité et FK d'`invoice_settlements`, `company_id`, écriture de
règlement qui pointe l'ancienne facture ?) et compatible avec les quatre paires combinées ?
(2) l'AC 14 révisé — le dialogue d'avoir affiche-t-il vraiment le message du serveur, et que
voit l'utilisateur sur une facture **entièrement** payée, où le bouton était masqué jusqu'ici ?
(3) l'e2e `get_credited_invoice_reports_zero_amount_due` — la forme JSON exacte de `amountDue`
(sérialisation de `Decimal`) pour que l'assertion ne soit pas fragile.
