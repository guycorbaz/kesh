# Story 35-8b : l'audit attribue à la clé d'API ce qu'elle a fait — partout ailleurs (#431)

Status: backlog

> ⚠️ **Reportée à l'Epic 36 le 2026-11-13** (C-e350-48, décision de l'orchestrateur ; Guy veut des epics courtes à petites
> stories, et celle-ci touche ≈ 140 fichiers et ≈ 1000 appels). Elle y sera **découpée en lots par domaine**. La fiche est
> gardée telle quelle ; ses numéros de ligne et la release « 0.17.7 » ci-dessous ne valent plus. **Au découpage, traiter
> d'abord les findings de sa validation P1** (`kesh-gate-logs/e35-8b-validation-p1.md` : V1-1 la restauration, qui signe
> délibérément par `audit_uid` — raison finale `RESTAURATION` déjà posée par la 35-8a ; V1-2 la table finale ; V1-3 les
> deux documents de l'attribution ; V1-4 le rayon ; LOW V1-5 à V1-7), et repartir de la garde de la 35-8a telle que
> fusionnée : clé par site, raison `A_CONVERTIR` (non plus `A_CONVERTIR_35_8B`), raisons `SYSTEME`, `RESTAURATION`,
> `DEFINITION`, `FIXTURE` (C-e350-49).

*Fiche rédigée le 2026-11-12 par l'agent de rédaction des fiches de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le
workflow `bmad-create-story`. Seconde des deux stories du découpage de la 35-8 (C-e350-38) : le **déploiement** du motif
posé par la **35-8a** (`Acteur`, `NewAuditLogEntry::par`, `From<&CurrentUser> for Acteur`, la garde
`audit_acteur_garde.rs`), dont elle **part fusionnée**. Story de *rollout* au sens du `CLAUDE.md` (§ *Règle de splitting
préventif*) : mécanique, revue fichier par fichier. Release **0.17.7**. **Aucune migration**. **`kesh-db` touché : gate
complet à chaque commit**. Code lu sur **`main`** (`4d39f377`). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -38 à
-41) ; `e35-choix-autonomes-e358b.md` pour la story, à créer au premier choix. Validation non lancée.*

## Story

En tant que titulaire dont un agent écrit dans Kesh par une clé d'API REST, je veux que **chaque** action de cet agent —
contacts, articles, comptes, factures, règlements, factures fournisseur, lots de paiement, règles d'affectation, verrou
des livres, exercices — soit attribuée à la clé dans le journal d'audit, afin qu'un réviseur qui filtre les actions des
intégrations les voie toutes.

## Ordre et dépendances

