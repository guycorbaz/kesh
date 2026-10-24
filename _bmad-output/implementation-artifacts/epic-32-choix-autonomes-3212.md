# Epic 32 — registre des choix autonomes de la story 32-12 (import d'une sauvegarde antérieure)

À verser par **union** dans `epic-32-choix-autonomes.md` au merge (`docs/agents-orchestration.md` § 1). Fiche :
`32-12-import-sauvegarde-anterieure.md`. Date : 2026-10-24. Toute la story est **sous réserve de Q4** (C-32-0-13).
Découpée le 2026-10-24 en 32-12a et 32-12b (C-32-12-37) : les choix 1 à 36 valent pour les deux clés, chacune pour les
AC qu'elle porte.

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
- **Révisé à la validation P1 (2026-10-24, annoté à la P2)** : le worktree au tag n'est plus écarté sans condition — il
  est le **repli** si `git diff --stat v0.15.0 HEAD -- crates/` n'est plus vide au développement (C-32-12-21).

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
- **Révisé à la validation P1 (2026-10-24)** : le placement du test et la visibilité de la porte sont fixés par
  C-32-12-15 — le « patron de `replay_with_registry` » ne transposait pas (module `pub` en `kesh-db`, quand
  `admin_backup` est `pub(crate)`).

### C-32-12-13 — Messages de refus existants gardés mot pour mot

- **Contexte** : des tests cherchent `absente du manifeste` et `hors inventaire applicatif`.
- **Retenu** : `Exigee` et `Inconnue` gardent les messages ; seul `HorsOrdre` est neuf.
- **Réversible** : oui.

### C-32-12-14 — `CLAUDE.md` P7 amendé dans la PR, soumis à Guy

- **Contexte** : P7 dit « un backup n'est importable que si son inventaire de tables est identique au nôtre » ; faux
  après la story.
- **Retenu** : la PR récrit le paragraphe ; Guy l'accepte ou non au merge.
- **Réversible** : oui.

---

*Choix 15 à 21 : remédiation de la validation P1 (2026-10-24, rapports `3212-validate-p1-{R,F}.md`).*

### C-32-12-15 — Le test de l'AC7 vit dans la crate ; `admin_backup` reste `pub(crate)`

- **Contexte** : F1 (HIGH) = R1 — `crates/kesh-api/src/lib.rs:10` déclare `pub(crate) mod admin_backup;`, avec un
  commentaire de sécurité de la revue 17-3a (module qui manipule le dump complet, hash de mots de passe compris) ; un
  test de `tests/` ne voit ni `parse_and_verify_with`, ni `check_schema_compat`, ni `ParsedBackup`.
- **Retenu** : AC7 dans `crates/kesh-api/src/admin_backup/import.rs`, `mod tests`,
  `#[sqlx::test(migrations = "../kesh-db/test-schema")]` (précédent `auth/bootstrap.rs:482`), fixture par
  `include_bytes!`, retrait de `products` par une fonction locale ; porte `pub(crate)`, sans `#[doc(hidden)]` (sans objet
  dans un module `pub(crate)`).
- **Écartées** : ré-exporter `admin_backup` ou ses fonctions (surface publique d'un module à secrets, pour un test) ;
  passer l'AC7 par le handler HTTP (exigerait d'injecter les règles dans `AppState`, déjà écarté par C-32-12-12).
- **Réversible** : oui.

### C-32-12-16 — L'AC2 (iii) s'écrit en union et disjonction

- **Contexte** : F2 = R3 — « exigées == `TABLES_TO_TRUNCATE` » rougirait à la 32-5 sans que le contrat dise quoi en faire.
- **Retenu** : `exigées ∪ tolérées == TABLES_TO_TRUNCATE` et `exigées ∩ tolérées == ∅`, tolérées = la liste écrite de
  (iv) ; vrai aujourd'hui, vrai à la 32-5 une fois sa table inscrite ; M3 reste tuée.
- **Écartée** : garder l'égalité et ajouter au contrat « réécrire (iii) » (un test que chaque story récrit n'est plus
  un garde).
- **Réversible** : oui.

### C-32-12-17 — Grep de l'AC17 ciblé ; commentaires des migrations publiées laissés faux, rectifiés ailleurs

- **Contexte** : F3 = R2 — la commande rendait 103 lignes et manquait la famille « importable » et le doc-comment de
  `missing_sentinels` ; trois migrations publiées (`20261024000001:5-7`, `20261001000004:15`, `20261009000001:11`)
  décrivent l'ancienne règle d'égalité.
