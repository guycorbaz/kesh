# Prompt — passe 8 de `bmad-create-story validate`, Story 25-2-b-2 — passe CIBLÉE, **la dernière**

*Versionné le 2026-09-21. Une lentille en contexte frais (Opus). ⛔ **C'est la 8ᵉ passe : le plafond
de budget de la § Review Iteration Rule.** Après elle, la fiche part en développement avec ce
qu'elle porte — ou Guy arbitre autre chose. **Dis donc ce qui reste ouvert, aussi clairement que ce
que tu trouves.***

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-2-b-1-devalidation-depot-api` (`main` au
`951cbce2`, release `v0.12.0` comprise).

## L'objet

```sh
git show a400e703 -- _bmad-output/implementation-artifacts/25-2-b-2-retrait-suppression-ecran-manuels.md
```

Deux corrections seulement : le **chiffre des appels `i18nMsg` RETIRÉ** de l'AC 8 (il comptait
l'import et un commentaire — 41 et 7 en réalité, et l'instruction opérante était de recompter au
développement), et l'**ordinal** du site de migration reformulé (« dixième de la liste, douzième
ancrage »).

⛔ **Sept passes durant, le défaut suivant est né de la remédiation précédente** — six fois sur
sept. Ce patch-ci est minuscule : **commence par lui**, puis élargis.

## Les axes, et tu déclareras lesquels tu as exercés

1. ⛔ **Les deux corrections sont-elles justes, et ne créent-elles pas leur propre défaut ?** Le
   retrait du chiffre laisse-t-il l'AC 8 opérant — un développeur sait-il quoi faire ? L'ordinal
   reformulé est-il arithmétiquement vrai (9 autres + 2 déjà nommés + 1 migration) ?
2. ⛔ **Ce que la passe 7 a déclaré NON VÉRIFIÉ, et qu'elle a eu raison de déclarer** — vérifie-le
   maintenant, c'est la dernière occasion :
   - les **PDF aplatis** des deux manuels FR contre les sites de l'AC 7 (⚠️ l'apostrophe y est
     `’`, U+2019, et non `'`) ;
   - le total `28 + 93 + 2 = 123` **recompté depuis `crates/kesh-api/src/audit_labels.rs`** ;
   - les lignes `505`, `540-541`, `621`, `781`, `837`, `969` de `user-manual.tex` ;
   - les **quatre locales** au-delà de `journal-entries-reverse-blocked-invoice` : la story
     exige-t-elle des clés que la fiche ne nomme pas ?
3. **La fiche est-elle IMPLÉMENTABLE telle quelle ?** Lis-la comme un développeur qui l'ouvre demain
   sans rien savoir de cette boucle : quelque chose l'arrêterait-il, ou le ferait-il deviner ? Cite
   l'endroit.
4. **Les deux filles ensemble** : un critère porté deux fois, aucune fois, ou dans le mauvais ordre ?
5. **Ce que huit passes n'ont pas regardé.** Dis-le, même sans finding — c'est la valeur propre de
   cette dernière passe.

## Ce que tu rends

- **Les findings** : sévérité, l'endroit exact, **la preuve**, le correctif. Nature : **découpage**,
  **conception**, **dérive extérieure** ou **procédure**.
- ⛔ **La liste des axes exercés ET non exercés**, et **une section « ce qui reste ouvert au moment
  de clore »** : ce qu'un développeur devra trancher lui-même, et ce qu'une revue de code devra
  regarder en priorité.

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh` (**le lire, jamais le lancer**), `scripts/regen-test-schema.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`merge`, `sqlx migrate`, `cargo test`/`cargo nextest`. Lecture, `grep`,
`sed -n`, `git show`/`diff`/`log`, `pdftotext` et `cargo check` sont autorisés.
