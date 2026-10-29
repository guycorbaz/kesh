# Story v016-1 : correctif 0.16.1 — ce que le serveur MCP rend à l'agent

Status: review

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
   Rien ne montre que la mise en garde `nonFiable` atteigne le modèle ; seule la recette de la release (point 3) peut
   le montrer. Ce qui est sûr : un canal statique a rendu le défaut de l'accusé invisible à la recette (C-v016-1-3).
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
| item 7, l. 3340 | « Combien de fois un agent relit … (non éprouvé) » | (d), même réserve : une relecture par envoi, après l'accord, sans boucle ; une de plus sur demande explicite *(observé sur la 0.16.0, où la consigne « ne le rappelez pas en boucle » de l'accusé n'atteignait vraisemblablement pas le modèle, pour la même raison)*. « S'il relit avant votre décision … (non éprouvé au parcours : décrit d'après le code) » **reste**. La réception du texte validé : « *(à confirmer avec Claude Code sur la 0.16.1 ; en 0.16.0, ce contenu ne l'atteignait pas, issue \#811)* » — levée par la recette de la release (point 3, sur la candidate), avant le tag final *(annoté le 2026-10-29, revue de code P2, E6 = A2 : la règle générale de l'AC-8 — « observé avec Claude Code 2.1.312 » — prime sur cette cellule ; le manuel porte « observé avec Claude Code 2.1.312 sur la 0.16.0, où… »)* |
| item 10, l. 3343 | 401 et OAuth « (non éprouvé) » | **reste** — (f) non observé |
| `sec:mcp-origin`, l. 3167 | « Un client qui n'envoie pas d'Origin n'est pas concerné. » | ajouter : Claude Code 2.1.312 n'en envoie pas (b) |
| `sec:mcp-outils`, « Ce que l'agent reçoit », l. 3209 | « Chaque réponse d'un outil commence par un en-tête fixe… » | ajouter : depuis la 0.16.1, la réponse ne porte **que** ce texte, aucun contenu structuré (#811) ; et la forme des **montants, quantités et taux** (« au moins deux décimales, sans zéro final superflu au-delà » — exemples `0.00`, `123.45`, `0.0125` ; une quantité `2` sort `2.00`, un taux `7.7` sort `7.70`) ; ne pas écrire « tout nombre décimal » : le délai `delaiInstructionS` de `kesh_version` est un nombre à virgule flottante, qui sort `30.0` |
| `sec:mcp-envois`, 3e puce du détail, l. 3227 | « La donnée structurée que l'outil produit aussi n'est pas envoyée… » | retirer la phrase (il n'y a plus de donnée structurée), et réécrire la phrase qui la précède, « C'est ce texte, et lui seul, que l'agent relira », que l'ajout contredirait : « C'est ce texte que l'agent relira, précédé d'une ligne de Kesh qui annonce l'envoi validé. » — la puce décrit l'écran d'un envoi, donc d'une clé requise : pas de « sous toute clé » ici (la règle générale est dite à `sec:mcp-outils`) |
| nouveau `\subsubsection{Passer à la 0.16.1}` avant `sec:maj-0-16-0` (l. 2480) | — | comme les sections sœurs (l. 2482) : on tire l'image et on redémarre ; « Depuis une version antérieure à la 0.16.0, appliquez d'abord la section suivante (\S\ref{sec:maj-0-16-0}) » ; le rappel de sauvegarde d'avant mise à jour (l. 2485, en plus court) ; aucune migration ; la 0.16.0 redémarre sur une base passée par la 0.16.1 ; un `.keshbackup` de l'une s'importe dans l'autre (mêmes tables) ; aucun réglage ; un client MCP n'a rien à changer — sauf une intégration qui lisait `structuredContent` : pour une sortie d'outil, elle lit le JSON du bloc de texte, après l'en-tête et une ligne vide ; pour un accusé ou une relecture d'envoi, le numéro et l'état ne sont plus en JSON, ils se lisent dans la phrase de Kesh — chaque cas a la sienne (« Envoi n° N en attente de validation… » à l'accusé, « L'envoi n° N attend encore… » à la relecture en attente, « Envoi n° N validé… » à la relecture validée, dont le texte validé est le second bloc ; refus, échéance et numéro inconnu, qui n'en ont jamais porté, gardent leur phrase, `textes.rs`) ; pour l'accusé d'une proposition de prompt, son nom et son état sont dans la phrase. **Chaque affirmation vérifiée**, ligne lue citée au Dev Agent Record : pas de migration (`git diff --stat main -- crates/kesh-db/migrations` vide) ; la 0.16.0 démarre sur la base (appel `crates/kesh-api/src/main.rs:122`, comparaison au seul `kesh_version_min_required` dans `crates/kesh-db/src/version.rs:222-230`, inchangé à `0.14.0`) ; l'import croisé (`routes/admin.rs:217`, même comparaison ; `kesh_version` du manifeste informative, l. 247 et 582) |

**Manuel utilisateur FR** (`docs/manual/fr/user-manual.tex`, § *L'agent IA et le serveur MCP*, l. 3242) : le paragraphe
*Invoquer un prompt* (l. 3268-3273) porte la **même** forme contredite (`/kesh:kesh-assistant (MCP)`, « non éprouvé ») ;
même correction que l'item 4 ci-dessus, en termes d'utilisateur, avec « observé avec Claude Code 2.1.312 ». Le reste de la
section se relit contre l'AC-8 (`grep -nE "éprouv|structur|kesh-assistant" user-manual.tex`), chaque occurrence restante
justifiée au Dev Agent Record.

Le PDF n'est **pas** régénéré par le développeur (release, point 5). Pour vérifier que le `.tex` compile, `make fr` sous
verrou est permis, mais il **réécrit les PDF versionnés** : les restaurer aussitôt (`git checkout -- docs/manual/fr/*.pdf`)
et constater, avant chaque commit, que `git status --porcelain` ne porte aucun `.pdf`.
*(Annoté le 2026-10-29, revue de code P1, A4 : le développement a régénéré et commité `admin-manual.pdf` et
`user-manual.pdf` sur consigne postérieure de l'orchestrateur — C-v016-1-11 ; ce paragraphe, relevé de la validation,
reste tel quel.)*

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
   texte ; un écart dû à cette seule cause se recale **côté test** (la fixture de `fixture_tailles`), et se dit au Dev
   Agent Record avec l'ancienne et la nouvelle valeur. Ni une constante `LIMITE_*` de `src/mcp/outils/` ni un plafond
   (`13_312`, `23_552`) ne bouge sans être déclaré comme un changement de la story.
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
| M7 | `Montant` sans `normalize` : l'échelle portée à 2 **seulement si elle est inférieure** (`if d.scale() < 2 { d.rescale(2) }`, sans `normalize` ; un `rescale(2)` inconditionnel arrondirait et ne rendrait jamais `123.4500`) | `montant_forme_unique` (rouge au premier cas qui diffère, `Decimal::new(0, 4)` → `"0.0000"`), `bilan_montants_une_seule_forme` (`"0.0000"`) |
| M8 | `Montant` sans `rescale(2)` | `montant_forme_unique` (`"0"`), e2e bilan (`"0"`) |
| M9 | `Montant` en `round_dp(2)` | `montant_forme_unique` (rouge au premier cas qui diffère, `Decimal::ZERO` → `"0"`) |
| M10 | `structure` de `mcp_pieges` qui rend `Value::Null` | les tests d'enveloppe (preuve que l'aide dérivée est lue) |
| M11 | remettre `"structuredContent": {…}` à `EnAttente` (S3) | K23 (S3), garde, `attendu_en_attente` |
| M12 | ajouter `"structuredContent": {}` au `json!` d'`OutilEnErreur` (S9) | K23 (S9), garde |
| M13 | retirer l'en-tête du texte des données, là où il est construit (`sortie.rs:434`, `format!("{TEXTE_ENTETE_DONNEES}\n\n{joli}")` dans `texte_et_valeur`) | `structure` de `mcp_pieges` (panique sur l'en-tête absent), donc les tests d'enveloppe et d'outils ; `mcp_enveloppe_e2e.rs:638` et `mcp_outils_e2e.rs:673-679` ; K8 et K23 (S1) s'ils lisent l'en-tête — l'ensemble rouge constaté et nommé au Dev Agent Record |
| M15 | ajouter `"structuredContent": {}` au `json!` de `un_bloc` (`reponse.rs:42-47`, sert S6 à S8) | K23 (S6, S8), garde, `echeance_ne_transmet_rien` (S7) |
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

L'ordre compte : la recette (point 3) se joue sur une **release candidate**, avant que les textes ne soient figés et
que le tag final ne déclenche `release.yml` — sinon lever une réserve du manuel exigerait une 0.16.2.

1. `scripts/prepare-release.sh 0.16.1` : dix crates à 0.16.1, CHANGELOG daté, rappels lus ; aucune migration, donc
   `kesh_version_min_required` inchangé et aucune exemption du registre de rejeu touchée (à constater).
2. Le paragraphe introductif du CHANGELOG relu après datation ; la ligne « observé avec Claude Code 2.1.312 ». La date posée par le script se **recontrôle au point 8** : si le tag tombe un autre jour (la recette du point 3 peut le repousser), redater l’entrée et `\keshReleaseDate`.
3. **Une release candidate `v0.16.1-rc1`, puis la recette Claude Code.** Les points 1, 2, 4 et 5 faits, les textes
   gardant leur réserve (« à confirmer… sur la 0.16.1 »), l'orchestrateur pose le tag `v0.16.1-rc1` — convention des tags
   `v0.1.0-rc1` à `-rc3` : Cargo reste à `0.16.1`, et le smoke compare la base du tag à `/health`
   (`.github/workflows/release.yml:139-143`). ⚠️ **Avant ce tag**, `release.yml` se corrige : il étiquette et pousse
   `:latest` pour tout tag (l. 64, 159) et crée une GitHub Release qui n'est pas marquée pré-release (l. 161-165) ; un tag portant un
   suffixe `-rcN` ne pousse que `gcorbaz/kesh:0.16.1-rc1` et marque sa release `prerelease` ; `docs/ci.md` (l. 18,
   163-164) et le manuel d'administration (`admin-manual.tex:2435`), qui disent qu'un tag pousse toujours `:latest`, se
   corrigent avec lui. Le changement va à la PR de
   release, revu comme elle. L'image candidate s'installe sur une **base neuve** (aucune donnée réelle) ; le Project
   Lead y joue la recette, guidé par l'orchestrateur, capture par relais comme le 2026-10-28, version de Claude Code
   relevée (`claude --version`) et écrite au CHANGELOG et au manuel si elle diffère de 2.1.312.
   Les URL du point 4 désignent `v0.16.1`, qui n'existe pas encore pendant la candidate : c'est attendu, la garde G19
   ne compare qu'à `\keshVersion`.
   **Le protocole de la recette ne s'écrit pas dans cette fiche** (décision du Project Lead, 2026-10-28) : il s'écrit à
   la préparation de la candidate, dans un document de release hors dépôt (`kesh-gate-logs/rel0161/recette.md`), qui
   dit, pour chaque réserve — items 5 et 7 de l'AC-8, dernière phrase de l'entrée *Corrigé* du CHANGELOG (AC-9),
   README (AC-10) —, l'observation qui la lève et la conduite si elle manque. Il reprend ce que les passes de validation
   ont appris (rapports `kesh-gate-logs/v0161-validate-p{2,3,4}*.md`). La règle, seule, est ici : **une réserve ne se
   lève que sur une observation de la recette**, et rien de ce qu'elle n'a pas vu ne se retire. Si la recette lève les
   réserves, les textes se réécrivent (manuel, CHANGELOG, README, PDF) et le tag final `v0.16.1` se pose (point 8) ;
   sinon, les réserves restent, #811 reste ouverte, et l'opportunité de publier se pose au Project Lead.
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
8. **L'ordre des gestes.** La PR de release s'ouvre avec les points 1, 2, 4 à 6 et la correction de `release.yml`, et
   **ne se fusionne pas avant le verdict de la recette** : `main`, le site (`deploy-pages`), le README et les versions
   Cargo n'annoncent rien avant lui. Le tag `v0.16.1-rc1` se pose sur la tête de la branche de release, PR ouverte (le
   workflow exécuté est celui du commit tagué, correction comprise). Aussitôt la candidate publiée, on constate que
   `gcorbaz/kesh:latest` n'a pas bougé (même condensé qu'avant le tag) et que la release GitHub `v0.16.1-rc1` est
   marquée pré-version ; sinon, on corrige avant la recette. Après la recette, sur la même branche : les textes
   réécrits ou laissés selon le point 3, puis les points 2, 4 et 5 rejoués — date du CHANGELOG et `\keshReleaseDate`
   au jour du tag final, PDF régénérés. Puis fusion, et tag `v0.16.1` sur le commit de fusion de `main` ; Docker :
   image `0.16.1` = `:latest`, smoke `/health` (version `0.16.1`).
