# Story v016-1 : correctif 0.16.1 — ce que le serveur MCP rend à l'agent

Status: ready-for-dev

*Fiche rédigée le 2026-10-28, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0161`, branche
`story/v016-1-mcp-sorties` partie de `main` `43926368` (release v0.16.0 et rétrospective de l'Epic 32 fusionnées).
Registre : `v016-1-choix-autonomes.md` (C-v016-1-1 à C-v016-1-10). Clé courte : `v0161`. Issues : **#811** (P2) et
**#812** (P4), relevant de #723 — en `refs` seulement dans les commits ; la PR portera `closes #812` et, pour #811, voir
§ *Ce que la release doit contenir*, point 9. Les numéros de ligne cités sont ceux de `43926368`.*

## Story

En tant que Guy, qui branche Claude Code sur Kesh sous une clé à validation requise, je veux que l'agent **reçoive le
contenu que j'ai validé** — et, plus généralement, tout ce que Kesh lui écrit (l'en-tête des données, la consigne de
l'accusé) —, et que les montants sortent sous **une seule forme**, afin que le serveur MCP soit utilisable avec ce client
dans le mode que le manuel recommande, et que le manuel dise enfin ce que la recette du 2026-10-28 a observé.

## Origine

- **#811 (P2)** — recette Claude Code de la 32-11, 2026-10-28 (Claude Code **2.1.312**, transport HTTP, protocole
  négocié `2025-11-25`, Kesh 0.16.0 publiée, base neuve sans donnée réelle). Sous une clé requise, la relecture d'un
  envoi validé (`kesh_lire_envoi`) renvoie bien le texte validé dans le second bloc de `content`, mais Claude Code ne
  transmet au modèle que `structuredContent`, qui ne porte que `{"envoi":1,"etat":"valide","modifie":false}` : **aucune
  donnée validée n'atteint l'agent**. Reproduit sur l'envoi n° 2. Sans validation, le chemin direct marche (le
  `structuredContent` y porte les données). Décision du Project Lead : corriger à la main ; le passage à `rmcp` se
  réexamine au kickoff de l'Epic 33.
- **#812 (P4)** — même recette : `kesh_bilan` rend `"report": "0.0000"` et `"resultat": "0"`, `"totalActifs": "0"` dans
  le même objet.
- **#723, commentaire de recette** — les observations (a) à (h) du § 2.4 de la spike (`epic-32-spike-mcp.md:306-345`),
  à verser au manuel : « retrait des « non éprouvé » établis ici dans le manuel et phrase du CHANGELOG « observé avec
  Claude Code 2.1.312 », à la release suivante ».

## Ce que le code dit (relevé le 2026-10-28, `43926368`)

### Prémisses de #811 — confirmées, et élargies

1. **Confirmée** — `crates/kesh-api/src/mcp/reponse.rs:62-76`, `ReponseEnvoi::Valide` : `content` porte deux blocs
   (`texte_envoi_valide(n, modifie)` puis `valide.texte()`), `structuredContent` porte `{"envoi", "etat": "valide",
   "modifie"}` seulement.
2. **Confirmée, et c'est la même classe** — quatre autres sites posent un `structuredContent` qui ne porte pas tout ce que
   porte `content` :
   - `reponse.rs:49-56` (`Depose`) et `:57-61` (`EnAttente`) : `{"envoi", "etat": "en_attente"}` ; le texte
     (`textes.rs:198-205`, `:208-212`) porte en plus la consigne « Demandez-le-lui, puis appelez kesh_lire_envoi avec
     {"envoi": n} ; ne le rappelez pas en boucle » et le chemin `/admin/mcp/sends/n` ;
   - `prompts.rs:207-216` (`AccuseProposition::en_resultat`) : `{"nom", "etat": "a_valider"}` ; le texte
     (`textes.rs:56-61`) porte « elle attend l'approbation d'un administrateur … Ne la proposez pas de nouveau » ;
   - `politique.rs:454-465` (`DonneesEnvoyables::en_resultat`, chemin direct) : `structuredContent` = la structure de
     l'outil ; le texte = `TEXTE_ENTETE_DONNEES` (`textes.rs:164-167`), une ligne vide, le JSON indenté de la même
     structure (`sortie.rs:423-439`). Les **données** passent ; l'**en-tête** non — dont la seule explication de la valeur
     masquée (« (masqué : portée contacts:read requise) » est retenue par Kesh).
