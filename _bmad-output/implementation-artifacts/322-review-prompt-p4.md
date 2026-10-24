# Spike 32-2 — prompt de la passe 4 de relecture (ciblée, une lentille)

Passe **ciblée** au sens de `CLAUDE.md` § *La passe ciblée* : **une seule lentille** (*regression hunter*), en contexte
frais, braquée sur **le seul commit de la remédiation de la passe 3** — `662fbef6` (`docs(32-2): relecture P3 — retrait :
…`), parent `2fe89753`. Modèle : **Sonnet**. Prompt versionné pour que le verdict se rejoue et se conteste.

La remédiation P3 est un **retrait** : la boucle recyclait sur la conception des types du § 5.3 (P1 7 MEDIUM → P2 4,
dont 3 nés de la remédiation → P3 3, tous nés de la remédiation P2). Le § 5.3 garde désormais des **exigences** (E1 à
E6, AC à porter par les fiches), ramène les types à une **esquisse indicative** et remet les trous connus aux fiches
32-6 et 32-8 (§ 5.3.3). Ta tâche n'est **pas** de reconcevoir les types : c'est de vérifier que le retrait est complet
et qu'il n'a rien perdu.

## Ce que tu relis

Worktree : `/home/gcorbaz/devel/kesh-32-2` (branche `story/32-2`). **Le diff aplati, d'un seul tenant** :

```sh
git -C /home/gcorbaz/devel/kesh-32-2 diff 2fe89753 662fbef6
```

Deux fichiers : `_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (document de décision) et
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-322.md` (registre, C-32-2-24 à 26 ajoutés, annotations
sur C-32-2-2, 17, 21). Le rapport qui a motivé la remédiation : `/home/gcorbaz/devel/kesh-gate-logs/322-review-p3.md`
(M-1 à M-3, L-1 à L-11). Le kickoff : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md`. Les journaux :
`/home/gcorbaz/devel/kesh-gate-logs/322-spike/`.

## La lentille — regression hunter

1. **Aucune affirmation de garantie non prouvée ne reste.** Grepe sur le fichier **final** (à `662fbef6`) : `par
   construction`, `garanti`, `ne compile pas`, `aucun chemin`, `ne peut pas`, `ne peut donc`, `seul … construit`,
   `impossible`, `toujours`, `jamais`, `exactement`. Pour chaque occurrence qui porte sur la **validation avant envoi**,
   la **lecture seule** ou les **erreurs** : est-elle écrite comme **exigence** (à prouver par une story nommée), comme
   **intention** d'une esquisse, ou comme **fait** ? Un fait non adossé à un essai ou à une source est un finding. Le
   texte dit-il, à chaque fois, **ce qui prouverait** la propriété ? Les commentaires du bloc de code du § 5.3.2
   affirment-ils encore une garantie ? Les §§ 1.3, 3.4, 4.3, 6 et 7 renvoient-ils à des types ou à des exigences ?
2. **Chaque exigence est portée par une story.** Pour E1 à E6 du § 5.3.1, et pour chaque point du § 5.3.3 : une story
   (32-6, 32-8, 32-10a, 32-9b…) est-elle nommée, et le § 6 de cette story le reprend-il ? Inversement, le § 6 cite-t-il
   encore un type de l'esquisse comme AC (greper `DonneesValidees`, `Reponse`, `MessagePrompt`, `PromptEnBase`,
   `McpToolPrompts`, `Accuse`, `scell`, `trybuild` hors du § 5.3.2/5.3.3) ? Les mutations 1 à 6 d'E6 sont-elles toutes
   attribuées (le § 6 en donne 1, 2, 3, 5 à la 32-6 et 4, 6 à la 32-8) ?
3. **Rien n'a été perdu par le retrait.** Compare à `2fe89753` : chaque élément de l'ancien § 5.3 (inventaire des
   méthodes, `-32601` des non servies, ressources en 0.17.0, cas assumés des prompts et de l'élicitation, clé dispensée
   journalisée, erreurs sans texte libre, valeur piégée, Q12 « toujours ») se retrouve-t-il dans une exigence, dans
   l'esquisse ou au § 5.3.3 ? Le tableau d'E2 est-il cohérent avec le texte (six requêtes, notifications, deux messages
   serveur → client) et avec le § 1.3 ?
4. **Les LOW mécaniques** (C-32-2-26) : L-1 (§ 1.3 et annotation de C-32-2-2), L-2 (§ 3.3 ; les citations du changelog
   `2026-07-28` sont dites **rapportées par la relecture P3** — vérifie-les **en ligne**, WebFetch permis :
   `https://modelcontextprotocol.io/specification/2026-07-28/changelog`), la citation de `…/2025-11-25/server/tools`
   reprise en E1 (même vérification), L-3 (`grep -n -A1 'name = "async-trait"' Cargo.lock` ; aucun `async-trait` dans
   un `Cargo.toml`), L-10, L-11. Le § 5.3.3 L-7 cite `ci.yml` l. 175-176 et `proto-main.rs` l. 49-62 : justes ?
