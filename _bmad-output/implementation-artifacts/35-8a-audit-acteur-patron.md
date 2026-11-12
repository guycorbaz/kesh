# Story 35-8a : l'audit attribue à la clé d'API ce qu'elle a fait — le type `Acteur`, la garde des sites non résolus, les écritures et le retour en brouillon d'une facture fournisseur (#431, #892 item 11)

Status: split

> **Découpée le 2026-11-13** (C-e350-50, décision de l'orchestrateur : petites stories) en
> **`35-8a1-audit-acteur-mecanisme.md`** (lot 1 : `Acteur`, `par`, la garde, le retour en brouillon d'une facture
> fournisseur ; AC3, AC5) et **`35-8a2-audit-acteur-ecritures.md`** (lot 2 : les écritures ; AC1, AC2, AC4). Cette fiche
> est gardée telle quelle ; elle fait foi pour l'inventaire détaillé, la table des raisons et le rayon, que les deux
> fiches reprennent sans en réécrire le fond.

*Fiche rédigée le 2026-11-12 par l'agent de rédaction des fiches de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le
workflow `bmad-create-story`. Née du **découpage préventif** de la 35-8 (C-e350-38) : la story touchait seize dépôts de
`kesh-db` et autant de routes de `kesh-api` — plus de cinq modules (`CLAUDE.md`, § *Règle de splitting préventif*). La
**35-8a** est le *story-zero* : elle pose le motif (le type `Acteur`, le constructeur d'audit, le pont depuis
`CurrentUser`, la garde qui inventorie les sites non résolus) et le convertit sur deux pilotes ; la **35-8b** fait le
déploiement. Release **0.17.7**. **Aucune migration** : les colonnes `actor_type` et `actor_api_key_id` existent depuis la
Story 17-2a (migration `20260605000002`). Boucle complète, **`kesh-db` touché : gate complet à chaque commit**. Code lu sur
**`main`** (`4d39f377`, la 0.17.6 publiée). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -38 à -41) ;
`e35-choix-autonomes-e358a.md` pour la story, à créer au premier choix. Validation P1 remédiée le 2026-11-13 (Change
Log) ; C-e350-48 à -50. **La 35-8b sort de l'Epic 35** (C-e350-48) : #431 reste ouverte à la fin de cette story, dont la
PR porte `refs #431` et `refs #892`.*

## Story

En tant que titulaire dont un agent écrit dans Kesh par une **clé d'API REST**, je veux que la piste de contrôle dise,
pour chaque écriture que cet agent passe, contre-passe ou supprime, que c'est **la clé** qui l'a fait — et non moi —,
afin que le journal d'audit réponde juste à la question qu'il existe pour poser : qui a fait quoi (CO art. 957-964,
OLICo art. 9).

## Ordre et dépendances

- **Part de `main` après la fusion de la 35-5a0, de la 35-5a, de la 35-5b, de la 35-5c** (`repositories/journal_entries.rs`,
  signatures et tests : la plomberie de l'horloge y fait passer la date du jour en paramètre, les tests y sont convertis ;
  fiches sur la branche `chore/epic-35-fiches-355`, `35-5a0-horloge-plomberie.md`, `35-5c-date-du-jour-tests.md:185` qui
  nomme déjà la 35-8) **et de la 35-3** (`repositories/company_invoice_settings.rs` ; touché par la 35-8b seulement, mais
  l'ordre de l'epic les met avant la 35-8, `epic-35-corrections-par-impact.md`, § *Ordre et dépendances*, point 3).
- **La 35-8b (reportée à l'Epic 36) part de la 35-8a fusionnée.** Les numéros de ligne ci-dessous sont ceux de `main` ; l'étape 0 les relit sur la
  base réelle de la branche.

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

**L'inventaire des sites non résolus** (relevé sur `main` `67efc51f`, code de production de `crates/*/src` : **les items
attribués `#[cfg(test)]` retirés** — accolades appariées pour un `mod … { }`, jusqu'au `;` pour un `mod …;` —,
commentaires `//` retirés). ⚠️ Couper le fichier au premier `#[cfg(test)]` est faux : `auth/bootstrap.rs:46-48` déclare
un module de test (`#[path = …] mod installations_atteintes;`) **avant** son code de production, dont deux sites (`:195`,
`:332`) ; même disposition, sans site aujourd'hui, à `auth/jwt.rs:83`, `kesh-db/src/post_restore.rs:820`,
`routes/reconciliation.rs:6766`. Quatre formes :

