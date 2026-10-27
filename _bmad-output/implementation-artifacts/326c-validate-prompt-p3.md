# Prompt — validation P3, Story 32-6c (le journal du filtre d'adresse du serveur MCP)

*Versionné le 2026-10-25. Passe **complète** (troisième passe), après la remédiation de la P2 (trend : P1 0 CRITICAL /
0 HIGH / 5 MEDIUM / 10 LOW → P2 0/0/5/12). **Deux lentilles, R et F**, chacune en contexte frais, **Sonnet** (la P2
était Opus : rotation Sonnet ↔ Opus). Lecture seule stricte, **sans `gh`**, **sans interpréteur**, **sans fichier
temporaire**.*

⚠️ **Signal D5 déclaré au Project Lead** (Change Log, entrée P2) : les dix-sept findings de la P2 naissaient **tous**
de la remédiation P1, et tous portaient sur ce que la fiche fait écrire au manuel. La remédiation P2 a donc **retiré
plutôt qu'enrichi** : une phrase du manuel non éprouvée sur le NAS se retire ou se marque « non éprouvé ». Juge-la à
cette aune : une phrase **ajoutée** par la P2 qui affirme sur DSM, `sudo`, Traefik ou un `sed` ce que personne n'a
éprouvé est un finding ; une phrase **retirée** dont l'absence laisse l'administrateur sans moyen de conclure en est un
aussi.

