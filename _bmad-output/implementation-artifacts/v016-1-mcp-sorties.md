# Story v016-1 : correctif 0.16.1 — ce que le serveur MCP rend à l'agent

Status: ready-for-dev

*Fiche rédigée le 2026-10-28, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0161`, branche
`story/v016-1-mcp-sorties` partie de `main` `43926368` (release v0.16.0 et rétrospective de l'Epic 32 fusionnées).
Registre : `v016-1-choix-autonomes.md` (C-v016-1-1 à C-v016-1-10). Clé courte : `v0161`. Issues : **#811** (P2) et
**#812** (P4), relevant de #723 — en `refs` seulement, dans les commits **et** dans la PR de la story ; la fermeture se
porte sur la **PR de release** (§ *Ce que la release doit contenir*, point 10). Défaut voisin, hors périmètre : **#813**
(API REST). Les numéros de ligne cités sont ceux de `43926368` (recontrôlés à la validation P1 ; ceux de `politique.rs`
corrigés ce jour-là).*

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
   - `reponse.rs:49-56` (`Depose`) : `{"envoi", "etat": "en_attente"}` ; le texte (`textes.rs:198-205`) porte en plus la
     consigne « Demandez-le-lui, puis appelez kesh_lire_envoi avec {"envoi": n} ; ne le rappelez pas en boucle » et le
     chemin `/admin/mcp/sends/n` ;
   - `reponse.rs:57-61` (`EnAttente`) : même structure ; le texte (`textes.rs:208-212`) dit seulement que l'envoi attend
     encore et que rien n'est parti — ce qu'il porte de plus que `{envoi, etat}` est cette phrase, non la consigne ;
   - `prompts.rs:207-216` (`AccuseProposition::en_resultat`) : `{"nom", "etat": "a_valider"}` ; le texte
     (`textes.rs:56-61`) porte « elle attend l'approbation d'un administrateur … Ne la proposez pas de nouveau » ;
   - `politique.rs:157-172` (`DonneesEnvoyables::en_resultat`, la clé à `:168`, chemin direct) : `structuredContent` = la structure de
     l'outil ; le texte = `TEXTE_ENTETE_DONNEES` (`textes.rs:164-167`), une ligne vide, le JSON indenté de la même
     structure (`sortie.rs:423-439`). Les **données** passent ; l'**en-tête** non — dont la seule explication de la valeur
     masquée (« (masqué : portée contacts:read requise) » est retenue par Kesh).
3. **La piste de l'issue (« non observé ici ») est fondée** : `DonneesEnvoyables::en_resultat` joint la structure dès que
   `Donnees.structure` est `Some`, ce qui est **toujours** le cas en production (`sortie.rs:458`, seul appel de
   `Donnees::nouvelles` dans `src/`).
4. **Les erreurs** (`reponse.rs:77-79`, `:284-291`) ne portent jamais `structuredContent` : `content` seul.
5. **Le canal `instructions`** (`textes.rs:149-156`, servi par `initialize`, `repartiteur.rs:187-198`) porte la mise en
   garde `nonFiable` et la consigne de relecture. Qu'il atteigne l'agent est **plausible, non établi** : de l'accusé, le
   modèle n'a reçu que `{"envoi":1,"etat":"en_attente"}`, et il a pourtant expliqué l'attente et rappelé `kesh_lire_envoi`
   après l'accord (observation (d)) — mais la description statique de l'outil, servie par `tools/list`, dit aussi « À
   appeler quand l'utilisateur a traité l'envoi, non en boucle. » (`textes.rs:188-191`) et suffit à expliquer ce geste.
   Rien ne montre que la mise en garde `nonFiable` atteigne le modèle ; seule la recette de la release (point 3) le
   vérifiera. Ce qui est sûr : un canal statique a rendu le défaut de l'accusé invisible à la recette (C-v016-1-3).
6. ⚠️ **Prémisse non établie, à ne pas affirmer** : que Claude Code transmette `content` quand `structuredContent` est
   **absent**. C'est la règle du schéma (`CallToolResult.content` requis, `structuredContent` facultatif), mais la recette
   ne l'a pas vu : la capture (`kesh-gate-logs/3211/capture-recette.log`) ne contient **aucune** réponse sans
   `structuredContent` (aucun `"isError":true`, chaque résultat de `tools/call` porte la clé). D'où C-v016-1-7.

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
- `Decimal` n'est pas `ChampSur` (`sortie.rs:59-87`, la liste des implémentations) : **tout `Decimal`** d'une sortie MCP
  passe par `Montant` — inventaire clos par le typage (C-v016-1-6). Le seul autre nombre non entier est un **`f64`**,
  `ChampSur` (`sortie.rs:70`) : `delaiInstructionS` de `kesh_version` (`outils/kesh_version.rs:26`), qui sort `30.0` et
  n'est pas concerné par #812 ni par la règle de l'AC-6. `Montant` porte aussi des **quantités** et des **taux
  de TVA** (`factures.rs:236-238`, `factures_fournisseur.rs:88-90`) : une échelle fixe à deux décimales perdrait des
  quantités réelles.
- Les tests MCP comparent les montants **par valeur** (`mcp_enveloppe_e2e.rs:1148`, `montant(v) -> Decimal` ; les
  montants de `mcp_outils_e2e.rs:826-838` sont des entrées de fixture, non des comparaisons) ; aucun littéral de montant n'y est épinglé sous forme de chaîne de sortie (`grep -nE
  '"-?[0-9]+\.[0-9]{2,4}"' tests/mcp_*.rs` ne rend qu'un argument d'API, `mcp_enveloppe_e2e.rs:1224`).

## Critères d'acceptation

### #811 — un seul canal

**AC-1 — La règle.** Aucun `result` de `tools/call` ne porte la clé `structuredContent` ; ses clés sont exactement
`content` et `isError`. Les cinq sites qui la posent (cinq lignes de code, dans trois fonctions `en_resultat` et les deux
variantes `Depose`/`EnAttente`) la retirent :

| Site | Fichier | Avant | Après |
|---|---|---|---|
| S1 — sortie d'outil, chemin direct | `politique.rs:157-172` | `content` + `structuredContent` + `isError` | `content` (un bloc, inchangé) + `isError` |
| S2 — accusé de dépôt | `reponse.rs:49-56` | idem | un bloc, inchangé |
| S3 — relecture « en attente » | `reponse.rs:57-61` | idem | un bloc, inchangé |
| S4 — relecture validée | `reponse.rs:62-76` | idem | deux blocs, inchangés |
| S5 — accusé de proposition | `prompts.rs:207-216` | idem | un bloc, inchangé |
| S6 à S8 — relecture refusée, échue, inconnue | `reponse.rs:77-79` | `content` + `isError: true` | inchangé |
| S9 — erreur d'outil (`OutilEnErreur`) | `reponse.rs:284-291` | `content` + `isError: true` | inchangé |

**Aucun texte ne change** — ni les blocs de S1 à S9, ni `INSTRUCTIONS`, ni les descriptions d'outil (C-v016-1-3) : le
test `textes_envoi_epingles_par_un_litteral` (`textes.rs:342-375`) et le littéral K1 (`mcp_contrat.rs:76-84`) restent
verts sans retouche. Les doc-comments qui décrivent l'émission (`politique.rs:14-16`
(« la donnée structurée n'est ni déposée, ni montrée, ni envoyée »), `:45-46`, `:116` et `:159-160`, `prompts.rs:204-206`,
`sortie.rs:4-9` et `:441-442`, `mcp/mod.rs` s'il en parle) sont réécrits —
`grep -rn "structur" crates/kesh-api/src/mcp/` relu en entier, chaque occurrence restante justifiée au Dev Agent Record.

**AC-2 — `Donnees` à un seul champ de données** (C-v016-1-5). `Donnees::nouvelles(texte: String)` ; le champ
`structure` disparaît (`politique.rs:36-52`, le champ à `:39`, `nouvelles` à `:47`) ; `sortie::donnees` passe le seul
texte (`sortie.rs:458`) ; `texte_et_valeur` garde sa signature et son test. Le champ public `Fixe.structure` des outils de
test (`common/mcp.rs:557-560`, construit en littéral à `:574` et à cinq endroits de `mcp_envois_e2e.rs` : l. 194, 256,
263, 889, 1129) disparaît avec lui.

**Le sort de `une_seule_representation`** (`mcp_envois_e2e.rs:185-222`, AC-B2 de la 32-8 : « la donnée structurée n'est
ni déposée ni envoyée ») : il pose `PIEGE_STRUCT` dans la structure pour prouver qu'elle ne part pas. Après ce correctif,
le cas est **inatteignable par construction** — il ne doit ni rester vert à vide (la boucle `for piege in [PIEGE_STRUCT,
PIEGE_RESUME]` passerait pour `PIEGE_STRUCT` sans plus rien prouver), ni disparaître en silence. Retirer `PIEGE_STRUCT`,
sa constante et la ligne `structure` ; garder la preuve sur le résumé (`PIEGE_RESUME`) ; annoter le test au doc-comment :
« l'AC-B2 de la 32-8, pour la donnée structurée, est tenu par construction depuis la 0.16.1 (C-v016-1-5) : `Donnees` n'a
plus de champ structuré » ; le dire au Dev Agent Record. Les gardes de `tests/mcp_gardes_lexicales.rs` qui citent la forme
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
`tests/mcp_gardes_lexicales.rs` : `sites(&sources_src(), tout, &re(r"(?i)structured_?content"))` est **vide** — aucune
ligne de code de `crates/kesh-api/src/` (hors lignes de commentaire, `est_commentaire`) ne porte la clé, ni sous sa forme
JSON, ni sous la forme d'un champ Rust `structured_content` qu'un `#[serde(rename_all = "camelCase")]` sérialiserait en
`structuredContent` (forme probable d'un retour par un SDK tel que `rmcp`) ; trois témoins factices (`sites_factices`)
prouvent que le motif voit `"structuredContent": s` et `structured_content: Some(v)`, et ignore `/// structuredContent`.
Ce que le motif ne voit pas reste dit au point (1) ci-dessous. Son message
nomme #811 et C-v016-1-1. **Angles morts écrits** au doc-comment de la garde : (1) une clé construite (`format!`,
concaténation) lui échappe ; K23 et les attendus par égalité la rattrapent sur chaque site existant ; (2) à l'inverse,
`est_commentaire` n'exclut que les lignes qui commencent par `//` : un commentaire de bloc `/* … */`, un `#[doc = "…"]`,
un commentaire en fin de ligne ou un test de `src/` (`mod tests`) qui écrirait la chaîne en code la feraient rougir **à
tort** — consigne au même doc-comment : ne pas tester l'absence de la clé par un littéral dans `src/`, le faire dans
`tests/`.

