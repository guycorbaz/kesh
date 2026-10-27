# Prompt — validation P6, Story 32-7 (l'écran « Serveur MCP » et la section du manuel)

*Versionné le 2026-10-25. Passe **ciblée** (sixième passe sur huit) : **une seule lentille, « regression hunter »**,
en contexte frais, **Opus**, braquée sur **le seul commit de la remédiation P5** — le commit qui suit `e3c23c6a` sur la
branche `story/32-7` (`git log --oneline -3`). Lecture seule stricte. La P1 (Sonnet ×2) a rendu 0/0/5/19, la P2
(Opus ×2) 0/0/5/25, la P3 (Sonnet ×2) 0/0/2/9, la P4 (Opus ×2) 0/0/3/17, la P5 (Sonnet ×2) 0/0/2/7 — son seul MEDIUM
(R1 = F1, l'exemple d'adresse `127.0.0.1/32`) **naissait de la remédiation P4**. ⚠️ Dès cette sixième passe,
l'orchestrateur ne remédie plus lui-même : il retire ou renvoie au développement (`CLAUDE.md`, *Review Iteration
Rule*, décision Q2 de la rétrospective de l'Epic 31). Un finding qui propose de **retirer** une phrase invérifiable
vaut mieux qu'un finding qui propose de l'enrichir. La boucle ne se clôt que sur une passe sans finding au-dessus de
LOW.*

Dépôt `/home/gcorbaz/devel/kesh-32-7`, branche `story/32-7` (partie de `main` `966baed5`). Fiche :
`_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-327.md` (C-32-7-1 à 27, **C-32-7-27 neuf en P5**).
Rapports P5 : `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p5-R.md` et `-F.md` ; entrée « validation P5 » du
Change Log de la fiche. Le diff à relire :

```sh
git -C /home/gcorbaz/devel/kesh-32-7 diff e3c23c6a HEAD -- \
  _bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md \
  _bmad-output/implementation-artifacts/epic-32-choix-autonomes-327.md
```

Lis-le **en entier**, puis chaque site touché dans son contexte (`sed -n` sur la fiche, une trentaine de lignes autour).
Code de la 32-6a, en lecture seule : `/home/gcorbaz/devel/kesh-32-6/crates/kesh-api/src/mcp/` (`reseau.rs`,
`transport.rs`) et la fiche `/home/gcorbaz/devel/kesh-32-6/_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md`
(point 2). Règles : `CLAUDE.md` (§ *La passe ciblée*, *Le prompt d'une passe doit NOMMER le manuel*, *Propagation
post-patch*, *Recompter ses propres comptes rendus* — et « recompter n'est pas réécrire l'historique »).

**Décisions de l'orchestrateur, acquises** (ne pas les contester ; contrôler qu'elles sont **appliquées sans résidu**) :
- **R1 = F1** : l'exemple `127.0.0.1/32` est **retiré** — de la phrase du `keshwarning` (AC-18 (4)), du motif de
  l'AC-21 (désormais **quinze** motifs sur `admin-manual.pdf`), de *Ce que les gardes ne voient pas* et de la demande 11.
  La phrase dit seulement « l'adresse sous laquelle Kesh voit le mandataire » et ne nomme que les deux cas établis :
  derrière Traefik sur le réseau Docker, une adresse du réseau interne ; derrière une publication de port par le
  mandataire en espace utilisateur, la passerelle Docker (AC-6 (1), fiche 32-6 point 2).
- **F2** : que Traefik pose `X-Forwarded-For` est écrit « non éprouvé », comme pour DSM ; rien d'autre n'est prétendu.
- **F4** : le blocage par une entrée stockée hors de l'écran est un **angle mort écrit** (C-32-7-27), sans test ni
  capture neufs.
- **F5** : un motif `Le serveur MCP` sur `user-manual.pdf` à l'AC-21. **R3** : G3-bis nommée à l'AC-14 (b). **R4** : le
  manuel ne cite pas le titre *Réglages du serveur* (G2), il dit « les réglages du serveur ».

