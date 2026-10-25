# Prompt — revue de code P1 de la Story 32-5b (l'authentification par clé MCP), trois lentilles, Sonnet

Tu relis le développement de la **partie b de la Story 32-5** de Kesh (comptabilité suisse, Rust/Axum, MariaDB) :
*la génération partagée des clés (`kesh_mcp_`), `Portee` devenue l'alias de `McpScope`, le service
`services::cles_mcp::authentifier`, les refus croisés (une clé MCP n'ouvre pas `/api/v1`, un PAT n'authentifie pas au
MCP), la séparation lecture / trace, et une clé restaurée qui authentifie*. Worktree `/home/gcorbaz/devel/kesh-32-5b`,
branche `story/32-5b`, partie de `b723e3d9` (`main`, 32-5a, 32-4a et 32-4b fusionnées). Le diff à relire, **aplati**
(un seul diff, pas la suite des commits) : `git -C /home/gcorbaz/devel/kesh-32-5b diff b723e3d9...HEAD`. Langue du
rapport : français. Modèle : **Sonnet**. Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Seule la partie b est en revue.** Les AC des parties c et d (routes de gestion, remise à zéro, réparation) ne sont
pas livrés et ne sont pas des findings ; en revanche, un état intermédiaire qui ouvrirait un accès ou ferait mentir un
texte en est un (fiche, § *Le découpage*, dernier alinéa : « après 32-5b, une clé s'authentifie au service, mais aucune
route ne la crée et aucun endpoint ne la consomme »).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-5-cles-et-reglages-mcp.md` — *Ce que le code dit* (faits 1, 2, 6),
*Partie B* (AC-b1 à AC-b6), tâches T-b0 à T-b4, mutations Mb1 à Mb6, *Ce que les gardes ne voient pas*, *Esquisse des
types*, *À recontrôler à l'étape 0* et son § *Pour la 32-5b*, et le **Dev Agent Record** de la partie b (étape 0,
notes, décomptes) — puis le registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-325.md`
(C-32-5-4, 7, 14, 15, 23, 37, et les entrées du développement de la 32-5b, C-32-5-38 à C-32-5-42). Le Dev Agent Record
déclare des gates, des mutations et un E2E : **ne les prends pas pour argent comptant**, recoupe-les avec les journaux
`/home/gcorbaz/devel/kesh-gate-logs/325b-*.log` (lecture seule). Aucun contrôle visible ne change : pas de capture
attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni ailleurs ; **aucune redirection** (`>`, `>>`, `tee`) : compare par
  des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement. Un fichier créé par erreur se
  **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni **`scripts/prepare-release.sh`**, ni
  `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`,
  ni aucun script de `kesh-gate-logs` ou d'un scratchpad, ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande,
  `cargo fmt` compris), ni `rustc`, ni `npm`/`npx`/`node`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni
  `python3`, ni `sed -i`, ni `perl`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `wc`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.
- **Données réelles** : aucune ; les fixtures sont synthétiques. Ne lis rien sous `~/kesh-test/`.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. `auth/api_key.rs` : `generate_prefixed` sert-il les trois générateurs sans changer le jeton de réinitialisation
   (format, longueur, absence de préfixe) ? `MCP_PREFIX` et `PAT_PREFIX` : aucun n'est préfixe de l'autre ? Le
   `debug_assert_eq!` de `base62_fixed_width` tient-il toujours ?
2. `services/cles_mcp.rs` : le préfixe est-il contrôlé **aux octets** et **avant** tout accès à la base ? La recherche
   se fait-elle sur la `Lecture` et la dernière utilisation sur la `Trace` — et nulle part ailleurs ? Un refus peut-il
   rendre autre chose qu'`AppError::Unauthenticated` (un `unwrap`, un 403, un 500 déguisé) ? Une erreur de base est-elle
   **toujours** propagée, jamais changée en 401 ? Le journal nomme-t-il jamais le jeton ou son condensat ? Un échec de
   `touch_last_used` peut-il faire échouer l'authentification ? `CleMcpAuthentifiee` dérive-t-elle quelque chose qui
   exposerait un secret (`Serialize`) ?
