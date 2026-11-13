# Story 35-8a1 : l'audit attribue à la clé d'API ce qu'elle a fait — le type `Acteur`, la garde des sites non résolus, et le retour en brouillon d'une facture fournisseur (#431, #892 item 11)

Status: ready-for-dev

*Fiche tirée le 2026-11-13 de la fiche `35-8a-audit-acteur-patron.md` (validation P1 remédiée, même date), par l'agent
de rédaction des fiches de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, sur décision de l'orchestrateur (C-e350-50,
« découpé » : Guy veut de petites stories). **Aucune réécriture du fond** : les décisions, les inventaires et les
critères sont ceux de la 35-8a, répartis entre la **35-8a1** (le mécanisme et le pilote fournisseur, lot 1) et la
**35-8a2** (les écritures, lot 2). Release **0.17.7**. **Aucune migration** (colonnes `actor_type`, `actor_api_key_id`
depuis la migration `20260605000002`, Story 17-2a). **`kesh-db` touché : gate complet à chaque commit.** Code lu sur
**`main`** (`67efc51f`). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -38 à -40, -48 à -50, -52) ;
`e35-choix-autonomes-e358a1.md` pour la story, à créer au premier choix. Validation P2 à lancer (la P1 a porté sur la
35-8a entière : `kesh-gate-logs/e35-8a-validation-p1.md`).*

**Issues** : la PR porte `refs #431` (#431 reste ouverte : la 35-8a2 puis le déploiement, reporté à l'Epic 36, C-e350-48)
et `refs #892` (item 11 ; #892 garde ses dix autres items). **Aucun mot-clé de fermeture.**

## Story

En tant que titulaire dont un agent écrit dans Kesh par une **clé d'API REST**, je veux que la piste de contrôle dise,
quand cet agent repasse une facture fournisseur en brouillon, que c'est **la clé** qui l'a fait — et non moi —, et que le
dépôt dispose d'un type et d'une garde qui rendent impossible d'oublier la clé ailleurs sans que rien ne rougisse (CO art.
957-964, OLICo art. 9).

## Ordre et dépendances

- **Ne touche pas `repositories/journal_entries.rs`** : `delete_in_tx` a déjà le paramètre `actor_api_key_id`
  (`journal_entries.rs:1858`) et reçoit `acteur.api_key_id`. Elle ne dépend d'aucune story **pour son contenu**.
- **Mais elle part après la 35-5a0 fusionnée** (validation P2, V2-1) : la 35-5a0 ajoute la date du jour à **la même
  signature** d'`unvalidate` (`supplier_invoices.rs:1074`, fiche 35-5a0 l. 34-38 : `commit_with_view` et « toutes les
  écritures qui se terminent par lui — `validate`, `unvalidate`, … »), au même appel de la route
  (`routes/supplier_invoices.rs:903-904`) et aux **30 mêmes appels de tests** dans les six fichiers cités ci-dessous : chaque
  appel gagne un argument chez l'une et change son `user_id` chez l'autre. La plomberie de la 35-5a0, « sans rien changer »,
  passe la première ; la 35-8a1 relit et recompte ces appels sur sa base (le compilateur en donne la liste close). Elle part
  aussi **après la 35-3** (`supplier_invoices.rs`, comptes désignés, autre fonction) et **après la 35-1** (`invoices.rs`,
  `credit_notes.rs`, que la garde compte ; `supplier_invoices.rs:1017`, numéro interne, autre hunk).