Dépôt `/home/gcorbaz/devel/kesh-32-6c`, branche `story/32-6c` (partie de `origin/main` `c5287dec`). Fiche :
`_bmad-output/implementation-artifacts/32-6c-journal-filtre-adresse.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-326c.md` (C-32-6c-1 à 13) ; entrées § *32-6c*, § *32-11*
et § *Réponses de Guy (2026-10-25)* de `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` ; clé
`32-6c-journal-filtre-adresse` de `_bmad-output/implementation-artifacts/sprint-status.yaml`. **La remédiation à relire**
est le dernier commit de la branche (`git show HEAD`, `git diff HEAD~1 HEAD`) ; les rapports de la P2 sont
`/home/gcorbaz/devel/kesh-gate-logs/326c-validate-p2-R.md` et `-F.md` (ceux de la P1 : `-p1-R.md`, `-p1-F.md`) — lis-les,
mais **ne tiens pour juste aucune de leurs affirmations sans l'avoir rouverte**. Contexte à lire : la fiche 32-6
`_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md` (AC-a7 à AC-a9, AC-b4, § *Ce que les gardes ne voient
pas*, § *Le manuel (axe de revue)*, points 2 et 2 bis pour Guy) et son registre `epic-32-choix-autonomes-326.md`
(C-32-6-4, 5, 6). Règles : `CLAUDE.md` (§ *Review Iteration Rule* et *La passe ciblée* — une boucle ne se clôt que sur
une passe sans finding au-dessus de LOW —, *Inventorier les sites NON RÉSOLUS*, *Règle de splitting préventif*, *Le
prompt d'une passe doit NOMMER le manuel*, *Recompter ses propres comptes rendus*, *Recompter n'est pas réécrire
l'historique*, *Recette visuelle*), `docs/agents-orchestration.md`.

**Contraintes de la story, à tenir pour vraies et à contrôler** : but **diagnostique** (Guy, 2026-10-25 : « on
vérifiera lors de la mise à jour : ajoute le nécessaire dans les logs au cas où ») ; **aucune clé, aucun en-tête
`Authorization`, aucun corps, aucune donnée comptable** au journal ; **le plus simple qui suffit** ; **un événement par
décision**, `warn` au refus, `debug` à l'admission ; **rien ne change pour le client** (route absente, réponses fixes) ;
**format global du journal inchangé** (C-32-6c-11) ; au plus cinq modules ; aucune donnée réelle (adresses de
documentation ou privées génériques). Ce que la fiche **prescrit** au développement n'existe pas encore : son absence
n'est pas un défaut.

**Décisions de Guy du 2026-10-25, à tenir pour acquises** (fiche, § *Points soumis à Guy*) : 1 refus en `warn` ; 2
formes et présences des en-têtes seulement ; 3 journaliser aussi filtre coupé ; 4 **pas** de conseil `X-Forwarded-For`
à DSM avant vérification ; 5 sous-section du manuel avant la section MCP de la 32-7 — la description détaillée du
fonctionnement de MCP revient à la **32-11** (epic, § 32-11), la 32-6c ne l'écrit pas. Ces décisions ne sont pas des
findings ; leur **transcription** fausse ou incomplète en est un.

**Décisions de l'orchestrateur pour la remédiation P2, à tenir pour acquises** (annexe) : procédure `RUST_LOG` réduite
à « remplacer la ligne du `.env`, `sudo docker compose up -d`, vérifier par les lignes `debug` » ; Traefik hors du cas
(i), en exemple « non éprouvé » du cas (ii) ; énumération des sorties du transport, citées à la ligne ; une seule
commande de lecture avec ses limites en une phrase, repli `sed` retiré. Ces choix ne sont pas des findings ; leur
**exécution** fausse en est un.

## Les trois axes obligatoires, pour les deux lentilles

- ⛔ **Chaque ligne citée se rouvre.** Pour chaque `fichier:ligne` de la fiche, du registre et des passages de l'epic
  touchés (faits 1 à 13, AC, mutations, Dev Notes, § *À recontrôler*, choix C-32-6c-7, 11 à 13), ouvre le site
  (`sed -n`, `grep -nF`) et **copie la sortie**. Une citation que le fichier ne porte pas est un finding. Les comptes
  écrits se **rejouent** : `grep -n "tracing::" crates/kesh-api/src/mcp/reseau.rs crates/kesh-api/src/mcp/transport.rs
  crates/kesh-api/src/services/serveur_mcp.rs` ; `grep -n "RouteAbsente" crates/kesh-api/src/mcp/transport.rs` ;
  `grep -nE 'RUST\\?_LOG' docs/manual/fr/admin-manual.tex`.
- ⛔ **Chaque affirmation sur une dépendance se prouve à sa source**, version du `Cargo.lock`
  (`~/.cargo/registry/src/*/`) : `tracing-subscriber-0.3.23` (`filter/env/directive.rs`, `fmt/format/mod.rs`,
  `fmt/fmt_layer.rs` — la ligne de **code** de l'ANSI et de `NO_COLOR`), `tracing-core-0.1.36/src/field.rs`,
  `tracing-0.1.44/src/macros.rs` (message en tête), `tracing-appender-0.2.5/src/rolling.rs` (nom du fichier en rotation
  `daily` **et** `never`), `http-1.4.2/src/header/value.rs` (validité de `0xE9` et de la valeur vide, `is_sensitive`).
  Une affirmation dont tu ne trouves pas la ligne est un finding.
- ⛔ **Les régressions de la remédiation P2** (axe obligatoire). *La sévérité ne stagne pas, elle se déplace vers ce
  qu'on vient d'écrire* (`CLAUDE.md`, § *La passe ciblée*) — et la P2 l'a mesuré : ses dix-sept findings venaient tous
  de la remédiation P1. Relis **chaque hunk** de `git diff HEAD~1 HEAD` et demande-toi : le correctif d'un finding P2
  en a-t-il introduit un autre ? Points à exercer au moins :
  (a) **`RUST_LOG`** (AC-9, fait 12, C-32-6c-7 annoté) : la procédure « remplacer la ligne `RUST_LOG=info` du `.env`
  (`.env.example:253`), `sudo docker compose up -d`, vérifier par les lignes `DEBUG` de décision » est-elle juste et
  suffisante ? Reste-t-il, à la fiche ou au registre, une phrase sur l'environnement du shell ou de `sudo` ? La
  vérification par les lignes `DEBUG` conclut-elle à coup sûr (et si aucune requête admise n'arrive) ?
  (b) **Traefik** (AC-9 (i) et (ii)) : le cas (i) ne cite-t-il plus que la publication de port en espace utilisateur de
  Docker ? L'exemple Traefik du cas (ii) est-il marqué « non éprouvé » et sa citation de la fiche 32-6
  (`:1017-1019`) juste ? La décision 4 de Guy reste-t-elle tenue (aucun conseil d'en-tête à DSM) ? La réserve (ii)
  enrichie (R7 : « ou, sans en-tête, être vu comme la passerelle », `reseau.rs:76-78`) dit-elle juste ?
  (c) **l'énumération de l'AC-9 (viii)** et le fait 2 : inventorie **toi-même** tous les retours `RouteAbsente` de
  `transport.rs` antérieurs à la décision, et tout chemin amont (`lib.rs`, `routes/mcp.rs`, le repli statique), puis
  compare à la liste écrite (`CLAUDE.md`, *Inventorier les sites NON RÉSOLUS*). La règle qui fait foi est-elle juste ?
  Le rattachement de `transport.rs:87` (erreur de la base) et de `services/serveur_mcp.rs:41` (ligne illisible)
  est-il exact ? La condition de niveau (« le filtre étant en `debug` ») est-elle juste et suffisante ? « Non activé
  pour cette société » est-il vrai dans tous les cas où il est écrit ? L'AC-7, qui renvoie désormais à (viii),
  garde-t-il ce que l'AC-T3 éprouve ?
  (d) **la commande de lecture** : une seule commande reste-t-elle, à la fiche, au registre et à l'epic ? Ses limites
  (rotation `never`, `KESH_LOG_HOST_DIR`, motif développé avant `sudo`) sont-elles exactes au code
  (`tracing-appender-0.2.5/src/rolling.rs`, `docker-compose.yml:140`, `docker-compose.prod.yml:165`) ? Le retrait du
  repli `sed` laisse-t-il un résidu (T-3, § *Ce que les gardes ne voient pas*, AC-11, C-32-6c-11) ou un cas sans
  issue que la fiche devrait au moins nommer ?
  (e) **les LOW intégrés** : fait 13 (`fmt_layer.rs:743`, montage par compose, nom sous `never`), AC-T5
  (`serveur_mcp.rs:31`), M19 (sans `split_terminator`), AC-T4 (`POST`, `Content-Type: application/json`,
  `transport.rs:100`, `:115`) — justes, et sans effet sur la mutation ou le test qu'ils touchent ?
  (f) **le point 1 pour Guy** : le complément P1 est-il maintenant **après** la réponse, présenté comme un ajout de
  l'orchestrateur, sans que la question ni la réponse aient changé ?
  (g) **le Change Log P2 et les annotations du registre** : décomptes justes (recompte), trend « 0/0/5/10 → 0/0/5/12 »
  cohérent avec les deux rapports P2, aucune entrée passée réécrite (l'entrée « création » garde 16 mutations et
  12 cas ; l'entrée P1 garde 0/0/5/10 ; les *Retenu* de C-32-6c-7 et C-32-6c-11 restent tels quels, annotés).
  (h) **l'epic** : le § 32-6c (« fichier de log `./log/kesh.*.log` ») reste-t-il vrai après la remédiation ?

## Lentille R — la spécification contre le code, les fiches et le manuel

Axes, tous à exercer :

1. **Les faits 1 à 13** : chacun est-il vrai au code de `c5287dec` ? En particulier le fait 2 (a) corrigé et son
   complément amont, le fait 12 précisé et le fait 13 précisé.
2. **La cohérence avec la 32-6** : la fiche contredit-elle AC-a7, AC-a8, AC-a9, AC-b4, C-32-6-4 à 6, la décision de
   Guy sur la garde 2 bis, ou ce que la fiche 32-6 dit de Traefik ? L'AC-3 et l'AC-4 laissent-ils la décision
   **exactement** celle de `c5287dec` sur les cas (a) à (s) et (n') ? L'AC-6 est-il vérifiable tel qu'écrit ?
3. **L'évènement** (AC-1, AC-2, AC-5) : la ligne attendue (message en tête, puis onze champs) est-elle celle que la
   capture et le formateur produiront ? Un champ porte-t-il une valeur reçue du client ?
4. **Le manuel — `.tex` et PDF** (axe obligatoire de `CLAUDE.md`) : l'AC-9, l'AC-10 et l'AC-11 décrivent-ils ce que le
   code prescrit fera, **sans promesse non vérifiée** (DSM, `sudo`, Traefik) ? Une phrase existante devient-elle fausse
   avec la story (`:574`, `:621`, `:652-655`, `:696`, `:792`, `:939-960`, `:957`, `:1641`, § `sec:traefik`) ? Contrôle
   le PDF actuel aplati : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE
   ".{60}(X-Real-IP|RUST_LOG|docker compose logs|kesh\.log|Traefik).{80}"` — copie la sortie.
5. **Les décomptes** : recompte à la source — 17 AC (11 + 6), 6 tâches, 21 mutations, 14 cas à l'AC-T1, 13 faits,
   C-32-6c-1 à 13, 3 modules — par `grep -c` sur la fiche et le registre, sortie copiée.

## Lentille F — faisabilité, simplicité, tests et fausses prémisses

1. **Le plus simple qui suffit** : la remédiation a-t-elle élargi ou réduit la story (fichiers, modules,
   configuration) ? Le choix C-32-6c-11 annoté (lire le fichier, repli retiré, `with_ansi(false)` à soumettre si le
   fichier ne suffit pas) tient-il ?
2. **Les tests** (AC-T1 à AC-T6) : chacun est-il réalisable avec `tests/common/capture_trace.rs` et
   `tests/common/mcp.rs` tels qu'ils sont (lis-les) ? Les quatorze cas donnent-ils le verdict et les champs écrits ?
   L'AC-T4 réécrit (`POST`, `Content-Type`) atteint-il bien l'authentification ?
3. **Les mutations M1 à M21** : chacune réalisable et tuée par le test nommé ? Une propriété de l'AC sans mutation ?
4. **La garde des secrets** (AC-5, AC-T4) : ferme-t-elle la classe « valeur d'en-tête au journal » sur ce site ?
5. **Le comportement inchangé et l'oracle** : inchangés par la remédiation ?
6. **Les points soumis à Guy** : les réponses sont-elles transcrites fidèlement, à la fiche, au registre et à l'epic ?
   Un point neuf, né de la remédiation, devrait-il lui être soumis ?

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt ou de la dépendance,
preuve (commande et sortie copiée, `grep -nF` pour toute affirmation de présence ou d'absence), correction proposée ;
pour chaque finding, dis s'il **naît de la remédiation P2** (hunk cité), de la remédiation P1, ou s'il est d'origine.
Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET non exercés** (R1 à R5 ou F1 à F6, et
les trois axes obligatoires, sous-points (a) à (h) compris) — sans elle, « 0 finding » ne compte pas comme passe.
Rapport écrit **d'un seul jet**, un seul fichier par lentille, avec l'outil d'écriture de fichier :
`/home/gcorbaz/devel/kesh-gate-logs/326c-validate-p3-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/326c-validate-p3-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`** et **`scripts/test-fast.sh`**), `make`, `latexmk`,
`cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, `wget`, **`python3`, `python`, `perl`, `ruby`, `node`, `awk`** et
tout autre interpréteur, `mariadb`/`mysql`, `sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`, et
**`gh` sous toutes ses formes** (même en lecture). `pdftotext` vers la sortie standard seulement. Aucune donnée réelle
de Guy (rien de `~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh` est hors de ton périmètre. Autorisés :
lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`, `pdftotext … -`, et les filtres en lecture seule `tr`,
`sort`, `cut`, `head`, `tail`, `wc`, `uniq` vers la sortie standard.

---

## Annexe — le prompt de l'orchestrateur pour la remédiation P2 (recopié tel quel)

> Tu es l'agent de spécification de la Story 32-6c de Kesh, le journal du filtre d'adresse. Tu travailles **uniquement** dans le worktree `/home/gcorbaz/devel/kesh-32-6c`, sur la branche `story/32-6c`, et tu écris en français.
>
> Applique les consignes de `/home/gcorbaz/.claude/projects/-home-gcorbaz-devel-kesh/memory/consignes-agents-epic15.md` :
> - chemins absolus ;
> - dépôt principal en lecture seule ;
> - ni push ni `gh` qui écrit ;
> - ni cargo, ni npm, ni docker, ni curl ; la lecture et `pdftotext` sont permis ;
> - si une commande est refusée, tu t'arrêtes ;
> - n'ouvre jamais `~/kesh-test/`.
>
> ## Tâche
> Remédie la validation P2 de la fiche `_bmad-output/implementation-artifacts/32-6c-journal-filtre-adresse.md`. Les rapports sont dans `/home/gcorbaz/devel/kesh-gate-logs/326c-validate-p2-R.md` (0/0/2/5) et `/home/gcorbaz/devel/kesh-gate-logs/326c-validate-p2-F.md` (0/0/3/7) ; lis-les en entier.
>
> **Tous les findings naissent de la remédiation P1** et portent sur ce que la fiche fait écrire au manuel. C'est un recyclage : déclare-le au Change Log comme signal D5 pour le Project Lead. La consigne de l'orchestrateur est donc de **retirer plutôt que d'enrichir**. Une phrase du manuel que personne n'a éprouvée sur le NAS se retire ou se marque « non éprouvé », elle ne se précise pas davantage.
>
> **Décisions de l'orchestrateur :**
> - **F1 + R4 + F7, où poser `RUST_LOG`.** La procédure se réduit à ceci : **remplacer** la ligne `RUST_LOG=info` du `.env` par `RUST_LOG=info,kesh_api::mcp::reseau=debug`, puis lancer `sudo docker compose up -d`. Vérifier ensuite l'effet par la présence des lignes `debug` au journal. Retire la vérification `env | grep RUST_LOG` et toute affirmation sur l'environnement sous `sudo`.
> - **F2 = R2, Traefik.** Retire Traefik des exemples du cas (i). Le cas (i), `xff_lignes=0`, ne cite que la publication de port en espace utilisateur de Docker. Pour Traefik, s'il en faut un exemple, il va au cas (ii), avec la mention « non éprouvé » que porte la fiche 32-6.
> - **F3 + R1, les causes d'absence de ligne.** L'énumération porte sur les **sorties du transport**, relues au code et citées à la ligne. Corrige le rattachement de `transport.rs:87`, qui correspond à une erreur de lecture de la base et non à des lignes illisibles. Ajoute le chemin voisin (`/mcp/`, `/MCP`). Retire la précision « (pour la société attendue) » ou corrige-la. Si l'énumération ne peut pas être close de façon vérifiable, écris-la comme non exhaustive, avec la règle qui fait foi.
> - **F8 + R5 + R3, la commande de lecture.** Garde une seule commande, qui marche dans la configuration par défaut, et écris ses limites en une phrase : rotation `never`, `KESH_LOG_HOST_DIR`, `sudo`. Ne prescris pas de `sed` qui n'a été éprouvé sur aucun NAS : retire le repli, ou marque-le « non éprouvé ».
> - **Les autres LOW** (F4, F5, F6, F9, F10, R6, R7) sont à intégrer. Pour F9 : rien n'établit que Guy a vu le complément inséré au point 1. Place-le après sa réponse, comme ajout de l'orchestrateur.
>
> **Après le patch :**
> - greppe les valeurs : `env | grep`, `Traefik`, `sed`, `kesh.*.log`, `pour la société attendue` ;
> - recompte à la source ;
> - ne réécris aucune entrée passée ;
> - ajoute l'entrée P2 au Change Log, trend 0/0/5/10 → 0/0/5/12.
>
> ## Livrables
> - Un commit `refs #723`, sans mot-clé de fermeture.
> - Le prompt de la P3 : `_bmad-output/implementation-artifacts/326c-validate-prompt-p3.md`. Lentilles R et F, **Sonnet**, contexte frais, rapports dans `/home/gcorbaz/devel/kesh-gate-logs/326c-validate-p3-{R,F}.md`. Mêmes exigences et interdits que la P2, plus un axe obligatoire sur les régressions de cette remédiation. Recopie ce prompt-ci en annexe.
> - Un rapport final bref.
