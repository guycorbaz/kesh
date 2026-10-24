# Spike 32-2 — prompt de la passe 2 de relecture (ciblée, une lentille)

Passe **ciblée** au sens de `CLAUDE.md` § *La passe ciblée* : **une seule lentille** (*regression hunter*), en contexte
frais, braquée sur **le seul commit de la remédiation de la passe 1** — `8f5e9459` (`docs(32-2): relecture P1 — …`),
parent `fd88d3cb`. Modèle : **Haiku** (les passes ciblées lui reviennent, décision D6 de Guy). Prompt versionné pour
que le verdict se rejoue et se conteste.

## Ce que tu relis

Worktree : `/home/gcorbaz/devel/kesh-32-2` (branche `story/32-2`). **Le diff aplati, d'un seul tenant** — jamais la
suite de commits :

```sh
git -C /home/gcorbaz/devel/kesh-32-2 diff fd88d3cb 8f5e9459
```

Deux fichiers : `_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (document de décision) et
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-322.md` (registre, C-32-2-10 à 18 ajoutés). Le rapport de
la passe 1, qui a motivé la remédiation : `/home/gcorbaz/devel/kesh-gate-logs/322-review-p1.md` (M1 à M7, L1 à L7). Les
décisions du Project Lead : `/home/gcorbaz/.claude/projects/-home-gcorbaz-devel-kesh/memory/serveur-mcp-decisions.md`.
Les journaux des essais : `/home/gcorbaz/devel/kesh-gate-logs/322-spike/`.

## La lentille — regression hunter

La sévérité se déplace vers ce qu'on vient d'écrire. Cherche **ce que la remédiation a introduit**, non ce que la passe
1 a déjà trouvé :

1. **Faits rectifiés** — revérifie chacun **en ligne** (WebFetch permis) et dis s'il est maintenant exact, cité
   fidèlement, et sans nouvelle sur-lecture :
   - Hermes : YOLO ne répond pas à l'élicitation ; `protocol` `auto`/`stateless`/`legacy`
     (`https://hermes-agent.nousresearch.com/docs/user-guide/features/mcp`,
     `https://hermes-agent.nousresearch.com/docs/reference/mcp-config-reference`).
   - Claude Code (`https://code.claude.com/docs/en/mcp`) : minuteur de premier octet = max(60 s, délai d'outil du
     serveur, `MCP_TIMEOUT`), le défaut de 28 h **non compté** — d'où **60 s par défaut** (le document réfute en partie
     le M3 sur ce point : vérifie qui a raison) ; passage en tâche de fond à 2 min, ce que reçoit l'agent, session non
     interactive ; runtimes v1/v2 et leur défaut.
   - Spécification `2025-11-25` : requête serveur → client sur le flux SSE du `tools/call` ; élicitation « flat objects
     with primitive properties only » ; MRTR et « explicit, server-minted handles » en `2026-07-28`.
2. **Raisonnement refait** — la justification de `2025-11-25` (§ 1.3) tient-elle sur les faits rectifiés, ou affirme-t-elle
   plus qu'eux (p. ex. sur le runtime v1 de Claude Code face à un serveur moderne seul) ? Le § 3.2 dit-il honnêtement ce
   qui reste du rejet de l'élicitation ? Le tableau du § 3.3 est-il exact case par case, et équitable entre les trois
   formes (aucune case qui avantage la forme recommandée sans source) ?
3. **Nouvelles affirmations** — chaque fait neuf est-il adossé (source, journal, ligne de code) ou marqué non éprouvé ?
   En particulier § 4.3 : scripts d'init qui ne rejouent qu'à la création (`docker-compose.dev.yml`, `.env.example`),
   `docker-compose.prod.yml` sans MariaDB, `CREATE USER` droit global, refus attendu `1142` (non essayé), `SHOW GRANTS`
   au démarrage ; § 1.3 : `Origin`/`Host` sous *DNS rebinding* ; § 4.4 : `touch_last_used` et `middleware/auth.rs`.
4. **Cohérence interne** — après la remédiation, le document se contredit-il ? Greper **les valeurs**, non les
   formulations : `240`, `60 s`, `2 min`, `300`, `par construction`, `PropositionEnvoi`, `legacy`, `YOLO`, `Q2`, `Q8` à
   `Q12`, `C-32-2-` (numéros cités et existants), `32-10a`/`32-10b`. Les « cinq décisions » en tête, les §§ 3.4, 4.3, 6
   et 7 et le registre disent-ils la même chose ? Aucune décision remise à Guy n'est-elle encore écrite comme prise ?
5. **Traits du § 5.3** — compilent-ils en principe (types, durées de vie, `dyn`), et tiennent-ils ce que le texte
   prétend (aucun outil de lecture ne peut contourner la validation d'une clé `Requise` ; `Accuse` ne peut porter de
   donnée des livres ; `McpPrompt::rendre` sans `Lecture`) ? Un trou que la remédiation aurait ouvert ?
6. **Règles du dépôt** — aucune donnée réelle ; aucun mot-clé de fermeture (`close`, `fix`, `resolve` suivis de `#N`)
   dans le message du commit ni dans le texte ; le registre n'a pas réécrit les entrées passées (annotations datées
   seulement).

## Interdits

- **Aucune commande qui écrit ou exécute** : seulement `git diff`, `git show`, `git log`, `cat`, `grep`, `ls`,
  `sed -n`, et WebFetch. Ni `cargo`, ni `docker`, ni `npm`, ni `mariadb`, ni `claude`, ni `hermes`, ni
  `scripts/prepare-release.sh` ou tout autre script du dépôt.
- **Aucune redirection** (`>`, `>>`, `tee`), **aucun fichier temporaire**, aucun `Edit`/`Write` dans le dépôt.
- **Seule écriture permise** : le rapport ci-dessous.

## Rapport

À écrire dans `/home/gcorbaz/devel/kesh-gate-logs/322-review-p2.md`, en français :

- **Décompte recompté** : CRITICAL / HIGH / MEDIUM / LOW, total cohérent avec la ventilation.
- Chaque finding : sévérité, site (fichier et section ou ligne du fichier **final**), ce qui est faux, la source qui
  l'établit (URL citée textuellement, ligne de journal, `grep` exécuté). Un finding qui affirme une absence se vérifie
  par `grep -nF` sur le fichier final avant d'être écrit.
- Les faits vérifiés et exacts, avec leur source.
- **Axes exercés et non exercés** (1 à 6 ci-dessus, et tout ce qui n'a pas pu être vérifié) — un « 0 finding » sans
  cette liste ne compte pas comme passe.
