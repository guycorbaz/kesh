# Prompt — validation P2, Story 32-11 (brancher un agent, de bout en bout)

*Versionné le 2026-10-27. Passe **complète** (deuxième passe), après la remédiation de la P1. **Deux lentilles, R et F**,
chacune en **contexte frais**, **Opus**. Lecture seule stricte, **sans interpréteur**, **sans fichier temporaire**,
**sans redirection**.*

Dépôt `/home/gcorbaz/devel/kesh-32-11`, branche `story/32-11` (partie de `origin/main` `ed981c7d` : 32-1 à 32-6c et 32-8
fusionnées ; 32-7, 32-9a, 32-9b, 32-10a, 32-10b **non** fusionnées). Fiche :
`_bmad-output/implementation-artifacts/32-11-brancher-un-agent.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3211.md` (C-32-11-1 à 27) ; § *32-11* de
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (l. 516-546, annotation de la P1 comprise) ; clé
`32-11-brancher-un-agent` de `_bmad-output/implementation-artifacts/sprint-status.yaml`. **La remédiation à relire** : le
dernier commit de la branche (`git show HEAD --stat`, puis `git diff HEAD~1 HEAD -- <fichier>`), son entrée « validation
P1 » au Change Log de la fiche, et les rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p1-R.md`,
`/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p1-F.md`. Contexte à lire : la spike
`_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (§ 1.1, 1.3, 2, 2.1, 2.4, 6) ; la fiche 31-4
`_bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md` (périmètre, T8, mutations, C-31-4-15) et la
rétrospective `epic-31-retro-2026-10-24.md` (§ 3.2, § 7.2) ; les fiches des stories non fusionnées, **en lecture seule
dans leurs worktrees** : `/home/gcorbaz/devel/kesh-32-7/_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md`
(AC-18, AC-20, AC-21, demandes à l'orchestrateur, C-32-7-13, C-32-7-22 dans `epic-32-choix-autonomes-327.md`),
`/home/gcorbaz/devel/kesh-32-9/_bmad-output/implementation-artifacts/32-9-outils-de-lecture.md` (l. 340-360, 1320-1380),
`/home/gcorbaz/devel/kesh-32-10a/_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md` (l. 1380-1400),
`/home/gcorbaz/devel/kesh-32-10b/_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md` (AC-7, AC-9, AC-19,
AC-20) ; dans ce worktree, `32-6-endpoint-mcp.md` (l. 1259-1323), `32-8-validation-avant-envoi.md` (l. 1425-1443) et
`epic-32-choix-autonomes-328.md` (C-32-8-26, C-32-8-27). Règles : `CLAUDE.md` (§ *Review Iteration Rule* et *La passe
ciblée*, *Inventorier les sites NON RÉSOLUS*, *Règle de splitting préventif*, *Le prompt d'une passe doit NOMMER le
manuel*, *Recompter ses propres comptes rendus*, *Recette visuelle*, *Test Locally First*), `docs/agents-orchestration.md`.

**Contraintes de la story, à tenir pour vraies et à contrôler** : dernière story de l'Epic 32, **développée après la
fusion de toutes les autres** ; **documentation exécutée** — le périmètre du parcours est **clos** (AC-A2), un défaut
révélé **ne se corrige pas** dans la story (AC-A7), **aucune garde de texte nouvelle** hors du compte des routes
(AC-T1) ; **Hermes hors du périmètre** (seul Claude Code est nommé) ; **aucun texte ne promet une écriture** ; **aucune
donnée réelle** (démonstration seulement) ; la vérification de `X-Forwarded-For` derrière DSM se fera chez Guy, la story
n'en affirme rien ; au plus cinq modules. Ce que la fiche **prescrit** n'existe pas encore : son absence n'est pas un
défaut. Ce qu'elle dit du code de stories non fusionnées se recontrôle à leur étape 0 : vérifie qu'il figure bien au
§ *À recontrôler à l'étape 0*, non qu'il soit vrai aujourd'hui.

## Les deux axes obligatoires, pour les deux lentilles

- ⛔ **Chaque ligne citée se rouvre.** Pour chaque `fichier:ligne` de la fiche et du registre (faits 1 à 16, AC, Dev
  Notes, § *À recontrôler*), ouvre le site (`sed -n`, `grep -nF`) et **copie la sortie**. Une citation que le fichier
  ne porte pas est un finding. Les comptes écrits se **rejouent** : 130/124/4/2 (`sed -n 877,930p
  crates/kesh-api/tests/audit_route_registry.rs`), « 122 des 128 » (`grep -nF '122 des 128'
  docs/manual/fr/admin-manual.tex`), `grep -rn -il contrat crates/kesh-api/tests/mcp_*.rs`, `grep -n 'jsonschema'
  Cargo.lock`, `grep -n -i mcp README.md website/*.html docs/api-external.md docs/manual/fr/*.tex`.
- ⛔ **Les manuels, `.tex` et PDF aplati.** Pour chaque phrase que la fiche dit vraie ou fausse aujourd'hui (faits 9 à
  12) : le `.tex` **et** `pdftotext docs/manual/fr/<m>.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oE '.{60}<motif>.{80}'`,
  sortie copiée — au moins pour `MCP`, `122 des 128`, `Outil du serveur MCP appelé`, `MCP server`.

