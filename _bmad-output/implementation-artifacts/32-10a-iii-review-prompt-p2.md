# Prompt — revue de code P2 de la Story 32-10a-iii, passe ciblée (regression hunter), Sonnet

Tu relis **une seule remédiation** de la partie iii de la Story 32-10a de Kesh (comptabilité suisse, Rust/Axum,
MariaDB 10.11) : la remédiation de la revue de code P1, en **deux commits de code** — **`c96fea3b`** (« remédiation de
la revue de code P1 ») et **`592adf53`** (« la règle de Guy sur les prompts proposés, dite à l'agent et au manuel ») —,
sur la branche `story/32-10a-iii` du worktree `/home/gcorbaz/devel/kesh-32-10a`. Passe **ciblée**, une lentille :
**regression hunter**. Modèle : **Sonnet**. Contexte frais. Langue du rapport : français.

Le diff à relire, **aplati** (les deux commits ensemble) : `git -C /home/gcorbaz/devel/kesh-32-10a diff
c96fea3b^..592adf53`. Le commit qui suit (`Change Log`, registre, prompt) ne touche que des comptes rendus. Le motif qui justifie cette passe : *la
sévérité se déplace vers ce qu'on vient d'écrire* — cherche **ce que cette remédiation a cassé ou rendu faux**, non les
défauts d'origine de la story (une passe complète P1 les a déjà cherchés : rapports
`/home/gcorbaz/devel/kesh-gate-logs/3210a-iii-review-p1-{A,B,E}.md`, à lire pour savoir ce qui était demandé).

Ce que la remédiation devait faire (consigne de l'orchestrateur) :
- **A1** — la précondition de lecture du plafond et de la numérotation (`REPEATABLE READ` : rien lu dans la
  transaction avant le premier appel du dépôt) au Pattern 5 de `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (ligne des
  « Writes of the MCP prompts ») et en **commentaire seulement** dans `crates/kesh-db/src/repositories/mcp_prompts.rs`.
- **E1** — un test qui appelle `services::prompts_mcp::proposer` directement avec une clé `books:read` seule :
  `HorsPortee`, rien d'écrit, sans prendre le verrou (`proposer_refuse_hors_portee_avant_tout`).
- **E2** — le manuel d'administration (`docs/manual/fr/admin-manual.tex`, § `sec:mcp-outils`) : la phrase sur les
  outils servis et la ligne `proposer_prompt` de la matrice ; `tests/mcp_matrice_manuel.rs` étendu à `PromptsWrite` ;
  **le PDF régénéré**.
- **A2** — l'entrée du `CHANGELOG.md` (0.16.0).
- **E6 = A3** — la description de `proposer_prompt` (`mcp/textes.rs`, `DESCRIPTION_PROPOSER_PROMPT`) rendue exacte,
  et son littéral dans `tests/mcp_transport_e2e.rs`.
- Choix C-32-10a-106 à 108 et annotation de C-32-10a-103 au registre
  `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3210a.md`.
- **La règle de Guy** (citée à C-32-10a-106) : « le prompt proposé par l'agent doit être validé et approuvé par un
  utilisateur, et un agent ne peut pas directement modifier un prompt validé ». La description de `proposer_prompt`, le
  manuel et le CHANGELOG doivent dire en clair que la proposition attend l'approbation d'un administrateur et que la
  version en vigueur reste servie d'ici là. Les effets de bord d'une proposition (409, proposition antérieure non
  approuvable) sont maintenus par Guy et tracés par la KF-055, issue #796.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; **aucun `git checkout`/`switch`/`restore`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui compile ou exécute** : ni `python`, ni `perl`, ni `node`/`npm`, ni `jq`, ni `cargo`
  (aucune sous-commande), ni `make`, ni `latexmk`, ni `docker`, ni `mysql`, ni `sqlx`, ni `curl`, ni `sed -i` ; ni
  **`scripts/prepare-release.sh`**, ni aucun script de `scripts/`, de `kesh-gate-logs` (dont `mutations-3210aiii.py`)
  ou d'un scratchpad. **Aucune commande `gh`.**
- Permis : `git show`, `git diff`, `git log`, `git grep`, `grep`, `sed -n`, `awk` (sans redirection ni `system()`),
  `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée au rapport.
- Aucune donnée réelle ; ne lis rien sous `~/kesh-test/`.

## Les axes de la lentille

1. **Le code de `kesh-db` est-il intact ?** `git show c96fea3b -- crates/kesh-db` ne doit montrer que des lignes de
   commentaire (`//`, `///`). Une ligne de code changée est un finding. Le commentaire ajouté dit-il vrai sur le code
   (lis `verrouiller_societe`, le `COUNT` de `proposer_version_agent_in_tx`, `inserer_version`, `decider`) ? Nomme-t-il
   toutes les écritures concernées, et seulement elles (`amorcer_standards` ouvre ses propres transactions) ? La garde
   lexicale `versions_jamais_reecrites` / `tables_des_prompts_ecrites_par_leur_depot` peut-elle prendre un mot de ce
   commentaire pour une instruction SQL (commentaires `//` retirés avant l'extraction des littéraux) ?
2. **Le Pattern 5** : la phrase ajoutée dit-elle vrai (InnoDB fige la vue à la première lecture **simple** ; un
   `SELECT … FOR UPDATE` ne la fige pas) ? Cite-t-elle les bons appelants et le bon test ? Le tableau Markdown reste-t-il
   d'une seule ligne (aucun `|` ni saut de ligne introduit dans la cellule) ?
3. **Le test E1** : prouve-t-il ce qu'il dit — le refus **avant** les arguments (des arguments invalides rendent
   `HorsPortee`), **avant** le verrou (délai d'une seconde pendant qu'une transaction tient la société), **rien
   d'écrit** ? Peut-il passer à vide (une clé sans portée refusée ailleurs, à l'authentification ; un verrou pris sur la
   mauvaise société) ? Les mutations P1M1 (contrôle retiré) et P1M2 (contrôle déplacé après les arguments) — journaux
   `/home/gcorbaz/devel/kesh-gate-logs/3210a-iii-mut-P1M*.log` — sont-elles tuées **par ce test** et pour la raison
   visée ?
