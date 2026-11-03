# Prompt — validation P1 de la fiche v016-1 (correctif 0.16.1 : #811, #812), deux lentilles, Sonnet

Tu valides la **fiche** de la Story v016-1 de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11) : le
correctif 0.16.1 du serveur MCP. **#811 (P2)** : Claude Code 2.1.312 ne transmet au modèle que `structuredContent` quand
il est présent ; la relecture d'un envoi validé n'y mettait pas le texte validé, si bien qu'aucune donnée validée
n'atteignait l'agent. La fiche retient de **retirer `structuredContent` de tout résultat de `tools/call`**, pose un
invariant de contrat (K23) et une garde lexicale. **#812 (P4)** : les montants sortaient tantôt `"0.0000"`, tantôt `"0"` ;
la fiche retient une forme normalisée à au moins deux décimales. S'y ajoutent les retraits de « non éprouvé » au manuel
d'administration, d'après la recette du 2026-10-28.

Worktree `/home/gcorbaz/devel/kesh-0161`, branche `story/v016-1-mcp-sorties`, partie de `main` `43926368`. Langue du
rapport : **français**. Modèle : **Sonnet**. Passe **complète** (première passe de validation). **Contexte frais** : ne
t'appuie sur aucune passe antérieure, ni sur ce que la fiche dit d'elle-même.

## À lire

1. La fiche `_bmad-output/implementation-artifacts/v016-1-mcp-sorties.md`, **en entier**.
2. Le registre `_bmad-output/implementation-artifacts/v016-1-choix-autonomes.md` (C-v016-1-1 à C-v016-1-10).
3. Les issues, **en lecture seule** : `gh issue view 811 --json title,body,comments`, idem 812, et les commentaires de
   #723 (`gh issue view 723 --json comments --jq '.comments[-3:][]|.body'`) — la recette Claude Code et ses observations
   (a) à (h).
4. Le code : `crates/kesh-api/src/mcp/` (en particulier `reponse.rs`, `politique.rs`, `prompts.rs`, `sortie.rs`,
   `textes.rs`, `repartiteur.rs`), `crates/kesh-api/tests/mcp_contrat.rs`, `tests/mcp_gardes_lexicales.rs`,
   `tests/common/mcp.rs`, `tests/common/mcp_pieges.rs` et les tests `tests/mcp_*.rs` que la fiche nomme.
5. Le **manuel** : `docs/manual/fr/admin-manual.tex`, § `sec:mcp` à `sec:mcp-brancher` (l. 3037-3346) et
   `sec:maj-0-16-0` (l. 2480) ; le PDF `docs/manual/fr/admin-manual.pdf` est celui de la 0.16.0 (la story ne le
   régénère pas) — lis-le aplati, si besoin, par `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`,
   **vers la sortie standard seulement**.
6. Pour contexte : `_bmad-output/planning-artifacts/epic-32-spike-mcp.md` § 2.4 et § 5.3.1 ; les fiches
   `32-8-validation-avant-envoi.md`, `32-9-outils-de-lecture.md`, `32-11-brancher-un-agent.md` ; `CLAUDE.md` du
   worktree (*Review Iteration Rule*, *Issue Tracking Rule*, *Règle de commit et push*, liste de contrôle pré-release).

## Deux lentilles, en contexte frais chacune

L'orchestrateur lance **deux agents distincts** sur ce même prompt ; chacun ne tient **que** sa lentille.

### Lentille R — la règle et sa preuve

- **Les prémisses** : chaque ligne citée par la fiche et le registre (`fichier:ligne`) dit-elle ce que la fiche lui fait
  dire ? Le défaut de #811 est-il bien de la classe décrite (S1 à S5), et la liste des sites qui produisent un `result`
  de `tools/call` (S1 à S9) est-elle **close** ? Cherche un site oublié (`grep -rn "\"content\"" crates/kesh-api/src/mcp`,
  les retours de `appeler` et `traiter_appel`, toute autre construction de `ReponseMcp`).
- **Le choix C-v016-1-1** : retirer `structuredContent` partout est-il compatible avec la spécification citée (relis la
  phrase SHOULD et la clause *Output Schema*) ? Casse-t-il une garantie posée par l'Epic 32 (E1 de la spike, AC-B2 de la
  32-8, C-32-9-6) ? Un consommateur réel de `structuredContent` existe-t-il dans le dépôt (frontend compris) ?
- **L'invariant (AC-3)** : est-il formulé de façon décidable ? K23 couvre-t-il chaque site joignable, ou un site reste-t-il
  sans test (le Dev Agent Record doit le dire site par site) ? Les attendus de K23 sont-ils des **littéraux** écrits dans
  le test, jamais tirés de `kesh_api::mcp::textes` ni des aides `attendu_*` ? Le geste prescrit pour chaque site (S6 :
  décision `refuse`, S9 : `{"envoi": 0}`, …) produit-il réellement ce site, au code ?
