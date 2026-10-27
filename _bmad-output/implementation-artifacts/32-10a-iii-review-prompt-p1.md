# Prompt — revue de code P1 de la Story 32-10a-iii (les écritures des prompts MCP, le journal et l'audit), trois lentilles, Opus

Tu relis le développement de la **partie iii de la Story 32-10a** de Kesh (comptabilité suisse, Rust/Axum, MariaDB
10.11) : *l'outil `proposer_prompt` (portée `prompts:write`, hors registre et hors politique, une version d'agent « à
valider » et l'entrée d'audit de l'appel dans une seule transaction, par la poignée `EcriturePrompts` sans pool
visible), le journal de `prompts/get` (`mcp.prompt_served`, fail closed), les sept routes de gestion de
l'administrateur sous `/api/v1/admin/mcp/prompts` (liste, détail au texte servi, prompt neuf, version
d'administrateur, approbation revalidée, refus, retour au standard), leur audit sans texte, ses libellés dans les quatre
locales, et les registres des routes.* Les tables, le dépôt et l'amorçage sont la 32-10a-i ; le catalogue servi
(`prompts/list`, `prompts/get`, `composer_version`, la marque) est la 32-10a-ii — toutes deux fusionnées ; l'écran et la
section du manuel *Les prompts du serveur MCP* sont la 32-10b.

Worktree `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-10a-iii`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-32-10a diff origin/main...HEAD`. Langue du rapport : français. Modèle : **Opus**.
Passe **complète** (première passe d'une revue de code : jamais Haiku). Contexte frais : ne t'appuie sur aucune passe
antérieure.

⚠️ **Cette partie porte une surface d'injection et une surface d'écriture** : un texte qui sort par `/mcp` **instruit
un agent** qui lit des livres réels, et un agent peut désormais **écrire** une proposition. Les critères : rien ne sort
avant l'approbation d'un administrateur ; aucune clé ne peut approuver ; aucun texte de prompt n'entre au journal
d'audit (exporté) ni au journal du serveur (lu par l'agent de développement) ; ce que l'écran montrera est ce que
l'agent recevra.

⚠️ **Hors périmètre** : un défaut que la 32-10a-iii ne touche pas est un finding **antérieur**, à classer comme tel.
La 32-10a-iii **ne touche pas `kesh-db`** (fiche, l. 190) : un besoin de dépôt qu'elle a contourné est un finding.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md` — § *Le découpage* (dont la démonstration « chaque
besoin de la iii a sa fonction de la i »), **Partie iii** (AC-iii1 à AC-iii13), § *L'inventaire clos des chemins par
lesquels un texte de prompt atteint un agent* (P1 à P27), *Tâches 32-10a-iii*, *Tests et mutations prévus* (Miii1 à
Miii12), *Gates*, *Ce que les gardes ne voient pas*, *Dev Notes* (§ *Approuvée par mégarde*), le § *À recontrôler à
l'étape 0* **en entier** — notes de l'orchestrateur (`entree_appel` à `envoi: None`, `CleMcpAuthentifiee`), amendements
**D1, D1 étendu, son complément, D1 champs du détail, D2**, le `\r` restauré, « Pour la 32-10a-iii » (contenu normalisé
enregistré, `composer_version` à la route de détail ; #783 points 1 et 4 : la précondition de lecture du plafond et de
la numérotation en `REPEATABLE READ`, la clé de la société) — et le **Dev Agent Record** et le **Change Log** de la
32-10a-iii ; puis le registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3210a.md`, **C-32-10a-100
et suivants** (choix du développement de la iii) et C-32-10a-7, 13, 17 à 21, 26, 27, 30. Le Dev Agent Record déclare
des gates, des mutations et un E2E : **ne les prends pas pour argent comptant**, recoupe-les avec les journaux
`/home/gcorbaz/devel/kesh-gate-logs/3210a-iii-*` (lecture seule) et le script `mutations-3210aiii.py` (à **lire**, jamais
à exécuter). Aucun écran ne change dans cette partie (l'écran est la 32-10b) : pas de capture attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`, `2>`, `&>`) : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au
  rapport.