- **Retenu** : motif ciblé (28 lignes au relevé de `6d24f6d1`) ; AC16 étendu à `backup_schedule.rs:502`,
  `post_restore.rs:718-720`, `import.rs:14` et `:232-233`, `backup.rs:667-668` ; les migrations **ne se touchent pas**
  (P8) — rectification par une phrase du doc-comment d'`IMPORT_FLOOR_MIGRATION` et par les Dev Notes ; la ligne d'audit
  d'idempotence de `20261024000001` reste vraie et n'est pas touchée.
- **Écartée** : corriger le commentaire des migrations (le binaire refuserait de démarrer sur toute base qui les porte).
- **Réversible** : oui.
- **Révisé à la validation P2 (2026-10-24)** : les migrations publiées concernées sont **quatre**, non trois —
  `20261003000001_invoices_frozen_pdf.sql:9-11` manquait (R1 = F1) ; le motif unique est remplacé par cinq familles de
  `grep` (C-32-12-22).

### C-32-12-18 — Deux migrations créatrices d'une même table : la plus ancienne, en production

- **Contexte** : F5 — la carte table → une version ne disait pas laquelle retenir ; inatteignable aujourd'hui.
- **Retenu** : la plus ancienne (table exigée : refus bruyant) ; le test AC1 (i) reste le garde qui le signale.
- **Écartée** : la plus récente (pourrait tolérer à tort une table ancienne).
- **Réversible** : oui.

### C-32-12-19 — Texte de l'encart : « Ressaisissez-les depuis l'application si nécessaire »

- **Contexte** : F6 — « Elles se recréent depuis l'application » promettait plus que le contrat (valide vide, ou amorcée).
- **Retenu** : formule qui n'affirme pas de recréation automatique.
- **Réversible** : oui.

### C-32-12-20 — Mutation M9 (`tablesAbsent` codé à `[]`) : report assumé à la 32-5

- **Contexte** : R4 — aucun test ne voit une liste non vide traverser le handler avant qu'une table postérieure existe.
- **Retenu** : M9 déclarée non tuée ; tuée par la reprise de l'AC8 par la 32-5 (`tablesAbsent == [<sa table>]`).
- **Écartée** : extraire le JSON du bilan en fonction pure testée avec une liste non vide (site de plus, pour une preuve
  que la 32-5 apporte de toute façon sur le vrai chemin).
- **Réversible** : oui.

### C-32-12-21 — Repli de la fixture si la branche ne porte plus le code de la v0.15.0

- **Contexte** : R7 — C-32-12-10 suppose `git diff --stat v0.15.0 HEAD -- crates/` vide au développement.
- **Retenu** : sinon, worktree temporaire au tag `v0.15.0` (patron de la fixture 0.14.0), disque vérifié, retiré aussitôt ;
  jamais de génération avec un diff non vide.
- **Réversible** : oui.

---

*Choix 22 à 29 : remédiation de la validation P2 (2026-10-24, rapports `3212-validate-p2-{R,F}.md`).*

### C-32-12-22 — La classe des textes devenus faux s'inventorie par son symptôme, non site par site

- **Contexte** : R1 = F1 et R2 = F3 — deux validations de suite ont trouvé des phrases devenues fausses que le motif
  unique de l'AC17 ne rendait pas (une quatrième migration ; « recalculée depuis le `MIGRATOR` », « dernière
  créatrice »), parce qu'elles ne portaient aucun de ses mots.
- **Retenu** : cinq familles de `grep` (importabilité, égalité, fenêtre calculée, dernière créatrice, couverture) et la
  valeur `20260917000001`, sur code, tests, migrations, `CLAUDE.md`, `docs/`, README, `website/` ; tableau
  *Inventaire de la classe* aux Dev Notes, où chaque site a un sort (récrit, intouchable P8 et rectifié ailleurs, repris
  par la 32-5, ou reste vrai avec la raison) ; une ligne absente du tableau est un site neuf à classer. La rectification
  des quatre migrations P8 les nomme une à une, sans les compter.