- **Sa garde compte des sites dans des fichiers que d'autres stories touchent** (`company_invoice_settings.rs` — 35-3 ;
  `invoices.rs`, `credit_notes.rs` — 35-1 ; `journal_entries.rs` — 35-5x) : la table se **recompte depuis la source**
  après toute fusion qui précède la sienne (règle d'union C-e350-8 ; une story fusionnée après elle qui ajoute un site
  le fait rougir — c'est le rôle de la garde).
- **La 35-8a2 part de la 35-8a1 fusionnée.**

## Ce qui se lit au code (`main`)

**Les constructeurs** — `crates/kesh-db/src/entities/audit_log.rs` : `NewAuditLogEntry::user` (`:173-191`, force
`actor_type = User`, `actor_api_key_id = None`) ; `for_actor(user_id, actor_api_key_id, …)` (`:195-215`) ; `api_key`
(`:219-237`). Le pont `crates/kesh-api/src/audit.rs` (`AuditActor::from_current_user`, `:20-58`) choisit l'un ou l'autre
d'après `CurrentUser.api_key_id`. Un couple voisin existe déjà : `letterings::Actor { user_id, api_key_id }`
(`repositories/letterings.rs:204-207`), dont l'en-tête dit l'écart de la contre-passation (`:201-202`).

**Le défaut** : un dépôt qui reçoit un `user_id: i64` nu ne peut pas savoir qu'une clé agit ; il écrit `::user`. Une clé
`read-write` atteint `comptable_routes` (`crates/kesh-api/src/lib.rs:524-1016`) ; `require_not_pat` n'est posé que sur
`admin_routes` (`:517-519`) ; les routes de clés (`routes/api_keys.rs:97`) et du journal d'audit (`ensure_not_pat`,
`lib.rs:998-1002`) refusent une clé dans le handler.

**L'inventaire des sites non résolus** — repris tel quel de la 35-8a (§ *Ce qui se lit au code*, quatre formes) : relevé
sur `main` `67efc51f`, items attribués `#[cfg(test)]` retirés (`auth/bootstrap.rs:46-48` déclare un module de test avant
sa production), `kesh-gate-logs/e358a/inventaire.py`, sortie `inventaire-main-67efc51f.txt` :
1. `NewAuditLogEntry::user(` : **80 sites** dans 30 fichiers (`kesh-db` 48, `kesh-api` 32 ; ventilation à la fiche 35-8a) ;
2. `api_key_id: None` : **11 sites** (neuf `letterings::Actor` de `kesh-db`, dont `supplier_invoices.rs:1377` — le
   lettrage dissous au retour en brouillon — et deux définitions : `middleware/auth.rs:179`, `entities/audit_log.rs:187`) ;
3. le `None` positionnel passé à une fonction qui attend la clé : à `delete_in_tx`, `invoices.rs:1689`,
   `supplier_invoices.rs:1410`, `:1412` ; **et ailleurs** (rejeu de la validation P2, V2-4, sur les 28 fonctions de `src`
   qui prennent `actor_api_key_id: Option<i64>`) : `reconciliation_cancel.rs:811` (`cancel` passe `user_id, None, None` à
   `cancel_in_tx`, enveloppe sans appelant de production) et `kesh-seed/src/pieges.rs:570` (`invoices::mark_emailed(…,
   None)`) — **au moins cinq** sites, dont deux hors `delete_in_tx`, ni l'un ni l'autre de cette story ;
4. `for_actor(<user_id>, None, …)` : 0 site.

**La restauration d'une sauvegarde** signe délibérément par `audit_uid` (`routes/admin.rs:534`, `:553-557`, `:593`,
`:654` ; `companies.rs:741-770`). **Cette story ne touche aucun de ces sites** ; sa garde leur donne une raison finale.

**Le pilote** — *retour en brouillon d'une facture fournisseur* : `supplier_invoices::unvalidate` (`:1074`) et
`unvalidate_in_tx` (`:1147` ; trace `supplier_invoice.unvalidated` `:1425`, lettrage dissous `:1365-1381` avec l'`Actor`
`:1377`, suppressions `:1410`, `:1412`) et sa route (`routes/supplier_invoices.rs:903-904`, `lib.rs:656-657`). Appelants :
la route, et 30 appels de tests dans 6 fichiers (`supplier_invoices_draft` 21, `supplier_invoices_internal_number` 5,
`supplier_invoices_repository`, `migrations_upgrade_path`, `admin_full_import_e2e`, `mcp_outils_e2e` 1 chacun).

