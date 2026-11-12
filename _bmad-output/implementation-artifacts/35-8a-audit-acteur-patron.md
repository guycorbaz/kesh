# Story 35-8a : l'audit attribue à la clé d'API ce qu'elle a fait — le type `Acteur`, la garde des sites non résolus, les écritures et le retour en brouillon d'une facture fournisseur (#431, #892 item 11)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de rédaction des fiches de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le
workflow `bmad-create-story`. Née du **découpage préventif** de la 35-8 (C-e350-38) : la story touchait seize dépôts de
`kesh-db` et autant de routes de `kesh-api` — plus de cinq modules (`CLAUDE.md`, § *Règle de splitting préventif*). La
**35-8a** est le *story-zero* : elle pose le motif (le type `Acteur`, le constructeur d'audit, le pont depuis
`CurrentUser`, la garde qui inventorie les sites non résolus) et le convertit sur deux pilotes ; la **35-8b** fait le
déploiement. Release **0.17.7**. **Aucune migration** : les colonnes `actor_type` et `actor_api_key_id` existent depuis la
Story 17-2a (migration `20260605000002`). Boucle complète, **`kesh-db` touché : gate complet à chaque commit**. Code lu sur
**`main`** (`4d39f377`, la 0.17.6 publiée). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -38 à -41) ;
`e35-choix-autonomes-e358a.md` pour la story, à créer au premier choix. Validation non lancée.*

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
- **La 35-8b part de la 35-8a fusionnée.** Les numéros de ligne ci-dessous sont ceux de `main` ; l'étape 0 les relit sur la
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

**L'inventaire des sites non résolus** (relevé sur `main`, code de production : avant le premier `#[cfg(test)]` de chaque
fichier de `crates/*/src`) — trois formes :

1. **`NewAuditLogEntry::user(`** : **78 sites** dans 29 fichiers — `kesh-db` 48 (accounts 4, backup_schedule 1,
   bank_profiles 3, companies 3, company_dunning_settings 1, company_invoice_settings 1, contacts 3, credit_notes 2,
   fiscal_years 1, installation_settings 1, invoice_settlements_write 3, invoices 6, journal_entries 2, payment_batches 3,
   products 3, reconciliation_rules 3, supplier_invoices 8) ; `kesh-api` 30 (audit.rs 1 — le pont lui-même —, admin 2,
   api_keys 2, auth 4, companies 3, mcp_envois 2, mcp_keys 3, mcp_prompts 5, mcp_server_settings 1, setup 1, users 4,
   serveur_mcp 2). Faux positif écarté : `services/mod.rs:56` (commentaire `//!`). Commande :
   ```sh
   for f in $(git grep -l "NewAuditLogEntry::user(" -- 'crates/*/src/*'); do
     t=$(/usr/bin/grep -n '^#\[cfg(test)\]' "$f" | head -1 | cut -d: -f1); t=${t:-999999}
     /usr/bin/grep -n "NewAuditLogEntry::user(" "$f" | awk -F: -v t="$t" -v f="$f" '$1<t{print f":"$1}'
   done | /usr/bin/grep -v 'services/mod.rs:56'
   ```
2. **`api_key_id: None`** (un `letterings::Actor` construit sans la clé) : **9 sites** de production de `kesh-db` —
   `credit_notes.rs:797`, `invoice_settlements_write.rs:424`, `:772`, `:1017`, `journal_entries.rs:3068` (la contre-passation,
   lettrage `reversal`), `supplier_invoices.rs:1377`, `:1942`, `:2594`, `:2863`. Hors dépôts : `helpers.rs:116` et
   `middleware/auth.rs:179` construisent un `CurrentUser` JWT, non une trace ; `entities/audit_log.rs:187` est le
   constructeur `::user` lui-même.
