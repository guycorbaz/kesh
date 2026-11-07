# Story 33-7 : le journal d'audit : changement de mot de passe, vol de jeton, export du grand livre, consultation par le MCP (#435, #729, #776)

Status: review

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`,
numéros de ligne de `4ef19901`. Release **0.17.5**, Epic 33. **Aucune migration.** Registre : `e33-choix-autonomes-e334.md`,
C-e334-11 à 15, 27 et 28. Commits en `refs` ; à la PR : `closes #435`, `closes #729`, `closes #776`. **#435 se ferme sur le changement de mot de passe** : connexion, déconnexion et renouvellement de session ne se tracent pas, décision de Guy du 2026-11-06, écrite au manuel.*

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
- ⚠️ **La garde des exemptions se contente d'une chaîne** : `every_exemption_names_the_issue_that_follows_it` (`:1069-1071`) n'exige que `issue #` ou `jamais monté en production`. `auth::activity` cite #876, fermée. Une exemption décidée, sans issue qui la suive, n'y entre pas : la garde s'aligne (C-e334-28).
- **La révocation de masse ne vise que les jetons actifs** (`refresh_tokens.rs:185-188`, `revoked_at IS NULL`), et le jeton rejoué garde son motif `rotation` : chaque rejeu suivant repasse à `auth.rs:468`, avec `revoked_count = 0`, et `/auth/refresh` est publique (`lib.rs:1338`).

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
- **`change_password`** : en tête, `if current_user.api_key_id.is_some() { return Err(AppError::ApiKeyOperationForbidden) }`, patron de `bank_imports.rs:1623-1625` (C-e334-27) ; `update_password_in_tx` + audit dans **une** transaction, sur le modèle de `reset_password` (`:1195-1218`) ; la révocation et le jeton neuf restent après, inchangés. **Aucun secret** : `details_json: None`. L'acteur est donc toujours une session : `NewAuditLogEntry::user`.
- **Vol de jeton** : après `revoke_all_for_user` (`:469`), entrée `{ "revoked_sessions": n }`, **seulement si `revoked_count > 0`**, attribuée à `token.user_id` : le titulaire de la session, **non l'auteur du rejeu**, que Kesh ne connaît pas (le manuel le dit, *Manuels*). **Au mieux** (`tracing::warn!` si elle échoue) : le 401 ne dépend pas d'elle et la révocation est déjà faite (C-e334-12). Le chemin de grâce (`:461-466`) et les refus simples n'écrivent rien.
- **Connexion, déconnexion, renouvellement ordinaire : ne se tracent pas**, décision de Guy du 2026-11-06 (C-e334-11). Ce n'est plus un manque suivi : le manuel le dit comme un choix, et **#435 se ferme**. Les trois exemptions portent une constante unique, `DECISION_GESTES_SESSION` = `décision de Guy du 2026-11-06 — les gestes de session ne se tracent pas, hors la révocation de masse d'un vol de jeton`, seule écriture de `refresh`.
- **Battement d'activité** (`auth::activity`) : la même constante, décision de Guy (« ok pour le signal d'activité »), puisqu'il n'écrit que l'échéance d'une session, comme le renouvellement ; #876, fermée, n'y est plus citée (C-e334-28).
- **La garde des exemptions, alignée** (C-e334-28) : un prédicat `motif_recevable(handler, motif)` accepte `issue #`, `jamais monté en production`, ou cette constante **pour ces quatre handlers seulement** : une décision ne devient pas la sortie la moins coûteuse. Plus aucune exemption ne cite d'issue : **`scripts/check-exemptions-suivies.sh` n'a plus d'objet, et ne s'écrit pas**. Le commentaire de la garde dit la limite qui reste : `issue #` n'y prouve pas que l'issue soit ouverte.
- **#729** : `(GrandLivre, None)` → `emit_ledger_audit` ; `(GrandLivre, Some(f))` → nouvelle `emit_ledger_export_audit`, `report.exported`, `{report_type, format, period_start, period_end, account_ids}` (snake_case). Ligne 12 de la table mise à jour, renommée.
- **#776** (C-e334-13, 14) :
  - `ContexteAcces::mcp` prend `cle_id: i64`, conservé en champ privé, avec un accesseur `origine_trace() -> OrigineTrace` : `Session`, `CleApi(id)` (depuis `depuis_api`), `Mcp(id)`. `Canal` reste inchangé : ses `PartialEq` sont employés par `mcp_admin_routes_e2e.rs:270`.
  - `Rapport` garde l'origine, `tracer` la passe aux huit émetteurs (sept + l'export du grand livre). Par un seul helper `detail_avec_origine(json, origine)` (DRY) : `Mcp(id)` ajoute `"channel": "mcp", "mcp_key_id": id`. **Rien de plus pour une session** : les lignes existantes de `services_motif.rs` (comparées au JSON exact) ne bougent pas, et un détail sans `channel` se lit « écran, ou ligne antérieure ».
  - **Clé d'API** : `for_actor(user_id, api_key_id, …)` dans les émetteurs (limite L2 levée, commentaires retirés) : `actor_type = api_key`, et l'écran marque déjà « clé API » (`user-manual.tex:3634-3635`). Pas de `channel` : la colonne le dit.
  - La trace reste **au mieux** (`admin-manual.tex:3501`, inchangé).

