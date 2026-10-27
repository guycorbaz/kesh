# Prompt — validation P5, Story 32-11 (brancher un agent, de bout en bout)

*Versionné le 2026-10-27. Passe **ciblée** (cinquième passe sur huit) : **une seule lentille, regression hunter**, **Sonnet**,
contexte frais, braquée sur le **seul commit** de la remédiation de la P4. Lecture seule stricte, **sans interpréteur**,
**sans fichier temporaire**, **sans redirection**.*

Dépôt `/home/gcorbaz/devel/kesh-32-11`, branche `story/32-11`. Fiche :
`_bmad-output/implementation-artifacts/32-11-brancher-un-agent.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3211.md` ; epic :
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ 32-11, § *Release v0.16.0*). **La remédiation à relire** :
`git show HEAD --stat`, puis `git diff HEAD~1 HEAD -- <fichier>` (fiche, registre, epic), l'entrée « validation P4 » du
Change Log, et le rapport de la P4 : `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p4.md` (0/0/2/6). Contexte : la spike
`_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (l. 83-97) ; `docs/agents-orchestration.md` (§ 2.1, § 4) ;
`docs/manual/fr/admin-manual.tex` (l. 2928-2990). Règles : `CLAUDE.md` (§ *La passe ciblée*, *Propagation post-patch*,
*Recompter ses propres comptes rendus*). Les numéros de ligne sont ceux de HEAD ; la P4 les citait à `fe66c4f8`.

## L'unique lentille — les régressions de la remédiation P4

Le motif mesuré du dépôt : *la sévérité se déplace vers ce qu'on vient d'écrire*, et la remédiation P3 a **recyclé** (deux
MEDIUM nés d'elle). La P4 a donc **retiré plutôt que corrigé**. Pour chacune des remédiations ci-dessous, **lis le site
remédié** (`git diff HEAD~1 HEAD`), dis si la correction **tient**, et si elle **a introduit un défaut** (contradiction avec
un autre site de la fiche, du registre ou de l'epic ; nouvelle affirmation fausse ou non vérifiable ; citation que le
fichier ne porte pas) :

1. **A1 — AC-D4 (CHANGELOG), retrait.** L'entrée ne nomme plus Claude Code et ne dit rien d'« éprouvé » ; ce qu'elle
   affirme (manuel d'administration, parcours joué sur une démonstration, restauration comprise, test de contrat en CI,
   manuel utilisateur, `docs/api-external.md`) est-il vrai au tag, selon les AC-A, B, C, E ? La phrase sur Claude Code,
   renvoyée à la release par l'orchestrateur (AC-A8 ; epic § *Release v0.16.0*, annotation datée), est-elle écrite de façon
   concordante dans les deux sites, sans contradiction avec AC-C2, AC-C3, C-32-11-29 ? Le titre « Brancher un agent sur
   Kesh » concorde-t-il avec le titre de la fiche et celui de la section des manuels (AC-C2) ?
2. **A2 — annotation datée.** L'entrée P4 du Change Log corrige-t-elle exactement les trois affirmations fausses de
   l'entrée P3, **sans réécrire celle-ci** (`git diff` : une ligne ajoutée au tableau, aucune retirée) ? Ses propres
   affirmations sont-elles vraies (greppe-les) ? La ligne datée ajoutée au registre (fin de fichier) et l'annotation de
   l'epic sont-elles de simples ajouts (aucune ligne retirée) ?
3. **A3 — clé d'amorçage.** `amorcage/cle-amorcage.entete` : nommé en P0 (a), AC-A4 et « La clé dans les journaux » ;
   cohérent avec les Dev Notes indicatives (bloc de commandes), le point 1 et la révocation de P0 (d) ?
4. **A4 — T-1.** `/tmp/kesh-e2e-3211-t8/…`, `kesh-gate-logs/3211/export/` et `t8/` : concordent avec AC-A4 et AC-A6 ?
5. **A5, A6 — faits 13 et 14.** Fait 13 : rejoue la commande bornée et compare à ce que la fiche cite. Fait 14 : l'AC-A8 et
   l'AC-C2 demandent-ils bien le relevé de la version à la recette ?
6. **A7 — K5 et K20.** Le tableau (cellules K5 et K20) et la règle de rangement de l'AC-B2 se contredisent-ils encore sur
   `CLAUSE` / `DECISION_KESH` ? Relis la spike (l. 83-97) : K5 et K20 y portent-ils un MUST ? D'autres sites écrivent-ils
   K5 ou K20 en `CLAUSE` (AC-B3, AC-B5 « clauses de K1 à K3, K5 à K7 et K14 », mutations M6, cas sans mutation) ?
7. **A8 — AC-E2.** `<X_R>` lu « à l'écran » : concorde avec l'interdit des Dev Notes (requêtes admises), AC-A1 et T-8 ?
8. **Propagation** : greppe les **valeurs, un mot à la fois** (jamais une phrase qu'un retour à la ligne peut couper) —
   `Claude Code`, `éprouvée`, `éprouvé`, `prévue`, `démonstration`, `cle-amorcage`, `export/`, `t8`, `K5`, `K20`,
   `claude --version` — sur la fiche, le registre et l'epic, et classe chaque occurrence (résidu, relevé daté, exclusion).
9. **Historique et décomptes** : aucune entrée passée réécrite ; décomptes recomptés par `grep -c` (33 AC : A 8, B 6, C 9,
   D 5, E 3, T 2 ; 11 tâches ; 22 mutations ; 22 cas K ; 12 gestes ; 9 requêtes ; 16 faits ; 5 points ; 29 choix ;
   4 modules) ; trend de l'entrée P4 (P3 0/0/1/13 → P4 0/0/2/6) cohérent avec le rapport de la P4.
10. **Manuels** : `docs/manual/fr/*.tex` et les PDF aplatis (`pdftotext … - | tr '\n' ' ' | tr -s ' '`) contredisent-ils
    ce que le CHANGELOG corrigé affirme aujourd'hui ?

## Ce que tu rends

Findings `A1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site, preuve (commande et sortie copiée ;
`grep -nF` pour toute affirmation de présence ou d'absence), correction proposée. Bilan d'en-tête **cohérent avec le
corps**. ⛔ **Liste des axes exercés ET non exercés** (1 à 10) — sans elle, « 0 finding » ne compte pas. Rapport écrit
**d'un seul jet**, avec l'outil d'écriture de fichier : `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p5.md`. Dernier
message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ **Rappel : les P1 à P4 ont enfreint ces interdits** (`claude --version`, `awk`, substitution de processus, `2>&1`,
redirection vers `/dev/null`, `2>&1 | head`). **`claude` et `socat` ne se lancent sous aucune forme**, pas même `--version`
ou `--help` ; **aucune redirection**, ni `>`, ni `>>`, ni `2>`, ni **`2>&1`**, ni `2>/dev/null`, ni `> /dev/null`, ni
`<(…)`, ni `| tee` ; une erreur de commande se lit telle qu'elle s'affiche.

⛔ N'écris aucun fichier hors de ton rapport, aucun fichier temporaire, rien dans le scratchpad. Aucun
`git checkout`/`switch` ; aucune commande qui écrit, compile, exécute ou joint le réseau : `scripts/*` (dont
`scripts/prepare-release.sh` et `scripts/test-fast.sh`), `make`, `latexmk`, `cargo`, `npm`, `docker`, `curl`, `wget`,
`socat`, `claude`, `python3`, `python`, `perl`, `ruby`, `node`, `awk`, `jq` et tout interpréteur, `mariadb`/`mysql`,
`sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push|worktree`, toute commande `gh` qui écrit. Aucune donnée
réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal et les autres worktrees sont en lecture seule. Autorisés :
lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`, `pdftotext … -`, `gh issue view`, et les filtres
`tr`, `sort`, `cut`, `head`, `tail`, `wc`, `uniq` vers la sortie standard.

## Annexe — le prompt de l'orchestrateur pour la remédiation de la P4 (recopié)

> Tu es l'agent de spécification de la Story 32-11 de Kesh. Tu travailles **uniquement** dans le worktree `/home/gcorbaz/devel/kesh-32-11`, sur la branche `story/32-11`, et tu écris en français.
>
> **Consignes** — applique `consignes-agents-epic15.md` : chemins absolus ; le dépôt principal est en lecture seule ; ni push, ni `gh` qui écrit ; ni cargo, ni npm, ni docker, ni curl, ni `claude`, ni `socat` ; si une commande est refusée, tu t'arrêtes ; n'ouvre jamais `~/kesh-test/`.
>
> **Tâche** — remédie la validation P4 de la fiche `32-11-brancher-un-agent.md`. Le rapport est dans `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p4.md` (0/0/2/6) ; lis-le en entier. Nous sommes à la passe 4 sur 8 et la remédiation P3 a recyclé : déclare-le au Change Log (D5). **Retire plutôt que corriger.**
>
> **Décisions de l'orchestrateur :**
> - **A1, retrait.** L'AC-D4 ne nomme plus du tout Claude Code dans l'entrée du CHANGELOG. L'entrée décrit ce que la 32-11 livre et qui est vrai au tag : le fonctionnement du serveur MCP décrit aux manuels, le test de contrat en CI, et la restauration jouée. La phrase sur Claude Code, « éprouvée » ou « non éprouvée », **s'écrit à la release, par l'orchestrateur, après la recette de Guy**. Ajoute-le à l'AC-A8 et à la ligne de l'epic, § *Release v0.16.0*, sous la forme d'une annotation datée.
> - **A2.** Mets une annotation datée dans l'entrée P4 neuve, pour corriger les trois affirmations fausses de l'entrée P3, sans la réécrire. Ajoute aussi une ligne datée qui amende C-32-11-29 (« déjà prévue »). **Grep des valeurs** : cherche un mot à la fois (`démonstration`, `prévue`, `éprouvée`), pas une phrase, qu'un retour à la ligne peut couper.
> - **LOW** : A3 : nommer le fichier d'en-tête à P0, à l'AC-A4 et dans « La clé dans les journaux » ; A4 : ajouter à T-1 les dossiers neufs, `t8/` et `export/` ; A5 : corriger la sortie citée au fait 13 ; A6 : le relevé de la version de Claude Code est demandé par l'AC-C2, à la recette ; A7 : K5 et K20 passent sous `DECISION_KESH` « à retrouver en MUST sur la page » dans le tableau ; A8 : nommer la requête permise pour lire `<X_R>` sur la seconde instance.
>
> **Après le patch** : greppe les valeurs mot par mot, recompte à la source, ne réécris aucune entrée passée, et ajoute l'entrée P4 avec le trend 0/0/1/13 → 0/0/2/6.
>
> **Livrables** : un commit `refs #723`, sans mot-clé de fermeture ; le prompt de la P5, `3211-validate-prompt-p5.md` (passe ciblée, une lentille *regression hunter*, **Sonnet**, contexte frais, sur le seul commit de cette remédiation, rapport `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p5.md`, mêmes interdits dont toute redirection, ce prompt-ci recopié en annexe) ; un rapport final bref.