1. **`NewAuditLogEntry::user(`** : **80 sites** dans 30 fichiers — `kesh-db` 48 (accounts 4, backup_schedule 1,
   bank_profiles 3, companies 3, company_dunning_settings 1, company_invoice_settings 1, contacts 3, credit_notes 2,
   fiscal_years 1, installation_settings 1, invoice_settlements_write 3, invoices 6, journal_entries 2, payment_batches 3,
   products 3, reconciliation_rules 3, supplier_invoices 8) ; `kesh-api` 32 (audit.rs 1 — le pont lui-même —,
   auth/bootstrap 2, admin 2, api_keys 2, auth 4, companies 3, mcp_envois 2, mcp_keys 3, mcp_prompts 5,
   mcp_server_settings 1, setup 1, users 4, serveur_mcp 2). Faux positif écarté : `services/mod.rs:56` (commentaire `//!`).
   Relevé par un script qui retire les items `#[cfg(test)]` (`kesh-gate-logs/e358a/inventaire.py`, sortie
   `inventaire-main-67efc51f.txt` ; motif tolérant les blancs, `NewAuditLogEntry::user\s*\(`). La commande de la première
   rédaction (coupe au premier `#[cfg(test)]`) rendait 78 : retirée.
2. **`api_key_id: None`** (sous-chaîne : `actor_api_key_id: None` compris) : **11 sites** de production. Neuf
   `letterings::Actor` de `kesh-db` construits sans la clé — `credit_notes.rs:797`, `invoice_settlements_write.rs:424`,
   `:772`, `:1017`, `journal_entries.rs:3068` (la contre-passation, lettrage `reversal`), `supplier_invoices.rs:1377`,
   `:1942`, `:2594`, `:2863` — et deux **définitions**, qui ne sont pas des traces : `middleware/auth.rs:179` (le
   `CurrentUser` d'une session JWT) et `entities/audit_log.rs:187` (le constructeur `::user`). `helpers.rs:116` est sous
   `#[cfg(test)]` (`helpers.rs:105`) : hors inventaire.
3. **Un `None` positionnel passé pour la clé** à une fonction qui l'attend : `journal_entries::delete_in_tx(…, user_id, None,
   …)` appelé par `invoices.rs:1689` (dévalidation d'une facture client), `supplier_invoices.rs:1410` et `:1412` (retour en
   brouillon d'une facture fournisseur) — **#892 item 11** : `journal_entry.deleted` (`journal_entries.rs:1992`) et
   `lettering.removed` (`letterings.rs:1428`, par l'`Actor` de `supplier_invoices.rs:1377`) sortent sans la clé. Cette forme
   ne se grep pas ; elle disparaît quand la fonction prend un `Acteur` (D1).
4. **`for_actor(<user_id>, None, …)`** : `for_actor` reste public (`entities/audit_log.rs:195-215`) et rend `::user` sur
   `None` ; **0 site** aujourd'hui (`git grep -n -A2 "for_actor(" -- 'crates/*/src/*'`, deuxième argument relu), mais
   c'est une sortie muette de la conversion — remplacer `::user(user_id, …)` par `for_actor(user_id, None, …)` décrémente
   le motif 1 et laisse la trace fausse : la garde le compte (D2).

**La restauration d'une sauvegarde** signe délibérément par un autre utilisateur que l'appelant : `routes/admin.rs:534`
(`audit_uid`, le plus petit administrateur **du jeu restauré**), `:593` (`books.restored`), `:654` (`admin.full_import`),
et `:553-557`, qui passe `RepairTrigger::Restore { actor_user_id: audit_uid, … }` à
`companies::repair_installation_in_tx`, dont la trace `installation.repaired` (`companies.rs:766-770`) signe aussi au
démarrage (`RepairTrigger::Startup`, appelant `auth/bootstrap.rs:411`, acteur choisi `:741-748`). Route sous
`require_not_pat`. **La 35-8a ne touche aucun de ces sites** (ni `admin.rs`, ni `repair_installation_in_tx` ne sont dans
ses pilotes) ; sa garde leur donne une raison finale (D2 : `RESTAURATION`, `SYSTEME`), ce qui interdit à la story qui
déploiera le motif d'y mettre l'acteur de la requête.

**Les deux pilotes** :
- *Écritures* — `journal_entries::create` (`:195`), `create_in_tx` (`:242`), `create_in_tx_inner` (`:281`, la trace
  `journal_entry.created` `:562`) ; `reverse` (`:3097`), `reverse_in_tx` (`:2558`), `reverse_owned_in_tx` (`:2628`),
  `reverse_at_next_fiscal_year_start[_in_tx]` (`:3122`, `:2677`), `reverse_in_tx_inner` (`:2796` ; trace
  `journal_entry.reversed` `:3017`, lettrage `:3068`) ; `delete_in_tx` (`:1858`, déjà `actor_api_key_id`) et `delete_by_id`
  (`:1756`), `update` (`:1444`, déjà). Routes : `routes/journal_entries.rs` (création, contre-passation, modification,
  suppression).
- *Retour en brouillon d'une facture fournisseur* — `supplier_invoices::unvalidate` et `unvalidate_in_tx` (trace
  `supplier_invoice.unvalidated` `:1425`, lettrage `:1377`, suppressions `:1410`, `:1412`) et sa route
  (`routes/supplier_invoices.rs:903-904`, `lib.rs:657`).

**Les gardes et tests existants** : `crates/kesh-api/tests/audit_label_registry.rs` (`FORMES`, `:43-48`, la position de
l'action à chaque constructeur — tout constructeur neuf s'y inscrit, sinon son action échappe au registre) ;
`crates/kesh-api/tests/api_keys_e2e.rs:438-505` (`pat_mutation_is_audited_as_api_key`, le patron d'un test par clé) ;
`tests/common/mod.rs:175` (`create_key_via_http`).

**Le manuel** : `docs/manual/fr/admin-manual.tex:3193` (« L'attribution n'est pas encore complète … une quinzaine de dépôts
… Correction suivie à l'issue \#431 ») ; `:3392` (le serveur MCP : « au nom du créateur de la clé, sans la marque « clé
API » » — hors de cette story, D5).

## Décisions

- **D1 — Le type `Acteur`** (C-e350-39), dans `kesh-db` (`entities/audit_log.rs`) : `pub struct Acteur { pub user_id: i64,
  pub api_key_id: Option<i64> }` (`Copy`), avec `NewAuditLogEntry::par(acteur, action, entity_type, entity_id, details)`
  (délègue à `for_actor`) et **un seul** constructeur sans clé, nommé pour être vu : `Acteur::utilisateur(user_id)`.
  `letterings::Actor` devient un alias (`pub type Actor = Acteur;`) ou est remplacé, au choix du développement, sans
  changer ses appelants autrement que par le nom. Côté `kesh-api`, `impl From<&CurrentUser> for Acteur` (dans `audit.rs`,
  à côté du pont existant). Une fonction de dépôt **convertie** prend `acteur: Acteur` **à la place** de `user_id: i64` (et
  de `actor_api_key_id` s'il existait) : un `None` positionnel devient impossible.
- **D2 — La garde des sites non résolus** (C-e350-40, C-e350-49), `crates/kesh-api/tests/audit_acteur_garde.rs` (neuve).
  - **Ce qu'elle lit** : le code de production de `crates/*/src`, **items attribués `#[cfg(test)]` retirés** (un
    `#[cfg(test)]` suivi d'un `mod … { }` : accolades appariées, chaînes et commentaires sautés ; suivi d'un `mod …;` ou de
    tout autre item : jusqu'à la fin de l'item) — **jamais** une coupe à partir du premier attribut, qui cache la production
    de `auth/bootstrap.rs` (`:46-48`, puis `:195`, `:332`). `syn` (déjà au graphe ? à vérifier à l'étape 0 ; sinon un
    parseur d'accolades local) au choix du développement ; commentaires `//` et `/* */` retirés. `kesh-seed/src` en fait
    partie (`pieges.rs` est derrière la fonctionnalité `pieges`, non sous `#[cfg(test)]`).
  - **Ce qu'elle compte**, sur le texte joint (blancs et retours à la ligne tolérés) : quatre motifs —
    `NewAuditLogEntry::user\s*\(`, `Acteur::utilisateur\s*\(`, `api_key_id:\s*None\b` (`actor_api_key_id: None` compris) et
    `for_actor\(\s*[^,()]+,\s*None\b` (V1-2 : la sortie muette de `for_actor`).
  - **Sa clé : le site, non le fichier** (V1-4). La table close est `(fichier, fonction englobante, motif) → (nombre,
    raison)` ; la fonction englobante est le dernier `fn <nom>` ouvert avant le site, au même parseur. Un compte par fichier
    reste vert quand on ajoute un `::user` dans `lock_books` en convertissant `unlock_books` (`companies.rs:1002`,
    `:1079`) : c'est le faux vert que § *Inventorier les sites NON RÉSOLUS* interdit. Tout écart rougit en nommant fichier,
    fonction et motif — un site neuf, un site converti sans que la table suive, un site qui change de fonction.
  - **Les raisons** sont des constantes ; **chaque site compté en a une**, et chaque entrée cite l'appelant qui l'établit
    (route et ligne de `lib.rs`, ou appelant non HTTP) ; une fonction appelée aussi par une route comptable n'entre dans
    aucune raison finale :
    - `PRE_AUTH` — avant toute authentification : `routes/setup.rs`, les fonctions de `routes/auth.rs` atteintes sans
      session (connexion, vol de session détecté `:600`, demande `:1116` et fin `:1288` de réinitialisation du mot de passe
      — chacune relue à l'étape 0) ;
    - `SYSTEME` — le démarrage, sans requête : `auth/bootstrap.rs:195` (création de l'administrateur), `:332`
      (réinitialisation de l'administrateur) ; `companies::repair_installation_in_tx` (`companies.rs:768`), dont le premier
      appelant est le démarrage (`bootstrap.rs:411`, `RepairTrigger::Startup`) et le second la restauration
      (`admin.rs:553`, `RepairTrigger::Restore`) — dans les deux cas l'acteur est **choisi** par l'appelant, jamais celui
      de la requête ;
    - `RESTAURATION` — `routes/admin.rs:593` (`books.restored`) et `:654` (`admin.full_import`) : `user_id = audit_uid`
      (`admin.rs:534`), l'administrateur **du jeu restauré**, délibérément (migration `20260910000001`) ; route sous
      `require_not_pat` ; à garder tel quel par le déploiement ;
    - `DEFINITION` — ce qui construit un acteur sans être une trace : `entities/audit_log.rs:187` (corps de `::user`), le
      corps d'`Acteur::utilisateur`, `middleware/auth.rs:179` (le `CurrentUser` d'une session JWT) ;
    - `PONT` — `audit.rs` (la branche `None` de `from_current_user`) ;
    - `FIXTURE` — `kesh-seed/src/pieges.rs` (`:511`, `journal_entries::create` pour les pièges de démonstration) : pas de
      requête, pas de clé ;
    - `MCP` — le serveur MCP (`services/serveur_mcp.rs`) : une clé MCP n'est pas une clé REST ; limite écrite au manuel
      (`admin-manual.tex:3392`), suivie à **#1057**, hors de #431 (D5) ;
    - `ADMIN_SANS_CLE` — atteint seulement par `admin_routes` sous `require_not_pat` ou par un handler qui refuse une clé :
      `users.rs`, `api_keys.rs`, `mcp_*`, `companies.rs` des routes adresse/e-mail/coordonnées, `routes/auth.rs:694`
      (`change_password_with`, atteint par `PUT /api/v1/auth/password`, `lib.rs:1139`, route authentifiée dont le handler
      refuse une clé, `auth.rs:635` — V1-10 : il n'est pas pré-authentification), et les dépôts qu'ils seuls appellent :
      `backup_schedule`, `installation_settings`, `unlock_books`, `company_invoice_settings::update`,
      `company_dunning_settings::update`, `invoices::delete`, `supplier_invoices::delete_draft_in_tx` ;
    - `A_CONVERTIR` — tout le reste, au compte exact : la story de déploiement (35-8b, reportée à l'Epic 36, C-e350-48) vide
      cette raison et, selon sa décision, `ADMIN_SANS_CLE`.
  - **Sondes** au Change Log, chacune rouge attendue puis rétablie : (a) un `NewAuditLogEntry::user(` ajouté dans une
    fonction non listée ; (b) un site converti sans que la table suive ; (c) la forme coupée sur deux lignes
    (`NewAuditLogEntry::user\n(`) ; (d) un site ajouté dans un fichier dont un `#[cfg(test)] mod …;` précède la production
    (sur le modèle de `bootstrap.rs`) ; (e) **l'échange** : un site ajouté dans `lock_books` et un autre retiré de
    `unlock_books` dans le même état ; (f) un `for_actor(user_id, None, …)`.
- **D3 — Les pilotes**, convertis en entier :
  - **écritures** : `create`, `create_in_tx`, `create_in_tx_inner` → `acteur: Acteur` ; `journal_entry.created` par
    `NewAuditLogEntry::par` ; `reverse*` → `acteur` ; `journal_entry.reversed` et le lettrage `reversal` (`:3068`) portent la
    clé (l'écart R3 de `letterings.rs:201-202` est levé) ; `delete_in_tx`, `delete_by_id`, `update` passent de `(user_id,
    actor_api_key_id)` à `acteur`. Les routes de `routes/journal_entries.rs` passent `Acteur::from(&current_user)`. Les
    **autres appelants** de ces fonctions (facturation, règlements, soldes d'ouverture, réconciliation, factures
    fournisseur…) passent `Acteur::utilisateur(user_id)` là où ils n'ont qu'un `user_id` : chacun est un site de la garde
    (`A_CONVERTIR`), au compte exact. **Ceux qui ont déjà la clé en main la passent** (V1-5) — les cinq
    `journal_entries::create_in_tx(tx, fiscal_year.id, user_id, new_je, false)` de **`kesh-api/src/routes/reconciliation.rs`** :
    `:2820` (`accept_one_invoice`, paramètre `actor_api_key_id` `:2377`), `:3812` (`accept_one_split`, `:3523`), `:4260`
    (`accept_one_rule`, `:3951`), `:5130` (`post_manual_once`, `current_user` `:5039`), `:5596` (`post_split_once`,
    `current_user` `:5499`) — passent `Acteur { user_id, api_key_id: actor_api_key_id }` ou `Acteur::from(current_user)` ;
    `reconciliation_cancel.rs:723` (`reverse_in_tx`) passe l'acteur qu'il a. Un rapprochement par clé écrit aujourd'hui
    `reconciliation.accepted` à la clé et `journal_entry.created` à l'utilisateur : c'est #431 même, réparé ici. (La
    première rédaction citait `reconciliation_link` et `invoices.rs:2655`, `:2758`, `:2828` — `mark_emailed`, `freeze_pdf`,
    `refreeze_pdf`, qui n'appellent aucune de ces fonctions : retirés.)
  - **retour en brouillon d'une facture fournisseur** (#892 item 11) : `unvalidate_in_tx` et sa route prennent l'`Acteur` ;
    `supplier_invoice.unvalidated`, les deux `journal_entry.deleted` et le `lettering.removed` portent la clé.
    `delete_in_tx` a déjà le paramètre `actor_api_key_id` (`journal_entries.rs:1858`) : ce pilote ne dépend pas de la
    conversion des signatures des écritures (Dev Notes, *Taille et découpage proposé*).
- **D4 — Ce qui ne change pas** : le `user_id` d'une entrée par clé reste le **créateur de la clé** (imputabilité, Story
  17-2a) ; le vocabulaire des actions ; le schéma ; l'affichage du journal d'audit (qui montre déjà `actor_type`). Aucune
  route nouvelle, aucune réponse d'API changée. **Le passé ne se corrige pas** (V1-8) : les entrées écrites avant la 0.17.7
  par une clé — dont celles de l'agent du titulaire dans ses livres réels — gardent `actor_type = 'user'` ; la clé n'ayant
  pas été enregistrée, rien ne permet de les reprendre. Le manuel et le CHANGELOG le disent (AC6).
- **D5 — Hors de la story** : le serveur MCP (raison `MCP` de la garde ; limite du manuel `admin-manual.tex:3392`, suivie à
  **#1057**) ; la restauration (raison `RESTAURATION`, inchangée) ; le reste des dépôts (35-8b, reportée à l'Epic 36,
  C-e350-48).

## Critères d'acceptation

1. **AC1 — Une écriture passée par clé** (`crates/kesh-api/tests/`, patron de `api_keys_e2e.rs:438-505`) : clé
   `read-write` d'un Comptable, `POST /api/v1/journal-entries` → la ligne `journal_entry.created` porte `actor_type =
   'api_key'`, `actor_api_key_id` = la clé, `user_id` = son créateur. Même requête par session (JWT) → `actor_type = 'user'`,
   `actor_api_key_id` nul. **Et un rapprochement par clé** (V1-5) : `POST /api/v1/reconciliation/manual` (`lib.rs:887`)
   par la même clé → `journal_entry.created` porte la clé, comme `reconciliation.accepted` ; un cas de
   `POST /api/v1/reconciliation/accept` (`lib.rs:878`, `accept_one_invoice` ou `accept_one_rule`) si le montage d'une
   proposition tient dans le test, sinon les trois sites d'`accept_*` couverts par la garde et nommés au Change Log.
2. **AC2 — Une contre-passation par clé** (`POST /api/v1/journal-entries/{id}/reverse`, et
   `…/reverse-at-next-fiscal-year-start`) : l'écriture contre-passée porte **obligatoirement** une ligne sur un compte
   lettrable (`letterings::is_letterable_account`, `letterings.rs:421` ; condition du lettrage `journal_entries.rs:3054`) ;
   `journal_entry.reversed` **et** le `lettering.created` d'origine `reversal` portent la clé. Une contre-passation sur des
   comptes de charges seuls ne lettre rien et laisse survivre M3 : elle ne compte pas.
3. **AC3 — #892 item 11** : une facture fournisseur, par clé et **par l'API seule** : brouillon → `validate` → `cancel`
   (`POST /api/v1/supplier-invoices/{id}/cancel`, `lib.rs:664`, qui pose les paires `reversal`) → `unvalidate`
   (`lib.rs:656`). Ce cas est **obligatoire** : `supplier_invoice.unvalidated`, **les deux** `journal_entry.deleted`
   (contre-passation et achat, `supplier_invoices.rs:1410`, `:1412`) et le `lettering.removed` (`:1377`) portent la clé.
   Plus le cas simple (validée → brouillon, un `journal_entry.deleted`). Les traces de l'annulation elle-même
   (`supplier_invoice.cancelled`, son `lettering.created`) ne sont pas de cette story : non assertées, dites au Change Log.
4. **AC4 — Non-régression des autres chemins** : la validation d'une facture client par session reste `'user'` ; un
   appelant converti en `Acteur::utilisateur` écrit exactement l'entrée d'avant (tests de dépôt existants verts).
5. **AC5 — La garde** (D2) : verte sur la branche ; ses six sondes (a) à (f) consignées au Change Log ;
   sa table recomptée **depuis la source** à l'étape finale, la ventilation par raison écrite au Change Log, et son total
   cohérent avec elle. `audit_label_registry.rs` connaît `NewAuditLogEntry::par(` (action en position 1, entité en 2), avec
   une sonde (une action non enregistrée passée par `par(` rougit).
6. **AC6 — Manuel et CHANGELOG** : `admin-manual.tex:3193` dit ce qui est désormais attribué à la clé (les écritures —
   saisie, modification, suppression, contre-passation — et le retour en brouillon d'une facture fournisseur) et ce qui ne
   l'est pas encore, renvoyé à #431 ; **et que les entrées écrites avant la 0.17.7 par une clé gardent la marque
   « utilisateur »**, sans correction possible (D4). `user-manual.tex:3959-3960` (sur `main`), qui dit aujourd'hui l'auteur
   « clé API » « si elle a été faite par une clé d'API » — faux pour la plupart des actions —, est nuancé : la marque vaut,
   depuis la 0.17.7, pour les écritures et le retour en brouillon d'une facture fournisseur ; les autres actions et les
   entrées antérieures paraissent au nom du créateur de la clé ; renvoi à #431. Les deux PDF régénérés, contrôlés aplatis
   (présence des phrases neuves, absence de « une quinzaine de dépôts »). CHANGELOG `## [0.17.7] — Non publié`, *Corrigé*,
   un point `refs #431` (la PR porte `refs #431`, #431 reste ouverte : le déploiement est reporté à l'Epic 36) et
   `refs #892` (#892 garde ses dix autres items) ; il dit aussi que les entrées antérieures gardent leur marque.

## Tests à écrire, et la preuve du rouge

AC1 à AC3 écrits d'abord, sur `main` à la base de la branche : rouges par l'`actor_type = 'user'` (consigné dans
`kesh-gate-logs/e358a/rouges.log`). AC4 vert au départ et dit comme tel. La garde (AC5) naît verte avec sa table — sa preuve
est la sonde. Aucun test ne dépend d'un état i18n (aucun texte d'interface touché).

**Mutations** : M1 `From<&CurrentUser> for Acteur` qui jette la clé (AC1) ; M2 `par` qui appelle `::user` (AC1, AC2, AC3) ;
M3 le lettrage `reversal` rendu à `api_key_id: None` (AC2, dont le montage lettre obligatoirement — et la garde, de
compte) ; M4 `unvalidate_in_tx` qui passe `None` pour la clé à `delete_in_tx` ou au lettrage `:1377` (AC3, cas annulée
puis repassée en brouillon — et la garde) ; M5 une entrée de la table de la garde décrémentée (AC5) ; M6 `par(` retiré de
`FORMES` (`audit_label_registry.rs`, la sonde d'AC5) ; M7 `post_manual_once` qui passe `Acteur::utilisateur(user_id)`
(AC1, rapprochement par clé). Toute mutation survivante est écrite au Change Log avec sa raison.

## Tâches

- [ ] Étape 0 : relire les lignes citées sur la base réelle de la branche (après 35-5a0 à 35-5c) ; **relancer
      l'inventaire** (`kesh-gate-logs/e358a/inventaire.py <worktree>`, items `#[cfg(test)]` retirés) et écrire sa sortie
      au Change Log ; établir, pour chaque site, sa fonction englobante et sa raison, et pour chaque raison finale
      l'appelant qui la prouve ; **recompter le rayon** (Dev Notes, *Taille*) et appliquer le seuil qui y est écrit.
- [ ] **Lot 1** (commit à part) : rouges d'AC3 ; `Acteur`, `par`, `From<&CurrentUser>`, alias `letterings::Actor` ;
      pilote du retour en brouillon fournisseur ; garde et sa table ; `audit_label_registry.rs`.
- [ ] **Lot 2** (commit à part) : rouges d'AC1, AC2 ; signatures des écritures → `acteur` ; les six sites qui ont la clé ;
      appelants et tests convertis ; table de la garde recomptée.
- [ ] Manuels (deux `.tex`, deux PDF) ; CHANGELOG.
- [ ] Gates : backend complet à chaque commit (`kesh-db`) ; frontend : rien ne change (gate au push de l'intégration) ;
      E2E complet au dernier commit de code.

## Dev Notes

### Taille et découpage proposé (V1-7, C-e350-50)

La 35-8 entière touchait 16 dépôts et une vingtaine de routes ; la 35-8a reste à deux pilotes. **Rayon réel, relevé sur
`main` `67efc51f`** (motif `journal_entries::(create|create_in_tx|reverse|reverse_in_tx|reverse_owned_in_tx|`
`reverse_at_next_fiscal_year_start(_in_tx)?|delete_in_tx|delete_by_id|update)\b\s*\(`, `git grep -c`) :

| | fichiers | appels |
|---|---|---|
| production, appelants des écritures (hors `journal_entries.rs`) | 10 — `routes/{journal_entries,reconciliation}.rs`, `credit_notes`, `invoice_settlements_write`, `invoices`, `opening_complement`, `reconciliation_cancel`, `supplier_invoices`, `kesh-seed/src/pieges.rs` (et `retry.rs:147`, doc-comment, hors compte) | ≈ 25 |
| production, mécanisme et pilote fournisseur | `entities/audit_log.rs`, `letterings.rs`, `journal_entries.rs`, `supplier_invoices.rs`, `audit.rs`, `routes/supplier_invoices.rs` | — |
| tests d'intégration (`crates/*/tests`), écritures | 47 | 129 (`create` 60, `reverse` 24, `reverse_at_next_fiscal_year_start` 13, `create_in_tx` 11, `reverse_owned_in_tx` 9, `update` 5, `reverse_in_tx` 4, `delete_by_id` 2, `…_in_tx` 1) |
| `mod tests` de `src`, écritures | `journal_entries.rs` (appels non qualifiés après `:3177` : `create(` 56, `update(` 11, `delete_in_tx(` 11, `create_in_tx(` 3, `reverse_in_tx(` 2 — relevé brut, à trier à l'étape 0 ; la validation P1 en estimait ≈ 37), `accounts.rs` (2) | ≈ 85 |
| tests d'`unvalidate` | 6 (`supplier_invoices_draft` 21, `supplier_invoices_internal_number` 5, quatre à 1) | 30 |
| neufs | `audit_acteur_garde.rs`, tests d'AC1-AC3 | — |

**Verdict** : la logique est petite (un type, un constructeur, une garde, deux pilotes), mais le lot 2 (signatures des
écritures) touche **≈ 60 fichiers et ≈ 240 appels**, presque tous mécaniques (`user_id` → `Acteur::utilisateur(user_id)`).
Au sens de Guy (« petites stories »), **la 35-8a n'est pas petite**. **Découpage proposé**, à trancher par l'orchestrateur
avant le développement (la fiche est ordonnée pour qu'il se fasse sans réécriture : les deux lots des *Tâches*) :
- **35-8a1** (petite) — le lot 1 : `Acteur`, `par`, `From<&CurrentUser>`, la garde et sa table complète, le registre des
  libellés, le pilote du retour en brouillon fournisseur (AC3, AC5 ; `delete_in_tx` garde sa signature et reçoit
  `acteur.api_key_id`) ; manuel et CHANGELOG pour ce seul geste. ≈ 8 fichiers de production, 6 fichiers de tests.
- **35-8a2** (moyenne, mécanique, revue fichier par fichier) — le lot 2 : les écritures (AC1, AC2, AC4), dont les six sites
  qui ont la clé ; part de la 35-8a1 fusionnée.

**Seuil, s'il n'est pas découpé avant** : à l'étape 0, si le rayon recompté dépasse 70 fichiers, ou si le lot 1 ne converge
pas en revue avant le lot 2, le lot 2 sort en story à part.

### Fichiers communs — règle d'union C-e350-8

`repositories/journal_entries.rs` (35-5a0, 35-5a, 35-5b, 35-5c avant elle : la 35-8a part d'elles fusionnées, aucun hunk
concurrent) ; `CHANGELOG.md` ; `admin-manual.tex:3193` (aucune autre story de l'Epic 35 n'y écrit) ;
`user-manual.tex:3959-3960` (paragraphe du journal d'audit, distinct de ceux des 35-2a et 35-3) ;
`crates/kesh-api/tests/audit_label_registry.rs` (`FORMES` ; à signaler à toute story qui ajouterait un constructeur
d'audit). Les appelants de `create_in_tx` touchés par la ripple (`invoices.rs`, `supplier_invoices.rs`,
`invoice_settlements_write.rs`, `opening_balances.rs`, `reconciliation*.rs`…) le sont **après** toutes les autres stories de
l'epic : la 35-8a est la dernière à partir, sauf la 35-1, sans fichier commun avec elle hors `invoices.rs` (D de la 35-1).

### Ce qui doit rester vrai

- L'imputabilité humaine : `user_id` reste le créateur de la clé. Aucune entrée d'audit perdue ni doublée. Fixtures
  synthétiques seulement.

### Références

- Issues #431 (et ses commentaires de recensement), #892 (item 11) ; Story 17-2a ; Story 25-1b (patron `for_actor`) ;
  Story 15-8 (modification et suppression d'écriture par clé).
- `CLAUDE.md` : § *Inventorier les sites NON RÉSOLUS* ; § *Règle de splitting préventif* ; § *Migration breaking policy*
  (aucune migration ici).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `main`, `4d39f377`).

### File List

## Change Log

- 2026-11-12 — Fiche rédigée (Epic 35), née du découpage de la 35-8 (C-e350-38). Validation non lancée.
- 2026-11-13 — **Validation P1** (Opus 5.5, `kesh-gate-logs/e35-8a-validation-p1.md`) : 0 CRITICAL, 1 HIGH, 7 MEDIUM,
  2 LOW, tous acceptés après lecture de leur prémisse sur `main` `67efc51f`, aucune réfutation. Remédiation (agent de
  rédaction, Opus 5.5) :
  - **V1-1 (HIGH)** : `auth/bootstrap.rs:46-48` lu (module de test avant la production), `:195`, `:332` lus ; inventaire
    relevé à nouveau par un script qui retire les items `#[cfg(test)]` (`kesh-gate-logs/e358a/inventaire.py`) : **80 sites,
    30 fichiers**, `kesh-api` 32 — l'écart de 2 est exactement `bootstrap.rs` ; D2 réécrite (items retirés, sonde (d)).
  - **V1-2** : `for_actor` public relu (`entities/audit_log.rs:195-215`), 0 site `None` ; quatrième motif et sonde (f).
  - **V1-3** : `middleware/auth.rs:179`, `entities/audit_log.rs:187`, `bootstrap.rs:411`, `pieges.rs:511` relus ; raisons
    `DEFINITION`, `SYSTEME`, `FIXTURE` ; `helpers.rs:116` est sous `#[cfg(test)]` (`:105`) : retiré de la liste « hors
    dépôts », où la première rédaction le donnait pour de la production.
  - **V1-4** : `companies.rs:1002` (`lock_books`) et `:1079` (`unlock_books`) relus ; clé de la table
    `(fichier, fonction englobante, motif)`, sonde (e) de l'échange.
  - **V1-5** : `invoices.rs:2648`, `:2751`, `:2821` sont `mark_emailed`, `freeze_pdf`, `refreeze_pdf` (aucun appel
    d'écriture) ; `reconciliation.rs:2820`, `:3812`, `:4260`, `:5130`, `:5596` relus : D3 remplacée, cas de rapprochement
    par clé ajouté à AC1, M7.
  - **V1-6** : `journal_entries.rs:3054` (`is_letterable_account`) et `supplier_invoices.rs:1365-1412` relus ; AC2 exige
    une ligne lettrable, AC3 le cas annulée puis repassée en brouillon, par l'API seule ; M3 tuée par AC2.
  - **V1-7** : rayon recompté (Dev Notes, *Taille et découpage proposé*) : ≈ 60 fichiers, ≈ 240 appels — **la 35-8a n'est
    pas petite** ; découpage en 35-8a1 / 35-8a2 proposé à l'orchestrateur (C-e350-50), *Tâches* ordonnées en deux lots,
    seuil écrit.
  - **V1-8** : `user-manual.tex:3959-3960` lu sur `main` ; D4 et AC6 : les entrées antérieures gardent leur marque, dit au
    manuel administrateur, au manuel utilisateur (nuancé) et au CHANGELOG.
  - **V1-9 (LOW)** : #1057 citée à la raison `MCP`. **V1-10 (LOW)** : `auth.rs:635` et `:694` relus, `lib.rs:1139` ;
    `change_password_with` passe sous `ADMIN_SANS_CLE`.
  - **Restauration** (35-8b V1-1) : `admin.rs:534`, `:553-557`, `:593`, `:654` et `companies.rs:741-770` relus ; la 35-8a
    ne touche aucun de ces sites ; raison finale `RESTAURATION` (et `SYSTEME` pour `repair_installation_in_tx`).
  - **Report** : la 35-8b sort de l'Epic 35 (C-e350-48, décision de l'orchestrateur) ; raison `A_CONVERTIR_35_8B`
    renommée `A_CONVERTIR` ; #431 reste ouverte, la PR porte `refs #431`, `refs #892`.
  - Symptômes grepés dans la fiche, l'epic et le registre : `78`, `29 fichiers`, `trois motifs`, `:2655`,
    `reconciliation_link`, `35_8B`, `quinzaine de fichiers`. Statut `ready-for-dev` ; passe P2 non lancée.