## Lentille R — la spécification contre le code, les fiches et l'epic

Axes, tous à exercer :

1. **Les faits 1 à 16** : chacun est-il vrai à `ed981c7d` (ou, pour une fiche voisine, à la ligne citée) ? En
   particulier le fait 3 (aucun test ne porte le contrat ; les tests existants comparent à ce que Kesh a décidé) et le
   fait 6 (Kesh ne journalise pas la révision demandée) : cherche **tout** test ou évènement qui les contredirait.
2. **La couverture de l'epic** : chaque élément du § 32-11 de l'epic (parcours, contrat en CI, `api-external.md` § 6.4
   et § 9, manuels admin et utilisateur, brochure, restauration jouée, description détaillée, `README.md:233`,
   `website/roadmap.html:375`) a-t-il son AC ? Chaque report des fiches voisines (32-6 B4/A3, 32-7 C-32-7-22 et demande 11,
   32-8 observation (d), 32-9 Q2 et demande 2, 32-10a l. 1388-1394, 32-10b AC-19/20) est-il repris, ou écarté par un choix
   écrit ? Un report oublié est un finding.
3. **La cohérence avec les décisions de Guy et les choix voisins** : Hermes (epic l. 36) ; « par construction » retiré
   (spike, réponse Q11) ; validation par clé (Q12) ; `sec:mcp-client` propriété de la 32-7 (C-32-7-22) ; C-32-8-26 (« la
   32-11 ajustera les textes ») contre C-32-11-3 — l'écart est-il déclaré et posé à Guy ?
4. **Le manuel prescrit** (AC-C1 à AC-C9, AC-E3) : chaque affirmation que la fiche fait écrire est-elle sourcée au code
   ou renvoyée à l'étape 0 ? Une seconde copie d'une section d'une autre story (matrice, configuration, écran des envois)
   est-elle prescrite quelque part ? La table de l'AC-C6 place-t-elle chaque limite au bon endroit, et l'une d'elles
   est-elle déjà dite au manuel aujourd'hui (`sec:mcp-envois`, `:2928-2969`) ? Les motifs de l'AC-C9 se trouveraient-ils
   dans un PDF aplati (apostrophe typographique, coupure, `L.agent`) ?
5. **Les textes publics** (AC-D1 à AC-D5) : les réécritures prescrites sont-elles vraies pour le code de l'epic (clé
   dispensée, `proposer_prompt`) ? Une autre phrase de `README.md`, `website/*.html`, `docs/api-external.md` ou de la
   brochure devient-elle fausse avec la 0.16.0 sans être prise en charge ni écartée ?
