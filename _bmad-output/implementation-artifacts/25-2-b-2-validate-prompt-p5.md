# Prompt — passe 5 de `bmad-create-story validate`, Story 25-2-b-2 — passe CIBLÉE

*Versionné le 2026-09-21. Une lentille en contexte frais (Sonnet) — les passes 2 et 4 étaient sur
Opus, comme l'auteur des patches. Passe **ciblée** : elle relit **la remédiation de la passe 4**.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-2-b-1-devalidation-depot-api`. ⚠️ **`main` a été
intégré à la branche** (`75a33c30`) : les citations doivent valoir sur l'arbre **de la branche**,
qui porte désormais la release `v0.12.0`.

## L'objet — le SEUL commit de remédiation

```sh
git show 7bfc72e4 -- _bmad-output/implementation-artifacts/25-2-b-2-retrait-suppression-ecran-manuels.md
```

La passe 4 avait rendu 2 HIGH, 4 MEDIUM, 5 LOW. Les patches ont : **interdit de toucher au compteur
de la section `[0.12.0]`** et fait **créer** une section « Non publié » ; nommé le **second chemin
de suppression** (écran de liste) ; tranché la contradiction sur
`delete_validated_in_locked_period_…` (**réécrit**, non supprimé) ; corrigé « huit » → **neuf**
sites avec sa ventilation ; supprimé le test neuf **doublon** de l'AC 1 et prescrit le **renommage**
des deux retargetés ; restreint la garde `isAdmin` au seul bouton **Supprimer** ; corrigé les
plages `1236-1282`, `1232-1235`, `1339-1349`, le retrait d'écran (`:919-943` **et** `:945`, import
`:3`) et « devient inconditionnel ».

⛔ **Trois passes de suite, le défaut s'est logé dans ce qu'une passe venait d'écrire.** Commence
par ces corrections.

## Les axes, et tu déclareras lesquels tu as exercés

1. ⛔ **Chaque correction de la passe 4 est-elle juste ?** Vérifie sur l'arbre de la branche :
   `invoices.rs:1232-1235`, `:1236-1282`, `:1331`, `:1339-1349` ; `+page.svelte:618-621`,
   `:622-625`, `:626-629`, `:616-630`, `:3`, `:936`, `:919-945` ; l'écran de liste
   `invoices/+page.svelte:423-429` et `:459-491` ; `CHANGELOG.md` — la section `[0.12.0]`, la ligne
   du « 123 libellés », et l'absence de toute section « Non publié ».
2. **Recompter TOUT total contre sa propre ventilation**, et pas seulement celui qui vient d'être
   corrigé : les « neuf autres sites » (+ les deux déjà nommés), les **huit** tests (5 + 2 + 1), les
   « quatre locales », les compteurs cités dans les critères. *Quatre décomptes de cette story ont
   déjà été faux.*
3. ⛔ **Énumérer les chemins, pas les écrans.** `grep -rn "deleteInvoice\|/invoices/.*DELETE" frontend/src`
   et le registre des routes : **combien de chemins suppriment une facture**, et la fiche les
   nomme-t-elle tous ? Même question pour la **dévalidation** : combien de sites l'appelleront, et
   l'AC 4 les couvre-t-il ? *Un écran nommé n'est pas un chemin énuméré — la passe 4 l'a payé.*
4. **Les corrections créent-elles leur propre défaut ?** Le tableau des huit tests et la liste qui le
   suit se contredisent-ils encore ? Le renommage prescrit casse-t-il une référence ailleurs
   (`sprint-status.yaml`, fiche mère, b-1) ? La section « Non publié » à créer entre-t-elle en
   conflit avec ce que `scripts/prepare-release.sh` attend (`## [X.Y.Z] — Non publié`) ?
5. **La cohérence entre les deux filles** : b-1 réécrit-elle encore `invoices.rs:1339-1349`, que
   b-2 déclare emporter ? Un critère est-il porté deux fois, ou aucune ?

## Ce que tu rends

- **Les findings** : sévérité, l'endroit exact, **la preuve**, le correctif. Pour chacun, dis s'il
  est de nature **découpage**, **conception**, ou **dérive extérieure** (le dépôt a bougé sous la
  fiche).
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/`add`/`stash`/`reset`/
`merge`, `sqlx migrate`, `cargo test`/`cargo nextest`. Lecture, `grep`, `sed -n`, `git show`/`diff`,
`pdftotext` et `cargo check` sont autorisés.