⚠️ **Outillage de ce poste** : `grep` est une fonction qui lance `ugrep`, qui refuse les répétitions bornées larges
(`.{0,80}` → « exceeds complexity limits »). Employer `command grep`, et des fenêtres de 30 au plus.

## La lentille — regression hunter sur la remédiation P5

*La sévérité se déplace vers ce qu'on vient d'écrire* : cherche ce que **ce commit** a cassé, contredit ou affirmé sans
source. Axes, tous à exercer :

1. **Chaque phrase neuve prescrite au manuel ou à l'écran** affirme-t-elle plus que sa source ? En particulier : « une
   adresse de ce réseau interne » derrière Traefik (`docs/manual/fr/admin-manual.tex:398`, `:440`) ; « la passerelle »
   derrière une publication de port en espace utilisateur (AC-6 (1), qui dit « peuvent paraître », et sa note « une
   publication de port ordinaire conserve l'adresse source ») — la phrase neuve est-elle aussi prudente que l'AC-6 (1),
   et cohérente avec la phrase précédente du même `keshwarning` (« il peut voir la passerelle (\texttt{172.17.0.1}) ») ?
   « que Traefik ou le proxy inversé de DSM le posent n'est pas éprouvé » ; « aucun en-tête `X-Forwarded-*` configuré »
   dans `sec:traefik` (l. 398-534) — **sortie `command grep -nF` copiée**.
2. **Résidus de la valeur retirée** : `command grep -nF '127.0.0.1'` et `command grep -nF '127\.0\.0\.1'` sur la fiche et
   le registre ; chaque site restant est-il une annotation, une entrée historique ou un sujet sans rapport (origines
   propres de l'AC-18 (5), `initialize` de l'AC-16) ? Et `seize`, `16 motifs`, `quinze`, `C-32-7-1 à C-32-7-2` :
   chaque site vivant dit-il la valeur à jour, chaque site historique est-il intact ou annoté ?
3. **Le contrôle du PDF (AC-21) — axe obligatoire, PDF compris** : **rejoue** chacun des quinze motifs sur
   `admin-manual.pdf` et le motif de `user-manual.pdf` :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | command grep -cE '<motif>'` (et de même pour
   `user-manual.pdf`), **sortie copiée** ; chacun doit rendre 0. Chaque motif correspond-il à une phrase que la fiche
   prescrit **réellement** d'écrire (sinon il rendrait 0 aussi après la story) — en particulier `Le serveur MCP` au
   manuel utilisateur, contre les renvois de l'AC-19 ? Le décompte « quinze » est-il juste, contre la liste écrite ?
   La phrase neuve du `keshwarning` reste-t-elle couverte par un motif (`ne filtre alors rien`, `proxy inversé de
   DSM`) ? Ce que le PDF publié dit déjà de Traefik, de `X-Forwarded-For` et des réglages contredit-il une phrase neuve :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | command grep -oiE '.{0,30}(Traefik|X-Forwarded-For|Réglages).{0,30}'`.
4. **Les gardes nommées** : G3-bis (`crates/kesh-i18n/src/loader.rs`, doc l. 1077, test l. 1091, liste l. 1092-1107 —
   la fiche corrige le 1096-1110 du rapport R) et G2 (`crates/kesh-api/tests/textes_coherents.rs:153-173`) : lignes
   exactes, ce qu'elles imposent dit juste ? « Aucun texte prescrit n'écrit « Administration → Serveur MCP » dans un
   catalogue » : vrai (`command grep -nF 'Administration →'` sur la fiche, sortie copiée) ? Une phrase prescrite au
   manuel contient-elle « Réglages » capitalisé (`command grep -nF 'Réglages'` sur la fiche) ?
5. **Références croisées et numérotation** : C-32-7-27 (contexte, retenu, écartées, réversible) ; l'en-tête
   (C-32-7-1 à 27) ; les deux points « ajoutés en validation P5 » du § *À recontrôler* ; la demande 11 récrite ; la
   capture (7) et le cas de test (d) cités par l'angle mort ; les renvois aux validations (P1 R5, P4 R2, P5 R1 = F1, F2,
   F4, F5, R3, R4) — tous exacts ?
