# Prompt — revue de code P1 de la Story 32-5c (les routes de gestion des clés et des réglages du serveur MCP), trois lentilles, Sonnet

Tu relis le développement de la **partie c de la Story 32-5** de Kesh (comptabilité suisse, Rust/Axum, MariaDB) :
*six routes d'administration — `GET`/`POST /api/v1/admin/mcp/keys`, `DELETE /api/v1/admin/mcp/keys/{id}`,
`PUT /api/v1/admin/mcp/keys/{id}/send-validation`, `GET`/`PUT /api/v1/admin/mcp/settings` — dans le bloc
`admin_routes` ; leur validation traduite dans les quatre locales ; quatre actions et deux types d'entité au journal
d'audit ; `ipnet` en dépendance directe ; les gardes et compteurs d'`admin_pat_denied_e2e` et d'`audit_route_registry` ;
`docs/api-external.md` et le compte des routes tracées au manuel d'administration*.
Worktree `/home/gcorbaz/devel/kesh-32-5c`, branche `story/32-5c`, partie de `fa5b3468` (`main`, 32-5a, 32-5b et 32-5d
fusionnées). Le diff à relire, **aplati** (un seul diff, pas la suite des commits) :
`git -C /home/gcorbaz/devel/kesh-32-5c diff fa5b3468...HEAD`. Langue du rapport : français. Modèle : **Sonnet**.
Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Seule la partie c est en revue.** Les parties a, b et d sont fusionnées et revues ; un défaut qu'elles portent
n'est un finding que si la partie c le rend atteignable ou le rend faux. Le **sens** des réglages (filtre d'adresse,
mandataires, désactivation → 404) et l'endpoint `/mcp` sont la 32-6 ; l'écran et la section *Le serveur MCP* du manuel
sont la 32-7 : leur absence n'est pas un finding.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-5-cles-et-reglages-mcp.md` — *Ce que le code dit* (faits 4, 5, 12, 13, 18),
*Partie C* (AC-c1 à AC-c7), tâches T-c0 à T-c4, mutations Mc1 à Mc6, *Ce que les gardes ne voient pas*, *Fichiers
touchés (prévus)* pour la 32-5c, § *Pour la 32-5c* (B1, B3, E3, E5, B2, A2), et le **Dev Agent Record** de la partie c
(étape 0, notes, décomptes) — puis le registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-325.md`
(C-32-5-7, 8, 17, 18, 19, 30, 31, 37, 43, et les entrées du développement de la 32-5c, C-32-5-51 à C-32-5-60). Le Dev
Agent Record déclare des gates, des mutations et un E2E : **ne les prends pas pour argent comptant**, recoupe-les avec
les journaux `/home/gcorbaz/devel/kesh-gate-logs/325c-*.log` (lecture seule). Aucun contrôle visible ne change : pas de
capture attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`) : compare par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement. Un fichier créé
  par erreur se **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`,
  ni aucun script de `kesh-gate-logs` ou d'un scratchpad ; ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande,
  `cargo fmt` compris), ni `rustc`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.
- **Données réelles** : aucune ; les fixtures sont synthétiques. Ne lis rien sous `~/kesh-test/`.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. `crates/kesh-api/src/lib.rs` : les six couples sont-ils **dans** le bloc borné par `KESH-ADMIN-ROUTES-BEGIN`/`END`,
   **au-dessus** des deux `route_layer` ? Un chemin entre-t-il en conflit avec une route existante (même chemin,
   méthode en double, chemin capturé par un autre `{param}`) ? Le commentaire « exige exactement 39 » est-il vrai ?
2. `crates/kesh-api/src/routes/mcp_keys.rs` : le secret (`key`) ne sort-il **que** de la réponse 201 ? Le condensat
   (`key_hash`) apparaît-il dans une réponse, un détail d'audit, un message d'erreur, un `Debug` journalisé ? L'ordre
   « valider tout → 404 → écrire → auditer → valider » laisse-t-il une écriture sans audit, ou un audit sans écriture
   (transaction) ? `validate_name` (rognage par `kesh_core::text::is_invisible`) : que devient un nom de 255
   caractères suivi d'un blanc ; un nom fait de caractères de contrôle intérieurs ? `validate_scopes` : le message d'une
   portée inconnue peut-il porter une chaîne arbitrairement longue fournie par le client ? `#[serde(flatten)]` de
   `CreateMcpKeyResponse` : un champ peut-il s'y dédoubler ? Le `PUT` de politique relit la clé **après** le commit :
   que rend-il si elle a été révoquée entre-temps ?
