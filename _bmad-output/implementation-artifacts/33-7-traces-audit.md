# Story 33-7 : le journal d'audit : changement de mot de passe, vol de jeton, export du grand livre, consultation par le MCP (#435, #729, #776)

Status: ready-for-dev

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`,
numéros de ligne de `4ef19901`. Release **0.17.5**, Epic 33. **Aucune migration.** Registre : `e33-choix-autonomes-e334.md`,
C-e334-11 à 15, 27 et 28. Commits en `refs` ; à la PR : `refs #435`, `closes #729`, `closes #776`. ⚠️ **#435 ne se ferme pas** : connexion, déconnexion et renouvellement y restent tracés, C-e334-11 étant à relire par Guy.*

## Story

En tant qu'Administrateur ou réviseur, je veux que le journal d'audit dise qu'un mot de passe a été changé, qu'un vol de jeton a révoqué des sessions,
qu'un grand livre a été **exporté** et dans quel format, et qu'un rapport a été lu **par un agent** ; pour que la piste de contrôle ne se taise plus
sur ces gestes, et ne les confonde plus avec une simple consultation.

## Ce que le code dit (prémisses établies)

**#435.**
- ⚠️ **Lignes de l'issue périmées** : `change_password` est à `routes/auth.rs:582-670` ; la révocation pour vol à `:468-478` ; les audits du parcours
  « oublié » à `:1033` et `:1205`.
- `change_password` : `users::update_password` sur le pool (`:618`), révocation (`:622`), jeton neuf (`:633`). `users::update_password_in_tx` existe (reset, `:1195`).
- ⚠️ **Lu : la route est ouverte à une clé d'API.** Elle est dans le routeur authentifié (`lib.rs:1127`), hors `admin_routes`, et le handler n'appelle pas `ensure_not_pat` (`routes/api_keys.rs:95`, employé à `auth.rs:726` pour une autre route). Une clé qui connaît le mot de passe courant le change, et reçoit en retour un cookie de session et un jeton neufs (`auth.rs:633`). Limite courte, levée ici (C-e334-27).
- Registre : `tests/audit_route_registry.rs:542,546,558-560`, `Exempt("issue #…")`, décompte `:1005-1007` (5 exemptions).
- ⚠️ **La garde des exemptions se contente d'une chaîne** : `every_exemption_names_the_issue_that_follows_it` (`:1071`) n'exige que le texte `issue #`. **Elle est déjà satisfaite par une issue fermée** : `auth::activity` cite #876, fermée à ce jour (lu par `gh issue view`), et le battement n'a plus de suivi. Un test hors réseau ne peut pas lire un état GitHub : le contrôle se pose au seul moment où `gh` est de droit, la release (C-e334-28).

**#729.**
- `services/rapports.rs:413-428` : `(DetailsTrace::GrandLivre, _)` → `emit_ledger_audit`, qui écrit `report.generated` sans format (`:857-893`) — pour les deux usages. La
  table de `tests/services_motif.rs:488-500` (ligne 12) épingle ce comportement, et annonce elle-même que son correctif la modifiera.