6. **Historique** : une entrée passée du Change Log ou du registre a-t-elle été **réécrite** au lieu d'être annotée
   (interdit) ? (`git diff e3c23c6a HEAD` : les lignes `-` ne doivent toucher aucune entrée P1 à P4 du Change Log ni
   aucune entrée C-32-7-1 à 26 du registre.)
7. **Décomptes** de l'entrée P5, recomptés à la source : trend P4 0/0/3/17 → P5 0/0/2/7 contre les deux rapports P5
   (doublons R1 = F1 et R2 = F3 comptés une fois par lentille) ; 25 AC, 9 tâches, 26 mutations, 35 reports, **27
   choix**, 29 captures, 21 faits, 4 modules, motifs 15 + 1.
8. **Ce que le commit omet** : un site qui dit encore l'ancienne phrase sous une autre forme (une adresse de boucle
   locale, « mandataire de l'hôte », « exemple » d'adresse), à l'écran (AC-6), au manuel (AC-18), aux mutations, aux
   captures, au registre.

## Ce que tu rends

Findings `P6-1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée — `grep -nF` ou `command grep -nF` pour toute présence ou absence), correction proposée — de préférence
un **retrait**. Pour chaque finding, dis s'il **naît de la remédiation P5** (oui / non). Un bilan d'en-tête
**cohérent avec le corps**. ⛔ **La liste des axes exercés ET non exercés** (1 à 8) — sans elle, « 0 finding » ne compte
pas. Ce que la fiche **prescrit** au développement n'existe pas encore : son absence n'est pas un défaut. Rapport écrit
**d'un seul jet**, par l'outil d'écriture de fichiers : `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p6.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad. **Aucune
redirection** (`>`, `>>`, `2>`, `tee`, `<<`), pas même vers un fichier jetable — un fichier créé par erreur se déclare
au rapport, il ne se supprime pas. **Aucun interpréteur** : ni `python3`, `python`, `perl`, `ruby`, `node`, `deno`, ni
**`awk`**, ni `sed` autrement qu'en `sed -n`. **Aucun `gh` qui écrit** (`pr`, `issue create|edit|close|comment`,
`api` en écriture…) ; seuls `gh issue view 738 --json title,body,comments` et `gh issue view 758 --json title,body`
sont permis. Aucun `git checkout`/`switch`/`commit`/`add`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`. Aucune
commande qui écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, **`scripts/test-fast.sh`**,
**`scripts/mem-guard.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`, Playwright, `docker`, `curl`,
`mariadb`/`mysql`, `sed -i`. `pdftotext` vers la sortie standard seulement (`-` en sortie). Rien de `~/kesh-test/`,
aucune donnée réelle. Le dépôt principal `/home/gcorbaz/devel/kesh` est hors de ton périmètre ; les worktrees voisins
ne se lisent qu'en lecture. Autorisés : lecture de fichiers, `grep`/`command grep`, `sed -n`,
`git log|show|diff|blame|merge-base`, `pdftotext … -`, les deux `gh issue view` ci-dessus, et les filtres `tr`, `sort`,
`cut`, `head`, `tail`, `wc`, `uniq` vers la sortie standard. Si une commande est refusée, arrête-toi et dis-le au
rapport.

---

## Annexe — le prompt de la remédiation P5, recopié pour que la boucle soit rejouable

> Tu es l'agent de spécification de la Story 32-7 de Kesh. Tu travailles **uniquement** dans le worktree
> `/home/gcorbaz/devel/kesh-32-7`, sur sa branche, et tu écris en français.
>
> ## Consignes
> Applique en entier les consignes de `/home/gcorbaz/.claude/projects/-home-gcorbaz-devel-kesh/memory/consignes-agents-epic15.md`. En particulier :
> - chemins absolus ;
> - le dépôt principal est en lecture seule ;
> - ni push, ni `gh` qui écrit, ni `prepare-release` ;
> - ni cargo, ni npm, ni docker, ni curl ;
> - la lecture est permise, ainsi que `pdftotext` vers la sortie standard ;
> - si une commande est refusée, tu t'arrêtes ;
> - n'ouvre jamais `~/kesh-test/`.
>
> ## Tâche
> Remédie la validation P5 de la fiche `_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md`. Les rapports sont `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p5-R.md` (0/0/1/3) et `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p5-F.md` (0/0/1/4) ; lis-les en entier.
>
> C'est la passe 5 sur 8. **La remédiation se fait par RETRAIT** : on ne corrige pas une phrase invérifiable, on la retire.
>
> ### Décision de l'orchestrateur sur le MEDIUM (R1 = F1)
> L'exemple « `127.0.0.1/32` » est une prescription de l'orchestrateur sans source. Il est **retiré** :
> - de la phrase du manuel (AC-18 (4)) ;
> - du motif de l'AC-21, qui passe à quinze motifs ;
> - de chaque site qui le cite.
>
> La phrase dit seulement qu'il faut déclarer « l'adresse sous laquelle Kesh voit le mandataire ». Elle peut nommer les deux cas que le dépôt établit :
> - derrière Traefik sur le réseau Docker, une adresse du réseau interne ;
> - derrière un port publié, la passerelle Docker ;
> - c'est ce que la fiche 32-6 et l'AC-6 (1) disent déjà.
>
> Ce cas est aussi un **recyclage né de la P4** : déclare-le au Change Log.
>
> ### LOW, tous à intégrer
> - **F2** : Traefik pose-t-il `X-Forwarded-For` ? Aucune source ne l'établit ; écris « non éprouvé », comme pour DSM. Ne prétends rien d'autre.
> - **F3 = R2** : corrige l'en-tête de la fiche (26 choix).
> - **F4** : ajoute un test et une capture pour le blocage par une entrée stockée hors de l'écran, ou écris-le comme angle mort. Choisis le plus simple.
> - **F5** : ajoute un motif sur `user-manual.pdf` à l'AC-21, s'il rend 0 à la base ; exécute la commande et cite la sortie.
> - **R3** : nomme la garde G3-bis (`crates/kesh-i18n/src/loader.rs:1096-1110`) parmi les gardes touchées, avec ce qu'elle impose.
> - **R4** : précise que le manuel ne cite pas le titre *Réglages du serveur* (G2, `textes_coherents.rs:153-166`), et donne la formulation à employer.
>
> ### Après le patch
> - Greppe la valeur `127.0.0.1` sur la fiche et sur le registre, ainsi que les décomptes de motifs (`seize`, `16 motifs`). Corrige les sites vivants et annote les sites historiques.
> - Recompte à la source.
> - Ne réécris aucune entrée passée.
> - Ajoute une entrée P5 neuve au Change Log : trend P4 0/0/3/17 → P5 0/0/2/7, en comptant le doublon R1 = F1 une fois par lentille.
>
> ## Livrables
> - **Un commit** `refs #723`, sans mot-clé de fermeture.
> - **Le prompt de la P6**, dans `_bmad-output/implementation-artifacts/327-validate-prompt-p6.md`. C'est une passe **ciblée** : une seule lentille, « regression hunter », sous **Opus**, en contexte frais, braquée sur le seul commit de cette remédiation. Le rapport ira dans `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p6.md`. Le prompt reprend :
>   - les mêmes exigences : axes exercés et non exercés, sorties `grep -nF`, PDF, rejeu des motifs ;
>   - les mêmes interdits.
>
>   Recopie ce prompt-ci en annexe.
> - **Un rapport final.**
