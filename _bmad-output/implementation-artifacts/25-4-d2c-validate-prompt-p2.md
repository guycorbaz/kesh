# Prompt — validation P2, Story 25-4-d2c (le rapport TVA retranche la TVA des soldes)

*Versionné le 2026-10-02. **Une lentille** (Opus), contexte frais. La passe 1 (Sonnet) a rendu 1 HIGH, 1 MED, 2 LOW ; ses corrections sont dans le commit `8fd48aeb` et au Change Log de la fiche — relis-les d'abord : une remédiation introduit souvent le défaut suivant.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d2c-rapport-tva-soldes`. Fiche :
`_bmad-output/implementation-artifacts/25-4-d2c-rapport-tva-soldes.md`. Stories précédentes : `25-4-d2a-ecriture-de-solde.md`
(l'écriture et la ventilation figée), `25-4-d2b-bouton-solder-le-reste.md`. Issues : `gh issue view 384`, `gh issue view
390`. Règles : `CLAUDE.md`. Checklist : `.claude/skills/bmad-create-story/checklist.md`.

Les arbitrages et les choix « retenus par défaut » sont **acquis** : conteste la mise en œuvre, pas le principe. ⚠️ La
fiche décrit du travail **à faire** : un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Axes — tous obligatoires

1. **Chaque référence `fichier:ligne`** existe et dit ce que la fiche affirme.
2. **Le calcul (AC 1, AC 2)** : la ventilation figée (`write_off_vat`, écrite par `write_off_vat_json` dans
   `crates/kesh-db/src/repositories/invoice_settlements.rs`) a-t-elle bien la forme que la fiche suppose (clés, chaînes) ?
   Le taux du JSON (`ratePercent`, arrondi à deux décimales) fusionne-t-il avec le taux des `rows` (`vat_rate` des lignes)
   pour un affichage cohérent ? Le filtre « de la société » : `invoice_settlements.company_id` existe-t-il ? La période :
   `ReportPeriod` (start/end, intra-exercice) et `settled_on` — même logique de bornes que `i.date BETWEEN` ? Changer le
   sens de `vat_balance` : qui le lit (CSV, PDF, écran, tests, autres modules, `grep -rn "vat_balance\|vatBalance"`) ?
3. **La réconciliation (AC 3)** : le delta net est-il nul par construction pour un escompte soldé dans la période sur
   une facture de la **même** période ? Et pour un solde dans la période sur une facture d'une période **antérieure** (le
   cas courant) — la TVA facturée n'est pas dans `total_vat_due` de cette période, mais la correction l'est : le delta
   reste-t-il nul ? Un solde **annulé** : la ligne disparaît, mais l'écriture de solde ET sa contre-passation restent au
   grand livre — la jointure sur `invoice_settlements` les voit-elle encore (la ligne n'existe plus) ? Un double comptage
   possible ?
4. **Les rendus (AC 4)** : le CSV a un en-tête à trois colonnes ; la section ajoutée y tient-elle sans casser un
   lecteur ou un test existant ? Le PDF a-t-il une gestion de pagination / de dépassement à respecter ?
5. **L'API et les types (AC 5)** : la route sérialise-t-elle `VatReport` tel quel (`routes/reports.rs`) ? Un export de
   souveraineté ou un autre consommateur du rapport ?
6. **Le manuel (AC 6)** (PDF aplati : `pdftotext … | tr '\n' ' ' | tr -s ' '` vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`), **les tests (AC 7)**,
   **les limites (AC 8 — sont-elles justes et complètes ?)**, **le périmètre** (modules recomptés), et **ce qui manque**.

7. **Les corrections de la passe 1** (`git show 8fd48aeb`) : la formule du delta (`soldes = SUM(debit) − SUM(credit)`)
   est-elle juste dans **tous** les cas — un solde **annulé** dans la période (l'écriture de solde ET sa contre-passation
   sont au grand livre, mais la ligne `invoice_settlements` a disparu : aucune des deux n'est jointe — et la
   contre-passation, datée du jour, n'est-elle reliée à rien d'autre ?) ; un solde d'une facture **annulée par avoir**
   (impossible ? vérifie l'ordre des gardes) ; un solde dont l'écriture a été **supprimée** ou **contre-passée par une
   autre voie** ? Le garde `isReportEmpty` : quels autres rapports partagent la fonction, et la modifier pour `'vat'`
   change-t-il leur comportement ? Le test de la vue : quels états (vide, sans soldes, avec soldes, delta) doit-il
   couvrir ?

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (commande et sortie, ou code cité), correction. ⛔ La liste des axes
exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base (`scripts/*` dont
`scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `git log`/`show`/`diff`,
`gh issue view`, `pdftotext` vers le scratchpad.
