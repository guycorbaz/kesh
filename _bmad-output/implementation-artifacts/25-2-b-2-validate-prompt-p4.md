# Prompt — passe 4 de `bmad-create-story validate`, Story 25-2-b-2 — passe CIBLÉE

*Versionné le 2026-09-21. Une lentille en contexte frais (Opus) — les passes 1 et 3 étaient sur Sonnet, la
passe 2 sur Opus, l'auteur des patches est un Opus. Passe **ciblée** : elle ne relit pas la fiche,
elle relit **la remédiation de la passe 2**.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-2-b-1-devalidation-depot-api` (les deux fiches
filles y vivent), `main` au `951cbce2`.

## L'objet — le SEUL commit de remédiation

```sh
git show 4812d6ec -- _bmad-output/implementation-artifacts/25-2-b-2-retrait-suppression-ecran-manuels.md
```

La passe 2 avait rendu 3 HIGH, 6 MEDIUM et 5 LOW, **douze sur quatorze de nature découpage**. Les
patches ont : recalé **sept citations** du manuel utilisateur sur `main` (la 25-2-b-zero, mergée
entre les passes, y avait inséré trois lignes) ; ajouté **deux sites créés par cette même story**
(`user-manual.tex:466-468`, `admin-manual.tex:1798`) ; porté la plage de refonte à `1003-1028` ;
rétabli **l'élargissement de rôle** du bouton (`isAdmin` → garde comptable) ; nommé **huit
doc-comments** qui deviennent faux ; précisé le retrait du bloc d'écran en **conservant** la branche
`{:else}` ; nommé le bouton de la branche **brouillon** (`:615-629`) ; dit que cinq des huit tests se
**retargettent** au lieu de se réécrire ; rétabli la clause i18n générique ; ajouté la § *Règle de
splitting*.

## Ce que la passe 3 a corrigé, et où le défaut se logera cette fois

La passe 3 a rendu 1 HIGH, 2 MEDIUM, 1 LOW. Les patches ont : **tranché une contradiction interne**
(l'AC 7 niait un contrôle que le compte rendu de la même passe déclarait fait — le contrôle a été
refait, les quatre passages sont dans le PDF) ; corrigé `invoices.rs:1228-1231` → **`1232-1282`**,
une citation **introduite par la remédiation précédente et fausse** ; remplacé la règle en bloc du
sort des huit tests par un **tableau les nommant un par un** ; corrigé `+page.svelte:615-629` →
`616-630`.

⛔ **Deux fois de suite, le défaut s'est logé dans ce qu'une passe venait d'écrire** — une citation
neuve fausse en passe 2, une autre en passe 3. **Commence par les quatre corrections ci-dessus**,
puis le reste.

## Les axes, et tu déclareras lesquels tu as exercés

1. ⛔ **Les quatre corrections de la passe 3 sont-elles justes ?** `invoices.rs:1232-1282` —
   l'arme `Some(inv) if inv.status == "validated"` commence-t-elle bien à 1232 et finit-elle bien à
   1282 ? `+page.svelte:616-630` — le bloc `{#if invoice?.status === 'draft'}` ? Et le **tableau des
   huit tests** : chaque destination est-elle tenable, et les cinq « supprimés » ont-ils vraiment
   leur contrepartie dans la fiche de la **25-2-b-1** — vérifie-le dans sa fiche, nommément, motif
   par motif. *C'est là que les deux dernières passes ont trouvé leur défaut.*
2. **Le recalage est-il complet ?** Reste-t-il, dans la fiche, une ligne citée qui vaille encore
   pour `44c6842f` et non pour `main` ? Et la fiche **sœur** (`25-2-b-1-devalidation-depot-api.md`)
   porte-t-elle des citations du même manuel, non recalées ?
3. **Les ajouts créent-ils leur propre défaut ?** L'élargissement de rôle est-il réalisable tel
   qu'écrit (`canManage`, `:65-67`) ? Le retrait « `:919`, `:920-942`, garder `:944` » laisse-t-il
   un fichier cohérent — balises fermées, `{:else}` devenu inconditionnel ? Les huit doc-comments
   nommés existent-ils tous, et sont-ils **tous** faux après cette story ?
4. **Le « retargetage » des cinq tests est-il tenable ?** Un test retargeté sur
   `INVOICE_MUST_BE_UNVALIDATED_FIRST` prouve-t-il encore quelque chose, ou devient-il cinq fois le
   même test ? Ce que la b-1 couvre vraiment se vérifie dans sa fiche.
5. **Les décomptes.** `HUIT` partout, sans résidu ; `sprint-status.yaml` recalé ; aucun nombre neuf
   introduit sans sa source.

## Ce que tu rends

- **Les findings** : sévérité, l'endroit exact, **la preuve**, le correctif. Pour chacun, dis s'il
  est de nature **découpage** ou **conception**.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/`add`/`stash`/`reset`,
`sqlx migrate`, `cargo test`/`cargo nextest`. Lecture, `grep`, `sed -n`, `git show`/`diff`,
`pdftotext` et `cargo check` sont autorisés.