- **La garde (AC-4)** : son motif, sa fonction `sites`/`est_commentaire`, ses témoins, son angle mort (M5) — tient-elle ce
  que la fiche dit ? Un doc-comment ou un test **dans `src/`** (`mod tests`) portant la chaîne la ferait-il rougir à tort ?
- **#812 (AC-6, AC-7)** : la règle `normalize` puis `rescale(2)` perd-elle une décimale réelle dans un cas ? (Relis
  `rust_decimal` du `Cargo.lock`, `~/.cargo/registry/src/*/rust_decimal-1.43.0/src/decimal.rs`, `normalize`, `rescale`.)
  L'inventaire « tout décimal passe par `Montant` » est-il vrai (`ChampSur`, `sortie.rs`) ? Le test e2e est-il **rouge
  avant** le correctif pour la bonne raison ? Le zéro négatif est-il constructible comme la fiche le suggère ?
- **Les mutations (M1 à M10)** : chacune rougit-elle le test annoncé, et seulement pour la raison annoncée ? Une
  régression probable reste-t-elle sans mutation ?
- **La propagation** : `grep -rn "structuredContent" crates/ docs/ README.md CHANGELOG.md website/` — chaque occurrence
  est-elle traitée par la fiche (§ *Sites de test à ajuster*, AC-1, AC-8) ?

### Lentille F — les textes et ce qu'ils affirment

- **Le manuel (AC-8)** — axe obligatoire. Pour chaque ligne du tableau : l'observation invoquée est-elle **dans** le
  commentaire de recette de #723, telle que la fiche la cite ? Un « non éprouvé » est-il retiré sans observation (par
  exemple ce que montre `/mcp`, ou le contenu reçu à la relecture) ? Un « non éprouvé » est-il gardé alors que la recette a
  vu ? La forme du prompt (observation (g)) est-elle recopiée exactement ? Une phrase du manuel actuel, **hors** du
  tableau, devient-elle fausse après le correctif (cherche « structur », « en-tête », « commence par », « deux
  représentations », la forme des montants, au `.tex` **et** au PDF aplati) ?
- **C-v016-1-7** : la fiche a-t-elle raison de dire que la recette n'a observé aucune réponse sans `structuredContent` ?
  (Tu peux lire, en lecture seule, `/home/gcorbaz/devel/kesh-gate-logs/3211/capture-recette.log`.) La conduite prescrite à
  la release (point 6, puis point 9 sur `closes #811`) est-elle cohérente avec le manuel et le CHANGELOG prescrits ?
- **CHANGELOG (AC-9)**, **README (AC-10)**, **§ Passer à la 0.16.1** : chaque affirmation prescrite est-elle vraie au code
  (aucune migration, retour à la 0.16.0, `.keshbackup` compatible) ? La forme `## [0.16.1] — Non publié` est-elle celle
  qu'attend `scripts/prepare-release.sh` (lis-le, ne l'exécute pas) ?
- **La liste de la release** : manque-t-il un geste que la liste de contrôle pré-release de `CLAUDE.md` exige pour une
  version sans migration (macros, URL `raw.githubusercontent`, PDF, site, décompte des issues) ? La fiche prescrit-elle
  au développeur un geste de release qu'elle dit réservé à l'orchestrateur ?
- **Les décomptes** de la fiche et du registre (57 lignes `Montant(`, trente et un appels de `structure`, 25 colonnes
  `DECIMAL(19,4)`, numéros de ligne) : recompte-les depuis la source.
- **Les règles du dépôt** : aucun mot-clé de fermeture prescrit dans un message de commit ; données réelles absentes ;
  périmètre (quatre modules) ; aucun défaut voisin intégré qui ne soit mécanique ; un défaut voisin constaté (par exemple
  la forme des montants de l'API REST) est-il bien renvoyé à une issue plutôt qu'intégré ?

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, au chemin de ta lentille :
`/home/gcorbaz/devel/kesh-gate-logs/v0161-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/v0161-validate-p1-F.md` (lentille F). Il contient :

1. **Les findings**, numérotés `R1`, `R2`… ou `F1`, `F2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`),
   le lieu (`fichier:ligne` de la fiche, du registre ou du code), **la ligne lue citée** (sortie `grep -nF` ou extrait
   copié) qui fonde l'affirmation, et le remède proposé. Une affirmation de présence ou d'absence sans sortie `grep -nF`
   copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés** de ta lentille, un par un, avec, pour chaque axe exercé, ce que tu as
   lu ou cherché. Un « 0 finding » sans cette liste ne clôt rien et ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au
  rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit (`issue create|edit|close|comment`, `pr …`) ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `make`, ni `latexmk`, ni
  `python3`, ni `node`, ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
