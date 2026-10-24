# Epic 32 — registre des choix autonomes de la story 32-12 (import d'une sauvegarde antérieure)

À verser par **union** dans `epic-32-choix-autonomes.md` au merge (`docs/agents-orchestration.md` § 1). Fiche :
`32-12-import-sauvegarde-anterieure.md`. Date : 2026-10-24. Toute la story est **sous réserve de Q4** (C-32-0-13).

---

### C-32-12-1 — Fiche complète malgré la réserve, avec sa forme réduite écrite en tête

- **Contexte** : Q4 n'est pas tranchée ; la 32-5 attend cette story.
- **Retenu** : fiche complète ; un encadré de tête dit que, si Guy accepte la limite, seuls AC20 (manuel) et AC21
  (CHANGELOG) subsistent, récrits pour **dire** la limite et conseiller une sauvegarde juste après la mise à jour.
- **Écartée** : attendre la réponse (bloque la 32-5).
- **Réversible** : oui.

### C-32-12-2 — Un plancher d'import qui gèle la fenêtre, au lieu d'une tolérance sans borne

- **Contexte** : la seule règle « tables absentes postérieures aux tables présentes » rendrait importables des
  sauvegardes de la 0.11 et rouvrirait les huit exemptions « Hors fenêtre » de `EXEMPT_MIGRATIONS`.
- **Retenu** : `IMPORT_FLOOR_MIGRATION = 20260917000001` (dernière migration créatrice de table applicative à la
  v0.15.0) ; toute table créée au plus tard par lui reste exigée ; la fenêtre cesse de se refermer à chaque table neuve.
  Aucune exemption ne devient fausse (inventaire des huit à la fiche, point 2).
- **Écartées** : tolérance sans plancher (rouvre les exemptions, élargit à des versions jamais testées) ; plancher
  calculé (aucune donnée du dépôt ne dit « la version publiée au moment où la règle est née ») ; plancher à chaque
  release (referme la fenêtre à chaque version, l'inverse de l'objet).
- **Réversible** : oui — relever le plancher est un geste délibéré, doc-comment à l'appui.

### C-32-12-3 — La version déclarée par le manifeste ne fonde pas la règle

- **Contexte** : `keshVersion` est un texte non signé ; et la 32-5 ajoutera sa table sans bump Cargo (migration non
  breaking), si bien qu'une règle « `keshVersion` < version du binaire » refuserait la sauvegarde 0.15.0 elle-même.
- **Retenu** : la preuve vient du schéma porté par le fichier (quelles tables, créées par quelle migration).
- **Écartée** : comparaison de versions SemVer ; table écrite à la main « table → première version publiée ».
- **Réversible** : oui.

### C-32-12-4 — Règle du suffixe : les absentes sont postérieures à toutes les présentes

