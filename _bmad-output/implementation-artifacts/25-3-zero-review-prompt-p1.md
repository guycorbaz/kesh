# Prompt — passe 1 de `bmad-code-review`, Story 25-3-zero

*Versionné le 2026-09-24. **Deux** lentilles en contexte frais (Sonnet), orthogonales à l'auteur
(Opus 5).*

**Pourquoi deux et non trois.** La story est un **refactor pur** d'une seule fonction, dans un seul
fichier : son périmètre ne porte pas trois lentilles. Mais il en faut deux, et elles regardent des
choses opposées — l'une doit prouver que **rien n'a changé**, l'autre que **quelque chose a changé**
et que c'est bien ce qui était voulu.

## Préambule commun

Tu es un **relecteur adversarial** en contexte frais. Dépôt `/home/gcorbaz/devel/kesh`, branche
`story/25-3-annuler-reglement-et-rapprochement`. Périmètre : le diff **`main...HEAD`**, restreint à
`crates/kesh-db/src/repositories/journal_entries.rs`.

```sh
git diff main...HEAD --stat
git diff main...HEAD -- crates/kesh-db/src/repositories/journal_entries.rs
```

La spec est `_bmad-output/implementation-artifacts/25-3-zero-reverse-in-tx.md`. La fiche mère
`25-3-annuler-reglement-et-rapprochement.md` (statut `split`) est la **source des faits** ; elle
porte en tête un tableau de **cinq corrections** — des décomptes que la spécification avait faux.
*Tiens le même soupçon pour tout ce que tu liras.*

⛔ **Rien ne se croit sur parole** — ni la spec, ni les commentaires, ni le Dev Agent Record, ni les
messages de commit. Chaque affirmation se vérifie dans le code actuel.

⛔ **Grep ground-truth obligatoire.** Tout finding `CRITICAL` ou `HIGH` affirmant **l'absence d'un
code attendu** ou **la présence d'un anti-pattern** se vérifie avant d'être rendu, par
`grep -nF "<chaîne exacte>" <fichier>` — le `-F` est obligatoire. Pour un bloc, `grep -nFA 5`. Sans
motif discriminant, lecture directe, et **cite l'extrait lu**.

⚠️ **Ne conteste pas les arbitrages** : le découpage de la 25-3 en trois ; le fait que cette story
n'annule rien ; le fait qu'elle ne touche pas `supplier_invoices::cancel`, dont la contre-passation
manuelle est un défaut **antérieur** explicitement laissé à la 25-3-a.

## Ce que tu rends

- **Les findings** : sévérité, `fichier:ligne`, **la commande ou l'extrait qui l'établit**, le
  correctif. Pour un scénario, montre que son état de départ est **atteignable**.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne compte pas comme passe.

## Interdits

⛔ **N'écris aucun fichier et n'exécute aucune commande qui écrit** — dans le dépôt comme en base.
Nommément : `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout
`git commit`/`push`/`checkout`/`add`/`stash`/`reset`/`rebase`/`restore`, `sqlx migrate`,
`cargo test`, `cargo nextest`, `docker … restart`, et tout `mariadb` portant `INSERT`, `UPDATE`,
`DELETE`, `DROP` ou `CREATE`. Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `cargo check`,
`mariadb` en `SELECT`/`SHOW`.

---

## Lentille A — « rien n'a changé » : prouve le contraire

**Ton hypothèse de travail : le refactor a modifié le comportement, et personne ne l'a vu.** Le
commit l'affirme *mesuré* parce qu'aucun test n'a été retouché — mais un comportement non couvert
par un test peut avoir bougé sans que rien ne rougisse.

1. ⛔ **Compare le corps AVANT et APRÈS, ligne à ligne.**
   `git show main:crates/kesh-db/src/repositories/journal_entries.rs` contre la version actuelle.
   L'ordre des étapes est-il **identique** — verrou `FOR UPDATE`, recensement des empêchements,
   garde des comptes archivés, recherche d'exercice ouvert, lecture ordonnée des lignes, création
   avec `reverses_entry_id`, audit ? Une étape déplacée, même d'un cran, est un finding.
2. **Les déréférencements.** L'extraction a transformé `&mut *tx` en `&mut **tx` (3 sites) et
   `&mut tx` en `tx` (4 sites). **Chacun visait-il la bonne cible ?** Un `&mut **tx` posé là où il
   fallait `tx` — ou l'inverse — compile parfois et change ce qui est verrouillé.
3. **Le wrapper.** `reverse` fait-il exactement ce que faisait l'original : `begin`, appel,
   `commit` sur succès, `rollback` sur erreur ? ⚠️ L'original faisait `let _ = tx.rollback().await;`
   — le résultat du rollback était **ignoré**. Est-ce toujours le cas, et est-ce voulu ?
4. **Les chemins d'erreur.** Chaque `?` du corps extrait remonte-t-il au même endroit qu'avant ?
   Une erreur qui sortait de l'`async` block et déclenchait le rollback sort-elle maintenant de
   `reverse_in_tx` **sans** rollback — et est-ce correct, l'appelant en étant responsable ?
5. **Les appelants.** `grep -rn "journal_entries::reverse\b"` — le seul appelant connu est
   `routes/journal_entries.rs:460`. Y en a-t-il d'autres, dans les tests ou ailleurs, dont le
   contrat aurait changé ?

## Lentille B — le test, les doc-comments et les comptes rendus

1. ⛔ **Le test de composabilité tranche-t-il VRAIMENT ?** Le commit dit l'avoir prouvé par
   mutation. **Refais le raisonnement** : la mutation décrite — un `COMMIT` au milieu de
   `reverse_in_tx` — ferait-elle échouer ce test-là, sur **assertion** et non sur erreur SQL ? Et
   le test passerait-il encore si `reverse_in_tx` ne faisait **rien du tout** ?
2. **Le montage du test.** Construit-il l'état qu'il croit ? `create`, `reverse_in_tx`, drop sans
   `commit`. Le `delete_all_by_company` final suffit-il, et que laisse le test **s'il panique au
   milieu** — la base des tests de dépôt est **partagée** (KF-039, #310) ?
3. **Les assertions.** `reverses_entry_id == Some(origine)`, puis zéro inverse, puis
   `reversed_by == None`. La troisième ajoute-t-elle quelque chose à la deuxième, ou est-elle
   redondante ? Une assertion redondante n'est pas un défaut, mais elle ne doit pas être **comptée**
   comme une garde.
4. **Les doc-comments neufs.** Chaque affirmation est-elle vraie ? En particulier celle sur
   `supplier_invoices::cancel` : vérifie **dans le code** qu'elle ne pose jamais
   `reverses_entry_id` et ne consulte jamais `reversal_blocker`. Et celle sur le rollback
   automatique de `sqlx` au drop.
5. ⛔ **Les comptes rendus.** Le Dev Agent Record affirme : `git diff --stat` **vide** sur les
   répertoires de tests, **27** tests de `journal_entry_reversal_e2e`, **3 + 4** corrections de
   déréférencement, gate **2419/2419**. **Recompte tout ce qui est recomptable sans exécuter**, et
   dis ce qui ne l'est pas.
6. **Ce que la story OMET.** Reste-t-il, dans le fichier, un appelant interne qui aurait dû passer
   à `reverse_in_tx` ? Une fonction du même module qui ouvre sa propre transaction et gagnerait à
   être composée — ou qui, elle aussi, réécrit une contre-passation ?