- **Aucun `rm`, sous aucune forme** (ni `rm -f`, ni `rm -rf`, ni `unlink`, ni `find -delete`) : supprimer, c'est écrire.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`, `merge`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`,
  ni aucun script de `kesh-gate-logs` ou d'un scratchpad (dont **`mutations-3210aiii.py`**) ; ni `make`, ni `latexmk`,
  ni `cargo` (aucune sous-commande, `cargo fmt` compris), ni `rustc`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni
  `curl`, ni `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée dans le
  rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques (`PIEGE_*_3210A`). Ne lis rien sous `~/kesh-test/`.

## L'axe obligatoire pour chaque lentille : ce qui sort, et ce qui s'écrit

1. **Ce qui sort par `/mcp`** — énumère, **à partir du code**, chaque champ de chaque réponse de `tools/list` (le
   descripteur de `proposer_prompt` : sous quelles portées, à quelle place, `readOnlyHint`), de `tools/call
   proposer_prompt` (accusé, chaque refus) et de `prompts/get` (servi, inconnu, erreur, **journal indisponible**), et sa
   source (constante, colonne, entier, valeur reçue). Peut-il porter un texte ou une description **reçus** ou **lus**,
   un numéro de version, un oracle (sur l'historique, sur l'existence d'un nom, sur l'activité d'autres clés — le
   plafond, angle mort P7 écrit) ? Un `tools/call proposer_prompt` **sans** `prompts:write` est-il **indiscernable**
   d'un outil inconnu, au corps et au journal ?
2. **Ce qui sort par l'API d'administration** — les sept routes : la liste (sans texte), le détail (texte **servi**
   composé par `services::prompts_mcp::composer_version`, `servable`, formes rendues visibles par
   `envois_mcp::rendre_visible`, texte du code d'un standard). Qui peut l'atteindre (Comptable, PAT, clé MCP,
   anonyme) ?
3. **Ce qui s'écrit** — chaque écriture des tables des prompts et d'`audit_log` sur ces chemins : dans **quelle**
   transaction, **quel** contenu (le contenu **normalisé** de `valider_contenu`, jamais le texte reçu), **quel** détail
   d'audit (jamais la description ni le texte ; sans le nom d'un prompt d'agent refusé jamais approuvé, F8), au nom de
   **qui** (l'utilisateur de la session, le créateur de la clé). La précondition de lecture (#783 point 1) : dans chaque
   transaction qui appelle `proposer_version_agent_in_tx`, `approuver_in_tx`, `refuser_in_tx`,
   `ajouter_version_admin_in_tx`, `creer_prompt_admin_in_tx`, `revenir_au_standard_in_tx`, **rien n'est-il lu avant**
   l'appel du dépôt ? Les lectures préalables faites **hors** de la transaction (historique, pour D2 et pour le détail
   d'audit du refus) introduisent-elles une fenêtre exploitable ?
4. **Le journal du serveur** (`tracing`) sur ces chemins : `grep -nF "tracing::" ` sur `services/prompts_mcp.rs`,
   `services/poignees.rs`, `services/serveur_mcp.rs`, `mcp/repartiteur.rs`, `routes/mcp_prompts.rs` — chaque appel
   lu : porte-t-il, directement ou par le `Display` d'une erreur (`ErreurEcriturePrompt`, `DbError`, `AppError`), une
   description, un texte ou un nom reçu ?

Recoupe avec l'inventaire P1 à P27 : un site non résolu que l'inventaire n'énumère pas est un finding.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. **`services/prompts_mcp.rs`** (`proposer`, `AccuseServi`, `RefusProposition`, `OUTIL_PROPOSER_PROMPT`,
   `EXIGENCE_PROPOSER`) — l'ordre : `exiger`, arguments stricts (`deny_unknown_fields`), contenu, nom, écriture ; la
   traduction de chaque `ErreurEcriturePrompt` (des bras inatteignables ? un bras `Err(e)` qui avale un cas qui
   mériterait une réponse distincte ?) ; la durée mesurée dans la fermeture d'audit.
2. **`services/poignees.rs`** (`EcriturePrompts::proposer`) — une transaction, l'ordre dépôt → audit → `commit`, un
   `?` qui laisserait la transaction ouverte ou validée à moitié ; le doc-comment de la précondition.
3. **`services/serveur_mcp.rs`** (`IssuePrompt`, `PromptJournalise`, `entree_prompt`, `journaliser_prompt`) — le
   détail exact `{cleMcpId, prompt, numero, issue}` ; le nom écrit vient-il **toujours** du catalogue, jamais du
   client ?
4. **`mcp/`** (`repartiteur.rs`, `prompts.rs`, `reponse.rs`, `textes.rs`, `outil.rs`, `mod.rs`) — la branche de
   `proposer_prompt` dans `traiter_appel` (avant le registre, après `kesh_lire_envoi` : un nom du registre peut-il la
   masquer, ou l'inverse ?), `Journal::DejaEcrit` pour un accusé seulement, `depuis_refus_de_proposition` exhaustif,
   les variantes `MotifArgument::Reserve` et `ErreurOutil::PropositionsTropNombreuses` (C-32-10a-100) et leurs textes ;
   le journal de `prompts/get` **avant** la réponse, l'erreur interne si le journal échoue ; `AccuseProposition` (champ
   privé, ni `Debug` ni `Clone`).
5. **`routes/mcp_prompts.rs`, `lib.rs`** — les sept routes dans le bloc `admin_routes` ; chaque écriture : validation
   avant transaction, dépôt, audit (action **littérale**), `commit` ; `revalider` (D2) ; `detail_du_refus` (F8) ;
   `version_detail` et `detail` (D1) ; les codes HTTP (404, 409, 400, 201, 200) de chaque refus du dépôt ; deux bras
   rendus en `AppError::Internal` (« inatteignables ») : le sont-ils ?
6. **Les tests** (`tests/mcp_prompts_e2e.rs` à partir de « Story 32-10a-iii », `tests/mcp_prompts_admin_e2e.rs`,
   `tests/mcp_traduction.rs`, `tests/mcp_transport_e2e.rs` — `tools_list_filtre`, `proposer_prompt_hors_registre` —,
   les registres `admin_pat_denied_e2e.rs`, `audit_route_registry.rs`, `services_motif.rs`, `mcp_gardes_lexicales.rs`)
   — un test peut-il passer **à vide** : une attente calculée par la fonction qu'elle éprouve
   (`texte_accuse_proposition`, `marque_agent`, `rendre_visible` sont-ils aussi épinglés par un littéral ?), une capture
   sans témoin, un refus obtenu pour une autre raison que celle visée, le test de concurrence
   `plafond_juste_apres_l_attente_du_verrou` qui passerait si la proposition n'attendait pas le verrou ?

## Lentille E — Edge cases

1. **Les arguments de `proposer_prompt`** : `arguments` absent, non objet, `nom`/`description`/`texte` d'un autre type,
   un champ en trop, un nom de 64 et 65 caractères, `kesh`, `kesh-`, `keshx`, `Kesh-x`, un nom existant **standard**,
   un nom existant **d'administrateur** (l'agent peut-il ajouter une version à valider sur le prompt d'un
   administrateur ? est-ce voulu, C-32-10a-27 ?), un nom existant **retiré**. Un texte de 16 000 caractères après
   normalisation et au-delà avant.
2. **Le plafond et la concurrence** : vingt propositions en attente et une vingt et unième ; deux propositions
   simultanées au dix-neuvième ; une proposition pendant une approbation ; une proposition pendant une remise à zéro
   (`reset_demo` prend `companies`) — l'ordre des verrous (société d'abord) tient-il partout sur ces chemins ?
3. **Les routes** : approuver une version qui n'est plus la dernière ; une version `standard` ou `admin` ; une version
   déjà décidée ; un numéro inexistant ; une `version` négative, nulle, très grande ; un retrait portant un texte ; un
   texte sans description ; le retour au standard d'un standard retiré du code (`STANDARDS` sans lui) ; un prompt d'une
   autre société sur **chaque** route (404 au même corps ?). D2 : une proposition restaurée portant `\r`, U+202E,
   U+E0100, une description sur deux lignes, une clé auteur nulle (la contrainte l'interdit — que fait le code si la
   base le permettait ?). D1 : une version de retrait, une version refusée, une version d'agent dont la clé est
   révoquée (`cleRevoquee`), un standard personnalisé (`standardDiffere`, `standardDuCode`).
4. **Les erreurs et le journal** : `audit_log` indisponible pendant une proposition (rien d'écrit ?), pendant un
   `prompts/get` (rien ne part ?), pendant une route d'administration ; une erreur de base dont le message porterait
   une valeur ; la tâche de l'appel interrompue.
5. **Les mutations** (`/home/gcorbaz/devel/kesh-gate-logs/3210a-iii-mut-*.log`, `3210a-iii-mutations.log`) : chacune
   tuée par une **assertion nommée** et pour la raison visée (lis le journal, pas seulement le verdict) ? Celles qui
   survivent ou sont tuées par une autre voie que la visée (défense doublée, contrainte de la base) sont-elles
   déclarées ? Une mutation plausible **non jouée** survivrait-elle — `exiger` déplacé après la désérialisation, le
   contenu brut enregistré au lieu du normalisé, `Journal::DejaEcrit` rendu pour un refus, le descripteur listé avant
   `kesh_lire_envoi`, `detail_du_refus` qui teste l'état de la seule version refusée, la route de détail qui compose avec
   `valider_contenu` au lieu de `composer_version` ? Raisonne sur le code, n'exécute rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-iii1 à AC-iii13, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve,
**en tenant compte des amendements de l'étape 0** (D1, D1 étendu et ses compléments, D2 ; `proposer` reçoit
`&CleMcpAuthentifiee` ; `entree_appel` avec `envoi: None` ; la 32-8b a déjà livré `ErreurOutil::LimiteAtteinte`, d'où
C-32-10a-100). En particulier :

- **AC-iii1** : `EcriturePrompts` sans `pool()`, une seule méthode ; `FABRIQUES_V2` et `MOTIF_V2` amendés ; les
  doc-comments de `acces.rs`, `Trace::pool`, `services/mod.rs`.
- **AC-iii2** : la liste exacte de ce que la iii change dans le code de la 32-6a — `registre.rs` et `politique.rs`
  **non touchés** (`git diff origin/main...HEAD --stat -- crates/kesh-api/src/mcp/registre.rs
  crates/kesh-api/src/mcp/politique.rs` vide ?) ; aucune ligne neuve de `src/mcp/` ne nomme `AppError`,
  `IntoResponse` ni `into_response` ; `registre_coherent` et `registre_standard_inventaire` inchangés.
- **AC-iii3 à AC-iii5** : chaque cas de la fiche couvert par un test à **corps égal** ; une ligne d'audit par appel,
  détail égal ; `proposition_et_audit_dans_la_meme_transaction` ; le journal de `prompts/get` fail closed ; l'issue
  `limite_atteinte`.
- **AC-iii6 à AC-iii10** : les sept routes, leurs codes ; les gardes (Comptable 403, PAT 403 aux deux portées — par
  `admin_pat_denied_e2e`, chaque compteur recompté —, clé MCP 401, anonyme 401) ; les messages par `t(…)` au repli
  égal à la valeur fr-CH, clés dans les **quatre** locales (`grep -c '^mcp-prompt-' crates/kesh-i18n/locales/*/messages.ftl`) ;
  l'audit (six actions et un type à `audit_labels.rs`, triés ; libellés dans les quatre locales ; le commentaire de
  compte de `fr-CH/messages.ftl` recompté) ; le registre des routes (cinq entrées `Traced`, compteurs et généalogies).
- **AC-iii11 à AC-iii13** : (a) à (f) de la version approuvée par mégarde ; `textes_jamais_journalises` et son
  témoin ; l'inventaire P1 à P27 — chaque site de la iii fermé par ce que la fiche nomme, ou écrit comme angle mort.
- **Les notes d'étape 0** : D1 (le texte servi égal octet pour octet à `prompts/get`, par la **même** fonction,
  `composer_version` ; `servable` = son verdict ; aucun type de `kesh_db` vers `src/mcp/`), D2 (le même code que la
  création), la précondition de lecture (prouvée par un test de concurrence ? la mutation qui ajoute une lecture
  avant le verrou est-elle tuée ?), la clé de la société (#783 point 4 : tenue par construction de
  `CleMcpAuthentifiee` — est-ce suffisant ?).
- **Les choix C-32-10a-100 et suivants** : chacun est-il justifié ; l'un d'eux contredit-il un AC, la fiche ou une
  décision de Guy — en particulier la documentation laissée hors de la partie iii (CHANGELOG, Pattern 5 de
  `docs/MULTI-TENANT-SCOPING-PATTERNS.md`, compte des routes du manuel) ?
- **Le Dev Agent Record** : chaque nombre (tests, mutations, gates, E2E) se recompte-t-il depuis sa source, avec son
  périmètre ?

## Axe du manuel

La section *Les prompts du serveur MCP* du manuel d'administration est la 32-10b ; la 32-10a-iii ne touche ni le
manuel ni le CHANGELOG (choix déclaré). Vérifie **contre le code**, PDF compris, que rien de publié ne **devient faux**
avec la iii : `grep -niE "prompt|propos|routes à verbe mutant|aucune écriture" docs/manual/fr/*.tex CHANGELOG.md
README.md website/*.html`, puis les **PDF aplatis** (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s
' ' | grep -oiE ".{80}(prompt|routes à verbe mutant).{120}"`, idem `user-manual.pdf`). En particulier : le compte « N
des M routes à verbe mutant » du manuel d'administration (D5 : déjà faux avant la iii) ; `README.md` et
`website/roadmap.html` (« aucune écriture par l'agent », confiés à la 32-11) ; l'entrée du CHANGELOG sur les prompts
(dit-elle encore vrai, maintenant qu'un agent peut proposer ?). Un texte publié rendu faux par la iii et laissé sans
porteur nommé est un finding.

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/3210a-iii-review-p1-B.md` (lentille
B), `/home/gcorbaz/devel/kesh-gate-logs/3210a-iii-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/3210a-iii-review-p1-A.md` (lentille A), selon la lentille qui t'est confiée — par
l'outil d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et
   ligne, la preuve (sortie de commande copiée, `grep -nF` pour toute présence ou absence), la correction proposée, et
   s'il est **né du développement** de la 32-10a-iii ou **antérieur**.
2. **L'énumération de ce qui sort et de ce qui s'écrit** (axe obligatoire), champ par champ, écriture par écriture.
3. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
4. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
5. Les fichiers créés par erreur, s'il y en a.