3. **Un `None` positionnel passé pour la clé** à une fonction qui l'attend : `journal_entries::delete_in_tx(…, user_id, None,
   …)` appelé par `invoices.rs:1689` (dévalidation d'une facture client), `supplier_invoices.rs:1410` et `:1412` (retour en
   brouillon d'une facture fournisseur) — **#892 item 11** : `journal_entry.deleted` (`journal_entries.rs:1992`) et
   `lettering.removed` (`letterings.rs:1428`, par l'`Actor` de `supplier_invoices.rs:1377`) sortent sans la clé. Cette forme
   ne se grep pas ; elle disparaît quand la fonction prend un `Acteur` (D1).

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
- **D2 — La garde des sites non résolus** (C-e350-40), `crates/kesh-api/tests/audit_acteur_garde.rs` (neuve) : elle lit le
  code de production de `crates/*/src` (avant le premier `#[cfg(test)]` de chaque fichier ; commentaires `//` retirés) et
  compte, **par fichier**, les trois motifs greppables : `NewAuditLogEntry::user(`, `Acteur::utilisateur(`,
  `api_key_id: None`. Elle compare à une **table close** `(fichier, motif, nombre, raison)` ; tout écart rougit, en nommant le
  fichier — un site neuf non listé comme un site converti sans que la table suive. Les raisons sont des constantes :
  - `PRE_AUTH` — avant toute authentification (`routes/auth.rs`, `routes/setup.rs`, `auth/bootstrap.rs`) ;
  - `ADMIN_SANS_CLE` — atteint seulement par `admin_routes` sous `require_not_pat` ou par un handler qui refuse une clé
    (`routes/admin.rs`, `users.rs`, `api_keys.rs`, `mcp_*`, `companies.rs` des routes adresse/e-mail/coordonnées, et les
    dépôts qu'ils seuls appellent : `backup_schedule`, `installation_settings`, `companies::repair_installation_in_tx`,
    `unlock_books`, `company_invoice_settings::update`, `company_dunning_settings::update`, `invoices::delete`,
    `supplier_invoices::delete_draft_in_tx`) — **chaque entrée cite l'appelant qui l'établit** (route et ligne de
    `lib.rs`) ; une fonction appelée aussi par une route comptable n'y entre pas ;
  - `MCP` — le serveur MCP (`services/serveur_mcp.rs`) : une clé MCP n'est pas une clé REST ; limite écrite au manuel
    (`admin-manual.tex:3392`), hors de #431 (D5) ;
  - `PONT` — `audit.rs` (la branche `None` de `from_current_user`) ;
  - `A_CONVERTIR_35_8B` — tout le reste, au compte exact : la 35-8b vide cette raison.
  Sondes au Change Log : un `NewAuditLogEntry::user(` ajouté à un fichier non listé rougit ; un site converti sans
  décrément rougit ; la même forme coupée sur deux lignes (`NewAuditLogEntry::user\n(`) — la garde lit après jointure des
  lignes ou le motif tolère les blancs (`NewAuditLogEntry::user\s*\(`), à écrire et à sonder.
- **D3 — Les pilotes**, convertis en entier :
  - **écritures** : `create`, `create_in_tx`, `create_in_tx_inner` → `acteur: Acteur` ; `journal_entry.created` par
    `NewAuditLogEntry::par` ; `reverse*` → `acteur` ; `journal_entry.reversed` et le lettrage `reversal` (`:3068`) portent la
    clé (l'écart R3 de `letterings.rs:201-202` est levé) ; `delete_in_tx`, `delete_by_id`, `update` passent de `(user_id,
    actor_api_key_id)` à `acteur`. Les routes de `routes/journal_entries.rs` passent `Acteur::from(&current_user)`. Les
    **autres appelants** de ces fonctions (facturation, règlements, soldes d'ouverture, réconciliation, factures
    fournisseur…) passent `Acteur::utilisateur(user_id)` là où ils n'ont qu'un `user_id` : chacun est un site de la garde
    (`A_CONVERTIR_35_8B`), au compte exact ; ceux qui ont déjà la clé en main (`reconciliation_cancel`, `reconciliation_link`,
    `invoices` aux lignes `:2655`, `:2758`, `:2828`) la passent.
  - **retour en brouillon d'une facture fournisseur** (#892 item 11) : `unvalidate_in_tx` et sa route prennent l'`Acteur` ;
    `supplier_invoice.unvalidated`, les deux `journal_entry.deleted` et le `lettering.removed` portent la clé.
- **D4 — Ce qui ne change pas** : le `user_id` d'une entrée par clé reste le **créateur de la clé** (imputabilité, Story
  17-2a) ; le vocabulaire des actions ; le schéma ; l'affichage du journal d'audit (qui montre déjà `actor_type`). Aucune
  route nouvelle, aucune réponse d'API changée.
- **D5 — Hors de la story** : le serveur MCP (raison `MCP` de la garde ; une issue est demandée à l'orchestrateur si
  aucune ne suit déjà la limite du manuel `admin-manual.tex:3392`) ; le reste des dépôts (35-8b).

## Critères d'acceptation

1. **AC1 — Une écriture passée par clé** (`crates/kesh-api/tests/`, patron de `api_keys_e2e.rs:438-505`) : clé
   `read-write` d'un Comptable, `POST /api/v1/journal-entries` → la ligne `journal_entry.created` porte `actor_type =
   'api_key'`, `actor_api_key_id` = la clé, `user_id` = son créateur. Même requête par session (JWT) → `actor_type = 'user'`,
   `actor_api_key_id` nul.
2. **AC2 — Une contre-passation par clé** (`POST /api/v1/journal-entries/{id}/reverse`, et
   `…/reverse-at-next-fiscal-year-start`) : `journal_entry.reversed` et, si la contre-passation lettre, le `lettering.created`
   de l'origine `reversal` portent la clé.
3. **AC3 — #892 item 11** : une facture fournisseur validée puis repassée en brouillon par clé
   (`POST /api/v1/supplier-invoices/{id}/unvalidate`, cas avec écriture d'achat et, si le montage le permet, contre-passation
   et lettrage) : `supplier_invoice.unvalidated`, chaque `journal_entry.deleted` et le `lettering.removed` portent la clé.
4. **AC4 — Non-régression des autres chemins** : la validation d'une facture client par session reste `'user'` ; un
   appelant converti en `Acteur::utilisateur` écrit exactement l'entrée d'avant (tests de dépôt existants verts).
5. **AC5 — La garde** (D2) : verte sur la branche ; ses sondes (positive, de compte, multi-ligne) consignées au Change Log ;
   sa table recomptée **depuis la source** à l'étape finale, la ventilation par raison écrite au Change Log, et son total
   cohérent avec elle. `audit_label_registry.rs` connaît `NewAuditLogEntry::par(` (action en position 1, entité en 2), avec
   une sonde (une action non enregistrée passée par `par(` rougit).
6. **AC6 — Manuel et CHANGELOG** : `admin-manual.tex:3193` dit ce qui est désormais attribué à la clé (les écritures —
   saisie, modification, suppression, contre-passation — et le retour en brouillon d'une facture fournisseur) et ce qui ne
   l'est pas encore, renvoyé à #431 ; PDF régénéré, contrôlé aplati (présence d'une phrase neuve, absence de « une quinzaine
   de dépôts »). CHANGELOG `## [0.17.7] — Non publié`, *Corrigé*, un point `refs #431` (la PR porte `refs #431`, #431 reste
   ouverte jusqu'à la 35-8b) et `refs #892` (#892 garde ses dix autres items).

## Tests à écrire, et la preuve du rouge

AC1 à AC3 écrits d'abord, sur `main` à la base de la branche : rouges par l'`actor_type = 'user'` (consigné dans
`kesh-gate-logs/e358a/rouges.log`). AC4 vert au départ et dit comme tel. La garde (AC5) naît verte avec sa table — sa preuve
est la sonde. Aucun test ne dépend d'un état i18n (aucun texte d'interface touché).

**Mutations** : M1 `From<&CurrentUser> for Acteur` qui jette la clé (AC1) ; M2 `par` qui appelle `::user` (AC1, AC2, AC3) ;
M3 le lettrage `reversal` rendu à `api_key_id: None` (AC2 — et la garde, de compte) ; M4 `unvalidate_in_tx` qui passe
`Acteur::utilisateur` à `delete_in_tx` (AC3 — et la garde) ; M5 une entrée de la table de la garde décrémentée (AC5) ; M6
`par(` retiré de `FORMES` (`audit_label_registry.rs`, la sonde d'AC5).

## Tâches

- [ ] Étape 0 : relire les lignes citées sur la base réelle de la branche (après 35-5a0 à 35-5c) ; **relancer la commande
      de l'inventaire** et écrire sa sortie au Change Log ; établir, pour chaque site `ADMIN_SANS_CLE`, l'appelant qui le
      prouve.
- [ ] Rouges ; `Acteur`, `par`, `From<&CurrentUser>` ; pilotes ; garde ; `audit_label_registry.rs` ; manuel ; CHANGELOG.
- [ ] Gates : backend complet à chaque commit (`kesh-db`) ; frontend : rien ne change (gate au push de l'intégration) ;
      E2E complet au dernier commit de code.

## Dev Notes

### Signal de découpage

La 35-8 entière touchait 16 dépôts et une vingtaine de routes ; la 35-8a reste à deux pilotes, mais la conversion de
`create_in_tx` fait passer `Acteur::utilisateur` à ses appelants (une quinzaine de fichiers, mécanique). Si la boucle de
validation fait voir que ce rayon dépasse une boucle, découper encore : `Acteur` et la garde seuls, puis les pilotes.

### Fichiers communs — règle d'union C-e350-8

`repositories/journal_entries.rs` (35-5a0, 35-5a, 35-5b, 35-5c avant elle : la 35-8a part d'elles fusionnées, aucun hunk
concurrent) ; `CHANGELOG.md` ; `admin-manual.tex:3193` (aucune autre story de l'Epic 35 n'y écrit) ;
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
