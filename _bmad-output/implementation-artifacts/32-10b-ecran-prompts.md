# Story 32.10b : l'écran des prompts du serveur MCP

Status: ready-for-dev

*Fiche rédigée le 2026-10-27, en autonomie (Opus 5.5), worktree `kesh-32-10b`, branche `story/32-10b` créée depuis la
branche de fiche `story/32-10a`, puis `origin/main` fusionné au commit `ed981c7d` (la 32-8 complète, 32-8c comprise ;
merge `42680725`). Registre : `epic-32-choix-autonomes-3210b.md` (C-32-10b-1 à C-32-10b-19 ; 17 à 19 ajoutés à la validation P1). Clé courte : `3210b`.
Issue de l'epic : #723 (CR « Serveur MCP intégré à Kesh »), en `refs` seulement. Les numéros de ligne du dépôt sont
ceux de `42680725` ; ceux de la fiche 32-10a (`32-10a-prompts-en-base.md`, validée, **non développée**) et de la fiche
32-7 (worktree `kesh-32-7`, commit `7e38f92d`, lue en lecture seule) sont cités d'après ces fiches.*

⛔ **Développement APRÈS la fusion des trois parties de la 32-10a** (i, ii, iii). Cette fiche se spécifie contre **la
fiche** de la 32-10a : tout ce qu'elle en reprend — routes de gestion, champs des réponses, codes d'erreur, noms des
clés de messages du serveur, règles d'approbation — **se relit au code à l'étape 0** (§ *À recontrôler à l'étape 0*),
ligne lue citée au Change Log ; une phrase de cette fiche que le code ne confirme pas **se réécrit selon le code**.

⚠️ **L'écran repose sur l'amendement « D1 étendu » de la 32-10a-iii**, accepté par l'orchestrateur et inscrit à la
fiche 32-10a (branche `story/32-10a`, commit `e7a90572`, § *À recontrôler à l'étape 0*) : la route `GET
/api/v1/admin/mcp/prompts/{id}` rend, pour chaque version, le verdict **`servable`** du serveur, le texte et la
description servis **exacts** (marque comprise), les mêmes **rendus visibles par le serveur** (`rendre_visible`), pour
un standard le **texte du code** rendu visible, les **noms** de l'auteur et du décideur et la **date** de dernière
modification. **L'écran n'a donc aucune logique Unicode** : il affiche ce que le serveur a rendu visible et le verdict
du serveur (validation P1). Si l'étape 0 constate que l'amendement manque en tout ou partie, le développement
**s'arrête** et rend compte à l'orchestrateur — il n'y a plus de variante de repli côté écran (C-32-10b-1, annoté).

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
   catégories `Format`, `PrivateUse`, `Unassigned` **ou** deux listes écrites ; `is_hidden_in_text` (l. 119 : Cc sauf
   `\n` et `\t`, ou `is_hidden_format` — le prédicat du **texte** d'un prompt, fiche 32-10a, note d'étape 0). Angle
   mort écrit (l. 79-85) : U+2800, U+1D159, U+16FE4, les `Zs` U+1680, U+2000–U+200A, U+202F, U+205F, U+3000 restent
   admis. Classification d'**Unicode 17.0.0**. **L'écran ne reproduit pas ce prédicat** (validation P1) : c'est le
   serveur qui rend visible et qui juge (fait 12).
7. **Le manuel d'administration** (`docs/manual/fr/admin-manual.tex`) : `\section{Sécurité}` l. 2863 ;
   `\subsection{Les envois du serveur MCP}\label{sec:mcp-envois}` l. 2928, **provisoirement sous *Sécurité*** (32-8c,
   la section `sec:mcp` de la 32-7 n'existant pas) ; la phrase « 122 des 128 routes à verbe mutant au moment de la
   rédaction » et « 122 + 4 + 2 = 128 » l. 2978 ; `\section{Monitoring et logs}` l. 3060. **Aucune** occurrence de
   « prompt » (`grep -ci prompt docs/manual/fr/admin-manual.tex` → 0).
8. **Le registre des routes** (`crates/kesh-api/tests/audit_route_registry.rs`) : partition `traced + exempt +
   no_matter == LIB_ROUTES.len()` (l. 899) et `assert_eq!(tout.len(), 133)` (l. 1265) — la phrase du manuel (fait 7)
   n'est gardée par aucun test (`grep -rn '128' crates/kesh-api/tests/textes_coherents.rs` → rien). *(Relevé de la
   validation P1, R10 :)* `LIB_ROUTES.len()` est écrit **130** (l. 893-897 : « + 1 : `/mcp`, 32-6a ; + 1 : la
   décision d'un envoi MCP, 32-8b ») et `traced` **124** (l. 900-901) : la phrase « 122 des 128 » du manuel est donc
   **déjà fausse de deux** — 124 + 4 + 2 = 130 au relevé. À recompter au développement, après la 32-10a-iii.
9. **L'E2E voisin** (`frontend/tests/e2e/mcp-sends.spec.ts`, 206 lignes) : `login` (l. 36), `preparer` (l. 45 : `PUT
   /api/v1/admin/mcp/settings` puis `POST /api/v1/admin/mcp/keys` `{ name, scopes }`), `clientMcp` (l. 77 : contexte
   **neuf**, sans cookie, `Authorization: Bearer`, `MCP-Protocol-Version: 2025-11-25`), `appeler` (l. 90 :
   `tools/call`) ; la propriété éprouvée : la valeur **lue dans la page** juste avant le clic est celle que relit le
   client MCP (l. 127-185) ; `seedTestState('with-company')` en `beforeAll`.
10. **Le registre de sprint** : `32-10b-ecran-prompts: backlog` (`sprint-status.yaml:832`) ; `32-10a-i`, `-ii`, `-iii`
    `ready-for-dev` (l. 829-831) ; `32-7-ecran-serveur-mcp: backlog` (l. 820) ; `32-9a`, `32-9b` `backlog`.
11. **Les captures de la 32-8c** vivent hors du dépôt : `/home/gcorbaz/devel/kesh-gate-logs/328c-captures/` (`ls`).
12. **Le rendu visible du serveur** (32-8b, `crates/kesh-api/src/services/envois_mcp.rs:75`) : `pub fn
    rendre_visible(s: &str) -> String` remplace chaque caractère vérifiant `is_hidden_in_text` (donc `\r` compris,
    `\n` et `\t` **non**) par son échappement `\u{XXXX}` (hexadécimal en capitales, quatre chiffres au moins) et
    laisse le reste intact ; idempotente ; une suite **littérale** `\u{202E}` déjà écrite reste telle quelle (test
    `rendre_visible_par_famille`, l. 291-334, assertion l. 333). Les espaces admises (U+00A0, U+202F, les `Zs` du
    fait 6) ne sont **pas** rendues visibles. La fiche 32-10a l'emploie pour le « texte rendu visible » de D1 étendu.
13. **Le patron d'un Comptable en E2E** (`frontend/tests/e2e/backup-schedule.spec.ts:200-217`) : `POST
    /api/v1/users` `{ username, password, role: 'Comptable' }` par `authedApiContext`, puis `clearAuthStorage` et
    `login(page, comptable, …)` ; **aucun** helper partagé (`ls frontend/tests/e2e/helpers/` : `api-fixtures.ts`,
    `bank-import-fixtures.ts`, `contrast*.ts`, `inbox.ts`, `test-state*.ts`).

## Ce que dit la fiche 32-10a, et que cette fiche consomme (à recompter à l'étape 0)

| Élément | Fiche 32-10a | Ce que l'écran en fait |
|---|---|---|
| `GET /api/v1/admin/mcp/prompts` : nom, standard, version en vigueur (numéro, origine, nature), nombre à valider, « le standard du code diffère », `version` | AC-iii6 ; dépôt `synthese_par_societe` (AC-i5) | la liste (AC-3) |
| `GET /…/{id}` : le prompt et **tout** son historique, textes compris ; **par D1 étendu** (note d'étape 0, `e7a90572`) : par version, `servable`, servi exact, servi rendu visible, noms de l'auteur et du décideur ; pour un standard, le texte du code rendu visible ; la date de dernière modification | AC-iii6 ; `historique` ; D1, D1 étendu | le détail (AC-4 à AC-8, AC-10) |
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
les seules demandes au serveur sont les amendements D1 (étendu) et D2 (C-32-10b-1), qui se font **dans la
32-10a-iii** et y sont inscrits.

## Le découpage

Convention des fiches 32-5, 32-6, 32-8, 32-10a : un crate pour `kesh-i18n` et la documentation ; le frontend compte
pour un.

| Module | Ce que la story y touche |
|---|---|
| **frontend** | `src/lib/features/mcp-prompts/` (neuf), `src/routes/(app)/admin/mcp/prompts/` (neuf), `src/routes/(app)/+layout.svelte` (une entrée), `src/lib/shared/i18n-keys.test.ts` (`sitesTotal`, `FAMILLES_RESOLUES`), la garde des replis (selon l'étape 0 : `i18n-repli-divergent-actif.test.ts` ou un module partagé, AC-15), `tests/e2e/mcp-prompts.spec.ts` (neuf) |
| **`kesh-i18n`** | `locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` (clés `mcp-prompts-*`, `nav-admin-mcp-prompts`) |
| **docs** | `docs/manual/fr/admin-manual.{tex,pdf}` (la section, le compte des routes), `CHANGELOG.md` |

**Trois modules**, conformes au kickoff. Aucun découpage. Si le développement découvre qu'il lui faut une route ou un
champ que la 32-10a-iii ne rend pas — un champ de D1 étendu manquant compris —, c'est un **écart déclaré** : il touche `kesh-api/routes` et l'audit — quatre modules ou plus — et revient à
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
2. **On rend visible ce qui ne se voit pas — par le serveur** (AC-6). Le serveur rend chaque caractère caché sous la
   forme `\u{XXXX}` (`rendre_visible`, fait 12) et dit si la version est **servable** (`valider_contenu`) ; l'écran
   affiche ce champ **sans aucune transformation** (test au niveau du DOM) et ce verdict. **L'écran n'a aucune logique
   Unicode** : ni prédicat recopié, ni classe du moteur JavaScript, ni garde de version — la classe « le navigateur ne
   classe pas comme le serveur » est fermée par la conception (C-32-8-20, appliqué à la lettre ; C-32-10b-3 annoté).
3. **On approuve ce qu'on a vu** (AC-7). L'approbation porte le **numéro** de la version affichée et la **`version`**
   du prompt lue **avec** cet affichage — jamais une relecture : une proposition arrivée entre-temps rend la demande
   périmée (409, AC-i7 de la 32-10a). *Approuver* n'existe que sur la dernière version, si elle est une version
   d'agent à valider **et servable** ; il ouvre un dialogue qui **restate** ce qui sera servi (nom, numéro, clé auteur,
   et que le texte portera la marque). Le retour au standard montre, **avant** le geste, le texte du code qu'il fera
   servir (AC-10).
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
   `mcp-prompts.api.ts`, `mcp-prompts.types.ts`, `mcp-prompts-format.ts`,
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
   « N à valider », absente à zéro) ; un lien vers le détail. **Le drapeau `standardDiffere`** (AC-i5 de la 32-10a :
   vrai si la version en vigueur d'un standard **n'est pas d'origine `standard`**, ou si son contenu diffère de celui
   du code) **n'est pas affiché tel quel** — il ne distingue pas « vous l'avez personnalisé » de « le texte de Kesh a
   changé » (validation P1, R3). L'écran montre : *Personnalisé* quand la version en vigueur d'un standard n'est pas
   d'origine `standard` (déjà lisible par l'origine) ; **« Diffère du standard de cette version de Kesh »** (C-32-10b-19) seulement
   quand elle est d'origine `standard` **et** que le drapeau est levé — le cas d'un standard non personnalisé pas
   encore repris, par exemple après une restauration, jusqu'au démarrage suivant (AC-18). Ordre du serveur (par nom).
   Bouton *Actualiser*. **Aucun texte ni description** n'y paraît (la liste ne lit que la synthèse). Un bouton
   *Nouveau prompt* ouvre le formulaire de l'AC-11. Tests (`McpPromptsList.test.ts`) : rendu de chacun des cas
   (standard servi, personnalisé par l'administrateur, version d'agent en vigueur, retiré, non servi, deux à valider,
   standard d'origine `standard` au drapeau levé) par `data-testid` ; `personnalise_sans_avis_de_standard_change` (un
   standard personnalisé, drapeau levé : *Personnalisé*, **pas** « Diffère du standard ») ; liste vide (« Aucun prompt »
   — n'arrive qu'en cas d'échec de l'amorçage).
4. **AC-4 — le détail : l'en-tête et ce qui est servi aujourd'hui.** En-tête : nom, *Standard* le cas échéant, la
   version en vigueur (ou « Non servi »), la date de dernière modification (champ de D1 étendu, `modifieLe`). Puis un
   bloc **Ce que l'agent reçoit aujourd'hui** : la description servie et le texte servi de la version en vigueur
   (AC-5), **si et seulement si** elle est un texte et que le serveur la dit `servable` ; sinon la phrase « Ce prompt
   n'est servi à aucun agent » et sa raison — « retiré » ; « aucune version approuvée » ; « sa version en vigueur
   n'est pas valide : Kesh refuse de la servir (elle a pu être restaurée d'une sauvegarde ou écrite hors de Kesh) »
   (`servable` faux, AC-ii2 et l. 665 de la fiche 32-10a) — et, dans ce dernier cas, le texte rendu visible montré
   **sous** la phrase, titré « Texte de la version en vigueur, non servi ». Puis les **versions à valider** (AC-7),
   puis l'**historique** (AC-8), puis les gestes (AC-9, AC-10). Avis permanent en tête (`role="note"`), **exact pour
   chaque bloc** (validation P1, F3) : « Un prompt est une consigne donnée à l'agent. Sous *Ce que l'agent reçoit
   aujourd'hui*, le texte est celui qu'il reçoit ; sous *À valider*, celui qu'il recevrait si vous approuviez la
   version. La marque d'un prompt écrit par un agent en fait partie, et Kesh y montre chaque caractère invisible sous
   la forme `\u{XXXX}`. » Tests : `bloc_du_jour_seulement_si_servable` (en vigueur `servable: false` → la phrase,
   aucun bloc *reçoit aujourd'hui*) ; `avis_distingue_servi_et_a_valider`.

### B — Ce qui est montré : le texte servi, la marque, les caractères cachés

5. **AC-5 — on montre le texte servi, tel que le serveur l'a rendu visible** (C-32-10b-1, C-32-10b-2 ; D1 étendu).
   - **Les champs** (noms indicatifs, alignés à l'étape 0 sur le DTO réel) : pour chaque version de nature *texte*, la
     réponse de `GET /…/{id}` porte `servi: { description, texte, descriptionVisible, texteVisible, marque }` —
     `description` et `texte` **exactement** ce que `prompts/list` et `prompts/get` rendraient si cette version était
     en vigueur, marque comprise ; `descriptionVisible` et `texteVisible` les mêmes passés par `rendre_visible` (fait
     12) ; `marque` la marque seule (nulle pour une version `standard` ou `admin`) — et `servable: boolean`, le verdict
     de `valider_contenu` sur le contenu stocké. Le serveur les calcule par **la même fonction** que le catalogue
     (AC-ii4, AC-ii7 de la 32-10a) : **aucun texte servi n'est recomposé côté écran**, et l'écran ne transforme aucun
     de ces champs.
   - **Ce qui sert à quoi.** Le texte **rendu visible** sert à l'**affichage**, et à lui seul ; le texte **exact**
     n'est jamais affiché — il sert à l'**égalité avec `prompts/get`** dans l'E2E (AC-16). La marque se compose
     d'entiers et d'un gabarit constant (AC-ii7) : elle n'a aucun caractère caché, si bien que, pour une version
     d'agent, `texteVisible` commence par `marque + "\n\n"` comme `texte` (à recontrôler à l'étape 0).
   - **Le rendu** (`McpPromptVersion.svelte`) : deux blocs `<pre class="whitespace-pre-wrap break-words font-mono"
     dir="ltr">`, `data-testid="mcp-prompt-served-description-{numero}"` et `"mcp-prompt-served-text-{numero}"`. Quand
     `servi.marque` n'est pas nulle **et** que le champ visible commence par `servi.marque + "\n\n"` (comparaison de
     chaînes, aucune logique Unicode), la portion de la marque est un `<span data-testid="mcp-prompt-mark">` **à
     l'intérieur** du `<pre>`, encadré ; la pastille *Écrit par un agent* est **hors** du `<pre>`, au-dessus. Le reste
     est une interpolation Svelte de la tranche qui suit la marque. Sinon, le champ visible entier est interpolé, sans
     encadré — jamais masqué — et, pour une version `agent` dont la marque manque en tête, un avis (`role="alert"`,
     `data-testid="mcp-prompt-mark-missing"`) : « La marque attendue d'un prompt d'agent manque en tête de ce texte :
     n'approuvez pas, et signalez-le. » (validation P1, R11 ; la description suit la même règle que le texte).
   - ⛔ **La règle des blancs de gabarit** (validation P1, F4, R8) : à l'intérieur de chaque `<pre>`, la source Svelte
     ne porte **aucun** blanc, retour à la ligne ni indentation entre les balises et les blocs `{#if}` / `{:else}` /
     `{/if}` — le contenu du `<pre>` s'écrit d'un tenant, sur une ligne, car un `<pre>` affiche les blancs du gabarit
     comme du texte. Aucun `{#each}` dans un `<pre>`. Aucun libellé, pastille ni titre à l'intérieur.
   - **Tests au niveau du DOM** (`McpPromptVersion.test.ts`, composant monté, jsdom) :
     `texte_affiche_egal_au_champ_visible` — pour une fixture **multi-paragraphe** à blanc de tête, tabulation, ligne
     vide finale et un caractère rendu visible (`texte` portant U+202E, `texteVisible` portant `\u{202E}` à sa place),
     `pre.textContent === servi.texteVisible` **exactement**, `pre.textContent !== servi.texte`, et la chaîne exacte
     `servi.texte` n'apparaît nulle part dans le `textContent` du composant ; idem pour la description
     (`description_affichee_egale_au_champ_visible`) ; `marque_dans_le_bloc` — version d'agent : `pre.contains(mark)`,
     `mark.textContent === servi.marque`, `pre.textContent.startsWith(servi.marque + "\n\n")`, et la pastille *Écrit
     par un agent* n'est **pas** descendante du `<pre>` ; `sans_marque_pour_standard_et_admin` ;
     `marque_manquante_signalee` (un `texteVisible` d'agent qui ne commence pas par la marque : rendu entier, aucun
     encadré, l'avis présent). ⚠️ `marque_dans_le_bloc` ne peut pas passer à vide : une marque rendue hors du `<pre>`
     échoue à `pre.contains(mark)` **et** à l'égalité de `textContent` (M13).
6. **AC-6 — les caractères cachés et le verdict : ce que dit le serveur, rien d'autre** (C-32-10b-3, C-32-10b-4,
   annotés à la validation P1).
   - L'écran n'a **aucune** logique Unicode : ni copie des listes de `kesh-core`, ni classe `\p{…}` du moteur
     JavaScript, ni compte de caractères cachés calculé dans le navigateur, ni garde de version d'Unicode. Un caractère
     caché paraît à l'écran **parce que le serveur l'a écrit** `\u{XXXX}` dans le champ visible (fait 12) ; la preuve
     que l'écran affiche ce champ sans transformation est le test `texte_affiche_egal_au_champ_visible` (AC-5).
   - **Le verdict** : sur chaque version de nature texte dont `servable` est faux, un avis (`role="alert"`,
     `data-testid="mcp-prompt-not-servable-{numero}"`) : « Kesh ne servirait pas cette version : son contenu n'est pas
     valide (caractère invisible, longueur ou description sur plusieurs lignes). Elle a pu être restaurée d'une
     sauvegarde ou écrite hors de Kesh. » (le motif exact s'affiche si la réponse le rend, relu à l'étape 0). Une
     version **servable** ne contient aucun caractère caché : toute suite `\u{…}` qu'on y lit est **écrite en toutes
     lettres** dans le texte — l'avis permanent et le manuel le disent (§ *Ce que les gardes ne voient pas*).
   - Tests (`McpPromptVersion.test.ts`) : `avis_non_servable_selon_le_serveur` (`servable: false` → l'avis ;
     `servable: true` sur un texte dont `texteVisible` porte une suite `\u{202E}` littérale → **aucun** avis : l'écran
     ne juge pas le texte, il lit le verdict).

### C — Les gestes

7. **AC-7 — la version d'agent à valider : on approuve ce qu'on a vu** (C-32-10b-5, C-32-10b-6).
   - Chaque version `agent` `pending` est montrée **en entier** — description et texte servis (AC-5, AC-6), jamais
     tronqués, jamais repliés, sans « voir plus » — avec : son numéro, la clé auteur (numéro et nom de la clé ;
     « révoquée » le cas échéant, si la réponse le rend), la date de la proposition, et la pastille *Écrit par un
     agent — à valider*. Test `version_a_valider_jamais_repliee` (validation P1, R15, F6) : le `<pre>` du texte de la
     version à valider est dans le DOM et **visible** au premier rendu, aucun ancêtre `<details>` fermé ni
     `hidden`, aucun bouton « voir plus ».
   - **Approuver** n'est rendu que sur la version qui est **la dernière** du prompt (plus grand numéro de la réponse),
     qui est `agent` `pending` (AC-i7 de la 32-10a) **et que le serveur dit `servable`** (C-32-10b-17 : l'approbation
     d'une version non servable est refusée par le serveur, D2 ; l'écran n'offre pas un geste que la route refuse,
     AC-14). Une version à valider qui n'est plus la dernière porte la phrase « Une version plus récente existe :
     celle-ci ne peut plus être approuvée, seulement refusée » et le seul bouton *Refuser* ; une version à valider non
     servable porte l'avis de l'AC-6 et le seul bouton *Refuser*.
   - *Approuver* ouvre un **dialogue de confirmation** qui restate : le nom du prompt, « version n° N, proposée par la
     clé n° K (nom) », « Une fois approuvé, ce texte sera servi à tout agent dont la clé porte `books:read`,
     **précédé de la marque** » (le texte montré porte déjà la marque : elle ne dit « approuvée » qu'une fois le geste
     fait, validation P1, F10 (b)) ; boutons *Annuler* et *Approuver la version n° N*.
   - ⛔ **Le corps part avec la `version` lue avec l'affichage** : la fonction d'approbation reçoit le `version` et le
     `numero` **de l'objet affiché** ; aucun `GET` entre l'affichage et le `POST`. Après la réponse (200), l'écran
     **relit** le détail (`GET`) et l'affiche — la relecture vient **après** la décision, jamais avant.
   - *Refuser* : dialogue de confirmation (variante destructive) — « Un refus est définitif ; la version reste à
     l'historique » — puis `POST …/refuse` avec la `version` lue.
   - Tests (`McpPromptDetail.test.ts`) : `approbation_porte_la_version_lue` (le `GET` simulé rend `version: 7` et la
     version d'agent n° 3 ; clic *Approuver*, confirmation → **un seul** `GET` avant le `POST`, `POST` sur `…/versions/3/
     approve` avec `{ version: 7 }`) ; `approbation_apres_confirmation` (sans confirmation, aucun `POST`) ;
     `approuver_seulement_la_derniere` (pending n° 3, standard n° 4 → pas de bouton *Approuver* sur 3, *Refuser* oui) ;
     `pas_d_approbation_d_une_version_non_servable` (dernière version `agent` `pending`, `servable: false` → pas
     d'*Approuver*, *Refuser* oui) ; `aucun_bouton_sur_une_version_decidee` ; `refus_apres_confirmation` — **fixture
     écrite** (validation P1, R6, F6) : version d'agent à valider n° 3, dernière n° 4 (d'administrateur) ; *Refuser* sur
     la n° 3, confirmation → `POST …/versions/3/refuse` (chemin **égal**, jamais `…/versions/4/…`) avec la `version`
     lue ; `dialogue_restate_numero_cle_et_marque`.
   - ⛔ **Règle d'envoi en cours, sur chaque dialogue et chaque bouton qui envoie** (validation P1, F8 ; C-32-10b-18) —
     *Approuver*, *Refuser*, *Enregistrer comme nouvelle version*, *Retirer*, *Revenir au texte standard*, *Créer* :
     dès le clic de confirmation, le bouton est désactivé (`disabled`, `aria-busy="true"`) jusqu'à la réponse ; un
     second clic n'émet rien ; la réponse (succès ou erreur) remplace l'état, jamais ne s'ajoute à un message
     antérieur. Test paramétré `un_seul_post_pendant_l_envoi` (un `POST` simulé en attente, deux clics → **un**
     appel), un cas par geste.
8. **AC-8 — l'historique, jamais réécrit.** Un tableau de **toutes** les versions, plus récente en tête : numéro ;
   origine (*Standard* ; *Administrateur* — nom de l'auteur ; *Agent* — clé n° K, pastille d'avertissement) ; état
   (*Approuvée*, *À valider*, *Refusée*) ; nature (*Texte*, *Retrait*) ; la décision d'une version d'agent (par qui,
   quand — nom du décideur et date, champs de D1 étendu) ; la date ; la pastille **En vigueur** sur la version en vigueur (champ du serveur si la réponse le rend,
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
     *Reprendre cette version*. Une aide sous les champs dit les bornes de l'AC-i3 de la 32-10a (500 caractères pour la
     description, 16 000 pour le texte, relues à l'étape 0) **sans compteur ni contrôle côté écran** ; ni aperçu des
     caractères cachés de la saisie (validation P1 : l'écran n'a aucune logique Unicode, et un compteur de « caractères »
     en demanderait une — R13 tombe). Un caractère caché tapé ou collé ne s'écrit pas : le serveur le refuse (400, son
     message affiché, saisies conservées, AC-12) ; rien de non vu n'entre donc à l'historique. Bouton *Enregistrer
     comme nouvelle version* → `POST /{id}/versions` avec `corpsTexte(version lue,
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
10. **AC-10 — revenir au standard, sans rien écraser, et sur un texte vu** (C-32-10b-10, annoté). Sur un prompt
    `is_standard` dont le drapeau `standardDiffere` est levé — ce qui, par sa définition (AC-i5 de la 32-10a), couvre
    la version en vigueur personnalisée par l'administrateur, la version d'agent approuvée, le retrait, et le standard
    non personnalisé dont le texte diffère de celui du code (la condition « origine ≠ standard **ou** drapeau » de la
    première rédaction était redondante, validation P1, R3 ; à recontrôler au code à l'étape 0) : bouton *Revenir au
    texte standard*. Le dialogue montre **d'abord le texte que le geste fera servir** (validation P1, R7, F2) — le
    champ `standardDuCode: { descriptionVisible, texteVisible }` de D1 étendu, rendu comme à l'AC-5 dans deux `<pre>`
    (`data-testid="mcp-prompt-standard-description"`, `"mcp-prompt-standard-text"`, même règle des blancs de gabarit) —,
    puis : « Une nouvelle version n° N+1 reprend ce texte, le standard de cette version de Kesh. Vos versions et celles
    de l'agent restent à l'historique, et vous pourrez reprendre l'une d'elles. Le prompt redevient *non personnalisé* :
    il suivra désormais le texte des prochaines versions de Kesh. » → `POST /{id}/standard` avec `corpsStandard(version
    lue)`. Après 201, relecture : la nouvelle version paraît en tête, d'origine *Standard*, en vigueur. Absent pour un
    prompt non standard et pour un standard au drapeau baissé. Tests : `retour_au_standard_seulement_si_utile` (les
    quatre cas, plus un non standard et un standard à jour) ; `retour_au_standard_apres_confirmation` ;
    `dialogue_standard_montre_le_texte_du_code` (`textContent` du `<pre>` du dialogue **égal** à
    `standardDuCode.texteVisible`, et le dialogue ouvert avant tout `POST`).
11. **AC-11 — créer un prompt** (C-32-10b-16). Formulaire *Nouveau prompt* (liste) : nom, description, texte. Aide sous le nom :
    « Lettres minuscules sans accent, chiffres, `-` et `_`, 64 au plus ; un nom ne commence pas par `kesh`, réservé aux
    prompts standard. » (règle de l'AC-i2 de la 32-10a, **non contrôlée côté écran** : le 400 du serveur est affiché).
    `POST` → 201 → navigation vers le détail du prompt créé. Tests : corps exact ; 400 affiché, saisies conservées.
12. **AC-12 — les erreurs** (patron de `erreurDeDecision`, fait 2 ; codes relus à l'étape 0). 409 (verrou périmé,
    version non la dernière, déjà décidée, standard inconnu du code, version non servable refusée à l'approbation par
    D2 — code relu à l'étape 0) → le message (celui du serveur s'il est traduit,
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
    `<textarea>`, `spellcheck="false"`. ⚠️ `dir="ltr"` fixe le sens de **base** ; il ne neutralise pas les lettres
    arabes ou hébraïques **visibles**, que l'algorithme bidirectionnel réordonne encore avec les signes voisins — angle
    mort écrit (§ *Ce que les gardes ne voient pas* ; validation P1, R9, F9). Les **contrôles** bidirectionnels, eux,
    sont rendus `\u{…}` par le serveur.
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
      valider » **sur la ligne du nom unique** de ce test (assertion bornée à cette ligne, validation P1, F7 : une
      proposition laissée par un run interrompu ne la fausse pas) ; le détail montre la version à valider, **marque
      comprise** ; l'E2E lit dans la page, **avant** le clic, le texte affiché `T` (`textContent` du `<pre
      data-testid="mcp-prompt-served-text-{n}">`, `n` lu dans la page) et la description affichée `D` ; il lit aussi, par
      `GET /…/{id}` en session, les champs de cette version et exige `T === servi.texteVisible`, `D ===
      servi.descriptionVisible` — l'affichage est le champ rendu visible — et, ce texte étant sans caractère caché,
      `servi.texteVisible === servi.texte` ; *Approuver*, confirmation ; puis le client appelle `prompts/get` →
      `messages[0].content.text` **égal** à `servi.texte` (le texte **exact**) **et** à `T`, et `description` **égale**
      à `servi.description` et à `D` ; `prompts/list` porte une entrée `{ name, description: D, arguments: [] }`. Et `T`
      commence par `[Prompt écrit par un agent` (la marque, relue à l'étape 0 dans `mcp/textes.rs`).
    - **(b) Refus.** Seconde proposition ; *Refuser*, confirmation ; l'historique montre *Refusée* ; `prompts/get` de ce
      nom → l'erreur `-32602` (corps égal à celle d'un nom inconnu, demandée dans le même test) ; `prompts/list` ne le
      porte pas.
    - **(c) Nouvelle version d'administrateur** sur le prompt de (a), par *Reprendre cette version* : l'avis P26 est
      visible ; le texte modifié au clavier ; *Enregistrer* ; le texte servi lu dans la page `T2` **ne commence pas**
      par la marque ; `prompts/get` → **égal** à `T2`.
    - **(d) Retour au standard** sur `kesh-assistant` : l'E2E lit d'abord dans la page le **nombre de versions** de
      l'historique, `k` (compte **relatif** : `kesh-assistant` est partagé entre runs et specs, validation P1, R5, F7) ;
      une version d'administrateur ; puis *Revenir au texte standard* : le texte du code lu **dans le dialogue**, avant
      la confirmation, `T3` ; confirmation ; `prompts/get` → **égal** à `T3` (le texte vu avant le geste est le texte
      servi) ; l'historique porte `k + 2` versions — **deux** de plus (n° k+1 *Administrateur*, n° k+2 *Standard*),
      aucune n'a disparu.
    - **(e) Rôles** : un Comptable, créé sur le patron de `backup-schedule.spec.ts:200-217` (fait 13 : `POST
      /api/v1/users` avec `role: 'Comptable'`, puis `clearAuthStorage` et `login` — **aucun** helper partagé n'existe ;
      la fonction locale s'écrit dans la spec), ne voit pas l'entrée de menu ; `/admin/mcp/prompts` lui affiche le
      refus.
    Échecs attendus jugés **fichier par fichier** contre `docs/testing.md` § *Les échecs attendus*.
17. **AC-17 — la recette visuelle** (`CLAUDE.md` § *Recette visuelle* ; **thème clair seulement**), captures sur une
    base de démonstration du worktree (`kesh_3210b`), rangées hors du dépôt dans
    `/home/gcorbaz/devel/kesh-gate-logs/3210b-captures/`, jointes au Dev Agent Record et au prompt de la revue de code,
    **une par état** : (1) l'entrée de menu ; (2) la liste — les deux standards seuls ; (3) la liste avec une version
    à valider, un prompt retiré, un prompt non servi, un standard *Personnalisé* et un standard « Diffère du standard
    de cette version de Kesh » ; (4) le détail d'un standard servi ;
    (5) le détail avec une version d'agent à valider, **marque encadrée** ; (6) le dialogue d'approbation ; (7) le
    dialogue de refus ; (8) une version à valider qui n'est plus la dernière (*Refuser* seul) ; (9) l'historique après
    approbation, *En vigueur* et la pastille *Agent* ; (10) une version **piégée** — U+202E, U+200B, U+E0041 et une
    espace insécable, posés par un `INSERT` SQL dans `kesh_3210b` (le seul chemin : le serveur les refuse à
    l'écriture) — rendue par le serveur en `\u{…}` (l'espace insécable, admise, n'est **pas** rendue visible : angle
    mort, § *Ce que les gardes ne voient pas*), avec l'avis « Kesh ne servirait pas cette version » et sans
    *Approuver* ; (11) l'éditeur pré-rempli depuis une version d'agent,
    **avis P26** ; (12) le 400 d'une saisie à caractère invisible (message du serveur) ; (13) le 409 (verrou périmé),
    *Recharger* ; (14) le dialogue de retrait et le prompt retiré (« n'est servi à aucun agent ») ; (15) le dialogue
    de retour au standard, **le texte du code montré avant le geste**, et son résultat ; (16) le formulaire *Nouveau prompt* et son 400 sur un nom en `kesh` ; (17)
    le refus affiché à un Comptable.
18. **AC-18 — la section du manuel d'administration** (C-32-10b-12). `\subsection{Les prompts du serveur
    MCP}\label{sec:mcp-prompts}`, placée **après** `sec:mcp-envois` — sous la section *Le serveur MCP* (`sec:mcp`) de la
    32-7 si elle a fusionné, **sinon sous *Sécurité*, provisoirement**, comme la 32-8c (fait 7), la 32-7 l'y rangeant
    (sa fiche, AC-18 (9)). Elle dit, **tel que le code le fait** (chaque affirmation confrontée au code à l'étape 0,
    ligne citée au Dev Agent Record) :
    - ce qu'est un prompt du serveur MCP (une consigne que l'utilisateur **choisit** de donner à l'agent dans son
      client) ; qu'il n'ouvre **aucun** droit (portées de la clé, lecture seule, validation avant envoi) ;
    - les deux **prompts standard** (`kesh-developpement`, `kesh-assistant`), installés d'office, en français ; qu'ils
      suivent les versions de Kesh **tant qu'ils ne sont pas personnalisés** ; ce qu'est « personnalisé » ; ce que
      dit « Diffère du standard de cette version de Kesh » (un standard non personnalisé pas encore repris, jusqu'au
      démarrage suivant — non un standard personnalisé, que l'écran dit *Personnalisé*) ; le **retour au standard**,
      le texte du code montré **avant** le geste, et ce qu'il fait de l'historique (rien n'est effacé) ;
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
      Kesh ; montrés à l'écran sous la forme `\u{XXXX}`, **calculée par le serveur** (le même rendu que l'écran des
      envois) ; une version que Kesh ne servirait pas est signalée comme telle et ne s'approuve pas ; une version
      servable n'en contient aucun, si bien qu'un `\u{…}` lu dans son texte y est **écrit en toutes lettres** ; les
      espaces spéciales (insécable, fine insécable) et les quelques invisibles que Kesh admet **ne sont pas** montrées ;
      les lettres arabes ou hébraïques visibles restent soumises au réordonnancement bidirectionnel ;
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
    mutantes tracées), commande et sortie citées, généalogie au Dev Agent Record. Les nombres sont **déjà faux de
    deux** avant la 32-10a-iii (fait 8, validation P1, R10 : le registre écrit 130 routes, dont 124 tracées — `/mcp`
    de la 32-6a et la décision d'un envoi de la 32-8b —, contre 128 et 122 au manuel) ; le Dev Agent Record le dit
    et les corrige au compte réel, recompté à la source au développement (130 au relevé, à reprendre). Le même
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
| E1 | l'écran montre le texte **stocké**, l'agent reçoit le texte **servi** (marque) | l'écran affiche le champ servi du serveur (D1), rendu visible par lui | AC-5, M2, E2E (a) |
| E2 | la marque montrée à côté du texte, non dedans — l'Administrateur croit qu'elle n'est qu'un ornement | elle est **dans** le `<pre>`, encadrée, à sa place ; test au niveau du DOM | AC-5, M13 |
| E3 | un caractère invisible ou bidirectionnel ne se voit pas | le serveur l'écrit `\u{XXXX}` (`rendre_visible`) ; l'écran affiche ce champ | AC-5, AC-6, M2, M3 — **angle mort** : les espaces admises et les lettres RTL visibles (§ *Ce que les gardes ne voient pas*) |
| E4 | le rendu **retire**, **remplace** ou **ajoute** un caractère (blancs de gabarit) : le texte affiché n'est plus le champ | `textContent` du `<pre>` **égal** au champ visible ; règle des blancs de gabarit | AC-5, M3, M13 |
| E5 | une proposition arrive entre la lecture et le clic | `numero` et `version` de l'objet **affiché** ; 409 | AC-7, M1 ; AC-i7 de la 32-10a |
| E6 | *Approuver* sur une version qui n'est pas la dernière | bouton absent ; le serveur refuse aussi (409) | AC-7, M6 |
| E7 | une approbation d'un clic, sans relecture | dialogue qui restate nom, numéro, clé, marque | AC-7, M7 |
| E8 | une version à valider repliée ou tronquée : on approuve un début | jamais repliée | AC-7, M15 |
| E9 | un texte d'agent recopié comme version d'administrateur perd sa marque (P26) | avis à l'écran et au manuel | AC-9, AC-18, M14 — **angle mort assumé** : la marque dit qui a écrit, non d'où viennent les mots |
| E10 | un texte modifié par l'écran avant l'envoi (`trim`, fins de ligne) : on enregistre autre chose que ce qu'on a tapé | valeurs exactes, fabriques de corps seules | AC-2, AC-9, M9 |
| E11 | du HTML dans un texte, une description ou un nom de clé | interpolation seule | AC-13, M5 |
| E12 | un retour au standard qui effacerait l'historique | une version de plus ; dit au dialogue | AC-10 ; AC-i10 de la 32-10a |
| E13 | le navigateur de l'Administrateur ne classe pas un caractère comme le serveur | **fermé par la conception** (validation P1) : l'écran n'a aucune logique Unicode ; rendu et verdict viennent du serveur | AC-6 |
| E14 | une version à caractère refusé (restaurée, SQL) approuvée : rien n'est servi, l'Administrateur croit le contraire | verdict `servable` du serveur affiché ; *Approuver* absent ; le serveur refuse l'approbation (D2, acceptée) | AC-4, AC-6, AC-7, M4 |
| E15 | un agent qui pilote le navigateur de l'Administrateur approuve à sa place | hors de Kesh | **angle mort** (comme la 32-8c) |
| E16 | le retour au standard fait servir un texte que l'Administrateur n'a pas vu (texte du binaire) | le dialogue montre `standardDuCode`, rendu visible par le serveur, **avant** le geste | AC-10, M12, E2E (d) — **angle mort** : un binaire mis à jour entre l'affichage et le clic (le texte du code change sans que la `version` du prompt bouge) |
| E17 | un `\u{202E}` écrit **en toutes lettres** par un agent se confond avec un caractère rendu visible | verdict `servable` : une version servable n'a aucun caractère caché, donc tout `\u{…}` y est littéral ; dit par l'avis et le manuel | AC-6, AC-18 — **angle mort** : la seule forme ne distingue pas les deux, le verdict le fait |

## Tâches

- [ ] **T0 — étape 0** (§ *À recontrôler*) : 32-10a-i, -ii, -iii fusionnées ; `git log -1` partie de `main` à jour ;
  DTO et chemins réels de `routes/mcp_prompts.rs` relus et cités ; **D1 étendu et D2 constatés faits**, champ par champ
  (sinon : arrêt et compte rendu à l'orchestrateur, aucune variante côté écran) ; préfixe des messages du serveur ;
  état de la 32-7 (menu, G13, `sec:mcp`) ; recomptes.
- [ ] **T1 — API, types, fonctions pures** : `mcp-prompts.api.ts`, `mcp-prompts.types.ts`, `mcp-prompts-format.ts`
  et leurs tests (AC-2, AC-12) — aucune fonction Unicode.
- [ ] **T2 — la liste et la création** (AC-1, AC-3, AC-11) ; l'entrée de menu ; `FAMILLES_RESOLUES`.
- [ ] **T3 — le détail** : texte servi rendu visible, marque, verdict `servable`, versions à valider, décision, envoi
  en cours, historique (AC-4 à AC-8, AC-13, AC-14).
- [ ] **T4 — les gestes** : éditeur, retrait, retour au standard et texte du code (AC-9, AC-10).
- [ ] **T5 — i18n** : quatre locales, garde des replis, `sitesTotal` (AC-15).
- [ ] **T6 — E2E** (AC-16) et **recette visuelle** (AC-17).
- [ ] **T7 — manuel, compte des routes, CHANGELOG, PDF** (AC-18, AC-19, AC-20).
- [ ] **T8 — mutations M1 à M17, gates, Dev Agent Record** (n'affirmer que ce qui a tourné, avec périmètre), prompt de la
  revue de code P1.

## Tests et mutations prévus

Une mutation à la fois, restaurée par `git checkout -- <f> && touch <f>` (`memory/mutation-restauree-binaire-perime`),
diff résiduel nul vérifié ; chacune nomme le test qui doit rougir ; une mutation qui survit, ou qui ne compile pas, se
déclare telle. *(Validation P1 : M2, M3, M4, M12 et M13 réécrites — les mutations des listes Unicode et des modes de
l'écran tombent avec sa logique Unicode ; M17 ajoutée.)*

| # | Mutation | Doit rougir |
|---|---|---|
| M1 | l'approbation relit le détail (`GET`) avant le `POST` et envoie la `version` relue | `approbation_porte_la_version_lue` (Vitest) |
| M2 | le bloc servi affiche `servi.texte` (exact) au lieu de `servi.texteVisible` | `texte_affiche_egal_au_champ_visible` ; E2E (a) (`T` ≠ `servi.texteVisible`) |
| M3 | un retour à la ligne et une indentation insérés dans le gabarit, à l'intérieur du `<pre>` (ou `trim()` appliqué au champ affiché) | `texte_affiche_egal_au_champ_visible` |
| M4 | *Approuver* rendu sur une version `servable: false` | `pas_d_approbation_d_une_version_non_servable` |
| M5 | `{@html servi.texteVisible}` dans `McpPromptVersion.svelte` | `ecran_sans_html_brut` |
| M6 | *Approuver* rendu sur toute version `agent` `pending`, dernière ou non | `approuver_seulement_la_derniere` |
| M7 | *Approuver* envoie le `POST` sans dialogue | `approbation_apres_confirmation` |
| M8 | le repli d'une clé `mcp-prompts-*` diffère de sa valeur fr-CH | la garde des replis (AC-15) |
| M9 | `corpsTexte` applique `trim()` au texte | `corps_texte_exact` ; `envoie_la_valeur_exacte` |
| M10 | *Revenir au texte standard* rendu sur un prompt non standard | `retour_au_standard_seulement_si_utile` |
| M11 | le refus envoie le numéro de la **dernière** version au lieu de celle affichée | `refus_apres_confirmation` (fixture n° 3 à valider, n° 4 dernière ; chemin `…/versions/3/refuse` égal) |
| M12 | le dialogue de retour au standard n'affiche pas `standardDuCode` | `dialogue_standard_montre_le_texte_du_code` ; E2E (d) |
| M13 | la marque rendue hors du `<pre>` (au-dessus, en titre) | `marque_dans_le_bloc` (`pre.contains(mark)` et égalité de `textContent`) |
| M14 | l'avis P26 retiré de l'éditeur | `avis_p26_si_reprise_d_une_version_d_agent` ; E2E (c) |
| M15 | la version à valider rendue repliée (`<details>` fermé) | `version_a_valider_jamais_repliee` (AC-7 : le texte entier est dans le DOM et visible) |
| M16 | `nav-admin-mcp-prompts` retiré des quatre catalogues, `'admin-mcp-prompts'` gardé à `FAMILLES_RESOLUES` | `i18n-keys.test.ts`, « toute clé demandée existe au catalogue » |
| M17 | le bouton de confirmation d'*Approuver* n'est plus désactivé pendant l'envoi | `un_seul_post_pendant_l_envoi` |

Tests hors mutations, nommés au Dev Agent Record : les témoins de chaque garde ;
`description_affichee_egale_au_champ_visible` ; `marque_manquante_signalee` ; `avis_non_servable_selon_le_serveur` ;
`bloc_du_jour_seulement_si_servable` ; `avis_distingue_servi_et_a_valider` ;
`personnalise_sans_avis_de_standard_change` ; `aucune_relance_automatique_sur_409` ; les tests de la liste.

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

- **Le rendu visible est celui du serveur, avec ses angles morts.** L'écran affiche ce que `rendre_visible` a écrit
  (fait 12) : les invisibles que le serveur **admet** — les espaces `Zs` autres que U+0020 (dont U+00A0 et U+202F),
  U+2800, U+1D159, U+16FE4 (l'angle mort écrit de `kesh-core`, fait 6) — ne sont **pas** rendus visibles et partent
  tels quels à l'agent ; `\t` non plus (admis dans un texte). C'est le point 3 pour Guy.
- **Un `\u{…}` écrit en toutes lettres** dans le texte d'un agent s'affiche exactement comme un caractère rendu visible
  par le serveur (E17). Le verdict `servable` les distingue : une version servable n'a aucun caractère caché. L'avis et
  le manuel le disent ; la forme seule ne le dit pas.
- **Les lettres bidirectionnelles visibles** (arabe, hébreu) : `dir="ltr"` fixe le sens de base, l'algorithme
  bidirectionnel les réordonne encore avec les signes voisins (validation P1, R9, F9). Les **contrôles** bidi, eux,
  sont rendus `\u{…}`.
- **Les homoglyphes** (un `а` cyrillique), visibles et trompeurs ; un texte **visible** qui dit de mal faire (P27 de la
  32-10a) : l'écran le montre, il ne le juge pas.
- **Le texte du code entre l'affichage et le clic** (E16) : un binaire mis à jour pendant que le dialogue de retour au
  standard est ouvert fait servir un texte que le dialogue ne montrait pas — la `version` du prompt ne bouge pas.
- **La marque dit qui a écrit, non d'où viennent les mots** (P26, E9) : l'avis de l'éditeur et le manuel le disent ;
  rien ne l'empêche.
- **La justesse des champs de D1 étendu** est tenue par les tests de la 32-10a-iii (égalité de `servi.texte` avec
  `prompts/get`) et, de bout en bout, par l'E2E (a) et (d) ; les tests Vitest de l'écran ne la voient pas (réponses
  simulées).
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

/** Ce que `prompts/get` servirait si la version était en vigueur (D1, D1 étendu). */
export interface ServiPrompt {
	description: string; texte: string;                  // exacts — jamais affichés (égalité E2E)
	descriptionVisible: string; texteVisible: string;    // rendus visibles par le serveur — seuls affichés
	marque: string | null;                               // nulle hors `agent`
}

export interface McpPromptVersion {
	numero: number; origine: Origine; etat: Etat; nature: Nature;
	description: string | null; texte: string | null;   // stockés (Reprendre cette version)
	servi: ServiPrompt | null;                          // nul pour un retrait
	servable: boolean | null;                           // verdict de `valider_contenu` ; nul pour un retrait
	cleAuteur: { id: number; nom: string; revoquee: boolean } | null;
	auteurNom: string | null; decideurNom: string | null; decideLe: string | null; creeLe: string;
}

export interface McpPromptDetail {
	id: number; nom: string; isStandard: boolean; version: number; modifieLe: string;
	numeroEnVigueur: number | null; standardDiffere: boolean;
	standardDuCode: { descriptionVisible: string; texteVisible: string } | null;   // nul hors standard
	versions: McpPromptVersion[];
}
```

### Les amendements D1, D1 étendu et D2 — acceptés, inscrits à la fiche 32-10a

Acceptés par l'orchestrateur et inscrits au § *À recontrôler à l'étape 0* de la fiche 32-10a (commits `47394982` pour
D1 et D2, `e7a90572` pour D1 étendu, branche `story/32-10a`). Ce que l'écran en attend, pour le recontrôle de son
étape 0 :

- **D1** : par version de nature texte, le texte et la description **tels que `prompts/get` et `prompts/list` les
  rendraient**, marque comprise, calculés par **la même fonction** que le catalogue ; `numeroEnVigueur`,
  `standardDiffere` ; le nom de la clé auteur et sa révocation. Test serveur : `servi.texte` **égal** au
  `messages[0].content.text` de `prompts/get` pour une version d'agent approuvée.
- **D1 étendu** : (a) `servable`, le verdict de `valider_contenu` (la fonction de la création et de l'approbation) ;
  (b) les mêmes champs passés par `rendre_visible` ; (c) pour un standard, le texte du code (`STANDARDS`) qu'un
  retour au standard ferait servir, rendu visible ; (d) les **noms** de l'auteur et du décideur, et la **date** de
  dernière modification (validation P1, R4, R12).
- ⛔ **Contrainte des primitives** (validation P1, R4 ; fiche 32-10a, validation P1 R1/F1, l. 666-670) : la fonction de
  composition prend des **primitives** (`i64`/`u32`, `&str`) — **aucun** type de `kesh_db` ne sort du service vers
  `src/mcp/` ; `PromptEnBase` et la route l'appellent tous deux. La description d'une version d'agent contient `\n\n`
  après la marque : le contrôle qui s'y applique est celui de l'AC-ii7, non le prédicat d'une ligne (validation P1,
  R1 : sans quoi chaque description d'agent saine serait dite non valide).
- **D2** : la route d'approbation revalide le contenu par `valider_contenu` et refuse une version non servable
  (code relu à l'étape 0) ; l'écran n'offre déjà pas le geste (C-32-10b-17).

### Esquisse des fichiers (indicative)

```text
frontend/src/lib/features/mcp-prompts/mcp-prompts.api.ts        — routes, fabriques de corps (seules)
frontend/src/lib/features/mcp-prompts/mcp-prompts.types.ts      — types des réponses
frontend/src/lib/features/mcp-prompts/mcp-prompts-format.ts     — badges, dates, erreurs, en vigueur (fonctions pures)
frontend/src/lib/features/mcp-prompts/McpPromptsList.svelte     — liste, création
frontend/src/lib/features/mcp-prompts/McpPromptDetail.svelte    — en-tête, servi aujourd'hui, à valider, historique
frontend/src/lib/features/mcp-prompts/McpPromptVersion.svelte   — rendu d'un texte servi visible (marque, verdict)
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
`crates/kesh-core/src/text.rs:79-85`, `:86-105`, `:119` ; `crates/kesh-api/src/services/envois_mcp.rs:75`,
`:291-334` ;
`docs/manual/fr/admin-manual.tex:2863`, `:2928`, `:2978`, `:3060` ;
`crates/kesh-api/tests/audit_route_registry.rs:893-901`, `:1265` ;
`frontend/tests/e2e/mcp-sends.spec.ts:36`, `:45`, `:77`, `:90`, `:127-185` ;
`frontend/tests/e2e/backup-schedule.spec.ts:200-217` ; `.github/workflows/ci.yml:218-222` (validation P1) ;
`_bmad-output/implementation-artifacts/sprint-status.yaml:820`, `:829-832` ;
fiche 32-10a l. 289-296, 333-358, 635-640, 663-670, 703-710, 899-916, 935-941, 991, 998-1011, 1276-1297, 1379-1397,
1533-1538 (D1, D2, D1 étendu, branche `story/32-10a` à `e7a90572`) ;
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
   *Tranchée par l'orchestrateur le 2026-10-27, en attendant ta réponse : oui* (D1, puis D1 étendu, inscrits à la
   fiche 32-10a). La 32-10a calcule la marque côté serveur ; si l'écran la recomposait, deux codes écriraient la même
   phrase, et un écart entre eux ferait approuver un texte que l'agent ne reçoit pas tel quel. La variante de repli
   côté écran est **retirée** (validation P1) : sans l'amendement, le développement s'arrête.
2. **Faut-il que le serveur refuse d'approuver une version qu'il ne servirait pas** (caractère invisible restauré
   d'une sauvegarde ou écrit hors de Kesh) ? *Tranchée par l'orchestrateur le 2026-10-27, en attendant ta réponse :
   oui* (D2) — sans quoi tu pourrais approuver une version qui ne part jamais, et croire le contraire. L'écran, qui lit
   désormais le verdict du serveur, n'offre pas non plus le bouton.
3. **Les espaces spéciales — insécable, fine insécable — et les quelques invisibles que Kesh admet ne seront pas
   montrés** : c'est le serveur qui rend les caractères visibles, par la fonction déjà employée pour les envois, et
   elle ne rend visible que ce que Kesh refuse. Acceptes-tu cet angle mort, ou faut-il que le serveur rende visibles
   aussi ces caractères admis — ce qui changerait le rendu des envois, ou demanderait une seconde fonction dans la
   32-10a ? *Recommandation* : accepter l'angle mort en 0.16.0 ; ces caractères sont admis partout dans Kesh et
   abondent dans un texte français (`:`, `;`, `?`, `!`, guillemets) — les montrer noierait le reste ; une issue
   s'ouvre si la recette en montre le besoin.
4. **Faut-il, à l'approbation, une comparaison ligne à ligne avec la version en vigueur** (ce que l'agent propose de
   changer) ? *Recommandation* : pas en 0.16.0 — la fiche montre le texte entier, jamais replié, et un dialogue qui
   restate ce qui sera servi ; une comparaison est un composant de plus, à ouvrir en issue si la recette le demande
   (C-32-10b-15).
5. **Le dialogue d'approbation te suffit-il, ou veux-tu une case « J'ai lu le texte en entier »** ?
   *Recommandation* : le dialogue suffit ; une case se coche par réflexe et n'ajoute qu'un clic.
6. **L'écran reste-t-il réservé à l'Administrateur, même en lecture** (un Comptable ne voit pas les prompts) ?
   *Recommandation* : oui — c'est la règle des routes de la 32-10a (C-32-10a-19), et un prompt instruit un agent qui
   lit les livres.
7. **Après un retour au standard, le prompt suit de nouveau le texte des versions de Kesh** : le dialogue le dit, et
   montre d'abord le texte qui sera servi. Le libellé *Revenir au texte standard* te convient-il ? *Recommandation* :
   oui.

## Demandes à l'orchestrateur

- **D1** et **D1 étendu** (§ *Dev Notes*) : **acceptées**, inscrites à la fiche 32-10a (`47394982`, `e7a90572`).
  Reste à les recontrôler au code à l'étape 0 de la 32-10b, champ par champ.
- **D2** (§ *Dev Notes*) : **acceptée**, inscrite à la fiche 32-10a (`47394982`).
- **D3** : aviser la fiche 32-7 (non fusionnée) qu'un troisième écran existe sous `/admin/mcp/` (`prompts/`) — son
  menu (ordre *Serveur MCP*, *Envois*, *Prompts*), sa garde G-a (sans descente, déjà écrite), un lien éventuel vers
  `/admin/mcp/prompts` à côté de celui des envois, et le rangement de `sec:mcp-prompts` sous `sec:mcp`.
- **D4** : si la recette le demande, ouvrir une issue `enhancement` « comparaison avec la version en vigueur à
  l'approbation d'un prompt » (point 4 pour Guy) — on trace toujours ; de même pour le point 3 si Guy ou la recette
  demandent de montrer les caractères admis.
- **D5** (validation P1, R16) : la fiche 32-10a cite `admin-manual.tex:2935` pour la phrase du compte des routes
  mutantes (ses l. 1008, 1331 et 1384) — juste sur sa propre base, où la phrase dit « 108 des 114 » ; sur `main`
  fusionné (`42680725`), la ligne est **2978** et dit « 122 des 128 » (`grep -n "routes à verbe mutant"
  docs/manual/fr/admin-manual.tex` dans les deux worktrees). À signaler à l'étape 0 de la 32-10a — et le compte de
  `main` est lui-même déjà faux de deux (fait 8).

## À recontrôler à l'étape 0 du développement

- **La 32-10a fusionnée, et son code réel relu** — c'est le point principal :
  - `crates/kesh-api/src/routes/mcp_prompts.rs` : les sept chemins, méthodes, corps attendus (noms des champs : `version`,
    `nature`, `description`, `texte` ?), DTO de réponse de chaque route, codes et `code` d'erreur (409 : lesquels ?
    `OPTIMISTIC_LOCK_CONFLICT`, `ILLEGAL_STATE_TRANSITION` ou neufs ?), statuts (201, 200) ; lignes citées au Change Log.
  - **D1 et D1 étendu faits, champ par champ** (`servi` exact et rendu visible, `servable`, `standardDuCode`,
    `numeroEnVigueur`, `standardDiffere`, clé auteur, noms de l'auteur et du décideur, `modifieLe`) ; s'il en manque un :
    **arrêt**, compte rendu à l'orchestrateur (§ *Le découpage* : écart déclaré). La fonction de composition à
    signature primitive ; le rendu visible fait par `rendre_visible` (fait 12) ; la description d'agent (`\n\n` après
    la marque) jugée par le contrôle de l'AC-ii7, et une version d'agent saine dite `servable`.
  - **Une version servable a-t-elle toujours `texteVisible === texte`** ? Oui si `valider_contenu` refuse tout ce que
    `rendre_visible` rend visible ; à vérifier sur `\r` (normalisé par `valider_contenu` **avant** le contrôle, AC-ii2,
    mais rendu `\u{000D}` par `rendre_visible`) : une version restaurée portant `\r\n` serait-elle servable avec un
    `\u{000D}` affiché ? La réponse (et ce que sert `prompts/get` dans ce cas) se cite au Change Log ; l'avis de l'AC-6
    et la ligne E17 se réécrivent selon le code.
  - **D2 fait** : le code de refus de l'approbation d'une version non servable (AC-12).
  - La règle de l'AC-i6 et de l'AC-i7 telle que codée (dernière version, états terminaux) — l'écran la reflète, il ne
    la refait pas.
  - Les bornes de longueur (500, 16 000) et la normalisation (`\r\n`) de `valider_contenu`.
  - Les clés de messages du serveur ajoutées par la 32-10a-iii (préfixe ; collision avec `mcp-prompts-`) et
    `audit-log-*` des prompts.
  - Le texte de la marque (`mcp/textes.rs`) pour l'E2E (a) et le manuel.
  - `seedTestState('with-company')` amorce bien les deux standards (AC-i11) ; le patron d'un Comptable pour l'E2E (e)
    (fait 13).
  - `prompts/get` et `prompts/list` : forme des réponses et texte de l'erreur `-32602` (AC-ii5, AC-ii6), et l'exigence
    `books:read` (C-32-10a-16).
- **La 32-7** : fusionnée ou non — l'ordre du menu (fait 1), `PREFIXES_REPLI_EGAL_AU_CATALOGUE` (AC-15 (a) ou (b)),
  la section `sec:mcp` (AC-18), sa garde G-a et le lien vers les envois.
- **La 32-9** si elle a fusionné : la forme `{"nonFiable": …}` nommée par les standards — le manuel peut le dire.
- **Recomptes** : `sitesTotal` (`i18n-keys.test.ts:732`) aux deux bornes ; `FAMILLES_RESOLUES` ; `NOMBRE_DE_CLES` de
  la garde des envois (48) si elle est réemployée ; la phrase « 122 des 128 » (`admin-manual.tex:2978`) contre le
  registre des routes (130 et 124 au relevé de la validation P1, fait 8) ; `grep -rn '{@html' frontend/src` → 0.
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
| 2026-10-27 | validation P1 | **Sonnet ×2** (lentilles R et F, contexte frais, prompt `3210b-validate-prompt-p1.md` ; rapports `kesh-gate-logs/3210b-validate-p1-{R,F}.md`) : R 0/2/5/9, F 0/1/3/6 ⇒ **trend P1 0/3/8/15** (doublons F1 = R2, F2 = R7, F6 ⊃ R6 + R15, F7 = R5, F9 = R9, comptés une fois par lentille). **Décision de l'orchestrateur : fermer les classes par la conception, pas par des gardes** — D1 **étendu** inscrit à la fiche 32-10a (`story/32-10a`, `e7a90572`, § *À recontrôler à l'étape 0*) : par version, `servable`, servi exact et **rendu visible par le serveur** (`rendre_visible`, 32-8b, fait 12), pour un standard le texte du code rendu visible, noms de l'auteur et du décideur, date de dernière modification. **L'écran n'a plus aucune logique Unicode** : retirés `caracteres-caches.ts` (`segmenter`, `reconstituer`), la copie des listes de `kesh-core` et sa garde, la garde de version d'Unicode du moteur, les pastilles et les comptes calculés dans le navigateur, l'aperçu et les compteurs de l'éditeur, la variante B de l'AC-5 (sans D1 étendu, arrêt et compte rendu). **HIGH** : R1 (description d'agent rendue en mode « ligne », `\n\n` signalé) → **tombe** avec la logique de l'écran ; la note de la 32-10a dit le contrôle de la description d'agent (AC-ii7) ; R2 = F1 (garde de version irréalisable dans le job Frontend, `ci.yml:218-222` relu : `node-version: 22`, aucune étape Rust) → **garde retirée**. ⚠️ Prémisses **non vérifiées et devenues sans objet** : la version d'Unicode de l'ICU de Node 22 (R2 : « peut être inférieure » ; F1 : « 15.1/16 ») — `node` interdit, aucune mesure ; écrit tel quel. **MEDIUM** : R3 (le drapeau `standardDiffere` vrai pour tout standard personnalisé, AC-i5 de la 32-10a l. 289-296 relue : confirmé) → *Personnalisé* et « Diffère du standard de cette version de Kesh » distingués, le drapeau seul conditionne *Revenir au texte standard*, C-32-10b-19 ; R4 → champs de D1 étendu, **contrainte des primitives** écrite (Dev Notes) ; R5 = F7 → E2E (d) « **deux** versions de plus », compte **relatif** `k → k + 2`, E2E (a) borné à la ligne du nom unique ; R6 (+ F6) → fixture de `refus_apres_confirmation` écrite (n° 3 à valider, n° 4 dernière, chemin `…/versions/3/refuse`) ; R7 = F2 → le dialogue de retour au standard montre `standardDuCode` **avant** le geste, test `dialogue_standard_montre_le_texte_du_code`, M12 réécrite, E16 ; F3 → verdict `servable` du serveur : avis de l'AC-6, *Approuver* absent sur une version non servable (C-32-10b-17, M4 réécrite), bloc « reçoit aujourd'hui » seulement si servable, **avis permanent de l'AC-4 rendu exact** pour une version à valider ; F4 (+ R8) → tests **au niveau du DOM** (`pre.textContent === servi.texteVisible`, `pre.contains(mark)`, pastille hors du `<pre>`), **règle des blancs de gabarit** écrite, M3 et M13 réécrites (M13 ne peut plus passer à vide : elle rompt aussi l'égalité de `textContent`). **LOW** : R8 (intégré à F4) ; R9 = F9 → angle mort écrit (AC-13, § *Ce que les gardes ne voient pas*) ; R10 → relevé confirmé (`audit_route_registry.rs:895` → 130, `:901` → 124) : « 122 des 128 » déjà faux de deux, fait 8 et AC-19 rectifiés, à recompter au développement ; R11 → avis `mcp-prompt-mark-missing`, la description suit la règle du texte ; R12 → `modifieLe` (D1 étendu) ; R13 → **tombe** (compteurs retirés, C-32-10b-8 annoté) ; R14 → fait 13, patron `backup-schedule.spec.ts:200-217` (relu : `POST /api/v1/users`, `role: 'Comptable'`, aucun helper ; `ls helpers/` cité) ; R15 → `version_a_valider_jamais_repliee` défini à l'AC-7, `marque_dans_le_bloc` exige l'intérieur du `<pre>` ; R16 → **en partie réfuté** : `admin-manual.tex:2935` est **juste sur la base de la fiche 32-10a** (`grep -n "routes à verbe mutant"` dans `kesh-32-10a` → l. 2935, « 108 des 114 ») ; sur `main` fusionné la phrase est l. 2978 — demande D5 à l'orchestrateur (lignes citées par la 32-10a : 1008, 1331, 1384, non 1383) ; F5 → **tombe** (plus de pastille ; l'imitation d'un `\u{…}` littéral est l'angle mort E17, distingué par `servable`) ; F6 → intégré (R6, R15) ; F7 → intégré (R5) ; F8 → règle d'envoi en cours sur chaque geste, test `un_seul_post_pendant_l_envoi`, M17, C-32-10b-18 ; F10 (a) → **tombe** (les espaces admises ne sont plus rendues : point 3 pour Guy reformulé) ; F10 (b) → dialogue « Une fois approuvé, ce texte sera servi… ». **Questions pour Guy** : 1 et 2 **tranchées par l'orchestrateur** (oui, oui) en attendant Guy ; 3 **reformulée** (le serveur rendant visible, accepter que les caractères admis ne soient pas montrés ?). **Registre** : C-32-10b-1, 3, 4, 8, 10 annotés ; C-32-10b-17 à 19 ajoutés. **Propagation** : greppés sur la fiche et le registre — `version_unicode`, `UNICODE_VERSION`, `is_hidden_in_line`, `trois versions`, `standard a changé`, `caracteres-caches`, `segmenter`, `reconstituer`, `LISTES_KESH_CORE`, `listes_egales`, `variante B`, `⟦`, `[data-cache]`, `process.versions`, `N / 16`, « comptes de l'AC-6 » ; ne restent que l'entrée de création du Change Log (non réécrite), les annotations datées du registre et les mentions de retrait. **Décomptes** (recomptés par `grep`) : 20 AC (A 4, B 2, C 8, D 6), 9 tâches (T0 à T8), **17 mutations** (M1 à M17 ; M2, M3, M4, M12, M13 réécrites), inventaire **E1 à E17**, 7 points pour Guy, **5 demandes** (D1 et D1 étendu, D2 acceptées ; D3, D4, D5), C-32-10b-1 à **19** ; **3 modules**, inchangés. **1 passe sur 8.** Prompt de la passe suivante : `3210b-validate-prompt-p2.md` (**Opus**, R et F, axe obligatoire des régressions de cette remédiation). |