4. **Le manuel, PDF compris** : la phrase de `admin-manual.tex:2930` et la ligne de la matrice disent-elles exactement
   ce que fait le code (`tools/list` sous `prompts:write` : `kesh_lire_envoi` puis `proposer_prompt` ; l'outil
   n'écrit rien dans les livres ; la proposition n'est servie qu'approuvée) ? La colonne s'intitule « Ce qu'il lit » :
   la ligne `proposer_prompt` y est-elle cohérente ? **Contrôle le PDF aplati** :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oE '.{60}(un seul outil qui écrit|proposer_prompt prompts:write).{260}'`
   — le PDF dit-il la même chose que le `.tex` ? Les autres phrases du manuel qui disent « lecture seule », « aucun
   outil n'écrit » ou « outils de lecture » (`grep -nE "lecture seule|n'écrit|outils de lecture" docs/manual/fr/admin-manual.tex`)
   restent-elles vraies ? Les autres PDF (`user-manual.pdf`, `marketing-brochure.pdf`) ont-ils changé
   (`git show --stat c96fea3b`) ? `mcp_matrice_manuel.rs` : l'extension à `PromptsWrite` change-t-elle le calcul de
   « toute clé » (un outil présent sous toutes les portées) de façon que `kesh_lire_envoi` reste « toute clé » et que
   `proposer_prompt` ne le devienne pas ? La mutation P1M3 (matrice ramenée aux portées de lecture) est-elle tuée ?
5. **Le CHANGELOG** : chaque affirmation de l'entrée ajoutée est-elle vraie au code (portée, plafond de vingt, noms en
   `kesh`, « ne relit pas ses propositions », sur un nom existant, sessions d'Administrateur seulement, approbation
   revérifiée, journal d'audit sans le texte, `prompts/get` fail closed) ? Contredit-elle une autre entrée du même
   CHANGELOG (« aucune écriture », « lecture seule ») ?
6. **La description de `proposer_prompt`** : le texte de `textes.rs` et le littéral de `tests/mcp_transport_e2e.rs`
   sont-ils égaux caractère pour caractère (continuations `\` comprises) ? Dit-il vrai au dépôt
   (`proposer_version_agent_in_tx`) ? **Grep du symptôme** : `grep -rnF "déjà proposé" crates docs CHANGELOG.md` doit
   être vide.
7. **La règle de Guy, au code** : sur **tous** les chemins, une proposition d'agent reste une version à valider et
   aucune écriture d'agent ne modifie la version en vigueur. Relis les lignes que cite C-32-10a-106
   (`NUMERO_EN_VIGUEUR_SQL`, `requete_en_vigueur`, `proposer_version_agent_in_tx`, le seul `UPDATE
   mcp_prompt_versions` dans `decider`, ses appelants) et cherche un chemin qu'elles manquent ; un chemin d'agent qui
   approuve ou remplace la version en vigueur est un **CRITICAL**. **Le registre** : C-32-10a-106 à 108 décrivent-ils exactement le code (les deux effets d'une proposition et leurs
   bornes ; le nom écrit par `version_created` ; `NOMS_RESERVES`) ?
8. **Les gates déclarés** au Dev Agent Record et au Change Log pour cette remédiation se recoupent-ils avec
   `/home/gcorbaz/devel/kesh-gate-logs/3210a-iii-p1-*` (gate complet, frontend, E2E) ?

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/3210a-iii-review-p2.md`, par l'outil d'écriture de fichier,
**jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`R1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et ligne,
   preuve (sortie copiée), correction proposée, et s'il est **né de la remédiation** `c96fea3b` ou antérieur.
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** (1 à 8 ci-dessus) — sans cette liste, un « 0
   finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, s'il y en a.
