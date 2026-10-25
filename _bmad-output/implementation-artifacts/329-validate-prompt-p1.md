# Prompt — validation P1, Story 32-9 (les outils de lecture du serveur MCP)

*Versionné le 2026-10-25. Passe **complète** (première passe). **Deux lentilles, R et F**, chacune en contexte frais,
**Sonnet**. Lecture seule stricte, **sans `gh`**, **sans interpréteur**, **sans redirection**.*

Dépôt `/home/gcorbaz/devel/kesh-32-9`, branche `story/32-9` (partie de `main` `fa5b3468` : 32-3, 32-4a, 32-4b, 32-5a,
32-5b, 32-5d fusionnées ; **les 32-6, 32-8 et 32-10a ne le sont pas** — la fiche en dépend, le dit, et cite leurs fiches
validées). Fiche : `_bmad-output/implementation-artifacts/32-9-outils-de-lecture.md` (deux parties : 32-9a, 32-9b) ;
registre : `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-329.md` (C-32-9-1 à 25) ; entrées
`32-9a-*` et `32-9b-*` de `_bmad-output/implementation-artifacts/sprint-status.yaml`.

Contexte, à lire : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-9*, § *Points d'attention* — la
matrice, l. 178 —, § *Décisions*, § *Questions pour Guy* et *Réponses de Guy*) ; la spike
`_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (**en entier** ; en particulier § 2.1 — limites de Claude Code —,
§ 5.3.1 — E1, E3, E5 —, § 6 — entrée *32-9a / 32-9b* —, § 7 et ses réponses Q10, Q11, Q12) ; les fiches **en lecture
seule** des worktrees voisins :
`/home/gcorbaz/devel/kesh-32-6/_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md` (AC-a13, AC-a16 à AC-a24,
AC-a28, AC-a30, AC-a31, § *L'inventaire clos*, Dev Notes, et son registre `epic-32-choix-autonomes-326.md`),
`/home/gcorbaz/devel/kesh-32-8/_bmad-output/implementation-artifacts/32-8-validation-avant-envoi.md` (AC-02, AC-B1 à
AC-B5, § *La propriété centrale*, § *Les signatures*, § *Ce que la 32-8 laisse aux stories suivantes*, et
`epic-32-choix-autonomes-328.md`), `/home/gcorbaz/devel/kesh-32-10a/_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md`
(§ des textes standard, l. 1255-1274) ; les fiches `32-3-services-motif.md`, `32-4a-*`, `32-4b-*` (Q1 de la 32-4b) de ce
dépôt. Règles : `CLAUDE.md` (§ *Review Iteration Rule* et *La passe ciblée* — une boucle ne se clôt que sur une passe
sans finding au-dessus de LOW —, *Inventorier les sites NON RÉSOLUS*, *Règle de splitting préventif*, *Le prompt d'une
passe doit NOMMER le manuel*, *Recompter ses propres comptes rendus*, *Un appariement automatique propose…*),
`docs/agents-orchestration.md`.

**Contraintes de la story, à tenir pour vraies et à contrôler** : portées `books:read`, `invoices:read`,
`contacts:read`, `documents:read` (Q5 sans réponse : hypothèse) ; la Q1 de la 32-4b tranchée **en hypothèse**,
réversible par une constante ; lecture seule **applicative** (Q11) ; validation **par clé** (Q12) : sous `required`,
tout passe par le dépôt de la 32-8 ; **aucune ressource MCP** en 0.16.0 (E3) ; la 32-9b peut glisser en 0.17.0 ; au plus
cinq modules par partie ; **jamais de donnée réelle**. Ce que la fiche **prescrit** au développement n'existe pas encore
: son absence n'est pas un défaut. Ce que la fiche dit des fiches 32-6, 32-8, 32-10a se juge **contre ces fiches**, non
contre le code (qui ne les porte pas encore).

## Les deux axes obligatoires, pour chaque lentille

- **(A) L'inventaire clos des champs libres** (§ *L'inventaire*) : cherche une `String` d'un type rendu par un service
  appelé (fait 6 ; relis chaque `struct` à sa source, `sed -n`) qui **n'est pas** à l'inventaire, ou un chemin par lequel
  un texte lu en base atteindrait l'agent **hors** de `TexteNonFiable`/`Tiers`/`Code` — un champ d'un type imbriqué, une
  `Vec<String>`, une clé de table associative, une erreur, le résumé, l'en-tête, `tools/list`, un message de service
  recopié, une énumération à `Serialize` manuel, un `&'static str` fabriqué à l'exécution. Le typage proposé
  (`ChampSur`, `sortie!`, le sceau `pub(in crate::mcp)`) **ferme-t-il vraiment** la classe en Rust — ou un `impl`, un
  `pub use`, un développement de macro, un `#[serde(serialize_with)]`, un `#[serde(flatten)]` sur une `HashMap`, ouvre-t-il
  une issue ?
