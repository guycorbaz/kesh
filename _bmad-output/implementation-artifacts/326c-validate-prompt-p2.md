# Prompt — validation P2, Story 32-6c (le journal du filtre d'adresse du serveur MCP)

*Versionné le 2026-10-25. Passe **complète** (deuxième passe), après la remédiation de la P1 (trend P1 : 0 CRITICAL /
0 HIGH / 5 MEDIUM / 10 LOW). **Deux lentilles, R et F**, chacune en contexte frais, **Opus** (la P1 était Sonnet :
rotation Sonnet ↔ Opus). Lecture seule stricte, **sans `gh`**, **sans interpréteur**, **sans fichier temporaire**.*

Dépôt `/home/gcorbaz/devel/kesh-32-6c`, branche `story/32-6c` (partie de `origin/main` `c5287dec`). Fiche :
`_bmad-output/implementation-artifacts/32-6c-journal-filtre-adresse.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-326c.md` (C-32-6c-1 à 13) ; entrées § *32-6c*, § *32-11*
et § *Réponses de Guy (2026-10-25)* de `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` ; clé
`32-6c-journal-filtre-adresse` de `_bmad-output/implementation-artifacts/sprint-status.yaml`. **La remédiation à relire**
est le dernier commit de la branche (`git show HEAD`, `git diff HEAD~1 HEAD`) ; les rapports de la P1 sont
`/home/gcorbaz/devel/kesh-gate-logs/326c-validate-p1-R.md` et `-F.md` — lis-les, mais **ne tiens pour juste aucune de
leurs affirmations sans l'avoir rouverte**. Contexte à lire : la fiche 32-6 `_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md`
(AC-a7 à AC-a9, AC-b4, § *Ce que les gardes ne voient pas*, § *Le manuel (axe de revue)*, points 2 et 2 bis pour Guy) et
son registre `epic-32-choix-autonomes-326.md` (C-32-6-4, 5, 6). Règles : `CLAUDE.md` (§ *Review Iteration Rule* et *La
passe ciblée* — une boucle ne se clôt que sur une passe sans finding au-dessus de LOW —, *Inventorier les sites NON
RÉSOLUS*, *Règle de splitting préventif*, *Le prompt d'une passe doit NOMMER le manuel*, *Recompter ses propres
comptes rendus*, *Recompter n'est pas réécrire l'historique*, *Recette visuelle*), `docs/agents-orchestration.md`.

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

## Les trois axes obligatoires, pour les deux lentilles

- ⛔ **Chaque ligne citée se rouvre.** Pour chaque `fichier:ligne` de la fiche, du registre et des passages de l'epic
  touchés (faits 1 à 13, AC, mutations, Dev Notes, § *À recontrôler*, choix C-32-6c-11 à 13), ouvre le site (`sed -n`,
  `grep -nF`) et **copie la sortie**. Une citation que le fichier ne porte pas est un finding. Les comptes écrits se
  **rejouent** : `grep -n "tracing::" crates/kesh-api/src/mcp/reseau.rs crates/kesh-api/src/mcp/transport.rs
  crates/kesh-api/src/services/serveur_mcp.rs` ; `grep -nE 'RUST\\?_LOG' docs/manual/fr/admin-manual.tex`.