3. `crates/kesh-api/src/routes/mcp_server_settings.rs` : `canonical_network` (adresse puis réseau, refus des bits
   d'hôte, forme `IpNet::to_string`) — IPv4 mappée en IPv6 (`::ffff:10.0.0.1`), zone IPv6 (`fe80::1%eth0`), préfixe
   `/0`, zéros de tête (`010.0.0.1`), blancs intérieurs ; `canonical_origin` — majuscules, port par défaut, port `0`,
   nom international, IPv6 entre crochets, `http://h:` (port vide), schéma `HTTP` ; `canonical_list` — la borne de 64
   avant ou après dédoublonnage, la borne de 255 avant ou après rognage. L'« avant » de l'audit lu **hors** transaction
   (doc-comment, C-32-5-54) : peut-il être faux pour un client honnête ?
4. Les tests (`mcp_admin_routes_e2e.rs`, `i18n_e2e.rs`, les tests unitaires des deux modules) : un test peut-il passer
   **à vide** — un 400 obtenu pour une autre raison que celle visée (corps mal formé → 422 ou 400 de l'extracteur ?),
   un 401 de garde obtenu parce que le chemin n'existe pas, un « rien d'écrit » vrai parce que rien n'aurait pu l'être,
   une recherche du secret dans un texte qui ne pourrait pas le contenir (`CAST(details_json AS CHAR)`, `LIKE` sur un
   jeton contenant `_`) ? Les témoins positifs existent-ils ?

## Lentille E — Edge cases

1. **Gardes (AC-c2)** : pour chacun des six couples — Comptable → `403 FORBIDDEN` ; PAT `read-write`, même d'un
   Administrateur → `403 API_KEY_ADMIN_FORBIDDEN` ; PAT `read` sur une méthode mutante → `API_KEY_READ_ONLY` ; clé MCP →
   401 ; sans authentification → 401 ; en-tête **et** cookie (`kesh_access_token` portant une clé MCP) ? Les deux `GET`
   sans `ensure_not_pat` sont-ils bien comptés parmi les `HEAD` discriminants (6 → 8) d'`admin_pat_denied_e2e` ?
2. **Validation (AC-c3, AC-c4)** : corps sans `scopes`, `scopes` non tableau, `name` absent, `expiresAt` mal formé ou
   sans fuseau, `sendValidation: null`, `version` hors `i32`, `version` négative au `PUT` de réglages, liste de 64
   entrées dont des doublons, entrée de 256 caractères en UTF-8 multi-octets. Chaque refus est-il un 400 **traduit**
   (`t`/`t_args`, repli égal à la valeur fr-CH, garde G9 de `textes_coherents.rs`), rien d'écrit, aucun audit ? Les
   18 clés neuves existent-elles dans les **quatre** catalogues, avec les mêmes variables Fluent (`{ $scope }`,
   `{ $entry }`, `{ $expected }`) ?
3. **Verrou optimiste** : `version: 0` alors qu'une ligne existe (409, jamais d'écrasement — Mc4 joué sur le dépôt,
   C-32-5-54) ; une ligne existante à `version = 0` (B3, C-32-5-53 : la décision de ne pas migrer tient-elle ? l'état
   est-il vraiment inatteignable par Kesh — import d'une sauvegarde fabriquée compris ?) ; révocation d'une clé déjà
   révoquée ; politique d'une clé révoquée (C-32-5-30) ; clé d'une autre société (404 indiscernable d'une clé absente) ;
   deux révocations concurrentes.
4. **AC-c6** : une clé créée par la route s'authentifie-t-elle avec **exactement** ses portées et sa politique
   (`services::cles_mcp::authentifier`) ; la politique changée est-elle rendue ; révoquée, n'authentifie-t-elle plus ?
   Une clé créée avec une expiration future s'authentifie-t-elle (le fuseau de `expiresAt` → `NaiveDateTime` UTC,
   contre `NOW(3)` de la session MariaDB) ?
5. **Les mutations** Mc1 à Mc6, mc7 et mc8 (`325c-mutations.log`, `325c-mut-*.log`) : chacune tuée par une **assertion
   nommée** ? Le motif muté est-il celui que décrit la fiche (Mc1 : la création montée Comptable ; Mc4 : sur le dépôt,
   non sur la route — est-ce légitime ?) ? Une mutation plausible **non jouée** survivrait-elle — `mcp_key.revoked`
   écrit hors de la transaction, la politique par défaut `exempt`, les doublons de portées refusés au lieu d'être
   normalisés, une origine stockée sans minuscules, le pré-contrôle 404 retiré (la révocation d'une clé d'une autre
   société rendrait-elle alors 409 au lieu de 404 ?) ? Raisonne sur le code, n'exécute rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-c1 à AC-c7, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve. En
particulier :

- **AC-c1, AC-c2** : les six routes ; les compteurs à **chaque** site — `lib.rs` (« 39 »), `admin_pat_denied_e2e.rs`
  (doc d'`ADMIN_COUPLES`, les « 39 » des commentaires, `(10, 29)` et sa généalogie, la doc « Les 10 `get` … les 29 …
  vingt-neuf-là », `discriminating` 8), `audit_route_registry.rs` (doc d'en-tête 128 / 131, `LIB_ROUTES.len()` 128,
  `traced` 122 et sa généalogie, registre 131, `sans_ecriture` 103 et son message « 102 routes de lib.rs »,
  `tout.len()` 131). Greper la **valeur** et non la phrase (`git grep -nE '\b(33|124|127|118|99)\b'` sur les deux
  fichiers de test et `lib.rs`, trier à la main les occurrences légitimes).
- **AC-c3, AC-c4, AC-c5** : messages traduits, quatre locales, G9 étendue (deux fichiers, 12 clés) ; `Cargo.lock` ne
  gagne qu'`ipnet` dans les dépendances de `kesh-api`, à la version 2.12.0 (`git diff fa5b3468...HEAD -- Cargo.lock`) ;
  les deux types et quatre actions à `audit_labels.rs` (triés), libellés dans les quatre locales,
  `audit_label_registry.rs` vert au journal ; test qui cherche le jeton **et** son condensat dans `details_json`.
- **AC-c7 — `docs/api-external.md` et le manuel, PDF compris** : § 2 *Format de la clé* (`docs/api-external.md:27-37`)
  dit-il qu'une clé d'API est `kesh_pat_…` et qu'une clé `kesh_mcp_…` n'en est pas une et reçoit 401 à `/api/v1` — et
  **rien d'autre** ? Chaque phrase est-elle vraie **à HEAD** (« Depuis la 0.16.0, l'Administrateur peut créer… » : par
  quel moyen, sans écran ? « une clé d'API n'ouvre pas le serveur MCP » : qu'est-ce qui le prouve aujourd'hui ?) ? Les
  § 6.4 (`:179-195`, « Il n'existe pas de serveur MCP « Kesh-natif » en v0.2 ») et § 9 (`:829`) sont-ils restés vrais ?
  Manuel d'administration `docs/manual/fr/admin-manual.tex:2935` : « 122 des 128 routes à verbe mutant » et
  « 122 + 4 + 2 = 128 » — **recompte** depuis `LIB_ROUTES` d'`audit_route_registry.rs` (attention : une entrée y est
  écrite sur plusieurs lignes) ; la commande est-elle écrite au Change Log ? Une autre phrase des manuels est-elle
  devenue fausse par la partie c (`grep -nF "clé" docs/manual/fr/admin-manual.tex | grep -iF "MCP"`, § *Journal
  d'audit*, § *clés d'accès personnelles*) ? Contrôle le **PDF aplati**, pas seulement le `.tex` :
  `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oE ".{60}routes à verbe mutant.{60}"` ;
  le PDF est-il régénéré depuis le `.tex` de HEAD ? Les PDF du manuel utilisateur et de la brochure sont-ils inchangés
  (`git diff --stat fa5b3468...HEAD -- docs/manual`) ? L'absence d'entrée au CHANGELOG (C-32-5-58) tient-elle ?
- **Étape 0** : chaque point de *À recontrôler à l'étape 0* et de § *Pour la 32-5c* traité ou reporté au Dev Agent
  Record, ligne lue citée — et ces lignes, relues à `fa5b3468` (`git show fa5b3468:<f> | sed -n …`), sont-elles
  exactes ? Les doc-comments Q11 de `lib.rs` (déjà corrigés par la 32-4b) ont-ils été laissés intacts ?
- **Décomptes** : fichiers et tests ajoutés recomptés (`git show fa5b3468:<f>` contre `HEAD`,
  `grep -cE '#\[(sqlx::|tokio::)?test'`), clés i18n recomptées (`grep -cE '^(admin-mcp-|audit-log-(entity|action)-mcp)'`
  sur chaque catalogue), périmètre déclaré ; chiffres du gate complet, du frontend et de l'E2E contre les journaux, au
  **dernier commit de code** (`git log --oneline fa5b3468..HEAD`) ; le binaire de l'E2E bâti après ce commit ; échecs
  E2E jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus*.
- **Messages de commit** : `git log fa5b3468..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
  doit ne rien rendre ; chaque commit porte `refs #723`.
- **Registre et statut** : clé `32-5c-routes-gestion-cles-mcp` de `sprint-status.yaml` à `review` ; registre complété
  par union (C-32-5-51 et suivantes), sans réécrire C-32-5-1 à 50.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/325c-review-p1-<B|E|A>.md` (selon ta lentille) :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `B1…`/`E1…`/`A1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`,
  `sed -n` ou `git diff`), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- rien d'autre n'est écrit nulle part.
