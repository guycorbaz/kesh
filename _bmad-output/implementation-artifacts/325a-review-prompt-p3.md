# Prompt — revue de code P3 de la Story 32-5a : passe ciblée, regression hunter, Sonnet

Tu relis **la seule remédiation de la revue de code P2** de la partie a de la Story 32-5 de Kesh (comptabilité suisse,
Rust/Axum, MariaDB) : *les tables MCP et la sauvegarde*. Worktree `/home/gcorbaz/devel/kesh-32-5`, branche
`story/32-5`. Modèle : **Sonnet**. Langue du rapport : français. Contexte frais : ne t'appuie sur aucune passe
antérieure, sinon pour savoir ce que la remédiation prétendait corriger.

## Ce qui est relu — et seulement cela

Les deux commits de la remédiation, désignés **par leur SHA** (ne les retrouve pas par `git log --grep`) :

- **`88eae2b2`** — `crates/kesh-db/src/entities/mcp_key.rs` : deux commentaires (le doc-comment de `McpScopes::from_json`
  et le commentaire du test `unknown_scope_string_is_refused`) disent la lecture tolérante de C-32-5-37 ; le test
  `stored_scopes_without_any_known_scope_are_not_usable` gagne `"BOOKS:READ"` et `" books:read"`.
- **`a08c17ee`** — artefacts : annotations datées de l'AC-a4 et des Dev Notes de la fiche
  (`_bmad-output/implementation-artifacts/32-5-cles-et-reglages-mcp.md`), du « Retenu » de C-32-5-3
  (`epic-32-choix-autonomes-325.md`, renvoi à C-32-5-37), de la phrase fausse de l'entrée « Revue de code P1 » du
  Change Log ; la phrase sur U+00A0 devenue « non éprouvé » (§ *Pour la 32-5c*) ; R5 reporté (§ *Pour la 32-5b*).

`git -C /home/gcorbaz/devel/kesh-32-5 show 88eae2b2`, puis `git -C … show a08c17ee`. Le commit qui suit (entrée
« Revue de code P2 » et ce prompt) n'est pas à relire. Pour le contexte, le rapport de la P2 :
`/home/gcorbaz/devel/kesh-gate-logs/325a-review-p2.md` (lecture seule).

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
- Permis : `git show`, `git diff`, `git log`, `git grep`, `grep`, `sed -n`, `awk` (sans redirection), `cat`, `ls`, `wc`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.

## La lentille — regression hunter

1. **Les deux commentaires de `mcp_key.rs`** : disent-ils maintenant vrai, au mot près, contre le code (`from_json`
   n'a-t-il bien plus aucun appelant de lecture de colonne : `git grep -nF "from_json" -- crates/`) ? Un commentaire
   voisin, ou le doc-comment de `StoredMcpScopes`, de `parse`, du module, dit-il encore autre chose ?
2. **Le test** : les deux cas neufs éprouvent-ils la casse **et** l'espace séparément (un seul cas pourrait-il tuer les
   deux mutations à la fois, ou aucune) ? L'assertion sur `unknown()` est-elle exacte (ordre, contenu) ? La mutation
   Mq1 (`s.trim().to_ascii_lowercase().parse()`) est-elle tuée par une assertion nommée
   (`kesh-gate-logs/325a-p2r-mutations.log`, `325a-p2r-mut-Mq1.log`) ? Une variante (`trim()` seul, ou casse seule)
   survivrait-elle ?
3. **Les annotations** : chacune est-elle datée, **ajoutée** sans réécrire le texte d'origine (compare au parent :
   `git show a08c17ee^:<fichier>`), et dit-elle vrai contre le code et C-32-5-37 ? L'annotation du Change Log
   corrige-t-elle exactement la phrase fausse, sans toucher d'autre entrée passée ?
4. **La propagation** : greppe toi-même sur tout le dépôt les valeurs `Strict`, `strict`, `jamais ignorée`, `décod`
   (`git grep -nE "Strict|strict|jamais ignorée|décod" -- crates docs _bmad-output CHANGELOG.md`), restreint à ce qui
   parle des portées MCP, de la colonne `scopes` ou de leur lecture : un site dit-il encore que la **lecture d'une ligne**
   est stricte, sans annotation ? (Ceux qui parlent de l'entrée d'API, de `send_validation`, des réglages
   `mcp_server_settings`, ou d'autres sujets ne sont pas des findings.)
5. **R4** : la phrase sur U+00A0 dit-elle maintenant « non éprouvé », et rien ailleurs ne l'affirme éprouvé
   (`grep -rnF "00A0"` sur la fiche, le registre, `crates/`) ?
6. **R5** : le report au § *Pour la 32-5b* est-il exact contre `crates/kesh-db/src/repositories/mcp_keys.rs`
   (`read_scopes`, `auth_row`) ? Ne nomme-t-il jamais le condensat comme quelque chose à journaliser ?
7. **P5 à P8** : la remédiation touche-t-elle `crates/kesh-db/migrations/` ou `migrations.sha384`
   (`git show --stat 88eae2b2 a08c17ee`) ? Elle ne le doit pas.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/325a-review-p3.md` :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `S1…`, chacun avec sévérité, fichier:ligne, **preuve copiée**, correctif proposé, et la mention
  « né de la remédiation » ou « défaut d'origine » ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- rien d'autre n'est écrit nulle part.
