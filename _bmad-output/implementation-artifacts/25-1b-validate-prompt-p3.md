# Prompt — passe 3 de `bmad-create-story validate`, Story 25-1b

*Versionné le 2026-09-12. Lentille unique (Sonnet 4.6), contexte frais, après la remédiation
de la passe 2.*

Tu es un **valideur adversarial** en contexte frais. Ton objet est
`_bmad-output/implementation-artifacts/25-1b-trous-alimentation.md`, dépôt
`/home/gcorbaz/devel/kesh`.

## Deux fronts, et le second est celui qu'on néglige

### (A) Ce que la passe 2 a fait écrire

Lis `git show 4ed81f66` **et** le commit qui le suit, puis interroge chaque patch :

1. **AC 3 — quatre transitions** (`created`, `reactivated`, `completed`, `discarded`) et deux
   extractions `_in_tx`. La quatrième (`completed`) vient d'être ajoutée : *est-elle juste ?*
   `mark_completed` est appelée dans un bloc `async` où une facture fournisseur vient d'être
   créée et auditée — **deux entrées d'audit dans la même transaction**, est-ce voulu, ordonné,
   sans interférence ?
2. **AC 6 — le chemin d'amorçage** `auth/bootstrap.rs:123` entre dans le périmètre. A-t-il une
   transaction ? Le sous-`SELECT` d'`actor_label` verrait-il l'utilisateur créé ? Et quel
   `entity_id` quand l'`INSERT` n'est pas encore commité ?
3. **AC 11 — les trois routes de `test_endpoints.rs`** à exempter nommément, l'échec bruyant sur
   doublon verbe+handler, l'ancrage sur `routes::`.
4. **AC 12 — cinq sites dans deux manuels.** Vérifie les cinq au sol, **PDF aplati compris**, et
   **cherche un sixième** : l'inventaire de ces documents a été pris en défaut **six fois** sur
   ces deux stories, dont deux fois en se déclarant complet.
5. **T4 — la position de la transaction** autour de la seule insertion, le rollback avant
   `dispose_failed`, le pool à 5 connexions.
6. **Les décomptes** : la spec annonce **14 routes**, **12 AC**, **11 tâches**. Recompte depuis la
   source, et vérifie que **le passage de 13 à 14 a été propagé partout** — c'est exactement le
   défaut que la story sœur a payé quatre fois.

### (B) ⛔ Les axes que la passe 2 a DÉCLARÉ ne pas avoir exercés

*C'est là que se trouve ce qu'elle a manqué, et c'est la moitié de ta mission.*

- **Les 73 routes dites « tracées » n'ont jamais été vérifiées une par une.** Sonde-les : en
  existe-t-il une qui ne le soit pas, ou dont la trace ne parte que d'une branche ?
- **Le frontend** : la spec affirme *« aucune surface visible ne change »*. Exerce-le.
- **i18n** : aucun catalogue n'a été consulté. Une action d'audit, un libellé, une clé
  manquent-ils ?
- **`kesh-seed`** sous les extractions `_in_tx` — `kesh-seed/src/lib.rs:122` appelle
  `companies::update`. Que devient-il ?
- **La politique de migration (P2-bis à P8)** : la spec affirme n'introduire **aucun** changement
  de schéma. Essaie de la prendre en défaut.
- **Le contenu de [#434] et [#435]** : seuls leurs titres avaient été lus. La spec les cite comme
  exemptions — disent-ils bien ce qu'elle leur fait dire ?
- **La faisabilité compilée** : tu **peux** lancer `cargo check`/`cargo clippy` (ils n'écrivent que
  dans `target/`). Tu ne peux pas prototyper. Dis donc explicitement ce qui reste **raisonné** et
  non **prouvé**, en particulier les emprunts dans les branches d'erreur de `process_one_file`.

## Ce que tu rends

- **Les findings** : sévérité, endroit exact, **la commande ou l'extrait qui l'établit**, et le
  correctif.
- **Pour chacun : né d'un patch de la passe 2, ou d'origine ?**
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding »
  non adossé à cette liste ne clôt rien. Ne qualifie jamais de « robuste » un point que tu n'as
  pas exécuté.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui mute** — nommément
`scripts/prepare-release.sh`, `scripts/install-hooks.sh`, tout `git commit`/`push`/`checkout`/`add`,
`sqlx migrate`, toute écriture en base. `cargo check` et `cargo clippy` sont autorisés.