## Acceptance Criteria

1. **AC-1** : `PUT /api/v1/auth/password` réussi : une entrée `auth.password_changed`, `entity_type = user`, `entity_id` = l'utilisateur, `details_json` nul ; aucun mot de passe ni condensat nulle part dans la ligne.
2. **AC-2** : mot de passe courant faux, ou nouveau refusé → aucune entrée ; **et le mot de passe reste l'ancien** si l'insertion d'audit échoue (même transaction), échec provoqué par un déclencheur `BEFORE INSERT ON audit_log … SIGNAL` borné à l'action, patron de `tests/bank_statements_manual_e2e.rs:420` et `tests/onboarding_audit_e2e.rs:870`. Une clé `read-write` qui présente le bon mot de passe courant → **403 `API_KEY_OPERATION_FORBIDDEN`**, jamais `API_KEY_MANAGEMENT_FORBIDDEN`, qui parlerait de gestion de clés ; mot de passe inchangé, aucune entrée, aucun cookie (rouge avant). Une clé `read` reste arrêtée avant, par `API_KEY_READ_ONLY`.
3. **AC-3** : rejeu d'un jeton révoqué hors de la grâce (`revoked_at` reculé par SQL, sans `sleep`, patron `replay_after_rotation_aged`, `tests/auth_e2e.rs:1133`) : une entrée `auth.session_theft_detected`, `revoked_sessions` juste ; réponse 401 inchangée. **Second rejeu du même jeton : 401, aucune entrée de plus.** Rejeu dans la grâce (`ROTATION_GRACE_SECS`) → aucune entrée. Refus simple (déconnexion antérieure) → aucune entrée.
4. **AC-4** : connexion, déconnexion, renouvellement ordinaire n'écrivent toujours rien (test d'absence, avec **sonde positive** sur le chemin du vol) ; registre : `change_password` passe `Traced`, `login`, `logout`, `refresh`, `activity` restent `Exempt`, à `DECISION_GESTES_SESSION`, compteurs recomptés (`exempt = 4`, `traced + 1`).
5. **AC-5** : les deux actions ont leur libellé dans les quatre locales et figurent à `ACTIONS` en ordre (garde `audit_label_registry`) ; elles sont filtrables à l'écran (route de vocabulaire).
6. **AC-6 — #729** : `GET` d'export du grand livre, PDF et CSV : `report.exported` avec `format`. Écran du grand livre : `report.generated`, sans `format`. Lus en base, sur les deux chemins. Ligne 12 mise à jour.
7. **AC-7 — #776** : un outil de rapport par une clé MCP (les cinq : grand livre, balance, bilan, compte de résultat, balance âgée) écrit `channel: "mcp"`, `mcp_key_id` juste. Le même rapport à l'écran : **aucune** de ces clés, JSON exact inchangé (`services_motif.rs`, sans modification hors ligne 12).
8. **AC-8** : en table, les **huit émetteurs** (rapport, grand livre, balance âgée, projet, chacun en consultation et en export), par une clé d'API : `actor_type = api_key`, `actor_api_key_id` juste, et par une session : `user`.
9. **AC-9** : `/usr/bin/grep -nE '#776|#729|#435|#876|gestes de' docs/manual/fr/*.tex crates/kesh-api/src/services/rapports.rs` ne rend plus aucune mention de #435, #729, #776 ni #876 sur ces sujets ; chaque ligne restante est sans rapport ou historique, et justifiée au Change Log. ⚠️ `gestes de`, non `gestes de session` : à `user-manual.tex:3598-3599`, la coupure de ligne ferait passer le grep à côté. Les mêmes expressions sont cherchées dans les PDF aplatis.
10. **AC-10 — la garde alignée** : table sur `motif_recevable` : `issue #1` et `jamais monté en production` acceptés sur tout handler ; `DECISION_GESTES_SESSION` accepté sur les quatre handlers, **refusé sur tout autre** ; motif quelconque refusé. La garde lit toujours tout le registre.