Après la **35-8a** fusionnée (donc après les 35-5a0, 35-5a, 35-5b, 35-5c et la 35-3, qu'elle attend). La **35-3** touche
`repositories/company_invoice_settings.rs` (`lock_designated_accounts_in_tx`, `check_written`) : hunks distincts de la trace
`company_invoice_settings.updated` (`:455` sur `main`). La **35-1** peut toucher `repositories/invoices.rs` (`:2334-2340`) :
hunk distinct ; l'ordre de fusion se règle à l'intégration.

## Ce qui se lit au code (`main`, à relire sur la base réelle)

L'inventaire de la 35-8a (sa fiche, § *Ce qui se lit au code*) ; ce qui reste après elle, classé par la fonction qui trace
et par sa **route** (les routes comptables admettent une clé `read-write` ; `admin_routes` est sous `require_not_pat`,
`crates/kesh-api/src/lib.rs:517-519`) :

| Dépôt (`kesh-db`) | Sites `::user` | Fonctions | Route(s) |
|---|---|---|---|
| `accounts.rs` | 4 (`:225`, `:809`, `:943`, `:1121`) | create, update, archive_guarded, reactivate | comptable (`/accounts…`) |
| `bank_profiles.rs` | 3 (`:67`, `:288`, `:333`) | create, update, delete | comptable |
| `companies.rs` | 3 (`:768`, `:1052`, `:1180`) | repair_installation_in_tx, lock_books, unlock_books | démarrage et admin ; comptable (`books-lock`) ; admin (`…/release`) |
| `company_dunning_settings.rs` | 1 (`:131`) | update | admin (`lib.rs:291`) |
| `company_invoice_settings.rs` | 1 (`:455`) | update | admin (`lib.rs:278`) |
| `contacts.rs` | 3 (`:318`, `:633`, `:711`) | create, update, archive | comptable |
| `credit_notes.rs` | 2 (`:814`, `:828`) + `Actor` sans clé `:797` | create_credit_note | comptable (`POST /credit-notes`) |
| `fiscal_years.rs` | 1 (`:300`, `build_audit_entry`) | création, clôture, réouverture | comptable ; admin (`reopen`) |
| `backup_schedule.rs`, `installation_settings.rs` | 1 + 1 | update_settings, update_session_idle_minutes | admin |
| `invoice_settlements_write.rs` | 3 (`:440`, `:781`, `:1092`) + `Actor` sans clé `:424`, `:772`, `:1017` | settle, write_off, cancel_settlement | comptable |
| `invoices.rs` | 6 (`:779`, `:1335`, `:1436`, `:1695`, `:2466`, `:2612`) + `None` positionnel `:1689` | create, update, delete, unvalidate, validate, set_dunning_pause | comptable ; admin (`DELETE`) |
| `payment_batches.rs` | 3 (`:306`, `:665`, `:754`) | create, confirm, cancel | comptable |
| `products.rs` | 3 (`:187`, `:356`, `:436`) | create, update, archive | comptable |
| `reconciliation_rules.rs` | 3 (`:233`, `:385`, `:443`) | create, update, soft_delete | comptable |
| `supplier_invoices.rs` | 7 hors `:1425` (`:661`, `:789`, `:887`, `:903`, `:1799`, `:2654`, `:2914`) + `Actor` sans clé `:1942`, `:2594`, `:2863` | create_draft, update_draft, delete_draft (deux traces), validate, cancel, cancel_settlement | comptable ; admin (`DELETE`) |

S'y ajoutent les `Acteur::utilisateur(user_id)` que la 35-8a a posés chez les appelants de `journal_entries::create_in_tx`
et des contre-passations (raison `A_CONVERTIR_35_8B` de la garde, au compte qu'elle aura écrit).

`kesh-api` (handlers qui construisent l'entrée eux-mêmes) : `routes/admin.rs` 2, `users.rs` 4, `api_keys.rs` 2,
`companies.rs` 3, `mcp_envois.rs` 2, `mcp_keys.rs` 3, `mcp_prompts.rs` 5, `mcp_server_settings.rs` 1 — tous derrière une
session (admin, ou handler qui refuse la clé) ; `auth.rs` 4 (`:600` vol de jeton, `:694` changement de mot de passe,
`:1116` et `:1288` mot de passe oublié) ; `setup.rs:252` ; `services/serveur_mcp.rs` 2 ; `audit.rs:51` (le pont).

**L'incohérence relevée à #431** (commentaire de la validation P4 de la 25-1b) : `DELETE /api/v1/reconciliation/rules/{id}`
trace `reconciliation_rule.deleted` par `::user` quand la règle était active (`reconciliation_rules.rs:443` sur `main`) et par
`from_current_user` quand elle était déjà inactive (`routes/reconciliation_rules.rs`) : l'`actor_type` dépend de l'effet.

## Décisions

- **D1 — Tout site atteint par un handler authentifié prend l'`Acteur` de la requête**, admin compris (C-e350-41) : la
  défense ne repose plus sur la place de `require_not_pat`, et la garde n'a plus à prouver qu'un dépôt n'est appelé que
  par l'administration. Les dépôts prennent `acteur: Acteur` à la place de `user_id` ; les handlers passent
  `Acteur::from(&current_user)` ; ceux de `kesh-api` qui construisent l'entrée emploient `from_current_user`.
- **D2 — Restent hors du motif, à la fin**, dans la garde : `PRE_AUTH` (`auth.rs` : vol de jeton, mot de passe oublié et
  réinitialisé ; `setup.rs`) ; `SYSTEME` (`companies::repair_installation_in_tx` au démarrage, que le manuel décrit déjà
  « signée par le plus ancien administrateur », `admin-manual.tex:2531` — son appel par la route admin prend l'`Acteur`) ;
  `PONT` (`audit.rs`) ; `MCP` (`serveur_mcp.rs`). **La raison `A_CONVERTIR_35_8B` est vide, et `ADMIN_SANS_CLE` aussi** (D1) ;
  la garde les retire de ses constantes. `auth.rs:694` (changement de mot de passe) : convertie si sa route admet une clé,
  `PRE_AUTH` sinon — l'étape 0 relit la garde de la route et l'écrit au Change Log.
- **D3 — `reconciliation_rule.deleted`** porte le même acteur dans ses deux cas.
- **D4 — Fermeture** : la PR porte `closes #431` ; le motif de fermeture dit ce qui reste hors de #431 (le serveur MCP, sa
  limite écrite au manuel, suivie à **#1057**).

## Critères d'acceptation

1. **AC1 — Un test par famille de geste, par clé** (patron `api_keys_e2e.rs:438-505`, `create_key_via_http`) : contacts
   (création), articles (création), comptes (création), facture client (création, validation, dévalidation — avec le
   `journal_entry.deleted` de `invoices.rs:1689`), règlement d'une facture et son annulation, facture fournisseur
   (brouillon, validation, annulation), lot de paiement (création), règle d'affectation (création, suppression d'une règle
   **active** et d'une règle **inactive** : même `actor_type`, D3), verrou des livres (`POST …/books-lock`), avoir (création),
   exercice (création ou clôture) : chaque trace porte `actor_type = 'api_key'`, la clé, son créateur. Le même geste par
   session reste `'user'` (un cas par famille suffit s'il passe par la même fonction).
2. **AC2 — La garde** : table finale à `PRE_AUTH`, `SYSTEME`, `PONT`, `MCP` seules, au compte exact recompté depuis la
   source et écrit au Change Log (ventilation et total cohérents) ; les constantes `A_CONVERTIR_35_8B` et `ADMIN_SANS_CLE`
   retirées ; sondes rejouées (positive, de compte).
3. **AC3 — Non-régression** : les tests de dépôts et d'API existants verts, l'attribut `user_id` inchangé partout ;
   `audit_label_registry.rs` et `audit_route_registry.rs` verts.
4. **AC4 — Manuel et CHANGELOG** : `admin-manual.tex:3193` dit l'attribution **complète** pour une clé REST et renvoie au
   serveur MCP pour sa limite (`:3392`) ; PDF régénéré, contrôlé aplati (absence de « pas encore complète » et de « \#431 »
   à cet endroit). CHANGELOG `## [0.17.7] — Non publié`, *Corrigé*, le point de #431 complété.

## Tests à écrire, et la preuve du rouge

AC1 écrit d'abord sur la base de la branche : rouge par `'user'` pour chaque famille (journal `kesh-gate-logs/e358b/
rouges.log`). AC3 vert au départ. Aucun test ne dépend d'un état i18n.

**Mutations** (une par dépôt de poids, chacune tuée par son test d'AC1) : M1 `contacts::create` revient à
`Acteur::utilisateur` ; M2 `invoices::unvalidate` repasse `None` à la suppression ; M3 `reconciliation_rules::soft_delete`
revient à `::user` (le cas « règle active ») ; M4 `companies::lock_books` idem ; M5 `invoice_settlements_write` rend un
`Actor` sans clé ; M6 une entrée `A_CONVERTIR_35_8B` réintroduite dans la garde (AC2).

## Tâches

- [ ] Étape 0 : relancer la commande d'inventaire de la 35-8a sur la base réelle ; écrire la table de départ au Change Log ;
      relire la garde de la route de `auth.rs:694`.
- [ ] Rouges ; conversion fichier par fichier (un commit par dépôt ou par petit groupe, gate complet à chacun) ; garde ;
      manuel ; CHANGELOG.
- [ ] Gates : backend complet à chaque commit ; E2E complet au dernier commit de code.

## Dev Notes

### Fichiers communs — règle d'union C-e350-8

`repositories/company_invoice_settings.rs` avec la 35-3 (fusionnée avant) ; `repositories/invoices.rs` avec la 35-1
(`:2334-2340` contre `:779`, `:1335`, `:1436`, `:1689-1695`, `:2466`, `:2612` : hunks distincts) ; `admin-manual.tex:3193` et
`audit_acteur_garde.rs` avec la 35-8a (fusionnée avant) ; `CHANGELOG.md`. Tous les autres dépôts et routes : aucune autre
story de l'epic n'y écrit après elle.

### Ce qui doit rester vrai

- `user_id` reste le créateur de la clé. Aucune trace perdue ni doublée ; aucune réponse d'API changée. Fixtures
  synthétiques seulement.

### Références

- Issue #431 et ses commentaires ; fiche 35-8a ; Stories 17-2a, 25-1b, 15-8.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `main`, `4d39f377`).

### File List

## Change Log

- 2026-11-12 — Fiche rédigée (Epic 35), née du découpage de la 35-8 (C-e350-38). Validation non lancée.