- ⛔ **Chaque affirmation sur une dépendance se prouve à sa source**, version du `Cargo.lock`
  (`~/.cargo/registry/src/*/`) : `tracing-subscriber-0.3.23` (`filter/env/directive.rs`, `fmt/format/mod.rs`,
  `fmt/fmt_layer.rs` pour l'ANSI et `NO_COLOR`), `tracing-core-0.1.36/src/field.rs`, `tracing-0.1.44/src/macros.rs`
  (message en tête), `tracing-appender-0.2.5/src/rolling.rs` (nom du fichier en rotation), `http-1.4.2/src/header/value.rs`
  (validité de `0xE9` et de la valeur vide, `is_sensitive`). Une affirmation dont tu ne trouves pas la ligne est un finding.
- ⛔ **Les régressions de la remédiation P1** (axe neuf). *La sévérité ne stagne pas, elle se déplace vers ce qu'on
  vient d'écrire* (`CLAUDE.md`, § *La passe ciblée*). Relis **chaque hunk** de `git diff HEAD~1 HEAD` et demande-toi :
  le correctif d'un finding P1 en a-t-il introduit un autre ? Points à exercer au moins :
  (a) le fait 13 et l'AC-9 — le fichier `./log/kesh.*.log` existe-t-il vraiment par défaut sur une installation DSM
  (composes, montage `./log`, droits `root`, `sudo grep` et expansion du motif par le shell de l'utilisateur) ? la
  commande `sed 's/\x1b\[[0-9;]*m//g'` retire-t-elle les séquences que `tracing-subscriber` émet (relis
  `nu-ansi-term`/le style émis), et sur le `sed` d'un DSM ? le format `json` « mêmes champs » est-il exact ?
  (b) la réserve de l'AC-9 (ii) et la phrase ajoutée au cas (i) : disent-elles juste, sans promettre ni conseiller ce
  que la décision 4 de Guy interdit ?
  (c) les cas 13 et 14 : verdicts, champs et comptes justes contre `reseau.rs:63-87` et l'esquisse des Dev Notes ? le
  cas 13 est-il constructible (`reglages` en SQL brut, `allowed_networks` avec une entrée invalide, `list_enabled` ne
  rejette-t-il pas la ligne comme illisible ?) ; `verdict(…)` accepte-t-il deux lignes `x-forwarded-for` et une valeur
  vide ?
  (d) M17 à M21 : réalisables, tuées par le test **nommé**, aucune à vide ? M20 et M21 visent-elles bien l'AC-T3 et
  l'AC-T5 tels que réécrits ?
  (e) l'AC-T4 réécrit (par `envoyer`/`requete`, `Origin` sur la seule requête refusée, 401 attendu de l'admise avec un
  jeton inconnu) et l'AC-T5 réécrit (deux `societe`, ordre `ORDER BY company_id` sans tri) : réalisables tels quels ?
  (f) le fait 2 complété et l'AC-7 / AC-9 (viii) : l'énumération des chemins sans décision est-elle maintenant close ?
  Inventorie toi-même **tous** les retours `RouteAbsente` de `transport.rs` antérieurs à la décision et tout chemin
  amont (`routes/mcp.rs`, `lib.rs`), plutôt que de vérifier la liste écrite (`CLAUDE.md`, *Inventorier les sites NON
  RÉSOLUS*).
  (g) le Change Log P1 et les annotations du registre : décomptes justes (recompte), aucune entrée passée réécrite
  (l'entrée « création » doit garder ses chiffres d'origine : 16 mutations, 12 cas) ?
  (h) l'epic : le paragraphe ajouté au § 32-11 et la puce du § *Réponses de Guy (2026-10-25)* transcrivent-ils la
  décision de Guy sans l'élargir ni la réduire ? la phrase d'en-tête de ce § (« oui pour tout ») reste-t-elle vraie ?

## Lentille R — la spécification contre le code, les fiches et le manuel

Axes, tous à exercer :

1. **Les faits 1 à 13** : chacun est-il vrai au code de `c5287dec` ? En particulier le fait 2 complété, le fait 12
   corrigé et le fait 13 neuf.
2. **La cohérence avec la 32-6** : la fiche contredit-elle AC-a7, AC-a8, AC-a9, AC-b4, C-32-6-4 à 6, ou la décision de
   Guy sur la garde 2 bis ? L'AC-3 et l'AC-4 laissent-ils la décision **exactement** celle de `c5287dec` sur les cas
   (a) à (s) et (n') ? L'AC-6 est-il vérifiable tel qu'écrit ?
3. **L'évènement** (AC-1, AC-2, AC-5) : la ligne attendue (message en tête, puis onze champs) est-elle celle que la
   capture et le formateur produiront ? Un champ porte-t-il une valeur reçue du client ?
4. **Le manuel — `.tex` et PDF** (axe obligatoire de `CLAUDE.md`) : l'AC-9, l'AC-10 et l'AC-11 décrivent-ils ce que le
   code prescrit fera, sans promesse non vérifiée (DSM) ? Une phrase existante devient-elle fausse avec la story
   (`:652-655`, `:696`, `:792`, `:939-960`, `:957`, `:1641`, `:621`, § `sec:traefik`) ? Contrôle le PDF actuel
   aplati : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE
   ".{60}(X-Real-IP|RUST_LOG|docker compose logs|kesh\.log).{80}"` — copie la sortie.
5. **Les décomptes** : recompte à la source — 17 AC (11 + 6), 6 tâches, 21 mutations, 14 cas à l'AC-T1, 13 faits,
   C-32-6c-1 à 13, 3 modules — par `grep -c` sur la fiche et le registre, sortie copiée.

## Lentille F — faisabilité, simplicité, tests et fausses prémisses

1. **Le plus simple qui suffit** : la remédiation a-t-elle élargi la story (fichiers, modules, configuration) ? Le choix
   C-32-6c-11 (lire le fichier, ne pas toucher au format global) tient-il contre l'alternative `with_ansi(false)` ?
2. **Les tests** (AC-T1 à AC-T6) : chacun est-il réalisable avec `tests/common/capture_trace.rs` et
   `tests/common/mcp.rs` tels qu'ils sont (lis-les) ? Les quatorze cas donnent-ils le verdict et les champs écrits ?
3. **Les mutations M1 à M21** : chacune réalisable et tuée par le test nommé ? Une propriété de l'AC sans mutation ?
4. **La garde des secrets** (AC-5, AC-T4) : ferme-t-elle la classe « valeur d'en-tête au journal » sur ce site, avec
   l'AC-T4 réécrit ?
5. **Le comportement inchangé et l'oracle** : inchangés par la remédiation ?
6. **Les points soumis à Guy** : les réponses sont-elles transcrites fidèlement, à la fiche, au registre et à l'epic ?
   Un point neuf, né de la remédiation, devrait-il lui être soumis ?

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt ou de la dépendance,
preuve (commande et sortie copiée, `grep -nF` pour toute affirmation de présence ou d'absence), correction proposée ;
pour chaque finding, dis s'il **naît de la remédiation P1** (hunk cité) ou s'il est d'origine. Un bilan d'en-tête
**cohérent avec le corps**. ⛔ **La liste des axes exercés ET non exercés** (R1 à R5 ou F1 à F6, et les trois axes
obligatoires, sous-points (a) à (h) compris) — sans elle, « 0 finding » ne compte pas comme passe. Rapport écrit **d'un
seul jet**, un seul fichier par lentille, avec l'outil d'écriture de fichier :
`/home/gcorbaz/devel/kesh-gate-logs/326c-validate-p2-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/326c-validate-p2-F.md`.
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

## Annexe — le prompt de l'orchestrateur pour la remédiation P1 (recopié tel quel)

> Tu es l'agent de spécification de la Story 32-6c de Kesh : le journal du filtre d'adresse. Tu travailles **uniquement** dans le worktree `/home/gcorbaz/devel/kesh-32-6c`, branche `story/32-6c`, et tu écris en français.
>
> ## Consignes
> Applique `/home/gcorbaz/.claude/projects/-home-gcorbaz-devel-kesh/memory/consignes-agents-epic15.md`. Pour rappel :
> - chemins absolus ;
> - le dépôt principal est en lecture seule ;
> - ni push ni `gh` qui écrit ;
> - ni cargo, ni npm, ni docker, ni curl ; lecture et `pdftotext` permis ;
> - si une commande est refusée, tu t'arrêtes ;
> - tu n'ouvres jamais `~/kesh-test/`.
>
> ## Tâche
> Remédie la validation P1 de la fiche `_bmad-output/implementation-artifacts/32-6c-journal-filtre-adresse.md`. Les rapports sont `/home/gcorbaz/devel/kesh-gate-logs/326c-validate-p1-R.md` (0/0/4/6) et `-F.md` (0/0/1/4). Lis-les en entier, vérifie la prémisse de chaque finding au code, et écris au Change Log ce que tu réfutes.
>
> **Réponses de Guy du 2026-10-25, à inscrire à la fiche** (points soumis 1 à 5) :
> 1. Refus en `warn` : oui.
> 2. Formes et présences seulement, jamais les valeurs des en-têtes : oui.
> 3. Journaliser aussi quand le filtre est coupé : oui.
> 4. Conseiller `X-Forwarded-For` à DSM avant vérification : **non**.
> 5. Sous-section du manuel avant la section MCP de la 32-7 : oui, **« ajoute la description détaillée du fonctionnement de MCP »**. Cette description détaillée **revient à la 32-11** (documentation de bout en bout). Inscris-la à l'epic, § 32-11 de `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` : manuels d'administration et utilisateur — ce qu'est MCP, le parcours d'une requête de l'agent jusqu'au journal, les portées, les outils, les prompts, la validation avant envoi, ce que Kesh garantit et ne garantit pas. Cite la décision de Guy. La 32-6c ne l'écrit pas.
>
> **Décisions de l'orchestrateur sur les MEDIUM :**
> - **R1** (le fait 12 est faux, `RUST\_LOG` est déjà décrit à `admin-manual.tex:792`) : corrige le fait. La sous-section renvoie à la ligne existante au lieu de redécrire la variable. Elle tient compte de `:1641` : les variables du terminal l'emportent sur le `.env`. La procédure dit donc où poser `RUST_LOG` pour qu'il prenne effet.
> - **R2 = F2** (couleurs ANSI sur la sortie standard) : choisis le plus simple qui rende le journal lisible au `grep`, sans élargir la story. Option préférée : lire le **fichier** de log s'il existe et s'il est exempt de couleurs (vérifie-le au code de `logging.rs`). Sinon, une commande de filtrage prescrite au manuel. Ne change pas le format global du journal, sauf si c'est une ligne et que tu le justifies au registre. La ligne d'exemple du manuel vient de la source sans couleurs.
> - **R3** (deux `error!` produisent le même silence : `services/serveur_mcp.rs:41`, `transport.rs:87`) : complète l'énumération des causes d'absence de ligne. Fais aussi le fait 2.
> - **R4** (réserve sur la déclaration du pair comme mandataire) : ajoute à l'AC-9 la phrase de réserve. Si le mandataire déclaré est une adresse par laquelle passent tous les clients, tout poste qui l'atteint sans passer par le mandataire peut forger `X-Forwarded-For`. Guy a accepté que le manuel le dise.
> - **F1** (`plages` compte les entrées invalides ; `xff_entrees` compte sur les octets) : ajoute les cas 13 et 14 et deux mutations.
>
> **Les LOW** (R5 à R10, F3 à F5) sont tous à intégrer.
>
> **Après le patch :**
> - greppe les valeurs (`RUST_LOG`, `with_ansi`, `aucune ligne`, les nombres de cas et de mutations) ;
> - recompte à la source ;
> - ne réécris aucune entrée passée ;
> - écris l'entrée P1 du Change Log : trend 0/0/5/10.
>
> ## Livrables
> - Un commit `refs #723`, sans mot-clé de fermeture.
> - Le prompt de la P2, `_bmad-output/implementation-artifacts/326c-validate-prompt-p2.md`. Lentilles R et F, **Opus**, contexte frais, rapports `/home/gcorbaz/devel/kesh-gate-logs/326c-validate-p2-{R,F}.md`. Mêmes exigences et mêmes interdits, plus un axe sur les régressions de cette remédiation. Recopie ce prompt-ci en annexe.
> - Un rapport final bref.
