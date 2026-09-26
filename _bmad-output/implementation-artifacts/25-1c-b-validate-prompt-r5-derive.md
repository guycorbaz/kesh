# Prompt — revalidation R5 « dérive », Stories 25-1c-b1 et 25-1c-b2

*Versionné le 2026-09-26. **Une lentille par fiche** (Sonnet), contexte frais.*

⛔ **Pourquoi cette passe.** Les deux fiches ont été **validées le 2026-09-15/16** (R4 close), sur un
`main` où la route de consultation (**25-1c-a**) n'était encore qu'une spec. Elles ont dormi dix jours
sur une branche locale jamais poussée. Depuis, sur `main` : la **25-1c-a a été implémentée, revue et
mergée (PR #439)** — son contrat réel peut différer de sa spec —, puis la 25-2-b-1/b-2, la 25-2-c, la
25-3-zero, 25-3-a-1, 25-3-a-2, 25-3-b et la 25-5-a, qui ont ajouté des actions d'audit, des clés i18n,
des pages de manuel et bougé des lignes. **Une fiche validée contre un état périmé n'est plus validée.**

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-1c-b1-journal-audit-ecran` (= `main` à `0e4c2682`
+ les fiches). Ta fiche est indiquée dans ta consigne ; lis aussi l'autre (elles se livrent dans la
**même PR**), la fiche mère `25-1c-b-journal-audit-ecran.md` et la fiche **mergée** de la route
`25-1c-a-journal-audit-route.md` (son Dev Agent Record et son Change Log disent ce qui a réellement
été livré). Arbitrages de Guy : § de la fiche de l'epic `_bmad-output/planning-artifacts/epic-25-vague1-suite.md`
(2026-09-11, 2026-09-15 soir et fin de soirée) — **ne pas les contester**.

## Ce que tu cherches — la DÉRIVE, pas la conception

1. **Le contrat de la route, tel qu'il est** : `crates/kesh-api/src/routes/` (le module d'audit, sa
   réponse JSON, ses paramètres de filtre et de pagination, son export CSV, ses rôles), et ce que la
   fiche en suppose. Chaque nom de champ, de paramètre, de code d'erreur, de chemin.
2. **Chaque référence `fichier:ligne`** de la fiche : existe-t-elle encore, et dit-elle encore ce que
   la fiche affirme ? Recompte les lignes, relis le code.
3. **Chaque décompte** (types d'entité, actions, libellés, `sitesTotal`, `CLES_RELEVEES`, clés FTL,
   pages du manuel, tests) : recompté **depuis la source actuelle**, jamais relu. Plusieurs stories
   mergées depuis ont ajouté des actions d'audit (`supplier_invoice.settlement_cancelled`,
   `invoice_settlement.cancelled`, rapprochement annulé…) et des types d'entité.
4. **Ce qui existe déjà** : une partie de ce que la fiche prescrit a-t-elle été livrée entre-temps
   (module de téléchargement, libellés, entrée de menu, vocabulaire « journal d'audit », sections du
   manuel) ? Une prescription devenue fausse, ou redondante ?
5. **Les manuels** (`docs/manual/fr/*.tex` et leurs PDF aplatis :
   `pdftotext <pdf> - | tr '\n' ' ' | tr -s ' '`, vers le scratchpad) : les phrases que la fiche
   prescrit de changer existent-elles encore, aux lignes dites ?

## Ce que tu rends

- **Findings** : sévérité (CRITICAL / HIGH / MEDIUM / LOW), l'endroit exact de la fiche, **la preuve**
  (commande et résultat, code lu), ce qu'il faut changer. Une référence décalée sans conséquence est
  LOW ; un contrat de route faux, un décompte faux qu'un test figera, une prescription devenue fausse
  sont au moins MEDIUM.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, tout `git commit`/`push`/`add`/`stash`/`reset`/
`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`cargo nextest`, `npm run`,
`npx playwright`. Lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` (vers
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/ca5ce2e2-67a3-4eeb-817f-c2de35620a1c/scratchpad/`) et
`cargo check` sont autorisés.
