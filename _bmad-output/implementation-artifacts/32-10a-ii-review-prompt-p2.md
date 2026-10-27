# Prompt — revue de code P2 de la Story 32-10a-ii, passe ciblée, une lentille (regression hunter), Sonnet

Tu relis **la seule remédiation de la revue de code P1** de la **partie ii de la Story 32-10a** de Kesh (comptabilité
suisse, Rust/Axum, MariaDB 10.11) : le catalogue des prompts que le serveur MCP sert par `prompts/list` et
`prompts/get`. C'est une **passe ciblée** (CLAUDE.md, § *La passe ciblée*) : une lentille, *regression hunter*, braquée
sur **un commit**. Ce que la P1 a relu n'est pas à relire ; ce qui compte est ce que la remédiation a **écrit**, et ce
qu'elle a pu **casser ou laisser à moitié**. Modèle : **Sonnet**. Contexte frais : ne t'appuie sur aucune passe
antérieure. Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-10a-ii`. **Le commit à relire** :
`git -C /home/gcorbaz/devel/kesh-32-10a show 8afeeb7e` (remédiation P1). Les commits qui l'entourent ne sont pas à
relire : `fff925c1` est la fusion d'`origin/main` (32-9b, #785 ; conflits résolus par union dans `CHANGELOG.md`,
`services/acces.rs` et la garde (vi) de `services_motif.rs`, qui compte désormais **seize** fichiers) — regarde-le
seulement si la remédiation s'y appuie ; le commit suivant n'ajoute que l'entrée du Change Log.

⚠️ **Cette partie porte une surface d'injection** : tout texte qui sort par `/mcp` **instruit un agent** qui lit des
livres réels. Le critère n'est pas « ça marche », c'est « rien d'autre ne sort ».

Lis, dans cet ordre :

1. Les trois rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/3210a-ii-review-p1-{B,E,A}.md` — ce qu'on a
   demandé de corriger, et pourquoi.
2. Le commit `8afeeb7e`, en entier.
3. Au registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3210a.md`, **C-32-10a-90, 91, 94, 95,
   96** ; à la fiche `_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md`, le § *À recontrôler à l'étape
   0* (amendements D1, D1 étendu et son complément, et le point neuf « Pour la 32-10a-iii (revue de code P1 de la
   32-10a-ii) ») et la tâche **T-iii0**.
4. L'entrée « Revue de code P1 » du **Change Log** de la 32-10a-ii et le **Dev Agent Record** de la remédiation : ils
   déclarent des gates et des mutations — **ne les prends pas pour argent comptant**, recoupe-les avec les journaux
   `/home/gcorbaz/devel/kesh-gate-logs/3210a-ii-p1-*` (lecture seule).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs. **Aucune redirection, sous aucune forme**
  (`>`, `>>`, `<`, `<<`, `tee`, `2>`, `&>`, `2>&1`) : compare par des commandes enchaînées (pipe). Un fichier créé par
  erreur se **déclare** au rapport.
- **Aucun `rm`, sous aucune forme** (ni `rm -f`, ni `rm -rf`, ni `unlink`, ni `find -delete`, ni `rmdir`) : supprimer,
  c'est écrire.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`, `merge`, `rerere`).