3. `services/acces.rs` : `pub use … McpScope as Portee` — les dérivations dont dépendent les services (`Ord` pour le
   `BTreeSet`, `Copy`, `Hash`) y sont-elles ? `Canal`, `Exigence`, `exiger` inchangés ?
4. `repositories/mcp_keys.rs` (un fichier de `kesh-db`) : `auth_row` rend-il **exactement** ce qu'il rendait (même
   `None` pour une clé sans portée connue, mêmes portées pour une clé mixte) ? `read_scopes` est-il encore appelé par
   les chemins d'administration ? Le journal nomme-t-il le condensat ?
5. Les tests (`cles_mcp_e2e.rs`, `admin_full_import_e2e.rs`, `services_motif.rs`, `services_rollout.rs`) : un test
   peut-il passer **à vide** — un 401 qui viendrait d'une route absente plutôt que du refus (le contrôle JWT de
   `b4_a_b_…` ouvre-t-il bien les deux chemins ?), un `last_used_at` déjà `NULL` par construction, un témoin (β) qui
   réussirait pour une autre raison, le montage orphelin sous `FOREIGN_KEY_CHECKS = 0` sur une autre connexion du
   pool, un refus de portée qui passerait parce que la clé n'a **aucune** portée ? `autre_portee` → `autres_portees` :
   le contrôle positif (portée exigée seule) existe-t-il encore à côté ?

## Lentille E — Edge cases

1. Les jetons : vide, très long, non ASCII, `kesh_mcp_` seul, `kesh_mcp_` suivi d'un PAT entier ou de son corps, casse
   (`KESH_MCP_`), espace de tête — chacun mène-t-il à 401 sans erreur de base ? Un jeton `kesh_mcp_…` présenté au
   middleware de `/api/v1` en **cookie** et en **en-tête** : par où passe-t-il (`middleware/auth.rs:83`, `:134`,
   `:146`), et le refus vient-il bien de `jwt::decode` ?
2. Le créateur : rétrogradé (`Comptable`, `Consultation`), désactivé, réactivé, supprimé (FK `RESTRICT` : possible ?) ;
   la société : absente ; la clé : expirée à la milliseconde près (`NOW(3)`), révoquée entre la recherche et
   `touch_last_used` (la clause `revoked_at IS NULL` de l'`UPDATE`).
3. La `Lecture` en `READ ONLY` : `find_active_auth_by_key_hash` n'écrit-il rien et ne pose-t-il aucun verrou
   (`FOR UPDATE`) ? La 32-6 recevra-t-elle un service qui fonctionne sur sa connexion en lecture seule — le test
   `b5_lecture_seule_et_trace` le prouve-t-il, et son témoin porte-t-il sur la **requête même** de `touch_last_used` ?
4. La restauration (AC-b6) : K1 supprimée puis restaurée, K2 créée après l'export, une clé révoquée avant l'export —
   et une clé révoquée **après** l'export (elle redevient active : le manuel d'administration le dit-il, `:2237`) ?
5. Les **mutations** Mb1 à Mb6 (sept jouées : Mb1, Mb2, Mb3, Mb4, Mb5, Mb6a « contrôle retiré », Mb6b « contrôle
   déplacé après la recherche ») : chacune tuée par une **assertion nommée** (`325b-mutations.log`) ? Le motif muté
   est-il celui que décrit la fiche ? Une mutation plausible **non jouée** survivrait-elle — `authentifier` qui rend
   les portées `ALL` au lieu de celles de la clé, la politique `Required` en dur, le `company_id` ou le `user_id`
   intervertis, `touch_last_used` retiré, `ROLE_CLE_MCP` à `Comptable`, le refus des quatre autres portées dans les
   services (une `Exigence` dont la portée serait changée) ? Raisonne sur le code, n'exécute rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-b1 à AC-b6, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve. En