9. La PR de story porte `refs #811`, `refs #812` (jamais de mot-clé de fermeture, ni dans ses commits ni dans son corps).
10. La **PR de release** porte `closes #812` ; `closes #811` **seulement** si la recette du point 3 a levé les réserves
    de la réception (items 5 et 7 de l'AC-8, CHANGELOG, README), `refs #811` sinon ; le corps de la PR s'écrit après le
    verdict, avant la fusion (point 8).

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
recette a vu. Le PDF n'est pas régénéré dans la story : la revue relit le `.tex`. *(Annoté le 2026-10-29, revue de
code P1, A4 : les PDF ont été régénérés et commités par le développement, C-v016-1-11 ; la revue relit aussi le PDF.)*

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


## Renvoyé à la préparation de la release (validation P6, non remédié dans la fiche)

Constats de la P6 (`kesh-gate-logs/v0161-validate-p6.md`), venus après la dernière passe où l'orchestrateur remédie
lui-même : la fiche n'est pas corrigée, l'orchestrateur les traite en écrivant le protocole de la candidate (point 3),
et dit au document de release, ligne lue citée, comment chacun est réglé.

1. **P6-1 (MEDIUM)** : le tag final `v0.16.1`, posé sur le commit de squash, reconstruit l'image ; rien ne vérifie
   qu'elle porte le code que la candidate a éprouvé. À constater avant le tag final :
   `git diff --stat v0.16.1-rc1 <commit de fusion> -- crates/ frontend/ Dockerfile Cargo.lock rust-toolchain.toml` vide ;
   sinon, nouvelle candidate `-rc2` et recette rejouée.
