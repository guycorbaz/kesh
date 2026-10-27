# Story 32.10b : l'écran des prompts du serveur MCP

Status: review  <!-- développement terminé le 2026-10-27 ; revue de code P1 à lancer (32-10b-review-prompt-p1.md, Sonnet, B, E, A) -->

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
fiche 32-10a (branche `story/32-10a`, commits `e7a90572` puis **`20a02a86`**, note « D1 étendu, complément », § *À
recontrôler à l'étape 0* ; relue à **`8e7ea39d`**, qui ne change que la dernière puce de la note) : la route `GET /api/v1/admin/mcp/prompts/{id}` rend, pour chaque version, le verdict
**`servable`** du serveur, le texte et la description servis, chacun **exact** et **rendu visible par le serveur**
(`rendre_visible`) — la marque fait partie du texte servi, **sans champ séparé** —, pour un standard le texte et la
description du code, exacts et rendus visibles, les **noms** de l'auteur et du décideur, `creeLe`, `decideLe`,
`numeroEnVigueur` et la **date** de dernière modification. **L'écran n'a donc aucune logique Unicode** : il affiche ce que le serveur a rendu visible et le verdict
du serveur (validation P1). Si l'étape 0 constate que l'amendement manque en tout ou partie, le développement
**s'arrête** et rend compte à l'orchestrateur — il n'y a plus de variante de repli côté écran (C-32-10b-1, annoté).

## Story

En tant que Guy, Administrateur d'une installation où un agent lira mes livres, je veux, dans **Administration →
Prompts du serveur MCP**, voir chaque prompt et **tout son historique**, lire **exactement** le texte que l'agent
recevra — marque d'un prompt d'agent comprise, certains caractères invisibles écrits en code —, approuver ou refuser une version proposée par
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
| `GET /…/{id}` : le prompt et **tout** son historique, textes compris ; **par D1 étendu et son complément** (note d'étape 0, `e7a90572`, `20a02a86`) : par version, `servable`, texte et description servis, chacun exact et rendu visible (marque comprise, sans champ séparé), `creeLe`, `decideLe`, noms de l'auteur et du décideur, nom et révocation de la clé auteur ; `numeroEnVigueur`, `standardDiffere` ; pour un standard, le texte et la description du code, exacts et rendus visibles ; la date de dernière modification ; la composition dans `services/` | AC-iii6 ; `historique` ; D1, D1 étendu | le détail (AC-4 à AC-8, AC-10) |
| `POST` (nom, description, texte → 201) | AC-iii6, AC-iii8 | la création (AC-11) |
| `POST /{id}/versions` (texte **ou** retrait, `version` attendue → 201) | AC-iii6 | l'édition, le retrait (AC-9) |
| `POST /{id}/versions/{numero}/approve` et `…/refuse` (`version` attendue → 200) | AC-iii6, AC-i7 | la décision (AC-7) |
| `POST /{id}/standard` (`version` attendue → 201) | AC-iii6, AC-iii11 (d) | le retour au standard (AC-10) |
| 404 autre société ; 409 verrou périmé, version non la dernière, déjà décidée, standard inconnu ; 400 validation (messages `t(…)`, quatre locales), et 400 au refus d'approuver une version non servable (D2, « le même code que la création », `20a02a86`) | AC-iii6, AC-iii8 | les erreurs (AC-12) |
| Comptable 403, PAT 403, clé MCP 401 | AC-iii7 | les rôles (AC-14) |
| la marque d'un prompt d'agent, en tête de la description **et** du texte servis, suivie d'une ligne vide | AC-ii7, *Dev Notes — La marque* | rien de propre : elle se lit dans le texte servi rendu visible, à sa place (AC-5) |
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
   marque ; elle fait partie du champ que rend le serveur, si bien que l'écran la montre **dans** le texte, à sa place,
   sans aucun traitement propre. La forme servie est calculée par le serveur, par la fonction de composition du
   catalogue, dans `services/` (D1, note `20a02a86`) ; l'E2E prouve que `prompts/get` rend **exactement**
   ce que la page montrait avant l'approbation (AC-16).
2. **On rend visible ce qui ne se voit pas — par le serveur** (AC-6). Le serveur écrit en code, sous la forme
   `\u{XXXX}`, certains caractères invisibles (`rendre_visible`, fait 12 ; ceux qu'il admet restent invisibles,
   § *Ce que les gardes ne voient pas*) et dit si la version est **servable** (`valider_contenu`) ; l'écran
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
4. **AC-4 — le détail : l'en-tête et le texte servi.** En-tête : nom, *Standard* le cas échéant, la
   version en vigueur (ou « Non servi »), la date de dernière modification (champ de D1 étendu, `modifieLe`). Puis un
   bloc **Texte servi par Kesh** (titre sans promesse d'actualité : il ne suppose pas le serveur MCP activé ;
   validation P3, F6) : la description servie et le texte servi de la version en vigueur
   (AC-5), **si et seulement si** elle est un texte et que le serveur la dit `servable` ; sinon la phrase « Ce prompt
   n'est servi à aucun agent » et sa raison — « retiré » ; « aucune version approuvée » ; « sa version en vigueur
   n'est pas valide : Kesh refuse de la servir (elle a pu être restaurée d'une sauvegarde ou écrite hors de Kesh) »
   (`servable` faux, AC-ii2 et l. 665 de la fiche 32-10a) — et, dans ce dernier cas, le texte rendu visible montré
   **sous** la phrase, titré « Texte de la version en vigueur, non servi ». Puis les **versions à valider** (AC-7),
   puis l'**historique** (AC-8), puis les gestes (AC-9, AC-10). Avis permanent en tête (`role="note"`,
   `data-testid="mcp-prompt-notice"`), réduit à **une phrase sans énumération** (validation P3, R3, R4, F3,
   R8), affichée **au-dessus du seul bloc *Texte servi par Kesh*** — ni sur une version non servable, ni sur une version
   refusée, ni dans l'éditeur (validation P4, H5) : « Le texte est montré tel que Kesh le servirait ; certains caractères invisibles y sont écrits sous la forme
   `\u{…}`. » Il n'énumère aucun caractère. Au catalogue, les accolades se produisent par un **paramètre**, comme
   l'avis de la 32-8c (`mcp-sends-notice`, paramètre `{ $code }`, `McpSendDetail.svelte:190-194`) : `… sous la forme
   { $code }.`, appelé avec `code: '\\u{…}'`. Tests : `bloc_servi_seulement_si_servable` (en vigueur `servable:
   false` → la phrase, aucun élément de zone `servi-` : le test juge par le préfixe, non par le titre) ; `avis_permanent_exact` — le `textContent` de l'avis
   **égal** à un attendu **écrit en dur** dans le test (la phrase ci-dessus, `\u{…}` compris), non lu du catalogue.

### B — Ce qui est montré : le texte servi, la marque, les caractères cachés

5. **AC-5 — on montre le texte servi, tel que le serveur l'a rendu visible** (C-32-10b-1, C-32-10b-2 ; D1 étendu et
   son complément, note `20a02a86`).
   - **Les champs** (noms indicatifs, alignés à l'étape 0 sur le DTO réel) : pour chaque version de nature *texte*, la
     réponse de `GET /…/{id}` porte `servi: { description, texte, descriptionVisible, texteVisible }` —
     `description` et `texte` **exactement** ce que `prompts/list` et `prompts/get` rendraient si cette version était
     en vigueur, **marque comprise** ; `descriptionVisible` et `texteVisible` les mêmes passés par `rendre_visible`
     (fait 12) — et `servable: boolean`, le verdict de `valider_contenu` sur le contenu stocké. **La marque n'a pas de
     champ séparé** : elle fait partie du texte et de la description servis. Le serveur les calcule par **la même
     fonction de composition** que le catalogue, qui vit dans `services/` (note `20a02a86`) : **aucun texte servi n'est
     recomposé côté écran**, et l'écran ne transforme aucun de ces champs.
   - **Ce qui sert à quoi.** Le texte **rendu visible** sert à l'**affichage**, et à lui seul ; le texte **exact**
     n'est jamais affiché — il sert à l'**égalité avec `prompts/get`** dans l'E2E (AC-16), et à lui seul ; le texte
     **stocké** (`description`, `texte` de la version) sert à l'**éditeur** seul (AC-8, AC-9). L'éditeur ne se
     pré-remplit **jamais** avec le texte servi (validation P3, R1).
   - **Le rendu** (`McpPromptVersion.svelte`) : deux blocs `<pre class="whitespace-pre-wrap break-words font-mono"
     dir="ltr">`, `data-testid="<zone>mcp-prompt-served-description-{numero}"` et
     `"<zone>mcp-prompt-served-text-{numero}"`, dont le contenu est l'**interpolation seule** du champ visible. La
     marque d'une version d'agent s'y lit à sa place, en tête, **sans traitement de l'écran** — ni encadré, ni
     découpe, ni comparaison (validation P2, R1, F6 : l'encadré séparé est retiré). La pastille *Écrit par un agent*,
     tirée de l'**origine** de la version, est **hors** du `<pre>`, au-dessus.
   - ⛔ **La règle des `data-testid`, écrite ici une fois** (validation P3, F1) : une même version peut être montée
     dans plusieurs zones de la page ; chaque `data-testid` porté par une version (ses deux `<pre>`, l'avis de l'AC-6)
     est donc **préfixé par la zone** qui la monte — `servi-` (le bloc *Texte servi par Kesh*), `non-servi-` (le texte non
     servi montré sous sa phrase), `a-valider-` (AC-7), `historique-` (AC-8) —, que `McpPromptVersion` reçoit en
     propriété. Un test ou un E2E vise la zone qu'il veut.
   - ⛔ **La règle des blancs de gabarit** (validation P1, F4, R8) : chaque `<pre>` s'écrit d'un tenant, sur une ligne
     — `<pre …>{servi.texteVisible}</pre>` —, sans blanc, retour à la ligne ni indentation entre les balises et
     l'interpolation, car un `<pre>` affiche les blancs du gabarit comme du texte. Aucun bloc `{#if}` ni `{#each}`,
     aucun libellé, pastille ni titre à l'intérieur.
   - **Tests au niveau du DOM** (`McpPromptVersion.test.ts`, composant monté, jsdom) :
     `texte_affiche_egal_au_champ_visible` — sur une version d'**agent** (origine fixée : la marque est dans le champ),
     fixture **multi-paragraphe** dont `texteVisible` commence par un texte de marque suivi de `\n\n`, à blanc de tête
     après la marque, tabulation, ligne vide finale et un caractère rendu visible (`texte` portant U+202E,
     `texteVisible` portant `\u{202E}` à sa place) : `pre.textContent === servi.texteVisible` **exactement**,
     `pre.textContent !== servi.texte`, et la chaîne exacte `servi.texte` n'apparaît nulle part dans le `textContent`
     du composant ; idem pour la description (`description_affichee_egale_au_champ_visible`, même version d'agent) ;
     `pastille_agent_hors_du_bloc` — la pastille *Écrit par un agent* n'est **pas** descendante du `<pre>`, et elle est
     absente pour une version `standard` ou `admin`.
6. **AC-6 — les caractères cachés et le verdict : ce que dit le serveur, rien d'autre** (C-32-10b-3, C-32-10b-4,
   annotés à la validation P1).
   - L'écran n'a **aucune** logique Unicode : ni copie des listes de `kesh-core`, ni classe `\p{…}` du moteur
     JavaScript, ni compte de caractères cachés calculé dans le navigateur, ni garde de version d'Unicode. Un caractère
     caché paraît à l'écran **parce que le serveur l'a écrit** `\u{XXXX}` dans le champ visible (fait 12) ; la preuve
     que l'écran affiche ce champ sans transformation est le test `texte_affiche_egal_au_champ_visible` (AC-5).
   - **Le verdict** : sur chaque version de nature texte dont `servable` est faux, un avis (`role="alert"`,
     `data-testid="<zone>mcp-prompt-not-servable-{numero}"`, règle des `data-testid` de l'AC-5) : « Kesh ne servirait
     pas cette version : son contenu n'est pas valide (caractère invisible, longueur, contenu vide ou description sur
     plusieurs lignes). Elle a pu être restaurée d'une sauvegarde ou écrite hors de Kesh. » (la note de la 32-10a ne
     promet pas de motif : l'écran n'en affiche pas ; les quatre cas sont ceux de `valider_contenu`, AC-ii2 de la
     32-10a, l. 635-640 ; validation P3, R6).
   - Tests (`McpPromptVersion.test.ts`) : `avis_non_servable_selon_le_serveur` (`servable: false` → l'avis ;
     `servable: true` sur un texte dont `texteVisible` porte une suite `\u{202E}` littérale → **aucun** avis : l'écran
     ne juge pas le texte, il lit le verdict).

