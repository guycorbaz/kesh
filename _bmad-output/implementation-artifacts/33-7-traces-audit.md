# Story 33-7 : le journal d'audit : changement de mot de passe, vol de jeton, export du grand livre, consultation par le MCP (#435, #729, #776)

Status: ready-for-dev

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`,
numéros de ligne de `4ef19901`. Release **0.17.5**, Epic 33. **Aucune migration.** Registre : `e33-choix-autonomes-e334.md`,
C-e334-11 à 15. Commits en `refs` ; à la PR : `closes #435`, `closes #729`, `closes #776`.*

## Story

En tant qu'Administrateur ou réviseur, je veux que le journal d'audit dise qu'un mot de passe a été changé, qu'un vol de jeton a révoqué des sessions,
qu'un grand livre a été **exporté** et dans quel format, et qu'un rapport a été lu **par un agent** ; pour que la piste de contrôle ne se taise plus
sur ces gestes, et ne les confonde plus avec une simple consultation.

## Ce que le code dit (prémisses établies)

**#435.**
- ⚠️ **Lignes de l'issue périmées** : `change_password` est à `routes/auth.rs:582-670` ; la révocation pour vol à `:468-478` ; les audits du parcours
  « oublié » à `:1033` et `:1205`.
- `change_password` : `users::update_password` sur le pool (`:618`), révocation (`:622`), jeton neuf (`:633`). `users::update_password_in_tx` existe (reset, `:1195`).
- La route est dans le routeur authentifié (`lib.rs:1127`), hors `admin_routes`, donc **ouverte à une clé d'API** ? `current_user.api_key_id` est à lire au code : l'entrée
  passe par `NewAuditLogEntry::for_actor` (`entities/audit_log.rs:194`) ou le `from_current_user` du trait `AuditActor` (`:153`), jamais `::user`, en conséquence.
- Registre : `tests/audit_route_registry.rs:542,558-560`, `Exempt("issue #435 — …")`, décompte `:1005-1007` (5 exemptions).

**#729.**
- `services/rapports.rs:413-428` : `(DetailsTrace::GrandLivre, _)` → `emit_ledger_audit`, qui écrit `report.generated` sans format (`:857-893`) — pour les deux usages. La
  table de `tests/services_motif.rs:488-500` (ligne 12) épingle ce comportement, et annonce elle-même que son correctif la modifiera.
