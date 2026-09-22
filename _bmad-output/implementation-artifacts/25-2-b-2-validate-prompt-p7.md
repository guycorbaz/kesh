# Prompt — passe 7 de `bmad-create-story validate`, Story 25-2-b-2 — passe CIBLÉE

*Versionné le 2026-09-21. Une lentille en contexte frais (Sonnet) — les passes 2, 4 et 6 étaient sur
Opus, comme l'auteur des patches. **Budget : 7ᵉ passe sur 8.***

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-2-b-1-devalidation-depot-api` (elle porte `main`
au `951cbce2`, release `v0.12.0` comprise).

## L'objet — le SEUL commit de remédiation

```sh
git show HEAD -- _bmad-output/implementation-artifacts/25-2-b-2-retrait-suppression-ecran-manuels.md
```

La passe 6 a rendu 8 MEDIUM et 7 LOW ; **quinze corrections** ont été appliquées, chacune vérifiée
dans le fichier après écriture. Les plus lourdes : l'interdit du `CHANGELOG` passé **d'un nombre à
une classe** (trois sites, `:32`, `:58`, `:267`) ; la section à créer nommée **`## [0.12.1] — Non
publié`** ; le **bloc** `invoices.rs:1337-1354` qui part en entier ; le **dixième site** logé dans
une **migration appliquée** qu'il est interdit de toucher (P8) ; la garde `isAdmin` ramenée au seul
bouton `:427-429` de l'écran de **liste** ; le registre frontend `i18n-keys.test.ts`
(`sitesTotal` 1638).

⛔ **Le motif de cette boucle, mesuré sur six passes : le défaut suivant naît dans la remédiation
précédente — et la passe 6 en a trouvé un qui était la RÉCIDIVE LITTÉRALE, sur l'écran jumeau, d'un
défaut corrigé quatre lignes plus haut par le même patch.** C'est là qu'il faut braquer.

## Les axes, et tu déclareras lesquels tu as exercés

1. ⛔ **Chaque correction de la passe 6 est-elle dans le fichier, et JUSTE ?** Vérifie les lignes sur
   l'arbre : `+page.svelte` de la **liste** (`:423`, `:424-426`, `:427-429`, `:459-491`),
   `invoices.rs:1337-1354` et `:1350`, `journal_entries.rs:982-989`,
   `migrations/20260715000001_invoice_reminders.sql:14`, `CHANGELOG.md:32`, `:58`, `:267`,
   `i18n-keys.test.ts:238`, `invoices.api.ts:62`, `README.md:218`, `:219`.
2. ⛔ **Le symptôme a-t-il été grepé PARTOUT, cette fois ?** Reprends les symptômes un par un —
   « suppression définitive », « supprimer une facture validée », la garde de rôle sur un bouton
   d'un bloc qui en contient plusieurs, un total sans sa ventilation — et cherche **le site que la
   remédiation a oublié**. *C'est ainsi que la passe 6 a trouvé la migration et l'écran jumeau.*
3. **Les corrections créent-elles leur propre défaut ?** En particulier : l'interdit de classe sur le
   `CHANGELOG` empêche-t-il, par ricochet, une correction **légitime** ? Le `## [0.12.1]` est-il le
   bon numéro si une autre story publie d'ici là ? Nommer le bloc `1337-1354` entre-t-il en conflit
   avec ce que la b-1 prescrit sur les mêmes lignes ?
4. **Cohérence interne** : contradictions entre critères, entre un critère et sa tâche, entre les
   deux filles ; total incohérent avec sa ventilation.
5. **Ce que personne n'a encore énuméré** — chemins, registres, compteurs, gardes qu'aucune passe n'a
   nommés. *Six passes n'ont pas épuisé ce dépôt : la 6 y a trouvé deux sites neufs.*

## Ce que tu rends

- **Les findings** : sévérité, l'endroit exact, **la preuve**, le correctif. Nature : **découpage**,
  **conception**, **dérive extérieure** ou **procédure**.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** ⚠️ **Un « 0 finding »
  non adossé à cette liste ne clôt rien** — et c'est la dernière passe avant le plafond de budget :
  dis clairement ce qui reste non vérifié.

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh` (**le lire, jamais le lancer**), `scripts/regen-test-schema.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`merge`, `sqlx migrate`, `cargo test`/`cargo nextest`. Lecture, `grep`,
`sed -n`, `git show`/`diff`/`log`, `pdftotext` et `cargo check` sont autorisés.