- **(B) Le manuel** (axe obligatoire de `CLAUDE.md`) : la sous-section que l'AC-a23 et l'AC-b5 prescrivent dans
  `docs/manual/fr/admin-manual.tex`, sa place (section de la 32-7 si elle existe, sinon chapitre de l'API) et ses
  affirmations contre le code prescrit ; ce que la fiche dit de ce qu'écrivent la 32-7 et la 32-11 (fiches de l'epic) ;
  et **le PDF aplati** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE
  ".{60}(MCP|portée|outil).{80}"`. Un texte publié devient-il faux avec la 32-9 sans que la fiche le nomme
  (`grep -n "MCP" README.md website/*.html docs/api-external.md`) ?

## Lentille R — la spécification contre le code et contre les fiches dont elle dépend

Pour chaque affirmation, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**. Axes, tous à exercer :

1. **Les faits 1 à 12** (§ *Ce que le code dit*) : services, exigences, bornes (constantes et comportement : refus ou
   écrêtage), `Rapport::afficher`, les types et leurs `String`, les contraintes `CHECK`, la validation d'un numéro de
   compte, la dérivation de `Serialize` des énumérations (fait 9), `seed_demo` et `final_body`, `serde_json` sans
   `preserve_order`. Chaque numéro de ligne est-il juste à `fa5b3468` ?
2. **Les emprunts aux fiches 32-6 et 32-8** (§ *Ce que disent les fiches*) : chaque nom (`McpTool`, `ContexteMcp`,
   `LectureOutils`, `BlocContenu`, `Donnees::nouvelles`, `ErreurOutil` et ses variantes, `arguments`, `depuis_app_error`,
   `Registre::{standard, …}`, `registre_coherent`, `rendre_visible`, `MCP_ENVOI_OCTETS_MAX`,
   `MCP_ENVOI_RESUME_OCTETS_MAX`, `TEXTE_ENVOI_DEPOSE`, `TEXTE_OUTIL_INCONNU`, `TEXTE_INTROUVABLE`, la route de décision)
   existe-t-il **sous ce nom et cette forme** dans la fiche qui le crée ? Chaque amendement d'un AC de la 32-6 (AC-a13,
   AC-a16, AC-a17, AC-a28, AC-a30) est-il **nommé** comme tel, et ses tests à amender identifiés ? La garde
   `mcp_sans_pool` (formes interdites) est-elle tenue par les outils prescrits — dont la réexportation des types des
   dépôts (C-32-9-25) ?
3. **Les outils** (AC-a10, AC-b2) : chaque outil appelle-t-il un service qui **existe** avec la signature dite
   (arguments, demande, type rendu) ? Les arguments listés existent-ils dans la demande du service (noms de champs,
   types) ? Le défaut et le plafond de chaque ligne du tableau égalent-ils la source ? L'exigence MCP du service est-elle
   la portée annoncée ? Les deux services neufs (`exercices`, `pieces`) respectent-ils R1 à R8 (`services/mod.rs`) et le
   RBAC de leur route (`lib.rs`) ?
4. **Le socle** (AC-a1 à AC-a9) : `rendre_visible` (32-8b) est-il appelable depuis `src/mcp/` ; `Rapport<T, Affichage>`
   nommable depuis `mcp/outil.rs` ; `ContexteMcp::tiers_visibles` sans enfreindre la garde N24 de la 32-6 ; l'en-tête et
   le plafond compatibles avec l'AC-B2 et l'AC-B4 de la 32-8 (le texte déposé égal au texte de l'outil : est-ce vrai,
   octet pour octet, après le second `rendre_visible`) ?
5. **Les pièges** (AC-a14, AC-b3) : les dépôts nommés (`contacts::create`, `imported_supplier_invoices::create`,
   `fiscal_years::create`, ceux des écritures et des factures) existent-ils et peuvent-ils poser ce que la fiche dit
   (une facture **validée** par un dépôt ? un `emailed_to` ? une référence de paiement non validée ?) — et les
   exemptions annoncées sont-elles **les bonnes** ?
6. **Le registre de sprint** : les deux entrées `32-9a-*`, `32-9b-*` se lisent-elles comme du YAML (`clé: valeur  #
   commentaire`, deux espaces d'indentation, aucun caractère qui ouvre une structure hors commentaire) et disent-elles
   les dépendances de la fiche ?

## Lentille F — faisabilité, découpage, sécurité et fausses prémisses

1. **Le typage** (C-32-9-2) : écris en esprit `sortie!`, `champ_enum!` et le sceau — compilent-ils (une macro
   `macro_rules!` qui pose `exige::<$ty>()` pour chaque champ ; `const _: fn(E) -> isize = |x| x as isize;` pour une
   énumération sans champ ; un module `pub(in crate::mcp)` atteint depuis un développement dans `mcp::outils`) ? Un
   champ `Option<String>`, `Vec<String>`, `Box<str>`, `Cow<str>` est-il bien refusé ? Une structure sans la macro peut-elle
   atteindre `donnees` ? Les mutations « ne compile pas » (Ma3, Ma3b) le sont-elles vraiment ?
2. **L'enveloppe** (C-32-9-3, C-32-9-5) : une injection peut-elle sortir de `{"nonFiable": "…"}` (échappement de
   `serde_json`, caractères de contrôle, U+2028/U+2029, substitution) ? Le parcours du test de l'AC-a15 peut-il passer à
   vide (piège absent, sortie vide, `isError`) — le témoin `inventaire_couvert` le ferme-t-il ? Sous `required`, le JSON
   « qui suit l'en-tête » s'analyse-t-il après `rendre_visible` ?
3. **Les tiers** (C-32-9-9) : la liste des champs `Tiers` est-elle complète au regard de ce que la question (Q1 de la
   32-4b) vise — un nom, un IBAN, une adresse d'un tiers dans un **autre** champ rendu sous `invoices:read` ou
   `books:read` (la pièce source d'une écriture, le document d'un poste ouvert, la balance âgée) ? La réversibilité
   « par une constante » est-elle réelle ?
4. **La lecture seule et la société** (AC-a19, AC-a20) : `CHECKSUM TABLE` est-il permis à l'utilisateur `kesh` du
   fichier versionné, et voit-il un `UPDATE` ? La table des écritures attendues (une ou deux lignes d'audit,
   `mcp_keys`, `mcp_sends`) est-elle juste — la 32-8b écrit-elle autre chose au dépôt (purge, audit) ? Le montage à deux
   sociétés existe-t-il dans la fiche 32-6 ?
5. **Les tailles** (C-32-9-7, C-32-9-8) : un appel **par défaut** de chaque outil, sur une base ordinaire (le plan PME,
   un exercice de quelques centaines d'écritures), tient-il sous 65 536 octets de texte indenté ? Le grand livre sans
   `accountIds` ? La tranche du plan comptable et des pièces en mémoire est-elle acceptable ?
6. **Le découpage** (C-32-9-1) : recompte les modules avec la convention écrite — 5 et 5 ? Les fichiers de chaque partie
   restent-ils dans ses modules (la réexportation touche `services`, la migration des gestionnaires `routes`) ? La
   32-9b peut-elle glisser sans retouche (AC-b7) ?
7. **Tests et mutations** : chaque AC est-il vérifiable tel qu'écrit, avec son test nommé ? Chaque mutation Ma1 à Ma22,
   Ma3b, Ma4b, Mb1 à Mb6 est-elle **réalisable** et tuée par le test **nommé** — une passerait-elle à vide (Ma15, Ma17,
   Ma18, Ma22) ? Les gardes de l'AC-a24 suivent-elles la **règle commune** (propriété, périmètre, formes, témoin rouge,
   témoin vert existant ou factice dit tel) ?
8. **Décomptes** : recompte à la source — 31 AC (24 / 7), 16 tâches (10 / 6), 30 mutations (24 / 6), C-32-9-1 à 25,
   seize outils au registre après la 32-9a, dix-neuf après la 32-9b, les lignes de l'inventaire — et la cohérence fiche,
   registre, `sprint-status.yaml`.
9. **Ce que la fiche omet** : un report de la spike ou des fiches 32-6, 32-8, 32-10a non repris ; un point qui devrait
   aller à Guy ou à l'orchestrateur et n'y est pas ; ce que la 32-9 laisse aux 32-7, 32-10a, 32-11 — juste, complet ?

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt ou de la fiche
voisine, preuve (commande et sortie copiée), correction proposée. Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La
liste des axes exercés ET non exercés** (R1 à R6 ou F1 à F9, et les deux axes obligatoires A et B) — sans elle,
« 0 finding » ne compte pas comme passe. Rapport écrit **d'un seul jet**, un seul fichier par lentille, avec ton outil
d'écriture de fichier (jamais par une redirection du shell) :
`/home/gcorbaz/devel/kesh-gate-logs/329-validate-p1-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/329-validate-p1-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`, `<<`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au
rapport, il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, **`scripts/test-fast.sh`**, **`scripts/mem-guard.sh`** et
**`scripts/regen-test-schema.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, `wget`,
**`python3`, `python`, `perl`, `ruby`, `node`, `awk`** et tout autre interpréteur, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, et **`gh` sous toutes ses formes** (même en lecture).
`pdftotext` vers la sortie standard seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal
`/home/gcorbaz/devel/kesh` est hors de ton périmètre ; les worktrees `kesh-32-6`, `kesh-32-8` et `kesh-32-10a` se
lisent, ne s'écrivent pas. Autorisés : lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`,
`pdftotext … -`, et les filtres en lecture seule `tr`, `sort`, `cut`, `head`, `tail`, `wc`, `uniq` vers la sortie
standard. Si une commande t'est refusée, arrête-toi et dis-le dans ton rapport.