- **Écartée** : allonger encore le motif unique (même mode d'échec : une énumération de formes est ouverte).
- **Réversible** : oui.

### C-32-12-23 — Reconnaissance `CREATE TABLE` dans `post_restore.rs` ; `last_table_creating_migration` retirée

- **Contexte** : F6 — l'AC2 (ii) appelait une fonction privée d'un autre `mod tests` ; `split_statements` est privée.
- **Retenu** : `creates_application_table`, `normalize` et `split_statements` en `pub(crate)` dans `post_restore.rs` ;
  `backup::table_creating_migrations` les appelle ; l'AC2 (ii) compare au maximum de la carte ;
  `last_table_creating_migration` retirée (sans appelant après l'AC9 ; `clippy -D warnings`).
- **Écartées** : la reconnaissance dans `backup.rs` (deux modules s'appelleraient l'un l'autre pour un même analyseur) ;
  garder `last_table_creating_migration` en enveloppe du maximum (code de test sans appelant).
- **Réversible** : oui.
- *Annoté à la validation P3 (2026-10-24)* : `creates_application_table` ne passe pas telle quelle en production —
  elle devient `created_application_table -> Option<&'static str>` (C-32-12-31).

### C-32-12-24 — Le message du refus se prouve hors HTTP, à l'égalité exacte

- **Contexte** : F5 = R4 — sous M1, l'AC6 (i) restait vert ; et, relevé de la remédiation, la réponse HTTP d'un
  `InvalidBackupStructure` ne porte **pas** le motif (`errors.rs:2171-2180`), si bien que l'AC6 (i) assertait un texte
  absent de la réponse. Le message `HorsOrdre` disait « créée après elle », faux pour deux tables d'une même migration.
- **Retenu** : AC6 au code `INVALID_BACKUP_STRUCTURE` seul ; AC7 (b), `#[test]` sans base, message d'`Exigee` à
  l'égalité exacte sur la fixture privée de `products` avec les règles de production ; `HorsOrdre` : « créée par la même
  migration qu'elle ou après ».
- **Écartée** : exposer le motif dans la réponse HTTP (change le contrat d'erreur pour un test).
- **Réversible** : oui.

### C-32-12-25 — Les six tests lexicaux qui lisent `routes/admin.rs` sont nommés

- **Contexte** : F7 — « les deux » de la remédiation P1 en omettait quatre, dont `echecs_avant_sauvegarde_tous_convertis`,
  qu'un `?` ajouté par l'AC12 ferait rougir.
- **Retenu** : les six nommés à T4 et au tableau des sites ; l'AC12 interdit tout `?` neuf avant `write_pre_import_backup(`.
- **Réversible** : oui.

### C-32-12-26 — `version.rs:272-275` récrit au même commit

- **Contexte** : R5 — « une source d'une version mineure plus récente mais non-breaking reste importable » est faux
  avant comme après la story (table inconnue → `Inconnue`).
- **Retenu** : récrit (« ce contrôle-ci ne refuse pas pour cette seule raison ; l'inventaire et les colonnes peuvent
  refuser ailleurs ») ; `version.rs` ajouté aux fichiers touchés.
- **Écartée** : le justifier comme « vrai du seul contrôle de version » sans le récrire (le texte resterait trompeur).
- **Réversible** : oui.

### C-32-12-27 — `post_restore.rs:441` confié à la 32-5

- **Contexte** : F3 (b) — la justification de l'exemption `20260917000001` dit « EST la dernière créatrice de table
  applicative » : vrai aujourd'hui, faux dès la table de la 32-5.
- **Retenu** : laissée telle ; *Contrat* point 5 et AC10 demandent à la 32-5 d'écrire « EST le plancher d'import ».
- **Écartée** : la récrire dès maintenant (exacte aujourd'hui sous l'une et l'autre forme ; c'est la table de la 32-5
  qui la rend fausse, et son gate la relira par G4).
- **Réversible** : oui.

### C-32-12-28 — `TOLEREES_ECRITES` et `tolerees_calculees`, deux objets nommés

- **Contexte** : F4 = R6 — « tolérées » désignait la liste écrite et l'ensemble calculé ; sous la seconde lecture,
  l'AC2 (iii) était une tautologie.
- **Retenu** : deux noms dans le code et la fiche ; (iii) sur la liste écrite ; « tolérées » seul réservé à l'AC3.
- **Réversible** : oui.

### C-32-12-29 — Dérogation au découpage préventif (huit modules)

- **Contexte** : F9 — compté au grain de la règle, la story touche huit modules (sept au relevé de F, plus `version.rs`
  par C-32-12-26).
- **Retenu** : story gardée unique, section *Dérogation règle de splitting* écrite (deux sites lourds, six mécaniques ;
  pas de recyclage de défauts entre P1 et P2) ; à confirmer par Guy avec Q4.
- **Écartée** : story-zéro `kesh-db` + rollout (livrerait une règle sans appelant ; la fixture traverse les deux).
- **Réversible** : oui.
- *Annoté à la validation P3 (2026-10-24)* : « pas de recyclage de défauts entre P1 et P2 » était faux (R2 = F6 de la
  P3) ; motivation récrite, recyclage partiel reconnu (C-32-12-36).

---

*Choix 30 à 36 : remédiation de la validation P3 (2026-10-24, rapports `3212-validate-p3-{R,F}.md`).*

### C-32-12-30 — Les commandes de l'AC17 s'écrivent telles qu'exécutées, insensibles à la casse, portée élargie

- **Contexte** : R1 = F1 — G3, G4, G5 écrits sans `-i` rendaient 28 / 6 / 16 au lieu des 29 / 10 / 18 déclarés, et G3
  manquait `docs/migrations-idempotence-audit.md:91` (« REFERME la fenêtre »), membre de la classe ; F9 — la portée
  laissait `scripts/prepare-release.sh:128` hors balayage, et six commentaires « fenêtre refermée » échappaient à G3.
- **Retenu** : `-i` sur les cinq familles ; `scripts` et `.config` ajoutés à `S` ; `fenêtre refermée` ajouté à G3 ;
  exclusions écrites dans chaque commande ; nombres **rendus par les commandes exécutées telles qu'écrites** (51 / 7 /
  36 / 10 / 18 / 5) ; homonymes nommés un à un aux Dev Notes (14).
- **Écartée** : corriger les nombres à 28 / 6 / 16 sans `-i` (la commande continuerait de manquer `audit:91` et toute
  graphie capitalisée d'un site neuf).
- **Réversible** : oui.

### C-32-12-31 — `created_application_table` rend le nom ; `table_creators` garde toutes les créatrices

- **Contexte** : R4 = F3 — `creates_application_table -> bool` ne dit pas quelle table ; et une carte d'une version par
  table (la plus ancienne) rend « deux créatrices » indiscernable de « une » : le test AC1 (i) ne pouvait pas s'écrire.
- **Retenu** : `created_application_table(&str) -> Option<&'static str>` (l'entrée de l'inventaire) ;
  `table_creators() -> &'static BTreeMap<&'static str, Vec<i64>>` ; `table_creating_migrations()` en prend le minimum ;
  le test (i) lit `table_creators()`.
- **Écartée** : recalculer les créatrices dans le test seul (deux analyseurs du même SQL, l'un pouvant diverger de
  l'autre).
- **Réversible** : oui.

### C-32-12-32 — L'AC11 couvre toute écriture dans une table postérieure au plancher, et l'extrait est exigé

- **Contexte** : R7 — une migration ultérieure qui amorce ou corrige une table créée après le plancher par une story
  précédente échappait à l'AC11 et pouvait être exemptée « par couverture », à tort ; F4 — une migration qui crée et
  amorce ne s'inscrit pas au registre par un `include_str!` entier (`registry_sql_contains_no_ddl`).
- **Retenu** : prédicat sur la **cible** (table de l'inventaire dont la créatrice est postérieure au plancher), quelle
  que soit la migration qui l'a créée ; extrait sous `post_restore/` écrit à l'AC11 et au *Contrat* point 2 ; M7 nourrie
  des deux cas.
- **Écartée** : déclarer la sous-classe angle mort (vacante aujourd'hui, mais la 32-6 écrira peut-être dans une table
  de la 32-5).
- **Réversible** : oui.

### C-32-12-33 — AC2 (iii) gardée pour son message, déclarée impliquée par (iv)

- **Contexte** : R6 = F7 — `exigees` et `tolerees_calculees` partitionnent l'inventaire ; (iv) vraie entraîne (iii).
- **Retenu** : (iii) reste (elle nomme la table orpheline) ; la fiche dit qu'elle ne rougit jamais seule ; M3 compte
  une condition.
- **Écartée** : supprimer (iii) (message moins parlant pour la 32-5).
- **Réversible** : oui.

### C-32-12-34 — Ordre de `tablesAbsent`, table présente sans créatrice, durées de vie des erreurs

- **Contexte** : R9 = F8 — deux tables d'une même migration n'avaient pas d'ordre ; une table présente sans
  migration connue n'avait pas de règle ; `presente` venait du manifeste.
- **Retenu** : ordre par version puis rang dans l'inventaire ; une présente sans créatrice compte comme au plus tard au
  plancher (aucune contrainte de suffixe) ; `Inconnue` vérifiée d'abord, `presente` résolue dans l'inventaire
  (`&'static str`), `Inconnue(String)`. Cas testés à l'AC3.
- **Écartée** : ordre alphabétique (sans lien avec l'inventaire que lit l'Administrateur ailleurs).
- **Réversible** : oui.

### C-32-12-35 — Le manuel ne donne pas d'exemple tiré de la 32-5

- **Contexte** : F5 — l'AC20 (ii) citait « un serveur MCP désactivé et sans clé », alors que l'AC21 interdit de rien
  promettre de la 32-5 et que le manuel versionné arriverait dans `main` avant elle.
- **Retenu** : paragraphe sans exemple d'une story à venir ; l'exemple concret est ajouté par la story qui crée la
  première table postérieure (*Contrat* point 3).
- **Réversible** : oui.

### C-32-12-36 — Dérogation au découpage : le recyclage est reconnu, le non-découpage motivé autrement

- **Contexte** : R2 = F6 — la dérogation niait tout recyclage ; les rapports P2 datent R1 = F1 de la remédiation P1, R1
  de la P3 vient de la remédiation P2, et la classe « textes devenus faux » est revenue à chaque passe.
- **Retenu** : signal déclaré levé, recyclage partiel écrit ; non-découpage motivé par la circonscription du recyclage
  à une classe, par le fait qu'un découpage séparerait la règle des textes qu'elle rend faux, et par l'unité de la
  règle sur huit modules ; coupe de repli écrite (32-12a fixture, règle, import et textes ; 32-12b bilan, écran,
  manuel, CHANGELOG). Décision soumise à Guy avec Q4.
- **Écartée** : découper d'office (décision du Project Lead, D5 : le signal se déclare, il ne découpe pas seul).
- **Réversible** : oui.
- *Annotation du 2026-10-24 (validation P4, P4-1 et P4-2)* : le motif d'écart cite D5 sans sa condition — « Ce qui
  découpe, c'est le recyclage … et la dispersion dans plus de cinq modules », toutes deux constatées ici ; et
  l'argument « aucune passe n'a trouvé deux fois un défaut de la règle, de la carte… » est contredit par les rapports
  (AC2 (iii) en P1, P2, P3 ; carte de l'AC1 en P1, P3). Choix **remplacé** par C-32-12-37 (découpage) ; texte
  ci-dessus laissé tel qu'écrit à sa date.

### C-32-12-37 — La dérogation est retirée : découpage en 32-12a et 32-12b

- **Contexte** : validation P4 (Haiku, ciblée, 0/0/2/1) — P4-1 : C-32-12-36 invoquait D5 à contresens ; P4-2 :
  l'argument (a) de la dérogation est contredit par les rapports P1-P3. Les deux conditions de D5 sont constatées :
  recyclage partiel de la classe « textes devenus faux » et dispersion sur huit modules.
- **Retenu** (décision de l'orchestrateur, **soumise à Guy avec Q4**) : la coupe de repli de la fiche. **32-12a**
  (`32-12a-regle-inventaire-et-textes`) = AC1-AC7, AC9-AC11, AC15-AC17, AC18a, AC19a (M1-M4, M6-M8) — fixture, règle,
  import qui l'applique, fenêtre des tests P7, et tous les textes qu'elles rendent faux ; cinq modules (`kesh-db/backup`,
  `kesh-db/post_restore`, `kesh-db/repositories`, `kesh-db/version`, `kesh-api/admin_backup`), 🗄️. **32-12b**
  (`32-12b-bilan-ecran-manuel`) = AC8, AC12-AC14, AC20-AC21, AC18b, AC19b (M5, M9) ; trois modules
  (`kesh-api/routes/admin`, `kesh-i18n`, `frontend/admin-restore`). Ordre : 32-12a, puis 32-12b, les deux avant la
  32-5. Une seule fiche, deux clés ; numérotation des AC et des mutations conservée. `ParsedBackup::tables_absent` lu
  par le `Debug` manuel à la 32-12a (aucun lecteur de production avant l'AC12). CHANGELOG : une entrée, à la 32-12b.
- **Écartées** : garder la dérogation en la récrivant comme dérogation explicite à D5 (correction proposée par P4-1 —
  mais sans l'argument (a), réfuté par P4-2, il ne reste rien pour la motiver) ; une autre coupe — story-zéro
  `kesh-db` seule, puis rollout (sépare la règle de son appelant et des textes qu'elle rend faux, ce que la coupe
  retenue évite) ; deux fiches distinctes (les AC se citent d'une clé à l'autre, la traçabilité des validations P1-P4
  se perdrait).
- **Réserve** : la 32-12a est **au seuil** (cinq modules) au grain qui a donné huit pour la story entière ; compter
  `kesh-api/tests` comme module en donnerait six. Signalé à Guy.
- **Réversible** : oui — les deux clés se recousent sans perte si Guy en décide ainsi.

## Développement de la 32-12a (2026-10-24, Opus 5.5, worktree `kesh-32-12a`)

### C-32-12-38 — `InventoryRules::creation` est un pointeur de fonction

- **Contexte** : l'AC3 laisse la forme des règles injectées ouverte (« ou équivalent »).
- **Retenu** : `creation: fn(&str) -> Option<i64>` — la production passe une fermeture sans capture
  (`table_creating_migrations().get(t).copied()`), les tests des fonctions nommées (`creation1`, `creation2`,
  `creation_products_apres_tout`) ; `InventoryRules` reste `Copy`, sans durée de vie ni `dyn`.
- **Écartées** : `&dyn Fn` (durée de vie à porter par la structure), une `BTreeMap` injectée (oblige les tests à
  recopier toute la carte de production pour l'AC7).
- **Réversible** : oui, interne à la crate et à ses tests.

### C-32-12-39 — La `presente` d'un `HorsOrdre` est la plus récente des présentes

- **Contexte** : l'AC3 nomme l'erreur `HorsOrdre { absente, presente }` sans dire laquelle des présentes citer.
- **Retenu** : la plus ancienne des absentes est comparée à la **plus récente** des présentes postérieures au
  plancher (à version égale, la plus petite de rang dans l'inventaire) — c'est la seule comparaison nécessaire et
  suffisante pour la règle du suffixe, et la table citée est celle qui rend le fichier impossible.
- **Écartées** : la première présente postérieure à l'absente dans l'ordre de l'inventaire (même verdict, message
  moins parlant quand plusieurs tables sont présentes).
- **Réversible** : oui (message seul ; aucun lecteur de production du champ).

### C-32-12-40 — La détection de l'AC11 est une fonction de test nourrie de SQL littéral

- **Contexte** : M7 propose une porte d'injection du registre et du `MIGRATOR`, ou une fonction de détection testée
  sur un SQL littéral.
- **Retenu** : la seconde — `post_floor_write_targets(sql, inventaire, creation, plancher)` dans le `mod tests` de
  `post_restore.rs`, testée sur deux migrations fictives (créer-et-amorcer ; écrire plus tard dans une table créée
  avant) et sur trois non-cas (table système, table exigée, DDL `ON UPDATE CURRENT_TIMESTAMP`) ; le test réel
  `post_floor_table_seeds_are_replayed` l'applique au `MIGRATOR` (vacant aujourd'hui).
- **Écartées** : une porte de production pour injecter un `MIGRATOR` (code de production pour un test).
- **Réversible** : oui.

### C-32-12-41 — M8 a un tueur : un test unitaire de la reconnaissance

- **Contexte** : la fiche prévoit M8 (accents graves non retirés) équivalente sur le dépôt actuel, faute de
  `` CREATE TABLE `x` `` dans les migrations.
- **Retenu** : `created_application_table_reads_the_header_only` (`backup.rs`, `mod tests`) éprouve la graphie à
  accents graves sur un littéral ; M8 n'est donc plus équivalente — l'AC1 (i) reste, lui, vert sous M8, ce que le Dev
  Agent Record déclare.
- **Réversible** : oui.

### C-32-12-42 — Écart de protocole déclaré : une réécriture de code par script

- **Contexte** : le bloc du test `registry_entries_are_within_import_window` (doc-comment, corps, et retrait des deux
  fonctions de test voisines) a été remplacé par un script `python3` de remplacement de texte, contre
  `docs/agents-orchestration.md` § 7 (« jamais de script `python3` de remplacement sur un fichier source »).
- **Constat** : résultat relu aussitôt (`git diff`, lecture du bloc) — conforme, rien d'autre touché ; toutes les
  autres éditions de code ont été faites par l'outil d'édition. Déclaré au Dev Agent Record et au rapport.
- **Réversible** : sans objet (le contenu est celui qu'aurait produit l'outil d'édition).
