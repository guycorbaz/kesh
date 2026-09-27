# Prompt — validation P3, Story 25-4-b2 (le résiduel aux rappels)

*Versionné le 2026-09-27. **Une lentille** (Opus), contexte frais.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-b2-residuel-aux-rappels` (empilée sur la 25-4-b1,
PR #475). Fiche à valider : `_bmad-output/implementation-artifacts/25-4-b2-residuel-aux-rappels.md`.
Mère : `25-4-propager-le-residuel.md` (arbitrages de Guy, et ceux de la fiche Q1/Q2 : **ne pas les
contester**, en contester la mise en œuvre). Sœur : `25-4-b1-residuel-aux-agregats.md`. Issue :
`gh issue view 416`. Contexte : `24-2-encaissement-client.md`, les stories de l'Epic 21 sur les
rappels (`ls _bmad-output/implementation-artifacts/21-*`). Règles : `CLAUDE.md`. Checklist :
`.claude/skills/bmad-create-story/checklist.md`.

## Axes — tous obligatoires

1. **Chaque référence `fichier:ligne`** existe et dit ce que la fiche affirme.
2. **Inventaire des sites qui disent un montant au client à l'occasion d'un rappel** — pars du
   symptôme, pas de la liste de la fiche : `grep -rn "total_due\|totalDue\|invoice_total_ttc\|total_ttc\|reminderFee\|fee_amount" crates/ frontend/src`.
   Pour chaque site : couvert par la fiche, légitimement au TTC, ou oublié (au moins MEDIUM).
   Existe-t-il d'autres chemins d'envoi ou d'impression d'un rappel (manuel, renvoi, lot, aperçu,
   export, page d'accueil) ?
3. **La QR (AC 7)** : la référence et le message sont-ils vraiment indépendants du montant ? Le
   montant de la QR d'un rappel peut-il différer du total imprimé sans que le PDF se contredise ?
   Arrondi : `amount_due` est en DECIMAL(19,4), la QR exige 2 décimales — où arrondir, et que
   devient un reste de 0.004 ?
4. **Les frais (AC 2, 3, 8)** : « frais cumulés » = autres niveaux + niveau courant — cohérent avec
   `sum_fees_deduped_excluding` et avec ce que dit `{totalDue}` ? `{feeNotice}` : la liste blanche,
   la validation des gabarits personnalisés existants (un gabarit enregistré avec `{reminderFee}`
   reste-t-il valide ?), l'éditeur de gabarits du frontend liste-t-il les variables (où ?).
5. **Le refus du reste nul (AC 9)** : le pattern batch du `CLAUDE.md` est-il respecté par la forme
   actuelle de la réponse du lot (lis-la) ? L'éligibilité (`dunning_eligibility.rs`) devrait-elle
   exclure ces factures en amont plutôt ?
6. **Le PDF (AC 5, 6, 8)** : géométrie (réserve, gardes), clés i18n et test positionnel, patron de
   surcharge de l'avoir — la locale utilisée (contact vs configuration) est-elle la bonne ?
   L'archivage #387 est-il vraiment hors d'atteinte ?
7. **Les tests (AC 10)** : chacun est-il faisable et prouve-t-il ce qu'il annonce ? L'E2E : peut-on
   monter un règlement partiel puis un aperçu de rappel depuis Playwright ? Mutations tuables ?
8. **Le manuel** : lignes citées, et **PDF aplati**
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, idem `admin-manual.pdf`,
   vers `/tmp/claude-1000/-home-gcorbaz-devel-kesh/4fb1e50e-3db4-4da6-9933-bcee4a186408/scratchpad/`).
   D'autres textes (manuels de/en/it, api-external, README, website) disent-ils que le rappel ou sa
   QR réclament le TTC ?
9. **Le périmètre** : cinq modules annoncés — recompte-les depuis les tâches. Quelque chose de la
   b1 ou de la 25-4-c/d appartient-il ici, ou l'inverse ?

10. **Les corrections des passes 1 et 2** (Change Log ; `git show ac38508f` et `git show 9c5c8a60`) : chacune est-elle juste, et a-t-elle introduit une contradiction ailleurs
    dans la fiche ? En particulier : le refus classé en lot (AC 9) — `BatchItemError::failed` rend-il
    bien la forme attendue par la réponse du lot ? La règle « un seul arrondi » (AC 7) est-elle
    compatible avec `{totalDue}` qui ajoute des frais en DECIMAL(7,2) ? La fonction `amount_credited`
    à créer : même contrat que ses deux voisines (scoping, signe) ?

11. ⚠️ **La passe 2 a rendu 0 finding sans preuve** et a manqué un MEDIUM sur l'axe 8 : ne te fie
    à aucune de ses conclusions. En particulier, refais les axes 2, 4, 6 et 7 comme s'ils n'avaient
    jamais été exercés. Cherche aussi ce que la fiche **n'énumère pas** : les tests existants que le
    changement de `{totalDue}`, des gabarits par défaut et du PDF joint va casser (grep des
    assertions sur ces textes dans `crates/*/tests`, `crates/*/src` et `frontend/tests/e2e`) —
    la fiche doit les nommer.

⚠️ Pour toute affirmation qu'un code manque ou qu'une ligne dit autre chose, cite la sortie de
`grep -nF` ou de `sed -n` qui l'établit — sinon le finding ne sera pas retenu.

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
