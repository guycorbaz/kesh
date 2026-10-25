# Prompt — revue de code P2 de la Story 32-5a : passe ciblée, regression hunter, Opus

Tu relis **la seule remédiation de la revue de code P1** de la partie a de la Story 32-5 de Kesh (comptabilité suisse,
Rust/Axum, MariaDB) : *les tables MCP et la sauvegarde*. Worktree `/home/gcorbaz/devel/kesh-32-5`, branche
`story/32-5`. Modèle : **Opus**. Langue du rapport : français. Contexte frais : ne t'appuie sur aucune passe antérieure,
sinon pour savoir ce que la remédiation prétendait corriger.

## Ce qui est relu — et seulement cela

Les trois commits de la remédiation, désignés **par leur SHA** (ne les retrouve pas par `git log --grep`) :

- **`24f80e91`** — le code : `StoredMcpScopes` (`crates/kesh-db/src/entities/mcp_key.rs`), la lecture tolérante du dépôt
  (`crates/kesh-db/src/repositories/mcp_keys.rs`), les tests (`crates/kesh-db/tests/mcp_keys_repository.rs`,
  `crates/kesh-api/tests/admin_backup_e2e.rs`), trois phrases du manuel d'administration (`docs/manual/fr/admin-manual.tex`).
- **`034d2a3b`** — la documentation : PDF régénéré, registre (`epic-32-choix-autonomes-325.md`, C-32-5-37 et le
  constat B2), fiche (`32-5-cles-et-reglages-mcp.md` : § *Pour la 32-5b*, § *Pour la 32-5c*, annotation A1).
