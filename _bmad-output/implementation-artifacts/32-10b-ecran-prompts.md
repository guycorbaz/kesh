# Story 32.10b : l'écran des prompts du serveur MCP

Status: ready-for-dev

*Fiche rédigée le 2026-10-27, en autonomie (Opus 5.5), worktree `kesh-32-10b`, branche `story/32-10b` créée depuis la
branche de fiche `story/32-10a`, puis `origin/main` fusionné au commit `ed981c7d` (la 32-8 complète, 32-8c comprise ;
merge `42680725`). Registre : `epic-32-choix-autonomes-3210b.md` (C-32-10b-1 à C-32-10b-16). Clé courte : `3210b`.
Issue de l'epic : #723 (CR « Serveur MCP intégré à Kesh »), en `refs` seulement. Les numéros de ligne du dépôt sont
ceux de `42680725` ; ceux de la fiche 32-10a (`32-10a-prompts-en-base.md`, validée, **non développée**) et de la fiche
32-7 (worktree `kesh-32-7`, commit `7e38f92d`, lue en lecture seule) sont cités d'après ces fiches.*

⛔ **Développement APRÈS la fusion des trois parties de la 32-10a** (i, ii, iii). Cette fiche se spécifie contre **la
fiche** de la 32-10a : tout ce qu'elle en reprend — routes de gestion, champs des réponses, codes d'erreur, noms des
clés de messages du serveur, règles d'approbation — **se relit au code à l'étape 0** (§ *À recontrôler à l'étape 0*),
ligne lue citée au Change Log ; une phrase de cette fiche que le code ne confirme pas **se réécrit selon le code**.

⚠️ **Une demande d'amendement de la 32-10a-iii conditionne la forme de l'AC-5** (C-32-10b-1, demande D1 à
l'orchestrateur, question 1 pour Guy) : que la route `GET /api/v1/admin/mcp/prompts/{id}` rende, pour chaque version de
nature *texte*, **le texte et la description tels que `prompts/get` les servirait**, marque comprise, calculés par la
**même fonction** que le catalogue. Si l'amendement n'est pas fait avant le développement, l'AC-5 suit sa variante B,
écrite (recomposition côté écran, gardée contre la source Rust).

## Story

En tant que Guy, Administrateur d'une installation où un agent lira mes livres, je veux, dans **Administration →
Prompts du serveur MCP**, voir chaque prompt et **tout son historique**, lire **exactement** le texte que l'agent
recevra — marque d'un prompt d'agent et caractères invisibles compris —, approuver ou refuser une version proposée par
un agent **sur ce que j'ai lu**, écrire ma propre version, retirer un prompt ou revenir au texte standard, sans que
rien ne soit jamais écrasé — afin qu'aucune instruction ne parvienne à un agent sans que je l'aie vue telle qu'elle
part.

## Origine

- **Kickoff** (`planning-artifacts/epic-32-serveur-mcp.md`) : décisions du Project Lead (l. 37-41 : prompts en base
  avec historique, l'agent écrit par une portée dédiée, son prompt marqué « écrit par un agent » et « à valider »
  jusqu'à l'approbation de l'administrateur) ; § *32-10* (l. 498-516) : « **32-10b — l'écran des prompts** : liste,
  édition (nouvelle version), historique, approbation ou refus d'une version d'agent, retour au standard. Modules :
  frontend, `kesh-i18n`, manuel d'administration — **3**. **AC** : E2E approbation et refus ; captures. **Risque** :
  faible à moyen. » ; risque de la 32-10a (l. 509-511) : « un prompt d'agent approuvé par mégarde en est un autre » ;
  *Réponses de Guy (2026-10-25)*, point *32-10a* (standards en français, aucun prompt `kesh-*` proposé par un agent,
  plafond de vingt).
- **Fiche 32-10a** (validée en cinq passes, `ready-for-dev`) : AC-i6 (version en vigueur, l. 333-337), AC-i7
  (approuver la dernière seulement, états terminaux, l. 338-343), AC-i8 (noms, préfixe `kesh` réservé, l. 344-358),
  AC-ii2 (validation d'un contenu, l. 635-640), AC-ii7 (la marque, l. 703-710), AC-iii6 (les sept routes, l. 899-907),
  AC-iii7 (Administrateur seulement, l. 908-913), AC-iii8 (400 à messages traduits, l. 914-916), AC-iii11 (approuvée
  par mégarde, l. 935-941) ; § *L'inventaire clos*, ligne **P26** (l. 991 : « écrit au manuel par la 32-10b ») ;
  § *La frontière avec la 32-10b* (l. 998-1011 : « la 32-10b n'ajoute aucune route ») ; § *Le manuel*, « Ce qu'elle
  laisse à la 32-10b » (l. 1379-1386) ; § *Dev Notes — La marque* (l. 1276-1284) et *Approuvée par mégarde*
  (l. 1286-1297).
- **Registre 3210a** : C-32-10a-14 (la marque), C-32-10a-16 (portée `books:read`), C-32-10a-19 (« l'écran pré-remplit,
  l'administrateur enregistre » : pas de route de copie), C-32-10a-20 (pas de « désapprobation »), C-32-10a-27.
- **Le modèle voisin, fusionné : la 32-8c** (`frontend/src/lib/features/mcp-sends/`, fiche `32-8-validation-avant-envoi.md`
  partie C, AC-C1 à AC-C11, l. 872-958) — la décision « tel quel » sur la version **lue**, l'écran en texte seul,
  la garde des replis, la recette visuelle, la sous-section du manuel sous *Sécurité*. Registre 328 : C-32-8-5,
  **C-32-8-20** (« aucun contrôle du prédicat côté client » ; réversible : « un signalement côté client peut s'ajouter,
  à titre d'aide »), C-32-8-21, C-32-8-71.
- **Fiche 32-7** (lecture seule, non fusionnée) : AC-2 (entrée `nav-admin-mcp` **avant** `nav-admin-mcp-sends`,
  l. 218-221), AC-14 (a) et (d) (G13 étendue par préfixe, `FAMILLES_RESOLUES['nav-']`, l. 529-586), AC-15 G-a (son
  périmètre exclut `routes/(app)/admin/mcp/prompts/`, l. 590-594), AC-18 (9) (« si elle existe à l'étape 0, la section
  *Les prompts du serveur MCP* (32-10b) » rangée sous `sec:mcp`, l. 858-862), AC-20 (« le compte "N des M routes à verbe
  mutant" — **32-10b**, qui le recompte après ses routes », l. 869-871).
- **Consignes de l'orchestrateur** (2026-10-27) : les points à traiter de ce prompt (texte servi exact et caractères
  cachés visibles, marque, retour au standard sans écrasement, rôles, recette visuelle, E2E dont la valeur servie
  égale à la valeur vue, manuel, replis i18n et #758, trois modules au plus).

## Ce que le code dit (relevé au code le 2026-10-27, `42680725`)

Chaque fait a été lu ; la ligne ou la commande est citée.

1. **Le menu d'administration** : groupe `adminOnly` de `frontend/src/routes/(app)/+layout.svelte:147-176` ; dernière
   entrée `{ i18nKey: 'nav-admin-mcp-sends', fallback: 'Envois du serveur MCP', href: '/admin/mcp/sends' }` (l. 171-175,
   clé l. 172). Les entrées sont des **données** résolues par `i18nMsg(item.i18nKey, item.fallback)` (`getItemLabel`,
   l. 180-185).
2. **L'écran voisin** : `frontend/src/lib/features/mcp-sends/` — `mcp-sends.api.ts` (61 lignes ; fabriques de corps
   `corpsTelQuel` l. 36, `corpsModifie` l. 45, `corpsRefus` l. 50, seules à construire un corps ; `decideMcpSend` l. 59),
   `mcp-sends.types.ts`, `mcp-sends-format.ts` (`dateLocale`, `etatAffiche`, `erreurDeDecision` : 409
   `ILLEGAL_STATE_TRANSITION` → message du serveur, 409 `OPTIMISTIC_LOCK_CONFLICT` → message de l'écran, *Recharger* ;
   404 ; 400 → message du serveur), `McpSendsList.svelte`, `McpSendDetail.svelte` (378 lignes ; `textarea` l. 220-227,
   `dir="ltr"`, `bind:value={valeur}` ; résumé en `<p … whitespace-pre-wrap … dir="ltr">` l. 202 ; dialogue de refus
   `Dialog.Root` ; `data-testid` préfixés `mcp-send-`). Pages minces `routes/(app)/admin/mcp/sends/+page.svelte` (11
   lignes) et `[id]/+page.svelte` (16 lignes, `{#key id}`), **sans** `+page.ts` : un non-administrateur reçoit le 403 de
   l'API, affiché.
3. **La garde des replis de la 32-8c** (`mcp-sends-i18n.test.ts`, `// @vitest-environment node`) : `PREFIXE =
   'mcp-sends-'` (l. 46), `DOSSIERS_ECRAN` (l. 47), `NOMBRE_DE_CLES = 48` en dur (l. 59, anti-test-muet),
   `ecartsAuCatalogue` **pure** (l. 101), égalité **dans les deux sens** entre clés relevées et clés du catalogue fr-CH,
   appartenance par `startsWith` ; l'entrée de menu lue à part dans `+layout.svelte` ; `ecran_sans_html_brut` (l. 219),
   témoins vert (`ReminderHistory.svelte:51`) et rouge factice ; `textarea` à `dir="ltr"` et `spellcheck="false"`.
   Son en-tête : « Aucune garde générale du dépôt ne tient un repli égal à sa valeur du catalogue (#758) ».
4. **#758** (`gh issue view 758`, lecture) : « i18n : aucun garde ne tient le repli d'une clé égal à sa valeur fr-CH
   (hors 13 clés) », **OPEN**. G13 (`frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts`) tient une liste
   fermée `REPLIS_A_SITE_UNIQUE` (l. 216) ; `PREFIXES_REPLI_EGAL_AU_CATALOGUE` n'existe pas encore (`grep -n
   PREFIXES_REPLI` → rien) — la 32-7 l'introduit (C-32-7-21), non fusionnée.
5. **Le compte des sites i18n** : `frontend/src/lib/shared/i18n-keys.test.ts` — `sitesTotal: 2393` (l. 732, généalogie
   de la 32-8c l. 724-731) ; `FAMILLES_RESOLUES` (l. 866) porte `'admin-mcp-sends'` (l. 876-877, ajouté par la 32-8c).
   G3-bis (`crates/kesh-i18n/src/loader.rs`, `ENTREES_ADMINISTRATION`, dernière entrée `nav-admin-restore` l. 1106)
   ne porte pas `nav-admin-mcp-sends` : aucun message de catalogue n'écrit « Administration → Envois… ».
6. **Le prédicat des caractères cachés** (32-8b0, `crates/kesh-core/src/text.rs`) : `is_hidden_format` (l. 86-105) =
   catégories `Format`, `PrivateUse`, `Unassigned` **ou** deux listes écrites (l. 94-104 : U+115F, U+1160, U+3164,
   U+FFA0 ; U+034F ; U+17B4, U+17B5 ; U+180B–U+180D, U+180F ; U+FE00–U+FE0F ; U+E0100–U+E01EF ; U+2028, U+2029) ;
   `is_hidden_in_text` (l. 119 : Cc sauf `\n` et `\t`, ou `is_hidden_format` — le prédicat du **texte** d'un prompt,
   fiche 32-10a, note d'étape 0) ; `is_hidden_in_line` (l. 131 : tout Cc, ou `is_hidden_format` — celui de la
   **description**). Angle mort écrit (l. 79-85) : U+2800, U+1D159, U+16FE4, les `Zs` U+1680, U+2000–U+200A, U+202F,
   U+205F, U+3000 restent admis. Classification d'**Unicode 17.0.0** ; Node du poste rend la même version
   (`node -e 'console.log(process.versions.unicode)'` → `17.0`).
7. **Le manuel d'administration** (`docs/manual/fr/admin-manual.tex`) : `\section{Sécurité}` l. 2863 ;
   `\subsection{Les envois du serveur MCP}\label{sec:mcp-envois}` l. 2928, **provisoirement sous *Sécurité*** (32-8c,
   la section `sec:mcp` de la 32-7 n'existant pas) ; la phrase « 122 des 128 routes à verbe mutant au moment de la
   rédaction » et « 122 + 4 + 2 = 128 » l. 2978 ; `\section{Monitoring et logs}` l. 3060. **Aucune** occurrence de
   « prompt » (`grep -ci prompt docs/manual/fr/admin-manual.tex` → 0).
8. **Le registre des routes** (`crates/kesh-api/tests/audit_route_registry.rs`) : partition `traced + exempt +
   no_matter == LIB_ROUTES.len()` (l. 899) et `assert_eq!(tout.len(), 133)` (l. 1265) — la phrase du manuel (fait 7)
   n'est gardée par aucun test (`grep -rn '128' crates/kesh-api/tests/textes_coherents.rs` → rien).
9. **L'E2E voisin** (`frontend/tests/e2e/mcp-sends.spec.ts`, 206 lignes) : `login` (l. 36), `preparer` (l. 45 : `PUT
   /api/v1/admin/mcp/settings` puis `POST /api/v1/admin/mcp/keys` `{ name, scopes }`), `clientMcp` (l. 77 : contexte
   **neuf**, sans cookie, `Authorization: Bearer`, `MCP-Protocol-Version: 2025-11-25`), `appeler` (l. 90 :
   `tools/call`) ; la propriété éprouvée : la valeur **lue dans la page** juste avant le clic est celle que relit le
   client MCP (l. 127-185) ; `seedTestState('with-company')` en `beforeAll`.
10. **Le registre de sprint** : `32-10b-ecran-prompts: backlog` (`sprint-status.yaml:832`) ; `32-10a-i`, `-ii`, `-iii`
    `ready-for-dev` (l. 829-831) ; `32-7-ecran-serveur-mcp: backlog` (l. 820) ; `32-9a`, `32-9b` `backlog`.
11. **Les captures de la 32-8c** vivent hors du dépôt : `/home/gcorbaz/devel/kesh-gate-logs/328c-captures/` (`ls`).

## Ce que dit la fiche 32-10a, et que cette fiche consomme (à recompter à l'étape 0)

| Élément | Fiche 32-10a | Ce que l'écran en fait |
|---|---|---|
| `GET /api/v1/admin/mcp/prompts` : nom, standard, version en vigueur (numéro, origine, nature), nombre à valider, « le standard du code diffère », `version` | AC-iii6 ; dépôt `synthese_par_societe` (AC-i5) | la liste (AC-3) |
| `GET /…/{id}` : le prompt et **tout** son historique, textes compris | AC-iii6 ; `historique` | le détail (AC-4 à AC-8) |
| `POST` (nom, description, texte → 201) | AC-iii6, AC-iii8 | la création (AC-11) |
| `POST /{id}/versions` (texte **ou** retrait, `version` attendue → 201) | AC-iii6 | l'édition, le retrait (AC-9) |
| `POST /{id}/versions/{numero}/approve` et `…/refuse` (`version` attendue → 200) | AC-iii6, AC-i7 | la décision (AC-7) |
| `POST /{id}/standard` (`version` attendue → 201) | AC-iii6, AC-iii11 (d) | le retour au standard (AC-10) |
| 404 autre société ; 409 verrou périmé, version non la dernière, déjà décidée, standard inconnu ; 400 validation (messages `t(…)`, quatre locales) | AC-iii6, AC-iii8 | les erreurs (AC-12) |
| Comptable 403, PAT 403, clé MCP 401 | AC-iii7 | les rôles (AC-14) |
| la marque d'un prompt d'agent, en tête de la description **et** du texte servis, suivie d'une ligne vide | AC-ii7, *Dev Notes — La marque* | le texte servi (AC-5) |
| approbation : version `agent` `pending` **et la dernière** ; refus : `agent` `pending`, dernière ou non ; états terminaux | AC-i7 | les boutons (AC-7) |
| servi : `books:read` (C-32-10a-16) ; `prompts/get` d'un prompt non servi → `-32602` à texte fixe | AC-ii5, AC-ii6 | l'E2E (AC-16) |

La 32-10b **n'ajoute aucune route** et **ne touche aucun code Rust** (§ *La frontière*, fiche 32-10a l. 1010-1011) ;
la seule demande au serveur est l'amendement D1 (C-32-10b-1), qui se fait **dans la 32-10a-iii**.

## Le découpage

Convention des fiches 32-5, 32-6, 32-8, 32-10a : un crate pour `kesh-i18n` et la documentation ; le frontend compte
pour un.

| Module | Ce que la story y touche |
|---|---|
| **frontend** | `src/lib/features/mcp-prompts/` (neuf), `src/routes/(app)/admin/mcp/prompts/` (neuf), `src/routes/(app)/+layout.svelte` (une entrée), `src/lib/shared/i18n-keys.test.ts` (`sitesTotal`, `FAMILLES_RESOLUES`), la garde des replis (selon l'étape 0 : `i18n-repli-divergent-actif.test.ts` ou un module partagé, AC-15), `tests/e2e/mcp-prompts.spec.ts` (neuf) |
| **`kesh-i18n`** | `locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` (clés `mcp-prompts-*`, `nav-admin-mcp-prompts`) |
| **docs** | `docs/manual/fr/admin-manual.{tex,pdf}` (la section, le compte des routes), `CHANGELOG.md` |

**Trois modules**, conformes au kickoff. Aucun découpage. ⚠️ Si l'étape 0 constate que la variante B de l'AC-5 est
nécessaire (amendement D1 non fait), elle reste dans le frontend (la garde lit une source Rust **sans la modifier**) :
toujours trois modules. Si au contraire le développement découvre qu'il lui faut une route ou un champ que la 32-10a-iii
ne rend pas, c'est un **écart déclaré** : il touche `kesh-api/routes` et l'audit — quatre modules ou plus — et revient à
l'orchestrateur avant d'écrire une ligne (§ *La frontière*, fiche 32-10a).

## La conception, en une page

**Ce qui doit être vrai, et pourquoi c'est le risque de la story.** Un prompt est une **instruction** donnée à un
agent qui lit des livres. Le danger n'est pas l'écran qui plante : c'est l'Administrateur qui **approuve un texte qu'il
n'a pas vu tel qu'il part** — parce qu'une marque manquait, qu'un caractère invisible portait une consigne cachée,
qu'une proposition plus récente est arrivée entre la lecture et le clic, ou qu'un bouton *Approuver* était au mauvais
endroit. D'où quatre règles, chacune tenue par un test :

1. **On montre le texte servi, non le texte stocké** (AC-5). Pour une version d'agent, le texte servi commence par la
   marque ; l'écran montre la marque **dans** le texte, à sa place, et non comme un ornement à côté. La forme servie
   est calculée par le serveur, par la fonction du catalogue (D1) ; l'E2E prouve que `prompts/get` rend **exactement**
   ce que la page montrait avant l'approbation (AC-16).
2. **On rend visible ce qui ne se voit pas** (AC-6). Chaque caractère que le serveur refuserait, et quelques invisibles
   qu'il admet (son angle mort écrit), s'affiche comme une **pastille** `U+XXXX` à sa place ; rien n'est retiré, rien
   n'est corrigé ; le texte affiché se **reconstitue** à l'identique (test). L'écran **signale**, il ne **juge** pas :
   le serveur reste seul juge de ce qu'il accepte et de ce qu'il sert (C-32-8-20, appliqué).
3. **On approuve ce qu'on a vu** (AC-7). L'approbation porte le **numéro** de la version affichée et la **`version`**
   du prompt lue **avec** cet affichage — jamais une relecture : une proposition arrivée entre-temps rend la demande
   périmée (409, AC-i7 de la 32-10a). *Approuver* n'existe que sur la dernière version, si elle est une version
   d'agent à valider ; il ouvre un dialogue qui **restate** ce qui sera servi (nom, numéro, clé auteur, et que le
   texte portera la marque).
4. **Rien ne s'écrase** (AC-8 à AC-10). Chaque geste — nouvelle version, retrait, retour au standard, décision — ajoute
   ou décide une version ; l'historique entier reste lisible, et l'écran le dit au moment du geste.

## Critères d'acceptation

Conventions : « vert » s'entend du test nommé, puis du gate complet (§ *Gates*). Aucune donnée réelle : textes de
démonstration et pièges `PIEGE_*_3210B` seulement ; `~/kesh-test/` jamais ouvert. Les noms de champs des réponses sont
**indicatifs** et s'alignent à l'étape 0 sur les DTO réels de `routes/mcp_prompts.rs` (32-10a-iii), relus et cités.
⛔ **« Égal » veut dire égal octet pour octet** (règle des fiches 32-6 et 32-10a).

⛔ **Règle commune des gardes lexicales** (fiches 32-6, 32-10a) : pour chaque garde, la fiche prescrit la propriété,
le périmètre et les formes ; un motif écrit ici est un **exemple** qui porte sa mesure. Au développement : chaque motif
greppé sur le dépôt réel à l'étape 0 ; un témoin rouge et un témoin vert par garde (forme existante, ou factice **dite
telle**) ; commandes et sorties citées au Dev Agent Record.

### A — Les pages et l'API

1. **AC-1 — l'entrée de navigation et les pages** (C-32-10b-13). Entrée `adminOnly` `{ i18nKey: 'nav-admin-mcp-prompts', fallback:
   'Prompts du serveur MCP', href: '/admin/mcp/prompts' }`, **immédiatement après** `nav-admin-mcp-sends` (fait 1 ;
   place relue à l'étape 0 — si la 32-7 a fusionné, l'ordre est *Serveur MCP*, *Envois*, *Prompts*). Pages minces
   `frontend/src/routes/(app)/admin/mcp/prompts/+page.svelte` (liste et création) et `…/prompts/[id]/+page.svelte`
   (détail, `{#key id}`), qui montent les composants de `frontend/src/lib/features/mcp-prompts/` (neuf :
   `mcp-prompts.api.ts`, `mcp-prompts.types.ts`, `mcp-prompts-format.ts`, `caracteres-caches.ts`,
   `McpPromptsList.svelte`, `McpPromptDetail.svelte`, `McpPromptVersion.svelte`, `McpPromptEditor.svelte`, leurs
   tests). Motif de la 32-8c (fait 2) : **pas** de `+page.ts` ; un non-administrateur qui saisit l'adresse reçoit le 403
   de l'API, affiché (C-32-10b-9). `'admin-mcp-prompts'` entre à `FAMILLES_RESOLUES['nav-']` (fait 5), avec un
   commentaire daté.
2. **AC-2 — la couche d'API** (`mcp-prompts.api.ts`, en-tête sur le patron de `mcp-sends.api.ts:1-11`). Fonctions :
   `listMcpPrompts()`, `getMcpPrompt(id)`, `createMcpPrompt({ nom, description, texte })`,
   `addMcpPromptVersion(id, corps)`, `approveMcpPromptVersion(id, numero, version)`,
   `refuseMcpPromptVersion(id, numero, version)`, `restoreMcpPromptStandard(id, version)`. **Les corps sont construits
   par des fabriques pures, seules à le faire** (patron fait 2) : `corpsTexte(version, description, texte)`,
   `corpsRetrait(version)`, `corpsDecision(version)`, `corpsStandard(version)`. ⛔ Les valeurs partent **exactes** — ni
   `trim`, ni normalisation des fins de ligne, ni substitution — : le serveur normalise et valide (AC-ii2 de la
   32-10a). Tests (`mcp-prompts.api.test.ts`, `apiClient` simulé) : chaque fonction appelle le chemin et la méthode
   attendus avec le corps **égal** (`toEqual`) ; `corps_texte_exact` (description à espace de tête, texte à tabulation,
   ligne vide finale, `\r\n`, `\u{202E}` littéral → corps identique) ; `corps_retrait_sans_texte` (aucun champ de texte).
3. **AC-3 — la liste.** Un tableau : nom (police à chasse fixe), *Standard* (pastille pour un prompt `is_standard`),
   **version en vigueur** — « n° N », origine (*Standard*, *Administrateur*, *Agent* — cette dernière en pastille
   d'avertissement, la même que dans l'historique, AC-8), et *Retiré* pour un retrait —, **ou** « Non servi » quand
   aucune version approuvée n'existe (un prompt dont seule une proposition attend) ; le **nombre à valider** (pastille
   « N à valider », absente à zéro) ; *Le standard a changé* quand le drapeau « le standard du code diffère » est levé ;
   un lien vers le détail. Ordre du serveur (par nom). Bouton *Actualiser*. **Aucun texte ni description** n'y paraît
   (la liste ne lit que la synthèse). Un bouton *Nouveau prompt* ouvre le formulaire de l'AC-11. Tests
   (`McpPromptsList.test.ts`) : rendu de chacun des cas (standard servi, personnalisé par l'administrateur, version
   d'agent en vigueur, retiré, non servi, deux à valider, standard changé) par `data-testid` ; liste vide (« Aucun
   prompt » — n'arrive qu'en cas d'échec de l'amorçage).
4. **AC-4 — le détail : l'en-tête et ce qui est servi aujourd'hui.** En-tête : nom, *Standard* le cas échéant, la
   version en vigueur (ou « Non servi »), la date de dernière modification. Puis un bloc **Ce que l'agent reçoit
   aujourd'hui** : la description servie et le texte servi de la version en vigueur (AC-5), ou, pour un prompt non
   servi, la phrase « Ce prompt n'est servi à aucun agent » et sa raison (« retiré » ; « aucune version approuvée »).
   Puis les **versions à valider** (AC-7), puis l'**historique** (AC-8), puis les gestes (AC-9, AC-10). Avis
   permanent en tête (`role="note"`) : « Un prompt est une consigne donnée à l'agent. Le texte ci-dessous est
   exactement celui qu'il reçoit : la marque d'un prompt écrit par un agent en fait partie, et chaque caractère
   invisible est montré par son code. »

### B — Ce qui est montré : le texte servi, la marque, les caractères cachés

5. **AC-5 — on montre le texte servi** (C-32-10b-1, C-32-10b-2).
   - **Variante A, la conception retenue (demande D1)** : pour chaque version de nature *texte*, la réponse de `GET
     /…/{id}` porte un objet `servi` — `{ description, texte, marque }` — où `description` et `texte` sont **exactement**
     ce que `prompts/get` (et `prompts/list` pour la description) rendraient si cette version était en vigueur, marque
     comprise, et `marque` la marque seule (nulle pour une version `standard` ou `admin`). Le serveur les calcule par
     **la même fonction** que le catalogue (AC-ii4, AC-ii7 de la 32-10a) : aucun texte servi n'est recomposé côté
     écran. L'écran affiche `servi.description` et `servi.texte` (rendus par l'AC-6) ; quand `servi.marque` n'est pas
     nulle **et** que `servi.texte` commence par `servi.marque + "\n\n"`, la portion de la marque est encadrée et
     précédée de la pastille *Écrit par un agent* — sinon le texte est montré tel quel, **sans** encadré (jamais
     masqué). Tests (`McpPromptVersion.test.ts`) : `apercu_servi_est_le_champ_servi` (le texte affiché, reconstitué,
     **égal** à `servi.texte` ; le champ `texte` stocké, différent, n'apparaît pas seul) ; `marque_encadree_en_tete`
     (version d'agent : l'élément `data-testid="mcp-prompt-mark"` porte la marque, il précède le reste du texte) ;
     `sans_marque_pour_standard_et_admin` ; `marque_incoherente_montree_telle_quelle` (un `servi.texte` qui ne commence
     pas par la marque : rendu entier, aucun encadré).
   - **Variante B (si D1 n'est pas fait à l'étape 0)** : `mcp-prompts-format.ts` porte `MARQUE_AGENT` (gabarit à deux
     entiers) et `texteServi(version)` = marque + `"\n\n"` + texte pour une version `agent`, le texte seul sinon ; idem
     pour la description. **Garde** `gabarit_de_marque_egal_au_serveur` (Vitest, `// @vitest-environment node`) : lit
     la source de `marque_agent` (`crates/kesh-api/src/mcp/textes.rs`, chemin relu à l'étape 0), en extrait le gabarit
     littéral et l'exige **égal** à celui de l'écran (patron de la garde `MCP_ENVOI_OCTETS_MAX`, fait 3) ; témoin rouge
     factice (un gabarit d'un caractère différent). L'E2E (AC-16) reste la preuve : il compare au `prompts/get` réel.
     La variante B se **déclare** au Change Log et au rapport final ; elle ajoute un angle mort (une fonction de
     composition modifiée côté serveur sans toucher au gabarit littéral : seule l'E2E le voit).
6. **AC-6 — les caractères cachés rendus visibles** (C-32-10b-3, C-32-10b-4). `caracteres-caches.ts`, fonctions
   **pures** :
   - `segmenter(texte, mode)` → `Segment[]`, où un segment est `{ type: 'texte', valeur }` ou `{ type: 'cache', valeur,
     code }` (`code` = `U+XXXX`, quatre à six chiffres hexadécimaux majuscules) ; `mode` ∈ {`texte`, `ligne`}.
   - **Signalé `refuse`** (ce que le serveur refuse, miroir de `kesh-core`, fait 6) : en mode `texte`, tout contrôle
     `\p{Cc}` **sauf** `\n` et `\t` ; en mode `ligne`, tout `\p{Cc}` ; dans les deux modes, `\p{gc=Cf}`, `\p{gc=Co}`,
     `\p{gc=Cn}` et les **deux listes écrites** de `is_hidden_format` (fait 6), recopiées en `LISTES_KESH_CORE`.
   - **Signalé `admis`** (ce que le serveur admet, mais qui ne se voit pas ou se confond — son angle mort écrit,
     `text.rs:79-85`) : en mode `texte`, la tabulation `\t` ; dans les deux modes, tout `\p{Zs}` **autre que**
     U+0020 (dont U+00A0 et U+202F), U+2800, U+1D159, U+16FE4.
   - `reconstituer(segments)` = la concaténation des `valeur` : **égale** au texte d'entrée pour toute entrée.
   - Rendu (`McpPromptVersion.svelte`) : un `<pre data-testid="mcp-prompt-served-text-{numero}"
     class="whitespace-pre-wrap break-words font-mono" dir="ltr">` dont chaque segment `texte` est une interpolation
     Svelte, et chaque segment `cache` un `<span data-cache={code} title=…>` affichant `⟦U+XXXX⟧` (le caractère
     lui-même **n'est pas** inséré : ce serait le cacher de nouveau). La pastille se distingue par son **style** (fond
     et bordure), non par son seul texte : un texte qui écrit `⟦U+202E⟧` en toutes lettres s'affiche comme du texte
     ordinaire et **n'entre pas** dans les comptes ci-dessous, qui ne comptent que des segments `cache` (test
     `pastille_imitee_non_comptee`). La description servie, d'une ligne, par le même
     rendu en mode `ligne`. Au-dessus du texte, un compte : « N caractère(s) invisible(s) refusé(s) par Kesh » (classe
     `refuse`, avertissement : « une version qui en contient n'est pas servie ; elle a pu être restaurée d'une
     sauvegarde ou écrite hors de Kesh ») et « N espace(s) spéciale(s) ou caractère(s) peu visible(s) » (classe
     `admis`, information). **Aucun bouton n'est désactivé par ce signalement** : le serveur reste seul juge
     (C-32-8-20 ; C-32-10b-4 ; demande D2).
   - Tests (`caracteres-caches.test.ts`) : `reconstitution_a_l_identique` (une liste d'entrées écrite et comptée :
     ASCII, accents, emoji simple U+26A0, emoji composé `⚠️`, `👨‍💻`, `\r\n`, U+202E, U+2066, U+E0041, U+E0100,
     U+0600, U+13430, U+0378, U+00A0, U+202F, U+2800, `\t`, texte vide) ; un test **par famille** (Cc, Cf, Co, Cn,
     chaque point et chaque borne des deux listes, chaque `admis`) ; `n_et_t_admis_en_mode_texte`,
     `n_et_t_signales_en_mode_ligne` ; `lettres_et_signes_voisins_non_signales` (U+1161, U+17B6, U+180A, U+FE10 —
     les voisins visibles de la fiche 32-10a, AC-ii1) ; `emoji_simple_non_signale`.
   - **Garde `listes_egales_a_kesh_core`** (Vitest, `// @vitest-environment node`, patron de la garde de
     `MCP_ENVOI_OCTETS_MAX`, fait 3) — *propriété* : `LISTES_KESH_CORE` est **égale**, point pour point et borne pour
     borne, aux deux listes du `matches!` de `is_hidden_format` lues dans `crates/kesh-core/src/text.rs` (formes
     `'\u{XXXX}'` et `'\u{XXXX}'..='\u{YYYY}'`, commentaires `//` retirés ; le bloc délimité par la ligne `pub fn
     is_hidden_format` et la fin de la fonction). *Mesure* : `sed -n 86,105p crates/kesh-core/src/text.rs | grep -oE
     "'\\\\u\{[0-9A-F]+\}'" | wc -l` → **16** jetons au relevé (10 points isolés et 3 plages, chacune deux jetons — à
     recompter à l'étape 0, sortie citée). *Anti-test-muet* : le nombre de jetons lus est écrit en dur. *Témoin rouge*
     factice : une source où U+180F manque → la garde nomme U+180F. *Témoin vert* : la vraie source.
   - **Garde `version_unicode_du_moteur`** : `process.versions.unicode` (Node de Vitest) est **égal** à la version
     d'Unicode de `unicode-properties` lue dans `Cargo.lock` puis dans `tables.rs` (`UNICODE_VERSION`, chemin relu à
     l'étape 0 ; `17.0` au relevé, fait 6) — sans quoi les classes `Cf`/`Co`/`Cn` du test ne sont pas celles du
     serveur. Ce que la garde **ne voit pas** : le navigateur de l'Administrateur (ses tables peuvent différer :
     § *Ce que les gardes ne voient pas*).

### C — Les gestes

7. **AC-7 — la version d'agent à valider : on approuve ce qu'on a vu** (C-32-10b-5, C-32-10b-6).
   - Chaque version `agent` `pending` est montrée **en entier** — description et texte servis (AC-5, AC-6), jamais
     tronqués, jamais repliés, sans « voir plus » — avec : son numéro, la clé auteur (numéro et nom de la clé ;
     « révoquée » le cas échéant, si la réponse le rend), la date de la proposition, et la pastille *Écrit par un
     agent — à valider*.
   - **Approuver** n'est rendu que sur la version qui est **la dernière** du prompt (plus grand numéro de la réponse) et
     qui est `agent` `pending` (AC-i7 de la 32-10a). Une version à valider qui n'est plus la dernière porte la phrase
     « Une version plus récente existe : celle-ci ne peut plus être approuvée, seulement refusée » et le seul bouton
     *Refuser*.
   - *Approuver* ouvre un **dialogue de confirmation** qui restate : le nom du prompt, « version n° N, proposée par la
     clé n° K (nom) », « Ce texte sera servi à tout agent dont la clé porte `books:read`, **précédé de la marque** » ;
     les comptes de l'AC-6 s'ils ne sont pas nuls ; boutons *Annuler* et *Approuver la version n° N*.
   - ⛔ **Le corps part avec la `version` lue avec l'affichage** : la fonction d'approbation reçoit le `version` et le
     `numero` **de l'objet affiché** ; aucun `GET` entre l'affichage et le `POST`. Après la réponse (200), l'écran
     **relit** le détail (`GET`) et l'affiche — la relecture vient **après** la décision, jamais avant.
   - *Refuser* : dialogue de confirmation (variante destructive) — « Un refus est définitif ; la version reste à
     l'historique » — puis `POST …/refuse` avec la `version` lue.
   - Tests (`McpPromptDetail.test.ts`) : `approbation_porte_la_version_lue` (le `GET` simulé rend `version: 7` et la
     version d'agent n° 3 ; clic *Approuver*, confirmation → **un seul** `GET` avant le `POST`, `POST` sur `…/versions/3/
     approve` avec `{ version: 7 }`) ; `approbation_apres_confirmation` (sans confirmation, aucun `POST`) ;
     `approuver_seulement_la_derniere` (pending n° 3, standard n° 4 → pas de bouton *Approuver* sur 3, *Refuser* oui) ;
     `aucun_bouton_sur_une_version_decidee` ; `refus_apres_confirmation` ; `dialogue_restate_numero_cle_et_marque`.
8. **AC-8 — l'historique, jamais réécrit.** Un tableau de **toutes** les versions, plus récente en tête : numéro ;
   origine (*Standard* ; *Administrateur* — nom de l'auteur ; *Agent* — clé n° K, pastille d'avertissement) ; état
   (*Approuvée*, *À valider*, *Refusée*) ; nature (*Texte*, *Retrait*) ; la décision d'une version d'agent (par qui,
   quand) ; la date ; la pastille **En vigueur** sur la version en vigueur (champ du serveur si la réponse le rend,
   sinon la règle de l'AC-i6 — approuvée de plus grand numéro, de toute nature — appliquée par une fonction pure
   testée, C-32-10b-7). Chaque version de nature *texte* se **déplie** pour montrer son texte servi (AC-5, AC-6) — le
   repli de l'historique est permis, celui d'une version **à valider** ne l'est pas (AC-7). Action **Reprendre cette
   version** sur une version de nature *texte* : pré-remplit l'éditeur (AC-9) avec sa description et son texte
   **stockés** (C-32-10a-19 : « l'écran pré-remplit, l'administrateur enregistre »). Tests : chaque origine, état et
   nature rendus ; `en_vigueur_selon_la_regle` (une `pending` postérieure, une `refused` postérieure, un retrait en
   vigueur) ; `marque_visible_dans_l_historique` (la pastille *Agent* sur chaque version d'agent, décidée ou non).
9. **AC-9 — écrire une nouvelle version, ou retirer** (C-32-10b-8).
   - **Éditeur** (`McpPromptEditor.svelte`) : un champ *Description* (une ligne) et une `<textarea>` *Texte* en police à
     chasse fixe, `dir="ltr"`, `spellcheck="false"` ; pré-remplis par la version en vigueur si elle est un texte, ou par
     *Reprendre cette version* ; un compteur de caractères (« N / 16 000 » pour le texte, « N / 500 » pour la
     description — bornes de l'AC-i3 de la 32-10a, relues à l'étape 0, sans contrôle bloquant côté écran) ; un aperçu
     des caractères cachés **de la saisie** (le rendu de l'AC-6, sans marque : une version d'administrateur n'en porte
     pas). Bouton *Enregistrer comme nouvelle version* → `POST /{id}/versions` avec `corpsTexte(version lue,
     description, texte)`, valeurs **exactes**. Une phrase sous le bouton : « Une nouvelle version n° N+1 est créée ; les
     versions précédentes restent à l'historique. »
   - **P26, rendu visible** : quand l'éditeur est pré-rempli depuis une version `agent`, un avis (`role="note"`,
     `data-testid="mcp-prompt-editor-agent-origin"`) : « Ce texte a été écrit par un agent. L'enregistrer en fait **votre**
     version : elle sera servie **sans** la marque d'un prompt d'agent. Relisez-le comme s'il venait de vous. »
   - **Retirer** : bouton *Retirer ce prompt* (variante destructive), dialogue : « Le prompt cessera d'être servi. Une
     version n° N+1 de retrait est ajoutée ; l'historique reste. Une nouvelle version le servira de nouveau. » → `POST
     /{id}/versions` avec `corpsRetrait(version lue)`. Absent si la version en vigueur est déjà un retrait.
   - Tests (`McpPromptEditor.test.ts`) : `envoie_la_valeur_exacte` ; `avis_p26_si_reprise_d_une_version_d_agent` (et
     absent depuis une version d'administrateur ou standard) ; `retrait_apres_confirmation` ;
     `aucun_controle_bloquant_cote_ecran` (un texte de 16 001 caractères et un U+202E partent ; le 400 simulé est
     affiché).
10. **AC-10 — revenir au standard, sans rien écraser** (C-32-10b-10). Sur un prompt `is_standard` dont la version en
    vigueur n'est **pas** d'origine `standard` — personnalisé par l'administrateur, version d'agent approuvée, ou
    retrait d'administrateur — **ou** dont le drapeau « le standard du code diffère » est levé : bouton *Revenir au texte
    standard*. Dialogue : « Une nouvelle version n° N+1 reprend le texte standard de cette version de Kesh. Vos
    versions et celles de l'agent restent à l'historique, et vous pourrez reprendre l'une d'elles. Le prompt redevient
    *non personnalisé* : il suivra désormais le texte des prochaines versions de Kesh. » → `POST /{id}/standard` avec
    `corpsStandard(version lue)`. Après 201, relecture : la nouvelle version paraît en tête, d'origine *Standard*, en
    vigueur. Absent pour un prompt non standard (et pour un standard non personnalisé au texte à jour). Tests :
    `retour_au_standard_seulement_si_utile` (les quatre cas) ; `retour_au_standard_apres_confirmation`.
11. **AC-11 — créer un prompt** (C-32-10b-16). Formulaire *Nouveau prompt* (liste) : nom, description, texte. Aide sous le nom :
    « Lettres minuscules sans accent, chiffres, `-` et `_`, 64 au plus ; un nom ne commence pas par `kesh`, réservé aux
    prompts standard. » (règle de l'AC-i2 de la 32-10a, **non contrôlée côté écran** : le 400 du serveur est affiché).
    `POST` → 201 → navigation vers le détail du prompt créé. Tests : corps exact ; 400 affiché, saisies conservées.
12. **AC-12 — les erreurs** (patron de `erreurDeDecision`, fait 2 ; codes relus à l'étape 0). 409 (verrou périmé,
    version non la dernière, déjà décidée, standard inconnu du code) → le message (celui du serveur s'il est traduit,
    sinon « Ce prompt a changé depuis son affichage : rechargez-le avant d'agir ») **et** le bouton *Recharger*, qui
    relit le détail ; **aucune** relance automatique. 404 → « Prompt introuvable ». 400 → le message du serveur, saisies
    **conservées**. 403 → « Réservé à l'Administrateur ». Le reste → `errorMessageOf`. Tests par code ;
    `aucune_relance_automatique_sur_409`.
13. **AC-13 — un texte non fiable se montre comme du texte.** Textes, descriptions, noms de clé et marques ne passent
    que par une interpolation Svelte ou la valeur d'un champ — **jamais** `{@html`. Garde `ecran_sans_html_brut` (règle
    commune), sur le patron de la 32-8c (fait 3) — *propriété* : aucune source de `frontend/src/lib/features/
    mcp-prompts/` ni de `frontend/src/routes/(app)/admin/mcp/prompts/` ne contient `{@html` ; *mesure* : `grep -rn
    '{@html' frontend/src/lib frontend/src/routes` → **0** au relevé (à recompter) ; *témoin vert existant* :
    `{@const cancelled = r.cancelledAt !== null}` (`src/lib/features/reminders/ReminderHistory.svelte:51`) ; *témoin
    rouge factice* : `{@html version.texte}`. Et chaque `<textarea>` et `<pre>` de texte porte `dir="ltr"` ; chaque
    `<textarea>`, `spellcheck="false"`.
14. **AC-14 — les rôles.** Qui voit, qui décide (AC-iii7 de la 32-10a ; C-32-0-9) : **l'Administrateur seul**, en
    session, voit l'écran et y agit ; un Comptable ne voit ni l'entrée de menu ni le contenu (403 affiché) ; une clé
    d'API ou une clé MCP n'atteint aucune route (403, 401) — **l'agent ne peut jamais approuver**. L'écran n'offre
    **aucun** geste de plus que les routes (pas de « désapprouver » : C-32-10a-20). Tests : E2E (AC-16 (e)), Vitest
    (403 affiché).

### D — i18n, E2E, captures, manuel

15. **AC-15 — i18n, et le repli égal au catalogue par une garde réelle** (C-32-10b-11 ; #758). Clés `mcp-prompts-*` et
    `nav-admin-mcp-prompts` dans les **quatre** locales ; `npm run lint-i18n-ownership` vert sans entrée neuve à
    `KNOWN_VIOLATIONS` ; `i18n-keys.test.ts` vert, `sitesTotal` **recompté depuis la source** aux deux bornes (base de
    l'étape 0 → HEAD), généalogie écrite fichier par fichier ; `FAMILLES_RESOLUES['nav-']` gagne `'admin-mcp-prompts'`
    (AC-1). **Préfixe** : `mcp-prompts-` — **à moins** que la 32-10a-iii n'emploie déjà ce préfixe pour ses messages de
    serveur (relu à l'étape 0 : `grep -n '^mcp-prompts\?-' crates/kesh-i18n/locales/fr-CH/messages.ftl`) ; dans ce cas
    l'écran prend `mcp-prompts-ecran-` et le déclare (sans quoi l'égalité dans les deux sens rougirait sur les clés du
    serveur). **La garde**, selon l'état constaté à l'étape 0 :
    - **(a) la 32-7 a fusionné** avec `PREFIXES_REPLI_EGAL_AU_CATALOGUE` (C-32-7-21) : le préfixe de l'écran y entre ;
      son `toBe(N)` anti-test-muet suit ;
    - **(b) sinon** : le relevé et `ecartsAuCatalogue` de `mcp-sends-i18n.test.ts` (fait 3) **sortent** en un module
      partagé `frontend/src/lib/shared/i18n-repli-catalogue.ts` (fonctions pures, mêmes témoins), que la garde des
      envois **et** une garde neuve `mcp-prompts-i18n.test.ts` emploient — DRY, sans changer le comportement de la
      première (ses tests verts, son `NOMBRE_DE_CLES` inchangé) ; la neuve porte son `NOMBRE_DE_CLES` en dur, lit
      `nav-admin-mcp-prompts` dans `+layout.svelte` comme la 32-8c, et tient l'égalité dans les deux sens par
      `startsWith`.
    *Témoin vert* (factice) : un catalogue portant `audit-log-entity-mcp_prompt` n'ajoute rien à l'ensemble du préfixe.
    **Angle mort écrit, avec renvoi** : le reste du catalogue reste hors garde (#758).
16. **AC-16 — le parcours de bout en bout** (C-32-10b-14 ; `frontend/tests/e2e/mcp-prompts.spec.ts`, Playwright ; montage de
    `docs/testing.md` § *Prérequis Playwright local* ; `seedTestState('with-company')` en `beforeAll`, qui amorce les
    standards — AC-i11 de la 32-10a). Préparation par l'API d'administration (session) : serveur MCP activé, une clé
    `books:read` + `prompts:write` (patron `preparer`, fait 9). Le client MCP est un contexte **neuf**, sans cookie.
    Noms de prompt uniques (`e2e-3210b-<horodatage>`).
    - **(a) Approbation — la valeur servie est la valeur vue.** Le client appelle `proposer_prompt` (`nom`,
      `description`, `texte` à accents et à deux paragraphes, sans caractère caché) ; la page de la liste montre « 1 à
      valider » ; le détail montre la version à valider, **marque comprise** ; l'E2E lit dans la page, **avant** le clic,
      le texte servi `T` (`textContent` du `<pre data-testid="mcp-prompt-served-text-1">`, après avoir constaté
      **aucune** pastille `[data-cache]` dans ce bloc) et la description servie `D` ; *Approuver*, confirmation ; puis
      le client appelle `prompts/get` → `messages[0].content.text` **égal** à `T` et `description` **égale** à `D` ;
      `prompts/list` porte une entrée `{ name, description: D, arguments: [] }`. Et `T` commence par `[Prompt écrit par
      un agent` (la marque, relue à l'étape 0 dans `mcp/textes.rs`).
    - **(b) Refus.** Seconde proposition ; *Refuser*, confirmation ; l'historique montre *Refusée* ; `prompts/get` de ce
      nom → l'erreur `-32602` (corps égal à celle d'un nom inconnu, demandée dans le même test) ; `prompts/list` ne le
      porte pas.
    - **(c) Nouvelle version d'administrateur** sur le prompt de (a), par *Reprendre cette version* : l'avis P26 est
      visible ; le texte modifié au clavier ; *Enregistrer* ; le texte servi lu dans la page `T2` **ne commence pas**
      par la marque ; `prompts/get` → **égal** à `T2`.
    - **(d) Retour au standard** sur `kesh-assistant` : une version d'administrateur, puis *Revenir au texte standard* ;
      le texte servi lu `T3` est **égal** au `prompts/get` ; l'historique porte **trois** versions de plus (aucune
      n'a disparu).
    - **(e) Rôles** : un Comptable (préréglage relu à l'étape 0) ne voit pas l'entrée de menu ; `/admin/mcp/prompts`
      lui affiche le refus.
    Échecs attendus jugés **fichier par fichier** contre `docs/testing.md` § *Les échecs attendus*.
17. **AC-17 — la recette visuelle** (`CLAUDE.md` § *Recette visuelle* ; **thème clair seulement**), captures sur une
    base de démonstration du worktree (`kesh_3210b`), rangées hors du dépôt dans
    `/home/gcorbaz/devel/kesh-gate-logs/3210b-captures/`, jointes au Dev Agent Record et au prompt de la revue de code,
    **une par état** : (1) l'entrée de menu ; (2) la liste — les deux standards seuls ; (3) la liste avec une version
    à valider, un prompt retiré, un prompt non servi et *Le standard a changé* ; (4) le détail d'un standard servi ;
    (5) le détail avec une version d'agent à valider, **marque encadrée** ; (6) le dialogue d'approbation ; (7) le
    dialogue de refus ; (8) une version à valider qui n'est plus la dernière (*Refuser* seul) ; (9) l'historique après
    approbation, *En vigueur* et la pastille *Agent* ; (10) une version **piégée** — U+202E, U+200B, U+E0041 et une
    espace insécable, posés par un `INSERT` SQL dans `kesh_3210b` (le seul chemin : le serveur les refuse à
    l'écriture) — rendue en pastilles, avec les deux comptes ; (11) l'éditeur pré-rempli depuis une version d'agent,
    **avis P26** ; (12) le 400 d'une saisie à caractère invisible (message du serveur) ; (13) le 409 (verrou périmé),
    *Recharger* ; (14) le dialogue de retrait et le prompt retiré (« n'est servi à aucun agent ») ; (15) le dialogue
    de retour au standard et son résultat ; (16) le formulaire *Nouveau prompt* et son 400 sur un nom en `kesh` ; (17)
    le refus affiché à un Comptable.
18. **AC-18 — la section du manuel d'administration** (C-32-10b-12). `\subsection{Les prompts du serveur
    MCP}\label{sec:mcp-prompts}`, placée **après** `sec:mcp-envois` — sous la section *Le serveur MCP* (`sec:mcp`) de la
    32-7 si elle a fusionné, **sinon sous *Sécurité*, provisoirement**, comme la 32-8c (fait 7), la 32-7 l'y rangeant
    (sa fiche, AC-18 (9)). Elle dit, **tel que le code le fait** (chaque affirmation confrontée au code à l'étape 0,
    ligne citée au Dev Agent Record) :
    - ce qu'est un prompt du serveur MCP (une consigne que l'utilisateur **choisit** de donner à l'agent dans son
      client) ; qu'il n'ouvre **aucun** droit (portées de la clé, lecture seule, validation avant envoi) ;
    - les deux **prompts standard** (`kesh-developpement`, `kesh-assistant`), installés d'office, en français ; qu'ils
      suivent les versions de Kesh **tant qu'ils ne sont pas personnalisés** ; ce qu'est « personnalisé » ; *Le
      standard a changé* ; le **retour au standard** et ce qu'il fait de l'historique (rien n'est effacé) ;
    - l'**écran** (*Administration → Prompts du serveur MCP*, `\keshpath{/admin/mcp/prompts}`), réservé à
      l'Administrateur : la liste, le détail, « ce que l'agent reçoit aujourd'hui » ;
    - l'**historique** : chaque modification crée une version, aucune n'est effacée ni réécrite ; la version en
      vigueur ; le **retrait** ;
    - les **propositions d'un agent** : une clé portant `prompts:write` propose ; la version attend, n'est **jamais**
      servie avant l'approbation ; **aucune clé ne peut approuver** ; un agent ne propose jamais sur un nom en `kesh`
      (C-32-10a-27) ; le **plafond de vingt** propositions en attente par société ;
    - **approuver et refuser** : seule la dernière version s'approuve ; l'approbation porte sur la version **lue** (une
      proposition arrivée entre-temps la rend périmée, *Recharger*) ; le refus est définitif ;
    - **la marque** : son texte, sa place en tête de la description et du texte servis, et qu'elle dit **qui a écrit la
      version, non d'où viennent ses mots** ; l'**angle mort P26** : enregistrer un texte d'agent comme sa propre
      version **ôte la marque** — l'écran le dit, le manuel aussi ;
    - **les caractères invisibles** : refusés à l'écriture et non servis s'ils ont été restaurés ou écrits hors de
      Kesh ; montrés à l'écran par leur code, dans une pastille, et comptés — un code écrit en toutes lettres dans le
      texte n'est ni une pastille ni compté : les comptes font foi ; les espaces spéciales montrées aussi, quoique admises ; que le
      signalement de l'écran dépend des tables Unicode du navigateur, le serveur restant seul juge ;
    - **une version approuvée par mégarde** : ce qui ne se fait pas (la désapprouver, l'effacer) et ce qui se fait
      (nouvelle version, retrait, retour au standard), chaque geste au journal d'audit ;
    - **les fenêtres** : après une restauration, les prompts sont ceux de la sauvegarde, et un standard non personnalisé
      reprend le texte de la version installée au **démarrage suivant** ; après la remise à zéro, les standards sont
      réinstallés aussitôt (la 32-10a-i l'écrit déjà au paragraphe de la remise à zéro : y renvoyer, ne pas le
      redire) ;
    - un renvoi à `sec:mcp-envois` et, si elle existe, à `sec:mcp-cles` (portée `prompts:write`).
    **Aucune donnée réelle** ; exemples factices. PDF régénéré (`make fr` dans `docs/manual/`) et contrôlé **aplati** :
    `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE '.{60}(Les prompts du serveur
    MCP|ôte la marque|Revenir au texte standard).{80}'`, sortie citée.
19. **AC-19 — le compte des routes mutantes et le CHANGELOG** (fiche 32-10a l. 1008 ; fiche 32-7 AC-20).
    `admin-manual.tex:2978` — « 122 des 128 routes à verbe mutant », « 122 + 4 + 2 = 128 » (fait 7) — **recompté
    depuis le registre** (`audit_route_registry.rs`, partition l. 899, relue après la 32-10a-iii, qui ajoute cinq routes
    mutantes tracées), commande et sortie citées, généalogie au Dev Agent Record ; si les nombres sont déjà faux avant la
    32-10a-iii (la 32-8b a ajouté une route mutante), le Dev Agent Record le dit et le corrige au compte réel. Le même
    recompte dans le PDF aplati. **CHANGELOG** : à l'entrée `Ajouté` de la version en cours, « l'écran des prompts du
    serveur MCP (Administration → Prompts du serveur MCP) : historique, approbation ou refus des propositions d'un agent,
    retour au texte standard » — sans date (la release la pose).
20. **AC-20 — ce que la 32-10b ne touche pas.** Aucune route, aucun code Rust, aucune migration, aucun texte servi à un
    agent. Le manuel utilisateur, `docs/api-external.md`, `README.md:233` et `website/roadmap.html:375` sont à la 32-11
    (fiche 32-10a l. 1390-1397). La section *Le serveur MCP* et la portée `prompts:write` à la création d'une clé sont à
    la 32-7. Si l'un d'eux est faux à l'étape 0 du fait de la 32-10b, le Dev Agent Record le dit ; la 32-10b ne le
    corrige pas.

## L'inventaire clos des chemins par lesquels l'écran pourrait faire approuver un texte non vu

*Règle de `CLAUDE.md` (§ Inventorier les sites NON RÉSOLUS). L'ensemble considéré : toute différence entre ce que
l'Administrateur **voit** au moment où il approuve, enregistre ou revient au standard, et ce que `prompts/get` servira
ensuite.*

| # | Site non résolu | Comment il se ferme | Par quoi |
|---|---|---|---|
| E1 | l'écran montre le texte **stocké**, l'agent reçoit le texte **servi** (marque) | l'écran affiche le champ `servi` du serveur (A) ou une recomposition gardée (B) | AC-5, M2, E2E (a) |
| E2 | la marque montrée à côté du texte, non dedans — l'Administrateur croit qu'elle n'est qu'un ornement | elle est dans le `<pre>`, encadrée, à sa place | AC-5, M13 |
| E3 | un caractère invisible ou bidirectionnel ne se voit pas | pastille `U+XXXX` à sa place, comptes | AC-6, M3, M4 |
| E4 | le rendu **retire** ou **remplace** un caractère (le texte affiché n'est plus le texte) | `reconstituer(segmenter(t)) === t` | AC-6, M3 |
| E5 | une proposition arrive entre la lecture et le clic | `numero` et `version` de l'objet **affiché** ; 409 | AC-7, M1 ; AC-i7 de la 32-10a |
| E6 | *Approuver* sur une version qui n'est pas la dernière | bouton absent ; le serveur refuse aussi (409) | AC-7, M6 |
| E7 | une approbation d'un clic, sans relecture | dialogue qui restate nom, numéro, clé, marque | AC-7, M7 |
| E8 | une version à valider repliée ou tronquée : on approuve un début | jamais repliée | AC-7, M15 |
| E9 | un texte d'agent recopié comme version d'administrateur perd sa marque (P26) | avis à l'écran et au manuel | AC-9, AC-18, M14 — **angle mort assumé** : la marque dit qui a écrit, non d'où viennent les mots |
| E10 | un texte modifié par l'écran avant l'envoi (`trim`, fins de ligne) : on enregistre autre chose que ce qu'on a tapé | valeurs exactes, fabriques de corps seules | AC-2, AC-9, M9 |
| E11 | du HTML dans un texte, une description ou un nom de clé | interpolation seule | AC-13, M5 |
| E12 | un retour au standard qui effacerait l'historique | une version de plus ; dit au dialogue | AC-10 ; AC-i10 de la 32-10a |
| E13 | le navigateur de l'Administrateur ne classe pas un caractère comme le serveur | le signalement est une aide ; le serveur refuse à l'écriture et ne sert pas | **angle mort écrit** (AC-6, *Ce que les gardes ne voient pas*) |
| E14 | une version à caractère refusé (restaurée, SQL) approuvée : rien n'est servi, l'Administrateur croit le contraire | le compte `refuse` et sa phrase ; D2 demande que le serveur refuse l'approbation | AC-6 — **reste** si D2 n'est pas fait |
| E15 | un agent qui pilote le navigateur de l'Administrateur approuve à sa place | hors de Kesh | **angle mort** (comme la 32-8c) |

## Tâches

- [ ] **T0 — étape 0** (§ *À recontrôler*) : 32-10a-i, -ii, -iii fusionnées ; `git log -1` partie de `main` à jour ;
  DTO et chemins réels de `routes/mcp_prompts.rs` relus et cités ; D1 et D2 constatés faits ou non (variante A ou B de
  l'AC-5) ; préfixe des messages du serveur ; état de la 32-7 (menu, G13, `sec:mcp`) ; recomptes.
- [ ] **T1 — API, types, fonctions pures** : `mcp-prompts.api.ts`, `mcp-prompts.types.ts`, `mcp-prompts-format.ts`,
  `caracteres-caches.ts` et leurs tests (AC-2, AC-6, AC-12) ; gardes `listes_egales_a_kesh_core`,
  `version_unicode_du_moteur` (et, en variante B, `gabarit_de_marque_egal_au_serveur`).
- [ ] **T2 — la liste et la création** (AC-1, AC-3, AC-11) ; l'entrée de menu ; `FAMILLES_RESOLUES`.
- [ ] **T3 — le détail** : texte servi, marque, caractères cachés, versions à valider, décision, historique (AC-4 à
  AC-8, AC-13, AC-14).
- [ ] **T4 — les gestes** : éditeur, retrait, retour au standard (AC-9, AC-10).
- [ ] **T5 — i18n** : quatre locales, garde des replis, `sitesTotal` (AC-15).
- [ ] **T6 — E2E** (AC-16) et **recette visuelle** (AC-17).
- [ ] **T7 — manuel, compte des routes, CHANGELOG, PDF** (AC-18, AC-19, AC-20).
- [ ] **T8 — mutations M1 à M16, gates, Dev Agent Record** (n'affirmer que ce qui a tourné, avec périmètre), prompt de la
  revue de code P1.

## Tests et mutations prévus

Une mutation à la fois, restaurée par `git checkout -- <f> && touch <f>` (`memory/mutation-restauree-binaire-perime`),
diff résiduel nul vérifié ; chacune nomme le test qui doit rougir ; une mutation qui survit, ou qui ne compile pas, se
déclare telle.

| # | Mutation | Doit rougir |
|---|---|---|
| M1 | l'approbation relit le détail (`GET`) avant le `POST` et envoie la `version` relue | `approbation_porte_la_version_lue` (Vitest) |
| M2 | le bloc servi affiche le champ `texte` stocké au lieu de `servi.texte` (variante B : la marque omise de `texteServi`) | `apercu_servi_est_le_champ_servi` ; E2E (a) (`prompts/get` ≠ `T`) |
| M3 | `segmenter` **omet** le caractère caché au lieu de le rendre en pastille | `reconstitution_a_l_identique` |
| M4 | U+180F retiré de `LISTES_KESH_CORE` | `listes_egales_a_kesh_core` ; le test de famille |
| M5 | `{@html servi.texte}` dans `McpPromptVersion.svelte` | `ecran_sans_html_brut` |
| M6 | *Approuver* rendu sur toute version `agent` `pending`, dernière ou non | `approuver_seulement_la_derniere` |
| M7 | *Approuver* envoie le `POST` sans dialogue | `approbation_apres_confirmation` |
| M8 | le repli d'une clé `mcp-prompts-*` diffère de sa valeur fr-CH | la garde des replis (AC-15) |
| M9 | `corpsTexte` applique `trim()` au texte | `corps_texte_exact` ; `envoie_la_valeur_exacte` |
| M10 | *Revenir au texte standard* rendu sur un prompt non standard | `retour_au_standard_seulement_si_utile` |
| M11 | le refus envoie le numéro de la **dernière** version au lieu de celle affichée | `refus_apres_confirmation` (chemin `…/versions/{numero}/refuse` égal) |
| M12 | `\t` signalé en mode `texte` (ou non signalé en mode `ligne`) | `n_et_t_admis_en_mode_texte` / `n_et_t_signales_en_mode_ligne` |
| M13 | la marque rendue hors du `<pre>` (au-dessus, en titre) | `marque_encadree_en_tete` |
| M14 | l'avis P26 retiré de l'éditeur | `avis_p26_si_reprise_d_une_version_d_agent` ; E2E (c) |
| M15 | la version à valider rendue repliée (`<details>` fermé) | `version_a_valider_jamais_repliee` (Vitest : le texte entier est dans le DOM et visible) |
| M16 | `nav-admin-mcp-prompts` retiré des quatre catalogues, `'admin-mcp-prompts'` gardé à `FAMILLES_RESOLUES` | `i18n-keys.test.ts`, « toute clé demandée existe au catalogue » |

Tests hors mutations, nommés au Dev Agent Record : les témoins de chaque garde ; `version_unicode_du_moteur` ;
`marque_incoherente_montree_telle_quelle` ; `aucune_relance_automatique_sur_409` ; les tests de la liste.

## Gates

- **Pas de 🗄️** : aucune migration, aucun dépôt. Entre les passes : `npm run check`, `npm run lint-i18n-ownership`,
  `npm run test:unit` (le dossier touché puis la suite), et, les `.ftl` changeant, `cargo test -p kesh-i18n` (gardes
  du chargeur, dont G3-bis) avec `cargo fmt --check` et `cargo clippy --workspace --all-targets -- -D warnings`.
- **Gate complet au dernier commit de code** (et avant tout push) : backend `scripts/test-fast.sh --ci` (les `.ftl`
  sont lus par les tests de `kesh-api`, dont `textes_coherents.rs`), frontend `check`, `lint-i18n-ownership`,
  `test:unit`, `build` ; base du worktree remise à zéro avant (`kesh_3210b`, `kesh_e2e_3210b`, **sans** redémarrer le
  conteneur) ; sous `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`.
- **E2E complet au dernier commit de code** — « rejouée au push » interdit (`CLAUDE.md`, D7) ; échecs attendus jugés
  fichier par fichier.
- **PDF** : `make fr` dans `docs/manual/`, sous le verrou.
- **Recette visuelle** : AC-17, avant la revue de code ; les captures sont jointes à son prompt.
- **Boucle de revue complète, Sonnet ↔ Opus** ; Haiku en passe ciblée seulement ; le prompt de chaque passe **nomme le
  manuel** (`.tex` et PDF aplati) et la propriété centrale (la valeur vue = la valeur servie).

## Ce que les gardes ne voient pas

- **Le navigateur de l'Administrateur.** Le signalement des caractères cachés emploie les classes Unicode du moteur
  JavaScript qui affiche la page ; Vitest vérifie celles de Node (17.0, comme le serveur), non celles de Chrome, Firefox
  ou Safari. Un navigateur plus ancien signale **plus** (un caractère récent y est `Cn`) ; un navigateur plus récent
  peut ne pas signaler un caractère qu'Unicode 17.0 dit non attribué — que le serveur refuse de toute façon.
- **Le signalement n'est pas un refus.** L'écran ne bloque rien (C-32-8-20) : une version à caractère refusé se
  montre, se signale, et le serveur la refuse à l'écriture ou ne la sert pas. Tant que D2 n'est pas fait, elle peut
  être **approuvée** sans être servie (E14).
- **Les invisibles d'autres catégories** que le serveur admet et que la liste `admis` ne nomme pas (l'angle mort de
  `kesh-core`, fait 6) ; les **homoglyphes** (un `а` cyrillique), visibles et trompeurs ; un texte **visible** qui dit
  de mal faire (P27 de la 32-10a) : l'écran le montre, il ne le juge pas.
- **Une pastille imitée** : un texte peut écrire `⟦U+202E⟧` en toutes lettres ; il ne prend pas le style d'une pastille
  et n'entre pas dans les comptes, mais un lecteur pressé peut le confondre — les comptes font foi, le manuel le dit.
- **La marque dit qui a écrit, non d'où viennent les mots** (P26, E9) : l'avis de l'éditeur et le manuel le disent ;
  rien ne l'empêche.
- **La variante B** (si D1 n'est pas fait) : une composition du texte servi modifiée côté serveur sans toucher au
  gabarit littéral de la marque — seule l'E2E la voit.
- **Les replis i18n hors des préfixes de l'écran** (#758).
- **Un agent qui piloterait le navigateur** de l'Administrateur pourrait approuver à sa place (E15) — comme pour la
  32-8c.
- **Le manuel** : aucun gate ne le lit ; les prompts de revue nomment l'axe et le PDF aplati.

## Dev Notes

### Les types (indicatifs dans la forme, exigés dans le fond ; alignés à l'étape 0 sur les DTO réels)

```ts
export type Origine = 'standard' | 'admin' | 'agent';
export type Etat = 'approved' | 'pending' | 'refused';
export type Nature = 'text' | 'withdrawal';

/** Une ligne de la liste : la synthèse, **sans** texte ni description. */
export interface McpPromptSummary {
	id: number; nom: string; isStandard: boolean; version: number;
	enVigueur: { numero: number; origine: Origine; nature: Nature } | null;
	aValider: number; standardDiffere: boolean;
}

/** Ce que `prompts/get` servirait si la version était en vigueur (D1). */
export interface ServiPrompt { description: string; texte: string; marque: string | null }

export interface McpPromptVersion {
	numero: number; origine: Origine; etat: Etat; nature: Nature;
	description: string | null; texte: string | null;   // stockés (Reprendre cette version)
	servi: ServiPrompt | null;                          // nul pour un retrait
	cleAuteur: { id: number; nom: string; revoquee: boolean } | null;
	auteur: string | null; decidePar: string | null; decideLe: string | null; creeLe: string;
}

export interface McpPromptDetail {
	id: number; nom: string; isStandard: boolean; version: number;
	numeroEnVigueur: number | null; standardDiffere: boolean;
	versions: McpPromptVersion[];
}
```

### La demande D1 (amendement de l'AC-iii6 de la 32-10a), formulée pour l'orchestrateur

> Dans la réponse de `GET /api/v1/admin/mcp/prompts/{id}`, chaque version de nature *texte* porte `servi:
> {description, texte, marque}` — la description et le texte **tels que `prompts/get` et `prompts/list` les
> rendraient si cette version était en vigueur**, marque comprise pour une version `agent` —, calculés par **la même
> fonction** que le catalogue (une fonction pure de composition, appelée par `PromptEnBase` et par la route ; aucun
> texte de marque écrit deux fois) ; `marque` est la marque seule, nulle hors `agent`. La réponse porte aussi
> `numeroEnVigueur` et `standardDiffere` (que la synthèse calcule déjà, AC-i5), et, pour une version d'agent, le nom de
> la clé auteur et si elle est révoquée. Test côté serveur : pour une version d'agent approuvée, `servi.texte` est
> **égal** au `messages[0].content.text` de `prompts/get`. Aucune route neuve ; module `routes` déjà compté par la
> 32-10a-iii.

### La demande D2 (souhaitable), formulée pour l'orchestrateur

> `POST …/versions/{numero}/approve` revalide le contenu de la version par `valider_contenu` (AC-ii2) avant d'écrire :
> une version que le catalogue ne servirait pas (caractère refusé restauré ou écrit hors de Kesh) n'est pas approuvable
> — 409 `PROMPT_VERSION_INVALIDE` (nom indicatif), message traduit. Sans cela, l'écran signale (E14) mais
> l'approbation passe, et le prompt reste non servi.

### Esquisse des fichiers (indicative)

```text
frontend/src/lib/features/mcp-prompts/mcp-prompts.api.ts        — routes, fabriques de corps (seules)
frontend/src/lib/features/mcp-prompts/mcp-prompts.types.ts      — types des réponses
frontend/src/lib/features/mcp-prompts/mcp-prompts-format.ts     — badges, dates, erreurs, en vigueur (fonctions pures)
frontend/src/lib/features/mcp-prompts/caracteres-caches.ts      — segmenter, reconstituer, LISTES_KESH_CORE
frontend/src/lib/features/mcp-prompts/McpPromptsList.svelte     — liste, création
frontend/src/lib/features/mcp-prompts/McpPromptDetail.svelte    — en-tête, servi aujourd'hui, à valider, historique
frontend/src/lib/features/mcp-prompts/McpPromptVersion.svelte   — rendu d'un texte servi (marque, pastilles)
frontend/src/lib/features/mcp-prompts/McpPromptEditor.svelte    — nouvelle version, retrait, avis P26
frontend/src/lib/features/mcp-prompts/*.test.ts                 — Vitest, gardes
frontend/src/routes/(app)/admin/mcp/prompts/+page.svelte        — mince
frontend/src/routes/(app)/admin/mcp/prompts/[id]/+page.svelte   — mince, {#key id}
frontend/tests/e2e/mcp-prompts.spec.ts                          — AC-16
```

### Ce qui est lu et cité

`frontend/src/routes/(app)/+layout.svelte:147-176`, `:171-175`, `:180-185` ;
`frontend/src/lib/features/mcp-sends/mcp-sends.api.ts:1-11`, `:36`, `:45`, `:50`, `:59` ;
`mcp-sends-i18n.test.ts:46`, `:47`, `:59`, `:101`, `:219` ; `McpSendDetail.svelte:202`, `:220-227` ;
`frontend/src/lib/shared/i18n-keys.test.ts:724-732`, `:866`, `:876-877` ;
`frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts:216` ; `crates/kesh-i18n/src/loader.rs:1106` ;
`crates/kesh-core/src/text.rs:79-85`, `:86-105`, `:94-104`, `:119`, `:131` ;
`docs/manual/fr/admin-manual.tex:2863`, `:2928`, `:2978`, `:3060` ;
`crates/kesh-api/tests/audit_route_registry.rs:899`, `:1265` ;
`frontend/tests/e2e/mcp-sends.spec.ts:36`, `:45`, `:77`, `:90`, `:127-185` ;
`_bmad-output/implementation-artifacts/sprint-status.yaml:820`, `:829-832` ;
fiche 32-10a l. 333-358, 635-640, 703-710, 899-916, 935-941, 991, 998-1011, 1276-1297, 1379-1397 ;
fiche 32-8 l. 872-958 ; fiche 32-7 l. 218-221, 529-594, 858-871 ; registre 328 l. 227-245 ; registre 3210a
l. 254-272.

### Références

- `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-10*, *Réponses de Guy*) ;
  `_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md` et `epic-32-choix-autonomes-3210a.md` ;
  `32-8-validation-avant-envoi.md` (partie C) et `epic-32-choix-autonomes-328.md` ;
  `/home/gcorbaz/devel/kesh-32-7/_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md` ; `CLAUDE.md` ;
  `docs/testing.md` ; `docs/agents-orchestration.md`.

## Points à soumettre à Guy

1. **Le texte que tu approuves doit-il être calculé par le serveur, exactement comme l'agent le recevra ?**
   *Recommandation* : oui (D1). La 32-10a calcule la marque côté serveur ; si l'écran la recomposait, deux codes
   écriraient la même phrase, et un écart entre eux ferait approuver un texte que l'agent ne reçoit pas tel quel. La
   variante B reste écrite au cas où l'amendement de la 32-10a-iii ne se ferait pas.
2. **Faut-il que le serveur refuse d'approuver une version qu'il ne servirait pas** (caractère invisible restauré
   d'une sauvegarde ou écrit hors de Kesh) ? *Recommandation* : oui (D2) — sans quoi tu peux approuver une version qui
   ne part jamais, et croire le contraire ; l'écran le signale, mais un signalement n'est pas un refus.
3. **L'écran doit-il montrer aussi les espaces spéciales** (insécable, fine insécable) et les quelques invisibles que
   Kesh admet, en plus des caractères qu'il refuse ? *Recommandation* : oui, en information distincte — c'est la seule
   façon de voir ce que l'agent recevra, et cela ne bloque rien.
4. **Faut-il, à l'approbation, une comparaison ligne à ligne avec la version en vigueur** (ce que l'agent propose de
   changer) ? *Recommandation* : pas en 0.16.0 — la fiche montre le texte entier, jamais replié, et un dialogue qui
   restate ce qui sera servi ; une comparaison est un composant de plus, à ouvrir en issue si la recette le demande
   (C-32-10b-15).
5. **Le dialogue d'approbation te suffit-il, ou veux-tu une case « J'ai lu le texte en entier »** ?
   *Recommandation* : le dialogue suffit ; une case se coche par réflexe et n'ajoute qu'un clic.
6. **L'écran reste-t-il réservé à l'Administrateur, même en lecture** (un Comptable ne voit pas les prompts) ?
   *Recommandation* : oui — c'est la règle des routes de la 32-10a (C-32-10a-19), et un prompt instruit un agent qui
   lit les livres.
7. **Après un retour au standard, le prompt suit de nouveau le texte des versions de Kesh** : le dialogue le dit. Le
   libellé *Revenir au texte standard* te convient-il ? *Recommandation* : oui.

## Demandes à l'orchestrateur

- **D1** (§ *Dev Notes*) : amender l'AC-iii6 de la fiche 32-10a **avant son développement** — le champ `servi` par
  version, `numeroEnVigueur`, `standardDiffere`, nom et révocation de la clé auteur ; un test serveur `servi.texte` =
  `prompts/get`. Réponse attendue avant le développement de la 32-10b ; sans elle, variante B de l'AC-5.
- **D2** (§ *Dev Notes*) : revalidation du contenu à l'approbation (souhaitable) — dans la 32-10a-iii, ou en issue.
- **D3** : aviser la fiche 32-7 (non fusionnée) qu'un troisième écran existe sous `/admin/mcp/` (`prompts/`) — son
  menu (ordre *Serveur MCP*, *Envois*, *Prompts*), sa garde G-a (sans descente, déjà écrite), un lien éventuel vers
  `/admin/mcp/prompts` à côté de celui des envois, et le rangement de `sec:mcp-prompts` sous `sec:mcp`.
- **D4** : si la recette le demande, ouvrir une issue `enhancement` « comparaison avec la version en vigueur à
  l'approbation d'un prompt » (point 4 pour Guy) — on trace toujours.

## À recontrôler à l'étape 0 du développement

- **La 32-10a fusionnée, et son code réel relu** — c'est le point principal :
  - `crates/kesh-api/src/routes/mcp_prompts.rs` : les sept chemins, méthodes, corps attendus (noms des champs : `version`,
    `nature`, `description`, `texte` ?), DTO de réponse de chaque route, codes et `code` d'erreur (409 : lesquels ?
    `OPTIMISTIC_LOCK_CONFLICT`, `ILLEGAL_STATE_TRANSITION` ou neufs ?), statuts (201, 200) ; lignes citées au Change Log.
  - **D1 fait ou non** (champ `servi`, `numeroEnVigueur`, `standardDiffere`, clé auteur) : variante A ou B de l'AC-5,
    déclarée. En B, le chemin et la forme de `marque_agent` (`crates/kesh-api/src/mcp/textes.rs`) et son gabarit.
  - **D2 fait ou non** : le texte d'E14 et de *Ce que les gardes ne voient pas* suit.
  - La règle de l'AC-i6 et de l'AC-i7 telle que codée (dernière version, états terminaux) — l'écran la reflète, il ne
    la refait pas.
  - Les bornes de longueur (500, 16 000) et la normalisation (`\r\n`) de `valider_contenu`.
  - Les clés de messages du serveur ajoutées par la 32-10a-iii (préfixe ; collision avec `mcp-prompts-`) et
    `audit-log-*` des prompts.
  - Le texte de la marque (`mcp/textes.rs`) pour l'E2E (a) et le manuel.
  - `seedTestState('with-company')` amorce bien les deux standards (AC-i11) ; le préréglage d'un Comptable pour
    l'E2E (e).
  - `prompts/get` et `prompts/list` : forme des réponses et texte de l'erreur `-32602` (AC-ii5, AC-ii6), et l'exigence
    `books:read` (C-32-10a-16).
- **La 32-7** : fusionnée ou non — l'ordre du menu (fait 1), `PREFIXES_REPLI_EGAL_AU_CATALOGUE` (AC-15 (a) ou (b)),
  la section `sec:mcp` (AC-18), sa garde G-a et le lien vers les envois.
- **La 32-9** si elle a fusionné : la forme `{"nonFiable": …}` nommée par les standards — le manuel peut le dire.
- **Recomptes** : `sitesTotal` (`i18n-keys.test.ts:732`) aux deux bornes ; `FAMILLES_RESOLUES` ; `NOMBRE_DE_CLES` de
  la garde des envois (48) si elle est réemployée ; les jetons des deux listes de `is_hidden_format` (16 au relevé,
  `text.rs:86-105`) ; `process.versions.unicode` (17.0) et `UNICODE_VERSION` de `unicode-properties` ; la phrase
  « 122 des 128 » (`admin-manual.tex:2978`) contre le registre des routes ; `grep -rn '{@html' frontend/src` → 0.
- **Les lignes du manuel** : `sec:mcp-envois` (2928), *Sécurité* (2863), *Monitoring et logs* (3060), le paragraphe
  de la remise à zéro (1415) et de la sauvegarde (2231, 2238) tels que la 32-10a-i les aura écrits.
- **L'E2E voisin** (`mcp-sends.spec.ts`) : `preparer` et la création d'une clé à deux portées (`books:read`,
  `prompts:write`) par `POST /api/v1/admin/mcp/keys`.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

| Date | Étape | Résumé |
|---|---|---|
| 2026-10-27 | création | Fiche rédigée en autonomie (Opus 5.5) contre `42680725` (branche `story/32-10b` = fiche 32-10a + `origin/main` `ed981c7d`) ; registre C-32-10b-1 à C-32-10b-16 ; **3 modules** (frontend, `kesh-i18n`, docs), aucun découpage ; **20 AC** (A : 4, B : 2, C : 8, D : 6), **9 tâches** (T0 à T8), **16 mutations** (M1 à M16) ; inventaire E1 à E15 ; 7 points pour Guy ; 4 demandes à l'orchestrateur (D1 : le texte servi calculé par le serveur, amendement de l'AC-iii6 de la 32-10a ; D2 : revalidation à l'approbation ; D3 : avis à la 32-7 ; D4 : issue de comparaison si la recette la demande). Prompt de validation P1 `3210b-validate-prompt-p1.md` (Sonnet, lentilles R et F). |