### C — Les gestes

7. **AC-7 — la version d'agent à valider : on approuve ce qu'on a vu** (C-32-10b-5, C-32-10b-6).
   - Chaque version `agent` `pending` est montrée **en entier** — description et texte servis (AC-5, AC-6), jamais
     tronqués, jamais repliés, sans « voir plus » — avec : son numéro, la clé auteur (numéro et nom de la clé ;
     « révoquée » le cas échéant, champs de D1), la date de la proposition (`creeLe`), et la pastille *Écrit par un
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
     **précédé de la marque** » ; boutons *Annuler* et *Approuver la version n° N*.
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
   - ⛔ **Règle d'envoi en cours, un seul état par page** (validation P1, F8 ; validation P2, F3, F8, R13 ;
     C-32-10b-18 ; modèle de la 32-8c, `McpSendDetail.svelte:47`, `:92`, `:309-368`) : la page de détail tient **un**
     état `envoiEnCours` pour **tous** ses gestes — *Approuver*, *Refuser*, *Enregistrer comme nouvelle version*,
     *Retirer*, *Revenir au texte standard*, les boutons de leurs dialogues — ; dès le clic de confirmation, **tous**
     sont désactivés (`disabled`) jusqu'à la réponse, et le bouton cliqué porte `aria-busy="true"` (précédent :
     `admin-backup/` ; la 32-8c n'en porte pas). Pendant l'envoi, un dialogue ouvert ne se ferme pas : les trois
     réglages du dialogue de la 32-8c (`McpSendDetail.svelte:345-347`) — `escapeKeydownBehavior` et
     `interactOutsideBehavior` à `'ignore'`, `showCloseButton` faux (validation P3, F10). Le gestionnaire garde son
     retour anticipé `if (envoiEnCours) return` : Svelte appelle le gestionnaire délégué même sur un bouton désactivé
     (validation P3, R5, F4), si bien que `disabled` ne suffit pas. La page de la liste fait de même pour *Créer*. Test
     paramétré `gestes_desactives_pendant_l_envoi` (`McpPromptDetail.test.ts`, un cas par geste ; un cas *Créer* dans
     `McpPromptsList.test.ts`) : un `POST` simulé laissé en attente, **chaque** bouton de geste de la page est
     `disabled`, le bouton cliqué porte `aria-busy="true"` ; puis **deux** événements `click` envoyés par `fireEvent`
     au bouton de confirmation, et **un seul** `POST` attendu (M17, M18).