5. **§ 6.1 (L-8, L-9)** : les affirmations sur le kickoff sont-elles fidèles au fichier d'epic (ordre des migrations,
   « trois migrations non breaking au plus », modules de la 32-8, texte de sa fiche, Q7, prémisse 4, 32-12) ? Le
   document dit-il clairement que la spike **ne modifie pas** le fichier d'epic ?
6. **Règles du dépôt** — aucune donnée réelle ; aucun mot-clé de fermeture (`close`, `fix`, `resolve` suivis de `#N`) dans
   le message de `662fbef6` ni dans le texte ; le registre n'a **rien supprimé**
   (`git -C /home/gcorbaz/devel/kesh-32-2 diff -U0 2fe89753 662fbef6 -- _bmad-output/implementation-artifacts/epic-32-choix-autonomes-322.md | grep -c '^-[^-]'`
   doit rendre 0) ; 26 en-têtes `### C-32-2-`, uniques ; les annotations sont datées par passe ; l'en-tête du document
   (relevé P3) concorde avec le rapport P3.

## Interdits

- **Aucune commande qui écrit ou exécute** : seulement `git diff`, `git show`, `git log`, `cat`, `grep`, `ls`,
  `sed -n`, `wc`, et WebFetch. Ni `cargo`, ni `docker`, ni `docker compose`, ni `curl`, ni `npm`, ni `mariadb`, ni
  `python3`, ni `claude`, ni `hermes`, ni `gh`, ni `scripts/prepare-release.sh` ou tout autre script du dépôt ; ni
  `git checkout`, `switch`, `reset`, `stash`, `commit`, `push`.
- **Aucune redirection** (`>`, `>>`, `tee`), **aucun fichier temporaire**, aucun `Edit`/`Write` dans le dépôt
  (`docs/agents-orchestration.md` § 2.1 : le dépôt principal est en lecture seule, le worktree aussi pour cette passe).
- **Seule écriture permise** : le rapport ci-dessous, par chemin absolu.

## Rapport

À écrire dans `/home/gcorbaz/devel/kesh-gate-logs/322-review-p4.md`, en français :

- **Décompte recompté** : CRITICAL / HIGH / MEDIUM / LOW, total cohérent avec la ventilation.
- Chaque finding : sévérité, site (fichier et section ou ligne du fichier **final**, à `662fbef6`), ce qui est faux, la
  source qui l'établit (URL citée textuellement, ligne de journal, `grep` exécuté), et s'il est **introduit par la
  remédiation P3** ou préexistant (vérifié contre `2fe89753`). Un finding qui affirme une absence se vérifie par
  `grep -nF` sur le fichier final avant d'être écrit.
- Les faits vérifiés et exacts, avec leur source.
- **Axes exercés et non exercés** (1 à 6 ci-dessus, et tout ce qui n'a pas pu être vérifié) — un « 0 finding » sans
  cette liste ne compte pas comme passe.

Rappel de la règle de clôture (`CLAUDE.md`) : la boucle ne se clôt que sur une passe qui ne rend **aucun** finding
au-dessus de LOW — même un MEDIUM qui ne porterait que sur la rédaction.