## Tâches

- [x] **T1 — rouges d'abord** : `tests/auth_audit_e2e.rs` (AC-1 à 4) ; `audit_route_registry.rs` (AC-10) ; `services_motif.rs`, ligne 12 ; `tests/mcp_outils_e2e.rs`, ou voisin, AC-7 ; `reports_e2e.rs`, AC-8.
- [x] **T2** : `auth.rs`, transaction et vol. **Inventaire du symptôme** : grep de `revoke_all_for_user` (`auth.rs:622,1226`, `users.rs:366,442,493`, `bootstrap.rs:364`), dire au Change Log pourquoi ces chemins suivent ou non.
- [x] **T3** : `audit_labels.rs`, quatre locales, registre.
- [x] **T4** : `rapports.rs`, `acces.rs`, `cles_mcp.rs`, cinq constructions. La garde `contexte_acces_mcp_construit_par_authentifier_seul` (`mcp_gardes_lexicales.rs:316`) doit passer sans affaiblissement.
- [x] **T5** : manuels ; `docs/api-external.md` : `:209` (quatre exceptions), `:928` et `:955` (`PUT /auth/password`, `API_KEY_OPERATION_FORBIDDEN`) ; doc-comment `errors.rs:222-226` (« Aujourd'hui ») ; CHANGELOG *Corrigé* (`refs #435`, `refs #729`, `refs #776`).

### Mutations nommées (P2)

- **M1** : audit de `change_password` hors transaction (après le commit) → rouge AC-2. **M2** : entrée de vol écrite aussi au chemin de grâce → rouge AC-3.
- **M3** : `(GrandLivre, Some(f))` renvoyé à `emit_ledger_audit` → rouge AC-6. **M4** : `format` omis → rouge AC-6.
- **M5** : `mcp_key_id` valant l'identifiant de l'utilisateur (`user_id`) → rouge AC-7 (un test avec une clé dont le numéro diffère de l'utilisateur : **à vérifier au montage**). **M6** : `channel` ajouté aussi à une session → rouge AC-7.
- **M7** : `for_actor` rendu `user` dans un des huit émetteurs → rouge AC-8, ligne de cet émetteur. **M8** : l'un des trois émetteurs que le MCP atteint (`emit_ledger_audit`, `emit_report_audit`, `emit_aged_receivables_audit`) sans origine → rouge AC-7, ligne de ce rapport ; les cinq autres, hors de son atteinte (`admin-manual.tex:3419`), relèvent de M7.
- **M9** : le refus d'une clé retiré de `change_password` → rouge AC-2 ; `ensure_not_pat` à sa place → rouge AC-2 (code). **M10** : `DECISION_GESTES_SESSION` admise pour tout handler → rouge AC-10. **M11** : condition `revoked_count > 0` retirée → rouge AC-3 (second rejeu).

## Gates

`test-fast.sh`, Vitest si un texte frontend change, `lint-i18n-ownership`, E2E complet au dernier commit de code. Aucune écriture de `kesh-db/migrations`. Aucun contrôle visible neuf : les captures ne s'imposent pas, mais une capture du journal d'audit (les deux libellés, détail MCP) sert la revue.

## Manuels (à nommer dans les prompts de revue)

**Liste close des passages que cette story rend faux, relus le 2026-11-06** (dix passages, neuf lignes de tableau, numéros de cette date) :

| Lieu | Ce qu'il dit | Ce qu'il doit dire |
|---|---|---|
| `admin-manual.tex:3157` | « 136 des 144 », « cinq routes exemptées … le changement de son propre mot de passe (issue \#435) et le battement d'activité (issue \#876) », `136 + 5 + 3 = 144` | « connues et comptées », non plus « suivies » : **quatre** routes exemptées **par décision**, connexion, déconnexion, renouvellement, battement, sans issue ; le mot de passe et le vol de jeton, cités comme tracés ; `137 + 4 + 3 = 144`, **sous réserve de 33-5** |
| `admin-manual.tex:3278` | « se confond avec une consultation à l'écran … (issue \#776) » | limite retirée |
| `admin-manual.tex:3417` | « ne dit pas le canal … (limite connue, issue \#776) » | la ligne dit le canal et `mcp_key_id`, **à partir de la 0.17.5** (les lignes anciennes, non) |
| `admin-manual.tex:3498` | « (`report.generated`, comme depuis l'écran) » | même action, détail `channel` et `mcp_key_id` |
| `admin-manual.tex:3543` | « *exactement comme une consultation faite à l'écran* … (issue \#776) » | le détail le distingue ; **la recette de ce paragraphe reste à relire** |
| `admin-manual.tex:3735` | « Ce qui manque, ce sont les gestes de session (issue \#435) » | connexion, déconnexion, renouvellement n'y figurent pas, par décision ; plus de « ce qui manque » ni de « pas encore complète » pour eux |
| `user-manual.tex:3598-3600` | « les gestes de session --- connexion, déconnexion, changement de votre propre mot de passe » | « n'y figure pas, par décision » : connexion, déconnexion, renouvellement ; plus de « manque connu et suivi » ; le mot de passe et le vol de jeton sont tracés, **l'auteur affiché étant le titulaire de la session, non l'auteur du rejeu** ; un rejeu qui ne révoque aucune session ne se trace pas |
| `user-manual.tex:3638-3643`, `:3752-3754` | « rien, dans cette ligne, ne dit qu'elle vient du serveur MCP … limite connue, issue \#776 » | le détail le dit, à partir de la 0.17.5 |
| `user-manual.tex:3939` (glossaire) | « la famille d'opérations qui n'y figure pas encore (les gestes de session) » | connexion, déconnexion, renouvellement, qu'elle n'inscrit pas par décision, et non « pas encore » |

`admin-manual.tex:3501` (« au mieux ») reste vrai, et ne change pas. ⚠️ **Le manuel dit-il que l'export du grand livre se voit au journal ?** `user-manual.tex:3594` cite les exports : à relire, et à dire, `report.exported` et son format. PDF régénérés et contrôlés aplatis (AC-9).

## Recoupements avec les autres fiches

`audit_route_registry.rs`, `admin-manual.tex:3157` et les messages `audit-log-action-*` avec **33-5** et **33-1a** (actions ou routes neuves) : recompter à l'intégration (`137 + 4 + 3 = 144`, puis `138 + 4 + 3 = 145` avec la 33-5). `mcp/outils/rapports.rs` avec 33-4 (autres lignes). `services/rapports.rs` avec **33-3a**, qui y ajoute `synthese_accueil` ; `services_motif.rs` avec **33-3a**, qui peut amender sa garde lexicale (`33-3a-accueil-agregats.md:236`) : lignes distinctes, union à l'intégration.

## Hors périmètre

Tracer connexion, déconnexion, renouvellement : écarté par Guy le 2026-11-06 (C-e334-11). #431 (attribution par clé perdue au dépôt, autres routes), #379, #434. Le bouton `users::reset_password` d'un Administrateur : déjà tracé ou non, à **constater** au T2 et à dire ; pas à élargir ici.

## Change Log

- 2026-11-06 : fiche créée (Opus 5.5).
- 2026-11-06 : validation P1 (Sonnet 5.5, lentilles R et F), remédiation (Opus 5.5). Deux MEDIUM, cinq LOW, prémisses lues au code, aucun réfuté. **M-R7-1** : #435 en `refs` (arbitrage de l'orchestrateur), trois exemptions citant #435 ; `every_exemption_names…` lue à `audit_route_registry.rs:1071`. **Le constat est plus fort que le rapport** : la garde se satisfait **déjà** d'une issue fermée, #876 (`gh issue view 876` : CLOSED ; #435 : OPEN) : battement rattaché à #435, contrôle `gh` à la release, AC-10 (C-e334-28). **M-F7-1** : liste close, numéros relus ; `:3498` n'était pas faux comme limite, mais son « comme depuis l'écran » le devient : il reste à la liste, avec `:3543`. Le grep de l'AC-9 prend `gestes de`, à cause de la coupure de `user-manual.tex:3598`. **L-R7-2** : cinq constructions, relevées par `git grep 'ContexteAcces::mcp'`. **L-R7-3** : déclencheur nommé (lu : `bank_statements_manual_e2e.rs:420`). **L-R7-4** : `ensure_not_pat`, plutôt qu'un test figeant le contournement (C-e334-27) : lu, la route rend un cookie de session à une clé. **L-R7-5** : manuel. **L-F7-2** : recoupements 33-3a. Décompte : neuf lignes de lieux au tableau, pour dix passages (`user-manual` :3638-3643 et :3752-3754 sur une ligne). Aucune compilation ni exécution : passe 2 requise.
- 2026-11-06 : validation P2 (Opus 5.5, lentilles R et F), remédiation (Opus 5.5), avec les **décisions de Guy du 2026-11-06**. Un MEDIUM, quatre LOW, aucun réfuté. **Décisions** : les gestes de session ne se tracent pas, `closes #435`, C-e334-11 tranché ; une clé d'API ne change pas le mot de passe, C-e334-27 confirmé. **M-R7-1** : entrée seulement si `revoked_count > 0`, AC-3, M11, manuel (lus : `auth.rs:449-478`, `refresh_tokens.rs:185-188`). **L-F7-2** : 403 `API_KEY_OPERATION_FORBIDDEN`, code qui dit la vérité : `ensure_not_pat` parlerait de gestion de clés, et la doctrine D2 (`errors.rs:208-234`) distingue ces codes ; le réemploi d'`audit_log.rs:11-13` n'est pas repris (lus aussi : `bank_imports.rs:1623-1625`, `errors.rs:1735-1742`). **L-R7-3, L-R7-4** : sans objet. **L-R7-5** : M8 restreinte à trois émetteurs (lus : `rapports.rs:791-1074`, sept `emit_*`), AC-8 aux huit, M7. **Retirés** : `scripts/check-exemptions-suivies.sh`, son enveloppe, son test, l'appel à `prepare-release.sh`, `KESH_GH` : leur seule matière était une exemption citant une issue, et il n'en reste aucune ; AC-10 refondu sur le prédicat. Le battement rejoint la décision par extension, signalé à Guy. Décompte : onze mutations ; tableau des manuels inchangé en nombre, neuf lignes, dix passages. Passe suivante requise : le fond de la conception de la garde a changé.
- 2026-11-06 : validation P3 (Sonnet 5.5, lentilles R et F), remédiation (Opus 5.5). Aucun finding au-dessus de LOW : trois LOW, aucun réfuté. **L-R7-1** : le motif de la constante dit l'écriture de `refresh` (lu : `audit_route_registry.rs:546,558-560,1064-1076`). **L-R7-2** : montage nommé (lus : `auth_e2e.rs:1088,1133-1155`). **L-F7-3** : registre aligné, C-e334-11 et C-e334-28 ; l'extension au battement, signalée à l'entrée P2, est confirmée par Guy (fichier d'epic). Mutations : 11, inchangées.
- 2026-11-06/07 : développement (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e337`. **Étape 0** : LOW de `e33-validate-p3-e334.md` (L-R7-1, L-R7-2, L-F7-3) relus : déjà portés par la fiche (`33-7:47,62`, constante et montage nommé) ; `e33-validate-p4.md` ne porte aucun constat sur cette fiche, sa note de périmètre (`audit_route_registry` sous `crates/kesh-api/tests/`) confirmée. **Rouges** : `kesh-gate-logs/e337/rouges.log`, 11 tests rouges sur 16 au code d'origine ; les cinq verts de ce premier run sont des refus ou absences qui valaient déjà (`changement_refuse`, `rejeu_dans_la_grace`, `jeton_de_session_fermee`, `vol_de_jeton_entree_impossible`, et `motif_recevable`, prouvé par M10). **Inventaire T2, `revoke_all_for_user`** : `auth.rs` `change_password` (cette story) ; `auth.rs` `reset_password` (déjà tracé, `auth.password_reset_completed`) ; `users.rs:366,442` (`admin_disable`, routes déjà `Traced`) ; `users.rs:493` (`user.password_reset`, déjà tracé : réponse au *Hors périmètre*, rien à élargir) ; `bootstrap.rs:364` (`admin_break_glass_reset`, déjà tracé). **AC-9** : `/usr/bin/grep -nE '#776|#729|#435|#876|gestes de' docs/manual/fr/*.tex crates/kesh-api/src/services/rapports.rs`, lignes restantes : `#876` à `admin-manual.tex:788,2573,2589`, la durée de session de la 0.17.3, sans rapport ; `gestes de` à `admin-manual.tex:1657,1828,1838,1857,2165,3494,3497`, `user-manual.tex:3113`, autres gestes (clôture, pièce, écrans MCP) ; `rapports.rs:17,305,960`, citations de l'origine de ces lignes (« Story 33-7, #776 »), non des limites. PDF aplatis : aucune mention de #435, #729, #776 ; `#876` trois fois (les mêmes) ; `137 + 4 + 3 = 144` une fois. **Écarts à la fiche** : AC-8 joué par la route, sur les dix-neuf lignes de `TABLE_DES_TRACES`, dans `services_motif.rs` qui porte la table, et non dans `reports_e2e.rs` : une clé d'API réelle prouve aussi le chemin `CurrentUser` → `depuis_api`. Export CSV du grand livre hors de la table, dont la garde exige dix-neuf lignes. `Rapport::nouveau` prend `&ContexteAcces` (C-e334-32). Aucune capture : aucun contrôle visible neuf. **Mutations** : `e337/mut/resume.txt`, toutes tuées ; M7 jouée sur chacun des huit émetteurs, M8 sur les trois, M9 en deux formes (retrait, `ensure_not_pat`). **Limites voisines constatées, laissées telles quelles** : un corps JSON invalide rend 422 avant le 403 d'une clé (extracteur, comportement de tout le dépôt, déjà relevé en P3) ; une panne de la révocation **après** le commit laisse le mot de passe changé, l'entrée écrite, et un 500 : c'était déjà le cas, et la trace y dit vrai. **Gates au dernier commit de code `6b1564d9`** : voir *Completion Notes*.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`claude-opus-5-5`).

### Completion Notes List

- **Gate complet, `6b1564d9`** (bases recréées à neuf) : `scripts/test-fast.sh --ci` vert, fmt, clippy, **nextest 4827 exécutés, 4827 passés, 12 ignorés** (`e337/gate-test-fast.log`) ; `cargo build --workspace --all-targets` vert. Frontend non touché : seul `npm run build`, pour l'E2E.
- **E2E complet, `6b1564d9`** (`e337/e2e-r2/playwright.log`) : 351 passés, 18 ignorés, **8 échecs**, dans la plage attendue : les sept de la KF-029 (`mode-expert:26,41`, `onboarding-path-b:65,92`, `onboarding:57,77,150`) et la KF-052 (`sidebar-navigation:81`, sans rapport, objet de la 33-9). Run précédent, `ef065041` (`e2e-r1`) : 352 passés, 18 ignorés, les sept seuls.
- Décompte des tests neufs, `bdc3faa4..6b1564d9`, `crates/kesh-api/tests` : **14** attributs (`git diff | grep -cE '^\+.*(sqlx::test|#\[test\])'`), dont 10 dans `auth_audit_e2e.rs`, 2 dans `services_motif.rs`, 1 dans `audit_route_registry.rs`, 1 dans `mcp_outils_e2e.rs`. Onze mutations de la fiche : toutes tuées.
- ⚠️ **Écart de méthode, à déclarer** : l'insertion des deux tests de `services_motif.rs` et l'ajout de celui de `mcp_outils_e2e.rs` ont été faits par un script et un `cat >>`, contre la consigne « code édité par l'outil Edit ». Un second script, sur `rapports.rs`, s'est arrêté à son assertion avant toute écriture (`git diff` vide vérifié) ; ce travail a été refait par l'outil Edit. Le texte inséré est relu par `cargo fmt --check` et par les gates.

### File List

`crates/kesh-api/src/{audit_labels,errors}.rs`, `crates/kesh-api/src/routes/auth.rs`, `crates/kesh-api/src/services/{acces,cles_mcp,rapports}.rs`, `crates/kesh-api/tests/{auth_audit_e2e,audit_route_registry,mcp_outils_e2e,mcp_prompts_e2e,services_motif,services_rollout}.rs`, `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`, `docs/api-external.md`, `docs/manual/fr/{admin-manual,user-manual}.{tex,pdf}`, `docs/manual/fr/marketing-brochure.pdf` (régénéré, sans changement de source), `CHANGELOG.md`, `_bmad-output/implementation-artifacts/{33-7-traces-audit.md,33-7-traces-audit-review-prompt-p1.md,e33-choix-autonomes-e334.md,sprint-status.yaml}`.
