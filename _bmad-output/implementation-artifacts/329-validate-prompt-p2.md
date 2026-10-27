# Prompt — validation P2, Story 32-9 (les outils de lecture du serveur MCP)

*Versionné le 2026-10-25. Passe **complète** (deuxième passe, après la remédiation de la P1). **Deux lentilles, R et
F**, chacune en contexte frais, **Opus**. Lecture seule stricte, **sans `gh`**, **sans interpréteur**, **sans
redirection**.*

Dépôt `/home/gcorbaz/devel/kesh-32-9`, branche `story/32-9` (partie de `main` `fa5b3468` : 32-3, 32-4a, 32-4b, 32-5a,
32-5b, 32-5d fusionnées ; **les 32-6, 32-8 et 32-10a ne le sont pas** — la fiche en dépend, le dit, et cite leurs fiches
validées). Fiche : `_bmad-output/implementation-artifacts/32-9-outils-de-lecture.md` (deux parties : 32-9a, 32-9b) ;
registre : `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-329.md` (C-32-9-1 à 28) ; entrées
`32-9a-*` et `32-9b-*` de `_bmad-output/implementation-artifacts/sprint-status.yaml`. Rapports de la P1, à lire pour
savoir ce qui a été corrigé (non pour le reprendre tel quel) : `/home/gcorbaz/devel/kesh-gate-logs/329-validate-p1-R.md`,
`/home/gcorbaz/devel/kesh-gate-logs/329-validate-p1-F.md` ; l'entrée « validation P1 » du Change Log de la fiche dit
comment chaque constat a été traité. `git diff 3b3ff322 HEAD -- _bmad-output/implementation-artifacts/` montre la
remédiation.

Contexte, à lire : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-9*, § *Points d'attention* — la
matrice, l. 178 —, § *Décisions*, § *Questions pour Guy* et *Réponses de Guy*) ; la spike
`_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (**en entier** ; en particulier § 2.1 — limites de Claude Code —,
§ 5.3.1 — E1, E3, E5 —, § 6 — entrée *32-9a / 32-9b* —, § 7 et ses réponses Q10, Q11, Q12) ; les fiches **en lecture
seule** des worktrees voisins :
`/home/gcorbaz/devel/kesh-32-6/_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md` (AC-a13, AC-a16 à AC-a24,
AC-a28, AC-a30, AC-a31, § *L'inventaire clos*, Dev Notes, et son registre `epic-32-choix-autonomes-326.md`),
`/home/gcorbaz/devel/kesh-32-8/_bmad-output/implementation-artifacts/32-8-validation-avant-envoi.md` (AC-02, AC-B1 à
AC-B6, AC-B13, AC-B15, § *La propriété centrale*, § *Les signatures*, § *Ce que la 32-8 laisse aux stories suivantes*,
et `epic-32-choix-autonomes-328.md`), `/home/gcorbaz/devel/kesh-32-10a/_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md`
(§ des textes standard, l. 1255-1274, et la précision de l'orchestrateur, l. 1455-1459) ; les fiches
`32-3-services-motif.md`, `32-4a-*`, `32-4b-*` (Q1 de la 32-4b) de ce dépôt. Règles : `CLAUDE.md` (§ *Review Iteration
Rule* et *La passe ciblée* — une boucle ne se clôt que sur une passe sans finding au-dessus de LOW ; une remédiation de
l'orchestrateur suit la même boucle —, *Inventorier les sites NON RÉSOLUS*, *Règle de splitting préventif*, *Le prompt
d'une passe doit NOMMER le manuel*, *Recompter ses propres comptes rendus* et *Recompter n'est pas réécrire
l'historique*, *Propagation post-patch*), `docs/agents-orchestration.md`.

**Contraintes de la story, à tenir pour vraies et à contrôler** : portées `books:read`, `invoices:read`,
`contacts:read`, `documents:read` (Q5 sans réponse : hypothèse) ; la Q1 de la 32-4b tranchée **en hypothèse** — depuis la
P1, seules l'IBAN, le QR-IBAN du créancier et l'adresse d'envoi sont masqués sans `contacts:read`, les noms des tiers ne
le sont pas (décision de l'orchestrateur) —, réversible par une constante et trois textes ; lecture seule
**applicative** (Q11) ; validation **par clé** (Q12) : sous `required`, tout passe par le dépôt de la 32-8 ; **aucune
ressource MCP** en 0.16.0 (E3) ; la 32-9b peut glisser en 0.17.0 ; au plus cinq modules par partie ; **jamais de donnée
réelle**. Ce que la fiche **prescrit** au développement n'existe pas encore : son absence n'est pas un défaut. Ce que la
fiche dit des fiches 32-6, 32-8, 32-10a se juge **contre ces fiches**, non contre le code.

⚠️ **Le motif mesuré de ce dépôt** : la sévérité se déplace vers ce qu'on vient d'écrire. La remédiation de la P1 a
ajouté trois sections (*L'inventaire* refait, *Les pièges et la largeur des colonnes*, *Les tailles mesurées*), réécrit
les AC-a3, a6, a9 à a11, a13 à a24, b1 à b7, les tâches, les mutations, et C-32-9-7 à 9 ; ajouté C-32-9-26 à 28. Relis-les
**en premier**, puis le reste.

## Les deux axes obligatoires, pour chaque lentille

- **(A) L'inventaire clos des champs libres** (§ *L'inventaire*) : cherche une `String` d'un type rendu par un service
  appelé (fait 6 ; relis chaque `struct` à sa source, `sed -n`) qui **n'est pas** à l'inventaire, ou un chemin par lequel
  un texte lu en base atteindrait l'agent **hors** de `TexteNonFiable`/`Tiers`/`Code` — un champ d'un type imbriqué, une
  `Vec<String>`, une clé de table associative, une erreur (dont les trois variantes neuves de l'AC-a22), le résumé,
  l'en-tête, `instructions`, `tools/list`, un message de service recopié, une énumération à `Serialize` manuel, un
  `&'static str` fabriqué à l'exécution, un type neuf (`FicheFactureFournisseur`, `PiecePublique`) qui porterait un champ
  de trop. Le typage (`ChampSur`, `sortie!`, le sceau `pub(in crate::mcp)`) ferme-t-il vraiment la classe en Rust ? Et
  la **conception révisée des tiers** : un IBAN, un QR-IBAN ou une adresse d'envoi sortent-ils par un autre champ rendu
  sous `invoices:read` ou `books:read` (un libellé composé, un document, une pièce source) ? Le grep de l'AC-a6 est-il
  juste et suffisant ?