particulier :

- **AC-b1** : `grep -c "OsRng.fill_bytes" crates/kesh-api/src/auth/api_key.rs` à `b723e3d9` et à HEAD ; les tests
  existants de `generate_pat` et de `generate_reset_token` inchangés (`git diff b723e3d9...HEAD -- …api_key.rs`).
- **AC-b2** : `Portee` est l'alias ; `TOUTES` vaut `Portee::ALL` ; chaque refus de portée de `services_motif.rs` et de
  `services_rollout.rs` éprouve la clé qui porte les quatre autres portées ; `ac10_vi` compte onze fichiers et
  `RESIDUS_VI` est inchangé (`git diff b723e3d9...HEAD -- crates/kesh-api/tests/services_motif.rs`). Les comptes de
  l'AC (écrits à `e59d63e8`) sont **recomptés relativement à `b723e3d9`** au Dev Agent Record : recompte-les
  (`git grep -o "Portee::" b723e3d9 -- 'crates/*.rs' | wc -l`, et de même à HEAD).
- **AC-b3** : l'ordre, les refus, l'erreur de base propagée, (α) et (β) ; les doc-comments de `poignees.rs`,
  `services/mod.rs` et `acces.rs` disent-ils le nouvel usage de la `Trace` et le constructeur de production — et les
  sites de la Q11, déclarés déjà corrigés par la 32-4b à l'étape 0, le sont-ils vraiment
  (`grep -rn "utilisateur MariaDB" crates --include=*.rs`) ?
- **AC-b4, AC-b5, AC-b6** : chaque test nommé par l'AC existe et asserte ce que l'AC exige.
- **Le manuel et `docs/api-external.md`** : la partie b ne change rien de visible (Dev Agent Record) — est-ce vrai ?
  Relis `docs/api-external.md` (§ 2 *Format de la clé*, l. 29 ; § 6.4 *Agent IA / serveur MCP*, l. 179-194 ; l. 829)
  et `docs/manual/fr/admin-manual.tex` (l'avertissement « secret » `:2230`, *Ce que la sauvegarde emporte* `:2237`)
  **contre le code à HEAD** : une phrase est-elle devenue fausse par la partie b ? La décision de laisser
  `api-external.md` à la 32-5c (C-32-5-42) tient-elle ? Contrôle aussi le **PDF aplati**
  (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{60}serveur MCP.{80}"`).
- **Étape 0** : chaque point de *À recontrôler à l'étape 0* et du § *Pour la 32-5b* qui concerne la partie b traité au
  Dev Agent Record, ligne lue citée — et ces lignes, relues à `b723e3d9`, sont-elles exactes ?
- **Décomptes** : tests ajoutés recomptés (`git show b723e3d9:<f>` contre `HEAD`,
  `grep -cE '#\[(sqlx::|tokio::)?test'`), périmètre déclaré ; chiffres du gate complet, du build du frontend et de
  l'E2E contre les journaux, au **dernier commit de code** (`git log --oneline b723e3d9..HEAD`) ; le binaire de l'E2E
  bâti après ce commit ; échecs E2E jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus*.
- **Exception `kesh-db`** : la partie b touche `crates/kesh-db/src/repositories/mcp_keys.rs` ; le gate complet a-t-il
  tourné au dernier commit de code (`CLAUDE.md`, *Test Locally First*, exception `kesh-db`) ?
- **Messages de commit** : `git log b723e3d9..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
  doit ne rien rendre ; chaque commit porte `refs #723`.
- **Registre et statut** : clé `32-5b-authentification-cle-mcp` de `sprint-status.yaml` à `review` ; registre complété
  par union (C-32-5-38 et suivantes), sans réécrire C-32-5-1 à 37.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/325b-review-p1-<B|E|A>.md` (selon ta lentille) :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `B1…`/`E1…`/`A1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`,
  `sed -n` ou `git diff`), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- rien d'autre n'est écrit nulle part.