3. **La piste de l'issue (« non observé ici ») est fondée** : `DonneesEnvoyables::en_resultat` joint la structure dès que
   `Donnees.structure` est `Some`, ce qui est **toujours** le cas en production (`sortie.rs:458`, seul appel de
   `Donnees::nouvelles` dans `src/`).
4. **Les erreurs** (`reponse.rs:77-79`, `:284-291`) ne portent jamais `structuredContent` : `content` seul.
5. **Le canal `instructions`** (`textes.rs:149-156`, servi par `initialize`, `repartiteur.rs:187-198`) porte la mise en
   garde `nonFiable` et la consigne de relecture. La recette montre qu'il atteint l'agent : de l'accusé, le modèle n'a reçu
   que `{"envoi":1,"etat":"en_attente"}`, et il a pourtant expliqué l'attente et rappelé `kesh_lire_envoi` après l'accord
   (observation (d)). C'est ce qui a rendu le défaut de l'accusé invisible à la recette (C-v016-1-3).
6. ⚠️ **Prémisse non établie, à ne pas affirmer** : que Claude Code transmette `content` quand `structuredContent` est
   **absent**. C'est la règle du schéma (`CallToolResult.content` requis, `structuredContent` facultatif), mais la recette
   ne l'a pas vu : la capture (`kesh-gate-logs/3211/capture-recette.log`) ne contient **aucune** réponse sans
   `structuredContent` (aucun `"isError":true`, chaque résultat porte la clé). D'où C-v016-1-7.

### La spécification (`2025-11-25`, `server/tools`, relue le 2026-10-28)

- *Structured Content* : « **Structured** content is returned as a JSON object in the `structuredContent` field of a
  result. For backwards compatibility, a tool that returns structured content SHOULD also return the serialized JSON in
  a TextContent block. » — la clause va du structuré vers le texte ; elle ne dit rien d'un serveur qui n'en rend pas.
- *Output Schema* : « If an output schema is provided: Servers **MUST** provide structured results that conform to this
  schema. » — aucun outil de Kesh ne déclare d'`outputSchema` (C-32-9-6) : `structuredContent` n'est exigé nulle part.
- L'exemple de réponse de *Calling Tools* ne porte que `content` et `isError`.

### Prémisses de #812 — confirmées

- `sortie.rs:197-206` : `Montant(pub Decimal)` sérialise le `Decimal` par le trait (`serde-str`, `Cargo.toml:47`),
  échelle comprise. Les colonnes sont en `DECIMAL(19,4)` (25 occurrences dans les migrations), `(18,2)`, `(7,2)`,
  `(5,2)` ; une somme calculée a l'échelle de son calcul.
- `Decimal` n'est pas `ChampSur` (`sortie.rs:59-87`, la liste des implémentations) : **tout** décimal d'une sortie MCP
  passe par `Montant` — inventaire clos par le typage (C-v016-1-6). `Montant` porte aussi des **quantités** et des **taux
  de TVA** (`factures.rs:236-238`, `factures_fournisseur.rs:88-90`) : une échelle fixe à deux décimales perdrait des
  quantités réelles.