6. **Les décomptes** : recompte à la source — 33 AC (A 8, B 6, C 9, D 5, E 3, T 2), 11 tâches, 18 mutations (+ MT1,
   MT2, MF1), 22 cas K, 12 gestes P (P0 à P11) et 2 conditionnels, 16 faits, 5 points pour Guy, 27 choix, 4 modules — par `grep -c` sur la fiche et le
   registre, sortie copiée. La clé de `sprint-status.yaml` se lit-elle (`clé: valeur  # commentaire`, deux espaces
   d'indentation, aucune ligne voisine touchée : `git diff ed981c7d -- _bmad-output/implementation-artifacts/sprint-status.yaml`) ?

## Lentille F — faisabilité, simplicité, tests et fausses prémisses

1. **Le parcours est-il jouable tel qu'écrit ?** Le montage (AC-A1 : binaire du worktree, `socat`, ports, bases) ;
   `claude -p` avec `--mcp-config`, `--strict-mcp-config`, `--allowedTools`, `--output-format stream-json`, `--resume`
   (la fiche les renvoie à `claude --help` à l'étape 0 : est-ce suffisant, ou une affirmation non sourcée s'est-elle
   glissée ?) ; la poignée en mode non interactif (P6 → P7 → P8) ; l'`Origin` à travers le mandataire (Dev Notes) ; la
   restauration (AC-E2). Un geste du périmètre clos est-il impossible à jouer par un agent, sans être marqué (AC-A8) ?
2. **Le périmètre est-il vraiment clos ?** Une phrase de la fiche ouvre-t-elle un geste hors de P1 à P11 (« ou ce que
   l'agent choisit », P-pièges, T-8) sans borne ? Une remédiation future aurait-elle une porte pour ajouter outillage,
   script ou garde — le motif de recyclage de la 31-4 ?
3. **Le test de contrat** (AC-B1 à AC-B6) : chaque cas K1 à K22 est-il réalisable avec `tests/common/mcp.rs` tel qu'il
   est (lis-le) ? L'oracle (AC-B2) est-il **réellement** indépendant de Kesh — un cas dont l'attendu ne pourrait venir que
   de la sortie de Kesh, ou d'un texte de `mcp::textes`, est-il un finding ? K10 (clé requise puis relecture) est-il
   faisable sans écran (dépôt de la 32-8) ? K21 est-il décidable ? Le témoin de la fixture (AC-B5, MF1) suffit-il à
   empêcher une fixture inventée ?
4. **Les mutations M1 à M16, MT1, MT2, MF1** : chacune est-elle réalisable sur le code de `mcp/` (lis `repartiteur.rs`,
   `reponse.rs`, `transport.rs`, `politique.rs`) et tuée par le cas **nommé** ? Une passerait-elle à vide (M5 si aucun
   cas ne lit les en-têtes ; M12 si `enveloppe_conforme` n'est pas appelée par K6) ? Une propriété d'un AC sans mutation ?
5. **La garde du compte** (AC-T1) : réalisable dans `audit_route_registry.rs` avec `#[path = "common/manuel.rs"] mod
   manuel;` (lis `common/manuel.rs` et la partition) ? La normalisation `N` laisse-t-elle les deux phrases lisibles
   (`\textbf`, `~`, coupure de ligne) ? Le retrait pré-déclaré est-il bien écrit ?
6. **Les points pour Guy** : chacun est-il posé en question, avec une recommandation et son coût ? En manque-t-il un
   (consommation du compte de Guy, refus de permission, image publiée, NAS) ? L'un est-il déjà tranché par une décision
   citée dans l'epic ou une fiche voisine ?
7. **Le découpage** : quatre modules comptés — un fichier prévu hors de ces modules (frontend, `kesh-i18n`, code de
   production) est-il rendu nécessaire par un AC ?

## Axe commun aux deux lentilles — les régressions de la remédiation P1

⛔ La P1 a été remédiée par l'agent de spécification ; ses patches n'ont été relus par personne. Le motif mesuré du
dépôt est que *la sévérité se déplace vers ce qu'on vient d'écrire* (`CLAUDE.md`, § *La passe ciblée*). Pour chaque
finding de la P1 (R1 à R9, F1 à F13), lis le site remédié et dis : (1) la correction tient-elle ; (2) a-t-elle introduit
un défaut — en particulier :
- **P0** (C-32-11-25) : la route, le corps, les comptes et la portée de la clé sont-ils vrais au code
  (`crates/kesh-api/src/lib.rs`, `docs/api-external.md`, `crates/kesh-core/assets/charts/pme.json`,
  `crates/kesh-seed/src/lib.rs`) ? Les soldes attendus sont-ils justes (recalcule-les) ? Le témoin positif de l'AC-A5
  (b) prouve-t-il ce qu'il dit, ou un `grep` peut-il encore passer à vide ? `socat` est-il bien lancé après P0 partout
  où la fiche en parle (AC-A1, Dev Notes) ? P0 rouvre-t-il le périmètre (une remédiation future y trouverait-elle une
  porte) ?
- **La consigne de P6, les sessions S1 à S4 et le nombre d'envois** : cohérents entre AC-A2, AC-A4, AC-A5, AC-A6,
  AC-E2, les Dev Notes et le point 1 pour Guy ? P5 (c) en mode `-p` et P4 « lu, non demandé » sont-ils jouables ?
- **Le montage du client** (P3, C-32-11-26, C-32-11-5 amendé) : une affirmation sur le comportement de `claude`
  s'est-elle glissée sans être renvoyée à l'étape 0 ? La déclaration des écritures de `claude` dans le magasin de Guy
  contredit-elle `CLAUDE.md` ou `docs/agents-orchestration.md` sans le dire ?
- **L'oracle** (AC-B2, AC-B3, table K, C-32-11-27) : chaque assertion est-elle rangée sous `CLAUSE` ou `DECISION_KESH`
  sans contradiction entre l'AC-B2, la table et les mutations (M4, M5, M10) ? M17 et M18 prouvent-elles réellement
  l'interdiction des aides `attendu_*` (lis `crates/kesh-api/src/mcp/textes.rs` et `tests/common/mcp.rs`) ? K21 et son
  angle mort sont-ils décidables ?
- **Les réfutations écrites au Change Log** (R7 en partie, F3 sur l'approbation) : sont-elles justes au dépôt et aux
  fiches citées ?
- **Les décomptes et les citations nouvelles** : chaque `fichier:ligne` ajouté par la remédiation se rouvre ; aucune
  entrée passée du Change Log n'a été réécrite (`git diff HEAD~1 HEAD` sur la fiche : seule une entrée est ajoutée à la
  fin du tableau) ; l'annotation de l'epic n'en réécrit pas le texte d'origine.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt ou de la fiche
voisine, preuve (commande et sortie copiée ; `grep -nF` pour toute affirmation de présence ou d'absence), correction
proposée, et, pour chacun, **s'il naît de la remédiation P1** ou d'un défaut d'origine. Un bilan d'en-tête **cohérent
avec le corps**. ⛔ **La liste des axes exercés ET non exercés** (R1 à R6 ou F1 à F7, l'axe des régressions et les deux
axes obligatoires) — sans elle, « 0 finding » ne compte pas comme passe. Rapport écrit **d'un seul jet**, un seul
fichier par lentille, avec l'outil d'écriture de fichier :
`/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p2-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p2-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ **Rappel : la P1 a enfreint ces interdits** (la lentille R a lancé `claude --version`, un `awk` redirigé vers
`/dev/null` et une substitution de processus `<(…)`). **`claude` et `socat` ne se lancent sous aucune forme**, pas même
`--version` ou `--help` ; **aucune redirection**, ni `>`, ni `>>`, ni `2>`, ni `2>/dev/null`, ni `<(…)`, ni `| tee` ;
une affirmation sur le comportement de `claude` se source à la fiche, à la spike ou à la documentation lue, et se
déclare « à constater à l'étape 0 ».

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile, exécute ou
joint le réseau : `scripts/*` (dont **`scripts/prepare-release.sh`** et **`scripts/test-fast.sh`**), `make`, `latexmk`,
`cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, `wget`, `socat`, **`claude`**, **`python3`, `python`, `perl`,
`ruby`, `node`, `awk`** et tout autre interpréteur, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push|worktree`, et **toute commande `gh` qui écrit** (`pr`, `issue
create|edit|close|comment`, `api` en écriture) — la lecture d'une issue (`gh issue view <n>`) est permise. `pdftotext`
vers la sortie standard seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal
`/home/gcorbaz/devel/kesh` et les autres worktrees sont **en lecture seule**. Autorisés : lecture, `grep`, `sed -n`,
`git log|show|diff|blame|merge-base`, `pdftotext … -`, `gh issue view`, et les filtres en lecture seule `tr`, `sort`,
`cut`, `head`, `tail`, `wc`, `uniq` vers la sortie standard.

## Annexe — le prompt de l'orchestrateur pour la remédiation de la P1 (recopié tel quel)

> Tu es l'agent de spécification de la Story 32-11 de Kesh, « brancher un agent, de bout en bout ». Tu travailles **uniquement** dans le worktree `/home/gcorbaz/devel/kesh-32-11`, sur la branche `story/32-11`, et tu écris en français.
>
> ## Consignes
> Applique `/home/gcorbaz/.claude/projects/-home-gcorbaz-devel-kesh/memory/consignes-agents-epic15.md` :
> - chemins absolus ;
> - le dépôt principal est en lecture seule ;
> - ni push ni `gh` qui écrit ; la lecture d'issues est permise ;
> - ni cargo, ni npm, ni docker, ni curl, ni `claude`, ni `socat` : c'est une fiche ;
> - si une commande est refusée, tu t'arrêtes ;
> - n'ouvre jamais `~/kesh-test/` ;
> - Hermes est hors périmètre.
>
> ## Tâche
> Remédier la validation P1 de la fiche `_bmad-output/implementation-artifacts/32-11-brancher-un-agent.md`. Les rapports sont dans `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p1-R.md` (0/1/1/7) et `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p1-F.md` (0/0/4/9). Lis-les en entier, vérifie chaque prémisse au dépôt, et écris au Change Log ce que tu réfutes.
>
> **Décisions de l'orchestrateur :**
>
> - **R1 (HIGH : la démonstration n'a aucune écriture).** Ajoute un geste d'amorçage nommé **P0** au périmètre clos. Il saisit, par l'API de Kesh avec une clé d'API de l'administrateur, quelques écritures synthétiques notées dans la fiche (montants, comptes, dates dans l'exercice de la démonstration), pour que le bilan ait de la matière. Un **témoin positif** prouve que le contrôle de l'AC-A5 voit ces montants ; sans lui, le contrôle passe à vide. Choisis le moyen le plus simple qui existe déjà au code (route des écritures) et cite-le.
>
> - **F1 (P6 → P8 et l'identifiant d'exercice).** La consigne donnée à l'agent d'essai fournit l'identifiant de l'exercice, lu à l'écran ou par l'API à P0. Le parcours ne fait alors qu'**un** envoi validé pour le bilan. Écris le nombre exact d'envois attendus.
>
> - **F2 (isolation du client).** Écris le montage le plus simple qui tienne :
>   - lancement depuis un répertoire temporaire **neuf, hors de tout dépôt**, pour que le `CLAUDE.md` de Kesh ne soit pas chargé ;
>   - `--strict-mcp-config` avec un fichier MCP jetable ;
>   - pas de dossier de configuration jetable, puisqu'il retirerait l'authentification. L'authentification de Guy reste donc, et les sessions s'écrivent dans son magasin habituel : écris-le, et retire « ne jamais écrire dans `~/.claude*` » ;
>   - un plafond de coût : nombre de tours ou budget, selon ce qu'offre `claude -p`.
>
>   Tout ce qui touche au comportement de `claude` est **à constater à l'étape 0** (version, options), puisque personne ne l'a lancé. La Q1 pour Guy reste ouverte, et la fiche dit ce qu'elle coûte.
>
> - **F3 (périmètre qui se contredit).** La liste des gestes est **la seule** source :
>   - P5 nomme ses sous-gestes, à savoir créer une version personnalisée et l'approuver ;
>   - le montage de l'AC-A8 y figure comme geste ;
>   - les variantes des Dev Notes sont retirées ou nommées dans la liste.
>
>   Recompte à la source.
>
> - **F4 et R2 (oracle du contrat).**
>   - Tout cas qui vérifie un choix de Kesh (session facultative et M5, clés closes K1, K7 et K14, `isError` K8 et M10, K10) est marqué `DECISION_KESH`, et non `CLAUSE`.
>   - Les attendus du contrat sont des **littéraux** écrits dans le test.
>   - Les aides `attendu_*` de `tests/common/mcp.rs` sont **interdites** au contrat, avec une garde ou une mutation qui le prouve.
>   - Ce qui touche au schéma officiel est à constater à l'étape 0, sur la page de la spécification.
>
> **Les LOW :** intègre-les tous, entre autres :
> - les mutations M11 et M16 ;
> - K21, dont la branche « sinon » devient un angle mort si elle est inatteignable ;
> - MF1 ;
> - `configuration_transmise.rs` et les deux gardes de `textes_coherents.rs` au gate ciblé ;
> - `KESH_TEST_MODE` et le build du frontend au montage ;
> - le retrait d'AC-T1 ;
> - K10 ;
> - `website/roadmap.html:43`, que tu ajoutes au point 5 pour Guy, à côté de `README.md` ;
> - R6, c'est-à-dire le `pdftotext` sous UTF-8 et la césure ;
> - R7 : nomme ce que le manuel dit déjà de #765 et #773 ;
> - R8 : corrige l'epic, l. ≈521-522 de `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md`. La configuration de Claude Code appartient à la 32-7 ; la 32-11 l'éprouve et l'amende en place. Annote, sans réécrire l'historique ;
> - R9.
>
> **Écarts des lentilles à mentionner au Change Log :** la lentille R a lancé `claude --version` (sortie `2.1.311`), un `awk` vide et une substitution de processus. Sans effet sur la fiche, mais la version est à reconfirmer à l'étape 0.
>
> **Après le patch :** greppe les valeurs, recompte à la source, ne réécris aucune entrée passée, et ajoute l'entrée P1 avec le trend 0/1/5/16.
>
> ## Livrables
> - Un commit `refs #723`, sans mot-clé de fermeture.
> - Le prompt de la P2, `_bmad-output/implementation-artifacts/3211-validate-prompt-p2.md` :
>   - lentilles R et F, **Opus**, contexte frais ;
>   - rapports dans `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p2-{R,F}.md` ;
>   - mêmes exigences et interdits que la P1, avec `claude`, `socat` et toute redirection explicitement interdits ;
>   - un axe sur les régressions de cette remédiation ;
>   - ce prompt-ci en annexe.
> - Un rapport final bref.
