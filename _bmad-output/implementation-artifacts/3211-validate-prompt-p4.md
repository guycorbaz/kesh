# Prompt — validation P4, Story 32-11 (brancher un agent, de bout en bout)

*Versionné le 2026-10-27. Passe **ciblée** (quatrième passe sur huit) : **une seule lentille, regression hunter**, **Opus**,
contexte frais, braquée sur le **seul commit** de la remédiation de la P3. Lecture seule stricte, **sans interpréteur**,
**sans fichier temporaire**, **sans redirection**.*

Dépôt `/home/gcorbaz/devel/kesh-32-11`, branche `story/32-11`. Fiche :
`_bmad-output/implementation-artifacts/32-11-brancher-un-agent.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3211.md` ; epic :
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ 32-11, § *Release v0.16.0*). **La remédiation à relire** :
`git show HEAD --stat`, puis `git diff HEAD~1 HEAD -- <fichier>` (fiche et epic), l'entrée « validation P3 » du Change Log,
et les rapports de la P3 : `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p3-R.md` (0/0/1/6),
`/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p3-F.md` (0/0/0/7). Contexte : la spike
`_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (l. 85-96, 160-162) ; `docs/agents-orchestration.md` (§ 2.1, § 4) ;
`crates/kesh-api/src/mcp/reponse.rs` (l. 60-70) et `repartiteur.rs` (l. 73-77) ; `docs/manual/fr/admin-manual.tex`
(l. 2928-2990). Règles : `CLAUDE.md` (§ *La passe ciblée*, *Propagation post-patch*, *Recompter ses propres comptes rendus*).

## L'unique lentille — les régressions de la remédiation P3

Le motif mesuré du dépôt : *la sévérité se déplace vers ce qu'on vient d'écrire*. Pour chacune des remédiations ci-dessous,
**lis le site remédié** (`git diff HEAD~1 HEAD`), dis si la correction **tient**, et si elle **a introduit un défaut**
(contradiction avec un autre site de la fiche, du registre ou de l'epic ; nouvelle affirmation fausse ou non vérifiable ;
citation que le fichier ne porte pas) :

1. **R1 — AC-D4 (CHANGELOG)** : la phrase n'affirme plus de « joué » que ce que la fiche joue (le parcours en client HTTP) ;
   « la configuration de Claude Code est décrite au manuel, non éprouvée avant la recette » concorde avec AC-C2, AC-C3, AC-A8.
2. **R2 — epic** : la ligne datée de § *Release v0.16.0* et l'annotation de la § 32-11 ; le texte d'origine est-il laissé
   intact (`git diff` : aucune ligne retirée) ? « inscrite à la release » (point 1) concorde-t-il avec C-32-11-29 et AC-A8 ?
3. **R3 = F7 — écritures** : Dev Notes (dossiers `/tmp/kesh-e2e-3211*`, § 4), AC-A4 (`export/`), clé d'amorçage par fichier
   d'en-tête (`amorcage/cle-amorcage.entete`) ; cohérence avec AC-A1, AC-E2, P0, « La clé dans les journaux » et le
   point 1 (dérogation § 2.1).
4. **R4 = F2 — AC-A5 (c)** : « bloc texte de données = second bloc de `result.content` » contre `reponse.rs:60-70`
   (rouvrir) ; cohérence avec K10 et AC-B2.
5. **F3 (R07 après R06)**, **F4** (interdit couvrant P0 et `GET /health`), **F5** (fait 14, Dev Notes), **F6** (`t8/`
   propre, cohérence avec AC-A1, AC-E2, T-8) : chaque site relu.
6. **R5, R6 = F1, R7** : fait 13 (commande bornée — rejoue-la), fait 6 (C-32-11-29), AC-B2 (« en MUST » contre la spike),
   AC-A8 (masquage de la clé de la capture).
7. **Propagation** : greppe les **valeurs** (pas les phrases) sur la fiche et le registre — `joué sur une démonstration`,
   `reconstater`, `écrites avant`, `reponses-t8`, `déjà prévue`, `claude --version`, `C-32-11-6`, `cle-amorcage`,
   `export/` — et classe chaque occurrence (résidu, relevé daté, exclusion).
8. **Historique et décomptes** : aucune entrée passée du Change Log réécrite (`git diff HEAD~1 HEAD` : une ligne ajoutée au
   tableau, aucune retirée) ; registre inchangé ; décomptes recomptés par `grep -c` (33 AC : A 8, B 6, C 9, D 5, E 3,
   T 2 ; 11 tâches ; 22 mutations ; 22 cas K ; 12 gestes ; 9 requêtes ; 16 faits ; 5 points ; 29 choix ; 4 modules) ;
   trend de l'entrée P3 (0/0/9/13 → 0/0/1/13) cohérent avec les bilans des rapports.
9. **Manuels** : le prompt d'une passe sur du texte visible nomme le manuel — vérifie que le CHANGELOG corrigé ne promet
   rien que `docs/manual/fr/*.tex` et les PDF aplatis (`pdftotext … - | tr '\n' ' ' | tr -s ' '`) contredisent aujourd'hui.

## Ce que tu rends

Findings `A1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site, preuve (commande et sortie copiée ;
`grep -nF` pour toute affirmation de présence ou d'absence), correction proposée. Bilan d'en-tête **cohérent avec le
corps**. ⛔ **Liste des axes exercés ET non exercés** (1 à 9) — sans elle, « 0 finding » ne compte pas. Rapport écrit
**d'un seul jet**, avec l'outil d'écriture de fichier : `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p4.md`. Dernier
message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ **Rappel : les P1, P2 et P3 ont enfreint ces interdits** (`claude --version`, `awk`, substitution de processus,
`2>&1`, redirection vers `/dev/null`, `2>&1 | head`). **`claude` et `socat` ne se lancent sous aucune forme**, pas même
`--version` ou `--help` ; **aucune redirection**, ni `>`, ni `>>`, ni `2>`, ni **`2>&1`**, ni `2>/dev/null`, ni
`> /dev/null`, ni `<(…)`, ni `| tee` ; une erreur de commande se lit telle qu'elle s'affiche.

⛔ N'écris aucun fichier hors de ton rapport, aucun fichier temporaire, rien dans le scratchpad. Aucun
`git checkout`/`switch` ; aucune commande qui écrit, compile, exécute ou joint le réseau : `scripts/*` (dont
`scripts/prepare-release.sh` et `scripts/test-fast.sh`), `make`, `latexmk`, `cargo`, `npm`, `docker`, `curl`, `wget`,
`socat`, `claude`, `python3`, `python`, `perl`, `ruby`, `node`, `awk`, `jq` et tout interpréteur, `mariadb`/`mysql`,
`sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push|worktree`, toute commande `gh` qui écrit. Aucune donnée
réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal et les autres worktrees sont en lecture seule. Autorisés :
lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`, `pdftotext … -`, `gh issue view`, et les filtres
`tr`, `sort`, `cut`, `head`, `tail`, `wc`, `uniq` vers la sortie standard.

## Annexe — le prompt de l'orchestrateur pour la remédiation de la P3 (recopié)

> Tu es l'agent de spécification de la Story 32-11 de Kesh, dans `/home/gcorbaz/devel/kesh-32-11`, branche `story/32-11`, en français. Consignes : `consignes-agents-epic15.md` (chemins absolus, dépôt principal en lecture seule, ni push ni `gh` qui écrit, ni cargo/npm/docker/curl/`claude`/`socat`, arrêt si une commande est refusée, jamais `~/kesh-test/`).
>
> Tâche : remédier la validation P3 (rapports `3211-validate-p3-R.md` 0/0/1/6 et `-F.md` 0/0/0/7), passe 3 sur 8, en changeant le moins de texte possible.
>
> Décisions : **R1 (MEDIUM, CHANGELOG)** l'AC-D4 ne promet pas un branchement de Claude Code « joué » ; il décrit ce que la 32-11 livre, plus la phrase « la configuration de Claude Code est décrite au manuel, non éprouvée avant la recette ». **R2** ligne datée dans l'epic § *Release v0.16.0* (recette de Guy : capture des requêtes de Claude Code et rejeu des gestes interactifs depuis le PDF, AC-A8) et annotation de la § 32-11 sans réécrire l'historique. **R3 = F7** Dev Notes : dossiers `/tmp/kesh-e2e-3211*` (§ 4) ; AC-A4 fixe la destination du `.keshbackup` de P10 sous `kesh-gate-logs/3211/` ; la clé d'amorçage passe par un fichier d'en-tête. **R4 = F2** AC-A5 (c) : c'est le bloc texte de données de R07 qui égale le texte validé, outil de décodage nommé. **F3** R07 écrite après R06. **F4** la Dev Note couvre P0 et `GET /health`. **F5 = fait 14** retirer « `claude --version` à reconstater ». **F6** le rejeu T-8 écrit sous `kesh-gate-logs/3211/t8/`. **R5, R6 = F1, R7** faits 13 et 6 corrigés ; AC-B2 « MUST » seulement pour ce que la spike cite comme tel ; secret de la clé dans `capture-recette.log` à masquer avant partage. Écarts des lentilles (redirection interdite chacune) : à mentionner au Change Log.
>
> Après le patch : grep des valeurs, recompte à la source, aucune entrée passée réécrite, entrée P3 au Change Log (trend 0/0/9/13 → 0/0/1/13).
>
> Livrables : un commit `refs #723` sans mot-clé de fermeture ; le prompt de la P4 (`3211-validate-prompt-p4.md`, passe ciblée, une lentille regression hunter, Opus, contexte frais, sur le seul commit de cette remédiation, rapport `3211-validate-p4.md`, mêmes interdits dont toute redirection, ce prompt en annexe) ; un rapport final bref.
