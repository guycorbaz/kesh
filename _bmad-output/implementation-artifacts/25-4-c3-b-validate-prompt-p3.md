# Prompt — validation P3 ciblée, Story 25-4-c3-b

*Versionné le 2026-09-30. **Une lentille** (Sonnet), contexte frais, **passe ciblée** sur la seule
remédiation P2 (§ *La passe ciblée*, `CLAUDE.md`).*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-c3-b-arrondi-au-centime`. Fiche :
`_bmad-output/implementation-artifacts/25-4-c3-b-arrondi-au-centime.md`. Remédiation à relire :
`git show 58938564` (et, pour le contexte de la règle, `git show aee52a8d`). Issue neuve : `gh issue view 490`.
⚠️ La fiche décrit du travail **à faire** : ne reproche ni au code ni aux manuels de ne pas encore le porter.

## Axes — tous obligatoires

1. **Normalisation** : `scale_within(&amount.normalize(), 2)` — `rust_decimal::Decimal::normalize` existe-t-il
   dans la version du dépôt (`Cargo.lock`) et fait-il ce qu'on dit (« 10.000 » → échelle 0 ou 2 ? « 10.10 »
   → « 10.1 ») ? Le montant **enregistré** doit-il être normalisé, ou seulement contrôlé ?
2. **Contrôle client** : l'équivalence affirmée (« avec deux décimales, saisie > reste arrondi ⇔ double borne
   de l'AC 3 ») est-elle vraie dans **tous** les cas (r > b, r < b, r = b) ? Démontre ou donne un contre-exemple.
   Le contrôle d'échelle en JS (`Number`, big.js) tient-il sur « 10.000 », « 10,01 », « 1e-3 », « 10. » ?
   Comment le dialogue parse-t-il aujourd'hui la saisie (`SettleInvoiceDialog.svelte:107`) ?
3. **La limite #490** : l'affirmation « défaut antérieur, ni créé ni fermé » est-elle exacte ? La story
   rend-elle ce cas **plus fréquent** (un acompte qui soldait hier… ou l'inverse) ? Les deux voies d'arrivée
   citées sont-elles réelles (un avoir peut-il laisser un reste brut de 0.004 : `credit_notes`, calcul du
   reste dû) ?
4. **Propagation** : les phrases modifiées contredisent-elles une autre phrase de la fiche ? Greppe
   `décimales`, `échelle`, `scale`, `arrondi` dans la fiche.
5. **Tests ajoutés à l'AC 7** : faisables et discriminants (chacun rougit-il avant le patch) ?

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (commande et sortie, ou code cité), correction. ⛔ La liste des
axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base
(`scripts/*` dont `scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`/…, `sqlx`,
`cargo test`/`nextest`, `npm run`, `npx playwright`, `gh issue create`/`comment`/`edit`). Autorisés :
lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `cargo check`, un script `node -e` ou
`evcxr`/petit test **hors dépôt** dans le scratchpad
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`.
