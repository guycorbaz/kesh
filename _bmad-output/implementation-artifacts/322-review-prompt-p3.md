# Spike 32-2 — prompt de la passe 3 de relecture (ciblée, une lentille)

Passe **ciblée** au sens de `CLAUDE.md` § *La passe ciblée* : **une seule lentille** (*regression hunter*), en contexte
frais, braquée sur **le seul commit de la remédiation de la passe 2** — `bbb6e06c` (`docs(32-2): relecture P2 — …`),
parent `bd7eda1e`. Modèle : **Sonnet** (la passe 2 était Haiku ; on change de modèle d'une passe à l'autre). Prompt
versionné pour que le verdict se rejoue et se conteste.

## Ce que tu relis

Worktree : `/home/gcorbaz/devel/kesh-32-2` (branche `story/32-2`). **Le diff aplati, d'un seul tenant** — jamais la
suite de commits :

```sh
git -C /home/gcorbaz/devel/kesh-32-2 diff bd7eda1e bbb6e06c
```

Deux fichiers : `_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (document de décision) et
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-322.md` (registre, C-32-2-19 à 23 ajoutés, annotations
sur C-32-2-4, 5, 6, 11, 12, 18). Le rapport de la passe 2, qui a motivé la remédiation :
`/home/gcorbaz/devel/kesh-gate-logs/322-review-p2.md` (M-1 à M-4, L-1 à L-11). Les journaux des essais :
`/home/gcorbaz/devel/kesh-gate-logs/322-spike/`. Le kickoff : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md`
et `_bmad-output/implementation-artifacts/epic-32-choix-autonomes.md` (C-32-0-10).

## La lentille — regression hunter

La sévérité se déplace vers ce qu'on vient d'écrire. Cherche **ce que la remédiation P2 a introduit**, non ce que les
passes 1 et 2 ont déjà trouvé :

1. **M-1 — justification de `2025-11-25`** (décision 1, §§ 1.3, 2.3, C-32-2-19). Revérifie **en ligne** (WebFetch
   permis) : `https://code.claude.com/docs/en/mcp` (runtimes v1/v2, « every other server as v1 does »,
   `MCP_PROTOCOL_NEGOTIATION`, conditions du v2 par défaut) ; `https://hermes-agent.nousresearch.com/docs/reference/mcp-config-reference`
   (`protocol`, citation complète) ; `https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle`
   (*Version Negotiation*, « SHOULD disconnect ») ; la matrice de `…/2026-07-28/basic/versioning`. Le texte affirme-t-il
   encore plus que les sources — ou, à l'inverse, en retire-t-il un fait établi ? La déduction « v1 n'a que l'ère
   `initialize` » est-elle marquée comme déduction ? La justification finale tient-elle sur ce qui reste ?
2. **M-2 — délais par forme** (§ 2.3, cellule (a) du § 3.3, C-32-2-20). Chaque délai cité pour (a), (b), (c) a-t-il sa
   source ? Le § 2.3, le tableau du § 3.3, le § 3.4 et Q10 se contredisent-ils encore (greper `300`, `240`, `60 s`,
   `2 min`, `5 min`, `aussitôt`, `immédiat`) ?
3. **M-3 — garantie de validation** (§ 5.3, § 6 32-6 et 32-9, Q12, C-32-2-21). L'inventaire des méthodes est-il **clos**
   pour `2025-11-25` (compare au schéma de la révision : `…/specification/2025-11-25/schema` ou les pages *server* et
   *client*) — une méthode servie, une notification serveur → client, ou un champ (`instructions`, `description` d'un
   prompt, `message` de progression, `data` d'une erreur JSON-RPC) qui porterait une donnée des livres sans passer par
   `DonneesValidees` ? Les types neufs (`DonneesValidees`, `Reponse`, `ErreurOutil` à variantes, `MotifArgument`,
   `NomPrompt`, `CataloguePrompts`, `PromptEnBase`, trait scellé) compilent-ils en principe et tiennent-ils ce que le
   texte dit ? Les tests proposés rougiraient-ils vraiment (un test d'inventaire qui compare une liste à elle-même ne
   prouve rien) ? La décision « aucune ressource en 0.16.0 » est-elle compatible avec C-32-0-10 et avec le reste du
   document (greper `ressource`, `resources`, `-32601`, `-32602`) ? L'ajout touchant Q12 (§ 7) est-il fidèle ?
4. **M-4 — bloc du § 4.2** : chaque ligne du bloc, hors marques `[…]`, se trouve-t-elle **telle quelle** dans
   `ro-pool.out` (`grep -nxF`) ? Les numéros de ligne annoncés sont-ils justes ? Le texte qui suit le bloc
   (« Lecture : … ») renvoie-t-il encore à ce que le bloc montre ? Le bloc du § 1.2, dit « remis en forme », dit-il vrai
   sur ce qu'il omet (comparer à `proto-curl.out`) ?
5. **LOW L-1 à L-11** : chaque correction est-elle exacte et sourcée — en particulier L-1 (le contrôle négatif à 70 s
   éprouve-t-il le minuteur ?), L-3 (conditions du v2 recopiées fidèlement), L-10 (citations de
   `https://hub.docker.com/_/mariadb`), L-11 (« non établi » justifié).
6. **Règles du dépôt** — aucune donnée réelle ; aucun mot-clé de fermeture (`close`, `fix`, `resolve` suivis de `#N`) dans
   le message du commit ni dans le texte ; le registre n'a **rien supprimé** (`git diff -U0 bd7eda1e bbb6e06c --
   <registre> | grep -c '^-[^-]'` doit rendre 0) ; les annotations sont datées par passe ; les numéros C-32-2-19 à 23 sont
   uniques et cités à bon escient dans le document.

## Interdits

- **Aucune commande qui écrit ou exécute** : seulement `git diff`, `git show`, `git log`, `cat`, `grep`, `ls`,
  `sed -n`, `wc`, et WebFetch. Ni `cargo`, ni `docker`, ni `npm`, ni `mariadb`, ni `claude`, ni `hermes`, ni `gh`, ni
  `scripts/prepare-release.sh` ou tout autre script du dépôt ; ni `git checkout`, `switch`, `reset`, `commit`, `push`.
- **Aucune redirection** (`>`, `>>`, `tee`), **aucun fichier temporaire**, aucun `Edit`/`Write` dans le dépôt.
- **Seule écriture permise** : le rapport ci-dessous.

## Rapport

À écrire dans `/home/gcorbaz/devel/kesh-gate-logs/322-review-p3.md`, en français :

- **Décompte recompté** : CRITICAL / HIGH / MEDIUM / LOW, total cohérent avec la ventilation.
- Chaque finding : sévérité, site (fichier et section ou ligne du fichier **final**, à `bbb6e06c`), ce qui est faux, la
  source qui l'établit (URL citée textuellement, ligne de journal, `grep` exécuté), et s'il est **introduit par la
  remédiation P2** ou préexistant (vérifié contre `bd7eda1e`). Un finding qui affirme une absence se vérifie par
  `grep -nF` sur le fichier final avant d'être écrit.
- Les faits vérifiés et exacts, avec leur source.
- **Axes exercés et non exercés** (1 à 6 ci-dessus, et tout ce qui n'a pas pu être vérifié) — un « 0 finding » sans
  cette liste ne compte pas comme passe.

Rappel de la règle de clôture (`CLAUDE.md`) : la boucle ne se clôt que sur une passe qui ne rend **aucun** finding
au-dessus de LOW.
