# Prompt — validation P1, Story 25-4-c (le résiduel au rapprochement)

*Versionné le 2026-09-29. **Une lentille** (Sonnet), contexte frais.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-c-residuel-au-rapprochement` (sur `main`, b2
mergée). Fiche à valider : `_bmad-output/implementation-artifacts/25-4-c-residuel-au-rapprochement.md`.
Mère : `25-4-propager-le-residuel.md`. Sœurs : `25-4-a-residuel-juste.md`,
`25-4-b1-residuel-aux-agregats.md`, `25-4-b2-residuel-aux-rappels.md`. Issues : `gh issue view 420`,
`gh issue view 476`. Contexte : `24-2-encaissement-client.md`, les stories de l'Epic 8 sur le
rapprochement (`ls _bmad-output/implementation-artifacts/8-*`). Règles : `CLAUDE.md` (notamment le
§ *Pattern batch*). Checklist : `.claude/skills/bmad-create-story/checklist.md`.

Les arbitrages Q1–Q3 de la fiche sont **retenus** : ne pas les contester, en contester la mise en œuvre.

## Axes — tous obligatoires

1. **Chaque référence `fichier:ligne`** existe et dit ce que la fiche affirme.
2. **Inventaire des sites qui comparent un montant bancaire à une facture client** — pars du
   symptôme, pas de la liste de la fiche :
   `grep -rn "total_ttc\|amount_due\|invoice_amount\|invoiceAmount\|amount_score\|OVERPAYMENT" crates/ frontend/src`.
   Pour chaque site : couvert par la fiche, légitimement au TTC, ou oublié (au moins MEDIUM).
   Existe-t-il d'autres chemins qui rapprochent ou règlent une facture client (import CAMT avec
   référence QR, accept_batch, règlement manuel, page de la facture, tableau de bord) ?
3. **La forme jointe (AC 1)** : `amount_due_derived_joins()` et `INVOICE_AMOUNT_DUE_DERIVED_SQL`
   s'insèrent-ils réellement dans la requête de `find_unpaid_invoices_for_window` (GROUP BY, HAVING,
   paramètres liés, `company_id` des tables dérivées) ? Le scoping multi-tenant reste-t-il prouvé ?
4. **Le re-score (AC 2)** : `amount_due` scalaire dans la transaction — verrou, isolation, ordre des
   gardes (score avant trop-perçu). Une course entre deux acceptations concurrentes sur la même
   facture change-t-elle de comportement ?
5. **L'arrondi (AC 5, 6)** : les cinq sites sont-ils les seuls ? Cherche toute comparaison
   `<= 0`, `> amount_due`, `== Decimal::ZERO` sur un reste dû (`grep -rn "due_after\|amount_due" crates/`).
   La stratégie `MidpointAwayFromZero` est-elle cohérente avec la QR émise par la b2 (le rappel) et
   par la facture ? Un reste de 0.004 : arrondi à 0.00, la facture est-elle soldée — et qui pose
   `paid_at`, l'audit `invoice.paid` ? Le reste **affiché** (AC 3) est-il arrondi aussi ?
6. **L'affichage (AC 3, Q3)** : le libellé « reste dû sur » — clé i18n, 4 locales,
   `lint-i18n-ownership`, formatage suisse des montants ; le type frontend doit-il porter le TTC en
   plus du reste (champ API nouveau → documentation `api-external` s'il y en a) ?
7. **Les tests (AC 7)** : chacun est-il faisable et prouve-t-il ce qu'il annonce ? Le cas 10.0050 :
   peut-on réellement fabriquer une facture dont le reste dû brut a 4 décimales (lignes, TVA) ?
   L'E2E Playwright : peut-on monter un règlement partiel puis un import bancaire ? Mutations tuables ?
8. **Le manuel (AC 8)** : lignes citées, et **PDF aplati**
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`).
   Le paragraphe du score est-il bien faux sur le code, point par point ? D'autres textes (manuels
   de/en/it, admin-manual, README, website, CHANGELOG) décrivent-ils le rapprochement sur le TTC ?
9. **Le périmètre** : cinq modules annoncés — recompte-les depuis les tâches. Quelque chose de la
   25-4-d (solder le reste) appartient-il ici, ou l'inverse ?

## Ce que tu rends

- **Findings** : sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit exact, **preuve** (commande et
  résultat, code lu), correction proposée.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un rapport sans elle
  ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`. Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`,
`pdftotext` vers le scratchpad, `cargo check`.