**AC-5 — Les tests existants suivent, sans perdre ce qu'ils prouvent.** § *Sites de test à ajuster*. En particulier :
- l'aide `mcp_pieges::structure` (`common/mcp_pieges.rs:121-129`) **dérive** la valeur du texte : unique bloc,
  `split_once("\n\n")`, `serde_json::from_str` de la suite ; elle panique si le texte ne commence pas par l'en-tête
  littéral « Données lues dans les livres de Kesh, en lecture seule. » — les trente et un appels de
  `mcp_enveloppe_e2e.rs` et `mcp_outils_e2e.rs` gardent leur sens. Les deux qui comparent le texte à sa propre
  re-sérialisation (`mcp_enveloppe_e2e.rs:636-638`, `mcp_outils_e2e.rs:673-679`) **restent telles quelles** : `serde_json`
  est sans `preserve_order` (aucune dépendance à `indexmap` dans son entrée du `Cargo.lock`), donc `from_str` puis
  `to_string_pretty` rend la forme **canonique** — clés triées, indentation de deux espaces ; l'égalité épingle, outil par
  outil, que le texte émis est exactement cette forme (un texte compact, ou indenté autrement, la ferait rougir). Leur
  message « une seule source » se réécrit en « forme canonique indentée » (la seconde source n'existe plus). *(Rectifié à
  la validation P2 : la P1 les avait déclarées tautologiques et prescrivait de les affaiblir.)* ;