- Autres mentions à retirer : `rapports.rs:78-79` (« sauf le grand livre, #729 »), `:368-369`, `:414`.

**#776.**
- `Rapport::nouveau` ne reçoit que `user_id` (`rapports.rs:302`) ; `ContexteAcces::mcp` n'a pas l'identifiant de clé (`services/acces.rs:89`), que
  `CleMcpAuthentifiee.cle_id` porte pourtant à la création (`services/cles_mcp.rs:108-115`). **Cinq sites le construisent** : un en production (`cles_mcp.rs:110`), quatre en tests (`services_motif.rs:854,862`, `mcp_prompts_e2e.rs:297`, `services_rollout.rs:433`) ; les chaînes factices de `mcp_gardes_lexicales.rs:322,518` ne se compilent pas et n'ont pas à changer.
- ⚠️ **Prémisse corrigée — lignes du manuel** : `user-manual.tex:3153-3157` cité par l'issue est aujourd'hui à **`:3638-3643`**, plus **`:3752-3754`**, `admin-manual.tex:3278`, `:3417`, `:3498`, `:3543` (liste close sous *Manuels*).
- **Limite voisine L2, courte** : les sept `emit_*` écrivent `NewAuditLogEntry::user` même par une clé d'API (`rapports.rs:940-944`). `CurrentUser.api_key_id` existe, et `for_actor` aussi.

**Vocabulaire public.** `audit_labels.rs` est la source unique (`ACTIONS`, triée, `:86-87,193-194`) ; `tests/audit_label_registry.rs` la garde ; les libellés vivent dans les quatre locales
(`audit-log-action-*`, `fr-CH/messages.ftl:3157-3158,3176-3177`). L'écran déplie le détail tel quel : une clé de plus s'y lit sans code.

## Conception

- **Actions neuves** : `auth.password_changed` (« Mot de passe modifié »), `auth.session_theft_detected` (« Vol de jeton détecté, sessions révoquées »). Entité `user`.
- **`change_password`** : `ensure_not_pat` en tête (C-e334-27) ; `update_password_in_tx` + audit dans **une** transaction, sur le modèle de `reset_password` (`:1195-1218`) ; la révocation et le jeton neuf restent après, inchangés. **Aucun secret** : `details_json: None`. L'acteur est donc toujours une session : `NewAuditLogEntry::user`.
- **Vol de jeton** : après `revoke_all_for_user` (`:469`), entrée `{ "revoked_sessions": n }`, attribuée à `token.user_id` : le titulaire de la session, **non l'auteur du rejeu**, que Kesh ne connaît pas (le manuel le dit, *Manuels*). **Au mieux** (`tracing::warn!` si elle échoue) : le 401 ne dépend pas d'elle et la révocation est déjà faite (C-e334-12). Le chemin de grâce (`:461-466`) et les refus simples n'écrivent rien.
- ⚠️ **Connexion, déconnexion, renouvellement ordinaire : pas de trace dans cette story, et le sujet reste ouvert** (C-e334-11, à relire par Guy). #435 pose elle-même que le volume « mérite un arbitrage » : il n'est pas rendu. **#435 reste ouverte** et porte ce volet ; l'orchestrateur y écrit le reste à arbitrer. Les trois exemptions gardent leur suivi, motif `issue #435 — login, logout, refresh, activity : le volume de la trace des gestes de session reste à arbitrer`.
- **Battement d'activité** (`auth::activity`) : même motif, #435 au lieu de #876 fermée (C-e334-28).
- **La garde d'une issue fermée** (C-e334-28) : `scripts/check-exemptions-suivies.sh` relève les numéros de `Exempt("issue #N` dans `audit_route_registry.rs` et lit leur état par `gh` (injection `KESH_GH`, patron `KESH_DOCKER`) ; une issue fermée fait **refuser** en la nommant ; `gh` absent ou en échec **avertit**, sans bloquer. `scripts/prepare-release.sh` l'appelle au pré-vol (⛔ les agents n'exécutent pas ce script, ils lancent le contrôle seul). La garde Rust reste hors réseau ; son commentaire dit cette limite.
- **#729** : `(GrandLivre, None)` → `emit_ledger_audit` ; `(GrandLivre, Some(f))` → nouvelle `emit_ledger_export_audit`, `report.exported`, `{report_type, format, period_start, period_end, account_ids}` (snake_case). Ligne 12 de la table mise à jour, renommée.
- **#776** (C-e334-13, 14) :
  - `ContexteAcces::mcp` prend `cle_id: i64`, conservé en champ privé, avec un accesseur `origine_trace() -> OrigineTrace` : `Session`, `CleApi(id)` (depuis `depuis_api`), `Mcp(id)`. `Canal` reste inchangé : ses `PartialEq` sont employés par `mcp_admin_routes_e2e.rs:270`.
  - `Rapport` garde l'origine, `tracer` la passe aux huit émetteurs (sept + l'export du grand livre). Par un seul helper `detail_avec_origine(json, origine)` (DRY) : `Mcp(id)` ajoute `"channel": "mcp", "mcp_key_id": id`. **Rien de plus pour une session** : les lignes existantes de `services_motif.rs` (comparées au JSON exact) ne bougent pas, et un détail sans `channel` se lit « écran, ou ligne antérieure ».
  - **Clé d'API** : `for_actor(user_id, api_key_id, …)` dans les émetteurs (limite L2 levée, commentaires retirés) : `actor_type = api_key`, et l'écran marque déjà « clé API » (`user-manual.tex:3634-3635`). Pas de `channel` : la colonne le dit.
  - La trace reste **au mieux** (`admin-manual.tex:3501`, inchangé).

## Acceptance Criteria

1. **AC-1** : `PUT /api/v1/auth/password` réussi : une entrée `auth.password_changed`, `entity_type = user`, `entity_id` = l'utilisateur, `details_json` nul ; aucun mot de passe ni condensat nulle part dans la ligne.
2. **AC-2** : mot de passe courant faux, ou nouveau refusé → aucune entrée ; **et le mot de passe reste l'ancien** si l'insertion d'audit échoue (même transaction), échec provoqué par un déclencheur `BEFORE INSERT ON audit_log … SIGNAL` borné à l'action, patron de `tests/bank_statements_manual_e2e.rs:420` et `tests/onboarding_audit_e2e.rs:870`. Une clé d'API qui présente le bon mot de passe courant → refus de `ensure_not_pat`, mot de passe inchangé, aucune entrée, aucun cookie (rouge avant).
3. **AC-3** : rejeu d'un jeton révoqué hors de la grâce : une entrée `auth.session_theft_detected`, `revoked_sessions` juste ; réponse 401 inchangée. Rejeu dans la grâce (`ROTATION_GRACE_SECS`) → aucune entrée. Refus simple (déconnexion antérieure) → aucune entrée.
4. **AC-4** : connexion, déconnexion, renouvellement ordinaire n'écrivent toujours rien (test d'absence, avec **sonde positive** sur le chemin du vol) ; registre : `change_password` passe `Traced`, `login`, `logout`, `refresh`, `activity` restent `Exempt` au motif neuf, citant #435, compteurs recomptés (`exempt = 4`, `traced + 1`).
5. **AC-5** : les deux actions ont leur libellé dans les quatre locales et figurent à `ACTIONS` en ordre (garde `audit_label_registry`) ; elles sont filtrables à l'écran (route de vocabulaire).
6. **AC-6 — #729** : `GET` d'export du grand livre, PDF et CSV : `report.exported` avec `format`. Écran du grand livre : `report.generated`, sans `format`. Lus en base, sur les deux chemins. Ligne 12 mise à jour.
7. **AC-7 — #776** : un outil de rapport par une clé MCP (les cinq : grand livre, balance, bilan, compte de résultat, balance âgée) écrit `channel: "mcp"`, `mcp_key_id` juste. Le même rapport à l'écran : **aucune** de ces clés, JSON exact inchangé (`services_motif.rs`, sans modification hors ligne 12).
8. **AC-8** : même rapport, par une clé d'API : `actor_type = api_key`, `actor_api_key_id` juste, et par une session : `user`. Consultation et export.
9. **AC-9** : `/usr/bin/grep -nE '#776|#729|#435|#876|gestes de' docs/manual/fr/*.tex crates/kesh-api/src/services/rapports.rs` ne rend plus que ce qui est historique, ou le volet resté ouvert de #435, et chaque ligne restante est justifiée au Change Log. ⚠️ `gestes de`, non `gestes de session` : à `user-manual.tex:3598-3599`, la coupure de ligne ferait passer le grep à côté. Les mêmes expressions sont cherchées dans les PDF aplatis.
10. **AC-10 — la garde d'une issue fermée** : `scripts/tests/check-exemptions-suivies.test.sh` (`gh` factice), lancé par une enveloppe Rust sur le modèle de `crates/kesh-db/tests/check_dev_grants_script.rs`, donc au gate et en CI : toutes ouvertes → 0 ; une fermée → non nul, numéro nommé ; `gh` absent → 0 et avertissement ; **sonde** : registre factice sans aucune exemption → message « aucune issue lue », non nul, pour qu'un motif reformulé ne rende pas le contrôle muet. `prepare-release.test.sh` reste sans réseau (`KESH_GH` factice).