- **Aucun interpréteur, rien qui écrit, compile, calcule ou exécute du code** : ni `python`/`python3`, ni `perl`, ni
  `ruby`, ni `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq`, ni **`bc`**, ni `expr`, ni `awk` ; ni
  **`scripts/prepare-release.sh`**, ni `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni
  `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`, ni aucun script de `kesh-gate-logs` ou d'un scratchpad (dont
  `muter-3210a-ii.sh` et `remplacer-3210a-ii.py`) ; ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande,
  `cargo fmt` compris), ni `rustc`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `cat`, `ls`, `find` (sans
  `-exec`, `-delete`), `wc`, `sort`, `uniq`, `head`, `tail`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée dans le
  rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques (`PIEGE_*_3210A`). Ne lis rien sous `~/kesh-test/`.

## La lentille — regression hunter, sur `8afeeb7e`

Pour chaque hunk du commit, demande : *qu'est-ce que ce changement a pu casser, et qu'a-t-il laissé à moitié ?*

1. **`composer_version`** (`crates/kesh-api/src/services/prompts_mcp.rs`) — la décision « origine → marque » est-elle
   **entièrement** dans cette fonction (plus aucun `Origine::Agent` ni appariement de clé ailleurs dans `src/`) ?
   `composer` est-elle **privée**, et plus rien hors du module ne l'appelle (`git grep -nF "composer("`) ? L'ordre
   « agent sans clé avant le contenu » est-il tenu et testé ? Une clé posée sur une version standard ou
   d'administrateur est ignorée : est-ce sûr (une marque manquerait-elle à un texte d'agent, une marque apparaîtrait-elle
   sur un texte qui n'en est pas) ? `OrigineVersion` et `RefusVersion` : un type de `kesh_db` entre-t-il ou sort-il
   encore de la signature publique ? `RefusVersion` porte-t-il un texte (Debug compris) ? La conversion
   `From<Origine>` est-elle exhaustive, sans `_` qui absorberait une variante future ?
2. **`servir`** — le nom revalidé par `nom_conforme` **avant** la composition ; les trois `tracing::error!` : aucun ne
   porte un nom, une description ou un texte (`grep -nF "tracing::" crates/kesh-api/src/services/prompts_mcp.rs`,
   chaque appel lu). Le comportement servi est-il **inchangé** pour une version valide (corps de `prompts/list` et
   `prompts/get` égaux à ceux d'avant la remédiation) ? Un chemin par lequel une version d'agent sortirait sans sa
   marque ?
3. **La politique écrite (C-32-10a-96, doc-comment de `lister`)** — décrit-elle exactement le code (le dépôt rend-il
   bien une erreur pour toute la liste sur une version sans contenu ; une version refusée est-elle omise seule) ? Les
   deux cas cités sont-ils les seuls ?
4. **Les tests neufs ou amendés** — `composer_version_par_origine`, `origine_de_la_base_convertie`,
   `servir_revalide_le_nom_et_la_cle`, `caracteres_par_champ` (U+3164, U+2028), `tri_octet_pour_octet`, la clé de plus
   du `catalogue` de `tests/mcp_prompts_e2e.rs` et son `assert_ne!` : un test passe-t-il **à vide** (attente calculée
   par la fonction éprouvée ; ordre attendu qui serait aussi celui de la collation ; témoin positif absent) ? La clé
   ajoutée au `catalogue` change-t-elle un autre test du fichier (comptes, numéros de clé attendus, corps égaux) ?
   L'ordre `utf8mb4_unicode_ci` de `a_b`, `a-b`, `a1` diffère-t-il réellement de l'ordre des octets (raisonne sur le
   DUCET, dis que tu ne l'as pas mesuré) ?
5. **Les gardes** (`crates/kesh-api/tests/mcp_gardes_lexicales.rs`) — `nomme_le_depot_directe` et
   `nomme_le_depot_en_liste` : leur union est-elle exactement l'ancien `nomme_le_depot` ? Les mesures séparées sur
   `api_keys` et le témoin `services/factures.rs` prennent-ils des **lignes réelles** (`grep -nF` à l'appui) ?
   `MOTIF_LIVRES` élargi à `kesh_db::{…repositories…}` et l'assertion sur le **code aplati** de `prompts_mcp.rs` :
   faux positif possible sur le code réel (une ligne de code qui, aplatie avec la suivante, formerait un jeton) ? Le
   vert factice `use kesh_db::{entities::Role, errors::DbError}; // repositories` est-il vert pour la bonne raison ?
6. **Les comptes rendus** — fiche (§ *À recontrôler*, T-iii0), registre C-32-10a-94 à 96, Change Log, Dev Agent Record :
   disent-ils ce que fait le code ? Chaque nombre (tests neufs, mutations, gates, E2E) se recompte-t-il depuis sa source,
   avec son périmètre ? Les mutations neuves sont-elles **tuées par le test visé**, pour la raison visée (lis la panique
   dans `3210a-ii-p1-mut-*.log`, pas seulement le verdict), avec un diff résiduel nul ?
7. **Ce qui sort par `/mcp`** — pour le seul code touché, refais l'énumération de la P1 (champ, source, ce qu'il peut
   porter) et dis si la remédiation y change quoi que ce soit.
8. **Axe du manuel** — la remédiation ne touche ni `.tex` ni PDF ni CHANGELOG ; vérifie que rien de publié (`CHANGELOG.md`
   entrée « Les prompts du serveur MCP », `docs/manual/fr/*.tex`, PDF aplatis) ne contredit le comportement
   remédié (nom revalidé, tri octet pour octet, marque).

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/3210a-ii-review-p2.md`, par l'outil
d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`R1`, `R2`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et
   ligne, la preuve (sortie de commande copiée, `grep -nF` pour toute présence ou absence), la correction proposée, et
   s'il est **né de la remédiation** `8afeeb7e` ou **antérieur**.
2. **L'énumération de ce qui sort par `/mcp`** pour le code touché (axe 7).
3. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
4. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
5. Les fichiers créés par erreur, s'il y en a.