- Les tests MCP comparent les montants **par valeur** (`mcp_enveloppe_e2e.rs:1148`, `montant(v) -> Decimal` ;
  `mcp_outils_e2e.rs:826-838`) ; aucun littéral de montant n'y est épinglé sous forme de chaîne de sortie (`grep -nE
  '"-?[0-9]+\.[0-9]{2,4}"' tests/mcp_*.rs` ne rend qu'un argument d'API, `mcp_enveloppe_e2e.rs:1224`).

## Critères d'acceptation

### #811 — un seul canal

**AC-1 — La règle.** Aucun `result` de `tools/call` ne porte la clé `structuredContent` ; ses clés sont exactement
`content` et `isError`. Les quatre constructeurs la retirent :

| Site | Fichier | Avant | Après |
|---|---|---|---|
| S1 — sortie d'outil, chemin direct | `politique.rs:454-465` | `content` + `structuredContent` + `isError` | `content` (un bloc, inchangé) + `isError` |
| S2 — accusé de dépôt | `reponse.rs:49-56` | idem | un bloc, inchangé |
| S3 — relecture « en attente » | `reponse.rs:57-61` | idem | un bloc, inchangé |
| S4 — relecture validée | `reponse.rs:62-76` | idem | deux blocs, inchangés |
| S5 — accusé de proposition | `prompts.rs:207-216` | idem | un bloc, inchangé |
| S6 à S8 — relecture refusée, échue, inconnue | `reponse.rs:77-79` | `content` + `isError: true` | inchangé |
| S9 — erreur d'outil (`OutilEnErreur`) | `reponse.rs:284-291` | `content` + `isError: true` | inchangé |

**Aucun texte ne change** — ni les blocs de S1 à S9, ni `INSTRUCTIONS`, ni les descriptions d'outil (C-v016-1-3) : le
test `textes_envoi_epingles_par_un_litteral` (`textes.rs:342-375`) et le littéral K1 (`mcp_contrat.rs:76-84`) restent
verts sans retouche. Les doc-comments qui décrivent l'émission (`reponse.rs:19-22` et `:39-40`, `politique.rs:336-339`
et `:451-453`, `prompts.rs:204-206`, `sortie.rs:4-9` et `:441-442`, `mcp/mod.rs` s'il en parle) sont réécrits —
`grep -rn "structur" crates/kesh-api/src/mcp/` relu en entier, chaque occurrence restante justifiée au Dev Agent Record.

**AC-2 — `Donnees` à un seul champ de données** (C-v016-1-5). `Donnees::nouvelles(texte: String)` ; le champ
`structure` disparaît (`politique.rs:330-346`) ; `sortie::donnees` passe le seul texte (`sortie.rs:458`) ;
`texte_et_valeur` garde sa signature et son test. Les gardes de `tests/mcp_gardes_lexicales.rs` qui citent la forme
(`:611-617`, `:638`) suivent le code — **leur motif** (`\bDonnees\s*::\s*nouvelles\b|\bDonnees\s*\{`) ne change pas, seuls
les témoins factices et la ligne attendue.

**AC-3 — L'invariant de contrat** (C-v016-1-2), écrit tel quel en tête du cas qui le porte :

> Soit `r` le `result` d'un `tools/call`. `L_C(r)` est la suite des `text` des blocs de `r.content` ; `L_S(r)` est
> `r.structuredContent` s'il est présent, `L_C(r)` sinon (Claude Code 2.1.312 suit la première branche, observé ; le
> schéma impose la seconde). **I** : pour tout `r`, réussi ou en erreur, les clés de `r` sont exactement `{content,
> isError}` — d'où `L_S(r) = L_C(r)`. **T** (témoin, par site réussi) : `L_C(r)` porte ce que le site doit rendre.

Cas **K23** `k23_un_seul_canal` dans `tests/mcp_contrat.rs`, marque `DECISION_KESH (v016-1, C-v016-1-1 ; #811)` pour I,
`TEMOIN` pour T. Il parcourt **par HTTP**, sur le banc de K10 (`banc`, clés `dispensee` et `requise`), les sites :

| Site | Geste | T |
|---|---|---|
| S1 | `kesh_plan_comptable` sous la clé dispensée | `COMPTE_CONTRAT` dans `L_C` **et** le JSON après la première ligne vide se lit (`serde_json::from_str`) |
| S2 | le même outil sous la clé requise | le littéral de l'accusé (celui de K10), numéro lu dans le texte |
| S3 | `kesh_lire_envoi` avant la décision | le littéral « en attente », écrit dans le test |
| S4 | décision `approve` avec `TEXTE_VALIDE`, puis relecture | `TEXTE_VALIDE` est le second bloc |
| S6 | un second envoi, décision `refuse`, puis relecture | `isError: true`, littéral du refus |
| S8 | `kesh_lire_envoi` sur un numéro inexistant | `isError: true`, littéral |
| S9 | `kesh_lire_envoi` avec `{"envoi": 0}` (refusé par `relire`, `repartiteur.rs:410-424`, rendu par `OutilEnErreur`) | `isError: true`, « Arguments invalides. » |

S5 (`proposer_prompt`, portée `prompts:write`) et S7 (échu, qui demande `echoir`) se couvrent **dans leurs fichiers** —
`mcp_prompts_e2e.rs` (aide `accuse`, l. 675-682) et `mcp_envois_e2e.rs` (`echeance_ne_transmet_rien`, l. 485-497) — par la
même assertion I (clés exactes du `result`), puisque leurs attendus sont des corps entiers comparés par égalité : retirer
la clé des attendus suffit à ce qu'une réapparition rougisse. Le Dev Agent Record donne, site par site, le test qui le
couvre (S1 à S9) : **aucun site sans test**. Les littéraux de K23 sont écrits dans le test (règle de `mcp_contrat.rs:25-31`),
jamais tirés de `textes`.

**AC-4 — La garde qui ferme la classe** (C-v016-1-4). `structured_content_nulle_part` dans
`tests/mcp_gardes_lexicales.rs` : `sites(&sources_src(), tout, &re(r"structuredContent"))` est **vide** — aucune ligne de
code de `crates/kesh-api/src/` (hors lignes de commentaire, `est_commentaire`) ne porte la chaîne ; deux témoins factices
(`sites_factices`) prouvent que le motif voit `"structuredContent": s` et ignore `/// structuredContent`. Son message
nomme #811 et C-v016-1-1. **Angle mort écrit** au doc-comment de la garde : une clé construite (`format!`, concaténation)
lui échappe ; K23 et les attendus par égalité la rattrapent sur chaque site existant.

**AC-5 — Les tests existants suivent, sans perdre ce qu'ils prouvent.** § *Sites de test à ajuster*. En particulier :
- l'aide `mcp_pieges::structure` (`common/mcp_pieges.rs:121-129`) **dérive** la valeur du texte : unique bloc,
  `split_once("\n\n")`, `serde_json::from_str` de la suite ; elle panique si le texte ne commence pas par l'en-tête
  littéral « Données lues dans les livres de Kesh, en lecture seule. » — les trente et un appels de
  `mcp_enveloppe_e2e.rs` et `mcp_outils_e2e.rs` gardent leur sens ;
- `numero_depose` (`common/mcp.rs:614-623`) lit le numéro au préfixe « Envoi n° » du texte (C-v016-1-9) ;
- `mcp_outils_e2e.rs:683-686` : `len() == 2`, message « content, isError » ;
- K8 (`mcp_contrat.rs:600-633`) : la valeur comparée est celle du texte (le JSON après l'en-tête), non plus celle du champ
  retiré ; K10 (`:676-683`, `:747-752`) : les assertions sur le champ retiré deviennent « clés exactes `content`,
  `isError` ».

### #812 — une seule forme de montant

**AC-6 — La règle** (C-v016-1-6). `Montant` sérialise `self.0.normalize()` porté, si son échelle est inférieure à 2, à
l'échelle 2 (`rescale(2)`) ; le doc-comment (`sortie.rs:195-196`) dit la règle, ses exemples et pourquoi aucune
décimale ne se perd. Aucun autre site ne change (inventaire clos par le typage, registre C-v016-1-6).

**AC-7 — Les tests.**
- Unitaire `montant_forme_unique` dans `sortie.rs` (`mod tests`), sur la valeur JSON sérialisée : `Decimal::new(0, 4)` →
  `"0.00"` ; `Decimal::ZERO` → `"0.00"` ; un zéro négatif (`-Decimal::new(0, 4)`, ou `Decimal::from_parts(0, 0, 0, true,
  4)` — à vérifier à l'étape 0 qu'il donne bien un zéro signé) → `"0.00"` ; `Decimal::new(1_234_500, 4)` → `"123.45"` ;
  `Decimal::new(100, 0)` → `"100.00"` ; `Decimal::new(125, 4)` → `"0.0125"` ; `Decimal::new(810, 2)` → `"8.10"` ;
  `Decimal::new(-5, 1)` → `"-0.50"`. Le test `donnees_une_seule_source` (`sortie.rs:567-592`) reste vert (`"123.45"`).
- E2E `bilan_montants_une_seule_forme` dans `mcp_outils_e2e.rs` : sur un exercice **sans écriture**, `kesh_bilan` sous une
  clé dispensée ; chaque champ montant de la sortie (`totalActifs`, `totalPassifs`, `totalFondsPropres`, `report`,
  `resultat`, et le `solde` de chaque ligne s'il y en a) vaut la **chaîne** `"0.00"`. **Rouge avant le correctif**
  (`"0.0000"` contre `"0"`, constaté par la recette) — le développeur le constate et le note.

### Les textes

**AC-8 — Manuel d'administration FR** (`docs/manual/fr/admin-manual.tex`, C-v016-1-7). Chaque retrait cite
l'observation et porte « observé avec Claude Code 2.1.312 » :

| Lieu | Aujourd'hui | Après |
|---|---|---|
| `sec:mcp-client`, l. 3314 | « La configuration de ce client n'est pas éprouvée contre un client réel… » | la commande `claude mcp add --transport http --scope local … --header "Authorization: Bearer …"` a été **employée** à la recette du 2026-10-28 (Claude Code 2.1.312, Kesh 0.16.0 derrière Traefik en HTTP) ; la portée `user` et l'expansion `${…}` restent « d'après sa documentation » ; la limite de 25 000 jetons aussi |
| `sec:mcp-brancher`, intro l. 3331 | parcours joué en client HTTP | ajouter la recette avec Claude Code 2.1.312 (base neuve, aucune donnée réelle) et ce qu'elle établit, sans réécrire ce que le parcours du 27 a établi |
| item 3, l. 3336 | `/mcp` « (non éprouvé) » | (a), (e), (h) observés : `initialize` en `2025-11-25` après une sonde `2026-07-28` refusée en `400` texte, puis `tools/list` et `prompts/list`, sans session. **Ce que montre `/mcp` reste « non éprouvé »** (la recette ne le rapporte pas) |
| item 4, l. 3337 | `/kesh:kesh-assistant (MCP)` « (non éprouvé) » | **corrigé selon (g)** : le menu montre `<serveur>:kesh-assistant` (par exemple `kesh:kesh-assistant`) ; la commande envoyée est `/mcp__<serveur>__kesh-assistant` ; `prompts/get` sans argument. Recopier la forme du commentaire de recette, mot pour mot |
| item 5, l. 3338 | « Ce que l'agent vous en dit (non éprouvé) » | (d) : l'agent explique l'attente et demande qu'on le prévienne |
| item 7, l. 3340 | « Combien de fois un agent relit … (non éprouvé) » | (d) : une relecture par envoi, après l'accord, sans boucle ; une de plus sur demande explicite. « S'il relit avant votre décision … (non éprouvé au parcours : décrit d'après le code) » **reste**. La réception du texte validé : « *(à confirmer avec Claude Code sur la 0.16.1 ; en 0.16.0, ce contenu ne l'atteignait pas, issue \#811)* » — levée par la release, point 6 |
| item 10, l. 3343 | 401 et OAuth « (non éprouvé) » | **reste** — (f) non observé |
| `sec:mcp-origin`, l. 3167 | « Un client qui n'envoie pas d'Origin n'est pas concerné. » | ajouter : Claude Code 2.1.312 n'en envoie pas (b) |
| `sec:mcp-outils`, « Ce que l'agent reçoit », l. 3209 | « Chaque réponse d'un outil commence par un en-tête fixe… » | ajouter : depuis la 0.16.1, la réponse ne porte **que** ce texte, aucun contenu structuré (#811) ; et la forme des montants (« au moins deux décimales, sans zéro final superflu au-delà » — exemple `0.00`, `123.45`, `0.0125`) |
| `sec:mcp-envois`, 3e puce du détail, l. 3227 | « La donnée structurée que l'outil produit aussi n'est pas envoyée sous une clé requise » | « Kesh n'envoie que ce texte, sous toute clé (depuis la 0.16.1) » |
| nouveau `\subsubsection{Passer à la 0.16.1}` avant `sec:maj-0-16-0` (l. 2480) | — | aucune migration ; la 0.16.0 redémarre sur une base passée par la 0.16.1 ; un `.keshbackup` de l'une s'importe dans l'autre (mêmes tables) ; aucun réglage ; un client MCP n'a rien à changer — sauf une intégration qui lisait `structuredContent`, à faire lire le JSON du bloc de texte. **Chaque affirmation vérifiée** (pas de migration : `git diff --stat main -- crates/kesh-db/migrations` vide) |

Le PDF n'est **pas** régénéré par le développeur (release, point 4) ; le `.tex` compile — `make fr` sous verrou
seulement si le développeur le juge nécessaire pour vérifier, sans commiter le PDF (consigne de l'orchestrateur : « PDF
régénérés par l'orchestrateur à la release »). Le **manuel utilisateur** ne décrit pas la forme des réponses
(`grep -n "MCP" user-manual.tex` : réinitialisation, sauvegarde, journal) : inchangé, constat écrit au Dev Agent Record.

**AC-9 — `CHANGELOG.md`** : une section `## [0.16.1] — Non publié` (forme que `scripts/prepare-release.sh:237` attend)
au-dessus de `[0.16.0]`, avec une phrase d'introduction (correctif de l'Epic 32, aucune migration, retour à la 0.16.0 sans
sauvegarde particulière), un paragraphe « observé avec Claude Code 2.1.312 » qui dit ce que la recette a établi, puis :
- **Corrigé** : #811 (l'agent reçoit le contenu validé — et, sur tout appel, l'en-tête et les consignes que Kesh lui
  écrit) ; #812 (une seule forme de montant) ;
- **Modifié** : plus de `structuredContent` dans les réponses d'outil — une intégration qui le lisait lit le JSON du bloc
  de texte, après l'en-tête et une ligne vide.
Pas de lien `closes` dans le texte ; des liens d'issue comme les sections voisines.

**AC-10 — `README.md`, l. 49** : « Le branchement avec Claude Code est décrit au manuel d'administration, non encore
éprouvé par une recette » devient vrai : éprouvé avec Claude Code 2.1.312 le 2026-10-28, la relecture d'un envoi validé
corrigée en 0.16.1 (#811). Le tableau des versions (l. 234) et le site relèvent de la release (point 7).

**AC-11 — `sprint-status.yaml`** : la clé `v016-1-mcp-sorties` passe à `in-progress` puis `review` selon le flux, avec
commentaire daté.

## Sites de test à ajuster (inventaire relevé, `grep -rn "structuredContent" crates/kesh-api/tests`)

| Fichier | Lignes | Geste |
|---|---|---|
| `tests/mcp_contrat.rs` | 600-633 (K8), 676-683 et 747-752 (K10) | AC-5 ; ajout de K23 |
| `tests/common/mcp.rs` | 380, 409, 463, 496, 599 (`Donnees::nouvelles`) ; 614-623 (`numero_depose`) ; 626-648 (`attendu_depose`, `attendu_en_attente`, `attendu_valide`) | signature ; numéro au texte ; attendus sans la clé |
| `tests/common/mcp_pieges.rs` | 121-129 | `structure` dérivée du texte |
| `tests/mcp_transport_e2e.rs` | 105-123, 1545-1556 | attendus sans la clé ; l'outil `Compteur` rend le texte `{"n":1}` |
| `tests/mcp_journal_e2e.rs` | 813 | lire `{"n": 1}` dans le texte |
| `tests/mcp_outils_e2e.rs` | 683-686 | `len() == 2` |
| `tests/mcp_prompts_e2e.rs` | 675-682 | attendu sans la clé |
| `tests/mcp_gardes_lexicales.rs` | 611-617, 638 | témoins et ligne attendue ; garde AC-4 |

Ce relevé n'est pas une preuve d'exhaustivité : le développeur relance le `grep` après ses patches et lit chaque
occurrence restante (code, doc-comments des tests, fiche) ; un test qui comparait un `result` entier sans nommer la clé
rougit au gate, c'est le filet.

## Tâches

1. **Étape 0** — § *À recontrôler* ; constater rouges, **avant tout patch** : K23 (rouge sur S1 à S5), la garde AC-4
   (rouge, cinq lignes), `bilan_montants_une_seule_forme` (rouge, `"0.0000"`), `montant_forme_unique` (rouge).
2. #811 : AC-1, AC-2 ; puis AC-5 ; gate ciblé.
3. #812 : AC-6, AC-7 ; gate ciblé.
4. Textes : AC-8 à AC-10 ; greper la **valeur** des symptômes corrigés (`structuredContent`, `structurée`,
   `non éprouvé`, `0.0000`) sur `docs/`, `README.md`, `CHANGELOG.md`, `website/`, `crates/` — liste au Dev Agent Record.
5. Mutations (§ *Tests et mutations*), sous verrou, restauration `git checkout -- <f> && touch <f>`.
6. Gate complet backend, gate frontend (aucun fichier frontend touché : le gate le confirme), **E2E Playwright complet**
   au dernier commit de code (recette `docs/testing.md`, port et bases attribués par l'orchestrateur), bilan des échecs
   contre la liste des échecs attendus.

## Tests et mutations prévus

| # | Mutation | Doit rougir |
|---|---|---|
| M1 | remettre `"structuredContent": {…}` à `ReponseEnvoi::Valide` | K23 (S4), garde AC-4, `attendu_valide` |
| M2 | le remettre à `Depose` | K23 (S2), garde, `attendu_depose` |
| M3 | le remettre à `AccuseProposition::en_resultat` | garde, `mcp_prompts_e2e` (`accuse`) |
| M4 | réinsérer la structure dans `DonneesEnvoyables::en_resultat` (champ rétabli) | K23 (S1), garde, `mcp_outils_e2e:683` |
| M5 | construire la clé par concaténation dans `Valide` (`format!("structured{}", "Content")` en clé de `Map`) | K23 (S4) et `attendu_valide` — **pas** la garde (angle mort assumé, AC-4) |
| M6 | retirer le second bloc de `Valide` | K23 (T de S4), `attendu_valide` |
| M7 | `Montant` sans `normalize` (rescale seul) | `montant_forme_unique` (`123.4500`), `bilan_montants_une_seule_forme` (`"0.0000"`) |
| M8 | `Montant` sans `rescale(2)` | `montant_forme_unique` (`"0"`), e2e bilan (`"0"`) |
| M9 | `Montant` en `round_dp(2)` | `montant_forme_unique` (`0.0125`) |
| M10 | `structure` de `mcp_pieges` qui rend `Value::Null` | les tests d'enveloppe (preuve que l'aide dérivée est lue) |

Chaque mutation : motif vérifié unique, résultat (test rouge nommé) au Dev Agent Record ; une mutation qui ne rougit
rien est un finding à traiter avant la revue.

## Gates

Gate complet (`scripts/test-fast.sh`, sous verrou, base de gate remise à zéro avant), `cargo fmt`, `clippy -D warnings`,
frontend (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`), E2E complet au dernier commit de code. Ne
déclarer que ce qui a tourné, chiffres recomptés (nextest exécutés et ignorés, Vitest, E2E par fichier contre la liste
des échecs attendus).

## Ce que les gardes ne voient pas

- Une clé `structuredContent` **construite** dans un site nouveau (M5) : seuls des tests par site la verraient.
- Un client autre que Claude Code qui exploiterait `structuredContent` par programme : rien dans Kesh ne le sait (aucun
  n'est documenté) ; le CHANGELOG et *Passer à la 0.16.1* le disent.
- Ce que Claude Code fait d'une réponse sans `structuredContent` : **aucun test de Kesh** ne le prouve (C-v016-1-7). Le
  test de contrat prouve que les deux lecteurs du schéma lisent la même chose ; la recette prouve le client.
- La forme des montants de l'API REST (même `Decimal` tel quel, probablement même défaut) : hors MCP, non traité. Le
  développeur le constate au code et, s'il est confirmé, le **demande en issue** dans son rapport (on trace toujours).

## Ce que la release doit contenir (orchestrateur ; ne pas prescrire au développeur)

1. `scripts/prepare-release.sh 0.16.1` : dix crates à 0.16.1, CHANGELOG daté, rappels lus ; aucune migration, donc
   `kesh_version_min_required` inchangé et aucune exemption du registre de rejeu touchée (à constater).
2. `docs/manual/shared/kesh-style.sty` : `\keshVersion{0.16.1}`, `\keshReleaseDate{…}`, `\keshTargetRelease{v0.16}`
   (inchangé) ; URL `raw.githubusercontent.com/guycorbaz/kesh/v0.16.1/…` des manuels, du README et de `website/` (garde
   G19, `textes_coherents.rs`).
3. Le paragraphe introductif du CHANGELOG relu après datation ; la ligne « observé avec Claude Code 2.1.312 ».
4. `make fr` : les trois PDF régénérés et commités ; PDF contrôlé aplati (`pdftotext … | tr '\n' ' '`) sur les
   phrases de l'AC-8.
5. Docker : image `0.16.1` = `:latest`, smoke `/health` (version `0.16.1`).
6. **Recette courte avec Claude Code contre l'image candidate** (base neuve, clé requise) : un outil, validation, relecture
   — le modèle cite-t-il le texte validé ? Si oui, lever au manuel la mention « à confirmer … sur la 0.16.1 » de l'item 7
   (et le dire au CHANGELOG) ; si non, #811 reste ouverte et le manuel garde la mention. Vérifier au passage que le
   préambule et la consigne de l'accusé atteignent le modèle.
7. `README.md` : ligne du tableau des versions (v0.16.1, l. 234) ; `website/index.html` (l. 70, « v0.16.0 released ») et
   `website/roadmap.html` si une ligne de version l'exige.
8. Décompte des issues ouvertes au tag (objectif : moins de 185, le compte au tag v0.16.0) et des bugs.
9. PR de release : `closes #812` ; `closes #811` **seulement** si le point 6 a confirmé, `refs #811` sinon.

## Dev Notes

### Fichiers touchés (prévus) et modules

- `crates/kesh-api/src/mcp/` : `reponse.rs`, `politique.rs`, `prompts.rs`, `sortie.rs` (un module).
- `crates/kesh-api/tests/` : `mcp_contrat.rs`, `mcp_gardes_lexicales.rs`, `common/mcp.rs`, `common/mcp_pieges.rs`,
  `mcp_transport_e2e.rs`, `mcp_journal_e2e.rs`, `mcp_outils_e2e.rs`, `mcp_prompts_e2e.rs` (un module).
- `docs/manual/fr/admin-manual.tex` (un module).
- `CHANGELOG.md`, `README.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml` (textes).

Quatre modules : sous le seuil de découpage. Aucune migration, aucun fichier frontend, aucune locale.

### Le manuel (axe de revue)

Les revues lisent l'AC-8 contre le **commentaire de recette de #723** (source des observations) et contre le code ;
elles vérifient qu'aucun « non éprouvé » n'a été retiré sans observation citée, et qu'aucun n'a été laissé là où la
recette a vu. Le PDF n'est pas régénéré dans la story : la revue relit le `.tex`.

### Références

- #811, #812, #723 (commentaire de recette du 2026-10-28) — `gh issue view N --comments`.
- Spécification `2025-11-25` : <https://modelcontextprotocol.io/specification/2025-11-25/server/tools>.
- Fiches `32-8-validation-avant-envoi.md` (AC-B5, AC-B6), `32-9-outils-de-lecture.md` (AC-a3, C-32-9-6),
  `32-11-brancher-un-agent.md` (contrat, K1 à K22) ; spike `epic-32-spike-mcp.md` § 2.4 et § 5.3.1.

## À recontrôler à l'étape 0 du développement

Chaque point se relit au code, ligne citée au Dev Agent Record ; une prescription que le code ne confirme pas se réécrit
selon le code.

1. Les numéros de ligne de ce document (S1 à S9, sites de test).
2. Qu'aucun autre site ne produit un `result` de `tools/call` : `reponse.rs:278-291` et les retours de `appeler`
   (`repartiteur.rs:300-608`) ; `ReponseMcp::Resultat` n'y paraît pas.
3. `rust_decimal` du `Cargo.lock` (1.43.0) : `normalize` (« converts -0 to 0 », `decimal.rs:1400`), `rescale` vers le
   haut sans perte (`:1026`), et la façon d'obtenir un zéro négatif pour le test.
4. Que `kesh_bilan` sur un exercice sans écriture rend bien `report` en `"0.0000"` et `resultat` en `"0"` (le rouge
   attendu de l'AC-7) — sinon choisir l'exercice qui le reproduit, et le dire.
5. Le texte exact de l'observation (g) au commentaire de recette de #723, pour l'item 4 du manuel.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-10-28 — fiche créée (Opus 5.5, autonomie) ; registre C-v016-1-1 à C-v016-1-10.