2. P6-2 (LOW) : de `docs/ci.md`, seule la l. 164 dit qu'un tag pousse `:latest` ; les l. 18 et 163 restent vraies.
3. P6-3 (LOW) : au point 8, la fusion et le tag final restent subordonnés à la décision du Project Lead quand une
   réserve demeure (point 3).
4. P6-4 (LOW) : si la candidate a déplacé `:latest`, le rétablir sur le condensé de la 0.16.0, relevé avant le tag.
5. P6-5 (LOW) : `admin-manual.tex:2437` dit les PDF attachés à la release GitHub, ce que `release.yml` ne fait pas —
   corrigé au même geste que la l. 2435, ou ouvert en issue.
## Dev Agent Record

### Agent Model Used

Opus 5.5 (`claude-opus-5-5`), autonomie, 2026-10-29 — worktree `/home/gcorbaz/devel/kesh-0161`, base `kesh_v0161`,
E2E `kesh_e2e_v0161`, port 3161, cible `target/` du worktree.

### Completion Notes List

**Étape 0 — § *À recontrôler*, point par point (lignes lues à `d3b3b649`).**

1. *Numéros de ligne* : confirmés — S1 `politique.rs:157-172` (clé à `:168`), S2 `reponse.rs:49-56` (clé `:53`), S3
   `:57-61` (`:59`), S4 `:62-76` (`:70`), S5 `prompts.rs:207-216` (`:213`), S6-S8 `reponse.rs:77-79` (`un_bloc`
   `:42-47`), S9 `:284-291` ; `sortie.rs:197` (`Montant`), `:434` (`format!("{TEXTE_ENTETE_DONNEES}\n\n{joli}")`), `:458`
   (`Ok(Donnees::nouvelles(texte, Some(valeur)).avec_resume(resume))`) ; `donnees_une_seule_source` est à `:568` (la fiche
   disait 567-592 : le `#[test]` est à 567). Sites de test : conformes au tableau (`common/mcp.rs:380, 409, 463, 496,
   599, 557-560, 574, 614-623, 626-648` ; `mcp_envois_e2e.rs:40, 194, 256, 263, 889, 1129` ; `mcp_transport_e2e.rs:118,
   1553` ; `mcp_journal_e2e.rs:813` ; `mcp_outils_e2e.rs:673-686` ; `mcp_prompts_e2e.rs:679` ; `mcp_gardes_lexicales.rs:
   613, 617, 638` ; `mcp_contrat.rs:608-633, 672-679, 744-748`).
2. *Aucun autre site* : `repartiteur.rs` — `ReponseMcp::Resultat` aux seules l. 187, 200, 218 (`initialize`, `ping`,
   `tools/list`) ; `appeler` (l. 300) et `relire` (l. 404) ne rendent que `OutilEnErreur`, `Outil`, `Envoi`, `Accuse`,
   `Erreur` (`grep -n "ReponseMcp::" repartiteur.rs`).
3. *`rust_decimal` 1.43.0* (`Cargo.lock:3385-3386`) : `normalize` « Strips any trailing zero's from a `Decimal` and
   converts -0 to 0 » (`decimal.rs:1399`) ; `rescale` « Rescaling to a higher scale preserves the value » (`:1026`) ;
   `from_parts` efface le signe d'un zéro (`:540-553`, `if lo == 0 && mid == 0 && hi == 0 { false }`) ; `Neg`
   (`:2609-2616`) pose le signe par `set_sign_negative` : `-Decimal::new(0, 4)` est un zéro négatif (témoin du test).
4. *Le rouge de l'AC-7* : reproduit tel que la recette l'a vu, sur un exercice sans écriture —
   `[("totalActifs", String("0")), ("totalPassifs", String("0")), ("totalFondsPropres", String("0")), ("report",
   String("0.0000")), ("resultat", String("0"))]` (journal `kesh-gate-logs/v0161-etape0-rouges.log`).