- Autres mentions à retirer : `rapports.rs:78-79` (« sauf le grand livre, #729 »), `:368-369`, `:414`.

**#776.**
- `Rapport::nouveau` ne reçoit que `user_id` (`rapports.rs:302`) ; `ContexteAcces::mcp` n'a pas l'identifiant de clé (`services/acces.rs:89`), que
  `CleMcpAuthentifiee.cle_id` porte pourtant à la création (`services/cles_mcp.rs:108-115`). Dix sites construisent ce contexte (un en production, neuf en tests).
- ⚠️ **Prémisse corrigée — lignes du manuel** : `user-manual.tex:3153-3157` cité par l'issue est aujourd'hui à **`:3637-3643`**, plus **`:3750-3754`**, `admin-manual.tex:3278`, `:3417`, `:3498`.
- **Limite voisine L2, courte** : les sept `emit_*` écrivent `NewAuditLogEntry::user` même par une clé d'API (`rapports.rs:940-944`). `CurrentUser.api_key_id` existe, et `for_actor` aussi.

**Vocabulaire public.** `audit_labels.rs` est la source unique (`ACTIONS`, triée, `:86-87,193-194`) ; `tests/audit_label_registry.rs` la garde ; les libellés vivent dans les quatre locales
(`audit-log-action-*`, `fr-CH/messages.ftl:3157-3158,3176-3177`). L'écran déplie le détail tel quel : une clé de plus s'y lit sans code.

## Conception

- **Actions neuves** : `auth.password_changed` (« Mot de passe modifié »), `auth.session_theft_detected` (« Vol de jeton détecté, sessions révoquées »). Entité `user`.
- **`change_password`** : `update_password_in_tx` + audit dans **une** transaction, sur le modèle de `reset_password` (`:1195-1218`) ; la révocation et le jeton neuf restent après, inchangés. **Aucun secret** : `details_json: None`.
- **Vol de jeton** : après `revoke_all_for_user` (`:469`), entrée `{ "revoked_sessions": n }`, attribuée à `token.user_id`. **Au mieux** (`tracing::warn!` si elle échoue) : le 401 ne dépend pas d'elle et la révocation est déjà faite (C-e334-12). Le chemin de grâce (`:461-466`) et les refus simples n'écrivent rien.
- ⚠️ **Connexion, déconnexion, renouvellement ordinaire : pas de trace, délibérément** (C-e334-11). #435 pose elle-même que c'est du trafic qui noierait les gestes comptables. Les exemptions du registre changent de motif, et l'exemption reste **nommée au manuel** (`admin-manual.tex:3157`). `Exempt` exige un numéro d'issue (`audit_route_registry.rs:307-310`) : #435 se fermant, la justification cite `C-e334-11` et #435 **comme décision**, ou une issue neuve si la revue l'exige.
- **#729** : `(GrandLivre, None)` → `emit_ledger_audit` ; `(GrandLivre, Some(f))` → nouvelle `emit_ledger_export_audit`, `report.exported`, `{report_type, format, period_start, period_end, account_ids}` (snake_case). Ligne 12 de la table mise à jour, renommée.
- **#776** (C-e334-13, 14) :
  - `ContexteAcces::mcp` prend `cle_id: i64`, conservé en champ privé, avec un accesseur `origine_trace() -> OrigineTrace` : `Session`, `CleApi(id)` (depuis `depuis_api`), `Mcp(id)`. `Canal` reste inchangé : ses `PartialEq` sont employés par `mcp_admin_routes_e2e.rs:270`.
  - `Rapport` garde l'origine, `tracer` la passe aux huit émetteurs (sept + l'export du grand livre). Par un seul helper `detail_avec_origine(json, origine)` (DRY) : `Mcp(id)` ajoute `"channel": "mcp", "mcp_key_id": id`. **Rien de plus pour une session** : les lignes existantes de `services_motif.rs` (comparées au JSON exact) ne bougent pas, et un détail sans `channel` se lit « écran, ou ligne antérieure ».
  - **Clé d'API** : `for_actor(user_id, api_key_id, …)` dans les émetteurs (limite L2 levée, commentaires retirés) : `actor_type = api_key`, et l'écran marque déjà « clé API » (`user-manual.tex:3634-3635`). Pas de `channel` : la colonne le dit.
  - La trace reste **au mieux** (`admin-manual.tex:3501`, inchangé).

## Acceptance Criteria

1. **AC-1** : `PUT /api/v1/auth/password` réussi : une entrée `auth.password_changed`, `entity_type = user`, `entity_id` = l'utilisateur, `details_json` nul ; aucun mot de passe ni condensat nulle part dans la ligne.
2. **AC-2** : mot de passe courant faux, ou nouveau refusé → aucune entrée ; **et le mot de passe reste l'ancien** si l'insertion d'audit échoue (même transaction).
3. **AC-3** : rejeu d'un jeton révoqué hors de la grâce : une entrée `auth.session_theft_detected`, `revoked_sessions` juste ; réponse 401 inchangée. Rejeu dans la grâce (`ROTATION_GRACE_SECS`) → aucune entrée. Refus simple (déconnexion antérieure) → aucune entrée.
4. **AC-4** : connexion, déconnexion, renouvellement ordinaire n'écrivent toujours rien (test d'absence, avec **sonde positive** sur le chemin du vol) ; registre : `change_password` passe `Traced`, `refresh` reste `Exempt` au motif neuf, compteurs recomptés (`exempt = 4`, `traced + 1`).
5. **AC-5** : les deux actions ont leur libellé dans les quatre locales et figurent à `ACTIONS` en ordre (garde `audit_label_registry`) ; elles sont filtrables à l'écran (route de vocabulaire).
6. **AC-6 — #729** : `GET` d'export du grand livre, PDF et CSV : `report.exported` avec `format`. Écran du grand livre : `report.generated`, sans `format`. Lus en base, sur les deux chemins. Ligne 12 mise à jour.
7. **AC-7 — #776** : un outil de rapport par une clé MCP (les cinq : grand livre, balance, bilan, compte de résultat, balance âgée) écrit `channel: "mcp"`, `mcp_key_id` juste. Le même rapport à l'écran : **aucune** de ces clés, JSON exact inchangé (`services_motif.rs`, sans modification hors ligne 12).
8. **AC-8** : même rapport, par une clé d'API : `actor_type = api_key`, `actor_api_key_id` juste, et par une session : `user`. Consultation et export.
9. **AC-9** : `/usr/bin/grep -n '#776\|\\#776\|#729\|\\#729\|\\#435' docs/manual/fr/*.tex crates/kesh-api/src/services/rapports.rs` ne rend plus que ce qui est historique, et chaque ligne restante est justifiée au Change Log.

