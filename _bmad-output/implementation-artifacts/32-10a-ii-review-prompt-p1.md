# Prompt — revue de code P1 de la Story 32-10a-ii (le catalogue des prompts servi par MCP), trois lentilles, Opus

Tu relis le développement de la **partie ii de la Story 32-10a** de Kesh (comptabilité suisse, Rust/Axum, MariaDB
10.11) : *le catalogue des prompts que le serveur MCP sert par `prompts/list` et `prompts/get` — la poignée
`LecturePrompts` (sans pool visible, sur le pool en lecture seule du MCP), le service `services::prompts_mcp`
(`exiger` puis lecture, revalidation de chaque version par `valider_contenu`, composition de la marque d'un prompt
d'agent), `PromptEnBase` seule implémentation du trait scellé `McpPrompt`, les charges `ListePrompts` et `PromptRendu`
de `ReponseMcp`, le répartiteur, et les gardes lexicales.* Les tables, le dépôt et l'amorçage des standards sont la
32-10a-i (fusionnée) ; l'outil `proposer_prompt`, les routes de gestion, le journal de `prompts/get` et l'audit sont
la 32-10a-iii ; l'écran et la section du manuel *Les prompts du serveur MCP* sont la 32-10b.

Worktree `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-10a-ii`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-32-10a diff origin/main...HEAD`. Langue du rapport : français. Modèle : **Opus**.
Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Cette partie porte une surface d'injection** : tout texte qui sort par `/mcp` **instruit un agent** qui lit des
livres réels. Le critère n'est pas « ça marche », c'est « rien d'autre ne sort ».

⚠️ **Hors périmètre** : un défaut que la 32-10a-ii ne touche pas est un finding **antérieur**, à classer comme tel.
Mais **un besoin de la iii que la ii ne couvre pas** est un finding de la ii — en particulier l'amendement D1 (la route
de détail de la iii doit rendre le texte servi **calculé par la même fonction** que le catalogue, sans nommer
`crate::mcp`) et le cas du `\r` restauré (§ *À recontrôler à l'étape 0* de la fiche).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md` — § *Le découpage*, **Partie ii** (AC-ii1 à
AC-ii10), § *L'inventaire clos des chemins par lesquels un texte de prompt atteint un agent* (P1 à P27), *Tâches
32-10a-ii*, *Tests et mutations prévus* (Mii2 à Mii13, Mii1 retirée), *Gates*, *Ce que les gardes ne voient pas*,
*Dev Notes* (§ *La marque*), le § *À recontrôler à l'étape 0* **en entier** (notes de l'orchestrateur sur la 32-8b0, la
32-8b et la 32-9 ; amendements D1, D1 étendu et son complément, D2 ; le `\r` restauré) — et le **Dev Agent Record** et
le **Change Log** de la 32-10a-ii ; puis le registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3210a.md`,
**C-32-10a-80 et suivants** (choix du développement) et C-32-10a-10, 12, 14, 15, 16, 30. Le Dev Agent Record déclare
des gates, des mutations et un E2E : **ne les prends pas pour argent comptant**, recoupe-les avec les journaux
`/home/gcorbaz/devel/kesh-gate-logs/3210a-ii-*` (lecture seule). Aucun contrôle visible ne change : pas de capture
attendue.

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
  ni aucun script de `kesh-gate-logs` ou d'un scratchpad (dont `mut-3210a.py` et `run-mut-3210a-ii.sh`) ; ni `make`, ni
  `latexmk`, ni `cargo` (aucune sous-commande, `cargo fmt` compris), ni `rustc`, ni `docker`, ni `mysql`/`mariadb`, ni
  `sqlx`, ni `curl`, ni `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée dans le
  rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques (`PIEGE_*_3210A`). Ne lis rien sous `~/kesh-test/`.

## L'axe obligatoire pour chaque lentille : ce qui sort par `/mcp`