- **`a5cc29c4`** — le doc-comment du module `mcp_key.rs`, qui disait encore la lecture stricte (propagation trouvée par
  l'orchestrateur après `034d2a3b`).

`git -C /home/gcorbaz/devel/kesh-32-5 show 24f80e91`, puis `034d2a3b` et `a5cc29c4`. Le commit qui suit (Change Log
et ce prompt) n'est pas à relire. Pour le contexte, les rapports de la
P1 : `/home/gcorbaz/devel/kesh-gate-logs/325a-review-p1-{B,E,A}.md` (lecture seule). La décision de l'orchestrateur
sur E1 : une portée inconnue se **tolère à la lecture** (ignorée, journalisée par id de clé, jamais le condensat), les
portées connues restent ; une clé sans aucune portée connue se liste et se révoque mais **n'authentifie pas** ;
l'écriture et le décodage des entrées d'API restent stricts (Ma3 vraie).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ;
  **aucune redirection** (`>`, `>>`, `tee`). Un fichier créé par erreur se **déclare** ; il ne se supprime pas.
- **Aucun `git checkout`, `git switch`, `git restore`**, aucun `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`,
  `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni **`scripts/prepare-release.sh`**, ni
  `scripts/regen-test-schema.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script de `kesh-gate-logs`
  ou du scratchpad, ni `make`, `latexmk`, `cargo` (aucune sous-commande), `rustc`, `npm`/`npx`/`node`, `docker`,
  `mysql`/`mariadb`, `sqlx`, `curl`, `python3`, `sed -i`, `perl`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git show`, `git diff`, `git log`, `git grep`, `grep`, `sed -n`, `awk` (sans redirection), `cat`, `ls`, `wc`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.

## La lentille — regression hunter

La sévérité se déplace vers ce qu'on vient d'écrire : cherche ce que **cette remédiation** casse ou laisse ouvert.

1. **`StoredMcpScopes::from_stored_json`** : un chemin qui échoue encore (panic, `unwrap`) ? Un élément connu perdu
   (doublons, casse, espaces) ? Que deviennent `null`, un tableau imbriqué, un nombre — et `unknown()` les rend-il sous
   une forme qui ne fuit rien d'autre que la colonne ? `usable()` : peut-il rendre un ensemble vide ? `From<McpScopes>`
   respecte-t-il l'invariant ?
2. **Le dépôt** (`mcp_keys.rs`) : **chaque** lecture passe-t-elle par `KeyRow`/`AuthRow` (aucun `query_as::<_, McpKey>`
   ni `query_as::<_, McpKeyAuthRow>` restant) ? `create_in_tx` aussi ? Le `tracing::warn!` nomme-t-il l'id et **jamais**
   `key_hash` ni le jeton (y compris dans `?stored.unknown()`) ? Une clé sans portée connue est-elle rendue par
   `find_by_id_for_company` (donc révocable) et **pas** par `find_active_auth_by_key_hash` ? Le refus est-il un `None`,
   jamais une erreur de base ?
3. **Le retrait de `impl Decode for McpScopes`** : un autre site du workspace en dépendait-il (`git grep -n "McpScopes"`,
   `crates/` entier) ? `Encode`/`Type` restent-ils cohérents pour le `bind` de `create_in_tx` ?
4. **Ce qui doit rester strict** : `McpScopes::parse` et `from_json` inchangés ? Ma3 (`unknown_scope_string_is_refused`)
   toujours vraie ? Le doc-comment du module `mcp_key.rs` (« le décodage est strict … jamais ignorée ») contredit-il
   désormais le code ? D'autres textes (doc-comments, fiche, registre C-32-5-33, manuel) disent-ils encore que la
   lecture est stricte ? Greppe la **valeur** : `strict`, `jamais ignorée`, `fails_to_decode`.
5. **Les tests neufs ou changés** : chacun peut-il passer **à vide** ?
   - `a_stored_unknown_scope_is_ignored_and_the_known_ones_remain`, `a_key_without_any_known_scope_is_listed_and_revocable_but_never_authenticates` ;
   - `list_by_company_orders_by_creation_then_id_descending` (horodatages posés après les insertions ; `ON UPDATE` de
     `updated_at` sans effet sur l'ordre ?) ; `auth_lookup_expiry_boundary` (fuseau de la session, `NOW(3)` côté base) ;
   - les assertions resserrées (« `mcp_keys.scopes` », « `mcp_server_settings.allowed_networks` ») : le message de
     MariaDB porte-t-il ces graphies exactes ? ;
   - les deux RESTRICT sur `companies` : la suppression pourrait-elle être refusée par **une autre** FK que celle
     qu'asserte le test (la société D a-t-elle d'autres enfants : `onboarding_state`, `company_invoice_settings` …) ? ;
   - l'aller-retour de `admin_backup_e2e.rs` : la révocation après l'export, la clé « créée depuis », et leurs assertions.
6. **Les mutations** Mp1 à Mp5 (`kesh-gate-logs/325a-p1r-mutations.log`, `325a-p1r-mut-*.log`) : tuées par l'assertion
   nommée ? Une mutation plausible de la remédiation **non jouée** survivrait-elle — `read_scopes` qui ne journalise
   plus, `find_by_id_for_company` qui filtrerait les clés sans portée connue, `from_stored_json` qui mettrait de côté
   une portée connue écrite en majuscules ? La déclaration « portée inconnue acceptée telle quelle : impossible à
   écrire » est-elle exacte ?
7. **Le manuel, PDF compris** : les trois phrases neuves (`admin-manual.tex` : restauration de l'état des clés, renvoi
   au paragraphe *Compatibilité*, forme normalisée trop longue) sont-elles vraies contre le code (`routes/admin.rs`,
   `backup.rs`) ? Le PDF (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`) les porte-t-il, et
   le renvoi y rend-il le bon numéro de section ?
8. **Les règles P5 à P8** : la remédiation touche-t-elle `crates/kesh-db/migrations/` (`git show --stat 24f80e91 034d2a3b a5cc29c4`)
   — elle ne le doit pas (P8 : la ligne de `migrations.sha384` est inscrite) ; aucune migration neuve (P5, P6) ; aucune
   écriture de données neuve (P7). Le constat B2 au registre dit-il vrai de l'en-tête et de l'ordre de `McpScope::ALL` ?
9. **Les reports** (§ *Pour la 32-5b*, § *Pour la 32-5c*) : exacts contre le code (en particulier « U+00A0 refusé en
   base, éprouvé à la main » — ce que tu ne peux qu'examiner au journal, sans exécuter : dis-le) ?

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/325a-review-p2.md` :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `R1…`, chacun avec sévérité, fichier:ligne, **preuve copiée**, correctif proposé, et la mention
  « né de la remédiation » ou « défaut d'origine » ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- rien d'autre n'est écrit nulle part.