5. *Observation (g)* (#723, commentaire de recette du 2026-10-28, lu par `gh issue view 723 --comments`) : « Menu :
   `kesh:kesh-assistant`, `kesh-direct:kesh-assistant`… ; commande envoyée : `/mcp__kesh-direct__kesh-assistant` ;
   `prompts/get` sans argument, cadre repris par le modèle. »
6. *Motif de la garde* : avant correctif, `(?i)structured_?content` ne rend, en code de `src/`, que les cinq lignes
   attendues (`politique.rs:168`, `prompts.rs:213`, `reponse.rs:53, 59, 70`) — constaté par le rouge de la garde ; la
   sixième occurrence de `src/` (`prompts.rs:205`) est un doc-comment, exclu.
7. *`serde_json` sans `preserve_order`* : `Cargo.lock:3610-3620`, dépendances `itoa`, `memchr`, `serde`,
   `serde_core`, `zmij` — aucune `indexmap` ; aucun `preserve_order` dans un `Cargo.toml` du workspace.

**Rouges constatés avant tout patch** (commit `9979706d`, journal `v0161-etape0-rouges.log`, nextest : 4 exécutés,
4 échoués) :
- `k23_un_seul_canal` — rouge au site S1 : `left: ["content", "isError", "structuredContent"]`, `right: ["content",
  "isError"]` ;
- `structured_content_nulle_part` — rouge, les cinq lignes du point 6 ;
- `bilan_montants_une_seule_forme` — rouge, sortie du point 4 ;
- `montant_forme_unique` — rouge au premier cas, `left: String("0.0000")`, `right: String("0.00")`.
Le rouge propre de S2, S3, S4 et S5 est constaté par les mutations M2, M11, M1 et M3 (ci-dessous).

**Le correctif.** #811 : les cinq lignes retirées ; `Depose` et `EnAttente` passent par `un_bloc` ;
`DonneesEnvoyables::en_resultat` est un `json!` à deux clés ; `Donnees::nouvelles(texte)`, champ `structure` retiré ;
`sortie::donnees` ne garde de `texte_et_valeur` que le texte (signature et test inchangés). #812 : `Montant` sérialise
`normalize()`, porté par `rescale(2)` si l'échelle est inférieure à 2. Aucun texte de `textes.rs` ne change ;
`textes_envoi_epingles_par_un_litteral` et K1 verts sans retouche.

**`grep -rn -i "structur" crates/kesh-api/src/mcp/`, relu en entier (15 occurrences)** : `politique.rs:18`,
`prompts.rs:205`, `sortie.rs:8` — doc-comments réécrits qui disent l'**absence** (C-v016-1-1) ; `textes.rs:163`
(« le JSON indenté de la structure » : la structure `sortie!`, source du texte, toujours vraie) ; `sortie.rs:5, 13, 17,
51, 90, 478, 483, 629` (la structure `sortie!` et les structures de test) ; `sortie.rs:307` (« coordonnée structurée
d'un tiers ») ; `outils/contacts.rs:140`, `outils/pieces.rs:64` (« Déstructuration »). `mcp/mod.rs` n'en parle pas.

**Site par site, le test qui le couvre (AC-3)** : S1 — K23, `cle_dispensee_sert_chaque_outil` (`len() == 2`),
`attendu_version_delai` (`mcp_transport_e2e.rs`) ; S2 — K23, K10, `attendu_depose` ; S3 — K23, `attendu_en_attente`,
`mcp-sends.spec.ts` (second test) ; S4 — K23, K10, `attendu_valide`, `mcp-sends.spec.ts` ; S5 — `accuse`
(`mcp_prompts_e2e.rs`, égalité du corps entier) ; S6 — K23, `attendu_refuse`, `mcp-sends.spec.ts` ; S7 —
`echeance_ne_transmet_rien` (`attendu_echu`) ; S8 — K23, `attendu_inconnu` ; S9 — K23, `attendu_outil_en_erreur`. Aucun
site sans test ; la garde AC-4 couvre les cinq sites à clé littérale.

**`une_seule_representation`** : `PIEGE_STRUCT`, sa constante et la ligne `structure` retirés ; la preuve sur
`PIEGE_RESUME` gardée ; doc-comment : l'AC-B2 de la 32-8, pour la donnée structurée, est tenu par construction depuis
la 0.16.1 (C-v016-1-5).

**Tests de taille** : `tailles_mesurees` et `plafond_au_maximum` restent verts sans retouche — **aucun recalage**,
aucune constante `LIMITE_*` ni plafond (`13_312`, `23_552`) touché.

**Mutations** (campagne sous verrou, `kesh-gate-logs/v0161-mutations/`, `muter.py` motif vérifié unique, restauration
`git checkout -- <f> && touch <f>`, filtre `binary(/^mcp/) | test(/^mcp::/)`, 224 tests par run ; `git status` propre
après la campagne). **Quinze jouées, quinze tuées** :

| # | Rougit (extrait ; liste complète au `bilan.txt`) |
|---|---|
| M1 | K23, K10, garde, `tel_quel_recu_octet_pour_octet`, `une_seule_representation` (`attendu_valide`) — 12 tests |
| M2 | K23, K10, garde, `cle_requise_deposee`, `cle_requise_depose_chaque_outil` (`attendu_depose`) — 31 tests |
| M3 | garde, `proposition_servie_accuse_et_audit`, `jamais_servi_avant_l_approbation`, `proposition_refus_corps_egaux` (`accuse`) — 4 |
| M4 | K23, garde, `cle_dispensee_sert_chaque_outil` (`mcp_outils_e2e`, l'ex-`:683`), `cle_dispensee_servie` — 10 |
| M5 | K23, K10, `tel_quel_recu_octet_pour_octet` (`attendu_valide`) — 11 ; **la garde reste verte** (angle mort assumé, AC-4) |
| M6 | K23, K10, `tel_quel_recu_octet_pour_octet` (`attendu_valide`) — 14 |
| M7 | `montant_forme_unique`, `bilan_montants_une_seule_forme` — 2 |
| M8 | `montant_forme_unique`, `bilan_montants_une_seule_forme` — 2 |
| M9 | `montant_forme_unique`, `bilan_montants_une_seule_forme` — 2 |
| M10 | les tests d'enveloppe (`pieges_toujours_enveloppes`…) et d'outils (`tailles_mesurees`…) — 17 |
| M11 | K23, garde, `e1_rien_ne_part_sous_cle_requise` (`attendu_en_attente`), `chaque_appel_est_journalise` — 4 |
| M12 | K23, garde, `arguments_de_kesh_lire_envoi`, `erreurs_de_demande_comprehensibles` — 24 |
| M13 | K8, K23 (S1, l'en-tête), `donnees_une_seule_source`, `pieges_toujours_enveloppes` (`mcp_enveloppe_e2e`), `cle_dispensee_sert_chaque_outil`, `bilan_montants_une_seule_forme` (via `structure`/`split_once`) — 30 |
| M15 | K23, garde, `echeance_ne_transmet_rien` (S7), `refus_ne_transmet_rien` — 31 |
| M14 | M1 rejouée, **`mcp-sends.spec.ts` seule** : le premier test rougit — `Error: v016-1 : clés exactes content, isError`, `+ "structuredContent"` ; le second (relecture en attente, non mutée) passe (`v0161-mutations/M14.log`, `v0161-e2e-3.log`) |

M4 réinsère la clé sans rétablir le champ (C-v016-1-12).

**Textes.** Manuel d'administration : les lieux du tableau de l'AC-8, chacun cité « observé avec Claude Code
2.1.312 » sur son observation ((a)/(e)/(h) item 3, (b) `sec:mcp-origin`, (g) item 4, (d) items 5 et 7 avec leur
réserve) ; *Passer à la 0.16.1* (`sec:maj-0-16-1`), chaque affirmation vérifiée : `git diff --stat main --
crates/kesh-db/migrations` vide ; `main.rs:122` (`check_downgrade_protection(&pool, env!("CARGO_PKG_VERSION"))`),
`version.rs:222-230` (lecture du seul `kesh_version_min_required`), dernière valeur posée `'0.14.0'`
(`20261015000001_imported_documents_without_qr.sql:54`) ; `routes/admin.rs:216-219` (`check_import_version_compat` sur
le seul `kesh_version_min_required` du manifeste), `kesh_version` du manifeste informatif (l. 247 et 582) ; les
phrases par cas tirées de `textes.rs:197-243`. `delaiInstructionS` en `30.0` : `f64` sérialisé par `serde_json`
(`attendu_version_delai`, `mcp_transport_e2e.rs:102-107`). Manuel utilisateur : *Invoquer un prompt*.
`grep -nE "éprouv|structur|kesh-assistant" user-manual.tex` après patch : les « structur » restants sont d'autres sujets
(plan comptable, adresse structurée, référence structurée, bilan CO 959a) ; aucun « éprouvé » restant ;
`kesh-assistant` aux seules l. 3269-3272 réécrites. « non éprouvé » restants dans `admin-manual.tex` : `sec:mcp-client`
(la phrase « les autres restent marqués »), intro de `sec:mcp-brancher`, item 3 (`/mcp`), item 7 (« décrit d'après le
code »), item 10 (401, (f) non observé), *Après une restauration* (restauration sur l'installation même), filtre (un
Traefik non déclaré) — chacun hors des observations (a) à (h). PDF régénérés (C-v016-1-11) et contrôlés aplatis,
`grep -oF` sous `timeout 60` : « observé avec Claude Code 2.1.312 » ×5, « Passer à la 0.16.1 » ×2, « Claude Code n’en
envoie pas » ×1, « au moins deux décimales » ×1, « à confirmer avec Claude Code sur la 0.16.1 » ×1,
`mcp__<serveur>__kesh-assistant` ×1, « précédé d’une ligne de Kesh qui annonce l’envoi validé » ×1, « aucun contenu
structuré ne l’accompagne (issue #811) » ×1 ; « donnée structurée » ×0 ; au manuel utilisateur `kesh:kesh-assistant`
×1, `mcp__<serveur>__kesh` ×1, « kesh-assistant (MCP) » ×0. CHANGELOG `## [0.16.1] — Non publié`, *Modifié* puis
*Corrigé* ; README l. 49.

**Grep de la valeur des symptômes** (`structuredContent`, `donnée structurée`, `contenu structuré`, `non éprouvé`,
`0.0000`) sur `docs/`, `README.md`, `CHANGELOG.md`, `website/`, `crates/`, `frontend/` : restent les commentaires et
messages qui disent l'absence (code et tests de la story), le manuel et le CHANGELOG ; `website/` n'en porte aucun ;
`0.0000` ne subsiste que dans l'API REST (`docs/api-external.md`, `project_reports_e2e.rs`, `inbox_import_e2e.rs`) —
hors MCP, #813.

**Annotations des relevés de l'Epic 32** (liste triée, C-v016-1-14) : `planning-artifacts/epic-32-spike-mcp.md` (E1,
§ 5.3.1) ; `32-6-endpoint-mcp.md` (AC-a28) ; `32-8-validation-avant-envoi.md` (note « Une seule représentation »,
AC-B2, AC-B5, AC-B6) ; `32-9-outils-de-lecture.md` (note « Une source, deux représentations », AC-a15, AC-a21) ;
`32-10a-prompts-en-base.md` (AC-iii2, note de l'accusé d'AC-iii3) ; `32-11-brancher-un-agent.md` (décision de K10) ;
`epic-32-choix-autonomes-326.md` (C-32-6-46), `-328.md` (C-32-8-4), `-329.md` (C-32-9-6), `-3210a.md` (C-32-10a-17),
`-3211.md` (C-32-11-39, C-32-11-42) — dix-huit annotations, onze fichiers, `git diff d3b3b649 -- _bmad-output` sans
ligne retirée hors fichiers `v016-1-*` et `sprint-status.yaml`. Laissés, avec leur motif : C-v016-1-14.

**Gates réellement exécutés, au dernier commit de code** (`de6d3507` ; `20355dac` ne touche que textes, PDF et
`_bmad-output/`) :
- backend `scripts/test-fast.sh --ci` sous verrou, base `kesh_v0161` remise à zéro avant (drop, create, migrations du
  worktree, seed) : fmt, clippy `-D warnings`, garde des droits, **nextest 4460 exécutés, 4460 passés, 7 ignorés**
  (`v0161-gate-backend.log`, 204 s) — 4456 au journal de la release 0.16.0, plus les quatre tests de la story ;
  `#[test]`/`#[sqlx::test]` de `crates/` : 4279 à `d3b3b649`, 4283 à `HEAD` ;
- frontend sous verrou : `npm run check` (0 erreur, 29 avertissements préexistants), `lint-i18n-ownership`, **Vitest
  181 fichiers, 2150 tests passés**, `npm run build` (`v0161-gate-frontend.log`) ;
- **E2E Playwright complet** (recette `docs/testing.md`, port 3161, dossiers `/tmp/kesh-e2e-v0161-1/`, base
  `kesh_e2e_v0161` reconstruite) : **324 passés, 8 échoués, 17 ignorés** (15,3 min, `v0161-e2e-1.log`). Les huit,
  fichier par fichier contre `docs/testing.md` § *Les échecs attendus* : `mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` — les sept KF-029 (#97) ;
  `reminders.spec.ts:235` (`page.fill` en délai dépassé au login) — **rejoué seul : passé** (`v0161-e2e-2.log`), donc
  pollution d'état entre specs ; 7 + 1 dans la fourchette 7 à 10. **`mcp-sends.spec.ts` : ses deux tests passés**
  (l. 149 et 216).
- Gate ciblé intermédiaire : `binary(/^mcp/) | test(/^mcp::/)`, 224 passés (`v0161-cible1.log`).

Tout backend lancé a été arrêté par son PID (757371, 898486, 904579) ; aucun ne reste.

**Écarts à la fiche** : PDF régénérés et commités (C-v016-1-11, consigne de l'orchestrateur) ; M4 sans rétablir le
champ (C-v016-1-12) ; `donnees_une_seule_source` à `:568` et non `:567`.

**Défauts observés, hors périmètre** : aucun neuf. `reminders.spec.ts:235` est un cas de la pollution déjà décrite
par `docs/testing.md` (rien à ouvrir) ; le format REST de #813 est déjà tracé.

**Remédiation de la revue de code P1 (2026-10-29, Opus 5.5, commit `d2b65bc0`).** Le détail des findings est au Change
Log. Ce qui a tourné :
- *Mutation `scale() < 3`* (prescrite ; appliquée par l'outil Edit sur `sortie.rs:217`, filtre `binary(/^mcp/) |
  test(/^mcp::/)`, sous verrou, restaurée par `git checkout -- sortie.rs && touch sortie.rs`, `git status` propre) :
  **224 passés, 0 échec** (`v0161-mutations/R1-lt3.log`) — **équivalente, démontrée** : après `normalize`, une échelle de
  2 est déjà à deux décimales et `rescale(2)` n'y change rien, une échelle de 3 ou plus n'est touchée ni par `< 2` ni par
  `< 3`. La variante que visait E3 est `scale() < 4` (ou `<= 3`), qui arrondit une échelle de 3 : jouée de même, **tuée**
  par `montant_forme_unique` seul — `left: String("1.13")`, `right: String("1.125")`, 223 passés, 1 échec
  (`R1-lt4.log`). Avec les huit cas d'avant (échelles 0, 1, 2 et 4), elle aurait survécu : déduit des échelles, non joué.
- *Gate ciblé* après patch : `fmt`, `clippy --workspace --all-targets -D warnings` (`v0161-r1-clippy.log`), nextest
  `binary(mcp_outils_e2e) | test(/^mcp::sortie::/)` : 31 passés (`v0161-r1-cible1.log`).
- *Gate complet au dernier commit de code* (`d2b65bc0`), base `kesh_v0161` reconstruite avant (drop, create, migrations
  du worktree, seed) : `scripts/test-fast.sh --ci` sous verrou — garde des droits conforme, fmt, clippy, **nextest 4460
  exécutés, 4460 passés, 7 ignorés** (`v0161-r1-gate-backend.log`, 188 s ; aucun test ajouté, deux cas et une seconde
  moitié dans des tests existants) ; frontend `npm run check` (0 erreur, 29 avertissements préexistants) et
  `lint-i18n-ownership` (PASS) (`v0161-r1-gate-frontend.log`) — aucun fichier frontend touché, Vitest et build non
  rejoués.
- *PDF* : `make fr` sous verrou (`v0161-r1-make-fr.log`, admin et utilisateur stables en 2 passes), aplatis et contrôlés
  par `grep -oF` sous `timeout 60` : utilisateur `/mcp__<serveur>__kesh-assistant` ×1 (0 avant), mot à `xMax` 390,67
  (`pdftotext -bbox`, 595,84 avant) ; administration « préfère le contenu structuré » ×0, « lit donc le même texte » ×0,
  « que le bloc de texte (content) » ×1, « à confirmer avec Claude Code sur la 0.16.1 » ×2 (1 avant), « observé avec
  Claude Code 2.1.312 » ×5, « non éprouvé » ×7. Débordements : aucun `Overfull \hbox` des journaux LaTeX dans un
  paragraphe que la story touche (admin l. 2480-2489, 3177, 3219, 3237, 3324, 3341, 3346-3348, 3350 ; utilisateur
  l. 3270-3276) ; aucun mot au-delà de `xMax` 524 sur la page du paragraphe utilisateur ; les quatre mots au-delà au
  manuel d'administration (p. 132 `kesh-developpement`, p. 135 `Content-Type:` ×2 et `»),`) sont dans des paragraphes
  non touchés (`v0161-r1-bbox-*.txt`).
- *E2E complet* (recette `docs/testing.md`, port 3161, dossiers neufs `/tmp/kesh-e2e-v0161-r1-1/`, base
  `kesh_e2e_v0161` reconstruite) : **325 passés, 7 échoués, 17 ignorés** (14,1 min, `v0161-e2e-r1-1.log`) ; les sept
  sont les KF-029 (#97) de `docs/testing.md` — `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
  `onboarding.spec.ts:57`, `:77`, `:150` ; **`mcp-sends.spec.ts` : ses deux tests passés** (l. 149 et 216). Backend
  arrêté par son PID (1240105).

### File List

- `crates/kesh-api/src/mcp/politique.rs`, `reponse.rs`, `prompts.rs`, `sortie.rs`
- `crates/kesh-api/tests/mcp_contrat.rs`, `mcp_gardes_lexicales.rs`, `mcp_outils_e2e.rs`, `mcp_envois_e2e.rs`,
  `mcp_enveloppe_e2e.rs`, `mcp_transport_e2e.rs`, `mcp_journal_e2e.rs`, `mcp_prompts_e2e.rs`, `common/mcp.rs`,
  `common/mcp_pieges.rs`
- `frontend/tests/e2e/mcp-sends.spec.ts`
- `docs/manual/fr/admin-manual.tex`, `admin-manual.pdf`, `user-manual.tex`, `user-manual.pdf`
- `CHANGELOG.md`, `README.md`
- `_bmad-output/implementation-artifacts/v016-1-mcp-sorties.md`, `v016-1-choix-autonomes.md`,
  `v016-1-review-prompt-p1.md`, `sprint-status.yaml`, et les onze relevés annotés listés ci-dessus

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
- 2026-10-28 — validation P4 (Opus, passe ciblée sur le diff `324b6986..0e1f25f3`, prompt
  `v016-1-validate-prompt-p4.md`, rapport hors dépôt `kesh-gate-logs/v0161-validate-p4.md`) : 0/0/3/5 — 3 MEDIUM, 5 LOW
  (tendance 8 → 6 → 1 → 3 au-dessus de LOW). Les trois MEDIUM portent sur les critères de la recette de la release
  (C3 absent de l'introduction du point 3 et de la conduite d'échec ; C3 indécidable si Claude Code a le dépôt, où le
  chemin `/admin/mcp/sends` est écrit ailleurs ; C2 sans consigne de recopie), et **naissent tous de la remédiation P3** :
  signal de découpage (recyclage) déclaré au Project Lead. Sur sa décision, le protocole de la recette **sort de la
  fiche** : le point 3 ne garde que la règle (une réserve ne se lève que sur une observation de la recette), la recette
  se joue sur une release candidate `v0.16.1-rc1` (convention `v0.1.0-rc1`), et `release.yml`, qui pousse `:latest`
  pour tout tag et ne marque aucune release `prerelease`, se corrige avant ce tag ; le protocole s'écrira à la
  préparation de la candidate, hors dépôt, avec les leçons des passes P2 à P4. Points 8 et 10, AC-8 item 7, registre
  C-v016-1-7 et C-v016-1-10 alignés. LOW remédiés : P4-4 (absorbé par l'alignement du registre), P4-5 (M7 et M9 :
  rouge au premier cas qui diffère), P4-6 (refus, échéance et numéro inconnu n'ont jamais porté de JSON), P4-7
  (recalage côté fixture, ni constante `LIMITE_*` ni plafond), P4-8 (`reponse.rs:42-47` à M15 ; l'entrée P3 reste telle
  quelle). Aucun finding rejeté. Prémisses relues : `release.yml:6`, `:64`, `:139-143`, `:159`, `:161-165` ; tags
  `v0.1.0-rc1` à `-rc3` (`git tag -l`) ; `prepare-release.sh:39-42` (refuse un suffixe, d'où Cargo à `0.16.1` et le
  seul tag en `-rc1`) ; `fixture_tailles` (`mcp_outils_e2e.rs:785`).
- 2026-10-28 — validation P5 (Sonnet, passe ciblée sur le diff `e9d2eefe..687cf600`, prompt
  `v016-1-validate-prompt-p5.md`, rapport hors dépôt `kesh-gate-logs/v0161-validate-p5.md`) : 0/0/1/5 — 1 MEDIUM, 5 LOW
  (tendance 8 → 6 → 1 → 3 → 1 au-dessus de LOW). Remédiés : P5-1 (point 8 : PR de release fusionnée seulement après le
  verdict, tag `-rc1` sur la tête de la branche, tag final sur le commit de fusion), P5-2 (registre C-v016-1-3 : la
  recette « peut le montrer »), P5-3 (`docs/ci.md:18`, `:163-164` et `admin-manual.tex:2435` corrigés avec
  `release.yml`), P5-4 (points 2, 4 et 5 rejoués après la recette), P5-5 (`closes #811` sur les items 5 et 7), P5-6
  (`:latest` et la pré-version constatés après la candidate). Prémisses relues : `docs/ci.md:18`, `:163-164` ;
  `admin-manual.tex:2435` ; registre l. 75-76. Aucun finding rejeté. **Dernière remédiation de l'orchestrateur** : dès la
  P6, ce qui reste se retire ou se renvoie au développement.
- 2026-10-28 — validation P6 (Opus, passe ciblée sur le diff `cd011a49..aacaa953`, prompt
  `v016-1-validate-prompt-p6.md`, rapport hors dépôt `kesh-gate-logs/v0161-validate-p6.md`) : 0/0/1/4 — 1 MEDIUM, 4 LOW
  (tendance 8 → 6 → 1 → 3 → 1 → 1 au-dessus de LOW). Sixième passe : l'orchestrateur ne remédie plus. Les cinq
  constats portent sur la procédure de release, qu'il exécute lui-même ; ils sont **renvoyés à la préparation de la
  release** (§ *Renvoyé à la préparation de la release*), sans correction de la fiche. Prémisse de P6-1 relue : dépôt
  fusionné en squash (`CLAUDE.md`, § *Issue Tracking Rule*), image reconstruite au tag (`release.yml:55-69`).
- 2026-10-29 — **validation terminée** sur décision du Project Lead (« v016-1 validé », 2026-10-29), au terme de la P6 :
  six passes, tendance 8 → 6 → 1 → 3 → 1 → 1 au-dessus de LOW. Le dernier MEDIUM (P6-1) n'est pas remédié dans la fiche :
  il porte sur la procédure de release et reste au § *Renvoyé à la préparation de la release*, que l'orchestrateur
  traite au protocole de la candidate. Statut inchangé, `ready-for-dev`.
- 2026-10-29 — développement (Opus 5.5, autonomie) : #811 (aucun `structuredContent`, K23, garde lexicale), #812
  (`Montant` normalisé, deux décimales au moins), manuels FR et PDF, CHANGELOG `[0.16.1] — Non publié`, README,
  annotations des relevés de l'Epic 32 ; quinze mutations, quinze tuées ; gate backend 4460/4460 (7 ignorés), Vitest
  2150, E2E 324 passés / 8 échecs attendus ; choix C-v016-1-11 à 14 ; prompt de revue P1 `v016-1-review-prompt-p1.md`.
  Statut `review`.
- 2026-10-29 — revue de code P1 (Sonnet, trois lentilles B, E, A, prompt `v016-1-review-prompt-p1.md`, rapports hors
  dépôt `kesh-gate-logs/v0161-review-p1-{B,E,A}.md`) : B 0/0/1/5, E 0/0/1/3, A 0/0/0/5 — 2 MEDIUM, 13 LOW ; B3, LOW à
  sa lentille, reclassé MEDIUM par l'orchestrateur comme le même constat que E1. Remédiation (Opus 5.5, commit
  `d2b65bc0`), chaque prémisse relue avant d'appliquer :
  - **B1** (MEDIUM, appliqué) — lu `user-manual.tex:3273`, `\texttt{/mcp\_\_<serveur>\_\_kesh-assistant}` en fin de
    ligne, et le mot à `xMax="595.841620"` du rapport B : la commande passe seule sur sa ligne (`center`), PDF
    régénéré, chaîne entière ×1, `xMax` 390,67.
  - **B3 = E1** (MEDIUM, appliqué) — lu `admin-manual.tex:3219`, « un client qui préfère le contenu structuré quand il en
    trouve un, comme Claude Code 2.1.312, lit donc le même texte qu'un autre » : la phrase dit ce que Kesh rend (le seul
    bloc `content`) et porte la réserve des items 5 et 7 (« à confirmer avec Claude Code sur la 0.16.1 »). Grep de la
    valeur (`préfère le contenu structuré`, `lit donc le même texte`, `structur`, puis `atteint`, `transmet`) sur `docs/`,
    `CHANGELOG.md`, `README.md`, `website/` : seule autre phrase qui affirmait la lecture côté client, le titre du
    CHANGELOG (E2 = A1) ; README l. 49 et items 5 et 7 déjà réservés.
  - **B2** (LOW, appliqué autrement que proposé) — lu `v0161-etape0-rouges.log:493`, `"actifs":[]`, `"passifs":[]`,
    `"fondsPropres":[]` : la boucle passait à vide, et l'assertion proposée rougirait toujours ; une écriture de
    `10.0000` fait paraître des lignes, au moins une exigée, chaque solde `"10.00"` ou `"-10.00"` (C-v016-1-16).
  - **E2 = A1** (LOW, appliqué) — lu `CHANGELOG.md:23`, « **Le contenu validé atteint l'agent** » : titre neutre, vrai
    dans les deux issues de la recette.
  - **A2** (LOW, appliqué) — lu `CHANGELOG.md:15`, « le relit une fois après l'accord, sans boucle. » : réserve de (d)
    ajoutée, comme à l'item 5.
  - **E3** (LOW, appliqué) — lu `sortie.rs:217`, `if d.scale() < 2 {`, et les échelles des huit cas (0, 1, 2, 4) : cas
    `1.125` et `2.5000` ajoutés ; la mutation `< 3` est équivalente (démontrée, 224/224 verts), la mutation `< 4` est
    tuée par `1.125` (Dev Agent Record).
  - **B5 = E4** (LOW, appliqué) — lu `sortie.rs:459`, `let (texte, _) = texte_et_valeur(sortie)?;` : valeur gardée,
    doc-comments de `texte_et_valeur` et de `donnees` réécrits (C-v016-1-15).
  - **A5 = B6** (LOW, appliqué) — lu `rust_decimal-1.43.0/src/decimal.rs`, doc de `rescale`, « This function never fails.
    Consequently, if a scale is provided that is unable to be represented using the given mantissa, then the maximum
    possible scale is used » : borne écrite au doc-comment de `Montant` ; manuel et CHANGELOG laissés (C-v016-1-17).
  - **A4** (LOW, appliqué) — lus l'AC-8 (« Le PDF n'est **pas** régénéré par le développeur »), les Dev Notes (« Le PDF
    n'est pas régénéré dans la story ») et C-v016-1-10 (« ni aux PDF ») : trois annotations datées, renvoi à
    C-v016-1-11, aucun relevé réécrit.
  - **A3 et B4** (LOW, **rejetés**) — prémisse « l'en-tête `Authorization: Bearer` est déduit, non observé » réfutée par
    l'orchestrateur sur la capture de la recette (`grep -ciF "authorization: bearer"` rend 27), que le développement ne
    lit pas : l'en-tête est observé ; phrases inchangées (`admin-manual.tex:3324`, `CHANGELOG.md:15`).
  Gates réellement exécutés : gate ciblé (fmt, clippy workspace, 31 tests) ; mutations `< 3` et `< 4` ; au dernier
  commit de code, `scripts/test-fast.sh --ci` 4460/4460 (7 ignorés), `npm run check`, `lint-i18n-ownership`, `make fr`,
  PDF aplatis, E2E complet 325 passés / 7 échecs KF-029, `mcp-sends.spec.ts` verte. Choix C-v016-1-15 à 17. Prompt de la
  revue P2 : `v016-1-review-prompt-p2.md` (Opus, passe complète, axe 5 sur la remédiation).
- 2026-10-29 — revue de code P2 (Opus, passe complète, trois lentilles B, E, A, prompt `v016-1-review-prompt-p2.md`,
  rapports hors dépôt `kesh-gate-logs/v0161-review-p2-{B,E,A}.md`) : B 0/0/2/3, E 0/0/1/5, A 0/0/0/6 — 3 MEDIUM
  distincts (B1 = E2 = A1, B2, E1 ; E2 et A1, LOW à leur lentille, sont le même constat que B1), 11 LOW. **Motif** : les
  trois MEDIUM portent sur des phrases du manuel qui disaient ce que l'agent « reçoit » ou « relit », deux écrites par
  cette story ou sa remédiation P1. Remédiation (Opus 5.5, commit `f6b338a9`), partie du symptôme et non de la liste
  (`grep -nE "agent (re)?(reçoit|relit|relira|lit)" docs/manual/fr/*.tex`, puis `reçoi[tv]`, `parvient`, `atteign`,
  `transmet`, `apprend`), chaque prémisse relue avant d'appliquer :
  - **B1 = E2 = A1** (MEDIUM, appliqué) — lu `admin-manual.tex:3219`, « Depuis la 0.16.1, la réponse ne porte
    \textbf{que} ce texte, sous toute clé », et `reponse.rs:53` (`Self::Depose(accuse) => un_bloc(textes::texte_envoi_depose(…), false)`),
    `:55-63` (`Valide` : deux blocs) : faux sous une clé requise. Paragraphe renommé « Ce que Kesh rend à l'agent » :
    clé dispensée, en-tête et JSON ; clé requise, l'accusé, puis à la relecture validée la phrase et le texte validé ;
    « quelle que soit la politique de la clé, … que des blocs de texte (`content`) et l'indication d'erreur (`isError`),
    et aucun contenu structuré », réserve « à confirmer avec Claude Code sur la 0.16.1 » gardée. Les deux deux-points
    enchaînés disparaissent.
  - **B2** (MEDIUM, appliqué) — lu `git show d3b3b649:crates/kesh-api/src/mcp/reponse.rs`, l. 53 et 59,
    `"structuredContent": { "envoi": n, "etat": "en_attente" },` : « son numéro et son état » à l'item 5
    (`admin-manual.tex:3348`) et au CHANGELOG (l. 15). Grep de la valeur (`que son état`, `que par son état`) : aucun
    autre site.
  - **E1** (MEDIUM, appliqué) — lus `admin-manual.tex:3237`, « C'est ce texte que l'agent relira », et
    `user-manual.tex:3264`, « l'agent relit ensuite ce qui a été validé, et seulement cela » : chacune dit désormais ce
    que Kesh rend à la relecture et porte la réserve « à confirmer avec Claude Code sur la 0.16.1 ; en 0.16.0, il ne
    l'atteignait pas, issue #811 ».
  - **B3 = E5** (LOW, appliqué) — lu `sortie.rs:211`, « les colonnes sont en `DECIMAL(19,4)`, quinze chiffres entiers au
    plus », et recompté `grep -ohiE "DECIMAL *\( *[0-9]+ *, *[0-9]+ *\)" crates/kesh-db/migrations/*.sql` : 25
    `(19,4)`, 4 `(18,2)`, 3 `(7,2)`, 7 `(5,2)` ; doc-comment réécrit (« seize chiffres entiers au plus »), C-v016-1-17
    annoté.
  - **B4** (LOW, appliqué) — lu `mcp_outils_e2e.rs:1526-1543` : la seconde moitié ne relisait que les soldes de ligne ;
    `totalActifs == "10.00"` et `totalPassifs ∈ {"10.00", "-10.00"}` ajoutés après l'écriture, verts.
  - **B5** (LOW, appliqué, mécanique) — lu `mcp_contrat.rs:1121`, seul `assert_eq!(r.statut, StatusCode::OK, "{STATUT_200}")`
    de `k23_un_seul_canal` : la même assertion posée avant chaque `enveloppe_conforme` des appels 232, 233, 234, 236,
    237, 238 et 239.
  - **E3** (LOW, appliqué) — lu `CHANGELOG.md:23`, « Le contenu validé est rendu dans le texte, seul canal du serveur
    MCP », et `git show d3b3b649:…/reponse.rs` (le texte validé déjà au second bloc) : titre « La relecture d'un envoi
    validé n'est plus accompagnée d'un contenu structuré qui ne portait pas le contenu validé » (C-v016-1-19).
  - **E4** (LOW, appliqué) — lu `admin-manual.tex:2488` (*Les clients MCP*), sans « modifié », et `textes.rs:217-223`
    (`", qui l'a modifié avant l'envoi"`) : la phrase dit où se lit le booléen.
  - **E6 = A2** (LOW, appliqué) — lu `admin-manual.tex:3350`, « *(observé sur la 0.16.0, où …)* » : « observé avec
    Claude Code 2.1.312 sur la 0.16.0 » ; la règle générale de l'AC-8 prime sur sa cellule de l'item 7, annotée (datée) sans être réécrite.
  - **A3** (LOW, appliqué) — lu `admin-manual.tex:3348`, « il appelle l'outil \texttt{kesh\_bilan} avec l'identifiant de
    l'exercice », sans marque, et le commentaire de recette (aucune observation ne relève l'outil appelé) : « *(non
    éprouvé : l'outil appelé n'a pas été relevé à la recette)* » ; les « non éprouvé » du manuel d'administration passent
    de 7 à 8 (`grep -oF`, `.tex` et PDF).
  - **A4** (LOW, appliqué) — lu `sprint-status.yaml:554`, « revue de code P1 à lancer » : commentaire daté ajouté en
    tête (P1 remédiée, P2 remédiée, P3 à lancer), le relevé passé laissé.
  - **A5** (LOW, appliqué) — lu `git log -1 --format=%ad 20355dac` (2026-10-29) et `git grep -cF "dépassé le 2026-10-28
    par la v016-1"` (18 occurrences, 11 fichiers, toutes écrites par cette branche) : date portée au 2026-10-29, la date
    du code qui les rend fausses ; recompté 18 / 11 après, 0 de l'ancienne. Ce sont des textes de cette branche, non des
    relevés passés.
  - **A6** (LOW, **rien changé**, tranché par l'orchestrateur) — `README.md:49` porte sa réserve (« que Claude Code
    transmette ce contenu au modèle reste à confirmer par la recette de la release »), et la release se préparera sur
    cette branche : rien n'atteint `main` avant le verdict de la recette.
  Aucun finding rejeté.

  **Phrases réécrites hors findings** (relecture entière de `sec:mcp` et de `sec:agent-mcp`, C-v016-1-18) :
  `admin-manual.tex:3066` (« Kesh ne rend à l'agent qu'un accusé »), `:3138` (« Kesh répond à l'agent par le refus
  d'une clé inconnue »), `:3219` (« l'en-tête précise… », « est rendu marqué comme tel »), `:3223` (« Kesh sert le
  carnet d'adresses », « il sert la liste des pièces importées »), `:3231` (« l'agent le relit en appelant l'outil… :
  Kesh lui rend le contenu validé »), `:3243` (« la phrase de Kesh qui précède le texte dit qu'il a été modifié »),
  `:3256` (« Kesh rend le code, jamais le caractère »), `:3309` (« l'appel rend une erreur »), `:3319` (« l'outil rend
  une erreur »), `:3324` (« ce que Kesh lui répond »), `:3348` (« Kesh ne lui rend aucune donnée » ; réserve harmonisée
  « à confirmer avec Claude Code sur la 0.16.1 »), `:3350` (« Kesh lui rend », « Kesh rend « en attente » », « que ces
  deux blocs atteignent l'agent ») ; `user-manual.tex:3261` (« Kesh ne rend pas tout de suite à l'agent ce qu'il
  demande ») ; `CHANGELOG.md:15` (réserve harmonisée). Laissées : les réponses du transport (« reçoit `403` »,
  « reçoit `202` »…), qui ne sont pas des réceptions par le modèle, et le libellé d'écran « Envoi n° 1 validé : l'agent
  peut le relire. ».

  Gates réellement exécutés : gate ciblé `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`,
  `nextest binary(mcp_contrat) | binary(mcp_outils_e2e)` 43/43 (`v0161-r2-cible1.log`) ; au dernier commit de code
  (`f6b338a9`), base `kesh_v0161` reconstruite, `scripts/test-fast.sh --ci` sous verrou : 4460 exécutés, 4460 passés, 7
  ignorés (`v0161-r2-gate-backend.log`) ; `npm run check` (0 erreur, 29 avertissements) et `lint-i18n-ownership` PASS
  (`v0161-r2-gate-frontend.log`) ; `make fr` sous verrou, deux PDF stables en deux passes (`v0161-r2-make-fr.log`) ; PDF
  aplatis (`grep -oF` des phrases réécrites : chacune ×1 ; « sous toute clé », « l’agent relira », « l’agent
  apprend », « l’agent relit ensuite » ×0 ; « non éprouvé » ×8, « observé avec Claude Code 2.1.312 » ×6, « à
  confirmer avec Claude Code sur la 0.16.1 » ×4 au PDF d'administration, ×1 au PDF utilisateur) ; `pdftotext -bbox`
  (`v0161-r2-bbox-{admin,user}-manual.txt`) : pages touchées 95, 123, 127-131, 135, 137 (administration) et 106
  (utilisateur) — au-delà de x = 524, p. 129 « l’Administrateur. » et p. 135 trois mots, lignes non touchées, comme au
  relevé R1 ; p. 106 aucun ; E2E complet (base `kesh_e2e_v0161` reconstruite, dossiers neufs `/tmp/kesh-e2e-v0161-r2-1/`, port
  3161, `v0161-e2e-r2-1.log`) : 325 passés, 7 échecs, 17 ignorés — les sept de la KF-029 (#97) de `docs/testing.md`
  § *Les échecs attendus* (`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`,
  `:77`, `:150`), aucun autre ; `mcp-sends.spec.ts:149` et `:216` verts ; backend arrêté par son PID.
  Prompt de la revue P3 : `v016-1-review-prompt-p3.md` (Sonnet, passe ciblée, une lentille, rapport
  `kesh-gate-logs/v0161-review-p3.md`).
