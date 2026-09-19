# Prompt — passe 2 de `bmad-code-review`, Story 25-2-c — passe CIBLÉE

*Versionné le 2026-09-19. **Une seule lentille**, « Regression Hunter », en contexte frais, sur
Sonnet — orthogonale à l'auteur du correctif (Opus 5). Passe ciblée au sens du `CLAUDE.md`
(§ « La passe ciblée ») : la passe 1 n'a rien trouvé au-dessus de MEDIUM, son correctif touche
un seul module et ne change aucune règle métier. Précédent : `22-2b-review-prompt-regression-hunter-p5.md`.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-2-c-numero-ecriture-par-compteur`.

## L'objet — le SEUL commit de remédiation

```sh
git show cd11aa35 -- . ':(exclude)_bmad-output'
```

C'est ce qu'il faut relire, et rien d'autre : le motif mesuré de ce dépôt est que *la sévérité
se déplace vers ce qu'on vient d'écrire*. Le reste de la story a été relu en passe 1 (rapport
consigné au Change Log de `_bmad-output/implementation-artifacts/25-2-c-numero-ecriture-par-compteur.md`,
entrée « revue P1 »). Tu peux lire le code environnant autant que nécessaire pour juger le commit.

## Ce que le commit fait

1. **`crates/kesh-db/src/repositories/journal_entry_number_sequences.rs`** — la lecture du
   « plancher » (`SELECT COALESCE(MAX(entry_number), 0) + 1 FROM journal_entries WHERE …`) passe
   en **`FOR UPDATE`**. Motif : sous `REPEATABLE READ`, elle lisait l'instantané de la première
   lecture de `create_in_tx` et ratait une écriture validée entre-temps. Plus un doc-comment de
   précondition sur `INSERT IGNORE`.
2. **`crates/kesh-db/src/repositories/journal_entries.rs`** — un test neuf,
   `le_plancher_voit_une_ecriture_validee_pendant_la_transaction`, qui reproduit la course.
   Déclaré rouge sur le code d'origine, vert après.
3. Documentation : ligne 91 de `docs/migrations-idempotence-audit.md`, commentaires de
   `crates/kesh-db/tests/migrations_upgrade_path.rs`, ligne E25 du `README.md`.

## ⛔ Les axes, et tu déclareras lesquels tu as exercés

1. **Le verrou ajouté introduit-il un interblocage ?** `create_in_tx` prend, dans l'ordre :
   l'exercice (`fiscal_years … FOR UPDATE`), le compteur, puis — nouveau — un verrou
   d'intervalle sur `journal_entries (company_id, fiscal_year_id, entry_number)`. Inventorie
   les **autres** chemins qui verrouillent des lignes de `journal_entries` **puis** l'exercice
   ou le compteur (modification, suppression, contre-passation, clôture d'exercice,
   rapprochement, validation de facture, encaissement) : un ordre inverse existe-t-il ? Le
   commentaire affirme que ce verrou d'intervalle « est celui que posait déjà le `MAX + 1`
   d'avant cette story » : **vérifie-le sur `main`** (`git show main:crates/kesh-db/src/repositories/journal_entries.rs`),
   y compris l'**ordre** dans lequel l'ancien code le prenait par rapport aux autres verrous.
2. **Le test prouve-t-il ce qu'il dit ?** Rougirait-il si l'on retirait `FOR UPDATE` ? Le montage
   (instantané ouvert par une lecture non verrouillante, insertion hors bande sur une autre
   connexion, puis allocateur) reproduit-il vraiment la course sous l'isolation effective du
   pool ? Laisse-t-il un résidu en base (la règle du dépôt : « un gate laisse la base piégée »)
   qui ferait rougir un test **suivant** ? Est-il sensible à l'ordre ou au parallélisme des tests
   (cf. `.config/nextest.toml`, groupe des tests lib de `kesh-db`) ?
3. **Les commentaires ajoutés disent-ils vrai ?** Doc-comment de précondition (« `create_in_tx`,
   seul appelant de production, verrouille l'exercice et rejette son absence ») : vérifie
   « seul appelant » par grep. Commentaire du plancher : chaque affirmation.
4. **La documentation corrigée.** Ligne 91 de l'audit : le fondement déclaré correspond-il
   désormais **mot pour mot** à la justification réelle dans `crates/kesh-db/src/post_restore.rs`
   (exemption `20260917000001`) ? `migrations_upgrade_path.rs` : recompte chaque jeton
   (`grep -nE "\b(33|34|35|68|69)\b"`) — frontière 34, fenêtre 35, total 69 — et signale tout
   jeton resté faux **ou rendu faux** par le correctif. README : la phrase ajoutée dit-elle vrai,
   sans sur-promettre (le trou subsiste) ?
5. **Propagation.** Le correctif a-t-il laissé ailleurs dans le dépôt une affirmation devenue
   fausse — un autre site qui décrit le plancher comme non verrouillant, ou la sérialisation
   comme reposant sur le seul verrou du compteur ?

## Ce que tu rends

- **Les findings** : sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), `fichier:ligne`, **la
  commande ou l'extrait qui l'établit**, et le correctif. Pour un scénario, montre que son état de
  départ est atteignable en production.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne compte pas comme passe. **Ne qualifie jamais de « vérifié » un point que
  tu n'as pas exécuté.**

## Discipline imposée par ce dépôt

⛔ Tout finding `CRITICAL` ou `HIGH` qui affirme l'ABSENCE d'un code attendu ou la PRÉSENCE d'un
anti-pattern doit être établi par un `grep -nF` cité dans ton rapport. Sans cette preuve il sera
écarté.

## Interdits

⛔ **N'écris AUCUN fichier du dépôt et n'exécute AUCUNE commande qui mute** — nommément
`scripts/prepare-release.sh` (il bumpe les versions Cargo), `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans
`docs/manual/`, tout `git commit`/`push`/`checkout`/`add`/`stash`/`reset`, `sqlx migrate`,
`cargo fmt` sans `--check`, `cargo test`/`cargo nextest` (ils écrivent dans la base partagée), et
toute écriture dans les bases `kesh` ou `kesh_e2e`. Lecture, `grep`, `git diff`/`log`/`show` et
`cargo check` sont autorisés.