**Énumère, à partir du code et non de la fiche, tout ce que `prompts/list` et `prompts/get` peuvent faire sortir** —
chaque champ de chaque réponse, réussie ou en erreur, et sa source (constante du binaire, colonne de la base, entier
calculé, valeur reçue du client) — puis dis, pour chacun, s'il peut porter : un texte d'une version **non** en vigueur
(à valider, refusée, antérieure, retirée), un texte d'une **autre société**, une **donnée des livres**, un **argument
du client**, un caractère **invisible ou bidirectionnel**, un texte d'agent **sans sa marque**, ou une différence qui
ferait **oracle** (un prompt existant mais non servi distinguable d'un nom inconnu ; une clé sans `books:read`
distinguable). Recoupe avec l'inventaire P1 à P27 de la fiche : un site non résolu que l'inventaire n'énumère pas est
un finding. Fais de même pour le **journal du serveur** (`tracing`) sur ce chemin : aucun évènement ne doit porter une
description ni un texte (`grep -nF "tracing::" crates/kesh-api/src/services/prompts_mcp.rs`, chaque appel lu).

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. **`crates/kesh-api/src/services/prompts_mcp.rs`** — `valider_contenu` : l'ordre normalisation → contrôles, la
   description contrôlée en une ligne (`is_hidden_in_line`), le texte par `is_hidden_in_text`, les bornes en caractères
   **après** normalisation, le rognage (contrôle seulement, le texte n'est pas rogné : est-ce cohérent avec le dépôt ?).
   `composer` : la marque sur la description **et** le texte d'une version d'agent, rien sur `standard` et `admin` ;
   la marque elle-même (entiers seulement). `servir` : une version d'agent **sans** clé auteur ; une erreur de contenu
   journalisée sans texte. `lister`/`obtenir` : `exiger` **avant** toute lecture, le nom validé en Rust avant la
   lecture, le tri. Les types (`PromptServi`, `VersionServie`, `ContenuPrompt`) : champs privés, `Debug` manuel sans
   texte — un chemin par lequel un texte serait recopié dans un `Debug`, une erreur ou un évènement ?
2. **`crates/kesh-api/src/mcp/prompts.rs`, `reponse.rs`, `repartiteur.rs`, `textes.rs`** — la chaîne des types
   (`PromptServi` → `PromptEnBase` → `ListePrompts`/`PromptRendu`) : peut-on construire une charge autrement ? Le trait
   amendé (`rendre()` sans argument, `arguments()`, `portee()`). Le répartiteur : `cursor` refusé, `name` absent ou non
   textuel, `arguments` ignorés, `HorsPortee` → liste vide ou « Prompt inconnu », `Interne` → `-32603` fixe. Les corps
   produits sont-ils conformes au schéma `2025-11-25` (`Prompt` : `name` requis, `description` et `arguments`
   facultatifs ; `GetPromptResult` : `messages` requis, `description` facultative ; `PromptMessage` : `role` ∈
   {`user`, `assistant`}, `content` un `ContentBlock` ; `TextContent` : `type: "text"`, `text`) — source : le dépôt
   `modelcontextprotocol`, `schema/2025-11-25/schema.ts` (dis ce que tu n'as pas pu relire en lecture seule) ?
3. **`crates/kesh-api/src/services/poignees.rs`, `lib.rs`, `mcp/mod.rs`** — `LecturePrompts` sans `pool()`, ses deux
   méthodes `pub(in crate::services)` ; le pool en lecture seule lié **une fois** puis enveloppé trois fois (outils,
   envois, prompts) ; `EtatMcp::lecture_prompts`.
4. **Les tests** (`tests/mcp_prompts_e2e.rs`, l'amendement d'`e1_rien_ne_part_sous_cle_requise` dans
   `tests/mcp_transport_e2e.rs`, les gardes de `tests/mcp_gardes_lexicales.rs` et de `tests/services_motif.rs`, les
   tests unitaires de `prompts_mcp.rs`) — un test peut-il passer **à vide** : une attente calculée par la fonction même
   qu'elle éprouve (la marque, `marque_agent`, est-elle aussi écrite **en toutes lettres** quelque part ?), une capture
   de journal sans témoin, une garde dont le motif ne prend aucune ligne réelle, un refus obtenu pour une autre raison
   que celle visée (le renommage de la table qui rendrait `HorsPortee` pour une mauvaise raison) ?

## Lentille E — Edge cases

1. **Les noms** : `KESH-ASSISTANT`, `kesh-assistant ` (blanc final), `kesh-assistant\n`, un nom de 65 caractères, un
   nom vide, un nom non ASCII qu'une collation `utf8mb4_unicode_ci` apparierait (`kesh-assistánt`) — la validation en
   Rust précède-t-elle **toujours** la lecture ? `name` d'un autre type JSON (nombre, `null`, objet).
2. **Les contenus restaurés ou écrits hors de Kesh** : `\r\n`, `\r` isolé, `\r\n` dans la **description**, un texte à
   16 000 caractères pile après normalisation mais au-delà avant, une description de 500 caractères plus la marque
   (la description servie dépasse-t-elle une borne qui compterait ?), U+202E, U+2066, U+E0100, U+FE0F, U+200D, U+00A0,
   un texte fait de `\n` seulement. Le texte **servi** est-il exactement le texte **contrôlé** (cas du `\r`), et
   `envois_mcp::rendre_visible` du texte servi y montrerait-il un `\u{000D}` ?
3. **La marque** : une version d'agent dont le texte commence par une fausse marque ; une version d'agent approuvée
   puis une version d'administrateur (la marque disparaît-elle ?) ; une version d'agent en vigueur dont la clé a été
   révoquée (servie ? marquée ? est-ce voulu ?).
4. **Les erreurs** : la base indisponible pour `prompts/list` et pour `prompts/get` ; une clé sans `books:read` **et**
   une base en panne (la réponse révèle-t-elle la panne ?) ; `Interne` produit par une version incohérente
   (`DbError::Invariant`) — un seul prompt corrompu fait-il tomber **toute** la liste ? Est-ce le bon choix ?
5. **La concurrence et le pool** : trois enveloppes sur un pool de `MCP_POOL_CONNEXIONS` connexions — un client qui
   enchaîne des `prompts/list` peut-il affamer les outils (délai, `READ ONLY`) ?
6. **Les mutations** (`/home/gcorbaz/devel/kesh-gate-logs/3210a-ii-mut-*.log`, `3210a-ii-mutations.log`) : chacune
   tuée par une **assertion nommée** et pour la raison visée (lis le journal, pas seulement le verdict) ? Les
   reformulations déclarées (Mii5, Mii6) sont-elles fidèles à l'intention de la fiche ? Une mutation plausible **non
   jouée** survivrait-elle — `obtenir` sans `nom_conforme`, le tri retiré, `exiger` déplacé après la lecture, la marque
   sur la seule description, `HorsPortee` traduit en erreur dans `prompts/list` ? Raisonne sur le code, n'exécute rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-ii1 à AC-ii10, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve,
**en tenant compte des amendements de l'étape 0** (la 32-8b0 a livré le prédicat ; la composition vit dans
`services/`, D1 complément ; C-32-8-31). En particulier :

- **AC-ii1** : le prédicat **réemployé** (non recréé) — `git diff origin/main...HEAD -- crates/kesh-core` doit être vide ;
  ce que la fiche demandait de tests est-il présent sur `main` (32-8b0) ou dans cette branche ?
- **AC-ii2, AC-ii3** : une seule fonction de validation ; la description par `is_hidden_in_line` ; le refus énuméré
  sans texte ; `standards_valides` ; les tests R7 (refus avant toute lecture prouvé par la table renommée, contrôle
  positif, exécution sur un pool en lecture seule) ; la garde (vi) de `services_motif.rs` à quinze fichiers.
- **AC-ii4, AC-ii8** : `PromptEnBase` construit seulement à partir d'un `&PromptServi` ; aucune `AppError`, aucun type
  de `kesh_db` dans `src/mcp/` lié aux prompts (`grep -nF "kesh_db" crates/kesh-api/src/mcp/prompts.rs`) ;
  `reponses_par_rendre_seul` inchangée.
- **AC-ii5 à AC-ii7** : chaque cas de la fiche couvert par un test à **corps égal** ; « Prompt inconnu » pour les neuf
  causes de l'AC-ii6 ; la marque de la fiche (§ *Dev Notes — La marque*) et sa mise en œuvre (une ligne, non deux :
  choix déclaré ?).
