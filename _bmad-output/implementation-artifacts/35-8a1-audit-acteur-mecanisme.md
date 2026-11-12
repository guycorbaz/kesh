# Story 35-8a1 : l'audit attribue à la clé d'API ce qu'elle a fait — le type `Acteur`, la garde des sites non résolus, et le retour en brouillon d'une facture fournisseur (#431, #892 item 11)

Status: ready-for-dev

*Fiche tirée le 2026-11-13 de la fiche `35-8a-audit-acteur-patron.md` (validation P1 remédiée, même date), par l'agent
de rédaction des fiches de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, sur décision de l'orchestrateur (C-e350-50,
« découpé » : Guy veut de petites stories). **Aucune réécriture du fond** : les décisions, les inventaires et les
critères sont ceux de la 35-8a, répartis entre la **35-8a1** (le mécanisme et le pilote fournisseur, lot 1) et la
**35-8a2** (les écritures, lot 2). Release **0.17.7**. **Aucune migration** (colonnes `actor_type`, `actor_api_key_id`
depuis la migration `20260605000002`, Story 17-2a). **`kesh-db` touché : gate complet à chaque commit.** Code lu sur
**`main`** (`67efc51f`). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -38 à -40, -48 à -50) ;
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
  (`journal_entries.rs:1858`) et reçoit `acteur.api_key_id`. Elle ne dépend donc ni des 35-5a0 à 35-5c ni de la 35-3, et
  peut partir de `main` dès maintenant.
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
3. le `None` positionnel passé à `delete_in_tx` : `invoices.rs:1689`, `supplier_invoices.rs:1410`, `:1412` ;
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
  Option<i64> }` (`Copy`) dans `entities/audit_log.rs` ; `NewAuditLogEntry::par(acteur, action, entity_type, entity_id,
  details)` (délègue à `for_actor`) ; un seul constructeur sans clé, `Acteur::utilisateur(user_id)` ; `letterings::Actor`
  devient un alias (`pub type Actor = Acteur;`) ou est remplacé, sans changer ses appelants autrement que par le nom ;
  `impl From<&CurrentUser> for Acteur` dans `kesh-api/src/audit.rs`. Une fonction **convertie** prend `acteur: Acteur` à
  la place de `user_id: i64`.
- **D2 — La garde des sites non résolus** (C-e350-40, C-e350-49) — reprise **intégralement** de la 35-8a, D2 :
  `crates/kesh-api/tests/audit_acteur_garde.rs` ; items `#[cfg(test)]` retirés (jamais une coupe au premier attribut) ;
  quatre motifs sur le texte joint (`NewAuditLogEntry::user\s*\(`, `Acteur::utilisateur\s*\(`, `api_key_id:\s*None\b`,
  `for_actor\(\s*[^,()]+,\s*None\b`) ; table close `(fichier, fonction englobante, motif) → (nombre, raison)` ; raisons
  `PRE_AUTH`, `SYSTEME`, `RESTAURATION`, `DEFINITION`, `PONT`, `FIXTURE`, `MCP` (#1057), `ADMIN_SANS_CLE`, `A_CONVERTIR` —
  chaque site compté en a une, chaque raison finale cite l'appelant qui l'établit (le détail, ligne par ligne, est à la
  fiche 35-8a, D2, qui fait foi) ; sondes (a) à (f). À la fin de cette story, les sites des écritures (`journal_entries.rs`,
  dont le lettrage `reversal` `:3068`, et leurs appelants) restent sous `A_CONVERTIR` : la 35-8a2 les convertit.
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
2. **AC5 — La garde** : verte sur la branche ; ses six sondes (a) à (f) consignées au Change Log ; sa table recomptée
   depuis la source à l'étape finale, ventilation par raison et total cohérents écrits au Change Log.
   `audit_label_registry.rs` connaît `NewAuditLogEntry::par(` (action en position 1, entité en 2), avec une sonde.
3. **AC6a — Manuel et CHANGELOG** : `admin-manual.tex:3193` dit que le retour en brouillon d'une facture fournisseur est
   désormais attribué à la clé, que le reste ne l'est pas encore (renvoi à #431), et que **les entrées écrites avant la
   0.17.7 par une clé gardent la marque « utilisateur »**, sans correction possible ; `user-manual.tex:3959-3960` nuancé de
   même ; les deux PDF régénérés, contrôlés aplatis. CHANGELOG `## [0.17.7] — Non publié`, *Corrigé*, `refs #431`,
   `refs #892`, avec la phrase sur les entrées antérieures. (La 35-8a2 complète les mêmes paragraphes pour les écritures.)

## Tests à écrire, et la preuve du rouge

AC3 écrit d'abord, sur la base de la branche : rouge par l'`actor_type = 'user'` (`kesh-gate-logs/e358a1/rouges.log`).
La garde naît verte avec sa table — sa preuve est la sonde.

**Mutations** : M1 `From<&CurrentUser> for Acteur` qui jette la clé (AC3) ; M2 `par` qui appelle `::user` (AC3) ; M4
`unvalidate_in_tx` qui passe `None` pour la clé à `delete_in_tx` ou au lettrage `:1377` (AC3 — et la garde) ; M5 une
entrée de la table de la garde décrémentée (AC5) ; M6 `par(` retiré de `FORMES` (AC5). (M3 et M7 vont à la 35-8a2.)

## Tâches

- [ ] Étape 0 : relire les lignes citées sur la base réelle ; relancer l'inventaire (`kesh-gate-logs/e358a/inventaire.py
      <worktree>`), sortie au Change Log ; fonction englobante et raison de chaque site, appelant de chaque raison finale.
- [ ] Rouges d'AC3 ; `Acteur`, `par`, `From<&CurrentUser>`, alias `letterings::Actor` ; pilote ; garde et sa table ;
      `audit_label_registry.rs` ; manuels (deux `.tex`, deux PDF) ; CHANGELOG.
- [ ] Gates : backend complet à chaque commit (`kesh-db`) ; frontend : rien ne change ; E2E complet au dernier commit de
      code.

## Dev Notes

### Taille

Production ≈ 8 fichiers : `kesh-db` `entities/audit_log.rs`, `repositories/{letterings,supplier_invoices}.rs` ;
`kesh-api` `audit.rs`, `routes/supplier_invoices.rs`, `tests/audit_acteur_garde.rs` (neuf),
`tests/audit_label_registry.rs` ; manuels. Tests : 6 fichiers, 30 appels d'`unvalidate*`, plus le test d'AC3. Petite.

### Fichiers communs — règle d'union C-e350-8

`repositories/supplier_invoices.rs` (35-3 y touche les comptes désignés, autre fonction) ; `CHANGELOG.md` ;
`admin-manual.tex:3193` et `user-manual.tex:3959-3960` (la 35-8a2 y écrit après elle) ; `audit_label_registry.rs`
(`FORMES`) ; la table de la garde (recomptée après chaque fusion antérieure).

### Ce qui doit rester vrai

- `user_id` reste le créateur de la clé. Aucune entrée d'audit perdue ni doublée. Fixtures synthétiques seulement.

### Références

- Fiche `35-8a-audit-acteur-patron.md` (`split`), qui fait foi pour l'inventaire détaillé et la table des raisons ;
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
