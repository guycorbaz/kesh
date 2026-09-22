# Prompt — passe 3 **CIBLÉE** de `bmad-code-review`, Story 25-2-b-1

*Versionné le 2026-09-22. **Une** lentille (Sonnet ; P1 était Sonnet ×3, P2 Opus + Haiku), en
contexte frais, braquée sur **le seul commit `7552d823`** — la remédiation de la passe 2.*

## Pourquoi cette passe est ciblée, et pourquoi elle porte sur les AFFIRMATIONS

La boucle converge : `3 HIGH → 0 HIGH` entre les passes 1 et 2. Et la remédiation de la passe 2
**ne touche aucune ligne de code exécutable** — vérifié :

```sh
git show 7552d823 -- crates/kesh-db/src/repositories/*.rs | grep -E '^\+' | grep -vE '^\+\+\+|^\+\s*//'
# (vide)
```

Elle ne contient que des **doc-comments**, un **test**, de la **documentation externe** et le story
file. Le motif habituel — *la remédiation casse le code* — n'a donc plus de prise.

⛔ **Mais un autre motif, lui, tient depuis trois passes d'affilée, et c'est le tien** : *chaque
passe a trouvé une affirmation fausse dans ce que la passe précédente venait d'écrire.* Passe 1 :
le Dev Agent Record déclarait cinq empêchements prouvés par mutation, deux ne l'étaient pas. Passe 2
(F-R1) : le paragraphe sur `details` écrit par la remédiation de la passe 1 était faux sur trois de
ses quatre affirmations. Passe 2 (F-R2) : le doc-comment corrigé par la passe 1 annonçait cinq
gardes et en énumérait six. Et en corrigeant F-R1, l'auteur a écrit « les **quatre** autres codes »
suivi de **cinq** — rattrapé à la volée, **quatrième** faute de décompte de la même famille.

**Ton objet n'est donc pas le code : c'est chaque phrase que ce commit ajoute, confrontée au code.**

## Ce que tu fais, exactement

```sh
git show 7552d823 --stat
git show 7552d823
```

Pour **chaque affirmation ajoutée ou modifiée** par ce commit — dans `docs/api-external.md`, dans
les doc-comments de `crates/kesh-db/src/repositories/{invoices,journal_entries}.rs`, dans les
commentaires du test, et dans le story file — tu réponds à trois questions :

1. **Est-elle vraie ?** Vérifie-la dans le code actuel, pas dans le commit qui l'énonce.
2. **Est-elle complète ?** Une affirmation vraie mais partielle sur un contrat public égare autant
   qu'une fausse — c'était le cas de F-R1.
3. **Tout total est-il cohérent avec sa propre ventilation ?** Recompte depuis la source, avec la
   commande. ⛔ **C'est la faute la plus récurrente de cette story : quatre occurrences.**

Points nommés, parce qu'ils sont les plus chargés :

- **Le tableau `details` de `docs/api-external.md`** — chaque cellule, contre `errors.rs` et contre
  les cinq sites de construction dans `invoices.rs`. Les `null` annoncés sont-ils exacts ? La ligne
  « 10 refus = 5 avec `details` + 5 sans » se recompte-t-elle sur les deux tableaux ?
- **Le doc-comment de `delete_in_tx`** sur les gardes de `delete` (trois) et d'`unvalidate` (cinq),
  et sur l'affirmation neuve que « le règlement partiel n'est pas une garde de plus, c'est la même
  en plus large ». Vrai ? Vérifie les deux fonctions.
- **Le doc-comment rendu à `ValidatedInvoice`** : est-il maintenant attaché à la bonne struct, et
  `unvalidate` a-t-elle retrouvé une ligne de résumé correcte ? Reste-t-il, ailleurs dans ces deux
  fichiers, un doc-comment décroché de ce qu'il documente ?
- **Les assertions neuves du test** : `get("journalEntryId")` distingue-t-il réellement l'absence
  du `null` ? `version + 1` est-il la bonne valeur attendue ? Ces assertions passeraient-elles
  encore si ce qu'elles prétendent couvrir disparaissait ?
- **Le § *Périmètre* du story file** sur le manuel : « il y en a **deux** dès ce merge » — recompte
  les chemins qui suppriment une écriture de facture. Le site `user-manual.tex:540` est-il bien
  celui cité, et la b-2 le porte-t-elle bien dans ses ancrages ?
- **Le Change Log de la passe 2** : chaque nombre qu'il avance.

## Ce que tu rends

- **Les findings** : sévérité, `fichier:ligne`, **la commande ou l'extrait qui l'établit**, le
  correctif. Une affirmation fausse sur un **contrat public** (`docs/api-external.md`) est au moins
  MEDIUM ; dans un doc-comment, au moins LOW — et MEDIUM si elle égare sur une garde.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne clôt rien : il sera repris à la main.

## Interdits

⛔ **N'écris aucun fichier et n'exécute aucune commande qui écrit** — dans le dépôt comme en base.
Nommément : `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout
`git commit`/`push`/`checkout`/`add`/`stash`/`reset`/`rebase`/`restore`, `sqlx migrate`,
`cargo test`, `cargo nextest`, `docker … restart`, et tout `mariadb` portant `INSERT`, `UPDATE`,
`DELETE`, `DROP` ou `CREATE`. Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`,
`pdftotext`, `cargo check`, `mariadb` en `SELECT`/`SHOW`.

⚠️ **Ne re-signale pas** : le re-fetch post-commit de `settle_invoice_handler`
(`routes/invoices.rs`, sur `main`) ; le variant `InvoiceMustBeUnvalidatedFirst` sans appelant (posé
pour la b-2) ; la redondance de `devalider_refuse_une_version_perimee` ; le fait que `delete` ne
lise que `paid_at` (transmis à la b-2). Tous connus et arbitrés.