- **(B) Le manuel** (axe obligatoire de `CLAUDE.md`) : la sous-section que l'AC-a23 et l'AC-b5 prescrivent dans
  `docs/manual/fr/admin-manual.tex`, sa place (section de la 32-7 si elle existe, sinon entre l. 2926 et 2927 de la
  `\section{Sécurité}` : vérifie les lignes) et chacune de ses affirmations (points (1) à (5)) contre le code prescrit ;
  ce que la fiche dit de ce qu'écrivent la 32-7 et la 32-11 (fiches de l'epic) ; et **le PDF aplati** : `pdftotext
  docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{60}(MCP|portée|outil).{80}"`. Un texte
  publié devient-il faux avec la 32-9 sans que la fiche le nomme (`grep -n "MCP" README.md website/*.html
  docs/api-external.md`) ?

## Lentille R — la spécification contre le code et contre les fiches dont elle dépend

Pour chaque affirmation, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**. Axes, tous à exercer :

1. **Les faits 1 à 12** (§ *Ce que le code dit*) et les sites **ajoutés en P1** : les libellés composés
   (`supplier_invoices.rs:512`, `:941`, `credit_notes.rs:590`, `invoices.rs:2347-2351`, `invoice_format.rs:259`, défaut
   du gabarit) ; ce que les dépôts lient tel quel (`supplier_invoices.rs:537-549`, `contacts.rs:252-300`,
   `invoices.rs:2652`) ; les variantes d'erreur (`errors.rs:1070`, `:1082`, `:1186-1215`, `LetteringAccountNotLetterable`
   et sa forme en sortie de `postes_ouverts::lister`) ; `routes/accounts.rs:223` ; `kesh-seed/src/lib.rs:215` ;
   `general_ledger.rs:93-96`. Chaque numéro de ligne est-il juste à `fa5b3468` ?
2. **Les largeurs des colonnes** (§ *Les pièges et la largeur des colonnes*) : relis **chaque** ligne du tableau à sa
   migration — y compris une `MODIFY COLUMN` ultérieure qui l'aurait élargie ou rétrécie (`grep -rn "MODIFY COLUMN\|CHANGE
   COLUMN" crates/kesh-db/migrations`) ; la forme (L, C, T, E) suit-elle la règle ; la longueur des trois formes (72, 13,
   7) est-elle juste (`printf '%s' … | wc -m`) ? Une colonne piégée « oui » à l'inventaire manque-t-elle au tableau ?