## Tâches

- [ ] **T1 — rouges d'abord** : `tests/auth_audit_e2e.rs` (AC-1 à 4) ; `scripts/tests/check-exemptions-suivies.test.sh` (AC-10) ; `services_motif.rs`, ligne 12 ; `tests/mcp_outils_e2e.rs`, ou voisin, AC-7 ; `reports_e2e.rs`, AC-8.
- [ ] **T2** : `auth.rs`, transaction et vol. **Inventaire du symptôme** : grep de `revoke_all_for_user` (reset, break-glass de `bootstrap.rs`), dire au Change Log pourquoi ces chemins suivent ou non.
- [ ] **T3** : `audit_labels.rs`, quatre locales, registre.
- [ ] **T4** : `rapports.rs`, `acces.rs`, `cles_mcp.rs`, cinq constructions. La garde `contexte_acces_mcp_construit_par_authentifier_seul` (`mcp_gardes_lexicales.rs:316`) doit passer sans affaiblissement.
- [ ] **T5** : `scripts/check-exemptions-suivies.sh` et son appel à `prepare-release.sh` ; manuels ; `docs/api-external.md` (`PUT /auth/password` refusé à une clé) ; CHANGELOG *Corrigé* (`refs #435`, `refs #729`, `refs #776`).

### Mutations nommées (P2)