- **Contexte** : « vérifié par le registre des migrations, non par le seul nom » (fichier d'epic).
- **Retenu** : une table absente doit être créée après le plancher **et** après chaque table présente ; deux tables
  nées dans la même migration sont toutes deux présentes ou toutes deux absentes. Carte table → migration dérivée du
  `MIGRATOR` (reconnaissance `CREATE TABLE` des tests, déplacée en production) ; une table sans migration connue est
  exigée.
- **Écartée** : carte écrite à la main (dérive silencieuse) ; tolérance de toute table postérieure au plancher sans
  ordre (accepte un fichier d'un ordre impossible).
- **Réversible** : oui.

### C-32-12-5 — Les colonnes ne datent pas le fichier

- **Contexte** : une colonne ajoutée depuis à une table ancienne date aussi le fichier.
- **Retenu** : non employée ; les stories de l'epic ne font que créer des tables.
- **Écartée** : généraliser `adds_column` (en `mod tests`) à la production — coût sans gain aujourd'hui.
- **Réversible** : oui — à reprendre si une story ajoute à la fois une table et une colonne datante.

### C-32-12-6 — Une table absente est restaurée vide (comportement actuel fixé, non changé)

- **Contexte** : `restore_body` vide déjà toutes les tables de l'inventaire et n'insère que les présentes.
- **Retenu** : vide — ni les lignes de la destination (deux installations mêlées ; clés MCP rattachées à des identifiants
  qui désignent d'autres personnes après l'import), ni les lignes posées par la migration (affaire du registre, AC11).
  Fixé par un test (AC4).
- **Écartées** : garder les lignes de la destination ; ré-exécuter la migration créatrice.
- **Réversible** : oui.

### C-32-12-7 — Fenêtre des tests P7 = plancher

- **Contexte** : `registry_entries_are_within_import_window` et `exemptions_claiming_out_of_window_really_are_out_of_window`
  prennent la dernière migration créatrice ; après la 32-5, le premier prescrirait de sortir M1 du registre (rejeu
  encore nécessaire aux sauvegardes 0.12.x–0.13.0), le second deviendrait permissif.
- **Retenu** : les deux comparent au plancher ; mutation déclarée équivalente jusqu'à la 32-5, qui la constate.
- **Écartée** : attendre la 32-5 pour le faire (le test y mentirait au premier rouge).
- **Réversible** : oui.

### C-32-12-8 — Une table postérieure amorcée va au registre, jamais en exemption

- **Contexte** : l'argument de couverture de `20260917000001` (« un backup sans ses lignes est un backup sans la
  table ») est faux pour une table postérieure au plancher, vidée à l'import.
- **Retenu** : test `post_floor_table_seeds_are_replayed` ; vacant aujourd'hui.
- **Écartée** : un marqueur textuel de plus dans `EXEMPT_MIGRATIONS` (les marqueurs se sont déjà montrés fragiles,
  `ExemptionBasis`).
- **Réversible** : oui.

### C-32-12-9 — Le bilan nomme les tables, à la réponse, à l'audit et à l'écran

- **Contexte** : « le bilan d'import le dit » (fichier d'epic).
- **Retenu** : `tablesAbsent: string[]` toujours présent (vide = rien ne manque), `tables_absent` à l'audit, encart
  d'avertissement quand la liste n'est pas vide, noms techniques des tables (lecteur : l'Administrateur).
- **Écartées** : un compte seul (ne dit pas quoi recréer) ; libellés traduits par table (une clé par table neuve, pour
  un écran rare).
- **Réversible** : oui.

### C-32-12-10 — Fixture 0.15.0 produite sur la branche, au premier commit, sans worktree temporaire

- **Contexte** : la fixture 0.14.0 a été produite dans un worktree au tag ; le code de la branche est celui de la v0.15.0
  (`git diff --stat v0.15.0 HEAD -- crates/ frontend/src` vide) ; disque saturé le 2026-10-24.
- **Retenu** : générateur `#[ignore]` jumeau, seed partagé, exécuté une fois avant tout changement de code, après
  constat du diff vide ; données synthétiques.
- **Écartées** : worktree au tag `v0.15.0` (20 à 50 Go de cible pour un résultat identique) ; fixture fabriquée à la
  main (`Fabrique` des tests unitaires : ne prouve rien du vrai exporteur).
- **Réversible** : oui.

### C-32-12-11 — La capture de l'état « tables absentes » est transférée à la 32-5

- **Contexte** : aucune table postérieure au plancher n'existe avant la 32-5 ; la recette visuelle veut une base de
  démonstration.
- **Retenu** : capture de l'état nominal ici ; l'état « tables absentes » capturé par la 32-5 sur un import réel ;
  l'encart prouvé ici par Vitest.
- **Écartée** : simuler la réponse dans le navigateur (capture d'un état que le serveur ne produit pas).
- **Réversible** : oui.

### C-32-12-12 — Tolérance prouvée bout à bout hors HTTP, par règles injectées

- **Contexte** : `TABLES_TO_TRUNCATE` est une constante ; aucune table réelle n'est postérieure au plancher.
- **Retenu** : `parse_and_verify_with` (porte de test, patron de `replay_with_registry`) ; test d'intégration qui
  déclare `products` postérieure, la retire de la fixture, et enchaîne vérification, compatibilité de schéma,
  restauration et rejeu.
- **Écartée** : injection des règles dans `AppState` (surface de production pour un test).
- **Réversible** : oui.

### C-32-12-13 — Messages de refus existants gardés mot pour mot

- **Contexte** : des tests cherchent `absente du manifeste` et `hors inventaire applicatif`.
- **Retenu** : `Exigee` et `Inconnue` gardent les messages ; seul `HorsOrdre` est neuf.
- **Réversible** : oui.

### C-32-12-14 — `CLAUDE.md` P7 amendé dans la PR, soumis à Guy

- **Contexte** : P7 dit « un backup n'est importable que si son inventaire de tables est identique au nôtre » ; faux
  après la story.
- **Retenu** : la PR récrit le paragraphe ; Guy l'accepte ou non au merge.
- **Réversible** : oui.