3. **Les emprunts aux fiches 32-6 et 32-8** : chaque nom (`McpTool`, `ContexteMcp`, `LectureOutils`, `BlocContenu`,
   `Donnees::nouvelles`, `ErreurOutil` et ses variantes, `MotifArgument::HorsBornes`, `arguments`, `depuis_app_error` et
   sa liste blanche, `Registre`, `registre_coherent`, `rendre_visible`, `MCP_ENVOI_OCTETS_MAX`,
   `MCP_ENVOI_RESUME_OCTETS_MAX`, `TEXTE_ENVOI_DEPOSE`, `TEXTE_ENVOI_VALIDE`, `NOM_LIRE_ENVOI`, `kesh_lire_envoi` et sa
   place dans `tools/list`, `tools_list_filtre` et ses clauses après la 32-8b, l'énumération des issues de la 32-6b)
   existe-t-il **sous ce nom et cette forme** dans la fiche qui le crée ? Chaque amendement d'un AC de la 32-6 ou de la
   32-8 (AC-a13, a16, a17, a18, a20, a22, a28, a30 ; AC-B13) est-il **nommé** comme tel, et ses tests à amender
   identifiés ?
4. **Les outils** (AC-a10, AC-b2) : chaque outil appelle-t-il un service qui **existe** (ou que la fiche crée :
   `exercices::lister`, `factures_fournisseur::detail_sans_fichier`, `pieces::{lister, lister_tout}`) avec la signature
   dite ? Le passage d'`accountId` unique au grand livre (`account_ids: Option<String>`, `rapports.rs:214-230`) est-il
   juste ? Les défauts et `maximum` du tableau égalent-ils le § *Les tailles mesurées* et `min(service,
   LIMITE_MAX_MCP)` ? Les deux services neufs respectent-ils R1 à R8 et le RBAC de leur route ; `pieces::lister`
   garde-t-il **l'ordre d'aujourd'hui** (société, puis validation) ?
5. **Le socle** (AC-a1 à AC-a9) : `texte_et_valeur`, pure, est-elle testable en `mod tests` sans `Donnees` ? `instructions`
   amendée sans conflit avec l'AC-B13 de la 32-8 ? `tiers_visibles` en `is_none_or` (Rust du dépôt) ? `ContexteMcp` et la
   garde `trace_confinee_dans_outil` contre les sites que la 32-6 a déjà posés (`EtatMcp.trace`, journal) ?
6. **Le registre de sprint** : les deux entrées `32-9a-*`, `32-9b-*` se lisent-elles comme du YAML (`clé: valeur  #
   commentaire`, deux espaces d'indentation, aucun caractère qui ouvre une structure hors commentaire) et disent-elles
   l'état (P1 remédiée, P2 à lancer) et les dépendances de la fiche ?

## Lentille F — faisabilité, découpage, sécurité et fausses prémisses

1. **Le typage** (C-32-9-2) : `sortie!`, `champ_enum!` et le sceau compilent-ils en esprit (une macro `macro_rules!` qui
   pose `exige::<$ty>()` pour chaque champ ; `const _: fn(E) -> isize = |x| x as isize;` ; un module
   `pub(in crate::mcp)` atteint depuis un développement dans `mcp::outils`) ? Ma2 à deux `impl`, Ma3 et Ma3b « ne
   compile pas » le sont-elles vraiment ?
2. **L'enveloppe et les pièges** (C-32-9-3, C-32-9-5, C-32-9-14) : une injection sort-elle de `{"nonFiable": "…"}` ? Le
   parcours (`P(IEGE_)?329[AB]`) peut-il se tromper — un `P329` qui apparaîtrait dans une donnée légitime de la fixture,
   un piège T (7 caractères) qui ne porte plus de consigne et ne prouve donc que l'enveloppe ? `largeurs_des_pieges`
   passe-t-il à vide ? La fonctionnalité `pieges` en dépendance de développement tient-elle hors du binaire de production
   avec le résolveur du workspace (`Cargo.toml` racine) ?
3. **Les tiers, révisés** (C-32-9-9) : la conception est-elle **honnête** — masquer l'IBAN protège-t-il quelque chose,
   ou l'IBAN sort-il par un autre chemin (un document, un libellé saisi, un export) ? La recherche `search`
   (`factures.rs:80`) lit-elle vraiment ni l'IBAN ni `emailed_to` ? La Q1 posée à Guy décrit-elle exactement la fuite ?
4. **La lecture seule et la société** (AC-a19, AC-a20) : `CHECKSUM TABLE` permis à `kesh` ; table des écritures
   attendues (sous `required`, un rapport : la trace de consultation **et** l'entrée du dépôt) ; l'aide
   `societe_complete` réalisable par les dépôts que `seed_demo` emploie, hors de sa garde d'unicité ?
5. **Les tailles** (C-32-9-7, C-32-9-8) : refais une mesure (`printf` d'un élément dans la forme prescrite, `wc -c`) pour
   au moins deux outils — la fiche a-t-elle oublié un champ qui grossit l'élément (lignes d'une écriture, `piece` d'un
   poste ouvert, résumé de l'échéancier) ? Le « ≈ 160 comptes » des rapports non paginés tient-il ? `plafond_au_maximum`
   est-il réalisable (des libellés de 500 caractères dans chaque colonne libre, dont des `VARCHAR(255)`) ?
6. **Le découpage** (C-32-9-1) : recompte les modules avec la convention écrite — 5 et 5, après les fichiers ajoutés
   (`detail_sans_fichier`, réexportations, deux `Cargo.toml`) ? Les fichiers de chaque partie restent-ils dans ses
   modules ? La 32-9b peut-elle glisser sans retouche (AC-b7) ?
7. **Tests et mutations** : chaque AC est-il vérifiable tel qu'écrit, avec son test nommé ? Chaque mutation Ma1 à Ma25,
   Ma3b, Ma4b, Mb1 à Mb7 est-elle **réalisable** et tuée par le test **nommé** — une passerait-elle à vide (Ma15, Ma17,
   Ma22, Ma25, Mb7) ? Les gardes de l'AC-a24 suivent-elles la **règle commune** (propriété, périmètre, formes, témoin
   rouge, témoin vert existant ou factice dit tel) ?
8. **Décomptes** : recompte à la source — 31 AC (24 / 7), 16 tâches (10 / 6), 34 mutations (27 / 7), C-32-9-1 à 28,
   seize outils au registre après la 32-9a, dix-neuf après la 32-9b — et la cohérence fiche, registre (annotations
   datées, non réécritures, sauf C-32-9-7 à 9 réécrits sur décision et marqués tels), `sprint-status.yaml`.
9. **Ce que la fiche omet** : un report de la spike ou des fiches 32-6, 32-8, 32-10a non repris ; un point qui devrait
   aller à Guy ou à l'orchestrateur et n'y est pas ; ce que la 32-9 laisse aux 32-7, 32-11 — juste, complet ?

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt ou de la fiche
voisine, preuve (commande et sortie copiée), correction proposée ; dis, pour chacun, s'il **vient de la remédiation de la
P1** ou de la rédaction d'origine. Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET non
exercés** (R1 à R6 ou F1 à F9, et les deux axes obligatoires A et B) — sans elle, « 0 finding » ne compte pas comme
passe. Rapport écrit **d'un seul jet**, un seul fichier par lentille, avec ton outil d'écriture de fichier (jamais par une
redirection du shell) : `/home/gcorbaz/devel/kesh-gate-logs/329-validate-p2-R.md` et
`/home/gcorbaz/devel/kesh-gate-logs/329-validate-p2-F.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`, `<<`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au
rapport, il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, **`scripts/test-fast.sh`**, **`scripts/mem-guard.sh`** et
**`scripts/regen-test-schema.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, `wget`,
**`python3`, `python`, `perl`, `ruby`, `node`, `awk`** et tout autre interpréteur, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, et **`gh` sous toutes ses formes** (même en lecture).
`pdftotext` vers la sortie standard seulement ; `printf … | wc -c` et `wc -m` permis pour mesurer. Aucune donnée réelle
de Guy (rien de `~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh` est hors de ton périmètre ; les worktrees
`kesh-32-6`, `kesh-32-8` et `kesh-32-10a` se lisent, ne s'écrivent pas. Autorisés : lecture, `grep`, `sed -n`,
`git log|show|diff|blame|merge-base`, `pdftotext … -`, et les filtres en lecture seule `tr`, `sort`, `cut`, `head`,
`tail`, `wc`, `uniq`, `printf` vers la sortie standard. Si une commande t'est refusée, arrête-toi et dis-le dans ton
rapport.