- **M1** : audit de `change_password` hors transaction (après le commit) → rouge AC-2. **M2** : entrée de vol écrite aussi au chemin de grâce → rouge AC-3.
- **M3** : `(GrandLivre, Some(f))` renvoyé à `emit_ledger_audit` → rouge AC-6. **M4** : `format` omis → rouge AC-6.
- **M5** : `mcp_key_id` valant l'identifiant de l'utilisateur (`user_id`) → rouge AC-7 (un test avec une clé dont le numéro diffère de l'utilisateur : **à vérifier au montage**). **M6** : `channel` ajouté aussi à une session → rouge AC-7.
- **M7** : `for_actor` rendu `user` → rouge AC-8. **M8** : un seul des huit émetteurs sans origine → rouge AC-7, ligne de ce rapport.
- **M9** : `ensure_not_pat` retiré de `change_password` → rouge AC-2. **M10** : le script accepte une issue fermée → rouge AC-10.

## Gates

`test-fast.sh`, Vitest si un texte frontend change, `lint-i18n-ownership`, E2E complet au dernier commit de code. Aucune écriture de `kesh-db/migrations`. Aucun contrôle visible neuf : les captures ne s'imposent pas, mais une capture du journal d'audit (les deux libellés, détail MCP) sert la revue.

## Manuels (à nommer dans les prompts de revue)

**Liste close des passages que cette story rend faux, relus le 2026-11-06** (dix passages, neuf lignes de tableau, numéros de cette date) :

| Lieu | Ce qu'il dit | Ce qu'il doit dire |
|---|---|---|
| `admin-manual.tex:3157` | « 136 des 144 », « cinq routes exemptées … le changement de son propre mot de passe (issue \#435) et le battement d'activité (issue \#876) », `136 + 5 + 3 = 144` | **quatre** routes exemptées, connexion, déconnexion, renouvellement, battement, toutes **issue \#435** ; le mot de passe et le vol de jeton, cités comme tracés ; `137 + 4 + 3 = 144`, **sous réserve de 33-5** |
| `admin-manual.tex:3278` | « se confond avec une consultation à l'écran … (issue \#776) » | limite retirée |
| `admin-manual.tex:3417` | « ne dit pas le canal … (limite connue, issue \#776) » | la ligne dit le canal et `mcp_key_id`, **à partir de la 0.17.5** (les lignes anciennes, non) |
| `admin-manual.tex:3498` | « (`report.generated`, comme depuis l'écran) » | même action, détail `channel` et `mcp_key_id` |
| `admin-manual.tex:3543` | « *exactement comme une consultation faite à l'écran* … (issue \#776) » | le détail le distingue ; **la recette de ce paragraphe reste à relire** |
| `admin-manual.tex:3735` | « Ce qui manque, ce sont les gestes de session (issue \#435) » | ce qui manque : connexion, déconnexion, renouvellement (issue \#435) |
| `user-manual.tex:3598-3600` | « les gestes de session --- connexion, déconnexion, changement de votre propre mot de passe » | sans le mot de passe ; le vol de jeton est tracé, **et l'auteur affiché est le titulaire de la session, non l'auteur du rejeu** (L-R7-5) |
| `user-manual.tex:3638-3643`, `:3752-3754` | « rien, dans cette ligne, ne dit qu'elle vient du serveur MCP … limite connue, issue \#776 » | le détail le dit, à partir de la 0.17.5 |
| `user-manual.tex:3939` (glossaire) | « la famille d'opérations qui n'y figure pas encore (les gestes de session) » | connexion, déconnexion, renouvellement |

`admin-manual.tex:3501` (« au mieux ») reste vrai, et ne change pas. ⚠️ **Le manuel dit-il que l'export du grand livre se voit au journal ?** `user-manual.tex:3594` cite les exports : à relire, et à dire, `report.exported` et son format. PDF régénérés et contrôlés aplatis (AC-9).

## Recoupements avec les autres fiches

`audit_route_registry.rs`, `admin-manual.tex:3157` et les messages `audit-log-action-*` avec **33-5** et **33-1a** (actions ou routes neuves) : recompter à l'intégration (`137 + 4 + 3 = 144`, puis `138 + 4 + 3 = 145` avec la 33-5). `mcp/outils/rapports.rs` avec 33-4 (autres lignes). `services/rapports.rs` avec **33-3a**, qui y ajoute `synthese_accueil` ; `services_motif.rs` avec **33-3a**, qui peut amender sa garde lexicale (`33-3a-accueil-agregats.md:236`) : lignes distinctes, union à l'intégration. `scripts/prepare-release.sh` : aucune autre fiche.

## Hors périmètre

Tracer connexion, déconnexion, renouvellement : C-e334-11, suivi par #435, qui reste ouverte. #431 (attribution par clé perdue au dépôt, autres routes), #379, #434. Le bouton `users::reset_password` d'un Administrateur : déjà tracé ou non, à **constater** au T2 et à dire ; pas à élargir ici.

## Change Log

- 2026-11-06 : fiche créée (Opus 5.5).
- 2026-11-06 : validation P1 (Sonnet 5.5, lentilles R et F), remédiation (Opus 5.5). Deux MEDIUM, cinq LOW, prémisses lues au code, aucun réfuté. **M-R7-1** : #435 en `refs` (arbitrage de l'orchestrateur), trois exemptions citant #435 ; `every_exemption_names…` lue à `audit_route_registry.rs:1071`. **Le constat est plus fort que le rapport** : la garde se satisfait **déjà** d'une issue fermée, #876 (`gh issue view 876` : CLOSED ; #435 : OPEN) : battement rattaché à #435, contrôle `gh` à la release, AC-10 (C-e334-28). **M-F7-1** : liste close, numéros relus ; `:3498` n'était pas faux comme limite, mais son « comme depuis l'écran » le devient : il reste à la liste, avec `:3543`. Le grep de l'AC-9 prend `gestes de`, à cause de la coupure de `user-manual.tex:3598`. **L-R7-2** : cinq constructions, relevées par `git grep 'ContexteAcces::mcp'`. **L-R7-3** : déclencheur nommé (lu : `bank_statements_manual_e2e.rs:420`). **L-R7-4** : `ensure_not_pat`, plutôt qu'un test figeant le contournement (C-e334-27) : lu, la route rend un cookie de session à une clé. **L-R7-5** : manuel. **L-F7-2** : recoupements 33-3a. Décompte : neuf lignes de lieux au tableau, pour dix passages (`user-manual` :3638-3643 et :3752-3754 sur une ligne). Aucune compilation ni exécution : passe 2 requise.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