- **AC-ii9, AC-ii10** : la garde `service_des_prompts_sans_poignee_des_livres` et ses témoins ; `prompts_scelles`
  amendée (formes génériques, qualifiées, coupées après `impl`) ; `prompts_lus_par_leur_service` (deux motifs, mesure
  sur `api_keys`, liste blanche exacte) ; `e1_rien_ne_part_sous_cle_requise` amendé.
- **Les choix C-32-10a-80 et suivants** : chacun est-il justifié ; l'un d'eux contredit-il un AC, la fiche ou une
  décision de Guy ?
- **Le Dev Agent Record** : chaque nombre (tests, mutations, gates, E2E) se recompte-t-il depuis sa source, avec son
  périmètre ?

## Axe du manuel

La section *Les prompts du serveur MCP* du manuel d'administration est la 32-10b ; la 32-10a-ii n'écrit que le
CHANGELOG. Vérifie quand même **contre le code**, PDF compris, que rien de publié ne contredit ce que la ii livre :
`grep -niE "prompt" docs/manual/fr/*.tex CHANGELOG.md README.md website/*.html`, puis les **PDF aplatis**
(`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{80}prompt.{120}"`, idem
`user-manual.pdf`). Le PDF n'a pas été régénéré par cette partie : un texte du `.tex` absent du PDF, ou l'inverse, est
un finding. L'entrée du CHANGELOG dit-elle exactement ce que fait le code (portée, politique de validation, marque,
caractères refusés, arguments ignorés) ?

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/3210a-ii-review-p1-B.md` (lentille B),
`/home/gcorbaz/devel/kesh-gate-logs/3210a-ii-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/3210a-ii-review-p1-A.md` (lentille A), selon la lentille qui t'est confiée — par
l'outil d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et
   ligne, la preuve (sortie de commande copiée, `grep -nF` pour toute présence ou absence), la correction proposée, et
   s'il est **né du développement** de la 32-10a-ii ou **antérieur**.
2. **L'énumération de ce qui sort par `/mcp`** (axe obligatoire), champ par champ.
3. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
4. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
5. Les fichiers créés par erreur, s'il y en a.
