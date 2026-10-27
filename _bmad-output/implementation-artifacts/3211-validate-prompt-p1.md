# Prompt — validation P1, Story 32-11 (brancher un agent, de bout en bout)

*Versionné le 2026-10-27. Passe **complète** (première passe). **Deux lentilles, R et F**, chacune en contexte frais,
**Sonnet**. Lecture seule stricte, **sans interpréteur**, **sans fichier temporaire**, **sans redirection**.*

Dépôt `/home/gcorbaz/devel/kesh-32-11`, branche `story/32-11` (partie de `origin/main` `ed981c7d` : 32-1 à 32-6c et 32-8
fusionnées ; 32-7, 32-9a, 32-9b, 32-10a, 32-10b **non** fusionnées). Fiche :
`_bmad-output/implementation-artifacts/32-11-brancher-un-agent.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3211.md` (C-32-11-1 à 24) ; § *32-11* de
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (l. 516-541, et les notes datées) ; clé `32-11-brancher-un-agent`
de `_bmad-output/implementation-artifacts/sprint-status.yaml`. Contexte à lire : la spike
`_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (§ 1.1, 1.3, 2, 2.1, 2.4, 6) ; la fiche 31-4
`_bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md` (périmètre, T8, mutations, C-31-4-15) et la
rétrospective `epic-31-retro-2026-10-24.md` (§ 3.2, § 7.2) ; les fiches des stories non fusionnées, **en lecture seule
dans leurs worktrees** : `/home/gcorbaz/devel/kesh-32-7/_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md`
(AC-18, AC-20, demandes à l'orchestrateur, C-32-7-13, C-32-7-22 dans `epic-32-choix-autonomes-327.md`),
`/home/gcorbaz/devel/kesh-32-9/_bmad-output/implementation-artifacts/32-9-outils-de-lecture.md` (l. 1320-1380),
`/home/gcorbaz/devel/kesh-32-10a/_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md` (l. 1380-1400),
`/home/gcorbaz/devel/kesh-32-10b/_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md` (AC-19, AC-20) ; dans ce
worktree, `32-6-endpoint-mcp.md` (l. 1259-1323), `32-8-validation-avant-envoi.md` (l. 1425-1443) et
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

- ⛔ **Chaque ligne citée se rouvre.** Pour chaque `fichier:ligne` de la fiche et du registre (faits 1 à 15, AC, Dev
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

1. **Les faits 1 à 15** : chacun est-il vrai à `ed981c7d` (ou, pour une fiche voisine, à la ligne citée) ? En
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
6. **Les décomptes** : recompte à la source — 33 AC (A 8, B 6, C 9, D 5, E 3, T 2), 11 tâches, 16 mutations (+ MT1,
   MT2, MF1), 22 cas K, 11 gestes P, 15 faits, 5 points pour Guy, 24 choix, 4 modules — par `grep -c` sur la fiche et le
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

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt ou de la fiche
voisine, preuve (commande et sortie copiée ; `grep -nF` pour toute affirmation de présence ou d'absence), correction
proposée. Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET non exercés** (R1 à R6 ou
F1 à F7, et les deux axes obligatoires) — sans elle, « 0 finding » ne compte pas comme passe. Rapport écrit **d'un seul
jet**, un seul fichier par lentille, avec l'outil d'écriture de fichier :
`/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p1-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p1-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

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