**Les gardes et tests existants** : `crates/kesh-api/tests/audit_label_registry.rs` (`FORMES`, `:43-48`) ;
`crates/kesh-api/tests/api_keys_e2e.rs:438-505` (`pat_mutation_is_audited_as_api_key`, le patron d'un test par clé) ;
`tests/common/mod.rs:175` (`create_key_via_http`).

**Le manuel** : `docs/manual/fr/admin-manual.tex:3193` (« une quinzaine de dépôts … \#431 ») ; `:3392` (le serveur MCP,
hors story) ; `docs/manual/fr/user-manual.tex:3959-3960` (« clé API » « si elle a été faite par une clé d'API » — faux pour
la plupart des actions).

## Décisions

- **D1 — Le type `Acteur`** (C-e350-39) — repris de la 35-8a, D1 : `pub struct Acteur { pub user_id: i64, pub api_key_id:
  Option<i64> }`, dérivant **`Debug, Clone, Copy, PartialEq, Eq`** comme `letterings::Actor` (sinon les tests qui comparent
  des `Actor` cassent quand il devient un alias ; validation P2, V2-7), dans `entities/audit_log.rs` ;
  `NewAuditLogEntry::par(acteur, action, entity_type, entity_id, details)` (délègue à `for_actor`) ; un seul constructeur
  sans clé, `Acteur::utilisateur(user_id)` ; `letterings::Actor` devient un alias (`pub type Actor = Acteur;`) ou est
  remplacé, sans changer ses appelants autrement que par le nom ; `impl From<&CurrentUser> for Acteur` dans
  `kesh-api/src/audit.rs`, et `AuditActor::from_current_user` (`audit.rs:33-58`, qui refait le `match` de `for_actor`)
  devient `par(Acteur::from(user), …)` — le site `PONT` (`audit.rs:51`) disparaît de la garde. Une fonction **convertie**
  prend `acteur: Acteur` à la place de `user_id: i64`.
- **D2 — La garde des sites non résolus** (C-e350-40, C-e350-49, C-e350-52), `crates/kesh-api/tests/audit_acteur_garde.rs`
  (neuve). **Cette fiche fait foi** (validation P2, V2-3 ; elle se lit seule) :
  - **Ce qu'elle lit** : le code de production de `crates/*/src` (`kesh-seed/src` compris : `pieges.rs` est derrière la
    fonctionnalité `pieges`, non sous `#[cfg(test)]`), **items attribués `#[cfg(test)]` retirés** — jamais une coupe au
    premier attribut, qui cacherait la production de `auth/bootstrap.rs` (`:46-48`, puis `:195`, `:332`) ; commentaires
    retirés. **Pas de parseur neuf** (V2-2, DRY) : `strip_line_comments`, `source_assainie` et `accolades_hors_chaines`
    (`crates/kesh-api/tests/audit_label_registry.rs:186`, `:238`, `:298` ; `source_assainie` appelle la première), qui retirent
    déjà les items `#[cfg(test)]` par appariement d'accolades, sont **extraites** vers
    `crates/kesh-api/tests/common/source_assainie.rs` (`#![allow(dead_code)]`, comme `autour_appel.rs`), incluses par
    `#[path]` (patron de `tests/common/autour_appel.rs`) dans les deux gardes. La fonction englobante d'un site : le dernier
    `fn <nom>` ouvert avant lui, par balayage textuel à profondeur d'accolades suivie (pas `syn` : un site dans une macro y
    serait invisible).
  - **Ce qu'elle compte**, sur le texte joint (blancs et retours à la ligne tolérés) : **cinq motifs** —
    `NewAuditLogEntry::user\s*\(`, `Acteur::utilisateur\s*\(`, `api_key_id:\s*None\b` (`actor_api_key_id: None` compris),
    `for_actor\(\s*[^,()]+,\s*None\b`, et (V2-4) `delete_in_tx\s*\(` **dont un argument est `None`** — le `None`
    positionnel de la suppression d'une écriture, qui donne à `invoices.rs:1689` (`invoices::unvalidate`) une ligne
    `A_CONVERTIR` (motif 5) ; à la 35-8a2, `delete_in_tx` prend l'`acteur` : le motif 5 tombe à zéro et l'entrée passe sous
    `Acteur::utilisateur`, toujours `A_CONVERTIR`.
  - **Sa clé : le site, non le fichier.** Table close `(fichier, fonction englobante, motif) → (nombre, raison)`. Tout
    écart rougit en nommant fichier, fonction et motif — un site neuf, un site converti sans que la table suive, un site qui
    change de fonction (un compte par fichier resterait vert quand on ajoute un `::user` dans `lock_books` en convertissant
    `unlock_books`, `companies.rs:1002`, `:1079`).
  - **Les raisons** sont des constantes ; chaque site compté en a une, et chaque entrée cite l'appelant qui l'établit (route
    et ligne de `lib.rs`, ou appelant non HTTP) ; une fonction appelée aussi par une route comptable n'entre dans aucune
    raison finale :
    - `PRE_AUTH` — avant toute authentification : `routes/setup.rs`, les fonctions de `routes/auth.rs` atteintes sans session
      (connexion, vol de session détecté `:600`, demande `:1116` et fin `:1288` de réinitialisation du mot de passe) ;
    - `SYSTEME` — le démarrage, sans requête : `auth/bootstrap.rs:195`, `:332` ; `companies::repair_installation_in_tx`
      (`companies.rs:768`), appelé au démarrage (`bootstrap.rs:411`) et à la restauration (`admin.rs:553`), l'acteur
      **choisi** par l'appelant ;
    - `RESTAURATION` — `routes/admin.rs:593` (`books.restored`) et `:654` (`admin.full_import`) : `user_id = audit_uid`
      (`admin.rs:534`), l'administrateur du jeu restauré, délibérément ; route sous `require_not_pat` ;
    - `DEFINITION` — ce qui construit un acteur sans être une trace : `entities/audit_log.rs:187` (corps de `::user`), le
      corps d'`Acteur::utilisateur`, `middleware/auth.rs:179` (le `CurrentUser` d'une session JWT) ;
    - `FIXTURE` — `kesh-seed/src/pieges.rs` (`:511`) : pas de requête, pas de clé. **N'apparaît qu'à la 35-8a2** (V2-6) :
      à la fin de cette story, `pieges.rs:511` n'entre dans aucun motif ; la constante n'est pas déclarée ici ;
    - `MCP` — le serveur MCP (`services/serveur_mcp.rs`) : une clé MCP n'est pas une clé REST ; suivi à **#1057** ;
    - `ADMIN_SANS_CLE` — atteint seulement par `admin_routes` sous `require_not_pat` ou par un handler qui refuse une clé :
      `users.rs`, `api_keys.rs`, `mcp_*`, `companies.rs` des routes adresse/e-mail/coordonnées, `routes/auth.rs:694`
      (`PUT /api/v1/auth/password`, `lib.rs:1139`, handler qui refuse une clé, `auth.rs:635`), et les dépôts qu'ils seuls
      appellent : `backup_schedule`, `installation_settings`, `unlock_books`, `company_invoice_settings::update`,
      `company_dunning_settings::update`, `invoices::delete`, `supplier_invoices::delete_draft_in_tx` ;
    - `A_CONVERTIR` — tout le reste, au compte exact (dont les sites des écritures : `journal_entries.rs`, le lettrage
      `reversal` `:3068`, leurs appelants ; la 35-8a2 les convertit ; le reste au déploiement, Epic 36).
    (`PONT` disparaît, D1.)
  - **Sondes** au Change Log, chacune rouge attendue puis rétablie : (a) un `NewAuditLogEntry::user(` ajouté dans une
    fonction non listée ; (b) un site converti sans que la table suive ; (c) la forme coupée sur deux lignes
    (`NewAuditLogEntry::user\n(`) ; (d) un site ajouté dans un fichier dont un `#[cfg(test)] mod …;` précède la production
    (modèle `bootstrap.rs`) ; (e) l'échange : un site ajouté dans `lock_books`, un autre retiré de `unlock_books` ; (f) un
    `for_actor(user_id, None, …)` ; (g) un site sous `#[cfg(not(test))]` (modèle `config.rs:591`) **reste compté** ; (h) un
    `delete_in_tx(…, None, …)` ajouté dans une fonction non listée.
  - **Angles morts, écrits au doc-comment de la garde** : (1) le `None` positionnel passé à une fonction autre que
    `delete_in_tx` qui prend `actor_api_key_id: Option<i64>` (**27 fonctions** de `src`, 26 hors `delete_in_tx`, recompté en
    P3 : `git grep -nE 'actor_api_key_id:\s*Option<i64>'` rend 34 lignes, moins 6 champs de struct et un doc-comment ;
    `reconciliation_cancel.rs:811`, `pieges.rs:570` aujourd'hui) — à reprendre au déploiement (Epic 36) ; (2) un fichier de
    `src` **entièrement** sous `#[cfg(test)]` par un `mod x;` ou un `#[path]` de son parent (`pre_upgrade_backup/tests.rs`,
    `kesh-core/src/bank_imports/manual_tests.rs`, `kesh-api/src/qr_decode_synthetique.rs` — **trois** ;
    `kesh-db/src/test_fixtures.rs` n'en est pas, module public compilé en production, `kesh-db/src/lib.rs:19`) : lu comme de
    la production ; **aucun site aujourd'hui** (vérifié en P2), donc pas de faux rouge, mais un site futur y serait compté ;
    (3) **les limites héritées de `source_assainie`**, écrites à son doc-comment (`audit_label_registry.rs:225-232`) : un `//`
    dans une chaîne qui tronque la ligne, les commentaires de bloc non retirés, les chaînes brutes — elles deviennent celles
    de la garde, par renvoi.
- **D3 — Le pilote du retour en brouillon** (#892 item 11) : `unvalidate` et `unvalidate_in_tx` prennent `acteur: Acteur`
  à la place de `user_id` ; la route passe `Acteur::from(&current_user)` ; `supplier_invoice.unvalidated` par
  `NewAuditLogEntry::par` ; le lettrage dissous (`:1377`) reçoit l'`acteur` ; les deux `delete_in_tx` (`:1410`, `:1412`)
  reçoivent `acteur.user_id, acteur.api_key_id` — **la signature de `delete_in_tx` ne change pas ici** (35-8a2). Les tests
  qui appellent `unvalidate*` passent `Acteur::utilisateur(user_id)` (sites de test, hors garde).
- **D4 — Ce qui ne change pas** : le `user_id` d'une entrée par clé reste le créateur de la clé ; vocabulaire des actions,
  schéma, affichage du journal d'audit inchangés ; aucune route, aucune réponse d'API changée. **Le passé ne se corrige
  pas** : les entrées écrites avant la 0.17.7 par une clé gardent `actor_type = 'user'` (AC6).
- **D5 — Hors de la story** : les écritures (35-8a2) ; le serveur MCP (#1057) ; la restauration (raison `RESTAURATION`) ;
  le reste des dépôts (35-8b, Epic 36).

## Critères d'acceptation

1. **AC3 — #892 item 11** (numérotation de la 35-8a conservée) : une facture fournisseur, par clé `read-write` d'un
   Comptable et **par l'API seule** : brouillon → `validate` → `cancel` (`POST /api/v1/supplier-invoices/{id}/cancel`,
   `lib.rs:664`, qui pose les paires `reversal`) → `unvalidate` (`lib.rs:656`). **Obligatoire** :
   `supplier_invoice.unvalidated`, **les deux** `journal_entry.deleted` (contre-passation et achat) et le
   `lettering.removed` portent `actor_type = 'api_key'`, `actor_api_key_id` = la clé, `user_id` = son créateur. Plus le cas
   simple (validée → brouillon, un `journal_entry.deleted`), et le même geste par session → `'user'`. Les traces de
   l'annulation elle-même (`supplier_invoice.cancelled`, son `lettering.created`) ne sont pas de cette story : non
   assertées, dites au Change Log.
2. **AC5 — La garde** : verte sur la branche ; ses huit sondes (a) à (h) consignées au Change Log ; sa table recomptée
   depuis la source à l'étape finale, ventilation par raison et total cohérents écrits au Change Log.
   `audit_label_registry.rs` connaît `NewAuditLogEntry::par(` (action en position 1, entité en 2), avec une sonde.
3. **AC6a — Manuel et CHANGELOG** : `admin-manual.tex:3193` dit que le retour en brouillon d'une facture fournisseur est
   désormais attribué à la clé, et qu'**une partie des dépôts** (factures, avoirs, règlements, comptes, contacts, produits,
   lots de paiement, règles de rapprochement…) inscrivent encore `'user'` (renvoi à #431) — non « le reste » : la
   modification et la suppression d'une écriture (`:3194`) et une cinquantaine d'autres sites de production (`for_actor`, recompté en P3 : ≈ 50 appels, à recompter à
   l'étape 0 ; le manuel ne chiffre pas) attribuent déjà la clé
   (validation P2, V2-5) ; `docs/api-external.md:404` relu à l'étape 0, dit au Change Log ; et que **les entrées écrites avant la
   0.17.7 par une clé gardent la marque « utilisateur »**, sans correction possible ; `user-manual.tex:3959-3960` nuancé de
   même ; les deux PDF régénérés, contrôlés aplatis. CHANGELOG `## [0.17.7] — Non publié`, *Corrigé*, `refs #431`,
   `refs #892`, avec la phrase sur les entrées antérieures. (La 35-8a2 complète les mêmes paragraphes pour les écritures.)

## Tests à écrire, et la preuve du rouge

AC3 écrit d'abord, sur la base de la branche : rouge par l'`actor_type = 'user'` (`kesh-gate-logs/e358a1/rouges.log`).
La garde naît verte avec sa table — sa preuve est la sonde.

**Mutations** : M1 `From<&CurrentUser> for Acteur` qui jette la clé (AC3) ; M2 `par` qui appelle `::user` (AC3) ; M4
`unvalidate_in_tx` qui passe `None` pour la clé à `delete_in_tx` (AC3, et la garde par son cinquième motif) ou au lettrage
`:1377` (AC3, et la garde par `api_key_id: None`) ; M5 une
entrée de la table de la garde décrémentée (AC5) ; M6 `par(` retiré de `FORMES` (AC5). (M3 et M7 vont à la 35-8a2.)

## Tâches

- [ ] Étape 0 : relire les lignes citées sur la base réelle ; relancer l'inventaire (`kesh-gate-logs/e358a/inventaire.py
      <worktree>`), sortie au Change Log ; fonction englobante et raison de chaque site, appelant de chaque raison finale.
- [ ] Rouges d'AC3 ; `Acteur`, `par`, `From<&CurrentUser>`, alias `letterings::Actor` ; pilote ; extraction de
      `strip_line_comments`, `source_assainie` et `accolades_hors_chaines` vers `tests/common/source_assainie.rs` ; garde et sa table ; `audit_label_registry.rs` ; manuels
      (deux `.tex`, deux PDF) ; CHANGELOG.
- [ ] Gates : backend complet à chaque commit (`kesh-db`) ; frontend : rien ne change ; E2E complet au dernier commit de
      code.

## Dev Notes

### Taille

Production ≈ 8 fichiers : `kesh-db` `entities/audit_log.rs`, `repositories/{letterings,supplier_invoices}.rs` ;
`kesh-api` `audit.rs`, `routes/supplier_invoices.rs`, `tests/audit_acteur_garde.rs` (neuf),
`tests/audit_label_registry.rs` ; manuels. Tests : 6 fichiers, 30 appels d'`unvalidate*`, plus le test d'AC3. Petite.

### Fichiers communs — règle d'union C-e350-8

`repositories/supplier_invoices.rs`, `routes/supplier_invoices.rs` et les six fichiers de tests des appels
d'`unvalidate` avec la **35-5a0** (même signature, mêmes appels : la 35-8a1 part après elle) ; `supplier_invoices.rs` aussi
avec la **35-3** (comptes désignés, autre fonction) et la **35-1** (`:1017`, autre hunk) ; `invoices.rs`, `credit_notes.rs`
avec la 35-1 (sites que la garde compte) ; `CHANGELOG.md` ;
`admin-manual.tex:3193` et `user-manual.tex:3959-3960` (la 35-8a2 y écrit après elle) ; `audit_label_registry.rs`
(`FORMES`) ; la table de la garde (recomptée après chaque fusion antérieure).

### Ce qui doit rester vrai

- `user_id` reste le créateur de la clé. Aucune entrée d'audit perdue ni doublée. Fixtures synthétiques seulement.

### Références

- Fiche `35-8a-audit-acteur-patron.md` (`split`) : origine ; **en cas d'écart, cette fiche fait foi** (validation P2, V2-3) ;
  `kesh-gate-logs/e35-8a-validation-p1.md`.
- Issues #431, #892 (item 11), #1057 ; Story 17-2a ; Story 25-1b ; `CLAUDE.md` § *Inventorier les sites NON RÉSOLUS*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-13 — Fiche tirée de la 35-8a (lot 1), sur décision de l'orchestrateur (C-e350-50, « découpé »), sans réécriture
  du fond. La validation P1 de la 35-8a (1 HIGH, 7 MEDIUM, 2 LOW) est remédiée dans la fiche d'origine ; P2 à lancer sur
  celle-ci.
- 2026-11-13 — **Validation P2** (Sonnet) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 4 LOW (`kesh-gate-logs/e35-8a1-validation-p2.md`).
  Remédiée par l'agent de rédaction des fiches (Opus 5.5). Prémisses relues sur `main` (`67efc51f`) :
  `audit_label_registry.rs:238` (`source_assainie`), `:298` (`accolades_hors_chaines`) ; `tests/common/autour_appel.rs` ;
  `kesh-api/Cargo.toml:117` (`syn`) ; `reconciliation_cancel.rs:811` ; `kesh-seed/src/pieges.rs:570` ; `config.rs:591-592` ;
  fiche 35-5a0 (branche `chore/epic-35-fiches-355`, l. 34-38 : `unvalidate` reçoit la date). Confirmées.
  - V2-1 (MEDIUM) : la 35-8a1 part **après la 35-5a0** (même signature, mêmes 30 appels de tests), la 35-3 et la 35-1 ;
    *Ordre*, *Fichiers communs*, `sprint-status.yaml`, ligne du fichier d'epic et annotation de C-e350-50 corrigés.
  - V2-2 (MEDIUM) : pas de parseur neuf — `source_assainie` extraite vers `tests/common/` et partagée ; fonction englobante
    par balayage textuel ; sonde (g) `#[cfg(not(test))]` ; fichiers entièrement sous `cfg(test)` écrits en angle mort.
  - V2-3 (MEDIUM) : sondes et raisons **recopiées** dans D2 ; cette fiche fait foi ; en tête de la 35-8a, la même phrase.
  - V2-4 (MEDIUM) : cinquième motif `delete_in_tx(…, None, …)` (C-e350-52), sonde (h), M4 précisée ; inventaire complété
    (`reconciliation_cancel.rs:811`, `pieges.rs:570`) et angle mort des 28 fonctions écrit.
  - LOW : V2-5 (AC6a : « une partie des dépôts », non « le reste ») ; V2-6 (`FIXTURE` à la 35-8a2 seulement) ; V2-7
    (dérivations d'`Acteur`, `from_current_user` par `par`, `PONT` disparaît) ; V2-8 (numérotation de la 35-8a conservée :
    dit en tête d'AC3 déjà, rien de plus).
  - Symptômes greppés (fiches 35-8a, 35-8a1, 35-8a2, epic, registre) : « peut partir de `main` », « ligne par ligne »,
    « six sondes », « le reste ne l'est pas » — remplacés ou annotés.
- 2026-11-13 — **Validation P3, passe ciblée** (Opus) sur la remédiation P2 : 0 finding au-dessus de LOW, 5 LOW
  (`kesh-gate-logs/e35-8a1-validation-p3-ciblee.md`). Appliqués : V3-1 (`strip_line_comments` extraite avec les deux autres ;
  limites héritées de `source_assainie` en angle mort (3)) ; V3-2 (`test_fixtures.rs` retiré de l'angle mort (2) : module
  public, compilé en production) ; V3-3 (27 fonctions, 26 hors `delete_in_tx` ; « une cinquantaine » de sites `for_actor`) ;
  V3-4 (`invoices.rs:1689` : motif 5 ici, `Acteur::utilisateur` à la 35-8a2, toujours `A_CONVERTIR` ; dit aussi dans la
  35-8a2) ; V3-5 (renvoi d'autorité de la 35-8a2 aligné ; C-e350-52 à la ligne du fichier d'epic). Rédaction seule.
- 2026-11-13 — **Validation terminée.** Série des findings au-dessus de LOW : P1 (sur la 35-8a entière) 1 HIGH + 7 MEDIUM →
  découpage en 35-8a1 et 35-8a2 (C-e350-50) → P2 4 MEDIUM → P3 0 (CRITICAL : 0). Modèles : Opus (P1), Sonnet (P2), Opus en
  passe ciblée (P3). La boucle se clôt sur une passe sans finding au-dessus de LOW ; la remédiation de P3 ne touche que la
  rédaction. Choix retenus : **C-e350-38** (découpage de la 35-8), **-39** (`Acteur`), **-40** (garde par raison), **-48**
  (35-8b à l'Epic 36), **-49** (garde par site), **-50** (découpage de la 35-8a), **-52** (cinquième motif, angles morts).
  Statut : `ready-for-dev`.