8. **AC-8 — l'historique, jamais réécrit.** Un tableau de **toutes** les versions, plus récente en tête : numéro ;
   origine (*Standard* ; *Administrateur* — nom de l'auteur ; *Agent* — clé n° K, pastille d'avertissement) ; état
   (*Approuvée*, *À valider*, *Refusée*) ; nature (*Texte*, *Retrait*) ; la décision d'une version d'agent (par qui,
   quand — `decideurNom` et `decideLe`, champs de D1 étendu) ; la date (`creeLe`) ; la pastille **En vigueur** sur la
   version dont le numéro est `numeroEnVigueur`, rendu par le serveur — **l'écran ne recalcule rien** (validation P2,
   R5 ; C-32-10b-7 annoté). Chaque version de nature *texte* se **déplie** pour montrer son texte servi (AC-5, AC-6) — le
   repli de l'historique est permis, celui d'une version **à valider** ne l'est pas (AC-7). Action **Reprendre cette
   version** sur une version de nature *texte* : pré-remplit l'éditeur (AC-9) avec sa description et son texte
   **stockés** (C-32-10a-19 : « l'écran pré-remplit, l'administrateur enregistre »). Tests : chaque origine, état et
   nature rendus ; `en_vigueur_selon_le_serveur` (la pastille sur la version de numéro `numeroEnVigueur` et sur elle
   seule, fixtures où une `pending` et une `refused` sont postérieures et où un retrait est en vigueur ; aucune
   pastille quand `numeroEnVigueur` est nul) ; `marque_visible_dans_l_historique` (la pastille *Agent* sur chaque version d'agent, décidée ou non).
9. **AC-9 — écrire une nouvelle version, ou retirer** (C-32-10b-8).
   - **Éditeur** (`McpPromptEditor.svelte`) : un champ *Description* (une ligne) et une `<textarea>` *Texte* en police à
     chasse fixe, `dir="ltr"`, `spellcheck="false"` ; pré-remplis par la description et le texte **stockés** de la
     version en vigueur si elle est un texte, ou par *Reprendre cette version* — jamais par le texte servi (AC-5).
     **Après toute relecture réussie** (décision, geste, *Recharger*), l'éditeur est **réinitialisé** sur
     la version alors en vigueur, avis P26 compris, comme `poser()` de la 32-8c (`McpSendDetail.svelte:56`) ; validation
     P3, F5. Une aide sous les champs dit les bornes de l'AC-i3 de la 32-10a (500 caractères pour la
     description, 16 000 pour le texte, relues à l'étape 0) **sans compteur ni contrôle côté écran** ; ni aperçu des
     caractères cachés de la saisie (validation P1 : l'écran n'a aucune logique Unicode, et un compteur de « caractères »
     en demanderait une — R13 tombe). Un caractère caché tapé ou collé ne s'écrit pas : le serveur le refuse (400, son
     message affiché, saisies conservées, AC-12) ; rien de non vu n'entre donc à l'historique. Bouton *Enregistrer
     comme nouvelle version* → `POST /{id}/versions` avec `corpsTexte(version lue,
     description, texte)`, valeurs **exactes**. Une phrase sous le bouton : « Une nouvelle version n° N+1 est créée ; les
     versions précédentes restent à l'historique. »
   - **P26, rendu visible** : quand l'éditeur est pré-rempli depuis une version `agent` — **d'office**, par la version
     en vigueur, comme par *Reprendre cette version* —, un avis (`role="note"`,
     `data-testid="mcp-prompt-editor-agent-origin"`) : « Ce texte a été écrit par un agent. L'enregistrer en fait **votre**
     version : elle sera servie **sans** la marque d'un prompt d'agent. Relisez-le comme s'il venait de vous. »
   - **Retirer** : bouton *Retirer ce prompt* (variante destructive), dialogue : « Le prompt cessera d'être servi. Une
     version n° N+1 de retrait est ajoutée ; l'historique reste. Une nouvelle version le servira de nouveau. » → `POST
     /{id}/versions` avec `corpsRetrait(version lue)`. Absent si la version en vigueur est déjà un retrait.
   - Tests (`McpPromptEditor.test.ts`) : `envoie_la_valeur_exacte` ; `avis_p26_si_pre_rempli_par_une_version_d_agent`
     (deux cas présents : le pré-remplissage d'office par une version d'agent en vigueur, et *Reprendre cette version*
     sur une version d'agent ; absent depuis une version d'administrateur ou standard ; validation P2, F12) ;
     `editeur_reinitialise_apres_relecture` (`McpPromptDetail.test.ts` : éditeur pré-rempli par une v3
     d'administrateur et modifié ; approbation de la v4 d'agent ; après la relecture, l'éditeur porte la description et
     le texte **stockés** de la v4, et l'avis P26) ; `retrait_apres_confirmation` ;
     `aucun_controle_bloquant_cote_ecran` (un texte de 16 001 caractères et un U+202E partent ; le 400 simulé est
     affiché).
10. **AC-10 — revenir au standard, sans rien écraser, et sur un texte vu** (C-32-10b-10, annoté). Le bouton *Revenir
    au texte standard* n'apparaît que si **`standardDiffere && standardDuCode !== null`** (validation P2, F5, R11) — le
    drapeau, par sa définition (AC-i5 de la 32-10a), couvre la version en vigueur personnalisée par l'administrateur,
    le retrait et le standard non personnalisé dont le texte diffère de celui du code (à recontrôler au code à l'étape
    0) ; `standardDuCode` nul — prompt non standard, ou standard que ce binaire ne porte plus, que la route refuserait
    (409 « standard inconnu du code ») — retire le bouton. *(Un agent ne propose jamais sur un nom en `kesh`,
    C-32-10a-27 : une version d'agent approuvée sur un standard n'existe pas, et ne se teste pas.)* Le dialogue montre
    **d'abord le texte que le geste fera servir** (validation P1, R7, F2) — les champs visibles de `standardDuCode`
    (D1 étendu et son complément), rendus comme à l'AC-5 dans deux `<pre>`
    (`data-testid="mcp-prompt-standard-description"`, `"mcp-prompt-standard-text"`, même règle des blancs de gabarit) —,
    puis : « Une nouvelle version n° N+1 reprend ce texte, le standard de cette version de Kesh. Vos versions et celles
    de l'agent restent à l'historique, et vous pourrez reprendre l'une d'elles. Le prompt redevient *non personnalisé* :
    il suivra désormais le texte des prochaines versions de Kesh. » → `POST /{id}/standard` avec `corpsStandard(version
    lue)`. Après 201, relecture : la nouvelle version paraît en tête, d'origine *Standard*, en vigueur. Tests :
    `retour_au_standard_seulement_si_utile` (présent : personnalisé par l'administrateur, retiré, standard non
    personnalisé au drapeau levé ; absent : non standard, standard à jour, standard au drapeau levé dont
    `standardDuCode` est nul) ; `retour_au_standard_apres_confirmation` ;
    `dialogue_standard_montre_le_texte_du_code` (`textContent` du `<pre>` du dialogue **égal** à
    `standardDuCode.texteVisible`, et le dialogue ouvert avant tout `POST`).
11. **AC-11 — créer un prompt** (C-32-10b-16). Formulaire *Nouveau prompt* (liste) : nom, description, texte. Aide sous le nom :
    « Lettres minuscules sans accent, chiffres, `-` et `_`, 64 au plus ; un nom ne commence pas par `kesh`, réservé aux
    prompts standard. » (règle de l'AC-i2 de la 32-10a, **non contrôlée côté écran** : le 400 du serveur est affiché).
    `POST` → 201 → navigation vers le détail du prompt créé. Tests : corps exact ; 400 affiché, saisies conservées.
12. **AC-12 — les erreurs** (patron de `erreurDeDecision`, fait 2 ; codes relus à l'étape 0). 409 (verrou périmé,
    version non la dernière, déjà décidée, standard inconnu du code) → le message (celui du serveur s'il est traduit,
    sinon « Ce prompt a changé depuis son affichage : rechargez-le avant d'agir ») **et** le bouton *Recharger*, qui
    relit le détail et réinitialise l'éditeur sur la version en vigueur (AC-9) : une saisie non enregistrée est perdue ; **aucune** relance automatique. 404 → « Prompt introuvable ». 400 → le message du serveur, saisies
    **conservées** — y compris le refus d'approuver une version non servable (D2 : **400**, « le même code que la
    création », note `20a02a86` ; validation P2, R4). 403 → « Réservé à l'Administrateur ». Le reste → `errorMessageOf`. Tests par code ;
    `aucune_relance_automatique_sur_409`.
13. **AC-13 — un texte non fiable se montre comme du texte.** Textes, descriptions, noms de clé et marques ne passent
    que par une interpolation Svelte ou la valeur d'un champ — **jamais** `{@html`. Garde `ecran_sans_html_brut` (règle
    commune), sur le patron de la 32-8c (fait 3) — *propriété* : aucune source de `frontend/src/lib/features/
    mcp-prompts/` ni de `frontend/src/routes/(app)/admin/mcp/prompts/` ne contient `{@html` ; *mesure* : `grep -rn
    '{@html' frontend/src/lib frontend/src/routes` → **0** au relevé (à recompter) ; *témoin vert existant* :
    `{@const cancelled = r.cancelledAt !== null}` (`src/lib/features/reminders/ReminderHistory.svelte:51`) ; *témoin
    rouge factice*, **composé par une constante** comme à la 32-8c (`mcp-sends-i18n.test.ts:231`, `HTML_BRUT`), pour
    que la garde ne rougisse pas sur son propre fichier (validation P2, R10). Et chaque `<textarea>` et `<pre>` de texte porte `dir="ltr"` ; chaque
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
    *Témoin vert* (factice, dit tel) : un catalogue portant `audit-log-entity-mcp-prompts-x` — qui **contient** le
    préfixe sans commencer par lui, comme `'audit-log-entity-mcp-sends-x'` à la 32-8c (`mcp-sends-i18n.test.ts:136`) —
    n'ajoute rien à l'ensemble du préfixe ; une appartenance par sous-chaîne le ferait rougir (validation P2, R9).
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
      data-testid="a-valider-mcp-prompt-served-text-{n}">`, zone *À valider*, règle de l'AC-5 ; `n` lu dans la page)
      et la description affichée `D` (`a-valider-mcp-prompt-served-description-{n}`) ; il lit aussi, par
      `GET /…/{id}` en session, les champs de cette version et exige `T === servi.texteVisible`, `D ===
      servi.descriptionVisible` — l'affichage est le champ rendu visible — et, ce texte étant sans caractère caché,
      `servi.texteVisible === servi.texte` ; *Approuver*, confirmation ; puis le client appelle `prompts/get` →
      `messages[0].content.text` **égal** à `servi.texte` (le texte **exact**) **et** à `T`, et `description` **égale**
      à `servi.description` et à `D` ; `prompts/list` porte une entrée `{ name, description: D, arguments: [] }`. Et `T`
      commence par `[Prompt écrit par un agent` (la marque, relue à l'étape 0 à son lieu réel : la fonction de
      composition de `services/`, note `20a02a86`).
    - **(b) Refus.** Seconde proposition, **sur un nom unique neuf** (sur le nom de (a), déjà approuvé, `prompts/get`
      ne rendrait pas d'erreur) ; *Refuser*, confirmation ; l'historique montre *Refusée* ; `prompts/get` de ce nom →
      l'erreur `-32602`, son membre `error` **égal** à celui d'un nom inconnu demandé dans le même test (les `id`
      JSON-RPC diffèrent ; validation P2, R14) ; `prompts/list` ne le porte pas.
    - **(c) Nouvelle version d'administrateur** sur le prompt de (a), par *Reprendre cette version* : l'avis P26 est
      visible ; le texte modifié au clavier ; *Enregistrer* ; le texte servi lu dans la page `T2`
      (`servi-mcp-prompt-served-text-{n}`, zone *Texte servi par Kesh*, règle de l'AC-5) **ne commence pas** par
      la marque ; `prompts/get` → **égal** à `T2`.
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
    (5) le détail avec une version d'agent à valider, **la marque en tête du texte** ; (6) le dialogue d'approbation ; (7) le
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
      l'Administrateur : la liste, le détail, le « Texte servi par Kesh » ;
    - l'**historique** : chaque modification crée une version, aucune n'est effacée ni réécrite ; la version en
      vigueur ; le **retrait** ;
    - les **propositions d'un agent** : une clé portant `prompts:write` propose ; la version attend, n'est **jamais**
      servie avant l'approbation ; **aucune clé ne peut approuver** ; un agent ne propose jamais sur un nom en `kesh`
      (C-32-10a-27) ; le **plafond de vingt** propositions en attente par société ;
    - **approuver et refuser** : seule la dernière version s'approuve ; l'approbation porte sur la version **lue** (une
      proposition arrivée entre-temps la rend périmée, *Recharger* ; *Recharger* remet l'éditeur sur la version en vigueur, et la saisie non enregistrée est perdue) ; le refus est définitif ;
    - **la marque** : son texte, sa place en tête de la description et du texte servis, et qu'elle dit **qui a écrit la
      version, non d'où viennent ses mots** ; l'**angle mort P26** : enregistrer un texte d'agent comme sa propre
      version **ôte la marque** — l'écran le dit, le manuel aussi ;
    - **les caractères invisibles** : refusés à l'écriture et non servis s'ils ont été restaurés ou écrits hors de
      Kesh ; montrés à l'écran sous la forme `\u{XXXX}`, **calculée par le serveur** (le même rendu que l'écran des
      envois) ; une version que Kesh ne servirait pas est signalée comme telle et ne s'approuve pas ; un `\u{…}` écrit
      **en toutes lettres** dans un texte s'affiche comme un caractère rendu visible, et l'écran ne les distingue pas
      (E17, angle mort ; validation P3) ; les espaces spéciales (insécable, fine insécable) et les quelques invisibles que Kesh admet **ne sont pas** montrées ;
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
| E2 | la marque montrée à côté du texte, non dedans — l'Administrateur croit qu'elle n'est qu'un ornement | elle fait partie du champ servi que rend le serveur (note `20a02a86`, sans champ séparé) ; l'écran l'affiche par la même interpolation que le reste, sans traitement ; test au niveau du DOM sur une version d'agent | AC-5, M13 |
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
| E17 | un `\u{202E}` écrit **en toutes lettres** par un agent se confond avec un caractère rendu visible | **non fermé** : l'écran affiche les deux sous la même forme, et aucun avis ne prétend les distinguer (validation P3 : la phrase de l'avis qui s'appuyait sur `servable` est retirée) | **angle mort** — dit au manuel (AC-18) |
| E18 | une restauration (ou la sauvegarde d'une autre installation) ramène le compteur `version` du prompt à une valeur déjà lue : l'onglet resté ouvert approuve, sous le même `(numero, version)`, une version d'un autre contenu | **non fermé** : le verrou `version` compte des écritures, il ne désigne pas un contenu | **angle mort** (validation P2, F7) — le même que l'issue #773 (point 15) pour les envois ; le fermer demanderait à la 32-10a une empreinte du contenu vu dans le corps d'approbation, hors de cette story |

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
- [ ] **T8 — mutations M1 à M18, gates, Dev Agent Record** (n'affirmer que ce qui a tourné, avec périmètre), prompt de la
  revue de code P1.

## Tests et mutations prévus

Une mutation à la fois, restaurée par `git checkout -- <f> && touch <f>` (`memory/mutation-restauree-binaire-perime`),
diff résiduel nul vérifié ; chacune nomme le test qui doit rougir ; une mutation qui survit, ou qui ne compile pas, se
déclare telle. *(Validation P1 : M2, M3, M4, M12 et M13 réécrites — les mutations des listes Unicode et des modes de
l'écran tombent avec sa logique Unicode ; M17 ajoutée. Validation P2 : M10, M13 et M17 réécrites ; M2 ne cite plus
l'E2E (a), qui ne monte aucun texte où les deux champs diffèrent, F4. Validation P3 : M18 ajoutée — le retrait du
retour anticipé, déclaré à la P2 comme non joué, se joue : Svelte appelle le gestionnaire délégué même sur un bouton
désactivé, R5, F4.)*

| # | Mutation | Doit rougir |
|---|---|---|
| M1 | l'approbation relit le détail (`GET`) avant le `POST` et envoie la `version` relue | `approbation_porte_la_version_lue` (Vitest) |
| M2 | le bloc servi affiche `servi.texte` (exact) au lieu de `servi.texteVisible` | `texte_affiche_egal_au_champ_visible` |
| M3 | un retour à la ligne et une indentation insérés dans le gabarit, à l'intérieur du `<pre>` (ou `trim()` appliqué au champ affiché) | `texte_affiche_egal_au_champ_visible` |
| M4 | *Approuver* rendu sur une version `servable: false` | `pas_d_approbation_d_une_version_non_servable` |
| M5 | `{@html servi.texteVisible}` dans `McpPromptVersion.svelte` | `ecran_sans_html_brut` |
| M6 | *Approuver* rendu sur toute version `agent` `pending`, dernière ou non | `approuver_seulement_la_derniere` |
| M7 | *Approuver* envoie le `POST` sans dialogue | `approbation_apres_confirmation` |
| M8 | le repli d'une clé `mcp-prompts-*` diffère de sa valeur fr-CH | la garde des replis (AC-15) |
| M9 | `corpsTexte` applique `trim()` au texte | `corps_texte_exact` ; `envoie_la_valeur_exacte` |
| M10 | la condition du bouton *Revenir au texte standard* réduite à `standardDiffere` (sans `standardDuCode !== null`) | `retour_au_standard_seulement_si_utile` (cas « drapeau levé, `standardDuCode` nul ») |
| M11 | le refus envoie le numéro de la **dernière** version au lieu de celle affichée | `refus_apres_confirmation` (fixture n° 3 à valider, n° 4 dernière ; chemin `…/versions/3/refuse` égal) |
| M12 | le dialogue de retour au standard n'affiche pas `standardDuCode` | `dialogue_standard_montre_le_texte_du_code` ; E2E (d) |
| M13 | le composant retire la marque du texte affiché (n'interpole que ce qui suit le premier `\n\n` d'une version d'agent) | `texte_affiche_egal_au_champ_visible` (version d'agent, égalité de `textContent`) |
| M14 | l'avis P26 retiré de l'éditeur | `avis_p26_si_pre_rempli_par_une_version_d_agent` ; E2E (c) |
| M15 | la version à valider rendue repliée (`<details>` fermé) | `version_a_valider_jamais_repliee` (AC-7 : le texte entier est dans le DOM et visible) |
| M16 | `nav-admin-mcp-prompts` retiré des quatre catalogues, `'admin-mcp-prompts'` gardé à `FAMILLES_RESOLUES` | `i18n-keys.test.ts`, « toute clé demandée existe au catalogue » |
| M17 | `disabled={envoiEnCours}` retiré du bouton de confirmation d'*Approuver* | `gestes_desactives_pendant_l_envoi` (cas *Approuver* : l'état `disabled` est asserté) |
| M18 | le retour anticipé `if (envoiEnCours) return` retiré du gestionnaire de confirmation d'*Approuver*, `disabled` gardé | `gestes_desactives_pendant_l_envoi` (cas *Approuver* : deux `click` par `fireEvent`, un seul `POST` attendu) |

Tests hors mutations, nommés au Dev Agent Record : les témoins de chaque garde ;
`description_affichee_egale_au_champ_visible` ; `pastille_agent_hors_du_bloc` ; `avis_non_servable_selon_le_serveur` ;
`bloc_servi_seulement_si_servable` ; `avis_permanent_exact` ; `editeur_reinitialise_apres_relecture` ;
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
  par le serveur (E17) : l'écran ne les distingue pas, et aucun avis ne le prétend (validation P3). Le manuel le dit.
- **L'éditeur pré-rempli depuis une version non servable** (restaurée, ou écrite hors de Kesh ; validation P3, R7) :
  la `<textarea>` montre le texte stocké, caractères invisibles compris, sans les rendre visibles, et le 400 du serveur
  ne dit pas où ils sont. L'Administrateur retape, ou s'aide du texte rendu visible que montre le détail. Rien de non
  vu n'est écrit : le serveur refuse.
- **Les lettres bidirectionnelles visibles** (arabe, hébreu) : `dir="ltr"` fixe le sens de base, l'algorithme
  bidirectionnel les réordonne encore avec les signes voisins (validation P1, R9, F9). Les **contrôles** bidi, eux,
  sont rendus `\u{…}`.
- **Les homoglyphes** (un `а` cyrillique), visibles et trompeurs ; un texte **visible** qui dit de mal faire (P27 de la
  32-10a) ; une **imitation de la marque** écrite par l'agent après la vraie (validation P3, F8 — la vraie est la
  première, et ses numéros de version et de clé sont ceux du dialogue) : l'écran le montre, il ne le juge pas.
- **Le texte du code entre l'affichage et le clic** (E16) : un binaire mis à jour pendant que le dialogue de retour au
  standard est ouvert fait servir un texte que le dialogue ne montrait pas — la `version` du prompt ne bouge pas.
- **La marque dit qui a écrit, non d'où viennent les mots** (P26, E9) : l'avis de l'éditeur et le manuel le disent ;
  rien ne l'empêche.
- **La justesse des champs de D1 étendu** est tenue par les tests de la 32-10a-iii ; les tests Vitest de l'écran ne la
  voient pas (réponses simulées). Le test au niveau du DOM couvre l'**affichage** (le `textContent` égal au champ
  visible) ; l'E2E couvre l'**égalité du texte exact avec `prompts/get`** (a, c, d). **Aucun E2E ne vérifie le champ
  rendu visible** (`texteVisible` égal à `rendre_visible(texte)`) : le serveur refuse à l'écriture tout texte où les
  deux champs diffèrent, si bien qu'un E2E à caractère caché passerait par une version restaurée — trop lourd pour
  cette story (validation P2, F4). La capture (10) de l'AC-17, sur une version posée par `INSERT`, en est le seul
  témoin de bout en bout, visuel.
- **La liste ne juge pas ; le détail le dit** (validation P2, R8) : la synthèse ne porte aucun verdict `servable`, si
  bien qu'un prompt dont la version en vigueur n'est pas valide (restaurée) paraît « n° N » dans la liste ; seul le
  détail dit « n'est servi à aucun agent ».
- **Un retour du compteur `version` par restauration** (E18, validation P2, F7) : le verrou compte des écritures, il ne
  désigne pas un contenu ; un onglet resté ouvert peut approuver, sous le même `(numero, version)`, une version d'un
  autre contenu. Même angle mort que l'issue #773 (point 15) pour les envois.
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

/** Ce que `prompts/get` servirait si la version était en vigueur (D1, D1 étendu et son complément) —
 *  marque comprise, sans champ séparé (note `20a02a86`). */
export interface ServiPrompt {
	description: string; texte: string;                  // exacts — E2E seulement : jamais affichés, jamais dans l'éditeur
	descriptionVisible: string; texteVisible: string;    // rendus visibles par le serveur — seuls affichés
}

export interface McpPromptVersion {
	numero: number; origine: Origine; etat: Etat; nature: Nature;
	description: string | null; texte: string | null;   // stockés — l'éditeur seul (pré-remplissage, Reprendre cette version)
	servi: ServiPrompt | null;                          // nul pour un retrait
	servable: boolean | null;                           // verdict de `valider_contenu` ; nul pour un retrait
	cleAuteur: { id: number; nom: string; revoquee: boolean } | null;
	auteurNom: string | null; decideurNom: string | null; decideLe: string | null; creeLe: string;
}

export interface McpPromptDetail {
	id: number; nom: string; isStandard: boolean; version: number; modifieLe: string;
	numeroEnVigueur: number | null; standardDiffere: boolean;
	standardDuCode: ServiPrompt | null;   // nul hors standard, ou pour un standard que ce binaire ne porte plus
	versions: McpPromptVersion[];
}
```

### Les amendements D1, D1 étendu et D2 — acceptés, inscrits à la fiche 32-10a

Acceptés par l'orchestrateur et inscrits au § *À recontrôler à l'étape 0* de la fiche 32-10a (commits `47394982` pour
D1 et D2, `e7a90572` pour D1 étendu, `20a02a86` pour son complément, branche `story/32-10a` ; la note « D1, champs du détail » à `a157255b`, l. ≈1541 — LOW J1 de la validation P5, inscrit à l’étape 0 du développement). Ce que l'écran en attend, pour le recontrôle de son
étape 0 :

- **D1** : par version de nature texte, le texte et la description **tels que `prompts/get` et `prompts/list` les
  rendraient**, marque comprise, calculés par **la même fonction** que le catalogue ; `numeroEnVigueur`,
  `standardDiffere` ; le nom de la clé auteur et sa révocation. Test serveur : `servi.texte` **égal** au
  `messages[0].content.text` de `prompts/get` pour une version d'agent approuvée.
- **D1 étendu** : (a) `servable`, le verdict de `valider_contenu` (la fonction de la création et de l'approbation) ;
  (b) les mêmes champs passés par `rendre_visible` ; (c) pour un standard, le texte du code (`STANDARDS`) qu'un
  retour au standard ferait servir, rendu visible ; (d) les **noms** de l'auteur et du décideur, et la **date** de
  dernière modification (validation P1, R4, R12).
- **D1 étendu, complément** (`20a02a86`, validation P2) : le texte **et la description** servis, chacun exact et rendu
  visible, la marque comprise dans le texte servi (**pas** de champ `marque` séparé) ; de même pour le standard du
  code (texte et description) ; `creeLe` et `decideLe` de chaque version, et `numeroEnVigueur`. **La fonction de
  composition du texte servi, marque comprise, vit dans `services/`**, appelée par le catalogue MCP et par la route de
  détail ; le refus de D2 rend **400**.
- ⛔ **Contrainte des primitives** (validation P1, R4 ; fiche 32-10a, validation P1 R1/F1, l. 666-670) : la fonction de
  composition prend des **primitives** (`i64`/`u32`, `&str`) — **aucun** type de `kesh_db` ne sort du service vers
  `src/mcp/` ; `PromptEnBase` et la route l'appellent tous deux, et elle vit dans `services/` (note `20a02a86`). La description d'une version d'agent contient `\n\n`
  après la marque : le contrôle qui s'y applique est celui de l'AC-ii7, non le prédicat d'une ligne (validation P1,
  R1 : sans quoi chaque description d'agent saine serait dite non valide).
- **D2** : la route d'approbation revalide le contenu par `valider_contenu` et refuse une version non servable
  (**400**, le même code que la création, `20a02a86`) ; l'écran n'offre déjà pas le geste (C-32-10b-17).

### Esquisse des fichiers (indicative)

```text
frontend/src/lib/features/mcp-prompts/mcp-prompts.api.ts        — routes, fabriques de corps (seules)
frontend/src/lib/features/mcp-prompts/mcp-prompts.types.ts      — types des réponses
frontend/src/lib/features/mcp-prompts/mcp-prompts-format.ts     — badges, dates, erreurs (fonctions pures)
frontend/src/lib/features/mcp-prompts/McpPromptsList.svelte     — liste, création
frontend/src/lib/features/mcp-prompts/McpPromptDetail.svelte    — en-tête, texte servi par Kesh, à valider, historique
frontend/src/lib/features/mcp-prompts/McpPromptVersion.svelte   — rendu d'un texte servi visible (interpolation seule, verdict)
frontend/src/lib/features/mcp-prompts/McpPromptEditor.svelte    — nouvelle version, retrait, avis P26
frontend/src/lib/features/mcp-prompts/*.test.ts                 — Vitest, gardes
frontend/src/routes/(app)/admin/mcp/prompts/+page.svelte        — mince
frontend/src/routes/(app)/admin/mcp/prompts/[id]/+page.svelte   — mince, {#key id}
frontend/tests/e2e/mcp-prompts.spec.ts                          — AC-16
```

### Ce qui est lu et cité

`frontend/src/routes/(app)/+layout.svelte:147-176`, `:171-175`, `:180-185` ;
`frontend/src/lib/features/mcp-sends/mcp-sends.api.ts:1-11`, `:36`, `:45`, `:50`, `:59` ;
`mcp-sends-i18n.test.ts:46`, `:47`, `:52`, `:59`, `:101`, `:136`, `:219`, `:231` ; `McpSendDetail.svelte:47`, `:92`,
`:56`, `:190-194`, `:202`, `:220-227`, `:309-368`, `:345-347` (validation P3) ; issue #773 (point 15, lecture) ;
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
1534-1541 (D1, D2, D1 étendu, D5, `\r` restauré, D1 étendu complément ; branche `story/32-10a` à `20a02a86`, relue
à `8e7ea39d` — seule la l. 1541 change — avec l'AC-iii6, l. 902 : « tout son historique, textes compris ») ;
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
6. **Après un retour au standard, le prompt suit de nouveau le texte des versions de Kesh** : le dialogue le dit, et
   montre d'abord le texte qui sera servi. Le libellé *Revenir au texte standard* te convient-il ? *Recommandation* :
   oui.

## Demandes à l'orchestrateur

- **D1** et **D1 étendu** (§ *Dev Notes*) : **acceptées**, inscrites à la fiche 32-10a (`47394982`, `e7a90572`), le
  complément de la validation P2 aussi (`20a02a86` : descriptions, `creeLe`, `decideLe`, `numeroEnVigueur`, la
  composition dans `services/`, le refus de D2 en 400). Reste à les recontrôler au code à l'étape 0 de la 32-10b, champ par champ.
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
  docs/manual/fr/admin-manual.tex` dans les deux worktrees). **Inscrite** à la fiche 32-10a (`d6cceafd`) — et le
  compte de `main` est lui-même déjà faux de deux (fait 8). *(Validation P2, R12 : la dernière puce de la note de la
  32-10a, l. 1541 à `20a02a86`, disait « les trois modules de la 32-10a-iii », que sa table du découpage compte cinq ;
  corrigée par l'orchestrateur à `8e7ea39d` : « les modules déjà comptés de la 32-10a-iii (routes de gestion,
  `services/`), sans en ajouter un » — validation P3, R9, F9.)*

## À recontrôler à l'étape 0 du développement

- **La 32-10a fusionnée, et son code réel relu** — c'est le point principal :
  - `crates/kesh-api/src/routes/mcp_prompts.rs` : les sept chemins, méthodes, corps attendus (noms des champs : `version`,
    `nature`, `description`, `texte` ?), DTO de réponse de chaque route, codes et `code` d'erreur (409 : lesquels ?
    `OPTIMISTIC_LOCK_CONFLICT`, `ILLEGAL_STATE_TRANSITION` ou neufs ?), statuts (201, 200) ; lignes citées au Change Log.
  - **D1, D1 étendu et son complément faits, champ par champ** — la liste d'arrêt, c'est-à-dire les champs que
    l'écran **consomme** (validation P3, R1) : par version, `description` et `texte` **stockés**, pour l'éditeur et
    *Reprendre cette version* (promis par l'AC-iii6 de la 32-10a, l. 902 à `8e7ea39d` : « le prompt et **tout** son
    historique, textes compris ») ; `servi.texteVisible` et `servi.descriptionVisible` (rendus visibles, marque
    comprise, sans champ `marque` séparé), pour l'affichage ; `version` du prompt (le verrou de l'AC-7) et, par version, `numero`, `origine`, `etat`, `nature` (note « D1, champs du détail », fiche 32-10a, `8e7ea39d`, l. 1534-1541) ; `servable`, `creeLe`, `decideLe`, `auteurNom`,
    `decideurNom`, la clé auteur (numéro, nom, révocation) ; par prompt, `numeroEnVigueur`, `standardDiffere`,
    `modifieLe`, et `standardDuCode` (texte et description rendus visibles ; nul hors standard ou pour un standard que
    le binaire ne porte plus) ; **E2E seulement** : `servi.texte` et `servi.description` (exacts), que l'écran
    n'affiche pas et dont l'éditeur ne se pré-remplit jamais ; s'il en manque un : **arrêt**,
    compte rendu à l'orchestrateur (§ *Le découpage* : écart déclaré). La fonction de composition à signature
    primitive, **dans `services/`** (note `20a02a86`) ; le rendu visible fait par `rendre_visible` (fait 12) ; la description d'agent (`\n\n` après
    la marque) jugée par le contrôle de l'AC-ii7, et une version d'agent saine dite `servable`.
  - **Une version servable a-t-elle toujours `texteVisible === texte`** ? Oui si `valider_contenu` refuse tout ce que
    `rendre_visible` rend visible ; à vérifier sur `\r` (normalisé par `valider_contenu` **avant** le contrôle, AC-ii2,
    mais rendu `\u{000D}` par `rendre_visible`) : une version restaurée portant `\r\n` serait-elle servable avec un
    `\u{000D}` affiché ? La réponse (et ce que sert `prompts/get` dans ce cas) se cite au Change Log ; aucune phrase
    de l'écran n'en dépend plus (validation P3 : l'avis de l'AC-4 n'en dit rien, E17 est un angle mort).
  - **D2 fait** : le refus de l'approbation d'une version non servable, **400** attendu (AC-12).
  - La règle de l'AC-i6 et de l'AC-i7 telle que codée (dernière version, états terminaux) — l'écran la reflète, il ne
    la refait pas.
  - Les bornes de longueur (500, 16 000) et la normalisation (`\r\n`) de `valider_contenu`.
  - Les clés de messages du serveur ajoutées par la 32-10a-iii (préfixe ; collision avec `mcp-prompts-`) et
    `audit-log-*` des prompts.
  - Le texte de la marque, à son lieu réel (la fonction de composition de `services/`), pour l'E2E (a) et le manuel.
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

- **Amendement de l'AC-12 : la saisie survit au 409** (décision du Project Lead, 2026-10-27, KF-055 #796). Une
  proposition d'agent relève la `version` du prompt. L'enregistrement d'une modification en cours de l'administrateur
  peut donc échouer en 409 sans que l'administrateur ait rien fait de travers. L'AC-12 dit aujourd'hui « une saisie
  non enregistrée est perdue » : **cela ne vaut plus pour l'éditeur**.
  - Sur un 409 rendu à l'enregistrement d'une nouvelle version, *Recharger* relit le détail. La saisie de
    l'administrateur (description et texte) reste **disponible** : par exemple, elle est gardée dans l'éditeur et
    l'historique rechargé est affiché à côté, ou elle est proposée par un geste *Reprendre ma saisie*. Le choix est au
    développement, et se consigne.
  - **Aucun envoi automatique** : l'administrateur réenregistre lui-même, après avoir vu la version qui a fait
    échouer son enregistrement.
  - Un test le prouve (`saisie_conservee_apres_409`), avec une mutation qui vide la saisie au rechargement.
  - Les 409 des gestes sans saisie (approuver, refuser, retirer, revenir au standard) restent régis par l'AC-12 tel
    qu'il est écrit.
  - *Écrit par l'orchestrateur après la validation : cette note est relue à l'étape 0 et par la revue de code P1, qui
    la contrôle comme une remédiation.*

- **LOW de la validation P5 (à corriger à l'étape 0, ligne lue citée au Change Log)** :
  - J1 : la note « D1, champs du détail » est à la 32-10a `a157255b` (l. ≈1541), non à `8e7ea39d` ; l'inscrire aussi à la liste des commits d'amendements.
  - J2 : les sélecteurs de zone sont **ancrés** en début de valeur (`[data-testid^="servi-"]`), puisque `non-servi-` contient `servi-`.
  - J3 : l'avis de l'AC-4 : « au-dessus du seul bloc *Texte servi par Kesh* » fait foi (retirer « en tête ») ; un test exige son absence sur une version non servable, refusée, et dans l'éditeur.
  - J4 : la phrase de *Recharger* n'est écrite qu'une fois (AC-12), les autres y renvoient.
  - J5 : l'entrée P3 du Change Log reçoit un renvoi daté à l'annotation H8 de l'entrée P4 (sans réécriture).

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`claude-opus-5-5`), en autonomie, worktree `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-10b`
(partie de `f2ac34d2` = `main` à `006e8790` fusionné). Port 3339 ; bases `kesh_3210b`, `kesh_e2e_3210b`.

### Étape 0 — recontrôlée au code (lignes lues)

- **La 32-10a fusionnée** : `006e8790` (32-10a-iii), `dbe0da15` (i), `209e978f` (ii) dans `git log` de `f2ac34d2`.
- **Les sept routes** (`crates/kesh-api/src/lib.rs:389-409`, `routes/mcp_prompts.rs`) : `GET /prompts` →
  `Vec<PromptResumeResponse>` (l. 106-123 : `id, nom, standard, version, derniere, enVigueur, aValider,
  standardDiffere`) ; `GET /prompts/{id}` → `PromptDetailResponse` (l. 207-223 : `id, nom, standard, version,
  numeroEnVigueur, standardDiffere, standardDuCode, modifieLe, versions`), chaque version `VersionDetailResponse`
  (l. 164-191 : `numero, origine, etat, nature, description, texte` — `TexteRendu { exact, visible }` **enregistrés** —,
  `servable, descriptionServie, texteServi` — servis, **nuls si non servable** —, `auteurNom, decideurNom,
  cleAuteurId, cleNom, cleRevoquee, creeLe, decideLe`) ; `POST /prompts` `{nom, description, texte}` → 201 ;
  `POST /{id}/versions` `{version, nature: text|withdrawal, description?, texte?}` → 201 (l. 277-286, 605-624) ;
  `…/approve` et `…/refuse` `{version}` → 200 ; `…/standard` `{version}` → 201. Codes (l. 385-425) : 409
  `OPTIMISTIC_LOCK_CONFLICT` (verrou périmé), 409 `ILLEGAL_STATE_TRANSITION` (non la dernière, déjà décidée, non une
  proposition, standard inconnu, non standard), 404, 400 `VALIDATION_ERROR`. Types alignés : C-32-10b-22.
- **D1, D1 étendu, son complément, champs du détail : faits.** `texteServi`/`descriptionServie` composés par
  `composer_version` (l. 462-469, la fonction du catalogue, `services/prompts_mcp.rs:268`), rendus visibles par
  `rendre_visible` (`TexteRendu::de`, l. 151-158) ; `servable = servie.is_some()` (l. 479) ; `standardDuCode` (l. 511-521,
  nul hors standard ou standard absent du code, `contenu_attendu` → `Retrait`) ; `numeroEnVigueur` (l. 499-500). Une
  version non servable n'a pas de texte servi : l'écran montre l'enregistré rendu visible (C-32-10b-21) — aucun champ
  consommé ne manque, **pas d'arrêt**.
- **Le `\r`** : `valider_contenu` normalise `\r\n` avant le contrôle et rend le normalisé (`prompts_mcp.rs:121-123`,
  `157-172`) ; un `\r` isolé est refusé ; test `retour_chariot_controle_et_rendu_alignes` (l. 745-770) : une version
  servable n'affiche jamais `\u{000D}`, `rendre_visible(texte servi) == texte servi`.
- **D2 fait** : `revalider` (l. 674-703) → `erreur_de_version` → 400 (`AppError::Validation`, l. 371-379).
- **Bornes** : 500 et 16 000 (messages `mcp-prompt-description-too-long`, `-text-too-long`, l. 344-359).
- **Préfixe** : le serveur emploie `mcp-prompt-` au singulier (`grep -n '^mcp-prompts\?-' …/fr-CH/messages.ftl` →
  l. 1771-1788, toutes `mcp-prompt-`) : l'écran garde `mcp-prompts-` (`startsWith` ne les confond pas).
- **La marque** : `marque_agent` (`prompts_mcp.rs:179-184`), « [Prompt écrit par un agent — version N, proposée par la
  clé MCP n° K et approuvée par un administrateur de Kesh. Ce n'est pas un prompt standard de Kesh.] », suivie d'une
  ligne vide (l. 298-301).
- **Standards et amorçage** : `kesh-developpement`, `kesh-assistant` (`mcp_prompt_standards.rs:25`, `:53`) ;
  `amorcer_standards` (`repositories/mcp_prompts.rs:848-867`) reprend un standard d'origine `standard`, laisse un
  personnalisé ; `PLAFOND_A_VALIDER = 20` (`entities/mcp_prompt.rs:50`). `seedTestState('with-company')` amorce les
  standards : constaté par l'E2E (d) (`kesh-assistant` trouvé).
- **`prompts/get` et `prompts/list`** (`mcp/repartiteur.rs:221-285`, `mcp/prompts.rs:146-158`) : `{description,
  messages:[{role:"user", content:{type:"text", text}}]}` ; inconnu ou non servi → `-32602` à texte fixe ; journal
  `mcp.prompt_served` à chaque appel, fail closed (`services/serveur_mcp.rs:274-308`).
- **La 32-7 non fusionnée** : `grep -n nav-admin-mcp …/+layout.svelte` → la seule `nav-admin-mcp-sends` ; `grep -rn
  PREFIXES_REPLI frontend/src` → rien ; aucune `sec:mcp`. D'où : menu *Envois* puis *Prompts*, AC-15 (b), section
  sous *Sécurité*.
- **Recomptes** : `sitesTotal` 2394 à `f2ac34d2` (non 2393 : le correctif #777/#778) → **2510** ; `grep -rn '{@html'
  frontend/src/lib frontend/src/routes | wc -l` → **0** ; `NOMBRE_DE_CLES` des envois 48, inchangé ; le registre des
  routes : `audit_route_registry.rs:903-934` — **135** routes, **129** tracées, 4 exemptées, 2 sans objet (la
  32-10a-iii a ajouté cinq routes mutantes tracées) ; le manuel disait « 122 des 128 », faux de sept.
- **Amendement de l'AC-12 (la saisie survit au 409)** : relu ; tenu par C-32-10b-20, test `saisie_conservee_apres_409`,
  mutation M19 (tuée) ; les 409 des autres gestes gardent l'AC-12 (test « un 409 d'un geste sans saisie »).
- **LOW de la validation P5** : J1 inscrit aux *Dev Notes* (`a157255b`) ; J2 — tous les sélecteurs de zone sont ancrés
  (`[data-testid^="servi-"]`, test `bloc_servi_seulement_si_servable`) ; J3 — l'avis au-dessus du seul bloc servi,
  absent d'une version non servable, refusée, à valider et de l'éditeur (tests « l'avis est absent… »,
  `bloc_servi_seulement_si_servable`, « aucun avis du texte servi dans l'éditeur ») ; J4 — au manuel, *Recharger* n'est
  décrit qu'une fois (§ « Recharger après un conflit ») ; J5 — renvoi daté ajouté à l'entrée P3 du Change Log.

### Completion Notes List

- **Tests rouges d'abord** : `mcp-prompts.api.test.ts`, `mcp-prompts-format.test.ts`, `McpPromptVersion.test.ts`,
  `McpPromptEditor.test.ts`, `McpPromptDetail.test.ts` écrits avant leur code et constatés rouges (« Failed to resolve
  import … Does the file exist? », journaux `kesh-gate-logs/3210b-rouge-*.log`). ⚠️ **Écart** : `McpPromptsList.svelte`
  a été écrit **avant** son test (aucun rouge constaté pour la liste).
- **Tests neufs** (périmètre `f2ac34d2` → HEAD, recomptés par `grep -cE "^\s*it\("` et le cas paramétré) : **86 Vitest**
  dans 7 fichiers (`McpPromptDetail` 36 dont 5 paramétrés, `mcp-prompts-format` 12, `mcp-prompts.api` 10,
  `mcp-prompts-i18n` 9, `McpPromptsList` 8, `McpPromptVersion` 6, `McpPromptEditor` 5) ; **3 E2E**
  (`mcp-prompts.spec.ts`). La garde des envois (`mcp-sends-i18n.test.ts`) passe par le module partagé, ses 9 tests
  inchangés.
- **Une découverte de test** : la `value` d'une `<textarea>` normalise elle-même `\r\n` en `\n` (norme HTML) — le corps
  exact à `\r\n` est figé au niveau de la fabrique (`corps_texte_exact`), non de l'éditeur.
- **Mutations** (`kesh-gate-logs/3210b-mutations.sh`, motif unique vérifié, restauration `git checkout -- <f>` puis
  `touch`, diff résiduel nul vérifié ; journal `3210b-mutations-2.log`, campagne rejouée en entier après le
  renforcement de M1) : M1 à M17 et M19 **tuées** (dont M3b `trim()`, variante de M3) ; **M18 équivalente** (C-32-10b-25 :
  `agir` garde son propre retour anticipé) ; **M18b** (les deux gardes retirées) **tuée** par
  `gestes_desactives_pendant_l_envoi — Approuver`. **M1 survivait** à la première campagne : son assertion, levée dans
  la simulation de l'API, était avalée par le `catch` de l'écran ; le test relève désormais le nombre de `GET` au moment
  du `POST`, et les réponses « une fois » sont remises à zéro avant chaque test (commit `test(3210b)`) ; M1 est tuée par
  `approbation_porte_la_version_lue`.
- **Recette visuelle** (AC-17, thème clair, `kesh_3210b`) : `kesh-gate-logs/3210b-captures/` — 01 menu, 02 liste des
  deux standards, 03 liste aux états, 04 standard servi, 05 version d'agent à valider (marque en tête), 06 dialogue
  d'approbation, 07 dialogue de refus, 08 n'est plus la dernière, 09 historique après approbation, 10 version piégée
  (U+202E, U+200B, U+E0041 rendus `\u{…}`, U+00A0 non rendue), 11 éditeur et avis P26, 12 400 caractère invisible,
  13a 409 et *Recharger*, 13b saisie conservée, 14a dialogue de retrait, 14b prompt retiré, 15a dialogue du standard,
  15b après le retour, 16 *Nouveau prompt* et 400 sur un nom en `kesh`, 17 refus au Comptable. Script
  `3210b-captures.mjs` (C-32-10b-28). LOW constaté sur les captures : la pastille *En vigueur* hérite de la police à
  chasse fixe de la cellule du numéro (cosmétique).
- **Manuel** : `sec:mcp-prompts` après `sec:mcp-envois` ; « 129 des 135 », « 129 + 4 + 2 = 135 ». PDF régénéré
  (`make fr`, 2 passes, stable) et contrôlé aplati (« 9.7 Les prompts du serveur MCP », « … ôte la marque : la version
  devient la vôtre… », « Revenir au texte standard. Sur un prompt standard… », « — 129 des 135 routes à verbe
  mutant… », « … 129 + 4 + 2 = 135 »). Le `user-manual.pdf` régénéré sans changement de source n'est pas commité.
- **CHANGELOG** : entrée « L'écran des prompts du serveur MCP » à `Ajouté` de la 0.16.0.
- **AC-20** : aucun fichier Rust, aucune migration ; seules les quatre locales de `kesh-i18n` changent dans `crates/`.

### Gates (réellement exécutés, au dernier commit de code)

- Backend : `scripts/test-fast.sh --ci` sous verrou, bases remises à zéro avant (DROP/CREATE, migrations, seed ; sans
  redémarrer le conteneur) : **4432 exécutés, 4432 passés, 7 ignorés** (`3210b-gate-complet.log`).
- `cargo test -p kesh-i18n` : 41 passés.
- Frontend sous verrou : `npm run check` 0 erreur (29 avertissements, aucun dans les fichiers de la story) ;
  `lint-i18n-ownership` PASS ; `npm run test:unit` **174 fichiers, 2065 tests passés** ; `npm run build` vert.
- E2E ciblé (`mcp-prompts.spec.ts`) : 3/3 (`3210b-e2e-cible-1.log`).
- E2E complet, au dernier commit de code (`98b745ff`, recette `docs/testing.md`, port 3339, dossiers `/tmp/kesh-e2e-3210b-3/` passés au backend et au runner, sous verrou) : **344 tests — 320 passés, 7 échoués, 17 ignorés** (13,0 min, `3210b-e2e-complet.log`). Les sept échecs, jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus* (l. 394-400) : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` — les sept KF-029 (#97), aucun autre. `mcp-prompts.spec.ts` : 3/3.

### File List

- `frontend/src/lib/features/mcp-prompts/` (neuf) : `mcp-prompts.types.ts`, `mcp-prompts.api.ts`,
  `mcp-prompts-format.ts`, `McpPromptsList.svelte`, `McpPromptDetail.svelte`, `McpPromptVersion.svelte`,
  `McpPromptEditor.svelte` et les sept fichiers de test.
- `frontend/src/routes/(app)/admin/mcp/prompts/+page.svelte`, `…/[id]/+page.svelte` (neufs) ;
  `frontend/src/routes/(app)/+layout.svelte` (une entrée).
- `frontend/src/lib/shared/i18n-repli-catalogue.ts` (neuf) ; `frontend/src/lib/features/mcp-sends/mcp-sends-i18n.test.ts`
  (emploie le module) ; `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`, `FAMILLES_RESOLUES`).
- `frontend/tests/e2e/mcp-prompts.spec.ts` (neuf).
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` (91 clés `mcp-prompts-*` + `nav-admin-mcp-prompts`).
- `docs/manual/fr/admin-manual.{tex,pdf}`, `CHANGELOG.md`.
- `_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md`, `epic-32-choix-autonomes-3210b.md`,
  `32-10b-review-prompt-p1.md`, `sprint-status.yaml`.

## Change Log

| Date | Étape | Résumé |
|---|---|---|
| 2026-10-27 | création | Fiche rédigée en autonomie (Opus 5.5) contre `42680725` (branche `story/32-10b` = fiche 32-10a + `origin/main` `ed981c7d`) ; registre C-32-10b-1 à C-32-10b-16 ; **3 modules** (frontend, `kesh-i18n`, docs), aucun découpage ; **20 AC** (A : 4, B : 2, C : 8, D : 6), **9 tâches** (T0 à T8), **16 mutations** (M1 à M16) ; inventaire E1 à E15 ; 7 points pour Guy ; 4 demandes à l'orchestrateur (D1 : le texte servi calculé par le serveur, amendement de l'AC-iii6 de la 32-10a ; D2 : revalidation à l'approbation ; D3 : avis à la 32-7 ; D4 : issue de comparaison si la recette la demande). Prompt de validation P1 `3210b-validate-prompt-p1.md` (Sonnet, lentilles R et F). |
| 2026-10-27 | validation P1 | **Sonnet ×2** (lentilles R et F, contexte frais, prompt `3210b-validate-prompt-p1.md` ; rapports `kesh-gate-logs/3210b-validate-p1-{R,F}.md`) : R 0/2/5/9, F 0/1/3/6 ⇒ **trend P1 0/3/8/15** (doublons F1 = R2, F2 = R7, F6 ⊃ R6 + R15, F7 = R5, F9 = R9, comptés une fois par lentille). **Décision de l'orchestrateur : fermer les classes par la conception, pas par des gardes** — D1 **étendu** inscrit à la fiche 32-10a (`story/32-10a`, `e7a90572`, § *À recontrôler à l'étape 0*) : par version, `servable`, servi exact et **rendu visible par le serveur** (`rendre_visible`, 32-8b, fait 12), pour un standard le texte du code rendu visible, noms de l'auteur et du décideur, date de dernière modification. **L'écran n'a plus aucune logique Unicode** : retirés `caracteres-caches.ts` (`segmenter`, `reconstituer`), la copie des listes de `kesh-core` et sa garde, la garde de version d'Unicode du moteur, les pastilles et les comptes calculés dans le navigateur, l'aperçu et les compteurs de l'éditeur, la variante B de l'AC-5 (sans D1 étendu, arrêt et compte rendu). **HIGH** : R1 (description d'agent rendue en mode « ligne », `\n\n` signalé) → **tombe** avec la logique de l'écran ; la note de la 32-10a dit le contrôle de la description d'agent (AC-ii7) ; R2 = F1 (garde de version irréalisable dans le job Frontend, `ci.yml:218-222` relu : `node-version: 22`, aucune étape Rust) → **garde retirée**. ⚠️ Prémisses **non vérifiées et devenues sans objet** : la version d'Unicode de l'ICU de Node 22 (R2 : « peut être inférieure » ; F1 : « 15.1/16 ») — `node` interdit, aucune mesure ; écrit tel quel. **MEDIUM** : R3 (le drapeau `standardDiffere` vrai pour tout standard personnalisé, AC-i5 de la 32-10a l. 289-296 relue : confirmé) → *Personnalisé* et « Diffère du standard de cette version de Kesh » distingués, le drapeau seul conditionne *Revenir au texte standard*, C-32-10b-19 ; R4 → champs de D1 étendu, **contrainte des primitives** écrite (Dev Notes) ; R5 = F7 → E2E (d) « **deux** versions de plus », compte **relatif** `k → k + 2`, E2E (a) borné à la ligne du nom unique ; R6 (+ F6) → fixture de `refus_apres_confirmation` écrite (n° 3 à valider, n° 4 dernière, chemin `…/versions/3/refuse`) ; R7 = F2 → le dialogue de retour au standard montre `standardDuCode` **avant** le geste, test `dialogue_standard_montre_le_texte_du_code`, M12 réécrite, E16 ; F3 → verdict `servable` du serveur : avis de l'AC-6, *Approuver* absent sur une version non servable (C-32-10b-17, M4 réécrite), bloc « reçoit aujourd'hui » seulement si servable, **avis permanent de l'AC-4 rendu exact** pour une version à valider ; F4 (+ R8) → tests **au niveau du DOM** (`pre.textContent === servi.texteVisible`, `pre.contains(mark)`, pastille hors du `<pre>`), **règle des blancs de gabarit** écrite, M3 et M13 réécrites (M13 ne peut plus passer à vide : elle rompt aussi l'égalité de `textContent`). **LOW** : R8 (intégré à F4) ; R9 = F9 → angle mort écrit (AC-13, § *Ce que les gardes ne voient pas*) ; R10 → relevé confirmé (`audit_route_registry.rs:895` → 130, `:901` → 124) : « 122 des 128 » déjà faux de deux, fait 8 et AC-19 rectifiés, à recompter au développement ; R11 → avis `mcp-prompt-mark-missing`, la description suit la règle du texte ; R12 → `modifieLe` (D1 étendu) ; R13 → **tombe** (compteurs retirés, C-32-10b-8 annoté) ; R14 → fait 13, patron `backup-schedule.spec.ts:200-217` (relu : `POST /api/v1/users`, `role: 'Comptable'`, aucun helper ; `ls helpers/` cité) ; R15 → `version_a_valider_jamais_repliee` défini à l'AC-7, `marque_dans_le_bloc` exige l'intérieur du `<pre>` ; R16 → **en partie réfuté** : `admin-manual.tex:2935` est **juste sur la base de la fiche 32-10a** (`grep -n "routes à verbe mutant"` dans `kesh-32-10a` → l. 2935, « 108 des 114 ») ; sur `main` fusionné la phrase est l. 2978 — demande D5 à l'orchestrateur (lignes citées par la 32-10a : 1008, 1331, 1384, non 1383) ; F5 → **tombe** (plus de pastille ; l'imitation d'un `\u{…}` littéral est l'angle mort E17, distingué par `servable`) ; F6 → intégré (R6, R15) ; F7 → intégré (R5) ; F8 → règle d'envoi en cours sur chaque geste, test `un_seul_post_pendant_l_envoi`, M17, C-32-10b-18 ; F10 (a) → **tombe** (les espaces admises ne sont plus rendues : point 3 pour Guy reformulé) ; F10 (b) → dialogue « Une fois approuvé, ce texte sera servi… ». **Questions pour Guy** : 1 et 2 **tranchées par l'orchestrateur** (oui, oui) en attendant Guy ; 3 **reformulée** (le serveur rendant visible, accepter que les caractères admis ne soient pas montrés ?). **Registre** : C-32-10b-1, 3, 4, 8, 10 annotés ; C-32-10b-17 à 19 ajoutés. **Propagation** : greppés sur la fiche et le registre — `version_unicode`, `UNICODE_VERSION`, `is_hidden_in_line`, `trois versions`, `standard a changé`, `caracteres-caches`, `segmenter`, `reconstituer`, `LISTES_KESH_CORE`, `listes_egales`, `variante B`, `⟦`, `[data-cache]`, `process.versions`, `N / 16`, « comptes de l'AC-6 » ; ne restent que l'entrée de création du Change Log (non réécrite), les annotations datées du registre et les mentions de retrait. **Décomptes** (recomptés par `grep`) : 20 AC (A 4, B 2, C 8, D 6), 9 tâches (T0 à T8), **17 mutations** (M1 à M17 ; M2, M3, M4, M12, M13 réécrites), inventaire **E1 à E17**, 7 points pour Guy, **5 demandes** (D1 et D1 étendu, D2 acceptées ; D3, D4, D5), C-32-10b-1 à **19** ; **3 modules**, inchangés. **1 passe sur 8.** Prompt de la passe suivante : `3210b-validate-prompt-p2.md` (**Opus**, R et F, axe obligatoire des régressions de cette remédiation). |
| 2026-10-27 | validation P2 | **Opus ×2** (lentilles R et F, contexte frais, prompt `3210b-validate-prompt-p2.md` ; rapports `kesh-gate-logs/3210b-validate-p2-{R,F}.md`) : R 0/0/3/11, F 0/0/7/5 ⇒ **trend 0/3/8/15 → 0/0/10/16** (doublons R1 = F6, R3 = F1, R11 = F5, R13 = F3, R6 = F11, comptés une fois par lentille). ⚠️ **Recyclage déclaré — signal D5 pour le Project Lead** : la sévérité tombe (HIGH → 0), mais MEDIUM → MEDIUM, et **la plupart des findings naissent de la remédiation P1** — R : 2 MEDIUM sur 3 et 7 LOW sur 11 classés « régression P1 » ; F : 5 MEDIUM sur 7 et 1 LOW (F6 en partie). Ce n'est pas la revue qui trouve des défauts d'origine neufs : c'est le patch précédent qui en crée. Consigne de l'orchestrateur appliquée : **retirer plutôt qu'enrichir** — toute promesse que l'écran ne tient pas simplement est retirée. **Note de la 32-10a complétée** par l'orchestrateur (`20a02a86`, « D1 étendu, complément ») : texte **et** description servis, chacun exact et rendu visible, **la marque comprise dans le texte servi, sans champ séparé** ; de même pour le standard du code ; `creeLe`, `decideLe`, `numeroEnVigueur` ; composition dans `services/` ; refus de D2 en **400**. **MEDIUM** : R1 = F6 → l'écran ne consomme que les champs de la note complétée ; **encadré séparé de la marque retiré** (`servi.marque`, le `<span data-testid="mcp-prompt-mark">`, `marque_dans_le_bloc`, `marque_manquante_signalee` et l'avis `mcp-prompt-mark-missing` retirés) : la marque se lit dans le texte rendu visible, à sa place, sans traitement ; `<pre>` d'un tenant, interpolation seule ; `texte_affiche_egal_au_champ_visible` porte sur une version d'**agent** (origine fixée, marque comprise — F2) ; `pastille_agent_hors_du_bloc` ; **M13 réécrite** (le composant retire la marque → égalité de `textContent`) ; liste d'arrêt de l'étape 0 réécrite champ par champ ; types (`ServiPrompt` sans `marque`, `standardDuCode: ServiPrompt | null`) ; R2 → réglé par la note : composition dans `services/`, cité (AC-5, *Dev Notes*, E2E (a)) ; R3 = F1 → **avis de l'AC-4 réduit à ce qui est vrai** (le texte que Kesh sert ; les caractères de contrôle et de mise en page **qu'il refuse** écrits en code ; la phrase sur le `\u{…}` en toutes lettres ajoutée, sur laquelle E17 et l'AC-6 s'appuient), « chaque caractère invisible » retiré (avis, Story, conception § 2) ; `avis_distingue_servi_et_a_valider` vérifie le texte de l'avis ; F3 = R13 → modèle de la 32-8c (`McpSendDetail.svelte:47`, `:92`, `:309-368`) : test `gestes_desactives_pendant_l_envoi`, qui asserte `disabled` de **tous** les gestes de la page (F8) et `aria-busy` du bouton cliqué ; **M17 réécrite** (`disabled={envoiEnCours}` retiré d'*Approuver*) ; le retrait du retour anticipé du gestionnaire est **déclaré équivalent** et retiré des mutations (un bouton désactivé ne reçoit pas le clic) ; « la réponse remplace l'état », non testé, retiré ; F4 → la prétention que l'E2E vérifie le champ rendu visible est **retirée** (§ *Ce que les gardes ne voient pas* ; M2 ne cite plus l'E2E (a)) : le test DOM couvre l'affichage, l'E2E l'égalité du texte exact avec `prompts/get` ; l'E2E à caractère caché, qui passerait par une version restaurée, est écrit en **angle mort** ; F5 = R11 → bouton seulement si `standardDiffere && standardDuCode !== null`, cas impossible retiré (F9), **M10 réécrite** sur la clause neuve ; F7 → ligne **E18** en **angle mort** (le verrou `version` compte des écritures, il ne désigne pas un contenu ; même angle mort que #773, point 15, relu par `gh issue view 773`). **LOW** : R4 → D2 rend **400**, rangé sous le 400 à l'AC-12, tableau de la 32-10a, *Dev Notes*, étape 0, C-32-10b-17 annoté ; R5 → la pastille *En vigueur* suit `numeroEnVigueur`, repli et fonction pure retirés, test `en_vigueur_selon_le_serveur`, C-32-10b-7 annoté, esquisse ; R6 = F11 → `sprint-status.yaml` (en-tête et ligne de la story) ; R7 → la phrase sur le `\u{…}` littéral rendue **conditionnelle** au recontrôle du `\r` (AC-4, AC-6, AC-18, E17, § *Ce que les gardes ne voient pas*, étape 0) ; R8 → angle mort « la liste ne juge pas ; le détail le dit » ; R9 → témoin vert `audit-log-entity-mcp-prompts-x` (`mcp-sends-i18n.test.ts:136`) ; R10 → témoin rouge composé par une constante (`:52`, `:231`) ; R12 → `20a02a86` cité (en-tête, tableau, *Dev Notes*, demandes, *Ce qui est lu et cité* : l. 1534-1541), D5 **inscrite** (`d6cceafd`) ; la phrase « trois modules de la 32-10a-iii » de la note (l. 1541, cinq à sa table) signalée à l'orchestrateur ; R14 → E2E (b) sur un **nom unique neuf**, membre `error` égal (les `id` diffèrent) ; F8 → intégré à F3 ; F9 → intégré à F5 ; F10 → **question 6 retirée** (déjà tranchée par l'AC-iii7 de la 32-10a ; la 32-10b n'ajoute aucune route), l'ancienne 7 devient la 6 ; F12 → avis P26 au pré-remplissage **d'office** par une version d'agent en vigueur, test renommé `avis_p26_si_pre_rempli_par_une_version_d_agent` (M14 suit) ; AC-6 : le « motif exact si la réponse le rend » retiré (la note n'en promet pas) ; AC-7 : `creeLe`. **Registre** : C-32-10b-1, 2, 7, 10, 17, 18 annotés (validation P2) ; aucun choix neuf. **Propagation** : greppés sur la fiche, le registre et `sprint-status.yaml` — `marque_dans_le_bloc`, `marque_manquante`, `mark-missing`, `mcp-prompt-mark`, `servi.marque`, `chaque caractère invisible`, `409`, `e7a90572`, `pastilles`, `16 mutations`, `encadr`, `un_seul_post`, `en_vigueur_selon_la_regle`, `avis_p26_si_reprise`, `mcp/textes.rs`, `si la réponse le rend` ; ne restent que les entrées passées du Change Log (non réécrites), les annotations datées du registre, `e7a90572` cité avec `20a02a86` comme commit de D1 étendu, les 409 légitimes (verrou, dernière version, standard inconnu, patron de la 32-8c) et, dans `sprint-status.yaml`, les « 16 mutations » et « encadr » d'autres stories. **Décomptes** (recomptés par `grep`) : 20 AC (A 4, B 2, C 8, D 6), 9 tâches (T0 à T8), **17 mutations** (M1 à M17 ; M10, M13, M17 réécrites ; une équivalente déclarée hors table), inventaire **E1 à E18**, **6 points pour Guy**, 5 demandes, C-32-10b-1 à 19 ; **3 modules**, inchangés. **2 passes sur 8.** Prompt de la passe suivante : `3210b-validate-prompt-p3.md` (**Sonnet**, R et F, axe obligatoire des régressions de cette remédiation). |
| 2026-10-27 | validation P3 | **Sonnet ×2** (lentilles R et F, contexte frais, prompt `3210b-validate-prompt-p3.md` ; rapports `kesh-gate-logs/3210b-validate-p3-{R,F}.md`) : R 0/0/1/9, F 0/0/1/9 ⇒ **trend 0/0/10/16 → 0/0/2/18** (doublons F7 = R2, F9 = R9, F4 = R5, F3 ⊂ R3 + R4 + R8, comptés une fois par lentille). ⚠️ **Recyclage déclaré — signal D5 pour le Project Lead** : MEDIUM → MEDIUM, et **les deux MEDIUM naissent des remédiations** (R1 de la P2, F1 de la P1) ; sur les vingt findings, douze sont classés « régression P2 » (R : 7 ; F : 5), trois « régression P1 » (R7, F1, F2), cinq « antérieurs » (R6, R10, F5, F6, F10). La revue ne trouve plus de défaut de la conception d'origine : elle relit ce que les patches ont écrit. Consigne de l'orchestrateur appliquée : **retirer ou simplifier plutôt qu'enrichir**. **MEDIUM** : F1 → **une seule règle des `data-testid`**, écrite une fois à l'AC-5 : tout `data-testid` porté par une version est préfixé par sa zone (`aujourdhui-`, `a-valider-`, `historique-`) ; appliquée à l'AC-5, à l'AC-6 et aux E2E (a) (`a-valider-…`) et (c) (`aujourdhui-…`) ; R1 → la **liste d'arrêt** de l'étape 0 nomme ce que l'écran consomme : `description` et `texte` **stockés** (éditeur, *Reprendre cette version* ; promis par l'AC-iii6 de la 32-10a, « tout son historique, textes compris », l. 902 à `8e7ea39d`, relue), les champs **rendus visibles** (affichage) ; `servi.texte` et `servi.description` exacts marqués **E2E seulement** ; « l'éditeur ne se pré-remplit jamais avec le texte servi » écrit à l'AC-5, à l'AC-9, aux types et à la liste d'arrêt. **LOW** : R3, R4, F3, R8 → l'avis de l'AC-4 réduit à **une phrase vraie quel que soit le cas** (« Le texte est montré tel que Kesh le servirait ; certains caractères invisibles y sont écrits sous la forme `\u{…}`. »), sans énumération ; accolades par paramètre Fluent comme `mcp-sends-notice` (`McpSendDetail.svelte:190-194`) ; test renommé `avis_permanent_exact`, attendu **écrit en dur** ; la phrase « un `\u{…}` est écrit en toutes lettres » retirée de l'avis, de l'AC-6 et de l'AC-18 ; **E17 devient angle mort** (le manuel le dit) ; le recontrôle du `\r` à l'étape 0 reste, sans phrase qui en dépende ; « contrôle et mise en page » retiré (Story, conception § 2) ; R5 = F4 → la mutation du retour anticipé **n'est pas équivalente** (Svelte appelle le gestionnaire délégué même sur un bouton désactivé, constat des lentilles par lecture des sources, non exécuté) : **M18** ajoutée, tuée par deux `click` envoyés par `fireEvent` et un seul `POST` attendu dans `gestes_desactives_pendant_l_envoi` ; le mot « équivalente » retiré de la fiche ; `aria-busy` : précédent `admin-backup/`, non la 32-8c ; F5 → après toute relecture réussie, l'éditeur est **réinitialisé** sur la version en vigueur, comme `poser()` (`McpSendDetail.svelte:56`), test `editeur_reinitialise_apres_relecture` ; F2 → parenthèse de l'AC-7 retirée ; F6 → le bloc s'appelle **Texte servi par Kesh** (test `bloc_servi_seulement_si_servable`, AC-18, esquisse) ; F7 = R2 → registre **annoté** (C-32-10b-2, 7, 10, 17, 18, 19), sans réécriture ; la déclaration de l'entrée P2 « ne restent que … les annotations datées du registre » était fausse à sa date pour ces titres et « Retenu » — annotée ici, non réécrite ; F8 → l'imitation de la marque, clause de la puce « homoglyphes » ; F9 = R9 → `8e7ea39d` cité (en-tête, *Ce qui est lu et cité*, liste d'arrêt), parenthèse de D5 corrigée (la puce est corrigée à `8e7ea39d`) ; F10 → les trois réglages du dialogue de la 32-8c (`McpSendDetail.svelte:345-347`) pendant l'envoi ; R6 → « contenu vide » ajouté à l'avis de l'AC-6 (les quatre cas de `valider_contenu`, AC-ii2, l. 635-640, relues) ; R7 → angle mort « l'éditeur pré-rempli depuis une version non servable » ; R10 → **statut `drafted`** (fiche et `sprint-status.yaml`) jusqu'à la clôture de la boucle — l'orchestrateur le passera à `ready-for-dev`. **Écart de la lentille R** (une copie de la fiche 32-10a écrite par redirection dans son propre scratchpad, déclarée par elle) : **sans effet sur la fiche** ni sur le dépôt. **Propagation** : greppés sur la fiche et le registre — `équivalente`, `aujourd'hui`, `-{numero}`, `servi.texte`, `contrôle et mise en page`, `de contrôle et de mise en page`, `avis_distingue`, `bloc_du_jour`, `toutes lettres`, `M17`, `17 mutations`, `reçoit aujourd`, `dernière phrase` ; ne restent que les entrées passées du Change Log (non réécrites), les annotations datées du registre, les trois `data-testid` à `<zone>` de l'AC-5 et de l'AC-6, les `servi.texte` des tests (AC-5), de l'E2E (a), de M2 et de la liste d'arrêt « E2E seulement », les « toutes lettres » de l'angle mort (AC-18, E17, § *Ce que les gardes ne voient pas*). **Décomptes** (recomptés par `grep`) : 20 AC (A 4, B 2, C 8, D 6), 9 tâches (T0 à T8), **18 mutations** (M1 à M18 ; M18 ajoutée, plus aucune équivalente déclarée), inventaire E1 à E18 (E17 réécrite en angle mort), 6 points pour Guy, 5 demandes, C-32-10b-1 à 19 (six annotés en P3, aucun neuf) ; **3 modules**, inchangés. **3 passes sur 8.** Prompt de la passe suivante : `3210b-validate-prompt-p4.md` (passe **ciblée**, lentille *regression hunter*, **Opus**, sur le seul commit de cette remédiation). *(Renvoi daté du 2026-10-27, étape 0 du développement — LOW J5 : voir l’annotation H8 de l’entrée P4 ; cette entrée n’est pas réécrite.)* |
| 2026-10-27 | validation P4 | **Opus** (passe **ciblée**, lentille *regression hunter*, contexte frais, prompt `3210b-validate-prompt-p4.md` ; rapport `kesh-gate-logs/3210b-validate-p4.md`, sur le commit `0dc8c9d2`) : **trend 0/0/2/18 → 0/0/1/7** (1 MEDIUM antérieur, 7 LOW dont 7 régressions P3). **MEDIUM** : H1 → la liste d'arrêt de l'étape 0 nomme aussi la `version` du prompt (verrou de l'AC-7) et, par version, `numero`, `origine`, `etat`, `nature`, citant la note « D1, champs du détail » de la fiche 32-10a (`8e7ea39d`, l. 1534-1541). **LOW** : H2 → zones `servi-` (bloc *Texte servi par Kesh*) et `non-servi-` (texte non servi), le test juge par le préfixe, non par le titre ; le préfixe `aujourdhui-` disparaît ; H3 → après un 409, *Recharger* remet l'éditeur sur la version en vigueur et la saisie est perdue (AC-12, AC-18 pour le manuel), comportement inchangé ; H4 → *Actualiser* retiré de la liste des relectures de l'éditeur (il n'existe que sur la liste, AC-3) ; H5 → l'avis de l'AC-4 ne s'affiche qu'au-dessus du bloc servi, ni sur une version non servable, ni sur une version refusée, ni dans l'éditeur ; H6 → C-32-10b-8 annoté ; H7 → statut remis à `ready-for-dev` (fiche et `sprint-status.yaml`), « validation en cours » en commentaire (`drafted` n'est pas défini) ; H8 → annotation datée de l'entrée P3, non réécrite : *R5 et F4 sont deux constats distincts* ; R5 = Svelte appelle le gestionnaire délégué d'un bouton désactivé (M18) ; F4 = soumission implicite par Entrée dans un `<input>` d'un `<form>`, **couverte par le maintien du retour anticipé** du gestionnaire (sans effet en production), et non par M18. **Propagation** : greppés sur la fiche, le registre et `sprint-status.yaml` — `aujourdhui`, `Actualiser`, `drafted` ; ne restent que l'*Actualiser* de la liste (AC-3), les entrées passées du Change Log et les mots de la présente entrée. **Décomptes** (recomptés par `grep`, inchangés) : 20 AC, 9 tâches, 18 mutations (M1 à M18), inventaire E1 à E18, 6 points pour Guy, 5 demandes, C-32-10b-1 à 19 (C-32-10b-8 annoté) ; **3 modules**. **4 passes sur 8.** Prompt de la passe suivante : `3210b-validate-prompt-p5.md` (passe **ciblée**, *regression hunter*, **Sonnet**, sur le seul commit de cette remédiation). |
| 2026-10-27 | **validation P5 — VALIDATION TERMINÉE** | **Sonnet**, passe ciblée *regression hunter* sur `c89a2b7c` (prompt `3210b-validate-prompt-p5.md` ; rapport `kesh-gate-logs/3210b-validate-p5.md`) : **0/0/0/5**. **Trend** : P1 (Sonnet ×2) 0/3/8/15 → P2 (Opus ×2) 0/0/10/16 → P3 (Sonnet ×2) 0/0/2/18 → P4 (Opus, ciblée) 0/0/1/7 → **P5 (Sonnet, ciblée) 0/0/0/5**. **Recyclage déclaré** en P2, P3 et P5 (signal D5), résolu par simplification de la conception (le serveur rend le texte servi, rendu visible, et le verdict `servable` ; l'écran n'a aucune logique Unicode). **Clôture** sur une passe sans finding au-dessus de LOW, **5 passes sur 8**. Les cinq LOW vont au § *À recontrôler à l'étape 0*. Développement **après la fusion de la 32-10a**, dont la fiche porte les amendements D1, D1 étendu, D2 et D5 à son étape 0 (`a157255b`). *(Écrit par l'orchestrateur.)* |
| 2026-10-27 | développement | **Opus 5.5**, autonomie, worktree `kesh-32-10a`, branche `story/32-10b` (partie de `f2ac34d2`). Étape 0 recontrôlée au code, lignes citées au *Dev Agent Record* : D1, D1 étendu, son complément et D2 **faits** ; DTO réels alignés (C-32-10b-22) ; une version non servable n'a pas de texte servi, l'écran montre l'enregistré rendu visible (C-32-10b-21, pas d'arrêt) ; 32-7 non fusionnée (AC-15 (b), section sous *Sécurité*) ; amendement de l'AC-12 tenu (C-32-10b-20) ; LOW J1 à J5 appliqués. Commits `3ff3ef30` (écran, i18n, garde), `06e5fb68` (manuel, compte des routes 129 des 135, CHANGELOG, E2E), `98b745ff` (test de M1 renforcé). **86 Vitest neufs**, 3 E2E. **Mutations** : M1 à M17, M3b, M18b, M19 tuées ; M18 équivalente (C-32-10b-25) ; M1 survivait à la première campagne (assertion avalée par le `catch`), test renforcé, campagne rejouée. **Gates au dernier commit de code** : backend 4432/4432 (7 ignorés) ; front check 0 erreur, lint PASS, 2065/2065, build vert ; E2E 320 passés, 7 KF-029, 17 ignorés. Captures 01 à 17 (`kesh-gate-logs/3210b-captures/`). Choix C-32-10b-20 à 28. Prompt de revue P1 : `32-10b-review-prompt-p1.md` (Sonnet, B, E, A). |
| 2026-10-27 | revue de code P1 | **Sonnet ×3** (lentilles B, E, A, contexte frais, prompt `32-10b-review-prompt-p1.md` ; rapports `kesh-gate-logs/3210b-review-p1-{B,E,A}.md`) : B 0/0/1/6, E 0/0/3/6, A 0/0/0/4 ⇒ **consolidé 0/0/3/≈10**. **Convergences** : B1 = E1 = A1 (relecture en échec après une écriture réussie, annoncée comme l'échec du geste), E2 = B2 = A2 (un *Recharger* en échec détruit l'écran et la saisie conservée), B3 = E8 (gestes actifs pendant un rechargement), B5 = E6 (dialogues pendant l'envoi non testés), B4 = E4 = A3 (saisie écrasée par un geste réussi ou *Reprendre cette version*). **Reclassements par convergence** (orchestrateur) : A1 et B2 étaient LOW, relevés en MEDIUM. **MEDIUM remédiés** (`bf810366`), chacun test rouge d'abord (`kesh-gate-logs/3210b-p1-rouge-*.log`) puis mutation : écriture et relecture en deux temps (C-32-10b-29, test `relecture_en_echec_apres_ecriture`, M20 et M20b tuées) ; rechargement en échec sans perte, *Réessayer* au premier chargement (C-32-10b-30, test `saisie_conservee_si_le_rechargement_echoue`, M21 et M21b tuées) ; erreur de l'éditeur près de lui, défilée et focalisée (C-32-10b-31, test `erreur_de_l_editeur_pres_de_l_editeur`, M22 et M22b tuées, capture 12 refaite). **LOW pris** : E7 (assertion effective), B5 = E6 (test `dialogue_ne_se_ferme_pas_pendant_l_envoi` avec témoin hors envoi ; M24, M24a, M24b tuées), B3 = E8 (`occupe`, C-32-10b-32, M23 tuée), B7 (C-32-10b-33, M25 tuée), E9 (`idDeLUrl`, C-32-10b-34, M26 tuée), la pastille *En vigueur* (C-32-10b-35). **LOW non pris, à ouvrir en issue** : B4 = E4 = A3, B6, E5, A4 ; et le défaut voisin de E9 dans la page des envois (`routes/(app)/admin/mcp/sends/[id]/+page.svelte:11`, `Number(page.params.id)`, 32-8c). **Manuel** : trois phrases au paragraphe « Recharger après un conflit », PDF régénéré et contrôlé aplati. **i18n** : trois clés (`mcp-prompts-error-reread`, `-retry`, `-created-open`) dans les quatre locales, `NOMBRE_DE_CLES` 91 → 94, `sitesTotal` 2510 → 2513 (généalogie dans `i18n-keys.test.ts`). **Tests neufs de la remédiation** (périmètre `fd6741c6` → HEAD) : 8 Vitest (détail 32 → 38, liste 8 → 9, format 12 → 13). Une erreur non gérée constatée au gate frontend complet (le test B7 ne répondait pas à la relecture de la liste) corrigée au test seul (`61d41aed`), M25 rejouée et tuée. **Grep du symptôme** : `if (envoiEnCours` (restent l'éditeur, qui reçoit `occupe`, et la liste), `detail = null` (reste le seul `id` illisible), `Number(page.params` (reste la page des envois, hors story), `await goto` (le seul site, corrigé). **Gates au dernier commit de code** (`61d41aed`), bases remises à zéro : backend 4432/4432 (7 ignorés) ; front check 0 erreur, lint PASS, **2073/2073**, build vert ; **E2E 320 passés, 7 échecs = les sept KF-029** (`mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`), 17 ignorés (`3210b-p1-e2e-complet.log`). Captures refaites (01 à 17). Choix C-32-10b-29 à 35. **1 passe de revue sur 8.** Prompt de la passe suivante : `32-10b-review-prompt-p2.md` (passe **ciblée**, **Opus**, chasse aux régressions sur le seul diff de la remédiation, rapport `kesh-gate-logs/3210b-review-p2.md`). |