- `numero_depose` (`common/mcp.rs:614-623`) lit le numéro au préfixe « Envoi n° » du texte (C-v016-1-9) ;
- `mcp_outils_e2e.rs:683-686` : `len() == 2`, message « content, isError » ;
- K8 (`mcp_contrat.rs:600-633`) : la valeur comparée est celle du texte (le JSON après l'en-tête), non plus celle du champ
  retiré ; K10 (`:676-683`, `:747-752`) : les assertions sur le champ retiré deviennent « clés exactes `content`,
  `isError` » ;
- **la spec Playwright `frontend/tests/e2e/mcp-sends.spec.ts`** — seul consommateur de `structuredContent` hors de
  `crates/`, et seul test qui traverse la frontière HTTP avec un vrai `APIRequestContext` pour le parcours dépôt → décision
  à l'écran → relecture : `deposer` (l. 102-108) lit le numéro au préfixe « Envoi n° » du texte de l'accusé (comme
  `numero_depose`) ; les assertions `toEqual` sur `structuredContent` (l. 152, 164, 202) et `toBeUndefined` (l. 175)
  deviennent, pour chaque relecture, `expect(Object.keys(r).sort()).toEqual(['content', 'isError'])`, plus l'état lu au
  texte du premier bloc (littéral écrit dans la spec) ; les assertions sur `content[1]` restent. Elle devient le **témoin
  E2E de l'invariant I**. Sans cette retouche, toute la spec rougit en `TypeError` au gate E2E.

### #812 — une seule forme de montant

**AC-6 — La règle** (C-v016-1-6). `Montant` sérialise `self.0.normalize()` porté, si son échelle est inférieure à 2, à
l'échelle 2 (`rescale(2)`) ; le doc-comment (`sortie.rs:195-196`) dit la règle, ses exemples et pourquoi aucune
décimale ne se perd. Aucun autre site ne change (inventaire clos par le typage, registre C-v016-1-6).

**AC-7 — Les tests.**
- Unitaire `montant_forme_unique` dans `sortie.rs` (`mod tests`), sur la valeur JSON sérialisée : `Decimal::new(0, 4)` →
  `"0.00"` ; `Decimal::ZERO` → `"0.00"` ; un zéro négatif construit par **`-Decimal::new(0, 4)`** (le `Neg` pose le bit
  de signe sur un zéro ; **pas** `Decimal::from_parts(0, 0, 0, true, 4)`, qui efface le signe d'un zéro,
  `rust_decimal-1.43.0/src/decimal.rs:540-553`, et ferait passer le test à vide), précédé du témoin
  `assert!(z.is_sign_negative())`, → `"0.00"` ; `Decimal::new(1_234_500, 4)` → `"123.45"` ;
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
| `sec:mcp-client`, l. 3314 | « La configuration de ce client n'est pas éprouvée contre un client réel… » | écrire **ce que la recette établit, et seulement cela** : le 2026-10-28, Claude Code 2.1.312 s'est branché en transport HTTP, portée `local`, la clé envoyée dans l'en-tête `Authorization: Bearer …` (vu à la capture ; la valeur n'est citée dans aucun texte du dépôt), sur Kesh 0.16.0 derrière Traefik en HTTP. La recette ne rapporte **pas** la ligne de commande employée : ne pas écrire que telle commande « a été employée ». Restent « d'après sa documentation » : la portée `user`, l'expansion `${…}`, l'écriture de la clé **en clair** dans `~/.claude.json`, la limite de 25 000 jetons |
| `sec:mcp-client`, même l. 3314, phrase suivante | « … non Claude Code lui-même ; ses gestes y restent marqués « non éprouvé » » | devient fausse : « … ; la recette du 2026-10-28 a éprouvé une partie de ses gestes, les autres restent marqués « non éprouvé » » |
| `sec:mcp-brancher`, intro l. 3331 | parcours joué en client HTTP ; « ce que fait Claude Code … est marqué « non éprouvé », geste par geste » | **garder** ce que le parcours du 27 a établi (ce que le serveur répond) ; **réécrire** la phrase « est marqué « non éprouvé », geste par geste », que l'AC-8 rend fausse : « ce que fait Claude Code a été observé, en partie, à la recette du 2026-10-28 (Claude Code 2.1.312, base neuve, aucune donnée réelle) ; chaque geste dit s'il est observé ou non éprouvé » |
| item 3, l. 3336 | `/mcp` « (non éprouvé) » | (a), (e), (h) observés : `initialize` en `2025-11-25` après une sonde `2026-07-28` refusée en `400` texte, puis `tools/list` et `prompts/list`, sans session. **Ce que montre `/mcp` reste « non éprouvé »** (la recette ne le rapporte pas) |
| item 4, l. 3337 | `/kesh:kesh-assistant (MCP)` « (non éprouvé) » | **corrigé selon (g)** : (g) rapporte au menu `kesh:kesh-assistant` et, comme commande envoyée, `/mcp__kesh-direct__kesh-assistant` — la forme `/kesh:kesh-assistant (MCP)` du manuel n'est pas celle observée (le commentaire ne dit rien d'un suffixe, il ne faut donc pas affirmer son absence). Phrase : « Dans Claude Code, le menu le montre sous la forme \texttt{<serveur>:kesh-assistant} (par exemple \texttt{kesh:kesh-assistant}), où \texttt{<serveur>} est le nom donné au serveur ; la commande envoyée est \texttt{/mcp\_\_<serveur>\_\_kesh-assistant}, et Kesh reçoit \texttt{prompts/get} sans argument *(observé avec Claude Code 2.1.312)*. » — les deux formes concrètes de (g) généralisées par `<serveur>`, rien de plus |
| item 5, l. 3338 | « Ce que l'agent vous en dit (non éprouvé) » | (d), **avec sa réserve** : « l'agent explique l'attente et demande qu'on le prévienne *(observé avec Claude Code 2.1.312 sur la 0.16.0 ; ce client ne transmettant alors au modèle que le contenu structuré quand il y en a un — constat de l'issue \#811 —, l'agent ne recevait vraisemblablement de l'accusé que son état ; ce que le texte de l'accusé produit est à confirmer sur la 0.16.1)* » |
| item 7, l. 3340 | « Combien de fois un agent relit … (non éprouvé) » | (d), même réserve : une relecture par envoi, après l'accord, sans boucle ; une de plus sur demande explicite *(observé sur la 0.16.0, où la consigne « ne le rappelez pas en boucle » de l'accusé n'atteignait vraisemblablement pas le modèle, pour la même raison)*. « S'il relit avant votre décision … (non éprouvé au parcours : décrit d'après le code) » **reste**. La réception du texte validé : « *(à confirmer avec Claude Code sur la 0.16.1 ; en 0.16.0, ce contenu ne l'atteignait pas, issue \#811)* » — levée par la recette de la release (point 3), avant le tag |
| item 10, l. 3343 | 401 et OAuth « (non éprouvé) » | **reste** — (f) non observé |
| `sec:mcp-origin`, l. 3167 | « Un client qui n'envoie pas d'Origin n'est pas concerné. » | ajouter : Claude Code 2.1.312 n'en envoie pas (b) |
| `sec:mcp-outils`, « Ce que l'agent reçoit », l. 3209 | « Chaque réponse d'un outil commence par un en-tête fixe… » | ajouter : depuis la 0.16.1, la réponse ne porte **que** ce texte, aucun contenu structuré (#811) ; et la forme des **montants, quantités et taux** (« au moins deux décimales, sans zéro final superflu au-delà » — exemples `0.00`, `123.45`, `0.0125` ; une quantité `2` sort `2.00`, un taux `7.7` sort `7.70`) ; ne pas écrire « tout nombre décimal » : le délai `delaiInstructionS` de `kesh_version` est un nombre à virgule flottante, qui sort `30.0` |
| `sec:mcp-envois`, 3e puce du détail, l. 3227 | « La donnée structurée que l'outil produit aussi n'est pas envoyée… » | retirer la phrase (il n'y a plus de donnée structurée), et réécrire la phrase qui la précède, « C'est ce texte, et lui seul, que l'agent relira », que l'ajout contredirait : « C'est ce texte que l'agent relira, précédé d'une ligne de Kesh qui annonce l'envoi validé. » — la puce décrit l'écran d'un envoi, donc d'une clé requise : pas de « sous toute clé » ici (la règle générale est dite à `sec:mcp-outils`) |
| nouveau `\subsubsection{Passer à la 0.16.1}` avant `sec:maj-0-16-0` (l. 2480) | — | comme les sections sœurs (l. 2482) : on tire l'image et on redémarre ; « Depuis une version antérieure à la 0.16.0, appliquez d'abord la section suivante (\S\ref{sec:maj-0-16-0}) » ; le rappel de sauvegarde d'avant mise à jour (l. 2485, en plus court) ; aucune migration ; la 0.16.0 redémarre sur une base passée par la 0.16.1 ; un `.keshbackup` de l'une s'importe dans l'autre (mêmes tables) ; aucun réglage ; un client MCP n'a rien à changer — sauf une intégration qui lisait `structuredContent` : pour une sortie d'outil, elle lit le JSON du bloc de texte, après l'en-tête et une ligne vide ; pour un accusé ou une relecture d'envoi, le numéro et l'état ne sont plus en JSON, ils se lisent dans la phrase de Kesh — chaque cas a la sienne (« Envoi n° N en attente de validation… » à l'accusé, « L'envoi n° N attend encore… » à la relecture en attente, « Envoi n° N validé… » à la relecture validée, dont le texte validé est le second bloc ; refus, échéance et numéro inconnu ont leur propre phrase, `textes.rs`) ; pour l'accusé d'une proposition de prompt, son nom et son état sont dans la phrase. **Chaque affirmation vérifiée**, ligne lue citée au Dev Agent Record : pas de migration (`git diff --stat main -- crates/kesh-db/migrations` vide) ; la 0.16.0 démarre sur la base (appel `crates/kesh-api/src/main.rs:122`, comparaison au seul `kesh_version_min_required` dans `crates/kesh-db/src/version.rs:222-230`, inchangé à `0.14.0`) ; l'import croisé (`routes/admin.rs:217`, même comparaison ; `kesh_version` du manifeste informative, l. 247 et 582) |

**Manuel utilisateur FR** (`docs/manual/fr/user-manual.tex`, § *L'agent IA et le serveur MCP*, l. 3242) : le paragraphe
*Invoquer un prompt* (l. 3268-3273) porte la **même** forme contredite (`/kesh:kesh-assistant (MCP)`, « non éprouvé ») ;
même correction que l'item 4 ci-dessus, en termes d'utilisateur, avec « observé avec Claude Code 2.1.312 ». Le reste de la
section se relit contre l'AC-8 (`grep -nE "éprouv|structur|kesh-assistant" user-manual.tex`), chaque occurrence restante
justifiée au Dev Agent Record.

Le PDF n'est **pas** régénéré par le développeur (release, point 5). Pour vérifier que le `.tex` compile, `make fr` sous
verrou est permis, mais il **réécrit les PDF versionnés** : les restaurer aussitôt (`git checkout -- docs/manual/fr/*.pdf`)
et constater, avant chaque commit, que `git status --porcelain` ne porte aucun `.pdf`.

**AC-9 — `CHANGELOG.md`** : une section `## [0.16.1] — Non publié` (forme que `scripts/prepare-release.sh:237` attend)
au-dessus de `[0.16.0]`, avec une phrase d'introduction (correctif de l'Epic 32, aucune migration, retour à la 0.16.0 sans
sauvegarde particulière), un paragraphe « observé avec Claude Code 2.1.312 » qui dit ce que la recette a établi, puis,
dans l'ordre des sections voisines (*Modifié* avant *Corrigé*) :
- **Modifié** : plus de `structuredContent` dans les réponses d'outil — une intégration qui le lisait lit le JSON du bloc
  de texte (sortie d'outil, après l'en-tête et une ligne vide), ou la phrase de Kesh, propre à chaque accusé et à chaque relecture (texte
  validé au second bloc), comme au § *Passer à la 0.16.1* ;
- **Corrigé** : #811, **formulé pour être vrai avant la recette de la release** : « Kesh ne rend plus qu'un seul canal,
  le texte, sur tout appel d'outil : le contenu validé, l'en-tête des données et les consignes que Kesh écrit pour
  l'agent y sont tous. Que Claude Code transmette ce texte au modèle en l'absence de contenu structuré reste à confirmer
  par la recette de la release. » — l'orchestrateur réécrit la dernière phrase au point 3 de la release, dans un sens ou
  dans l'autre ; #812 (une seule forme pour les montants, quantités et taux).
Pas de lien `closes` dans le texte ; des liens d'issue comme les sections voisines.

**AC-10 — `README.md`, l. 49** : « Le branchement avec Claude Code est décrit au manuel d'administration, non encore
éprouvé par une recette » devient vrai : éprouvé avec Claude Code 2.1.312 le 2026-10-28 ; la relecture d'un envoi validé,
qui n'atteignait pas l'agent en 0.16.0 (#811), est corrigée côté Kesh en 0.16.1 — même réserve que le CHANGELOG, levée au
point 3 de la release. Le tableau des versions (l. 234) et le site relèvent de la release (point 6).

**AC-11 — `sprint-status.yaml`** : la clé `v016-1-mcp-sorties` passe à `in-progress` puis `review` selon le flux, avec
commentaire daté.

## Sites de test à ajuster (inventaire relevé, `grep -rIn "structuredContent" crates/ frontend/ --exclude-dir=node_modules`, puis `grep -rn "structure" crates/kesh-api/tests`)

| Fichier | Lignes | Geste |
|---|---|---|
| `tests/mcp_contrat.rs` | 600-633 (K8), 676-683 et 747-752 (K10) | AC-5 ; ajout de K23 |
| `tests/common/mcp.rs` | 380, 409, 463, 496, 599 (`Donnees::nouvelles`) ; 557-560 et 574 (`Fixe.structure`) ; 614-623 (`numero_depose`) ; 626-648 (`attendu_depose`, `attendu_en_attente`, `attendu_valide`) | signature ; champ retiré ; numéro au texte ; attendus sans la clé |
| `tests/mcp_envois_e2e.rs` | 40 (`PIEGE_STRUCT`), 185-222 (`une_seule_representation`), 194, 256, 263, 889, 1129 (`structure:`) | AC-2, § *Le sort de `une_seule_representation`* |
| `tests/mcp_enveloppe_e2e.rs` | 636-638 | AC-5 : assertion gardée (forme canonique indentée), message réécrit |
| `frontend/tests/e2e/mcp-sends.spec.ts` | 102-108 (`deposer`), 152, 164, 175, 202 | AC-5 : numéro au texte ; clés exactes ; témoin E2E de I |
| `tests/common/mcp_pieges.rs` | 121-129 | `structure` dérivée du texte |
| `tests/mcp_transport_e2e.rs` | 105-123, 1545-1556 | attendus sans la clé ; l'outil `Compteur` rend le texte `{"n":1}` |
| `tests/mcp_journal_e2e.rs` | 813 | lire `{"n": 1}` dans le texte |
| `tests/mcp_outils_e2e.rs` | 673-679 ; 683-686 | AC-5 : assertion gardée (forme canonique indentée) ; `len() == 2` |
| `tests/mcp_prompts_e2e.rs` | 675-682 | attendu sans la clé |
| `tests/mcp_gardes_lexicales.rs` | 611-617, 638 | témoins et ligne attendue ; garde AC-4 |

Ce relevé n'est pas une preuve d'exhaustivité : le développeur relance le `grep` après ses patches et lit chaque
occurrence restante (code, doc-comments des tests, fiche) ; un test qui comparait un `result` entier sans nommer la clé
rougit au gate, c'est le filet.

## Tâches

1. **Étape 0** — § *À recontrôler* ; constater rouges, **avant tout patch** : K23 (rouge **au premier site** qu'il
   parcourt, S1 — un cas s'arrête à sa première assertion fausse ; le rouge propre de S2, S3 et S4 se constate après le
   correctif, par les mutations M2, M11 et M1 ; S5 est couvert par `mcp_prompts_e2e.rs`, rouge après retrait de la clé de
   son attendu), la garde AC-4 (rouge, cinq lignes), `bilan_montants_une_seule_forme` (rouge, `"0.0000"`), `montant_forme_unique` (rouge).
2. #811 : AC-1, AC-2 ; puis AC-5 ; gate ciblé.
3. #812 : AC-6, AC-7 ; gate ciblé.
4. Textes : AC-8 à AC-10 ; greper la **valeur** des symptômes corrigés (`structuredContent`, `structurée`,
   `non éprouvé`, `0.0000`) sur `docs/`, `README.md`, `CHANGELOG.md`, `website/`, `crates/`, `frontend/` — liste au Dev
   Agent Record. **Annoter, sans les réécrire**, les relevés que le correctif rend faux — une annotation datée « *(dépassé
   le 2026-10-28 par la v016-1, C-v016-1-1 : `content` seul depuis la 0.16.1)* » à chacun. Ne pas partir d'une liste :
   partir du symptôme, `grep -rn "structuredContent\|donnée structurée" _bmad-output/planning-artifacts _bmad-output/implementation-artifacts`
   hors fichiers `v016-1-*`, et trier chaque occurrence (décision qui devient fausse → annotée ; récit d'une passe ou
   prompt de revue daté → laissé, c'est un relevé de sa date). Au moins, d'après la validation P2 : C-32-9-6
   (`epic-32-choix-autonomes-329.md`), E1 de la spike (`epic-32-spike-mcp.md` § 5.3.1), AC-B2, AC-B5 et AC-B6 de
   `32-8-validation-avant-envoi.md`, C-32-10a-17, la décision de K8 de la 32-11 (`epic-32-choix-autonomes-3211.md`), et
   les registres `epic-32-choix-autonomes-326.md` et `-328.md` ; la liste triée va au Dev Agent Record.
5. Mutations (§ *Tests et mutations*), sous verrou, restauration `git checkout -- <f> && touch <f>`.
   Les tests de **taille** (`tailles_mesurees`, `mcp_outils_e2e.rs:1342` ; `plafond_au_maximum`, `:1385` ; les `limit`
   mesurés du même fichier) se relisent : un montant `0.0000` devenu `0.00`, ou `0` devenu `0.00`, change la longueur du
   texte ; un écart dû à cette seule cause se recale, et se dit au Dev Agent Record avec l'ancienne et la nouvelle valeur.
6. Gate complet backend, gate frontend (une spec E2E touchée, `mcp-sends.spec.ts` ; aucun fichier de `frontend/src/` :
   `npm run check` et `lint` la couvrent, Vitest non), **E2E Playwright complet** au dernier commit de code (recette
   `docs/testing.md`, port et bases attribués par l'orchestrateur), `mcp-sends.spec.ts` verte et nommée au bilan, échecs
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
| M7 | `Montant` sans `normalize` : l'échelle portée à 2 **seulement si elle est inférieure** (`if d.scale() < 2 { d.rescale(2) }`, sans `normalize` ; un `rescale(2)` inconditionnel arrondirait et ne rendrait jamais `123.4500`) | `montant_forme_unique` (`123.4500`), `bilan_montants_une_seule_forme` (`"0.0000"`) |
| M8 | `Montant` sans `rescale(2)` | `montant_forme_unique` (`"0"`), e2e bilan (`"0"`) |
| M9 | `Montant` en `round_dp(2)` | `montant_forme_unique` (`0.0125`) |
| M10 | `structure` de `mcp_pieges` qui rend `Value::Null` | les tests d'enveloppe (preuve que l'aide dérivée est lue) |
| M11 | remettre `"structuredContent": {…}` à `EnAttente` (S3) | K23 (S3), garde, `attendu_en_attente` |
| M12 | ajouter `"structuredContent": {}` au `json!` d'`OutilEnErreur` (S9) | K23 (S9), garde |
| M13 | retirer l'en-tête du texte des données, là où il est construit (`sortie.rs:434`, `format!("{TEXTE_ENTETE_DONNEES}\n\n{joli}")` dans `texte_et_valeur`) | `structure` de `mcp_pieges` (panique sur l'en-tête absent), donc les tests d'enveloppe et d'outils ; `mcp_enveloppe_e2e.rs:638` et `mcp_outils_e2e.rs:673-679` ; K8 et K23 (S1) s'ils lisent l'en-tête — l'ensemble rouge constaté et nommé au Dev Agent Record |
| M15 | ajouter `"structuredContent": {}` au `json!` de `un_bloc` (`reponse.rs:41-46`, sert S6 à S8) | K23 (S6, S8), garde, `echeance_ne_transmet_rien` (S7) |
| M14 | rétablir `structuredContent` à la relecture validée **et** lancer `mcp-sends.spec.ts` seule | la spec (clés exactes) — preuve que le témoin E2E voit la clé |

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
- La forme des montants de l'API REST (même `Decimal` tel quel, probablement même défaut) : hors MCP, non traité,
  **ouvert en #813** (P4) par l'orchestrateur le 2026-10-28. Le développeur n'y touche pas.

## Ce que la release doit contenir (orchestrateur ; ne pas prescrire au développeur)

L'ordre compte : la recette (point 3) se joue **avant** que les textes ne soient figés et que le tag ne déclenche
`release.yml` (qui ne construit l'image qu'au `push` du tag) — sinon lever une réserve du manuel exigerait une 0.16.2.

1. `scripts/prepare-release.sh 0.16.1` : dix crates à 0.16.1, CHANGELOG daté, rappels lus ; aucune migration, donc
   `kesh_version_min_required` inchangé et aucune exemption du registre de rejeu touchée (à constater).
2. Le paragraphe introductif du CHANGELOG relu après datation ; la ligne « observé avec Claude Code 2.1.312 ». La date posée par le script se **recontrôle au point 8** : si le tag tombe un autre jour (la recette du point 3 peut le repousser), redater l’entrée et `\keshReleaseDate`.
3. **Recette courte avec Claude Code, avant le tag**, contre le **binaire de la branche de release lancé en local**
   (`cargo run -p kesh-api`, base neuve, aucune donnée réelle), par le Project Lead guidé par l'orchestrateur, capture
   par relais comme le 2026-10-28, **version de Claude Code relevée** (`claude --version`) et écrite au CHANGELOG et au
   manuel si elle diffère de 2.1.312. Deux critères **décidables**, les seuls qui lèvent la réserve et ferment #811 :
   - **C1, clé requise** : un outil, puis, à l'écran, une validation **modifiée** dont le texte porte une ligne-marqueur
     inventée pour la recette (par exemple `MARQUEUR-0161-<quatre chiffres>`) ; après la relecture, on demande au modèle
     de recopier la ligne-marqueur. Réussi si sa réponse la reproduit exactement (le marqueur n'existe nulle part
     ailleurs que dans le texte validé).
   - **C2, clé dispensée** (le chemin direct, le seul que la recette du 2026-10-28 a vu marcher, et que le correctif fait
     passer de `structuredContent` à `content` seul) : sur la base neuve, une écriture de démonstration d'un montant
     **inventé et distinctif** (par exemple `4321.09`, aucune donnée réelle), puis `kesh_bilan` ; réussi si le modèle rend
     ce montant, sous la forme `4321.09`. (Un bilan tout à zéro ne discrimine pas : la recette du 2026-10-28 rendait déjà
     `"0"`.)
   - **C3, l'accusé** (clé requise, juste après le dépôt de C1, avant la validation) : on demande au modèle quel chemin de
     Kesh l'accusé lui indique ; réussi s'il rend `/admin/mcp/sends/<n>` — ce chemin n'est écrit que dans le texte de
     l'accusé (`textes.rs:199-204`), ni dans `INSTRUCTIONS` ni dans la description de l'outil.
   **Observés sans être critères** (ils ne se décident pas sur une phrase du modèle) : combien de fois il relit, ce qu'il
   dit de l'attente, ce qu'il fait de la mise en garde de l'en-tête ; ils vont au commentaire de recette, comme le
   2026-10-28, et ne lèvent rien.
   Chaque réserve se lève sur **son** critère : celle de l'item 7 (réception du texte validé) sur C1 ; celle de l'item 5
   (ce que le texte de l'accusé produit : réduite à « l'agent reçoit le texte de l'accusé ») sur C3 ; la dernière phrase
   de l'entrée *Corrigé* du CHANGELOG et la réserve du README (AC-10) sur C1 **et** C2. Le comportement observé de (d)
   — expliquer l'attente, relire une fois — garde sa mention « observé sur la 0.16.0 » tant que la recette ne le revoit
   pas : un critère réussi ne vaut pas observation de ce comportement. Si l'un échoue : #811 reste
   ouverte, le manuel, le CHANGELOG **et le README** gardent la réserve, et l'opportunité de publier se pose au Project
   Lead (C2 en échec serait une régression par rapport à la 0.16.0).
4. `docs/manual/shared/kesh-style.sty` : `\keshVersion{0.16.1}`, `\keshReleaseDate{…}`, `\keshTargetRelease{v0.16}`
   (inchangé) ; URL `raw.githubusercontent.com/guycorbaz/kesh/v0.16.1/…` des manuels, du README et de `website/` (garde
   G19, `textes_coherents.rs`).
5. `make fr` : les trois PDF régénérés et commités ; PDF contrôlé aplati (`pdftotext … | tr '\n' ' '`) sur les
   phrases de l'AC-8, manuel utilisateur compris.
6. `README.md` : ligne du tableau des versions (v0.16.1, l. 234). `website/index.html:70` écrit « v0.16.0 released » :
   le porter à 0.16.1 ; `website/roadmap.html:372` nomme la version où l'Epic 32 a été livré (v0.16.0) : vrai, inchangé.
   Constats : `website/issues.html` ne liste ni #811 ni #812 ; le paragraphe *Current release* de `CLAUDE.md` passe à
   v0.16.1.
7. Décompte des issues ouvertes au tag (objectif : moins de 185, le compte au tag v0.16.0) et des bugs. ⚠️ Le
   2026-10-28 au soir, **189** issues sont ouvertes (185 au tag v0.16.0, plus #809, #811, #812 et #813, ouvertes depuis) :
   même avec #811 et #812 fermées, le tag en portera **187** au moins. L'objectif sera manqué ; c'est un objectif, non
   une barrière (règle de release du 2026-10-24) — l'écart et sa cause s'écrivent à la PR de release, et aucune issue ne se
   ferme pour le chiffre.
8. PR de release, puis tag `v0.16.1` ; Docker : image `0.16.1` = `:latest`, smoke `/health` (version `0.16.1`).
9. La PR de story porte `refs #811`, `refs #812` (jamais de mot-clé de fermeture, ni dans ses commits ni dans son corps).
10. La **PR de release** porte `closes #812` ; `closes #811` **seulement** si le point 3 a rendu C1 et C2 réussis,
    `refs #811` sinon.

## Dev Notes

### Fichiers touchés (prévus) et modules

- `crates/kesh-api/src/mcp/` : `reponse.rs`, `politique.rs`, `prompts.rs`, `sortie.rs` (un module).
- `crates/kesh-api/tests/` : `mcp_contrat.rs`, `mcp_gardes_lexicales.rs`, `common/mcp.rs`, `common/mcp_pieges.rs`,
  `mcp_transport_e2e.rs`, `mcp_journal_e2e.rs`, `mcp_outils_e2e.rs`, `mcp_prompts_e2e.rs` (un module).
- `crates/kesh-api/tests/mcp_envois_e2e.rs`, `mcp_enveloppe_e2e.rs` (même module que les tests ci-dessus).
- `frontend/tests/e2e/mcp-sends.spec.ts` (un module).
- `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/user-manual.tex` (un module).
- `CHANGELOG.md`, `README.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml`, et les annotations datées des
  relevés de l'Epic 32 que la tâche 4 trie (au moins `epic-32-choix-autonomes-326.md`, `-328.md`, `-329.md`, `-3211.md`,
  `32-8-validation-avant-envoi.md`, le relevé de C-32-10a-17, `epic-32-spike-mcp.md` ; liste exacte au Dev Agent Record) —
  des textes de planification, rangés avec `sprint-status.yaml` : ils n'ajoutent pas de module.

Cinq modules : au seuil, non au-delà — pas de découpage. Aucune migration, aucun fichier de `frontend/src/`, aucune locale.

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
6. Que le motif `(?i)structured_?content` de la garde AC-4 ne touche, avant correctif, que les cinq lignes de code
   attendues (aucune occurrence en code de `src/` sous une autre casse) — sinon le dire et ajuster le motif.
7. Que `serde_json` reste sans `preserve_order` (`Cargo.lock`, ses dépendances ; aucun `features = ["preserve_order"]`
   dans le workspace) : c'est ce qui fait épingler la forme canonique aux deux assertions gardées de l'AC-5.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-10-28 — fiche créée (Opus 5.5, autonomie) ; registre C-v016-1-1 à C-v016-1-10.
- 2026-10-28 — validation P1 (Sonnet, deux lentilles R et F, prompt `v016-1-validate-prompt-p1.md`, rapports hors dépôt
  `kesh-gate-logs/v0161-validate-p1-{R,F}.md`) : R 0/1/3/5, F 0/0/4/9 — 1 HIGH, 7 MEDIUM, 14 LOW. Prémisses des findings
  appliqués relues au code par l'orchestrateur avant remédiation (`mcp-sends.spec.ts:105-202`, `user-manual.tex:3268-3273`,
  `politique.rs` 173 lignes, `rust_decimal` `from_parts`, `Cargo.toml:47`). Remédiés à la fiche et au registre : R1 (spec
  Playwright, témoin E2E de I, M14), R2 (`Fixe.structure`, sort de `une_seule_representation`), R3 (deux assertions
  devenues tautologiques), R4 (recette du chemin direct), F1 (manuel utilisateur), F2=R5 (numéros de `politique.rs`,
  doc-comments), F3 (réserve de (d) : observé sur la 0.16.0), F4 (CHANGELOG vrai dans les deux branches ; recette avant le
  tag, binaire local), et les LOW R6 (zéro négatif), R7=F6 (rédaction), R8 (angles morts de la garde, M11-M13), R9
  (annotations C-32-9-6 et E1), F5 (quantités et taux), F7 (pas de « commande employée »), F8 (`INSTRUCTIONS` : plausible,
  non établi), F9 (chaînage, lignes citées), F10 (constats de release), F11 (PDF restaurés), F12 (`refs` à la PR de story),
  F13 (défaut REST ouvert en **#813**, P4). Périmètre porté à cinq modules (au seuil). Aucun finding rejeté.
- 2026-10-28 — validation P2 (Opus, lentilles R et F, prompt `v016-1-validate-prompt-p2.md`, rapports hors dépôt
  `kesh-gate-logs/v0161-validate-p2-{R,F}.md`) : R 0/0/2/7, F 0/0/4/9 — 6 MEDIUM, 16 LOW ; six findings nés de la
  remédiation P1 (R1, R6, R7, R8, F4, F5). Prémisses relues au code avant remédiation (`serde_json` sans `preserve_order`
  au `Cargo.lock`, `sortie.rs:426-434`, `sortie.rs:70` et `kesh_version.rs:26`, `main.rs:122`, `version.rs:222-230`,
  `admin-manual.tex:3314` et `:3331`, ordre des rubriques du CHANGELOG). Remédiés : R1 (**la remédiation P1 de son R3 était
  fausse** : les deux assertions de re-sérialisation épinglent la forme canonique indentée et restent ; l'entrée P1
  ci-dessus est laissée telle quelle, relevé de sa date), R2 (motif de la garde `(?i)structured_?content`, trois
  témoins), F1 (deux phrases du manuel rendues fausses), F2=R3 (« tout `Decimal` », le `f64` de `kesh_version` exclu),
  F3 (migration d'une intégration : JSON pour S1, phrase de Kesh pour S2 à S5), F4 (critères décidables C1 et C2, seuls à
  lever la réserve, alignés au registre), et les LOW R4, R5=F11, R6, R7, R8 (annotations : partir du symptôme), R9, F5,
  F6, F7, F8, F9, F10, F12, F13. Points 6 et 7 ajoutés à *À recontrôler*. Aucun finding rejeté.
- 2026-10-28 — validation P3 (Sonnet, lentilles R et F, prompt `v016-1-validate-prompt-p3.md`, rapports hors dépôt
  `kesh-gate-logs/v0161-validate-p3-{R,F}.md`) : R 0/0/0/6, F 0/0/1/5 — 1 MEDIUM, 11 LOW (tendance 8 → 6 → 1 au-dessus
  de LOW). Prémisses relues : 189 issues ouvertes (`gh issue list`), `"totalActifs":"0"` dans la capture, textes des
  relectures et chemin de l'accusé (`textes.rs:196-230`), `un_bloc` (`reponse.rs:41-46`), tests de taille
  (`mcp_outils_e2e.rs:1342`, `:1385`). Remédiés : F1 (remédiation P2 : chaque réserve se lève sur son critère ; **C3**
  ajouté pour l'accusé, le chemin `/admin/mcp/sends/<n>` qu'il est seul à porter ; le comportement de (d) ne se lève sur
  aucun), F2 (la clé n'est pas « masquée » dans la capture : formulation retirée), F3 (C2 sur un montant inventé, un
  bilan nul ne discriminant pas), F4 (« et lui seul » du manuel), F5=R6 (chaque accusé et relecture a sa phrase), F6
  (objectif d'issues manqué, 187 au moins au tag, à écrire à la PR de release), R1 (fichiers annotés aux Dev Notes), R2
  (M15 sur `un_bloc`), R3 (M7 précisée), R4 (tests de taille), R5 (doc-comments de `reponse.rs` hors liste). Aucun
  finding rejeté. Hors dépôt, signalé au Project Lead : la capture de la recette porte la clé (révoquée) en clair, alors
  que le commentaire de recette de #723 dit « Bearer jamais cité ».