## Tâches

- [ ] **T1 — rouges d'abord** : `tests/auth_audit_e2e.rs` (AC-1 à 4) ; `services_motif.rs`, ligne 12 ; `tests/mcp_outils_e2e.rs`, ou voisin, AC-7 ; `reports_e2e.rs`, AC-8.
- [ ] **T2** : `auth.rs`, transaction et vol. **Inventaire du symptôme** : grep de `revoke_all_for_user` (reset, break-glass de `bootstrap.rs`), dire au Change Log pourquoi ces chemins suivent ou non.
- [ ] **T3** : `audit_labels.rs`, quatre locales, registre.
- [ ] **T4** : `rapports.rs`, `acces.rs`, `cles_mcp.rs`, dix constructions. La garde `contexte_acces_mcp_construit_par_authentifier_seul` (`mcp_gardes_lexicales.rs:316`) doit passer sans affaiblissement.
- [ ] **T5** : manuels, puis CHANGELOG *Corrigé* (`refs #435`, `refs #729`, `refs #776`).

### Mutations nommées (P2)

- **M1** : audit de `change_password` hors transaction (après le commit) → rouge AC-2. **M2** : entrée de vol écrite aussi au chemin de grâce → rouge AC-3.
- **M3** : `(GrandLivre, Some(f))` renvoyé à `emit_ledger_audit` → rouge AC-6. **M4** : `format` omis → rouge AC-6.
- **M5** : `mcp_key_id` valant l'identifiant de l'utilisateur (`user_id`) → rouge AC-7 (un test avec une clé dont le numéro diffère de l'utilisateur : **à vérifier au montage**). **M6** : `channel` ajouté aussi à une session → rouge AC-7.
- **M7** : `for_actor` rendu `user` → rouge AC-8. **M8** : un seul des huit émetteurs sans origine → rouge AC-7, ligne de ce rapport.

## Gates

`test-fast.sh`, Vitest si un texte frontend change, `lint-i18n-ownership`, E2E complet au dernier commit de code. Aucune écriture de `kesh-db/migrations`. Aucun contrôle visible neuf : les captures ne s'imposent pas, mais une capture du journal d'audit (les deux libellés, détail MCP) sert la revue.

## Manuels (à nommer dans les prompts de revue)

- `admin-manual.tex:3157` : retirer le changement de mot de passe des exemptions, décompte `137 + 4 + 3 = 144` **sous réserve de 33-5** ; citer vol de jeton et mot de passe.
- `admin-manual.tex:3278,3417,3498` et `user-manual.tex:3637-3643,3750-3754` : « limite connue, issue #776 » retirée ; le détail nomme `mcp_key_id` ; la ligne de consultation dit désormais le canal, **à partir de la 0.17.5** (les lignes anciennes, non).
- ⚠️ **Le manuel dit-il que l'export du grand livre se voit au journal ?** `user-manual` § traçabilité : à relire, et à dire.
- PDF régénérés et contrôlés aplatis.

## Recoupements avec les autres fiches

`audit_route_registry.rs`, `admin-manual.tex:3157` et les messages `audit-log-action-*` avec **33-5** et **33-1a** (actions ou routes neuves) : recompter à l'intégration. `mcp/outils/rapports.rs` avec 33-4 (autres lignes). `services_motif.rs` : aucune autre.

## Hors périmètre

Connexion, déconnexion, renouvellement : C-e334-11. #431 (attribution par clé perdue au dépôt, autres routes), #379, #434. Le bouton `users::reset_password` d'un Administrateur : déjà tracé ou non, à **constater** au T2 et à dire ; pas à élargir ici.

## Change Log

- 2026-11-06 : fiche créée (Opus 5.5).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
